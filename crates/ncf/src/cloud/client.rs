use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use super::vault::Stored;
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: String,
}

impl Device {
    pub fn load_or_create(folder: &Path) -> Device {
        let file = folder.join("device-id");
        let id = std::fs::read_to_string(&file).ok().map(|s| s.trim().to_string()).filter(|s| is_id(s)).unwrap_or_else(|| {
            let id = new_id();
            let _ = crate::platform::write_atomic(&file, id.as_bytes());
            id
        });
        Device { id, name: default_name(crate::OS), platform: platform_name(crate::OS).to_string() }
    }
}

pub fn platform_name(os: crate::Os) -> &'static str {
    match os {
        crate::Os::Mac => "macos",
        crate::Os::Windows => "windows",
        _ => "linux",
    }
}

pub fn default_name(os: crate::Os) -> String {
    let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
    let kind = match os {
        crate::Os::Mac => "Mac",
        crate::Os::Windows => "PC",
        _ => "computer",
    };
    let user = user.trim();
    if user.is_empty() {
        format!("This {kind}")
    } else {
        let mut c = user.chars();
        let first: String = c.next().map(|f| f.to_uppercase().collect::<String>()).unwrap_or_default();
        format!("{first}{}'s {kind}", c.as_str())
    }
}

struct Inner {
    cfg: CloudConfig,
    http: Box<dyn Transport>,
    session: Mutex<Option<Session>>,
    refreshing: Mutex<()>,
    vault: Option<Vault>,
    skew: AtomicI64,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn parse<T: DeserializeOwned>(v: Value) -> Result<T, CloudError> {
    serde_json::from_value(v).map_err(|e| CloudError::Server(format!("unexpected reply: {e}")))
}

impl Client {
    pub fn new(cfg: CloudConfig, http: impl Transport + 'static) -> Client {
        Client::build(cfg, Box::new(http), None)
    }

    pub fn with_vault(cfg: CloudConfig, http: impl Transport + 'static, vault: Vault) -> Client {
        let c = Client::build(cfg, Box::new(http), Some(vault));
        c.restore();
        c
    }

    pub fn for_app(cfg: CloudConfig, info: &crate::AppInfo) -> Client {
        let folder = crate::platform::app_folder(info.name);
        let ua = format!("{}/{} NativeCloud", info.name, info.version);
        Client::with_vault(cfg, Ureq::new(&ua), Vault::system(&folder, info.id))
    }

    fn build(cfg: CloudConfig, http: Box<dyn Transport>, vault: Option<Vault>) -> Client {
        Client { inner: Arc::new(Inner { cfg, http, session: Mutex::new(None), refreshing: Mutex::new(()), vault, skew: AtomicI64::new(0) }) }
    }

    pub fn config(&self) -> &CloudConfig {
        &self.inner.cfg
    }

    pub fn restore(&self) -> bool {
        let Some(stored) = self.inner.vault.as_ref().and_then(Vault::load) else {
            return false;
        };
        *lock(&self.inner.session) =
            Some(Session { access_token: String::new(), refresh_token: stored.refresh_token, expires_at: 0, user_id: stored.user_id, email: stored.email });
        true
    }

    pub fn session(&self) -> Option<Session> {
        lock(&self.inner.session).clone()
    }

    pub fn signed_in(&self) -> bool {
        lock(&self.inner.session).is_some()
    }

    pub fn user_id(&self) -> String {
        self.session().map(|s| s.user_id).unwrap_or_default()
    }

    pub fn email(&self) -> String {
        self.session().map(|s| s.email).unwrap_or_default()
    }

    pub fn server_now(&self) -> i64 {
        unix_now() + self.inner.skew.load(Ordering::Relaxed)
    }

    pub fn set_session(&self, s: Option<Session>) {
        if let Some(v) = &self.inner.vault {
            match &s {
                Some(s) => v.save(&Stored { refresh_token: s.refresh_token.clone(), user_id: s.user_id.clone(), email: s.email.clone() }),
                None => v.clear(),
            }
        }
        *lock(&self.inner.session) = s;
    }

    fn send(&self, req: &Request) -> Result<Reply, CloudError> {
        let reply = self.inner.http.send(req).map_err(|_| CloudError::Offline)?;
        if let Some(t) = reply.date.as_deref().and_then(parse_http_date) {
            self.inner.skew.store(t - unix_now(), Ordering::Relaxed);
        }
        Ok(reply)
    }

    fn base(&self, method: &'static str, path: &str) -> Request {
        Request::new(method, self.inner.cfg.endpoint(path)).header("apikey", &self.inner.cfg.anon_key)
    }

    pub(super) fn auth_post(&self, path: &str, body: &Value) -> Result<Value, CloudError> {
        let reply = self.send(&self.base("POST", path).json(body))?;
        if reply.ok() {
            Ok(reply.json())
        } else {
            Err(auth_error(&reply))
        }
    }

    pub(super) fn auth_user(&self, body: &Value) -> Result<Value, CloudError> {
        let reply = self.authed_send("PUT", "/auth/v1/user", Some(body))?;
        if reply.ok() {
            Ok(reply.json())
        } else {
            Err(auth_error(&reply))
        }
    }

    pub fn refresh(&self) -> Result<(), CloudError> {
        let before = self.session().ok_or(CloudError::SignedOut)?;
        let _guard = lock(&self.inner.refreshing);
        let now = self.session().ok_or(CloudError::SignedOut)?;
        if now.access_token != before.access_token && !now.access_token.is_empty() {
            return Ok(());
        }
        match self.auth_post("/auth/v1/token?grant_type=refresh_token", &json!({"refresh_token": now.refresh_token})) {
            Ok(v) => {
                let mut s = Session::from_value(&v, self.server_now()).ok_or_else(|| CloudError::Server("no session in the reply".into()))?;
                if s.user_id.is_empty() {
                    s.user_id = now.user_id;
                }
                if s.email.is_empty() {
                    s.email = now.email;
                }
                self.set_session(Some(s));
                Ok(())
            }
            Err(e @ (CloudError::Offline | CloudError::Rate | CloudError::Server(_))) => Err(e),
            Err(_) => {
                self.set_session(None);
                Err(CloudError::SignedOut)
            }
        }
    }

    fn token(&self) -> Result<String, CloudError> {
        let s = self.session().ok_or(CloudError::SignedOut)?;
        if s.access_token.is_empty() || s.expires_at - self.server_now() < 60 {
            self.refresh()?;
            return self.session().map(|s| s.access_token).ok_or(CloudError::SignedOut);
        }
        Ok(s.access_token)
    }

    pub(super) fn authed_send(&self, method: &'static str, path: &str, body: Option<&Value>) -> Result<Reply, CloudError> {
        let mut retried = false;
        loop {
            let token = self.token()?;
            let mut req = self.base(method, path).header("authorization", &format!("Bearer {token}"));
            if let Some(b) = body {
                req = req.json(b);
            }
            let reply = self.send(&req)?;
            if reply.status == 401 && !retried {
                retried = true;
                if let Some(mut s) = self.session() {
                    s.expires_at = 0;
                    *lock(&self.inner.session) = Some(s);
                }
                continue;
            }
            if reply.status == 401 {
                self.set_session(None);
                return Err(CloudError::SignedOut);
            }
            return Ok(reply);
        }
    }

    fn data(&self, method: &'static str, path: &str, body: Option<&Value>) -> Result<Value, CloudError> {
        let reply = self.authed_send(method, path, body)?;
        if reply.ok() {
            Ok(reply.json())
        } else {
            let e = data_error(&reply);
            if e == CloudError::SignedOut {
                self.set_session(None);
            }
            Err(e)
        }
    }

    pub fn rpc(&self, name: &str, args: Value) -> Result<Value, CloudError> {
        self.data("POST", &format!("/rest/v1/rpc/{name}"), Some(&args))
    }

    pub fn get(&self, table_query: &str) -> Result<Value, CloudError> {
        self.data("GET", &format!("/rest/v1/{table_query}"), None)
    }

    pub fn function(&self, name: &str, body: &Value) -> Result<Value, CloudError> {
        self.data("POST", &format!("/functions/v1/{name}"), Some(body))
    }

    fn rows<T: DeserializeOwned>(&self, table_query: &str) -> Result<Vec<T>, CloudError> {
        parse(self.get(table_query)?)
    }

    fn call<T: DeserializeOwned>(&self, name: &str, args: Value) -> Result<T, CloudError> {
        parse(self.rpc(name, args)?)
    }

    fn id_reply(v: Value) -> Result<String, CloudError> {
        v.as_str().map(str::to_string).ok_or_else(|| CloudError::Server("expected an id".into()))
    }

    // you and your devices here---

    pub fn me(&self) -> Result<Me, CloudError> {
        self.call("me", json!({}))
    }

    pub fn usage(&self) -> Result<Summary, CloudError> {
        self.call("usage_summary", json!({}))
    }

    pub fn update_profile(&self, name: &str, hue: Option<i32>) -> Result<(), CloudError> {
        self.rpc("update_profile", json!({"display_name": name.trim(), "avatar_hue": hue})).map(|_| ())
    }

    pub fn register_device(&self, d: &Device) -> Result<(), CloudError> {
        self.rpc("register_device", json!({"device_id": d.id, "name": d.name, "platform": d.platform})).map(|_| ())
    }

    pub fn forget_device(&self, id: &str) -> Result<(), CloudError> {
        self.rpc("forget_device", json!({"device_id": id})).map(|_| ())
    }

    pub fn devices(&self) -> Result<Vec<DeviceRow>, CloudError> {
        self.rows("devices?select=id,name,platform,last_seen&order=last_seen.desc")
    }

    pub fn put_app_settings(&self, app: &str, body: &str) -> Result<(), CloudError> {
        self.rpc("put_app_settings", json!({"app_id": app, "body": body})).map(|_| ())
    }

    pub fn app_settings(&self, app: &str) -> Result<Option<Settings>, CloudError> {
        let mut rows: Vec<Settings> = self.rows(&format!("app_settings?select=body,updated_at&app_id=eq.{}", enc(app)))?;
        Ok(rows.pop())
    }

    // teams here---

    pub fn workspaces(&self) -> Result<Vec<Workspace>, CloudError> {
        self.rows("workspaces?select=id,name,kind,owner_id&order=created_at")
    }

    pub fn memberships(&self) -> Result<Vec<Member>, CloudError> {
        self.rows("memberships?select=workspace_id,user_id,role,joined_at&order=joined_at")
    }

    pub fn profiles(&self) -> Result<Vec<Profile>, CloudError> {
        self.rows("profiles?select=id,display_name,avatar_hue")
    }

    pub fn teams(&self) -> Result<Vec<Team>, CloudError> {
        let me = self.user_id();
        let spaces = self.workspaces()?;
        let members = self.memberships()?;
        let people = self.profiles()?;
        let mut teams = Vec::new();
        for w in spaces {
            let mine: Vec<(Member, Person)> = members
                .iter()
                .filter(|m| m.workspace_id == w.id)
                .map(|m| {
                    let p = people.iter().find(|p| p.id == m.user_id).map(Profile::person).unwrap_or(Person { id: m.user_id.clone(), ..Default::default() });
                    (m.clone(), p)
                })
                .collect();
            let my_role = mine.iter().find(|(m, _)| m.user_id == me).map(|(m, _)| m.role).unwrap_or_default();
            let invites = if my_role.can_manage() && !w.is_personal() { self.team_invites(&w.id).unwrap_or_default() } else { Vec::new() };
            teams.push(Team { workspace: w, members: mine, invites, my_role });
        }
        Ok(teams)
    }

    pub fn create_workspace(&self, name: &str) -> Result<String, CloudError> {
        Client::id_reply(self.rpc("create_workspace", json!({"name": name.trim()}))?)
    }

    pub fn rename_workspace(&self, ws: &str, name: &str) -> Result<(), CloudError> {
        self.rpc("rename_workspace", json!({"ws": ws, "name": name.trim()})).map(|_| ())
    }

    pub fn delete_workspace(&self, ws: &str) -> Result<(), CloudError> {
        self.rpc("delete_workspace", json!({"ws": ws})).map(|_| ())
    }

    pub fn invite_member(&self, ws: &str, email: &str, role: Role) -> Result<String, CloudError> {
        Client::id_reply(self.rpc("invite_member", json!({"ws": ws, "email": email.trim().to_lowercase(), "role": role.as_str()}))?)
    }

    pub fn send_invite(&self, invite: &str, app: &str) -> Result<bool, CloudError> {
        let v = self.function("send-invite", &json!({"invite_id": invite, "app": app}))?;
        Ok(v.get("sent").and_then(Value::as_bool).unwrap_or(false))
    }

    pub fn revoke_invite(&self, invite: &str) -> Result<(), CloudError> {
        self.rpc("revoke_invite", json!({"invite": invite})).map(|_| ())
    }

    pub fn my_invites(&self) -> Result<Vec<Invite>, CloudError> {
        self.call("my_invites", json!({}))
    }

    pub fn team_invites(&self, ws: &str) -> Result<Vec<Invite>, CloudError> {
        let rows: Vec<Invite> = self.rows(&format!(
            "invites?select=id,email,role,expires_at,accepted_at,workspace_id&workspace_id=eq.{}&accepted_at=is.null&declined_at=is.null&revoked_at=is.null&order=created_at",
            enc(ws)
        ))?;
        let now = self.server_now();
        Ok(rows.into_iter().filter(|i| parse_time(&i.expires_at).map(|t| t > now).unwrap_or(true)).collect())
    }

    pub fn accept_invite(&self, invite: &str) -> Result<String, CloudError> {
        Client::id_reply(self.rpc("accept_invite", json!({"invite": invite}))?)
    }

    pub fn decline_invite(&self, invite: &str) -> Result<(), CloudError> {
        self.rpc("decline_invite", json!({"invite": invite})).map(|_| ())
    }

    pub fn set_role(&self, ws: &str, member: &str, role: Role) -> Result<(), CloudError> {
        self.rpc("set_role", json!({"ws": ws, "member": member, "role": role.as_str()})).map(|_| ())
    }

    pub fn remove_member(&self, ws: &str, member: &str) -> Result<(), CloudError> {
        self.rpc("remove_member", json!({"ws": ws, "member": member})).map(|_| ())
    }

    pub fn leave_workspace(&self, ws: &str) -> Result<(), CloudError> {
        self.rpc("leave_workspace", json!({"ws": ws})).map(|_| ())
    }

    pub fn transfer_ownership(&self, ws: &str, member: &str) -> Result<(), CloudError> {
        self.rpc("transfer_ownership", json!({"ws": ws, "member": member})).map(|_| ())
    }

    // projects here---

    pub fn projects(&self, app: &str) -> Result<Vec<Project>, CloudError> {
        self.rows(&format!(
            "projects?select=id,workspace_id,app_id,title,meta,version,size,updated_by,created_by,updated_at,deleted_at&app_id=eq.{}&order=updated_at.desc",
            enc(app)
        ))
    }

    pub fn project(&self, id: &str) -> Result<Project, CloudError> {
        let mut rows: Vec<Project> = self.rows(&format!(
            "projects?select=id,workspace_id,app_id,title,meta,version,size,updated_by,created_by,updated_at,deleted_at,body&id=eq.{}",
            enc(id)
        ))?;
        rows.pop().ok_or(CloudError::NotFound)
    }

    pub fn export(&self) -> Result<Vec<Project>, CloudError> {
        self.rows("projects?select=id,workspace_id,app_id,title,meta,version,updated_at,deleted_at,body&order=updated_at.desc")
    }

    pub fn create_project(&self, ws: &str, app: &str, title: &str, body: &str, meta: &Value) -> Result<Project, CloudError> {
        self.call("create_project", json!({"ws": ws, "app_id": app, "title": title, "body": body, "meta": meta}))
    }

    pub fn open_project(&self, id: &str, device: &Device, want_edit: bool) -> Result<Opened, CloudError> {
        self.call("open_project", json!({"p": id, "device_id": device.id, "device_name": device.name, "want_edit": want_edit}))
    }

    pub fn request_edit(&self, id: &str, device: &Device) -> Result<Opened, CloudError> {
        self.call("request_edit", json!({"p": id, "device_id": device.id, "device_name": device.name}))
    }

    pub fn renew_lease(&self, id: &str, device: &Device) -> Result<Renewal, CloudError> {
        self.call("renew_lease", json!({"p": id, "device_id": device.id}))
    }

    pub fn release_lease(&self, id: &str, device: &Device) -> Result<(), CloudError> {
        self.rpc("release_lease", json!({"p": id, "device_id": device.id})).map(|_| ())
    }

    pub fn leases(&self) -> Result<Vec<(String, Lease)>, CloudError> {
        let rows = self.get("leases?select=project_id,user_id,device_id,device_name,expires_at")?;
        let me = self.user_id();
        let now = self.server_now();
        let people = self.profiles().unwrap_or_default();
        let mut out = Vec::new();
        for r in rows.as_array().cloned().unwrap_or_default() {
            let s = |k: &str| r.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            let expires_at = s("expires_at");
            if parse_time(&expires_at).map(|t| t <= now).unwrap_or(false) {
                continue;
            }
            let user = s("user_id");
            let person = people.iter().find(|p| p.id == user).map(Profile::person).unwrap_or(Person { id: user.clone(), ..Default::default() });
            out.push((s("project_id"), Lease { user: Some(person), device_id: s("device_id"), device_name: s("device_name"), expires_at, mine: user == me }));
        }
        Ok(out)
    }

    pub fn save_project(&self, id: &str, base_version: i64, body: &str, title: Option<&str>, meta: Option<&Value>, device: Option<&Device>) -> Result<Saved, CloudError> {
        let v = self.rpc(
            "save_project",
            json!({"p": id, "base_version": base_version, "body": body, "title": title, "meta": meta, "device_id": device.map(|d| d.id.clone())}),
        )?;
        Saved::from_value(&v).ok_or_else(|| CloudError::Server("unexpected save reply".into()))
    }

    pub fn take_snapshot(&self, id: &str, label: &str, kind: &str, body: Option<&str>) -> Result<String, CloudError> {
        Client::id_reply(self.rpc("take_snapshot", json!({"p": id, "label": label, "kind": kind, "body": body}))?)
    }

    pub fn snapshots(&self, id: &str) -> Result<Vec<Snapshot>, CloudError> {
        self.rows(&format!(
            "project_versions?select=id,project_id,version,size,label,kind,created_by,created_at&project_id=eq.{}&order=created_at.desc",
            enc(id)
        ))
    }

    pub fn snapshot_body(&self, snapshot: &str) -> Result<String, CloudError> {
        let mut rows: Vec<Snapshot> = self.rows(&format!("project_versions?select=id,body&id=eq.{}", enc(snapshot)))?;
        rows.pop().and_then(|s| s.body).ok_or(CloudError::NotFound)
    }

    pub fn delete_snapshot(&self, snapshot: &str) -> Result<(), CloudError> {
        self.rpc("delete_snapshot", json!({"snapshot": snapshot})).map(|_| ())
    }

    pub fn restore_snapshot(&self, snapshot: &str, base_version: i64, device: Option<&Device>) -> Result<Saved, CloudError> {
        let v = self.rpc("restore_snapshot", json!({"snapshot": snapshot, "base_version": base_version, "device_id": device.map(|d| d.id.clone())}))?;
        Saved::from_value(&v).ok_or_else(|| CloudError::Server("unexpected restore reply".into()))
    }

    pub fn edit_sessions(&self, id: &str) -> Result<Vec<EditSession>, CloudError> {
        self.rows(&format!("edit_sessions?project_id=eq.{}&order=ended_at.desc&limit=50", enc(id)))
    }

    pub fn delete_project(&self, id: &str) -> Result<(), CloudError> {
        self.rpc("delete_project", json!({"p": id})).map(|_| ())
    }

    pub fn restore_project(&self, id: &str) -> Result<(), CloudError> {
        self.rpc("restore_project", json!({"p": id})).map(|_| ())
    }

    pub fn purge_project(&self, id: &str) -> Result<(), CloudError> {
        self.rpc("purge_project", json!({"p": id})).map(|_| ())
    }

    pub fn move_project(&self, id: &str, to: &str) -> Result<(), CloudError> {
        self.rpc("move_project", json!({"p": id, "to_workspace": to})).map(|_| ())
    }

    pub fn duplicate_project(&self, id: &str, title: Option<&str>) -> Result<Project, CloudError> {
        self.call("duplicate_project", json!({"p": id, "title": title}))
    }

    pub fn set_star(&self, id: &str, starred: bool) -> Result<(), CloudError> {
        self.rpc("set_star", json!({"p": id, "starred": starred})).map(|_| ())
    }

    pub fn stars(&self) -> Result<Vec<String>, CloudError> {
        let v = self.get("stars?select=project_id")?;
        Ok(v.as_array().map(|a| a.iter().filter_map(|r| r.get("project_id").and_then(Value::as_str).map(str::to_string)).collect()).unwrap_or_default())
    }

    pub fn changes_since(&self, since: i64) -> Result<Feed, CloudError> {
        self.call("changes_since", json!({"since": since}))
    }
}

fn text(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

pub fn auth_error(reply: &Reply) -> CloudError {
    if reply.status == 429 {
        return CloudError::Rate;
    }
    if reply.status >= 500 {
        return CloudError::Server(format!("HTTP {}", reply.status));
    }
    let v = reply.json();
    let mut code = text(&v, "error_code");
    if code.is_empty() {
        code = text(&v, "error");
    }
    let message = [text(&v, "msg"), text(&v, "error_description"), text(&v, "message")].into_iter().find(|m| !m.is_empty()).unwrap_or_default();
    let lower = message.to_lowercase();
    if lower.contains("email not confirmed") {
        return CloudError::Unconfirmed;
    }
    if code == "invalid_grant" && lower.contains("invalid login") {
        return CloudError::Auth("invalid_credentials".into());
    }
    if code == "invalid_grant" && (lower.contains("refresh token") || lower.contains("already used")) {
        return CloudError::SignedOut;
    }
    if code.is_empty() && lower.contains("password") && reply.status == 422 {
        return CloudError::Auth("weak_password".into());
    }
    if code.is_empty() && (lower.contains("expired") || lower.contains("invalid")) && lower.contains("token") {
        return CloudError::Auth("otp_expired".into());
    }
    CloudError::from_auth(&code, &message)
}

pub fn data_error(reply: &Reply) -> CloudError {
    let v = reply.json();
    let message = [text(&v, "message"), text(&v, "msg"), text(&v, "error")].into_iter().find(|m| !m.is_empty()).unwrap_or_default();
    if message.starts_with("nc:") {
        return CloudError::from_code(&message);
    }
    if text(&v, "code") == "PGRST301" || reply.status == 401 {
        return CloudError::SignedOut;
    }
    if reply.status == 429 {
        return CloudError::Rate;
    }
    if !text(&v, "error_code").is_empty() || reply.status == 403 {
        let e = auth_error(reply);
        if e != CloudError::Auth(String::new()) {
            return e;
        }
    }
    CloudError::Server(if message.is_empty() { format!("HTTP {}", reply.status) } else { message })
}

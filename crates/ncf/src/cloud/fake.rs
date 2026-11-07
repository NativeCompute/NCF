use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{json, Value};

use super::time::format_time;
use super::*;

pub const LEASE_SECONDS: i64 = 90;
pub const WEB_URL: &str = "https://cloud.nativecloud.test";
const GRANT_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

#[derive(Clone, Debug)]
struct User {
    id: String,
    email: String,
    password: String,
    confirmed: bool,
    name: String,
    hue: i64,
    pending_email: Option<String>,
}

#[derive(Clone, Debug)]
struct Ws {
    id: String,
    name: String,
    kind: String,
    org: Option<String>,
    owner: String,
    at: i64,
    archived: bool,
}

#[derive(Clone, Debug)]
struct Org {
    id: String,
    name: String,
    slug: String,
    plan: String,
    at: i64,
    members_create_spaces: bool,
}

#[derive(Clone, Debug)]
struct OMem {
    org: String,
    user: String,
    role: String,
    at: i64,
}

#[derive(Clone, Debug)]
struct Grant {
    code: String,
    user: String,
    app: String,
    device: String,
    challenge: Option<String>,
    expires: i64,
    used: bool,
    attempts: u32,
}

#[derive(Clone, Debug)]
struct Mem {
    ws: String,
    user: String,
    role: String,
    at: i64,
}

#[derive(Clone, Debug)]
struct Inv {
    id: String,
    org: String,
    email: String,
    role: String,
    spaces: Vec<String>,
    space_role: String,
    by: String,
    expires: i64,
    open: bool,
}

#[derive(Clone, Debug)]
struct Proj {
    id: String,
    ws: String,
    app: String,
    title: String,
    body: String,
    meta: Value,
    version: i64,
    created_by: String,
    updated_by: String,
    updated_at: i64,
    deleted_at: Option<i64>,
}

#[derive(Clone, Debug)]
struct Ver {
    id: String,
    project: String,
    version: i64,
    body: String,
    label: String,
    kind: String,
    by: String,
    at: i64,
}

#[derive(Clone, Debug)]
struct Hold {
    project: String,
    user: String,
    device: String,
    device_name: String,
    expires: i64,
}

#[derive(Clone, Debug)]
struct Ch {
    seq: i64,
    ws: Option<String>,
    org: Option<String>,
    project: Option<String>,
    kind: String,
    by: String,
    at: i64,
}

#[derive(Clone, Debug)]
struct Dev {
    id: String,
    user: String,
    name: String,
    platform: String,
    seen: i64,
}

#[derive(Default)]
struct World {
    offline: bool,
    shift: i64,
    users: Vec<User>,
    codes: HashMap<(String, String), String>,
    access: HashMap<String, (String, i64)>,
    refresh: HashMap<String, String>,
    spaces: Vec<Ws>,
    members: Vec<Mem>,
    orgs: Vec<Org>,
    org_members: Vec<OMem>,
    grants: Vec<Grant>,
    invites: Vec<Inv>,
    projects: Vec<Proj>,
    versions: Vec<Ver>,
    leases: Vec<Hold>,
    changes: Vec<Ch>,
    devices: Vec<Dev>,
    stars: Vec<(String, String)>,
    settings: HashMap<(String, String), (String, i64)>,
    log: Vec<String>,
    fail: Vec<(String, u16, String)>,
    counter: u64,
    max_projects: i64,
    max_orgs: i64,
    max_people: i64,
    token_life: i64,
}

type Out = Result<Value, String>;

fn nc(code: &str) -> String {
    format!("nc:{code}")
}

fn s(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(x)) => x.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn http_date(t: i64) -> String {
    let days = t.div_euclid(86400);
    let wd = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][days.rem_euclid(7) as usize];
    let iso = format_time(t);
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let m: usize = iso[5..7].parse().unwrap_or(1);
    format!("{wd}, {} {} {} {} GMT", &iso[8..10], months[m - 1], &iso[0..4], &iso[11..19])
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').chars().take(40).collect::<String>();
    if out.is_empty() {
        "org".into()
    } else {
        out
    }
}

impl World {
    fn now(&self) -> i64 {
        unix_now() + self.shift
    }

    fn id(&mut self) -> String {
        new_id()
    }

    fn token(&mut self) -> String {
        self.counter += 1;
        format!("t{}{}", self.counter, &new_id()[..8])
    }

    fn user(&self, id: &str) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    fn by_email(&self, email: &str) -> Option<&User> {
        let e = email.trim().to_lowercase();
        self.users.iter().find(|u| u.email == e)
    }

    fn person(&self, id: &str) -> Value {
        match self.user(id) {
            Some(u) => json!({"id": u.id, "name": u.name, "hue": u.hue}),
            None => Value::Null,
        }
    }

    fn role(&self, ws: &str, user: &str) -> Option<String> {
        self.members.iter().find(|m| m.ws == ws && m.user == user).map(|m| m.role.clone())
    }

    fn is_member(&self, ws: &str, user: &str) -> bool {
        self.role(ws, user).is_some()
    }

    fn can_write(&self, ws: &str, user: &str) -> bool {
        let live = self.spaces.iter().any(|w| w.id == ws && !w.archived);
        live && self.role(ws, user).map(|r| r != "viewer").unwrap_or(false)
    }

    fn org_role(&self, org: &str, user: &str) -> Option<String> {
        self.org_members.iter().find(|m| m.org == org && m.user == user).map(|m| m.role.clone())
    }

    fn manages(&self, org: &str, user: &str) -> bool {
        self.org_role(org, user).map(|r| r == "owner" || r == "admin").unwrap_or(false)
    }

    fn space_org(&self, ws: &str) -> Option<String> {
        self.spaces.iter().find(|w| w.id == ws).and_then(|w| w.org.clone())
    }

    fn sees_space(&self, ws: &Ws, user: &str) -> bool {
        self.is_member(&ws.id, user) || ws.org.as_deref().map(|o| self.manages(o, user)).unwrap_or(false)
    }

    fn need_manage(&self, org: &str, user: &str) -> Result<String, String> {
        match self.org_role(org, user) {
            None => Err(nc("not_found")),
            Some(r) if r == "owner" || r == "admin" => Ok(r),
            Some(_) => Err(nc("forbidden")),
        }
    }

    fn note(&mut self, ws: &str, project: Option<&str>, kind: &str, by: &str) {
        let seq = self.changes.last().map(|c| c.seq).unwrap_or(0) + 1;
        let at = self.now();
        let org = self.space_org(ws);
        self.changes.push(Ch { seq, ws: Some(ws.into()), org, project: project.map(str::to_string), kind: kind.into(), by: by.into(), at });
    }

    fn note_org(&mut self, org: &str, kind: &str, by: &str) {
        let seq = self.changes.last().map(|c| c.seq).unwrap_or(0) + 1;
        let at = self.now();
        self.changes.push(Ch { seq, ws: None, org: Some(org.into()), project: None, kind: kind.into(), by: by.into(), at });
    }

    fn add_space(&mut self, org: &str, name: &str) -> String {
        let id = self.id();
        let at = self.now();
        self.spaces.push(Ws { id: id.clone(), name: name.trim().into(), kind: "space".into(), org: Some(org.into()), owner: String::new(), at, archived: false });
        let people: Vec<String> = self.org_members.iter().filter(|m| m.org == org && m.role != "guest").map(|m| m.user.clone()).collect();
        for user in people {
            self.members.push(Mem { ws: id.clone(), user, role: "editor".into(), at });
        }
        id
    }

    fn drop_from_org(&mut self, org: &str, member: &str) {
        let spaces: Vec<String> = self.spaces.iter().filter(|w| w.org.as_deref() == Some(org)).map(|w| w.id.clone()).collect();
        let projects: Vec<String> = self.projects.iter().filter(|p| spaces.contains(&p.ws)).map(|p| p.id.clone()).collect();
        self.leases.retain(|l| !(l.user == member && projects.contains(&l.project)));
        self.members.retain(|m| !(m.user == member && spaces.contains(&m.ws)));
        self.org_members.retain(|m| !(m.org == org && m.user == member));
    }

    fn delete_org(&mut self, org: &str) {
        let spaces: Vec<String> = self.spaces.iter().filter(|w| w.org.as_deref() == Some(org)).map(|w| w.id.clone()).collect();
        let gone: Vec<String> = self.projects.iter().filter(|p| spaces.contains(&p.ws)).map(|p| p.id.clone()).collect();
        self.projects.retain(|p| !gone.contains(&p.id));
        self.versions.retain(|v| !gone.contains(&v.project));
        self.leases.retain(|l| !gone.contains(&l.project));
        self.members.retain(|m| !spaces.contains(&m.ws));
        self.spaces.retain(|w| w.org.as_deref() != Some(org));
        self.org_members.retain(|m| m.org != org);
        self.invites.retain(|i| i.org != org);
        self.orgs.retain(|o| o.id != org);
    }

    fn grant_code(&mut self) -> String {
        let seed = new_id();
        let d = ring::digest::digest(&ring::digest::SHA256, format!("{seed}{}", self.counter).as_bytes());
        self.counter += 1;
        d.as_ref().iter().take(8).map(|b| GRANT_ALPHABET[(*b as usize) % GRANT_ALPHABET.len()] as char).collect()
    }

    fn make_grant(&mut self, user: &str, app: &str, device: &str, challenge: Option<String>, life: i64) -> String {
        let code = self.grant_code();
        let expires = self.now() + life;
        self.grants.push(Grant { code: code.clone(), user: user.into(), app: app.into(), device: device.into(), challenge, expires, used: false, attempts: 0 });
        code
    }

    fn redeem(&mut self, body: &Value) -> (u16, Value) {
        let bad = |code: &str| (400, json!({"error": nc(code)}));
        let code: String = s(body, "code").chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase();
        if code.is_empty() || code.len() > 32 {
            return bad("invalid:code");
        }
        let verifier = body.get("verifier").and_then(Value::as_str).map(str::to_string);
        if let Some(v) = &verifier {
            if v.len() < 43 || v.len() > 128 || !v.chars().all(|c| c.is_ascii_alphanumeric() || "._~-".contains(c)) {
                return bad("invalid:verifier");
            }
        }
        let now = self.now();
        let Some(i) = self.grants.iter().position(|g| g.code == code && !g.used && g.expires > now && g.attempts < 5) else {
            return bad("not_found");
        };
        let challenge = verifier.as_deref().map(challenge_of);
        if self.grants[i].challenge != challenge {
            self.grants[i].attempts += 1;
            if self.grants[i].attempts >= 5 {
                self.grants[i].used = true;
            }
            return bad("forbidden");
        }
        if challenge.is_none() && self.grants[i].app != "web" {
            return bad("invalid:verifier");
        }
        self.grants[i].used = true;
        let g = self.grants[i].clone();
        if self.user(&g.user).is_none() {
            return bad("not_found");
        }
        let mut v = self.session(&g.user);
        v["app"] = json!(g.app);
        v["device_name"] = json!(g.device);
        (200, v)
    }

    fn bill(&self, ws: &str) -> String {
        match self.spaces.iter().find(|w| w.id == ws) {
            Some(w) => w.org.clone().unwrap_or_else(|| w.owner.clone()),
            None => String::new(),
        }
    }

    fn billed_projects(&self, bill: &str) -> i64 {
        let spaces: Vec<&str> = self.spaces.iter().filter(|w| w.org.as_deref() == Some(bill) || (w.org.is_none() && w.owner == bill)).map(|w| w.id.as_str()).collect();
        self.projects.iter().filter(|p| p.deleted_at.is_none() && spaces.contains(&p.ws.as_str())).count() as i64
    }

    fn room_for(&self, ws: &str) -> bool {
        self.billed_projects(&self.bill(ws)) < self.max_projects
    }

    fn free_plan(&self) -> Value {
        json!({"id": "free", "name": "Free", "scope": "personal", "max_people": 1, "max_spaces": 1, "max_projects": self.max_projects,
               "max_bytes": 104857600, "max_owned_orgs": self.max_orgs, "max_auto_snapshots": 30, "max_kept_snapshots": 20,
               "max_saves_per_minute": 30, "trial_days": 0, "price_cents": 0, "paid": false})
    }

    fn make_user(&mut self, email: &str, password: &str, name: &str, confirmed: bool) -> String {
        let id = self.id();
        let hue = (self.counter as i64 * 47 + 200) % 360;
        self.users.push(User { id: id.clone(), email: email.trim().to_lowercase(), password: password.into(), confirmed, name: name.into(), hue, pending_email: None });
        let ws = self.id();
        let at = self.now();
        self.spaces.push(Ws { id: ws.clone(), name: "My cloud".into(), kind: "personal".into(), org: None, owner: id.clone(), at, archived: false });
        self.members.push(Mem { ws, user: id.clone(), role: "owner".into(), at });
        id
    }

    fn code_for(&mut self, kind: &str, email: &str) -> String {
        self.counter += 1;
        let code = format!("{:06}", (self.counter * 7919 + 104_729) % 900_000 + 100_000);
        self.codes.insert((kind.into(), email.to_lowercase()), code.clone());
        code
    }

    fn session(&mut self, user: &str) -> Value {
        let (a, r) = (self.token(), self.token());
        let exp = self.now() + self.token_life;
        self.access.insert(a.clone(), (user.to_string(), exp));
        self.refresh.insert(r.clone(), user.to_string());
        let email = self.user(user).map(|u| u.email.clone()).unwrap_or_default();
        json!({"access_token": a, "refresh_token": r, "token_type": "bearer", "expires_in": self.token_life, "expires_at": exp, "user": {"id": user, "email": email}})
    }

    fn live_lease(&self, p: &str) -> Option<&Hold> {
        let now = self.now();
        self.leases.iter().find(|l| l.project == p && l.expires > now)
    }

    fn lease_json(&self, l: Option<&Hold>, me: &str) -> Value {
        match l {
            Some(l) => json!({"user": self.person(&l.user), "device_id": l.device, "device_name": l.device_name, "expires_at": format_time(l.expires), "mine": l.user == me}),
            None => Value::Null,
        }
    }

    fn take_lease(&mut self, p: &str, user: &str, device: &str, name: &str) {
        let expires = self.now() + LEASE_SECONDS;
        self.leases.retain(|l| l.project != p);
        self.leases.push(Hold { project: p.into(), user: user.into(), device: device.into(), device_name: name.into(), expires });
    }

    fn project_json(&self, p: &Proj, body: bool) -> Value {
        let mut v = json!({
            "id": p.id, "workspace_id": p.ws, "app_id": p.app, "title": p.title, "meta": p.meta, "version": p.version,
            "size": p.body.len(), "updated_at": format_time(p.updated_at), "updated_by": self.person(&p.updated_by),
            "created_by": self.person(&p.created_by), "deleted_at": p.deleted_at.map(format_time)
        });
        if body {
            v["body"] = json!(p.body);
        }
        v
    }

    fn load(&self, p: &str, me: &str) -> Result<Proj, String> {
        let row = self.projects.iter().find(|x| x.id == p).cloned().ok_or_else(|| nc("not_found"))?;
        if !self.is_member(&row.ws, me) {
            return Err(nc("not_found"));
        }
        Ok(row)
    }

    fn put_project(&mut self, p: Proj) {
        match self.projects.iter_mut().find(|x| x.id == p.id) {
            Some(x) => *x = p,
            None => self.projects.push(p),
        }
    }

    fn signed_in(&self, req: &Request) -> Result<String, (u16, Value)> {
        let token = req.get_header("authorization").and_then(|h| h.strip_prefix("Bearer ")).unwrap_or("");
        match self.access.get(token) {
            Some((user, exp)) if *exp > self.now() && self.user(user).is_some() => Ok(user.clone()),
            Some(_) => Err((401, json!({"code": "PGRST301", "message": "JWT expired"}))),
            None => Err((401, json!({"code": "PGRST301", "message": "JWT invalid"}))),
        }
    }

    fn reply(&mut self, req: &Request) -> (u16, Value) {
        let path = req.path().to_string();
        let (route, query) = path.split_once('?').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((path.clone(), String::new()));
        let body: Value = req.body.as_deref().and_then(|b| serde_json::from_str(b).ok()).unwrap_or(Value::Null);
        if let Some(rest) = route.strip_prefix("/auth/v1/") {
            return self.auth(rest, &query, &body, req);
        }
        if route == "/functions/v1/app-token" {
            return self.redeem(&body);
        }
        let me = match self.signed_in(req) {
            Ok(u) => u,
            Err(e) => return e,
        };
        let out = if let Some(name) = route.strip_prefix("/rest/v1/rpc/") {
            self.rpc(name, &body, &me)
        } else if let Some(table) = route.strip_prefix("/rest/v1/") {
            self.table(table, &query, &me)
        } else if let Some(name) = route.strip_prefix("/functions/v1/") {
            self.function(name, &body, &me, req)
        } else {
            Err("unknown route".into())
        };
        match out {
            Ok(v) => (200, v),
            Err(m) if m.starts_with("nc:") => (400, json!({"code": "P0001", "message": m})),
            Err(m) => (404, json!({"code": "PGRST202", "message": m})),
        }
    }

    // auth here---

    fn auth(&mut self, route: &str, query: &str, body: &Value, req: &Request) -> (u16, Value) {
        let err = |status: u16, code: &str, msg: &str| (status, json!({"code": status, "error_code": code, "msg": msg}));
        let email = s(body, "email").trim().to_lowercase();
        match route {
            "signup" => {
                let password = s(body, "password");
                if !password_ok(&password) {
                    return err(422, "weak_password", "Password is too weak");
                }
                if !email_ok(&email) {
                    return err(400, "email_address_invalid", "Email address is invalid");
                }
                let name = body.get("data").map(|d| s(d, "display_name")).unwrap_or_default();
                let id = match self.by_email(&email) {
                    Some(u) if u.confirmed => return (200, json!({"id": new_id(), "email": email})),
                    Some(u) => u.id.clone(),
                    None => self.make_user(&email, &password, &name, false),
                };
                self.code_for("signup", &email);
                (200, json!({"id": id, "email": email}))
            }
            "verify" => {
                let kind = s(body, "type");
                let token = s(body, "token");
                let lookup = if kind == "email_change" {
                    self.users.iter().find(|u| u.pending_email.as_deref() == Some(email.as_str())).map(|u| u.email.clone()).unwrap_or(email.clone())
                } else {
                    email.clone()
                };
                let key = (kind.clone(), email.clone());
                if self.codes.get(&key) != Some(&token) {
                    return err(403, "otp_expired", "Token has expired or is invalid");
                }
                self.codes.remove(&key);
                let Some(u) = self.users.iter_mut().find(|u| u.email == lookup) else {
                    return err(403, "otp_expired", "Token has expired or is invalid");
                };
                u.confirmed = true;
                if kind == "email_change" {
                    if let Some(new) = u.pending_email.take() {
                        u.email = new;
                    }
                }
                let id = u.id.clone();
                (200, self.session(&id))
            }
            "resend" => {
                let kind = s(body, "type");
                if let Some(u) = self.by_email(&email).cloned() {
                    if !u.confirmed || kind != "signup" {
                        self.code_for(&kind, &email);
                    }
                }
                (200, json!({}))
            }
            "recover" => {
                if self.by_email(&email).is_some() {
                    self.code_for("recovery", &email);
                }
                (200, json!({}))
            }
            "token" if query.contains("grant_type=password") => {
                let password = s(body, "password");
                match self.by_email(&email).cloned() {
                    Some(u) if u.password == password && u.confirmed => (200, self.session(&u.id)),
                    Some(u) if u.password == password => err(400, "email_not_confirmed", "Email not confirmed"),
                    _ => err(400, "invalid_credentials", "Invalid login credentials"),
                }
            }
            "token" if query.contains("grant_type=refresh_token") => {
                let r = s(body, "refresh_token");
                match self.refresh.remove(&r) {
                    Some(user) if self.user(&user).is_some() => (200, self.session(&user)),
                    _ => err(400, "refresh_token_not_found", "Invalid Refresh Token: Refresh Token Not Found"),
                }
            }
            "user" => {
                let me = match self.signed_in(req) {
                    Ok(u) => u,
                    Err(_) => return err(401, "bad_jwt", "invalid JWT"),
                };
                if let Some(p) = body.get("password").and_then(Value::as_str) {
                    if !password_ok(p) {
                        return err(422, "weak_password", "Password is too weak");
                    }
                    if self.user(&me).map(|u| u.password == p).unwrap_or(false) {
                        return err(422, "same_password", "New password should be different from the old password.");
                    }
                    if let Some(u) = self.users.iter_mut().find(|u| u.id == me) {
                        u.password = p.to_string();
                    }
                }
                if !email.is_empty() {
                    if self.by_email(&email).is_some() {
                        return err(422, "email_exists", "A user with this email address has already been registered");
                    }
                    if let Some(u) = self.users.iter_mut().find(|u| u.id == me) {
                        u.pending_email = Some(email.clone());
                    }
                    self.code_for("email_change", &email);
                }
                let u = self.user(&me).cloned().unwrap();
                (200, json!({"id": u.id, "email": u.email, "new_email": u.pending_email}))
            }
            "logout" => {
                let me = match self.signed_in(req) {
                    Ok(u) => u,
                    Err(_) => return err(401, "session_not_found", "Session not found"),
                };
                let token = req.get_header("authorization").and_then(|h| h.strip_prefix("Bearer ")).unwrap_or("").to_string();
                if query.contains("scope=global") {
                    self.access.retain(|_, (u, _)| *u != me);
                    self.refresh.retain(|_, u| *u != me);
                } else {
                    self.access.remove(&token);
                }
                (204, Value::Null)
            }
            _ => err(404, "not_found", "Not found"),
        }
    }

    // tables here---

    fn table(&self, table: &str, query: &str, me: &str) -> Out {
        let mut filters: Vec<(String, String)> = Vec::new();
        let mut select = String::new();
        let mut limit = usize::MAX;
        for part in query.split('&').filter(|p| !p.is_empty()) {
            let (k, v) = part.split_once('=').unwrap_or((part, ""));
            match k {
                "select" => select = v.to_string(),
                "order" => {}
                "limit" => limit = v.parse().unwrap_or(usize::MAX),
                _ => filters.push((k.to_string(), v.to_string())),
            }
        }
        let wants_body = select.split(',').any(|c| c == "body") || select.is_empty();
        let mine: Vec<String> = self.members.iter().filter(|m| m.user == me).map(|m| m.ws.clone()).collect();
        let my_email = self.user(me).map(|u| u.email.clone()).unwrap_or_default();
        let seen_spaces: Vec<String> = self.spaces.iter().filter(|w| self.sees_space(w, me)).map(|w| w.id.clone()).collect();
        let my_orgs: Vec<(String, String)> = self.org_members.iter().filter(|m| m.user == me).map(|m| (m.org.clone(), m.role.clone())).collect();
        let org_rows: Vec<&OMem> = self
            .org_members
            .iter()
            .filter(|m| m.user == me || my_orgs.iter().any(|(o, r)| *o == m.org && (r != "guest")))
            .collect();
        let rows: Vec<Value> = match table {
            "workspaces" => self
                .spaces
                .iter()
                .filter(|w| seen_spaces.contains(&w.id))
                .map(|w| {
                    json!({"id": w.id, "name": w.name, "kind": w.kind, "org_id": w.org, "owner_id": if w.owner.is_empty() { Value::Null } else { json!(w.owner) },
                           "archived_at": if w.archived { json!(format_time(w.at)) } else { Value::Null }, "created_at": format_time(w.at)})
                })
                .collect(),
            "memberships" => self
                .members
                .iter()
                .filter(|m| mine.contains(&m.ws) || (seen_spaces.contains(&m.ws) && self.space_org(&m.ws).map(|o| self.manages(&o, me)).unwrap_or(false)))
                .map(|m| json!({"workspace_id": m.ws, "user_id": m.user, "role": m.role, "joined_at": format_time(m.at)}))
                .collect(),
            "org_members" => org_rows.iter().map(|m| json!({"org_id": m.org, "user_id": m.user, "role": m.role, "joined_at": format_time(m.at)})).collect(),
            "profiles" => {
                let mut people: Vec<&String> = self.members.iter().filter(|m| mine.contains(&m.ws)).map(|m| &m.user).collect();
                people.extend(org_rows.iter().map(|m| &m.user));
                self.users.iter().filter(|u| u.id == me || people.contains(&&u.id)).map(|u| json!({"id": u.id, "display_name": u.name, "avatar_hue": u.hue})).collect()
            }
            "invites" => self
                .invites
                .iter()
                .filter(|i| i.open && (self.manages(&i.org, me) || i.email == my_email))
                .map(|i| {
                    json!({"id": i.id, "org_id": i.org, "email": i.email, "role": i.role, "spaces": i.spaces, "space_role": i.space_role,
                           "expires_at": format_time(i.expires), "accepted_at": null, "declined_at": null, "revoked_at": null})
                })
                .collect(),
            "projects" => self
                .projects
                .iter()
                .filter(|p| mine.contains(&p.ws))
                .map(|p| {
                    let mut v = json!({
                        "id": p.id, "workspace_id": p.ws, "app_id": p.app, "title": p.title, "meta": p.meta, "version": p.version,
                        "size": p.body.len(), "updated_by": p.updated_by, "created_by": p.created_by,
                        "updated_at": format_time(p.updated_at), "deleted_at": p.deleted_at.map(format_time)
                    });
                    if wants_body {
                        v["body"] = json!(p.body);
                    }
                    v
                })
                .collect(),
            "project_versions" => {
                let seen: Vec<&String> = self.projects.iter().filter(|p| mine.contains(&p.ws)).map(|p| &p.id).collect();
                self.versions
                    .iter()
                    .rev()
                    .filter(|v| seen.contains(&&v.project))
                    .map(|v| {
                        let mut r = json!({"id": v.id, "project_id": v.project, "version": v.version, "size": v.body.len(), "label": v.label, "kind": v.kind, "created_by": v.by, "created_at": format_time(v.at)});
                        if wants_body {
                            r["body"] = json!(v.body);
                        }
                        r
                    })
                    .collect()
            }
            "leases" => {
                let seen: Vec<&String> = self.projects.iter().filter(|p| mine.contains(&p.ws)).map(|p| &p.id).collect();
                self.leases
                    .iter()
                    .filter(|l| seen.contains(&&l.project))
                    .map(|l| json!({"project_id": l.project, "user_id": l.user, "device_id": l.device, "device_name": l.device_name, "expires_at": format_time(l.expires)}))
                    .collect()
            }
            "devices" => self
                .devices
                .iter()
                .filter(|d| d.user == me)
                .map(|d| json!({"id": d.id, "name": d.name, "platform": d.platform, "last_seen": format_time(d.seen)}))
                .collect(),
            "stars" => self.stars.iter().filter(|(u, _)| u == me).map(|(_, p)| json!({"project_id": p})).collect(),
            "app_settings" => self
                .settings
                .iter()
                .filter(|((u, _), _)| u == me)
                .map(|((_, app), (body, at))| json!({"app_id": app, "body": body, "updated_at": format_time(*at)}))
                .collect(),
            "edit_sessions" => Vec::new(),
            _ => return Err(format!("unknown table {table}")),
        };
        let kept: Vec<Value> = rows
            .into_iter()
            .filter(|r| {
                filters.iter().all(|(k, v)| match v.split_once('.') {
                    Some(("eq", want)) => s(r, k) == *want,
                    Some(("is", "null")) => r.get(k).map(Value::is_null).unwrap_or(true),
                    _ => true,
                })
            })
            .take(limit)
            .collect();
        Ok(Value::Array(kept))
    }

    // rpc here---

    fn rpc(&mut self, name: &str, a: &Value, me: &str) -> Out {
        let now = self.now();
        let p = s(a, "p");
        let ws = s(a, "ws");
        match name {
            "me" | "usage_summary" => {
                let u = self.user(me).cloned().ok_or_else(|| nc("auth"))?;
                let plan = self.free_plan();
                let personal: Vec<&String> = self.spaces.iter().filter(|w| w.owner == me && w.org.is_none()).map(|w| &w.id).collect();
                let bytes: usize = self.projects.iter().filter(|p| personal.contains(&&p.ws)).map(|p| p.body.len()).sum();
                let owned = self.org_members.iter().filter(|m| m.user == me && m.role == "owner").count();
                let usage = json!({"projects": self.billed_projects(me), "bytes": bytes, "owned_orgs": owned});
                if name == "usage_summary" {
                    return Ok(json!({"plan": plan, "usage": usage}));
                }
                let personal = self.spaces.iter().find(|w| w.owner == me && w.kind == "personal").map(|w| w.id.clone());
                let mut orgs: Vec<Value> = self
                    .org_members
                    .iter()
                    .filter(|m| m.user == me)
                    .filter_map(|m| self.orgs.iter().find(|o| o.id == m.org).map(|o| (o, m)))
                    .map(|(o, m)| {
                        json!({"id": o.id, "name": o.name, "slug": o.slug, "role": m.role, "plan": o.plan,
                               "trial_ends_at": format_time(o.at + 30 * 86400), "suspended": false})
                    })
                    .collect();
                orgs.sort_by_key(|o| s(o, "name").to_lowercase());
                Ok(json!({"id": u.id, "email": u.email, "display_name": u.name, "avatar_hue": u.hue, "plan": plan, "usage": usage,
                          "personal_workspace": personal, "staff": null, "orgs": orgs}))
            }
            "update_profile" => {
                let n = s(a, "display_name");
                if n.chars().count() > 80 {
                    return Err(nc("invalid:name"));
                }
                if let Some(u) = self.users.iter_mut().find(|u| u.id == me) {
                    u.name = n.trim().to_string();
                    if let Some(h) = a.get("avatar_hue").and_then(Value::as_i64) {
                        u.hue = h.rem_euclid(360);
                    }
                }
                Ok(Value::Null)
            }
            "register_device" => {
                let id = s(a, "device_id");
                self.devices.retain(|d| d.id != id);
                self.devices.push(Dev { id, user: me.into(), name: s(a, "name"), platform: s(a, "platform"), seen: now });
                Ok(Value::Null)
            }
            "forget_device" => {
                let id = s(a, "device_id");
                self.devices.retain(|d| !(d.id == id && d.user == me));
                Ok(Value::Null)
            }
            "put_app_settings" => {
                self.settings.insert((me.into(), s(a, "app_id")), (s(a, "body"), now));
                Ok(Value::Null)
            }
            "create_org" => {
                let n = s(a, "name").trim().to_string();
                if n.is_empty() || n.chars().count() > 80 {
                    return Err(nc("invalid:name"));
                }
                if self.org_members.iter().filter(|m| m.user == me && m.role == "owner").count() as i64 >= self.max_orgs {
                    return Err(nc("limit:orgs"));
                }
                let base = slugify(&n);
                let mut slug = base.clone();
                let mut k = 2;
                while self.orgs.iter().any(|o| o.slug == slug) {
                    slug = format!("{base}-{k}");
                    k += 1;
                }
                let id = self.id();
                self.orgs.push(Org { id: id.clone(), name: n, slug: slug.clone(), plan: "team".into(), at: now, members_create_spaces: true });
                self.org_members.push(OMem { org: id.clone(), user: me.into(), role: "owner".into(), at: now });
                let space = self.add_space(&id, "General");
                self.note_org(&id, "org", me);
                Ok(json!({"id": id, "slug": slug, "space": space}))
            }
            "create_space" => {
                let org = s(a, "org");
                let mine = self.org_role(&org, me).ok_or_else(|| nc("not_found"))?;
                let open = self.orgs.iter().any(|o| o.id == org && o.members_create_spaces);
                if !(mine == "owner" || mine == "admin" || (mine == "member" && open)) {
                    return Err(nc("forbidden"));
                }
                let n = s(a, "name").trim().to_string();
                if n.is_empty() || n.chars().count() > 80 {
                    return Err(nc("invalid:name"));
                }
                if self.spaces.iter().filter(|w| w.org.as_deref() == Some(org.as_str())).count() >= 5 {
                    return Err(nc("limit:spaces"));
                }
                let id = self.add_space(&org, &n);
                self.note_org(&org, "workspace", me);
                Ok(json!(id))
            }
            "invite_member" => {
                let org = s(a, "org");
                let mine = self.need_manage(&org, me)?;
                let role = s(a, "role");
                let space_role = a.get("space_role").and_then(Value::as_str).unwrap_or("editor").to_string();
                if !["admin", "member", "guest"].contains(&role.as_str()) || !["editor", "viewer"].contains(&space_role.as_str()) {
                    return Err(nc("invalid:role"));
                }
                if role == "admin" && mine != "owner" {
                    return Err(nc("forbidden"));
                }
                let email = s(a, "email").trim().to_lowercase();
                if !email_ok(&email) {
                    return Err(nc("invalid:email"));
                }
                if self.org_members.iter().any(|m| m.org == org && self.user(&m.user).map(|u| u.email == email).unwrap_or(false)) {
                    return Err(nc("invalid:member"));
                }
                let spaces: Vec<String> = a.get("spaces").and_then(Value::as_array).map(|l| l.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default();
                if spaces.iter().any(|w| self.space_org(w).as_deref() != Some(org.as_str())) || (role == "guest" && spaces.is_empty()) {
                    return Err(nc("invalid:spaces"));
                }
                let people = self.org_members.iter().filter(|m| m.org == org).count() as i64;
                let waiting = self.invites.iter().filter(|i| i.org == org && i.open && i.expires > now && i.email != email).count() as i64;
                if people + waiting >= self.max_people {
                    return Err(nc("limit:members"));
                }
                for i in self.invites.iter_mut().filter(|i| i.org == org && i.email == email) {
                    i.open = false;
                }
                let id = self.id();
                self.invites.push(Inv { id: id.clone(), org: org.clone(), email, role, spaces, space_role, by: me.into(), expires: now + 14 * 86400, open: true });
                self.note_org(&org, "members", me);
                Ok(json!(id))
            }
            "revoke_invite" => {
                let id = s(a, "invite");
                let i = self.invites.iter().find(|i| i.id == id).cloned().ok_or_else(|| nc("not_found"))?;
                if !self.manages(&i.org, me) {
                    return Err(nc("not_found"));
                }
                if let Some(i) = self.invites.iter_mut().find(|i| i.id == id) {
                    i.open = false;
                }
                self.note_org(&i.org, "members", me);
                Ok(Value::Null)
            }
            "my_invites" => {
                let email = self.user(me).map(|u| u.email.clone()).unwrap_or_default();
                let list: Vec<Value> = self
                    .invites
                    .iter()
                    .filter(|i| i.open && i.email == email && i.expires > now)
                    .map(|i| {
                        let o = self.orgs.iter().find(|o| o.id == i.org);
                        let by = self.user(&i.by).map(|u| u.name.clone()).unwrap_or_default();
                        json!({"id": i.id, "role": i.role, "expires_at": format_time(i.expires), "org_id": i.org,
                               "org": o.map(|o| o.name.clone()), "org_slug": o.map(|o| o.slug.clone()), "invited_by": by})
                    })
                    .collect();
                Ok(Value::Array(list))
            }
            "accept_invite" | "decline_invite" => {
                let id = s(a, "invite");
                let u = self.user(me).cloned().ok_or_else(|| nc("auth"))?;
                let i = self.invites.iter().find(|i| i.id == id && i.open && i.email == u.email && i.expires > now).cloned().ok_or_else(|| nc("not_found"))?;
                if name == "accept_invite" && !u.confirmed {
                    return Err(nc("unconfirmed"));
                }
                if let Some(x) = self.invites.iter_mut().find(|x| x.id == id) {
                    x.open = false;
                }
                if name == "decline_invite" {
                    return Ok(Value::Null);
                }
                if self.org_role(&i.org, me).is_none() {
                    self.org_members.push(OMem { org: i.org.clone(), user: me.into(), role: i.role.clone(), at: now });
                }
                let spaces: Vec<String> = self
                    .spaces
                    .iter()
                    .filter(|w| w.org.as_deref() == Some(i.org.as_str()) && (i.role != "guest" || i.spaces.contains(&w.id)))
                    .map(|w| w.id.clone())
                    .collect();
                let role = if i.role == "guest" { i.space_role.clone() } else { "editor".to_string() };
                for ws in spaces {
                    if !self.is_member(&ws, me) {
                        self.members.push(Mem { ws, user: me.into(), role: role.clone(), at: now });
                    }
                }
                self.note_org(&i.org, "members", me);
                Ok(json!(i.org))
            }
            "set_org_role" => {
                let org = s(a, "org");
                let mine = self.need_manage(&org, me)?;
                let member = s(a, "member");
                let role = s(a, "role");
                let theirs = self.org_role(&org, &member).ok_or_else(|| nc("not_found"))?;
                if !["admin", "member", "guest"].contains(&role.as_str()) || theirs == "owner" {
                    return Err(nc("invalid:role"));
                }
                if mine == "admin" && (theirs == "admin" || role == "admin") {
                    return Err(nc("forbidden"));
                }
                if let Some(m) = self.org_members.iter_mut().find(|m| m.org == org && m.user == member) {
                    m.role = role.clone();
                }
                if theirs == "guest" && role != "guest" {
                    let spaces: Vec<String> = self.spaces.iter().filter(|w| w.org.as_deref() == Some(org.as_str())).map(|w| w.id.clone()).collect();
                    for ws in spaces {
                        if !self.is_member(&ws, &member) {
                            self.members.push(Mem { ws, user: member.clone(), role: "editor".into(), at: now });
                        }
                    }
                }
                self.note_org(&org, "members", me);
                Ok(Value::Null)
            }
            "remove_member" => {
                let org = s(a, "org");
                let mine = self.need_manage(&org, me)?;
                let member = s(a, "member");
                let theirs = self.org_role(&org, &member).ok_or_else(|| nc("not_found"))?;
                if theirs == "owner" || (theirs == "admin" && mine != "owner") {
                    return Err(nc("forbidden"));
                }
                self.drop_from_org(&org, &member);
                self.note_org(&org, "members", me);
                Ok(Value::Null)
            }
            "leave_org" => {
                let org = s(a, "org");
                let mine = self.org_role(&org, me).ok_or_else(|| nc("not_found"))?;
                if mine == "owner" {
                    return Err(nc("invalid:owner"));
                }
                self.drop_from_org(&org, me);
                self.note_org(&org, "members", me);
                Ok(Value::Null)
            }
            "create_app_grant" => {
                let challenge = s(a, "challenge");
                if challenge.len() != 43 || !challenge.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                    return Err(nc("invalid:challenge"));
                }
                let app = s(a, "app");
                if app.is_empty() || app.len() > 40 {
                    return Err(nc("invalid:app"));
                }
                let device: String = s(a, "device").trim().chars().take(80).collect();
                Ok(json!(self.make_grant(me, &app, &device, Some(challenge), 120)))
            }
            "create_project" => {
                if !self.is_member(&ws, me) {
                    return Err(nc("not_found"));
                }
                if !self.can_write(&ws, me) {
                    return Err(nc("forbidden"));
                }
                if !self.room_for(&ws) {
                    return Err(nc("limit:projects"));
                }
                let id = self.id();
                let row = Proj {
                    id: id.clone(),
                    ws: ws.clone(),
                    app: s(a, "app_id"),
                    title: s(a, "title"),
                    body: s(a, "body"),
                    meta: a.get("meta").cloned().filter(|m| !m.is_null()).unwrap_or(json!({})),
                    version: 1,
                    created_by: me.into(),
                    updated_by: me.into(),
                    updated_at: now,
                    deleted_at: None,
                };
                self.projects.push(row.clone());
                self.note(&ws, Some(&id), "project", me);
                Ok(self.project_json(&row, false))
            }
            "open_project" | "request_edit" => {
                let row = self.load(&p, me)?;
                if row.deleted_at.is_some() {
                    return Err(nc("not_found"));
                }
                let device = s(a, "device_id");
                let want = name == "request_edit" || a.get("want_edit").and_then(Value::as_bool).unwrap_or(true);
                if name == "request_edit" && !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                let held = self.live_lease(&p).cloned();
                let mut access = "read";
                if want && self.can_write(&row.ws, me) {
                    let free = match &held {
                        None => true,
                        Some(h) if name == "request_edit" => h.user == me,
                        Some(h) => h.user == me && h.device == device,
                    };
                    if free {
                        self.take_lease(&p, me, &device, &s(a, "device_name"));
                        access = "edit";
                        if held.is_none() || name == "request_edit" {
                            self.note(&row.ws, Some(&p), "lease", me);
                        }
                    }
                }
                let lease = self.live_lease(&p).cloned();
                Ok(json!({"project": self.project_json(&row, true), "access": access, "can_write": self.can_write(&row.ws, me), "lease": self.lease_json(lease.as_ref(), me)}))
            }
            "renew_lease" => {
                let row = self.load(&p, me)?;
                let device = s(a, "device_id");
                let held = self.live_lease(&p).cloned();
                match held {
                    Some(h) if !(h.user == me && h.device == device) => Ok(json!({"held": false, "lease": self.lease_json(Some(&h), me), "version": row.version})),
                    h => {
                        if !self.can_write(&row.ws, me) || row.deleted_at.is_some() {
                            return Ok(json!({"held": false, "lease": null, "version": row.version}));
                        }
                        let n = h.map(|h| h.device_name).unwrap_or_default();
                        self.take_lease(&p, me, &device, &n);
                        Ok(json!({"held": true, "version": row.version}))
                    }
                }
            }
            "release_lease" => {
                let device = s(a, "device_id");
                let before = self.leases.len();
                self.leases.retain(|l| !(l.project == p && l.user == me && l.device == device));
                if self.leases.len() != before {
                    let ws = self.projects.iter().find(|x| x.id == p).map(|x| x.ws.clone()).unwrap_or_default();
                    self.note(&ws, Some(&p), "lease", me);
                }
                Ok(Value::Null)
            }
            "save_project" | "restore_snapshot" => {
                let restoring = name == "restore_snapshot";
                let (p, body) = if restoring {
                    let id = s(a, "snapshot");
                    let v = self.versions.iter().find(|v| v.id == id).cloned().ok_or_else(|| nc("not_found"))?;
                    (v.project.clone(), v.body)
                } else {
                    (p, s(a, "body"))
                };
                let mut row = self.load(&p, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                if row.deleted_at.is_some() {
                    return Err(nc("not_found"));
                }
                let device = s(a, "device_id");
                let held = self.live_lease(&p).cloned();
                if let Some(h) = &held {
                    if !(h.user == me && (device.is_empty() || h.device == device)) {
                        return Ok(json!({"status": "locked", "lease": self.lease_json(Some(h), me), "version": row.version}));
                    }
                }
                let base = a.get("base_version").and_then(Value::as_i64).unwrap_or(-1);
                if row.version != base {
                    return Ok(json!({"status": "conflict", "project": self.project_json(&row, true)}));
                }
                if restoring {
                    let vid = self.id();
                    self.versions.push(Ver { id: vid, project: p.clone(), version: row.version, body: row.body.clone(), label: "Before restoring".into(), kind: "restore".into(), by: me.into(), at: now });
                }
                row.body = body;
                if !restoring {
                    if let Some(t) = a.get("title").and_then(Value::as_str) {
                        row.title = t.chars().take(200).collect();
                    }
                    if let Some(m) = a.get("meta").filter(|m| !m.is_null()) {
                        row.meta = m.clone();
                    }
                }
                row.version += 1;
                row.updated_by = me.into();
                row.updated_at = now;
                let (ws, version) = (row.ws.clone(), row.version);
                self.put_project(row);
                if !device.is_empty() {
                    let n = held.map(|h| h.device_name).unwrap_or_default();
                    self.take_lease(&p, me, &device, &n);
                }
                self.note(&ws, Some(&p), "project", me);
                Ok(json!({"status": "saved", "version": version, "updated_at": format_time(now)}))
            }
            "take_snapshot" => {
                let row = self.load(&p, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                let kind = if a.get("kind").map(Value::is_null).unwrap_or(true) { "manual".to_string() } else { s(a, "kind") };
                if !["manual", "conflict", "offline", "revision", "restore"].contains(&kind.as_str()) {
                    return Err(nc("invalid:kind"));
                }
                if (kind == "manual" || kind == "revision") && self.versions.iter().filter(|v| v.project == p && (v.kind == "manual" || v.kind == "revision")).count() >= 20 {
                    return Err(nc("limit:snapshots"));
                }
                let body = a.get("body").and_then(Value::as_str).map(str::to_string).unwrap_or(row.body.clone());
                let id = self.id();
                self.versions.push(Ver { id: id.clone(), project: p.clone(), version: row.version, body, label: s(a, "label"), kind, by: me.into(), at: now });
                self.note(&row.ws, Some(&p), "versions", me);
                Ok(json!(id))
            }
            "delete_snapshot" => {
                let id = s(a, "snapshot");
                let v = self.versions.iter().find(|v| v.id == id).cloned().ok_or_else(|| nc("not_found"))?;
                let row = self.load(&v.project, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                self.versions.retain(|v| v.id != id);
                self.note(&row.ws, Some(&row.id), "versions", me);
                Ok(Value::Null)
            }
            "delete_project" | "restore_project" => {
                let mut row = self.load(&p, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                if name == "restore_project" && !self.room_for(&row.ws) {
                    return Err(nc("limit:projects"));
                }
                row.deleted_at = if name == "delete_project" { Some(now) } else { None };
                let ws = row.ws.clone();
                self.put_project(row);
                self.leases.retain(|l| l.project != p);
                self.note(&ws, Some(&p), "project", me);
                Ok(Value::Null)
            }
            "purge_project" => {
                let row = self.load(&p, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                if row.deleted_at.is_none() {
                    return Err(nc("invalid:not_deleted"));
                }
                self.projects.retain(|x| x.id != p);
                self.versions.retain(|v| v.project != p);
                self.note(&row.ws, Some(&p), "project", me);
                Ok(Value::Null)
            }
            "move_project" => {
                let mut row = self.load(&p, me)?;
                let to = s(a, "to_workspace");
                if !self.is_member(&to, me) {
                    return Err(nc("not_found"));
                }
                if !self.can_write(&row.ws, me) || !self.can_write(&to, me) {
                    return Err(nc("forbidden"));
                }
                if self.bill(&to) != self.bill(&row.ws) && !self.room_for(&to) {
                    return Err(nc("limit:projects"));
                }
                let from = row.ws.clone();
                row.ws = to.clone();
                row.version += 1;
                row.updated_at = now;
                self.put_project(row);
                self.leases.retain(|l| l.project != p);
                self.note(&from, Some(&p), "project", me);
                self.note(&to, Some(&p), "project", me);
                Ok(Value::Null)
            }
            "duplicate_project" => {
                let row = self.load(&p, me)?;
                if !self.can_write(&row.ws, me) {
                    return Err(nc("forbidden"));
                }
                if !self.room_for(&row.ws) {
                    return Err(nc("limit:projects"));
                }
                let id = self.id();
                let title = a.get("title").and_then(Value::as_str).map(str::to_string).unwrap_or(format!("{} copy", row.title));
                let copy = Proj { id: id.clone(), title, version: 1, created_by: me.into(), updated_by: me.into(), updated_at: now, deleted_at: None, ..row };
                self.projects.push(copy.clone());
                self.note(&copy.ws, Some(&id), "project", me);
                Ok(self.project_json(&copy, false))
            }
            "set_star" => {
                self.load(&p, me)?;
                self.stars.retain(|(u, x)| !(u == me && *x == p));
                if a.get("starred").and_then(Value::as_bool).unwrap_or(false) {
                    self.stars.push((me.into(), p));
                }
                Ok(Value::Null)
            }
            "changes_since" => {
                let since = a.get("since").and_then(Value::as_i64).unwrap_or(0);
                let mine: Vec<String> = self.members.iter().filter(|m| m.user == me).map(|m| m.ws.clone()).collect();
                let orgs: Vec<String> = self.org_members.iter().filter(|m| m.user == me).map(|m| m.org.clone()).collect();
                let list: Vec<Value> = self
                    .changes
                    .iter()
                    .filter(|c| c.seq > since && (c.ws.as_ref().map(|w| mine.contains(w)).unwrap_or(false) || (c.ws.is_none() && c.org.as_ref().map(|o| orgs.contains(o)).unwrap_or(false))))
                    .take(1000)
                    .map(|c| json!({"seq": c.seq, "workspace_id": c.ws, "org_id": c.org, "project_id": c.project, "kind": c.kind, "by": c.by, "at": format_time(c.at)}))
                    .collect();
                Ok(json!({"latest": self.changes.last().map(|c| c.seq).unwrap_or(0), "oldest": self.changes.first().map(|c| c.seq).unwrap_or(0), "changes": list}))
            }
            other => Err(format!("unknown function {other}")),
        }
    }

    fn function(&mut self, name: &str, body: &Value, me: &str, req: &Request) -> Out {
        match name {
            "send-invite" => {
                let id = s(body, "invite_id");
                let i = self.invites.iter().find(|i| i.id == id).cloned().ok_or_else(|| nc("not_found"))?;
                if !self.manages(&i.org, me) {
                    return Err(nc("not_found"));
                }
                Ok(json!({"sent": false}))
            }
            "web-link" => {
                let next = s(body, "next");
                let safe = next.starts_with('/') && !next.starts_with("//") && !next.contains('\\') && next.len() <= 300;
                let path = if safe { next } else { "/".to_string() };
                let code = self.make_grant(me, "web", "", None, 300);
                Ok(json!({"url": format!("{WEB_URL}/auth/handoff#t={code}&next={}", super::client::enc(&path))}))
            }
            "delete-account" => {
                let token = req.get_header("authorization").and_then(|h| h.strip_prefix("Bearer ")).unwrap_or("");
                let issued = self.access.get(token).map(|(_, exp)| exp - self.token_life).unwrap_or(0);
                if self.now() - issued > 600 {
                    return Err(nc("reauth"));
                }
                let owned: Vec<String> = self.org_members.iter().filter(|m| m.user == me && m.role == "owner").map(|m| m.org.clone()).collect();
                for org in owned {
                    let pick = |role: &str| self.org_members.iter().filter(|m| m.org == org && m.user != me && m.role == role).min_by_key(|m| m.at).map(|m| m.user.clone());
                    match pick("admin").or_else(|| pick("member")) {
                        Some(heir) => {
                            for m in self.org_members.iter_mut().filter(|m| m.org == org && m.user == heir) {
                                m.role = "owner".into();
                            }
                        }
                        None => self.delete_org(&org),
                    }
                }
                let personal: Vec<String> = self.spaces.iter().filter(|w| w.owner == me && w.org.is_none()).map(|w| w.id.clone()).collect();
                self.projects.retain(|p| !personal.contains(&p.ws));
                self.spaces.retain(|w| !personal.contains(&w.id));
                self.members.retain(|m| m.user != me);
                self.org_members.retain(|m| m.user != me);
                self.leases.retain(|l| l.user != me);
                self.devices.retain(|d| d.user != me);
                self.grants.retain(|g| g.user != me);
                self.users.retain(|u| u.id != me);
                self.access.retain(|_, (u, _)| u != me);
                self.refresh.retain(|_, u| u != me);
                Ok(json!({"deleted": true}))
            }
            other => Err(format!("unknown edge function {other}")),
        }
    }
}

#[derive(Clone)]
pub struct FakeCloud {
    world: Arc<Mutex<World>>,
}

impl Default for FakeCloud {
    fn default() -> Self {
        FakeCloud::new()
    }
}

impl FakeCloud {
    pub fn new() -> FakeCloud {
        FakeCloud { world: Arc::new(Mutex::new(World { max_projects: 50, max_orgs: 3, max_people: 10, token_life: 3600, ..Default::default() })) }
    }

    fn w(&self) -> MutexGuard<'_, World> {
        self.world.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn config() -> CloudConfig {
        CloudConfig::new("https://fake.nativecloud.test", "anon-key").with_web(WEB_URL)
    }

    pub fn client(&self) -> Client {
        Client::new(FakeCloud::config(), self.clone())
    }

    pub fn add_user(&self, email: &str, password: &str, name: &str) -> String {
        self.w().make_user(email, password, name, true)
    }

    pub fn signed_in(&self, email: &str, name: &str) -> Client {
        self.add_user(email, "Password123", name);
        let c = self.client();
        c.sign_in(email, "Password123").expect("fake sign in");
        c
    }

    pub fn code(&self, kind: &str, email: &str) -> Option<String> {
        self.w().codes.get(&(kind.to_string(), email.trim().to_lowercase())).cloned()
    }

    pub fn set_offline(&self, offline: bool) {
        self.w().offline = offline;
    }

    pub fn advance(&self, seconds: i64) {
        self.w().shift += seconds;
    }

    pub fn set_token_life(&self, seconds: i64) {
        self.w().token_life = seconds;
    }

    pub fn set_max_projects(&self, n: i64) {
        self.w().max_projects = n;
    }

    pub fn set_max_people(&self, n: i64) {
        self.w().max_people = n;
    }

    pub fn approve(&self, email: &str, app: &str, device: &str, challenge: &str) -> String {
        let mut w = self.w();
        let me = w.by_email(email).map(|u| u.id.clone()).unwrap_or_default();
        w.make_grant(&me, app, device, Some(challenge.to_string()), 120)
    }

    pub fn grant_used(&self, code: &str) -> bool {
        self.w().grants.iter().any(|g| g.code == code && g.used)
    }

    pub fn org_role(&self, org: &str, email: &str) -> Option<String> {
        let w = self.w();
        let me = w.by_email(email).map(|u| u.id.clone())?;
        w.org_role(org, &me)
    }

    pub fn space_role(&self, ws: &str, email: &str) -> Option<String> {
        let w = self.w();
        let me = w.by_email(email).map(|u| u.id.clone())?;
        w.role(ws, &me)
    }

    pub fn fail_next(&self, route: &str, status: u16, body: &str) {
        self.w().fail.push((route.to_string(), status, body.to_string()));
    }

    pub fn requests(&self) -> Vec<String> {
        self.w().log.clone()
    }

    pub fn count(&self, fragment: &str) -> usize {
        self.w().log.iter().filter(|l| l.contains(fragment)).count()
    }

    pub fn clear_log(&self) {
        self.w().log.clear();
    }

    pub fn user_id(&self, email: &str) -> String {
        self.w().by_email(email).map(|u| u.id.clone()).unwrap_or_default()
    }

    pub fn body(&self, project: &str) -> String {
        self.w().projects.iter().find(|p| p.id == project).map(|p| p.body.clone()).unwrap_or_default()
    }

    pub fn version(&self, project: &str) -> i64 {
        self.w().projects.iter().find(|p| p.id == project).map(|p| p.version).unwrap_or(0)
    }

    pub fn workspace_of(&self, project: &str) -> String {
        self.w().projects.iter().find(|p| p.id == project).map(|p| p.ws.clone()).unwrap_or_default()
    }

    pub fn exists(&self, project: &str) -> bool {
        self.w().projects.iter().any(|p| p.id == project && p.deleted_at.is_none())
    }

    pub fn snapshots(&self, project: &str) -> Vec<(String, String, String)> {
        self.w().versions.iter().filter(|v| v.project == project).map(|v| (v.kind.clone(), v.label.clone(), v.body.clone())).collect()
    }

    pub fn lease_holder(&self, project: &str) -> Option<String> {
        let w = self.w();
        w.live_lease(project).and_then(|l| w.user(&l.user)).map(|u| u.email.clone())
    }

    pub fn edit_as(&self, email: &str, project: &str, body: &str) -> i64 {
        let mut w = self.w();
        let me = w.by_email(email).map(|u| u.id.clone()).unwrap_or_default();
        let now = w.now();
        let Some(i) = w.projects.iter().position(|p| p.id == project) else {
            return 0;
        };
        w.projects[i].body = body.to_string();
        w.projects[i].version += 1;
        w.projects[i].updated_by = me.clone();
        w.projects[i].updated_at = now;
        let (ws, v) = (w.projects[i].ws.clone(), w.projects[i].version);
        w.note(&ws, Some(project), "project", &me);
        v
    }

    pub fn lock_as(&self, email: &str, project: &str, device_name: &str) {
        let mut w = self.w();
        let me = w.by_email(email).map(|u| u.id.clone()).unwrap_or_default();
        let device = new_id();
        w.take_lease(project, &me, &device, device_name);
        let ws = w.projects.iter().find(|p| p.id == project).map(|p| p.ws.clone()).unwrap_or_default();
        w.note(&ws, Some(project), "lease", &me);
    }

    pub fn unlock(&self, project: &str) {
        self.w().leases.retain(|l| l.project != project);
    }

    pub fn trim_feed(&self, keep_after: i64) {
        self.w().changes.retain(|c| c.seq > keep_after);
    }
}

impl Transport for FakeCloud {
    fn send(&self, req: &Request) -> Result<Reply, String> {
        let mut w = self.w();
        let path = req.path().to_string();
        w.log.push(format!("{} {}", req.method, path));
        if w.offline {
            return Err("offline".into());
        }
        if req.get_header("apikey") != Some("anon-key") {
            return Ok(Reply { status: 401, body: json!({"message": "Invalid API key"}).to_string(), date: None });
        }
        if let Some(i) = w.fail.iter().position(|(r, _, _)| path.contains(r.as_str())) {
            let (_, status, body) = w.fail.remove(i);
            return Ok(Reply { status, body, date: Some(http_date(w.now())) });
        }
        let (status, v) = w.reply(req);
        let body = if v.is_null() { String::new() } else { v.to_string() };
        Ok(Reply { status, body, date: Some(http_date(w.now())) })
    }
}

pub struct Demo {
    pub cloud: FakeCloud,
    pub signed_in: super::Account,
    pub signed_out: super::Account,
    pub org: String,
    pub space: String,
}

pub fn account_for(info: crate::AppInfo, client: Client, device_name: &str) -> super::Account {
    let device = Device { id: new_id(), name: device_name.into(), platform: "macos".into() };
    let mut a = super::Account::with_engine(info, Engine::inline(Worker::new(client, device.clone(), Outbox::memory(), Heartbeat::off())));
    a.poll();
    a
}

pub fn demo(info: crate::AppInfo) -> Demo {
    let cloud = FakeCloud::new();
    let ann = cloud.signed_in("ann@example.com", "Ann Lee");
    let ben = cloud.signed_in("ben@example.com", "Ben Ito");
    cloud.signed_in("cal@example.com", "Cal Diaz");
    let made = ann.create_org("Writers room").unwrap_or_default();
    let pitches = ann.create_space(&made.id, "Pitches").unwrap_or_default();
    for (email, role, spaces) in [("ben@example.com", OrgRole::Admin, Vec::new()), ("cal@example.com", OrgRole::Guest, vec![pitches.clone()])] {
        if let Ok(i) = ann.invite_member(&made.id, email, role, &spaces) {
            let other = cloud.client();
            let _ = other.sign_in(email, "Password123");
            let _ = other.accept_invite(&i);
        }
    }
    let _ = ann.invite_member(&made.id, "dee@example.com", OrgRole::Member, &[]);
    if let Ok(season) = ben.create_org("Pilot season") {
        let _ = ben.invite_member(&season.id, "ann@example.com", OrgRole::Member, &[]);
    }
    let personal = ann.me().map(|m| m.personal_workspace).unwrap_or_default();
    for (ws, title, words) in [(&personal, "Night shift", 18_000), (&personal, "Untitled pilot", 4_000), (&made.space, "Harbour", 26_000)] {
        let _ = ann.create_project(ws, "blackline", title, &"word ".repeat(words), &serde_json::json!({}));
    }
    let _ = ann.register_device(&Device { id: new_id(), name: "Ann's PC".into(), platform: "windows".into() });
    let mut signed_in = account_for(info, ann, "Ann's Mac");
    signed_in.select_org(Some(&made.id));
    let signed_out = account_for(info, cloud.client(), "This Mac");
    Demo { cloud, signed_in, signed_out, org: made.id, space: made.space }
}

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

fn person_or_id<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Person>, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::String(id) => Some(Person { id, ..Default::default() }),
        Value::Object(_) => serde_json::from_value(v).ok(),
        _ => None,
    })
}

fn text_or_null<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::String(s) => s,
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Person {
    pub id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub name: String,
    pub hue: i32,
}

impl Person {
    pub fn shown(&self) -> String {
        if self.name.trim().is_empty() {
            "Someone".to_string()
        } else {
            self.name.trim().to_string()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub max_people: i64,
    pub max_spaces: i64,
    pub max_projects: i64,
    pub max_bytes: i64,
    pub max_owned_orgs: i64,
    pub max_auto_snapshots: i64,
    pub max_kept_snapshots: i64,
    pub max_saves_per_minute: i64,
    pub trial_days: i64,
    pub price_cents: i64,
    pub paid: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub projects: i64,
    pub bytes: i64,
    pub owned_orgs: i64,
    pub people: i64,
    pub pending: i64,
    pub spaces: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    Owner,
    Admin,
    #[default]
    Member,
    Guest,
}

impl OrgRole {
    pub const INVITABLE: [OrgRole; 3] = [OrgRole::Admin, OrgRole::Member, OrgRole::Guest];

    pub fn as_str(self) -> &'static str {
        match self {
            OrgRole::Owner => "owner",
            OrgRole::Admin => "admin",
            OrgRole::Member => "member",
            OrgRole::Guest => "guest",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OrgRole::Owner => "Owner",
            OrgRole::Admin => "Admin",
            OrgRole::Member => "Member",
            OrgRole::Guest => "Guest",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            OrgRole::Owner => "Everything, including the plan and deleting the organization.",
            OrgRole::Admin => "Manages people, spaces and settings.",
            OrgRole::Member => "Writes in every space.",
            OrgRole::Guest => "Only the spaces they're invited to.",
        }
    }

    pub fn parse(s: &str) -> Option<OrgRole> {
        match s.trim() {
            "owner" => Some(OrgRole::Owner),
            "admin" => Some(OrgRole::Admin),
            "member" => Some(OrgRole::Member),
            "guest" => Some(OrgRole::Guest),
            _ => None,
        }
    }

    pub fn can_manage(self) -> bool {
        matches!(self, OrgRole::Owner | OrgRole::Admin)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrgRef {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub role: OrgRole,
    #[serde(deserialize_with = "text_or_null")]
    pub plan: String,
    pub trial_ends_at: Option<String>,
    pub suspended: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrgCreated {
    pub id: String,
    pub slug: String,
    pub space: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrgMember {
    pub org_id: String,
    pub user_id: String,
    pub role: OrgRole,
    pub joined_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Me {
    pub id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub email: String,
    #[serde(deserialize_with = "text_or_null")]
    pub display_name: String,
    pub avatar_hue: i32,
    pub plan: Plan,
    pub usage: Usage,
    #[serde(deserialize_with = "text_or_null")]
    pub personal_workspace: String,
    pub staff: Option<String>,
    pub orgs: Vec<OrgRef>,
}

impl Me {
    pub fn shown(&self) -> String {
        if self.display_name.trim().is_empty() {
            self.email.split('@').next().unwrap_or("").to_string()
        } else {
            self.display_name.trim().to_string()
        }
    }

    pub fn person(&self) -> Person {
        Person { id: self.id.clone(), name: self.shown(), hue: self.avatar_hue }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Summary {
    pub plan: Plan,
    pub usage: Usage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Editor,
    #[default]
    Viewer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Editor => "editor",
            Role::Viewer => "viewer",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Role::Owner => "Owner",
            Role::Editor => "Editor",
            Role::Viewer => "Viewer",
        }
    }

    pub fn parse(s: &str) -> Option<Role> {
        match s.trim() {
            "owner" => Some(Role::Owner),
            "editor" => Some(Role::Editor),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    pub fn can_write(self) -> bool {
        self != Role::Viewer
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub org_id: Option<String>,
    pub owner_id: Option<String>,
    pub archived_at: Option<String>,
}

impl Workspace {
    pub fn is_personal(&self) -> bool {
        self.kind == "personal"
    }

    pub fn shown(&self) -> String {
        if self.is_personal() {
            "My cloud".to_string()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Member {
    pub workspace_id: String,
    pub user_id: String,
    pub role: Role,
    pub joined_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub display_name: String,
    pub avatar_hue: i32,
}

impl Profile {
    pub fn person(&self) -> Person {
        Person { id: self.id.clone(), name: self.display_name.clone(), hue: self.avatar_hue }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Invite {
    pub id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub email: String,
    pub role: OrgRole,
    #[serde(deserialize_with = "text_or_null")]
    pub expires_at: String,
    pub accepted_at: Option<String>,
    #[serde(deserialize_with = "text_or_null")]
    pub org_id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub org: String,
    #[serde(deserialize_with = "text_or_null")]
    pub org_slug: String,
    #[serde(deserialize_with = "text_or_null")]
    pub invited_by: String,
    #[serde(deserialize_with = "text_or_null")]
    pub state: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lease {
    pub user: Option<Person>,
    pub device_id: String,
    pub device_name: String,
    pub expires_at: String,
    pub mine: bool,
}

impl Lease {
    pub fn holder(&self) -> String {
        self.user.as_ref().map(Person::shown).unwrap_or_else(|| "Someone".into())
    }

    pub fn expires(&self) -> Option<i64> {
        super::parse_time(&self.expires_at)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub app_id: String,
    pub title: String,
    pub meta: Value,
    pub version: i64,
    pub size: i64,
    pub updated_at: String,
    #[serde(deserialize_with = "person_or_id")]
    pub updated_by: Option<Person>,
    #[serde(deserialize_with = "person_or_id")]
    pub created_by: Option<Person>,
    pub deleted_at: Option<String>,
    pub body: Option<String>,
}

impl Project {
    pub fn in_trash(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn body_text(&self) -> &str {
        self.body.as_deref().unwrap_or("")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Opened {
    pub project: Project,
    pub access: String,
    pub can_write: bool,
    pub lease: Option<Lease>,
}

impl Opened {
    pub fn editing(&self) -> bool {
        self.access == "edit"
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Renewal {
    pub held: bool,
    pub version: i64,
    pub lease: Option<Lease>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Saved {
    Saved { version: i64 },
    Conflict(Project),
    Locked(Option<Lease>),
}

impl Saved {
    pub fn from_value(v: &Value) -> Option<Saved> {
        match v.get("status")?.as_str()? {
            "saved" => Some(Saved::Saved { version: v.get("version")?.as_i64()? }),
            "conflict" => Some(Saved::Conflict(serde_json::from_value(v.get("project")?.clone()).ok()?)),
            "locked" => Some(Saved::Locked(v.get("lease").and_then(|l| serde_json::from_value(l.clone()).ok()))),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub id: String,
    pub project_id: String,
    pub version: i64,
    pub size: i64,
    pub label: String,
    pub kind: String,
    #[serde(deserialize_with = "text_or_null")]
    pub created_by: String,
    pub created_at: String,
    pub body: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditSession {
    pub id: i64,
    pub project_id: String,
    #[serde(deserialize_with = "text_or_null")]
    pub user_id: String,
    pub started_at: String,
    pub ended_at: String,
    pub words_delta: i64,
    pub saves: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Change {
    pub seq: i64,
    pub workspace_id: Option<String>,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
    pub kind: String,
    pub by: Option<String>,
    pub at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Feed {
    pub latest: i64,
    pub oldest: i64,
    pub changes: Vec<Change>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceRow {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_seen: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub body: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Space {
    pub workspace: Workspace,
    pub my_role: Role,
    pub org: Option<OrgRef>,
    pub members: Vec<(Member, Person)>,
}

impl Space {
    pub fn is_personal(&self) -> bool {
        self.workspace.is_personal()
    }

    pub fn shown(&self) -> String {
        match &self.org {
            Some(o) if !self.workspace.is_personal() => format!("{} / {}", o.name, self.workspace.name),
            _ => self.workspace.shown(),
        }
    }

    pub fn archived(&self) -> bool {
        self.workspace.archived_at.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrgView {
    pub org: OrgRef,
    pub members: Vec<(OrgMember, Person)>,
    pub invites: Vec<Invite>,
    pub spaces: Vec<Workspace>,
}

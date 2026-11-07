use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pending {
    pub project: String,
    pub base_version: i64,
    pub body: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub queued_at: i64,
}

impl Pending {
    pub fn new(project: &str, base_version: i64, body: &str) -> Pending {
        Pending { project: project.to_string(), base_version, body: body.to_string(), title: None, meta: None, queued_at: super::unix_now() }
    }

    pub fn titled(mut self, title: &str) -> Pending {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_meta(mut self, meta: Value) -> Pending {
        self.meta = Some(meta);
        self
    }
}

#[derive(Debug, Default)]
pub struct Outbox {
    path: Option<PathBuf>,
    items: Vec<Pending>,
}

impl Outbox {
    pub fn memory() -> Outbox {
        Outbox::default()
    }

    pub fn load(path: PathBuf) -> Outbox {
        let items = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        Outbox { path: Some(path), items }
    }

    pub fn put(&mut self, p: Pending) {
        match self.items.iter_mut().find(|i| i.project == p.project) {
            Some(old) => {
                let base = old.base_version.min(p.base_version);
                let queued = old.queued_at;
                *old = Pending { base_version: base, queued_at: queued, ..p };
            }
            None => self.items.push(p),
        }
        self.save();
    }

    pub fn remove(&mut self, project: &str) -> Option<Pending> {
        let i = self.items.iter().position(|p| p.project == project)?;
        let p = self.items.remove(i);
        self.save();
        Some(p)
    }

    pub fn get(&self, project: &str) -> Option<&Pending> {
        self.items.iter().find(|p| p.project == project)
    }

    pub fn items(&self) -> &[Pending] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        if self.items.is_empty() {
            let _ = std::fs::remove_file(path);
            return;
        }
        let text = serde_json::to_string(&self.items).unwrap_or_default();
        let _ = super::vault::write_private(path, &text);
    }
}

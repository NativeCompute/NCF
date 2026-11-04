#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Off,
    Checking,
    Latest,
    Newer(Release),
    Failed,
}

pub fn releases_api(repo: &str) -> String {
    format!("https://api.github.com/repos/{repo}/releases/latest")
}

pub fn releases_page(repo: &str) -> String {
    format!("https://github.com/{repo}/releases")
}

pub fn releases_latest(repo: &str) -> String {
    format!("https://github.com/{repo}/releases/latest")
}

fn parts(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("")
        .split('.')
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

pub fn newer(current: &str, latest: &str) -> bool {
    let (mut a, mut b) = (parts(current), parts(latest));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    b > a
}

pub fn parse(json: &str, repo: &str) -> Option<Release> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let tag = v.get("tag_name")?.as_str()?;
    let url = v
        .get("html_url")
        .and_then(|u| u.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| releases_page(repo));
    Some(Release {
        version: tag.trim_start_matches(['v', 'V']).to_string(),
        url,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(repo: &str, current: &str) -> std::sync::mpsc::Receiver<Status> {
    let (repo, current) = (repo.to_string(), current.to_string());
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(check(&repo, &current));
    });
    rx
}

#[cfg(not(target_arch = "wasm32"))]
fn check(repo: &str, current: &str) -> Status {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(&format!("NativeCompute-app/{current}"))
        .build();
    let body = match agent
        .get(&releases_api(repo))
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r.into_string().ok(),
        Err(ureq::Error::Status(404, _)) => return Status::Latest,
        Err(_) => None,
    };
    match body.as_deref().and_then(|b| parse(b, repo)) {
        Some(r) if newer(current, &r.version) => Status::Newer(r),
        Some(_) => Status::Latest,
        None => Status::Failed,
    }
}

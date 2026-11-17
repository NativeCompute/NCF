#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub art: Option<Art>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub rgba: std::sync::Arc<Vec<u8>>,
}

impl std::fmt::Debug for Picture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Picture({}x{})", self.width, self.height)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Art {
    pub title: String,
    pub body: String,
    pub image: Option<Picture>,
    pub accent: Option<[u8; 3]>,
    pub link: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub title: String,
    pub body: String,
    pub image: Option<String>,
    pub accent: Option<[u8; 3]>,
    pub link: Option<String>,
}

pub const ART_BASE: &str = "https://nativecompute.net/updates";
pub const ART_HOSTS: [&str; 2] = ["nativecompute.net", "www.nativecompute.net"];
pub const LINK_HOSTS: [&str; 3] = ["nativecompute.net", "www.nativecompute.net", "github.com"];
pub const MANIFEST_MAX: usize = 32 * 1024;
pub const IMAGE_MAX: usize = 2 * 1024 * 1024;
pub const IMAGE_SIDE: usize = 2048;

pub fn slug_of(repo: &str) -> String {
    repo.rsplit('/').next().unwrap_or(repo).to_lowercase()
}

pub fn art_url(slug: &str, version: &str) -> String {
    format!("{ART_BASE}/{slug}/{}.json", version.trim_start_matches(['v', 'V']))
}

fn host_of(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let host = rest.split(['/', '?', '#']).next()?;
    (!host.is_empty() && !host.contains(['@', ':'])).then_some(host)
}

pub fn allowed(url: &str, hosts: &[&str]) -> bool {
    host_of(url).is_some_and(|h| hosts.iter().any(|x| x.eq_ignore_ascii_case(h)))
}

pub fn resolve(manifest_url: &str, path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() || path.contains("..") || path.contains('\\') {
        return None;
    }
    let url = if path.starts_with("https://") {
        path.to_string()
    } else if let Some(rest) = path.strip_prefix('/') {
        format!("https://{}/{rest}", host_of(manifest_url)?)
    } else {
        let base = manifest_url.rsplit_once('/')?.0;
        format!("{base}/{path}")
    };
    allowed(&url, &ART_HOSTS).then_some(url)
}

fn clip(s: &str, max: usize) -> String {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= max {
        t
    } else {
        let mut out: String = t.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn hex(s: &str) -> Option<[u8; 3]> {
    let h = s.trim().strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(h, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

pub fn parse_manifest(json: &str, manifest_url: &str) -> Result<Manifest, String> {
    if json.len() > MANIFEST_MAX {
        return Err("the manifest is too large".into());
    }
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let text = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_string);
    let title = clip(&text("title").ok_or("the manifest has no title")?, 80);
    if title.is_empty() {
        return Err("the manifest has no title".into());
    }
    let body = clip(&text("body").unwrap_or_default(), 420);
    let image = match text("image") {
        Some(p) => Some(resolve(manifest_url, &p).ok_or("the image is not on nativecompute.net")?),
        None => None,
    };
    let accent = text("accent").and_then(|a| hex(&a));
    let link = text("link").filter(|l| allowed(l, &LINK_HOSTS));
    Ok(Manifest { title, body, image, accent, link })
}

pub fn decode_png(bytes: &[u8]) -> Result<Picture, String> {
    if bytes.len() > IMAGE_MAX {
        return Err("the image is too large".into());
    }
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16 | png::Transformations::ALPHA);
    let mut reader = dec.read_info().map_err(|e| e.to_string())?;
    let (w, h) = {
        let info = reader.info();
        (info.width as usize, info.height as usize)
    };
    if w == 0 || h == 0 || w > IMAGE_SIDE || h > IMAGE_SIDE {
        return Err("the image is too big on screen".into());
    }
    let mut buf = vec![0; reader.output_buffer_size().ok_or("the image is too large")?];
    let frame = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let data = &buf[..frame.buffer_size()];
    let rgba: Vec<u8> = match frame.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::GrayscaleAlpha => data.chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        png::ColorType::Rgb => data.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err("the image has an unusual colour type".into()),
    };
    if rgba.len() != w * h * 4 {
        return Err("the image did not decode".into());
    }
    Ok(Picture { width: w, height: h, rgba: std::sync::Arc::new(rgba) })
}

pub fn art_from(m: &Manifest, image: Option<&[u8]>) -> Art {
    Art {
        title: m.title.clone(),
        body: m.body.clone(),
        image: image.and_then(|b| decode_png(b).ok()),
        accent: m.accent,
        link: m.link.clone(),
    }
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
        art: None,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(repo: &str, current: &str) -> std::sync::mpsc::Receiver<Status> {
    spawn_with(repo, current, None)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_with(repo: &str, current: &str, cache: Option<std::path::PathBuf>) -> std::sync::mpsc::Receiver<Status> {
    let (repo, current) = (repo.to_string(), current.to_string());
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let status = match check(&repo, &current) {
            Status::Newer(mut r) => {
                r.art = fetch_art(&slug_of(&repo), &r.version, &current, cache.as_deref());
                Status::Newer(r)
            }
            other => other,
        };
        let _ = tx.send(status);
    });
    rx
}

#[cfg(not(target_arch = "wasm32"))]
fn read_capped(r: ureq::Response, max: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    if !allowed(r.get_url(), &ART_HOSTS) {
        return None;
    }
    let mut out = Vec::new();
    r.into_reader().take(max as u64 + 1).read_to_end(&mut out).ok()?;
    (out.len() <= max).then_some(out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_art(slug: &str, version: &str, current: &str, cache: Option<&std::path::Path>) -> Option<Art> {
    let url = art_url(slug, version);
    let dir = cache.map(|c| c.join(format!("{slug}-{}", version.trim_start_matches(['v', 'V']))));
    if let Some(d) = &dir {
        if let Ok(json) = std::fs::read_to_string(d.join("manifest.json")) {
            if let Ok(m) = parse_manifest(&json, &url) {
                let image = std::fs::read(d.join("image.png")).ok();
                return Some(art_from(&m, image.as_deref()));
            }
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(3))
        .redirects(3)
        .user_agent(&format!("NativeCompute-app/{current}"))
        .build();
    let bytes = read_capped(agent.get(&url).call().ok()?, MANIFEST_MAX)?;
    let json = String::from_utf8(bytes).ok()?;
    let m = parse_manifest(&json, &url).ok()?;
    let image = m.image.as_ref().and_then(|u| agent.get(u).call().ok()).and_then(|r| read_capped(r, IMAGE_MAX));
    let art = art_from(&m, image.as_deref());
    if let Some(d) = &dir {
        if std::fs::create_dir_all(d).is_ok() {
            let _ = std::fs::write(d.join("manifest.json"), &json);
            if let (Some(b), Some(_)) = (&image, &art.image) {
                let _ = std::fs::write(d.join("image.png"), b);
            }
        }
    }
    Some(art)
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

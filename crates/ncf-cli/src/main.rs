mod template;

use std::path::{Path, PathBuf};

use ncf::mark::{line, slab, AppMark};

const USAGE: &str = "ncf — the NativeCompute Framework

  ncf new <Name> [--dir <path>] [--tagline <text>] [--repo <Owner/name>] [--ncf-path <path>]
      Make a new NativeCompute desktop app.
  ncf tokens [--check]
      Write (or check) the Rust and CSS token files from tokens/ncf.toml.
  ncf mark --svg <path> | --ico <path> | --icns <path>
      Write the NativeCompute mark.
  ncf --version";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("new") => new(&args[1..]),
        Some("tokens") => tokens(args.iter().any(|a| a == "--check")),
        Some("mark") => mark(&args[1..]),
        Some("--version") | Some("-V") => {
            println!("ncf {}", ncf::VERSION);
            Ok(())
        }
        _ => {
            println!("{USAGE}");
            Ok(())
        }
    };
    if let Err(e) = result {
        eprintln!("ncf: {e}");
        std::process::exit(1);
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

pub fn slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    s.trim_end_matches('-').to_string()
}

pub struct Plan {
    pub name: String,
    pub slug: String,
    pub id: String,
    pub repo: String,
    pub tagline: String,
    pub ncf_path: Option<PathBuf>,
}

pub fn dep_lines(plan: &Plan) -> (String, String, String) {
    match &plan.ncf_path {
        Some(p) => {
            let p = toml_path(p);
            (
                format!("ncf = {{ path = \"{p}/crates/ncf\" }}"),
                format!("ncf = {{ path = \"{p}/crates/ncf\", features = [\"testing\"] }}"),
                format!("ncf-build = {{ path = \"{p}/crates/ncf-build\" }}"),
            )
        }
        None => {
            let tag = format!("v{}", ncf::VERSION);
            (
                format!("ncf = {{ git = \"https://github.com/NativeCompute/ncf\", tag = \"{tag}\" }}"),
                format!("ncf = {{ git = \"https://github.com/NativeCompute/ncf\", tag = \"{tag}\", features = [\"testing\"] }}"),
                format!("ncf-build = {{ git = \"https://github.com/NativeCompute/ncf\", tag = \"{tag}\" }}"),
            )
        }
    }
}

pub fn toml_path(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    s.strip_prefix(r"\\?\").unwrap_or(&s).replace('\\', "/")
}

pub fn fill(text: &str, plan: &Plan) -> String {
    let (dep, testing, build) = dep_lines(plan);
    text.replace("__NCF_TESTING__", &testing)
        .replace("__NCF_BUILD__", &build)
        .replace("__NCF__", &dep)
        .replace("__NAME__", &plan.name)
        .replace("__SLUG__", &plan.slug)
        .replace("__ID__", &plan.id)
        .replace("__REPO__", &plan.repo)
        .replace("__TAGLINE__", &plan.tagline)
}

pub const DEFAULT_MARK: AppMark = AppMark {
    name: "app",
    parts: &[slab((0.27, 0.83), (0.49, 0.17)), line((0.585, 0.83), (0.805, 0.17))],
};

pub fn write_app(dir: &Path, plan: &Plan) -> Result<(), String> {
    if dir.exists() && dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
        return Err(format!("{} already has files in it", dir.display()));
    }
    for (path, text) in template::FILES {
        let dest = dir.join(fill(path, plan));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&dest, fill(text, plan)).map_err(|e| format!("{}: {e}", dest.display()))?;
    }
    let mark = AppMark { name: "app", parts: DEFAULT_MARK.parts };
    std::fs::write(dir.join(format!("packaging/{}.svg", plan.slug)), mark.svg().replace("<title>app</title>", &format!("<title>{}</title>", plan.name)))
        .map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("packaging/{}.ico", plan.slug)), mark.ico()).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = dir.join("install.sh");
        let _ = std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755));
    }
    Ok(())
}

fn new(args: &[String]) -> Result<(), String> {
    let name = args.first().filter(|a| !a.starts_with("--")).ok_or("ncf new <Name>: name the app")?.clone();
    let slug = slug(&name);
    if slug.is_empty() {
        return Err("the name needs at least one letter or digit".into());
    }
    let plan = Plan {
        id: format!("net.nativecompute.{}", name.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>()),
        repo: flag(args, "--repo").unwrap_or_else(|| format!("NativeCompute/{slug}")),
        tagline: flag(args, "--tagline").unwrap_or_else(|| "A NativeCompute app.".to_string()),
        ncf_path: flag(args, "--ncf-path").map(PathBuf::from).map(|p| p.canonicalize().unwrap_or(p)),
        slug: slug.clone(),
        name,
    };
    let dir = flag(args, "--dir").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(&slug));
    write_app(&dir, &plan)?;
    println!("Made {} in {}", plan.name, dir.display());
    println!("  cd {} && cargo run", dir.display());
    Ok(())
}

fn tokens(check: bool) -> Result<(), String> {
    if check {
        let (r, c) = ncf_tokens::generate()?;
        let root = ncf_tokens::root();
        let same = |p: &str, want: &str| std::fs::read_to_string(root.join(p)).map(|s| s == want).unwrap_or(false);
        if same(ncf_tokens::RUST_PATH, &r) && same(ncf_tokens::CSS_PATH, &c) {
            println!("tokens are up to date");
            Ok(())
        } else {
            Err("tokens are out of date: run `ncf tokens`".into())
        }
    } else {
        ncf_tokens::write()?;
        println!("wrote {} and {}", ncf_tokens::RUST_PATH, ncf_tokens::CSS_PATH);
        Ok(())
    }
}

fn mark(args: &[String]) -> Result<(), String> {
    if let Some(p) = flag(args, "--svg") {
        std::fs::write(&p, ncf::mark::NATIVECOMPUTE.svg()).map_err(|e| e.to_string())?;
        println!("wrote {p}");
    }
    if let Some(p) = flag(args, "--ico") {
        std::fs::write(&p, ncf::mark::NATIVECOMPUTE.ico()).map_err(|e| e.to_string())?;
        println!("wrote {p}");
    }
    if let Some(p) = flag(args, "--icns") {
        std::fs::write(&p, ncf::mark::NATIVECOMPUTE.icns()).map_err(|e| e.to_string())?;
        println!("wrote {p}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_paths_become_toml_paths() {
        assert_eq!(toml_path(std::path::Path::new(r"\\?\D:\a\ncf\ncf")), "D:/a/ncf/ncf");
        assert_eq!(toml_path(std::path::Path::new("/home/me/ncf")), "/home/me/ncf");
    }

    #[test]
    fn names_become_slugs() {
        assert_eq!(slug("Blackline"), "blackline");
        assert_eq!(slug("My Great App!"), "my-great-app");
        assert_eq!(slug("  ...  "), "");
    }

    #[test]
    fn every_template_file_is_shipped() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/app");
        let mut on_disk = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("tmpl") {
                    on_disk.push(p);
                }
            }
        }
        assert_eq!(on_disk.len(), template::FILES.len(), "a .tmpl file is missing from crates/ncf-cli/src/template.rs");
    }

    #[test]
    fn a_new_app_leaves_no_placeholder_behind() {
        let dir = std::env::temp_dir().join(format!("ncf-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let plan = Plan {
            name: "Ledger".into(),
            slug: "ledger".into(),
            id: "net.nativecompute.Ledger".into(),
            repo: "NativeCompute/ledger".into(),
            tagline: "Money, written down.".into(),
            ncf_path: None,
        };
        write_app(&dir, &plan).unwrap();
        for (path, _) in template::FILES {
            let text = std::fs::read_to_string(dir.join(fill(path, &plan))).unwrap();
            assert!(!text.contains("__"), "{path} still has a placeholder");
        }
        assert!(dir.join("packaging/ledger.ico").exists());
        assert!(std::fs::read_to_string(dir.join("Cargo.toml")).unwrap().contains(&format!("tag = \"v{}\"", ncf::VERSION)));
        assert!(write_app(&dir, &plan).is_err(), "never writes over an existing folder");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

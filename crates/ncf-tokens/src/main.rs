fn main() {
    let check = std::env::args().any(|a| a == "--check");
    if check {
        match ncf_tokens::generate() {
            Ok((r, c)) => {
                let root = ncf_tokens::root();
                let same = |p: &str, want: &str| std::fs::read_to_string(root.join(p)).map(|s| s == want).unwrap_or(false);
                if same(ncf_tokens::RUST_PATH, &r) && same(ncf_tokens::CSS_PATH, &c) {
                    println!("tokens are up to date");
                } else {
                    eprintln!("tokens are out of date: run `cargo run -p ncf-tokens`");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }
    match ncf_tokens::write() {
        Ok(()) => println!("wrote {} and {}", ncf_tokens::RUST_PATH, ncf_tokens::CSS_PATH),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

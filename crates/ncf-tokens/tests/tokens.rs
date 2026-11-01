use ncf_tokens::{contrast, generate, root, Tokens, CSS_PATH, RUST_PATH, TOKENS_PATH};

#[test]
fn the_generated_files_match_the_tokens() {
    let (r, c) = generate().unwrap();
    assert_eq!(std::fs::read_to_string(root().join(RUST_PATH)).unwrap(), r, "run `cargo run -p ncf-tokens`");
    assert_eq!(std::fs::read_to_string(root().join(CSS_PATH)).unwrap(), c, "run `cargo run -p ncf-tokens`");
}

#[test]
fn every_text_colour_reads_on_every_surface() {
    let t = Tokens::load(&std::fs::read_to_string(root().join(TOKENS_PATH)).unwrap()).unwrap();
    for cast in ["dark", "light"] {
        for bg in ["void", "deep", "panel", "panel_2", "sunken"] {
            let b = t.color(cast, bg).unwrap();
            for fg in ["text", "text_2", "text_3", "signal_hi"] {
                let f = t.color(cast, fg).unwrap();
                let ratio = contrast(f, b);
                assert!(ratio >= 4.5, "{cast}: {fg} on {bg} is {ratio:.2}:1, under WCAG AA");
            }
        }
        let ink = t.color(cast, "ink").unwrap();
        let signal = t.color(cast, "signal").unwrap();
        assert!(contrast(ink, signal) >= 4.5, "{cast}: text on the primary button");
    }
}

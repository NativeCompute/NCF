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

fn load() -> Tokens {
    Tokens::load(&std::fs::read_to_string(root().join(TOKENS_PATH)).unwrap()).unwrap()
}

fn over(top: ncf_tokens::Rgba, bottom: ncf_tokens::Rgba) -> ncf_tokens::Rgba {
    let a = top.3 as f64 / 255.0;
    let m = |t: u8, b: u8| (t as f64 * a + b as f64 * (1.0 - a)).round() as u8;
    ncf_tokens::Rgba(m(top.0, bottom.0), m(top.1, bottom.1), m(top.2, bottom.2), 255)
}

#[test]
fn text_on_fills_reads_in_both_casts() {
    let t = load();
    for cast in ["dark", "light"] {
        let c = |n: &str| t.color(cast, n).unwrap();
        for fill in ["signal", "signal_deep"] {
            let r = contrast(c("ink"), c(fill));
            assert!(r >= 4.5, "{cast}: ink on {fill} is {r:.2}:1");
        }
        let r = contrast(c("on_danger"), c("danger"));
        assert!(r >= 4.5, "{cast}: on_danger on danger is {r:.2}:1");
        for (i, b) in t.brights(cast).unwrap().into_iter().enumerate() {
            let r = contrast(c("on_tile"), b);
            assert!(r >= 4.5, "{cast}: on_tile on bright {i} is {r:.2}:1");
        }
        for bg in ["void", "deep"] {
            let r = contrast(c("focus"), c(bg));
            assert!(r >= 3.0, "{cast}: focus ring on {bg} is {r:.2}:1");
        }
        for bg in ["deep", "panel"] {
            for (i, g) in t.groups(cast).unwrap().into_iter().enumerate() {
                let r = contrast(g, c(bg));
                assert!(r >= 4.5, "{cast}: group {i} on {bg} is {r:.2}:1");
            }
        }
    }
}

#[test]
fn text_reads_over_glass_and_the_wallpaper() {
    let t = load();
    for cast in ["dark", "light"] {
        let c = |n: &str| t.color(cast, n).unwrap();
        let mut grounds = vec![over(c("glass"), c("void"))];
        for w in ["wall_a", "wall_b", "wall_c", "glow"] {
            grounds.push(over(c(w), c("void")));
            grounds.push(over(c("glass"), over(c(w), c("void"))));
        }
        for g in grounds {
            for fg in ["text", "text_2"] {
                let r = contrast(c(fg), g);
                assert!(r >= 4.5, "{cast}: {fg} over a wallpaper or glass ground is {r:.2}:1");
            }
        }
    }
}

#[test]
fn the_accent_is_a_fill_and_signal_hi_is_its_text() {
    let t = load();
    let light = |n: &str| t.color("light", n).unwrap();
    assert!(contrast(light("signal"), light("void")) < 4.5, "coral never reads as text on light paper, which is why signal_hi exists");
    assert!(contrast(light("signal_hi"), light("void")) >= 4.5);
    assert_eq!(t.brights("dark").unwrap(), t.brights("light").unwrap(), "the tile colours are the same in both casts");
}

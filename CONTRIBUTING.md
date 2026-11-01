# Contributing

Thanks for helping. NCF is the shared base of every NativeCompute app, so changes are held to a few rules.

1. Read [AGENTS.md](AGENTS.md). It applies to people too.
2. Design values go in `tokens/ncf.toml`, then `cargo run -p ncf-tokens`. Never edit `crates/ncf/src/tokens.rs` or `web/ncf-tokens.css` by hand.
3. A new component goes in `crates/ncf` and, where it makes sense, `web/`. Show it in `examples/gallery` and `web/gallery.html`, and document it in `docs/components.md`.
4. No comments in source code; explanations go in `docs/`. A lazy marker such as `// thing part here` is fine. `tools/no-comments.sh` checks this.
5. Before you open a pull request, run:
   ```sh
   cargo test --workspace --all-features
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo build -p ncf --lib --target wasm32-unknown-unknown
   tools/no-comments.sh
   tools/shots.sh
   ```
   and look at the screenshots in both casts.
6. Changes that alter how an existing app looks or behaves need a line in `CHANGELOG.md` saying what changed and how to move.

Bugs and ideas: open an issue on GitHub. Security problems: see [SECURITY.md](SECURITY.md).

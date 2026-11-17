# Repository guide

- `rh` is a Rust 2024 package with a library in `src/lib.rs` and a terminal chat loop in `src/main.rs`. At startup, `src/config.rs` loads and validates `config.toml` and `config.json` from system, user, and project `.rh` directories (Git root or cwd); later locations override earlier ones and `-v` overrides config. The loop uses `rustyline` for editing and in-session history, echoes the first three whitespace-separated words per line, and exits on Ctrl-C or EOF. Prompts appear only for terminal input; piped input is supported. The README's async input and introspection features are still goals.
- Rust 2024 requires Rust 1.85 or newer; no toolchain is pinned.
- Run with `cargo run`; pass CLI flags after `--`, e.g. `cargo run -- --help` or `cargo run -- -v`. Verbose diagnostics go to stderr. Verify with `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test`; use `cargo test <name>` to filter tests.

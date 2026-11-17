# Startup configuration discovery

## Checklist

- [x] Review the interrupted implementation and run the existing tests.
- [x] Complete configuration discovery, regression tests, documentation, and validation.

## Existing implementation

`src/config.rs` reads optional `config.toml` and `config.json` files before the chat loop starts. Supported settings currently contain only `verbose`. Missing files are ignored; read errors, invalid syntax, invalid values, and unknown settings stop startup with the filename in the error.

Configuration is merged from low to high priority:

- Unix: reversed `XDG_CONFIG_DIRS` (default `/etc/xdg`), then `XDG_CONFIG_HOME` (default `$HOME/.config`). Append `rh` to each directory. Ignore relative environment paths.
- macOS: the Unix locations, plus `/Library/Application Support/rh` before user locations and `$HOME/Library/Application Support/rh` after them.
- Windows: `%ProgramData%\rh`, `%USERPROFILE%\.config\rh`, then `%APPDATA%\rh`.
- Project: `.rh` in the nearest ancestor containing `.git` (directory or worktree file), or the current directory outside Git.

Read TOML before JSON in each directory. Later files override only explicitly supplied settings. The CLI `-v` flag enables verbosity regardless of configuration. Verbose startup reports loaded filenames to stderr.

## Completion work

Keep the recovered behavior. Extend subprocess tests for missing files, system/user loading, platform-specific precedence, XDG fallbacks and ordering, cwd fallback, worktrees, partial settings, and read/parse errors. Use child-process environments rather than modifying process-global environment variables.

The interrupted code already passes its five configuration tests. Add regression coverage before changing production code. Fix any failures found, run rustfmt, and remove the unused Tokio dependency reported by Cargo; async support remains a future goal. Document configuration paths, filenames, precedence, and an example in `README.md`.

Validate with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`. Commit the completed change on `main`, as requested by Doug. Leave the unrelated `/tmp` ignore entry untouched and uncommitted. Native Windows and Linux execution is not available on this macOS host; platform-specific tests will run on those hosts.

## Validation results

- The recovered implementation passed all added regression tests without behavioral changes.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed without warnings.
- `cargo test`: passed, 24 tests including 16 configuration tests on macOS.
- `git diff --check`: passed.
- Windows-specific tests are included but were not executed on this host. Linux was not tested natively.

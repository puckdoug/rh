# rh: an experimental AI harness written in Rust

This is currently an experiment to create a harness for running AI agents in rust (rh: Rust Harness). Goals beyond what standard harnesses provide:

1. Allow async communication (e.g. the harness should accept input even when working on a task)
2. Allow extensive introspection so the harness can answer questions regarding its own operation.
3. It's rust, so of course run fast and stable.

The harness will manage a suite of specialized agents to deliver complete responses. This will need async (Tokio) interaction on top of the interaction with the user.

## Configuration

At startup, `rh` reads optional `config.toml` and `config.json` files from the directories below, in order. Later files override earlier settings. Within each directory, JSON overrides TOML.

| Platform | Directories, from lowest to highest priority |
| --- | --- |
| Unix/Linux | Each `$XDG_CONFIG_DIRS/rh` (default `/etc/xdg/rh`), then `$XDG_CONFIG_HOME/rh` (default `~/.config/rh`) |
| macOS | Each `$XDG_CONFIG_DIRS/rh` (default `/etc/xdg/rh`), `/Library/Application Support/rh`, `$XDG_CONFIG_HOME/rh` (default `~/.config/rh`), `~/Library/Application Support/rh` |
| Windows | `%ProgramData%\rh`, `%USERPROFILE%\.config\rh`, `%APPDATA%\rh` |

`XDG_CONFIG_DIRS` is colon-separated; its first directory has the highest system priority. Empty or unset XDG variables use the defaults. Relative environment paths are ignored.

Finally, `rh` checks `.rh` in the nearest Git project root, including worktrees. Outside Git, it checks `.rh` in the current directory instead. Project settings override system and user settings.

`rh` also reads `AGENTS.md` from that project root (or the current directory outside Git). Its UTF-8 Markdown is kept unchanged for future agent use; the current chat loop does not act on it yet. Missing `AGENTS.md` is allowed. Read errors or invalid UTF-8 stop startup with the filename in the error. Verbose mode reports the filename and byte count, not its contents. Instruction files elsewhere are not loaded.

For example, create `.rh/config.toml` in your project:

```toml
verbose = true
```

Or use `.rh/config.json`:

```json
{"verbose": true}
```

`verbose` defaults to `false`. Omitted settings keep their earlier values. `-v` or `--verbose` always enables diagnostics, including loaded configuration filenames, on stderr. Invalid configuration, unknown settings, and unreadable files stop startup with an error naming the file; missing files are skipped.

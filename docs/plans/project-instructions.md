# Project AGENTS.md

## Checklist

- [x] Add failing tests, implement startup reading, document behavior, and validate.

## Design

Continue on `main` as Doug requested for startup configuration work. Keep the unrelated `.gitignore` change uncommitted.

After loading configuration, read `AGENTS.md` from the same project root: the nearest ancestor with a `.git` file or directory, otherwise cwd. Do not search user/system directories, `.rh`, or subdirectories for instructions.

Add `ProjectInstructions { path: PathBuf, contents: String }` and an optional `project_instructions` field to `Config`. Retain Markdown verbatim as UTF-8, without parsing it as configuration. Missing files yield `None`; other read errors, including invalid UTF-8, stop startup with the path in the error. Keep empty files as present instructions. In verbose mode, print the filename and byte count to stderr, never the text. Normal output stays unchanged. This stores instructions for future agent integration; the current chat loop still only echoes words.

## Tests

Start from a passing `cargo test`. Add subprocess tests before production changes: root discovery from a subdirectory, nearest Git/worktree root, cwd outside Git, empty instructions, read errors, invalid UTF-8, quiet startup, and no fallback to unrelated instruction files. Existing missing-configuration tests cover absent AGENTS.md. Add a unit test proving the exact contents are retained.

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`. Update README and repository guide. Commit the completed change without pushing.

## Results

Five new subprocess tests failed before implementation. After implementation, corrected path assertions to allow macOS cwd path normalization and moved unit tests to the end of the module as required by Clippy. All 31 tests now pass on macOS, including seven new tests. Formatting, strict Clippy, and whitespace checks pass. README and repository guide describe startup instruction loading.

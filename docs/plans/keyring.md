# System keyring credentials

## Checklist

- [x] Add failing tests, implement the credential API, document it, and validate.

## Design

Continue on `main` as requested for earlier work. Preserve and include Doug's existing keyring 4.2 dependency and lockfile changes. This change adds a library API, not credential CLI commands or LLM integration.

Add `rh::credentials::Credential` in `src/credentials.rs`. Use a fixed service name `rh`; callers supply a stable account name such as `openai/api-key`. `new(account)` rejects blank names before accessing the OS, then constructs a keyring Entry. `set(&str)` stores or replaces a password; `get()` returns `Result<Option<String>>` with missing entries represented by None; `delete()` succeeds even if the entry is missing. Propagate other keyring errors so callers can distinguish locked or unavailable storage. Do not log credentials or derive Debug for the wrapper. Never fall back to plaintext or an in-memory production store. Normal chat startup must not open the keyring.

Keep keyring's default v1 feature, which selects macOS Keychain, Windows Credential Manager, and Secret Service on other Unix platforms. Document that Linux needs a working session D-Bus and Secret Service provider. Set the package minimum Rust version to 1.88, as required by keyring 4.2.

## Tests and documentation

The existing 31 tests pass; Cargo currently warns that the new keyring dependency is unused. Add tests before implementation using keyring-core's mock store as a dev-dependency. Inject mock entries directly without changing global defaults or initializing the native store. Cover missing entries, set/get, replacement, independent accounts, deletion, blank account validation, and storage error propagation. Tests must not access Doug's system keyring.

Document the public API with a no-run example and note that retrieved strings and native error objects must be handled as sensitive data. Update README and AGENTS.md. Validate with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`, then commit without pushing.

## Results

The new tests initially failed to compile because the credential API did not exist; corrected the mock trait import during this step. All five credential tests now pass using local mock entries. Full validation passes on macOS: 36 tests plus one compiled no-run documentation example, formatting, strict Clippy, and whitespace checks. Native keyring operations and Windows/Linux builds were not tested; no real credentials were read or written. The declared Rust 1.88 minimum follows keyring's requirement and was not tested with that older toolchain.

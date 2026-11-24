# Project guidance

This repository uses Anchor Book at the full revision pinned in Cargo.toml and
Cargo.lock. Run `scripts/setup-book-tools.sh` after an engine upgrade or when the
project-local tools are missing. Add `target/book-tools/bin` to PATH, then read
`anchorbook contract --config anchorbook.toml`,
then `book/working-on-taxudromeio.md`, `anchorbook.toml`, and all chapters in
`book/SUMMARY.md`. Project phase is authoritative in anchorbook.toml.

Run `anchorbook check --config anchorbook.toml` when starting
work and after Book edits. Tests validate records and roadmap references too.

For implementation changes run `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings`, and
`cargo test --locked --all-targets`. Book changes also require `mdbook build` and
`cargo test --locked --test book -- --ignored`. Run
`anchorbook verify --config anchorbook.toml` after changing evidence or criteria.
The setup and upgrade procedure is in `book/working-on-taxudromeio.md`.

Credentials and login names belong only in Keychain. Never add password CLI
arguments, environment overrides, credential logging, SMTP, unrestricted IMAP
commands, or folder-scope bypasses. Every operation must enforce its policy on
the selected configuration. Do not access real mail or Keychain during ordinary
tests. Tests use fake credential stores and deterministic protocol fixtures.

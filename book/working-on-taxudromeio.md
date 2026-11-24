# Working on Taxudromeio

Use `scripts/setup-book-tools.sh` from the repository root to install the native
Anchor Book CLI at the full revision selected by Cargo.lock, mdBook 0.5.4,
mdbook-mermaid 0.17.1, and cargo-nextest 0.9.151 into `target/book-tools`.
The script installs tools and diagram assets; it does not dispatch Book operations.
Then set the current shell's path:

```sh
export PATH="$PWD/target/book-tools/bin:$PATH"
anchorbook contract --config anchorbook.toml
```

Read that contract and every chapter in SUMMARY.md. The phase is defined only in
anchorbook.toml. The authoring references for this pin are
[anchor definitions](https://github.com/anchor-book/anchor-book/blob/02945a9990de3481144edc8cd47808cdf231c135/book/design/anchor-definitions.md),
[authority definitions](https://github.com/anchor-book/anchor-book/blob/02945a9990de3481144edc8cd47808cdf231c135/book/design/authority-definitions.md),
and [roadmap conventions](https://github.com/anchor-book/anchor-book/blob/02945a9990de3481144edc8cd47808cdf231c135/book/implementation-roadmaps/index.md).

To upgrade, select a merged Anchor Book revision, update the full `rev` in
Cargo.toml, run `cargo update -p anchor-book`, and rerun the setup script. Update
these versioned authoring references and follow the engine's migration guidance.
The installer derives its revision from Cargo.lock, so there is no separate CLI
pin to maintain. Run the checks below and the installed-tool acceptance test,
which compares CLI revision and contract with the locked library and verifies the
actual build rejects invalid Book input. Keep missing evidence visible during an
upgrade; changing a verification method does not create evidence.

Sources are in src/, integration tests in tests/, and the Book in book/.
Generated output is under target/ and must not be committed.
Use Rust stable with edition 2024. This prototype is tested on macOS with Rust
1.98.1. The upstream imap-proto 0.10.2 emits a future-incompatibility warning
about expression macros; the current build passes, and compiler upgrades must
reassess that dependency.

Commands:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
anchorbook check --config anchorbook.toml
mdbook build
cargo test --locked --test book -- --ignored
anchorbook verify --config anchorbook.toml
cargo install --path . --locked
```

`mdbook build` invokes the installed native engine: structural checking gates
record and roadmap rendering, followed by Mermaid. The installed-tool test checks
both a missing role and an invalid roadmap criterion, so a direct renderer call
cannot silently bypass the gate. Rust tests also validate the Book's records and
roadmaps through the same pinned library.
The engine is a development dependency; the installed mail tool does not
depend on it at runtime.

Ordinary tests never open Keychain or connect to a real mailbox. Fake stores
exercise persistence failures; protocol fixtures exercise actual IMAP commands;
MCP tests exercise initialization, tool discovery, and dispatch. Parsed CLI and
MCP adapter tests compare every supported operation through one fake backend,
and subprocess tests check help examples, configuration output and policy denial.
Authentication
tests use localhost TCP/TLS listeners and a private client trust root; they
need loopback socket permission. The TLS leaf fixture expires in November 2027;
regenerate it with `python3 scripts/regenerate_tls_fixtures.py` using OpenSSL 3.
Manual account
verification uses `taxudromeio auth` and performs no mailbox mutations.
Do not claim real-provider or Keychain acceptance without recording that run.
Profile tests use synthetic XML/binary/CMS data with fake credentials. A
user-authorized inspection may parse a supplied profile without authentication;
report only nonsecret connection settings and credential-presence booleans.
Never copy personal profiles or their decoded contents into the repository.

This scoped prototype uses tests and code review rather than formal proofs.
No proof criteria are declared. `[verification]` connects the existing source
links to nextest. Every criterion must have current passing evidence before its
anchor is satisfied. The native-login criterion requires an executable macOS
fixture and remains unlinked; a human/provider acceptance run cannot satisfy it.
The discovery-output obligation is separate from configuration-file protection.
Verification is an explicit command; Book build does not run tests or claim to
render a current verification report.

CI runs ordinary checks, installed-tool acceptance, and verification on Linux and
macOS using the locked engine. It does not access real accounts or Keychain.
The Linux developer setup additionally needs OpenSSL development headers and
pkg-config; Linux checks exercise portable fixtures, not a supported native
credential store. Mutation tooling remains unconfigured;
adopting a Building workflow requires agreeing on that testing policy first. Local checks do
not establish provider interoperability, crash-atomic cross-store writes, or
MCP cancellation of an operation already accepted by a mail server.

Do not commit, publish, or access actual mailbox data or credentials as part of
development checks. The reference repository was read for behavior and patterns;
its private local contents are not part of this project.

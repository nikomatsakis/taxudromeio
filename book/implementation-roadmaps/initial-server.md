# Initial server

**Roadmap identifier:** `initial-server`

## Goal

Deliver an installable local Rust CLI and MCP server with independently scoped IMAP
configurations, native macOS credentials, and filtered triage tools.
This is a prototype assignment from the user's initial request.

## Milestone 1. A scoped triage server is installable and verified {#initial-server-1}

**Outcome:** A local MCP client can list configurations and triage their scoped mail; the installed binary offers interactive Keychain authentication.

**Status:** In progress

**Identifier:** `initial-server-1`

**Dependencies:** None for the assigned prototype. Real-provider acceptance requires an operator's account.

**Design basis:** [Access](../design/access.md), [authentication](../design/authentication.md), and [requirements](../authority/requirements.md).

**Tasks**

- [x] Implement independent TOML configurations and Keychain credential storage.
- [x] Implement interactive auth and local stdio MCP.
- [x] Implement typed policy, folder confinement, UID checks, reads, moves, copies, flags and draft appends.
- [x] Run deterministic unit, CLI, MCP and IMAP boundary tests.
- [x] Validate and render the Book using the pinned Anchor Book engine.
- [ ] Implement and run the isolated, executable native Keychain/authentication fixture.
- [ ] Record a real-provider and native Keychain operator acceptance run.

### Completion criteria

#### Scoped triage {#initial-server-1-scoped-triage}

The supported operations and per-configuration boundaries pass deterministic tests through production adapters.

- **Anchor coverage:** `A-policy-triage` · `operation-filter`, `direct-call-enforcement`, `draft-only` — Full
- **Anchor coverage:** `A-folder-confined` · `root-boundaries`, `wire-boundaries` — Full
- **Anchor coverage:** `A-message-stable` · `stale-identifiers`, `wire-safety` — Full
- **Anchor coverage:** `A-config-independent` · `same-server` — Full
- **Evidence:** Passed on 2026-11-17: 22 unit tests and four CLI/MCP integration tests exercise policy narrowing, same-host IDs, root boundaries, stale UIDVALIDITY, header injection, modified UTF-7, read-only EXAMINE/BODY.PEEK, UID MOVE, copy and flag commands, draft APPEND flags, and rejection before backend access. Protocol fixtures verify no CLOSE, EXPUNGE or deletion fallback. Book validation adds one test, for 27 total.

#### Credential lifecycle {#initial-server-1-credential-lifecycle}

Login and credentials are stored only in Keychain; configuration-save failure restores previous state, and real macOS authentication is observed.

- **Anchor coverage:** `A-auth-keychain` · `no-file-secrets`, `discovery-redaction`, `save-failure`, `native-login` — Full
- **Evidence:** Partial on 2026-11-17: TOML rejection, nonsecret CLI/discovery output, and fake-store rollback tests passed. Native Security Framework integration compiled on macOS. The native-login review has not run; ordinary tests do not access real Keychain or accounts.

#### Development checks {#initial-server-1-development-checks}

The binary builds, CLI help works, formatting and Clippy pass, and the Book's structure, records and roadmaps validate and render.

- **Anchor coverage:** None — contributor tooling and packaging.
- **Evidence:** Passed on 2026-11-17: formatting, all-target Clippy with warnings denied, 27 tests, Anchor Book structural/v1 check, record and roadmap validation, and mdBook 0.5.4 rendering. A release installation into a temporary directory succeeded. The installed binary passed a real stdio subprocess smoke check for initialization, tool discovery, selected configuration discovery, send denial and clean EOF shutdown.

### Evidence {#initial-server-1-evidence}

**Revision:** Uncommitted initial working tree on main, with no baseline commit.
Code and design were checked together on 2026-11-17 using Rust 1.98.1 and the
Anchor Book revision pinned in Cargo.toml. The final evidence edit changes only
this roadmap's progress and findings.

**Checks:** `cargo fmt --check`, `cargo test --locked --all-targets`,
`cargo clippy --locked --all-targets -- -D warnings`,
`cargo run --quiet --locked --example anchorbook -- check`, and `mdbook build`.
Release installation used `cargo install --path . --locked --offline --root`
with a temporary destination, followed by a subprocess MCP smoke check.

**Results:** Local checks and release smoke passed. Rendered anchor records and
criterion links were inspected. CI, mutation testing, native Keychain and
real-provider acceptance have not run. No mailbox credentials were supplied.
The upstream imap-proto 0.10.2 dependency emits a macro future-incompatibility
warning; it builds successfully with the tested compiler, but a future Rust
upgrade requires reassessment.

### Completion record

Pending executable native Keychain verification and real-provider operator
acceptance in addition to local checks. Milestone 10 migrates `native-login` from
review to test without claiming that either acceptance has run. It separates
`discovery-redaction` from `no-file-secrets` and adds the missing source links;
this preserves the credential-protection obligations in the design.

## Milestone 2. IMAP encryption and OAuth renewal support are verified {#initial-server-2}

**Outcome:** Configurations support required STARTTLS and renewable OAuth logins, with failed authentication stopping before mailbox operations.

**Status:** Complete

**Identifier:** `initial-server-2`

**Dependencies:** The local server in milestone 1. Live-provider interoperability remains part of its pending acceptance run.

**Design basis:** [Authentication](../design/authentication.md), [requirements](../authority/requirements.md), and [decisions](../authority/decisions.md). Assigned by the user's 2026-11-17 annotation requesting STARTTLS and automatic OAuth refresh.

**Tasks**

- [x] Implement explicit TLS/STARTTLS configuration and mandatory encrypted login.
- [x] Extend Keychain records and interactive auth with renewable OAuth grants.
- [x] Refresh before expiry and retry rejected authentication once before operations.
- [x] Preserve rotated grants before use and across reauthentication failures.
- [x] Add deterministic TLS, HTTPS, concurrency and failure fixtures.
- [x] Run the complete development checks and record evidence.

### Completion criteria

#### Encrypted IMAP {#initial-server-2-encrypted-imap}

STARTTLS establishes verified TLS before password or XOAUTH2 login and never falls back after upgrade or certificate failures.

- **Anchor coverage:** `A-auth-encrypted` · `transport-settings`, `upgrade-before-login`, `no-downgrade`, `certificate-verification` — Full
- **Evidence:** Passed on 2026-11-17: transport default and compatibility tests; real loopback STARTTLS handshakes for password and XOAUTH2; refused and unsupported upgrades; untrusted certificate and hostname mismatch rejection with no login bytes.

#### Renewable credentials {#initial-server-2-renewable-credentials}

Automatic refresh exchanges grants over HTTPS, preserves rotation in Keychain, reuses fresh tokens, and retries authentication without replaying mailbox operations.

- **Anchor coverage:** `A-oauth-renewable` · `provider-exchange`, `refresh-lifecycle`, `authentication-retry`, `failure-safety`, `legacy-records` — Full
- **Evidence:** Passed on 2026-11-17: HTTPS refresh exchanges for public and secret-bearing clients, expiry and token rotation, concurrent refresh sharing, reauthentication rollback preserving renewal, one authentication retry, failure before IMAP access, redirect rejection, bounded and redacted responses, and legacy Keychain record parsing. Stores and provider grants are fixtures.

#### Updated development checks {#initial-server-2-development-checks}

Formatting, Clippy, all tests, pinned Book checks, Book rendering, and the installed CLI pass with the new authentication behavior.

- **Anchor coverage:** None — contributor tooling and packaging.
- **Evidence:** Passed on 2026-11-17: formatting, all-target Clippy with warnings denied, 36 unit tests plus one Book test and four CLI/MCP integration tests (41 total), pinned Anchor Book structural check and record/roadmap validation, and mdBook rendering. Temporary release installation passed CLI and stdio MCP smoke checks.

### Evidence {#initial-server-2-evidence}

**Revision:** Uncommitted working tree on main, with no baseline commit.

**Checks:** `cargo fmt --check`, `cargo clippy --locked --all-targets --offline -- -D warnings`, `cargo test --locked --all-targets --offline`, `cargo run --quiet --locked --offline --example anchorbook -- check`, and `mdbook build`. Socket tests ran with loopback permission because the workspace sandbox blocks listeners. `cargo install --path . --locked --offline --force --root /private/tmp/taxudromeio-install` succeeded. The installed binary passed auth-help, TLS/STARTTLS default-port, stdio initialization, tool discovery, configuration subset, send-denial and EOF-shutdown checks.

**Results:** All local checks passed. Tests use fake credentials and loopback
services; no real-provider or native Keychain acceptance is claimed.
Browser consent and provider registration are external to the refresh-grant
setup. Refresh serialization covers one process and configuration; sharing a
rotating grant across IDs or concurrent processes requires coordination.
The preexisting imap-proto future-incompatibility warning remains.
Only this roadmap's evidence and completion fields changed after those checks;
Book validation and rendering were rerun on that final evidence.

### Completion record

Completed the assigned STARTTLS and automatic refresh implementation with
deterministic local evidence on 2026-11-17. Native Keychain and live-provider
acceptance remain recorded in milestone 1.

## Milestone 3. Executable help explains setup and client configuration {#initial-server-3}

**Outcome:** A user can follow --help to authenticate and configure a local MCP client, including copying valid JSON with the executable's path.

**Status:** Complete

**Identifier:** `initial-server-3`

**Dependencies:** The CLI and authentication implemented in milestones 1 and 2.

**Design basis:** [Authentication](../design/authentication.md) and [requirements](../authority/requirements.md). Assigned by the user's 2026-11-17 setup-help request.

**Tasks**

- [x] Add setup instructions and executable-specific client JSON to top-level help.
- [x] Explain optional authentication, selected configurations and custom settings.
- [x] Keep short help compact and add auth/serve setup details.
- [x] Extend CLI verification to parse and execute the help's JSON example.
- [x] Run development and Book checks and record evidence.

### Completion criteria

#### Setup guidance {#initial-server-3-setup-guidance}

The executable explains authentication and offers usable client JSON without needing saved credentials or settings to display help.

- **Anchor coverage:** `A-auth-setup-help` · `setup-instructions`, `client-json`, `command-details` — Full
- **Evidence:** Passed on 2026-11-17: actual CLI subprocess help includes authentication, Keychain storage and client verification instructions. The integration test extracts and parses the displayed JSON, checks its absolute executable path and serve arguments, runs that command with --help, and checks compact -h and expanded auth/serve guidance. The rendered top-level output was inspected.

#### Development checks {#initial-server-3-development-checks}

Formatting, Clippy, tests, Book validation and rendering pass.

- **Anchor coverage:** None — contributor tooling.
- **Evidence:** Passed on 2026-11-17: formatting, all-target Clippy with warnings denied, 41 tests, pinned Anchor Book structural and record/roadmap checks, mdBook rendering, and whitespace validation.

### Evidence {#initial-server-3-evidence}

**Revision:** Working tree based on commit efdf087.

**Checks:** `cargo fmt --check`, `cargo clippy --locked --all-targets --offline -- -D warnings`, `cargo test --locked --all-targets --offline`, `cargo run --quiet --locked --offline --example anchorbook -- check`, `mdbook build`, and `git diff --check`. Actual help was inspected with `cargo run --quiet --locked --offline -- --help`. Socket permission was used only for the existing deterministic loopback authentication fixtures; no real mail or Keychain was accessed.

**Results:** All checks passed. Client JSON uses the executable generating the
help and needs an updated command path if that binary is moved. The existing
imap-proto future-incompatibility warning remains. Only this roadmap's progress
and evidence fields changed after the implementation checks; final Book
validation and rendering were rerun.

### Completion record

Completed the requested setup-help implementation with local evidence on
2026-11-17. Native provider acceptance remains part of milestone 1.

## Milestone 4. Apple profiles can supply mail setup {#initial-server-4}

**Outcome:** A user can run auth with a mobileconfig file to supply IMAP settings and credentials while choosing local scope and policy.

**Status:** Complete

**Identifier:** `initial-server-4`

**Dependencies:** The verified authentication and Keychain flow in milestones 1 and 2.

**Design basis:** [Authentication](../design/authentication.md) and [requirements](../authority/requirements.md). Assigned by the user's 2026-11-18 request to configure from an Apple profile.

**Tasks**

- [x] Inspect the supplied profile without disclosing credentials or installing it.
- [x] Decode plain XML/binary and DER CMS profile data in Rust.
- [x] Import only IMAP setup and incoming credentials into auth.
- [x] Preserve encrypted login, Keychain storage, and local scope/policy choices.
- [x] Support selecting among IMAP payloads and explicit connection overrides.
- [x] Add synthetic profile fixtures, failure tests, help and documentation.
- [x] Run development checks and record evidence.

### Completion criteria

#### Profile setup {#initial-server-4-profile-setup}

Supported profiles supply IMAP setup without importing other services or permissions.

- **Anchor coverage:** `A-auth-profile-import` · `profile-formats`, `encrypted-connection`, `account-selection`, `local-boundaries`, `failure-safety` — Full
- **Evidence:** Passed on 2026-11-18: XML/binary and signed synthetic CMS extraction, ignored unrelated payloads, optional credential handling, root-prefix hints, encrypted transport selection, repeated-server account selection, explicit overrides, fake-Keychain persistence without TOML credentials, and redacted malformed/oversized input failures. CLI tests verify the account-selector flag dependency and setup help.

#### Development checks {#initial-server-4-development-checks}

Formatting, Clippy, tests, Book validation/rendering and the actual file's read-only parser inspection pass.

- **Anchor coverage:** None — contributor tooling and user-authorized file inspection.
- **Evidence:** Passed on 2026-11-18: formatting, all-target Clippy with warnings denied, 43 unit tests plus one Book test, three CLI tests and two MCP tests (49 total), pinned Anchor Book structural and record/roadmap validation, mdBook rendering and whitespace validation. Production parser inspection of the supplied file reported its expected IMAP connection and credential presence.

### Evidence {#initial-server-4-evidence}

**Revision:** Working tree based on efdf087, preserving the preceding setup-help changes.

**Checks:** `cargo fmt --check`, `cargo clippy --locked --all-targets --offline -- -D warnings`, `cargo test --locked --all-targets --offline`, `cargo run --quiet --locked --offline --example anchorbook -- check`, `mdbook build`, and `git diff --check`. The actual supplied profile was parsed once
through the production library using a temporary inspection utility. It
reported one IMAP account using TLS on port 993, with login and password
present. Credential values were not displayed; no mailbox authentication,
Keychain access, profile installation or signature trust verification occurred.
The personal file and decoded contents are not test fixtures or project files.

**Results:** All local checks passed. Input is limited to XML/binary property
lists and DER CMS SignedData, password-based IMAP accounts, 1 MiB and 128
payloads. Signatures are not verified and the profile is not installed.
Interactive auth and a user-chosen root are still required to save verified
credentials in native Keychain. Native provider acceptance remains pending
in milestone 1. Only this roadmap's evidence and status changed after
implementation validation; Book checks and rendering were rerun.

### Completion record

Completed profile-based setup with deterministic tests and authorized read-only
inspection on 2026-11-18. No real mailbox or Keychain operation was performed.

## Milestone 5. The renamed tool supports shared CLI mail operations {#initial-server-5}

**Outcome:** taxudromeio offers Clap-parsed mail commands sharing MCP execution, and --help teaches an agent to use both interfaces.

**Status:** Complete

**Identifier:** `initial-server-5`

**Dependencies:** The scoped server, authentication and profile setup from milestones 1–4.

**Design basis:** [Access](../design/access.md), [authentication](../design/authentication.md), [requirements](../authority/requirements.md) and [decisions](../authority/decisions.md). Assigned by the user's 2026-11-18 CLI and rename request.

**Tasks**

- [x] Rename the Cargo package, executable, MCP identity and user instructions.
- [x] Preserve the existing settings directory and Keychain service identity.
- [x] Extract shared discovery, authorization and operation execution.
- [x] Add Clap commands using the same typed arguments as MCP.
- [x] Explain all mail commands and the LLM workflow in executable help.
- [x] Verify adapter parity, policy failures, parsing and usable help examples.
- [x] Run development and Book checks and record evidence.

### Completion criteria

#### Shared operations {#initial-server-5-shared-operations}

CLI mail commands and MCP tools share typed requests, results and enforcement.

- **Anchor coverage:** `A-interface-shared` · `adapter-parity`, `policy-boundary`, `clap-boundary` — Full
- **Anchor coverage:** `A-policy-triage` · `direct-call-enforcement` — Full
- **Evidence:** Passed on 2026-11-18: all nine CLI operations and equivalent MCP calls deliver matching typed arguments and JSON through the same fake backend. Coverage includes defaults, pagination, every search field, booleans, Unicode and multiline text, repeated recipients, UIDs and flags. Read-only, empty allowlists and missing configurations fail before backend access. Actual CLI subprocesses refuse a read-only mutation and an unknown ID before Keychain access. Invalid/unsupported Clap arguments and oversized aggregate UID/flag lists are refused.

#### Renamed agent guidance {#initial-server-5-agent-guidance}

Help teaches the CLI workflow and preserves usable setup JSON after the rename.

- **Anchor coverage:** `A-cli-agent-help` · `workflow-help`, `renamed-interface` — Full
- **Anchor coverage:** `A-auth-setup-help` · `setup-instructions`, `client-json`, `command-details` — Full
- **Evidence:** Passed on 2026-11-18: actual help teaches discovery, pagination, UIDVALIDITY, JSON output and mutation handling; all nine printed examples parse with Clap and every nested command help exits without settings or credentials. Setup JSON references the running taxudromeio executable and successfully invokes serve help. MCP reports the renamed identity. The existing settings path is checked, prior settings fixtures load, and inspection confirms the Keychain service remains io.taxudromeio.mcp.imap.

#### Development checks {#initial-server-5-development-checks}

Formatting, Clippy, all tests, pinned Book validation and rendering pass.

- **Anchor coverage:** None — contributor tooling.
- **Evidence:** Passed on 2026-11-18: formatting, all-target Clippy with warnings denied, 43 unit tests plus one Book test, five CLI tests, two MCP tests and three shared-operation tests (54 total), pinned Book structural/record/roadmap checks, mdBook rendering and whitespace validation. Temporary release installation succeeded; its actual stdio discovery exposes both synthetic configurations for default invocation and plain serve, and only the selected ID with --configuration.

### Evidence {#initial-server-5-evidence}

**Revision:** Working tree based on efdf087, preserving preceding help and profile-import changes.

**Checks:** `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
`cargo test --locked --all-targets`,
`cargo run --quiet --locked --example anchorbook -- check`, `mdbook build`
and `git diff --check`. Socket tests used loopback permission. Release packaging
used `cargo install --path . --locked --offline --force --root /private/tmp/taxudromeio-cli-install`.
The installed binary's top-level and all nine nested command helps, discovery
JSON, stdio initialization and selected/all-configuration discovery were checked
with synthetic settings. No real mail or Keychain was accessed.

**Results:** All local checks passed. An initial boundary test caught that Clap's
token count did not bound comma-expanded UID lists. The shared request validator
now rejects oversized aggregate UID and flag lists before backend/credential
access; both adapters use it, and regression checks pass. CLI argument decoding
is tested with a fake backend; the shared production IMAP backend retains its
deterministic wire/TLS tests. Native Keychain and real-provider acceptance remain
pending in milestone 1. Settings and credential storage identities are preserved;
the installed executable and MCP client command now use taxudromeio.
The preexisting imap-proto future-incompatibility warning remains.
Only Book progress/evidence and descriptions changed after implementation
validation; final Book checks and rendering were rerun.

### Completion record

Completed the assigned CLI and rename work with local deterministic evidence on
2026-11-18. Live-provider and native Keychain acceptance remain part of milestone 1.

## Milestone 6. Folder names remain single IMAP arguments {#initial-server-6}

**Outcome:** Authentication can discover a root containing spaces and colons, and shared mailbox operations quote their destinations correctly.

**Status:** Complete

**Identifier:** `initial-server-6`

**Dependencies:** The shared IMAP backend from milestones 1–5.

**Design basis:** [Access](../design/access.md) and [authentication](../design/authentication.md). Assigned by the user's 2026-11-18 auth failure for a root named INBOX/Sender: Work.

**Tasks**

- [x] Reproduce the unquoted LIST argument in a deterministic wire fixture.
- [x] Correct serialization at LIST, UID COPY and APPEND call boundaries.
- [x] Cover spaces, colons, Unicode and destination quote/backslash escaping.
- [x] Run development and Book checks and record evidence.

### Completion criteria

#### Quoted mailbox arguments {#initial-server-6-quoted-mailboxes}

Mailbox serialization keeps the selected names intact across authentication and shared mail operations.

- **Anchor coverage:** `A-folder-confined` · `mailbox-quoting`, `wire-boundaries` — Full
- **Anchor coverage:** `A-message-stable` · `wire-safety` — Full
- **Evidence:** Passed on 2026-11-18 after the initial regression reproduced an unquoted LIST pattern. Wire fixtures now observe one quoted root with spaces/colons and modified UTF-7 names, unchanged root boundaries, correct root EXAMINE, and scoped UID COPY/MOVE and draft APPEND with escaped quote/backslash destinations. The loopback TLS password-login fixture discovers the exact reported root, INBOX/Sender: Work.

#### Development checks {#initial-server-6-development-checks}

Formatting, Clippy, all tests, pinned Book checks and rendering pass.

- **Anchor coverage:** None — contributor tooling.
- **Evidence:** Passed on 2026-11-18: formatting, all-target Clippy with warnings denied, 45 unit tests plus one Book test, five CLI tests, two MCP tests and three shared-operation tests (56 total), pinned Book checks, record/roadmap validation, mdBook rendering and whitespace validation. The installed release executable was replaced successfully and its help and binary identity were checked.

### Evidence {#initial-server-6-evidence}

**Revision:** Working tree based on efdf087, preserving the preceding profile, help and CLI work.

**Checks:** `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
`cargo test --locked --all-targets`,
`cargo run --quiet --locked --example anchorbook -- check`, `mdbook build`
and `git diff --check`. Socket tests used loopback permission. The regression
`cargo test --locked --lib root_discovery_quotes_spaces_and_colons` initially
failed on the exact unquoted wire command and passes after correction.
`cargo install --path . --locked --offline --force --root /Users/nikomat/.cargo`
replaced the user's installed taxudromeio executable. A SHA-256 comparison confirms
it matches the corrected release build; actual --help succeeds and its JSON
references that installed path.

**Results:** All local checks passed. The imap 2.4.1 APIs serialize mailbox
arguments inconsistently: LIST and UID COPY interpolate names, APPEND surrounds
them without escaping, and SELECT/EXAMINE/UID MOVE quote and escape internally.
The shared adapter now handles each boundary appropriately, preserving scoped
names and modified UTF-7. All fixtures use fake credentials and local services.
No real mailbox or Keychain was accessed; the operator must retry interactive
auth to establish provider acceptance. The preexisting imap-proto warning remains.
Only roadmap evidence/status changed after implementation checks; final Book
checks and rendering were rerun.

### Completion record

Completed the mailbox-quoting correction and installed the tested release
executable on 2026-11-18. Live-provider and native Keychain acceptance remain
part of milestone 1.

## Milestone 7. The executable prints a detailed usage skill {#initial-server-7}

**Outcome:** Users and agents can run taxudromeio skill before setup to learn login, commands, configuration choices and MCP tradeoffs from one Markdown document.

**Status:** Complete

**Identifier:** `initial-server-7`

**Dependencies:** CLI, shared operation service and authentication from milestones 1–6.

**Design basis:** [Access](../design/access.md), [authentication](../design/authentication.md) and [requirements](../authority/requirements.md). Assigned by the user's 2026-11-18 skill subcommand request.

**Tasks**

- [x] Add a Clap skill command that prints embedded Markdown before configuration lookup.
- [x] Document CLI-first login and assisted profile setup without credential disclosure.
- [x] Cover every mail operation, scope, identifiers, multiple configurations and MCP setup.
- [x] Explain persistent-process benefits and current lack of IMAP pooling or measured token savings.
- [x] Verify standalone output and usable shell-quoted examples.
- [x] Run development and Book checks and update the installed executable.

### Completion criteria

#### Usage guide {#initial-server-7-usage-guide}

The executable's standalone guide teaches the requested workflow with usable examples.

- **Anchor coverage:** `A-cli-skill` · `standalone-guide`, `guide-examples` — Full
- **Evidence:** tests/skill.rs verifies output before setup, required guidance and Clap-parsed examples covering every mail operation, shell quoting and valid JSON.

#### Development checks {#initial-server-7-development-checks}

Formatting, Clippy, tests, Book checks/rendering and installed skill output pass.

- **Anchor coverage:** None — contributor tooling and packaging.
- **Evidence:** All required checks passed; the installed release prints the embedded document from an empty directory with a nonexistent settings path.

### Evidence {#initial-server-7-evidence}

**Revision:** Working tree based on efdf087, preserving previous local work.

**Checks:**

- cargo fmt --check — passed.
- cargo clippy --locked --all-targets -- -D warnings — passed.
- cargo test --locked --all-targets — 58 tests passed.
- cargo run --quiet --locked --example anchorbook -- check — passed.
- mdbook build — passed.
- git diff --check — passed.
- cargo install --path . --locked --offline --force --root /Users/nikomat/.cargo — passed.
- Installed release digest matches target/release; skill stdout matches docs/skill.md and help advertises the command — passed.

**Results:** The executable prints a 427-line Markdown guide before configuration
lookup. It teaches CLI-first login, agent-assisted profile setup in an interactive
terminal, every mail command, folder and UID boundaries, multiple configurations
and MCP integration. It distinguishes persistent-process and structured-tool
benefits from unmeasured overall latency or token savings; IMAP connections are
still opened per operation.

Guide examples use synthetic addresses and credential-profile paths, and tests
do not run real authentication. No real mail or Keychain was accessed. The
preexisting imap-proto future-compatibility warning remains. Only roadmap
evidence/status changed after implementation checks; final Book checks and
rendering were rerun.

### Completion record

Completed the detailed usage guide and installed the tested release executable
on 2026-11-18. Live-provider and native Keychain acceptance remain part of
milestone 1; no performance benchmark was added.

## Milestone 8. Help is compact and points to the skill guide {#initial-server-8}

**Outcome:** Command help provides a concise reference, while skill holds detailed setup, mail workflows and MCP JSON.

**Status:** Complete

**Identifier:** `initial-server-8`

**Dependencies:** The CLI and standalone skill guide from milestones 5 and 7.

**Design basis:** [Access](../design/access.md), [authentication](../design/authentication.md) and [requirements](../authority/requirements.md). Assigned by the user's 2026-11-18 request to shorten help given that skill exists.

**Tasks**

- [x] Remove long top-level and auth/serve/mail help appendices.
- [x] Retain command lists, flags, defaults and brief operational notes.
- [x] Direct help callers to skill and correct guide/documentation references.
- [x] Verify compact help, command reference and guide examples/client JSON.
- [x] Run development and Book checks and update the installed executable.

### Completion criteria

#### Help and guide {#initial-server-8-help-guide}

Help is compact without losing command argument reference or discoverability of detailed setup.

- **Anchor coverage:** `A-auth-setup-help` · `setup-instructions`, `client-json`, `command-details` — Full
- **Anchor coverage:** `A-cli-agent-help` · `workflow-help`, `renamed-interface` — Full
- **Anchor coverage:** `A-cli-skill` · `standalone-guide`, `guide-examples` — Full
- **Evidence:** CLI subprocess tests check both help forms within a compact output budget, auth/serve/mail argument reference and all nine operation helps without settings or credentials. Skill tests retain every operation example and validate client JSON/serve arguments.

#### Development checks {#initial-server-8-development-checks}

Formatting, Clippy, tests, Book checks/rendering and installed help/skill output pass.

- **Anchor coverage:** None — contributor tooling and packaging.
- **Evidence:** All required checks passed. Installed -h and --help each produce 21 lines; the installed release matches the build and skill stdout matches the revised source.

### Evidence {#initial-server-8-evidence}

**Revision:** Working tree based on f60511c.

**Checks:**

- cargo fmt --check — passed.
- cargo clippy --locked --all-targets -- -D warnings — passed.
- cargo test --locked --all-targets — 58 tests passed.
- cargo run --quiet --locked --example anchorbook -- check — passed.
- mdbook build — passed.
- git diff --check — passed.
- cargo install --path . --locked --offline --force --root /Users/nikomat/.cargo — passed.
- Installed release digest, compact help and skill output checked with nonexistent settings from an empty directory — passed.

**Results:** Top-level help now has a command overview, flags, one authentication
starting point and pointers to skill and command help. The long setup, workflow
and JSON appendices were removed from help; nested commands retain argument
reference and brief operational notes. The guide retains detailed setup and MCP
JSON, with instructions to find the installed executable's absolute path.

This requested refinement supersedes the expanded help
layout recorded in milestones 3 and 5. Their evidence describes the historical
implementation; detailed guidance and client JSON now belong to skill.
No real mail or Keychain was accessed. The preexisting imap-proto
future-compatibility warning remains. Only roadmap evidence/status changed after
implementation checks; final Book checks and rendering were rerun.

### Completion record

Completed compact help and installed the tested release executable on
2026-11-18. Authentication, policy and mail operation behavior are unchanged.

## Milestone 9. MCP client JSON is generated by a command {#initial-server-9}

**Outcome:** Users can run taxudromeio mcp --json to generate client launch configuration for the actual executable and optional selected settings.

**Status:** Complete

**Identifier:** `initial-server-9`

**Dependencies:** Compact help and the standalone skill guide from milestones 7–8.

**Design basis:** [Authentication](../design/authentication.md) and [requirements](../authority/requirements.md). Assigned by the user's 2026-11-18 request for MCP client JSON output, followed by a request to document use and configuration in the README.

**Tasks**

- [x] Add a Clap mcp command with --json and configuration selection.
- [x] Generate absolute executable/settings paths and serve arguments before account lookup.
- [x] Route mcp without --json through the existing serve startup path.
- [x] Document the generator in compact help, the guide and README.
- [x] Expand README onboarding with installation, login, profile import, roots, policies, multiple IDs, custom settings, CLI operations and MCP setup.
- [x] Verify standalone JSON and usable launch options without credentials.
- [x] Run development and Book checks and update the installed executable.

### Completion criteria

#### Client JSON {#initial-server-9-client-json}

Actual command output describes the executable and selected launch options without account setup.

- **Anchor coverage:** `A-mcp-client-json` · `standalone-json`, `launch-options` — Full
- **Anchor coverage:** `A-cli-skill` · `standalone-guide`, `guide-examples` — Full
- **Evidence:** CLI subprocess tests parse actual output from an empty directory, verify the executable and default/custom launch arguments, and execute those arguments with serve help. Relative nonexistent paths with spaces, quotes and Unicode round-trip through JSON. Guide examples and mcp help pass.

#### Development checks {#initial-server-9-development-checks}

Formatting, Clippy, tests, Book checks/rendering and installed JSON output pass.

- **Anchor coverage:** None — contributor tooling and packaging.
- **Evidence:** All required checks passed. Installed output uses the installed executable's absolute path and selected launch options; the release digest and guide match the build sources.

### Evidence {#initial-server-9-evidence}

**Revision:** Working tree based on 5580d7b.

**Checks:**

- cargo fmt --check — passed.
- cargo clippy --locked --all-targets -- -D warnings — passed.
- cargo test --locked --all-targets — 59 tests passed.
- cargo run --quiet --locked --example anchorbook -- check — passed.
- mdbook build — passed.
- git diff --check — passed.
- cargo install --path . --locked --offline --force --root /Users/nikomat/.cargo — passed.
- Installed release digest, default/custom JSON output, compact help and skill source checked — passed.
- README validation parsed 27 published commands through Clap/shared request validation, one settings TOML through Config validation, and one client JSON through serde/Clap — passed without executing authentication or mail operations.

**Results:** mcp --json prints a pretty-printed mcpServers entry with the current
executable's absolute path and serve arguments. Explicit settings paths become
absolute without requiring the file to exist; repeated configuration IDs remain
separate arguments. The output contains no credentials or environment overrides.
JSON generation happens before settings or credential access. Settings and
selected IDs are validated when the client starts the server. mcp without --json
uses the existing serve branch, including configuration selection and policy.
The README now gives a complete onboarding sequence and a configuration-first
reference, including the quoted INBOX/Sender: Work root, imported credentials,
TLS/OAuth prerequisites, TOML allowlists and generated client JSON.

No real mail or Keychain was accessed. The preexisting imap-proto
future-compatibility warning remains. Only roadmap evidence/status changed after
implementation checks; final Book checks and rendering were rerun.

### Completion record

Completed the client JSON command and installed the tested release executable
on 2026-11-18.


## Milestone 10. Book tools use the pinned engine and evaluate its criteria {#initial-server-10}

**Outcome:** Contributors can install the locked Anchor Book revision, retrieve its configured contract, check and render the Book through the native CLI, and obtain current criterion evidence without maintaining copied engine dispatch.

**Status:** Complete

**Identifier:** `initial-server-10`

**Dependencies:** Existing Book and deterministic tests. Native Keychain and provider acceptance remain in milestone 1 and do not block tooling integration.

**Design basis:** [Credential protection](../design/authentication.md#A-auth-keychain), [configuration isolation](../design/authentication.md#A-config-independent), and [project tool and upgrade guidance](../working-on-taxudromeio.md).

**Tasks**

- [x] Update the engine dependency and use its installed commands for configured contract retrieval, checking, rendering, and verification.
- [x] Derive tool installation from Cargo.lock and document the upgrade procedure and versioned authoring references.
- [x] Configure Mermaid and add an installed-tool regression test for the build gate and roadmap rendering.
- [x] Configure nextest evidence, separate discovery protection, and associate its existing output checks and configuration-subset checks.
- [x] Replace the native-login review method with an explicit executable obligation, leaving its missing evidence visible.
- [x] Make public TLS fixture packaging and platform-specific imports usable for Linux and macOS validation.
- [x] Validate the final change locally and through CI on both hosts.

### Completion criteria

#### Native Book tooling {#initial-server-10-tooling}

Installed CLI revision and configured contract agree with the locked library.
Actual Book builds render roadmap milestones and Mermaid, and fail for a missing
required role or an unknown criterion in roadmap coverage. Repeating setup at
the same pin preserves project sources.

- **Anchor coverage:** None — consumer tooling, exercised by the installed-tool acceptance test.
- **Evidence:** Passed locally on 2026-11-24, branch `book-assessment` based on `7cf0f7e` plus this change, using Rust 1.99.1 and Anchor Book `02945a9990de3481144edc8cd47808cdf231c135`. The installed-tool test matches CLI revision/contract to the locked library, renders all milestones and Mermaid, and rejects both a missing Design role and an unknown roadmap criterion. Repeating setup preserves project sources. Browser inspection covers light, navy, narrow, and print layouts with no script errors.

#### Explicit criterion evidence {#initial-server-10-evidence}

Configured verification runs the linked tests and exposes the missing native-login
fixture. Configuration-file and discovery-output protection retain independent
coverage, with CLI/MCP output fields asserted explicitly. Unknown selected IDs
remain covered through the actual CLI.

- **Anchor coverage:** `A-auth-keychain` · `no-file-secrets`, `discovery-redaction` — Full; `A-config-independent` · `same-server` — Full.
- **Evidence:** Configured nextest verification passes locally with no findings: 12 anchors satisfied and one partial; 38 of 39 criteria satisfied. `native-login` remains unlinked and is deliberately outside this milestone's completion claim; its executable integration remains outstanding in milestone 1. Configuration persistence and both CLI/MCP discovery assertions are linked to their respective obligations. The existing unknown-subset CLI assertions now contribute to `same-server`.

#### Reproducible checks {#initial-server-10-checks}

Formatting, Clippy, ordinary tests, Book build, installed-tool acceptance, and
configured verification pass on Linux and macOS using the same engine pin.
Tests use synthetic credentials and loopback fixtures; no native Keychain or
real-provider acceptance is inferred from these runs.

- **Anchor coverage:** None — contributor setup and CI portability.
- **Evidence:** Formatting, Clippy with warnings denied, 59 ordinary tests, installed-tool acceptance, Book build, and configured verification pass locally on Linux. OpenSSL development headers are a host prerequisite; no legacy provider override is needed after repackaging the existing public test identity. The preexisting imap-proto future-incompatibility warning remains. [Linux and macOS CI](https://github.com/kwdev/taxudromeio/actions/runs/45453005643) both passed for `88fb4c6431713264bb7fbf8bd3723868090c6e4c`, including the installed-tool gate regression. Both retained verification reports have the same 12 satisfied/one partial anchor and 38 satisfied/one unlinked criterion totals.

### Completion record

Complete for native Book tooling and deterministic verification. Native Keychain
integration and real-provider acceptance remain outstanding in milestone 1.

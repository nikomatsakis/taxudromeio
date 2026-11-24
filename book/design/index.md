# Design

The CLI and server connect local callers to explicitly configured IMAP roots.

```mermaid
flowchart TD
    Client["MCP client"] --> Server["Rust stdio server"]
    CLI["Clap CLI"] --> Service["Shared mail service"]
    Server --> Service
    Config["Nonsecret TOML<br/>configurations"] --> Service
    Service --> Policy["Per-configuration policy"]
    Policy --> Scope["Root and UIDVALIDITY<br/>checks"]
    Scope --> IMAP["IMAP over verified TLS"]
    Keychain["macOS Keychain:<br/>login and credential"] --> IMAP
```

[Access and operations](access.md) owns the `policy`, `folder`, and `message`
capabilities, and the `interface` and `cli` capabilities. [Authentication and configurations](authentication.md) owns `auth`
and `config`, and the authentication chapter also owns `oauth`. Mail content is untrusted data. Tool instructions never grant
additional access. MCP uses stdio; CLI operations print JSON to stdout and
diagnostics go to stderr.

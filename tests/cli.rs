use std::process::Command;

/// anchorbook: A-auth-setup-help#setup-instructions
#[test]
fn top_level_help_is_compact_and_points_to_the_skill_guide() {
    for option in ["-h", "--help"] {
        let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
            .args(["--config", "/nonexistent/taxudromeio-settings", option])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        for command in [
            "skill",
            "auth",
            "mcp",
            "serve",
            "configurations",
            "mail",
            "remove",
        ] {
            assert!(text.contains(command), "{text}");
        }
        assert!(text.contains("Get started: taxudromeio auth"));
        assert!(text.contains("taxudromeio skill"));
        assert!(text.contains("taxudromeio mcp --json"));
        assert!(text.contains("taxudromeio <COMMAND> --help"));
        // Keep a usable command overview rather than restoring the full guide.
        assert!(text.lines().count() <= 35, "{text}");
        assert!(text.len() <= 2_000, "{text}");
        assert!(!text.contains("mcpServers"));
        assert!(!text.contains("next_before_uid"));
    }
}

/// anchorbook: A-mcp-client-json#standalone-json
/// anchorbook: A-mcp-client-json#launch-options
#[test]
fn mcp_json_prints_usable_launch_settings_before_account_setup() {
    let directory = tempfile::tempdir().unwrap();
    let executable = env!("CARGO_BIN_EXE_taxudromeio");
    let settings = directory
        .path()
        .canonicalize()
        .unwrap()
        .join("settings with spaces/quoted \"é\".toml");
    let cases = [
        (vec!["mcp", "--json"], serde_json::json!(["serve"])),
        (
            vec![
                "mcp",
                "--json",
                "--config",
                "settings with spaces/quoted \"é\".toml",
                "--configuration",
                "work",
                "--configuration",
                "archive",
            ],
            serde_json::json!([
                "--config",
                settings.to_str().unwrap(),
                "serve",
                "--configuration",
                "work",
                "--configuration",
                "archive"
            ]),
        ),
    ];
    for (args, expected) in cases {
        let output = Command::new(executable)
            .current_dir(directory.path())
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty());
        let client: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(client.as_object().unwrap().len(), 1);
        let server = &client["mcpServers"]["taxudromeio"];
        assert_eq!(server.as_object().unwrap().len(), 2);
        assert_eq!(server["args"], expected);
        let command = server["command"].as_str().unwrap();
        assert!(std::path::Path::new(command).is_absolute());
        assert_eq!(
            std::fs::canonicalize(command).unwrap(),
            std::fs::canonicalize(executable).unwrap()
        );
        let args = server["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg.as_str().unwrap())
            .collect::<Vec<_>>();
        // Use the actual exported launch command, without contacting mail or Keychain.
        let output = Command::new(command)
            .current_dir(directory.path())
            .args(args)
            .arg("--help")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("stdin/stdout")
        );
    }
    assert!(!settings.exists());
}

#[test]
fn mobileconfig_account_selection_requires_a_profile_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
        .args(["auth", "--mail-account", "2"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("--mobileconfig")
    );
}

/// anchorbook: A-auth-keychain#discovery-redaction
/// anchorbook: A-config-independent#same-server
#[test]
fn configuration_listing_does_not_need_keychain_and_missing_ids_fail() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "version = 1\n[[accounts]]\nid = 'work'\nhost = 'imap.example.com'\nroot = 'Projects'\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
        .arg("--config")
        .arg(&path)
        .arg("configurations")
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"configurations": [{
            "id": "work", "host": "imap.example.com", "port": 993, "transport": "tls",
            "root": "Projects", "drafts_folder": null,
            "operations": ["mail_list_folders", "mail_folder_status", "mail_list_messages",
                "mail_search", "mail_read_message", "mail_move_messages", "mail_copy_messages",
                "mail_set_flags", "mail_save_draft"]
        }]})
    );
    assert_eq!(value["configurations"][0]["root"], "Projects");
    assert!(
        value["configurations"][0]["operations"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("mail_save_draft"))
    );
    assert!(
        !String::from_utf8(output.stdout)
            .unwrap()
            .contains("username")
    );
    for command in ["serve", "mcp"] {
        let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
            .arg("--config")
            .arg(&path)
            .args([command, "--configuration", "missing"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("Unknown configuration")
        );
    }
}

/// anchorbook: A-interface-shared#policy-boundary
#[test]
fn cli_refuses_disabled_operations_and_missing_ids_before_keychain() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "version = 1\n[[accounts]]\nid = 'archive'\nhost = 'imap.example.test'\nroot = 'INBOX'\n[accounts.policy]\nprofile = 'read_only'\n",
    )
    .unwrap();
    for (args, expected) in [
        (
            vec![
                "mail",
                "move",
                "--configuration",
                "archive",
                "--uids",
                "12",
                "--uid-validity",
                "7",
                "--destination",
                "Done",
            ],
            "disabled for this configuration",
        ),
        (
            vec!["mail", "folders", "--configuration", "missing"],
            "Unknown configuration",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
            .arg("--config")
            .arg(&path)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(expected), "{error}");
        assert!(!error.contains("Keychain"), "{error}");
    }
}

/// anchorbook: A-cli-agent-help#workflow-help
/// anchorbook: A-cli-agent-help#renamed-interface
/// anchorbook: A-auth-setup-help#command-details
#[test]
fn nested_help_preserves_command_arguments_and_brief_usage_notes() {
    use taxudromeio::config;

    for (command, guidance) in [
        (
            "auth",
            vec![
                "--drafts-folder",
                "--transport",
                "--mobileconfig",
                "--mail-account",
                "--auth-type",
                "--profile",
                "starttls",
                "oauth2",
                "interactive terminal",
                "macOS Keychain",
            ],
        ),
        (
            "serve",
            vec![
                "--configuration",
                "omitted means all",
                "stdin/stdout",
                "Restart after configuration changes",
            ],
        ),
        (
            "mcp",
            vec![
                "--json",
                "--configuration",
                "omitted means all",
                "no saved accounts",
            ],
        ),
        (
            "mail",
            vec![
                "folders",
                "status",
                "list",
                "search",
                "read",
                "move",
                "copy",
                "flags",
                "draft",
                "Commands return JSON",
                "relative to the selected root",
            ],
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
            .args([
                "--config",
                "/nonexistent/taxudromeio-settings",
                command,
                "--help",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("taxudromeio skill"), "{help}");
        for phrase in guidance {
            assert!(help.contains(phrase), "Missing {phrase} in {help}");
        }
        assert!(!help.contains("mcpServers"));
        assert!(!help.contains("provider-issued IMAP refresh token"));
    }
    for command in [
        "folders", "status", "list", "search", "read", "move", "copy", "flags", "draft",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_taxudromeio"))
            .args(["mail", command, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("--configuration <CONFIGURATION>"), "{help}");
    }
    assert!(
        config::default_path()
            .unwrap()
            .ends_with("taxudromeio-mcp/config.toml")
    );
}

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Value, json};
use taxudromeio::{
    config::{Account, AuthType, Config},
    mail::{MailBackend, MailRequest},
    policy::{Policy, Profile},
    server::MailServer,
};

struct FixtureBackend(AtomicUsize);

impl MailBackend for FixtureBackend {
    fn execute(&self, account: &Account, request: MailRequest) -> Result<Value> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"configuration": account.id, "tool": request.operation().tool_name()}))
    }
}

fn account(id: &str, profile: Profile) -> Account {
    Account {
        id: id.into(),
        host: "imap.example.com".into(),
        port: 993,
        transport: Default::default(),
        root: "Projects".into(),
        drafts_folder: Some("Drafts".into()),
        auth_type: AuthType::Password,
        policy: Policy {
            profile,
            allowed_operations: None,
        },
    }
}

/// anchorbook: A-policy-triage#direct-call-enforcement
/// anchorbook: A-config-independent#same-server
/// anchorbook: A-auth-keychain#discovery-redaction
#[tokio::test]
async fn real_mcp_handshake_lists_tools_and_enforces_each_configuration() {
    let backend = Arc::new(FixtureBackend(AtomicUsize::new(0)));
    let config = Config {
        accounts: vec![
            account("triage", Profile::Triage),
            account("archive", Profile::ReadOnly),
        ],
        ..Config::default()
    };
    let server = MailServer::new(config, backend.clone()).unwrap();
    let (client_io, server_io) = tokio::io::duplex(64 * 1024);
    let server_task = tokio::spawn(async move {
        server
            .serve(server_io)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });
    let client = ().serve(client_io).await.unwrap();
    assert_eq!(client.peer_info().unwrap().server_info.name, "taxudromeio");
    let tools = client.list_all_tools().await.unwrap();
    assert_eq!(tools.len(), 10);
    assert!(tools.iter().any(|tool| tool.name == "mail_save_draft"));
    assert!(tools.iter().all(|tool| tool.name != "mail_send"));
    let result = client
        .call_tool(CallToolRequestParams::new("mail_list_configurations"))
        .await
        .unwrap();
    assert_eq!(
        result.structured_content.as_ref().unwrap(),
        &json!({"configurations": [
            {"id": "triage", "host": "imap.example.com", "port": 993, "transport": "tls",
             "root": "Projects", "drafts_folder": "Drafts",
             "operations": ["mail_list_folders", "mail_folder_status", "mail_list_messages",
                 "mail_search", "mail_read_message", "mail_move_messages", "mail_copy_messages",
                 "mail_set_flags", "mail_save_draft"]},
            {"id": "archive", "host": "imap.example.com", "port": 993, "transport": "tls",
             "root": "Projects", "drafts_folder": "Drafts",
             "operations": ["mail_list_folders", "mail_folder_status", "mail_list_messages",
                 "mail_search", "mail_read_message"]}
        ]})
    );
    assert_eq!(backend.0.load(Ordering::SeqCst), 0);
    let mut args = json!({
        "configuration": "archive", "folder": ".", "destination": "Done", "uids": [12], "uid_validity": 7
    });
    let result = client
        .call_tool(
            CallToolRequestParams::new("mail_move_messages")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    assert_eq!(result.is_error, Some(true));
    assert_eq!(backend.0.load(Ordering::SeqCst), 0);
    args["configuration"] = json!("triage");
    let result = client
        .call_tool(
            CallToolRequestParams::new("mail_move_messages")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content.unwrap()["configuration"],
        "triage"
    );
    assert_eq!(backend.0.load(Ordering::SeqCst), 1);
    let result = client
        .call_tool(
            CallToolRequestParams::new("mail_send")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    assert_eq!(result.is_error, Some(true));
    assert_eq!(backend.0.load(Ordering::SeqCst), 1);
    client.cancel().await.unwrap();
    server_task.await.unwrap();
}

/// anchorbook: A-policy-triage#operation-filter
#[tokio::test]
async fn read_only_hides_write_tools_and_rejects_direct_calls() {
    let backend = Arc::new(FixtureBackend(AtomicUsize::new(0)));
    let server = MailServer::new(
        Config {
            accounts: vec![account("work", Profile::ReadOnly)],
            ..Config::default()
        },
        backend.clone(),
    )
    .unwrap();
    assert_eq!(server.tools().len(), 6);
    assert!(server.invoke("mail_save_draft", json!({
        "configuration": "work", "from": "me@example.com", "subject": "Draft", "body": ""
    })).await.is_err());
    assert!(
        server
            .invoke(
                "mail_read_message",
                json!({
                    "configuration": "missing", "uid": 1, "uid_validity": 7
                })
            )
            .await
            .is_err()
    );
    assert!(server.invoke("mail_set_flags", json!({
        "configuration": "work", "uids": [1], "uid_validity": 7, "action": "add", "flags": ["deleted"]
    })).await.is_err());
    assert_eq!(backend.0.load(Ordering::SeqCst), 0);
}

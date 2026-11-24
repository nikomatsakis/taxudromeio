use std::{
    io::IsTerminal,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use clap::{Args, ValueEnum};
use dialoguer::{Input, Password, Select};

use crate::{
    config::{Account, AuthType, Config, Transport, validate_id},
    credentials::{CredentialStore, Credentials, OAuthRefresh},
    mail, oauth,
    policy::Profile,
    profile,
};

#[derive(Clone, Copy, ValueEnum)]
pub enum AuthMode {
    Password,
    Oauth2,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum AccessProfile {
    Triage,
    ReadOnly,
}

#[derive(Args, Default)]
pub struct AuthArgs {
    /// Import IMAP settings and any mail credential from an Apple profile.
    #[arg(long, value_name = "PATH")]
    pub mobileconfig: Option<PathBuf>,
    /// Choose an IMAP account within the profile (1-based; otherwise prompted).
    #[arg(long, requires = "mobileconfig", value_name = "NUMBER")]
    pub mail_account: Option<usize>,
    /// Stable, unique ID. Reusing an ID updates that configuration.
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub host: Option<String>,
    /// Connection mode. STARTTLS requires a successful TLS upgrade before login.
    #[arg(long, value_enum)]
    pub transport: Option<Transport>,
    /// IMAP port (defaults to 993 for TLS, 143 for STARTTLS).
    #[arg(long)]
    pub port: Option<u16>,
    #[arg(long)]
    pub root: Option<String>,
    /// Relative path inside the root; "." uses the root itself.
    #[arg(long)]
    pub drafts_folder: Option<String>,
    #[arg(long, value_enum)]
    pub profile: Option<AccessProfile>,
    #[arg(long, value_enum)]
    pub auth_type: Option<AuthMode>,
}

fn ask(label: &str, default: Option<&str>) -> Result<String> {
    let mut prompt = Input::<String>::new().with_prompt(label);
    if let Some(default) = default {
        prompt = prompt.default(default.into());
    }
    Ok(prompt.interact_text()?)
}

fn profile_connection(
    args: &AuthArgs,
    imported: Option<&profile::ImportedAccount>,
) -> Option<(String, Transport, u16)> {
    let imported = imported?;
    let transport = args.transport.unwrap_or(imported.transport);
    let port = args.port.unwrap_or_else(|| {
        if transport == imported.transport {
            imported.port
        } else {
            transport.default_port()
        }
    });
    Some((
        args.host.as_ref().unwrap_or(&imported.host).clone(),
        transport,
        port,
    ))
}

pub fn run(path: &Path, args: AuthArgs, store: &dyn CredentialStore) -> Result<()> {
    ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "Authentication requires an interactive terminal; run taxudromeio auth there"
    );
    let mut imported = match &args.mobileconfig {
        None => None,
        Some(path) => {
            let accounts = profile::load(path)?;
            let number = match args.mail_account {
                Some(number) => number,
                None if accounts.len() == 1 => 1,
                None => {
                    let choices = accounts
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            format!("Mail account {} ({}, port {})", i + 1, a.host, a.port)
                        })
                        .collect::<Vec<_>>();
                    Select::new()
                        .with_prompt("IMAP account to import")
                        .items(&choices)
                        .default(0)
                        .interact()?
                        + 1
                }
            };
            let imported = profile::select(accounts, number)?;
            eprintln!(
                "Imported IMAP settings from the profile. Only mail settings are used; the profile is not installed."
            );
            Some(imported)
        }
    };
    let connection = profile_connection(&args, imported.as_ref());
    let mut config = Config::load_for_auth(path)?;
    let id = match args.id {
        Some(id) => id,
        None => ask("Configuration ID", Some("personal"))?,
    };
    validate_id(&id)?;
    let previous = config.accounts.iter().find(|a| a.id == id).cloned();
    let mut previous_credentials = store.find(&id)?;
    eprintln!(
        "Login names and credentials are saved in macOS Keychain.\nConfiguration: {}",
        path.display()
    );
    let host = match args.host {
        Some(host) => host,
        None if connection.is_some() => connection.as_ref().unwrap().0.clone(),
        None => ask("IMAP host", previous.as_ref().map(|a| a.host.as_str()))?,
    };
    let transport = match args.transport {
        Some(transport) => transport,
        None if connection.is_some() => connection.as_ref().unwrap().1,
        None => match Select::new()
            .with_prompt("IMAP encryption")
            .items(["Implicit TLS", "Required STARTTLS"])
            .default(usize::from(
                previous
                    .as_ref()
                    .is_some_and(|a| a.transport == Transport::Starttls),
            ))
            .interact()?
        {
            0 => Transport::Tls,
            _ => Transport::Starttls,
        },
    };
    let port = match args.port {
        Some(port) => port,
        None if connection.is_some() => connection.as_ref().unwrap().2,
        None => ask(
            "IMAP port",
            Some(
                &previous
                    .as_ref()
                    .filter(|a| a.transport == transport)
                    .map_or(transport.default_port(), |a| a.port)
                    .to_string(),
            ),
        )?
        .parse()
        .context("Port must be an integer")?,
    };
    let root = match args.root {
        Some(root) => root,
        None => ask(
            "Root folder to expose",
            Some(
                previous
                    .as_ref()
                    .map(|a| a.root.as_str())
                    .or_else(|| imported.as_ref().and_then(|a| a.root_hint.as_deref()))
                    .unwrap_or("INBOX"),
            ),
        )?,
    };
    let drafts_folder = match args.drafts_folder {
        Some(folder) => Some(folder),
        None => {
            let default = previous
                .as_ref()
                .and_then(|a| a.drafts_folder.as_deref())
                .unwrap_or("");
            let folder: String = Input::new()
                .with_prompt("Drafts folder relative to root (blank disables default)")
                .default(default.into())
                .allow_empty(true)
                .interact_text()?;
            if folder.is_empty() {
                None
            } else {
                Some(folder)
            }
        }
    };
    let mut policy = previous
        .as_ref()
        .map(|a| a.policy.clone())
        .unwrap_or_default();
    if let Some(profile) = args.profile {
        policy.profile = match profile {
            AccessProfile::Triage => Profile::Triage,
            AccessProfile::ReadOnly => Profile::ReadOnly,
        };
    }
    let auth_type = match args.auth_type {
        Some(AuthMode::Password) => AuthType::Password,
        Some(AuthMode::Oauth2) => AuthType::Oauth2,
        None if imported.is_some() => AuthType::Password,
        None => {
            let default = usize::from(
                previous
                    .as_ref()
                    .is_some_and(|a| matches!(a.auth_type, AuthType::Oauth2)),
            );
            match Select::new()
                .with_prompt("Credential type")
                .items(["App password", "OAuth with automatic refresh"])
                .default(default)
                .interact()?
            {
                0 => AuthType::Password,
                _ => AuthType::Oauth2,
            }
        }
    };
    let username = match imported.as_mut().and_then(|a| a.username.take()) {
        Some(username) => username,
        None => ask(
            "IMAP login / email",
            previous_credentials.as_ref().map(|c| c.username.as_str()),
        )?,
    };
    let mut credentials = match auth_type {
        AuthType::Password => Credentials {
            username,
            secret: match imported.as_mut().and_then(|a| a.password.take()) {
                Some(password) => password,
                None => Password::new().with_prompt("App password").interact()?,
            },
            oauth: None,
        },
        AuthType::Oauth2 => {
            eprintln!(
                "Use your provider's registered OAuth client and an IMAP-enabled refresh token.\nToken endpoint and OAuth registration are saved only in Keychain."
            );
            let old = previous_credentials.as_ref().and_then(|c| c.oauth.as_ref());
            let token_endpoint = ask(
                "HTTPS OAuth token endpoint",
                old.map(|o| o.token_endpoint.as_str()),
            )?;
            let client_id = ask("OAuth client ID", old.map(|o| o.client_id.as_str()))?;
            let old = old.filter(|o| {
                o.token_endpoint == token_endpoint
                    && o.client_id == client_id
                    && previous_credentials
                        .as_ref()
                        .is_some_and(|c| c.username == username)
            });
            let refresh_token = Password::new()
                .with_prompt(if old.is_some() {
                    "Refresh token (blank keeps existing)"
                } else {
                    "Refresh token"
                })
                .allow_empty_password(old.is_some())
                .interact()?;
            let existing_secret = old.and_then(|o| o.client_secret.as_deref());
            let client_secret = match Select::new()
                .with_prompt("OAuth client authentication")
                .items(["Public client (no secret)", "Client secret in request body"])
                .default(usize::from(existing_secret.is_some()))
                .interact()?
            {
                0 => None,
                _ => {
                    let secret = Password::new()
                        .with_prompt(if existing_secret.is_some() {
                            "OAuth client secret (blank keeps existing)"
                        } else {
                            "OAuth client secret"
                        })
                        .allow_empty_password(existing_secret.is_some())
                        .interact()?;
                    Some(if secret.is_empty() {
                        existing_secret.unwrap().into()
                    } else {
                        secret
                    })
                }
            };
            Credentials {
                username,
                secret: String::new(),
                oauth: Some(OAuthRefresh {
                    token_endpoint,
                    client_id,
                    refresh_token: if refresh_token.is_empty() {
                        old.unwrap().refresh_token.clone()
                    } else {
                        refresh_token
                    },
                    client_secret,
                    expires_at: None,
                }),
            }
        }
    };
    ensure!(
        crate::config::safe_text(&credentials.username),
        "Login must be nonempty and contain no control characters"
    );
    let account = Account {
        id: id.clone(),
        host,
        port,
        transport,
        root,
        drafts_folder,
        auth_type,
        policy,
    };
    config.accounts.retain(|a| a.id != id);
    config.accounts.push(account.clone());
    config.validate()?;
    if matches!(account.auth_type, AuthType::Oauth2) {
        eprintln!("Obtaining an OAuth access token…");
        obtain_auth_token(
            &id,
            &mut credentials,
            &mut previous_credentials,
            store,
            oauth::refresh,
        )?;
    }
    ensure!(
        crate::config::safe_text(&credentials.secret),
        "Credential must be nonempty and contain no control characters"
    );
    eprintln!("Verifying TLS, IMAP login and root scope…");
    mail::verify(&account, &credentials)?;
    persist(
        path,
        &config,
        &id,
        &credentials,
        previous_credentials.as_ref(),
        store,
    )?;
    eprintln!(
        "Saved configuration '{id}'. Profile: {:?}. No mail was sent or changed.",
        account.policy.profile
    );
    Ok(())
}

fn obtain_auth_token(
    id: &str,
    credentials: &mut Credentials,
    previous: &mut Option<Credentials>,
    store: &dyn CredentialStore,
    refresh: impl FnOnce(&mut Credentials) -> Result<()>,
) -> Result<()> {
    // Reusing a grant can rotate its refresh token before mailbox verification.
    // Preserve that renewal even if verification or the TOML update later fails.
    let same_grant = previous.as_ref().is_some_and(|old| {
        old.username == credentials.username
            && old
                .oauth
                .as_ref()
                .zip(credentials.oauth.as_ref())
                .is_some_and(|(old, new)| {
                    old.token_endpoint == new.token_endpoint
                        && old.client_id == new.client_id
                        && old.refresh_token == new.refresh_token
                })
    });
    refresh(credentials)?;
    if same_grant {
        store.save(id, credentials).context(
            "Renewed OAuth credentials could not be saved in Keychain; authentication stopped",
        )?;
        *previous = Some(credentials.clone());
    }
    Ok(())
}

pub fn persist(
    path: &Path,
    config: &Config,
    id: &str,
    credentials: &Credentials,
    previous: Option<&Credentials>,
    store: &dyn CredentialStore,
) -> Result<()> {
    config.validate()?;
    store.save(id, credentials)?;
    if let Err(error) = config.save(path) {
        let rollback = match previous {
            Some(previous) => store.save(id, previous),
            None => store.delete(id),
        };
        if let Err(rollback) = rollback {
            return Err(error.context(format!(
                "Configuration write failed and Keychain rollback failed: {rollback}"
            )));
        }
        return Err(error.context("Configuration write failed; Keychain change was rolled back"));
    }
    Ok(())
}

pub fn remove(path: &Path, id: &str, store: &dyn CredentialStore) -> Result<()> {
    let original = Config::load(path)?;
    original.account(id)?;
    let mut next = original.clone();
    next.accounts.retain(|a| a.id != id);
    next.save(path)?;
    if let Err(error) = store.delete(id) {
        original
            .save(path)
            .context("Keychain deletion failed and configuration could not be restored")?;
        return Err(error.context("Configuration removal rolled back"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MemoryStore(Mutex<Option<Credentials>>);

    impl CredentialStore for MemoryStore {
        fn load(&self, _: &str) -> Result<Credentials> {
            anyhow::bail!("unused")
        }
        fn save(&self, _: &str, c: &Credentials) -> Result<()> {
            *self.0.lock().unwrap() = Some(c.clone());
            Ok(())
        }
        fn delete(&self, _: &str) -> Result<()> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    /// anchorbook: A-auth-profile-import#local-boundaries
    #[test]
    fn explicit_connection_flags_override_profile_and_transport_changes_choose_a_port() {
        let mut imported = profile::select(
            profile::parse(include_bytes!("../tests/fixtures/mail-profile.der")).unwrap(),
            1,
        )
        .unwrap();
        imported.port = 1993;
        let mut args = AuthArgs::default();
        assert_eq!(profile_connection(&args, Some(&imported)).unwrap().2, 1993);
        args.transport = Some(Transport::Starttls);
        assert_eq!(profile_connection(&args, Some(&imported)).unwrap().2, 143);
        args.host = Some("override.example.test".into());
        args.port = Some(1143);
        assert_eq!(
            profile_connection(&args, Some(&imported)).unwrap(),
            ("override.example.test".into(), Transport::Starttls, 1143)
        );
    }

    /// anchorbook: A-auth-profile-import#local-boundaries
    /// anchorbook: A-auth-keychain#no-file-secrets
    #[test]
    fn imported_login_and_password_are_saved_only_to_the_credential_store() {
        let mut imported = profile::select(
            profile::parse(include_bytes!("../tests/fixtures/mail-profile.der")).unwrap(),
            1,
        )
        .unwrap();
        let credentials = Credentials {
            username: imported.username.take().unwrap(),
            secret: imported.password.take().unwrap(),
            oauth: None,
        };
        let config = Config {
            accounts: vec![Account {
                id: "profile".into(),
                host: imported.host.clone(),
                port: imported.port,
                transport: imported.transport,
                root: "Projects".into(),
                drafts_folder: Some("Drafts".into()),
                auth_type: AuthType::Password,
                policy: Default::default(),
            }],
            ..Default::default()
        };
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let store = MemoryStore(Mutex::new(None));
        persist(&path, &config, "profile", &credentials, None, &store).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(!text.contains(&credentials.username));
        assert!(!text.contains(&credentials.secret));
        assert!(!text.contains("OutgoingPassword"));
        let saved = store.0.lock().unwrap();
        assert_eq!(saved.as_ref().unwrap().username, "fixture@example.test");
        assert_eq!(saved.as_ref().unwrap().secret, "fixture-app-password");
        assert_eq!(config.accounts[0].root, "Projects");
        assert_eq!(config.accounts[0].drafts_folder.as_deref(), Some("Drafts"));
        assert!(matches!(config.accounts[0].policy.profile, Profile::Triage));
    }

    /// anchorbook: A-auth-keychain#save-failure
    #[test]
    fn failed_config_write_restores_previous_keychain_record() {
        let dir = tempfile::tempdir().unwrap();
        let impossible = dir.path().join("not-a-directory/config.toml");
        std::fs::write(dir.path().join("not-a-directory"), "").unwrap();
        let old = Credentials {
            username: "old".into(),
            secret: "old-secret".into(),
            oauth: None,
        };
        let new = Credentials {
            username: "new".into(),
            secret: "new-secret".into(),
            oauth: None,
        };
        let store = MemoryStore(Mutex::new(None));
        assert!(
            persist(
                &impossible,
                &Config::default(),
                "work",
                &new,
                Some(&old),
                &store
            )
            .is_err()
        );
        assert_eq!(
            store.0.lock().unwrap().as_ref().unwrap().secret,
            "old-secret"
        );
        assert!(persist(&impossible, &Config::default(), "work", &new, None, &store).is_err());
        assert!(store.0.lock().unwrap().is_none());
    }

    /// anchorbook: A-oauth-renewable#refresh-lifecycle
    #[test]
    fn reauthentication_preserves_rotated_grant_even_when_configuration_save_fails() {
        let old = crate::test_support::credentials();
        let mut previous = Some(old.clone());
        let mut new = old;
        let store = MemoryStore(Mutex::new(None));
        obtain_auth_token("work", &mut new, &mut previous, &store, |c| {
            c.secret = "new-access".into();
            c.oauth.as_mut().unwrap().refresh_token = "rotated-refresh".into();
            Ok(())
        })
        .unwrap();
        // This save precedes mailbox verification, so a verification error retains the grant.
        assert_eq!(
            store
                .0
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .oauth
                .as_ref()
                .unwrap()
                .refresh_token,
            "rotated-refresh"
        );
        let dir = tempfile::tempdir().unwrap();
        let impossible = dir.path().join("not-a-directory/config.toml");
        std::fs::write(dir.path().join("not-a-directory"), "").unwrap();
        assert!(
            persist(
                &impossible,
                &Config::default(),
                "work",
                &new,
                previous.as_ref(),
                &store
            )
            .is_err()
        );
        assert_eq!(
            store
                .0
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .oauth
                .as_ref()
                .unwrap()
                .refresh_token,
            "rotated-refresh"
        );
    }
}

#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use zeroize::Zeroizing;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(target_os = "macos")]
use crate::config::validate_id;

#[cfg(target_os = "macos")]
const SERVICE: &str = "io.taxudromeio.mcp.imap";

#[derive(Clone, Deserialize, Serialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    pub username: String,
    pub secret: String,
    /// Provider registration and renewable credentials also remain in Keychain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthRefresh>,
}

#[derive(Clone, Deserialize, Serialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub struct OAuthRefresh {
    pub token_endpoint: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub refresh_token: String,
    pub expires_at: Option<u64>,
}

// Intentionally no Debug: neither the login nor its credential belongs in logs.
pub trait CredentialStore: Send + Sync {
    fn load(&self, id: &str) -> Result<Credentials>;
    fn find(&self, id: &str) -> Result<Option<Credentials>> {
        self.load(id).map(Some)
    }
    fn save(&self, id: &str, credentials: &Credentials) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
}

pub struct Keychain;

#[cfg(target_os = "macos")]
impl CredentialStore for Keychain {
    fn load(&self, id: &str) -> Result<Credentials> {
        self.find(id)?.with_context(|| {
            format!("No Keychain credential for '{id}'; run taxudromeio auth --id {id}")
        })
    }

    fn find(&self, id: &str) -> Result<Option<Credentials>> {
        validate_id(id)?;
        let bytes = match security_framework::passwords::get_generic_password(SERVICE, id) {
            Ok(bytes) => Zeroizing::new(bytes),
            Err(error) if error.code() == -25300 => return Ok(None),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Keychain unavailable for '{id}'; unlock it and grant access")
                });
            }
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .context("Invalid Keychain credential record")
    }

    fn save(&self, id: &str, credentials: &Credentials) -> Result<()> {
        validate_id(id)?;
        let bytes = Zeroizing::new(serde_json::to_vec(credentials)?);
        security_framework::passwords::set_generic_password(SERVICE, id, &bytes)
            .context("Could not save the login and credential in macOS Keychain")
    }

    fn delete(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        match security_framework::passwords::delete_generic_password(SERVICE, id) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()), // errSecItemNotFound
            Err(error) => Err(error).context("Could not delete the macOS Keychain credential"),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl CredentialStore for Keychain {
    fn load(&self, _: &str) -> Result<Credentials> {
        anyhow::bail!("This release requires macOS Keychain")
    }
    fn save(&self, _: &str, _: &Credentials) -> Result<()> {
        anyhow::bail!("This release requires macOS Keychain")
    }
    fn delete(&self, _: &str) -> Result<()> {
        anyhow::bail!("This release requires macOS Keychain")
    }
}

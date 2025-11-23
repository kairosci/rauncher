use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub account_id: String,
}

impl AuthToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn save(&self) -> Result<()> {
        // TODO: Encrypt tokens at rest instead of storing as plain JSON
        // TODO: Use OS keychain/credential manager for secure storage

        let auth_path = Self::auth_path()?;

        if let Some(parent) = auth_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&auth_path, &contents)?;

        // Set restrictive file permissions (0600) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&auth_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&auth_path, perms)?;
        }

        Ok(())
    }

    pub fn load() -> Result<Option<Self>> {
        // TODO: Decrypt tokens if encryption is implemented
        // TODO: Handle migration from old token formats

        let auth_path = Self::auth_path()?;

        if !auth_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&auth_path)?;
        let token: AuthToken = serde_json::from_str(&contents)?;

        Ok(Some(token))
    }

    pub fn delete() -> Result<()> {
        let auth_path = Self::auth_path()?;

        if auth_path.exists() {
            fs::remove_file(&auth_path)?;
        }

        Ok(())
    }

    fn auth_path() -> Result<PathBuf> {
        let data_dir = Config::data_dir()?;
        Ok(data_dir.join("auth.json"))
    }
}

#[derive(Clone)]
pub struct AuthManager {
    token: Option<AuthToken>,
}

impl AuthManager {
    pub fn new() -> Result<Self> {
        let token = AuthToken::load()?;
        Ok(Self { token })
    }

    pub fn is_authenticated(&self) -> bool {
        if let Some(token) = &self.token {
            !token.is_expired()
        } else {
            false
        }
    }

    pub fn get_token(&self) -> Result<&AuthToken> {
        match &self.token {
            Some(token) if !token.is_expired() => Ok(token),
            _ => Err(Error::NotAuthenticated),
        }
    }

    /// Check if token will expire soon (within 5 minutes)
    pub fn token_needs_refresh(&self) -> bool {
        if let Some(token) = &self.token {
            let now = chrono::Utc::now();
            let time_until_expiry = token.expires_at.signed_duration_since(now);
            time_until_expiry.num_minutes() < 5
        } else {
            false
        }
    }

    pub fn get_refresh_token(&self) -> Option<String> {
        self.token.as_ref().map(|t| t.refresh_token.clone())
    }

    pub fn set_token(&mut self, token: AuthToken) -> Result<()> {
        token.save()?;
        self.token = Some(token);
        Ok(())
    }

    pub fn logout(&mut self) -> Result<()> {
        AuthToken::delete()?;
        self.token = None;
        Ok(())
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().unwrap_or(Self { token: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_not_authenticated_by_default() {
        let manager = AuthManager { token: None };
        assert!(!manager.is_authenticated());
    }

    #[test]
    fn test_auth_token_expiry() {
        let expired_token = AuthToken {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            account_id: "test".to_string(),
        };
        assert!(expired_token.is_expired());

        let valid_token = AuthToken {
            access_token: "test".to_string(),
            refresh_token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            account_id: "test".to_string(),
        };
        assert!(!valid_token.is_expired());
    }
}

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

    /// Garantisce che il token sia valido: se prossimo alla scadenza o scaduto,
    /// prova a effettuare il refresh tramite l'implementazione di TokenRefresher.
    /// Restituisce un riferimento al token valido in memoria.
    pub fn ensure_valid_token<T: TokenRefresher>(
        &mut self,
        refresher: &T,
    ) -> Result<&AuthToken> {
        // Se non abbiamo token → non autenticato
        if self.token.is_none() {
            return Err(Error::NotAuthenticated);
        }

        // Valutiamo se serve refresh senza mantenere un prestito lungo
        let needs_refresh = self.token_needs_refresh() || self.token.as_ref().map(|t| t.is_expired()).unwrap_or(true);

        if !needs_refresh {
            return self.get_token();
        }

        let refresh = self
            .get_refresh_token()
            .ok_or_else(|| Error::NotAuthenticated)?;

        let new_token = refresher.refresh_token(&refresh)?;
        self.set_token(new_token)?;
        Ok(self.get_token()?)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().unwrap_or(Self { token: None })
    }
}

/// Trait minimo per permettere all'AuthManager di effettuare il refresh del token
/// senza dipendere direttamente da un tipo concreto del client API.
pub trait TokenRefresher {
    fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    struct MockRefresher;
    impl TokenRefresher for MockRefresher {
        fn refresh_token(&self, _refresh_token: &str) -> Result<AuthToken> {
            Ok(AuthToken {
                access_token: "new_access".into(),
                refresh_token: "new_refresh".into(),
                expires_at: Utc::now() + Duration::hours(1),
                account_id: "acc".into(),
            })
        }
    }

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

    #[test]
    fn test_ensure_valid_token_no_refresh_needed() {
        let token = AuthToken {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: Utc::now() + Duration::minutes(30),
            account_id: "acc".into(),
        };
        let mut manager = AuthManager { token: Some(token.clone()) };
        let got = manager.ensure_valid_token(&MockRefresher).unwrap();
        assert_eq!(got.access_token, token.access_token);
    }

    #[test]
    fn test_ensure_valid_token_does_refresh_on_expiring() {
        let token = AuthToken {
            access_token: "old".into(),
            refresh_token: "refresh".into(),
            expires_at: Utc::now() + Duration::minutes(1), // within 5 minutes threshold
            account_id: "acc".into(),
        };
        let mut manager = AuthManager { token: Some(token) };
        let got = manager.ensure_valid_token(&MockRefresher).unwrap();
        assert_eq!(got.access_token, "new_access");
        // and persisted
        assert_eq!(manager.get_token().unwrap().access_token, "new_access");
    }
}

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::auth::AuthToken;
use crate::{Error, Result};

// Request timeout configuration
const REQUEST_TIMEOUT_SECS: u64 = 30;

// Epic Games Store API endpoints
const OAUTH_TOKEN_URL: &str =
    "https://account-public-service-prod.ol.epicgames.com/account/api/oauth/token";
const DEVICE_AUTH_URL: &str =
    "https://account-public-service-prod.ol.epicgames.com/account/api/oauth/deviceAuthorization";
const LIBRARY_API_URL: &str =
    "https://library-service.live.use1a.on.epicgames.com/library/api/public";
const LAUNCHER_API_URL: &str =
    "https://launcher-public-service-prod.ol.epicgames.com/launcher/api/public";

// Epic Games launcher client credentials
const CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
const CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub app_name: String,
    pub app_title: String,
    pub app_version: String,
    pub install_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    pub verification_uri_complete: String,
    pub user_code: String,
    pub device_code: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryResponse {
    records: Vec<LibraryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryItem {
    #[serde(rename = "appName")]
    app_name: String,
    #[serde(rename = "namespace")]
    _namespace: String,
    #[serde(rename = "catalogItemId")]
    catalog_item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetResponse {
    id: String,
    #[serde(rename = "appName")]
    app_name: String,
    label_name: String,
    metadata: AssetMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetMetadata {
    #[serde(rename = "applicationId")]
    application_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct CatalogItem {
    id: String,
    title: String,
    #[serde(rename = "currentVersion")]
    current_version: Option<String>,
}

// Manifest structures for Epic Games manifest format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManifest {
    #[serde(rename = "ManifestFileVersion")]
    pub manifest_file_version: String,
    #[serde(rename = "bIsFileData")]
    pub is_file_data: bool,
    #[serde(rename = "AppNameString")]
    pub app_name: String,
    #[serde(rename = "AppVersionString")]
    pub app_version: String,
    #[serde(rename = "LaunchExeString")]
    pub launch_exe: String,
    #[serde(rename = "LaunchCommand")]
    pub launch_command: String,
    #[serde(rename = "BuildSizeInt")]
    pub build_size: u64,
    #[serde(rename = "FileManifestList")]
    pub file_list: Vec<FileManifest>,
    #[serde(rename = "ChunkHashList")]
    pub chunk_hash_list: std::collections::HashMap<String, String>,
    #[serde(rename = "ChunkShaList")]
    pub chunk_sha_list: std::collections::HashMap<String, Vec<u8>>,
    #[serde(rename = "DataGroupList")]
    pub data_group_list: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    #[serde(rename = "Filename")]
    pub filename: String,
    #[serde(rename = "FileHash")]
    pub file_hash: Vec<u8>,
    #[serde(rename = "FileChunkParts")]
    pub file_chunk_parts: Vec<ChunkPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPart {
    #[serde(rename = "Guid")]
    pub guid: String,
    #[serde(rename = "Offset")]
    pub offset: u64,
    #[serde(rename = "Size")]
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub total_files: usize,
    pub downloaded_files: usize,
    pub current_file: String,
}

pub struct EpicClient {
    client: Client,
}

impl EpicClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("rauncher/0.1.0")
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()?;

        Ok(Self { client })
    }

    /// Request device authorization (Step 1 of OAuth device flow)
    pub async fn request_device_auth(&self) -> Result<DeviceAuthResponse> {
        log::info!("Requesting device authorization from Epic Games");

        let device_auth_response = self
            .client
            .post(DEVICE_AUTH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
            .send()
            .await?;

        if !device_auth_response.status().is_success() {
            let status = device_auth_response.status();
            let error_text = device_auth_response.text().await.unwrap_or_default();
            return Err(Error::Auth(format!(
                "Failed to request device authorization: {} - {}",
                status, error_text
            )));
        }

        let device_auth: DeviceAuthResponse = device_auth_response.json().await?;

        log::debug!(
            "Device code received. Verification URL: {}",
            device_auth.verification_uri_complete
        );

        Ok(device_auth)
    }

    /// Poll for token using device code (Step 2 of OAuth device flow)
    pub async fn poll_for_token(&self, device_code: &str) -> Result<Option<AuthToken>> {
        let response = self
            .client
            .post(OAUTH_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
            .form(&[("grant_type", "device_code"), ("device_code", device_code)])
            .send()
            .await?;

        if response.status().is_success() {
            let oauth_response: OAuthTokenResponse = response.json().await?;

            log::info!("Successfully authenticated with Epic Games");

            let token = AuthToken {
                access_token: oauth_response.access_token,
                refresh_token: oauth_response.refresh_token,
                expires_at: chrono::Utc::now()
                    + chrono::Duration::seconds(oauth_response.expires_in),
                account_id: oauth_response.account_id,
            };

            return Ok(Some(token));
        }

        // Check if we got an error that means we should continue polling
        let status = response.status();
        if status == 400 {
            // This is expected while waiting for user to authenticate
            log::debug!("Still waiting for user authentication...");
            return Ok(None);
        }

        // Any other error should be reported
        let error_text = response.text().await.unwrap_or_default();
        Err(Error::Auth(format!(
            "Authentication failed: {} - {}",
            status, error_text
        )))
    }

    /// Authenticate with Epic Games using device code flow (combined method for CLI)
    pub async fn authenticate(&self) -> Result<(String, String, AuthToken)> {
        // Step 1: Request device authorization
        let device_auth = self.request_device_auth().await?;

        let device_code = device_auth.device_code.clone();
        let user_code = device_auth.user_code.clone();
        let verification_url = device_auth.verification_uri_complete.clone();

        // Step 2: Poll for token
        // Poll every 5 seconds for up to 10 minutes
        let max_attempts = 120; // 10 minutes
        let poll_interval = Duration::from_secs(5);

        for attempt in 0..max_attempts {
            if attempt > 0 {
                tokio::time::sleep(poll_interval).await;
            }

            log::debug!(
                "Polling for token (attempt {}/{})",
                attempt + 1,
                max_attempts
            );

            if let Some(token) = self.poll_for_token(&device_code).await? {
                return Ok((user_code, verification_url, token));
            }
        }

        Err(Error::Auth(
            "Authentication timed out. Please try again.".to_string(),
        ))
    }

    /// Refresh an expired access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken> {
        log::info!("Refreshing access token");

        let response = self
            .client
            .post(OAUTH_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Auth(format!(
                "Failed to refresh token: {} - {}",
                status, error_text
            )));
        }

        let oauth_response: OAuthTokenResponse = response.json().await?;

        log::info!("Successfully refreshed access token");

        Ok(AuthToken {
            access_token: oauth_response.access_token,
            refresh_token: oauth_response.refresh_token,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(oauth_response.expires_in),
            account_id: oauth_response.account_id,
        })
    }

    /// Get the user's game library
    pub async fn get_games(&self, token: &AuthToken) -> Result<Vec<Game>> {
        log::info!("Fetching game library from Epic Games");

        let library_url = format!("{}/users/{}/items", LIBRARY_API_URL, token.account_id);

        let response = self
            .client
            .get(&library_url)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api(format!(
                "Failed to fetch library: {} - {}",
                status, error_text
            )));
        }

        let library_response: LibraryResponse = response.json().await?;

        log::debug!("Found {} items in library", library_response.records.len());

        // Convert library items to games
        // Note: We need to fetch additional details for each game
        let mut games = Vec::new();

        for item in library_response.records {
            // For now, we'll create basic game entries
            // In a full implementation, we'd fetch catalog details for each
            games.push(Game {
                app_name: item.app_name.clone(),
                app_title: item.app_name.clone(), // Will be replaced with catalog lookup
                app_version: "unknown".to_string(), // Will be replaced with catalog lookup
                install_path: None,
            });
        }

        log::info!("Successfully fetched {} games from library", games.len());

        Ok(games)
    }

    /// Get game manifest URL for download
    pub async fn get_game_manifest(&self, token: &AuthToken, app_name: &str) -> Result<String> {
        log::info!("Fetching manifest for game: {}", app_name);

        // Get asset information from launcher API
        let asset_url = format!("{}/assets/Windows?label=Live", LAUNCHER_API_URL);

        let response = self
            .client
            .get(&asset_url)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api(format!(
                "Failed to fetch assets: {} - {}",
                status, error_text
            )));
        }

        let assets: Vec<AssetResponse> = response.json().await?;

        // Find the asset for the requested app
        let asset = assets
            .iter()
            .find(|a| a.app_name.eq_ignore_ascii_case(app_name))
            .ok_or_else(|| Error::GameNotFound(app_name.to_string()))?;

        log::info!("Found asset for {}: {}", app_name, asset.id);

        // Return the asset ID which would be used to construct manifest URL
        // In a real implementation, we would fetch the actual manifest from CDN
        Ok(asset.id.clone())
    }

    /// Download and parse game manifest
    pub async fn download_manifest(
        &self,
        token: &AuthToken,
        app_name: &str,
    ) -> Result<GameManifest> {
        // TODO: Implement real CDN manifest download
        // TODO: Parse manifest URL from asset metadata (build_info or manifest_location fields)
        // TODO: Handle gzip decompression for manifest files
        // TODO: Validate manifest signature/checksum for security
        // TODO: Cache manifests to reduce API calls
        // TODO: Handle manifest format version differences

        log::info!("Downloading manifest for game: {}", app_name);

        // Get asset ID first
        let _asset_id = self.get_game_manifest(token, app_name).await?;

        // In a real implementation, we would:
        // 1. Get the manifest URL from the asset metadata
        // 2. Download the manifest file (usually gzipped JSON)
        // 3. Decompress if needed
        // 4. Parse the manifest JSON

        // For now, create a minimal manifest structure for testing
        // This allows the installation process to proceed
        log::warn!("Using mock manifest data - real CDN download not implemented");

        // Choose a platform-appropriate launcher name for our mock manifest
        let launcher_name = {
            #[cfg(target_os = "windows")]
            { format!("{}.bat", app_name) }
            #[cfg(not(target_os = "windows"))]
            { "run.sh".to_string() }
        };

        Ok(GameManifest {
            manifest_file_version: "21".to_string(),
            is_file_data: true,
            app_name: app_name.to_string(),
            app_version: "1.0.0".to_string(),
            launch_exe: launcher_name,
            launch_command: String::new(),
            build_size: 0,
            file_list: Vec::new(),
            chunk_hash_list: std::collections::HashMap::new(),
            chunk_sha_list: std::collections::HashMap::new(),
            data_group_list: std::collections::HashMap::new(),
        })
    }

    /// Download a game chunk
    pub async fn download_chunk(&self, chunk_guid: &str, _token: &AuthToken) -> Result<Vec<u8>> {
        // TODO: Implement real CDN chunk download
        // TODO: Construct proper CDN URL from chunk GUID and game-specific CDN base
        // TODO: Implement parallel chunk downloads with connection pooling
        // TODO: Add retry logic with exponential backoff for failed downloads
        // TODO: Verify chunk integrity with SHA hash from manifest
        // TODO: Handle chunk decompression (zlib/gzip)
        // TODO: Support resume capability for interrupted downloads
        // TODO: Add download progress reporting
        // TODO: Implement bandwidth throttling option

        log::debug!("Downloading chunk: {}", chunk_guid);

        // In a real implementation:
        // 1. Construct CDN URL for the chunk
        // 2. Download the chunk data
        // 3. Verify integrity with SHA hash
        // 4. Decompress if needed

        log::warn!("Chunk download not implemented - returning empty data");
        Ok(Vec::new())
    }

    /// Check for game updates
    pub async fn check_for_updates(
        &self,
        token: &AuthToken,
        app_name: &str,
        current_version: &str,
    ) -> Result<Option<String>> {
        log::info!("Checking for updates for {}", app_name);

        // Get latest manifest
        let manifest = self.download_manifest(token, app_name).await?;

        if manifest.app_version != current_version {
            log::info!(
                "Update available: {} -> {}",
                current_version,
                manifest.app_version
            );
            Ok(Some(manifest.app_version))
        } else {
            log::info!("Game is up to date");
            Ok(None)
        }
    }

    /// Get cloud saves for a game
    pub async fn get_cloud_saves(
        &self,
        _token: &AuthToken,
        app_name: &str,
    ) -> Result<Vec<CloudSave>> {
        // TODO: Implement real cloud save API integration
        // TODO: Query Epic's cloud save endpoints (per-game save metadata)
        // TODO: Handle pagination for games with many saves
        // TODO: Parse save metadata (timestamps, size, etc.)
        // TODO: Implement save versioning and history

        log::info!("Fetching cloud saves for {}", app_name);

        // In a real implementation:
        // 1. Query Epic's cloud save API
        // 2. Get list of available saves
        // 3. Return save metadata

        log::warn!("Cloud save fetching not implemented");
        Ok(Vec::new())
    }

    /// Download a cloud save file
    pub async fn download_cloud_save(&self, _token: &AuthToken, save_id: &str) -> Result<Vec<u8>> {
        // TODO: Implement cloud save download
        // TODO: Get download URL from Epic API
        // TODO: Handle encrypted saves (decrypt with user keys)
        // TODO: Verify save integrity with checksums
        // TODO: Handle save conflicts (local vs cloud)

        log::info!("Downloading cloud save: {}", save_id);

        // In a real implementation:
        // 1. Get save download URL
        // 2. Download save data
        // 3. Verify integrity

        log::warn!("Cloud save download not implemented");
        Ok(Vec::new())
    }

    /// Upload a cloud save file
    pub async fn upload_cloud_save(
        &self,
        _token: &AuthToken,
        app_name: &str,
        save_data: &[u8],
    ) -> Result<()> {
        // TODO: Implement cloud save upload
        // TODO: Request upload URL from Epic API
        // TODO: Encrypt saves if required by game
        // TODO: Handle upload conflicts with existing saves
        // TODO: Implement save metadata (timestamp, game version)
        // TODO: Add upload progress reporting for large saves

        log::info!(
            "Uploading cloud save for {} ({} bytes)",
            app_name,
            save_data.len()
        );

        // In a real implementation:
        // 1. Get upload URL from API
        // 2. Upload save data
        // 3. Verify upload success

        log::warn!("Cloud save upload not implemented");
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSave {
    pub id: String,
    pub app_name: String,
    pub filename: String,
    pub size: u64,
    pub uploaded_at: String,
}

impl Default for EpicClient {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epic_client_creation() {
        let client = EpicClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_game_serialization() {
        let game = Game {
            app_name: "test_app".to_string(),
            app_title: "Test Game".to_string(),
            app_version: "1.0.0".to_string(),
            install_path: None,
        };
        let serialized = serde_json::to_string(&game).unwrap();
        let deserialized: Game = serde_json::from_str(&serialized).unwrap();
        assert_eq!(game.app_name, deserialized.app_name);
    }

    #[test]
    fn test_oauth_token_response_deserialization() {
        let json = r#"{
            "access_token": "test_access",
            "refresh_token": "test_refresh",
            "expires_in": 3600,
            "account_id": "test_account"
        }"#;
        let response: OAuthTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_access");
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_library_response_deserialization() {
        let json = r#"{
            "records": [
                {
                    "appName": "Fortnite",
                    "namespace": "fn",
                    "catalogItemId": "4fe75bbc5a674f4f9b356b5c90567da5"
                }
            ]
        }"#;
        let response: LibraryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].app_name, "Fortnite");
    }
}

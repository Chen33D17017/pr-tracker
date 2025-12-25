use anyhow::Result;
use keyring::Entry;
use serde::{Deserialize, Serialize};

// GitHub API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubTokenInfo {
    pub valid: bool,
    pub user: Option<GitHubUser>,
    pub scopes: Vec<String>,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_total: Option<u32>,
}

const KEYCHAIN_SERVICE: &str = "PRTracker";
const KEYCHAIN_ACCOUNT: &str = "github_token";

pub struct GitHubTokenManager {
    entry: Entry,
}

impl GitHubTokenManager {
    pub fn new() -> Result<Self> {
        let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
            .map_err(|e| anyhow::anyhow!("Failed to create keychain entry: {}", e))?;

        Ok(GitHubTokenManager { entry })
    }

    /// Save GitHub token to macOS Keychain
    pub fn save_token(&self, token: &str) -> Result<()> {
        println!("💾 Saving GitHub token to keychain (service: {}, account: {})", KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);

        self.entry
            .set_password(token)
            .map_err(|e| anyhow::anyhow!("Failed to save token to keychain: {}", e))?;

        println!("✅ GitHub token saved to macOS Keychain");

        // Immediately test retrieval
        match self.entry.get_password() {
            Ok(_) => println!("🔍 Verification: Token can be retrieved immediately after save"),
            Err(e) => println!("⚠️ Verification: Token cannot be retrieved immediately after save: {}", e),
        }

        Ok(())
    }

    /// Retrieve GitHub token from macOS Keychain
    pub fn get_token(&self) -> Result<Option<String>> {
        println!("🔍 Attempting to retrieve GitHub token from keychain (service: {}, account: {})", KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);

        match self.entry.get_password() {
            Ok(token) => {
                println!("✅ GitHub token retrieved from macOS Keychain (length: {} chars)", token.len());
                Ok(Some(token))
            }
            Err(keyring::Error::NoEntry) => {
                println!("ℹ️ No GitHub token found in keychain (service: {}, account: {})", KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);
                Ok(None)
            }
            Err(e) => {
                println!("❌ Failed to retrieve token from keychain: {} (service: {}, account: {})", e, KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);
                Err(anyhow::anyhow!("Failed to retrieve token from keychain: {}", e))
            }
        }
    }

    /// Delete GitHub token from macOS Keychain
    pub fn delete_token(&self) -> Result<()> {
        match self.entry.delete_credential() {
            Ok(_) => {
                println!("✅ GitHub token deleted from macOS Keychain");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                println!("ℹ️ No GitHub token found in keychain to delete");
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to delete token from keychain: {}", e)),
        }
    }

    /// Verify GitHub token and get user info
    pub async fn verify_token(&self, token: &str) -> Result<GitHubTokenInfo> {
        println!("🔍 Token verification - length: {}, first 10 chars: '{}...', last 4 chars: '...{}'",
            token.len(),
            &token.chars().take(10).collect::<String>(),
            &token.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>()
        );

        let client = reqwest::Client::new();

        // Make a request to GitHub API to verify the token
        let response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "PRTracker/1.0")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        // Check rate limits from headers
        let rate_limit_remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok());

        let rate_limit_total = response
            .headers()
            .get("x-ratelimit-limit")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok());

        // Get scopes from headers
        let scopes = response
            .headers()
            .get("x-oauth-scopes")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        if response.status().is_success() {
            let user: GitHubUser = response.json().await?;

            println!("✅ GitHub token verified for user: {}", user.login);
            println!("   Rate limit: {}/{}",
                rate_limit_remaining.unwrap_or(0),
                rate_limit_total.unwrap_or(5000)
            );
            println!("   Scopes: {:?}", scopes);

            Ok(GitHubTokenInfo {
                valid: true,
                user: Some(user),
                scopes,
                rate_limit_remaining,
                rate_limit_total,
            })
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            println!("❌ GitHub token verification failed: {} - {}", status, error_text);

            Ok(GitHubTokenInfo {
                valid: false,
                user: None,
                scopes: Vec::new(),
                rate_limit_remaining,
                rate_limit_total,
            })
        }
    }

    /// Test connection with current stored token
    pub async fn test_stored_token(&self) -> Result<GitHubTokenInfo> {
        if let Some(token) = self.get_token()? {
            self.verify_token(&token).await
        } else {
            Ok(GitHubTokenInfo {
                valid: false,
                user: None,
                scopes: Vec::new(),
                rate_limit_remaining: None,
                rate_limit_total: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_manager_creation() {
        let manager = GitHubTokenManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_token_storage() {
        let manager = GitHubTokenManager::new().unwrap();

        // Test saving a dummy token
        let test_token = "ghp_test_token_1234567890";
        assert!(manager.save_token(test_token).is_ok());

        // Test retrieving the token
        let retrieved = manager.get_token().unwrap();
        assert_eq!(retrieved, Some(test_token.to_string()));

        // Clean up
        let _ = manager.delete_token();
    }
}
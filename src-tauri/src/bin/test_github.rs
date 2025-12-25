// Test GitHub token management functionality
use pr_tracker_lib::github::GitHubTokenManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing GitHub Token Management with macOS Keychain...\n");

    let manager = GitHubTokenManager::new()?;

    // Test 1: Check if there's an existing token
    println!("1️⃣ Checking for existing token...");
    match manager.get_token()? {
        Some(token) => {
            println!("   Found existing token: {}...{}",
                     &token[..4.min(token.len())],
                     &token[token.len().saturating_sub(4)..]);
        }
        None => {
            println!("   No existing token found");
        }
    }

    // Test 2: Save a test token
    println!("\n2️⃣ Saving test token...");
    let test_token = "ghp_test1234567890abcdefghijklmnopqrstuvwxyz";
    manager.save_token(test_token)?;

    // Test 3: Retrieve the saved token
    println!("\n3️⃣ Retrieving saved token...");
    let retrieved_token = manager.get_token()?;
    match retrieved_token {
        Some(token) if token == test_token => {
            println!("   ✅ Token retrieved successfully and matches!");
        }
        Some(token) => {
            println!("   ❌ Token retrieved but doesn't match: {}", token);
        }
        None => {
            println!("   ❌ Failed to retrieve token");
        }
    }

    // Test 4: Test invalid token verification (expect failure)
    println!("\n4️⃣ Testing invalid token verification...");
    match manager.verify_token(test_token).await {
        Ok(info) if !info.valid => {
            println!("   ✅ Invalid token correctly identified as invalid");
            println!("   Rate limit remaining: {:?}", info.rate_limit_remaining);
        }
        Ok(info) if info.valid => {
            println!("   ❓ Test token was unexpectedly valid (this is unusual)");
            if let Some(user) = info.user {
                println!("   User: {}", user.login);
            }
        }
        Ok(info) => {
            println!("   ✅ Token verification completed: valid={}", info.valid);
        }
        Err(e) => {
            println!("   ⚠️ Network error during verification: {}", e);
            println!("   (This is expected if no internet connection)");
        }
    }

    // Test 5: Test stored token verification
    println!("\n5️⃣ Testing stored token verification...");
    match manager.test_stored_token().await {
        Ok(info) if !info.valid => {
            println!("   ✅ Stored test token correctly identified as invalid");
        }
        Ok(info) if info.valid => {
            println!("   ❓ Stored token is valid (this is unusual for test token)");
            if let Some(user) = info.user {
                println!("   User: {}", user.login);
            }
        }
        Ok(info) => {
            println!("   ✅ Token info retrieved: valid={}", info.valid);
        }
        Err(e) => {
            println!("   ⚠️ Network error during verification: {}", e);
            println!("   (This is expected if no internet connection)");
        }
    }

    // Test 6: Clean up - delete the test token
    println!("\n6️⃣ Cleaning up test token...");
    manager.delete_token()?;

    // Test 7: Verify token was deleted
    println!("\n7️⃣ Verifying token deletion...");
    match manager.get_token()? {
        Some(_) => {
            println!("   ❌ Token still exists after deletion");
        }
        None => {
            println!("   ✅ Token successfully deleted from keychain");
        }
    }

    println!("\n🎉 GitHub Token Management test completed!");
    println!("\n💡 Next steps:");
    println!("   - Get a real GitHub Personal Access Token from:");
    println!("     https://github.com/settings/personal-access-tokens/new");
    println!("   - Test with your real token using the Tauri app");
    println!("   - The token will be securely stored in macOS Keychain");

    Ok(())
}
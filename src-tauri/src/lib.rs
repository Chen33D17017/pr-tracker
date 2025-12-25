pub mod database;
pub mod github;

use anyhow::Result;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::Mutex;
use database::{Database, Project, PullRequest};
use github::{GitHubTokenManager, GitHubTokenInfo};

// Global database instance
type DbState = Arc<Mutex<Option<Database>>>;

// Initialize database connection
#[tauri::command]
async fn init_database(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let db = Database::new().await.map_err(|e| e.to_string())?;

    println!("✅ Database initialized successfully (without sample data)");

    let mut db_state = state.lock().await;
    *db_state = Some(db);

    Ok(())
}

// Clear all data from database (for clean start)
#[tauri::command]
async fn clear_all_data(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    println!("🧹 Clearing all data from database...");

    // Clear in order of dependencies
    sqlx::query("DELETE FROM pull_requests").execute(&db.pool).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM review_history").execute(&db.pool).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM team_members").execute(&db.pool).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM projects").execute(&db.pool).await.map_err(|e| e.to_string())?;

    println!("✅ All data cleared from database");
    Ok(())
}

// Get all projects
#[tauri::command]
async fn get_projects(state: tauri::State<'_, DbState>) -> Result<Vec<Project>, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.get_projects().await.map_err(|e| e.to_string())
}

// Add a new project
#[tauri::command]
async fn add_project(
    name: String,
    description: Option<String>,
    state: tauri::State<'_, DbState>
) -> Result<Project, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.add_project(name, description).await.map_err(|e| e.to_string())
}

// Update an existing project
#[tauri::command]
async fn update_project(
    id: i64,
    name: String,
    description: Option<String>,
    state: tauri::State<'_, DbState>
) -> Result<Project, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.update_project(id, name, description).await.map_err(|e| e.to_string())
}

// Delete a project
#[tauri::command]
async fn delete_project(
    id: i64,
    state: tauri::State<'_, DbState>
) -> Result<(), String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.delete_project(id).await.map_err(|e| e.to_string())
}

// Get project by ID
#[tauri::command]
async fn get_project_by_id(
    id: i64,
    state: tauri::State<'_, DbState>
) -> Result<Option<Project>, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.get_project_by_id(id).await.map_err(|e| e.to_string())
}

// Get all pull requests with author and project names
#[tauri::command]
async fn get_pull_requests(state: tauri::State<'_, DbState>) -> Result<Vec<PullRequest>, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.get_pull_requests().await.map_err(|e| e.to_string())
}

// Update PR status
#[tauri::command]
async fn update_pr_status(
    pr_id: i64,
    status: String,
    state: tauri::State<'_, DbState>
) -> Result<(), String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.update_pr_status(pr_id, status).await.map_err(|e| e.to_string())
}

// Update PR score
#[tauri::command]
async fn update_pr_score(
    pr_id: i64,
    score: i32,
    state: tauri::State<'_, DbState>
) -> Result<(), String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.update_pr_score(pr_id, score).await.map_err(|e| e.to_string())
}

// Update PR project assignment
#[tauri::command]
async fn update_pr_project(
    pr_id: i64,
    project_id: i64,
    state: tauri::State<'_, DbState>
) -> Result<(), String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.update_pr_project(pr_id, project_id).await.map_err(|e| e.to_string())
}

// GitHub Token Management Commands

/// Save GitHub token to macOS Keychain
#[tauri::command]
async fn save_github_token(token: String) -> Result<(), String> {
    let manager = GitHubTokenManager::new().map_err(|e| e.to_string())?;
    manager.save_token(&token).map_err(|e| e.to_string())
}

/// Retrieve GitHub token from macOS Keychain
#[tauri::command]
async fn get_github_token() -> Result<Option<String>, String> {
    let manager = GitHubTokenManager::new().map_err(|e| e.to_string())?;
    manager.get_token().map_err(|e| e.to_string())
}

/// Delete GitHub token from macOS Keychain
#[tauri::command]
async fn delete_github_token() -> Result<(), String> {
    let manager = GitHubTokenManager::new().map_err(|e| e.to_string())?;
    manager.delete_token().map_err(|e| e.to_string())
}

/// Verify GitHub token and get user info
#[tauri::command]
async fn verify_github_token(token: String) -> Result<GitHubTokenInfo, String> {
    let manager = GitHubTokenManager::new().map_err(|e| e.to_string())?;
    manager.verify_token(&token).await.map_err(|e| e.to_string())
}

/// Test connection with stored GitHub token
#[tauri::command]
async fn test_github_connection() -> Result<GitHubTokenInfo, String> {
    let manager = GitHubTokenManager::new().map_err(|e| e.to_string())?;
    manager.test_stored_token().await.map_err(|e| e.to_string())
}


/// Check if a PR with the given GitHub ID already exists
#[tauri::command]
async fn check_pr_exists_by_github_id(
    github_id: i64,
    state: tauri::State<'_, DbState>
) -> Result<Option<PullRequest>, String> {
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;

    db.get_pull_request_by_github_id(github_id).await.map_err(|e| e.to_string())
}

/// Test command to verify Tauri invoke is working
#[tauri::command]
async fn test_invoke(message: String) -> Result<String, String> {
    println!("🧪 Test invoke called with message: {}", message);
    Ok(format!("Test successful: {}", message))
}

/// Add PR from GitHub URL - fetches data and correlates with database
#[tauri::command]
async fn add_pr_from_github_url(
    pr_url: String,
    project_id: i64,
    token: String,
    state: tauri::State<'_, DbState>
) -> Result<PullRequest, String> {
    println!("🚀 Starting add_pr_from_github_url with URL: {}", pr_url);
    println!("📝 Function parameters: project_id={}, token_length={}", project_id, token.len());

    println!("🔒 Acquiring database lock...");
    let db_state = state.lock().await;
    let db = db_state.as_ref().ok_or("Database not initialized")?;
    println!("✅ Database lock acquired successfully");

    // Use the passed token directly instead of retrieving from keychain
    println!("🔑 Using provided GitHub token (length: {} chars)", token.len());

    // Parse GitHub URL to extract owner, repo, and PR number
    println!("🔗 Parsing GitHub URL...");
    let url_parts = parse_github_pr_url(&pr_url)?;
    println!("📊 Parsed URL - Owner: {}, Repo: {}, PR: {}", url_parts.owner, url_parts.repo, url_parts.pr_number);

    // Fetch PR data from GitHub API
    println!("🌐 Fetching PR data from GitHub API...");
    let pr_data = fetch_github_pr_data(&token, &url_parts.owner, &url_parts.repo, url_parts.pr_number).await?;
    println!("📋 PR Data fetched - Title: {}, Author: {}", pr_data.title, pr_data.user.login);

    // Check if this PR already exists in the database
    println!("🔍 Checking for existing PR with GitHub ID: {}", pr_data.id);
    if let Some(existing_pr) = db.get_pull_request_by_github_id(pr_data.id).await.map_err(|e| e.to_string())? {
        println!("⚠️ PR already exists in database with ID: {}", existing_pr.id);
        let project_name = existing_pr.project_name.unwrap_or("Unknown Project".to_string());
        return Err(format!(
            "This PR is already added to the system!\n\nPR: {} ({})\nProject: {}\nStatus: {}",
            existing_pr.title.unwrap_or("Untitled".to_string()),
            url_parts.pr_number,
            project_name,
            existing_pr.status
        ));
    }

    // Check if team member exists, create or update if needed
    println!("👥 Ensuring team member exists for author: {}", pr_data.user.login);
    let author_id = ensure_team_member_exists(&db, &pr_data.author()).await.map_err(|e| e.to_string())?;
    println!("✅ Team member handled - author_id: {}", author_id);

    // Add PR to database with 'Waiting' status
    println!("💾 Adding PR to database...");
    let new_pr = db.add_pull_request(
        pr_data.id,
        url_parts.pr_number,
        Some(pr_data.title),
        author_id,
        Some(project_id),
        Some(pr_data.head.ref_field), // branch name
        "Waiting".to_string(),
        Some(url_parts.owner.clone()),
        Some(url_parts.repo.clone())
    ).await.map_err(|e| e.to_string())?;

    println!("🎉 PR successfully added to database with ID: {}", new_pr.id);
    Ok(new_pr)
}

#[derive(Debug)]
struct GitHubPRUrl {
    owner: String,
    repo: String,
    pr_number: i64,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubPRData {
    id: i64,
    title: String,
    user: GitHubUser,
    head: GitHubHead,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GitHubUser {
    login: String,
    avatar_url: String,
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubHead {
    #[serde(rename = "ref")]
    ref_field: String,
}

// Helper struct for easier access
#[derive(Debug)]
struct PRAuthor {
    login: String,
    avatar_url: String,
    name: Option<String>,
}

impl From<GitHubUser> for PRAuthor {
    fn from(user: GitHubUser) -> Self {
        PRAuthor {
            login: user.login,
            avatar_url: user.avatar_url,
            name: user.name,
        }
    }
}

impl GitHubPRData {
    fn author(&self) -> PRAuthor {
        self.user.clone().into()
    }
}

fn parse_github_pr_url(url: &str) -> Result<GitHubPRUrl, String> {
    let re = Regex::new(r"github\.com/([^/]+)/([^/]+)/pull/(\d+)")
        .map_err(|e| format!("Regex error: {}", e))?;

    let caps = re.captures(url)
        .ok_or("Invalid GitHub PR URL format. Expected: https://github.com/owner/repo/pull/123")?;

    let owner = caps.get(1).unwrap().as_str().to_string();
    let repo = caps.get(2).unwrap().as_str().to_string();
    let pr_number = caps.get(3).unwrap().as_str().parse::<i64>()
        .map_err(|e| format!("Invalid PR number: {}", e))?;

    Ok(GitHubPRUrl { owner, repo, pr_number })
}

async fn fetch_github_pr_data(token: &str, owner: &str, repo: &str, pr_number: i64) -> Result<GitHubPRData, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, repo, pr_number);
    println!("📡 Making GitHub API request to: {}", url);
    println!("🔑 Token format check: first 10 chars = '{}...', last 4 chars = '...{}'",
        &token.chars().take(10).collect::<String>(),
        &token.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>()
    );

    // First, let's test if we can access the repository at all
    let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    println!("🔍 Testing repository access: {}", repo_url);

    let repo_response = client
        .get(&repo_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "PR-Tracker")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to test repository access: {}", e))?;

    let repo_status = repo_response.status();
    println!("🏛️ Repository access status: {}", repo_status);
    if !repo_status.is_success() {
        let repo_error = repo_response.text().await.unwrap_or_default();
        println!("❌ Repository access error: {}", repo_error);
        return Err(format!("Cannot access repository {}/{}. Status: {} - {}", owner, repo, repo_status, repo_error));
    } else {
        println!("✅ Repository access successful");
    }

    // Now try to access the specific PR
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "PR-Tracker")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch PR data: {}", e))?;

    let status = response.status();
    println!("📊 GitHub API response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        println!("❌ GitHub API error response: {}", error_text);

        // Provide more specific error messages based on status code
        let error_msg = match status.as_u16() {
            404 => {
                // More specific 404 debugging
                format!("PR #{} not found in repository {}/{}. Possible reasons:\n\
                        • PR number doesn't exist\n\
                        • PR might be in 'draft' state\n\
                        • Fine-grained token doesn't have 'Pull requests' read permission\n\
                        • Token doesn't have access to this specific repository\n\
                        \nPlease check:\n\
                        1. The PR URL is correct: https://github.com/{}/{}/pull/{}\n\
                        2. Your fine-grained token has 'Pull requests' read permission\n\
                        3. Your token has access to the {} repository",
                        pr_number, owner, repo, owner, repo, pr_number, repo)
            },
            401 => "GitHub token is invalid or expired. Please update your token in settings.".to_string(),
            403 => {
                if error_text.contains("forbids access via a personal access token (classic)") {
                    "Organization requires fine-grained token. This organization blocks classic tokens. \
                    Please create a fine-grained personal access token at GitHub Settings > Personal Access Tokens > Fine-grained tokens.".to_string()
                } else {
                    "Access forbidden. For private repositories, your GitHub token needs the 'repo' scope. \
                    Please go to GitHub Settings > Personal Access Tokens and create a new token with 'repo' permission.".to_string()
                }
            },
            _ => format!("GitHub API error: {} - {}", status, error_text)
        };

        return Err(error_msg);
    }

    let pr_data: GitHubPRData = response.json().await
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    println!("✅ Successfully parsed PR data");
    Ok(pr_data)
}

async fn ensure_team_member_exists(db: &Database, author: &PRAuthor) -> Result<i64, anyhow::Error> {
    // Check if team member exists by GitHub username
    if let Some(existing_member) = db.get_team_member_by_username(&author.login).await? {
        // Update avatar and display name if changed
        let needs_update = existing_member.avatar_url.as_ref() != Some(&author.avatar_url) ||
                          existing_member.display_name != author.name;

        if needs_update {
            db.update_team_member_info(
                existing_member.id,
                author.avatar_url.as_str(),
                author.name.as_deref()
            ).await?;
        }

        Ok(existing_member.id)
    } else {
        // Create new team member
        let new_member = db.add_team_member(
            &author.login,
            Some(&author.avatar_url),
            author.name.as_deref()
        ).await?;

        Ok(new_member.id)
    }
}

// Keep original greet command for testing
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DbState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            test_invoke,
            // Database commands
            init_database,
            clear_all_data,
            get_projects,
            add_project,
            update_project,
            delete_project,
            get_project_by_id,
            get_pull_requests,
            update_pr_status,
            update_pr_score,
            update_pr_project,
            check_pr_exists_by_github_id,
            // GitHub token management commands
            save_github_token,
            get_github_token,
            delete_github_token,
            verify_github_token,
            test_github_connection,
            // GitHub PR integration
            add_pr_from_github_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

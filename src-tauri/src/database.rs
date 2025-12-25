use anyhow::Result;
use chrono::Utc;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: i64,
    pub github_username: String,
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub github_id: i64,
    pub pr_number: i64,
    pub title: Option<String>,
    pub author_id: i64,
    pub project_id: Option<i64>,
    pub last_updated_at: i64,
    // Additional fields for UI
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
    pub author_display_name: Option<String>,
    pub project_name: Option<String>,
    pub status: String,
    pub branch: Option<String>,
    pub score: Option<i32>,
    pub repository_owner: Option<String>,
    pub repository_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewHistory {
    pub id: i64,
    pub pr_id: i64,
    pub action: String,
    pub performed_at: i64,
}

pub struct Database {
    pub pool: SqlitePool, // Make public for testing
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = get_database_path()?;

        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the file if it doesn't exist
        if !db_path.exists() {
            std::fs::File::create(&db_path)?;
            println!("Created database file: {:?}", db_path);
        }

        let database_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());
        println!("Connecting to database: {}", database_url);

        let pool = SqlitePool::connect(&database_url).await?;

        let db = Database { pool };
        db.initialize_tables().await?;

        Ok(db)
    }

    async fn migrate_database(&self) -> Result<()> {
        // Check if we need to add avatar_url and display_name columns to team_members
        let columns_exist = sqlx::query(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='team_members'"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = columns_exist {
            let table_sql: String = row.get("sql");

            // Add avatar_url column if it doesn't exist
            if !table_sql.contains("avatar_url") {
                sqlx::query("ALTER TABLE team_members ADD COLUMN avatar_url TEXT")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added avatar_url column to team_members table");
            }

            // Add display_name column if it doesn't exist
            if !table_sql.contains("display_name") {
                sqlx::query("ALTER TABLE team_members ADD COLUMN display_name TEXT")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added display_name column to team_members table");
            }
        }

        // Check if we need to add new columns to pull_requests table
        let pr_columns_exist = sqlx::query(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='pull_requests'"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = pr_columns_exist {
            let table_sql: String = row.get("sql");

            // Add status column if it doesn't exist
            if !table_sql.contains("status") {
                sqlx::query("ALTER TABLE pull_requests ADD COLUMN status TEXT DEFAULT 'Waiting'")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added status column to pull_requests table");
            }

            // Add branch column if it doesn't exist
            if !table_sql.contains("branch") {
                sqlx::query("ALTER TABLE pull_requests ADD COLUMN branch TEXT")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added branch column to pull_requests table");
            }

            // Add score column if it doesn't exist
            if !table_sql.contains("score") {
                sqlx::query("ALTER TABLE pull_requests ADD COLUMN score INTEGER")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added score column to pull_requests table");
            }

            // Add repository_owner column if it doesn't exist
            if !table_sql.contains("repository_owner") {
                sqlx::query("ALTER TABLE pull_requests ADD COLUMN repository_owner TEXT")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added repository_owner column to pull_requests table");
            }

            // Add repository_name column if it doesn't exist
            if !table_sql.contains("repository_name") {
                sqlx::query("ALTER TABLE pull_requests ADD COLUMN repository_name TEXT")
                    .execute(&self.pool)
                    .await?;
                println!("✅ Added repository_name column to pull_requests table");
            }
        }

        Ok(())
    }

    async fn initialize_tables(&self) -> Result<()> {
        // Run migrations first
        self.migrate_database().await?;
        // Team members table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS team_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                github_username TEXT UNIQUE NOT NULL,
                avatar_url TEXT,
                display_name TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Projects table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Pull requests table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pull_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                github_id INTEGER UNIQUE NOT NULL,
                pr_number INTEGER NOT NULL,
                title TEXT,
                author_id INTEGER NOT NULL,
                project_id INTEGER,
                last_updated_at INTEGER NOT NULL,
                status TEXT DEFAULT 'Waiting',
                branch TEXT,
                score INTEGER,
                FOREIGN KEY (author_id) REFERENCES team_members(id),
                FOREIGN KEY (project_id) REFERENCES projects(id)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Review history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS review_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pr_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                performed_at INTEGER NOT NULL,
                FOREIGN KEY (pr_id) REFERENCES pull_requests(id)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Project operations
    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        let rows = sqlx::query("SELECT id, name, description, created_at FROM projects ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let projects = rows.into_iter().map(|row| Project {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(projects)
    }

    pub async fn add_project(&self, name: String, description: Option<String>) -> Result<Project> {
        let current_time = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            "INSERT INTO projects (name, description, created_at) VALUES (?, ?, ?) RETURNING id"
        )
        .bind(&name)
        .bind(&description)
        .bind(current_time)
        .fetch_one(&self.pool)
        .await?;

        let id: i64 = result.get("id");

        Ok(Project {
            id,
            name,
            description,
            created_at: current_time,
        })
    }

    pub async fn update_project(&self, id: i64, name: String, description: Option<String>) -> Result<Project> {
        sqlx::query(
            "UPDATE projects SET name = ?, description = ? WHERE id = ?"
        )
        .bind(&name)
        .bind(&description)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // Return updated project
        let row = sqlx::query(
            "SELECT id, name, description, created_at FROM projects WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Project {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn delete_project(&self, id: i64) -> Result<()> {
        // First check if any PRs are assigned to this project
        let pr_count: i64 = sqlx::query(
            "SELECT COUNT(*) as count FROM pull_requests WHERE project_id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?
        .get("count");

        if pr_count > 0 {
            return Err(anyhow::anyhow!(
                "Cannot delete project: {} pull requests are assigned to this project. Please reassign them first.",
                pr_count
            ));
        }

        // Safe to delete - no PRs are assigned to this project
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Project not found"));
        }

        Ok(())
    }

    pub async fn get_project_by_id(&self, id: i64) -> Result<Option<Project>> {
        let row = sqlx::query(
            "SELECT id, name, description, created_at FROM projects WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Project {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    // Team member operations
    pub async fn get_or_create_team_member(&self, github_username: String) -> Result<TeamMember> {
        // Try to get existing member
        if let Ok(row) = sqlx::query(
            "SELECT id, github_username, avatar_url, display_name, created_at FROM team_members WHERE github_username = ?"
        )
        .bind(&github_username)
        .fetch_one(&self.pool)
        .await {
            return Ok(TeamMember {
                id: row.get("id"),
                github_username: row.get("github_username"),
                avatar_url: row.get("avatar_url"),
                display_name: row.get("display_name"),
                created_at: row.get("created_at"),
            });
        }

        // Create new member
        let current_time = chrono::Utc::now().timestamp();
        let result = sqlx::query(
            "INSERT INTO team_members (github_username, created_at) VALUES (?, ?) RETURNING id"
        )
        .bind(&github_username)
        .bind(current_time)
        .fetch_one(&self.pool)
        .await?;

        let id: i64 = result.get("id");

        Ok(TeamMember {
            id,
            github_username,
            avatar_url: None,
            display_name: None,
            created_at: current_time,
        })
    }

    // Update team member with GitHub data (avatar, display name)

    // Pull request operations
    pub async fn get_pull_requests(&self) -> Result<Vec<PullRequest>> {
        let rows = sqlx::query(
            r#"
            SELECT
                pr.id, pr.github_id, pr.pr_number, pr.title, pr.author_id,
                pr.project_id, pr.last_updated_at, pr.status, pr.branch, pr.score,
                pr.repository_owner, pr.repository_name,
                tm.github_username as author_name,
                tm.avatar_url as author_avatar,
                tm.display_name as author_display_name,
                p.name as project_name
            FROM pull_requests pr
            LEFT JOIN team_members tm ON pr.author_id = tm.id
            LEFT JOIN projects p ON pr.project_id = p.id
            ORDER BY pr.last_updated_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let prs = rows.into_iter().map(|row| PullRequest {
            id: row.get("id"),
            github_id: row.get("github_id"),
            pr_number: row.get("pr_number"),
            title: row.get("title"),
            author_id: row.get("author_id"),
            project_id: row.get("project_id"),
            last_updated_at: row.get("last_updated_at"),
            author_name: row.get("author_name"),
            author_avatar: row.get("author_avatar"),
            author_display_name: row.get("author_display_name"),
            project_name: row.get("project_name"),
            status: row.get("status"),
            branch: row.get("branch"),
            score: row.get("score"),
            repository_owner: row.get("repository_owner"),
            repository_name: row.get("repository_name"),
        }).collect();

        Ok(prs)
    }

    pub async fn update_pr_status(&self, pr_id: i64, status: String) -> Result<()> {
        sqlx::query("UPDATE pull_requests SET status = ? WHERE id = ?")
            .bind(status)
            .bind(pr_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_pr_score(&self, pr_id: i64, score: i32) -> Result<()> {
        sqlx::query("UPDATE pull_requests SET score = ? WHERE id = ?")
            .bind(score)
            .bind(pr_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_pr_project(&self, pr_id: i64, project_id: i64) -> Result<()> {
        sqlx::query("UPDATE pull_requests SET project_id = ? WHERE id = ?")
            .bind(project_id)
            .bind(pr_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Add sample data for testing
    pub async fn add_sample_data(&self) -> Result<()> {
        // Add sample projects if none exist
        let project_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM projects")
            .fetch_one(&self.pool)
            .await?
            .get("count");

        if project_count == 0 {
            let projects = vec![
                ("Frontend Core", Some("Main React application")),
                ("Backend API", Some("REST API and services")),
                ("Mobile App", Some("iOS and Android applications")),
                ("DevOps Tools", Some("CI/CD and deployment tools")),
            ];

            for (name, desc) in projects {
                self.add_project(name.to_string(), desc.map(|s| s.to_string())).await?;
            }
        }

        // Add sample team members and PRs
        let member_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM team_members")
            .fetch_one(&self.pool)
            .await?
            .get("count");

        if member_count == 0 {
            let members = vec!["Alex Chen", "Sarah Liao", "Michael Wu"];

            for member in members {
                let team_member = self.get_or_create_team_member(member.to_string()).await?;

                // Add sample PRs for each member
                let current_time = chrono::Utc::now().timestamp();
                sqlx::query(
                    r#"
                    INSERT INTO pull_requests
                    (github_id, pr_number, title, author_id, project_id, last_updated_at, status, branch, score)
                    VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?)
                    "#
                )
                .bind(1000 + team_member.id)
                .bind(1000 + team_member.id)
                .bind(format!("Sample PR from {}", member))
                .bind(team_member.id)
                .bind(current_time)
                .bind("Reviewing")
                .bind(format!("feature/sample-{}", team_member.id))
                .bind(if team_member.id % 2 == 0 { Some(9) } else { None::<i32> })
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    // Team member management methods
    pub async fn get_team_member_by_username(&self, username: &str) -> Result<Option<TeamMember>> {
        let row = sqlx::query(
            "SELECT id, github_username, avatar_url, display_name, created_at
             FROM team_members WHERE github_username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(TeamMember {
                id: row.get("id"),
                github_username: row.get("github_username"),
                avatar_url: row.get("avatar_url"),
                display_name: row.get("display_name"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn add_team_member(
        &self,
        github_username: &str,
        avatar_url: Option<&str>,
        display_name: Option<&str>
    ) -> Result<TeamMember> {
        let now = chrono::Utc::now().timestamp();

        let id = sqlx::query(
            r#"
            INSERT INTO team_members (github_username, avatar_url, display_name, created_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(github_username)
        .bind(avatar_url)
        .bind(display_name)
        .bind(now)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(TeamMember {
            id,
            github_username: github_username.to_string(),
            avatar_url: avatar_url.map(|s| s.to_string()),
            display_name: display_name.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn update_team_member_info(
        &self,
        member_id: i64,
        avatar_url: &str,
        display_name: Option<&str>
    ) -> Result<()> {
        sqlx::query(
            "UPDATE team_members SET avatar_url = ?, display_name = ? WHERE id = ?"
        )
        .bind(avatar_url)
        .bind(display_name)
        .bind(member_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a PR with the given GitHub ID already exists
    pub async fn get_pull_request_by_github_id(&self, github_id: i64) -> Result<Option<PullRequest>> {
        let row = sqlx::query(
            r#"
            SELECT
                pr.id, pr.github_id, pr.pr_number, pr.title, pr.author_id, pr.project_id,
                pr.last_updated_at, pr.status, pr.branch, pr.score, pr.repository_owner, pr.repository_name,
                tm.github_username as author_name,
                tm.avatar_url as author_avatar,
                tm.display_name as author_display_name,
                p.name as project_name
            FROM pull_requests pr
            LEFT JOIN team_members tm ON pr.author_id = tm.id
            LEFT JOIN projects p ON pr.project_id = p.id
            WHERE pr.github_id = ?
            "#
        )
        .bind(github_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(PullRequest {
                id: row.get("id"),
                github_id: row.get("github_id"),
                pr_number: row.get("pr_number"),
                title: row.get("title"),
                author_id: row.get("author_id"),
                project_id: row.get("project_id"),
                last_updated_at: row.get("last_updated_at"),
                author_name: row.get("author_name"),
                author_avatar: row.get("author_avatar"),
                author_display_name: row.get("author_display_name"),
                project_name: row.get("project_name"),
                status: row.get("status"),
                branch: row.get("branch"),
                score: row.get("score"),
                repository_owner: row.get("repository_owner"),
                repository_name: row.get("repository_name"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn add_pull_request(
        &self,
        github_id: i64,
        pr_number: i64,
        title: Option<String>,
        author_id: i64,
        project_id: Option<i64>,
        branch: Option<String>,
        status: String,
        repository_owner: Option<String>,
        repository_name: Option<String>
    ) -> Result<PullRequest> {
        let now = chrono::Utc::now().timestamp();

        let id = sqlx::query(
            r#"
            INSERT INTO pull_requests (github_id, pr_number, title, author_id, project_id, branch, status, repository_owner, repository_name, last_updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(github_id)
        .bind(pr_number)
        .bind(&title)
        .bind(author_id)
        .bind(project_id)
        .bind(&branch)
        .bind(&status)
        .bind(&repository_owner)
        .bind(&repository_name)
        .bind(now)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        // Fetch the complete PR with joined data
        let row = sqlx::query(
            r#"
            SELECT
                pr.id,
                pr.github_id,
                pr.pr_number,
                pr.title,
                pr.author_id,
                pr.project_id,
                pr.last_updated_at,
                tm.github_username as author_name,
                tm.avatar_url as author_avatar,
                tm.display_name as author_display_name,
                p.name as project_name,
                pr.status,
                pr.branch,
                pr.score,
                pr.repository_owner,
                pr.repository_name
            FROM pull_requests pr
            LEFT JOIN team_members tm ON pr.author_id = tm.id
            LEFT JOIN projects p ON pr.project_id = p.id
            WHERE pr.id = ?
            "#
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PullRequest {
            id: row.get("id"),
            github_id: row.get("github_id"),
            pr_number: row.get("pr_number"),
            title: row.get("title"),
            author_id: row.get("author_id"),
            project_id: row.get("project_id"),
            last_updated_at: row.get("last_updated_at"),
            author_name: row.get("author_name"),
            author_avatar: row.get("author_avatar"),
            author_display_name: row.get("author_display_name"),
            project_name: row.get("project_name"),
            status: row.get("status"),
            branch: row.get("branch"),
            score: row.get("score"),
            repository_owner: row.get("repository_owner"),
            repository_name: row.get("repository_name"),
        })
    }
}

fn get_database_path() -> Result<PathBuf> {
    let data_dir = data_dir().ok_or_else(|| anyhow::anyhow!("Cannot find data directory"))?;
    let app_dir = data_dir.join("PRTracker");

    // Ensure the directory exists
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
        println!("Created database directory: {:?}", app_dir);
    }

    let db_path = app_dir.join("database.sqlite");
    println!("Database path: {:?}", db_path);
    Ok(db_path)
}
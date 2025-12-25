// Test project CRUD operations
use pr_tracker_lib::database::Database;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ Testing Project CRUD Operations...\n");

    // Initialize database
    let db = Database::new().await?;
    println!("✅ Database connection established");

    // 1. Test Create (add_project)
    println!("\n1️⃣ Testing CREATE - Adding new projects...");

    let timestamp = chrono::Utc::now().timestamp();

    let project1 = db.add_project(
        format!("Frontend Redesign Test {}", timestamp),
        Some("Complete redesign of the user interface - Test version".to_string())
    ).await?;
    println!("   ✅ Added project: {} (ID: {})", project1.name, project1.id);

    let project2 = db.add_project(
        format!("API v2 Test {}", timestamp),
        None
    ).await?;
    println!("   ✅ Added project: {} (ID: {})", project2.name, project2.id);

    let project3 = db.add_project(
        format!("Mobile App Test {}", timestamp),
        Some("iOS and Android applications - Test version".to_string())
    ).await?;
    println!("   ✅ Added project: {} (ID: {})", project3.name, project3.id);

    // 2. Test Read (get_projects and get_project_by_id)
    println!("\n2️⃣ Testing READ - Fetching projects...");

    let all_projects = db.get_projects().await?;
    println!("   ✅ Found {} total projects:", all_projects.len());
    for project in &all_projects {
        let desc = project.description.as_deref().unwrap_or("No description");
        println!("      - {} (ID: {}) - {}", project.name, project.id, desc);
    }

    // Test get by specific ID
    let fetched_project = db.get_project_by_id(project1.id).await?;
    if let Some(project) = fetched_project {
        println!("   ✅ Fetched project by ID: {} - {}",
                 project.name,
                 project.description.as_deref().unwrap_or("No description"));
    } else {
        println!("   ❌ Failed to fetch project by ID");
    }

    // Test getting non-existent project
    let non_existent = db.get_project_by_id(999999).await?;
    if non_existent.is_none() {
        println!("   ✅ Correctly returned None for non-existent project");
    } else {
        println!("   ❌ Should have returned None for non-existent project");
    }

    // 3. Test Update (update_project)
    println!("\n3️⃣ Testing UPDATE - Modifying projects...");

    let updated_project = db.update_project(
        project2.id,
        format!("API v2.1 Test Updated {}", timestamp),
        Some("Enhanced API with new endpoints and improved performance".to_string())
    ).await?;

    println!("   ✅ Updated project: {} -> {}", project2.name, updated_project.name);
    println!("   ✅ Added description: {}",
             updated_project.description.as_deref().unwrap_or("None"));

    // Verify the update
    let verification = db.get_project_by_id(project2.id).await?;
    if let Some(project) = verification {
        if project.name.contains("API v2.1 Test Updated") {
            println!("   ✅ Update verified successfully");
        } else {
            println!("   ❌ Update verification failed");
        }
    }

    // 4. Test Delete protection (should fail if PRs are assigned)
    println!("\n4️⃣ Testing DELETE protection - Projects with assigned PRs...");

    // First, let's create a team member and PR to test the protection
    let team_member = db.get_or_create_team_member("test-user".to_string()).await?;

    // Manually insert a test PR assigned to our project
    let current_time = chrono::Utc::now().timestamp();
    sqlx::query(
        r#"
        INSERT INTO pull_requests
        (github_id, pr_number, title, author_id, project_id, last_updated_at, status, branch)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(999999)  // fake github_id
    .bind(123)     // pr_number
    .bind("Test PR")
    .bind(team_member.id)
    .bind(project1.id)  // Assign to project1
    .bind(current_time)
    .bind("Waiting")
    .bind("test-branch")
    .execute(&db.pool)
    .await?;

    println!("   📝 Created test PR assigned to project '{}'", project1.name);

    // Try to delete project with assigned PRs (should fail)
    match db.delete_project(project1.id).await {
        Err(error) => {
            println!("   ✅ Delete correctly blocked: {}", error);
        }
        Ok(_) => {
            println!("   ❌ Delete should have been blocked!");
        }
    }

    // 5. Test successful deletion (project without PRs)
    println!("\n5️⃣ Testing DELETE - Removing unused project...");

    match db.delete_project(project3.id).await {
        Ok(_) => {
            println!("   ✅ Successfully deleted project '{}'", project3.name);

            // Verify deletion
            let deleted_check = db.get_project_by_id(project3.id).await?;
            if deleted_check.is_none() {
                println!("   ✅ Deletion verified - project no longer exists");
            } else {
                println!("   ❌ Deletion failed - project still exists");
            }
        }
        Err(error) => {
            println!("   ❌ Delete failed unexpectedly: {}", error);
        }
    }

    // 6. Test deleting non-existent project
    match db.delete_project(999999).await {
        Err(error) => {
            println!("   ✅ Correctly failed to delete non-existent project: {}", error);
        }
        Ok(_) => {
            println!("   ❌ Should have failed to delete non-existent project");
        }
    }

    // 7. Final state check
    println!("\n6️⃣ Final state verification...");
    let final_projects = db.get_projects().await?;
    println!("   📊 Final project count: {}", final_projects.len());
    for project in &final_projects {
        println!("      - {} (ID: {})", project.name, project.id);
    }

    println!("\n🎉 Project CRUD operations test completed!");

    println!("\n💡 Summary of CRUD capabilities:");
    println!("   ✅ CREATE: Add new projects with name and optional description");
    println!("   ✅ READ: Get all projects or fetch by specific ID");
    println!("   ✅ UPDATE: Modify project name and description");
    println!("   ✅ DELETE: Remove projects (with safety checks for assigned PRs)");
    println!("   ✅ PROTECTION: Prevents deletion of projects with assigned PRs");

    Ok(())
}
// Simple test to verify database functionality
use pr_tracker_lib::database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing SQLite database setup...");

    // Initialize database
    let db = Database::new().await?;
    println!("✅ Database connection established");

    // Add sample data
    db.add_sample_data().await?;
    println!("✅ Sample data added");

    // Test getting projects
    let projects = db.get_projects().await?;
    println!("✅ Found {} projects:", projects.len());
    for project in &projects {
        println!("   - {} (ID: {})", project.name, project.id);
    }

    // Test getting PRs
    let prs = db.get_pull_requests().await?;
    println!("✅ Found {} pull requests:", prs.len());
    for pr in &prs {
        println!("   - PR #{}: {} (Author: {:?})",
                 pr.pr_number,
                 pr.title.as_deref().unwrap_or("No title"),
                 pr.author_name.as_deref().unwrap_or("Unknown"));
    }

    println!("\n🎉 Database test completed successfully!");
    Ok(())
}
# PR Tracker - GitHub Pull Request Management Desktop App

## Project Overview
A fully functional Tauri-based desktop application for tracking and managing GitHub Pull Request reviews. The app is complete with both frontend UI and backend implementation, providing real GitHub integration, local database storage, and comprehensive PR management features.

## Tech Stack
- **Frontend**: React 18 + TypeScript + Tailwind CSS
- **Backend**: Tauri (Rust) with SQLite database
- **Database**: SQLite with comprehensive schema
- **API**: GitHub REST API integration
- **Security**: macOS Keychain for secure token storage

## Current Status (Complete Implementation)
✅ **UI Implementation**: Complete React interface with Tailwind CSS styling
✅ **Database Setup**: SQLite with full schema and CRUD operations
✅ **GitHub Integration**: Full GitHub API integration with token management
✅ **PR Management**: Add, update, archive, and organize PRs
✅ **Project Organization**: Create and manage projects for PR categorization
✅ **Review Tracking**: Track review status and progress
✅ **Copy Features**: Copy branch details and GitHub URLs to clipboard
✅ **Archive System**: Archive completed PRs (hidden from main view)
✅ **Performance Tracking**: Member performance ranking system
✅ **Secure Storage**: GitHub tokens stored in macOS Keychain

## Key Features

### 1. GitHub Pull Request Integration
- **Add PRs from URL**: Paste any GitHub PR URL to fetch real PR data
- **Real-time Data**: Fetches actual PR metadata (title, author, branch, etc.)
- **Duplicate Prevention**: Prevents adding the same PR twice
- **Repository Support**: Works with public and private repositories (with proper tokens)

### 2. Project Management
- **Project Organization**: Create projects to categorize PRs
- **CRUD Operations**: Full create, read, update, delete for projects
- **PR Assignment**: Assign PRs to specific projects during addition
- **Filter by Project**: Filter PR list by selected project

### 3. Review Status Management
- **Status Tracking**: Waiting → Reviewing → Action → Approved workflow
- **Conditional UI**: Review progress hidden for approved PRs
- **Status Updates**: One-click status changes with immediate UI feedback
- **Archive System**: Archive completed PRs (excluded from active count)

### 4. User Experience Features
- **Copy to Clipboard**:
  - Branch details: "cr into main from feature-branch" format
  - GitHub URLs: Direct URL copying instead of browser opening
- **Performance Ranking**: Member performance tracking with N/A handling
- **Active PR Count**: Displays count excluding archived PRs
- **Responsive Design**: Clean, modern interface with hover effects
- **Real-time Updates**: UI updates immediately after database changes

## Database Schema

```sql
-- Team members who create PRs
CREATE TABLE team_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_username TEXT UNIQUE NOT NULL,
    avatar_url TEXT,
    display_name TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Projects for organizing PRs
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Pull requests to review
CREATE TABLE pull_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id INTEGER UNIQUE NOT NULL,
    pr_number INTEGER NOT NULL,
    title TEXT,
    author_id INTEGER NOT NULL,
    project_id INTEGER,
    repository_owner TEXT NOT NULL,
    repository_name TEXT NOT NULL,
    last_updated_at INTEGER NOT NULL,
    status TEXT DEFAULT 'Waiting',
    branch TEXT,
    score INTEGER,
    FOREIGN KEY (author_id) REFERENCES team_members(id),
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Review history (actions taken by user)
CREATE TABLE review_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL,
    action TEXT NOT NULL, -- 'approved', 'changes_requested', 'commented'
    performed_at INTEGER NOT NULL,
    FOREIGN KEY (pr_id) REFERENCES pull_requests(id)
);
```

## Implemented Tauri Commands

### GitHub Token Management
```rust
save_github_token(token: String) -> Result<()>
get_github_token() -> Result<Option<String>>
delete_github_token() -> Result<()>
verify_github_token(token: String) -> Result<GitHubTokenInfo>
test_github_connection() -> Result<GitHubTokenInfo>
```

### Database Operations
```rust
init_database() -> Result<()>
get_pull_requests() -> Result<Vec<PullRequest>>
get_projects() -> Result<Vec<Project>>
add_project(name: String, description: Option<String>) -> Result<Project>
update_pr_status(pr_id: i64, status: String) -> Result<()>
update_pr_score(pr_id: i64, score: i32) -> Result<()>
update_pr_project(pr_id: i64, project_id: i64) -> Result<()>
```

### GitHub API Integration
```rust
add_pr_from_github_url(prUrl: String, projectId: i64, token: String) -> Result<PullRequest>
```

## Key Application Files

### Frontend
- **`/src/App.tsx`**: Complete React application with all UI components and state management
- **`/src/main.tsx`**: React entry point
- **`/src/index.css`**: Tailwind CSS configuration and custom styles

### Backend (Tauri)
- **`/src-tauri/src/main.rs`**: Tauri application entry point
- **`/src-tauri/src/lib.rs`**: Main application logic with all Tauri commands
- **`/src-tauri/src/database.rs`**: SQLite database operations and queries
- **`/src-tauri/src/github.rs`**: GitHub API integration and token management
- **`/src-tauri/Cargo.toml`**: Rust dependencies and project configuration

## Application Flow

### Add PR Workflow
1. **User Input**: User clicks "Add PR" and enters GitHub PR URL
2. **Validation**: Frontend validates URL format and checks for required data
3. **Backend Processing**: Tauri command `add_pr_from_github_url` processes the request:
   - Parses owner, repo, and PR number from URL
   - Makes authenticated GitHub API call to fetch PR data
   - Checks for duplicate PRs in database
   - Creates team member record if author doesn't exist
   - Stores PR with project assignment in database
4. **UI Update**: Frontend refreshes PR list and closes modal automatically

### GitHub Token Management
1. **Storage**: Tokens securely stored in macOS Keychain using `keyring` crate
2. **Validation**: Real GitHub API validation before saving tokens
3. **Auto-load**: Tokens automatically loaded on app startup
4. **User Info**: Displays GitHub user information when token is valid

### Review Status Management
1. **Status Flow**: Waiting → Reviewing → Action → Approved → Archived
2. **Conditional UI**: Review controls hidden for approved PRs
3. **Archive System**: Archived PRs excluded from active views and counts
4. **Persistence**: All status changes saved to database immediately

## Development Setup

### Prerequisites
- Node.js 18+ for frontend development
- Rust (latest stable) for Tauri backend
- Tauri CLI tools

### Commands
```bash
# Install dependencies
npm install

# Development (with hot reload)
npm run tauri dev

# Build for production
npm run tauri build
```

## Configuration

### GitHub Token Setup
1. Generate a GitHub Personal Access Token with repo permissions
2. Open app Settings (gear icon in sidebar)
3. Enter token and click "Test & Save"
4. Token is automatically validated and stored securely

### Project Management
1. Create projects in Settings → Projects section
2. Assign PRs to projects when adding them
3. Filter PRs by project using sidebar navigation

## Security Considerations
- GitHub tokens stored in macOS Keychain (not localStorage)
- All API calls authenticated with user tokens
- Database stored locally in user's Application Support directory
- No sensitive data transmitted to external services

## Performance Features
- Efficient SQLite queries with proper indexing
- Minimal GitHub API calls (cached PR data)
- Responsive UI with optimized re-renders
- Background database operations

## User Experience Highlights
- **One-click PR addition** from GitHub URLs
- **Real-time status updates** with immediate visual feedback
- **Smart filtering** by project with active PR counts
- **Clipboard integration** for branch details and GitHub URLs
- **Clean archive system** keeps completed work organized
- **Performance tracking** shows team member rankings
- **Secure token management** with visual validation

## Current Implementation Status
The application is **feature-complete** and fully functional:
- All core features implemented and tested
- Frontend and backend fully integrated
- Database operations working correctly
- GitHub API integration operational
- User interface polished and responsive
- Security measures properly implemented

This PR Tracker is ready for daily use in managing GitHub pull request reviews with a complete feature set for team productivity.
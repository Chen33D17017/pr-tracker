# PR Tracker

A fully functional desktop application for tracking and managing GitHub Pull Request reviews.

**Built with:** Tauri + React + TypeScript + Tailwind CSS + SQLite

## ✨ Features

- 🔗 **Add PRs from GitHub URLs** - Real GitHub API integration
- 📁 **Project Organization** - Categorize PRs by project
- ✅ **Review Status Tracking** - Waiting → Reviewing → Action → Approved → Archived
- 📋 **Copy to Clipboard** - Branch details and GitHub URLs
- 🔒 **Secure Token Storage** - macOS Keychain integration
- 📊 **Performance Tracking** - Team member rankings

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Development
npm run tauri dev

# Build for production
npm run tauri build
```

## ⚙️ Setup

1. **GitHub Token**: Generate a Personal Access Token with repo permissions
2. **Settings**: Open app settings (⚙️) and add your token
3. **Add PRs**: Paste GitHub PR URLs to start tracking

## 📖 Documentation

📄 **[CLAUDE.md](CLAUDE.md)** - Complete project documentation, features, and implementation details

## 🛠️ Development

**Requirements:** Node.js 18+, Rust, Tauri CLI

**IDE Setup:** VS Code + [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

---

*Ready for daily use in managing GitHub pull request reviews*
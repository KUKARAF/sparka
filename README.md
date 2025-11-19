# Sparka - Android Ticket Manager

Android-first app for automatically analyzing tickets and adding them to Google Calendar with screen lock overlay display.

## Features

- 📱 Share files to the app for automatic ticket analysis
- 🤖 Uses Groq AI to extract ticket information
- 📅 Automatically creates calendar events
- 🔐 Google OAuth authentication
- 📋 Calendar selection
- 📁 **Google Drive integration** - automatic file upload and sharing
- 👥 **Calendar user sharing** - files shared with calendar users automatically
- 📤 **Manual sharing** - share with any Google Drive user
- 📊 **Import tracking** - standardized JSON log of all imports
- 🔒 Displays tickets over screen lock 1 hour before/after events
- 💾 Offline ticket storage
- 🏗️ Built with Rust + cargo-mobile

## Development

### Prerequisites

- Rust toolchain
- Android Studio/SDK
- cargo-mobile2

### Building

```bash
# Initialize project
cargo mobile init --android

# Build APK
cargo mobile android build --release

# Install on device
cargo mobile android run
```

### GitHub Actions

Automatic APK building and release on tag push:
- Builds APK using cargo-mobile
- Uploads artifacts to GitHub releases
- Supports cross-platform building

## Configuration

Required environment variables:
- `GOOGLE_CLIENT_ID` - Google OAuth client ID
- `GOOGLE_CLIENT_SECRET` - Google OAuth client secret
- `GROQ_API_KEY` - Groq API key for ticket analysis

## Architecture

- **Rust Core**: Native logic for file handling, API calls, and storage
- **Android UI**: Java activities and services for Android integration
- **Google Drive**: File storage and sharing system
- **Google Calendar**: Event creation and user management
- **SQLite**: Local storage for tickets and settings
- **Import Tracking**: JSON-based audit trail in Drive
- **Overlay System**: Screen lock overlay for ticket display

## Permissions

- Internet (API calls)
- Storage (file reading)
- System Alert Window (overlay display)
- Boot completed (auto-start services)
- Get Accounts (user information)
- Google Drive (file upload and sharing)
- Google Calendar (event creation)
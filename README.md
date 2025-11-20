# Sparka - Android Personal Assistant

## 🎯 Overview

Sparka is an offline-first Android personal assistant with AI integration. Features AI-powered chat, file management, calendar integration, and Google Drive sync. User synchronization happens via JSON files on Google Drive.

## 📱 Features

- **AI Chat**: Groq-powered intelligent conversations
- **Offline Support**: Local SQLite storage for offline functionality
- **Google Drive Integration**: File sync and user data backup
- **Google Calendar Integration**: Event management and scheduling
- **Overlay System**: Screen lock display for quick access
- **Push Notifications**: FCM integration for alerts

## 🏗 Architecture

- **Platform**: Android native with Material Design 3
- **Language**: Kotlin with MVVM architecture
- **Storage**: Local SQLite with Google Drive sync
- **AI**: Groq API integration
- **Authentication**: Google OAuth2

## 🚀 Installation

1. Download the latest APK from [Releases](https://github.com/KUKARAF/sparka/releases)
2. Enable "Install from unknown sources" in Android settings
3. Install the APK file
4. Sign in with your Google account to enable sync

## 🔧 Setup

### Required APIs
Enable these APIs in your Google Cloud Console:
- Google Drive API
- Google Calendar API
- Firebase Cloud Messaging (FCM)

### Environment Variables
Set these in your environment or `local.properties`:
```
GROQ_API_KEY=your-groq-api-key
GOOGLE_DRIVE_API_KEY=your-google-drive-api-key
GOOGLE_CALENDAR_API_KEY=your-google-calendar-api-key
```

## 📦 Build

```bash
# Clone repository
git clone https://github.com/KUKARAF/sparka.git
cd sparka

# Build APK
cd android
./gradlew assembleDebug

# Install on device
adb install app/build/outputs/apk/debug/app-debug.apk
```

## 🔄 Sync Protocol

User data is synchronized via JSON files stored in Google Drive:
- `sparka_settings.json` - User preferences and configuration
- `sparka_chat_history.json` - Chat conversation history
- `sparka_calendar_events.json` - Calendar events and reminders

## 📋 Permissions

The app requires these permissions:
- **Internet**: For AI API calls and Google services
- **Storage**: For local data and file management
- **Calendar**: For event management
- **Overlay**: For screen lock display
- **Notifications**: For alerts and reminders

## 🔒 Privacy

- **Local-First**: All data stored locally on device
- **Encrypted Sync**: Google Drive sync uses encryption
- **Minimal Data**: Only essential data is synced
- **User Control**: Full control over what gets synced

## 🐛 Issues

Report issues on [GitHub Issues](https://github.com/KUKARAF/sparka/issues)

## 📄 License

This project is licensed under the MIT License.
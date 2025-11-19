# Google Drive Integration Guide

## Overview

Sparka now includes Google Drive integration for:
- **Automatic file upload** when tickets are shared
- **Calendar user sharing** - files automatically shared with calendar users
- **Manual sharing** - share with any Google Drive user
- **Import tracking** - standardized JSON log of all imported files

## Features

### 1. Automatic Drive Upload
- All shared files are uploaded to a "Sparka" folder in Google Drive
- Files are organized and accessible from any device
- Supports PDFs, images, and text files

### 2. Calendar User Sharing
- Files automatically shared with users who have calendar access
- No manual configuration required
- Respects existing calendar permissions

### 3. Manual Sharing
- Share files with any Google Drive user via email
- Reader permissions by default
- Can be extended to writer/commenter roles

### 4. Import Tracking
- `import_log.json` tracks all file imports
- Records who imported what and when
- Links to calendar events
- Tracks import status and errors

## API Scopes Required

```java
List<Scope> requestedScopes = Arrays.asList(
    new Scope("https://www.googleapis.com/auth/calendar"),
    new Scope("https://www.googleapis.com/auth/calendar.readonly"),
    new Scope("https://www.googleapis.com/auth/drive.file"),
    new Scope("https://www.googleapis.com/auth/userinfo.email"),
    new Scope("https://www.googleapis.com/auth/userinfo.profile")
);
```

## Import Log Format

```json
{
  "version": "1.0",
  "last_updated": "2025-01-19T10:30:00Z",
  "imports": [
    {
      "file_id": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms",
      "file_name": "concert_ticket.pdf",
      "imported_by": "user@example.com",
      "imported_at": "2025-01-19T10:30:00Z",
      "calendar_event_id": "event123@example.com",
      "status": "imported",
      "error_message": null
    }
  ]
}
```

## File Structure

```
Google Drive/
└── Sparka/
    ├── concert_ticket.pdf
    ├── movie_ticket.jpg
    ├── import_log.json
    └── shared_with_users/
```

## Usage Flow

1. **User shares file** → Sparka app
2. **File uploaded** → Google Drive Sparka folder
3. **Calendar users extracted** → From selected calendars
4. **File shared** → With calendar users automatically
5. **Groq analysis** → Extract ticket information
6. **Calendar event created** → With ticket details
7. **Import logged** → In import_log.json
8. **Manual sharing** → Optional via email

## Security Considerations

- ✅ Files stored in user's Google Drive
- ✅ Respects existing Google permissions
- ✅ No file storage on app servers
- ✅ Import tracking for audit trail
- ✅ User-controlled sharing

## Error Handling

- Failed uploads are logged in import log
- Calendar creation errors don't affect file upload
- Sharing failures are tracked but don't stop import
- Network errors handled gracefully

## Future Enhancements

- OCR for image-based tickets
- File versioning
- Advanced sharing permissions
- File expiration
- Bulk operations
- Search and filtering
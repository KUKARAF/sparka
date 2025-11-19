# Android OAuth Setup Guide

## 1. Google Cloud Console Setup

### Create Android Client ID
1. Go to [Google Cloud Console](https://console.cloud.google.com/auth/clients)
2. Click **Create Client**
3. Select **Android** application type
4. Enter:
   - **Name**: Sparka Android
   - **Package name**: `com.sparka`
   - **SHA-1 signing certificate**: Get from your keystore

### Get SHA-1 Certificate
```bash
# For debug keystore
keytool -list -v -keystore ~/.android/debug.keystore -alias androiddebugkey -storepass android -keypass android

# For release keystore
keytool -list -v -keystore your-release-key.keystore -alias your-alias
```

### Create Web Client ID (for server auth code exchange)
1. Click **Create Client** again
2. Select **Web application** type
3. Name it: Sparka Web Backend
4. **No need to set redirect URIs** (used for server-side flow)

## 2. Update Configuration

Replace these placeholders in the code:

### MainActivity.java
```java
private static final String SERVER_CLIENT_ID = "YOUR_WEB_CLIENT_ID";
```

### auth.rs (backend operations)
```rust
// For server-side token exchange
client_id = "YOUR_WEB_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
```

## 3. Android Authorization Flow

The app now uses the proper Android AuthorizationClient:

1. **Request Authorization**: Uses `AuthorizationClient.authorize()`
2. **Handle Response**: Gets access token directly on device
3. **Server Auth Code**: Optional - send to backend for refresh token
4. **Offline Access**: Requested for long-term access

## 4. Key Differences from Web OAuth

| Web OAuth | Android AuthorizationClient |
|-----------|----------------------------|
| Uses redirect URIs | Uses PendingIntent |
| Manual token exchange | Direct access token |
| Browser-based flow | Native Android flow |
| Client secret required | No client secret on device |
| Custom scheme handling | Built-in intent handling |

## 5. Security Best Practices

- ✅ Store client secrets on backend only
- ✅ Use Android keystore for sensitive data
- ✅ Request minimal scopes needed
- ✅ Handle token revocation properly
- ✅ Use refresh tokens for long-term access

## 6. Testing

1. Build and install the app
2. Click "Connect Google Calendar"
3. Grant permissions in Google consent screen
4. Verify calendar access works
5. Test file sharing functionality
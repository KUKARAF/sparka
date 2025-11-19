package com.sparka;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.util.Log;
import android.widget.Toast;
import android.widget.Button;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import com.google.android.gms.auth.api.identity.*;
import com.google.android.gms.common.api.Scope;
import java.util.Arrays;
import java.util.List;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "Sparka";
    private static final String SERVER_CLIENT_ID = "YOUR_WEB_CLIENT_ID";
    
    private ActivityResultLauncher<IntentSenderRequest> startAuthorizationIntent;
    private AuthorizationClient authorizationClient;
    
    static {
        System.loadLibrary("sparka");
    }
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        // Initialize authorization client
        authorizationClient = Identity.getAuthorizationClient(this);
        
        // Register activity result launcher
        startAuthorizationIntent = registerForActivityResult(
            new ActivityResultContracts.StartIntentSenderForResult(),
            activityResult -> {
                try {
                    AuthorizationResult authorizationResult = authorizationClient
                        .getAuthorizationResultFromIntent(activityResult.getData());
                    
                    if (authorizationResult != null) {
                        handleAuthorizationResult(authorizationResult);
                    }
                } catch (Exception e) {
                    Log.e(TAG, "Authorization failed", e);
                    Toast.makeText(this, "Authorization failed: " + e.getMessage(), Toast.LENGTH_LONG).show();
                }
            }
        );
        
        // Initialize native code
        initNative();
        
        // Set up button click listeners
        Button authButton = findViewById(R.id.auth_button);
        authButton.setOnClickListener(v -> requestCalendarAuthorization());
        
        Button driveFilesButton = findViewById(R.id.drive_files_button);
        driveFilesButton.setOnClickListener(v -> loadDriveFiles());
        
        Button shareButton = findViewById(R.id.share_button);
        EditText emailInput = findViewById(R.id.email_input);
        shareButton.setOnClickListener(v -> {
            String email = emailInput.getText().toString();
            if (!email.isEmpty()) {
                shareWithUser(email);
            }
        });
        
        // Handle incoming intents
        handleIntent(getIntent());
    }
    
    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        handleIntent(intent);
    }
    
    private void handleIntent(Intent intent) {
        String action = intent.getAction();
        
        if (Intent.ACTION_SEND.equals(action) && intent.hasExtra(Intent.EXTRA_STREAM)) {
            // Handle file sharing
            Uri fileUri = intent.getParcelableExtra(Intent.EXTRA_STREAM);
            if (fileUri != null) {
                Log.d(TAG, "Received shared file: " + fileUri.toString());
                String result = handleSharedFile(fileUri.toString());
                Toast.makeText(this, result, Toast.LENGTH_LONG).show();
            }
        }
    }
    
    private void requestCalendarAuthorization() {
        // Define the scopes we need
        List<Scope> requestedScopes = Arrays.asList(
            new Scope("https://www.googleapis.com/auth/calendar"),
            new Scope("https://www.googleapis.com/auth/calendar.readonly"),
            new Scope("https://www.googleapis.com/auth/drive.file"),
            new Scope("https://www.googleapis.com/auth/userinfo.email"),
            new Scope("https://www.googleapis.com/auth/userinfo.profile")
        );
        
        // Build authorization request
        AuthorizationRequest authorizationRequest = AuthorizationRequest.builder()
            .setRequestedScopes(requestedScopes)
            .requestOfflineAccess(SERVER_CLIENT_ID) // For refresh token
            .build();
        
        // Start authorization flow
        authorizationClient.authorize(authorizationRequest)
            .addOnSuccessListener(authorizationResult -> {
                if (authorizationResult.hasResolution()) {
                    // User needs to grant permission
                    startAuthorizationIntent.launch(
                        new IntentSenderRequest.Builder(
                            authorizationResult.getPendingIntent().getIntentSender()
                        ).build()
                    );
                } else {
                    // Access was previously granted
                    handleAuthorizationResult(authorizationResult);
                }
            })
            .addOnFailureListener(e -> {
                Log.e(TAG, "Failed to authorize", e);
                Toast.makeText(this, "Authorization failed: " + e.getMessage(), Toast.LENGTH_LONG).show();
            });
    }
    
    private void handleAuthorizationResult(AuthorizationResult authorizationResult) {
        try {
            if (authorizationResult.getAccessToken() != null) {
                // Store access token in native code
                String accessToken = authorizationResult.getAccessToken();
                String result = storeGoogleToken(accessToken);
                Toast.makeText(this, result, Toast.LENGTH_LONG).show();
                
                // Load calendars after successful auth
                loadCalendars();
            }
            
            // Handle server auth code for refresh token (send to backend)
            if (authorizationResult.getServerAuthCode() != null) {
                String serverAuthCode = authorizationResult.getServerAuthCode();
                // In a real app, you'd send this to your backend server
                Log.d(TAG, "Server auth code received: " + serverAuthCode);
            }
            
        } catch (Exception e) {
            Log.e(TAG, "Error handling authorization result", e);
            Toast.makeText(this, "Error: " + e.getMessage(), Toast.LENGTH_LONG).show();
        }
    }
    
    private void loadCalendars() {
        try {
            String calendars = getCalendars();
            Log.d(TAG, "Calendars loaded: " + calendars);
            
            // Extract calendar users and set them for sharing
            // TODO: Parse calendar JSON and extract user emails
            // setCalendarUsers(usersJson);
            
            // Update UI with calendars
        } catch (Exception e) {
            Log.e(TAG, "Failed to load calendars", e);
        }
    }
    
    private void loadDriveFiles() {
        try {
            String files = getDriveFiles();
            Log.d(TAG, "Drive files loaded: " + files);
            Toast.makeText(this, "Files loaded: " + files.length(), Toast.LENGTH_SHORT).show();
        } catch (Exception e) {
            Log.e(TAG, "Failed to load Drive files", e);
            Toast.makeText(this, "Failed to load files", Toast.LENGTH_SHORT).show();
        }
    }
    
    public void revokeAccess() {
        List<Scope> requestedScopes = Arrays.asList(
            new Scope("https://www.googleapis.com/auth/calendar")
        );
        
        RevokeAccessRequest revokeAccessRequest = RevokeAccessRequest.builder()
            .setScopes(requestedScopes)
            .build();
        
        authorizationClient.revokeAccess(revokeAccessRequest)
            .addOnSuccessListener(unused -> {
                Log.i(TAG, "Successfully revoked access");
                Toast.makeText(this, "Access revoked", Toast.LENGTH_SHORT).show();
            })
            .addOnFailureListener(e -> {
                Log.e(TAG, "Failed to revoke access", e);
                Toast.makeText(this, "Failed to revoke access", Toast.LENGTH_SHORT).show();
            });
    }
    
    // Native methods
    public native void initNative();
    public native String handleSharedFile(String fileUri);
    public native String storeGoogleToken(String accessToken);
    public native String getCalendars();
    public native void setSelectedCalendars(String calendarsJson);
    public native void setCalendarUsers(String usersJson);
    public native String shareWithUser(String email);
    public native String getDriveFiles();
}
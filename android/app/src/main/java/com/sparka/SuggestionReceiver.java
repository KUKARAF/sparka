package com.sparka;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.util.Log;
import android.widget.Toast;

public class SuggestionReceiver extends BroadcastReceiver {
    private static final String TAG = "SuggestionReceiver";
    
    @Override
    public void onReceive(Context context, Intent intent) {
        String action = intent.getAction();
        String suggestionId = intent.getStringExtra("suggestion_id");
        
        if (suggestionId == null) return;
        
        if ("ACCEPT_SUGGESTION".equals(action)) {
            String title = intent.getStringExtra("title");
            String startTime = intent.getStringExtra("start_time");
            
            Log.d(TAG, "Accepting suggestion: " + suggestionId);
            
            // Call native method to accept suggestion
            acceptSuggestion(suggestionId, title, startTime);
            
            Toast.makeText(context, "Suggestion accepted: " + title, Toast.LENGTH_SHORT).show();
            
        } else if ("REJECT_SUGGESTION".equals(action)) {
            Log.d(TAG, "Rejecting suggestion: " + suggestionId);
            
            // Call native method to reject suggestion
            rejectSuggestion(suggestionId);
            
            Toast.makeText(context, "Suggestion rejected", Toast.LENGTH_SHORT).show();
        }
    }
    
    // Native methods
    private native void acceptSuggestion(String suggestionId, String title, String startTime);
    private native void rejectSuggestion(String suggestionId);
}
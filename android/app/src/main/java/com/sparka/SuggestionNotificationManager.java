package com.sparka;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.util.Log;
import androidx.core.app.NotificationCompat;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

public class SuggestionNotificationManager {
    private static final String TAG = "SuggestionNotification";
    private static final String CHANNEL_ID = "schedule_suggestions";
    private static final String CHANNEL_NAME = "Schedule Suggestions";
    private static final String CHANNEL_DESCRIPTION = "AI-powered schedule suggestions";
    
    private Context context;
    private NotificationManager notificationManager;
    
    public SuggestionNotificationManager(Context context) {
        this.context = context;
        this.notificationManager = (NotificationManager) context.getSystemService(Context.NOTIFICATION_SERVICE);
        createNotificationChannel();
    }
    
    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                CHANNEL_ID,
                CHANNEL_NAME,
                NotificationManager.IMPORTANCE_DEFAULT
            );
            channel.setDescription(CHANNEL_DESCRIPTION);
            notificationManager.createNotificationChannel(channel);
        }
    }
    
    public void showSuggestionNotification(String suggestionJson) {
        try {
            JSONObject suggestion = new JSONObject(suggestionJson);
            String title = suggestion.getString("title");
            String description = suggestion.getString("description");
            String startTime = suggestion.getString("start_time");
            String suggestionId = suggestion.getString("id");
            
            // Create accept intent
            Intent acceptIntent = new Intent(context, SuggestionReceiver.class);
            acceptIntent.setAction("ACCEPT_SUGGESTION");
            acceptIntent.putExtra("suggestion_id", suggestionId);
            acceptIntent.putExtra("title", title);
            acceptIntent.putExtra("start_time", startTime);
            PendingIntent acceptPendingIntent = PendingIntent.getBroadcast(
                context, 
                0, 
                acceptIntent, 
                PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
            );
            
            // Create reject intent
            Intent rejectIntent = new Intent(context, SuggestionReceiver.class);
            rejectIntent.setAction("REJECT_SUGGESTION");
            rejectIntent.putExtra("suggestion_id", suggestionId);
            PendingIntent rejectPendingIntent = PendingIntent.getBroadcast(
                context, 
                1, 
                rejectIntent, 
                PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
            );
            
            Notification notification = new NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .setContentTitle("Schedule Suggestion")
                .setContentText(title)
                .setStyle(new NotificationCompat.BigTextStyle().bigText(
                    title + "\n" + description + "\n" + "Start: " + startTime
                ))
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .addAction(android.R.drawable.ic_menu_save, "Accept", acceptPendingIntent)
                .addAction(android.R.drawable.ic_menu_close_clear_cancel, "Reject", rejectPendingIntent)
                .setAutoCancel(true)
                .build();
            
            int notificationId = suggestionId.hashCode();
            notificationManager.notify(notificationId, notification);
            
        } catch (JSONException e) {
            Log.e(TAG, "Error parsing suggestion JSON", e);
        }
    }
    
    public void showGoalCreationNotification(String goalDescription) {
        Intent createIntent = new Intent(context, MainActivity.class);
        createIntent.setAction("CREATE_GOAL");
        createIntent.putExtra("goal_description", goalDescription);
        createIntent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
        
        PendingIntent createPendingIntent = PendingIntent.getActivity(
            context, 
            0, 
            createIntent, 
            PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
        );
        
        Notification notification = new NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle("Create Scheduling Goal")
            .setContentText("Tap to create goal: " + goalDescription)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(createPendingIntent)
            .setAutoCancel(true)
            .build();
        
        notificationManager.notify(9999, notification);
    }
    
    public void showMultipleSuggestionsNotification(String suggestionsJson) {
        try {
            JSONArray suggestions = new JSONArray(suggestionsJson);
            int count = suggestions.length();
            
            Intent viewIntent = new Intent(context, MainActivity.class);
            viewIntent.setAction("VIEW_SUGGESTIONS");
            viewIntent.putExtra("suggestions", suggestionsJson);
            viewIntent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
            
            PendingIntent viewPendingIntent = PendingIntent.getActivity(
                context, 
                0, 
                viewIntent, 
                PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
            );
            
            Notification notification = new NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .setContentTitle("Schedule Suggestions Available")
                .setContentText(count + " new suggestions for your goals")
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .setContentIntent(viewPendingIntent)
                .setAutoCancel(true)
                .setNumber(count)
                .build();
            
            notificationManager.notify(8888, notification);
            
        } catch (JSONException e) {
            Log.e(TAG, "Error parsing suggestions JSON", e);
        }
    }
    
    public void cancelNotification(String suggestionId) {
        int notificationId = suggestionId.hashCode();
        notificationManager.cancel(notificationId);
    }
    
    public void cancelAllNotifications() {
        notificationManager.cancelAll();
    }
}
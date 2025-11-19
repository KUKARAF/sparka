package com.sparka;

import android.app.AlarmManager;
import android.app.PendingIntent;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.util.Log;

public class SchedulingService extends BroadcastReceiver {
    private static final String TAG = "SchedulingService";
    private static final int DAILY_CHECK_REQUEST_CODE = 1001;
    
    @Override
    public void onReceive(Context context, Intent intent) {
        Log.d(TAG, "Daily scheduling check triggered");
        
        if (Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction()) || 
            "DAILY_SCHEDULING_CHECK".equals(intent.getAction())) {
            
            // Trigger native scheduling check
            performDailySchedulingCheck();
            
            // Schedule next check
            scheduleNextDailyCheck(context);
        }
    }
    
    public static void scheduleNextDailyCheck(Context context) {
        AlarmManager alarmManager = (AlarmManager) context.getSystemService(Context.ALARM_SERVICE);
        
        Intent checkIntent = new Intent(context, SchedulingService.class);
        checkIntent.setAction("DAILY_SCHEDULING_CHECK");
        
        PendingIntent pendingIntent = PendingIntent.getBroadcast(
            context, 
            DAILY_CHECK_REQUEST_CODE, 
            checkIntent, 
            PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
        );
        
        // Schedule for next day at 8 AM
        java.util.Calendar calendar = java.util.Calendar.getInstance();
        calendar.setTimeInMillis(System.currentTimeMillis());
        calendar.set(java.util.Calendar.HOUR_OF_DAY, 8);
        calendar.set(java.util.Calendar.MINUTE, 0);
        calendar.set(java.util.Calendar.SECOND, 0);
        calendar.add(java.util.Calendar.DAY_OF_MONTH, 1);
        
        if (alarmManager != null) {
            alarmManager.setExactAndAllowWhileIdle(
                AlarmManager.RTC_WAKEUP,
                calendar.getTimeInMillis(),
                pendingIntent
            );
        }
        
        Log.d(TAG, "Next daily check scheduled for: " + calendar.getTime());
    }
    
    private void performDailySchedulingCheck() {
        // This will trigger the native scheduling check
        Log.d(TAG, "Performing daily scheduling check");
        
        // In a real implementation, this would:
        // 1. Get active goals from database
        // 2. Fetch calendar events for next 7 days
        // 3. Generate suggestions using AI
        // 4. Show notifications for new suggestions
    }
}
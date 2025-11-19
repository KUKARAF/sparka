package com.sparka;

import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.util.Log;
import android.os.Handler;
import android.os.Looper;

public class TicketCheckService extends Service {
    private static final String TAG = "TicketCheckService";
    private Handler handler;
    private Runnable checkRunnable;
    
    @Override
    public void onCreate() {
        super.onCreate();
        handler = new Handler(Looper.getMainLooper());
        
        checkRunnable = new Runnable() {
            @Override
            public void run() {
                checkTickets();
                // Check every 30 minutes
                handler.postDelayed(this, 30 * 60 * 1000);
            }
        };
    }
    
    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        // Start periodic checking
        handler.post(checkRunnable);
        return START_STICKY;
    }
    
    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
    
    private void checkTickets() {
        try {
            Log.d(TAG, "Checking for active tickets...");
            // This would trigger the native check
            checkAndShowOverlays();
        } catch (Exception e) {
            Log.e(TAG, "Error checking tickets", e);
        }
    }
    
    @Override
    public void onDestroy() {
        super.onDestroy();
        if (handler != null) {
            handler.removeCallbacks(checkRunnable);
        }
    }
    
    // Native method
    public native int checkAndShowOverlays();
}
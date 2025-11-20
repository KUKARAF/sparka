package com.sparka;

import android.app.Service;
import android.content.Intent;
import android.util.Log;

public class OverlayService extends Service {
    private static final String TAG = "OverlayService";
    
    @Override
    public void onCreate() {
        super.onCreate();
        Log.d(TAG, "OverlayService created");
    }
    
    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        Log.d(TAG, "OverlayService started");
        return START_STICKY;
    }
    
    @Override
    public void onDestroy() {
        super.onDestroy();
        Log.d(TAG, "OverlayService destroyed");
    }
    
    static {
        System.loadLibrary("sparka");
    }
}
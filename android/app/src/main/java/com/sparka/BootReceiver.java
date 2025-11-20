package com.sparka;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.util.Log;

public class BootReceiver extends BroadcastReceiver {
    private static final String TAG = "BootReceiver";
    
    @Override
    public void onReceive(Context context, Intent intent) {
        Log.d(TAG, "Boot received");
        // Start overlay service on boot
        Intent serviceIntent = new Intent(context, OverlayService.class);
        context.startService(serviceIntent);
    }
    
    static {
        System.loadLibrary("sparka");
    }
}
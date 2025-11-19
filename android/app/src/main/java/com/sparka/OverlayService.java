package com.sparka;

import android.app.Service;
import android.content.Intent;
import android.graphics.PixelFormat;
import android.os.IBinder;
import android.util.Log;
import android.view.Gravity;
import android.view.LayoutInflater;
import android.view.View;
import android.view.WindowManager;
import android.widget.TextView;
import android.os.Handler;
import android.os.Looper;

public class OverlayService extends Service {
    private static final String TAG = "OverlayService";
    private WindowManager windowManager;
    private View overlayView;
    private Handler handler;
    private Runnable checkRunnable;
    
    @Override
    public void onCreate() {
        super.onCreate();
        windowManager = (WindowManager) getSystemService(WINDOW_SERVICE);
        handler = new Handler(Looper.getMainLooper());
        
        checkRunnable = new Runnable() {
            @Override
            public void run() {
                checkAndShowOverlays();
                // Check every 5 minutes
                handler.postDelayed(this, 5 * 60 * 1000);
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
    
    private void checkAndShowOverlays() {
        try {
            int activeCount = checkAndShowOverlays();
            Log.d(TAG, "Active overlays: " + activeCount);
            
            if (activeCount > 0) {
                showOverlay();
            } else {
                hideOverlay();
            }
        } catch (Exception e) {
            Log.e(TAG, "Error checking overlays", e);
        }
    }
    
    private void showOverlay() {
        if (overlayView != null) {
            return; // Already showing
        }
        
        LayoutInflater inflater = (LayoutInflater) getSystemService(LAYOUT_INFLATER_SERVICE);
        overlayView = inflater.inflate(R.layout.overlay_layout, null);
        
        WindowManager.LayoutParams params = new WindowManager.LayoutParams(
            WindowManager.LayoutParams.MATCH_PARENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE |
            WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL,
            PixelFormat.TRANSLUCENT
        );
        
        params.gravity = Gravity.TOP;
        
        windowManager.addView(overlayView, params);
        
        // Update overlay content
        updateOverlayContent();
    }
    
    private void hideOverlay() {
        if (overlayView != null) {
            windowManager.removeView(overlayView);
            overlayView = null;
        }
    }
    
    private void updateOverlayContent() {
        if (overlayView == null) return;
        
        TextView ticketText = overlayView.findViewById(R.id.ticket_text);
        // Get ticket info from native code
        String ticketInfo = getActiveTicketInfo();
        if (ticketInfo != null) {
            ticketText.setText(ticketInfo);
        }
    }
    
    @Override
    public void onDestroy() {
        super.onDestroy();
        hideOverlay();
        if (handler != null) {
            handler.removeCallbacks(checkRunnable);
        }
    }
    
    // Native methods
    public native int checkAndShowOverlays();
    public native String getActiveTicketInfo();
}
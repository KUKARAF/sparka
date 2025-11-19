package com.sparka;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.util.Log;

public class BootReceiver extends BroadcastReceiver {
    private static final String TAG = "BootReceiver";
    
    @Override
    public void onReceive(Context context, Intent intent) {
        if (Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())) {
            Log.d(TAG, "Boot completed, starting services");
            
            // Start the overlay service
            Intent overlayService = new Intent(context, OverlayService.class);
            context.startService(overlayService);
            
            // Start the ticket check service
            Intent ticketService = new Intent(context, TicketCheckService.class);
            context.startService(ticketService);
        }
    }
}
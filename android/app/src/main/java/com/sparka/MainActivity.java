package com.sparka;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import android.widget.Toast;
import android.widget.Button;
import androidx.appcompat.app.AppCompatActivity;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "Sparka";
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        // Simple button to test the app
        Button testButton = findViewById(R.id.test_button);
        if (testButton != null) {
            testButton.setOnClickListener(v -> {
                Toast.makeText(this, "Sparka is working!", Toast.LENGTH_SHORT).show();
                Log.d(TAG, "Test button clicked");
            });
        }
    }
    
    static {
        System.loadLibrary("sparka");
    }
}
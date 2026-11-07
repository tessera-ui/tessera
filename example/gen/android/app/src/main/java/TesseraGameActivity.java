package com.tessera.example;

import android.os.Bundle;

import androidx.core.view.WindowCompat;

import com.google.androidgamesdk.GameActivity;

public class TesseraGameActivity extends GameActivity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        super.onCreate(savedInstanceState);
    }
}

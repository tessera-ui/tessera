package com.tessera.example;

import android.os.Bundle;
import android.view.KeyEvent;

import androidx.activity.BackEventCompat;
import androidx.activity.OnBackPressedCallback;
import androidx.annotation.Keep;
import androidx.core.view.WindowCompat;

import com.google.androidgamesdk.GameActivity;

public class TesseraGameActivity extends GameActivity {
    private long backHandler;
    private long gestureHandler;

    private final OnBackPressedCallback backCallback = new OnBackPressedCallback(false) {
        @Override
        public void handleOnBackStarted(BackEventCompat event) {
            gestureHandler = backHandler;
            dispatchTesseraBack(gestureHandler, 0, event.getProgress(), event.getSwipeEdge());
        }

        @Override
        public void handleOnBackProgressed(BackEventCompat event) {
            dispatchTesseraBack(gestureHandler, 1, event.getProgress(), event.getSwipeEdge());
        }

        @Override
        public void handleOnBackCancelled() {
            finishBack(2);
        }

        @Override
        public void handleOnBackPressed() {
            finishBack(3);
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        super.onCreate(savedInstanceState);
        getOnBackPressedDispatcher().addCallback(this, backCallback);
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() != KeyEvent.KEYCODE_BACK) {
            return super.dispatchKeyEvent(event);
        }
        // GameActivity can consume Back before the Activity dispatcher sees it.
        if (event.getAction() == KeyEvent.ACTION_UP && !event.isCanceled()) {
            getOnBackPressedDispatcher().onBackPressed();
        }
        return true;
    }

    private void finishBack(int phase) {
        long handler = gestureHandler != 0 ? gestureHandler : backHandler;
        gestureHandler = 0;
        dispatchTesseraBack(handler, phase, 0, 0);
    }

    @Keep
    public void setTesseraBackHandler(long handler) {
        runOnUiThread(() -> {
            backHandler = handler;
            backCallback.setEnabled(handler != 0);
        });
    }

    @Keep
    private native void dispatchTesseraBack(long handler, int phase, float progress, int edge);
}

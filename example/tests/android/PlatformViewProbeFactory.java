package com.tessera.example;

import android.app.Activity;
import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.text.Editable;
import android.text.TextWatcher;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;

import com.tessera.TesseraPlatformViewFactory;

public final class PlatformViewProbeFactory implements TesseraPlatformViewFactory {
    private static final String TAG = "TesseraViewProbe";
    private final Context context;
    private final int index;

    public PlatformViewProbeFactory(Context context, int index) {
        this.context = context;
        this.index = index;
    }

    @Override
    public View create() {
        if (index == 2) {
            EditText editor = new EditText(context);
            editor.setSingleLine(true);
            editor.setHint("Native editor");
            editor.setId(View.generateViewId());
            editor.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_YES);
            editor.setAutofillHints(View.AUTOFILL_HINT_USERNAME);
            editor.setOnFocusChangeListener((view, focused) -> Log.i(TAG, "editor focus " + focused));
            editor.addTextChangedListener(new TextWatcher() {
                @Override
                public void beforeTextChanged(CharSequence text, int start, int count, int after) {}

                @Override
                public void onTextChanged(CharSequence text, int start, int before, int count) {}

                @Override
                public void afterTextChanged(Editable text) {
                    if ("Hello".contentEquals(text)) Log.i(TAG, "editor typed Hello");
                }
            });
            return editor;
        }
        boolean surfaceBacked = context instanceof Activity
                && (((Activity) context).getIntent().getBooleanExtra(
                        "tessera.platform_view_both_surface_backed", false)
                     || (index == 1 || index == 3) && ((Activity) context).getIntent().getBooleanExtra(
                        "tessera.platform_view_surface_backed", false));
        if (surfaceBacked) {
            ProbeSurfaceView view = new ProbeSurfaceView(context, index);
            view.setContentDescription("Native " + index);
            logLayoutSize(view);
            Log.i(TAG, "create surface-backed " + index);
            return view;
        }
        Button button = new Button(context);
        button.setText("Native " + index);
        button.setBackgroundColor(probeColor(index, false));
        button.setOnTouchListener((view, event) -> {
            Log.i(TAG, "touch " + index + " " + event.getActionMasked());
            return false;
        });
        button.setOnClickListener(view -> Log.i(TAG, "click " + index));
        logLayoutSize(button);
        Log.i(TAG, "create " + index);
        return button;
    }

    static int probeColor(int index, boolean pressed) {
        if (index == 0) return Color.rgb(32, 200, 80);
        if (index == 3) return Color.rgb(30, 160, 240);
        return pressed ? Color.rgb(160, 90, 30) : Color.rgb(240, 150, 30);
    }

    private void logLayoutSize(View view) {
        view.addOnLayoutChangeListener((changed, left, top, right, bottom,
                oldLeft, oldTop, oldRight, oldBottom) -> {
            int width = right - left;
            int height = bottom - top;
            if (width != oldRight - oldLeft || height != oldBottom - oldTop) {
                Log.i(TAG, "layout " + index + " " + width + "x" + height);
            }
        });
    }

    @Override
    public void reset(View view) {
        if (view instanceof ProbeSurfaceView) ((ProbeSurfaceView) view).resetPressed();
        Log.i(TAG, "reset " + index);
    }

    @Override
    public void release(View view) {
        Log.i(TAG, "release " + index);
    }
}

final class ProbeSurfaceView extends SurfaceView implements SurfaceHolder.Callback {
    private static final String TAG = "TesseraViewProbe";
    private final int index;
    private boolean pressed;

    ProbeSurfaceView(Context context, int index) {
        super(context);
        this.index = index;
        getHolder().addCallback(this);
        setClickable(true);
        setFocusable(true);
        setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        drawSurface(holder);
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        drawSurface(holder);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {}

    private void drawSurface(SurfaceHolder holder) {
        Canvas canvas = holder.lockCanvas();
        if (canvas == null) return;
        try {
            canvas.drawColor(PlatformViewProbeFactory.probeColor(index, pressed));
        } finally {
            holder.unlockCanvasAndPost(canvas);
        }
    }

    void resetPressed() {
        if (!pressed) return;
        pressed = false;
        if (getHolder().getSurface().isValid()) drawSurface(getHolder());
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        if (event.getActionMasked() == MotionEvent.ACTION_DOWN) {
            Log.i(TAG, "touch " + index + " " + event.getActionMasked());
            return true;
        }
        if (event.getActionMasked() == MotionEvent.ACTION_UP) {
            Log.i(TAG, "touch " + index + " " + event.getActionMasked());
            performClick();
            return true;
        }
        return true;
    }

    @Override
    public boolean performClick() {
        pressed = !pressed;
        drawSurface(getHolder());
        Log.i(TAG, "click " + index);
        return super.performClick();
    }
}

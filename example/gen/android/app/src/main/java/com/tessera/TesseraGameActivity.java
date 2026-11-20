package com.tessera;

import android.os.Bundle;
import android.os.Build;
import android.graphics.PixelFormat;
import android.graphics.Rect;
import android.graphics.SurfaceTexture;
import android.hardware.DataSpace;
import android.hardware.HardwareBuffer;
import android.hardware.SyncFence;
import android.media.Image;
import android.media.ImageReader;
import android.os.Handler;
import android.os.Looper;
import android.text.Editable;
import android.text.TextWatcher;
import android.util.SparseArray;
import android.view.AttachedSurfaceControl;
import android.view.FocusFinder;
import android.view.KeyEvent;
import android.view.Surface;
import android.view.View;
import android.view.SurfaceControl;
import android.view.SurfaceControlViewHost;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.TextureView;
import android.view.ViewGroup;
import android.view.ViewStructure;
import android.view.ViewTreeObserver;
import android.view.MotionEvent;
import android.view.autofill.AutofillManager;
import android.view.autofill.AutofillValue;
import android.view.inputmethod.InputMethodManager;
import android.widget.FrameLayout;
import android.widget.TextView;
import android.window.InputTransferToken;
import android.window.SurfaceSyncGroup;

import java.io.IOException;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.ArrayList;
import java.util.List;
import java.util.WeakHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.Executor;
import java.util.concurrent.Executors;

import androidx.activity.BackEventCompat;
import androidx.activity.OnBackPressedCallback;
import androidx.annotation.Keep;
import androidx.core.view.WindowCompat;

import com.google.androidgamesdk.GameActivity;

public class TesseraGameActivity extends GameActivity {
    private static final int FIRST_TESSERA_PLATFORM_LAYER = 10_000;
    private static final int PLATFORM_VIEW_GEOMETRY_STRIDE = 11;
    private static final int MAX_ACQUIRED_SLICE_IMAGES = 3;
    private static final Executor SCENE_BUFFER_RELEASES = Executors.newSingleThreadExecutor();
    private final Map<Long, HostedPlatformView> tesseraPlatformViews = new HashMap<>();
    private final List<HostedSliceSurface[]> tesseraSliceSurfaces = new CopyOnWriteArrayList<>();
    private final List<FrameLayout> legacySliceContainers = new ArrayList<>();
    private final List<View> legacyChildOrder = new ArrayList<>();
    private final ViewTreeObserver.OnDrawListener legacySurfaceOrderListener =
            this::orderLegacySurfacesOnDraw;
    private final ViewTreeObserver.OnPreDrawListener legacySurfaceOrderPreDrawListener = () -> {
        orderLegacySurfacesOnDraw();
        return true;
    };
    private long[] legacySurfaceOrder = new long[0];
    private boolean legacySurfaceOrderPending;
    private List<HostedInputRegion> tesseraInputBlockers = new ArrayList<>();
    private List<HostedInputRegion> legacyNativeHitRegions = new ArrayList<>();
    private List<HostedInputRegion> pendingInputBlockers;
    private List<HostedInputRegion> pendingLegacyNativeHitRegions;
    private FrameLayout accessibilityAnchors;
    private FrameLayout legacyCompositionRoot;
    private SurfaceView legacySurfaceOrderRoot;
    private SurfaceControl legacyHiddenPrimary;
    private long tesseraPlatformViewFrame;
    private long tesseraPendingCompositionHost;
    private long tesseraFocusHost;
    private long reportedNativeFocus;
    private long lastNativePlatformFocus;
    private long[] reportedNativeFocusables = new long[0];
    private boolean focusReportQueued;
    private boolean tesseraCompositionStopped;
    private Runnable pendingLegacyComposition;
    private boolean legacyLatchCheckScheduled;
    private Runnable pendingStagedComposition;
    private Image[] pendingStagedImages;
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
        getWindow().setFormat(PixelFormat.TRANSLUCENT);
        super.onCreate(savedInstanceState);
        mSurfaceView.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
        mSurfaceView.setZOrderOnTop(Build.VERSION.SDK_INT >= 35);
        getOnBackPressedDispatcher().addCallback(this, backCallback);
    }

    @Keep
    public static void registerTesseraPlatformView(
            android.app.Activity activity, long id, TesseraPlatformViewFactory factory) {
        ((TesseraGameActivity) activity).registerPlatformView(id, factory);
    }

    private void registerPlatformView(long id, TesseraPlatformViewFactory factory) {
        runOnUiThread(() -> {
            if (isDestroyed()) return;
            if (Build.VERSION.SDK_INT < 30 || mSurfaceView == null
                    || mSurfaceView.getSurfaceControl() == null) {
                throw new UnsupportedOperationException(
                        "Tessera native platform views require Android 11 or newer and an attached GameActivity surface");
            }
            if (Build.VERSION.SDK_INT >= 35 && mSurfaceView.getRootSurfaceControl() == null) {
                throw new IllegalStateException("GameActivity root SurfaceControl is unavailable");
            }
            HostedPlatformView existing = tesseraPlatformViews.get(id);
            if (existing != null && !existing.retiring) return;
            HostedPlatformView previous = tesseraPlatformViews.remove(id);
            if (previous != null) previous.release();
            View view = factory.create();
            if (view == null) throw new IllegalStateException("Platform-view factory returned null");
            if (Build.VERSION.SDK_INT < 35) {
                ensureLegacyCompositionHost();
                FrameLayout wrapper = new PlatformViewInputHost(id);
                wrapper.setClipChildren(true);
                wrapper.setVisibility(View.INVISIBLE);
                FrameLayout placement = new FrameLayout(this);
                if (view instanceof SurfaceView) {
                    // Compose above WGPU but below the window's TextureView overlays.
                    ((SurfaceView) view).setZOrderMediaOverlay(true);
                    ensureLegacySurfaceOrderRoot();
                }
                legacyCompositionRoot.addView(wrapper, new FrameLayout.LayoutParams(1, 1));
                wrapper.addView(placement, new FrameLayout.LayoutParams(1, 1));
                placement.addView(view, new FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
                tesseraPlatformViews.put(id, new HostedPlatformView(
                        factory, view, wrapper, placement, null, null, null, null));
                return;
            }
            AttachedSurfaceControl root = mSurfaceView.getRootSurfaceControl();
            InputTransferToken inputToken = root.getInputTransferToken();
            if (inputToken == null) {
                throw new IllegalStateException("GameActivity root has no input-transfer token");
            }
            SurfaceControlViewHost host = new SurfaceControlViewHost(this, getDisplay(), inputToken);
            PlatformViewInputHost inputHost = new PlatformViewInputHost(id);
            FrameLayout placement = new FrameLayout(this);
            inputHost.addView(placement, new FrameLayout.LayoutParams(1, 1));
            placement.addView(view, new FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.MATCH_PARENT,
                    FrameLayout.LayoutParams.MATCH_PARENT));
            host.setView(inputHost, 1, 1);
            SurfaceControlViewHost.SurfacePackage surfacePackage = host.getSurfacePackage();
            SurfaceControl control = surfacePackage.getSurfaceControl();
            PlatformViewAccessibilityAnchor anchor = new PlatformViewAccessibilityAnchor(surfacePackage, view);
            if (accessibilityAnchors == null) {
                accessibilityAnchors = new FrameLayout(this);
                addContentView(accessibilityAnchors, new ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            }
            accessibilityAnchors.addView(anchor, new FrameLayout.LayoutParams(1, 1));
            tesseraPlatformViews.put(id, new HostedPlatformView(
                    factory, view, null, placement, host, surfacePackage, control, anchor));
        });
    }

    @Keep
    public static void unregisterTesseraPlatformView(android.app.Activity activity, long id) {
        ((TesseraGameActivity) activity).unregisterPlatformView(id);
    }

    private void unregisterPlatformView(long id) {
        runOnUiThread(() -> {
            HostedPlatformView view = tesseraPlatformViews.get(id);
            if (view == null) return;
            // An older submitted frame may still name this view. Keep drawing
            // it until a frame that omits it commits with the new Tessera slices.
            view.retiring = true;
            releaseRetiredViews();
        });
    }

    private void releaseRetiredViews() {
        List<HostedPlatformView> released = new ArrayList<>();
        tesseraPlatformViews.values().removeIf(view -> {
            if (!view.retiring || (!tesseraCompositionStopped && (view.committed || view.staged))) return false;
            released.add(view);
            return true;
        });
        for (HostedPlatformView view : released) view.release();
    }

    @Keep
    public boolean stagesTesseraSceneBuffers() {
        return Build.VERSION.SDK_INT >= 30;
    }

    @Keep
    public Surface getTesseraSliceSurface(int index, int slot, int width, int height) {
        if (Build.VERSION.SDK_INT < 30 || mSurfaceView == null
                || mSurfaceView.getSurfaceControl() == null) {
            throw new UnsupportedOperationException(
                    "Tessera compositor slices require Android 11 or newer and an attached GameActivity surface");
        }
        if (index < 0 || slot < 0 || slot > 1 || width < 1 || height < 1) {
            throw new IllegalArgumentException("Slice index, slot, and extent must be valid");
        }
        if (Build.VERSION.SDK_INT < 35) {
            while (tesseraSliceSurfaces.size() <= index) {
                HostedSliceSurface[] slots = new HostedSliceSurface[2];
                for (int bufferSlot = 0; bufferSlot < slots.length; bufferSlot++) {
                    SliceTexture textureSurface = new SliceTexture();
                    textureSurface.setDefaultBufferSize(width, height);
                    slots[bufferSlot] = new HostedSliceSurface(
                            null, new Surface(textureSurface), width, height, textureSurface, null);
                }
                tesseraSliceSurfaces.add(slots);
                // Enqueue attachment before the frame that consumes these buffers. Blocking
                // here can deadlock against GameActivity's native surface lifecycle callbacks.
                runOnUiThread(() -> attachLegacySlice(slots));
            }
            HostedSliceSurface hosted = tesseraSliceSurfaces.get(index)[slot];
            if (hosted.width != width || hosted.height != height) {
                hosted.width = width;
                hosted.height = height;
                hosted.textureSurface.setDefaultBufferSize(width, height);
            }
            return hosted.surface;
        }
        while (tesseraSliceSurfaces.size() <= index) {
            int sliceIndex = tesseraSliceSurfaces.size();
            HostedSliceSurface[] slots = new HostedSliceSurface[2];
            String name = sliceIndex == 0 ? "Tessera scene base"
                    : "Tessera scene slice " + (sliceIndex - 1);
            SurfaceControl control = new SurfaceControl.Builder()
                    .setName(name)
                    .setParent(mSurfaceView.getSurfaceControl())
                    .setHidden(true)
                    .build();
            for (int bufferSlot = 0; bufferSlot < slots.length; bufferSlot++) {
                ImageReader reader = new ImageReader.Builder(width, height)
                        .setMaxImages(MAX_ACQUIRED_SLICE_IMAGES)
                        .setDefaultHardwareBufferFormat(HardwareBuffer.RGBA_8888)
                        .setDefaultDataSpace(DataSpace.DATASPACE_SRGB)
                        .setUsage(HardwareBuffer.USAGE_GPU_SAMPLED_IMAGE
                                | HardwareBuffer.USAGE_COMPOSER_OVERLAY)
                        .build();
                reader.setOnImageAvailableListener(ignored -> tryStagedComposition(),
                        new Handler(Looper.getMainLooper()));
                slots[bufferSlot] = new HostedSliceSurface(
                        bufferSlot == 0 ? control : null, reader.getSurface(), width, height,
                        null, reader);
            }
            tesseraSliceSurfaces.add(slots);
        }
        return tesseraSliceSurfaces.get(index)[slot].surface;
    }

    private void attachLegacySlice(HostedSliceSurface[] slots) {
        if (isDestroyed()) return;
        ensureLegacyCompositionHost();
        FrameLayout container = new FrameLayout(this);
        container.setClipChildren(false);
        for (int index = 0; index < slots.length; index++) {
            HostedSliceSurface slot = slots[index];
            TextureView texture = new TextureView(this);
            texture.setOpaque(false);
            texture.setSurfaceTexture(slot.textureSurface);
            texture.setAlpha(0f);
            container.addView(texture, new FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.MATCH_PARENT, FrameLayout.LayoutParams.MATCH_PARENT));
            slot.textureView = texture;
        }
        container.setVisibility(View.INVISIBLE);
        legacyCompositionRoot.addView(container, new FrameLayout.LayoutParams(1, 1));
        legacySliceContainers.add(container);
    }

    private void ensureLegacyCompositionHost() {
        if (legacyCompositionRoot != null) return;
        legacyCompositionRoot = new LegacyCompositionRoot();
        legacyCompositionRoot.setClipChildren(true);
        legacyCompositionRoot.setClipToPadding(true);
        addContentView(legacyCompositionRoot, new ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        android.util.Log.i("Tessera", "Using TextureView composition fallback");
    }

    private void ensureLegacySurfaceOrderRoot() {
        if (legacySurfaceOrderRoot != null) return;
        SurfaceView root = new SurfaceView(this);
        root.setZOrderMediaOverlay(true);
        root.getHolder().setFormat(PixelFormat.TRANSLUCENT);
        root.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        legacyCompositionRoot.addView(root, 0, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        legacySurfaceOrderRoot = root;
        ViewTreeObserver observer = getWindow().getDecorView().getViewTreeObserver();
        if (Build.VERSION.SDK_INT >= 34) {
            // Android 14 captures pending transactions before dispatching OnDraw.
            observer.addOnPreDrawListener(legacySurfaceOrderPreDrawListener);
        } else {
            observer.addOnDrawListener(legacySurfaceOrderListener);
        }
    }

    private void orderLegacySurfacesOnDraw() {
        if (tesseraCompositionStopped || legacySurfaceOrder.length == 0) return;
        // Android 12/12L may reassign a SurfaceView's relative layer on a later
        // traversal, including one with no new Tessera frame. Run after Android's
        // pre-draw listeners and merge the layer order with the window draw.
        if (!legacySurfaceOrderPending && Build.VERSION.SDK_INT > 32) return;
        View decor = getWindow().getDecorView();
        SurfaceControl parent = legacySurfaceOrderRoot.getSurfaceControl();
        if (parent == null || !parent.isValid()) return;
        SurfaceControl.Transaction transaction = null;
        for (int index = 0; index < legacySurfaceOrder.length; index++) {
            HostedPlatformView hosted = tesseraPlatformViews.get(legacySurfaceOrder[index]);
            if (hosted == null || !hosted.staged || !(hosted.view instanceof SurfaceView)) continue;
            SurfaceControl control = ((SurfaceView) hosted.view).getSurfaceControl();
            if (control != null && control.isValid()) {
                if (transaction == null) transaction = new SurfaceControl.Transaction();
                transaction.reparent(control, parent).setLayer(control, index);
            }
        }
        if (transaction == null) return;
        if (!applyLegacyTransactionOnDraw(decor, transaction)) {
            try (SurfaceControl.Transaction pending = transaction) {
                pending.apply();
            }
        }
        legacySurfaceOrderPending = false;
    }

    private void alignLegacySurfaceDefaults() {
        if (Build.VERSION.SDK_INT > 32 || legacySurfaceOrderRoot == null) return;
        boolean front = true;
        for (int index = legacySurfaceOrder.length - 1; index >= 0; index--) {
            HostedPlatformView hosted = tesseraPlatformViews.get(legacySurfaceOrder[index]);
            if (hosted == null || !hosted.committed || !(hosted.view instanceof SurfaceView)) continue;
            // If Android reapplies a View's media sublayer during layout, keep
            // the committed front surface ahead of lower surfaces. Set this
            // after the commit: it must not anticipate the next scene order.
            if (hosted.legacyMediaOverlay != front) {
                ((SurfaceView) hosted.view).setZOrderMediaOverlay(front);
                hosted.legacyMediaOverlay = front;
            }
            front = false;
        }
    }

    private void updateLegacyComposition(
            long[] ids, float[] bounds, float[] opacity, boolean[] visible,
            int[] overlaysAfter, int sliceSlot) {
        if (legacyCompositionRoot == null) return;
        int width = Math.max(1, mSurfaceView.getWidth());
        int height = Math.max(1, mSurfaceView.getHeight());

        for (Map.Entry<Long, HostedPlatformView> entry : tesseraPlatformViews.entrySet()) {
            HostedPlatformView view = entry.getValue();
            view.staged = false;
            boolean placed = false;
            for (long id : ids) placed |= id == entry.getKey();
            if (!placed && view.legacyContainer != null) {
                view.legacyContainer.setVisibility(View.INVISIBLE);
            }
        }
        for (int i = overlaysAfter.length + 1; i < legacySliceContainers.size(); i++) {
            legacySliceContainers.get(i).setVisibility(View.INVISIBLE);
        }

        List<View> order = new ArrayList<>(1 + ids.length + overlaysAfter.length);
        List<HostedInputRegion> hitRegions = new ArrayList<>(ids.length);
        order.add(showLegacySlice(0, sliceSlot, width, height));
        int overlay = 0;
        for (int i = 0; i < ids.length; i++) {
            HostedPlatformView hosted = tesseraPlatformViews.get(ids[i]);
            if (hosted != null) {
                int offset = i * PLATFORM_VIEW_GEOMETRY_STRIDE;
                placeLegacyView(hosted, bounds, offset, opacity[i], visible[i]);
                order.add(hosted.legacyContainer);
                float left = bounds[offset] + Math.max(0, bounds[offset + 4]);
                float top = bounds[offset + 1] + Math.max(0, bounds[offset + 5]);
                float right = bounds[offset] + Math.min(bounds[offset + 2], bounds[offset + 6]);
                float bottom = bounds[offset + 1] + Math.min(bounds[offset + 3], bounds[offset + 7]);
                if (visible[i] && opacity[i] > 0 && right > left && bottom > top) {
                    hosted.staged = true;
                    hitRegions.add(new HostedInputRegion(
                            ids[i], left, top, right - left, bottom - top));
                }
            }

            if (overlay < overlaysAfter.length && overlaysAfter[overlay] == i) {
                order.add(showLegacySlice(++overlay, sliceSlot, width, height));
            }
        }
        for (View child : order) {
            if (child.getParent() == null) legacyCompositionRoot.addView(child);
            else if (!legacyChildOrder.equals(order)) legacyCompositionRoot.bringChildToFront(child);
        }
        if (!legacyChildOrder.equals(order)) {
            legacyChildOrder.clear();
            legacyChildOrder.addAll(order);
        }
        pendingLegacyNativeHitRegions = hitRegions;
    }

    private FrameLayout showLegacySlice(int index, int sliceSlot, int width, int height) {
        FrameLayout container = legacySliceContainers.get(index);
        setViewLayout(container, 0, 0, width, height);
        for (int slot = 0; slot < container.getChildCount(); slot++) {
            View view = container.getChildAt(slot);
            view.setAlpha(slot == sliceSlot ? 1f : 0f);
        }
        container.setVisibility(View.VISIBLE);
        return container;
    }

    private void placeLegacyView(
            HostedPlatformView hosted, float[] bounds, int offset, float opacity, boolean visible) {
        float left = bounds[offset];
        float top = bounds[offset + 1];
        int clipLeft = Math.round(left + bounds[offset + 4]);
        int clipTop = Math.round(top + bounds[offset + 5]);
        int clipRight = Math.round(left + bounds[offset + 6]);
        int clipBottom = Math.round(top + bounds[offset + 7]);
        int clipWidth = Math.max(0, clipRight - clipLeft);
        int clipHeight = Math.max(0, clipBottom - clipTop);

        setViewLayout(hosted.legacyContainer, clipLeft, clipTop, clipWidth, clipHeight);
        placeNativeContent(hosted.placement, bounds, offset, left - clipLeft, top - clipTop);
        hosted.update();
        hosted.legacyContainer.setAlpha(opacity);
        hosted.legacyContainer.setVisibility(
                visible && clipWidth > 0 && clipHeight > 0 ? View.VISIBLE : View.INVISIBLE);
    }

    private static void placeNativeContent(
            FrameLayout placement, float[] bounds, int offset, float left, float top) {
        setViewLayout(placement, 0, 0, (int) bounds[offset + 8], (int) bounds[offset + 9]);
        placement.setPivotX(0);
        placement.setPivotY(0);
        placement.setScaleX(bounds[offset + 10]);
        placement.setScaleY(bounds[offset + 10]);
        placement.setTranslationX(left);
        placement.setTranslationY(top);
    }

    private static void setViewLayout(View view, int left, int top, int width, int height) {
        FrameLayout.LayoutParams params = (FrameLayout.LayoutParams) view.getLayoutParams();
        if (params.width == width && params.height == height
                && params.leftMargin == left && params.topMargin == top) return;
        params.width = width;
        params.height = height;
        params.leftMargin = left;
        params.topMargin = top;
        view.setLayoutParams(params);
    }

    private final class LegacyCompositionRoot extends FrameLayout {
        private final int[] surfaceLocation = new int[2];
        private boolean relaying;

        LegacyCompositionRoot() {
            super(TesseraGameActivity.this);
        }

        @Override
        public boolean dispatchTouchEvent(MotionEvent event) {
            int action = event.getActionMasked();
            // A Tessera gesture must survive switching the displayed TextureView slot.
            if (action == MotionEvent.ACTION_DOWN) {
                mSurfaceView.getLocationOnScreen(surfaceLocation);
                relaying = shouldRelayLegacyTouch(event.getRawX() - surfaceLocation[0],
                        event.getRawY() - surfaceLocation[1]);
            } else if (relaying) {
                mSurfaceView.getLocationOnScreen(surfaceLocation);
            }
            if (!relaying) return super.dispatchTouchEvent(event);

            MotionEvent forwarded = MotionEvent.obtain(event);
            try {
                forwarded.offsetLocation(event.getRawX() - event.getX() - surfaceLocation[0],
                        event.getRawY() - event.getY() - surfaceLocation[1]);
                mSurfaceView.dispatchTouchEvent(forwarded);
            } finally {
                forwarded.recycle();
            }
            if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_CANCEL) relaying = false;
            return true;
        }
    }

    private boolean shouldRelayLegacyTouch(float x, float y) {
        for (int i = legacyNativeHitRegions.size() - 1; i >= 0; i--) {
            HostedInputRegion nativeView = legacyNativeHitRegions.get(i);
            if (nativeView.contains(x, y)) {
                return isTesseraPlatformViewInputBlocked(nativeView.platformViewId, x, y);
            }
        }
        return true;
    }

    @Keep
    public void setTesseraPlatformViewFrames(
            long host, long frame, long[] ids, float[] bounds, float[] opacity, boolean[] visible,
            long[] blockerIds, float[] blockerBounds, int[] overlaysAfter, int sliceSlot) {
        runOnUiThread(() -> {
            if (isDestroyed() || frame <= tesseraPlatformViewFrame
                    || ids.length * PLATFORM_VIEW_GEOMETRY_STRIDE != bounds.length
                    || ids.length != opacity.length || ids.length != visible.length
                    || blockerBounds.length != blockerIds.length * 4
                    || overlaysAfter.length + 1 > tesseraSliceSurfaces.size()
                    || sliceSlot < 0 || sliceSlot > 1) {
                dispatchTesseraPlatformViewFrameCommitted(host, frame);
                return;
            }
            tesseraPlatformViewFrame = frame;
            tesseraPendingCompositionHost = host;
            Runnable acknowledge = () -> acknowledgeComposition(frame);
            pendingInputBlockers = new ArrayList<>(blockerIds.length);
            for (int i = 0; i < blockerIds.length; i++) {
                pendingInputBlockers.add(new HostedInputRegion(
                        blockerIds[i], blockerBounds[i * 4], blockerBounds[i * 4 + 1],
                        blockerBounds[i * 4 + 2], blockerBounds[i * 4 + 3]));
            }
            if (Build.VERSION.SDK_INT < 35) {
                pendingLegacyComposition = () -> {
                    if (tesseraCompositionStopped) return;
                    int count = overlaysAfter.length + 1;
                    for (int i = 0; i < count; i++) {
                        if (!tesseraSliceSurfaces.get(i)[sliceSlot].textureSurface.hasFrame) return;
                    }
                    if (!legacySlicesLatched(count, sliceSlot)) return;
                    for (int i = 0; i < count; i++) {
                        SliceTexture texture = tesseraSliceSurfaces.get(i)[sliceSlot].textureSurface;
                        texture.hasFrame = false;
                        texture.presentedTimestamp = texture.getTimestamp();
                    }
                    pendingLegacyComposition = null;
                    updateLegacyComposition(ids, bounds, opacity, visible, overlaysAfter, sliceSlot);
                    legacySurfaceOrder = ids;
                    legacySurfaceOrderPending = true;
                    View decor = getWindow().getDecorView();
                    if (!tesseraCompositionStopped && decor.isHardwareAccelerated() && decor.isShown()) {
                        hideLegacyPrimaryOnDraw(decor);
                        decor.getViewTreeObserver().registerFrameCommitCallback(acknowledge);
                        decor.postInvalidateOnAnimation();
                    } else {
                        acknowledge.run();
                    }
                };
                tryLegacyComposition();
                return;
            }
            Image[] images = new Image[overlaysAfter.length + 1];
            pendingStagedImages = images;
            pendingStagedComposition = () -> {
                for (int i = 0; i < images.length; i++) {
                    if (images[i] == null) {
                        HostedSliceSurface target = tesseraSliceSurfaces.get(i)[sliceSlot];
                        if (target.acquiredImages == MAX_ACQUIRED_SLICE_IMAGES) return;
                        images[i] = target.reader.acquireNextImage();
                        if (images[i] == null) return;
                        target.acquiredImages++;
                    }
                }
                pendingStagedComposition = null;
                pendingStagedImages = null;
                applyStagedComposition(ids, bounds, opacity, visible, overlaysAfter,
                        sliceSlot, images, acknowledge);
            };
            tryStagedComposition();
        });
    }

    private void tryLegacyComposition() {
        if (pendingLegacyComposition != null) pendingLegacyComposition.run();
    }

    private boolean legacySlicesLatched(int count, int slot) {
        boolean ready = true;
        for (int i = 0; i < count; i++) {
            HostedSliceSurface slice = tesseraSliceSurfaces.get(i)[slot];
            if (slice.textureSurface.getTimestamp() == slice.textureSurface.presentedTimestamp) {
                FrameLayout container = legacySliceContainers.get(i);
                setViewLayout(container, 0, 0, Math.max(1, mSurfaceView.getWidth()),
                        Math.max(1, mSurfaceView.getHeight()));
                container.setVisibility(View.VISIBLE);
                slice.textureView.invalidate();
                ready = false;
            }
        }
        if (!ready && !legacyLatchCheckScheduled) {
            // A queued SurfaceTexture image is not necessarily latched yet. Let HWUI
            // consume it behind alpha zero before exposing any part of the new scene.
            View decor = getWindow().getDecorView();
            legacyLatchCheckScheduled = true;
            decor.getViewTreeObserver().registerFrameCommitCallback(() -> {
                legacyLatchCheckScheduled = false;
                tryLegacyComposition();
            });
            decor.postInvalidateOnAnimation();
        }
        return ready;
    }

    private void hideLegacyPrimaryOnDraw(View decor) {
        SurfaceControl primary = mSurfaceView.getSurfaceControl();
        if (primary == null || !primary.isValid() || primary == legacyHiddenPrimary) return;
        legacyHiddenPrimary = primary;
        // Keep GameActivity's View and input channel. Only its old, separately queued
        // buffer must disappear when TextureView takes over the base slice.
        SurfaceControl.Transaction transaction = new SurfaceControl.Transaction();
        transaction.setVisibility(primary, false);
        if (applyLegacyTransactionOnDraw(decor, transaction)) return;
        transaction.close();
        decor.getViewTreeObserver().registerFrameCommitCallback(() -> {
            if (!primary.isValid()) return;
            try (SurfaceControl.Transaction pending = new SurfaceControl.Transaction()) {
                pending.setVisibility(primary, false).apply();
            }
        });
    }

    private static boolean applyLegacyTransactionOnDraw(
            View view, SurfaceControl.Transaction transaction) {
        if (Build.VERSION.SDK_INT < 31) return false;
        AttachedSurfaceControl root = view.getRootSurfaceControl();
        if (root == null || !root.applyTransactionOnDraw(transaction)) return false;
        // Android 12 and 13 keep the original transaction in a RenderThread callback.
        // Let Android's native allocation registry release it once that reference is gone.
        if (Build.VERSION.SDK_INT >= 34) {
            transaction.close();
        }
        return true;
    }

    private void tryStagedComposition() {
        if (pendingStagedComposition != null) pendingStagedComposition.run();
    }

    private void applyStagedComposition(long[] ids, float[] bounds, float[] opacity,
            boolean[] visible, int[] overlaysAfter, int sliceSlot, Image[] images,
            Runnable acknowledge) {
        SurfaceControl.Transaction transaction = new SurfaceControl.Transaction();
        SurfaceSyncGroup resizeGroup = null;
        int layer = FIRST_TESSERA_PLATFORM_LAYER;
        for (int i = 0; i < ids.length; i++, layer += 2) {
            HostedPlatformView hosted = tesseraPlatformViews.get(ids[i]);
            if (hosted == null) continue;
            int offset = i * PLATFORM_VIEW_GEOMETRY_STRIDE;
            float left = bounds[offset];
            float top = bounds[offset + 1];
            int width = Math.max(0, (int) Math.ceil(bounds[offset + 2]));
            int height = Math.max(0, (int) Math.ceil(bounds[offset + 3]));
            android.graphics.Rect crop = new android.graphics.Rect(
                    Math.max(0, Math.round(bounds[offset + 4])),
                    Math.max(0, Math.round(bounds[offset + 5])),
                    Math.min(width, Math.round(bounds[offset + 6])),
                    Math.min(height, Math.round(bounds[offset + 7])));
            int hostWidth = Math.max(1, width);
            int hostHeight = Math.max(1, height);
            boolean show = visible[i] && !crop.isEmpty();
            ViewGroup.LayoutParams placementSize = hosted.placement.getLayoutParams();
            if (show && (!hosted.committed || hosted.hostWidth != hostWidth
                    || hosted.hostHeight != hostHeight || placementSize.width != (int) bounds[offset + 8]
                    || placementSize.height != (int) bounds[offset + 9]
                    || hosted.placement.getScaleX() != bounds[offset + 10])) {
                if (resizeGroup == null) resizeGroup = new SurfaceSyncGroup("Tessera native layout");
                // A windowless ViewRoot can otherwise draw its resized content after the
                // Activity has already exposed the new crop and Tessera buffers.
                if (!resizeGroup.add(hosted.surfacePackage, null)) {
                    android.util.Log.w("Tessera", "Could not synchronize a platform-view layout");
                }
            }
            hosted.update();
            if (hosted.hostWidth != hostWidth || hosted.hostHeight != hostHeight) {
                hosted.host.relayout(hostWidth, hostHeight);
                hosted.hostWidth = hostWidth;
                hosted.hostHeight = hostHeight;
            }
            // Scale the view tree so layout and accessibility keep their own coordinate spaces.
            placeNativeContent(hosted.placement, bounds, offset, 0, 0);
            hosted.staged = show;
            int anchorLeft = Math.round(left);
            int anchorTop = Math.round(top);
            hosted.accessibilityAnchor.place(transaction, layer + 1,
                    anchorLeft, anchorTop, width, height, crop, show);
            transaction
                    .setPosition(hosted.control, left - anchorLeft, top - anchorTop)
                    .setAlpha(hosted.control, opacity[i])
                    .setVisibility(hosted.control, show);
            if (!crop.isEmpty()) transaction.setCrop(hosted.control, crop);
        }
        SurfaceControl parent = mSurfaceView.getSurfaceControl();
        for (int i = 0; i < tesseraSliceSurfaces.size(); i++) {
            HostedSliceSurface[] slots = tesseraSliceSurfaces.get(i);
            SurfaceControl control = slots[0].control;
            if (i < images.length) {
                int z = i == 0 ? 1 : FIRST_TESSERA_PLATFORM_LAYER
                        + (overlaysAfter[i - 1] + 1) * 2;
                transaction.reparent(control, parent).setLayer(control, z);
                setStagedBuffer(transaction, control, images[i], slots[sliceSlot]);
                transaction.setVisibility(control, true);
            } else {
                transaction.setVisibility(control, false);
            }
        }
        for (Map.Entry<Long, HostedPlatformView> entry : tesseraPlatformViews.entrySet()) {
            boolean placed = false;
            for (long id : ids) placed |= id == entry.getKey();
            if (!placed) {
                HostedPlatformView hidden = entry.getValue();
                hidden.staged = false;
                hidden.accessibilityAnchor.setCompositionVisibility(false);
                transaction.setVisibility(hidden.control, false);
            }
        }
        commitPlatformViewComposition(transaction, acknowledge, resizeGroup);
    }

    private void setStagedBuffer(SurfaceControl.Transaction transaction,
            SurfaceControl control, Image image, HostedSliceSurface target) {
        try (HardwareBuffer buffer = image.getHardwareBuffer(); SyncFence ready = image.getFence()) {
            transaction.setDataSpace(control, image.getDataSpace());
            transaction.setBuffer(control, buffer, ready, release -> SCENE_BUFFER_RELEASES.execute(() -> {
                // Android 15's ImageReader does not implement Image.setFence().
                try (SyncFence fence = release) {
                    if (fence.isValid()) fence.awaitForever();
                } finally {
                    image.close();
                    getMainExecutor().execute(() -> {
                        target.acquiredImages--;
                        tryStagedComposition();
                    });
                }
            }));
        } catch (IOException error) {
            image.close();
            target.acquiredImages--;
            throw new IllegalStateException("Could not acquire scene buffer fence", error);
        }
    }

    @Keep
    private native void dispatchTesseraPlatformViewFrameCommitted(long host, long frame);

    @Keep
    private native void dispatchTesseraPlatformViewFocus(
            long host, long focused, long rejected, long[] focusable);

    private void schedulePlatformFocusReport() {
        if (focusReportQueued || tesseraFocusHost == 0) return;
        focusReportQueued = true;
        getMainExecutor().execute(() -> {
            focusReportQueued = false;
            reportPlatformFocus(0);
        });
    }

    private void reportPlatformFocus(long rejected) {
        if (tesseraFocusHost == 0) return;
        long focused = 0;
        long[] focusables = new long[tesseraPlatformViews.size()];
        int count = 0;
        for (Map.Entry<Long, HostedPlatformView> entry : tesseraPlatformViews.entrySet()) {
            HostedPlatformView hosted = entry.getValue();
            if (!hosted.committed || hosted.retiring || !hosted.placement.hasFocusable()) continue;
            long id = entry.getKey();
            focusables[count++] = id;
            if (hosted.placement.hasFocus() && hosted.placement.hasWindowFocus()) focused = id;
        }
        if (Build.VERSION.SDK_INT >= 35 && focused == 0 && !hasWindowFocus()
                && !tesseraCompositionStopped) {
            // System UI can take window focus without changing the active editor.
            HostedPlatformView previous = tesseraPlatformViews.get(lastNativePlatformFocus);
            if (previous != null && previous.committed && !previous.retiring
                    && previous.placement.hasFocus()) {
                focused = lastNativePlatformFocus;
            }
        }
        focusables = Arrays.copyOf(focusables, count);
        Arrays.sort(focusables);
        if (focused != 0) {
            HostedPlatformView previous = tesseraPlatformViews.get(lastNativePlatformFocus);
            if (previous != null && lastNativePlatformFocus != focused
                    && previous.accessibilityAnchor != null) {
                previous.accessibilityAnchor.syncAutofillFocus(null);
            }
            lastNativePlatformFocus = focused;
        }
        if (rejected == 0 && focused == reportedNativeFocus
                && Arrays.equals(focusables, reportedNativeFocusables)) return;
        reportedNativeFocus = focused;
        reportedNativeFocusables = focusables;
        dispatchTesseraPlatformViewFocus(tesseraFocusHost, focused, rejected, focusables);
    }

    @Keep
    public void requestTesseraPlatformViewFocus(long id, boolean backwards) {
        runOnUiThread(() -> {
            HostedPlatformView hosted = tesseraPlatformViews.get(id);
            if (id == 0) {
                HostedPlatformView previous = tesseraPlatformViews.get(lastNativePlatformFocus);
                if (previous != null) {
                    if (previous.accessibilityAnchor != null) {
                        previous.accessibilityAnchor.syncAutofillFocus(null);
                    }
                    mSurfaceView.setFocusableInTouchMode(true);
                    mSurfaceView.requestFocus();
                    if (previous.accessibilityAnchor != null) {
                        ((PlatformViewInputHost) previous.placement.getParent()).clearEmbeddedFocus();
                    }
                }
                lastNativePlatformFocus = 0;
                reportPlatformFocus(0);
                return;
            }
            if (hosted == null || !hosted.committed || hosted.retiring) {
                reportPlatformFocus(id);
                return;
            }
            if (reportedNativeFocus == id) return;
            int direction = backwards ? View.FOCUS_BACKWARD : View.FOCUS_FORWARD;
            View target = platformFocusEdge(hosted.placement, direction);
            if (target == null || !target.requestFocus(direction)) {
                reportPlatformFocus(id);
                return;
            }
            if (hosted.accessibilityAnchor != null) {
                hosted.accessibilityAnchor.setFocusableInTouchMode(true);
                hosted.accessibilityAnchor.requestFocus(direction);
            }
            reportPlatformFocus(0);
        });
    }

    private static View platformFocusEdge(ViewGroup root, int direction) {
        View edge = FocusFinder.getInstance().findNextFocus(root, null, direction);
        if (edge != null) return edge;
        // FocusFinder can return null when the subtree has exactly one candidate.
        ArrayList<View> focusables = root.getFocusables(direction);
        return focusables.size() == 1 ? focusables.get(0) : null;
    }

    private void acknowledgeComposition(long frame) {
        if (frame == tesseraPlatformViewFrame && tesseraPendingCompositionHost != 0) {
            long host = tesseraPendingCompositionHost;
            tesseraPendingCompositionHost = 0;
            tesseraInputBlockers = pendingInputBlockers;
            pendingInputBlockers = null;
            if (pendingLegacyNativeHitRegions != null) {
                legacyNativeHitRegions = pendingLegacyNativeHitRegions;
                pendingLegacyNativeHitRegions = null;
            }
            for (HostedPlatformView view : tesseraPlatformViews.values()) view.committed = view.staged;
            alignLegacySurfaceDefaults();
            releaseRetiredViews();
            tesseraFocusHost = host;
            reportPlatformFocus(0);
            dispatchTesseraPlatformViewFrameCommitted(host, frame);
        }
    }

    private void commitPlatformViewComposition(
            SurfaceControl.Transaction transaction, Runnable acknowledge, SurfaceSyncGroup resizeGroup) {
        try (SurfaceControl.Transaction pending = transaction) {
            if (acknowledge != null) {
                pending.addTransactionCommittedListener(getMainExecutor(), acknowledge::run);
            }
            AttachedSurfaceControl root = mSurfaceView.getRootSurfaceControl();
            if (resizeGroup != null) {
                // Join the Activity's draw as well: anchor layout and nested SCVH
                // buffers must reach the compositor with the slice transaction.
                resizeGroup.add(root, null);
            }
            // The renderer may have already presented its last buffer. Schedule a ViewRoot
            // frame so the layer transaction cannot remain queued on an idle GameActivity surface.
            if (!tesseraCompositionStopped && root != null && root.applyTransactionOnDraw(pending)) {
                getWindow().getDecorView().postInvalidateOnAnimation();
            } else if (resizeGroup != null) {
                resizeGroup.addTransaction(pending);
            } else {
                pending.apply();
            }
        } finally {
            if (resizeGroup != null) resizeGroup.markSyncReady();
        }
    }

    private final class PlatformViewAccessibilityAnchor extends SurfaceView
            implements SurfaceHolder.Callback {
        private int layer = FIRST_TESSERA_PLATFORM_LAYER;
        private final View nativeView;
        private final Map<View, Integer> autofillIds = new WeakHashMap<>();
        private int nextAutofillId;
        private int autofillFocus;
        private TextView observedAutofillEditor;
        private final TextWatcher autofillEdits = new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence text, int start, int count, int after) {}
            @Override public void onTextChanged(CharSequence text, int start, int before, int count) {}

            @Override public void afterTextChanged(Editable text) {
                AutofillManager manager = getContext().getSystemService(AutofillManager.class);
                if (manager == null || autofillFocus == 0 || observedAutofillEditor == null) return;
                AutofillValue value = observedAutofillEditor.getAutofillValue();
                if (value != null) manager.notifyValueChanged(
                        PlatformViewAccessibilityAnchor.this, autofillFocus, value);
            }
        };

        PlatformViewAccessibilityAnchor(SurfaceControlViewHost.SurfacePackage surfacePackage,
                View nativeView) {
            super(TesseraGameActivity.this);
            this.nativeView = nativeView;
            setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
            setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_YES);
            setZOrderOnTop(true);
            setSurfaceLifecycle(SURFACE_LIFECYCLE_FOLLOWS_ATTACHMENT);
            getHolder().setFormat(PixelFormat.TRANSLUCENT);
            getHolder().addCallback(this);
            setVisibility(View.INVISIBLE);
            // Keep the package in its SurfaceView: moving only the package can leave the
            // accessibility service with a leashed child it cannot resolve.
            setChildSurfacePackage(new SurfaceControlViewHost.SurfacePackage(surfacePackage));
        }

        void place(SurfaceControl.Transaction transaction, int layer, int left, int top,
                int width, int height, android.graphics.Rect crop, boolean visible) {
            this.layer = layer;
            setViewLayout(this, left, top, width, height);
            if (!crop.equals(getClipBounds())) setClipBounds(crop);
            setCompositionVisibility(visible);
            setCompositionLayer(transaction);
        }

        void setCompositionVisibility(boolean visible) {
            if (!visible) syncAutofillFocus(null);
            setVisibility(visible ? View.VISIBLE : View.INVISIBLE);
        }

        // Android's Activity autofill snapshot omits WindowlessWindowManager roots.
        // Publish the real fields under the in-window anchor, with IDs the Activity
        // can resolve when it applies a dataset.
        @Override
        public void onProvideAutofillVirtualStructure(ViewStructure structure, int flags) {
            if (!isShown()) return;
            List<View> fields = new ArrayList<>();
            collectAutofillFields(nativeView, fields);
            int index = structure.addChildCount(fields.size());
            for (View field : fields) {
                ViewStructure child = structure.newChild(index++);
                field.onProvideAutofillStructure(child, flags);
                child.setAutofillId(getAutofillId(), autofillId(field));
                Rect bounds = fieldBounds(field);
                child.setDimens(bounds.left, bounds.top, 0, 0, bounds.width(), bounds.height());
            }
        }

        @Override
        public void autofill(SparseArray<AutofillValue> values) {
            if (!isShown()) return;
            List<View> fields = new ArrayList<>();
            collectAutofillFields(nativeView, fields);
            for (View field : fields) {
                Integer id = autofillIds.get(field);
                AutofillValue value = id == null ? null : values.get(id);
                if (value != null) field.autofill(value);
            }
        }

        private void collectAutofillFields(View view, List<View> fields) {
            int importance = view.getImportantForAutofill();
            if (!view.isShown() || importance == IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS) return;
            if (view.isImportantForAutofill() && view.getAutofillType() != AUTOFILL_TYPE_NONE
                    && !fieldBounds(view).isEmpty()) {
                fields.add(view);
            }
            if (view instanceof ViewGroup && importance != IMPORTANT_FOR_AUTOFILL_YES_EXCLUDE_DESCENDANTS) {
                ViewGroup group = (ViewGroup) view;
                for (int i = 0; i < group.getChildCount(); i++) {
                    collectAutofillFields(group.getChildAt(i), fields);
                }
            }
        }

        private int autofillId(View field) {
            return autofillIds.computeIfAbsent(field, ignored -> ++nextAutofillId);
        }

        private Rect fieldBounds(View field) {
            Rect bounds = new Rect();
            if (!field.getGlobalVisibleRect(bounds)) return new Rect();
            Rect clip = getClipBounds();
            if (clip != null && !bounds.intersect(clip)) bounds.setEmpty();
            return bounds;
        }

        void syncAutofillFocus(View focused) {
            AutofillManager manager = getContext().getSystemService(AutofillManager.class);
            if (manager == null || !manager.isEnabled()) {
                observeAutofillEditor(null);
                autofillFocus = 0;
                return;
            }
            Rect bounds = focused == null || !isShown() || !focused.isImportantForAutofill()
                    || focused.getAutofillType() == AUTOFILL_TYPE_NONE ? new Rect() : fieldBounds(focused);
            int id = bounds.isEmpty() ? 0 : autofillId(focused);
            if (id == autofillFocus) return;
            if (autofillFocus != 0) manager.notifyViewExited(this, autofillFocus);
            autofillFocus = id;
            observeAutofillEditor(id != 0 && focused instanceof TextView ? (TextView) focused : null);
            if (id != 0) {
                int[] screen = new int[2];
                getLocationOnScreen(screen);
                bounds.offset(screen[0], screen[1]);
                manager.notifyViewEntered(this, id, bounds);
            }
        }

        private void observeAutofillEditor(TextView editor) {
            if (observedAutofillEditor != null) {
                observedAutofillEditor.removeTextChangedListener(autofillEdits);
            }
            observedAutofillEditor = editor;
            // Android reports edits to the real view's ID, but the Activity
            // save session owns the anchor's virtual ID instead.
            if (editor != null) editor.addTextChangedListener(autofillEdits);
        }

        private void setCompositionLayer(SurfaceControl.Transaction transaction) {
            SurfaceControl anchor = getSurfaceControl();
            SurfaceControl main = mSurfaceView.getSurfaceControl();
            if (anchor != null && anchor.isValid() && main != null && main.isValid()) {
                // Move the anchor and its embedded hierarchy together, keeping SurfaceView's
                // own package and accessibility connection intact.
                transaction.reparent(anchor, main).setLayer(anchor, layer);
            }
        }

        @Override
        public void surfaceCreated(SurfaceHolder holder) {
            SurfaceControl.Transaction transaction = new SurfaceControl.Transaction();
            setCompositionLayer(transaction);
            commitPlatformViewComposition(transaction, null, null);
        }

        @Override
        public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {}

        @Override
        public void surfaceDestroyed(SurfaceHolder holder) {}
    }

    private static final class HostedPlatformView {
        final TesseraPlatformViewFactory factory;
        final View view;
        final FrameLayout legacyContainer;
        final FrameLayout placement;
        final SurfaceControlViewHost host;
        final SurfaceControlViewHost.SurfacePackage surfacePackage;
        final SurfaceControl control;
        final PlatformViewAccessibilityAnchor accessibilityAnchor;
        int hostWidth = 1;
        int hostHeight = 1;
        boolean retiring;
        boolean staged;
        boolean committed;
        boolean legacyMediaOverlay = true;

        HostedPlatformView(TesseraPlatformViewFactory factory, View view, FrameLayout legacyContainer,
                FrameLayout placement, SurfaceControlViewHost host,
                SurfaceControlViewHost.SurfacePackage surfacePackage, SurfaceControl control,
                PlatformViewAccessibilityAnchor accessibilityAnchor) {
            this.factory = factory;
            this.view = view;
            this.legacyContainer = legacyContainer;
            this.placement = placement;
            this.host = host;
            this.surfacePackage = surfacePackage;
            this.control = control;
            this.accessibilityAnchor = accessibilityAnchor;
        }

        void update() {
            if (!retiring) factory.update(view);
        }

        void release() {
            View focused = placement.findFocus();
            if (focused != null) focused.clearFocus();
            factory.reset(view);
            placement.removeView(view);
            if (legacyContainer != null) {
                ViewGroup parent = (ViewGroup) legacyContainer.getParent();
                if (parent != null) parent.removeView(legacyContainer);
            } else {
                accessibilityAnchor.setCompositionVisibility(false);
                ViewGroup parent = (ViewGroup) accessibilityAnchor.getParent();
                if (parent != null) parent.removeView(accessibilityAnchor);
                try (SurfaceControl.Transaction transaction = new SurfaceControl.Transaction()) {
                    transaction.reparent(control, null).apply();
                }
                surfacePackage.release();
                host.release();
            }
            factory.release(view);
        }
    }

    private final class PlatformViewInputHost extends FrameLayout {
        private final long platformViewId;
        private View pendingImeEditor;
        private boolean handledTab;

        PlatformViewInputHost(long platformViewId) {
            super(TesseraGameActivity.this);
            this.platformViewId = platformViewId;
        }

        @Override
        public void requestChildFocus(View child, View focused) {
            super.requestChildFocus(child, focused);
            scheduleAutofillFocus();
            schedulePlatformFocusReport();
        }

        @Override
        public void clearChildFocus(View child) {
            super.clearChildFocus(child);
            scheduleAutofillFocus();
            schedulePlatformFocusReport();
        }

        void clearEmbeddedFocus() {
            View focused = findFocus();
            if (focused == null) return;
            // Moving Activity focus away from the SurfaceView only revokes input
            // focus for the embedded window. Clear its local focus without letting
            // the separate ViewRoot immediately restore the same child.
            int descendants = getDescendantFocusability();
            setDescendantFocusability(FOCUS_BLOCK_DESCENDANTS);
            try {
                focused.clearFocus();
            } finally {
                setDescendantFocusability(descendants);
            }
        }

        @Override
        public boolean dispatchKeyEvent(KeyEvent event) {
            if (event.getKeyCode() != KeyEvent.KEYCODE_TAB
                    || !(event.hasNoModifiers() || event.hasModifiers(KeyEvent.META_SHIFT_ON))) {
                return super.dispatchKeyEvent(event);
            }
            if (event.getAction() == KeyEvent.ACTION_UP && handledTab) {
                handledTab = false;
                return true;
            }
            if (super.dispatchKeyEvent(event)) return true;
            if (event.getAction() != KeyEvent.ACTION_DOWN || findFocus() == null) return false;
            int direction = event.isShiftPressed() ? View.FOCUS_BACKWARD : View.FOCUS_FORWARD;
            FocusFinder finder = FocusFinder.getInstance();
            View next = finder.findNextFocus(this, findFocus(), direction);
            View edge = platformFocusEdge(this, direction);
            if (next != null && next != findFocus() && next != edge && next.requestFocus(direction)) {
                handledTab = true;
                return true;
            }
            reportPlatformFocus(0);
            handledTab = TesseraGameActivity.this.onKeyDown(KeyEvent.KEYCODE_TAB, event);
            return handledTab;
        }

        private void scheduleAutofillFocus() {
            if (Build.VERSION.SDK_INT < 35) return;
            post(() -> {
                HostedPlatformView hosted = tesseraPlatformViews.get(platformViewId);
                if (hosted != null && hosted.accessibilityAnchor != null && hasWindowFocus()) {
                    View focused = findFocus();
                    if (focused != null) hosted.accessibilityAnchor.syncAutofillFocus(focused);
                }
            });
        }

        @Override
        public boolean dispatchTouchEvent(MotionEvent event) {
            int action = event.getActionMasked();
            if (action == MotionEvent.ACTION_DOWN || action == MotionEvent.ACTION_CANCEL) {
                pendingImeEditor = null;
            }
            boolean handled = super.dispatchTouchEvent(event);
            if (handled && action == MotionEvent.ACTION_UP
                    && !isTesseraPlatformViewInputBlocked(
                            platformViewId, event.getRawX(), event.getRawY())) {
                View focused = findFocus();
                if (focused instanceof TextView && focused.onCheckIsTextEditor()
                        && ((TextView) focused).getShowSoftInputOnFocus()) {
                    InputMethodManager ime = getContext().getSystemService(InputMethodManager.class);
                    if (ime != null && !ime.isActive(focused)) {
                        // A first tap can focus the editor before its embedded window is
                        // served by the IME, causing the editor's own show request to fail.
                        pendingImeEditor = focused;
                        post(this::showPendingIme);
                    }
                }
            }
            return handled;
        }

        @Override
        public void onWindowFocusChanged(boolean hasWindowFocus) {
            super.onWindowFocusChanged(hasWindowFocus);
            if (hasWindowFocus && pendingImeEditor != null) post(this::showPendingIme);
            scheduleAutofillFocus();
            schedulePlatformFocusReport();
        }

        private void showPendingIme() {
            if (!hasWindowFocus()) return;
            View editor = pendingImeEditor;
            pendingImeEditor = null;
            if (editor != null && editor.isFocused() && editor.isShown()) {
                InputMethodManager ime = getContext().getSystemService(InputMethodManager.class);
                if (ime != null) ime.showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT);
            }
        }

        @Override
        public boolean onInterceptTouchEvent(MotionEvent event) {
            if (Build.VERSION.SDK_INT < 35 || event.getActionMasked() != MotionEvent.ACTION_DOWN
                    || !isTesseraPlatformViewInputBlocked(
                            platformViewId, event.getRawX(), event.getRawY())) {
                return false;
            }
            HostedPlatformView hosted = tesseraPlatformViews.get(platformViewId);
            if (hosted == null || hosted.surfacePackage.getInputTransferToken() == null) {
                return false;
            }
            AttachedSurfaceControl root = mSurfaceView.getRootSurfaceControl();
            InputTransferToken hostToken = root == null ? null : root.getInputTransferToken();
            if (hostToken == null) return false;
            try {
                boolean transferred = getWindowManager().transferTouchGesture(
                        hosted.surfacePackage.getInputTransferToken(), hostToken);
                android.util.Log.d("Tessera", "Transferred native touch to Tessera: " + transferred);
                return transferred;
            } catch (UnsupportedOperationException error) {
                android.util.Log.w("Tessera", "Native-to-Tessera touch transfer is unavailable", error);
                return false;
            }
        }
    }

    @Keep
    public boolean isTesseraPlatformViewInputBlocked(long id, float x, float y) {
        for (HostedInputRegion blocker : tesseraInputBlockers) {
            if (blocker.platformViewId == id && blocker.contains(x, y)) {
                return true;
            }
        }
        return false;
    }

    private static final class HostedSliceSurface {
        final SurfaceControl control;
        final Surface surface;
        final SliceTexture textureSurface;
        final ImageReader reader;
        TextureView textureView;
        int acquiredImages;
        int width;
        int height;

        HostedSliceSurface(SurfaceControl control, Surface surface, int width, int height,
                SliceTexture textureSurface, ImageReader reader) {
            this.control = control;
            this.surface = surface;
            this.width = width;
            this.height = height;
            this.textureSurface = textureSurface;
            this.reader = reader;
        }

        void release() {
            surface.release();
            if (control != null) control.release();
            if (textureSurface != null) textureSurface.release();
            if (reader != null) reader.close();
        }
    }

    private final class SliceTexture extends SurfaceTexture {
        private final Handler ui = new Handler(Looper.getMainLooper());
        private boolean hasFrame;
        private long presentedTimestamp;

        SliceTexture() {
            super(false);
            setOnFrameAvailableListener(null);
        }

        @Override public void setOnFrameAvailableListener(OnFrameAvailableListener listener) {
            setOnFrameAvailableListener(listener, ui);
        }

        @Override public void setOnFrameAvailableListener(OnFrameAvailableListener listener, Handler handler) {
            // TextureView removes its listener when hidden. Still observe the producer
            // so a slot is not made visible before its new frame has been queued.
            super.setOnFrameAvailableListener(texture -> {
                hasFrame = true;
                if (listener != null) listener.onFrameAvailable(texture);
                tryLegacyComposition();
            }, ui);
        }
    }

    private static final class HostedInputRegion {
        final long platformViewId;
        final float left;
        final float top;
        final float width;
        final float height;

        HostedInputRegion(long platformViewId, float left, float top, float width, float height) {
            this.platformViewId = platformViewId;
            this.left = left;
            this.top = top;
            this.width = width;
            this.height = height;
        }

        boolean contains(float x, float y) {
            return x >= left && x < left + width && y >= top && y < top + height;
        }
    }

    @Override
    protected void onStart() {
        super.onStart();
        tesseraCompositionStopped = false;
        legacyHiddenPrimary = null;
        tryLegacyComposition();
    }

    @Override
    protected void onStop() {
        super.onStop();
        tesseraCompositionStopped = true;
        legacyLatchCheckScheduled = false;
        // A hidden ViewRoot can stop drawing before it acknowledges its last frame.
        if (Build.VERSION.SDK_INT < 35 && pendingLegacyComposition == null) {
            acknowledgeComposition(tesseraPlatformViewFrame);
        }
        releaseRetiredViews();
    }

    @Override
    protected void onDestroy() {
        if (legacySurfaceOrderRoot != null) {
            ViewTreeObserver observer = getWindow().getDecorView().getViewTreeObserver();
            if (Build.VERSION.SDK_INT >= 34) {
                observer.removeOnPreDrawListener(legacySurfaceOrderPreDrawListener);
            } else {
                observer.removeOnDrawListener(legacySurfaceOrderListener);
            }
        }
        pendingLegacyComposition = null;
        pendingStagedComposition = null;
        if (pendingStagedImages != null) {
            for (Image image : pendingStagedImages) {
                if (image != null) image.close();
            }
            pendingStagedImages = null;
        }
        for (HostedPlatformView view : tesseraPlatformViews.values()) view.release();
        tesseraPlatformViews.clear();
        // GameActivity joins the renderer before the Java-owned slice windows are released.
        super.onDestroy();
        for (HostedSliceSurface[] slots : tesseraSliceSurfaces) {
            for (HostedSliceSurface slice : slots) slice.release();
        }
        tesseraSliceSurfaces.clear();
    }

    @Override
    public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        schedulePlatformFocusReport();
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

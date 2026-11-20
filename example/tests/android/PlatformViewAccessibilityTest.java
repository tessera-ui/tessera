package com.tessera.example;

import android.accessibilityservice.AccessibilityServiceInfo;
import android.app.Activity;
import android.app.Application;
import android.app.Instrumentation;
import android.app.UiAutomation;
import android.content.Intent;
import android.graphics.Rect;
import android.os.Bundle;
import android.os.ParcelFileDescriptor;
import android.os.Process;
import android.os.SystemClock;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityWindowInfo;
import android.view.autofill.AutofillManager;
import android.view.InputDevice;
import android.view.MotionEvent;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

public final class PlatformViewAccessibilityTest extends Instrumentation {
    private boolean surfaceBacked;
    private boolean bothSurfaceBacked;
    private boolean enableAccessibilityAfterLayout;
    private boolean relaunchActivity;
    private boolean autofill;
    private boolean autofillSave;
    private boolean accessibilityFocus;

    @Override
    public void onCreate(Bundle arguments) {
        super.onCreate(arguments);
        bothSurfaceBacked = "true".equals(arguments.getString("bothSurfaceBacked"));
        surfaceBacked = bothSurfaceBacked || "true".equals(arguments.getString("surfaceBacked"));
        enableAccessibilityAfterLayout = "true".equals(arguments.getString("lateAccessibility"));
        relaunchActivity = "true".equals(arguments.getString("relaunchActivity"));
        autofill = "true".equals(arguments.getString("autofill"));
        autofillSave = autofill && "true".equals(arguments.getString("autofillSave"));
        accessibilityFocus = "true".equals(arguments.getString("accessibilityFocus"));
        start();
    }

    @Override
    public void onStart() {
        final String[] labels = {"Native 1", "Tessera foreground probe"};
        UiAutomation automation = enableAccessibilityAfterLayout && !autofill
                ? getUiAutomation(UiAutomation.FLAG_DONT_USE_ACCESSIBILITY)
                : connectAccessibility();
        Intent intent = new Intent(Intent.ACTION_MAIN)
                .setClassName(getTargetContext(), "com.tessera.TesseraGameActivity")
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                .putExtra("tessera.platform_view_probe", true)
                .putExtra("tessera.platform_view_surface_backed", surfaceBacked)
                .putExtra("tessera.platform_view_both_surface_backed", bothSurfaceBacked)
                .putExtra("tessera.platform_view_editor", autofill);
        Activity activity = startActivitySync(intent);
        if (autofill) {
            verifyAutofill(activity, automation);
            return;
        }

        boolean laidOutBeforeAccessibility = false;
        if (enableAccessibilityAfterLayout) {
            long layoutDeadline = SystemClock.uptimeMillis() + 10_000;
            do {
                laidOutBeforeAccessibility = lifecycleLogs(automation).stream()
                        .anyMatch(line -> line.startsWith("layout 1 ") && !line.equals("layout 1 1x1"));
                if (laidOutBeforeAccessibility) break;
                SystemClock.sleep(100);
            } while (SystemClock.uptimeMillis() < layoutDeadline);
            automation = connectAccessibility();
        }
        int[] counts = new int[labels.length];
        Rect[] bounds = {new Rect(), new Rect()};
        float density = activity.getResources().getDisplayMetrics().density;
        Rect expected = new Rect(Math.round(112 * density), Math.round(112 * density),
                Math.round(280 * density), Math.round(256 * density));
        boolean scaledBounds;
        long deadline = SystemClock.uptimeMillis() + 15_000;
        do {
            counts = new int[labels.length];
            collect(automation.getRootInActiveWindow(), labels, counts, bounds);
            scaledBounds = closeBounds(bounds[0], expected);
            if (counts[0] == 1 && counts[1] == 1 && scaledBounds) break;
            SystemClock.sleep(200);
        } while (SystemClock.uptimeMillis() < deadline);
        Bundle result = new Bundle();
        for (int index = 0; index < labels.length; index++) {
            result.putInt(labels[index], counts[index]);
            if (counts[index] == 1) {
                result.putString(labels[index] + " screen bounds", bounds[index].toShortString());
            }
        }
        result.putBoolean("native accessibility bounds follow layout scale", scaledBounds);
        if (enableAccessibilityAfterLayout) {
            result.putBoolean("native view laid out before accessibility connected",
                    laidOutBeforeAccessibility);
        }
        boolean focusMoved = true;
        if (accessibilityFocus) {
            String first = moveAccessibilityFocus(automation, labels[0], labels[1]);
            String second = moveAccessibilityFocus(automation, labels[1], labels[0]);
            String third = moveAccessibilityFocus(automation, labels[0], labels[1]);
            result.putString("first native accessibility focus", first);
            result.putString("Tessera accessibility focus", second);
            result.putString("restored native accessibility focus", third);
            focusMoved = "ok".equals(first) && "ok".equals(second) && "ok".equals(third);
        }
        boolean destroyed = finishAndWait(activity);
        List<String> logs = lifecycleLogs(automation);
        boolean released = destroyed && releasedViewsOnce(logs);
        result.putBoolean("native views reset and released exactly once", released);
        if (!released) {
            result.putBoolean("Activity destruction completed", destroyed);
            result.putString("native lifecycle log", logs.toString());
        }
        boolean passed = counts[0] == 1 && counts[1] == 1 && scaledBounds && released && focusMoved
                && (!enableAccessibilityAfterLayout || laidOutBeforeAccessibility);
        if (relaunchActivity && passed) {
            Activity replacement = startActivitySync(intent);
            boolean laidOutAgain;
            deadline = SystemClock.uptimeMillis() + 15_000;
            do {
                int[] replacementCounts = new int[labels.length];
                collect(automation.getRootInActiveWindow(), labels, replacementCounts, bounds);
                laidOutAgain = replacementCounts[0] == 1 && replacementCounts[1] == 1
                        && closeBounds(bounds[0], expected);
                if (laidOutAgain) break;
                SystemClock.sleep(200);
            } while (SystemClock.uptimeMillis() < deadline);
            boolean replacementDestroyed = finishAndWait(replacement);
            List<String> replacementLogs = lifecycleLogs(automation);
            passed = laidOutAgain && replacementDestroyed
                    && replacementLogs.size() >= logs.size()
                    && releasedViewsOnce(replacementLogs.subList(logs.size(), replacementLogs.size()));
            result.putBoolean("Activity relaunched and released in the same process", passed);
        }
        finish(passed ? Activity.RESULT_OK : Activity.RESULT_CANCELED, result);
    }

    private void verifyAutofill(Activity activity, UiAutomation automation) {
        Bundle result = new Bundle();
        long deadline = SystemClock.uptimeMillis() + 15_000;
        Rect editorBounds = new Rect();
        do {
            for (AccessibilityWindowInfo window : automation.getWindows()) {
                if (visitNamedNode(window.getRoot(), "Native editor", node -> {
                    if (!node.isEditable()) return false;
                    node.getBoundsInScreen(editorBounds);
                    return editorBounds.width() > 1 && editorBounds.height() > 1;
                })) break;
            }
            if (editorBounds.width() > 1 && editorBounds.height() > 1) break;
            SystemClock.sleep(150);
        } while (SystemClock.uptimeMillis() < deadline);

        boolean ready = editorBounds.width() > 1 && editorBounds.height() > 1;
        result.putBoolean("native editor ready", ready);
        boolean focused = false;
        if (ready) {
            // An embedded ViewRoot can select its editor before it gets input
            // focus. The system-provided dataset below is the observable check
            // that this editor actually became the active autofill target.
            tap(automation, editorBounds.exactCenterX(), editorBounds.exactCenterY());
            deadline = SystemClock.uptimeMillis() + 5_000;
            do {
                focused = lifecycleLogs(automation).contains("editor focus true");
                if (focused) break;
                SystemClock.sleep(150);
            } while (SystemClock.uptimeMillis() < deadline);
        }
        result.putBoolean("native editor focused", focused);

        boolean offered = false;
        boolean filled = false;
        if (focused) {
            deadline = SystemClock.uptimeMillis() + 15_000;
            String[] labels = {"Tessera test autofill"};
            Rect[] bounds = {new Rect()};
            do {
                int[] counts = {0};
                for (AccessibilityWindowInfo window : automation.getWindows()) {
                    collect(window.getRoot(), labels, counts, bounds);
                }
                if (counts[0] != 0 && !bounds[0].isEmpty()) {
                    tap(automation, bounds[0].exactCenterX(), bounds[0].exactCenterY());
                    offered = true;
                    break;
                }
                SystemClock.sleep(200);
            } while (SystemClock.uptimeMillis() < deadline);
        }
        if (offered) {
            deadline = SystemClock.uptimeMillis() + 5_000;
            do {
                filled = lifecycleLogs(automation).contains("editor typed Hello");
                if (filled) break;
                SystemClock.sleep(150);
            } while (SystemClock.uptimeMillis() < deadline);
        }
        result.putBoolean("system offered native autofill", offered);
        result.putBoolean("system filled native editor", filled);
        boolean saved = !autofillSave || filled && verifyAutofillSave(activity, automation, result);
        if (autofillSave) result.putBoolean("system saved edited native value", saved);
        finishAndWait(activity);
        finish(filled && saved ? Activity.RESULT_OK : Activity.RESULT_CANCELED, result);
    }

    private boolean verifyAutofillSave(Activity activity, UiAutomation automation, Bundle result) {
        String message = "received edited probe value in save request";
        String logCommand = "logcat -d -v raw -s TesseraAutofillProbe:I";
        long before = shellLines(automation, logCommand).stream().filter(message::equals).count();
        Bundle edit = new Bundle();
        edit.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, "HelloSaved");
        boolean applied = false;
        for (AccessibilityWindowInfo window : automation.getWindows()) {
            if (visitNamedNode(window.getRoot(), "Hello", node -> node.isEditable()
                    && node.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, edit))) {
                applied = true;
                break;
            }
        }
        result.putBoolean("native autofill field accepted edit action", applied);
        if (!applied) return false;
        long editDeadline = SystemClock.uptimeMillis() + 5_000;
        boolean edited = false;
        do {
            for (AccessibilityWindowInfo window : automation.getWindows()) {
                if (visitNamedNode(window.getRoot(), "HelloSaved", AccessibilityNodeInfo::isEditable)) {
                    edited = true;
                    break;
                }
            }
            if (edited) break;
            SystemClock.sleep(150);
        } while (SystemClock.uptimeMillis() < editDeadline);
        result.putBoolean("native autofill field shows edited value", edited);
        if (!edited) return false;
        runOnMainSync(() -> activity.getSystemService(AutofillManager.class).commit());
        long deadline = SystemClock.uptimeMillis() + 8_000;
        boolean confirmed = false;
        do {
            if (!confirmed) {
                for (AccessibilityWindowInfo window : automation.getWindows()) {
                    if (confirmAutofillSave(window.getRoot())) {
                        confirmed = true;
                        break;
                    }
                }
            }
            result.putBoolean("system offered native autofill save", confirmed);
            if (confirmed && shellLines(automation, logCommand).stream().filter(message::equals).count()
                    > before) return true;
            SystemClock.sleep(150);
        } while (SystemClock.uptimeMillis() < deadline);
        return false;
    }

    private static boolean confirmAutofillSave(AccessibilityNodeInfo root) {
        if (root == null) return false;
        List<AccessibilityNodeInfo> buttons;
        try {
            buttons = root.findAccessibilityNodeInfosByViewId("android:id/autofill_save_yes");
        } finally {
            root.recycle();
        }
        boolean confirmed = false;
        for (AccessibilityNodeInfo button : buttons) {
            if (!confirmed && button.isVisibleToUser()) {
                confirmed = button.performAction(AccessibilityNodeInfo.ACTION_CLICK);
            }
            button.recycle();
        }
        return confirmed;
    }

    private static void tap(UiAutomation automation, float x, float y) {
        long time = SystemClock.uptimeMillis();
        for (int action : new int[]{MotionEvent.ACTION_DOWN, MotionEvent.ACTION_UP}) {
            MotionEvent event = MotionEvent.obtain(time, SystemClock.uptimeMillis(), action, x, y, 0);
            event.setSource(InputDevice.SOURCE_TOUCHSCREEN);
            automation.injectInputEvent(event, true);
            event.recycle();
        }
    }

    private boolean releasedViewsOnce(List<String> logs) {
        return releasedOnce(logs, bothSurfaceBacked ? "create surface-backed 0" : "create 0", 0)
                && releasedOnce(logs, surfaceBacked ? "create surface-backed 1" : "create 1", 1);
    }

    private static boolean closeBounds(Rect actual, Rect expected) {
        // The probe crosses fractional density and rounds local, layout, and screen pixels.
        return Math.abs(actual.left - expected.left) <= 3 && Math.abs(actual.top - expected.top) <= 3
                && Math.abs(actual.right - expected.right) <= 3
                && Math.abs(actual.bottom - expected.bottom) <= 3;
    }

    private boolean finishAndWait(Activity activity) {
        CountDownLatch destroyed = new CountDownLatch(1);
        Application.ActivityLifecycleCallbacks observer = new Application.ActivityLifecycleCallbacks() {
            @Override public void onActivityCreated(Activity value, Bundle state) {}
            @Override public void onActivityStarted(Activity value) {}
            @Override public void onActivityResumed(Activity value) {}
            @Override public void onActivityPaused(Activity value) {}
            @Override public void onActivityStopped(Activity value) {}
            @Override public void onActivitySaveInstanceState(Activity value, Bundle state) {}
            @Override public void onActivityDestroyed(Activity value) {}

            @Override
            public void onActivityPostDestroyed(Activity value) {
                if (activity == value) destroyed.countDown();
            }
        };
        runOnMainSync(() -> {
            activity.registerActivityLifecycleCallbacks(observer);
            activity.finishAndRemoveTask();
        });
        try {
            return destroyed.await(10, TimeUnit.SECONDS);
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            return false;
        }
    }

    private UiAutomation connectAccessibility() {
        UiAutomation automation = getUiAutomation();
        AccessibilityServiceInfo info = automation.getServiceInfo();
        // Resolving a SurfaceView's leashed child needs the embedded window connection.
        info.flags |= AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS
                | AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS;
        if (accessibilityFocus) {
            info.flags |= AccessibilityServiceInfo.FLAG_REQUEST_TOUCH_EXPLORATION_MODE;
        }
        automation.setServiceInfo(info);
        return automation;
    }

    private static List<String> lifecycleLogs(UiAutomation automation) {
        return shellLines(automation, "logcat -d --pid=" + Process.myPid() + " -v raw -s TesseraViewProbe:I");
    }

    private static List<String> shellLines(UiAutomation automation, String command) {
        ParcelFileDescriptor pipe = automation.executeShellCommand(command);
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(
                new ParcelFileDescriptor.AutoCloseInputStream(pipe), StandardCharsets.UTF_8))) {
            List<String> logs = new ArrayList<>();
            String line;
            while ((line = reader.readLine()) != null) logs.add(line);
            return logs;
        } catch (IOException error) {
            throw new IllegalStateException("Could not read probe shell output", error);
        }
    }

    private static boolean releasedOnce(List<String> logs, String create, int index) {
        String reset = "reset " + index;
        String release = "release " + index;
        int createdAt = logs.indexOf(create);
        int resetAt = logs.indexOf(reset);
        int releaseAt = logs.indexOf(release);
        return createdAt >= 0 && createdAt < resetAt && resetAt < releaseAt
                && createdAt == logs.lastIndexOf(create)
                && resetAt == logs.lastIndexOf(reset)
                && releaseAt == logs.lastIndexOf(release);
    }

    private static String moveAccessibilityFocus(UiAutomation automation, String next, String previous) {
        if (!visitNamedNode(automation.getRootInActiveWindow(), next,
                node -> node.isAccessibilityFocused() || node.performAction(
                        AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS))) return "focus action rejected";
        long deadline = SystemClock.uptimeMillis() + 3_000;
        boolean entered;
        boolean exited;
        do {
            entered = visitNamedNode(automation.getRootInActiveWindow(), next,
                    AccessibilityNodeInfo::isAccessibilityFocused);
            exited = visitNamedNode(automation.getRootInActiveWindow(), previous,
                    node -> !node.isAccessibilityFocused());
            if (entered && exited) return "ok";
            SystemClock.sleep(100);
        } while (SystemClock.uptimeMillis() < deadline);
        return "focus entered=" + entered + ", previous focus released=" + exited;
    }

    private static boolean visitNamedNode(AccessibilityNodeInfo node, String label,
            java.util.function.Predicate<AccessibilityNodeInfo> check) {
        if (node == null) return false;
        try {
            if (!node.isVisibleToUser()) return false;
            if (matchesLabel(node, label)) return check.test(node);
            for (int index = 0; index < node.getChildCount(); index++) {
                if (visitNamedNode(node.getChild(index), label, check)) return true;
            }
            return false;
        } finally {
            node.recycle();
        }
    }

    private static boolean matchesLabel(AccessibilityNodeInfo node, String label) {
        return label.equalsIgnoreCase(String.valueOf(node.getText()))
                || label.equalsIgnoreCase(String.valueOf(node.getContentDescription()))
                || node.isShowingHintText() && label.equalsIgnoreCase(String.valueOf(node.getHintText()));
    }

    private static void collect(AccessibilityNodeInfo node, String[] labels, int[] counts,
            Rect[] bounds) {
        if (node == null) return;
        try {
            if (!node.isVisibleToUser()) return;
            for (int index = 0; index < labels.length; index++) {
                if (matchesLabel(node, labels[index])) {
                    counts[index]++;
                    node.getBoundsInScreen(bounds[index]);
                }
            }
            for (int index = 0; index < node.getChildCount(); index++) {
                collect(node.getChild(index), labels, counts, bounds);
            }
        } finally {
            node.recycle();
        }
    }
}

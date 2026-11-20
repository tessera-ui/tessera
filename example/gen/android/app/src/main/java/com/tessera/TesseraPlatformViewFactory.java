package com.tessera;

import android.view.View;

/** Creates and updates a native view on the Android UI thread. */
public interface TesseraPlatformViewFactory {
    View create();

    default void update(View view) {}

    /** Clears transient state just before Tessera detaches the view for release. */
    default void reset(View view) {}

    default void release(View view) {}
}

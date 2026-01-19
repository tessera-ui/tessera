package com.tessera.platform

import android.app.Activity
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context

object ClipboardPlugin {
    @JvmStatic
    fun hasText(activity: Activity): Boolean {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
            ?: return false
        val description = clipboard.primaryClipDescription ?: return false
        return description.hasMimeType(ClipDescription.MIMETYPE_TEXT_PLAIN) ||
            description.hasMimeType(ClipDescription.MIMETYPE_TEXT_HTML)
    }

    @JvmStatic
    fun getText(activity: Activity): String {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
            ?: return ""
        val clip = clipboard.primaryClip ?: return ""
        if (clip.itemCount == 0) {
            return ""
        }
        val text = clip.getItemAt(0).coerceToText(activity) ?: return ""
        return text.toString()
    }

    @JvmStatic
    fun setText(activity: Activity, text: String) {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
            ?: return
        val clip = ClipData.newPlainText("text", text)
        clipboard.setPrimaryClip(clip)
    }

    @JvmStatic
    fun clear(activity: Activity) {
        val clipboard = activity.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
            ?: return
        val clip = ClipData.newPlainText("", "")
        clipboard.setPrimaryClip(clip)
    }
}

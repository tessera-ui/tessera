use std::sync::OnceLock;

use windows::Win32::{
    Foundation::{GetLastError, HWND, SetLastError, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::{GCLP_HICON, GCLP_HICONSM, ICON_BIG, SendMessageW, WM_GETICON},
};
use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Icon, Window},
};

// Keep the class fallback off windows created independently through winit.
pub(super) const WINDOW_CLASS: &str = "TesseraRendererWindow";

pub(super) fn install_class_icon(window: &Window, icon: &Icon) {
    // The dedicated renderer class stays registered even after its window is destroyed.
    // Keep the icon alive for the class, not just for the window.
    static CLASS_ICON: OnceLock<Icon> = OnceLock::new();

    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);

    // winit has already installed this icon as ICON_BIG. The class gives the shell
    // a fallback when it cannot get a timely WM_GETICON reply from the UI thread.
    let handle = unsafe { SendMessageW(hwnd, WM_GETICON, Some(WPARAM(ICON_BIG as usize)), None) }.0;
    if handle == 0 || CLASS_ICON.set(icon.clone()).is_err() {
        return;
    }

    for index in [GCLP_HICON, GCLP_HICONSM] {
        // SAFETY: hwnd is a live, hidden Tessera renderer window on this thread.
        // The class borrows the HICON, owned by CLASS_ICON for the process lifetime.
        unsafe {
            SetLastError(WIN32_ERROR(0));
            #[cfg(target_pointer_width = "64")]
            let previous =
                windows::Win32::UI::WindowsAndMessaging::SetClassLongPtrW(hwnd, index, handle);
            #[cfg(target_pointer_width = "32")]
            let previous =
                windows::Win32::UI::WindowsAndMessaging::SetClassLongW(hwnd, index, handle as i32);
            if previous == 0 {
                let error = GetLastError();
                if error != WIN32_ERROR(0) {
                    tracing::warn!(?error, "Failed to install Windows window class icon");
                }
            }
        }
    }
}

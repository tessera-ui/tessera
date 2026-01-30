//! # Thread Utilities

/// Sets the name of the current thread for debugging and profiling purposes.
///
/// This function provides a cross-platform way to set thread names, which is
/// invaluable for debugging multi-threaded applications. Thread names appear
/// in debuggers, profilers, and system monitoring tools.
///
/// # Arguments
///
/// - `name`: The name to assign to the current thread. Should be descriptive
///   and follow platform-specific length limitations.
///
/// # Platform Behavior
///
/// - **Unix-like systems**: Uses `pthread_setname_np()` to set the thread name
/// - **Windows**: Uses `SetThreadDescription()` API
/// - **Other platforms**: No operation is performed
pub fn set_thread_name(name: &str) {
    #[cfg(target_family = "unix")]
    set_thread_name_unix(name);

    #[cfg(target_os = "windows")]
    set_thread_name_windows(name);

    #[cfg(not(any(target_family = "unix", target_os = "windows")))]
    {
        // No-op for unsupported platforms
        let _ = name; // Suppress unused variable warning
    }
}

// Sets the thread name on Unix-like systems using pthread APIs.
#[cfg(target_family = "unix")]
fn set_thread_name_unix(name: &str) {
    use std::ffi::CString;

    // Convert to C string, panicking if null bytes are present
    let cname = CString::new(name).unwrap();

    unsafe {
        // Set the name for the current thread
        #[cfg(target_vendor = "apple")]
        libc::pthread_setname_np(cname.as_ptr());
        #[cfg(not(target_vendor = "apple"))]
        libc::pthread_setname_np(libc::pthread_self(), cname.as_ptr());
    }
}

// Sets the thread name on Windows using the Win32 API.
#[cfg(target_os = "windows")]
fn set_thread_name_windows(name: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Convert to UTF-16 with null terminator
    let name_wide: Vec<u16> = OsStr::new(name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use windows::{
            Win32::System::Threading::{GetCurrentThread, SetThreadDescription},
            core::PCWSTR,
        };

        // Set the thread description, ignoring any errors
        let _ = SetThreadDescription(GetCurrentThread(), PCWSTR(name_wide.as_ptr()));
    }
}

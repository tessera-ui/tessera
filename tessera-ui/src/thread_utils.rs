//! # Thread Utilities
//!
//! This module provides cross-platform utilities for thread management and
//! debugging in the Tessera UI framework.

/// Sets the name of the current thread for debugging and profiling purposes.
///
/// This function provides a cross-platform way to set thread names, which is
/// invaluable for debugging multi-threaded applications. Thread names appear
/// in debuggers, profilers, and system monitoring tools.
///
/// # Arguments
///
/// * `name` - The name to assign to the current thread. Should be descriptive
///   and follow platform-specific length limitations.
///
/// # Platform Behavior
///
/// - **Unix-like systems**: Uses `pthread_setname_np()` to set the thread name
/// - **Windows**: Uses `SetThreadDescription()` API
/// - **Other platforms**: No operation is performed
///
/// # Panics
///
/// This function may panic if:
/// - The thread name contains null bytes (Unix systems)
/// - Memory allocation fails during string conversion
///
/// # Performance
///
/// This is a lightweight operation that should have minimal performance impact.
/// However, it involves system calls, so avoid calling it frequently in
/// performance-critical code paths.
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

/// Sets the thread name on Unix-like systems using pthread APIs.
///
/// This function uses the POSIX `pthread_setname_np` function to set the
/// thread name. The name is converted to a C-compatible null-terminated
/// string before passing to the system API.
///
/// # Arguments
///
/// * `name` - The thread name to set
///
/// # Platform Notes
///
/// - **Linux**: Thread names are limited to 15 characters (plus null
///   terminator)
/// - **macOS**: Thread names can be up to 63 characters
/// - Names exceeding the limit may be truncated or cause the operation to fail
///
/// # Panics
///
/// Panics if the name contains null bytes, as this would create an invalid
/// C string.
///
/// # Safety
///
/// This function uses unsafe code to call the `pthread_setname_np` system call.
/// The safety is ensured by:
/// - Using a valid C string created from the input
/// - Calling with the current thread handle (`pthread_self()`)
/// - The pthread API is designed to handle these parameters safely
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

/// Sets the thread name on Windows using the Win32 API.
///
/// This function uses the Windows `SetThreadDescription` API, which was
/// introduced in Windows 10 version 1607 and Windows Server 2016. The
/// thread name is converted to UTF-16 format as required by the Windows API.
///
/// # Arguments
///
/// * `name` - The thread name to set
///
/// # Platform Notes
///
/// - Available on Windows 10 version 1607 and later
/// - On older Windows versions, this function will fail silently
/// - Thread names have no specific length limit but shorter names are
///   recommended
///
/// # Error Handling
///
/// Any errors from the Windows API are ignored (the result is discarded with
/// `let _`). This is intentional as thread naming is a debugging aid and should
/// not cause application failures.
///
/// # Safety
///
/// This function uses unsafe code to call Windows APIs. The safety is ensured
/// by:
/// - Creating a valid UTF-16 null-terminated string
/// - Using the current thread handle from `GetCurrentThread()`
/// - Properly constructing the `PCWSTR` parameter
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

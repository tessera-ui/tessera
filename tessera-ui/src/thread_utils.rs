//! # Thread Utilities

/// Thread-safety bound for abstractions that own GPU resources.
///
/// Native targets require `Send + Sync + 'static`; wasm requires only `'static`.
#[cfg(not(target_family = "wasm"))]
pub trait SendSync: Send + Sync + 'static {}
#[cfg(not(target_family = "wasm"))]
impl<T: Send + Sync + 'static> SendSync for T {}

/// Thread-safety bound for abstractions that own GPU resources.
///
/// Requires only `'static` on wasm targets.
#[cfg(target_family = "wasm")]
pub trait SendSync: 'static {}
#[cfg(target_family = "wasm")]
impl<T: 'static> SendSync for T {}
/// Sets the current thread's name for debugging and profiling on Unix and Windows.
/// Names must follow platform-specific length limits. Other platforms do nothing.
pub fn set_thread_name(name: &str) {
    #[cfg(target_family = "unix")]
    set_thread_name_unix(name);

    #[cfg(target_os = "windows")]
    set_thread_name_windows(name);

    #[cfg(not(any(target_family = "unix", target_os = "windows")))]
    {
        let _ = name;
    }
}

#[cfg(target_family = "unix")]
fn set_thread_name_unix(name: &str) {
    use std::ffi::CString;

    let cname = CString::new(name).expect("thread name contains null bytes");

    unsafe {
        #[cfg(target_vendor = "apple")]
        libc::pthread_setname_np(cname.as_ptr());
        #[cfg(not(target_vendor = "apple"))]
        libc::pthread_setname_np(libc::pthread_self(), cname.as_ptr());
    }
}

#[cfg(target_os = "windows")]
fn set_thread_name_windows(name: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let name_wide: Vec<u16> = OsStr::new(name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use windows::{
            Win32::System::Threading::{GetCurrentThread, SetThreadDescription},
            core::PCWSTR,
        };

        let _ = SetThreadDescription(GetCurrentThread(), PCWSTR(name_wide.as_ptr()));
    }
}

pub fn set_thread_name(name: &str) {
    #[cfg(target_family = "unix")]
    set_thread_name_unix(name);

    #[cfg(target_os = "windows")]
    set_thread_name_windows(name);
}

#[cfg(target_family = "unix")]
fn set_thread_name_unix(name: &str) {
    use std::ffi::CString;
    let cname = CString::new(name).unwrap();

    unsafe {
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

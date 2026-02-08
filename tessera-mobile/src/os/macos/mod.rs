mod ffi;
pub(super) mod info;

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    ptr,
};

use core_foundation::{
    array::CFArray,
    base::TCFType,
    error::{CFError, CFErrorRef},
    string::{CFString, CFStringRef},
    url::CFURL,
};
use thiserror::Error;

use crate::{DuctExpressionExt, apple::deps::xcode_plugin::xcode_developer_dir, env::ExplicitEnv};

pub use crate::{env::Env, util::ln};

// This can hopefully be relied upon... https://stackoverflow.com/q/8003919
static RUST_UTI: &str = "dyn.ah62d4rv4ge81e62";

#[derive(Debug, Error)]
pub enum DetectEditorError {
    #[error(transparent)]
    LookupFailed(CFError),
}

#[derive(Debug, Error)]
pub enum OpenFileError {
    #[error("Failed to convert path {path} into a `CFURL`.")]
    PathToUrlFailed { path: PathBuf },
    #[error("Failed to launch {path} with {command}: {error}")]
    LaunchFailed {
        path: String,
        command: &'static str,
        error: std::io::Error,
    },
}

#[derive(Debug)]
pub struct Application {
    url: CFURL,
}

impl Application {
    pub fn detect_editor() -> Result<Self, DetectEditorError> {
        fn inner(uti: CFStringRef) -> Result<CFURL, CFError> {
            let mut err: CFErrorRef = ptr::null_mut();
            // SAFETY: `uti` is a valid CoreFoundation string reference, and
            // `err` points to writable storage for LaunchServices to fill.
            let out_url = unsafe {
                ffi::LSCopyDefaultApplicationURLForContentType(uti, ffi::kLSRolesEditor, &mut err)
            };
            if out_url.is_null() {
                // SAFETY: LaunchServices returned an owned `CFErrorRef` on
                // failure according to Create Rule conventions.
                Err(unsafe { TCFType::wrap_under_create_rule(err) })
            } else {
                // SAFETY: LaunchServices returned an owned `CFURLRef` on
                // success according to Create Rule conventions.
                Ok(unsafe { TCFType::wrap_under_create_rule(out_url) })
            }
        }
        let uti = CFString::from_static_string(RUST_UTI);
        let url = inner(uti.as_concrete_TypeRef()).map_err(DetectEditorError::LookupFailed)?;
        Ok(Self { url })
    }

    pub fn open_file(&self, path: impl AsRef<Path>) -> Result<(), OpenFileError> {
        let path = path.as_ref();
        let item_url = CFURL::from_path(path, path.is_dir()).ok_or_else(|| {
            OpenFileError::PathToUrlFailed {
                path: path.to_owned(),
            }
        })?;
        let items = CFArray::from_CFTypes(&[item_url]);
        let spec = ffi::LSLaunchURLSpec::new(
            self.url.as_concrete_TypeRef(),
            items.as_concrete_TypeRef(),
            ffi::kLSLaunchDefaults,
        );
        let status = unsafe { ffi::LSOpenFromURLSpec(&spec, ptr::null_mut()) };
        if status == 0 {
            Ok(())
        } else {
            Err(OpenFileError::LaunchFailed {
                path: path.to_string_lossy().to_string(),
                command: "LSOpenFromURLSpec",
                error: std::io::Error::other(format!("finished with status code {status}")),
            })
        }
    }
}

pub fn open_file_with(
    application: impl AsRef<OsStr>,
    path: impl AsRef<OsStr>,
    env: &Env,
) -> Result<(), OpenFileError> {
    let mut application = application.as_ref().to_os_string();

    if application == "Xcode" {
        if let Ok(xcode_developer_dir) = xcode_developer_dir() {
            // xcode_developer_dir is /Applications/Xcode.app/Contents/Developer
            // we want to open the app in /Applications/Xcode.app
            let xcode_app_dir = xcode_developer_dir
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(&xcode_developer_dir);
            if xcode_app_dir.extension().unwrap_or_default() == "app" {
                application = xcode_app_dir.to_path_buf().into_os_string();
                log::debug!(
                    "Using Xcode app directory from `xcode-select -p`: {}",
                    application.to_string_lossy()
                );
            } else {
                log::debug!(
                    "Xcode directory {} from `xcode-select -p` is not a valid Xcode app path",
                    xcode_developer_dir.display()
                );
            }
        }
    }

    let application_ = application.clone();
    let path = path.as_ref().to_os_string();
    duct::cmd("open", ["-a"])
        .before_spawn(move |cmd| {
            cmd.arg(&application_).arg(&path);
            Ok(())
        })
        .vars(env.explicit_env())
        .run_and_detach()
        .map_err(|error| OpenFileError::LaunchFailed {
            path: application.to_string_lossy().to_string(),
            command: "open -a",
            error,
        })?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn command_path(name: &str) -> std::io::Result<std::process::Output> {
    duct::cmd("command", ["-v", name]).dup_stdio().run()
}

pub fn code_command() -> duct::Expression {
    duct::cmd!("code")
}

pub fn replace_path_separator(path: OsString) -> OsString {
    path
}

pub fn open_in_xcode(path: impl AsRef<OsStr>) -> Result<(), OpenFileError> {
    duct::cmd("xed", [path.as_ref()])
        .run_and_detach()
        .map_err(|error| OpenFileError::LaunchFailed {
            path: path.as_ref().to_string_lossy().to_string(),
            command: "xed",
            error,
        })?;
    Ok(())
}

pub mod consts {
    pub const CLANG: &str = "clang";
    pub const CLANGXX: &str = "clang++";
    pub const AR: &str = "ar";
    pub const LD: &str = "ld";
    pub const READELF: &str = "readelf";
    pub const NDK_STACK: &str = "ndk-stack";
}

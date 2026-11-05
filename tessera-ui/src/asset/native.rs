use std::{io, path::PathBuf};

pub(super) fn path(asset: &str) -> io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let root = executable
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent directory"))?;
    let standalone = executable.with_extension("resources");
    if standalone.is_dir() {
        return Ok(standalone.join(asset));
    }
    #[cfg(target_os = "macos")]
    {
        let resources = root.join("../Resources");
        if root.file_name().is_some_and(|name| name == "MacOS") && resources.is_dir() {
            return Ok(resources.join(asset));
        }
    }
    #[cfg(target_os = "linux")]
    {
        let resources = root.join("../lib").join(
            executable
                .file_stem()
                .ok_or_else(|| io::Error::other("executable has no name"))?,
        );
        if resources.is_dir() {
            return Ok(resources.join(asset));
        }
    }
    Ok(root.join(asset))
}

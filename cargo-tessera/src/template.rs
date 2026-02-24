use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow};
use handlebars::Handlebars;
use include_dir::{Dir, DirEntry};

fn write_if_changed(path: &Path, new_bytes: &[u8]) -> Result<()> {
    match fs::read(path) {
        Ok(existing) if existing == new_bytes => return Ok(()),
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("Failed to read {}", path.display()));
        }
    }

    fs::write(path, new_bytes).with_context(|| format!("Failed to write {}", path.display()))
}

pub fn write_template_dir(
    dir: &Dir<'_>,
    out_root: &Path,
    handlebars: &Handlebars<'_>,
    data: &serde_json::Value,
) -> Result<()> {
    write_template_dir_at(dir, out_root, Path::new(""), handlebars, data)
}

pub fn write_template_dir_at(
    dir: &Dir<'_>,
    out_root: &Path,
    base: &Path,
    handlebars: &Handlebars<'_>,
    data: &serde_json::Value,
) -> Result<()> {
    let strip_base = !base.as_os_str().is_empty();
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(subdir) => {
                let rel_path = if strip_base {
                    subdir.path().strip_prefix(base).unwrap_or(subdir.path())
                } else {
                    subdir.path()
                };
                let out_dir = out_root.join(rel_path);
                fs::create_dir_all(&out_dir)
                    .with_context(|| format!("Failed to create directory {}", out_dir.display()))?;
                write_template_dir_at(subdir, out_root, base, handlebars, data)?;
            }
            DirEntry::File(file) => {
                let rel_path = if strip_base {
                    file.path().strip_prefix(base).unwrap_or(file.path())
                } else {
                    file.path()
                };
                let mut out_path = out_root.join(rel_path);
                let is_template = out_path.extension().is_some_and(|ext| ext == "hbs");
                if is_template {
                    out_path.set_extension("");
                }

                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create directory {}", parent.display())
                    })?;
                }

                if is_template {
                    let contents = file.contents_utf8().ok_or_else(|| {
                        anyhow!("Template is not valid UTF-8: {}", file.path().display())
                    })?;
                    let rendered = handlebars
                        .render_template(contents, data)
                        .with_context(|| format!("Failed to render {}", file.path().display()))?;
                    write_if_changed(&out_path, rendered.as_bytes())?;
                } else {
                    write_if_changed(&out_path, file.contents())?;
                }

                #[cfg(unix)]
                if out_path.file_name().is_some_and(|f| f == "gradlew") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&out_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&out_path, perms)?;
                }

                #[cfg(windows)]
                if out_path.file_name().is_some_and(|f| f == "gradlew") {
                    mark_gradlew_executable_with_git(&out_path);
                }
            }
        }
    }
    Ok(())
}

pub fn write_template_file(
    dir: &Dir<'_>,
    template_path: &Path,
    out_root: &Path,
    handlebars: &Handlebars<'_>,
    data: &serde_json::Value,
) -> Result<()> {
    let file = dir
        .get_file(template_path)
        .ok_or_else(|| anyhow!("Template file not found: {}", template_path.display()))?;

    let mut out_path = out_root.join(template_path);
    let is_template = out_path.extension().is_some_and(|ext| ext == "hbs");
    if is_template {
        out_path.set_extension("");
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    if is_template {
        let contents = file
            .contents_utf8()
            .ok_or_else(|| anyhow!("Template is not valid UTF-8: {}", template_path.display()))?;
        let rendered = handlebars
            .render_template(contents, data)
            .with_context(|| format!("Failed to render {}", template_path.display()))?;
        write_if_changed(&out_path, rendered.as_bytes())?;
    } else {
        write_if_changed(&out_path, file.contents())?;
    }

    #[cfg(unix)]
    if out_path.file_name().is_some_and(|f| f == "gradlew") {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&out_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&out_path, perms)?;
    }

    #[cfg(windows)]
    if out_path.file_name().is_some_and(|f| f == "gradlew") {
        mark_gradlew_executable_with_git(&out_path);
    }

    Ok(())
}

#[cfg(windows)]
fn mark_gradlew_executable_with_git(out_path: &Path) {
    use std::{path::PathBuf, process::Command};

    let parent = out_path.parent().unwrap_or_else(|| Path::new("."));
    let Ok(output) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(parent)
        .output()
    else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let root = String::from_utf8_lossy(&output.stdout);
    let root = root.trim();
    if root.is_empty() {
        return;
    }
    let root_path = PathBuf::from(root);
    let Ok(rel_path) = out_path.strip_prefix(&root_path) else {
        return;
    };
    let rel_path = rel_path.to_string_lossy().replace('\\', "/");
    let _ = Command::new("git")
        .args(["update-index", "--chmod=+x", &rel_path])
        .current_dir(root_path)
        .status();
}

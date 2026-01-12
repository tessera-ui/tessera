use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow};
use handlebars::Handlebars;
use include_dir::{Dir, DirEntry};

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
                    fs::write(&out_path, rendered)
                        .with_context(|| format!("Failed to write {}", out_path.display()))?;
                } else {
                    fs::write(&out_path, file.contents())
                        .with_context(|| format!("Failed to write {}", out_path.display()))?;
                }

                #[cfg(unix)]
                if out_path.file_name().is_some_and(|f| f == "gradlew") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&out_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&out_path, perms)?;
                }
            }
        }
    }
    Ok(())
}

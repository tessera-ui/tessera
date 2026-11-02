use std::{
    fs,
    io::{self, ErrorKind, IsTerminal},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};

use crate::output;

mod scope;

use scope::InstallScope;

static SKILL: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills/tessera-good-taste");

fn skill_files<'a>(dir: &'a Dir<'_>) -> Vec<(&'a Path, &'a [u8])> {
    let mut files: Vec<_> = dir
        .files()
        .map(|file| (file.path(), file.contents()))
        .collect();
    for child in dir.dirs() {
        files.extend(skill_files(child));
    }
    files
}

pub fn execute(project: Option<&Path>, force: bool, global: bool, local: bool) -> Result<()> {
    let scope = scope::resolve(
        global,
        local,
        io::stdin().is_terminal() && io::stdout().is_terminal(),
        scope::prompt,
    )?;
    install_selected(scope, project, force, dirs::home_dir)
}

fn installation_root(
    scope: InstallScope,
    project: Option<&Path>,
    home: impl FnOnce() -> Option<PathBuf>,
) -> Result<PathBuf> {
    match scope {
        InstallScope::Local => Ok(project.unwrap_or_else(|| Path::new(".")).to_path_buf()),
        InstallScope::Global => {
            if project.is_some() {
                bail!(
                    "--path is only supported for local installation. Choose local or omit --path."
                );
            }
            home().context("Could not locate the user home directory for global installation")
        }
    }
}

fn install_selected(
    scope: Option<InstallScope>,
    project: Option<&Path>,
    force: bool,
    home: impl FnOnce() -> Option<PathBuf>,
) -> Result<()> {
    let Some(scope) = scope else {
        output::note("Skill installation cancelled; no files were changed.");
        return Ok(());
    };
    install(&installation_root(scope, project, home)?, force)
}

fn install(project: &Path, force: bool) -> Result<()> {
    if !project.is_dir() {
        bail!(
            "Installation directory does not exist: {}",
            project.display()
        );
    }
    let destination = project.join(".agents/skills/tessera-good-taste");
    let mut writes = Vec::new();
    for (relative, contents) in skill_files(&SKILL) {
        let path = destination.join(relative);
        match fs::read(&path) {
            Ok(existing) if existing == contents => continue,
            Ok(_) if !force => {
                bail!(
                    "Skill file differs: {}. Use --force to replace bundled skill files.",
                    path.display()
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("Failed to read {}", path.display()));
            }
        }
        writes.push((path, contents));
    }
    for (path, contents) in writes {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    output::status("Installed", destination.display().to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{InstallScope, SKILL, install, install_selected, installation_root, scope};

    struct ProjectDirectory(PathBuf);

    impl ProjectDirectory {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("tessera-skill-install-{}-{id}", std::process::id()));
            fs::create_dir(&path).expect("isolated project directory");
            Self(path)
        }
    }

    impl Drop for ProjectDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn scopes_install_into_project_or_home_without_touching_the_other_root() {
        let project = ProjectDirectory::new();
        let home = ProjectDirectory::new();
        let relative = ".agents/skills/tessera-good-taste/SKILL.md";
        install_selected(Some(InstallScope::Local), Some(&project.0), false, || {
            panic!("local installation must not resolve home")
        })
        .unwrap();
        assert!(project.0.join(relative).is_file());
        assert!(!home.0.join(".agents").exists());

        install_selected(Some(InstallScope::Global), None, false, || {
            Some(home.0.clone())
        })
        .unwrap();
        assert_eq!(
            fs::read(home.0.join(relative)).unwrap(),
            SKILL.get_file("SKILL.md").unwrap().contents()
        );
        assert_eq!(
            installation_root(InstallScope::Local, None, || panic!(
                "must not resolve home"
            ))
            .unwrap(),
            PathBuf::from(".")
        );
    }

    #[test]
    fn cancelled_prompt_does_not_create_directories_or_resolve_home() {
        let project = ProjectDirectory::new();
        let missing = project.0.join("not-created");
        let selected = scope::resolve(false, false, true, || Ok(None)).unwrap();
        install_selected(selected, Some(&missing), true, || panic!("cancelled")).unwrap();
        assert!(!missing.exists());
        assert!(!project.0.join(".agents").exists());
    }

    #[test]
    fn global_rejects_a_project_override_before_home_lookup_or_installation() {
        let project = ProjectDirectory::new();
        let error = install_selected(Some(InstallScope::Global), Some(&project.0), false, || {
            panic!("must reject --path before home lookup")
        })
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("--path is only supported for local")
        );
        assert!(!project.0.join(".agents").exists());
        assert!(installation_root(InstallScope::Global, None, || None).is_err());
    }

    #[test]
    fn global_installation_preserves_modified_and_unrelated_files() {
        let home = ProjectDirectory::new();
        install_selected(Some(InstallScope::Global), None, false, || {
            Some(home.0.clone())
        })
        .unwrap();
        let destination = home.0.join(".agents/skills/tessera-good-taste");
        let entry = destination.join("SKILL.md");
        fs::write(&entry, "User skill").unwrap();
        let custom = destination.join("custom.md");
        fs::write(&custom, "User notes").unwrap();
        let error = install_selected(Some(InstallScope::Global), None, false, || {
            Some(home.0.clone())
        })
        .unwrap_err();
        assert!(error.to_string().contains("Skill file differs:"));
        assert_eq!(fs::read_to_string(&entry).unwrap(), "User skill");
        install_selected(Some(InstallScope::Global), None, true, || {
            Some(home.0.clone())
        })
        .unwrap();
        assert_eq!(
            fs::read(&entry).unwrap(),
            SKILL.get_file("SKILL.md").unwrap().contents()
        );
        assert_eq!(fs::read_to_string(custom).unwrap(), "User notes");
    }

    #[test]
    fn skill_installation_preserves_project_guidance_and_other_skills() {
        let project = ProjectDirectory::new();
        let guidance = project.0.join("AGENTS.md");
        fs::write(&guidance, "Existing project instructions\n").expect("project guidance");
        let other = project.0.join(".agents/skills/other/SKILL.md");
        fs::create_dir_all(other.parent().expect("skill parent")).expect("other skill directory");
        fs::write(&other, "Other skill\n").expect("other skill");

        install(&project.0, false).expect("install bundled skill");
        install(&project.0, false).expect("repeat matching installation");

        for relative in [
            "SKILL.md",
            "references/public-apis.md",
            "references/composition.md",
            "references/state-and-resources.md",
            "references/layout-and-visuals.md",
            "references/interaction.md",
            "references/platform-and-extensions.md",
            "references/headless.md",
            "references/tooling.md",
        ] {
            let expected = SKILL
                .get_file(relative)
                .expect("bundled reference")
                .contents();
            assert_eq!(
                fs::read(
                    project
                        .0
                        .join(".agents/skills/tessera-good-taste")
                        .join(relative)
                )
                .expect("installed skill file"),
                expected
            );
        }
        assert_eq!(
            fs::read_to_string(guidance).expect("preserved guidance"),
            "Existing project instructions\n"
        );
        assert_eq!(
            fs::read_to_string(other).expect("preserved other skill"),
            "Other skill\n"
        );
    }

    #[test]
    fn skill_installation_requires_force_for_modified_files_before_writing_anything() {
        let project = ProjectDirectory::new();
        let reference = project
            .0
            .join(".agents/skills/tessera-good-taste/references/public-apis.md");
        fs::create_dir_all(reference.parent().expect("reference parent"))
            .expect("customized skill directory");
        fs::write(&reference, "Local reference\n").expect("customized reference");
        let entry = project.0.join(".agents/skills/tessera-good-taste/SKILL.md");

        let error = install(&project.0, false).expect_err("modified file requires explicit force");
        assert!(error.to_string().contains("Skill file differs:"));
        assert!(!entry.exists());
        assert_eq!(
            fs::read_to_string(&reference).expect("preserved reference"),
            "Local reference\n"
        );

        install(&project.0, true).expect("explicit replacement");
        assert_eq!(
            fs::read(reference).expect("replaced reference"),
            SKILL
                .get_file("references/public-apis.md")
                .expect("bundled reference")
                .contents()
        );
        assert!(entry.is_file());
        assert!(!project.0.join("AGENTS.md").exists());
    }
}

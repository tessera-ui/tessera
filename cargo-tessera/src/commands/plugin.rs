use std::{fs, path::Path, time::Instant};

use anyhow::{Context, Result};
use handlebars::Handlebars;
use include_dir::{Dir, include_dir};
use inquire::Select as ChoicePrompt;
use serde_json::json;

use crate::{
    output,
    template::{write_project_guidance, write_template_dir_at},
};

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/plugin");

pub fn select_template_interactive() -> Result<String> {
    let templates: Vec<String> = TEMPLATES
        .dirs()
        .filter_map(|d| d.path().file_name()?.to_str().map(|s| s.to_string()))
        .collect();

    if templates.is_empty() {
        anyhow::bail!("No plugin templates found");
    }

    if templates.len() == 1 {
        return Ok(templates[0].clone());
    }

    let selection = ChoicePrompt::new("Select a plugin template", templates.clone())
        .with_help_message("Use arrow keys to navigate, Enter to confirm")
        .prompt()?;

    Ok(selection)
}

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_dir).context("Failed to create plugin directory")?;
    let started = Instant::now();
    generate_from_template(project_dir, template)?;

    let duration = output::format_duration(started.elapsed());
    output::status(
        "Created",
        format!(
            "tessera plugin `{}` (template `{}`) in {}",
            name, template, duration
        ),
    );
    output::note("Next steps:");
    output::step(format!("cd {}", name));
    output::step("cargo build");

    Ok(())
}

fn generate_from_template(project_dir: &Path, template: &str) -> Result<()> {
    let template_dir = TEMPLATES
        .get_dir(template)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template))?;

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tessera-plugin")
        .to_string();
    let project_name_snake = project_name.replace('-', "_");
    let android_package = format!("com.tessera.plugin.{project_name_snake}");
    let android_package_path = android_package.replace('.', "/");

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let data = json!({
        "project_name": project_name,
        "project_name_snake": project_name_snake,
        "android": {
            "package": android_package,
            "package_path": android_package_path,
        },
    });

    write_template_dir_at(
        template_dir,
        project_dir,
        template_dir.path(),
        &handlebars,
        &data,
    )?;
    write_project_guidance(project_dir)
}

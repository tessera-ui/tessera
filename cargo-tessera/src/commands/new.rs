use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use comfy_table::{
    Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS as RoundCorners,
    presets::UTF8_FULL,
};
use dialoguer::{Input, Select};
use include_dir::{Dir, include_dir};
use owo_colors::colored::*;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Prompt for project name interactively with validation
pub fn prompt_project_name() -> Result<String> {
    loop {
        let name: String = Input::new().with_prompt("Project name").interact_text()?;

        // Validate project name
        if name.is_empty() {
            println!("{}", "❌ Project name cannot be empty".red());
            continue;
        }

        // Check if name contains invalid characters
        // Valid Rust/Cargo package names: lowercase letters, numbers, hyphens, underscores
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            println!(
                "{}",
                "❌ Project name must contain only lowercase letters, numbers, hyphens, and underscores"
                    .red()
            );
            continue;
        }

        // Check if starts with a letter
        if !name.chars().next().unwrap().is_ascii_lowercase() {
            println!(
                "{}",
                "❌ Project name must start with a lowercase letter".red()
            );
            continue;
        }

        // Check if directory already exists
        if Path::new(&name).exists() {
            println!(
                "{}",
                format!("❌ Directory '{}' already exists", name).red()
            );
            continue;
        }

        return Ok(name);
    }
}

pub fn execute(name: &str, template: &str) -> Result<()> {
    println!("{}", "🎨 Creating new Tessera project...".bright_cyan());

    let project_dir = Path::new(name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create project directory
    fs::create_dir_all(project_dir).context("Failed to create project directory")?;

    // Generate project from template
    generate_from_template(project_dir, template)?;

    println!(
        "\n{} Project '{}' created successfully!",
        "✅".green(),
        name.bright_green()
    );
    print_project_summary(name, template);

    Ok(())
}

/// Select template interactively if not specified
pub fn select_template_interactive() -> Result<String> {
    let templates: Vec<&str> = TEMPLATES
        .dirs()
        .map(|d| d.path().file_name().unwrap().to_str().unwrap())
        .collect();

    if templates.is_empty() {
        anyhow::bail!("No templates found");
    }

    if templates.len() == 1 {
        return Ok(templates[0].to_string());
    }

    let selection = Select::new()
        .with_prompt("Select a template")
        .items(&templates)
        .default(0)
        .interact()?;

    Ok(templates[selection].to_string())
}

fn generate_from_template(project_dir: &Path, template: &str) -> Result<()> {
    // Find template directory
    let template_dir = TEMPLATES
        .get_dir(template)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template))?;

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tessera-app");

    // Template variables for substitution
    let mut vars = HashMap::new();
    vars.insert("project_name", project_name);

    copy_dir(template_dir, project_dir, &vars)
}

fn copy_dir(dir: &Dir, dest: &Path, vars: &HashMap<&str, &str>) -> Result<()> {
    fs::create_dir_all(dest).context(format!("Failed to create {}", dest.display()))?;

    for file in dir.files() {
        let filename = file.path().file_name().ok_or_else(|| {
            anyhow::anyhow!("Invalid file path in template: {}", file.path().display())
        })?;

        let content = file.contents_utf8().ok_or_else(|| {
            anyhow::anyhow!(
                "Template file '{}' is not valid UTF-8",
                file.path().display()
            )
        })?;

        let processed_content = apply_template_vars(content, vars);
        let output_path = dest.join(filename);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, processed_content)
            .context(format!("Failed to write {}", output_path.display()))?;
    }

    for subdir in dir.dirs() {
        let dirname = subdir.path().file_name().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid directory path in template: {}",
                subdir.path().display()
            )
        })?;

        copy_dir(subdir, &dest.join(dirname), vars)?;
    }

    Ok(())
}

/// Simple template variable substitution
fn apply_template_vars(content: &str, vars: &HashMap<&str, &str>) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

fn print_project_summary(name: &str, template: &str) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(RoundCorners)
        .set_width(60)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("✨ Field").add_attribute(Attribute::Bold),
            Cell::new("Details").add_attribute(Attribute::Bold),
        ]);

    table.add_row(vec![
        Cell::new("Name"),
        Cell::new(format!("{}", name.bright_green())),
    ]);

    table.add_row(vec![
        Cell::new("Template"),
        Cell::new(format!("{}", template.cyan())),
    ]);

    let next_steps = format!("cd {}\ncargo tessera dev", name);
    table.add_row(vec![Cell::new("Next"), Cell::new(next_steps)]);

    println!("\n{table}");
}

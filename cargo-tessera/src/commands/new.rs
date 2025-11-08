use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use comfy_table::{
    Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS as RoundCorners,
    presets::UTF8_FULL,
};
use include_dir::{Dir, include_dir};
use inquire::{
    Select as ChoicePrompt, Text,
    error::CustomUserError,
    ui::{Attributes, Color, ErrorMessageRenderConfig, RenderConfig, StyleSheet, Styled},
    validator::Validation,
};
use owo_colors::colored::*;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Prompt for project name interactively with validation
pub fn prompt_project_name() -> Result<String> {
    let validator = |input: &str| -> Result<Validation, CustomUserError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(Validation::Invalid(
                "Project name cannot be empty".to_string().into(),
            ));
        }

        if !trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Ok(Validation::Invalid(
                "Only lowercase letters, digits, '-' and '_' are allowed".into(),
            ));
        }

        if !trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
        {
            return Ok(Validation::Invalid(
                "Project name must start with a lowercase letter".into(),
            ));
        }

        if Path::new(trimmed).exists() {
            return Ok(Validation::Invalid(
                format!("Directory '{trimmed}' already exists").into(),
            ));
        }

        Ok(Validation::Valid)
    };

    let name = Text::new("Project name")
        .with_render_config(prompt_theme())
        .with_help_message("lowercase, numbers, '-' or '_', must start with a letter")
        .with_placeholder("my-tessera-app")
        .with_validator(validator)
        .prompt()?;

    Ok(name.trim().to_string())
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
    let templates: Vec<String> = TEMPLATES
        .dirs()
        .map(|d| d.path().file_name().unwrap().to_str().unwrap().to_string())
        .collect();

    if templates.is_empty() {
        anyhow::bail!("No templates found");
    }

    if templates.len() == 1 {
        return Ok(templates[0].clone());
    }

    let selection = ChoicePrompt::new("Select a template", templates.clone())
        .with_render_config(prompt_theme())
        .with_help_message("Use ↑ ↓ to navigate, Enter to confirm")
        .prompt()?;

    Ok(selection)
}

fn generate_from_template(project_dir: &Path, template: &str) -> Result<()> {
    // Find template directory
    let template_dir = TEMPLATES
        .get_dir(template)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template))?;

    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("tessera-app")
        .to_string();
    let project_name_snake = project_name.replace('-', "_");

    // Template variables for substitution
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("project_name", project_name);
    vars.insert("project_name_snake", project_name_snake);

    copy_dir(template_dir, project_dir, &vars)
}

fn copy_dir(dir: &Dir, dest: &Path, vars: &HashMap<&str, String>) -> Result<()> {
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
fn apply_template_vars(content: &str, vars: &HashMap<&str, String>) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

fn prompt_theme() -> RenderConfig<'static> {
    let accent = Color::LightCyan;
    let mut config = RenderConfig::default_colored()
        .with_prompt_prefix(Styled::new("❯").with_fg(accent))
        .with_answered_prompt_prefix(Styled::new("✔").with_fg(Color::LightGreen))
        .with_canceled_prompt_indicator(Styled::new("✖ cancelled").with_fg(Color::LightRed))
        .with_highlighted_option_prefix(Styled::new("›").with_fg(accent))
        .with_scroll_up_prefix(Styled::new("↑").with_fg(Color::DarkGrey))
        .with_scroll_down_prefix(Styled::new("↓").with_fg(Color::DarkGrey))
        .with_selected_checkbox(Styled::new("◉").with_fg(accent))
        .with_unselected_checkbox(Styled::new("○").with_fg(Color::DarkGrey))
        .with_error_message(
            ErrorMessageRenderConfig::default_colored()
                .with_prefix(Styled::new("⚠").with_fg(Color::LightRed)),
        );

    config.prompt = StyleSheet::new()
        .with_fg(Color::White)
        .with_attr(Attributes::BOLD);
    config.answer = StyleSheet::new().with_fg(accent);
    config.placeholder = StyleSheet::new().with_fg(Color::DarkGrey);
    config.help_message = StyleSheet::new().with_fg(Color::DarkGrey);
    config.text_input = StyleSheet::new().with_fg(Color::White);
    config
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

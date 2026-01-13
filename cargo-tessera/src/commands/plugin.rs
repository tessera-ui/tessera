use std::{fs, path::Path};

use anyhow::{Context, Result};
use comfy_table::{
    Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS as RoundCorners,
    presets::UTF8_FULL,
};
use handlebars::Handlebars;
use include_dir::{Dir, include_dir};
use inquire::{
    Select as ChoicePrompt, Text,
    error::CustomUserError,
    ui::{Attributes, Color, ErrorMessageRenderConfig, RenderConfig, StyleSheet, Styled},
    validator::Validation,
};
use owo_colors::colored::*;
use serde_json::json;

use crate::template::write_template_dir_at;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/plugin");

pub fn prompt_plugin_name() -> Result<String> {
    let validator = |input: &str| -> Result<Validation, CustomUserError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(Validation::Invalid(
                "Plugin name cannot be empty".to_string().into(),
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
                "Plugin name must start with a lowercase letter".into(),
            ));
        }

        if Path::new(trimmed).exists() {
            return Ok(Validation::Invalid(
                format!("Directory '{trimmed}' already exists").into(),
            ));
        }

        Ok(Validation::Valid)
    };

    let name = Text::new("Plugin name")
        .with_render_config(prompt_theme())
        .with_help_message("lowercase, numbers, '-' or '_', must start with a letter")
        .with_placeholder("tessera-plugin")
        .with_validator(validator)
        .prompt()?;

    Ok(name.trim().to_string())
}

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
        .with_render_config(prompt_theme())
        .with_help_message("Use ↑ ↓ to navigate, Enter to confirm")
        .prompt()?;

    Ok(selection)
}

pub fn execute(name: &str, template: &str) -> Result<()> {
    println!("{}", "Creating new Tessera plugin...".bright_cyan());

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_dir).context("Failed to create plugin directory")?;
    generate_from_template(project_dir, template)?;

    println!(
        "\n{} Plugin '{}' created successfully!",
        "Success".green(),
        name.bright_green()
    );
    print_plugin_summary(name, template);

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
    )
}

fn prompt_theme() -> RenderConfig<'static> {
    let accent = Color::LightCyan;
    let mut config = RenderConfig::default_colored()
        .with_prompt_prefix(Styled::new(">").with_fg(accent))
        .with_answered_prompt_prefix(Styled::new("ok").with_fg(Color::LightGreen))
        .with_canceled_prompt_indicator(Styled::new("cancelled").with_fg(Color::LightRed))
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

fn print_plugin_summary(name: &str, template: &str) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(RoundCorners)
        .set_width(60)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Field").add_attribute(Attribute::Bold),
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

    let next_steps = format!("cd {}\ncargo build", name);
    table.add_row(vec![Cell::new("Next"), Cell::new(next_steps)]);

    println!("\n{table}");
}

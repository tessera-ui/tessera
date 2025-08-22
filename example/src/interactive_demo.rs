use std::sync::Arc;

use tessera_ui::{Color, Dp, tessera};
use tessera_ui_basic_components::{
    button::{ButtonArgsBuilder, button},
    checkbox::{CheckboxArgsBuilder, checkbox},
    column::ColumnArgsBuilder,
    column_ui,
    glass_button::{GlassButtonArgs, glass_button},
    row::RowArgsBuilder,
    row_ui,
    text::{TextArgsBuilder, text},
};

use crate::{app_state::AppState, material_colors::md_colors, misc::create_spacer};

fn title_component() {
    text(
        TextArgsBuilder::default()
            .text("Interactive Components Demo".to_string())
            .size(tessera_ui::Dp(24.0))
            .color(md_colors::ON_SURFACE)
            .build()
            .unwrap(),
    )
}

fn buttons_heading() {
    text(
        TextArgsBuilder::default()
            .text("Interactive Buttons with Hover Effects:".to_string())
            .size(tessera_ui::Dp(18.0))
            .color(md_colors::ON_SURFACE_VARIANT)
            .build()
            .unwrap(),
    )
}

fn primary_button_component(app_state: Arc<AppState>) {
    let state = app_state.primary_button_ripple.clone();
    button(
        ButtonArgsBuilder::default()
            .color(md_colors::PRIMARY)
            .hover_color(Some(Color::new(0.3, 0.6, 0.9, 1.0)))
            .padding(Dp(12.0))
            .on_click(Arc::new(|| {
                println!("Primary button clicked!");
            }))
            .build()
            .unwrap(),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Primary Button (Hover Effect)".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn success_button_component(app_state: Arc<AppState>) {
    let state = app_state.success_button_ripple.clone();
    button(
        ButtonArgsBuilder::default()
            .color(md_colors::TERTIARY)
            .hover_color(Some(Color::new(0.2, 0.8, 0.4, 1.0)))
            .padding(Dp(12.0))
            .on_click(Arc::new(|| {
                println!("Success button clicked!");
            }))
            .build()
            .unwrap(),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Success Button (Hover Effect)".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn danger_button_component(app_state: Arc<AppState>) {
    let state = app_state.danger_button_ripple.clone();
    button(
        ButtonArgsBuilder::default()
            .color(md_colors::ERROR)
            .hover_color(Some(Color::new(0.9, 0.3, 0.3, 1.0)))
            .padding(Dp(12.0))
            .on_click(Arc::new(|| {
                println!("Danger button clicked!");
            }))
            .build()
            .unwrap(),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Danger Button (Hover Effect)".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn checkbox_row_component(app_state: Arc<AppState>) {
    let checked = *app_state.checkbox_state.checked.read();
    let on_toggle = {
        let checked_arc = app_state.checkbox_state.checked.clone();
        Arc::new(move |new_checked| {
            *checked_arc.write() = new_checked;
        })
    };

    row_ui!(
        RowArgsBuilder::default()
            .cross_axis_alignment(
                tessera_ui_basic_components::alignment::CrossAxisAlignment::Center
            )
            .build()
            .unwrap(),
        move || checkbox(
            CheckboxArgsBuilder::default()
                .checked(checked)
                .on_toggle(on_toggle)
                .state(Some(app_state.checkbox_state.state.clone()))
                .build()
                .unwrap()
        ),
        || create_spacer(8)(),
        move || {
            let label = if checked {
                "Checkbox is ON (GPU-rendered checkmark)"
            } else {
                "Checkbox is OFF (Click to see animation)"
            };
            text(
                TextArgsBuilder::default()
                    .text(label.to_string())
                    .color(md_colors::ON_SURFACE)
                    .build()
                    .unwrap(),
            )
        }
    )
}

fn glass_button_primary(app_state: Arc<AppState>) {
    let state = app_state.primary_glass_button_ripple.clone();
    glass_button(
        GlassButtonArgs::primary(Arc::new(|| {
            println!("Primary Glass button clicked!");
        })),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Primary Glass Button".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn glass_button_secondary(app_state: Arc<AppState>) {
    let state = app_state.secondary_glass_button_ripple.clone();
    glass_button(
        GlassButtonArgs::secondary(Arc::new(|| {
            println!("Secondary Glass button clicked!");
        })),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Secondary Glass Button".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn glass_button_success(app_state: Arc<AppState>) {
    let state = app_state.success_glass_button_ripple.clone();
    glass_button(
        GlassButtonArgs::success(Arc::new(|| {
            println!("Success Glass button clicked!");
        })),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Success Glass Button".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

fn glass_button_danger(app_state: Arc<AppState>) {
    let state = app_state.danger_glass_button_ripple.clone();
    glass_button(
        GlassButtonArgs::danger(Arc::new(|| {
            println!("Danger Glass button clicked!");
        })),
        state,
        || {
            text(
                TextArgsBuilder::default()
                    .text("Danger Glass Button".to_string())
                    .color(md_colors::ON_SURFACE)
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        },
    )
}

/// Demo component showcasing interactive surfaces and buttons
#[tessera]
pub fn interactive_demo(app_state: Arc<AppState>) {
    column_ui!(
        ColumnArgsBuilder::default().build().unwrap(),
        || title_component(),
        || (create_spacer(16))(),
        || buttons_heading(),
        {
            let app_state = app_state.clone();
            move || primary_button_component(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || success_button_component(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || danger_button_component(app_state.clone())
        },
        || (create_spacer(16))(),
        || {
            text(
                TextArgsBuilder::default()
                    .text("Animated Checkboxes with Custom Checkmark:".to_string())
                    .size(tessera_ui::Dp(18.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        },
        {
            let app_state = app_state.clone();
            move || checkbox_row_component(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || glass_button_primary(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || glass_button_secondary(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || glass_button_success(app_state.clone())
        },
        || (create_spacer(8))(),
        {
            let app_state = app_state.clone();
            move || glass_button_danger(app_state.clone())
        },
    )
}

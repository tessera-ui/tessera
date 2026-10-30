//! Interactive Material component specification pages.
//!
//! ## Usage
//!
//! Explore component anatomy, configurations, tokens and behavior in context.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    text_table::{TextTableCell, TextTableHeading, text_table},
    theme::TextStyle,
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, tessera};

mod action_articles;
mod annotations;
mod app_bars;
mod article;
mod article_navigation;
mod article_tab_visual;
mod badges;
mod button_groups;
mod buttons;
mod cards;
mod carousel;
mod catalog;
mod chips;
mod date_pickers;
mod dialogs;
mod dividers;
mod drawers;
mod extended_fabs;
mod fab_menus;
mod floating_actions;
mod icon_buttons;
mod lists;
mod loading_indicator;
mod media;
mod menus;
mod navigation;
mod navigation_bar;
mod navigation_rail;
mod progress_indicators;
mod radio_buttons;
mod scroll_viewport;
mod search;
mod segmented;
mod selection_articles;
mod selection_controls;
mod sheets;
mod sliders;
mod sliders_tokens;
mod snackbars;
mod split_buttons;
mod tabs;
mod text_fields;
mod text_fields_accessibility;
mod time_pickers;
mod toolbars;
mod tooltips;

const BACKGROUND: Color = Color::from_rgb(0.991102097, 0.964686248, 1.000000000);
const PANEL: Color = Color::from_rgb(0.938685728, 0.879622397, 0.921581856);
const INK: Color = Color::from_rgb(0.011612245, 0.010960094, 0.012286488);
const ACCENT: Color = Color::from_rgb(0.1356333, 0.08021982, 0.3712377);

fn srgb(r: u8, g: u8, b: u8) -> Color {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::from_rgb(channel(r), channel(g), channel(b))
}

fn type_style(size: f64, line: f64) -> TextStyle {
    TextStyle {
        font_size: Dp(size),
        line_height: Some(Dp(line)),
        font_family: Some(if size >= 24.0 {
            "Example Sans Display"
        } else {
            "Example Sans Text"
        }),
        font_weight: if size >= 24.0 { 475 } else { 400 },
        ..Default::default()
    }
}
#[tessera]
fn gap(height: Dp) {
    spacer().modifier(Modifier::new().height(height));
}

#[tessera]
fn heading(label: String, compact: Option<bool>, level: Option<u8>) {
    let level = level.unwrap_or(2).clamp(2, 4);
    text()
        .content(label)
        .style(if level >= 3 {
            type_style(24.0, 32.0)
        } else if compact.unwrap_or(false) {
            type_style(32.0, 40.0)
        } else {
            type_style(45.0, 52.0)
        })
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(tessera_ui::accesskit::Role::Heading),
            heading_level: Some(u32::from(level)),
            ..Default::default()
        }))
        .color(INK);
}

#[tessera]
fn subheading(label: String, level: Option<u8>) {
    text()
        .content(label)
        .style(type_style(24.0, 32.0))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(tessera_ui::accesskit::Role::Heading),
            heading_level: Some(u32::from(level.unwrap_or(3).clamp(3, 4))),
            ..Default::default()
        }));
}

#[tessera]
fn panel(content: RenderSlot) {
    surface()
        .style(PANEL.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(Modifier::new().fill_max_width())
        .child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(40.0)))
                .children(move || {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children_shared(content);
                });
        });
}

#[tessera]
fn spec_table(headers: Vec<String>, rows: Vec<Vec<String>>) {
    let cells = rows
        .into_iter()
        .enumerate()
        .flat_map(|(row, values)| {
            values
                .into_iter()
                .enumerate()
                .map(move |(column, text)| TextTableCell {
                    text,
                    row: row + 1,
                    column,
                    ..Default::default()
                })
        })
        .collect();
    spec_table_cells(headers, cells);
}

#[tessera]
fn spec_table_cells(headers: Vec<String>, cells: Vec<TextTableCell>) {
    media::load_fonts();
    let columns = headers.len();
    let mut values = headers
        .into_iter()
        .enumerate()
        .map(|(column, text)| TextTableCell {
            text,
            column,
            heading: TextTableHeading::Column,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    values.extend(cells);
    text_table(values)
        .columns(columns)
        .style(type_style(16.0, 24.0))
        .header_style(type_style(16.0, 24.0))
        .modifier(Modifier::new().fill_max_width())
        .color(INK)
        .header_color(srgb(77, 66, 86))
        .header_background(PANEL);
}

#[tessera]
pub(super) fn material_spec() {
    catalog::component_pages();
}

#[tessera]
fn slider_spec() {
    sliders::slider_spec();
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "material_spec/text_table_adapter_tests.rs"]
mod text_table_adapter_tests;

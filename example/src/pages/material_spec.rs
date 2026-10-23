//! Interactive Material component specification pages.
//!
//! ## Usage
//!
//! Explore component anatomy, configurations, tokens and behavior in context.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::TextStyle,
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, State, remember, tessera};

mod action_articles;
mod activity;
mod annotations;
mod app_bars;
mod article;
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
mod fab_menu;
mod floating_actions;
mod icon_buttons;
mod lists;
mod media;
mod menus;
mod navigation;
mod radio_buttons;
mod search;
mod segmented;
mod selection_articles;
mod selection_controls;
mod sheets;
mod sliders;
mod snackbars;
mod split_buttons;
mod tabs;
mod text_fields;
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

type TableData = (Vec<String>, Vec<Vec<String>>);

#[tessera]
fn spec_table(headers: Vec<String>, rows: Vec<Vec<String>>) {
    let current = (headers, rows);
    let data = remember(|| current.clone());
    if data.with(|data| data != &current) {
        data.set(current);
    }
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(tessera_ui::accesskit::Role::Table),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            table_row(data);
            for index in 0..data.with(|data| data.1.len()) {
                horizontal_divider();
                table_row(data).row_index(index);
            }
        });
}

#[tessera]
fn table_row(data: State<TableData>, row_index: Option<usize>) {
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_all(Dp(16.0))
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Row),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            let values = data.with(|data| match row_index {
                Some(index) => data.1[index].clone(),
                None => data.0.clone(),
            });
            for value in values {
                text()
                    .content(value)
                    .style(TextStyle {
                        font_weight: if row_index.is_none() { 500 } else { 400 },
                        ..type_style(14.0, 20.0)
                    })
                    .modifier(Modifier::new().weight(1.0).semantics(SemanticsArgs {
                        role: Some(if row_index.is_none() {
                            tessera_ui::accesskit::Role::ColumnHeader
                        } else {
                            tessera_ui::accesskit::Role::Cell
                        }),
                        ..Default::default()
                    }));
            }
        });
}

#[tessera]
pub(super) fn material_spec() {
    catalog::component_pages();
}

#[tessera]
fn slider_spec() {
    sliders::slider_spec();
}

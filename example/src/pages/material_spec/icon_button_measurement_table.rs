//! Native corner-radius reference table for icon-button measurements.
//!
//! ## Usage
//!
//! Compare button corners across sizes on narrow screens.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, accesskit::Role, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use crate::pages::material_spec::{media, type_style};

const MIN_WIDTH: Dp = Dp(616.0);
const ROW_HEIGHT: Dp = Dp(56.0);
const CELLS: [[&str; 6]; 4] = [
    ["", "XS", "S", "M", "L", "XL"],
    ["A. Round button", "Full", "Full", "Full", "Full", "Full"],
    ["B. Square button", "12dp", "12dp", "16dp", "28dp", "28dp"],
    ["C. Pressed state", "8dp", "8dp", "12dp", "16dp", "16dp"],
];

#[tessera]
pub(super) fn figure() {
    media::load_fonts();
    let width = remember(|| MIN_WIDTH);
    let scroll = remember(ScrollableController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                    let next = Dp::from(coordinates.local_size.width);
                    if width.get() != next {
                        width.set(next);
                    }
                })
                .horizontal_scroll(scroll),
        )
        .children(move || {
            let shape = Shape::rounded_rectangle(Dp(32.0));
            boxed()
                .modifier(
                    Modifier::new()
                        .width(Dp(width.get().0.max(MIN_WIDTH.0)))
                        .height(Dp(224.0))
                        .background_with_shape(scheme.surface, shape)
                        .clip_shape(shape),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
                            role: Some(Role::Table),
                            label: Some("Icon button corner radius values".into()),
                            collection_info: Some((4, 6, false)),
                            merge_descendants: true,
                            ..Default::default()
                        }))
                        .children(move || {
                            for (index, values) in CELLS.into_iter().enumerate() {
                                table_row(
                                    index,
                                    values.map(str::to_owned),
                                    scheme.on_surface,
                                    scheme.surface_container_low,
                                    scheme.outline_variant,
                                );
                            }
                        });
                    boxed().modifier(Modifier::new().fill_max_size().border_with_shape(
                        Dp(1.0),
                        scheme.outline_variant,
                        shape,
                    ));
                });
        });
}

#[tessera]
fn table_row(index: usize, values: [String; 6], ink: Color, header: Color, rule: Color) {
    let modifier = Modifier::new()
        .fill_max_width()
        .height(ROW_HEIGHT)
        .semantics(SemanticsArgs {
            role: Some(Role::Row),
            collection_item_info: Some((index, 1, 0, 6, false)),
            merge_descendants: true,
            ..Default::default()
        });
    let modifier = if index == 0 {
        modifier.background(header)
    } else {
        modifier
    };
    boxed().modifier(modifier).children(move || {
        let values = values.clone();
        row()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                for (column, value) in values.iter().enumerate() {
                    cell(index, column, value.clone(), ink, rule);
                }
            });
        if index < CELLS.len() - 1 {
            spacer().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(1.0))
                    .align(Alignment::BottomStart)
                    .background(rule),
            );
        }
    });
}

#[tessera]
fn cell(row: usize, column: usize, value: String, ink: Color, rule: Color) {
    let role = if row == 0 && column > 0 {
        Role::ColumnHeader
    } else if row > 0 && column == 0 {
        Role::RowHeader
    } else {
        Role::Cell
    };
    boxed()
        .alignment(if row == 0 {
            Alignment::Center
        } else {
            Alignment::CenterStart
        })
        .modifier(
            Modifier::new()
                .weight(if column == 0 { 2.0 } else { 1.0 })
                .fill_max_height()
                .semantics(SemanticsArgs {
                    role: Some(role),
                    label: Some(value.clone()),
                    collection_item_info: Some((
                        row,
                        1,
                        column,
                        1,
                        matches!(role, Role::ColumnHeader | Role::RowHeader),
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(value.clone())
                .style(type_style(16.0, 24.0))
                .color(ink)
                .modifier(Modifier::new().padding_symmetric(Dp(24.0), Dp(16.0)));
            if column < CELLS[0].len() - 1 {
                spacer().modifier(
                    Modifier::new()
                        .width(Dp(1.0))
                        .fill_max_height()
                        .align(Alignment::CenterEnd)
                        .background(rule),
                );
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_measurement_table_tests.rs"]
mod tests;

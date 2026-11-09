use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, tessera, use_context};

use crate::res::I18N;

use super::{INK, gap, type_style};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

#[tessera]
pub(super) fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
pub(super) fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
pub(super) fn spaced_caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
pub(super) fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text(format!("•  {}", I18N.text(item))).style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
pub(super) fn dot_bullets(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                row()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children(move || {
                        spacer().modifier(
                            Modifier::new()
                                .size(Dp(8.0), Dp(8.0))
                                .background_with_shape(scheme.on_surface_variant, Shape::Ellipse),
                        );
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text(I18N.text(&(item))).style(type_style(16.0, 24.0));
                    });
            }
        });
}

#[tessera]
pub(super) fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

#[tessera]
pub(super) fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(label))).style(type_style(16.0, 24.0));
        });
}

#[tessera]
pub(super) fn guidance_line(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
pub(super) fn do_dont(#[prop(into)] good: String, #[prop(into)] bad: String) {
    column().children(move || {
        guidance_line("Do", good.clone(), DO_COLOR);
        gap(Dp(12.0));
        guidance_line("Don't", bad.clone(), DONT_COLOR);
    });
}

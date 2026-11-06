use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    fluid_glass::fluid_glass,
    icon::icon,
    lazy_list::lazy_column,
    material_icons::filled,
    modifier::{DragDelta, ModifierExt},
    shape_def::Shape,
    slider::slider,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Color, Dp, Modifier, Px, remember, tessera, use_context};

use crate::res::I18N;

const MAX_BLUR_RADIUS: Dp = Dp(20.0);
const LENS_SIZE: Dp = Dp(120.0);
const LENS_ORIGIN: Dp = Dp(48.0);
const TILE_SIZE: Dp = Dp(64.0);
const TILE_SPACING: Dp = Dp(12.0);
const TILE_COLUMNS: usize = 8;
const TILE_ROWS: usize = 3;
const BACKDROP_PADDING: Dp = Dp(24.0);
const DEMO_WIDTH: Dp = Dp(TILE_SIZE.0 * TILE_COLUMNS as f64
    + TILE_SPACING.0 * (TILE_COLUMNS - 1) as f64
    + BACKDROP_PADDING.0 * 2.0);
const CONTENT_MAX_WIDTH: Dp = Dp(720.0);

#[shard]
pub fn glass_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let offset = remember(|| (Px::ZERO, Px::ZERO));
    let blur_radius_factor = remember(|| 0.25);

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .content_padding(Dp(24.0))
        .item_spacing(Dp(16.0))
        .item(move || {
            text(I18N.text("ui-glass-components"))
                .style(theme.with(|t| t.typography.headline_large))
                .color(theme.with(|t| t.color_scheme.on_surface));
        })
        .item(move || {
            text(I18N.text("ui-drag-the-glass-lens"))
                .style(theme.with(|t| t.typography.body_large))
                .color(theme.with(|t| t.color_scheme.on_surface_variant))
                .modifier(Modifier::new().size_in(None, Some(CONTENT_MAX_WIDTH), None, None));
        })
        .item(move || {
            let shape = theme.with(|t| t.shapes.extra_large);
            surface()
                .style(theme.with(|t| t.color_scheme.surface_container_low).into())
                .shape(shape)
                .modifier(
                    Modifier::new()
                        .size_in(None, Some(DEMO_WIDTH), None, None)
                        .fill_max_width(),
                )
                .child(move || {
                    boxed()
                        .modifier(Modifier::new().fill_max_width().clip_shape(shape))
                        .children(move || {
                            glass_backdrop();
                            fluid_glass()
                                .shape(Shape::CAPSULE)
                                .blur_radius(MAX_BLUR_RADIUS * blur_radius_factor.get())
                                .accessibility_label(I18N.text("ui-glass-lens"))
                                .modifier(
                                    Modifier::new()
                                        .size(LENS_SIZE, LENS_SIZE)
                                        .offset(
                                            Dp(LENS_ORIGIN.0 + offset.get().0.to_dp().0),
                                            Dp(LENS_ORIGIN.0 + offset.get().1.to_dp().0),
                                        )
                                        .draggable(move |delta: DragDelta| {
                                            offset.set((
                                                offset.get().0 + delta.x,
                                                offset.get().1 + delta.y,
                                            ));
                                        }),
                                );
                        });
                });
        })
        .item(move || {
            column()
                .modifier(
                    Modifier::new()
                        .size_in(None, Some(DEMO_WIDTH), None, None)
                        .fill_max_width(),
                )
                .children(move || {
                    text(I18N.text("ui-blur-radius"))
                        .style(theme.with(|t| t.typography.title_small))
                        .color(theme.with(|t| t.color_scheme.on_surface));
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    slider()
                        .modifier(Modifier::new().fill_max_width())
                        .value(blur_radius_factor.get())
                        .accessibility_label(I18N.text("ui-blur-radius"))
                        .on_change(move |value| blur_radius_factor.set(value));
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    button()
                        .tonal()
                        .on_click(move || offset.set((Px::ZERO, Px::ZERO)))
                        .leading_icon(|| {
                            icon().painter(filled::RESTART_ALT_SVG).size(Dp(18.0));
                        })
                        .child(|| {
                            text(I18N.text("ui-reset-lens-position"));
                        });
                });
        });
}

#[tessera]
fn glass_backdrop() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let palette: [Color; 6] = [
        scheme.primary,
        scheme.tertiary,
        scheme.secondary,
        scheme.primary_container,
        scheme.tertiary_container,
        scheme.secondary_container,
    ];
    flow_row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_all(BACKDROP_PADDING),
        )
        .item_spacing(TILE_SPACING)
        .line_spacing(TILE_SPACING)
        .children(move || {
            for index in 0..TILE_COLUMNS * TILE_ROWS {
                let line = index / TILE_COLUMNS;
                let color = palette[(index % TILE_COLUMNS + line * 2) % palette.len()];
                spacer().modifier(
                    Modifier::new()
                        .size(TILE_SIZE, TILE_SIZE)
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(16.0))),
                );
            }
        });
}

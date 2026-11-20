//! Window toolbars with a title, navigation, and glass toolbar buttons.
//!
//! ## Usage
//!
//! Title a window and offer navigation and window-wide actions above its content.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _};
use tessera_ui::{
    Callback, Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier,
    RenderSlot,
    accesskit::Role,
    layout::{Stack, layout},
    modifier::ModifierCapabilityExt as _,
    renderer::AppTitleBar,
    tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    res::I18N,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};

use super::{
    divider::divider,
    glass::{GlassAppearance, GlassMaterial, glass},
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "toolbar_tests.rs"]
mod tests;

const HEIGHT: Dp = Dp(52.0);
/// Width reserved for the window controls at the leading edge.
const CONTROLS_WIDTH: f64 = 92.0;
const ITEM_SIZE: Dp = Dp(36.0);
const ITEM_SPACING: Dp = Dp(8.0);

/// A symbol drawn inside a [`toolbar_button`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarSymbol {
    /// A leading chevron for returning to the previous page.
    Back,
    /// A window with a leading sidebar, for showing or hiding the sidebar.
    Sidebar,
}

fn window_background(appearance: Appearance) -> Color {
    match appearance {
        Appearance::Light => Color::WHITE,
        Appearance::Dark => srgb(30, 30, 30),
    }
}

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

/// # toolbar
///
/// Shows a window's title bar with its window controls, navigation, title, and actions.
///
/// ## Usage
///
/// Place at the top of a window to title it and offer back or sidebar navigation and actions.
///
/// ## Parameters
///
/// - `title` — window title shown after the navigation items.
/// - `navigation` — leading navigational items, usually [`toolbar_button`]s.
/// - `actions` — trailing items, usually [`toolbar_button`]s.
/// - `window_chrome` — reserves room for the system window controls, or draws them when the
///   window has no system title bar; defaults to false.
/// - `modifier` — layout and semantics of the toolbar.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::toolbar::{ToolbarSymbol, toolbar, toolbar_button};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// toolbar()
///     .window_chrome(true)
///     .title("Documents")
///     .navigation(|| {
///         toolbar_button()
///             .symbol(ToolbarSymbol::Back)
///             .accessibility_label("Back")
///             .on_click(|| { /* return to the previous page */ });
///     })
///     .actions(|| {
///         toolbar_button().label("Share").on_click(|| { /* share the document */ });
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn toolbar(
    #[prop(into)] title: Option<String>,
    navigation: Option<RenderSlot>,
    actions: Option<RenderSlot>,
    window_chrome: Option<bool>,
    modifier: Option<Modifier>,
) {
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let title_bar = if window_chrome.unwrap_or(false) {
        AppTitleBar::current()
    } else {
        AppTitleBar::None
    };
    let has_navigation = navigation.is_some();
    let leading = match title_bar {
        AppTitleBar::None => Dp(4.0),
        _ => Dp(CONTROLS_WIDTH),
    };
    column()
        .modifier(
            modifier
                .unwrap_or_default()
                .fill_max_width()
                .background(window_background(theme.appearance)),
        )
        .children(move || {
            let title = title.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width().height(HEIGHT))
                .children(move || {
                    let title = title.clone();
                    boxed()
                        .alignment(Alignment::CenterStart)
                        .modifier(Modifier::new().width(leading).fill_max_height())
                        .children(move || {
                            if title_bar == AppTitleBar::Custom {
                                window_controls();
                            }
                        });
                    if let Some(navigation) = navigation {
                        spacer().modifier(Modifier::new().width(Dp(4.0)));
                        flow_row()
                            .item_spacing(ITEM_SPACING)
                            .children_shared(navigation);
                    }
                    boxed()
                        .alignment(Alignment::CenterStart)
                        .modifier(
                            Modifier::new()
                                .weight(1.0)
                                .fill_max_height()
                                .padding_symmetric(
                                    Dp(if has_navigation { 12.0 } else { 16.0 }),
                                    Dp::ZERO,
                                )
                                .window_drag_region(),
                        )
                        .children(move || {
                            if let Some(title) = title.clone() {
                                text(title)
                                    .single_line(true)
                                    .style(TextStyle {
                                        font_size: Dp(15.0),
                                        line_height: Dp(19.0),
                                        font_weight: 600,
                                        ..Default::default()
                                    })
                                    .color(theme.label_color());
                            }
                        });
                    if let Some(actions) = actions {
                        flow_row()
                            .item_spacing(ITEM_SPACING)
                            .children_shared(actions);
                    }
                    spacer().modifier(Modifier::new().width(ITEM_SPACING));
                });
            divider();
        });
}

#[derive(Clone, Copy, PartialEq)]
enum WindowControl {
    Close,
    Minimize,
    Zoom,
}

#[tessera]
fn window_controls() {
    row()
        .modifier(Modifier::new().padding_symmetric(Dp(19.0), Dp::ZERO))
        .children(|| {
            window_control(WindowControl::Close);
            spacer().modifier(Modifier::new().width(Dp(9.0)));
            window_control(WindowControl::Minimize);
            spacer().modifier(Modifier::new().width(Dp(9.0)));
            window_control(WindowControl::Zoom);
        });
}

#[tessera]
fn window_control(control: WindowControl) {
    let (color, label, action): (Color, &str, fn()) = match control {
        WindowControl::Close => (
            srgb(0xFF, 0x5F, 0x57),
            "hig-close-window",
            tessera_platform::window::close,
        ),
        WindowControl::Minimize => (
            srgb(0xFE, 0xBC, 0x2E),
            "hig-minimize-window",
            tessera_platform::window::minimize,
        ),
        WindowControl::Zoom => (
            srgb(0x28, 0xC8, 0x40),
            "hig-zoom-window",
            tessera_platform::window::toggle_maximize,
        ),
    };
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(14.0), Dp(14.0))
            .background_with_shape(color, Shape::Ellipse)
            .clickable(ClickableArgs {
                on_click: Callback::new(action),
                role: Some(Role::Button),
                label: Some(I18N.text(label)),
                ..Default::default()
            }),
    );
}

/// # toolbar_button
///
/// Shows a glass toolbar button with a symbol or a text label.
///
/// ## Usage
///
/// Offer navigation and window-wide actions inside a [`toolbar`].
///
/// ## Parameters
///
/// - `symbol` — symbol shown in a circular button; takes precedence over `label`.
/// - `label` — text shown in a capsule button and its default accessible name.
/// - `on_click` — action invoked when the button is activated.
/// - `modifier` — layout and semantics of the button.
/// - `accessibility_label` — spoken name, required for symbol buttons.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::toolbar::{ToolbarSymbol, toolbar_button};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// toolbar_button()
///     .symbol(ToolbarSymbol::Sidebar)
///     .accessibility_label("Show sidebar")
///     .on_click(|| { /* toggle the sidebar */ });
/// toolbar_button().label("Share").on_click(|| { /* share the document */ });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn toolbar_button(
    symbol: Option<ToolbarSymbol>,
    #[prop(into)] label: Option<String>,
    on_click: Option<Callback>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let appearance = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance)
        .unwrap_or_default();
    let color = HigTheme { appearance }.label_color();
    let shown = if symbol.is_some() {
        None
    } else {
        label.clone()
    };
    let mut size = modifier.unwrap_or_default().height(ITEM_SIZE);
    if symbol.is_some() {
        size = size.width(ITEM_SIZE);
    }
    let button = glass();
    if let Some(on_click) = on_click {
        button.on_click_shared(on_click);
    }
    button
        .material(GlassMaterial::Regular)
        .appearance(match appearance {
            Appearance::Light => GlassAppearance::Light,
            Appearance::Dark => GlassAppearance::Dark,
        })
        .modifier(size)
        .accessibility_role(Role::Button)
        .accessibility_label_optional(accessibility_label.or(label))
        .child(move || {
            let shown = shown.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(if symbol.is_some() {
                    Modifier::new().size(ITEM_SIZE, ITEM_SIZE)
                } else {
                    Modifier::new()
                        .height(ITEM_SIZE)
                        .padding_symmetric(Dp(12.0), Dp::ZERO)
                })
                .children(move || {
                    if let Some(symbol) = symbol {
                        symbol_glyph(symbol, color);
                    } else if let Some(label) = shown.clone() {
                        text(label).single_line(true).color(color);
                    }
                });
        });
}

#[tessera]
fn symbol_glyph(symbol: ToolbarSymbol, color: Color) {
    match symbol {
        ToolbarSymbol::Back => {
            layout(Stack).modifier(
                Modifier::new()
                    .size(Dp(18.0), Dp(18.0))
                    .push_draw(BackChevron { color }),
            );
        }
        ToolbarSymbol::Sidebar => {
            let frame = Shape::rounded_rectangle(Dp(3.5));
            boxed()
                .modifier(Modifier::new().size(Dp(19.0), Dp(15.0)).border_with_shape(
                    Dp(1.4),
                    color,
                    frame,
                ))
                .children(move || {
                    spacer().modifier(
                        Modifier::new()
                            .offset(Dp(6.5), Dp::ZERO)
                            .size(Dp(1.4), Dp(15.0))
                            .background(color),
                    );
                    for line in 0..3 {
                        spacer().modifier(
                            Modifier::new()
                                .offset(Dp(2.6), Dp(3.4 + 2.4 * f64::from(line)))
                                .size(Dp(2.6), Dp(1.2))
                                .background(color),
                        );
                    }
                });
        }
    }
}

#[derive(Clone, PartialEq)]
struct BackChevron {
    color: Color,
}

impl DrawModifierNode for BackChevron {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: Color::TRANSPARENT,
                radii: [Dp::ZERO; 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                edge_smoothness: 1.0,
                line: None,
                stroke: Some(PerceptualStroke {
                    raster_mask: true,
                    color: self.color,
                    width: Dp(1.8),
                    points: [
                        [11.0 / 18.0, 3.0 / 18.0],
                        [5.0 / 18.0, 9.0 / 18.0],
                        [11.0 / 18.0, 15.0 / 18.0],
                    ],
                }),
            });
        content.draw(context.render_input);
    }
}

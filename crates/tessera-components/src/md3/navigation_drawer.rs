//! Navigation drawer surfaces and selectable destinations.
//!
//! ## Usage
//!
//! Organize primary destinations and account sections in spacious layouts.

use tessera_foundation::modifier::{InteractionState, ModifierExt as _, Padding, SelectableArgs};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, accesskit::Role,
    modifier::FocusModifierExt as _, provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment,
    column::column,
    md3::{
        ripple_state::{RippleSpec, RippleState},
        surface::surface,
        text::text,
        theme::MaterialTheme,
    },
    modifier::ModifierExt as _,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
};

/// Presentation of a navigation drawer surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationDrawerStyle {
    /// A surface displayed alongside page content.
    #[default]
    Standard,
    /// A raised surface intended for a modal overlay.
    Modal,
}

/// # navigation_drawer
///
/// Arrange primary destinations on a 360dp navigation surface.
///
/// ## Usage
///
/// Provide persistent navigation or drawer content inside a modal side sheet.
///
/// ## Parameters
///
/// - `modifier` — drawer placement and optional height constraints.
/// - `style` — standard surface or raised modal surface treatment.
/// - `content` — destinations, headings, and separators in display order.
/// - `accessibility_label` — name announced for this navigation region.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::navigation_drawer::{navigation_drawer, navigation_drawer_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn navigation() {
///     let selected = remember(|| 0usize);
///     navigation_drawer().content(move || {
///         navigation_drawer_item()
///             .label("Inbox")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///     });
/// }
/// ```
#[tessera]
pub fn navigation_drawer(
    modifier: Option<Modifier>,
    style: Option<NavigationDrawerStyle>,
    content: Option<RenderSlot>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let modal = style.unwrap_or_default() == NavigationDrawerStyle::Modal;
    let round = RoundedCorner::manual(Dp(16.0), 2.0);
    let square = RoundedCorner::manual(Dp(0.0), 2.0);
    surface()
        .modifier(
            modifier
                .unwrap_or_default()
                .width(Dp(360.0))
                .focus_group(None),
        )
        .style(
            (if modal {
                scheme.surface_container_low
            } else {
                scheme.surface
            })
            .into(),
        )
        .shape(Shape::RoundedRectangle {
            top_left: square,
            top_right: round,
            bottom_right: round,
            bottom_left: square,
        })
        .elevation(if modal { Dp(1.0) } else { Dp(0.0) })
        .block_input(false)
        .accessibility_role(Role::Navigation)
        .accessibility_label_optional(accessibility_label)
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(12.0), Dp(12.0)),
                )
                .children_shared(content.clone().unwrap_or_else(RenderSlot::empty));
        });
}

/// # navigation_drawer_item
///
/// Show a selectable destination with an optional icon and trailing badge.
///
/// ## Usage
///
/// Switch between primary destinations while announcing the current selection.
///
/// ## Parameters
///
/// - `modifier` — additional layout and semantics.
/// - `label` — destination name.
/// - `selected` — whether this is the current destination.
/// - `enabled` — whether pointer and keyboard activation are available.
/// - `on_click` — destination activation callback.
/// - `icon` — optional 24dp leading icon.
/// - `badge` — optional trailing count or status.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::navigation_drawer::navigation_drawer_item;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn destination() {
///     let selected = remember(|| false);
///     navigation_drawer_item()
///         .label("Saved")
///         .selected(selected.get())
///         .on_click(move || selected.set(true));
/// }
/// ```
#[tessera]
pub fn navigation_drawer_item(
    modifier: Option<Modifier>,
    #[prop(into)] label: Option<String>,
    selected: Option<bool>,
    enabled: Option<bool>,
    on_click: Option<Callback>,
    icon: Option<RenderSlot>,
    badge: Option<RenderSlot>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let selected = selected.unwrap_or(false);
    let enabled = enabled.unwrap_or(true);
    let label = label.unwrap_or_default();
    let foreground = if selected {
        scheme.on_secondary_container
    } else {
        scheme.on_surface_variant
    };
    let foreground = if enabled {
        foreground
    } else {
        scheme.on_surface.with_alpha(0.38)
    };
    let interaction = remember(InteractionState::new);
    let ripple = remember(RippleState::new);
    let focus = remember(FocusRequester::new).get();
    let modifier = modifier
        .unwrap_or_default()
        .fill_max_width()
        .height(Dp(56.0))
        .selectable(SelectableArgs {
            selected,
            enabled,
            on_click: on_click.unwrap_or_default(),
            role: Some(Role::Tab),
            label: Some(label.clone()),
            interaction_state: Some(interaction),
            focus_requester: Some(focus),
            on_press: Some(
                (move |ctx: tessera_foundation::modifier::PointerEventContext| {
                    ripple.with_mut(|state| {
                        state.start_animation_with_spec(
                            ctx.normalized_pos,
                            ctx.size,
                            RippleSpec {
                                bounded: true,
                                radius: None,
                            },
                            ctx.timestamp,
                        )
                    });
                })
                .into(),
            ),
            on_release: Some(
                (move |_ctx| {
                    ripple.with_mut(|state| state.release());
                })
                .into(),
            ),
            ..Default::default()
        });
    surface()
        .modifier(modifier)
        .style(
            (if selected {
                scheme.secondary_container
            } else {
                tessera_ui::Color::TRANSPARENT
            })
            .into(),
        )
        .content_color(foreground)
        .shape(Shape::CAPSULE)
        .enabled(enabled)
        .block_input(false)
        .interaction_state(interaction)
        .focus_requester(focus)
        .show_focus_indicator(true)
        .ripple_state(ripple)
        .ripple_color(foreground)
        .child(move || {
            let label = label.clone();
            row()
                .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                    Dp(16.0),
                    Dp::ZERO,
                    Dp(24.0),
                    Dp::ZERO,
                )))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    if let Some(icon) = &icon {
                        icon.render();
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                    }
                    text(label.clone())
                        .style(theme.typography.label_large)
                        .color(foreground);
                    spacer().modifier(Modifier::new().weight(1.0));
                    if let Some(badge) = &badge {
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        provide_context(theme.typography.label_large, || {
                            badge.render();
                        });
                    }
                });
        });
}

#[cfg(test)]
mod tests {

    use tessera_foundation::modifier::SemanticsArgs;
    use tessera_ui::{PxPosition, State, testing::UiTest};

    use crate::md3::theme::material_theme;

    use super::*;

    #[tessera]
    fn fixture(output: tessera_ui::testing::TestProbe<Option<State<usize>>>) {
        let selected = remember(|| 0usize);
        output.set(Some(selected));
        material_theme().child(move || {
            column().children(move || {
                navigation_drawer()
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        test_tag: Some("drawer".into()),
                        ..Default::default()
                    }))
                    .content(move || {
                        navigation_drawer_item()
                            .label("Inbox")
                            .selected(selected.get() == 0)
                            .on_click(move || selected.set(0));
                        navigation_drawer_item()
                            .label("Saved")
                            .badge(|| {
                                text("24").modifier(Modifier::new().semantics(SemanticsArgs {
                                    test_tag: Some("saved-badge".into()),
                                    ..Default::default()
                                }));
                            })
                            .selected(selected.get() == 1)
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                test_tag: Some("saved".into()),
                                ..Default::default()
                            }))
                            .on_click(move || selected.set(1));
                        navigation_drawer_item()
                            .label("Disabled")
                            .enabled(false)
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                test_tag: Some("disabled".into()),
                                ..Default::default()
                            }))
                            .on_click(move || selected.set(2));
                    });
            });
        });
    }

    #[test]
    fn destinations_keep_token_bounds_select_and_ignore_disabled_input() {
        let output = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| fixture(output.clone()))
            .viewport(400, 400);
        let snapshot = session.frame();
        snapshot.node("saved").size(336, 56);
        let saved = snapshot.node("saved").bounds();
        let badge = snapshot.node("saved-badge").bounds();
        let drawer = snapshot.node("drawer").bounds();
        assert_eq!(
            (saved.x + saved.width - badge.x - badge.width).to_f32(),
            24.0
        );
        assert_eq!(
            (drawer.x + drawer.width - badge.x - badge.width).to_f32(),
            36.0
        );
        let disabled = snapshot.node("disabled").bounds();
        for bounds in [saved, disabled] {
            session.click_at(PxPosition::from_f32_xy(
                bounds.x.to_f32() + 100.0,
                bounds.y.to_f32() + 28.0,
            ));
            session.frames(1);
            assert_eq!(output.get().unwrap().get(), 1);
        }
    }

    #[tessera]
    fn shape_fixture() {
        material_theme()
            .theme(|| {
                let mut theme = MaterialTheme::default();
                theme.color_scheme.surface = tessera_ui::Color::BLACK;
                theme
            })
            .child(|| {
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_size()
                            .background(tessera_ui::Color::WHITE),
                    )
                    .children(|| {
                        navigation_drawer().modifier(Modifier::new().height(Dp(100.0)));
                    });
            });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter to verify navigation drawer corner pixels"]
    fn standard_drawer_rounds_only_the_ending_corners() {
        let mut renderer = tessera_ui::testing::UiTest::new()
            .viewport(400, 120)
            .rendered(crate::TesseraComponents)
            .content(shape_fixture);
        renderer.frames(3);
        let pixels = renderer.read_pixels();
        let pixel = |x: usize, y: usize| -> [u8; 4] {
            let offset = (y * 400 + x) * 4;
            pixels[offset..offset + 4].try_into().unwrap()
        };
        for (x, y) in [(0, 0), (0, 99), (343, 0), (343, 99), (359, 16), (359, 83)] {
            assert_eq!(pixel(x, y), [0, 0, 0, 255]);
        }
        for (x, y) in [(359, 0), (359, 99), (356, 2), (356, 97)] {
            assert_eq!(pixel(x, y), [255; 4]);
        }
    }
}

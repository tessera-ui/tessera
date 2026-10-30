//! Navigation drawer surfaces and selectable destinations.
//!
//! ## Usage
//!
//! Organize primary destinations and account sections in spacious layouts.

use tessera_ui::{
    Callback, Dp, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _,
    provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment,
    column::column,
    modifier::{InteractionState, ModifierExt, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, TextStyle},
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
/// use tessera_components::navigation_drawer::{navigation_drawer, navigation_drawer_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn navigation() {
///     let selected = remember(|| 0usize);
///     assert_eq!(selected.get(), 0);
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
        .modifier(modifier.unwrap_or_default().width(Dp(360.0)).focus_group())
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
/// use tessera_components::navigation_drawer::navigation_drawer_item;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn destination() {
///     let selected = remember(|| false);
///     assert!(!selected.get());
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
    let modifier = modifier
        .unwrap_or_default()
        .fill_max_width()
        .height(Dp(56.0))
        .selectable_with(SelectableArgs {
            selected,
            enabled,
            on_click: on_click.unwrap_or_default(),
            role: Some(Role::Tab),
            label: Some(label.clone()),
            interaction_state: Some(interaction),
            on_press: Some(
                (move |ctx: crate::modifier::PointerEventContext| {
                    ripple.with_mut(|state| {
                        state.start_animation_with_spec(
                            ctx.normalized_pos,
                            ctx.size,
                            RippleSpec {
                                bounded: true,
                                radius: None,
                            },
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
        .ripple_state(ripple)
        .ripple_color(foreground)
        .child(move || {
            let label = label.clone();
            row()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .padding_symmetric(Dp(16.0), Dp(0.0)),
                )
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    if let Some(icon) = &icon {
                        icon.render();
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                    }
                    text()
                        .content(label.clone())
                        .style(theme.typography.label_large)
                        .color(foreground);
                    spacer().modifier(Modifier::new().weight(1.0));
                    if let Some(badge) = &badge {
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        provide_context(
                            || theme.typography.label_large,
                            || {
                                let style = use_context::<TextStyle>()
                                    .expect("drawer badge style must be provided");
                                if style.get() != theme.typography.label_large {
                                    style.set(theme.typography.label_large);
                                }
                                badge.render();
                            },
                        );
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition, State,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use crate::{modifier::SemanticsArgs, theme::material_theme};

    use super::*;

    thread_local! { static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) }; }

    #[tessera]
    fn fixture() {
        let selected = remember(|| 0usize);
        SELECTED.set(Some(selected));
        material_theme().child(move || {
            column().children(move || {
                navigation_drawer().content(move || {
                    navigation_drawer_item()
                        .label("Inbox")
                        .selected(selected.get() == 0)
                        .on_click(move || selected.set(0));
                    navigation_drawer_item()
                        .label("Saved")
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
        let mut session = start_layout_test_session(
            layout_test(|| {
                fixture();
            })
            .viewport_px(400, 400),
        );
        let snapshot = pump_layout_test_session(&mut session, 0);
        snapshot.node("saved").size(336, 56);
        let saved = snapshot.node("saved").bounds();
        let disabled = snapshot.node("disabled").bounds();
        for (index, bounds) in [saved, disabled].into_iter().enumerate() {
            let cursor = Some(PxPosition::from_f32_xy(
                bounds.x.to_f32() + 100.0,
                bounds.y.to_f32() + 28.0,
            ));
            for (event, content) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                pump_layout_test_session_with_input(
                    &mut session,
                    (index * 2 + event + 1) as u64 * 16_666_667,
                    vec![PointerChange::new(
                        tessera_ui::time::Instant::now(),
                        1,
                        content,
                        GestureState::TapCandidate,
                    )],
                    cursor,
                );
            }
            pump_layout_test_session(&mut session, (index as u64 + 1) * 100_000_000);
            assert_eq!(SELECTED.get().unwrap().get(), 1);
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
        let mut renderer = tessera_ui::HeadlessRenderer::new(
            Box::new(shape_fixture),
            vec![Box::new(crate::TesseraComponents)],
            tessera_ui::HeadlessConfig {
                width: 400,
                height: 120,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let pixels = renderer.read_pixels().unwrap();
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

//! Floating toolbars for frequently used page actions.
//!
//! ## Usage
//!
//! Keep editing, formatting, and contextual actions close to the current task.

use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError, Modifier,
    Px, PxPosition, RenderSlot,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    tessera, use_context,
};

use crate::{modifier::ModifierExt, shape_def::Shape, surface::surface, theme::MaterialTheme};

/// Direction in which toolbar actions are arranged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarOrientation {
    /// Arrange actions from left to right.
    #[default]
    Horizontal,
    /// Arrange actions from top to bottom.
    Vertical,
}

/// Color treatment for a floating toolbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarColorStyle {
    /// Use the surface container color.
    #[default]
    Standard,
    /// Emphasize the toolbar with the primary container color.
    Vibrant,
}

#[tessera]
fn toolbar_actions(visible: bool, actions: Option<RenderSlot>) {
    if visible && let Some(actions) = actions {
        actions.render();
    }
}

#[derive(Clone, Copy, PartialEq)]
struct FloatingToolbarLayout {
    orientation: ToolbarOrientation,
}

impl tessera_ui::LayoutModifierNode for FloatingToolbarLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let horizontal = self.orientation == ToolbarOrientation::Horizontal;
        let parent = *input.parent_constraint().as_ref();
        let (main, cross) = if horizontal {
            (parent.width, parent.height)
        } else {
            (parent.height, parent.width)
        };
        let children = input.children();
        let gap = Dp(4.0).to_px();
        let gaps = gap * children.len().saturating_sub(1) as i32;
        let mut total = Px::ZERO;
        let mut max_cross = Px::ZERO;
        let mut measured = Vec::with_capacity(children.len());
        for child in children {
            let main_constraint = AxisConstraint::new(
                Px::ZERO,
                main.resolve_max()
                    .map(|max| (max - gaps - total).max(Px::ZERO)),
            );
            let cross_constraint = AxisConstraint::new(Px::ZERO, cross.resolve_max());
            let constraint = if horizontal {
                Constraint::new(main_constraint, cross_constraint)
            } else {
                Constraint::new(cross_constraint, main_constraint)
            };
            let size = child.measure(input, &constraint)?;
            let (child_main, child_cross) = if horizontal {
                (size.width, size.height)
            } else {
                (size.height, size.width)
            };
            total += child_main;
            max_cross = max_cross.max(child_cross);
            measured.push((child, child_main, child_cross));
        }
        let main_size = main.clamp(total + gaps);
        let cross_size = cross.clamp(max_cross);
        let size = if horizontal {
            ComputedData {
                width: main_size,
                height: cross_size,
            }
        } else {
            ComputedData {
                width: cross_size,
                height: main_size,
            }
        };
        let mut result = LayoutResult::new(size);
        let mut cursor = Px::ZERO;
        for (child, child_main, child_cross) in measured {
            let cross_offset = (cross_size - child_cross) / 2;
            let position = if horizontal {
                PxPosition::new(cursor, cross_offset)
            } else {
                PxPosition::new(cross_offset, cursor)
            };
            result.place_child(child, position);
            cursor += child_main + gap;
        }
        Ok(result)
    }
}

/// # floating_toolbar
///
/// Group frequent actions in a rounded horizontal or vertical floating surface.
///
/// ## Usage
///
/// Show contextual editing actions while preserving room for page content.
///
/// ## Parameters
///
/// - `modifier` — placement and size constraints for the toolbar.
/// - `orientation` — horizontal by default, or vertical.
/// - `color_style` — standard surface or vibrant primary container colors.
/// - `container_color`, `content_color` — optional color overrides.
/// - `elevation` — shadow elevation, 6dp by default; override with zero when
///   the background provides enough visual separation.
/// - `expanded` — whether optional leading and trailing actions are visible.
/// - `leading`, `trailing` — additional actions shown while expanded.
/// - `content` — persistent actions, preferably with 48dp touch targets.
/// - `accessibility_label` — name announced for the toolbar group.
///
/// ## Examples
///
/// ```
/// use tessera_components::{text::text, toolbar::floating_toolbar};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn editor_actions() {
///     let expanded = remember(|| true);
///     assert!(expanded.get());
///     floating_toolbar().expanded(expanded.get()).content(|| {
///         text().content("Editing actions");
///     });
/// }
/// ```
#[tessera]
pub fn floating_toolbar(
    modifier: Option<Modifier>,
    orientation: Option<ToolbarOrientation>,
    color_style: Option<ToolbarColorStyle>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    elevation: Option<Dp>,
    expanded: Option<bool>,
    leading: Option<RenderSlot>,
    content: Option<RenderSlot>,
    trailing: Option<RenderSlot>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let vibrant = color_style.unwrap_or_default() == ToolbarColorStyle::Vibrant;
    let background = container_color.unwrap_or(if vibrant {
        scheme.primary_container
    } else {
        scheme.surface_container
    });
    let foreground = content_color.unwrap_or(if vibrant {
        scheme.on_primary_container
    } else {
        scheme.on_surface_variant
    });
    let orientation = orientation.unwrap_or_default();
    let expanded = expanded.unwrap_or(true);
    surface()
        .modifier(modifier.unwrap_or_default().focus_group())
        .style(background.into())
        .content_color(foreground)
        .shape(Shape::CAPSULE)
        .elevation(elevation.unwrap_or(Dp(6.0)))
        .block_input(false)
        .accessibility_role(Role::Toolbar)
        .accessibility_label_optional(accessibility_label)
        .child(move || {
            let actions = RenderSlot::new(move || {
                toolbar_actions(expanded).actions_optional(leading);
                toolbar_actions(true).actions_optional(content);
                toolbar_actions(expanded).actions_optional(trailing);
            });
            let modifier = match orientation {
                ToolbarOrientation::Horizontal => Modifier::new().height(Dp(64.0)),
                ToolbarOrientation::Vertical => Modifier::new().width(Dp(64.0)),
            };
            layout()
                .modifier(modifier.padding_all(Dp(8.0)))
                .policy(FloatingToolbarLayout { orientation })
                .child_shared(actions);
        });
}

#[derive(Clone, Copy, PartialEq)]
struct DockedToolbarLayout;

impl tessera_ui::LayoutModifierNode for DockedToolbarLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let padding = Dp(16.0).to_px();
        let height = input.parent_constraint().height().clamp(Dp(64.0).to_px());
        let children = input.children();
        let constraints = Constraint::new(
            AxisConstraint::new(
                Px::ZERO,
                input
                    .parent_constraint()
                    .width()
                    .resolve_max()
                    .map(|width| (width - padding * 2).max(Px::ZERO)),
            ),
            AxisConstraint::new(Px::ZERO, Some((height - Dp(16.0).to_px()).max(Px::ZERO))),
        );
        let mut measured = Vec::with_capacity(children.len());
        let mut total = Px::ZERO;
        for child in children {
            let size = child.measure(input, &constraints)?;
            total += size.width;
            measured.push((child, size));
        }
        let gaps = measured.len().saturating_sub(1) as i32;
        let min_gap = Dp(4.0).to_px();
        let max_gap = Dp(32.0).to_px();
        let width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(total + padding * 2 + min_gap * gaps);
        let width = input.parent_constraint().width().clamp(width);
        let gap = if gaps == 0 {
            Px::ZERO
        } else {
            ((width - padding * 2 - total) / gaps).clamp(min_gap, max_gap)
        };
        let mut cursor = ((width - total - gap * gaps) / 2).max(padding);
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (child, size) in measured {
            result.place_child(child, PxPosition::new(cursor, (height - size.height) / 2));
            cursor += size.width + gap;
        }
        Ok(result)
    }
}

/// # docked_toolbar
///
/// Present frequent actions across a full-width surface at the edge of a page.
///
/// ## Usage
///
/// Keep page-level editing actions available at the bottom of a workspace.
///
/// ## Parameters
///
/// - `modifier` — placement and optional size constraints.
/// - `color_style` — standard surface or vibrant primary container colors.
/// - `container_color`, `content_color` — optional color overrides.
/// - `content` — action controls with 48dp touch targets, spaced between 4dp
///   and 32dp.
/// - `accessibility_label` — name announced for the toolbar group.
///
/// ## Examples
///
/// ```
/// use tessera_components::{text::text, toolbar::docked_toolbar};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn editing_actions() {
///     let saved = remember(|| false);
///     assert!(!saved.get());
///     docked_toolbar().content(move || {
///         text().content(if saved.get() { "Saved" } else { "Editing" });
///     });
/// }
/// ```
#[tessera]
pub fn docked_toolbar(
    modifier: Option<Modifier>,
    color_style: Option<ToolbarColorStyle>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    content: Option<RenderSlot>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let vibrant = color_style.unwrap_or_default() == ToolbarColorStyle::Vibrant;
    surface()
        .modifier(modifier.unwrap_or_default().fill_max_width().focus_group())
        .style(
            container_color
                .unwrap_or(if vibrant {
                    scheme.primary_container
                } else {
                    scheme.surface_container
                })
                .into(),
        )
        .content_color(content_color.unwrap_or(if vibrant {
            scheme.on_primary_container
        } else {
            scheme.on_surface_variant
        }))
        .shape(Shape::RECTANGLE)
        .elevation(Dp(0.0))
        .block_input(false)
        .accessibility_role(Role::Toolbar)
        .accessibility_label_optional(accessibility_label)
        .child(move || {
            layout()
                .policy(DockedToolbarLayout)
                .child_shared(content.unwrap_or_else(RenderSlot::empty));
        });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        Modifier, State, remember, tessera,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use crate::{spacer::spacer, theme::material_theme};

    use super::*;

    thread_local! {
        static EXPANDED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static CONTENT_SIZE: Cell<Option<State<Dp>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture(orientation: ToolbarOrientation, expanded: bool) {
        let expanded = remember(|| expanded);
        EXPANDED.set(Some(expanded));
        material_theme().child(move || {
            crate::column::column().children(move || {
                crate::boxed::boxed()
                    .modifier(Modifier::new().semantics(crate::modifier::SemanticsArgs {
                        test_tag: Some("toolbar".into()),
                        ..Default::default()
                    }))
                    .children(move || {
                        floating_toolbar()
                            .orientation(orientation)
                            .expanded(expanded.get())
                            .leading(|| {
                                spacer().modifier(
                                    Modifier::new().size(Dp(48.0), Dp(48.0)).semantics(
                                        crate::modifier::SemanticsArgs {
                                            test_tag: Some("leading-action".into()),
                                            ..Default::default()
                                        },
                                    ),
                                );
                            })
                            .content(|| {
                                let size = remember(|| Dp(48.0));
                                CONTENT_SIZE.set(Some(size));
                                spacer().modifier(
                                    Modifier::new().size(size.get(), size.get()).semantics(
                                        crate::modifier::SemanticsArgs {
                                            test_tag: Some("content-action".into()),
                                            ..Default::default()
                                        },
                                    ),
                                );
                            })
                            .trailing(|| {
                                spacer().modifier(
                                    Modifier::new().size(Dp(48.0), Dp(48.0)).semantics(
                                        crate::modifier::SemanticsArgs {
                                            test_tag: Some("trailing-action".into()),
                                            ..Default::default()
                                        },
                                    ),
                                );
                            });
                    });
            });
        });
    }

    #[test]
    fn toolbar_measures_both_orientations_and_collapsed_actions() {
        for (orientation, expanded, width, height) in [
            (ToolbarOrientation::Horizontal, true, 168, 64),
            (ToolbarOrientation::Vertical, true, 64, 168),
            (ToolbarOrientation::Horizontal, false, 64, 64),
            (ToolbarOrientation::Vertical, false, 64, 64),
        ] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    fixture(orientation, expanded);
                })
                .viewport_px(400, 400),
            );
            let snapshot = pump_layout_test_session(&mut session, 0);
            snapshot.node("toolbar").size(width, height);
            if expanded {
                for (tag, offset) in [
                    ("leading-action", 8),
                    ("content-action", 60),
                    ("trailing-action", 112),
                ] {
                    let (x, y) = match orientation {
                        ToolbarOrientation::Horizontal => (offset, 8),
                        ToolbarOrientation::Vertical => (8, offset),
                    };
                    snapshot.node(tag).size(48, 48).position(x, y);
                }
                let toolbar = snapshot.node("toolbar").bounds();
                let trailing = snapshot.node("trailing-action").bounds();
                let end_padding = match orientation {
                    ToolbarOrientation::Horizontal => {
                        toolbar.x + toolbar.width - trailing.x - trailing.width
                    }
                    ToolbarOrientation::Vertical => {
                        toolbar.y + toolbar.height - trailing.y - trailing.height
                    }
                };
                assert_eq!(end_padding, Dp(8.0).to_px());
            } else {
                snapshot.node("content-action").size(48, 48).position(8, 8);
            }
        }
    }

    #[test]
    fn floating_toolbar_collapse_and_reexpand_preserve_live_content_state() {
        for orientation in [ToolbarOrientation::Horizontal, ToolbarOrientation::Vertical] {
            let horizontal = orientation == ToolbarOrientation::Horizontal;
            let mut session = start_layout_test_session(
                layout_test(move || {
                    fixture(orientation, true);
                })
                .viewport_px(400, 400),
            );
            let initial = pump_layout_test_session(&mut session, 0);
            initial.node("toolbar").size(
                if horizontal { 168 } else { 64 },
                if horizontal { 64 } else { 168 },
            );
            EXPANDED.get().expect("fixture state").set(false);
            let collapsed = pump_layout_test_session(&mut session, 16_666_667);
            collapsed.node("toolbar").size(64, 64);
            collapsed.node("content-action").size(48, 48).position(8, 8);

            CONTENT_SIZE
                .get()
                .expect("persistent content state")
                .set(Dp(40.0));
            let updated = pump_layout_test_session(&mut session, 33_333_334);
            updated.node("toolbar").size(
                if horizontal { 56 } else { 64 },
                if horizontal { 64 } else { 56 },
            );
            updated.node("content-action").size(40, 40).position(
                if horizontal { 8 } else { 12 },
                if horizontal { 12 } else { 8 },
            );

            EXPANDED.get().expect("fixture state").set(true);
            let expanded = pump_layout_test_session(&mut session, 50_000_001);
            expanded.node("toolbar").size(
                if horizontal { 160 } else { 64 },
                if horizontal { 64 } else { 160 },
            );
            expanded.node("leading-action").size(48, 48).position(8, 8);
            expanded.node("content-action").size(40, 40).position(
                if horizontal { 60 } else { 12 },
                if horizontal { 12 } else { 60 },
            );
            expanded.node("trailing-action").size(48, 48).position(
                if horizontal { 104 } else { 8 },
                if horizontal { 8 } else { 104 },
            );
            assert_eq!(
                CONTENT_SIZE.get().expect("persistent content state").get(),
                Dp(40.0)
            );
        }
    }

    #[cfg(not(target_family = "wasm"))]
    thread_local! {
        static ELEVATION: Cell<Option<State<Option<Dp>>>> = const { Cell::new(None) };
    }

    #[cfg(not(target_family = "wasm"))]
    #[tessera]
    fn elevation_fixture() {
        let elevation = remember(|| None::<Dp>);
        ELEVATION.set(Some(elevation));
        material_theme().child(move || {
            surface()
                .modifier(Modifier::new().fill_max_size())
                .style(Color::WHITE.into())
                .shape(Shape::RECTANGLE)
                .child(move || {
                    crate::boxed::boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .alignment(crate::alignment::Alignment::Center)
                        .children(move || {
                            floating_toolbar()
                                .container_color(Color::WHITE)
                                .elevation_optional(elevation.get())
                                .content(|| {
                                    spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)));
                                });
                        });
                });
        });
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn floating_toolbar_default_shadow_matches_6dp_and_zero_override_removes_it() {
        crate::test_utils::run_isolated_test(
            "toolbar::tests::floating_toolbar_default_shadow_matches_6dp_and_zero_override_removes_it",
            check_default_shadow,
        );
    }

    #[cfg(not(target_family = "wasm"))]
    fn check_default_shadow() {
        use tessera_ui::headless::{HeadlessConfig, HeadlessRenderer};

        let mut renderer = HeadlessRenderer::new(
            Box::new(|| elevation_fixture()),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 160,
                height: 160,
                ..Default::default()
            },
        )
        .expect("native GPU renderer");
        renderer.render_frames(4);
        let default = renderer.read_pixels().expect("default elevation pixels");
        let elevation = ELEVATION.get().expect("fixture elevation state");
        elevation.set(Some(Dp(6.0)));
        renderer.render_frames(4);
        assert_eq!(
            default,
            renderer.read_pixels().expect("explicit 6dp pixels"),
            "the default must reach the same surface drawing as explicit 6dp"
        );

        elevation.set(Some(Dp::ZERO));
        renderer.render_frames(4);
        let flat = renderer.read_pixels().expect("zero elevation pixels");
        let shadow_pixels = |pixels: &[u8]| {
            let mut crop = Vec::new();
            for y in 112..128 {
                for x in 32..128 {
                    let offset = (y * 160 + x) * 4;
                    crop.extend_from_slice(&pixels[offset..offset + 4]);
                }
            }
            crop
        };
        let default_shadow = shadow_pixels(&default);
        let flat_shadow = shadow_pixels(&flat);
        assert!(
            flat_shadow.chunks_exact(4).all(|pixel| pixel == [255; 4]),
            "zero elevation must leave only the white background below the toolbar"
        );
        assert_ne!(
            default_shadow, flat_shadow,
            "the default must draw a visible shadow outside the 64dp container"
        );

        elevation.set(None);
        renderer.render_frames(4);
        assert_eq!(
            default,
            renderer.read_pixels().expect("restored default pixels")
        );
    }

    #[tessera]
    fn docked_fixture() {
        material_theme().child(|| {
            crate::column::column().children(|| {
                crate::boxed::boxed()
                    .modifier(Modifier::new().semantics(crate::modifier::SemanticsArgs {
                        test_tag: Some("docked".into()),
                        ..Default::default()
                    }))
                    .children(|| {
                        docked_toolbar().content(|| {
                            spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)).semantics(
                                crate::modifier::SemanticsArgs {
                                    test_tag: Some("first-action".into()),
                                    ..Default::default()
                                },
                            ));
                            spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)).semantics(
                                crate::modifier::SemanticsArgs {
                                    test_tag: Some("second-action".into()),
                                    ..Default::default()
                                },
                            ));
                        });
                    });
            });
        });
    }

    #[test]
    fn docked_toolbar_fills_the_available_width_at_64dp_height() {
        for (width, gap) in [(132, 4), (140, 12), (320, 32), (800, 32)] {
            let snapshot = layout_test(|| {
                docked_fixture();
            })
            .viewport_px(width, 400)
            .run();
            snapshot.node("docked").size(width as i32, 64);
            let first = snapshot.node("first-action").bounds();
            let second = snapshot.node("second-action").bounds();
            assert_eq!(second.x - first.x - first.width, Dp(gap as f64).to_px());
            assert_eq!(first.y, Dp(8.0).to_px());
        }
    }
}

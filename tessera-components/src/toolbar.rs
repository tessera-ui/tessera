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

use crate::{
    alignment::CrossAxisAlignment, column::column, modifier::ModifierExt, row::row,
    shape_def::Shape, surface::surface, theme::MaterialTheme,
};

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
        .elevation(Dp(0.0))
        .block_input(false)
        .accessibility_role(Role::Toolbar)
        .accessibility_label_optional(accessibility_label)
        .child(move || {
            let actions = RenderSlot::new(move || {
                toolbar_actions(expanded).actions_optional(leading);
                toolbar_actions(true).actions_optional(content);
                toolbar_actions(expanded).actions_optional(trailing);
            });
            match orientation {
                ToolbarOrientation::Horizontal => {
                    row()
                        .modifier(Modifier::new().height(Dp(64.0)).padding_all(Dp(8.0)))
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children_shared(actions);
                }
                ToolbarOrientation::Vertical => {
                    column()
                        .modifier(Modifier::new().width(Dp(64.0)).padding_all(Dp(8.0)))
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children_shared(actions);
                }
            }
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
    use tessera_ui::{
        Modifier, tessera,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use crate::{spacer::spacer, theme::material_theme};

    use super::*;

    #[tessera]
    fn fixture(orientation: ToolbarOrientation, expanded: bool) {
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
                            .expanded(expanded)
                            .leading(|| {
                                spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)));
                            })
                            .content(|| {
                                spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)));
                            })
                            .trailing(|| {
                                spacer().modifier(Modifier::new().size(Dp(48.0), Dp(48.0)));
                            });
                    });
            });
        });
    }

    #[test]
    fn toolbar_measures_both_orientations_and_collapsed_actions() {
        for (orientation, expanded, width, height) in [
            (ToolbarOrientation::Horizontal, true, 160, 64),
            (ToolbarOrientation::Vertical, true, 64, 160),
            (ToolbarOrientation::Horizontal, false, 64, 64),
        ] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    fixture(orientation, expanded);
                })
                .viewport_px(400, 400),
            );
            let snapshot = pump_layout_test_session(&mut session, 0);
            snapshot.node("toolbar").size(width, height);
        }
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

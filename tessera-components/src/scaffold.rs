//! Scaffold layout for persistent bars and app content.
//!
//! ## Usage
//!
//! Layer top/bottom bars, floating buttons, and snackbars above app content.
use tessera_ui::{
    ComputedData, Constraint, Dp, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::{
    alignment::Alignment,
    modifier::{ModifierExt as _, Padding},
};

fn center_axis(container: Px, child: Px) -> Px {
    (container - child) / 2
}

fn compute_overlay_offset(
    alignment: Alignment,
    container_w: Px,
    container_h: Px,
    child_w: Px,
    child_h: Px,
) -> (Px, Px) {
    match alignment {
        Alignment::TopStart => (Px::ZERO, Px::ZERO),
        Alignment::TopCenter => (center_axis(container_w, child_w), Px::ZERO),
        Alignment::TopEnd => (container_w - child_w, Px::ZERO),
        Alignment::CenterStart => (Px::ZERO, center_axis(container_h, child_h)),
        Alignment::Center => (
            center_axis(container_w, child_w),
            center_axis(container_h, child_h),
        ),
        Alignment::CenterEnd => (container_w - child_w, center_axis(container_h, child_h)),
        Alignment::BottomStart => (Px::ZERO, container_h - child_h),
        Alignment::BottomCenter => (center_axis(container_w, child_w), container_h - child_h),
        Alignment::BottomEnd => (container_w - child_w, container_h - child_h),
    }
}

#[derive(Clone, PartialEq)]
struct ScaffoldLayout {
    content_padding: Padding,
    fab_alignment: Alignment,
    fab_offset: [Dp; 2],
    snackbar_alignment: Alignment,
    snackbar_offset: [Dp; 2],
    has_content: bool,
    has_top_bar: bool,
    has_bottom_bar: bool,
    has_fab: bool,
    has_snackbar: bool,
}

impl ScaffoldLayout {
    fn content_constraint(
        &self,
        parent_constraint: Constraint,
        top_bar_height: Px,
        bottom_bar_height: Px,
    ) -> Constraint {
        let left: Px = self.content_padding.left.into();
        let top: Px = self.content_padding.top.into();
        let right: Px = self.content_padding.right.into();
        let bottom: Px = self.content_padding.bottom.into();

        Constraint::new(
            parent_constraint.width - (left + right),
            parent_constraint.height - (top + bottom + top_bar_height + bottom_bar_height),
        )
    }
}

impl LayoutPolicy for ScaffoldLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let mut iter = children.into_iter();

        let content = self.has_content.then(|| {
            iter.next()
                .expect("scaffold content slot must exist when has_content is true")
        });
        let bottom_bar = self.has_bottom_bar.then(|| {
            iter.next()
                .expect("scaffold bottom bar slot must exist when has_bottom_bar is true")
        });
        let top_bar = self.has_top_bar.then(|| {
            iter.next()
                .expect("scaffold top bar slot must exist when has_top_bar is true")
        });
        let snackbar = self.has_snackbar.then(|| {
            iter.next()
                .expect("scaffold snackbar slot must exist when has_snackbar is true")
        });
        let fab = self.has_fab.then(|| {
            iter.next()
                .expect("scaffold fab slot must exist when has_fab is true")
        });

        let parent_constraint = *input.parent_constraint().as_ref();
        let unconstrained_children = input.parent_constraint().without_min();

        let top_bar_size = if let Some(child) = top_bar {
            child.measure(&unconstrained_children)?.size()
        } else {
            ComputedData {
                width: Px::ZERO,
                height: Px::ZERO,
            }
        };

        let bottom_bar_size = if let Some(child) = bottom_bar {
            child.measure(&unconstrained_children)?.size()
        } else {
            ComputedData {
                width: Px::ZERO,
                height: Px::ZERO,
            }
        };

        let content_size = if let Some(child) = content {
            let constraint = self.content_constraint(
                parent_constraint,
                top_bar_size.height,
                bottom_bar_size.height,
            );
            let size = child.measure(&constraint)?.size();
            let content_x: Px = self.content_padding.left.into();
            let content_y: Px = self.content_padding.top.into();
            result.place_child(
                child,
                PxPosition::new(content_x, content_y + top_bar_size.height),
            );
            Some(size)
        } else {
            None
        };

        let mut intrinsic_width = top_bar_size.width.max(bottom_bar_size.width);
        if let Some(size) = content_size {
            let left: Px = self.content_padding.left.into();
            let right: Px = self.content_padding.right.into();
            intrinsic_width = intrinsic_width.max(size.width + left + right);
        }

        let intrinsic_height = top_bar_size.height
            + bottom_bar_size.height
            + content_size
                .map(|size| {
                    let top: Px = self.content_padding.top.into();
                    let bottom: Px = self.content_padding.bottom.into();
                    size.height + top + bottom
                })
                .unwrap_or(Px::ZERO);

        let final_width = parent_constraint.width.clamp(intrinsic_width);
        let final_height = parent_constraint.height.clamp(intrinsic_height);

        if let Some(child) = top_bar {
            result.place_child(
                child,
                PxPosition::new(center_axis(final_width, top_bar_size.width), Px::ZERO),
            );
        }

        if let Some(child) = bottom_bar {
            result.place_child(
                child,
                PxPosition::new(
                    center_axis(final_width, bottom_bar_size.width),
                    final_height - bottom_bar_size.height,
                ),
            );
        }

        let bottom_reserved = bottom_bar_size.height;

        if let Some(child) = snackbar {
            let size = child.measure(&unconstrained_children)?.size();
            let (x, mut y) = compute_overlay_offset(
                self.snackbar_alignment,
                final_width,
                final_height,
                size.width,
                size.height,
            );
            if matches!(
                self.snackbar_alignment,
                Alignment::BottomStart | Alignment::BottomCenter | Alignment::BottomEnd
            ) {
                y -= bottom_reserved;
            }
            let offset_x: Px = self.snackbar_offset[0].into();
            let offset_y: Px = self.snackbar_offset[1].into();
            result.place_child(child, PxPosition::new(x + offset_x, y + offset_y));
        }

        if let Some(child) = fab {
            let size = child.measure(&unconstrained_children)?.size();
            let (x, mut y) = compute_overlay_offset(
                self.fab_alignment,
                final_width,
                final_height,
                size.width,
                size.height,
            );
            if matches!(
                self.fab_alignment,
                Alignment::BottomStart | Alignment::BottomCenter | Alignment::BottomEnd
            ) {
                y -= bottom_reserved;
            }
            let offset_x: Px = self.fab_offset[0].into();
            let offset_y: Px = self.fab_offset[1].into();
            result.place_child(child, PxPosition::new(x + offset_x, y + offset_y));
        }

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

/// # scaffold
///
/// Layout top/bottom bars with floating content for app screens with persistent
/// actions.
///
/// ## Usage
///
/// Use for screens with app bars, floating actions, and transient messages.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the scaffold container.
/// - `content_padding` — padding applied around the content area.
/// - `content` — optional main content slot.
/// - `top_bar` — optional top bar slot.
/// - `bottom_bar` — optional bottom bar slot.
/// - `floating_action_button` — optional floating action button slot.
/// - `floating_action_button_alignment` — optional floating action button
///   alignment.
/// - `floating_action_button_offset` — additional x/y offset applied to the
///   floating action button.
/// - `snackbar_host` — optional snackbar host slot.
/// - `snackbar_alignment` — optional snackbar host alignment.
/// - `snackbar_offset` — additional x/y offset applied to the snackbar host.
///
/// ## Examples
///
/// ```
/// use tessera_components::app_bar::top_app_bar;
/// use tessera_components::scaffold::scaffold;
/// use tessera_components::text::text;
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let counter = remember(|| 1u32);
///             scaffold()
///                 .top_bar(|| {
///                     top_app_bar().title("Inbox");
///                 })
///                 .content(|| {
///                     text().content("Hello scaffold");
///                 });
///             assert_eq!(counter.get(), 1);
///         });
/// }
/// ```
#[tessera]
pub fn scaffold(
    modifier: Option<Modifier>,
    content_padding: Padding,
    content: Option<RenderSlot>,
    top_bar: Option<RenderSlot>,
    bottom_bar: Option<RenderSlot>,
    floating_action_button: Option<RenderSlot>,
    floating_action_button_alignment: Option<Alignment>,
    floating_action_button_offset: [Dp; 2],
    snackbar_host: Option<RenderSlot>,
    snackbar_alignment: Option<Alignment>,
    snackbar_offset: [Dp; 2],
) {
    let modifier = modifier.unwrap_or_else(|| Modifier::new().fill_max_size());
    let fab_alignment = floating_action_button_alignment.unwrap_or(Alignment::BottomEnd);
    let snackbar_alignment = snackbar_alignment.unwrap_or(Alignment::BottomCenter);

    layout()
        .modifier(modifier)
        .layout_policy(ScaffoldLayout {
            content_padding,
            fab_alignment,
            fab_offset: floating_action_button_offset,
            snackbar_alignment,
            snackbar_offset,
            has_content: content.is_some(),
            has_top_bar: top_bar.is_some(),
            has_bottom_bar: bottom_bar.is_some(),
            has_fab: floating_action_button.is_some(),
            has_snackbar: snackbar_host.is_some(),
        })
        .child(move || {
            if let Some(content) = content {
                content.render();
            }
            if let Some(bottom_bar) = bottom_bar {
                bottom_bar.render();
            }
            if let Some(top_bar) = top_bar {
                top_bar.render();
            }
            if let Some(snackbar_host) = snackbar_host {
                snackbar_host.render();
            }
            if let Some(floating_action_button) = floating_action_button {
                floating_action_button.render();
            }
        });
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier, NoopRenderPolicy, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::modifier::{ModifierExt as _, SemanticsArgs};

    use super::scaffold;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn scaffold_layout_case() {
        scaffold()
            .modifier(Modifier::new().constrain(
                Some(tessera_ui::AxisConstraint::exact(Px::new(100))),
                Some(tessera_ui::AxisConstraint::exact(Px::new(80))),
            ))
            .top_bar(|| {
                fixed_test_box()
                    .tag("scaffold_top".to_string())
                    .width(100)
                    .height(10);
            })
            .bottom_bar(|| {
                fixed_test_box()
                    .tag("scaffold_bottom".to_string())
                    .width(100)
                    .height(12);
            })
            .content(|| {
                fixed_test_box()
                    .tag("scaffold_content".to_string())
                    .width(20)
                    .height(8);
            });
    }

    #[test]
    fn scaffold_positions_top_bottom_and_content_slots() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                scaffold_layout_case();
            },
            expect: {
                node("scaffold_top").position(0, 0).size(100, 10);
                node("scaffold_content").position(0, 10).size(20, 8);
                node("scaffold_bottom").position(0, 68).size(100, 12);
            }
        }
    }
}

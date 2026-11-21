//! Titled containers for related controls.
//!
//! ## Usage
//!
//! Organize settings, inspector sections, and related form controls.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Color, ComputedData, Constraint,
    Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, SemanticsModifierNode,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    tessera, use_context,
};

use crate::{
    alignment::Alignment, boxed::boxed, pipelines::perceptual_fill::PerceptualFillCommand,
};

use super::{
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "group_box_tests.rs"]
mod tests;

struct GroupSemantics(Option<String>);

impl SemanticsModifierNode for GroupSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.role.get_or_insert(Role::Group);
        if node.label.is_none() {
            node.label = self.0.clone();
        }
        node.hide_descendants = false;
    }
}

#[tessera]
fn group_box_content(child: RenderSlot) {
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().clip_to_bounds())
        .children(move || {
            child.render();
        });
}

#[derive(Clone, Copy, PartialEq)]
struct GroupLayout(bool);

impl LayoutPolicy for GroupLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let children = scope.children();
        let padding_width = Px::from(Dp(10.0));
        let padding_height = Px::from(Dp(if self.0 { 22.0 } else { 10.0 }));
        let mut result = LayoutResult::default();
        let title = if self.0 {
            Some(children[0].measure(
                scope,
                &Constraint::new(
                    (parent.width - Px::from(Dp(22.0))).without_min(),
                    AxisConstraint::at_most(
                        Px::from(Dp(14.0)).min(parent.height.max.unwrap_or(Px::MAX)),
                    ),
                ),
            )?)
        } else {
            None
        };
        let content = children[usize::from(self.0)];
        let mut content_width = parent.width - padding_width;
        if let Some(title) = title {
            content_width.min =
                content_width.clamp(title.width + Px::from(Dp(22.0)) - padding_width);
        }
        let measured = content.measure(
            scope,
            &Constraint::new(content_width, parent.height - padding_height),
        )?;
        let width = parent.width.clamp(
            (measured.width + padding_width)
                .max(title.map_or(Px::ZERO, |title| title.width + Px::from(Dp(22.0)))),
        );
        let height = parent.height.clamp(measured.height + padding_height);
        if self.0 {
            result.place_child(
                children[0],
                PxPosition::from_f32_xy(
                    Dp(11.0).to_pixels_f32().min(width.to_f32()),
                    -Dp(0.5).to_pixels_f32(),
                ),
            );
        }
        result.place_child(
            content,
            PxPosition::from_f32_xy(
                Dp(5.0).to_pixels_f32().min(width.to_f32()),
                Dp(if self.0 { 17.0 } else { 5.0 })
                    .to_pixels_f32()
                    .min(height.to_f32()),
            ),
        );
        Ok(result.with_size(ComputedData { width, height }))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct GroupFill {
    titled: bool,
    color: Color,
}

impl DrawModifierNode for GroupFill {
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
                color: self.color,
                radii: [Dp(12.0); 2],
                insets: [
                    if self.titled { Dp(12.0) } else { Dp::ZERO },
                    Dp::ZERO,
                    Dp::ZERO,
                    Dp::ZERO,
                ],
                clip_radius: None,
                edge_smoothness: 1.0,
                stroke: None,
                line: None,
            });
        content.draw(context.render_input);
    }
}

/// # group_box
///
/// Groups related controls in a rounded container with an optional title.
///
/// ## Usage
///
/// Organize settings or inspector controls without making the container focusable.
///
/// ## Parameters
///
/// - `title` — optional visible title and default accessible group name.
/// - `modifier` — sizing, placement, and authored semantics.
/// - `child` — grouped content, inset by five points inside the container.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{group_box::group_box, toggle::toggle};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let email = remember(|| false);
/// group_box().title("Notifications").child(move || {
///     toggle(email.get()).label("Email").on_value_change(move |on| email.set(on));
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn group_box(
    #[prop(into)] title: Option<String>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let title = title.filter(|title| !title.is_empty());
    let titled = title.is_some();
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    layout(GroupLayout(titled))
        .modifier(
            modifier
                .unwrap_or_default()
                .push_semantics(GroupSemantics(title.clone()))
                .push_draw(GroupFill {
                    titled,
                    color: if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.03),
                }),
        )
        .child(move || {
            if let Some(title) = &title {
                text(title.clone())
                    .style(TextStyle {
                        font_size: Dp(11.0),
                        line_height: Dp(14.0),
                        ..Default::default()
                    })
                    .single_line(true)
                    .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                        hidden: true,
                        ..Default::default()
                    }));
            }
            group_box_content(child.unwrap_or_else(RenderSlot::empty));
        });
}

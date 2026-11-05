//! Measurement and placement for snackbar text and actions.
//!
//! ## Usage
//!
//! Keep message text and actions aligned as notifications resize.

use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutModifierNode, LayoutResult,
    MeasurementError, Px, PxPosition, layout::MeasureScope,
};

use crate::modifier::Padding;

use super::SnackbarDefaults;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct SnackbarLayout {
    pub stacked: bool,
    pub action: bool,
    pub dismiss: bool,
    pub padding: Option<Padding>,
}

impl LayoutModifierNode for SnackbarLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = input.parent_constraint();
        let width_axis = parent.width();
        let height_axis = parent.height();
        let width = width_axis.clamp(width_axis.max.unwrap_or(Dp(600.0).to_px()));
        let defaults = self.padding.is_none();
        let padding = self.padding.unwrap_or(SnackbarDefaults::CONTENT_PADDING);
        let left = padding.left.to_px().min(width);
        let right = if defaults && self.dismiss {
            Px::ZERO
        } else if defaults && self.action {
            Dp(8.0).to_px()
        } else {
            padding.right.to_px()
        };
        let inner_width = (width - left - right).max(Px::ZERO);
        let height_max = height_axis.max;
        let children = input.children();
        let Some(message) = children.first().copied() else {
            return Ok(LayoutResult::new(ComputedData {
                width,
                height: height_axis.clamp(Px::ZERO),
            }));
        };
        let action = self.action.then(|| children[1]);
        let dismiss = self.dismiss.then(|| children[1 + usize::from(self.action)]);
        let dismiss_size = if let Some(child) = dismiss {
            Some(child.measure(
                input,
                &Constraint::new(
                    AxisConstraint::at_most(inner_width),
                    AxisConstraint::new(Px::ZERO, height_max),
                ),
            )?)
        } else {
            None
        };
        let dismiss_width = dismiss_size.map_or(Px::ZERO, |s| s.width);
        let action_size = if let Some(child) = action {
            Some(child.measure(
                input,
                &Constraint::new(
                    AxisConstraint::at_most((inner_width - dismiss_width).max(Px::ZERO)),
                    AxisConstraint::new(Px::ZERO, height_max),
                ),
            )?)
        } else {
            None
        };
        let action_width = action_size.map_or(Px::ZERO, |s| s.width);
        let controls_height = action_size
            .map_or(Px::ZERO, |s| s.height)
            .max(dismiss_size.map_or(Px::ZERO, |s| s.height));
        let text_top = padding.top.to_pixels_f32();
        let text_padding = Dp(padding.top.0 + padding.bottom.0).to_px();
        let stacked_bottom = if self.stacked {
            SnackbarDefaults::ACTION_VERTICAL_SPACING.to_px()
        } else {
            Px::ZERO
        };
        let gap = if !self.stacked && self.action && !self.dismiss {
            Dp(8.0).to_px()
        } else {
            Px::ZERO
        };
        let text_width = if self.stacked {
            if defaults {
                (width - left - padding.right.to_px()).max(Px::ZERO)
            } else {
                inner_width
            }
        } else {
            (inner_width - action_width - dismiss_width - gap).max(Px::ZERO)
        };
        let message_height_max = height_max.map(|h| {
            (h - text_padding
                - if self.stacked {
                    controls_height + stacked_bottom
                } else {
                    Px::ZERO
                })
            .max(Px::ZERO)
        });
        let text = message.measure(
            input,
            &Constraint::new(
                AxisConstraint::at_most(text_width),
                AxisConstraint::new(Px::ZERO, message_height_max),
            ),
        )?;
        let message_height = text.height + text_padding;
        let body_height = if self.stacked {
            message_height + controls_height + stacked_bottom
        } else {
            message_height.max(controls_height)
        };
        let height =
            height_axis.clamp(body_height.max(SnackbarDefaults::MIN_HEIGHT_ONE_LINE.to_px()));
        let content_height = height;
        let mut result = LayoutResult::new(ComputedData { width, height });
        let text_y = if self.stacked {
            text_top
        } else {
            (content_height - message_height).0 as f32 / 2.0 + text_top
        };
        result.place_child(message, PxPosition::from_f32_xy(left.0 as f32, text_y));
        let controls_y = if self.stacked {
            message_height.0 as f32
        } else {
            (content_height - controls_height).0 as f32 / 2.0
        };
        if let (Some(child), Some(size)) = (action, action_size) {
            result.place_child(
                child,
                PxPosition::from_f32_xy(
                    (width - right - dismiss_width - size.width).max(left).0 as f32,
                    controls_y + (controls_height - size.height).0 as f32 / 2.0,
                ),
            );
        }
        if let (Some(child), Some(size)) = (dismiss, dismiss_size) {
            result.place_child(
                child,
                PxPosition::from_f32_xy(
                    (width - right - size.width).max(left).0 as f32,
                    controls_y + (controls_height - size.height).0 as f32 / 2.0,
                ),
            );
        }
        Ok(result)
    }
}

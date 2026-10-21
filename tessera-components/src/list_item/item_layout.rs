//! Adaptive sizing and placement for Material list rows.
//!
//! ## Usage
//!
//! Keep list content consistent across layout and state changes.

use tessera_ui::{
    ComputedData, Constraint, Dp, LayoutModifierNode, LayoutResult, MeasurementError, Px,
    PxPosition, layout::MeasureScope,
};

use crate::modifier::Padding;

use super::{ListItemDefaults, ListItemVerticalAlignment};

#[derive(Clone, PartialEq)]
pub(super) struct ListItemLayout {
    pub padding: Padding,
    pub min_height: Option<Dp>,
    pub has_overline: bool,
    pub has_supporting: bool,
    pub supporting_line_height: Dp,
    pub vertical_alignment: ListItemVerticalAlignment,
}

fn rounded(value: Dp) -> Px {
    Px(value.to_pixels_f32().round().max(0.0) as i32)
}

impl LayoutModifierNode for ListItemLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = scope.children();
        let parent = *scope.parent_constraint().as_ref();
        let left = rounded(self.padding.left);
        let right = rounded(self.padding.right);
        let top = rounded(self.padding.top);
        let bottom = rounded(self.padding.bottom);
        let horizontal = left + right;
        let vertical = top + bottom;
        let mut constraint = Constraint::new(
            parent.width.without_min() - horizontal,
            parent.height.without_min() - vertical,
        );
        let leading = children[0].measure(scope, &constraint)?.size();
        constraint.width -= leading.width;
        let trailing = children[1].measure(scope, &constraint)?.size();
        constraint.width -= trailing.width;
        let overline = children[2].measure(scope, &constraint)?.size();
        constraint.height -= overline.height;
        let headline = children[3].measure(scope, &constraint)?.size();
        constraint.height -= headline.height;
        let supporting = children[4].measure(scope, &constraint)?.size();
        let multiline =
            supporting.height.to_f32() > self.supporting_line_height.to_pixels_f32().ceil();
        let minimum = self.min_height.unwrap_or_else(|| {
            if multiline {
                ListItemDefaults::MIN_HEIGHT_THREE_LINE
            } else {
                ListItemDefaults::min_height(self.has_overline, self.has_supporting)
            }
        });
        let main_height = overline.height + headline.height + supporting.height;
        let content_height = main_height.max(leading.height).max(trailing.height);
        let height = parent
            .height
            .clamp((content_height + vertical).max(rounded(minimum)));
        let main_width = overline.width.max(headline.width).max(supporting.width);
        let natural_width = horizontal + leading.width + main_width + trailing.width;
        let width = parent
            .width
            .clamp(parent.width.max.unwrap_or(natural_width));
        let available = (height - vertical).max(Px::ZERO);
        let alignment = match self.vertical_alignment {
            ListItemVerticalAlignment::Auto
                if available
                    < rounded(Dp(
                        (80.0 - self.padding.top.0 - self.padding.bottom.0).max(0.0)
                    )) =>
            {
                ListItemVerticalAlignment::Center
            }
            ListItemVerticalAlignment::Auto => ListItemVerticalAlignment::Top,
            value => value,
        };
        let y = |child_height: Px| {
            let free = (available - child_height).max(Px::ZERO);
            top + match alignment {
                ListItemVerticalAlignment::Center => Px((free.to_f32() * 0.5).round() as i32),
                ListItemVerticalAlignment::Bottom => free,
                _ => Px::ZERO,
            }
        };
        let mut result = LayoutResult::new(ComputedData { width, height });
        result.place_child(children[0], PxPosition::new(left, y(leading.height)));
        result.place_child(
            children[1],
            PxPosition::new(
                (width - right - trailing.width).max(left),
                y(trailing.height),
            ),
        );
        let x = left + leading.width;
        let mut main_y = y(main_height);
        for (child, size) in [
            (children[2], overline),
            (children[3], headline),
            (children[4], supporting),
        ] {
            result.place_child(child, PxPosition::new(x, main_y));
            main_y += size.height;
        }
        Ok(result)
    }
}

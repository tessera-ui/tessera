//! Logical axes for slider layout and input.
//!
//! ## Usage
//!
//! Keep slider segments aligned when their orientation or direction changes.

use std::collections::HashMap;

use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition,
    layout::{LayoutChild, LayoutPlacementTarget, MeasureScope},
};

use crate::{
    modifier::ModifierExt as _,
    shape_def::{RoundedCorner, Shape},
};

use super::SliderOrientation;

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct SliderAxis {
    pub orientation: SliderOrientation,
    pub reversed: bool,
}

impl SliderAxis {
    pub fn accessibility_orientation(self) -> tessera_ui::accesskit::Orientation {
        match self.orientation {
            SliderOrientation::Horizontal => tessera_ui::accesskit::Orientation::Horizontal,
            SliderOrientation::Vertical => tessera_ui::accesskit::Orientation::Vertical,
        }
    }
    pub fn track_modifier(self, thickness: Px) -> Modifier {
        match self.orientation {
            SliderOrientation::Horizontal => Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(thickness))),
            SliderOrientation::Vertical => Modifier::new()
                .fill_max_height()
                .constrain(Some(AxisConstraint::exact(thickness)), None),
        }
    }

    pub fn shape(self, shape: Shape) -> Shape {
        match shape {
            Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => {
                let [top_left, top_right, bottom_right, bottom_left] =
                    self.corners([top_left, top_right, bottom_right, bottom_left]);
                Shape::RoundedRectangle {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                }
            }
            Shape::Ellipse => Shape::Ellipse,
        }
    }
    pub fn main_length(self, size: ComputedData) -> Px {
        match self.orientation {
            SliderOrientation::Horizontal => size.width,
            SliderOrientation::Vertical => size.height,
        }
    }

    pub fn size(self, logical: ComputedData) -> ComputedData {
        match self.orientation {
            SliderOrientation::Horizontal => logical,
            SliderOrientation::Vertical => ComputedData {
                width: logical.height,
                height: logical.width,
            },
        }
    }

    pub fn constraint(self, logical: Constraint) -> Constraint {
        match self.orientation {
            SliderOrientation::Horizontal => logical,
            SliderOrientation::Vertical => Constraint::new(logical.height, logical.width),
        }
    }

    pub fn position(
        self,
        mut logical: PxPosition,
        child_size: ComputedData,
        length: Px,
    ) -> PxPosition {
        if self.reversed {
            logical.x = length.to_f32() - logical.x - child_size.width.to_f32();
        }
        match self.orientation {
            SliderOrientation::Horizontal => logical,
            SliderOrientation::Vertical => PxPosition::from_f32_xy(logical.y, logical.x),
        }
    }

    pub fn input_main(self, position: PxPosition, length: Px) -> f32 {
        let main = match self.orientation {
            SliderOrientation::Horizontal => position.x,
            SliderOrientation::Vertical => position.y,
        };
        if self.reversed {
            length.to_f32() - main
        } else {
            main
        }
    }

    pub fn corners(self, corners: [RoundedCorner; 4]) -> [RoundedCorner; 4] {
        let corners = if self.reversed {
            [corners[1], corners[0], corners[3], corners[2]]
        } else {
            corners
        };
        match self.orientation {
            SliderOrientation::Horizontal => corners,
            SliderOrientation::Vertical => [corners[0], corners[3], corners[2], corners[1]],
        }
    }
}

pub(super) struct SliderMeasureScope<'a, 'b> {
    scope: &'a mut MeasureScope<'b>,
    axis: SliderAxis,
    sizes: HashMap<u64, ComputedData>,
}

impl<'a, 'b> SliderMeasureScope<'a, 'b> {
    pub fn new(scope: &'a mut MeasureScope<'b>, axis: SliderAxis) -> Self {
        Self {
            scope,
            axis,
            sizes: HashMap::new(),
        }
    }

    pub fn children(&self) -> Vec<SliderChild> {
        self.scope.children().into_iter().map(SliderChild).collect()
    }

    pub fn finish(self, mut result: LayoutResult) -> LayoutResult {
        let length = result.size.width;
        for (key, position) in &mut result.placements {
            let size = self
                .sizes
                .get(key)
                .expect("slider child must be measured before placement");
            *position = self.axis.position(*position, *size, length);
        }
        result.size = self.axis.size(result.size);
        result
    }
}

#[derive(Clone, Copy)]
pub(super) struct SliderChild(LayoutChild);

impl LayoutPlacementTarget for SliderChild {
    fn instance_key(&self) -> u64 {
        self.0.instance_key()
    }
}

impl SliderChild {
    pub fn measure(
        self,
        scope: &mut SliderMeasureScope<'_, '_>,
        constraint: &Constraint,
    ) -> Result<ComputedData, MeasurementError> {
        let measured = self
            .0
            .measure(scope.scope, &scope.axis.constraint(*constraint))?;
        let logical_size = scope.axis.size(measured.size());
        scope.sizes.insert(self.instance_key(), logical_size);
        Ok(logical_size)
    }
}

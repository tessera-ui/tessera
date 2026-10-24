//! Fractional geometry and uniform coordinate transformations.
//!
//! ## Usage
//!
//! Map local control geometry into window coordinates without changing layout
//! units.

use crate::{Px, PxPosition, PxRect, PxSize};

/// A rectangle in fractional physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloatRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

impl FloatRect {
    /// An empty rectangle at the origin.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    /// Creates a rectangle from a fractional origin and integer layout size.
    pub fn from_position_size(position: PxPosition, size: PxSize) -> Self {
        Self {
            x: position.x,
            y: position.y,
            width: size.width.0 as f32,
            height: size.height.0 as f32,
        }
    }

    /// Returns the smallest integer rectangle containing this rectangle.
    pub fn outward(self) -> PxRect {
        let x = self.x.floor() as i32;
        let y = self.y.floor() as i32;
        let right = (self.x + self.width).ceil() as i32;
        let bottom = (self.y + self.height).ceil() as i32;
        PxRect {
            x: Px(x),
            y: Px(y),
            width: Px(right.saturating_sub(x).max(0)),
            height: Px(bottom.saturating_sub(y).max(0)),
        }
    }

    /// Rounds an origin and layout extents to the nearest integer pixels.
    pub fn rounded(self) -> PxRect {
        PxRect {
            x: Px(self.x.round() as i32),
            y: Px(self.y.round() as i32),
            width: Px(self.width.round() as i32),
            height: Px(self.height.round() as i32),
        }
    }

    /// Returns whether a point is inside the rectangle, excluding its far
    /// edges.
    pub fn contains(self, point: PxPosition) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }

    /// Returns the positive-area intersection of two rectangles.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (right > x && bottom > y).then_some(Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }
}

impl From<PxRect> for FloatRect {
    fn from(rect: PxRect) -> Self {
        Self {
            x: rect.x.0 as f32,
            y: rect.y.0 as f32,
            width: rect.width.0 as f32,
            height: rect.height.0 as f32,
        }
    }
}

/// A positive uniform scale followed by a translation in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalToWindow {
    scale: f32,
    translation: PxPosition,
}

impl Default for LocalToWindow {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl LocalToWindow {
    /// The identity coordinate transformation.
    pub const IDENTITY: Self = Self {
        scale: 1.0,
        translation: PxPosition::ZERO,
    };

    /// Creates a finite, invertible uniform coordinate transformation.
    pub fn new(scale: f32, translation: PxPosition) -> Option<Self> {
        (scale.is_finite()
            && scale > 0.0
            && scale.recip().is_finite()
            && translation.x.is_finite()
            && translation.y.is_finite())
        .then_some(Self { scale, translation })
    }

    /// Returns the uniform scale.
    pub const fn scale(self) -> f32 {
        self.scale
    }

    /// Returns the mapped local origin.
    pub const fn translation(self) -> PxPosition {
        self.translation
    }

    /// Maps a local point into window coordinates.
    pub fn map_point(self, point: PxPosition) -> PxPosition {
        self.translation + self.map_vector(point)
    }

    /// Maps a local displacement into window coordinates.
    pub fn map_vector(self, vector: PxPosition) -> PxPosition {
        PxPosition::from_f32_xy(vector.x * self.scale, vector.y * self.scale)
    }

    /// Maps a window point into local coordinates.
    pub fn inverse_point(self, point: PxPosition) -> PxPosition {
        self.inverse_vector(point - self.translation)
    }

    /// Maps a window displacement into local coordinates.
    pub fn inverse_vector(self, vector: PxPosition) -> PxPosition {
        PxPosition::from_f32_xy(vector.x / self.scale, vector.y / self.scale)
    }

    /// Composes a child transformation followed by this transformation.
    pub fn compose(self, child: Self) -> Option<Self> {
        Self::new(self.scale * child.scale, self.map_point(child.translation))
    }

    /// Moves the local origin without changing the scale.
    pub fn with_local_origin(self, origin: PxPosition) -> Self {
        Self {
            scale: self.scale,
            translation: self.map_point(origin),
        }
    }

    /// Maps an integer layout size into fractional window extents.
    pub fn map_size(self, size: PxSize) -> [f32; 2] {
        [
            size.width.0 as f32 * self.scale,
            size.height.0 as f32 * self.scale,
        ]
    }

    /// Maps a local rectangle into window coordinates.
    pub fn map_rect(self, rect: FloatRect) -> FloatRect {
        let origin = self.map_point(PxPosition::from_f32_xy(rect.x, rect.y));
        FloatRect {
            x: origin.x,
            y: origin.y,
            width: rect.width * self.scale,
            height: rect.height * self.scale,
        }
    }

    /// Maps a window rectangle into local coordinates.
    pub fn inverse_rect(self, rect: FloatRect) -> FloatRect {
        let origin = self.inverse_point(PxPosition::from_f32_xy(rect.x, rect.y));
        FloatRect {
            x: origin.x,
            y: origin.y,
            width: rect.width / self.scale,
            height: rect.height / self.scale,
        }
    }
}

/// Local layout dimensions and their exact placement in the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutCoordinates {
    /// Unscaled physical layout dimensions.
    pub local_size: PxSize,
    /// Coordinate transformation for this layout node.
    pub local_to_window: LocalToWindow,
}

impl LayoutCoordinates {
    /// Returns the exact bounds of this layout node in the window.
    pub fn window_bounds(self) -> FloatRect {
        self.local_to_window.map_rect(FloatRect::from_position_size(
            PxPosition::ZERO,
            self.local_size,
        ))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ScaleChildren(f32);

impl crate::BuildModifierNode for ScaleChildren {
    fn apply(&self, runtime: &mut crate::runtime::TesseraRuntime) {
        runtime.set_current_children_scale(self.0);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ScaleLayout(f32);

impl crate::LayoutModifierNode for ScaleLayout {
    fn measure(
        &self,
        scope: &mut crate::layout::MeasureScope<'_>,
    ) -> Result<crate::LayoutResult, crate::MeasurementError> {
        let children = scope.children();
        if children.is_empty() {
            return Ok(crate::LayoutResult::new(crate::ComputedData {
                width: scope.parent_constraint().width().min,
                height: scope.parent_constraint().height().min,
            }));
        }
        if children.len() != 1 {
            return Err(crate::MeasurementError::MeasureFnFailed(
                "A scale boundary requires exactly one layout child".to_owned(),
            ));
        }
        let parent = *scope.parent_constraint().as_ref();
        let inverse_axis = |axis: crate::AxisConstraint| {
            let max = axis
                .max
                .map(|value| Px((value.0 as f32 / self.0).floor() as i32));
            let min = Px((axis.min.0 as f32 / self.0).ceil() as i32);
            crate::AxisConstraint::new(max.map_or(min, |max| min.min(max)), max)
        };
        let constraint = crate::Constraint {
            width: inverse_axis(parent.width),
            height: inverse_axis(parent.height),
        };
        let child = children[0].measure(scope, &constraint)?;
        let size = crate::ComputedData {
            width: parent
                .width
                .clamp(Px((child.width.0 as f32 * self.0).ceil() as i32)),
            height: parent
                .height
                .clamp(Px((child.height.0 as f32 * self.0).ceil() as i32)),
        };
        let mut result = crate::LayoutResult::new(size);
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

/// # scaled_box
///
/// Scale a single native child uniformly for responsive illustrations and
/// previews.
///
/// ## Usage
///
/// Fit an interactive control or fixed-size scene into a smaller display
/// region.
///
/// ## Parameters
///
/// - `scale` — finite, positive, invertible scale relative to the child's
///   layout units.
/// - `child` — optional content producing one direct layout child; empty when
///   omitted.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{LocalToWindow, Px, PxPosition, PxSize, layout::layout, scaled_box};
///
/// #[tessera_ui::tessera]
/// fn preview() {
///     scaled_box(0.625).child(|| {
///         layout().child(|| {});
///     });
/// }
///
/// let transform = LocalToWindow::new(0.625, PxPosition::ZERO).unwrap();
/// assert_eq!(
///     transform.map_size(PxSize::new(Px(240), Px(56))),
///     [150.0, 35.0]
/// );
/// ```
#[crate::tessera(crate)]
pub fn scaled_box(scale: f32, child: Option<crate::RenderSlot>) {
    let transform = LocalToWindow::new(scale, PxPosition::ZERO)
        .expect("Scale must be finite, positive, and invertible");
    let child = child.unwrap_or_else(crate::RenderSlot::empty);
    crate::layout::layout()
        .policy(ScaleLayout(transform.scale()))
        .modifier(crate::Modifier::new().push_build(ScaleChildren(transform.scale())))
        .child(move || child.render());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_scale_preserves_exact_control_geometry_and_outward_clip() {
        let transform = LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5)).unwrap();
        let bounds = transform.map_rect(FloatRect::from_position_size(
            PxPosition::ZERO,
            PxSize::new(Px(240), Px(56)),
        ));
        assert_eq!(
            bounds,
            FloatRect {
                x: 10.25,
                y: 20.5,
                width: 150.0,
                height: 35.0
            }
        );
        assert_eq!(
            bounds.outward(),
            PxRect {
                x: Px(10),
                y: Px(20),
                width: Px(151),
                height: Px(36)
            }
        );
        assert_eq!(
            transform.map_point(PxPosition::from_f32_xy(52.0, 0.0)).x,
            42.75
        );
        assert_eq!(
            transform.inverse_point(PxPosition::from_f32_xy(40.25, 35.5)),
            PxPosition::from_f32_xy(48.0, 24.0)
        );
    }

    #[test]
    fn nested_uniform_transforms_compose_once_and_reject_noninvertible_values() {
        let parent = LocalToWindow::new(0.5, PxPosition::from_f32_xy(10.25, 20.5)).unwrap();
        let child = LocalToWindow::new(1.25, PxPosition::from_f32_xy(8.0, 12.0)).unwrap();
        let combined = parent.compose(child).unwrap();
        assert_eq!(
            combined.map_point(PxPosition::from_f32_xy(16.0, 24.0)),
            PxPosition::from_f32_xy(24.25, 41.5)
        );
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::from_bits(1)] {
            assert!(LocalToWindow::new(scale, PxPosition::ZERO).is_none());
        }
    }
}

#[cfg(all(test, feature = "testing"))]
#[path = "geometry_tests.rs"]
mod layout_tests;

//! Physical pixel geometry.
//!
//! ## Usage
//!
//! Use Px for measured dimensions and PxPosition for fractional placement.

use std::ops::{AddAssign, Neg, SubAssign};

use crate::{ComputedData, dp::Dp};

/// A signed physical pixel coordinate, independent of display density.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Px(pub i32);

impl Px {
    /// A constant representing zero pixels.
    pub const ZERO: Self = Self(0);

    /// A constant representing the maximum possible pixel value.
    pub const MAX: Self = Self(i32::MAX);

    /// Returns the raw i32 value.
    pub fn raw(self) -> i32 {
        self.0
    }

    /// Creates a new `Px` instance from an i32 value.
    pub const fn new(value: i32) -> Self {
        Px(value)
    }

    /// Scales density-independent units to whole physical pixels.
    pub fn from_dp(dp: Dp) -> Self {
        Px(dp.to_pixels_f64() as i32)
    }

    /// Converts to density-independent units using the active session's density.
    pub fn to_dp(self) -> Dp {
        Dp::from_pixels_f64(self.0 as f64)
    }

    /// Returns the absolute value as a u32
    pub fn abs(self) -> u32 {
        self.0.unsigned_abs()
    }

    /// Returns only the positive value, or zero if negative.
    pub fn positive(self) -> u32 {
        if self.0 < 0 { 0 } else { self.0 as u32 }
    }

    /// Returns the negative value, or zero if positive.
    pub fn negative(self) -> i32 {
        if self.0 > 0 { 0 } else { self.0 }
    }

    /// Converts the pixel value to f32.
    pub fn to_f32(self) -> f32 {
        self.0 as f32
    }

    /// Truncates a fractional pixel value toward zero, saturating at `i32` bounds.
    pub fn from_f32(value: f32) -> Self {
        Px(value as i32)
    }

    /// Truncates a fractional pixel value toward zero, saturating at `i32` bounds.
    pub fn saturating_from_f32(value: f32) -> Self {
        Self::from_f32(value)
    }

    /// Adds with saturation at `i32` bounds.
    ///
    /// ```
    /// use tessera_ui::Px;
    /// assert_eq!(Px(i32::MAX).saturating_add(Px(1)), Px(i32::MAX));
    /// ```
    pub fn saturating_add(self, rhs: Self) -> Self {
        Px(self.0.saturating_add(rhs.0))
    }

    /// Subtracts with saturation at `i32` bounds.
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Px(self.0.saturating_sub(rhs.0))
    }

    /// Multiplies by a scalar, truncating to whole physical pixels.
    pub fn mul_f32(self, rhs: f32) -> Self {
        Px((self.0 as f32 * rhs) as i32)
    }

    /// Divides by a scalar, truncating to whole physical pixels.
    pub fn div_f32(self, rhs: f32) -> Self {
        Px::from_f32(self.to_f32() / rhs)
    }
}

/// A fractional physical-pixel position; x increases rightward and y downward.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PxPosition {
    /// The x-coordinate in physical pixels. May be fractional.
    pub x: f32,
    /// The y-coordinate in physical pixels. May be fractional.
    pub y: f32,
}

impl PxPosition {
    /// A constant representing the zero position (0, 0).
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a new position from whole-pixel coordinates.
    pub const fn new(x: Px, y: Px) -> Self {
        Self {
            x: x.0 as f32,
            y: y.0 as f32,
        }
    }

    /// Creates a position from fractional pixel coordinates.
    pub const fn from_f32_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the x-coordinate rounded to the nearest whole pixel.
    pub fn x_px(self) -> Px {
        Px(self.x.round() as i32)
    }

    /// Returns the y-coordinate rounded to the nearest whole pixel.
    pub fn y_px(self) -> Px {
        Px(self.y.round() as i32)
    }

    /// Offsets the position by the given whole-pixel deltas.
    pub fn offset(self, dx: Px, dy: Px) -> Self {
        Self {
            x: self.x + dx.0 as f32,
            y: self.y + dy.0 as f32,
        }
    }

    /// Offsets the position by fractional pixel deltas.
    pub fn offset_f32(self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Offsets the position with saturating arithmetic.
    pub fn saturating_offset(self, dx: Px, dy: Px) -> Self {
        const MIN: f32 = i32::MIN as f32;
        const MAX: f32 = i32::MAX as f32;
        Self {
            x: (self.x + dx.0 as f32).clamp(MIN, MAX),
            y: (self.y + dy.0 as f32).clamp(MIN, MAX),
        }
    }

    /// Calculates the Euclidean distance to another position.
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Converts the position to a 2D f32 array.
    pub fn to_f32_arr2(self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// Converts the position to a 3D f32 array with z=0.
    pub fn to_f32_arr3(self) -> [f32; 3] {
        [self.x, self.y, 0.0]
    }

    /// Creates a position from a 2D f32 array.
    pub fn from_f32_arr2(arr: [f32; 2]) -> Self {
        Self::from_f32_xy(arr[0], arr[1])
    }

    /// Creates a position from a 3D f32 array, ignoring z.
    pub fn from_f32_arr3(arr: [f32; 3]) -> Self {
        Self::from_f32_xy(arr[0], arr[1])
    }

    /// Converts the position to a 2D f64 array.
    pub fn to_f64_arr2(self) -> [f64; 2] {
        [self.x as f64, self.y as f64]
    }

    /// Converts the position to a 3D f64 array with z=0.
    pub fn to_f64_arr3(self) -> [f64; 3] {
        [self.x as f64, self.y as f64, 0.0]
    }

    /// Creates a position from a 2D f64 array.
    pub fn from_f64_arr2(arr: [f64; 2]) -> Self {
        Self::from_f32_xy(arr[0] as f32, arr[1] as f32)
    }

    /// Creates a position from a 3D f64 array, ignoring the z component.
    pub fn from_f64_arr3(arr: [f64; 3]) -> Self {
        Self::from_f32_xy(arr[0] as f32, arr[1] as f32)
    }
}

/// A width and height in whole physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PxSize {
    /// The width in physical pixels
    pub width: Px,
    /// The height in physical pixels
    pub height: Px,
}

impl PxSize {
    /// A constant representing zero size (0×0).
    pub const ZERO: Self = Self {
        width: Px(0),
        height: Px(0),
    };

    /// Creates a new size from width and height.
    pub const fn new(width: Px, height: Px) -> Self {
        Self { width, height }
    }

    /// Converts the size to a 2D f32 array.
    pub fn to_f32_arr2(self) -> [f32; 2] {
        [self.width.0 as f32, self.height.0 as f32]
    }
}

/// A 2D rectangle in physical pixel space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PxRect {
    /// The x-coordinate of the top-left corner
    pub x: Px,
    /// The y-coordinate of the top-left corner
    pub y: Px,
    /// The width of the rectangle
    pub width: Px,
    /// The height of the rectangle
    pub height: Px,
}

impl PxRect {
    /// A constant representing a zero rectangle (0×0 at position (0, 0)).
    pub const ZERO: Self = Self {
        x: Px::ZERO,
        y: Px::ZERO,
        width: Px::ZERO,
        height: Px::ZERO,
    };

    /// Creates a new rectangle from position and size.
    pub const fn new(x: Px, y: Px, width: Px, height: Px) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a new rectangle from a position and size.
    pub fn from_position_size(position: PxPosition, size: PxSize) -> Self {
        Self::new(position.x_px(), position.y_px(), size.width, size.height)
    }

    pub(crate) fn enclosing(position: PxPosition, size: PxSize) -> Self {
        Self::enclosing_f32(position, size.to_f32_arr2())
    }

    pub(crate) fn enclosing_f32(position: PxPosition, size: [f32; 2]) -> Self {
        let x = position.x.floor() as i32;
        let y = position.y.floor() as i32;
        Self::new(
            Px(x),
            Px(y),
            Px(((position.x + size[0]).ceil() as i32).saturating_sub(x)),
            Px(((position.y + size[1]).ceil() as i32).saturating_sub(y)),
        )
    }

    /// Returns whether the rectangles do not overlap; touching edges do not overlap.
    pub fn is_orthogonal(&self, other: &Self) -> bool {
        let x_overlap = self.x.0 < other.x.0 + other.width.0 && other.x.0 < self.x.0 + self.width.0;

        let y_overlap =
            self.y.0 < other.y.0 + other.height.0 && other.y.0 < self.y.0 + self.height.0;

        !x_overlap || !y_overlap
    }

    /// Returns the smallest rectangle containing both rectangles.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxRect};
    ///
    /// let rect1 = PxRect::new(Px::new(0), Px::new(0), Px::new(100), Px::new(100));
    /// let rect2 = PxRect::new(Px::new(50), Px::new(50), Px::new(100), Px::new(100));
    /// let union_rect = rect1.union(&rect2);
    /// assert_eq!(
    ///     union_rect,
    ///     PxRect::new(Px::new(0), Px::new(0), Px::new(150), Px::new(150))
    /// );
    /// ```
    pub fn union(&self, other: &Self) -> Self {
        let x = self.x.0.min(other.x.0);
        let y = self.y.0.min(other.y.0);
        let width = (self.x.0 + self.width.0).max(other.x.0 + other.width.0) - x;
        let height = (self.y.0 + self.height.0).max(other.y.0 + other.height.0) - y;

        Self {
            x: Px(x),
            y: Px(y),
            width: Px(width),
            height: Px(height),
        }
    }

    /// Returns the area, treating negative width or height as zero.
    pub fn area(&self) -> u32 {
        let width = self.width.0.max(0) as u32;
        let height = self.height.0.max(0) as u32;
        width * height
    }

    /// Returns the overlapping rectangle, or None when edges only touch.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxRect};
    ///
    /// let rect1 = PxRect::new(Px::new(0), Px::new(0), Px::new(100), Px::new(100));
    /// let rect2 = PxRect::new(Px::new(50), Px::new(50), Px::new(100), Px::new(100));
    /// let intersection = rect1.intersection(&rect2);
    /// assert_eq!(
    ///     intersection,
    ///     Some(PxRect::new(
    ///         Px::new(50),
    ///         Px::new(50),
    ///         Px::new(50),
    ///         Px::new(50)
    ///     ))
    /// );
    /// ```
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let x1 = self.x.0.max(other.x.0);
        let y1 = self.y.0.max(other.y.0);
        let x2 = (self.x.0 + self.width.0).min(other.x.0 + other.width.0);
        let y2 = (self.y.0 + self.height.0).min(other.y.0 + other.height.0);

        if x1 < x2 && y1 < y2 {
            Some(Self {
                x: Px(x1),
                y: Px(y1),
                width: Px(x2 - x1),
                height: Px(y2 - y1),
            })
        } else {
            None
        }
    }

    /// Includes the top and left edges and excludes the bottom and right edges.
    pub fn contains(&self, point: PxPosition) -> bool {
        point.x >= self.x.0 as f32
            && point.x < (self.x.0 + self.width.0) as f32
            && point.y >= self.y.0 as f32
            && point.y < (self.y.0 + self.height.0) as f32
    }
}

impl From<[Px; 2]> for PxSize {
    fn from(size: [Px; 2]) -> Self {
        Self {
            width: size[0],
            height: size[1],
        }
    }
}

impl From<PxSize> for winit::dpi::PhysicalSize<i32> {
    fn from(size: PxSize) -> Self {
        winit::dpi::PhysicalSize {
            width: size.width.raw(),
            height: size.height.raw(),
        }
    }
}

impl From<winit::dpi::PhysicalSize<u32>> for PxSize {
    fn from(size: winit::dpi::PhysicalSize<u32>) -> Self {
        Self {
            width: Px(size.width as i32),
            height: Px(size.height as i32),
        }
    }
}

impl From<ComputedData> for PxSize {
    fn from(data: ComputedData) -> Self {
        Self {
            width: data.width,
            height: data.height,
        }
    }
}

impl From<PxSize> for winit::dpi::Size {
    fn from(size: PxSize) -> Self {
        winit::dpi::PhysicalSize::from(size).into()
    }
}

impl std::ops::Add for Px {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Px(self.0 + rhs.0)
    }
}

impl Neg for Px {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Px::new(-self.0)
    }
}

impl std::ops::Sub for Px {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Px(self.0 - rhs.0)
    }
}

impl std::ops::Mul for Px {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Px(self.0 * rhs.0)
    }
}

impl std::ops::Div for Px {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Px(self.0 / rhs.0)
    }
}

impl std::ops::Mul<i32> for Px {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Px(self.0 * rhs)
    }
}

impl std::ops::Div<i32> for Px {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Px(self.0 / rhs)
    }
}

impl From<i32> for Px {
    fn from(value: i32) -> Self {
        Px(value)
    }
}

impl From<u32> for Px {
    fn from(value: u32) -> Self {
        Px(value as i32)
    }
}

impl From<Dp> for Px {
    fn from(dp: Dp) -> Self {
        Px::from_dp(dp)
    }
}

impl From<PxPosition> for winit::dpi::PhysicalPosition<i32> {
    fn from(pos: PxPosition) -> Self {
        winit::dpi::PhysicalPosition {
            x: pos.x_px().0,
            y: pos.y_px().0,
        }
    }
}

impl From<PxPosition> for winit::dpi::Position {
    fn from(pos: PxPosition) -> Self {
        winit::dpi::PhysicalPosition::from(pos).into()
    }
}

impl AddAssign for Px {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Px {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Add for PxPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PxPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for PxPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        PxPosition {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<[i32; 2]> for PxPosition {
    fn from(pos: [i32; 2]) -> Self {
        PxPosition::from_f32_xy(pos[0] as f32, pos[1] as f32)
    }
}

impl From<PxPosition> for [i32; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x_px().0, pos.y_px().0]
    }
}

impl From<[u32; 2]> for PxPosition {
    fn from(pos: [u32; 2]) -> Self {
        PxPosition::from_f32_xy(pos[0] as f32, pos[1] as f32)
    }
}

impl From<PxPosition> for [u32; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x_px().positive(), pos.y_px().positive()]
    }
}

impl From<[Px; 2]> for PxPosition {
    fn from(pos: [Px; 2]) -> Self {
        PxPosition::from_f32_xy(pos[0].0 as f32, pos[1].0 as f32)
    }
}

impl From<PxPosition> for [Px; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x_px(), pos.y_px()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_from_f32() {
        assert_eq!(Px::saturating_from_f32(f32::MAX), Px(i32::MAX));
        assert_eq!(Px::saturating_from_f32(f32::MIN), Px(i32::MIN));
        assert_eq!(Px::saturating_from_f32(100.5), Px(100));
        assert_eq!(Px::saturating_from_f32(-100.5), Px(-100));
    }
}

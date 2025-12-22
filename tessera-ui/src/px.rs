//! Physical pixel coordinate system for Tessera UI framework.
//!
//! This module provides types and operations for working with physical pixel
//! coordinates, positions, and sizes. Physical pixels represent actual screen
//! pixels and are used internally by the rendering system.
//!
//! # Key Types
//!
//! - [`Px`] - A single physical pixel coordinate value that supports negative
//!   values for scrolling
//! - [`PxPosition`] - A 2D position in physical pixel space (x, y coordinates)
//! - [`PxSize`] - A 2D size in physical pixel space (width, height dimensions)
//!
//! # Coordinate System
//!
//! The coordinate system uses:
//! - Origin (0, 0) at the top-left corner
//! - X-axis increases to the right
//! - Y-axis increases downward
//! - Negative coordinates are supported for scrolling and off-screen
//!   positioning
//!
//! # Conversion
//!
//! Physical pixels can be converted to and from density-independent pixels
//! ([`Dp`]):
//! - Use [`Px::from_dp`] to convert from Dp to Px
//! - Use [`Px::to_dp`] to convert from Px to Dp
//!
//! # Example
//!
//! ```
//! use tessera_ui::dp::Dp;
//! use tessera_ui::px::{Px, PxPosition, PxSize};
//!
//! // Create pixel values
//! let x = Px::new(100);
//! let y = Px::new(200);
//!
//! // Create a position
//! let position = PxPosition::new(x, y);
//!
//! // Create a size
//! let size = PxSize::new(Px::new(300), Px::new(400));
//!
//! // Arithmetic operations
//! let offset_position = position.offset(Px::new(10), Px::new(-5));
//!
//! // Convert between Dp and Px
//! let dp_value = Dp(16.0);
//! let px_value = Px::from_dp(dp_value);
//! ```

use std::ops::{AddAssign, Neg, SubAssign};

use crate::dp::{Dp, SCALE_FACTOR};

/// A physical pixel coordinate value.
///
/// This type represents a single coordinate value in physical pixel space.
/// Physical pixels correspond directly to screen pixels and are used internally
/// by the rendering system. Unlike density-independent pixels ([`Dp`]),
/// physical pixels are not scaled based on screen density.
///
/// # Features
///
/// - Supports negative values for scrolling and off-screen positioning
/// - Provides arithmetic operations (addition, subtraction, multiplication,
///   division)
/// - Includes saturating arithmetic to prevent overflow
/// - Converts to/from density-independent pixels ([`Dp`])
/// - Converts to/from floating-point values with overflow protection
///
/// # Examples
///
/// ```
/// use tessera_ui::px::Px;
///
/// // Create pixel values
/// let px1 = Px::new(100);
/// let px2 = Px::new(-50); // Negative values supported
///
/// // Arithmetic operations
/// let sum = px1 + px2; // Px(50)
/// let doubled = px1 * 2; // Px(200)
///
/// // Saturating arithmetic prevents overflow
/// let max_px = Px::new(i32::MAX);
/// let safe_add = max_px.saturating_add(Px::new(1)); // Still Px(i32::MAX)
///
/// // Convert to absolute value for rendering
/// let abs_value = Px::new(-10).abs(); // 0 (negative becomes 0)
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Px(pub i32);

impl Px {
    /// A constant representing zero pixels.
    pub const ZERO: Self = Self(0);

    /// A constant representing the maximum possible pixel value.
    pub const MAX: Self = Self(i32::MAX);

    /// Returns the raw i32 value.
    ///
    /// This provides direct access to the underlying integer value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px = Px::new(42);
    /// assert_eq!(px.raw(), 42);
    /// ```
    pub fn raw(self) -> i32 {
        self.0
    }

    /// Creates a new `Px` instance from an i32 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as an i32. Negative values are allowed.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let positive = Px::new(100);
    /// let negative = Px::new(-50);
    /// let zero = Px::new(0);
    /// ```
    pub const fn new(value: i32) -> Self {
        Px(value)
    }

    /// Converts from density-independent pixels ([`Dp`]) to physical pixels.
    ///
    /// This conversion uses the current scale factor to determine how many
    /// physical pixels correspond to the given Dp value. The scale factor
    /// is typically determined by the screen's pixel density.
    ///
    /// # Arguments
    ///
    /// * `dp` - The density-independent pixel value to convert
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::dp::Dp;
    /// use tessera_ui::px::Px;
    ///
    /// let dp_value = Dp(16.0);
    /// let px_value = Px::from_dp(dp_value);
    /// ```
    pub fn from_dp(dp: Dp) -> Self {
        Px(dp.to_pixels_f64() as i32)
    }

    /// Converts from physical pixels to density-independent pixels ([`Dp`]).
    ///
    /// This conversion uses the current scale factor to determine the Dp value
    /// that corresponds to this physical pixel value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px_value = Px::new(32);
    /// let dp_value = px_value.to_dp();
    /// ```
    pub fn to_dp(self) -> Dp {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((self.0 as f64) / scale_factor)
    }

    /// Returns the absolute value as a u32
    ///
    /// This method is primarily used for coordinate conversion during
    /// rendering, where negative coordinates need to be handled
    /// appropriately.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// assert_eq!(Px::new(10).abs(), 10);
    /// assert_eq!(Px::new(-5).abs(), 5);
    /// assert_eq!(Px::new(0).abs(), 0);
    /// ```
    pub fn abs(self) -> u32 {
        self.0.unsigned_abs()
    }

    /// Returns only the positive value, or zero if negative.
    ///
    /// This is useful for ensuring that pixel values are always non-negative,
    /// especially when dealing with rendering or layout calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// assert_eq!(Px::new(10).positive(), 10);
    /// assert_eq!(Px::new(-5).positive(), 0);
    /// assert_eq!(Px::new(0).positive(), 0);
    /// ```
    pub fn positive(self) -> u32 {
        if self.0 < 0 { 0 } else { self.0 as u32 }
    }

    /// Returns the negative value, or zero if positive.
    ///
    /// This is useful for ensuring that pixel values are always non-positive,
    /// especially when dealing with rendering or layout calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// assert_eq!(Px::new(10).negative(), 0);
    /// assert_eq!(Px::new(-5).negative(), -5);
    /// assert_eq!(Px::new(0).negative(), 0);
    pub fn negative(self) -> i32 {
        if self.0 > 0 { 0 } else { self.0 }
    }

    /// Converts the pixel value to f32.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px = Px::new(42);
    /// assert_eq!(px.to_f32(), 42.0);
    /// ```
    pub fn to_f32(self) -> f32 {
        self.0 as f32
    }

    /// Creates a `Px` from an f32 value.
    ///
    /// # Panics
    ///
    /// This function may panic on overflow in debug builds when the f32 value
    /// cannot be represented as an i32.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px = Px::from_f32(42.7);
    /// assert_eq!(px.raw(), 42);
    /// ```
    pub fn from_f32(value: f32) -> Self {
        Px(value as i32)
    }

    /// Creates a `Px` from an f32 value, saturating at the numeric bounds
    /// instead of overflowing.
    ///
    /// This is the safe alternative to [`from_f32`](Self::from_f32) that
    /// handles overflow by clamping the value to the valid i32 range.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let normal = Px::saturating_from_f32(42.7);
    /// assert_eq!(normal.raw(), 42);
    ///
    /// let max_val = Px::saturating_from_f32(f32::MAX);
    /// assert_eq!(max_val.raw(), i32::MAX);
    ///
    /// let min_val = Px::saturating_from_f32(f32::MIN);
    /// assert_eq!(min_val.raw(), i32::MIN);
    /// ```
    pub fn saturating_from_f32(value: f32) -> Self {
        let clamped_value = value.clamp(i32::MIN as f32, i32::MAX as f32);
        Px(clamped_value as i32)
    }

    /// Saturating integer addition.
    ///
    /// Computes `self + rhs`, saturating at the numeric bounds instead of
    /// overflowing. This prevents integer overflow by clamping the result
    /// to the valid i32 range.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand side value to add
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let a = Px::new(10);
    /// let b = Px::new(5);
    /// assert_eq!(a.saturating_add(b), Px::new(15));
    ///
    /// // Prevents overflow
    /// let max = Px::new(i32::MAX);
    /// assert_eq!(max.saturating_add(Px::new(1)), max);
    /// ```
    pub fn saturating_add(self, rhs: Self) -> Self {
        Px(self.0.saturating_add(rhs.0))
    }

    /// Saturating integer subtraction.
    ///
    /// Computes `self - rhs`, saturating at the numeric bounds instead of
    /// overflowing. This prevents integer underflow by clamping the result
    /// to the valid i32 range.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand side value to subtract
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let a = Px::new(10);
    /// let b = Px::new(5);
    /// assert_eq!(a.saturating_sub(b), Px::new(5));
    ///
    /// // Prevents underflow
    /// let min = Px::new(i32::MIN);
    /// assert_eq!(min.saturating_sub(Px::new(1)), min);
    /// ```
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Px(self.0.saturating_sub(rhs.0))
    }

    /// Multiplies the pixel value by a scalar f32.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The scalar value to multiply by
    ///
    /// # Returns
    ///
    /// A new `Px` instance with the result of the multiplication.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px = Px::new(10);
    /// let result = px.mul_f32(2.0);
    /// assert_eq!(result, Px::new(20));
    /// ```
    pub fn mul_f32(self, rhs: f32) -> Self {
        Px((self.0 as f32 * rhs) as i32)
    }

    /// Divides the pixel value by a scalar f32.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The scalar value to divide by
    ///
    /// # Returns
    ///
    /// A new `Px` instance with the result of the division.
    ///
    /// # Panics
    ///
    /// This function may panic if `rhs` is zero, as division by zero is
    /// undefined.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::Px;
    ///
    /// let px = Px::new(20);
    /// let result = px.div_f32(2.0);
    /// assert_eq!(result, Px::new(10));
    /// ```
    pub fn div_f32(self, rhs: f32) -> Self {
        Px::from_f32(self.to_f32() / rhs)
    }
}

/// A 2D position in physical pixel space.
///
/// This type represents a position with x and y coordinates in physical pixel
/// space. Physical pixels correspond directly to screen pixels and are used
/// internally by the rendering system.
///
/// # Coordinate System
///
/// - Origin (0, 0) is at the top-left corner
/// - X-axis increases to the right
/// - Y-axis increases downward
/// - Negative coordinates are supported for scrolling and off-screen
///   positioning
///
/// # Examples
///
/// ```
/// use tessera_ui::px::{Px, PxPosition};
///
/// // Create a position
/// let position = PxPosition::new(Px::new(100), Px::new(200));
///
/// // Offset the position
/// let offset_position = position.offset(Px::new(10), Px::new(-5));
///
/// // Calculate distance between positions
/// let other_position = PxPosition::new(Px::new(103), Px::new(196));
/// let distance = position.distance_to(other_position);
///
/// // Arithmetic operations
/// let sum = position + other_position;
/// let diff = position - other_position;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PxPosition {
    /// The x-coordinate in physical pixels
    pub x: Px,
    /// The y-coordinate in physical pixels
    pub y: Px,
}

impl PxPosition {
    /// A constant representing the zero position (0, 0).
    pub const ZERO: Self = Self { x: Px(0), y: Px(0) };

    /// Creates a new position from x and y coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate in physical pixels
    /// * `y` - The y-coordinate in physical pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(100), Px::new(200));
    /// assert_eq!(position.x, Px::new(100));
    /// assert_eq!(position.y, Px::new(200));
    /// ```
    pub const fn new(x: Px, y: Px) -> Self {
        Self { x, y }
    }

    /// Offsets the position by the given deltas.
    ///
    /// # Panics
    ///
    /// This function may panic on overflow in debug builds.
    ///
    /// # Arguments
    ///
    /// * `dx` - The x-axis offset in physical pixels
    /// * `dy` - The y-axis offset in physical pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// let offset_position = position.offset(Px::new(5), Px::new(-3));
    /// assert_eq!(offset_position, PxPosition::new(Px::new(15), Px::new(17)));
    /// ```
    pub fn offset(self, dx: Px, dy: Px) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Offsets the position with saturating arithmetic.
    ///
    /// This prevents overflow by clamping the result to the valid coordinate
    /// range.
    ///
    /// # Arguments
    ///
    /// * `dx` - The x-axis offset in physical pixels
    /// * `dy` - The y-axis offset in physical pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// let offset_position = position.saturating_offset(Px::new(5), Px::new(-3));
    /// assert_eq!(offset_position, PxPosition::new(Px::new(15), Px::new(17)));
    ///
    /// // Prevents overflow
    /// let max_position = PxPosition::new(Px::new(i32::MAX), Px::new(i32::MAX));
    /// let safe_offset = max_position.saturating_offset(Px::new(1), Px::new(1));
    /// assert_eq!(safe_offset, max_position);
    /// ```
    pub fn saturating_offset(self, dx: Px, dy: Px) -> Self {
        Self {
            x: self.x.saturating_add(dx),
            y: self.y.saturating_add(dy),
        }
    }

    /// Calculates the Euclidean distance to another position.
    ///
    /// # Arguments
    ///
    /// * `other` - The other position to calculate distance to
    ///
    /// # Returns
    ///
    /// The distance as a floating-point value
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let pos1 = PxPosition::new(Px::new(0), Px::new(0));
    /// let pos2 = PxPosition::new(Px::new(3), Px::new(4));
    /// assert_eq!(pos1.distance_to(pos2), 5.0);
    /// ```
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = (self.x.0 - other.x.0) as f32;
        let dy = (self.y.0 - other.y.0) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Converts the position to a 2D f32 array.
    ///
    /// # Returns
    ///
    /// An array `[x, y]` where both coordinates are converted to f32
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// assert_eq!(position.to_f32_arr2(), [10.0, 20.0]);
    /// ```
    pub fn to_f32_arr2(self) -> [f32; 2] {
        [self.x.0 as f32, self.y.0 as f32]
    }

    /// Converts the position to a 3D f32 array with z=0.
    ///
    /// # Returns
    ///
    /// An array `[x, y, 0.0]` where x and y are converted to f32 and z is 0.0
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// assert_eq!(position.to_f32_arr3(), [10.0, 20.0, 0.0]);
    /// ```
    pub fn to_f32_arr3(self) -> [f32; 3] {
        [self.x.0 as f32, self.y.0 as f32, 0.0]
    }

    /// Creates a position from a 2D f32 array.
    ///
    /// # Arguments
    ///
    /// * `arr` - An array `[x, y]` where both values will be converted to i32
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::from_f32_arr2([10.5, 20.7]);
    /// assert_eq!(position, PxPosition::new(Px::new(10), Px::new(20)));
    /// ```
    pub fn from_f32_arr2(arr: [f32; 2]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Creates a position from a 3D f32 array, ignoring the z component.
    ///
    /// # Arguments
    ///
    /// * `arr` - An array `[x, y, z]` where only x and y are used
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::from_f32_arr3([10.5, 20.7, 30.9]);
    /// assert_eq!(position, PxPosition::new(Px::new(10), Px::new(20)));
    /// ```
    pub fn from_f32_arr3(arr: [f32; 3]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Converts the position to a 2D f64 array.
    ///
    /// # Returns
    ///
    /// An array `[x, y]` where both coordinates are converted to f64
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// assert_eq!(position.to_f64_arr2(), [10.0, 20.0]);
    /// ```
    pub fn to_f64_arr2(self) -> [f64; 2] {
        [self.x.0 as f64, self.y.0 as f64]
    }

    /// Converts the position to a 3D f64 array with z=0.
    ///
    /// # Returns
    ///
    /// An array `[x, y, 0.0]` where x and y are converted to f64 and z is 0.0
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::new(Px::new(10), Px::new(20));
    /// assert_eq!(position.to_f64_arr3(), [10.0, 20.0, 0.0]);
    /// ```
    pub fn to_f64_arr3(self) -> [f64; 3] {
        [self.x.0 as f64, self.y.0 as f64, 0.0]
    }

    /// Creates a position from a 2D f64 array.
    ///
    /// # Arguments
    ///
    /// * `arr` - An array `[x, y]` where both values will be converted to i32
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::from_f64_arr2([10.5, 20.7]);
    /// assert_eq!(position, PxPosition::new(Px::new(10), Px::new(20)));
    /// ```
    pub fn from_f64_arr2(arr: [f64; 2]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Creates a position from a 3D f64 array, ignoring the z component.
    ///
    /// # Arguments
    ///
    /// * `arr` - An array `[x, y, z]` where only x and y are used
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxPosition};
    ///
    /// let position = PxPosition::from_f64_arr3([10.5, 20.7, 30.9]);
    /// assert_eq!(position, PxPosition::new(Px::new(10), Px::new(20)));
    /// ```
    pub fn from_f64_arr3(arr: [f64; 3]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }
}

/// A 2D size in physical pixel space.
///
/// This type represents dimensions (width and height) in physical pixel space.
/// Physical pixels correspond directly to screen pixels and are used internally
/// by the rendering system.
///
/// # Examples
///
/// ```
/// use tessera_ui::px::{Px, PxSize};
///
/// // Create a size
/// let size = PxSize::new(Px::new(300), Px::new(200));
///
/// // Convert to array formats for graphics APIs
/// let f32_array = size.to_f32_arr2();
/// assert_eq!(f32_array, [300.0, 200.0]);
///
/// // Create from array
/// let from_array = PxSize::from([Px::new(400), Px::new(300)]);
/// ```
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
    ///
    /// # Arguments
    ///
    /// * `width` - The width in physical pixels
    /// * `height` - The height in physical pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxSize};
    ///
    /// let size = PxSize::new(Px::new(300), Px::new(200));
    /// assert_eq!(size.width, Px::new(300));
    /// assert_eq!(size.height, Px::new(200));
    /// ```
    pub const fn new(width: Px, height: Px) -> Self {
        Self { width, height }
    }

    /// Converts the size to a 2D f32 array.
    ///
    /// This is useful for interfacing with graphics APIs that expect
    /// floating-point size values.
    ///
    /// # Returns
    ///
    /// An array `[width, height]` where both dimensions are converted to f32
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxSize};
    ///
    /// let size = PxSize::new(Px::new(300), Px::new(200));
    /// assert_eq!(size.to_f32_arr2(), [300.0, 200.0]);
    /// ```
    pub fn to_f32_arr2(self) -> [f32; 2] {
        [self.width.0 as f32, self.height.0 as f32]
    }
}

/// A 2D rectangle in physical pixel space.
///
/// This type represents a rectangle with a position (top-left corner) and
/// dimensions in physical pixel space.
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
        Self {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Checks if this rectangle is orthogonal (non-overlapping) with another
    /// rectangle.
    ///
    /// Two rectangles are orthogonal if they do not overlap in either the x or
    /// y axis. This is useful for barrier batching optimization where
    /// non-overlapping rectangles can be processed together without
    /// requiring barriers.
    ///
    /// # Arguments
    ///
    /// * `other` - The other rectangle to check orthogonality against
    ///
    /// # Returns
    ///
    /// `true` if the rectangles are orthogonal (non-overlapping), `false`
    /// otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::px::{Px, PxRect};
    ///
    /// let rect1 = PxRect::new(Px::new(0), Px::new(0), Px::new(100), Px::new(100));
    /// let rect2 = PxRect::new(Px::new(150), Px::new(0), Px::new(100), Px::new(100));
    /// assert!(rect1.is_orthogonal(&rect2));
    ///
    /// let rect3 = PxRect::new(Px::new(50), Px::new(50), Px::new(100), Px::new(100));
    /// assert!(!rect1.is_orthogonal(&rect3));
    /// ```
    pub fn is_orthogonal(&self, other: &Self) -> bool {
        // Check if rectangles overlap on x-axis
        let x_overlap = self.x.0 < other.x.0 + other.width.0 && other.x.0 < self.x.0 + self.width.0;

        // Check if rectangles overlap on y-axis
        let y_overlap =
            self.y.0 < other.y.0 + other.height.0 && other.y.0 < self.y.0 + self.height.0;

        // Rectangles are orthogonal if they don't overlap on either axis
        !x_overlap || !y_overlap
    }

    /// Creates a new rectangle that is the union of this rectangle and another
    /// rectangle. Which is the smallest rectangle that contains both
    /// rectangles.
    ///
    /// # Arguments
    ///
    /// * `other` - The other rectangle to union with
    ///
    /// # Returns
    ///
    /// A new `PxRect` that is the union of this rectangle and the other
    /// rectangle
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

    /// Returns the area of this rectangle.
    ///
    /// # Returns
    ///
    /// The area as a positive integer, or 0 if width or height is negative
    pub fn area(&self) -> u32 {
        let width = self.width.0.max(0) as u32;
        let height = self.height.0.max(0) as u32;
        width * height
    }

    /// Gets the intersection of this rectangle with another rectangle.
    ///
    /// If the rectangles do not intersect, returns `None`.
    ///
    /// # Arguments
    ///
    /// * `other` - The other rectangle to intersect with
    ///
    /// # Returns
    ///
    /// An `Option<PxRect>` that is `Some` if the rectangles intersect,
    /// or `None` if they do not.
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

    /// Check if a point is inside the rectangle.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check
    ///
    /// # Returns
    ///
    /// An bool shows that whether the point is inside rectangle.
    pub fn contains(&self, point: PxPosition) -> bool {
        point.x.0 >= self.x.0
            && point.x.0 < self.x.0 + self.width.0
            && point.y.0 >= self.y.0
            && point.y.0 < self.y.0 + self.height.0
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

impl From<crate::component_tree::ComputedData> for PxSize {
    fn from(data: crate::component_tree::ComputedData) -> Self {
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
            x: pos.x.0,
            y: pos.y.0,
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

// Arithmetic operations support - PxPosition
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

// Type conversion implementations
impl From<[i32; 2]> for PxPosition {
    fn from(pos: [i32; 2]) -> Self {
        PxPosition {
            x: Px(pos[0]),
            y: Px(pos[1]),
        }
    }
}

impl From<PxPosition> for [i32; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x.0, pos.y.0]
    }
}

impl From<[u32; 2]> for PxPosition {
    fn from(pos: [u32; 2]) -> Self {
        PxPosition {
            x: Px(pos[0] as i32),
            y: Px(pos[1] as i32),
        }
    }
}

impl From<PxPosition> for [u32; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x.positive(), pos.y.positive()]
    }
}

impl From<[Px; 2]> for PxPosition {
    fn from(pos: [Px; 2]) -> Self {
        PxPosition {
            x: pos[0],
            y: pos[1],
        }
    }
}

impl From<PxPosition> for [Px; 2] {
    fn from(pos: PxPosition) -> Self {
        [pos.x, pos.y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_creation() {
        let px = Px::new(42);
        assert_eq!(px.0, 42);

        let px_neg = Px::new(-10);
        assert_eq!(px_neg.0, -10);
    }

    #[test]
    fn test_px_arithmetic() {
        let a = Px(10);
        let b = Px(5);

        assert_eq!(a + b, Px(15));
        assert_eq!(a - b, Px(5));
        assert_eq!(a * 2, Px(20));
        assert_eq!(a / 2, Px(5));
        assert_eq!(a * b, Px(50));
        assert_eq!(a / b, Px(2));
    }

    #[test]
    fn test_px_saturating_arithmetic() {
        let max = Px(i32::MAX);
        let min = Px(i32::MIN);
        assert_eq!(max.saturating_add(Px(1)), max);
        assert_eq!(min.saturating_sub(Px(1)), min);
    }

    #[test]
    fn test_saturating_from_f32() {
        assert_eq!(Px::saturating_from_f32(f32::MAX), Px(i32::MAX));
        assert_eq!(Px::saturating_from_f32(f32::MIN), Px(i32::MIN));
        assert_eq!(Px::saturating_from_f32(100.5), Px(100));
        assert_eq!(Px::saturating_from_f32(-100.5), Px(-100));
    }

    #[test]
    fn test_px_abs() {
        assert_eq!(Px(10).abs(), 10);
        assert_eq!(Px(-5).abs(), 5);
        assert_eq!(Px(0).abs(), 0);
    }

    #[test]
    fn test_px_position() {
        let pos = PxPosition::new(Px(10), Px(-5));
        assert_eq!(pos.x, Px(10));
        assert_eq!(pos.y, Px(-5));

        let offset_pos = pos.offset(Px(2), Px(3));
        assert_eq!(offset_pos, PxPosition::new(Px(12), Px(-2)));
    }

    #[test]
    fn test_px_position_arithmetic() {
        let pos1 = PxPosition::new(Px(10), Px(20));
        let pos2 = PxPosition::new(Px(5), Px(15));

        let sum = pos1 + pos2;
        assert_eq!(sum, PxPosition::new(Px(15), Px(35)));

        let diff = pos1 - pos2;
        assert_eq!(diff, PxPosition::new(Px(5), Px(5)));
    }

    #[test]
    fn test_px_position_conversions() {
        let i32_pos: [i32; 2] = [10, -5];
        let px_pos: PxPosition = i32_pos.into();
        let back_to_i32: [i32; 2] = px_pos.into();
        assert_eq!(i32_pos, back_to_i32);

        let u32_pos: [u32; 2] = [10, 5];
        let px_from_u32: PxPosition = u32_pos.into();
        let back_to_u32: [u32; 2] = px_from_u32.into();
        assert_eq!(u32_pos, back_to_u32);
    }

    #[test]
    fn test_distance() {
        let pos1 = PxPosition::new(Px(0), Px(0));
        let pos2 = PxPosition::new(Px(3), Px(4));
        assert_eq!(pos1.distance_to(pos2), 5.0);
    }
}

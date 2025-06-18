use std::ops::{AddAssign, Neg};

use crate::dp::{Dp, SCALE_FACTOR};

/// Physical pixel coordinate type, supports negative values for scrolling
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Px(pub i32);

impl Px {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(i32::MAX);

    /// Create a new Px instance
    pub const fn new(value: i32) -> Self {
        Px(value)
    }

    /// Convert from Dp to Px
    pub fn from_dp(dp: Dp) -> Self {
        Px(dp.to_pixels_f64() as i32)
    }

    /// Convert to Dp
    pub fn to_dp(self) -> Dp {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((self.0 as f64) / scale_factor)
    }

    /// Get absolute value (used for coordinate conversion during rendering)
    pub fn abs(self) -> u32 {
        self.0.max(0) as u32
    }

    /// Convert to f32
    pub fn to_f32(self) -> f32 {
        self.0 as f32
    }

    /// Create from f32
    pub fn from_f32(value: f32) -> Self {
        Px(value as i32)
    }
}

/// Physical pixel position type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PxPosition {
    pub x: Px,
    pub y: Px,
}

impl PxPosition {
    /// Create a zero position
    pub const ZERO: Self = Self { x: Px(0), y: Px(0) };

    /// Create a new position
    pub const fn new(x: Px, y: Px) -> Self {
        Self { x, y }
    }

    /// Offset the position
    pub fn offset(self, dx: Px, dy: Px) -> Self {
        Self {
            x: Px(self.x.0 + dx.0),
            y: Px(self.y.0 + dy.0),
        }
    }

    /// Calculate the distance to another point
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = (self.x.0 - other.x.0) as f32;
        let dy = (self.y.0 - other.y.0) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Convert to a f32 array (2D)
    pub fn to_f32_arr2(self) -> [f32; 2] {
        [self.x.0 as f32, self.y.0 as f32]
    }

    /// Convert to a f32 array (3D)
    pub fn to_f32_arr3(self) -> [f32; 3] {
        [self.x.0 as f32, self.y.0 as f32, 0.0]
    }

    /// Create from a f32 array (2D)
    pub fn from_f32_arr2(arr: [f32; 2]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Create from a f32 array (3D)
    /// Note: The third element will be ignored
    pub fn from_f32_arr3(arr: [f32; 3]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Convert to a f64 array (2D)
    pub fn to_f64_arr2(self) -> [f64; 2] {
        [self.x.0 as f64, self.y.0 as f64]
    }

    /// Convert to a f64 array (3D)
    pub fn to_f64_arr3(self) -> [f64; 3] {
        [self.x.0 as f64, self.y.0 as f64, 0.0]
    }

    /// Create from a f64 array (2D)
    pub fn from_f64_arr2(arr: [f64; 2]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
    }

    /// Create from a f64 array (3D)
    /// Note: The third element will be ignored
    pub fn from_f64_arr3(arr: [f64; 3]) -> Self {
        Self {
            x: Px::new(arr[0] as i32),
            y: Px::new(arr[1] as i32),
        }
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

impl AddAssign for Px {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
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
        [pos.x.abs(), pos.y.abs()]
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
    }

    #[test]
    fn test_px_abs() {
        assert_eq!(Px(10).abs(), 10);
        assert_eq!(Px(-5).abs(), 0);
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

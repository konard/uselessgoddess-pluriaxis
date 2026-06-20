//! A minimal 2D vector type.
//!
//! `spellvm` deliberately has no third-party dependencies, so it ships its own
//! tiny [`Vec2`] instead of pulling in `glam`. The host application (e.g. the
//! Bevy game) can convert to and from its own vector type at the boundary.

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// A 2D vector / point with `f32` components.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
  /// The horizontal component.
  pub x: f32,
  /// The vertical component.
  pub y: f32,
}

/// Shorthand constructor, mirroring `glam::vec2`.
#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
  Vec2::new(x, y)
}

impl Vec2 {
  /// The zero vector `(0, 0)`.
  pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
  /// The unit vector along the x axis `(1, 0)`.
  pub const X: Vec2 = Vec2::new(1.0, 0.0);
  /// The unit vector along the y axis `(0, 1)`.
  pub const Y: Vec2 = Vec2::new(0.0, 1.0);

  /// Creates a new vector from its components.
  #[inline]
  pub const fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }

  /// Creates a vector with both components set to `v`.
  #[inline]
  pub const fn splat(v: f32) -> Self {
    Self { x: v, y: v }
  }

  /// Returns the unit vector pointing at `angle` radians (counter-clockwise
  /// from the positive x axis).
  #[inline]
  pub fn from_angle(angle: f32) -> Self {
    Self::new(angle.cos(), angle.sin())
  }

  /// The squared length, avoiding the square root.
  #[inline]
  pub fn length_squared(self) -> f32 {
    self.x * self.x + self.y * self.y
  }

  /// The Euclidean length (magnitude).
  #[inline]
  pub fn length(self) -> f32 {
    self.length_squared().sqrt()
  }

  /// The distance between two points.
  #[inline]
  pub fn distance(self, other: Vec2) -> f32 {
    (self - other).length()
  }

  /// The dot product.
  #[inline]
  pub fn dot(self, other: Vec2) -> f32 {
    self.x * other.x + self.y * other.y
  }

  /// The vector rotated 90 degrees counter-clockwise.
  #[inline]
  pub fn perp(self) -> Self {
    Self::new(-self.y, self.x)
  }

  /// The angle of this vector in radians, in `(-pi, pi]`.
  #[inline]
  pub fn angle(self) -> f32 {
    self.y.atan2(self.x)
  }

  /// Returns this vector scaled to unit length, or [`Vec2::ZERO`] when the
  /// length is (near) zero. Never produces `NaN`.
  #[inline]
  pub fn normalize_or_zero(self) -> Self {
    let len = self.length();
    if len > f32::EPSILON { self / len } else { Self::ZERO }
  }

  /// Rotates this vector by `angle` radians counter-clockwise.
  #[inline]
  pub fn rotate(self, angle: f32) -> Self {
    let (sin, cos) = angle.sin_cos();
    Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
  }

  /// Linearly interpolates towards `other` by `t` (clamped to `0..=1`).
  #[inline]
  pub fn lerp(self, other: Vec2, t: f32) -> Self {
    let t = t.clamp(0.0, 1.0);
    self + (other - self) * t
  }

  /// The component-wise midpoint between two points.
  #[inline]
  pub fn midpoint(self, other: Vec2) -> Self {
    (self + other) * 0.5
  }

  /// Returns the components as an array, convenient for interop.
  #[inline]
  pub fn to_array(self) -> [f32; 2] {
    [self.x, self.y]
  }
}

impl Add for Vec2 {
  type Output = Vec2;
  #[inline]
  fn add(self, rhs: Vec2) -> Vec2 {
    Vec2::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl AddAssign for Vec2 {
  #[inline]
  fn add_assign(&mut self, rhs: Vec2) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl Sub for Vec2 {
  type Output = Vec2;
  #[inline]
  fn sub(self, rhs: Vec2) -> Vec2 {
    Vec2::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl SubAssign for Vec2 {
  #[inline]
  fn sub_assign(&mut self, rhs: Vec2) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

impl Mul<f32> for Vec2 {
  type Output = Vec2;
  #[inline]
  fn mul(self, rhs: f32) -> Vec2 {
    Vec2::new(self.x * rhs, self.y * rhs)
  }
}

impl Mul<Vec2> for f32 {
  type Output = Vec2;
  #[inline]
  fn mul(self, rhs: Vec2) -> Vec2 {
    rhs * self
  }
}

impl Div<f32> for Vec2 {
  type Output = Vec2;
  #[inline]
  fn div(self, rhs: f32) -> Vec2 {
    Vec2::new(self.x / rhs, self.y / rhs)
  }
}

impl Neg for Vec2 {
  type Output = Vec2;
  #[inline]
  fn neg(self) -> Vec2 {
    Vec2::new(-self.x, -self.y)
  }
}

impl From<[f32; 2]> for Vec2 {
  #[inline]
  fn from([x, y]: [f32; 2]) -> Self {
    Self::new(x, y)
  }
}

impl From<Vec2> for [f32; 2] {
  #[inline]
  fn from(v: Vec2) -> Self {
    [v.x, v.y]
  }
}

impl From<(f32, f32)> for Vec2 {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self {
    Self::new(x, y)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn length_and_normalize() {
    let v = Vec2::new(3.0, 4.0);
    assert_eq!(v.length(), 5.0);
    let n = v.normalize_or_zero();
    assert!((n.length() - 1.0).abs() < 1e-6);
    assert_eq!(Vec2::ZERO.normalize_or_zero(), Vec2::ZERO);
  }

  #[test]
  fn from_angle_is_unit() {
    for i in 0..8 {
      let a = i as f32 * core::f32::consts::FRAC_PI_4;
      assert!((Vec2::from_angle(a).length() - 1.0).abs() < 1e-6);
    }
  }

  #[test]
  fn rotate_preserves_length() {
    let v = Vec2::new(2.0, -1.0);
    let r = v.rotate(core::f32::consts::FRAC_PI_2);
    assert!((r.length() - v.length()).abs() < 1e-6);
    // 90 degree rotation matches `perp`.
    assert!((r - v.perp()).length() < 1e-5);
  }

  #[test]
  fn lerp_clamps() {
    let a = Vec2::ZERO;
    let b = Vec2::new(10.0, 0.0);
    assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 0.0));
    assert_eq!(a.lerp(b, -1.0), a);
    assert_eq!(a.lerp(b, 2.0), b);
  }
}

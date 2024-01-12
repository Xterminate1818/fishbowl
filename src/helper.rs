use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

#[derive(Copy, Clone)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
}

impl Vector2 {
  pub const fn new(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
  }

  pub fn length2(&self) -> f32 {
    (self.x * self.x) + (self.y * self.y)
  }
}

impl Add for Vector2 {
  type Output = Vector2;

  fn add(self, v: Vector2) -> Self {
    Vector2 {
      x: self.x + v.x,
      y: self.y + v.y,
    }
  }
}

impl Add<f32> for Vector2 {
  type Output = Vector2;

  fn add(self, value: f32) -> Self {
    Vector2 {
      x: self.x + value,
      y: self.y + value,
    }
  }
}

impl AddAssign for Vector2 {
  fn add_assign(&mut self, v: Vector2) {
    *self = *self + v;
  }
}

impl Sub for Vector2 {
  type Output = Vector2;

  fn sub(self, v: Vector2) -> Self {
    Vector2 {
      x: self.x - v.x,
      y: self.y - v.y,
    }
  }
}

impl SubAssign for Vector2 {
  fn sub_assign(&mut self, v: Vector2) {
    *self = *self - v;
  }
}

impl Mul<f32> for Vector2 {
  type Output = Vector2;

  fn mul(self, value: f32) -> Self {
    Vector2 {
      x: self.x * value,
      y: self.y * value,
    }
  }
}

impl MulAssign<f32> for Vector2 {
  fn mul_assign(&mut self, value: f32) {
    *self = *self * value;
  }
}

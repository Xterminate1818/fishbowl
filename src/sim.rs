use std::hash::{Hash, Hasher};

use crate::{helper::*, make_progress};
use image::{ImageBuffer, Rgb};

pub struct Circle {
  pub position: Vector2,
  last_position: Vector2,
  pub radius: f32,
  pub color: Color,
  index: usize,
}

pub struct Simulation {
  pub circles: Vec<Circle>,
  pub colors: Vec<Color>,
  pub max_circles: usize,
  pub clock: usize,
  pub substeps: usize,
  rand_seed: usize,
  timescale: f32,
  circle_radius: f32,
  radius_variance: f32,
  area_size: (f32, f32),
  gravity: f32,
  response_mod: f32,
}

impl Simulation {
  pub const POST_PROCESS: usize = 120;

  pub fn new(
    width: f32,
    height: f32,
    circle_radius: f32,
    rand_seed: usize,
  ) -> Self {
    let area = width * height;
    let circle_area = circle_radius.powi(2) * std::f32::consts::PI;
    let approx_max = ((area / circle_area).round() * 0.9) as usize;
    Self {
      circles: Vec::with_capacity(approx_max),
      max_circles: approx_max,
      timescale: 1.0 / 60.0,
      substeps: 8,
      colors: vec![Color(255, 255, 255); approx_max],
      clock: rand_seed,
      rand_seed,
      circle_radius,
      radius_variance: circle_radius * 0.1,
      area_size: (width, height),
      gravity: height,
      response_mod: 0.9,
    }
  }

  #[inline]
  async fn assign_colors_from_image(
    &mut self,
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
  ) {
    let (width, height) = (img.width() as f32 - 1.0, img.height() as f32 - 1.0);
    for (pos, index) in self.circles.iter().map(|c| (c.position, c.index)) {
      let img_x =
        ((pos.x / self.area_size.0).clamp(0.0, 1.0) * width).round() as u32;
      let img_y =
        ((pos.y / self.area_size.1).clamp(0.0, 1.0) * height).round() as u32;
      let pixel = img.get_pixel(img_x, img_y);
      let color = Color(pixel[0], pixel[1], pixel[2]);

      self.colors[index] = color;
    }
  }

  #[inline]
  pub async fn simulate_image(
    width: f32,
    height: f32,
    circle_radius: f32,
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
  ) -> (Self, usize, usize) {
    let image_hash = ({
      let mut s = std::hash::DefaultHasher::new();
      img.hash(&mut s);
      s.finish()
    } % 1204) as usize;
    let mut sim = Simulation::new(width, height, circle_radius, image_hash);
    let progress = make_progress(
      "Preprocessing",
      (sim.max_circles + Self::POST_PROCESS) as u64,
    );
    while sim.circles() < sim.max_circles {
      sim.step().await;
      progress.inc(2);
    }
    for _ in 0..Self::POST_PROCESS {
      sim.step().await;
      progress.inc(1);
    }
    progress.finish();
    let total_iterations = sim.clock;
    let max_circles = sim.circles.len();
    sim.assign_colors_from_image(img).await;
    sim.circles.clear();
    sim.clock = sim.rand_seed;
    (sim, total_iterations, max_circles)
  }

  #[inline]
  pub fn add_circle(&mut self, position: Vector2, velocity: Vector2) {
    self.circles.push(Circle {
      position,
      last_position: position - velocity,
      radius: self.circle_radius
        + (self.clock as f32).sin() * self.radius_variance,
      color: self.colors[self.circles.len()],
      index: self.circles.len(),
    })
  }

  #[inline]
  pub fn launch(&mut self) {
    let time = self.clock as f32 / (10.0 * std::f32::consts::PI);
    let halfwidth = self.area_size.0 / 2.0;
    let x = ((halfwidth - self.circle_radius * 2.0) * time.cos().abs())
      + self.circle_radius;
    self.add_circle(
      Vector2::new(x, self.circle_radius),
      Vector2::new(time.cos(), time.sin().abs()),
    )
  }

  pub fn launch2(&mut self) {
    let time = self.clock as f32 / (10.0 * std::f32::consts::PI);
    let halfwidth = self.area_size.0 / 2.0;
    let x = ((halfwidth - self.circle_radius * 2.0) * time.sin().abs())
      + self.circle_radius;
    let x = x + halfwidth;
    self.add_circle(
      Vector2::new(x, self.circle_radius),
      Vector2::new(time.cos(), time.sin().abs()),
    )
  }

  // Insertion sort
  #[inline]
  async fn sort(&mut self) {
    if self.circles.len() == 1 {
      return;
    }
    for i in 1..self.circles.len() {
      let mut j = i;
      while j > 0 && self.circles[j - 1].position.x > self.circles[j].position.x
      {
        self.circles.swap(j - 1, j);
        j -= 1;
      }
    }
  }

  #[inline]
  async fn integrate(&mut self) {
    let delta = self.timescale * (1.0 / self.substeps as f32);
    let gravity = Vector2::new(0.0, self.gravity) * delta.powi(2);
    self.circles.iter_mut().for_each(|circle| {
      let velocity = circle.position - circle.last_position;
      circle.last_position = circle.position;
      circle.position = circle.position + velocity + gravity;
    });
  }

  #[inline]
  async fn collide(&mut self) {
    for i in 0..self.circles.len() {
      // Apply gravity
      for j in i..self.circles.len() {
        let circle = &self.circles[i].position;
        let other = &self.circles[j].position;
        let this_radius = self.circles[i].radius;
        let other_radius = self.circles[i].radius;
        let diameter = this_radius + other_radius;
        if circle.x < other.x - diameter {
          break; // No further collisions possible
        }
        let dy = (circle.y - other.y).abs();
        if dy >= diameter {
          continue; // Skip over obvious noncollisions
        }
        let combined = *circle - *other;
        let distance_squared = combined.length2();
        if distance_squared >= diameter.powi(2) || distance_squared == 0.0 {
          continue;
        }
        // Finally, resort to expensive calculation
        let distance = distance_squared.sqrt();
        let normalized = combined * (1.0 / distance);
        let delta = 0.5 * self.response_mod * (distance - diameter);
        self.circles[i].position -= normalized * delta * 0.5;
        self.circles[j].position += normalized * delta * 0.5;
      }
    }
  }

  #[inline]
  fn constrain_rect(&mut self) {
    self
      .circles
      .iter_mut()
      .map(|c| &mut c.position)
      .for_each(|pos| {
        pos.x = pos
          .x
          .clamp(self.circle_radius, self.area_size.0 - self.circle_radius);
        pos.y = pos
          .y
          .clamp(self.circle_radius, self.area_size.1 - self.circle_radius);
      });
  }

  #[inline]
  pub async fn step(&mut self) {
    if self.circles.len() < self.max_circles {
      self.launch();
    }
    if self.circles.len() < self.max_circles {
      self.launch2();
    }

    for _ in 0..self.substeps {
      self.constrain_rect();
      self.sort().await;
      self.collide().await;
      self.integrate().await;
      self.clock += 1;
    }
  }

  #[inline]
  pub async fn steps(&mut self, steps: usize) {
    for _ in 0..steps {
      self.step().await;
    }
  }

  pub fn circles(&self) -> usize {
    self.circles.len()
  }
}

use std::io::Write;

use image::{ImageBuffer, Rgb};

mod drawing;
mod helper;
mod sim;

fn write_frame<W: Write>(
  sim: &sim::Simulation,
  encoder: &mut gif::Encoder<&mut W>,
) {
  let mut pixbuf = tiny_skia::Pixmap::new(500, 500).unwrap();
  drawing::clear(&mut pixbuf);
  for index in 0..sim.circles.len() {
    let circle = &sim.circles[index];
    drawing::draw_circle(&mut pixbuf, circle);
  }
  let mut frame = gif::Frame::from_rgba(500, 500, pixbuf.data_mut());
  frame.delay = 1;
  encoder.write_frame(&frame).unwrap();
}

async fn write_frame_async<W: Write>(
  sim: &sim::Simulation,
  encoder: &mut gif::Encoder<&mut W>,
) {
  write_frame(sim, encoder);
}

pub fn generate(image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<u8> {
  let (mut sim, it) = sim::Simulation::simulate_image(500.0, 500.0, 8.0, image);

  let mut buffer = Vec::<u8>::new();
  let mut encoder = gif::Encoder::new(&mut buffer, 500, 500, &[]).unwrap();
  // encoder.set_repeat(gif::Repeat::Infinite).unwrap();
  while sim.clock < it {
    write_frame(&sim, &mut encoder);
    (0..5).for_each(|_| sim.step());
  }
  drop(encoder);
  buffer
}

pub async fn generate_async(image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<u8> {
  let (mut sim, it) = sim::Simulation::simulate_image(500.0, 500.0, 8.0, image);

  let mut buffer = Vec::<u8>::new();
  let mut encoder = gif::Encoder::new(&mut buffer, 500, 500, &[]).unwrap();
  // encoder.set_repeat(gif::Repeat::Infinite).unwrap();
  while sim.clock < it {
    write_frame_async(&sim, &mut encoder).await;
    (0..5).for_each(|_| sim.step());
  }
  drop(encoder);
  buffer
}

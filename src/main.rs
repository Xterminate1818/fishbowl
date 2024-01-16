#![feature(future_join)]
use std::{cell::RefCell, future::join, io::Write, path::Path, sync::Arc};

use draw::QuickDraw;
use gif::Frame;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;

pub mod draw;
pub mod helper;
pub mod sim;
pub mod tests;

pub fn make_progress(msg: &'static str, max: u64) -> ProgressBar {
  let progress = ProgressBar::new(max);
  progress.set_style(ProgressStyle::with_template("{msg} :: {bar}").unwrap());
  progress.set_message(msg);
  progress
}

const WIDTH: f32 = 512.0;
const HEIGHT: f32 = 512.0;

pub async fn preprocess(
  image: ImageBuffer<Rgb<u8>, Vec<u8>>,
  radius: f32,
) -> (sim::Simulation, usize, usize) {
  let (sim, it, max_circles) =
    sim::Simulation::simulate_image(WIDTH, HEIGHT, radius, image).await;
  (sim, it, max_circles)
}

pub async fn simulate(
  draw: &mut QuickDraw,
  mut sim: Simulation,
  it: usize,
  step: usize,
  max_circles: usize,
) -> Vec<Frame<'static>> {
  draw.resize(WIDTH as u32, HEIGHT as u32, max_circles).await;
  let mut frames = vec![];
  let progress =
    make_progress("Simulating   ", ((it - sim.clock) / sim.substeps) as u64);
  while sim.clock < it {
    let circles = &sim
      .circles
      .iter()
      .map(|c| draw::Circle {
        position: [c.position.x, c.position.y],
        radius: c.radius,
        color: [c.color.0, c.color.1, c.color.2, 255],
      })
      .collect::<Vec<draw::Circle>>();
    let bytes_future = draw.draw_circles(circles);
    let steps = sim.steps(step);
    let mut bytes = join!(bytes_future, steps).await.0;
    let frame = gif::Frame::from_rgba(WIDTH as u16, HEIGHT as u16, &mut bytes);
    frames.push(frame);
    progress.inc(step as u64);
  }
  progress.finish();
  frames
}

pub async fn encode(frames: Vec<Frame<'static>>, repeat: bool) -> Vec<u8> {
  let mut buffer = Vec::<u8>::new();
  let mut encoder =
    gif::Encoder::new(&mut buffer, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
  let progress = make_progress("Encoding     ", frames.len() as u64);
  for mut frame in frames {
    frame.delay = 1;
    encoder.write_frame(&frame).unwrap();
    progress.inc(1);
  }
  progress.finish();
  if repeat {
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();
  }
  drop(encoder);
  buffer
}

use clap::Parser;
use sim::Simulation;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Input file to process. All common image types are
  /// supported, see the `image` crate docs for specific
  /// compatibility
  #[arg(short = 'i', value_hint = clap::ValueHint::DirPath)]
  input: std::path::PathBuf,

  /// Output file path ('./output.gif' by default)
  #[arg(short = 'o', value_hint = clap::ValueHint::DirPath)]
  output: Option<std::path::PathBuf>,

  /// How many physics-steps between frames. Affects the
  /// speed and length of the animation
  #[arg(short = 's')]
  step: Option<u32>,

  /// Radius of the circle, smaller means more detailed but
  /// slower to compute (8.0 by default, must be between
  /// 1.0 and 50.0 inclusive)
  #[arg(short = 'r')]
  radius: Option<f32>,

  /// Loop the GIF
  #[arg(short = 'l', long = "loop")]
  looping: bool,
}

fn open_image<P: AsRef<Path>>(path: P) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
  let image = match image::io::Reader::open(path) {
    Ok(i) => i,
    Err(e) => {
      eprintln!("Error opening file for processing: ");
      eprintln!("{}", e);
      std::process::exit(1);
    },
  };

  let decoded = match image.decode() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Error processing file: ");
      eprintln!("{}", e);
      std::process::exit(1);
    },
  };
  decoded.to_rgb8()
}

fn main() {
  let args = Args::parse();

  let output = args.output.unwrap_or("output.gif".into());
  let step = args.step.unwrap_or(20) as usize;
  if step == 0 {
    eprintln!("Physics step cannot be 0");
    std::process::exit(1);
  }
  let radius = args.radius.unwrap_or(8.0);
  if radius < 1.0 || radius > 50.0 {
    eprintln!("Invalid radius {}", radius);
    eprintln!("Must be between 1.0 and 50.0 inclusive");
    std::process::exit(1);
  }

  let image = open_image(args.input);
  let (sim, it, max) = pollster::block_on(preprocess(image, radius));
  let mut draw = pollster::block_on(QuickDraw::new(512, 512, 1000));
  let frames = pollster::block_on(simulate(&mut draw, sim, it, step, max));
  let gif = pollster::block_on(encode(frames, args.looping));
  let mut file = match std::fs::File::create(output.clone()) {
    Ok(f) => f,
    Err(e) => {
      eprintln!("Error opening file for writing:");
      eprintln!("{}", e);
      std::process::exit(1);
    },
  };

  match file.write(&gif) {
    Ok(_) => {
      println!("Successfully created file at '{}'", output.display());
    },
    Err(e) => {
      eprintln!("Error writing output file: ");
      eprintln!("{}", e);
      std::process::exit(1);
    },
  };
}

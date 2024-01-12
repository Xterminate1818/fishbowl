use tiny_skia::*;

use crate::sim::Circle;
pub fn clear(pixmap: &mut Pixmap) {
  let mut black = Paint::default();
  black.set_color_rgba8(0, 0, 0, 255);
  pixmap.fill_rect(
    Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
      .unwrap(),
    &black,
    Transform::identity(),
    None,
  );
}

pub fn draw_circle(pixmap: &mut Pixmap, circle: &Circle) {
  let x = circle.position.x;
  let y = circle.position.y;
  let radius = circle.radius;
  let r = circle.color.0;
  let g = circle.color.1;
  let b = circle.color.2;
  let mut paint = Paint::default();
  paint.set_color_rgba8(r, g, b, 255);
  let mut brush = PathBuilder::new();
  brush.push_circle(x, y, radius);
  brush.move_to(x, y);
  brush.close();

  pixmap.fill_path(
    &brush.finish().unwrap(),
    &paint,
    FillRule::Winding,
    tiny_skia::Transform::identity(),
    None,
  )
}

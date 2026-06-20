//! The playfield: a circular arena drawn procedurally, plus helpers to keep
//! things inside it.

use crate::prelude::*;

pub fn plugin(app: &mut App) {
  app.init_resource::<Arena>();
  app.add_systems(Update, draw_arena.run_if(in_state(Screen::Playing)));
}

/// The bounds of the playground, centred on the world origin.
#[derive(Resource, Clone, Copy)]
pub struct Arena {
  pub radius: f32,
}

impl Default for Arena {
  fn default() -> Self {
    Self { radius: 330.0 }
  }
}

impl Arena {
  /// Pull `p` back so it stays at least `margin` inside the boundary.
  pub fn clamp(&self, p: Vec2, margin: f32) -> Vec2 {
    let max = (self.radius - margin).max(0.0);
    if p.length() > max { p.normalize_or_zero() * max } else { p }
  }
}

fn draw_arena(mut gizmos: Gizmos, arena: Res<Arena>, time: Res<Time>) {
  let r = arena.radius;

  // A faint grid, chords clipped to the circle so it reads as a disc.
  let grid = Color::linear_rgb(0.04, 0.04, 0.11);
  let step = 80.0;
  let n = (r / step) as i32;
  for i in -n..=n {
    let x = i as f32 * step;
    let hx = (r * r - x * x).max(0.0).sqrt();
    gizmos.line_2d(vec2(x, -hx), vec2(x, hx), grid);
    let y = i as f32 * step;
    let hy = (r * r - y * y).max(0.0).sqrt();
    gizmos.line_2d(vec2(-hy, y), vec2(hy, y), grid);
  }

  // The boundary, gently breathing.
  let pulse = 1.0 + (time.elapsed_secs() * 0.6).sin() * 0.12;
  let edge = Color::linear_rgb(0.22 * pulse, 0.16 * pulse, 0.5 * pulse);
  gizmos.circle_2d(Isometry2d::from_translation(Vec2::ZERO), r, edge);
  gizmos.circle_2d(
    Isometry2d::from_translation(Vec2::ZERO),
    r - 3.0,
    edge.with_alpha(0.4),
  );
}

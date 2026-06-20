//! Training dummies arranged around the arena. They take damage, flash, die and
//! respawn so there is always something to practise on.

use crate::{game::arena::Arena, prelude::*};

pub fn plugin(app: &mut App) {
  app.add_systems(OnEnter(Screen::Playing), spawn_targets);
  app.add_systems(
    Update,
    (tick_targets, draw_targets).run_if(in_state(Screen::Playing)),
  );
}

#[derive(Component)]
pub struct Target {
  pub hp: f32,
  pub max: f32,
  /// Recent-hit glow, decaying to zero.
  pub flash: f32,
  respawn: Timer,
}

impl Target {
  fn new(max: f32) -> Self {
    let mut respawn = Timer::from_seconds(2.2, TimerMode::Once);
    respawn.pause();
    Self { hp: max, max, flash: 0.0, respawn }
  }

  pub fn alive(&self) -> bool {
    self.hp > 0.0
  }

  /// Apply `dmg`; start the respawn countdown if it kills the dummy.
  pub fn hit(&mut self, dmg: f32) {
    if !self.alive() {
      return;
    }
    self.hp -= dmg;
    self.flash = 1.0;
    if self.hp <= 0.0 {
      self.hp = 0.0;
      self.respawn.reset();
      self.respawn.unpause();
    }
  }
}

fn spawn_targets(mut commands: Commands, arena: Res<Arena>) {
  let ring = arena.radius * 0.56;
  let count = 6;
  for i in 0..count {
    let a = i as f32 / count as f32 * TAU;
    let p = vec2(a.cos(), a.sin()) * ring;
    commands.spawn((
      Name::new("Target"),
      Target::new(26.0),
      Transform::from_translation(p.extend(0.0)),
      DespawnOnExit(Screen::Playing),
    ));
  }
}

fn tick_targets(time: Res<Time>, mut q: Query<&mut Target>) {
  let dt = time.delta_secs();
  for mut t in &mut q {
    if t.flash > 0.0 {
      t.flash = (t.flash - dt * 3.0).max(0.0);
    }
    if !t.alive() {
      t.respawn.tick(time.delta());
      if t.respawn.is_finished() {
        t.hp = t.max;
        t.flash = 0.0;
        t.respawn.pause();
        t.respawn.reset();
      }
    }
  }
}

fn draw_targets(mut gizmos: Gizmos, q: Query<(&Transform, &Target)>) {
  for (tf, t) in &q {
    let p = tf.translation.truncate();
    if !t.alive() {
      // A dim ember marks where it will return.
      gizmos.circle_2d(
        Isometry2d::from_translation(p),
        5.0,
        Color::linear_rgb(0.25, 0.06, 0.06),
      );
      continue;
    }
    let frac = (t.hp / t.max).clamp(0.0, 1.0);
    let f = t.flash;
    let body = Color::linear_rgb(
      0.35 + (1.0 - frac) * 1.4 + f * 2.0,
      0.4 * frac + f,
      0.35 * frac,
    );
    gizmos.circle_2d(Isometry2d::from_translation(p), 16.0, body);
    // Health ring: a partial arc proportional to remaining hp.
    health_ring(&mut gizmos, p, 20.0, frac, f);
  }
}

fn health_ring(
  gizmos: &mut Gizmos,
  center: Vec2,
  radius: f32,
  frac: f32,
  flash: f32,
) {
  let segments = 28;
  let lit = (segments as f32 * frac).ceil() as i32;
  let col = Color::linear_rgb(0.3 + flash, 0.85, 0.4).with_alpha(0.85);
  let mut prev = None;
  for i in 0..=lit.min(segments) {
    let a = i as f32 / segments as f32 * TAU - FRAC_PI_2;
    let pt = center + vec2(a.cos(), a.sin()) * radius;
    if let Some(prev) = prev {
      gizmos.line_2d(prev, pt, col);
    }
    prev = Some(pt);
  }
}

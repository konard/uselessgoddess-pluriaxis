//! The bridge from spell semantics to the simulation: a [`SpellOutcome`] is
//! turned into [`Effect`] entities, each of which moves, collides, expires and
//! (optionally) triggers a nested outcome on impact.
//!
//! One generic component, [`Effect`], carries the whole [`SpellComponent`]; the
//! systems branch on `comp.form` so adding a new form is a localised change.

use crate::{
  core::motion::Velocity,
  game::{arena::Arena, player::Player, target::Target},
  prelude::*,
};

pub fn plugin(app: &mut App) {
  app.add_systems(
    Update,
    (
      age_effects,
      home_effects,
      bounce_effects,
      follow_wards,
      collide_effects,
      reap_effects,
    )
      .chain()
      .in_set(AppSystems::Update)
      .run_if(in_state(Screen::Playing))
      .run_if(in_state(Hud::Field)),
  );
}

/// A live spell effect in the world.
#[derive(Component)]
pub struct Effect {
  pub comp: SpellComponent,
  /// Facing/travel direction (also used to orient beams and walls).
  pub dir: Vec2,
  pub age: f32,
  pub life: f32,
  bounce_left: u8,
  /// Targets already damaged by this effect (so persistent effects hit each
  /// dummy once rather than every frame).
  hit: Vec<Entity>,
}

impl Effect {
  fn new(comp: SpellComponent, dir: Vec2, life: f32) -> Self {
    let bounce_left = comp.bounce;
    Self { comp, dir, age: 0.0, life, bounce_left, hit: Vec::new() }
  }

  /// How far through its life the effect is, in `0..=1`.
  pub fn progress(&self) -> f32 {
    (self.age / self.life.max(1e-3)).clamp(0.0, 1.0)
  }

  pub fn damage(&self) -> f32 {
    self.comp.power as f32 * 4.0
  }

  /// Solid body radius for the projectile forms.
  pub fn body_radius(&self) -> f32 {
    match self.comp.form {
      Form::Bolt => 7.0 + self.comp.power as f32 * 1.1,
      Form::Orb => 14.0 + self.comp.power as f32 * 2.0,
      _ => 0.0,
    }
  }

  /// The growing radius of a nova.
  pub fn nova_radius(&self) -> f32 {
    self.progress() * self.comp.range
  }

  /// The radius of a ward ring.
  pub fn ward_radius(&self) -> f32 {
    46.0 + self.comp.power as f32 * 3.0
  }

  pub fn beam_len(&self) -> f32 {
    self.comp.range
  }

  /// Half-length of a wall segment.
  pub fn wall_half(&self) -> f32 {
    (self.comp.range * 0.35).clamp(40.0, 220.0)
  }
}

/// Marks an effect for cleanup this frame; carries where it died so we can
/// spawn its on-impact trigger there.
#[derive(Component)]
struct Dying {
  at: Vec2,
}

/// A ward that tracks the caster's position.
#[derive(Component)]
struct WardFollow;

// --- spawning -------------------------------------------------------------

/// Spawn every component of an outcome from `origin`, headed `dir`.
pub fn cast_outcome(
  commands: &mut Commands,
  outcome: &SpellOutcome,
  origin: Vec2,
  dir: Vec2,
) {
  for comp in &outcome.components {
    spawn_component(commands, comp, origin, dir);
  }
}

fn spawn_component(
  commands: &mut Commands,
  comp: &SpellComponent,
  origin: Vec2,
  dir: Vec2,
) {
  let dir = dir.normalize_or_zero();
  let dir = if dir == Vec2::ZERO { Vec2::Y } else { dir };
  let n = comp.count.max(1) as usize;

  match comp.form {
    Form::Bolt => {
      let speed = 430.0 + comp.power as f32 * 14.0;
      let life = (comp.range / speed).clamp(0.15, 4.0);
      spawn_fan(commands, comp, origin, dir, n, speed, life);
    }
    Form::Orb => {
      let speed = 120.0;
      let life = (comp.duration_ms as f32 / 1000.0).max(0.4);
      spawn_fan(commands, comp, origin, dir, n, speed, life);
    }
    Form::Beam => {
      let life = 0.16 + comp.power as f32 * 0.012;
      spawn_fan(commands, comp, origin, dir, n.min(5), 0.0, life);
    }
    Form::Nova => {
      let life = (comp.duration_ms as f32 / 1000.0).clamp(0.3, 1.4);
      spawn_at(commands, comp, origin, dir, Vec2::ZERO, life);
    }
    Form::Ward => {
      let life = (comp.duration_ms as f32 / 1000.0).max(0.6);
      let e = spawn_at(commands, comp, origin, dir, Vec2::ZERO, life);
      commands.entity(e).insert(WardFollow);
    }
    Form::Wall => {
      let life = (comp.duration_ms as f32 / 1000.0).max(0.6);
      let pos = origin + dir * 70.0;
      spawn_at(commands, comp, pos, dir, Vec2::ZERO, life);
    }
  }
}

/// Spawn `n` copies fanned symmetrically around `dir`.
fn spawn_fan(
  commands: &mut Commands,
  comp: &SpellComponent,
  origin: Vec2,
  dir: Vec2,
  n: usize,
  speed: f32,
  life: f32,
) {
  let spread = 0.17;
  for i in 0..n {
    let off = (i as f32 - (n as f32 - 1.0) / 2.0) * spread;
    let d = Rot2::radians(off) * dir;
    spawn_at(commands, comp, origin, d, d * speed, life);
  }
}

fn spawn_at(
  commands: &mut Commands,
  comp: &SpellComponent,
  pos: Vec2,
  dir: Vec2,
  vel: Vec2,
  life: f32,
) -> Entity {
  commands
    .spawn((
      Name::new("Effect"),
      Effect::new(comp.clone(), dir, life),
      Velocity(vel),
      Transform::from_translation(pos.extend(0.0)),
      DespawnOnExit(Screen::Playing),
    ))
    .id()
}

// --- simulation -----------------------------------------------------------

fn age_effects(
  time: Res<Time>,
  mut commands: Commands,
  mut q: Query<(Entity, &mut Effect, &Transform)>,
) {
  let dt = time.delta_secs();
  for (e, mut fx, tf) in &mut q {
    fx.age += dt;
    if fx.age >= fx.life {
      commands.entity(e).insert(Dying { at: tf.translation.truncate() });
    }
  }
}

fn home_effects(
  time: Res<Time>,
  targets: Query<(&Transform, &Target)>,
  mut q: Query<(&mut Velocity, &Transform, &mut Effect)>,
) {
  let dt = time.delta_secs();
  for (mut vel, tf, mut fx) in &mut q {
    if !fx.comp.homing || vel.0 == Vec2::ZERO {
      continue;
    }
    let p = tf.translation.truncate();
    let mut best = None;
    let mut best_d = f32::MAX;
    for (ttf, t) in &targets {
      if !t.alive() {
        continue;
      }
      let d = ttf.translation.truncate().distance_squared(p);
      if d < best_d {
        best_d = d;
        best = Some(ttf.translation.truncate());
      }
    }
    let Some(goal) = best else { continue };
    let want = (goal - p).normalize_or_zero();
    let speed = vel.0.length();
    let steered = vel
      .0
      .normalize_or_zero()
      .lerp(want, (dt * 4.0).min(1.0))
      .normalize_or_zero();
    vel.0 = steered * speed;
    fx.dir = steered;
  }
}

fn bounce_effects(
  mut commands: Commands,
  arena: Res<Arena>,
  mut q: Query<(Entity, &mut Transform, &mut Velocity, &mut Effect)>,
) {
  for (e, mut tf, mut vel, mut fx) in &mut q {
    if vel.0 == Vec2::ZERO {
      continue; // stationary forms expire by age, not by the wall
    }
    let p = tf.translation.truncate();
    let r = fx.body_radius().max(2.0);
    if p.length() + r < arena.radius {
      continue;
    }
    if fx.bounce_left > 0 {
      let normal = p.normalize_or_zero();
      let v = vel.0;
      vel.0 = v - 2.0 * v.dot(normal) * normal;
      fx.dir = vel.0.normalize_or_zero();
      tf.translation = arena.clamp(p, r + 1.0).extend(tf.translation.z);
      fx.bounce_left -= 1;
    } else {
      commands.entity(e).insert(Dying { at: p });
    }
  }
}

fn follow_wards(
  players: Query<&Transform, (With<Player>, Without<Effect>)>,
  mut wards: Query<&mut Transform, With<WardFollow>>,
) {
  let Ok(ptf) = players.single() else { return };
  let p = ptf.translation.truncate();
  for mut tf in &mut wards {
    tf.translation = p.extend(tf.translation.z);
  }
}

fn collide_effects(
  mut commands: Commands,
  mut targets: Query<(Entity, &Transform, &mut Target)>,
  mut effects: Query<(Entity, &Transform, &mut Effect)>,
) {
  for (e, tf, mut fx) in &mut effects {
    let center = tf.translation.truncate();
    let dmg = fx.damage();
    let mut died = false;
    for (te, ttf, mut target) in &mut targets {
      if !target.alive() || fx.hit.contains(&te) {
        continue;
      }
      if !overlaps(&fx, center, ttf.translation.truncate()) {
        continue;
      }
      target.hit(dmg);
      fx.hit.push(te);
      // A plain bolt is consumed by the first thing it strikes.
      if matches!(fx.comp.form, Form::Bolt) && !fx.comp.pierce {
        died = true;
        break;
      }
    }
    if died {
      commands.entity(e).insert(Dying { at: center });
    }
  }
}

fn reap_effects(mut commands: Commands, q: Query<(Entity, &Effect, &Dying)>) {
  for (e, fx, dying) in &q {
    if let Some(trigger) = &fx.comp.on_impact {
      cast_outcome(&mut commands, trigger, dying.at, fx.dir);
    }
    commands.entity(e).despawn();
  }
}

// --- geometry -------------------------------------------------------------

/// Does effect `fx` (centred at `center`) touch a target at `tp`?
fn overlaps(fx: &Effect, center: Vec2, tp: Vec2) -> bool {
  const TR: f32 = 16.0; // target radius
  match fx.comp.form {
    Form::Bolt | Form::Orb => center.distance(tp) <= fx.body_radius() + TR,
    Form::Nova => {
      (center.distance(tp) - fx.nova_radius()).abs() <= 14.0 + TR * 0.5
    }
    Form::Ward => {
      (center.distance(tp) - fx.ward_radius()).abs() <= 14.0 + TR * 0.5
    }
    Form::Beam => {
      dist_to_segment(tp, center, center + fx.dir * fx.beam_len()) <= 12.0 + TR
    }
    Form::Wall => {
      let perp = vec2(-fx.dir.y, fx.dir.x);
      let h = fx.wall_half();
      dist_to_segment(tp, center - perp * h, center + perp * h) <= 12.0 + TR
    }
  }
}

fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
  let ab = b - a;
  let len2 = ab.length_squared();
  let t =
    if len2 <= 1e-6 { 0.0 } else { ((p - a).dot(ab) / len2).clamp(0.0, 1.0) };
  p.distance(a + ab * t)
}

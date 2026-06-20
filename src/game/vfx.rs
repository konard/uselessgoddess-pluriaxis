//! Procedural rendering for live effects, plus the sigil flash that blooms at
//! the caster on every cast. All visuals are gizmos coloured past `1.0` so the
//! HDR bloom makes them glow — no textures, no meshes.

use crate::{
  core::motion::Velocity, game::projectile::Effect, grimoire::render,
  prelude::*, theme::palette,
};

pub fn plugin(app: &mut App) {
  app.add_message::<CastFx>();
  app.add_systems(
    Update,
    (spawn_flash, age_flashes).run_if(in_state(Screen::Playing)),
  );
  app.add_systems(
    Update,
    (draw_effects, draw_flashes).run_if(in_state(Screen::Playing)),
  );
}

/// Emitted whenever a spell is cast, so the VFX layer can bloom its sigil.
#[derive(Message)]
pub struct CastFx {
  pub origin: Vec2,
  pub spell: Spell,
  pub dir: Vec2,
}

#[derive(Component)]
struct Flash {
  sigil: Sigil,
  origin: Vec2,
  /// Base orientation, taken from the cast direction so the sigil faces the aim.
  angle: f32,
  age: f32,
  life: f32,
  tint: Color,
}

fn spawn_flash(mut commands: Commands, mut ev: MessageReader<CastFx>) {
  for cast in ev.read() {
    let sigil = Sigil::encode(&cast.spell, Style::Circle);
    let tint = cast
      .spell
      .outcome()
      .ok()
      .and_then(|o| o.primary().map(|c| palette::glow(c.color(), 2.4)))
      .unwrap_or(palette::ACCENT);
    let angle = cast.dir.to_angle();
    commands.spawn((
      Name::new("CastFlash"),
      Flash { sigil, origin: cast.origin, angle, age: 0.0, life: 0.45, tint },
      DespawnOnExit(Screen::Playing),
    ));
  }
}

fn age_flashes(
  time: Res<Time>,
  mut commands: Commands,
  mut q: Query<(Entity, &mut Flash)>,
) {
  for (e, mut f) in &mut q {
    f.age += time.delta_secs();
    if f.age >= f.life {
      commands.entity(e).despawn();
    }
  }
}

fn draw_flashes(mut gizmos: Gizmos, q: Query<&Flash>) {
  for f in &q {
    let t = (f.age / f.life).clamp(0.0, 1.0);
    let radius = 34.0 + t * 30.0;
    let tint = f.tint.with_alpha(1.0 - t);
    render::draw_sigil(
      &mut gizmos,
      &f.sigil,
      f.origin,
      radius,
      f.angle + t * 2.0,
      tint,
      true,
    );
  }
}

fn draw_effects(
  mut gizmos: Gizmos,
  time: Res<Time>,
  q: Query<(&Transform, &Effect, &Velocity)>,
) {
  let now = time.elapsed_secs();
  for (tf, fx, vel) in &q {
    let p = tf.translation.truncate();
    let col = palette::glow(fx.comp.color(), 2.4);
    match fx.comp.form {
      Form::Bolt => {
        let tail = if vel.0 != Vec2::ZERO {
          p - vel.0.normalize_or_zero() * 16.0
        } else {
          p
        };
        gizmos.line_2d(tail, p, col);
        gizmos.circle_2d(
          Isometry2d::from_translation(p),
          fx.body_radius(),
          col,
        );
      }
      Form::Orb => {
        let r = fx.body_radius();
        gizmos.circle_2d(Isometry2d::from_translation(p), r, col);
        let inner = (r * (0.55 + 0.3 * (now * 6.0).sin())).max(1.0);
        gizmos.circle_2d(
          Isometry2d::from_translation(p),
          inner,
          col.with_alpha(0.6),
        );
      }
      Form::Nova => {
        let r = fx.nova_radius().max(1.0);
        let a = 1.0 - fx.progress();
        gizmos.circle_2d(Isometry2d::from_translation(p), r, col.with_alpha(a));
        gizmos.circle_2d(
          Isometry2d::from_translation(p),
          (r - 4.0).max(1.0),
          col.with_alpha(a * 0.5),
        );
      }
      Form::Beam => {
        let end = p + fx.dir * fx.beam_len();
        let a = 1.0 - fx.progress();
        gizmos.line_2d(p, end, col.with_alpha(a));
        gizmos.line_2d(
          p,
          end,
          Color::linear_rgb(2.6, 2.6, 2.9).with_alpha(a * 0.7),
        );
      }
      Form::Ward => {
        let r = fx.ward_radius() * (1.0 + 0.05 * (now * 8.0).sin());
        gizmos.circle_2d(
          Isometry2d::from_translation(p),
          r,
          col.with_alpha(0.85),
        );
        gizmos.circle_2d(
          Isometry2d::from_translation(p),
          r - 3.0,
          col.with_alpha(0.4),
        );
      }
      Form::Wall => {
        let perp = vec2(-fx.dir.y, fx.dir.x);
        let h = fx.wall_half();
        gizmos.line_2d(p - perp * h, p + perp * h, col);
        for k in -3..=3 {
          let o = perp * (h * k as f32 / 3.0);
          gizmos.line_2d(
            p + o - fx.dir * 4.0,
            p + o + fx.dir * 4.0,
            col.with_alpha(0.6),
          );
        }
      }
    }
  }
}

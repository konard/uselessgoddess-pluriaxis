//! The wizard you steer: movement, aiming at the cursor, and the cast cooldown.

use crate::{game::arena::Arena, prelude::*};

pub fn plugin(app: &mut App) {
  app.add_systems(OnEnter(Screen::Playing), spawn_player);
  app.add_systems(
    Update,
    (move_player, aim_player)
      .in_set(AppSystems::Update)
      .run_if(in_state(Screen::Playing))
      .run_if(in_state(Hud::Field)),
  );
  app.add_systems(Update, draw_player.run_if(in_state(Screen::Playing)));
}

#[derive(Component)]
pub struct Player;

/// The unit direction the player is aiming (toward the cursor).
#[derive(Component, Default)]
pub struct Aim(pub Vec2);

/// Cooldown gating how often the player may cast.
#[derive(Component)]
pub struct CastClock(pub Timer);

const SPEED: f32 = 270.0;

fn spawn_player(mut commands: Commands) {
  let mut clock = Timer::from_seconds(0.26, TimerMode::Once);
  let full = clock.duration();
  clock.set_elapsed(full); // ready to cast at once

  commands.spawn((
    Name::new("Player"),
    Player,
    Aim(Vec2::Y),
    CastClock(clock),
    Transform::default(),
    DespawnOnExit(Screen::Playing),
  ));
}

fn move_player(
  time: Res<Time>,
  keys: Res<ButtonInput<KeyCode>>,
  arena: Res<Arena>,
  mut q: Query<&mut Transform, With<Player>>,
) {
  let mut dir = Vec2::ZERO;
  if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
    dir.y += 1.0;
  }
  if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
    dir.y -= 1.0;
  }
  if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
    dir.x -= 1.0;
  }
  if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
    dir.x += 1.0;
  }
  let dir = dir.normalize_or_zero();
  for mut tf in &mut q {
    let p = tf.translation.truncate() + dir * SPEED * time.delta_secs();
    tf.translation = arena.clamp(p, 14.0).extend(0.0);
  }
}

fn aim_player(
  windows: Query<&Window, With<PrimaryWindow>>,
  cam: Query<(&Camera, &GlobalTransform)>,
  mut q: Query<(&Transform, &mut Aim), With<Player>>,
) {
  let Ok(window) = windows.single() else { return };
  let Ok((cam, cam_tf)) = cam.single() else { return };
  let Some(cursor) = window.cursor_position() else { return };
  let Ok(world) = cam.viewport_to_world_2d(cam_tf, cursor) else { return };
  for (tf, mut aim) in &mut q {
    let d = world - tf.translation.truncate();
    if d.length_squared() > 4.0 {
      aim.0 = d.normalize();
    }
  }
}

fn draw_player(
  mut gizmos: Gizmos,
  time: Res<Time>,
  q: Query<(&Transform, &Aim), With<Player>>,
) {
  for (tf, aim) in &q {
    let p = tf.translation.truncate();
    gizmos.circle_2d(
      Isometry2d::from_translation(p),
      13.0,
      Color::linear_rgb(0.5, 1.4, 2.6),
    );
    let pulse = 5.0 + (time.elapsed_secs() * 5.0).sin() * 1.5;
    gizmos.circle_2d(
      Isometry2d::from_translation(p),
      pulse,
      Color::linear_rgb(1.6, 2.2, 3.0),
    );
    // Wand tick pointing where we aim.
    gizmos.line_2d(
      p + aim.0 * 12.0,
      p + aim.0 * 24.0,
      Color::linear_rgb(2.6, 2.2, 0.8),
    );
  }
}

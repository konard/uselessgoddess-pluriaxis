//! A tiny shared kinematics layer: anything with a [`Velocity`] drifts.

use crate::prelude::*;

pub fn plugin(app: &mut App) {
  app.add_systems(
    Update,
    integrate
      .in_set(AppSystems::Update)
      .run_if(in_state(Screen::Playing))
      .run_if(in_state(Hud::Field)),
  );
}

/// Linear velocity in world units per second.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

fn integrate(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
  let dt = time.delta_secs();
  for (mut tf, v) in &mut q {
    tf.translation += (v.0 * dt).extend(0.0);
  }
}

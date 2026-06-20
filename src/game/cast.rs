//! Reading the fire button and turning the equipped spell into effects.

use crate::{
  game::{
    player::{Aim, CastClock, Player},
    projectile,
    vfx::CastFx,
  },
  grimoire::book::Book,
  prelude::*,
};

pub fn plugin(app: &mut App) {
  app.add_systems(
    Update,
    tick_clocks
      .in_set(AppSystems::TickTimers)
      .run_if(in_state(Screen::Playing)),
  );
  app.add_systems(
    Update,
    cast
      .in_set(AppSystems::Update)
      .run_if(in_state(Screen::Playing))
      .run_if(in_state(Hud::Field)),
  );
}

fn tick_clocks(time: Res<Time>, mut q: Query<&mut CastClock>) {
  for mut c in &mut q {
    c.0.tick(time.delta());
  }
}

fn cast(
  mut commands: Commands,
  mouse: Res<ButtonInput<MouseButton>>,
  keys: Res<ButtonInput<KeyCode>>,
  book: Res<Book>,
  mut players: Query<(&Transform, &Aim, &mut CastClock), With<Player>>,
  mut fx: MessageWriter<CastFx>,
) {
  let fire = mouse.pressed(MouseButton::Left) || keys.pressed(KeyCode::Space);
  if !fire {
    return;
  }
  for (tf, aim, mut clock) in &mut players {
    if !clock.0.is_finished() {
      continue;
    }
    let Some(spell) = book.equipped.get(book.selected) else { continue };
    let Ok(outcome) = spell.outcome() else { continue };
    if outcome.is_empty() {
      continue;
    }
    let origin = tf.translation.truncate();
    projectile::cast_outcome(&mut commands, &outcome, origin, aim.0);
    fx.write(CastFx { origin, spell: spell.clone(), dir: aim.0 });
    clock.0.reset();
  }
}

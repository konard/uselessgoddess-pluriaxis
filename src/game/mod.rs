//! The playground: the arena, the player, training targets, casting, the effect
//! simulation and its procedural rendering.

use crate::prelude::*;

pub mod arena;
mod cast;
pub mod player;
pub mod projectile;
pub mod target;
pub mod vfx;

pub fn plugin(app: &mut App) {
  app.add_plugins((
    arena::plugin,
    player::plugin,
    target::plugin,
    cast::plugin,
    projectile::plugin,
    vfx::plugin,
  ));
}

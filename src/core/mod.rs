//! Cross-cutting foundations: the camera, app states, system ordering and basic
//! kinematics that every other feature builds on.

use crate::prelude::*;

mod camera;
pub mod motion;
pub mod state;

pub use {motion::*, state::*};

pub fn plugin(app: &mut App) {
  app.add_plugins((state::plugin, camera::plugin, motion::plugin));
}

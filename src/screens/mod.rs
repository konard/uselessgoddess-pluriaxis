//! Top-level screens: the title menu and the playground frame.

use crate::prelude::*;

mod playing;
mod title;

pub fn plugin(app: &mut App) {
  app.add_plugins((title::plugin, playing::plugin));
}

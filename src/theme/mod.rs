//! Reusable UI building blocks and button feedback.

use crate::prelude::*;

mod interaction;
pub mod palette;
pub mod widget;

pub fn plugin(app: &mut App) {
  app.add_plugins(interaction::plugin);
}

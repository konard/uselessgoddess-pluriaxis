//! The spellbook feature: the [`Book`](book::Book) resource, the in-world editor
//! overlay, and the shared sigil renderer.

use crate::prelude::*;

pub mod book;
mod editor;
pub mod render;

pub fn plugin(app: &mut App) {
  app.add_plugins((book::plugin, editor::plugin));
}

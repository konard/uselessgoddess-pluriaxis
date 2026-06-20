//! A fat prelude: `use crate::prelude::*;` at the top of every module gives it
//! Bevy, the common std bits, and the `spellvm` types the game speaks in.
#![allow(unused_imports)]

pub use core::{f32::consts::*, fmt::Debug, hash::Hash, time::Duration};

pub use bevy::{
  color::palettes::{css, tailwind},
  math::{Isometry2d, Rot2, vec2, vec3},
  prelude::*,
  window::PrimaryWindow,
};

// `spellvm` is imported item-by-item on purpose: globbing it would clash with
// Bevy's `Vec2`, `vec2` and UI `Node`. Its geometry uses `spellvm::Vec2`, which
// we bridge into Bevy space with [`sv2`].
pub use spellvm::{
  Element, Form, Glyph, Modifier, Sigil, Spell, SpellComponent, SpellError,
  SpellOutcome, Style, sigil::Node as SigilNode,
};

pub use crate::core::*;

/// Convert a `spellvm` vector (used by sigils and runes) into a Bevy `Vec2`.
#[inline]
pub fn sv2(v: spellvm::Vec2) -> Vec2 {
  Vec2::new(v.x, v.y)
}

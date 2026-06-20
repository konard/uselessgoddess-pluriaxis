//! The game's colour language. Procedural effects use elemental colours straight
//! from [`spellvm`]; this palette is for the UI chrome around them.

use crate::prelude::*;

/// Panel background — deep, slightly translucent indigo.
pub const PANEL: Color = Color::srgba(0.06, 0.06, 0.12, 0.94);
/// A button at rest.
pub const BUTTON: Color = Color::srgb(0.12, 0.12, 0.20);
/// A hovered button.
pub const BUTTON_HOVER: Color = Color::srgb(0.20, 0.20, 0.34);
/// A pressed button.
pub const BUTTON_PRESSED: Color = Color::srgb(0.34, 0.30, 0.54);

/// Primary text.
pub const TEXT: Color = Color::srgb(0.92, 0.92, 0.97);
/// Secondary, dimmer text.
pub const TEXT_DIM: Color = Color::srgb(0.62, 0.62, 0.74);
/// The arcane accent — used for the title and highlights.
pub const ACCENT: Color = Color::srgb(0.74, 0.52, 1.00);

/// Bring an [`Rgb`](spellvm::Element) elemental colour into Bevy, scaled past
/// `1.0` so the HDR camera's bloom makes it glow.
pub fn glow(c: spellvm::Rgb, intensity: f32) -> Color {
  Color::linear_rgb(c.r * intensity, c.g * intensity, c.b * intensity)
}

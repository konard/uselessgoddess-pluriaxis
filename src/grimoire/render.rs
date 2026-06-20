//! Drawing a [`Sigil`] with gizmos — the one place that knows how to turn the
//! abstract points-and-lines notation into glowing geometry. Used by the title
//! backdrop, the grimoire editor and the cast flash.

use crate::{prelude::*, theme::palette};

/// Draw `sigil` centred at `center`, inscribed in a circle of `radius`, turned
/// by `rotation` radians. When `draw_runes` is set each node also shows its
/// glyph's procedural rune.
pub fn draw_sigil(
  gizmos: &mut Gizmos,
  sigil: &Sigil,
  center: Vec2,
  radius: f32,
  rotation: f32,
  tint: Color,
  draw_runes: bool,
) {
  let rot = Rot2::radians(rotation);
  let place = |p: spellvm::Vec2| center + rot * (sv2(p) * radius);

  // The outer enclosure the glyphs are inscribed in.
  let ring: Vec<Vec2> = sigil.enclosure(96).into_iter().map(place).collect();
  if ring.len() >= 2 {
    let mut closed = ring.clone();
    closed.push(ring[0]);
    gizmos.linestrip_2d(closed, tint.with_alpha(0.30));
  }

  // The path connecting glyph nodes, in casting order.
  let path: Vec<Vec2> = sigil.path().into_iter().map(place).collect();
  if path.len() >= 2 {
    gizmos.linestrip_2d(path, tint);
  }

  // Each node: a dot, optionally annotated with its rune.
  for node in &sigil.nodes {
    let at = place(node.position);
    let col = node
      .glyph
      .as_element()
      .map(|e| palette::glow(e.color(), 2.4))
      .unwrap_or_else(|| palette::glow(spellvm::Rgb::new(0.82, 0.6, 1.0), 2.0));
    gizmos.circle_2d(
      Isometry2d::from_translation(at),
      radius * 0.045 + 2.0,
      col,
    );
    if draw_runes {
      draw_rune(gizmos, node.glyph, at, radius * 0.16, rot, col);
    }
  }
}

fn draw_rune(
  gizmos: &mut Gizmos,
  glyph: Glyph,
  at: Vec2,
  scale: f32,
  rot: Rot2,
  col: Color,
) {
  for stroke in &glyph.rune().strokes {
    let pts: Vec<Vec2> =
      stroke.iter().map(|p| at + rot * (sv2(*p) * scale)).collect();
    if pts.len() >= 2 {
      gizmos.linestrip_2d(pts, col);
    }
  }
}

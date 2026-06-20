//! Sigils: the *geometry* of a spell.
//!
//! In the fiction, a wizard does not type a spell — they inscribe it as points
//! connected by lines inside a circle. A [`Sigil`] is exactly that: an ordered
//! set of [`Node`]s (a glyph placed at a point) laid out in one of several
//! [`Style`]s.
//!
//! The crucial property is that **the style is cosmetic**: a circle, a spiral, a
//! straight line and a master's four-quarter arrangement of the *same* spell all
//! [`decode`](Sigil::decode) back to the *same* [`Spell`]. The layout is the
//! wizard's handwriting; the spell is what it says.

use crate::glyph::Glyph;
use crate::math::Vec2;
use crate::spell::Spell;

/// The visual arrangement of a sigil's nodes — a wizard's "handwriting".
///
/// Every style preserves the glyph order, so they all decode identically; they
/// differ only in where the nodes sit and how they look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Style {
  /// Evenly spaced around a circle — the apprentice's first form.
  #[default]
  Circle,
  /// An outward spiral — the journeyman's flowing hand.
  Spiral,
  /// A straight left-to-right line — the researcher's terse notation.
  Line,
  /// Four arcs in four quadrants — the master's structured sigil.
  Quarters,
  /// Alternating upper/lower arcs — the adept's interleaved script.
  Semicircles,
  /// Deterministically scattered points — the dark wizard's cipher.
  Scattered,
}

impl Style {
  /// Every style, in canonical order.
  pub const ALL: [Style; 6] = [
    Style::Circle,
    Style::Spiral,
    Style::Line,
    Style::Quarters,
    Style::Semicircles,
    Style::Scattered,
  ];

  /// The human-readable name of the style.
  pub fn name(self) -> &'static str {
    match self {
      Style::Circle => "Circle",
      Style::Spiral => "Spiral",
      Style::Line => "Line",
      Style::Quarters => "Quarters",
      Style::Semicircles => "Semicircles",
      Style::Scattered => "Scattered",
    }
  }
}

impl core::fmt::Display for Style {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.name())
  }
}

/// A single inscribed point: a glyph at a position within the sigil.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
  /// The glyph drawn at this point.
  pub glyph: Glyph,
  /// The position within a unit disc of roughly radius `1`.
  pub position: Vec2,
}

/// A laid-out spell: ordered [`Node`]s with a [`Style`].
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sigil {
  /// The handwriting style this sigil was laid out in.
  pub style: Style,
  /// The inscribed nodes, in casting order.
  pub nodes: Vec<Node>,
}

impl Sigil {
  /// Lays a spell out as a sigil in the given style.
  ///
  /// All node positions land within a unit disc (radius `~1`), centred on the
  /// origin, ready to be scaled to any on-screen radius by the host.
  pub fn encode(spell: &Spell, style: Style) -> Sigil {
    let n = spell.glyphs.len();
    let nodes = spell
      .glyphs
      .iter()
      .enumerate()
      .map(|(i, &glyph)| Node { glyph, position: layout(style, i, n) })
      .collect();
    Sigil { style, nodes }
  }

  /// Reads the spell back out of the sigil.
  ///
  /// Decoding depends only on node *order*, never on geometry, so every style
  /// of the same spell decodes to the same result.
  pub fn decode(&self) -> Spell {
    self.nodes.iter().map(|node| node.glyph).collect()
  }

  /// The number of nodes.
  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// Whether the sigil has no nodes.
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  /// The polyline connecting node positions in casting order — the "lines"
  /// drawn between the points of the spell.
  pub fn path(&self) -> Vec<Vec2> {
    self.nodes.iter().map(|node| node.position).collect()
  }

  /// A closed polyline approximating the enclosing circle, for rendering the
  /// ring around the sigil. `segments` controls smoothness.
  pub fn enclosure(&self, segments: usize) -> Vec<Vec2> {
    use core::f32::consts::TAU;
    let segments = segments.max(3);
    (0..=segments)
      .map(|i| Vec2::from_angle(i as f32 * TAU / segments as f32))
      .collect()
  }
}

/// Computes the position of node `i` of `n` in the given style, in a unit disc.
fn layout(style: Style, i: usize, n: usize) -> Vec2 {
  use core::f32::consts::{FRAC_PI_2, PI, TAU};

  // Fraction in 0..=1 across the sequence, robust to n == 1.
  let frac = if n <= 1 { 0.5 } else { i as f32 / (n - 1) as f32 };

  match style {
    Style::Circle => {
      if n <= 1 {
        return Vec2::new(0.0, 0.85);
      }
      // Start at the top, proceed clockwise.
      let a = FRAC_PI_2 - i as f32 * TAU / n as f32;
      Vec2::from_angle(a) * 0.85
    }
    Style::Spiral => {
      let turns = 1.5;
      let a = FRAC_PI_2 + frac * turns * TAU;
      let r = 0.15 + frac * 0.75;
      Vec2::from_angle(a) * r
    }
    Style::Line => Vec2::new(-0.9 + frac * 1.8, 0.0),
    Style::Quarters => {
      // Cycle through the four quadrants, ringing outward.
      const CENTERS: [Vec2; 4] = [
        Vec2::new(0.42, 0.42),
        Vec2::new(-0.42, 0.42),
        Vec2::new(-0.42, -0.42),
        Vec2::new(0.42, -0.42),
      ];
      let q = i % 4;
      let ring = (i / 4) as f32;
      let a = FRAC_PI_2 + ring * (TAU / 5.0);
      CENTERS[q] + Vec2::from_angle(a) * 0.2
    }
    Style::Semicircles => {
      // Even nodes on the upper arc, odd nodes on the lower arc.
      let top = i.is_multiple_of(2);
      let rank = (i / 2) as f32;
      let count = if top { n.div_ceil(2) } else { n / 2 }.max(1) as f32;
      let f = if count <= 1.0 { 0.5 } else { rank / (count - 1.0) };
      let r = 0.85;
      if top {
        let a = PI - f * PI; // left -> right along the top
        Vec2::new(a.cos() * r, a.sin() * r * 0.9)
      } else {
        let a = PI - f * PI;
        Vec2::new(a.cos() * r, -a.sin() * r * 0.9)
      }
    }
    Style::Scattered => {
      // Deterministic "random" scatter from a hash of the index.
      let r = hash01(i as u32 * 2 + 1).sqrt() * 0.82;
      let a = hash01(i as u32 * 2 + 7_919) * TAU;
      Vec2::from_angle(a) * r
    }
  }
}

/// A deterministic hash of `seed` into `[0, 1)`, used to scatter cipher sigils
/// without any randomness (so layouts are reproducible).
fn hash01(seed: u32) -> f32 {
  let mut x = seed.wrapping_mul(2_654_435_761).wrapping_add(0x9E37_79B9);
  x ^= x >> 16;
  x = x.wrapping_mul(0x7feb_352d);
  x ^= x >> 15;
  (x & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample() -> Spell {
    Spell::parse("Fire Water Fuse Amplify Bolt Trigger Earth Nova").unwrap()
  }

  #[test]
  fn round_trips_in_every_style() {
    let spell = sample();
    for style in Style::ALL {
      let sigil = Sigil::encode(&spell, style);
      assert_eq!(sigil.decode(), spell, "style {style} broke the round trip");
    }
  }

  #[test]
  fn all_styles_decode_identically() {
    let spell = sample();
    let decoded: Vec<Spell> = Style::ALL
      .into_iter()
      .map(|style| Sigil::encode(&spell, style).decode())
      .collect();
    for d in &decoded {
      assert_eq!(d, &spell);
    }
  }

  #[test]
  fn positions_stay_within_the_unit_disc() {
    // Try several lengths to exercise the layout maths.
    for n in 1..=24 {
      let spell: Spell = std::iter::repeat_n(Glyph::Fire, n).collect();
      for style in Style::ALL {
        for node in Sigil::encode(&spell, style).nodes {
          let len = node.position.length();
          assert!(
            len <= 1.001,
            "style {style} node escaped the disc: len {len}"
          );
          assert!(node.position.x.is_finite() && node.position.y.is_finite());
        }
      }
    }
  }

  #[test]
  fn path_has_one_point_per_node() {
    let sigil = Sigil::encode(&sample(), Style::Spiral);
    assert_eq!(sigil.path().len(), sigil.nodes.len());
  }

  #[test]
  fn empty_spell_makes_empty_sigil() {
    let sigil = Sigil::encode(&Spell::default(), Style::Circle);
    assert!(sigil.is_empty());
    assert!(sigil.decode().is_empty());
  }

  #[test]
  fn outcome_is_independent_of_style() {
    let spell = sample();
    let want = spell.outcome().unwrap();
    for style in Style::ALL {
      let got = Sigil::encode(&spell, style).decode().outcome().unwrap();
      assert_eq!(got, want);
    }
  }
}

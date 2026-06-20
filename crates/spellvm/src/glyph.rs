//! Glyphs: the instruction set of the spell VM.
//!
//! A glyph is a single rune a wizard can draw. Glyphs are the *opcodes* of the
//! [`Vm`](crate::vm::Vm): a spell is just an ordered sequence of them. Every
//! glyph also carries a [procedurally generated rune](Glyph::rune) so the host
//! can render it as line art without any texture assets.

use crate::math::Vec2;
use crate::outcome::{Form, Modifier};

/// The broad role a glyph plays in a spell program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Category {
  /// Pushes an element onto the stack.
  Element,
  /// Combines values already on the stack (e.g. `Fuse`).
  Operator,
  /// Finalises a spell component with a delivery shape.
  Form,
  /// Tunes the spell currently being built.
  Modifier,
  /// Alters control flow (e.g. `Trigger` for nested spells).
  Control,
}

/// A single rune — one opcode of the spell VM.
///
/// The numeric `repr` is stable and used both as a compact id and as the seed
/// for the glyph's [procedural rune](Glyph::rune).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum Glyph {
  // --- Elements ---
  /// Push [`Element::Fire`](crate::element::Element::Fire).
  Fire,
  /// Push [`Element::Water`](crate::element::Element::Water).
  Water,
  /// Push [`Element::Earth`](crate::element::Element::Earth).
  Earth,
  /// Push [`Element::Air`](crate::element::Element::Air).
  Air,
  /// Push [`Element::Aether`](crate::element::Element::Aether).
  Aether,

  // --- Operator ---
  /// Fuse the top two elements into one (Magicka-style mixing).
  Fuse,

  // --- Forms ---
  /// Finalise a [`Form::Bolt`].
  Bolt,
  /// Finalise a [`Form::Beam`].
  Beam,
  /// Finalise a [`Form::Nova`].
  Nova,
  /// Finalise a [`Form::Orb`].
  Orb,
  /// Finalise a [`Form::Ward`].
  Ward,
  /// Finalise a [`Form::Wall`].
  Wall,

  // --- Modifiers ---
  /// Apply [`Modifier::Amplify`].
  Amplify,
  /// Apply [`Modifier::Split`].
  Split,
  /// Apply [`Modifier::Reach`].
  Reach,
  /// Apply [`Modifier::Linger`].
  Linger,
  /// Apply [`Modifier::Pierce`].
  Pierce,
  /// Apply [`Modifier::Seek`].
  Seek,
  /// Apply [`Modifier::Ricochet`].
  Ricochet,

  // --- Control ---
  /// Begin a nested spell that fires when the next form impacts.
  Trigger,
}

impl Glyph {
  /// Every glyph, in canonical (declaration) order.
  pub const ALL: [Glyph; 20] = [
    Glyph::Fire,
    Glyph::Water,
    Glyph::Earth,
    Glyph::Air,
    Glyph::Aether,
    Glyph::Fuse,
    Glyph::Bolt,
    Glyph::Beam,
    Glyph::Nova,
    Glyph::Orb,
    Glyph::Ward,
    Glyph::Wall,
    Glyph::Amplify,
    Glyph::Split,
    Glyph::Reach,
    Glyph::Linger,
    Glyph::Pierce,
    Glyph::Seek,
    Glyph::Ricochet,
    Glyph::Trigger,
  ];

  /// The category (role) of this glyph.
  pub fn category(self) -> Category {
    use Glyph::*;
    match self {
      Fire | Water | Earth | Air | Aether => Category::Element,
      Fuse => Category::Operator,
      Bolt | Beam | Nova | Orb | Ward | Wall => Category::Form,
      Amplify | Split | Reach | Linger | Pierce | Seek | Ricochet => {
        Category::Modifier
      }
      Trigger => Category::Control,
    }
  }

  /// If this glyph pushes an element, the element it pushes.
  pub fn as_element(self) -> Option<crate::element::Element> {
    use crate::element::Element;
    Some(match self {
      Glyph::Fire => Element::Fire,
      Glyph::Water => Element::Water,
      Glyph::Earth => Element::Earth,
      Glyph::Air => Element::Air,
      Glyph::Aether => Element::Aether,
      _ => return None,
    })
  }

  /// If this glyph finalises a form, the form it produces.
  pub fn as_form(self) -> Option<Form> {
    Some(match self {
      Glyph::Bolt => Form::Bolt,
      Glyph::Beam => Form::Beam,
      Glyph::Nova => Form::Nova,
      Glyph::Orb => Form::Orb,
      Glyph::Ward => Form::Ward,
      Glyph::Wall => Form::Wall,
      _ => return None,
    })
  }

  /// If this glyph is a modifier, the modifier it applies.
  pub fn as_modifier(self) -> Option<Modifier> {
    Some(match self {
      Glyph::Amplify => Modifier::Amplify,
      Glyph::Split => Modifier::Split,
      Glyph::Reach => Modifier::Reach,
      Glyph::Linger => Modifier::Linger,
      Glyph::Pierce => Modifier::Pierce,
      Glyph::Seek => Modifier::Seek,
      Glyph::Ricochet => Modifier::Ricochet,
      _ => return None,
    })
  }

  /// The human-readable, capitalised name of the glyph (e.g. `"Fire"`).
  ///
  /// This is also the token accepted by [`Glyph::from_token`].
  pub fn name(self) -> &'static str {
    use Glyph::*;
    match self {
      Fire => "Fire",
      Water => "Water",
      Earth => "Earth",
      Air => "Air",
      Aether => "Aether",
      Fuse => "Fuse",
      Bolt => "Bolt",
      Beam => "Beam",
      Nova => "Nova",
      Orb => "Orb",
      Ward => "Ward",
      Wall => "Wall",
      Amplify => "Amplify",
      Split => "Split",
      Reach => "Reach",
      Linger => "Linger",
      Pierce => "Pierce",
      Seek => "Seek",
      Ricochet => "Ricochet",
      Trigger => "Trigger",
    }
  }

  /// Parses a glyph from its name, case-insensitively (e.g. `"fire"`).
  ///
  /// ```
  /// use spellvm::Glyph;
  /// assert_eq!(Glyph::from_token("fire"), Some(Glyph::Fire));
  /// assert_eq!(Glyph::from_token("BOLT"), Some(Glyph::Bolt));
  /// assert_eq!(Glyph::from_token("nope"), None);
  /// ```
  pub fn from_token(token: &str) -> Option<Glyph> {
    Glyph::ALL.into_iter().find(|g| g.name().eq_ignore_ascii_case(token.trim()))
  }

  /// A short, procedurally generated rune for this glyph.
  ///
  /// The geometry lives in a unit cell of roughly `[-1, 1]^2` and is made of
  /// one or more polyline strokes the host can draw with gizmos or a line
  /// mesh — no textures required. The shape is deterministic: the same glyph
  /// always produces the same rune.
  pub fn rune(self) -> Rune {
    Rune::generate(self)
  }
}

impl core::fmt::Display for Glyph {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.name())
  }
}

/// Procedurally generated line art for a [`Glyph`].
///
/// Each [stroke](Rune::strokes) is an open polyline (a connected run of points)
/// to be drawn as a line strip. Coordinates are normalised to about `[-1, 1]`.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rune {
  /// The polyline strokes that make up the rune.
  pub strokes: Vec<Vec<Vec2>>,
}

impl Rune {
  /// Builds the rune for `glyph` from its category and stable id.
  fn generate(glyph: Glyph) -> Rune {
    use core::f32::consts::{FRAC_PI_2, TAU};

    let id = glyph as u32;
    let mut strokes: Vec<Vec<Vec2>> = Vec::new();

    // --- Body: a star polygon whose shape is seeded by the glyph id. ---
    let n = 3 + (id % 5) as usize; // 3..=7 vertices
    let step = 1 + (id % (n as u32 - 1)) as usize; // 1..=n-1
    let radius = 0.92;
    // Start pointing up so runes feel upright and consistent.
    let phase = FRAC_PI_2;
    let mut body = Vec::with_capacity(n + 1);
    let mut idx = 0usize;
    for _ in 0..=n {
      let a = phase + idx as f32 * TAU / n as f32;
      body.push(Vec2::from_angle(a) * radius);
      idx = (idx + step) % n;
    }
    strokes.push(body);

    // --- Inner mark: a flourish that distinguishes the category. ---
    match glyph.category() {
      Category::Element => strokes.extend(Self::element_mark(id)),
      Category::Operator => strokes.extend(Self::operator_mark()),
      Category::Form => strokes.extend(Self::form_mark()),
      Category::Modifier => strokes.extend(Self::modifier_mark(id)),
      Category::Control => strokes.extend(Self::control_mark()),
    }

    Rune { strokes }
  }

  /// A small diamond plus radial pips — one pip per element index.
  fn element_mark(id: u32) -> Vec<Vec<Vec2>> {
    use core::f32::consts::{FRAC_PI_2, TAU};
    let mut out = Vec::new();

    // Central diamond.
    let d = 0.22;
    out.push(vec![
      Vec2::new(0.0, d),
      Vec2::new(d, 0.0),
      Vec2::new(0.0, -d),
      Vec2::new(-d, 0.0),
      Vec2::new(0.0, d),
    ]);

    // `id + 1` short radial pips evenly spread around the diamond.
    let pips = id + 1;
    for k in 0..pips {
      let a = FRAC_PI_2 + k as f32 * TAU / pips as f32;
      let dir = Vec2::from_angle(a);
      out.push(vec![dir * 0.34, dir * 0.5]);
    }
    out
  }

  /// An "X": two crossing strokes, for the fusion operator.
  fn operator_mark() -> Vec<Vec<Vec2>> {
    let r = 0.3;
    vec![
      vec![Vec2::new(-r, -r), Vec2::new(r, r)],
      vec![Vec2::new(-r, r), Vec2::new(r, -r)],
    ]
  }

  /// An upward arrow, for a delivery form.
  fn form_mark() -> Vec<Vec<Vec2>> {
    vec![vec![
      Vec2::new(0.0, -0.4),
      Vec2::new(0.0, 0.45),
      Vec2::new(-0.18, 0.2),
      Vec2::new(0.0, 0.45),
      Vec2::new(0.18, 0.2),
    ]]
  }

  /// A stack of chevrons — one per modifier index — pointing right.
  fn modifier_mark(id: u32) -> Vec<Vec<Vec2>> {
    // Modifiers start at id 12; make a 1-based count for visual variety.
    let rank = id.saturating_sub(11).clamp(1, 4);
    let mut out = Vec::new();
    for k in 0..rank {
      let y = -0.18 + k as f32 * 0.18;
      out.push(vec![
        Vec2::new(-0.22, y - 0.1),
        Vec2::new(0.0, y),
        Vec2::new(-0.22, y + 0.1),
      ]);
    }
    out
  }

  /// A small spiral, for the trigger / control glyph.
  fn control_mark() -> Vec<Vec<Vec2>> {
    use core::f32::consts::TAU;
    let turns = 2.0;
    let steps = 28;
    let mut spiral = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
      let t = i as f32 / steps as f32;
      let a = t * turns * TAU;
      let r = 0.05 + t * 0.4;
      spiral.push(Vec2::from_angle(a) * r);
    }
    vec![spiral]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_is_complete_and_unique() {
    assert_eq!(Glyph::ALL.len(), 20);
    let mut seen = std::collections::HashSet::new();
    for g in Glyph::ALL {
      assert!(seen.insert(g), "duplicate glyph {g}");
    }
  }

  #[test]
  fn token_round_trip() {
    for g in Glyph::ALL {
      assert_eq!(Glyph::from_token(g.name()), Some(g));
      assert_eq!(Glyph::from_token(&g.name().to_lowercase()), Some(g));
    }
    assert_eq!(Glyph::from_token("  Fire  "), Some(Glyph::Fire));
    assert_eq!(Glyph::from_token("definitely-not-a-glyph"), None);
  }

  #[test]
  fn category_matches_accessors() {
    for g in Glyph::ALL {
      match g.category() {
        Category::Element => assert!(g.as_element().is_some()),
        Category::Form => assert!(g.as_form().is_some()),
        Category::Modifier => assert!(g.as_modifier().is_some()),
        Category::Operator | Category::Control => {
          assert!(g.as_element().is_none());
          assert!(g.as_form().is_none());
          assert!(g.as_modifier().is_none());
        }
      }
    }
  }

  #[test]
  fn runes_are_non_empty_and_bounded() {
    for g in Glyph::ALL {
      let rune = g.rune();
      assert!(!rune.strokes.is_empty(), "{g} has no strokes");
      for stroke in &rune.strokes {
        assert!(stroke.len() >= 2, "{g} has a degenerate stroke");
        for p in stroke {
          assert!(
            p.x.abs() <= 1.01 && p.y.abs() <= 1.01,
            "{g} rune point {p:?} escapes the unit cell"
          );
          assert!(p.x.is_finite() && p.y.is_finite());
        }
      }
    }
  }
}

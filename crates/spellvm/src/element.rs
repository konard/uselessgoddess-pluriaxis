//! Elements: the raw substances a spell is made of, and how they fuse.
//!
//! There are five **base** elements that glyphs can push directly, and a set of
//! **compound** elements that only ever arise from [fusing](Element::fuse) two
//! different bases. Fusion is total and commutative, so any two elements always
//! produce a well-defined result — discovering the interesting combinations is
//! part of the gameplay (think Magicka).

/// An sRGB colour with components in `0.0..=1.0`.
///
/// The VM assigns every element a canonical colour so the host can render
/// elemental effects procedurally without any texture assets. This is *data*,
/// not rendering: the game maps it onto its own colour type.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rgb {
  /// Red channel, `0.0..=1.0`.
  pub r: f32,
  /// Green channel, `0.0..=1.0`.
  pub g: f32,
  /// Blue channel, `0.0..=1.0`.
  pub b: f32,
}

impl Rgb {
  /// Creates a colour from its channels.
  pub const fn new(r: f32, g: f32, b: f32) -> Self {
    Self { r, g, b }
  }

  /// Returns the channels as an array, convenient for interop.
  pub const fn to_array(self) -> [f32; 3] {
    [self.r, self.g, self.b]
  }
}

/// An element. Either a base substance or a compound produced by fusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Element {
  // --- Base elements (can be pushed directly by a glyph) ---
  /// Fire — aggressive, high damage.
  Fire,
  /// Water — control and cooling.
  Water,
  /// Earth — mass and defence.
  Earth,
  /// Air — speed and reach.
  Air,
  /// Aether — raw, neutral mana. The "default" payload of an empty form.
  Aether,

  // --- Compound elements (only produced by `fuse`) ---
  /// Fire + Water.
  Steam,
  /// Fire + Earth.
  Lava,
  /// Fire + Air.
  Lightning,
  /// Fire + Aether.
  Plasma,
  /// Water + Earth.
  Mud,
  /// Water + Air.
  Ice,
  /// Water + Aether.
  Mist,
  /// Earth + Air.
  Sand,
  /// Earth + Aether.
  Crystal,
  /// Air + Aether.
  Storm,
  /// Any other, less stable mixture (e.g. fusing two compounds).
  Arcane,
}

impl Element {
  /// The five base elements, in canonical order.
  pub const BASES: [Element; 5] = [
    Element::Fire,
    Element::Water,
    Element::Earth,
    Element::Air,
    Element::Aether,
  ];

  /// All elements, bases first then compounds.
  pub const ALL: [Element; 16] = [
    Element::Fire,
    Element::Water,
    Element::Earth,
    Element::Air,
    Element::Aether,
    Element::Steam,
    Element::Lava,
    Element::Lightning,
    Element::Plasma,
    Element::Mud,
    Element::Ice,
    Element::Mist,
    Element::Sand,
    Element::Crystal,
    Element::Storm,
    Element::Arcane,
  ];

  /// Whether this is one of the five base elements.
  pub fn is_base(self) -> bool {
    Self::BASES.contains(&self)
  }

  /// Whether this element is a compound (the result of a fusion).
  pub fn is_compound(self) -> bool {
    !self.is_base()
  }

  /// Fuses two elements into one.
  ///
  /// Fusion is **commutative** (`a.fuse(b) == b.fuse(a)`) and **total** (it is
  /// defined for every pair):
  ///
  /// * fusing an element with itself yields the same element (concentration is
  ///   tracked separately by the VM as extra power);
  /// * the ten distinct base pairs each map to a signature compound;
  /// * anything else (any pair involving a compound) collapses to
  ///   [`Element::Arcane`], an unstable catch-all.
  ///
  /// ```
  /// use spellvm::Element;
  /// assert_eq!(Element::Fire.fuse(Element::Water), Element::Steam);
  /// assert_eq!(Element::Water.fuse(Element::Fire), Element::Steam); // commutative
  /// assert_eq!(Element::Fire.fuse(Element::Fire), Element::Fire);
  /// assert_eq!(Element::Steam.fuse(Element::Lava), Element::Arcane);
  /// ```
  pub fn fuse(self, other: Element) -> Element {
    use Element::*;

    if self == other {
      return self;
    }

    // Order the pair canonically by declaration order so the match only
    // needs to list each base pair once.
    let (a, b) =
      if (self as u8) <= (other as u8) { (self, other) } else { (other, self) };

    match (a, b) {
      (Fire, Water) => Steam,
      (Fire, Earth) => Lava,
      (Fire, Air) => Lightning,
      (Fire, Aether) => Plasma,
      (Water, Earth) => Mud,
      (Water, Air) => Ice,
      (Water, Aether) => Mist,
      (Earth, Air) => Sand,
      (Earth, Aether) => Crystal,
      (Air, Aether) => Storm,
      // At least one operand is a compound: the mixture is unstable.
      _ => Arcane,
    }
  }

  /// The human-readable display name of the element.
  pub fn name(self) -> &'static str {
    use Element::*;
    match self {
      Fire => "Fire",
      Water => "Water",
      Earth => "Earth",
      Air => "Air",
      Aether => "Aether",
      Steam => "Steam",
      Lava => "Lava",
      Lightning => "Lightning",
      Plasma => "Plasma",
      Mud => "Mud",
      Ice => "Ice",
      Mist => "Mist",
      Sand => "Sand",
      Crystal => "Crystal",
      Storm => "Storm",
      Arcane => "Arcane",
    }
  }

  /// The canonical colour identity of this element, for procedural rendering.
  pub fn color(self) -> Rgb {
    use Element::*;
    match self {
      Fire => Rgb::new(1.00, 0.35, 0.10),
      Water => Rgb::new(0.20, 0.50, 1.00),
      Earth => Rgb::new(0.55, 0.40, 0.20),
      Air => Rgb::new(0.80, 0.92, 1.00),
      Aether => Rgb::new(0.72, 0.42, 1.00),
      Steam => Rgb::new(0.85, 0.90, 0.95),
      Lava => Rgb::new(1.00, 0.45, 0.05),
      Lightning => Rgb::new(1.00, 0.95, 0.40),
      Plasma => Rgb::new(1.00, 0.50, 0.90),
      Mud => Rgb::new(0.45, 0.32, 0.18),
      Ice => Rgb::new(0.60, 0.85, 1.00),
      Mist => Rgb::new(0.70, 0.75, 0.85),
      Sand => Rgb::new(0.85, 0.75, 0.45),
      Crystal => Rgb::new(0.60, 0.90, 0.85),
      Storm => Rgb::new(0.50, 0.55, 0.70),
      Arcane => Rgb::new(0.85, 0.30, 0.85),
    }
  }
}

impl core::fmt::Display for Element {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.name())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fusion_is_commutative_and_total() {
    for &a in &Element::ALL {
      for &b in &Element::ALL {
        assert_eq!(a.fuse(b), b.fuse(a), "fuse({a}, {b}) must be symmetric");
      }
    }
  }

  #[test]
  fn same_element_fuses_to_itself() {
    for &a in &Element::ALL {
      assert_eq!(a.fuse(a), a);
    }
  }

  #[test]
  fn every_base_pair_is_a_distinct_compound() {
    use Element::*;
    let pairs = [
      (Fire, Water, Steam),
      (Fire, Earth, Lava),
      (Fire, Air, Lightning),
      (Fire, Aether, Plasma),
      (Water, Earth, Mud),
      (Water, Air, Ice),
      (Water, Aether, Mist),
      (Earth, Air, Sand),
      (Earth, Aether, Crystal),
      (Air, Aether, Storm),
    ];
    for (a, b, expected) in pairs {
      assert_eq!(a.fuse(b), expected, "{a} + {b}");
      assert!(expected.is_compound());
    }
    // All ten compounds from base pairs are distinct.
    let mut seen = std::collections::HashSet::new();
    for (a, b, _) in pairs {
      assert!(seen.insert(a.fuse(b)));
    }
  }

  #[test]
  fn compound_mixtures_are_arcane() {
    assert_eq!(Element::Steam.fuse(Element::Lava), Element::Arcane);
    assert_eq!(Element::Ice.fuse(Element::Fire), Element::Arcane);
  }

  #[test]
  fn bases_and_compounds_partition() {
    assert_eq!(Element::BASES.len(), 5);
    assert!(Element::BASES.iter().all(|e| e.is_base()));
    assert_eq!(Element::ALL.iter().filter(|e| e.is_base()).count(), 5);
    assert_eq!(Element::ALL.iter().filter(|e| e.is_compound()).count(), 11);
  }

  #[test]
  fn colors_are_in_range() {
    for &e in &Element::ALL {
      let c = e.color();
      for ch in c.to_array() {
        assert!((0.0..=1.0).contains(&ch));
      }
    }
  }
}

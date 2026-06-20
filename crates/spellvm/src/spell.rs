//! A [`Spell`]: an ordered list of [`Glyph`]s, plus human-friendly parsing and
//! formatting so spells can be written, stored and shared as plain text.

use crate::glyph::Glyph;
use crate::outcome::SpellOutcome;
use crate::vm::{Vm, VmError};
use core::str::FromStr;

/// An error produced while building a spell from text or compiling it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpellError {
  /// A token did not name any known glyph.
  UnknownGlyph {
    /// The offending token.
    token: String,
    /// Its zero-based position in the source.
    position: usize,
  },
  /// The glyph sequence was syntactically fine but failed to evaluate.
  Vm(VmError),
}

impl core::fmt::Display for SpellError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      SpellError::UnknownGlyph { token, position } => {
        write!(f, "unknown glyph {token:?} at position {position}")
      }
      SpellError::Vm(e) => write!(f, "{e}"),
    }
  }
}

impl std::error::Error for SpellError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      SpellError::Vm(e) => Some(e),
      SpellError::UnknownGlyph { .. } => None,
    }
  }
}

impl From<VmError> for SpellError {
  fn from(e: VmError) -> Self {
    SpellError::Vm(e)
  }
}

/// A spell: the program a wizard inscribes, as an ordered list of glyphs.
///
/// Spells can be written as text — glyph names separated by whitespace and/or
/// commas — and compiled into a [`SpellOutcome`] with [`Spell::outcome`].
///
/// ```
/// use spellvm::{Spell, Form};
/// let spell: Spell = "Fire Amplify Bolt".parse().unwrap();
/// let outcome = spell.outcome().unwrap();
/// assert_eq!(outcome.components[0].form, Form::Bolt);
/// assert_eq!(outcome.components[0].power, 2);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spell {
  /// The glyphs that make up the spell, in casting order.
  pub glyphs: Vec<Glyph>,
}

impl Spell {
  /// Creates a spell from an explicit glyph list.
  pub fn new(glyphs: impl Into<Vec<Glyph>>) -> Self {
    Self { glyphs: glyphs.into() }
  }

  /// Parses a spell from text such as `"Fire Water Fuse Bolt"`.
  ///
  /// Tokens are separated by any whitespace and/or commas, and glyph names are
  /// matched case-insensitively. Returns [`SpellError::UnknownGlyph`] for the
  /// first unrecognised token.
  pub fn parse(src: &str) -> Result<Spell, SpellError> {
    let mut glyphs = Vec::new();
    for (position, token) in src
      .split(|c: char| c.is_whitespace() || c == ',')
      .filter(|t| !t.is_empty())
      .enumerate()
    {
      let glyph = Glyph::from_token(token).ok_or_else(|| {
        SpellError::UnknownGlyph { token: token.to_string(), position }
      })?;
      glyphs.push(glyph);
    }
    Ok(Spell { glyphs })
  }

  /// The glyphs of this spell.
  pub fn glyphs(&self) -> &[Glyph] {
    &self.glyphs
  }

  /// The number of glyphs in the spell.
  pub fn len(&self) -> usize {
    self.glyphs.len()
  }

  /// Whether the spell has no glyphs.
  pub fn is_empty(&self) -> bool {
    self.glyphs.is_empty()
  }

  /// Appends a glyph to the spell.
  pub fn push(&mut self, glyph: Glyph) {
    self.glyphs.push(glyph);
  }

  /// Compiles the spell into its [`SpellOutcome`] by running the [`Vm`].
  pub fn outcome(&self) -> Result<SpellOutcome, VmError> {
    Vm::run(&self.glyphs)
  }
}

impl FromStr for Spell {
  type Err = SpellError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Spell::parse(s)
  }
}

impl core::fmt::Display for Spell {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    for (i, glyph) in self.glyphs.iter().enumerate() {
      if i > 0 {
        f.write_str(" ")?;
      }
      f.write_str(glyph.name())?;
    }
    Ok(())
  }
}

impl From<Vec<Glyph>> for Spell {
  fn from(glyphs: Vec<Glyph>) -> Self {
    Spell { glyphs }
  }
}

impl FromIterator<Glyph> for Spell {
  fn from_iter<I: IntoIterator<Item = Glyph>>(iter: I) -> Self {
    Spell { glyphs: iter.into_iter().collect() }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::element::Element;
  use crate::outcome::Form;

  #[test]
  fn parse_basic() {
    let spell = Spell::parse("Fire Water Fuse Bolt").unwrap();
    assert_eq!(
      spell.glyphs,
      vec![Glyph::Fire, Glyph::Water, Glyph::Fuse, Glyph::Bolt]
    );
  }

  #[test]
  fn parse_is_separator_tolerant() {
    let spell = Spell::parse("  fire, water ,fuse\tbolt\n").unwrap();
    assert_eq!(spell.len(), 4);
  }

  #[test]
  fn parse_reports_unknown_glyph_position() {
    let err = Spell::parse("Fire Wadddter Bolt").unwrap_err();
    assert_eq!(
      err,
      SpellError::UnknownGlyph { token: "Wadddter".to_string(), position: 1 }
    );
  }

  #[test]
  fn display_round_trips_through_parse() {
    let text = "Fire Water Fuse Amplify Bolt Trigger Earth Nova";
    let spell = Spell::parse(text).unwrap();
    assert_eq!(spell.to_string(), text);
    assert_eq!(Spell::parse(&spell.to_string()).unwrap(), spell);
  }

  #[test]
  fn outcome_matches_vm() {
    let spell = Spell::parse("Fire Water Fuse Bolt").unwrap();
    let out = spell.outcome().unwrap();
    assert_eq!(out.components[0].element, Element::Steam);
    assert_eq!(out.components[0].form, Form::Bolt);
  }

  #[test]
  fn empty_spell_is_empty() {
    let spell = Spell::parse("   ").unwrap();
    assert!(spell.is_empty());
    assert!(spell.outcome().unwrap().is_empty());
  }
}

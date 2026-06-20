//! End-to-end tests that drive `spellvm` purely through its public API, the way
//! the game (or any other host) would. These double as living documentation of
//! the spells the design doc references.

use spellvm::prelude::*;

/// Compile a spell from text, asserting it parses and evaluates.
fn cast(src: &str) -> SpellOutcome {
  src
    .parse::<Spell>()
    .unwrap_or_else(|e| panic!("parse {src:?}: {e}"))
    .outcome()
    .unwrap_or_else(|e| panic!("compile {src:?}: {e}"))
}

#[test]
fn worked_example_fire_bolt() {
  let out = cast("Fire Bolt");
  assert_eq!(out.components.len(), 1);
  let c = &out.components[0];
  assert_eq!(c.element, Element::Fire);
  assert_eq!(c.form, Form::Bolt);
}

#[test]
fn worked_example_steam_nova() {
  // Water + Air would be Ice; Fire + Water is Steam. Verify the fusion path.
  let out = cast("Fire Water Fuse Nova");
  let c = &out.components[0];
  assert_eq!(c.element, Element::Steam);
  assert_eq!(c.form, Form::Nova);
  assert_eq!(c.power, 2, "a fused compound starts at power 2");
}

#[test]
fn worked_example_homing_split_bolt() {
  let out = cast("Air Split Split Seek Bolt");
  let c = &out.components[0];
  assert_eq!(c.element, Element::Air);
  assert_eq!(c.count, 3, "two Splits over base 1");
  assert!(c.homing);
}

#[test]
fn worked_example_triggered_nova_on_bolt() {
  // The headline example from the design doc.
  let out = cast("Fire Bolt Trigger Fire Nova");
  assert_eq!(out.depth(), 2);
  assert_eq!(out.total_components(), 2);
  let bolt = &out.components[0];
  assert_eq!(bolt.form, Form::Bolt);
  let nova = bolt.on_impact.as_ref().unwrap().primary().unwrap();
  assert_eq!(nova.form, Form::Nova);
  assert_eq!(nova.element, Element::Fire);
}

#[test]
fn a_developer_can_author_any_circle_programmatically() {
  // Building a spell glyph-by-glyph (as the game's grimoire editor does)
  // produces the same result as parsing its text form.
  let mut spell = Spell::default();
  for g in
    [Glyph::Earth, Glyph::Earth, Glyph::Fuse, Glyph::Amplify, Glyph::Wall]
  {
    spell.push(g);
  }
  let from_text = "Earth Earth Fuse Amplify Wall".parse::<Spell>().unwrap();
  assert_eq!(spell, from_text);

  let c = &spell.outcome().unwrap().components[0];
  assert_eq!(c.element, Element::Earth);
  assert_eq!(c.form, Form::Wall);
  assert_eq!(c.power, 3); // charge 2 from fuse + 1 amplify
}

#[test]
fn every_style_is_just_handwriting() {
  // The same spell, drawn six different ways, is still the same spell.
  let spell: Spell =
    "Water Air Fuse Reach Beam Trigger Earth Nova".parse().unwrap();
  let canonical = spell.outcome().unwrap();
  for style in Style::ALL {
    let sigil = Sigil::encode(&spell, style);
    assert_eq!(sigil.decode(), spell, "{style} changed the spell");
    assert_eq!(
      sigil.decode().outcome().unwrap(),
      canonical,
      "{style} changed the outcome"
    );
  }
}

#[test]
fn property_random_programs_round_trip_through_geometry() {
  // A deterministic pseudo-random walk over the glyph set: whatever compiles
  // must survive an encode/decode round trip in every style unchanged.
  let glyphs = Glyph::ALL;
  let mut state: u32 = 0x1234_5678;
  let mut next = || {
    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    glyphs[(state >> 24) as usize % glyphs.len()]
  };

  for _ in 0..200 {
    let program: Vec<Glyph> = (0..12).map(|_| next()).collect();
    let spell = Spell::new(program);
    // We only assert the geometric invariant, which holds regardless of
    // whether the program itself compiles to a valid outcome.
    for style in Style::ALL {
      let decoded = Sigil::encode(&spell, style).decode();
      assert_eq!(decoded, spell);
      assert_eq!(spell.outcome(), decoded.outcome());
    }
  }
}

#[test]
fn runes_exist_for_every_glyph() {
  // The host renders glyphs from these procedurally; none may be empty.
  for g in Glyph::ALL {
    let rune = g.rune();
    assert!(!rune.strokes.is_empty(), "{g} has no rune");
  }
}

#[cfg(feature = "serde")]
#[test]
fn spells_and_outcomes_survive_serde_round_trip() {
  let spell: Spell =
    "Fire Water Fuse Amplify Bolt Trigger Earth Nova".parse().unwrap();

  let spell_json = serde_json::to_string(&spell).unwrap();
  let spell_back: Spell = serde_json::from_str(&spell_json).unwrap();
  assert_eq!(spell, spell_back);

  let outcome = spell.outcome().unwrap();
  let outcome_json = serde_json::to_string(&outcome).unwrap();
  let outcome_back: SpellOutcome = serde_json::from_str(&outcome_json).unwrap();
  assert_eq!(outcome, outcome_back);

  let sigil = Sigil::encode(&spell, Style::Spiral);
  let sigil_json = serde_json::to_string(&sigil).unwrap();
  let sigil_back: Sigil = serde_json::from_str(&sigil_json).unwrap();
  assert_eq!(sigil, sigil_back);
}

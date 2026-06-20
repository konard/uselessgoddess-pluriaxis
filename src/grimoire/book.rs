//! The player's spellbook: the draft being composed, the equipped spells, and
//! the hotkeys that pick between them.

use crate::prelude::*;

pub fn plugin(app: &mut App) {
  app.init_resource::<Book>();
  app.add_systems(
    Update,
    select_hotkeys
      .in_set(AppSystems::RecordInput)
      .run_if(in_state(Screen::Playing)),
  );
}

/// Everything the player knows and is carrying.
#[derive(Resource)]
pub struct Book {
  /// The spell currently being inscribed in the editor.
  pub draft: Vec<Glyph>,
  /// Spells bound to slots 1-9.
  pub equipped: Vec<Spell>,
  /// Which equipped slot is active.
  pub selected: usize,
  /// Glyphs available to inscribe with.
  pub known: Vec<Glyph>,
}

impl Default for Book {
  fn default() -> Self {
    // A handful of starter spells that show off the breadth of the VM.
    let equipped = vec![
      Spell::new([Glyph::Fire, Glyph::Bolt]),
      Spell::new([Glyph::Water, Glyph::Fuse, Glyph::Air, Glyph::Beam]),
      Spell::new([Glyph::Earth, Glyph::Nova, Glyph::Amplify]),
      Spell::new([Glyph::Air, Glyph::Bolt, Glyph::Split, Glyph::Seek]),
      Spell::new([Glyph::Aether, Glyph::Orb, Glyph::Linger, Glyph::Ricochet]),
    ];
    Self {
      draft: Vec::new(),
      equipped,
      selected: 0,
      known: Glyph::ALL.to_vec(),
    }
  }
}

impl Book {
  /// The draft glyphs as a [`Spell`].
  pub fn draft_spell(&self) -> Spell {
    Spell::new(self.draft.clone())
  }
}

fn select_hotkeys(keys: Res<ButtonInput<KeyCode>>, mut book: ResMut<Book>) {
  const DIGITS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
  ];
  for (i, key) in DIGITS.iter().enumerate() {
    if keys.just_pressed(*key) && i < book.equipped.len() {
      book.selected = i;
    }
  }
}

/// A short human-readable summary of an outcome, for HUD readouts.
pub fn summarize(outcome: &SpellOutcome) -> String {
  let Some(c) = outcome.primary() else {
    return "fizzles".into();
  };
  let count =
    if c.count > 1 { format!("{}x ", c.count) } else { String::new() };

  let mut tags = Vec::new();
  if c.pierce {
    tags.push("pierce");
  }
  if c.homing {
    tags.push("seek");
  }
  if c.bounce > 0 {
    tags.push("ricochet");
  }
  let extra = if tags.is_empty() {
    String::new()
  } else {
    format!(" [{}]", tags.join(", "))
  };

  let more = match outcome.components.len() {
    0 | 1 => String::new(),
    n => format!(" +{} more", n - 1),
  };

  format!("{count}{} {} (pwr {}){extra}{more}", c.element, c.form, c.power)
}

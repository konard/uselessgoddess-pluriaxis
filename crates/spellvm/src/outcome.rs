//! The compiled result of a spell: a pure, engine-agnostic description of what
//! the spell *does*. The host game turns this data into entities, particles and
//! gameplay; the VM never renders or simulates anything itself.

use crate::element::{Element, Rgb};

/// Base power (damage / intensity tier) of any form before modifiers.
pub const BASE_POWER: u8 = 1;
/// Base instance count (projectiles, orbs, ...) before [`Modifier::Split`].
pub const BASE_COUNT: u8 = 1;
/// Base reach in world units before [`Modifier::Reach`].
pub const BASE_RANGE: f32 = 320.0;
/// Extra reach added per [`Modifier::Reach`].
pub const RANGE_STEP: f32 = 160.0;
/// Base effect lifetime before [`Modifier::Linger`].
pub const BASE_DURATION_MS: u32 = 1200;
/// Extra lifetime added per [`Modifier::Linger`].
pub const DURATION_STEP_MS: u32 = 900;

/// Upper bound for accumulated power, keeping values sane and overflow-free.
pub const MAX_POWER: u8 = 16;
/// Upper bound for instance count.
pub const MAX_COUNT: u8 = 12;
/// Upper bound for ricochets.
pub const MAX_BOUNCE: u8 = 12;

/// How a spell is delivered into the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Form {
  /// A single travelling projectile.
  Bolt,
  /// A continuous, instantaneous ray from the caster.
  Beam,
  /// A radial burst centred on the caster (or on impact when triggered).
  Nova,
  /// A projectile that orbits the caster (Vampire-Survivors style).
  Orb,
  /// A protective ward / buff attached to the caster.
  Ward,
  /// A static barrier placed in the world.
  Wall,
}

impl Form {
  /// The human-readable name of the form.
  pub fn name(self) -> &'static str {
    match self {
      Form::Bolt => "Bolt",
      Form::Beam => "Beam",
      Form::Nova => "Nova",
      Form::Orb => "Orb",
      Form::Ward => "Ward",
      Form::Wall => "Wall",
    }
  }
}

impl core::fmt::Display for Form {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.name())
  }
}

/// A glyph that tunes the spell currently being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Modifier {
  /// Increase power (damage / intensity).
  Amplify,
  /// Increase the number of instances (multicast).
  Split,
  /// Increase range / travel distance.
  Reach,
  /// Increase effect lifetime.
  Linger,
  /// Projectiles pass through targets instead of stopping.
  Pierce,
  /// Projectiles home toward targets.
  Seek,
  /// Projectiles bounce off geometry.
  Ricochet,
}

impl Modifier {
  /// The human-readable name of the modifier.
  pub fn name(self) -> &'static str {
    match self {
      Modifier::Amplify => "Amplify",
      Modifier::Split => "Split",
      Modifier::Reach => "Reach",
      Modifier::Linger => "Linger",
      Modifier::Pierce => "Pierce",
      Modifier::Seek => "Seek",
      Modifier::Ricochet => "Ricochet",
    }
  }
}

impl core::fmt::Display for Modifier {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.name())
  }
}

/// One self-contained effect produced by a spell.
///
/// A spell program can finalise several components (e.g. the "four quarters" of
/// a master's sigil), so a [`SpellOutcome`] holds a list of these.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpellComponent {
  /// The elemental payload (possibly a fused compound).
  pub element: Element,
  /// How the effect is delivered.
  pub form: Form,
  /// Power tier; always `>= 1`.
  pub power: u8,
  /// Number of instances spawned; always `>= 1`.
  pub count: u8,
  /// Reach in world units.
  pub range: f32,
  /// Lifetime in milliseconds.
  pub duration_ms: u32,
  /// Whether projectiles pierce through targets.
  pub pierce: bool,
  /// Whether projectiles home toward targets.
  pub homing: bool,
  /// Number of ricochets remaining (`0` = none).
  pub bounce: u8,
  /// A nested spell cast when this component impacts ([`Modifier`]-free
  /// recursion enabled by the `Trigger` control glyph).
  pub on_impact: Option<Box<SpellOutcome>>,
}

impl SpellComponent {
  /// The canonical colour of this component, from its element.
  pub fn color(&self) -> Rgb {
    self.element.color()
  }

  /// Whether this component triggers a nested spell on impact.
  pub fn has_trigger(&self) -> bool {
    self.on_impact.is_some()
  }
}

/// The fully compiled effect of a spell program.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpellOutcome {
  /// The independent effects this spell produces, in finalisation order.
  pub components: Vec<SpellComponent>,
}

impl SpellOutcome {
  /// Whether the spell produces no components at all.
  pub fn is_empty(&self) -> bool {
    self.components.is_empty()
  }

  /// The first (primary) component, if any.
  pub fn primary(&self) -> Option<&SpellComponent> {
    self.components.first()
  }

  /// The maximum nesting depth across all components (`1` for a flat spell).
  pub fn depth(&self) -> usize {
    fn comp_depth(c: &SpellComponent) -> usize {
      1 + c.on_impact.as_deref().map_or(0, SpellOutcome::depth)
    }
    self.components.iter().map(comp_depth).max().unwrap_or(0)
  }

  /// Total number of components, including nested ones.
  pub fn total_components(&self) -> usize {
    fn comp_total(c: &SpellComponent) -> usize {
      1 + c.on_impact.as_deref().map_or(0, SpellOutcome::total_components)
    }
    self.components.iter().map(comp_total).sum()
  }
}

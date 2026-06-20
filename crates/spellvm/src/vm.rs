//! The spell virtual machine: a tiny stack machine that turns a sequence of
//! [`Glyph`]s into a [`SpellOutcome`].
//!
//! The model is concatenative (Forth / Magicka / Noita flavoured):
//!
//! * **Element** glyphs push an [`Ingredient`] (an element with a *charge*) onto
//!   the stack.
//! * **`Fuse`** pops the top two ingredients and combines them — same elements
//!   stack their charge into raw power, different elements mix into a compound.
//! * **Modifier** glyphs accumulate into the spell currently being built; their
//!   order relative to element pushes does not matter.
//! * **Form** glyphs finalise a [`SpellComponent`], consuming the top ingredient
//!   (defaulting to [`Element::Aether`] on an empty stack) and the accumulated
//!   modifiers.
//! * **`Trigger`** makes the next finalised form fire on the previous
//!   component's impact, enabling nested spells like
//!   `Fire Bolt Trigger Fire Nova`.

use crate::element::Element;
use crate::glyph::Glyph;
use crate::outcome::{
  BASE_COUNT, BASE_DURATION_MS, BASE_RANGE, DURATION_STEP_MS, Form, MAX_BOUNCE,
  MAX_COUNT, MAX_POWER, RANGE_STEP, SpellComponent, SpellOutcome,
};

/// Maximum allowed nesting depth for [`Glyph::Trigger`] chains.
pub const MAX_DEPTH: usize = 6;

/// An element on the VM stack together with its accumulated *charge*.
///
/// A freshly pushed element has a charge of `1`. Fusing two ingredients adds
/// their charges, so `Fire Fire Fuse` yields `Fire` with charge `2`, which the
/// finalising form turns into extra power.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ingredient {
  /// The element of this ingredient.
  pub element: Element,
  /// How concentrated it is; always `>= 1`.
  pub charge: u8,
}

impl Ingredient {
  /// A fresh, single-charge ingredient of `element`.
  pub fn new(element: Element) -> Self {
    Self { element, charge: 1 }
  }
}

/// Modifiers accumulated for the spell component currently being built.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Pending {
  amplify: u8,
  split: u8,
  reach: u8,
  linger: u8,
  bounce: u8,
  pierce: bool,
  seek: bool,
}

/// Something that went wrong while evaluating a spell program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VmError {
  /// A `Trigger` appeared before any form had been finalised, so there is no
  /// component for the nested spell to attach to.
  TriggerWithoutBase,
  /// The program ended with a `Trigger` still waiting for its payload form.
  DanglingTrigger,
  /// A `Trigger` chain exceeded [`MAX_DEPTH`].
  NestingTooDeep {
    /// The depth limit that was exceeded.
    max: usize,
  },
}

impl core::fmt::Display for VmError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      VmError::TriggerWithoutBase => {
        f.write_str("a Trigger glyph needs a finalised form before it")
      }
      VmError::DanglingTrigger => {
        f.write_str("the spell ended with a Trigger that never received a form")
      }
      VmError::NestingTooDeep { max } => {
        write!(f, "Trigger nesting exceeded the maximum depth of {max}")
      }
    }
  }
}

impl std::error::Error for VmError {}

/// The spell virtual machine.
///
/// Drive it one [`Glyph`] at a time with [`Vm::step`] (handy for a step
/// debugger UI) and call [`Vm::finish`] to extract the [`SpellOutcome`], or use
/// the [`Vm::run`] convenience to evaluate a whole program at once.
#[derive(Clone, Debug, Default)]
pub struct Vm {
  stack: Vec<Ingredient>,
  pending: Pending,
  outcome: SpellOutcome,
  awaiting_trigger: bool,
}

impl Vm {
  /// Creates a fresh, empty VM.
  pub fn new() -> Self {
    Self::default()
  }

  /// The current ingredient stack, bottom first (for inspection / debugging).
  pub fn stack(&self) -> &[Ingredient] {
    &self.stack
  }

  /// Whether the VM is waiting for a form to complete a [`Glyph::Trigger`].
  pub fn awaiting_trigger(&self) -> bool {
    self.awaiting_trigger
  }

  /// Evaluates a whole program and returns its outcome.
  ///
  /// ```
  /// use spellvm::{Glyph, Vm, Form};
  /// let out = Vm::run(&[Glyph::Fire, Glyph::Bolt]).unwrap();
  /// assert_eq!(out.components.len(), 1);
  /// assert_eq!(out.components[0].form, Form::Bolt);
  /// ```
  pub fn run(glyphs: &[Glyph]) -> Result<SpellOutcome, VmError> {
    let mut vm = Vm::new();
    for &g in glyphs {
      vm.step(g)?;
    }
    vm.finish()
  }

  /// Feeds a single glyph to the VM, mutating its state.
  pub fn step(&mut self, glyph: Glyph) -> Result<(), VmError> {
    if let Some(element) = glyph.as_element() {
      self.stack.push(Ingredient::new(element));
      return Ok(());
    }
    if let Some(modifier) = glyph.as_modifier() {
      self.apply_modifier(modifier);
      return Ok(());
    }
    if let Some(form) = glyph.as_form() {
      return self.finalize_form(form);
    }
    match glyph {
      Glyph::Fuse => {
        self.fuse();
        Ok(())
      }
      Glyph::Trigger => {
        if self.outcome.components.is_empty() {
          return Err(VmError::TriggerWithoutBase);
        }
        self.awaiting_trigger = true;
        Ok(())
      }
      // Element / modifier / form glyphs are handled above.
      _ => unreachable!("unhandled glyph {glyph:?}"),
    }
  }

  /// Consumes the VM and returns the finished outcome.
  ///
  /// Fails with [`VmError::DanglingTrigger`] if a `Trigger` never received its
  /// payload form.
  pub fn finish(self) -> Result<SpellOutcome, VmError> {
    if self.awaiting_trigger {
      return Err(VmError::DanglingTrigger);
    }
    Ok(self.outcome)
  }

  fn apply_modifier(&mut self, modifier: crate::outcome::Modifier) {
    use crate::outcome::Modifier::*;
    let p = &mut self.pending;
    match modifier {
      Amplify => p.amplify = p.amplify.saturating_add(1),
      Split => p.split = p.split.saturating_add(1),
      Reach => p.reach = p.reach.saturating_add(1),
      Linger => p.linger = p.linger.saturating_add(1),
      Ricochet => p.bounce = p.bounce.saturating_add(1),
      Pierce => p.pierce = true,
      Seek => p.seek = true,
    }
  }

  fn fuse(&mut self) {
    // Fusion needs two ingredients; otherwise it is a no-op so a stray
    // `Fuse` never breaks an experiment.
    if self.stack.len() < 2 {
      return;
    }
    let a = self.stack.pop().unwrap();
    let b = self.stack.pop().unwrap();
    let element = a.element.fuse(b.element);
    let charge =
      (a.charge as u16 + b.charge as u16).min(MAX_POWER as u16) as u8;
    self.stack.push(Ingredient { element, charge });
  }

  fn finalize_form(&mut self, form: Form) -> Result<(), VmError> {
    let ingredient =
      self.stack.pop().unwrap_or_else(|| Ingredient::new(Element::Aether));
    let p = core::mem::take(&mut self.pending);

    let power = (ingredient.charge as u16 + p.amplify as u16)
      .clamp(1, MAX_POWER as u16) as u8;
    let count =
      (BASE_COUNT as u16 + p.split as u16).clamp(1, MAX_COUNT as u16) as u8;
    let range = BASE_RANGE + p.reach as f32 * RANGE_STEP;
    let duration_ms = BASE_DURATION_MS + p.linger as u32 * DURATION_STEP_MS;
    let bounce = p.bounce.min(MAX_BOUNCE);

    let component = SpellComponent {
      element: ingredient.element,
      form,
      power,
      count,
      range,
      duration_ms,
      pierce: p.pierce,
      homing: p.seek,
      bounce,
      on_impact: None,
    };

    if self.awaiting_trigger {
      self.awaiting_trigger = false;
      let base = self
        .outcome
        .components
        .last_mut()
        .ok_or(VmError::TriggerWithoutBase)?;
      if chain_len(base) + 1 > MAX_DEPTH {
        return Err(VmError::NestingTooDeep { max: MAX_DEPTH });
      }
      attach_on_impact(base, SpellOutcome { components: vec![component] });
    } else {
      self.outcome.components.push(component);
    }
    Ok(())
  }
}

/// The number of components along a component's `on_impact` chain, inclusive.
fn chain_len(component: &SpellComponent) -> usize {
  match component.on_impact.as_deref().and_then(|o| o.components.last()) {
    Some(next) => 1 + chain_len(next),
    None => 1,
  }
}

/// Attaches `payload` at the deepest end of a component's `on_impact` chain.
fn attach_on_impact(component: &mut SpellComponent, payload: SpellOutcome) {
  match component.on_impact.as_mut() {
    None => component.on_impact = Some(Box::new(payload)),
    Some(inner) => {
      let last = inner
        .components
        .last_mut()
        .expect("a nested trigger outcome always has one component");
      attach_on_impact(last, payload);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn run(glyphs: &[Glyph]) -> SpellOutcome {
    Vm::run(glyphs).expect("program should evaluate")
  }

  #[test]
  fn simple_bolt() {
    let out = run(&[Glyph::Fire, Glyph::Bolt]);
    assert_eq!(out.components.len(), 1);
    let c = &out.components[0];
    assert_eq!(c.element, Element::Fire);
    assert_eq!(c.form, Form::Bolt);
    assert_eq!(c.power, 1);
    assert_eq!(c.count, 1);
    assert_eq!(c.range, BASE_RANGE);
    assert!(!c.pierce && !c.homing && c.bounce == 0);
    assert!(c.on_impact.is_none());
  }

  #[test]
  fn empty_form_defaults_to_aether() {
    let out = run(&[Glyph::Bolt]);
    assert_eq!(out.components[0].element, Element::Aether);
  }

  #[test]
  fn fuse_same_element_accumulates_charge() {
    let out = run(&[Glyph::Fire, Glyph::Fire, Glyph::Fuse, Glyph::Bolt]);
    assert_eq!(out.components[0].element, Element::Fire);
    assert_eq!(out.components[0].power, 2);
  }

  #[test]
  fn fuse_different_elements_makes_compound() {
    let out = run(&[Glyph::Fire, Glyph::Water, Glyph::Fuse, Glyph::Bolt]);
    assert_eq!(out.components[0].element, Element::Steam);
    // Fusion always sums charge, so a compound is inherently more potent
    // than a single base element.
    assert_eq!(out.components[0].power, 2);
  }

  #[test]
  fn lone_fuse_is_a_noop() {
    let out = run(&[Glyph::Fuse, Glyph::Fire, Glyph::Bolt]);
    assert_eq!(out.components[0].element, Element::Fire);
  }

  #[test]
  fn modifiers_apply_to_the_next_form() {
    let out = run(&[
      Glyph::Fire,
      Glyph::Amplify,
      Glyph::Amplify,
      Glyph::Split,
      Glyph::Reach,
      Glyph::Linger,
      Glyph::Pierce,
      Glyph::Seek,
      Glyph::Ricochet,
      Glyph::Bolt,
    ]);
    let c = &out.components[0];
    assert_eq!(c.power, 3); // charge 1 + 2 amplify
    assert_eq!(c.count, 2); // base 1 + 1 split
    assert_eq!(c.range, BASE_RANGE + RANGE_STEP);
    assert_eq!(c.duration_ms, BASE_DURATION_MS + DURATION_STEP_MS);
    assert!(c.pierce && c.homing);
    assert_eq!(c.bounce, 1);
  }

  #[test]
  fn modifier_order_is_irrelevant() {
    let a = run(&[Glyph::Amplify, Glyph::Fire, Glyph::Bolt]);
    let b = run(&[Glyph::Fire, Glyph::Amplify, Glyph::Bolt]);
    assert_eq!(a, b);
  }

  #[test]
  fn pending_modifiers_reset_between_forms() {
    let out = run(&[
      Glyph::Fire,
      Glyph::Amplify,
      Glyph::Bolt,
      Glyph::Water,
      Glyph::Bolt,
    ]);
    assert_eq!(out.components[0].power, 2);
    assert_eq!(out.components[1].power, 1); // amplify did not leak
  }

  #[test]
  fn trigger_nests_the_next_form() {
    let out = run(&[
      Glyph::Fire,
      Glyph::Bolt,
      Glyph::Trigger,
      Glyph::Fire,
      Glyph::Nova,
    ]);
    assert_eq!(out.components.len(), 1);
    let bolt = &out.components[0];
    assert_eq!(bolt.form, Form::Bolt);
    let nested = bolt.on_impact.as_ref().expect("trigger payload");
    assert_eq!(nested.components[0].form, Form::Nova);
    assert_eq!(out.depth(), 2);
  }

  #[test]
  fn triggers_chain_in_order() {
    let out = run(&[
      Glyph::Fire,
      Glyph::Bolt,
      Glyph::Trigger,
      Glyph::Water,
      Glyph::Nova,
      Glyph::Trigger,
      Glyph::Earth,
      Glyph::Wall,
    ]);
    assert_eq!(out.components.len(), 1);
    assert_eq!(out.depth(), 3);
    let bolt = &out.components[0];
    let nova = &bolt.on_impact.as_ref().unwrap().components[0];
    assert_eq!(nova.element, Element::Water);
    let wall = &nova.on_impact.as_ref().unwrap().components[0];
    assert_eq!(wall.element, Element::Earth);
    assert_eq!(wall.form, Form::Wall);
  }

  #[test]
  fn trigger_without_base_is_an_error() {
    assert_eq!(Vm::run(&[Glyph::Trigger]), Err(VmError::TriggerWithoutBase));
  }

  #[test]
  fn dangling_trigger_is_an_error() {
    let err = Vm::run(&[Glyph::Fire, Glyph::Bolt, Glyph::Trigger]);
    assert_eq!(err, Err(VmError::DanglingTrigger));
  }

  #[test]
  fn excessive_nesting_is_an_error() {
    let mut program = vec![Glyph::Fire, Glyph::Bolt];
    // Each Trigger+form adds one level of depth.
    for _ in 0..MAX_DEPTH {
      program.push(Glyph::Trigger);
      program.push(Glyph::Fire);
      program.push(Glyph::Nova);
    }
    assert_eq!(
      Vm::run(&program),
      Err(VmError::NestingTooDeep { max: MAX_DEPTH })
    );
  }

  #[test]
  fn multiple_top_level_components() {
    let out = run(&[Glyph::Fire, Glyph::Bolt, Glyph::Water, Glyph::Nova]);
    assert_eq!(out.components.len(), 2);
    assert_eq!(out.components[0].form, Form::Bolt);
    assert_eq!(out.components[1].form, Form::Nova);
  }

  #[test]
  fn empty_program_is_empty_outcome() {
    let out = run(&[]);
    assert!(out.is_empty());
  }
}

//! # spellvm
//!
//! A small, **dependency-free** stack virtual machine for crafting spells out of
//! rune glyphs and circular sigils. It is the rules engine behind the game
//! *Pluriaxis*, but it knows nothing about Bevy, rendering or input — it is pure
//! data in, pure data out, so it can be tested, fuzzed and reused on its own.
//!
//! ## The idea
//!
//! A wizard inscribes a spell as points connected by lines inside a circle. Each
//! point is a [`Glyph`] (an opcode); the order they are visited is a program for
//! a tiny concatenative [`Vm`]. Running that program yields a [`SpellOutcome`] —
//! a plain description of what to spawn — which the host game turns into
//! projectiles, novas, wards and particle effects.
//!
//! ```
//! use spellvm::prelude::*;
//!
//! // Write a spell as text...
//! let spell: Spell = "Fire Water Fuse Amplify Bolt".parse().unwrap();
//!
//! // ...compile it to an engine-agnostic effect description...
//! let outcome = spell.outcome().unwrap();
//! let bolt = &outcome.components[0];
//! assert_eq!(bolt.element, Element::Steam); // Fire + Water
//! assert_eq!(bolt.form, Form::Bolt);
//! assert_eq!(bolt.power, 3); // 2 from fusing two ingredients, +1 from Amplify
//!
//! // ...and lay it out as a circular sigil for drawing. The style is purely
//! // cosmetic: every style decodes back to the very same spell.
//! let sigil = Sigil::encode(&spell, Style::Circle);
//! assert_eq!(sigil.decode(), spell);
//! ```
//!
//! ## Layers
//!
//! * [`element`] — the elemental substances and how they [`fuse`](Element::fuse).
//! * [`glyph`] — the instruction set and its procedural [rune art](Glyph::rune).
//! * [`vm`] — the stack machine that evaluates a glyph program.
//! * [`outcome`] — the engine-agnostic description the VM produces.
//! * [`spell`] — a friendly [`Spell`] wrapper with text parsing.
//! * [`sigil`] — the geometric layout / "handwriting" of a spell.
//! * [`math`] — a tiny self-contained [`Vec2`].
//!
//! The optional `serde` feature derives `Serialize`/`Deserialize` for every
//! public data type so spells and sigils can be saved and shared.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod element;
pub mod glyph;
pub mod math;
pub mod outcome;
pub mod sigil;
pub mod spell;
pub mod vm;

pub use element::{Element, Rgb};
pub use glyph::{Category, Glyph, Rune};
pub use math::{Vec2, vec2};
pub use outcome::{Form, Modifier, SpellComponent, SpellOutcome};
pub use sigil::{Node, Sigil, Style};
pub use spell::{Spell, SpellError};
pub use vm::{Ingredient, Vm, VmError};

/// A convenient glob-import of the crate's most-used types.
///
/// ```
/// use spellvm::prelude::*;
/// let outcome = Vm::run(&[Glyph::Fire, Glyph::Bolt]).unwrap();
/// assert_eq!(outcome.components[0].form, Form::Bolt);
/// ```
pub mod prelude {
    pub use crate::element::{Element, Rgb};
    pub use crate::glyph::{Category, Glyph, Rune};
    pub use crate::math::{Vec2, vec2};
    pub use crate::outcome::{Form, Modifier, SpellComponent, SpellOutcome};
    pub use crate::sigil::{Node, Sigil, Style};
    pub use crate::spell::{Spell, SpellError};
    pub use crate::vm::{Ingredient, Vm, VmError};
}

# spellvm

A small, **dependency-free** stack virtual machine for crafting spells out of
rune glyphs and circular sigils. It is the rules engine behind the game
[*Pluriaxis*](https://github.com/uselessgoddess/pluriaxis), but it knows nothing
about Bevy, rendering or input — it is *pure data in, pure data out*, so it can
be unit-tested, fuzzed and reused entirely on its own.

```text
  glyphs ─▶ [ Vm ] ─▶ SpellOutcome ─▶ (the game spawns entities & VFX)
   ▲
   └── a Sigil is just glyphs laid out as points-and-lines in a circle
```

## The idea

In the fiction, a wizard inscribes a spell as **points connected by lines inside
a circle**. Each point is a [`Glyph`] (an opcode); the order they are visited is
a program for a tiny [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
virtual machine. Running that program yields a [`SpellOutcome`] — a plain
description of what to spawn — which the host game turns into projectiles,
novas, wards and particle effects.

```rust
use spellvm::prelude::*;

// Write a spell as text...
let spell: Spell = "Fire Water Fuse Amplify Bolt".parse().unwrap();

// ...compile it to an engine-agnostic effect description...
let outcome = spell.outcome().unwrap();
let bolt = &outcome.components[0];
assert_eq!(bolt.element, Element::Steam); // Fire + Water
assert_eq!(bolt.form, Form::Bolt);
assert_eq!(bolt.power, 3);                // 2 from fusing + 1 Amplify

// ...and lay it out as a circular sigil for drawing. The style is purely
// cosmetic: every style decodes back to the very same spell.
let sigil = Sigil::encode(&spell, Style::Circle);
assert_eq!(sigil.decode(), spell);
```

## The instruction set

A spell is an ordered list of glyphs. There are five kinds:

| Category   | Glyphs | Effect |
|------------|--------|--------|
| **Element**  | `Fire` `Water` `Earth` `Air` `Aether` | Push an ingredient onto the stack. |
| **Operator** | `Fuse` | Pop the top two ingredients and combine them. |
| **Form**     | `Bolt` `Beam` `Nova` `Orb` `Ward` `Wall` | Finalise one effect, consuming the top ingredient. |
| **Modifier** | `Amplify` `Split` `Reach` `Linger` `Pierce` `Seek` `Ricochet` | Tune the spell currently being built. |
| **Control**  | `Trigger` | Make the next form fire on the previous one's impact (nested spells). |

### How evaluation works

* **Elements** push an [`Ingredient`] — an element with a *charge* of `1`.
* **`Fuse`** pops two ingredients and pushes one. Charges always **add**, so a
  fused compound is inherently more potent (`Fire Fire Fuse` → `Fire` charge 2;
  `Fire Water Fuse` → `Steam` charge 2). Different bases mix Magicka-style into a
  compound element; identical ones simply concentrate.
* **Modifiers** accumulate and apply to the **next** form, so their order
  relative to element pushes does not matter.
* **Forms** finalise a [`SpellComponent`], consuming the top ingredient (or
  defaulting to `Aether` on an empty stack) plus the pending modifiers.
* **`Trigger`** attaches the next finalised form to the previous component's
  `on_impact`, allowing recursive spells such as `Fire Bolt Trigger Fire Nova`
  ("a fire bolt that bursts into a nova on impact"), up to [`vm::MAX_DEPTH`].

### Fusion table

|         | Fire | Water | Earth | Air | Aether |
|---------|------|-------|-------|-----|--------|
| **Fire**   | Fire | Steam | Lava | Lightning | Plasma |
| **Water**  | — | Water | Mud | Ice | Mist |
| **Earth**  | — | — | Earth | Sand | Crystal |
| **Air**    | — | — | — | Air | Storm |
| **Aether** | — | — | — | — | Aether |

Any other mixture (e.g. fusing two compounds) collapses to the unstable
`Arcane`. Fusion is **total** and **commutative**.

## Sigils: the same spell, six ways

A [`Sigil`] is a spell laid out geometrically. The [`Style`] is the wizard's
*handwriting* — apprentice `Circle`, journeyman `Spiral`, researcher `Line`,
master `Quarters`, adept `Semicircles`, dark-wizard `Scattered` — and it is
**purely cosmetic**: every style of the same spell decodes to the same program.
Node positions land in a unit disc so the host can scale them to any radius, and
[`Glyph::rune`] yields procedural line art for each glyph so nothing needs a
texture.

## Features

* `serde` *(off by default)* — derive `Serialize`/`Deserialize` for every public
  type, so spells and sigils can be saved, shared and version-controlled.

## Design goals

* **Zero runtime dependencies** (only optional `serde`). It ships its own
  [`Vec2`].
* **Engine-agnostic**: no rendering, no ECS, no I/O — only data.
* **Total and panic-free** on any input: malformed programs return a typed
  [`SpellError`]/[`VmError`], never a panic.
* **Thoroughly tested**: unit tests, end-to-end integration tests, doctests and
  property-style round-trip checks.

## Testing

```sh
cargo test -p spellvm                 # core suite
cargo test -p spellvm --features serde  # + serde round-trips
```

## License

Licensed under either of MIT or Apache-2.0 at your option.

[`Glyph`]: https://docs.rs/spellvm/latest/spellvm/glyph/enum.Glyph.html
[`Glyph::rune`]: https://docs.rs/spellvm/latest/spellvm/glyph/enum.Glyph.html#method.rune
[`Ingredient`]: https://docs.rs/spellvm/latest/spellvm/vm/struct.Ingredient.html
[`SpellComponent`]: https://docs.rs/spellvm/latest/spellvm/outcome/struct.SpellComponent.html
[`SpellError`]: https://docs.rs/spellvm/latest/spellvm/spell/enum.SpellError.html
[`SpellOutcome`]: https://docs.rs/spellvm/latest/spellvm/outcome/struct.SpellOutcome.html
[`Sigil`]: https://docs.rs/spellvm/latest/spellvm/sigil/struct.Sigil.html
[`Style`]: https://docs.rs/spellvm/latest/spellvm/sigil/enum.Style.html
[`Vec2`]: https://docs.rs/spellvm/latest/spellvm/math/struct.Vec2.html
[`VmError`]: https://docs.rs/spellvm/latest/spellvm/vm/enum.VmError.html
[`vm::MAX_DEPTH`]: https://docs.rs/spellvm/latest/spellvm/vm/constant.MAX_DEPTH.html

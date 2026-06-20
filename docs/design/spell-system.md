# Pluriaxis — Spell System Design (the `spellvm` VM)

> Status: this describes the spell engine **as built**. It is the companion to
> [`gameplay.md`](./gameplay.md): that document is player-facing ("what casting
> *feels* like"); this one is the engineering contract ("how a sigil becomes an
> effect"). Where the two overlap, this document is authoritative for behaviour
> and the source (`crates/spellvm/`) is authoritative for both.

---

## 1. Why a virtual machine

The fiction's central conceit is that a wizard's hand-drawn **sigil** — points
connected by lines inside a circle — is *literally source code*. The game takes
that seriously: a sigil is a program for a small
[concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
(stack-based, Forth/PostScript-flavoured) virtual machine. Running the program
produces a plain, engine-agnostic *description of what to spawn*. Nothing about
the VM knows that Bevy, rendering, input or even a screen exists.

```text
  text  ─parse─▶  Spell {glyphs}  ─Vm::run─▶  SpellOutcome  ─host─▶  entities + VFX
                       ▲   │
              Sigil::decode  Sigil::encode
                       │   ▼
                  Sigil {style, nodes}   (the same program, drawn as points-and-lines)
```

This split is the load-bearing design decision. It was an **explicit project
requirement**: the spell system must be a *separate, independent crate* that is
testable and extensible on its own, while the GUI and the spell *effects* live
on the Bevy side. The payoff:

* **Testability.** The whole rules engine is pure `data in → data out`, so it is
  covered by fast unit tests, end-to-end integration tests, doctests and
  property-style round-trip checks — no engine, no window, no GPU. The crate
  currently ships **41 unit + 9 integration + 6 doctests = 56 tests** that run in
  milliseconds.
* **Extensibility.** Adding a glyph, an element, a form or a modifier is a
  localised change in one file with a compiler-checked `match`; the game picks it
  up through the shared types.
* **Reuse & determinism.** The same crate can power a CLI, a level editor, a
  balance simulator or a server. Identical input always yields identical output
  (no clocks, no RNG state held across calls), which makes spells safe to save,
  diff and share.
* **Zero runtime dependencies.** `spellvm` pulls in nothing at runtime (only an
  *optional* `serde`), so it can never drag engine internals or version churn
  into the rules. It even ships its own tiny [`Vec2`](#8-the-vec2-boundary).

```rust
use spellvm::prelude::*;

// Author a spell as text (the game also builds it glyph-by-glyph)...
let spell: Spell = "Fire Water Fuse Amplify Bolt".parse().unwrap();

// ...compile it to an engine-agnostic effect description...
let out = spell.outcome().unwrap();
let bolt = &out.components[0];
assert_eq!(bolt.element, Element::Steam); // Fire + Water
assert_eq!(bolt.power, 3);                // 2 from fusing + 1 Amplify

// ...and lay it out as a circle to draw. Style is cosmetic: it round-trips.
let sigil = Sigil::encode(&spell, Style::Circle);
assert_eq!(sigil.decode(), spell);
```

---

## 2. Crate map

`crates/spellvm/` is one module per concept; everything public is re-exported
from the crate root and from `spellvm::prelude`.

| Module | Type(s) | Responsibility |
|---|---|---|
| `element` | `Element`, `Rgb` | The 5 bases + 11 compounds and their fusion algebra and colours. |
| `glyph` | `Glyph`, `Category`, `Rune` | The 20-opcode instruction alphabet and its procedural line-art. |
| `vm` | `Vm`, `Ingredient`, `Pending`, `VmError` | The stack machine that evaluates a glyph program. |
| `outcome` | `SpellOutcome`, `SpellComponent`, `Form`, `Modifier` | The engine-agnostic result of evaluation. |
| `spell` | `Spell`, `SpellError` | A glyph program: parse / build / compile / display. |
| `sigil` | `Sigil`, `Style`, `Node` | The geometric encoding (points-and-lines) and its layouts. |
| `math` | `Vec2`, `vec2` | A dependency-free 2D vector for the geometry. |

Crate-wide invariants (`lib.rs`): `#![forbid(unsafe_code)]` and
`#![warn(missing_docs)]`. The library is **panic-free on all inputs** — malformed
programs surface a typed error, never a panic.

---

## 3. The type pipeline

Three representations, each a pure function away from the next:

1. **`Spell` — the program.** An ordered `Vec<Glyph>`. This is the canonical,
   serialisable form of a spell. Built three ways, all equivalent:
   * parsed from text (`"Fire Bolt".parse()`), whitespace/comma separated;
   * assembled glyph-by-glyph (`spell.push(Glyph::Fire)`) — what the in-game
     grimoire editor does;
   * collected from an iterator (`FromIterator<Glyph>`).
2. **`SpellOutcome` — the result.** What `Spell::outcome()` (i.e. `Vm::run`)
   produces: a `Vec<SpellComponent>`, each a fully-resolved effect (element,
   form, power, count, range, duration, flags, and an optional nested
   `on_impact` outcome). This is the contract the game consumes.
3. **`Sigil` — the drawing.** `Sigil::encode(&spell, style)` lays the glyphs out
   as `Node { glyph, position }` points in a unit disc; `sigil.decode()` reads
   them back. The host scales the unit-disc positions to any pixel radius.

The boundary is deliberately narrow: **the game depends on `SpellOutcome` and on
`Sigil`'s geometry, and on nothing else inside the VM.**

---

## 4. The instruction set (20 glyphs, 5 categories)

A glyph is an opcode. There are 20, in five categories (`Glyph::ALL` enumerates
them; `Glyph::from_token` parses case-insensitively):

| Category | Glyphs | Stack effect |
|---|---|---|
| **Element** | `Fire` `Water` `Earth` `Air` `Aether` | Push an `Ingredient` (element, charge 1). |
| **Operator** | `Fuse` | Pop the top two ingredients, push their combination. |
| **Form** | `Bolt` `Beam` `Nova` `Orb` `Ward` `Wall` | Finalise one `SpellComponent`, consuming the top ingredient. |
| **Modifier** | `Amplify` `Split` `Reach` `Linger` `Pierce` `Seek` `Ricochet` | Tune the **next** form built. |
| **Control** | `Trigger` | Nest: make the next form fire on the previous component's impact. |

### 4.1 Evaluation model

The VM (`vm.rs`) holds: a `stack` of ingredients, a bag of `pending` modifiers,
the `outcome` accumulated so far, and an `awaiting_trigger` flag. `Vm::run`
`step`s each glyph then `finish`es:

* **Element → push.** Pushes `Ingredient { element, charge: 1 }`.
* **`Fuse` → combine.** Pops two ingredients, [fuses](#5-the-element-algebra)
  their elements, and **adds their charges** (so a fused compound is inherently
  more potent). With fewer than two ingredients it is a no-op — total, never a
  panic. Charge is the seed for a form's `power`.
* **Modifier → accumulate.** Recorded in `pending` and applied to the *next*
  form, so a modifier's position relative to element pushes does not matter
  (`Air Split Bolt` ≡ `Split Air Bolt`). Stackable modifiers (`Amplify`,
  `Split`, `Reach`, `Linger`, `Ricochet`) count repetitions; flag modifiers
  (`Pierce`, `Seek`) latch on.
* **Form → finalise.** Pops the top ingredient (**defaulting to `Aether` on an
  empty stack**, so a lone `Bolt` still casts), folds in the pending modifiers,
  and pushes a finished `SpellComponent`. Pending resets for the next form.
* **`Trigger` → nest.** Sets `awaiting_trigger`; the **next** finalised form is
  attached to the previous component's `on_impact` instead of being appended at
  top level. This is what makes recursive spells like *"a fire bolt that bursts
  into a frost nova on impact"* expressible.

### 4.2 Modifier semantics

| Modifier | Stacks? | Effect on the component |
|---|---|---|
| `Amplify` | yes | `power += 1` per use (seeds higher base damage/size). |
| `Split` | yes | `count += 1` per use (extra projectiles, fired in a fan). |
| `Reach` | yes | `range += RANGE_STEP` per use (travel distance / beam length / nova radius). |
| `Linger` | yes | `duration_ms += DURATION_STEP_MS` per use (how long persistent forms last). |
| `Ricochet` | yes | `bounce += 1` per use (wall/edge rebounds before expiry). |
| `Pierce` | flag | projectile passes through the first target instead of being consumed. |
| `Seek` | flag | `homing` — the effect steers toward the nearest live target. |

All numeric results are **clamped** (`MAX_POWER = 16`, `MAX_COUNT = 12`,
`MAX_BOUNCE = 12`) so no combination can produce absurd or overflowing values.

### 4.3 Errors

`Vm::run` returns `Result<SpellOutcome, VmError>`. The only failure modes are
structural, and all are typed:

* `TriggerWithoutBase` — `Trigger` with no preceding component to attach to.
* `DanglingTrigger` — the program ends still `awaiting_trigger`.
* `NestingTooDeep { max }` — `on_impact` chains deeper than `MAX_DEPTH = 6`.

Everything else degrades gracefully: too few ingredients for `Fuse`, an empty
stack at a form, or an unknown text token (surfaced earlier as
`SpellError::UnknownGlyph { token, position }`) never panic.

---

## 5. The element algebra

`Element` has **5 bases** (`Fire`, `Water`, `Earth`, `Air`, `Aether`) and **11
compounds**. `Element::fuse(a, b)` is the combination rule used by `Fuse`. It is
**total** (defined for every pair) and **commutative** (`fuse(a,b) == fuse(b,a)`):

* same element → itself (`Fire` + `Fire` = `Fire`, concentrating charge);
* two *distinct bases* → their signature compound (the table below);
* anything else (e.g. fusing two compounds) → the unstable catch-all `Arcane`.

### Fusion table

|        | Fire | Water | Earth | Air | Aether |
|--------|------|-------|-------|-----|--------|
| **Fire**   | Fire | Steam | Lava | Lightning | Plasma |
| **Water**  | —  | Water | Mud | Ice | Mist |
| **Earth**  | —  | — | Earth | Sand | Crystal |
| **Air**    | —  | — | — | Air | Storm |
| **Aether** | —  | — | — | — | Aether |

Each element carries a colour (`Element::color() -> Rgb`). The game multiplies
that colour past `1.0` so the HDR camera's bloom makes the effect glow — the
element *is* the palette, which is why the playground needs almost no art assets.

This is intentionally a small, learnable table (Magicka's discovery model):
10 distinct base pairings plus 5 self-fusions plus `Arcane`, all derivable by
experiment, never memorised from a manual.

---

## 6. The outcome contract (what the game consumes)

`SpellOutcome { components: Vec<SpellComponent> }`. Each component is fully
resolved — no further interpretation needed:

```rust
pub struct SpellComponent {
  pub element: Element,
  pub form: Form,                 // Bolt | Beam | Nova | Orb | Ward | Wall
  pub power: u8,                  // damage / size seed
  pub count: u8,                  // projectiles in the fan
  pub range: f32,                 // travel / length / radius budget
  pub duration_ms: u32,          // lifetime of persistent forms
  pub pierce: bool,
  pub homing: bool,
  pub bounce: u8,
  pub on_impact: Option<Box<SpellOutcome>>,  // nested spell, fired on hit
}
```

The forms partition into how the game simulates them (`src/game/projectile.rs`):

| Form | Behaviour | Geometry the host draws |
|---|---|---|
| `Bolt` | Fast travelling projectile, consumed on first hit (unless `Pierce`). | A glowing dart + tail. |
| `Orb` | Slow, large, long-lived projectile. | A pulsing disc. |
| `Beam` | Instant line from the caster along the aim. | A bright segment that fades. |
| `Nova` | Expanding ring centred on the cast point. | A growing circle. |
| `Ward` | Ring that follows the caster (defensive). | An orbiting shell. |
| `Wall` | Static barrier placed ahead of the caster. | A braced line segment. |

Helper queries (`is_empty`, `primary`, `depth`, `total_components`) let the host
read a spell without walking the tree — e.g. the HUD's one-line `summarize`.
Base tuning constants live in `outcome.rs` (`BASE_POWER`, `BASE_RANGE = 320`,
`RANGE_STEP = 160`, `BASE_DURATION_MS = 1200`, `DURATION_STEP_MS = 900`, …) so
balance is one file, not scattered magic numbers.

### Worked examples

| Spell | Outcome |
|---|---|
| `Fire Bolt` | One Fire bolt, base power. |
| `Fire Water Fuse Nova` | One **Steam** nova, power 2 (fused charge). |
| `Air Split Split Seek Bolt` | **3** homing Air bolts in a fan. |
| `Earth Earth Fuse Amplify Wall` | One Earth wall, power 3 (charge 2 + 1 amplify). |
| `Fire Bolt Trigger Fire Nova` | A fire bolt that **detonates into a fire nova on impact** (`depth() == 2`). |

Every row above is an assertion in `crates/spellvm/tests/integration.rs`, so the
documentation cannot silently drift from the engine.

---

## 7. Sigils: one spell, six handwritings

A `Sigil { style, nodes }` is a `Spell` laid out as geometry. `Style` is the
wizard's *handwriting*:

| Style | In-fiction author | Layout |
|---|---|---|
| `Circle` *(default)* | apprentice | Evenly spaced around one ring. |
| `Spiral` | journeyman | An inward spiral. |
| `Line` | researcher | A straight row (fits many spells on a page). |
| `Quarters` | master | Four arcs, packing related spells together. |
| `Semicircles` | household adept | Two stacked half-rings. |
| `Scattered` | dark wizard | Deterministically "random" — encryption without a key. |

The crucial invariant: **style is purely cosmetic.** `decode()` depends only on
the *order* of the nodes, so **every style of the same spell decodes to the exact
same program** — and therefore the same outcome. This is verified for all styles
across 200 pseudo-random programs in the integration suite
(`every_style_is_just_handwriting`, `property_random_programs_round_trip…`).

Geometry is asset-free: node positions land in the unit disc `[-1, 1]²` (the host
scales them), `Sigil::path()` gives the connect-the-dots casting order,
`Sigil::enclosure(segments)` gives the surrounding ring, and `Glyph::rune()`
returns procedural line-art (a star-polygon body plus a category mark) for each
glyph so even the runes need no textures. `Scattered` uses a deterministic
`hash01` of the node index, so an "encrypted" grimoire is still reproducible.

---

## 8. The `Vec2` boundary

`spellvm` ships its own `math::Vec2` rather than depend on `glam`, keeping the
zero-dependency promise. The Bevy side converts at the seam with a one-line
bridge (`src/prelude.rs`):

```rust
pub fn sv2(v: spellvm::Vec2) -> Vec2 { vec2(v.x, v.y) }
```

`spellvm` is also imported item-by-item on the game side (not glob-imported) so
its `Vec2` / `vec2` / `Node` never clash with Bevy's own. This is the *only*
friction the independence costs, and it is contained to one file.

---

## 9. Host integration (the Bevy side)

The engine-specific half lives under `src/game/` and `src/grimoire/` and consumes
the contract above:

* **`grimoire::book`** — the player's `Book` resource: the draft being inscribed,
  the equipped spells (slots 1–9), and the known glyph alphabet.
* **`grimoire::editor`** — the in-world overlay: click glyphs to build a draft,
  see its sigil and a live `summarize` readout, then Equip. This is the
  "make ANY circle as a developer" surface — any sequence of `Glyph::ALL` is
  authorable.
* **`grimoire::render`** — the single place that turns a `Sigil` into gizmos
  (enclosure ring + node path + per-node dots and runes).
* **`game::cast`** — reads the fire button, compiles the equipped `Spell` to a
  `SpellOutcome`, and hands it to the projectile layer.
* **`game::projectile`** — the simulation: one generic `Effect` component carries
  the whole `SpellComponent`; systems branch on `comp.form`, so adding a form is
  a localised change. Handles travel, homing (`Seek`), bouncing (`Ricochet`),
  piercing, ward-follow, collision, and — on death — spawning the `on_impact`
  nested outcome (recursion realised).
* **`game::vfx`** — all visuals are gizmos coloured past `1.0` for bloom; on each
  cast it blooms the spell's sigil, oriented to the aim direction.

The arrow of dependency points one way only: the game depends on `spellvm`;
`spellvm` depends on nothing in the game. You can delete the entire `src/` tree
and the VM still compiles, tests and ships.

---

## 10. Extending the system

Because the VM is small and exhaustive `match`es are compiler-checked, growth is
mechanical and safe:

* **New element** — add a variant to `Element`, extend `fuse`, `name`, `color`,
  and the `ALL`/`BASES` tables. The compiler flags every `match` that needs a new
  arm.
* **New form** — add a `Form` variant + a `Glyph`, handle it in
  `vm::finalize_form` and in the game's `spawn_component` / `overlaps` /
  `draw_effects`. The VM half is engine-free and unit-testable on its own.
* **New modifier** — add a `Modifier` + `Glyph`, fold it into the `Pending`
  accumulation and into `SpellComponent`. Position-independence comes for free.
* **New style** — add a `Style` variant and a layout function. The decode path is
  unchanged (it only reads node order), so the round-trip invariant holds
  automatically — and the property test will prove it.

Each of these is a few lines in one or two files, with tests as the spec.

---

## 11. Testing strategy

The VM is the most-tested part of the project precisely because it is pure:

* **Unit tests** (in each module) — fusion algebra, glyph parsing, VM stepping,
  modifier accumulation, error paths, sigil round-trips, `Vec2` math.
* **Integration tests** (`tests/integration.rs`) — drive only the public API, and
  double as executable documentation of the worked examples above (incl. the
  developer "author any circle programmatically" path).
* **Property-style checks** — 200 deterministic pseudo-random programs must
  survive encode→decode in every style unchanged, and decode to an identical
  outcome.
* **Doctests** — the README and `lib.rs` examples are compiled and run.
* **`serde` round-trips** — behind the optional feature, `Spell`/`SpellOutcome`/
  `Sigil` survive JSON serialisation, so saved spells are forward-safe.

```sh
cargo test -p spellvm                    # core suite (50 tests)
cargo test -p spellvm --features serde   # + serde round-trips (6 more)
```

This is why a balance change or a new glyph can be made with confidence: the
table of worked examples *is* the test suite, and the geometry invariant is
machine-checked across the whole glyph space.

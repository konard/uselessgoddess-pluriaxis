# Pluriaxis

> A top-down action playground where wizards write spells as **sigils** — points
> connected by lines inside a circle — and that hand-drawn notation is *literally*
> a program for a small virtual machine. A sigil is source code. The circle is
> just the page.

Built on **[Bevy 0.19](https://bevy.org/news/bevy-0-19/)**, in Rust. Almost no
art assets: every spell, sigil and effect is drawn procedurally with gizmos and
made to glow by an HDR + bloom camera.

```text
  text / clicks ─▶ Spell ─▶ [ spellvm ] ─▶ SpellOutcome ─▶ Bevy spawns effects & VFX
                     ▲                                            (projectiles, novas,
              a Sigil is the same                                  wards, walls, beams)
              program drawn in a circle
```

---

## The idea

In the fiction this game is based on, a wizard inscribes a spell as **points
connected by lines inside a circle**. Each point is a glyph (an opcode); the
order they are visited is a program for a tiny
[concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language)
stack machine. That one conceit gives the game its depth: because a sigil encodes
a *linear sequence of instructions*, the **same spell can be written in many
styles** — an apprentice's plain circle, a journeyman's spiral, a master's four
quarters, a dark wizard's scattered cipher — all decoding to the same program.

You don't pick spells from a menu. You **invent** them: compose glyphs in your
grimoire, watch the sigil take shape, and cast it with your own hands. The fun is
writing a fire bolt that bursts into a frost nova on impact and realising you can
do better.

---

## Quick start

```sh
# Run the playground (fast incremental dev build — Bevy is linked dynamically).
cargo run

# Run the engine-agnostic rules tests (milliseconds, no window needed).
cargo test -p spellvm
```

> **Linux build prerequisites.** This repo links with **LLD** (see
> `.cargo/config.toml`) and pulls in `libudev` transitively, so install:
> `lld` and `libudev-dev` (Debian/Ubuntu: `sudo apt-get install lld libudev-dev`;
> Arch: `sudo pacman -S lld systemd-libs`; Fedora: `sudo dnf install lld systemd-devel`).

### Controls

| Input | Action |
|---|---|
| `WASD` / arrows | Move |
| Mouse | Aim |
| `LMB` / `Space` | Cast the selected spell |
| `1`–`9` | Select an equipped spell |
| `Tab` | Open / close the **grimoire** (inscribe & equip spells) |
| `Esc` | Close the grimoire, or leave to the menu |

In the grimoire, click glyphs to inscribe a draft, watch its sigil and a live
readout of what it will do, then **Equip** it to a slot. Any sequence of glyphs
is authorable — make *any* circle you like.

---

## Workspace layout

A Cargo workspace with two members. The split is the project's core architectural
decision: **the rules engine knows nothing about the engine.**

```
pluriaxis/
├─ src/                     # the Bevy game (binary `pluriaxis`)
│  ├─ core/                 # camera (HDR+bloom), app states, system order, motion
│  ├─ game/                 # arena, player, targets, casting, effect sim + VFX
│  ├─ grimoire/             # the spellbook, the in-world editor, the sigil renderer
│  ├─ screens/              # title menu + playground frame/HUD
│  ├─ theme/                # palette, UI widgets, button feedback
│  └─ prelude.rs            # the one place the game ↔ spellvm boundary is bridged
├─ crates/
│  └─ spellvm/              # the spell virtual machine — zero runtime deps, no Bevy
├─ docs/design/             # gameplay + spell-system design documents
└─ .github/workflows/ci.yml # fmt · clippy · test · build
```

### `spellvm` — the spell VM (independent crate)

`crates/spellvm/` is a **dependency-free**, engine-agnostic stack machine: pure
*data in → data out*. It parses/builds a `Spell` (a list of glyphs), evaluates it
to a `SpellOutcome` (a description of what to spawn), and lays it out as a `Sigil`
(points-and-lines geometry that round-trips through any drawing style). It can be
unit-tested, fuzzed and reused entirely on its own, with **56 tests** that run in
milliseconds. See [`crates/spellvm/README.md`](crates/spellvm/README.md).

```rust
use spellvm::prelude::*;

let spell: Spell = "Fire Water Fuse Amplify Bolt".parse().unwrap();
let bolt = &spell.outcome().unwrap().components[0];
assert_eq!(bolt.element, Element::Steam); // Fire + Water
assert_eq!(bolt.power, 3);                // fused charge 2 + Amplify

// The same spell drawn as a circle decodes right back to itself.
assert_eq!(Sigil::encode(&spell, Style::Circle).decode(), spell);
```

The game side (`src/`) is the only part that knows about Bevy: it turns a
`SpellOutcome` into entities, simulates them (travel, homing, bouncing, piercing,
walls, wards, impact-triggered nested spells) and renders everything with glowing
gizmos.

---

## Design documents

* **[docs/design/gameplay.md](docs/design/gameplay.md)** — genre, pacing, the core
  loop, progression, combat, art direction, MVP scope and roadmap. Decision: a
  **Noita + Magicka hybrid** top-down action roguelite, because a programming-like
  spell system only earns its keep when the player *actively* uses it.
* **[docs/design/spell-system.md](docs/design/spell-system.md)** — the engineering
  spec for the `spellvm` VM: the instruction set, the element/fusion algebra, the
  outcome contract, sigil encoding, the host-integration boundary and how to
  extend it.

---

## Procedural graphics

There are effectively **no image assets**. The look comes from:

* **Gizmos** for all geometry — the arena, sigils, runes, projectiles, novas,
  beams, wards and walls are immediate-mode line/circle draws.
* **HDR + Bloom**: colours are emitted past `1.0` (`Color::linear_rgb(r, g, b)`
  with `r,g,b > 1`), and the camera's bloom turns that into glow. Each element
  carries its own colour, so the spell *is* the palette.
* The only embedded binary data is Bevy's tonemapping LUT (decoded at runtime via
  `ktx2` + `zstd`), so there are no asset files to ship.

---

## Building & profiles

| Command | What it does |
|---|---|
| `cargo run` | Dev build with **dynamic linking** for fast iteration. |
| `cargo run --no-default-features` | Statically-linked build (no `bevy_dylib`). |
| `cargo build --release` | Optimised build (`lto = "thin"`, one codegen unit). |
| `cargo build --profile ci --no-default-features` | The lean profile CI uses (opt-level 0, fastest to compile). |

Dev builds optimise dependencies once (`opt-level = 3`) and keep our own code at
`opt-level = 1`, so the game runs smoothly while staying quick to rebuild. See the
profile and tuning comments in [`Cargo.toml`](Cargo.toml).

## Continuous integration

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs on every push to
`main` and every pull request:

* `cargo fmt --all --check` — in its own near-instant job;
* `cargo clippy --workspace --all-targets -- -D warnings`;
* `cargo test --workspace` and the `spellvm` `serde` suite;
* a binary build of the game.

The heavy steps share one job (and a `Swatinem/rust-cache` cache) so Bevy is
compiled once per run rather than three times.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.

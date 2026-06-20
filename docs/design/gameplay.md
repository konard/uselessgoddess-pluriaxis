# Pluriaxis — Gameplay Design Document

> Status: living document describing the intended design. Where it refers to
> systems that do not yet exist, treat it as direction, not contract. The
> **MVP / Playground Scope** and **Roadmap** sections mark the boundary between
> what the first playable delivers and the longer vision.

---

## 1. Vision / Elevator Pitch

**Pluriaxis** is a top-down action roguelite about *writing* magic, not merely
selecting it. Every spell in the game is a tiny program. Wizards here do not pick
spells from a menu — they inscribe **sigils**: points connected by lines,
arranged in circular figures, that any literate mage can read aloud and cast. The
fiction's central conceit, and the mechanical heart of Pluriaxis, is that this
hand-drawn notation is *actually* a visual encoding of a program for a small
**stack-based virtual machine**. A sigil is source code. The circle is just the
page it is written on.

That single idea unlocks unusual depth. Because the notation encodes a *linear
sequence of instructions*, the **same spell can be drawn in many geometric
styles**. An apprentice writes it as a plain circle. A journeyman prefers elegant
spirals. A master splits the ring into four quarters to pack four spells onto one
figure. A dark wizard scatters the points into apparent noise to encrypt their
grimoire from rivals. A researcher runs the program down a straight line to fit
dozens of experiments on a single page. And the archmage, having seen every
style, returns to the simple circle — *no point fixing what isn't broken.* All of
them decode to the **same glyph program**. The style is craft, identity, and
(eventually) a measurable mastery tier; the program underneath is the truth.

Pluriaxis wraps this toy in deliberate, active, replayable combat. You explore,
discover new **glyphs** (the instruction alphabet), compose them into sigils in
your grimoire, and cast them with your own hands against procedurally themed
enemies. The fun is not in finding the "best build" handed to you — it is in
*inventing* a spell nobody told you was possible, watching a fire arrow you wrote
detonate into a frost nova on impact, and realizing you can do better. Pluriaxis
is a programming language disguised as a spellbook, disguised as a game.

---

## 2. Genre & Pacing

The open question was which reference fantasy to build toward. Three obvious
candidates, each a different relationship between the player and the act of
casting:

| Reference | Casting model | Pacing | What it optimizes | Cost for us |
|---|---|---|---|---|
| **Vampire Survivors** | Fully automatic; weapons fire themselves | Very fast, dense swarms | Build *assembly* and dopamine; zero per-cast thought | Our spell system would be invisible — you'd never *use* the language, only stack passives |
| **Noita** | Active; wands are programs you assemble, fired manually | Medium-fast, lethal, emergent | Combinatorial depth, "I can't believe that worked" moments | High lethality + physics chaos can bury a readable spell-authoring fantasy |
| **Magicka** | Active; you compose elements live, per cast, from a small alphabet | Frantic but intentional | *Concatenative casting* as the core verb; discovery via combination | Pure live-typing is hard to balance solo and at speed |

**Decision: a hybrid leaning Noita + Magicka — a top-down twin-stick action
roguelite spellcaster.**

The rationale follows from the pitch. Our spell system is a deep, combinatorial,
*programming-like* toy, and a toy like that earns its keep only when the player
**deliberately and actively** uses it. The Vampire Survivors model would reduce
the entire VM to a passive number-stacking exercise — you would never read or
write a sigil, only collect upgrades that fire on their own. That throws away the
star of the show.

So we take the **authoring depth of Noita** (spells are real programs, including
nested "on-impact" sub-spells) and the **concatenative, element-combining,
discovery-driven grammar of Magicka** (a compact glyph alphabet whose power comes
from combination, with fusions you find by experimenting). We keep **runs snappy
and roguelite-replayable**: short, intense, restart-friendly, with
meta-progression between attempts.

Crucially, the Vampire Survivors fantasy stays **reachable as one playstyle among
many**. The **Orbit** form spawns spell components that circle you and strike
automatically, and an opt-in **auto-cast** binding can fire a chosen sigil on a
cooldown. A player who wants to lean back and watch the swarm melt can build
toward exactly that — they simply *write* an orbiting, auto-firing sigil, rather
than having automation imposed on them. Active, authored casting is the default;
automation is a build you choose.

---

## 3. Core Loop

### Minute-to-minute (the action loop)

Twin-stick controls: **left stick / WASD** moves, **right stick / mouse** aims.

1. **Move & position** — top-down arena movement; spacing and line-of-sight
   matter because most Forms are directional.
2. **Aim** — your active sigil points where you aim.
3. **Cast** — fire the currently equipped sigil. Each cast runs its glyph
   program on the spell VM and produces real effects in the world.
4. **Dodge & manage** — a dash for repositioning, plus mana economy: casting
   draws from a mana pool that regenerates, so spamming your most expensive sigil
   has a rhythm and a cost.
5. **Swap** — you carry several prepared sigils on a quick-select; switching the
   active sigil is part of moment-to-moment play (burst sigil vs. zoning sigil vs.
   defensive Ward).

The skill ceiling lives in *which sigil, aimed where, at what moment* — and, one
layer up, in *what you wrote into those sigils before the fight.*

### Run-to-run (the roguelite loop)

`Explore → Discover glyphs → Craft / edit sigils → Fight & grow power → Boss /
floor end → Death or Victory → Meta-progression → (repeat)`

- **Explore** procedurally themed floors/arenas.
- **Discover glyphs** as drops from enemies/chests and as level-up rewards.
- **Craft & edit sigils** at safe moments (between rooms, at a grimoire bench).
- **Grow power** within the run by adding Modifiers, longer programs, and newly
  discovered Fusions.
- **Death or Victory** ends the run; either way you bank meta-progression that
  carries forward (see §5).

---

## 4. The Spell System as the Star

### The big idea, in player-facing terms

A **sigil** is a drawing: dots (we call them *nodes*) joined by lines inside a
circular figure. Read it in order and it spells out a sequence of **glyphs**.
Each glyph is one instruction. Feed that sequence to the **spell VM** — a small
**stack machine** — and you get a spell. That's it. The pretty circle is a
*program listing*, and casting is *running the program*.

Why a stack? Because it makes "concatenative" casting natural. **Elements push
ingredients onto a stack. Operators and Forms pop them off.** You read
left-to-right (around the ring), each glyph acts on whatever the previous glyphs
left behind. This is the same mental model as a calculator that works in postfix,
or a Forth program, or assembling a Noita wand — and it is *exactly* what makes
short sigils expressive.

### Reading and writing a sigil

- **Reading.** Start at the sigil's mark (the entry node) and follow the lines.
  Each node you pass is one glyph. The order is the program order.
- **Writing.** In the grimoire, you assemble a glyph sequence; the game *draws*
  the sigil for you in your current **style** (see below). You are authoring the
  underlying program; the geometry is rendered from it. (Advanced players will be
  able to read foreign sigils by eye — and dark wizards will make that hard on
  purpose.)

### The glyph alphabet

Glyphs are grouped by what they do to the VM stack.

| Category | Glyph | VM behavior | Notes |
|---|---|---|---|
| **Element** | Fire | push Fire | raw ingredient |
| **Element** | Water | push Water | |
| **Element** | Earth | push Earth | |
| **Element** | Air | push Air | |
| **Element** | Aether | push Aether | raw mana / "neutral" ingredient |
| **Operator** | Fuse | pop 2 ingredients → push 1 combined | the discovery engine (see fusion table) |
| **Form** | Bolt | consume 1 element → projectile | the workhorse delivery |
| **Form** | Beam | consume 1 element → continuous ray | |
| **Form** | Nova | consume 1 element → radial burst | self-centered AoE |
| **Form** | Orb | consume 1 element → orbiting body | the "VS" auto-hitter |
| **Form** | Ward | consume 1 element → self shield | defensive |
| **Form** | Wall | consume 1 element → placed barrier | zoning / control |
| **Modifier** | Amplify | boost power of the current spell component | stacks |
| **Modifier** | Split | increase multicast count | more projectiles / instances |
| **Modifier** | Reach | increase range | |
| **Modifier** | Linger | increase duration | great on Orb / Wall / status |
| **Modifier** | Pierce | pass through targets | |
| **Modifier** | Seek | homing | |
| **Modifier** | Ricochet | bounce between targets/surfaces | |
| **Control** | Trigger | the **rest of the program** becomes an on-impact sub-spell | enables nesting / recursion |

Two structural rules give the system its range:

1. **Multiple Forms = multiple simultaneous spell components.** A program with
   two Forms produces two spells at once; four Forms, four spells. This is the
   logical machinery behind "two semicircles = a pair" and "four quarters = four
   spells."
2. **Trigger nests programs.** Everything after a `Trigger` is packaged as a
   sub-spell that runs when its parent Form impacts — the Noita-style recursive
   "cast a spell on hit." Sub-spells can themselves contain Triggers.

### The fusion table (Magicka-style discovery)

`Fuse` pops the top two ingredients and pushes their combination. Players
**discover** these by experimenting; the codex fills in as they do. A starting
set:

| Combine | Result | Flavor |
|---|---|---|
| Fire + Water | Steam | obscuring, scalding |
| Fire + Earth | Lava | burning ground |
| Fire + Air | Smoke / Flare | spreading flame |
| Water + Air | Ice | freeze / slow |
| Water + Earth | Mud | slow / root |
| Earth + Air | Sand | abrasive, blinds |
| Air + Aether | Storm | chaining lightning |
| Fire + Aether | Plasma | high raw damage |
| Water + Aether | Frost-mana | strong slow + chip |
| Earth + Aether | Crystal | hardened projectile / shield |

> Fusions are **data**, not hardcoded special cases — new fusions (and reactions
> between *results*, e.g. Steam + Air) can be added without touching the VM.

### Worked examples

Each example is a glyph sequence (the program) and its decoded effect; the notes
show how the VM consumes ingredients off the stack.

- **`Fire Bolt`** — the first spell every apprentice learns. Push Fire → `Bolt`
  consumes it. A single fire projectile; stack ends empty.
- **`Fire Water Fuse Bolt Amplify`** — push Fire, push Water, `Fuse` → Steam.
  `Bolt` consumes Steam → a steam projectile. `Amplify` boosts that component's
  power. One strong steam bolt.
- **`Fire Bolt Trigger Fire Nova`** — a fire arrow that explodes. `Fire Bolt`
  makes a fire projectile; `Trigger` declares *the rest is an on-impact
  sub-spell*; `Fire Nova` is that sub-spell. The arrow detonates into a radial
  fire burst where it lands.
- **`Water Air Fuse Nova`** — a frost nova. Push Water, push Air, `Fuse` → Ice.
  `Nova` consumes Ice → a self-centered ring of frost that slows everything
  around you.
- **`Fire Bolt Air Bolt`** (two Forms) — a twin shot drawn as **two
  semicircles**. The first `Fire Bolt` fires a fire projectile, the second fires
  an air projectile: two components from one cast, exactly the "pair" the
  two-semicircle style depicts.

### Encoding styles — cosmetic *and* functional

The style is **how the same program is laid out as geometry**. Every style
decodes to an identical glyph sequence. Styles differ in node layout, line
shapes, and — for some — in *capacity and legibility*, which is what makes them
functional rather than purely decorative.

| Style | Geometry | Functional role |
|---|---|---|
| **Circle** | Nodes evenly on one ring | Baseline, maximally legible. The default. |
| **Spiral / curved** | Curved/spiraling lines | Denser layout; fits a longer single program on the page. |
| **Two semicircles** | Ring split into two arcs | Reads as two spell components — a natural "pair." |
| **Four quarters** | Ring split into four arcs | Four components on one figure — four spells at once. |
| **Scattered** | Points appear randomly placed | *Encrypted.* Hard for others to read; protects your grimoire. |
| **Straight line + curves** | Program runs along a line | Maximum capacity per page; the researcher's notebook. |

These styles are not just art: the geometry **carries logical structure** (arc =
component boundary) and **trades legibility for density or secrecy**. That is the
bridge to progression.

---

## 5. Progression & Discovery

### How you learn glyphs

The instruction alphabet is the progression. You acquire glyphs three ways, and
each maps to a different kind of fun:

1. **Found in the world** — glyphs drop from enemies, chests, and shrines on a
   floor. Exploration is rewarded with new vocabulary.
2. **Earned on level-up** — leveling within a run offers a choice of new glyphs
   or upgrades, the standard roguelite "pick one of three" beat.
3. **Unlocked by experimentation** — **Fusions** are discovered by *trying*
   combinations. The first time you `Fuse` Water and Air and see Ice, the codex
   records it. Whole branches of the spell space open only because you were
   curious.

### Inventing your own spells is the core fun

Pluriaxis deliberately does **not** hand you finished spells. It hands you an
*alphabet* and a *machine that runs it*. The dopamine is authorship: stacking a
`Trigger` onto a fused bolt and discovering you've built a delayed explosion; or
chaining `Split` and `Seek` and watching a fan of homing missiles fold inward.
The "build" is something you *wrote*, not something you looted. This is the
Magicka/Noita lineage taken seriously: the design's job is to make a small,
combinable alphabet that yields surprising, legible, exploitable results.

### The grimoire (discovery / codex metaphor)

Your **grimoire** is both the codex and the editor:

- **Codex pages** record every glyph you know and every Fusion you've discovered
  (with the combinations that produce it).
- **Spell pages** are your saved sigils — authored programs, rendered in your
  current style, equippable to your quick-select.
- Discovery is *visible progress*: empty slots in the codex are an invitation to
  go experiment.

### Styles as mastery tiers — the wizard-rank narrative

The encoding **styles are themselves unlockable**, and they double as a mastery
ladder that mirrors the wizard ranks of the source fiction. You begin only able
to *write* in the apprentice circle; higher ranks unlock richer layouts (and the
ability to *read* what they encode).

| Rank | Style unlocked | What it grants | Narrative |
|---|---|---|---|
| **Apprentice** | Circle | Basic single-program sigils | "Start with the ring everyone can read." |
| **Journeyman** | Spiral / curved | Longer programs per sigil | Elegance and density; the craftsman's hand. |
| **Adept** | Two semicircles | Paired components, authored deliberately | First taste of multi-spell sigils. |
| **Master** | Four quarters | Four components on one figure | Pack four spells into a single cast. |
| **Dark wizard** | Scattered / encrypted | Conceal sigils; read enemy encryptions | Secrecy as power; decode rival grimoires. |
| **Researcher** | Straight line + curves | Maximum capacity per page | The lab notebook — many experiments at once. |
| **Archmage** | Circle (mastered) | Bonuses while writing in the humble circle | "No point fixing what isn't broken." |

Progressing the rank ladder is a meta-progression axis distinct from the
in-run glyph collection: glyphs are *vocabulary*, styles are *handwriting*, and
the archmage's return to the circle is the design's wink at mastery — once you
understand the program beneath every style, the simplest page is enough.

---

## 6. Combat & Enemies

Combat exists to give the spell language something to chew on; keep it readable
and reactive rather than bullet-hell-opaque.

- **Procedurally themed enemies.** Enemies are assembled from themes (e.g.
  *ember*, *tide*, *grove*) that drive palette, movement, attacks, and —
  importantly — **elemental affinity**, keeping variety high with few bespoke
  assets.
- **Elemental weaknesses & resonances.** Each enemy resists some elements and is
  vulnerable to others, and *fused* results matter: an icy foe shrugs off Water
  but shatters under Lava; a storm-touched flier resists Air but is grounded by
  Earth/Sand. This rewards **thoughtful element choice** and makes Fusion
  tactically relevant, not cosmetic.
- **Resonance combos.** Applying an element, then a reacting one, triggers a bonus
  (soak a target in Water, then hit with Storm for a chain). The same reaction
  logic that powers Fusions powers these on-target combos.
- **Encounters as puzzles at speed.** A room mixing affinities pushes the player
  to swap sigils — the burst sigil for the vulnerable mob, the Wall to zone the
  resistant one — exactly the decision the active casting model is built to
  reward. Bosses (later milestones) are theme capstones with multi-phase affinity
  shifts that force re-authoring or swapping sigils mid-fight.

---

## 7. Art Direction

**Guiding direction (not a hard requirement yet): minimum texture assets,
maximum procedural graphics and effects.** Pluriaxis is, at its core, *lines and
points and light* — a gift for a procedural renderer.

- **Procedural runes & sigils.** Sigils are drawn from their **line geometry** at
  runtime. Because a sigil is generated from its program, the renderer can lay out
  nodes and stroke connecting lines for *any* style (circle, spiral, quarters,
  scattered, line) without authored sprites. The notation *is* vector art.
- **Particle-driven elemental VFX.** Fire, Water, Air, Earth, Aether and their
  Fusions are expressed through **particle systems** and motion — sparks, mist,
  dust, arcs — not texture sheets.
- **Shader-based glows & SDF shapes.** Use **signed-distance-field** shapes for
  crisp scalable glyph marks, bloom/glow for energy, and shader animation for
  charge/impact. Assets stay tiny and resolution-independent.
- **Readable, high-contrast palette.** Color is *information*: an element's hue,
  an enemy's affinity, a sigil's style should read at a glance.

This direction keeps the project lean (few binary assets, fast iteration) and
plays directly to the fiction — magic here literally *is* geometry and light. It
is a **target**, not a constraint; pragmatic textures are fine where procedural
generation isn't worth the cost yet.

---

## 8. MVP / Playground Scope

The first playable (this PR) is a **playground**, not a full game. Its job is to
prove the fantasy: *a character you run around, casting spells you composed from
glyphs, rendered with procedural VFX.*

**In scope for the MVP playground:**

- A **top-down arena** with a controllable character (twin-stick / WASD + mouse).
- A **working slice of the spell VM**: a stack machine that executes glyph
  programs for at least the core set — Elements (Fire/Water/Air, plus Aether),
  `Fuse`, the workhorse Forms (Bolt, Nova, and one of Orb/Ward), and a few
  Modifiers (Amplify, Split).
- **Active casting**: equip a sigil, aim, and cast; spells appear in the world.
- **Procedural VFX** for elements and forms (particles + simple shader glow),
  proving the "no texture sheets" direction.
- A **compose/preview surface**: a minimal way to assemble a glyph sequence and
  **see the resulting sigil drawn** (at least the **circle** style) and preview
  its effect by casting it.

**Explicitly out of scope for the MVP (deferred to the vision):**

- Full enemy roster, affinities, and combat tuning.
- Roguelite run structure (floors, drops, level-up choices).
- Meta-progression and the full rank/style ladder.
- All six encoding styles (MVP needs only the circle to validate the concept).
- Codex/grimoire persistence and the full editor UX.

The MVP is the smallest thing that lets a player *write a spell and cast it* —
the irreducible core that everything else decorates.

---

## 9. Roadmap

A phased path from playground to game; each phase ends on something playable.

- **Phase 0 — MVP Playground (this PR).** Top-down character, core spell VM slice,
  active casting, procedural elemental VFX, and a compose-and-preview surface
  rendering the circle sigil. *Goal: the core fantasy is real and fun for five
  minutes.*
- **Phase 1 — Spell Editor UI.** A proper grimoire: build/edit/save sigils, browse
  the alphabet, preview the rendered sigil and its decoded effect, equip to a
  quick-select; plumbing so the codex records discovered Fusions. *Goal: authoring
  is first-class.*
- **Phase 2 — Enemies & Combat.** Procedurally themed enemies, elemental
  weaknesses/resonances, on-target reaction combos, mana economy and dash tuning.
  *Goal: the language has worthy targets.*
- **Phase 3 — Roguelite Run Structure.** Floors/arenas, glyph drops, level-up
  choices, run start/end, first boss. *Goal: a real run you can win or lose.*
- **Phase 4 — Meta-progression & Style Ladder.** Persistent unlocks between runs;
  the wizard-rank ladder unlocking encoding styles (spiral → semicircles →
  quarters → scattered/encrypted → researcher line → archmage circle), each with
  its functional payoff. *Goal: long-term reasons to return.*
- **Phase 5 — Polish.** Audio, juice, accessibility, balance passes, additional
  themes/Fusions/Forms, performance. *Goal: ship-quality feel.*

---

## Design Principles

- **The program is the truth; the circle is the page.** Every visual is a view of
  an underlying glyph program — never the other way around.
- **A small alphabet, deep combinations.** Power comes from composing few glyphs,
  not from a sprawling spell list. Don't add a glyph that a combination already
  expresses.
- **Authorship over loot.** The best spell in any run is the one the player wrote,
  not the one the game dropped fully-formed.
- **Active by default, automatable by choice.** Deliberate casting is the core
  verb; the lean-back (Orbit/auto-cast) fantasy is a build you opt into.
- **Discovery is content.** Experimentation (especially Fusions) is a first-class
  source of progress; the codex makes the unknown visible and inviting.
- **Data, not special cases.** Fusions, reactions, themes, and styles are data the
  VM and renderer consume — extend the game without rewriting its core.
- **Procedural first.** Prefer geometry, particles, and shaders over texture
  assets; the fiction is lines and light, so let the renderer speak it.
- **Legibility is a feature.** Sigils, elements, and affinities should be readable
  at a glance — except where the *dark wizard* makes illegibility the point.

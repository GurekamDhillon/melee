# Brawl boss for Envoy: Petey Piranha (plan)

Status: planning only. Nothing built, no game runs, no Brawl disc touched while writing this.
Target: the boss of Envoy milestone 5 (`docs/rust-port/notes/envoy-plan.md`, branch `port/envoyplan`).
Facts marked **[V]** are well-known public knowledge; **[?]** must be verified by observation on the
player's own disc/console (or with BrawlCrate on our own dumps) before the numbers are trusted.

## 1. Which boss first

User decision: **Petey Piranha.** Porky was the starting idea but needs an auto-scrolling arena.

| Boss | Assets | Behaviour | Arena | Fit for a Melee-engine Envoy run |
|---|---|---|---|---|
| **Petey Piranha** | one skeletal model + 2 cage props, small anim set | ~6 attacks, two win conditions (HP or break a cage) | flat stage, fixed camera | best: co-op rescue fits Envoy's party, small slice |
| Porky (Statue) | big model, many parts, projectiles/minions | multi-attack, mobile | **scrolling arena**, so a new stage system | later, after the scrolling-stage work |
| Rayquaza | large model, water/air effects | flies off-stage, lasers | water plus floating platforms | medium, but camera/effects heavy |
| Galleom / Duon | mid-size mech, Duon has two modes | medium (Duon is two bosses in one) | flat | good second/third boss |
| Tabuu | wings, many effect-heavy attacks | complex (Off Waves) | void arena | final boss, last |

## 2. Petey's fight as reimplemented (observed behaviour, public sources) [V unless marked]

- Setting: the Midair Stadium opening of the Subspace Emissary. Petey holds **one cage in each hand**
  (Peach and Zelda in the original). The player wins by draining Petey's HP **or** by breaking one cage
  (the freed character joins; the other cage's occupant is lost). Cages take damage separately; cage
  HP and Petey HP values are **[?]**.
- Hurtbox: body/head is the main target; the cages are separate hurtboxes that also hit players while
  swung. Petey takes damage as HP, not percent, and does not flinch from normal hits (super armour).
- Attacks (names are ours, timings **[?]**): (a) cage slam: lifts both cages, slams them down;
  (b) cage sweep: horizontal swing across the stage; (c) belly flop/jump stomp with shockwave;
  (d) spin/tornado across the stage; (e) mouth bite/lunge forward; (f) short stun when hit hard in the
  head (a punish window). Each attack has a clear wind-up telegraph.
- Phases: one main loop; at low HP he speeds up and chains attacks (we add a tunable phase-2 cutoff,
  e.g. 50% HP).

Envoy's twist (proposal): the cages hold **party allies or unlocked characters**. Breaking a cage
recruits that character into the run (or revives a fallen party member in co-op); winning on HP
frees both.

## 3. Asset path (player's own disc, first run)

- Brawl stores data as `.pac` archives (ARC containers, often compressed) holding **BRRES**: MDL0
  (model + skeleton), TEX0/PLT0 (textures), CHR0 (bone animation), SRT0/PAT0/CLR0/VIS0 (material,
  texture-swap, colour and visibility animation). Effects are REFF/REFT; sounds live in `smashbros_sound.brsar`
  (RSAR/RWSD/RWAV, DSP-ADPCM). Boss files are in the enemy/boss directories of the disc filesystem;
  the exact Petey file name is **[?]** (look it up in BrawlCrate on a personal dump).
- Open tools/docs to read *formats* from: BrawlBox/BrawlCrate (C#, file-format reference), the
  Custom Mario Kart / Wiibrew wikis on BRRES, VGMStream for BRSAR audio. We read their docs as format
  specs; no game code is copied.
- Decompilation: a Brawl decomp project exists publicly but is early/partial; we do **not** use it
  (clean-room rule, section 6).
- Pipeline: reuse the `--brawl-disc` first-run extractor from the Meta Knight work (`firstrun.rs`
  on `port/ygpack`, `ports/halberd`, `ports/ir`). Add a Petey job: unpack ARC, decode BRRES, write a
  cached Geno-form pack under the user data dir.
  - Converts cleanly to HSD/Melee rendering: MDL0 meshes, single-bind and weighted skinning, TEV-style
    materials that map to HSD's simple modes, TEX0 (same GX texture formats, CMPR/I4/RGB5A3 etc.),
    CHR0 to our joint animation (resample to 60 fps keys).
  - Needs a new path in `melee_gx`: MDL0 shader stages beyond what HSD material objects express
    (multi-stage TEV), SRT0/PAT0 material animation, and REFF effects (start with hand-built Melee
    effects instead; port REFF later).
  - Audio: decode RWAV DSP-ADPCM into our audio bank at extraction.
- YouGame dependency: shipped builds need YouGame's **second-disc** feature request (player supplies
  a Brawl image beside Melee). Until then only local builds with `--brawl-disc` get Petey; Envoy
  without a Brawl disc falls back to Master Hand/Crazy Hand as the boss.

## 4. Behaviour in our engine

- Template: `rust/melee_chara/src/chars/ftMasterHand.rs` (Master Hand is a fighter kind with its own
  action states, HP mode and AI-driven attack picker). Petey is a new boss fighter kind `ftPetey`
  following that shape, plus two child items for the cages (item entities with their own HP and
  hurtboxes, parented to his hand joints).
- State machine: `Idle -> Choose -> Telegraph(attack) -> Active -> Recovery -> Idle`, `Stunned`,
  `Phase2` speed multiplier, `CageBroken(side)`, `Defeated`. Hitboxes/hurtboxes are authored as data
  (a `petey.json` table: per-attack frames, bone, radius, damage, angle, knockback), not from the disc.
- Determinism/rollback: all choices come from the match RNG seeded per-run; state lives in the fighter
  and item structs that rollback already snapshots; no wall-clock, no float order differences outside
  the existing engine paths. Hash oracle checks cover the new kind.
- Co-op: target selection picks the nearest/highest-damage player with seeded tie-breaks; Boss HP
  scales with party size (e.g. HP x (1 + 0.6 x (players-1)), tunable); cage recruitment goes to the
  player who dealt the breaking hit or to the party pool (open question).

## 5. Arena

A flat stage is enough: build "Midair Stadium"-style ground with the Blender parametric platform
pipeline (one wide main platform, blast zones sized for a large boss, fixed/slow camera that keeps
Petey framed). No Brawl stage conversion needed.

## 6. Legal / asset policy

No Brawl code or decompiled code is copied or consulted; behaviour is reimplemented from public
documentation and observation. Assets are read at runtime from the player's own Brawl disc, cached
locally, never shipped, never committed (same as Brawl characters). Attack data tables are our own
authored numbers.

## 7. Milestones (smallest playable slice first)

| # | Slice | Size | Risk | Model |
|---|---|---|---|---|
| P0 | Locate Petey files on a personal dump with BrawlCrate; list MDL0/CHR0/TEX0 contents (user-run, no agent disc access) | S | file names | Sonnet |
| P1 | Extractor job: Petey MDL0+TEX0+CHR0 to Geno pack; render idle in a test viewer | M | skinning/TEV gaps | Opus |
| P2 | **First playable**: `ftPetey` on a flat Blender arena, idle + cage slam + HP bar + defeat | M | boss camera, armour | Opus |
| P3 | Cages as items with HP; break-a-cage win; ally recruit into Envoy | M | item parenting | Opus |
| P4 | Remaining attacks, telegraphs, phase 2, stun window | M | balance | Sonnet (data), Opus (states) |
| P5 | Co-op scaling, rollback/netplay test, hash oracle | M | desync | Opus |
| P6 | Sounds (BRSAR), effects (Melee stand-ins, then REFF) | M-L | REFF renderer | Opus |
| P7 | Envoy M5 integration, no-disc fallback to Master Hand | S | — | Sonnet |

UI (boss HP bar, cage bars) is Opus.

## 8. Open questions

1. Who goes in the cages: party members (rescue), unlockable characters (recruit), or both?
2. Breaking a cage: does the other occupant become lost (Brawl-like) or both freed?
3. Is HP scaling with party size OK, and should cage rescue reward the breaker or the party?
4. Phase 2 at 50% HP, or keep Brawl's single loop?
5. Arena look: recreate Midair Stadium in the parametric pipeline, or an Envoy-original stage?
6. Without a Brawl disc, is Master Hand/Crazy Hand the right fallback boss?
7. Effects: accept Melee-style stand-in effects for the first release, deferring REFF?
8. Is Petey also a Vs/LAB-selectable boss outside Envoy?
9. Who runs P0 on a personal Brawl dump (agents must not touch Brawl discs)?

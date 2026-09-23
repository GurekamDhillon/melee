# Geno - GD's Melee's fighter-extension layer

Status: **v0** (foundation), private branch `private/geno` in the melee fork. Working name, approved by GD.
Nothing here may reach a public branch until GD says so.

## 1. What Geno is, and what it is not

Geno is the native layer for **new fighter content** in GD's Melee. Its first customers are a Brawl
Kirby port and then Meta Knight, which need character abilities m-ex cannot express: more jumps than
Melee's multi-jump table, gliding, crawling, wall clinging, new specials, and script logic
(variables and if/else) of the kind Brawl's PSA scripts use.

**Scope rule (GD):** Geno adds **character-level** abilities *inside Melee's rules*. It does **not**
import Brawl's engine-wide mechanics. In particular Geno will never add:

- tripping,
- Brawl's non-directional air dodge (or any change to Melee's air dodge / wavedash physics),
- hitstun cancelling, or any change to Melee's hitstun / knockback formulas,
- Brawl's ledge rules (ledge-grab invincibility, auto-sweetspot, ledge trump) or momentum rules.

A Brawl fighter ported through Geno plays by Melee's physics, hitstun, shields, ledges and tech;
only what belongs to *that fighter* (its numbers, its moves, its unique movement options) comes
across. Anything global stays exactly as Melee (and m-ex) define it.

## 2. Layering

```
  +--------------------------------------------------------------+
  | Geno native layer (pc/geno/, pc/platform/geno_*)              |  opt-in per fighter, from
  |   registry (geno.json, stable ids), state block, script      |  mods/<id>/geno.json
  |   escape, native hooks, features (attributes, multi-jump...) |
  +--------------------------------------------------------------+
  | m-ex compatibility layer (pc/platform/gw_mex_*) - UNCHANGED  |  every m-ex disc / split mod
  |   MxDt tables, dense slots, 46 Arch_FighterFunc slots,       |  (ACE, Akaneia, ...) behaves
  |   ftFunction interpreter + bridge                            |  exactly as before
  +--------------------------------------------------------------+
  | retail engine (decomp, gwtool-retargeted)                    |
  +--------------------------------------------------------------+
```

Geno sits *on top* of m-ex: a Geno overlay can attach to a vanilla fighter or to an m-ex fighter,
and its dispatch points run after m-ex's (e.g. Geno's on_frame runs after the m-ex onFrame
dispatch). Geno never edits m-ex's tables, slot numbering or hook registration.

## 3. Compatibility contract

1. **m-ex content is unchanged.** Every engine call site Geno adds is under `TARGET_PC` and returns
   immediately for a fighter with no Geno profile whose scripts never used the escape. Verified:
   test suite 133/133 on ACE, Akaneia and vanilla; all 157 ACE netplay identities (fighters and
   stages) bit-identical to the pre-Geno build; ACE fighter crash sweep A/B against the pc-port
   exe (see the report for the exact numbers).
2. **Geno is opt-in per fighter / per mod.** Only a `geno.json` in a *mounting* mod activates
   anything. No `geno.json` anywhere = Geno is inert (test `geno_registry_empty_is_inert`).
3. **The script escape collides with nothing shipped.** Opcode 59 is past the end of the retail
   command tables; `pc/geno/tools/scan_ftcmd_opcodes.py` walked every fighter script on vanilla,
   ACE 2.0, Akaneia, TM-CE and 20XX (138,365 commands on ACE alone): highest opcode used is 58 on
   every disc. Opcodes 60-63 keep their retail behaviour (out-of-table).
4. **m-ex's dense slot numbering is untouched.** Geno identifies fighters by Pl file name and by
   content-derived ids; it resolves "which FighterKind is PlSh.dat on this install" at boot, so a
   Geno overlay follows an m-ex fighter wherever the enabled set puts it.
5. **Netplay.** A Geno overlay changes how its target plays, so the overlay's id is mixed into the
   *target fighter's* content identity (`gw_mexid.c`) - only when an overlay exists; every other
   identity and the global hash are unchanged. Two players with different overlays for Kirby see
   Kirby greyed out for each other, exactly like two different Kirby mods; nothing global refuses
   the connection.

## 4. Determinism and rollback contract

- **All mutable Geno state is game state.** `Geno_StateBlock` (pc/geno/geno_game.c) is a plain game
  global (`_gw_Geno_StateBlock`, a common symbol), and `gw_snap.c` also treats every `pc_geno_*`
  object as game state. Savestates, SyncTest and rollback cover it with no extra code (test
  `geno_state_savestate`: values survive a gw_snap save/modify/load round trip).
- **Registry and hook tables are install-time only**: built from files before the first fighter
  spawns, never changed during a match (same rule as m-ex's hook tables). The hook table is `const`.
- **No pointers in the state block**, only ints and floats; no host time, no host randomness. Any
  future randomness must use the game's RNG (which is snapshotted).
- **No intra-frame loops through Geno**: script skips only go forward.
- **Logs are rate-limited** (resimulation repeats frames; logs are diagnostics, never state).
- Peers must run the same exe (already required by the m-ex bridge).

## 5. Naming

- Log lines start with `geno:`. C: `Geno_*` (game half, engine call sites), `GenoGame_*` (game half,
  called by the native half), `gw_Geno_*` (native half), `GENO_*` constants in `pc/geno/geno.h`.
- IR engine id (experiment/character-ir): `melee.geno`.

## 6. Files

| file | half | what |
|---|---|---|
| `pc/geno/geno.h` | shared | constants: escape encoding, banks, sub-commands, hook numbers, events |
| `pc/geno/geno_game.c` | game (gwtool) | state block, escape interpreter, hooks, attributes, multi-jump |
| `pc/platform/geno_registry.c` | native | JSON reader, geno.json loading, stable ids, target resolution, scalar API, registry tests |
| `pc/geno/geno_tests.c` | game (gwtool) | escape / loops / resets / multi-jump / savestate tests |
| `pc/geno/tools/scan_ftcmd_opcodes.py` | tool | opcode census of every fighter script on a disc |
| engine sites | game | `ftaction.c` (3 loops), `fighter.c` (reset, action change, on_frame), `ftchangeparam.c` (attributes), `ftCo_JumpAerialF1.c` (multi-jump) |
| `gw_snap.c`, `gw_mexid.c`, `gw_tests_core.c` | native | snapshot coverage, netplay identity salt, test registration |

Link: the three objects are listed in `_build/agents/beta/melee_link_objects_geno.rsp` (beta's list
plus `pc_geno_geno_game.c.obj`, `pc_geno_geno_tests.c.obj`, `geno_registry.obj`); build with
`GW_LINK_OBJECTS` pointing at it. They are deliberately **not** in the shared link list.

## 7. geno.json

`mods/<id>/geno.json`, beside `mod.json` (never a disc file). The mod mounts through the normal
mods folder (`docs/mods-packaging.md` in the root repo), so it can also ship replacement disc files.

```json
{
  "geno": 1,
  "fighters": [
    {
      "attach": "kirby",
      "name": "Kirby (Brawl numbers)",
      "attributes": { "gravity": 0.085, "terminal_velocity": 1.6, "max_jumps": 6 },
      "jumps": { "max": 6, "air_vy": [1.8, 1.55, 1.35, 1.2, 1.1] },
      "hooks": { "on_init": ["geno.log:1"], "on_frame": ["geno.jumps.to_var:0"], "on_action": [] }
    }
  ]
}
```

- `attach`: an existing fighter - a vanilla name (`kirby`, `marth`, `gnw`, ...) or a Pl file
  (`PlKb.dat`, or an m-ex fighter's `PlSh.dat`). This is **(a)**: overlays on existing fighters, the
  Phase 1 path.
- `define`: **(b)** brand-new fighters. Reserved; v0 logs and skips it (section 13).
- `attributes`: any `ftCo_DatAttrs` field by its decomp name (40 of them: walk/dash/jump/air/
  gravity/weight/shield...). Applied right after the engine copies the attributes from the file,
  *before* scale/metal/etc. modifiers, at spawn, respawn and every re-apply.
- `jumps.max`: total jumps (ground + air, like `max_jumps`); `jumps.air_vy`: per-air-jump vertical
  impulse for multi-jump fighters (the last entry repeats). See section 10.
- `hooks`: native hooks bound to dispatch points, as `"name"` or `"name:arg"`.
- Unknown keys are ignored (a newer file still loads what this version knows) but they **are**
  hashed into the id, so two installs never agree on an id while behaving differently.
- Two entries for the same fighter: the later mod in mount order wins (logged).

**Stable ids.** Each fighter entry gets a 64-bit id = hash of its canonical form (type tags, keys in
file order, numbers as `%.9g`, strings raw, no whitespace), salted with "GENO" and the Geno version.
Same definition -> same id on every install, whatever the mod folder is called and whatever dense
m-ex slot the target has. The canonical form is pinned by a test (`geno_registry_stable_ids`);
changing it is a Geno version bump.

## 8. The script escape (ftcmd)

A subaction command's opcode is the top 6 bits of its first word. Retail uses 0-58: 0-9 through
`Command_Execute` (lbCommand), 10-58 through `ftAction_803C06E8` / `ftAction_803C07AC` with lengths in
`ftAction_803C0870`. **Geno claims 59** (first byte 0xEC-0xEF).

Why 59: it is the first opcode past the retail tables, so on hardware and in the unpatched port it
indexes off the end of three static arrays - no working script can contain it; the census found it
(and 60-63) on no disc; HSDRawViewer's command list (the community's reference decoder) defines
nothing above 58. 60-63 stay free for future needs.

```
word0  [31:26] 59  [25:20] sub  [19:16] len (total words, 1-15; 0 reads as 1)  [15:0] sub-specific
       variable subs: [15:8] var A   [7] B is a var   [6:4] compare (IF)   [3:0] 0
word1  operand B: immediate (int, or float bits when A is a float var) or a var ref in [7:0]
word2  IF only: words to skip forward (from the end of the command) when the test fails
var ref (8 bits): [7:6] bank  [5:0] index   banks: 0 LA int, 1 RA int, 2 LA float, 3 RA float
```

| sub | name | effect |
|---|---|---|
| 0x00 | NOP | skip `len` words |
| 0x01-0x04 | SET / ADD / SUB / MUL | `A op= B` (int or float by A's bank; B converted) |
| 0x05 / 0x06 | SETBIT / CLRBIT | `A |= 1<<B` / `A &= ~(1<<B)` |
| 0x10 | IF | if `!(A cmp B)` skip `word2` words; cmp: EQ NE LT LE GT GE BIT NOBIT |
| 0x11 | SKIP | skip `word1` words (jump over an else branch) |
| 0x20 | CALL | call native hook `word1` with argument `word2` |
| other | - | skipped by `len`, logged once: scripts for a newer Geno still walk |

`if (c) {T} else {E}` compiles to `IF !c ->skip |T|+2 ; T ; SKIP |E| ; E`. Variables and control flow
run in all three ftAction loops (exec, first-frame, fast-forward) - they are script logic, like
lbCommand 0-9; CALL runs only in exec and first-frame. Mixed freely with vanilla commands (the test
script interleaves SetCmdVar inside Geno branches).

**Variables** follow Brawl PSA's split so translated scripts map 1:1: LA (long-term: kept across
actions, reset at spawn/respawn) and RA (per-action: cleared on every action change), each 64 int +
64 float; PSA "bit" variables are bits of the int vars (SETBIT/CLRBIT/IF BIT). Native hooks
`geno.jumps.to_var` etc. move engine values into vars.

## 9. State block, hooks, dispatch points

`GenoState` per fighter object: `[player slot 0-5][sub-fighter]` (Nana, the inactive Zelda/Sheik),
holding profile, flags, the four banks, counters. Reset in `Fighter_UnkInitReset_80067C98` (spawn,
respawn, Zelda/Sheik swap - the incoming form starts fresh); RA banks cleared in
`Fighter_ChangeMotionState` right after `motion_id` is set.

Native hooks have **stable numbers** (geno.h; never renumber) and names:

| # | name | does |
|---|---|---|
| 1 | `geno.log` | log arg and LA0 |
| 2 | `geno.jumps.refill` | give back `arg` air jumps (0 = all) |
| 3 | `geno.jumps.to_var` | LA int[arg] = air jumps left |
| 4 | `geno.count_frames` | LA int[arg] += 1 |

A feature adds hooks as rows of the `const` table in geno_game.c. Hooks run from the escape's CALL
and from dispatch points bound in geno.json: `on_init` (after the reset), `on_frame` (after m-ex's
onFrame), `on_action` (after the RA clear). Only fighters with a profile dispatch.

## 10. Mechanic v0: multi-jump past Melee's cap

How Melee does it: Kirby and Jigglypuff set `can_multijump`; air jumps go through
`ftCo_800D730C` / `ftCo_800D74A4` using the per-kind table `fp->x2D0` (`Fighter_x2D0_t`: 5 vertical
impulses `x14[5]`, state count `x28` = 5, first state `x2C`, helmet variant `x30`). The nth air jump
enters motion `x2C + jumpsUsed - 1` with impulse `x14[jumpsUsed - 1]`; `ftCo_800D72A0` treats the 5
states as "in a multi-jump" (so the next jump waits for the script's cmd_var). The index is never
bounded: `max_jumps` above 6 would read past `x14` and enter motion ids after the multi-jump states.
Normal fighters use `JumpAerial` and only compare `jumpsUsed < max_jumps` (a u8), so raising
`max_jumps` alone already works for them.

Geno: `jumps.max` sets `max_jumps` (via the attribute hook). In `ftCo_800D74A4`, `Geno_MultiJump`
repeats the **last** multi-jump state for air jumps past the table (so `ftCo_800D72A0` still
recognises it) and takes the impulse from `air_vy` (last entry repeats) or the table's last row;
`air_vy` also overrides the table's own rows (Brawl numbers). Meta Knight (5 air jumps in Brawl)
and Brawl Kirby need exactly this.

Demo: `_build/agents/beta/mods-geno/geno-demo-kirby` (not committed) gives vanilla Kirby 9 jumps.
Run log: `geno: kind 4 player 0 air jump 8 of 8 (beyond Melee's multi-jump table)`; screenshot
`_build/agents/beta/shots/geno_kirby_1500.png` (Kirby high above Battlefield mid-chain).

## 11. Design: glide (next mechanic, not built)

Meta Knight's (and later Pit's/Charizard's) Brawl glide, in Melee terms:

- **New action states.** Geno reserves motion ids after the fighter's own special states and
  supplies `MotionState` rows with native callbacks (anim / input / phys / coll) from geno_game.c,
  installed the way m-ex swaps MoveLogic (`ftData_CharacterStateTables` per kind), so vanilla and
  m-ex rows are untouched. States: GlideStart, Glide, GlideAttack, GlideLanding, GlideEnd. Their
  subaction (animation + script) indices come from geno.json (`"glide": {"subactions": {...}}`),
  pointing at rows the fighter's Pl file ships (Brawl-port content adds them).
- **Input.** From JumpAerial / multi-jump states and Fall: jump *held* for N frames after the
  jump's apex (Brawl's rule) enters GlideStart. In Glide: stick Y pitches; A = GlideAttack; landing
  = GlideLanding; shield or the timer = GlideEnd -> FallSpecial (Melee's helpless fall).
- **Physics** (per frame, in the phys callback, state in the LA float bank): angle += stickY *
  turn_rate, clamped to [min, max]; speed += (sin(angle) * accel_down) - drag; velocity from angle and
  speed; a small constant sink replaces gravity. All parameters in geno.json (`glide.*`), defaulting
  to Brawl's MK values. Melee's global physics (air dodge, landing lag rules, ledge grab) apply
  unchanged: the glide is only a new *character* state.
- **Rollback:** all state is in the Fighter struct or the Geno block; nothing native changes at
  runtime.

The same "Geno action states" mechanism later carries crawl and wall cling.

## 11b. Design: HUD elements (roadmap, not built)

Evidence (coordinator, experiment/hud-meters/): three ACE fighters hand-roll a HUD meter with the
same template - S. Mewtwo (`PlSmHUD.dat` / `MgMtr_scene_models`: a 0-10 gauge from `fp+0x22D0`,
float mode via `fp+0x22E0/0x22E4`), Fay (`PlFyHUD.dat` / `WpInd_scene_models`: weapon icon, frame
from the int at `fp+0x22F8`), Toad (`Meters.dat` / `Relax_scene_models`: a vertical bar). All three
render correctly in the port today. The template: load the HUD file and its `*_scene_models`
symbol, create a JObj GObj with a GX link; every frame hide it if the owner has 0 stocks or the game
is debug-paused, place it at `ifAll_GetPlayerHUDPosition(port)`, set the animation frame from a
fighter variable, unhide.

Geno version - data-driven, drawn natively, no fighter code:

```json
"hud": [ { "file": "PlKbHUD.dat", "symbol": "Meter_scene_models",
           "value": "LA_INT:3",              // a Geno variable (or a hook-filled one)
           "map": { "mode": "discrete", "min": 0, "max": 10, "frames": [0, 100] },  // or "continuous"
           "visible": { "stocks_gt": 0, "hide_when_paused": true },
           "offset": [0, 12] } ]
```

- The element's GObj is created at fighter spawn from the (preloaded) file, owned by the fighter's
  player slot, destroyed with the fighter.
- Per frame (after on_frame hooks): visibility rules, position from the port's HUD anchor plus
  offset, frame = map(variable). The only input is the Geno state block, which is rollback state;
  the HUD object itself is render-side and fully re-derived every frame, so a rollback needs
  nothing extra (the same reason the existing meters survive rollback).
- m-ex fighters' own hand-rolled meters keep working untouched; this is for Geno content.

## 12. How Phase 1 (Brawl-numbers Kirby) plugs in

1. **Numbers** - a `geno.json` overlay on `kirby`: Brawl's common attributes mapped to Melee's
   `ftCo_DatAttrs` (walk/dash/air speeds, gravity, fall speeds, weight), `jumps.air_vy` from Brawl's
   Kirby multi-jump table, `jumps.max` 6. Works today.
2. **Moves** - (v1) subaction script overlays: geno.json names a subaction index and a script file
   in the mod (`mods/<id>/geno/<fighter>.ftcmd`), loaded once at boot into a read-only guest buffer
   (rollback-safe: never written after boot) and swapped into the fighter's action table rows at
   load. The PSA translator (a tool on top of experiment/character-ir) emits Melee commands for
   what Melee has (hitboxes with unit conversion, GFX, SFX, timers) and Geno escapes for variables,
   if/else and hook calls for PSA events Melee lacks.
3. **Animations / model** - through the normal mods file overlay (`PlKb*.dat`), outside Geno.
4. **Netplay** - automatic: the overlay's id folds into Kirby's identity.
5. **IR** - `melee.geno` engine blocks record the overlay (attributes, scripts, hooks) so the IR can
   diff Brawl Kirby against the Geno result.

## 13. Roadmap

| version | adds |
|---|---|
| v0 (this) | registry + stable ids + netplay salt, state block, escape (vars, if/else, CALL), 4 hooks, 3 dispatch points, attribute overrides, multi-jump past the table, tests, opcode census |
| v1 | subaction script overlays (Phase 1 moves); escape subs to read engine values into vars (percent, velocity, ground/air, facing, motion, frame) and to write a few back; DIV and a game-RNG random sub; special-attribute (`dat_attrs`) overrides; `air_vy` for non-multi-jump fighters; `on_hit` / `on_land` dispatch |
| v2 | Geno action states (section 11): glide, then crawl and wall cling; Meta Knight on top |
| v2.5 | HUD elements (section 11b): data-driven meters/icons bound to Geno variables |
| v3 | `define`: brand-new fighters with Geno-native registration (their own kind range and content ids, independent of m-ex's dense slots), CSS/SSS entries via gw_uigen |
| later | an IR emitter/loader for `melee.geno`; per-profile merge rules instead of "later mod wins" |

Constraints that stay: Melee's global rules (section 1), m-ex untouched (section 3), rollback-safe
(section 4), privacy of this branch until GD decides otherwise.

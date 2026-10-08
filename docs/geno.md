# Geno - GD's Melee's fighter-extension layer

**2026-10-04 GF1 update:** the Geno engine has an offline native Mario-reference `define` backend in format 6. Section 22 supersedes earlier statements that `define` is reserved. Source/tooling verification is separate from game acceptance; no GF1 game build or launch has been performed.

**Source reference current as of 2026-10-03.** This dated overview supersedes older status
summaries in this file; versioned verification sections remain historical results for their
named runs, not HEAD validation. JSON `GENO_VERSION` is **6**, with additive v5.5 commands,
articles, on-hit/counter hooks and native effects. The LAB and its `gd` API are public.
Stable earlier instruction ids retain their meanings; sections 15-20 describe their additions.

**Verification limits:** the 2026-10-03 guide-gap repairs were syntax-checked and the standalone
LAB checker passed; they were not built or run in the game. Counts such as 133/133, 149/149 and
189/189 below belong to historical runs, not the current suite. Paths under `_build/agents/`
and their demo mods, pad drivers, screenshots and NOTES are private historical evidence,
not repository examples or reproducible prerequisites. Use the committed `geno_*` tests and
`pc/geno/tools/lab_stage_d_check.lua`; gameplay acceptance for these repairs is still pending.

## 1. What Geno is, and what it is not

Geno is the native layer for **new fighter content** in GD's Melee. Its current customers are
Halberd (Meta Knight), Ultimate Kirby and Sora (the workspace's `ports/README.md` lists the pipelines).
They use character abilities beyond the port's m-ex compatibility surface: more jumps than
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
   stages) bit-identical to the pre-Geno build; ACE fighter crash sweep (15 CPU matches) A/B
   against the pc-port exe: the same failures in both, all the known CPU-AI crash
   (`ftCo_800B4AB0+0x7A`, flaky between runs); every vanilla-fighter group passes.
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
- IR engine id (ports/ir (workspace repo)): `melee.geno`.

## 6. Files

| file | half | what |
|---|---|---|
| `pc/geno/geno.h` | shared | constants: escape encoding, banks, sub-commands, hook numbers, events |
| `pc/geno/geno_state.h` | game (gwtool) | the `GenoState` block layout (shared by geno_game.c and geno_tests.c) |
| `pc/geno/geno_game.c` | game (gwtool) | state block, escape interpreter, hooks, attributes, multi-jump; v1: engine values, change action, rehit, autolink, landing edges, special attributes, script overlays |
| `pc/platform/geno_registry.c` | native | JSON reader, geno.json loading, stable ids, target resolution, scalar API, registry tests |
| `pc/geno/geno_tests.c` | game (gwtool) | escape / loops / resets / multi-jump / savestate tests |
| `pc/geno/tools/scan_ftcmd_opcodes.py` | tool | opcode census of every fighter script on a disc |
| engine sites | game | `ftaction.c` (3 loops), `fighter.c` (reset, action change, on_frame; v1: pre-anim checks, around the collision callback), `ftchangeparam.c` (attributes), `ftCo_JumpAerialF1.c` (multi-jump); v1: `ftcommon.c` (landing / take-off edges), `ftcoll.c` (autolink) |
| `gw_snap.c`, `gw_mexid.c`, `gw_tests_core.c` | native | snapshot coverage, netplay identity salt, test registration |

Build through the workspace's `tools/port/build.sh` (see `pc/docs/PORT_DEV_QUICKREF.md`). The
old beta-only link-list recipe was historical; do not depend on that uncommitted lane's files.

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
  The registry's 27 retail names (aliases after `/`) are `mario`, `fox`, `captain/falcon`,
  `donkey/dk`, `kirby`, `koopa/bowser`, `link`, `seak/sheik`, `ness`, `peach`, `popo`, `nana`,
  `pikachu`, `samus`, `yoshi`, `purin/jigglypuff`, `mewtwo`, `luigi`, `mars/marth`, `zelda`,
  `clink/younglink`, `drmario`, `falco`, `pichu`, `gamewatch/gnw`, `ganon/ganondorf`, `emblem/roy`.
- `define`: **(b)** native fighter definitions. Version 6 implements the offline Mario-reference preset (section 22); runtime acceptance awaits the integrator rebuild.
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
| 5 | `geno.article.spawn` | v5: spawn the profile's article `arg` (section 19) |

A feature adds hooks as rows of the `const` table in geno_game.c. Hooks run from the escape's CALL
and from dispatch points bound in geno.json: `on_init` (after the reset), `on_frame` (after m-ex's
onFrame), `on_action` (after the RA clear), v1 `on_land` (15.5), v5 `on_hit` (section 19.3). Only
fighters with a profile dispatch.

## 10. Mechanic v0: multi-jump past Melee's cap

How Melee does it: Kirby and Jigglypuff set `can_multijump`; air jumps go through
`ftCo_800D730C` / `ftCo_800D74A4` using the per-kind table `fp->x2D0` (`Fighter_x2D0_t`: 5 vertical
impulses `x14[5]`, state count `x28` = 5, first state `x2C`, helmet variant `x30`). The nth air jump
enters motion `x2C + jumpsUsed - 1` with impulse `x14[jumpsUsed - 1]`; `ftCo_800D72A0` treats the 5
states as "in a multi-jump" (so the next jump waits for the script's cmd_var). The index is never
bounded: `max_jumps` above 6 would read past `x14` and enter motion ids after the multi-jump states.
Normal fighters use `JumpAerial` and only compare `jumpsUsed < max_jumps` (a u8), so raising
`max_jumps` alone already works for them.

Geno: `jumps.max` sets `max_jumps` (via the attribute hook). Both `jumps.max` and
`attributes.max_jumps` accept integer totals **1..250**, clamping outside that range with a
named log line; the limit keeps the game's u8 jump counter from wrapping. `jumps.max` wins
when both keys are present. Ordinary double-jumpers use their existing input/count check;
`air_vy` overrides remain specific to multi-jump fighters.

In `ftCo_800D74A4`, the impulse index is bounded **before** reading the table.
`Geno_MultiJump` repeats the **penultimate** multi-jump state while more jumps remain,
and uses the last state for the declared final jump. Kirby's last script never sets the
next-jump `cmd_var[0]`; the penultimate script opens that gate at animation frame 28.
Helmet variants use their own first-state base. The impulse still follows the requested
jump number: `air_vy` (last entry repeats), or the table's own row/last impulse past its end.

The old private demo's nine-jump claim was contradicted by the 2026-10-03 course run, which
stopped at five air jumps. The source repair has regressions `geno_multijump_script_gate`
and `geno_ordinary_jumps`; a fresh gameplay retest is required before claiming nine jumps work.

## 11. Design: glide (superseded: built in v2, see section 16)

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
   load. The PSA translator (a tool on top of ports/ir (workspace repo)) emits Melee commands for
   what Melee has (hitboxes with unit conversion, GFX, SFX, timers) and Geno escapes for variables,
   if/else and hook calls for PSA events Melee lacks.
3. **Animations / model** - through the normal mods file overlay (`PlKb*.dat`), outside Geno.
4. **Netplay** - automatic: the overlay's id folds into Kirby's identity.
5. **IR** - `melee.geno` engine blocks record the overlay (attributes, scripts, hooks) so the IR can
   diff Brawl Kirby against the Geno result.

## 13. Roadmap

| version | adds |
|---|---|
| v0 | registry + stable ids + netplay salt, state block, escape (vars, if/else, CALL), 4 hooks, 3 dispatch points, attribute overrides, multi-jump past the table, tests, opcode census |
| v1 (built, section 15) | subaction script overlays; engine values GET/PUT/IFV; DIV, RAND; change action (Brawl requirements, persistent/once, CHGAND); REHIT; LINK (autolink 365); special-attribute overrides; `on_land` (script checks, geno.json map, hooks). Not done from the old v1 list: `air_vy` for non-multi-jump fighters (`on_hit`: v5) |
| v5 (built, section 19) | articles (projectiles as Melee items, models from a .dat), `on_hit` dispatch point 4, counter windows |
| v2 (built, section 16) | Geno action states, glide and native specials; crawl and wall cling remain future work |
| v3 (built, section 17) | root-motion states, Dimensional Cape and Drill Rush refinements |
| v4 (built, section 18) | animation-rate and glide refinements |
| future, unversioned | HUD elements (section 11b): data-driven meters/icons bound to Geno variables |
| v6, integration pending | `define`: offline Mario-reference native registration, independent effective rows and descriptors, unused resident aliases, selection defaults (section 22) |
| later | an IR emitter/loader for `melee.geno`; per-profile merge rules instead of "later mod wins" |

Constraints that stay: Melee's global rules (section 1), m-ex untouched (section 3), rollback-safe
(section 4), stable public encodings and explicit version checks.

The old roadmap used "v3" for the still-reserved `define` design. That label was a plan,
not a second meaning for JSON version 3; sections 17-19 use the implemented feature versions.

## 14. Geno Lab (inspection tool)

An in-engine, frame-steppable lab for seeing *why* a fighter feels the way it does, including
Halberd, Ultimate Kirby and Sora. The native part is thin: scalar read-only getters, the game's own
develop-mode drawing switches, a camera projection, engine event hooks and a snapshot history.
The tool itself is a public Lua script mod, shipped as `mods/geno-lab`. Its native calls extend
scripting API 1; `gd.lab_api == 1`. The workspace's `docs/scripting.md` inventories all registered
`gd` functions; this section gives the detailed fields and LAB behaviour.

### 14.1 Files

| file | half | what |
|---|---|---|
| `pc/gameworld/script_lab.h` | shared | field numbers, event kinds, draw bits (plain enums) |
| `pc/gameworld/script_game.c` (end) | game (gwtool) | `ScriptGame_Lab*`, `Hit*`, `Hurt*`, `Joint*`, `CameraF`, attrs; the two draw switches |
| `pc/platform/gw_script.c` | native | the Lua API below, event queue, history ring, projection, tests `script_lab_*` |
| `pc/platform/gw_motion_names.inc` | native | generated by `pc/geno/tools/gen_motion_names.py` from the decomp's motion enums |
| `pc/platform/gw_snap.c` | native | slots by index (`gw_snap_reserve`, `gw_snap_save_index`, `gw_snap_load_index`), max 48 slots |
| engine sites | game | `fighter.c` (action change, hitlag enter/leave), `ftcoll.c` (hit), `ftcommon.c` (land), `gmscene.c` (`Script_PostRender`) |
| `pc/geno/mods/geno-lab/` | mod | the Lab: `mod.json` + `scripts/lab.lua` |
| `gmfrontend_menus.inc`, `gw_script_pad.c` | game / native | SOLO > LAB entry; `gd.mirror_pad` |

These are shared-code files (no Geno object needed): the non-Geno link list builds them too.

### 14.2 Develop-mode drawing (verified in the decomp)

`gd.debug_draw(port)` reads and `gd.debug_draw(port, flags)` writes `Fighter.x21FC_flag` for every
fighter object of that port (Nana too) - the byte `fn_CheckAnimationInfo` (dbanim.c, R + D-pad)
cycles and `ftDrawCommon_800805C8` reads. PPC bitfields: `b7` is 0x01. Returns the previous byte.

| `gd.draw.` | bit | field | what the game draws |
|---|---|---|---|
| `MODEL` (`DEFAULT`) | 0x01 | b7 | the model (and its shadow); clear it to see only collision |
| `HIT` = `HURT` = `COLL` | 0x02 | b6 | hitboxes, hurtboxes (coloured by intangible/invincible), reflect, absorb and shield bubbles - one switch in the game |
| `DYNAMICS` | 0x04 | b5 | dynamic bones and their collision spheres (`x1670`) |
| `STOMP` | 0x08 | b4 | the enemy-stomp range line (`dmg.x1930`, Adventure) |
| `CPU` | 0x10 | b3 | CPU AI debug display (`ftCo_800B395C`, CPU players only) |
| `ITEM_PICKUP` | 0x20 | b2 | item pickup ranges (`itPickup`) |
| `THROWN` | 0x40 | b1 | the thrown hitbox |
| `COIN` | 0x80 | b0 | coin pickup spheres (`x1614`, Coin mode) |

The ECB and ledge grab are **not** in this byte: they belong to the match camera's collision
display. `gd.debug_stage([flags])` reads/writes it (`cm/camera.c` accessors):

| `gd.stage_draw.` | bit | what |
|---|---|---|
| `COLL` = `ECB` | 1 | stage lines + every fighter's and item's ECB + the ledge-snap (grab) boxes (`mpLib_8005A2DC`) |
| `TERRAIN` | 2 | colour the lines by terrain kind |
| `LEDGES` | 4 | colour the lines by ledge / platform |
| `POINTS` | 8 | spawn / respawn / item / special points |
| `ZONES` | 16 | camera and blast zones (write-only in the decomp; read back from a native cache) |

Both writes are **offline only**: they need a gameplay script (or the console) and are refused
during any netplay/rollback session, even for `rollback_safe` scripts (the byte is game memory and
would differ between peers). The byte resets to 1 when a fighter respawns; the Lab re-applies.

### 14.3 Reads (never write the game)

`gd.player(port)` gains: `motion_name` (the decomp's enum name for common states and each vanilla
fighter's specials, Kirby clones use Kirby's table; for m-ex fighters without a table, the
animation name, else `Special<n>`), `anim_id`, `anim_name` (from the fighter file's figatree symbol,
so it names m-ex moves), `anim_symbol`, `anim_frame_f`, `anim_rate`, `hitstun` (frames left, 0 when
not in hitstun), `in_hitlag`, `in_hitstun`, `intangible` / `invincible` (timer frames),
`body_state` / `timed_state` (`"normal" | "invincible" | "intangible"`: the subaction's body state
and the timers' state), `kb_vx`, `kb_vy` (knockback velocity; `vx`/`vy` stay the fighter's own),
`ground_vel`, `kb_applied`, `z`, `scale`, `cmd_timer`, `ecb = {top, bottom, left, right}` (world
`{x, y}`), `ecb_lock`, `jumps_used`, `jumps_max`, `jumps_left`, `walljumps_used`, `shield`, `iasa`
(the script's interruptible flag), `ledge_cooldown`, `draw_flags`, `joint_count`, `hurtbox_count`,
`hitboxes` (below, active ones).

| function | returns |
|---|---|
| `gd.hitboxes(port [, all])` | active hitboxes (all = slots 0-4 on or off): `{id, state, active, thrown, group, bone, damage, angle, kbg, bkb, wbk, element, element_name, shield_damage, radius, x, y, z, px, py, pz, ox, oy, oz, hit_air, hit_ground, clank, rebound, sfx_severity, sfx_kind}`. `id` 0-3 = `fp->x914[]`, 4 = the thrown hitbox (live only while it has an owner); `x,y,z` now, `px..` last frame (the swept capsule); `bone` = joint index |
| `gd.hurtboxes(port)` | `{id, bone, state, height ("low"/"mid"/"high"), grabbable, ax, ay, az, bx, by, bz, radius}` |
| `gd.joints(port [, fresh])` | per joint, list position = index + 1: `{index, parent (-1 root), valid, x, y, z, sx, sy, on}` - world position from the joint's last computed matrix, `sx, sy` projected. A joint nothing used this frame keeps an OLD matrix; `fresh = true` (offline only) brings every matrix up to date first (`HSD_JObjSetupMatrix`, which recomputes only dirty matrices from the current pose, so no game state changes) |
| `gd.dobjs(port)` | the fighter's draw list, read-only: per DObj (list position = index in `fp->dobj_list` + 1) `{index, hidden, render, tobjs = {{id, src, flags, tu, tv, su, sv, frame, fmt, w, h}}}` (MObj render mode; per TObj its GXTexMapID / GXTexGenSrc / flags, texture translate and scale, its texture anim's frame, image format and size), plus `models` (each model-part model's current state) and `costume` (the costume texture-anim TObjs, e.g. the eyes, same fields) |
| `gd.project(x, y [, z])` | `sx, sy, visible, depth` on the 640x480 script screen through the match camera (same maths as `lbVector_WorldToScreen`); `nil` without a camera; `visible` false behind the camera or off screen |
| `gd.attrs(port)` | the 40 named `ftCo_DatAttrs` fields as the fighter has them now (gravity, jump velocities, max_jumps, weight, ...) |
| `gd.motion_name(id [, port])` | a motion id's name (the port picks the fighter's table) |

`on_draw` now runs **after the render pass** (`gw_Script_PostRender`, gmscene.c) and the overlay
is handed the finished list (two lists, swapped), so projected joints and hitbox labels use the
camera and joint matrices of the frame being shown. `on_tick` still runs first; a scene loop that
never reaches the hook still gets `on_draw` at the next tick. The list holds 2048 items. (Also
fixed on the way: `gd.text` without a colour always failed.)

### 14.4 Engine events

| hook | arguments | engine site |
|---|---|---|
| `on_action_change(port, old, new, sub)` | motion ids | `Fighter_ChangeMotionState` |
| `on_hit(attacker, victim, info)` | attacker port (nil = no fighter owner), victim port, `info = {dealt, hitbox, item, attacker_sub, victim_sub}` + the attacker's hitbox fields as it was when it connected (`damage, angle, kbg, bkb, wbk, element, element_name, radius, x, y, z, ...`) | `ftColl_80076ED8` (fighter hitbox -> hurtbox, the damage branch) and `ftColl_80077C60` (item) |
| `on_hitlag(port, entering, sub)` | | `Fighter_UnkRecursiveFunc_8006D044` / `Fighter_8006D10C` (x2219_b5 edges) |
| `on_land(port, motion, sub)` | the motion it landed from | `ftCommon_8007D6A4` (air -> ground) |
| `on_match_start()` | (existing) | first frame with fighters |

Determinism: the sites only **queue** (`gw_Script_GameEvent`, nothing runs mid-frame; the queue
is off when no script defines one of these hooks). The queue is dispatched in order at
`gw_Script_FramePost`, before `on_frame`, never for a resimulated frame (the queue ignores events
while resimulating), and dropped at a scene change. During a netplay/rollback session an event
hook may not call any gameplay write, rollback_safe or not (the frame it reports may still be
rolled back). Reading the attacker's hitbox at queue time is read-only.

### 14.5 History and step-back

**Replaced by the long rewind (14.10).** The old ring kept one full snapshot per frame (26.6 MB a
frame, 40 frames at most). `gd.history(frames)` and `gd.step_back([n])` keep their names and their
`false, why` answers; `gd.history()` returns more fields now (14.10). Savestates, loads and
rewinds requested while **paused** are still applied at once, at the loop top between frames.
Loading a savestate restarts the history, because it belongs to the other timeline.
`gd.player().action_frame` is restored by loads. `on_loadstate(0)` reports a step-back.

### 14.6 The Lab mod

`pc/geno/mods/geno-lab` (mod.json: `"kind": "script"`, `"gameplay": true`, not rollback_safe). Put
it (or a junction to it) in the mods folder; script id `geno-lab/lab`. Settings persist in
`scripts-data/geno-lab_lab/settings.txt`. The keys and the display modes are in 14.9 (stage C
replaced the original `P` / `N` / `B` / `1`..`0` / `X` map).

Online the Lab only reads.

### 14.7 Stage 2: timelines, set motion, lock-step, LAB menu entry

| function | |
|---|---|
| `gd.timeline(port [, motion])` | the **effective** script of the current action (or of `motion`'s row), including Geno overlays and Geno states' reused subactions. Walked read-only: `wait` adds frames, `wait_until` jumps to an animation frame, loops / calls / gotos and Geno ORIG / SKIP are followed; stops at `end`, `wait_anim` ("anim_end"), 2000 commands or frame 1000 ("limit"). Returns `{motion, motion_name, anim_id, anim_name, end_frame, length, stop, script, events, conditional}`; each event `{frame (1-based), op, name, addr, words, ...}` with decoded fields for `hitbox` (`id, group, bone, damage, size, ox/oy/oz, angle, kbg, wbk, bkb, element, element_name, shield_damage, hit_ground, hit_air`), `gfx`, `sfx`, `hitbox_damage/size`, `hitbox_remove`, `cmd_var`, body/hurtbox/visibility states. Opcode 59 names are `geno.SET`, `geno.PUT`, `geno.CALL`, `geno.CHG`, etc.; fields include `sub`, `flags`, `var`, `operand_is_var`, `compare`, `arg1..argN`, readable `detail`, and raw `words`. IF/IFV mark `conditional=true`: the static walk shows their fall-through path, not evaluated fighter variables |
| `gd.set_motion(port or {ports}, motion [, frame [, rate [, lift]]])` | offline, gameplay. At the next boundary (at once when paused) the fighter enters `motion` - the plain `Fighter_ChangeMotionState` its entry function would make - then the game runs up to `frame` (default 1; entering is frame 1) and pauses, so the fighter shows that frame **with everything its script did on the way** (hitboxes included). `lift` > 0 puts a grounded fighter in the air that high first (aerials would land at once). Several ports in one call (or calls before the same boundary) start together: **lock-step**. Motions are bounded by the decomp's tables (common states; a vanilla fighter's specials, Kirby clones use Kirby's) so a garbage row is never entered. States whose entry function sets more up than the motion change itself may misbehave (specials with state variables) |
| `gd.mirror_pad(from, to)` / `gd.mirror_pad()` | offline, gameplay: port `to` gets exactly what port `from` sends (after every other source) - two fighters under the same inputs; `to` must be a human port (CPUs ignore pads) |
| `gd.lab_request([clear])` | true when SOLO > LAB (or `MELEE_LAB=1`) asked for the Lab |

`gd.step(n)` now runs up to 30 frames per rendered tick (it was 1), so long steps and
`set_motion` land quickly.

**SOLO > LAB** (`gmfrontend_menus.inc`, selection 0x42): shown only when a `geno-lab/*` script is
loaded; it sets the request and goes to Training's CSS. The Lab turns itself on in matches when
requested (setting `always` turns it on everywhere) and drops the request when the menus come
back after a Lab match.

Lab keys added: `M` move timeline (a bar per fighter: hitbox windows coloured by id, IASA green,
body/hurtbox state white, GFX blue, SFX purple, visibility yellow, the red cursor = the frame
shown; a text line lists every window `fN-M #id dmg% angle kbg bkb wbk radius`), `PAGEUP` /
`PAGEDOWN` scrub the focused fighter's move one frame (replays it from frame 1 via
`set_motion`; hold to repeat), `HOME` replay from frame 1, `C` lock-step (every fighter into the
focused fighter's move at its scrub frame), `R` mirror P1's controller onto P2. Console: `lab move
<motion id> [frame]`, `lab events` (the focused fighter's decoded script).

Not built: live edits written back to `experiment/brawl-kirby/tuning.json` (the "later" item).

### 14.8 LAB, the Lab's own game mode (stage A)

SOLO > LAB (icon `ico_lab`) now opens **LAB**, a game mode of its own (`GM_LAB` = `GM_COUNT + 2`,
`pc/geno/geno_lab_mode.c`); Training is untouched (a plain TRAINING launch, or `MELEE_LAB=1` with
Training, work as before).

- **Flow:** the kit's character select (SOLO / LAB) -> the kit's stage select -> the loading
  screen -> the match -> back to LAB's character select. B on the character select goes to the
  menus. The select screens are always the kit's (`gmFrontend_ModeSelect`), even with
  `MELEE_NATIVE_CSS=1`.
- **Rules** (`GenoLab_ApplyRules`): VS's machinery on LAB's own VsModeData row (not the save's):
  any fighters on any ports, humans and CPUs; time mode with the clock off (a KO respawns, the
  match never ends on its own); no stocks; no items; any stage; Melee's pause off. No results
  screen, no statistics.
- **Pause menu** (`lab.lua`, LAB only): START on any controller (or Esc) freezes the game and opens
  a kit panel in the Lab art (glass `#111122`@219, cyan `#38c9d9`, the kit's gold for the
  selection): Resume, Frame step, Overlays (every Lab toggle), Reset positions (loads the state
  taken at the match's first frame, slot 4), Save state / Load state (slot 1, as F5/F6), Change
  characters (-> LAB's CSS), Change stage (-> the SSS), Quit (no contest -> the menus). Pad: up /
  down, A, B back, START close; keyboard: arrows, Enter, Backspace, Esc. Closing holds every pad
  neutral for 10 frames so the closing button does not reach the fighters.
- **Script API:** `gd.lab_mode()` (true while LAB runs), `gd.lab_leave("css" | "sss" | "menu")`
  (ends the LAB match with a no contest; offline, gameplay). The mode also sets the Lab request, so
  the Lab script turns itself on.
- **Launch straight in:** `MELEE_SCENE="mode=lab;p1=fox;p2=falco/cpu0;stage=fd"` (the usual grammar;
  `at=css` / `at=sss` open the select screens). Only a LAB scene seeds LAB.
- **Tests:** `geno_lab_mode_table`, `geno_lab_rules`, `geno_lab_scene`, `geno_lab_select_flow`.
- The pause menu was redesigned in stage C (14.9).

### 14.9 Display modes, the kit HUD, the full-screen pause menu (stage C)

**Nothing plain on screen.** `lab.lua` draws only with `gd.kit` and the Lab art (`ui/`); there is
no `gd.text` left. `gd.line` / `gd.fill` remain only for the skeleton, bones and ECB lines. The
port's FPS panels (`shim_vi.c`, opt-in) use the same menu kit art and fonts outside LAB. They are
suppressed during a LAB match (`GenoLab_InMatch()`). The Lab's **Performance** overlay uses
`gd.perf()` and the kit instead;
toggle it in DISPLAY or with `F4` (menu closed). The F9 info panel and toasts are still there;
they only appear when you ask for them.

**Keyboard in LAB.** During a LAB match (`GenoLab_InMatch()`: match start to match end) the
keyboard never reaches the game pad (`shim_pad.c`: sampled as unfocused; in keyboard+ mode it
does not take the port; F1's C-stick hotkey is off). Every key belongs to the Lab. The public
"keyboard = hotkeys only" change will generalise this. Reconcile it with that change.

**Global keys** (every mode, menu closed):

| key | |
|---|---|
| `SPACE` | pause / resume |
| `RIGHT` | step +1 (hold = slow play, `CTRL` = 10) |
| `LEFT` | step -1 from the history ring (hold, `CTRL` = 10) |
| `F5` / `F6` | save / load state 1 |
| `TAB` / `SHIFT+TAB` | next / previous mode; `1`..`5` pick one directly |
| `F` | focus the next fighter |
| `H` | hide / show the whole Lab UI (overlays go back to the game's default drawing) |
| `F3` | help: a kit panel with this mode's keys plus the global ones |
| `F4` | toggle the Performance panel in every display mode |
| `ESC` (or START on any pad) | the pause menu (LAB matches) |

**Modes.** Each mode chooses its panels and overlays, and which keys are live. Each mode
remembers its own toggles. The mode and every toggle persist in `settings.txt`, for example
`mode=hitboxes` or `hitboxes.ecb=false`.

| # | mode | panels / overlays | keys (default) |
|---|---|---|---|
| 1 | CLEAN | a tiny mode chip only (gold while paused); model on, no boxes | none |
| 2 | HITBOXES | game hit/hurtbox draw, hitbox label chips, ECB, hitbox data panel (focused fighter) | `B` boxes (on), `L` labels (on), `E` ECB (off), `D` data (on) |
| 3 | FRAMES | action/frame + history chip; move timeline (focused + next fighter); boxes | `T` timeline (on), `B` boxes (on); actions `Q`/`E` scrub -1/+1 (hold), `HOME` replay, `C` lock-step, `R` mirror P1 -> P2 |
| 4 | STAGE | stage debug draw | `C` collision (on), `L` ledges (on), `T` terrain, `P` points, `Z` zones |
| 5 | INSPECT | skeleton, joint-number chips, info panel (2 fighters), attribute compare (differences first, in gold), event log | `M` model (on), `S` skeleton (on), `J` joints, `I` info (on), `A` attributes, `L` log (on) |

Every mode except CLEAN shows a key strip along the bottom: the mode chip (icon + name, gold
while paused), a key chip plus a toggle icon for each mode key (an OFF toggle is
`ico_lab_slash_gap`, then the icon in its off tint, then `ico_lab_slash`), the frame and
step-back depth, and `TAB mode` / `F3 help`.

**Pause menu (full screen).** START or Esc freezes and dims the game. A glass slab with a
cyan edge covers the left. The tab name is drawn huge, bleeding off the edge. Decoration is
the flask, the hitbox burst and hazard stripes.
- Tabs sit across the top and switch with L / R (keyboard `Q` / `E`, PgUp / PgDn). A cyan wipe
  plays on each switch.
- Big sheared rows run down the left. The selected row is gold, pushed right, and has a
  chevron.
- A detail panel on the right shows the icon, the name, the current value, a one-line
  description, the button hints and the match key for the same action. In DISPLAY, the "Display
  mode" row also previews that mode's keys.
- A controls strip runs along the bottom.
- Rows open with a staggered slide-in (about 7 frames).
- Tabs and rows are recorded as hit rects (`menu.hits`, `menu_hit(x, y)`). A future
  `gd.mouse()` (`{x, y, pressed}`) is already polled when it exists.

**The pause menu in Atlas (`lab ui on`; off is the default until the owner has looked).** The same `TABS` and the same rows, drawn by one mapper in `lab.lua` as a `gd.ui`
screen (`geno-lab.pause`, chapter I, trail `LAB > PAUSE > <tab>`, over the frozen game: `backdrop = "world"`). What is the same: the tabs and rows, START and ESC to open, the
freeze, the offline-only rule (no Atlas LAB screen exists online), `menu_close` resuming only when the menu paused the game. What changed: tabs move with L and R, Tab and Shift+Tab
(**Q and E no longer change tabs here**: letters stay free for hotkeys); a toggle row is a toggle (A, left and right flip it once), a row with a value that left and right change
(Focus, Damage, Fly speed, History, Hot reload, Display mode) is a **stepper** (left and right change it, A runs it), a value-only row is text; the key a row mirrors is its sub line
("Match key F"); the description is the explainer's WHAT (at most 110 characters, so eighteen were shortened: what moved out is in 14.12, 14.13 and 14.16) and DISPLAY's mode row lists
the mode's keys as WITH tags; **DELETE is Y**, and asks first in a dialog (A deletes, B keeps; the focus lands on a neighbour); the saved-state library is paged (25 a page, a
`Library page` row) because the record holds 32 rows; entries other mods register under `lab.pause` become a MODS tab (A activates one through the registry; offline only). Row icons
are not drawn (Atlas has no icon set yet; the `ico_lab_*` art stays for the dev overlays). `lab ui on|off` is per session and does not survive a script reload; if `gd.ui` is
unavailable (fonts missing) the legacy menu opens. The menu is the script's own screen, so another script cannot touch it.
**The LAB HUD in Atlas** (menu closed, same switch): the info panel is a **readout** per fighter (ten rows, P1 top left, P2 top right; the ECB row is not shown), the move timeline a
**track** for the focused fighter at the bottom (the second fighter's timeline is not drawn: two do not fit above the retail plates), the mode strip a **chips** strip and `say()`
notices **notes** with a tone; with the menu open, the Lab hidden or a netplay session the HUD is empty. Every other overlay (hitboxes, skeleton, ECB, labels, performance, the move
browser, launch, A/B, the rollback strip, help, the event log, drill results) keeps its primitive drawing. **Verified**: the Lua checks in `pc/geno/tools/lab_stage_d_check.lua`
(run against the `gd.ui` stand-in as the script `geno-lab`), `tools/port/lab_locals.sh` (main-chunk locals stay at 179), and the host tests; **not run in the game**.

| tab | rows |
|---|---|
| PLAY | Resume; Step +1; Step -1; Step +10; Focus (left / right) |
| DISPLAY | Display mode (left / right); the current mode's toggles; Performance; Lab UI shown / hidden |
| DUMMY | Target (left / right); Damage (left / right steps of 10, A applies `gd.set_percent`); Lock-step; Replay move; Mirror my pad |
| STATES | Save to library; Quick save / Quick load (memory slot 1-3); Reset positions (slot 4, match start); History (2 / 5 / 10 / 20 s); Hot reload (replay 1 / 2 / 3 / 5 s); then the library, one row per saved state (A loads, Y / DELETE deletes after a confirm). Stage B, 14.10 |
| EXIT | Change fighters; Change stage; Quit (no contest) |

Pad: stick / d-pad up and down, left and right change a value, A, B or START close, L / R
switch tabs. Keyboard: the arrows, Enter or Space, Backspace or Esc, `Q` / `E`. While the menu
is open it takes every key, so no Lab shortcut fires underneath it.

**Performance panel.** The compact kit panel shows current presented FPS and target (0 means
uncapped), the latest total frame time, draw calls, decoded GX vertices and live Geno FX particles.
Its graph covers up to 160 recent presented frames, paired into 80 columns. Cyan is game-thread
logic, gold is time inside the timed GX submission calls, and red is Aurora enqueue time. The
8.3 ms and 16.7 ms reference lines mark 120 and 60 FPS. Violet marks render-worker busy time as
an independent trace: it overlaps game-thread work and is therefore not part of the stacked total.
The gray remainder of total time is pacing, events and frame-slot waiting. Samples are host-side
diagnostics, not rollback state; collection stops shortly after the panel is closed. `F4` has no
other Lab binding. Outside LAB, Video > Show FPS > Performance (or `show_fps = 2` in `video.cfg`)
draws the same data with the menu kit's `frame_*` art, font atlas and palette through the host
overlay. `show_fps = 1` uses a smaller kit panel for the FPS readout; `0` draws neither panel and
does not touch the kit from this path.

Console: `lab status` (mode, toggles, draw and stage flags, menu tab and row, hit-rect count),
`lab mode <name|1-5>`, `lab set <mode>.<toggle> on|off`, `lab hide`,
`lab menu [tab|close]`, plus the older `port / history / back / dump / move / events`.

**Art added** (`art/lab_art.py`, 73 textures, all checks pass): icons `ico_lab_clean`,
`inspect`, `points`, `zones`, `terrain`, `dummy`, `display`, `exit`, `eye`, `keys`, `percent`
and `modes`; chrome masks `lab_solid` (flat or sheared quads), `lab_fade`, `lab_stripes`,
`lab_ruler`, `lab_burst`, `lab_bracket`, `lab_chev` and `lab_chip_l/r`. Review:
`art/preview/lab_sheet.png` sections 7-9 (the pieces, the HUD strip mock-up and the pause menu
mock-up).

### 14.10 Stage B: the long rewind, the state library, hot reload

**The long rewind** (`gw_snap.c` "THE GENO LAB'S LONG REWIND", driven by `gw_script.c`):
- **Storage.** One full image, the *base*: MEM1 plus the game globals, the oldest frame you can
  reach. Then a **delta keyframe** every N frames (default 30). Its MEM1 part is the pages written
  since the previous keyframe (the write-watch set `sn_poll` already keeps, fed into its own
  bitmap). Its globals part is the 4 KB chunks that changed. A keyframe = the base + every earlier
  delta + its own, the newest copy of a page winning. The window slides: a keyframe older than the
  window is folded into the base and freed.
- **The per-frame log** (`gs_log`, 4096 frames) holds everything outside the snapshot that the
  simulation reads:
  - the `PADStatus[4]` the frame renewed, taken in `HSD_PadRenewMasterStatus` (controller.c), or
    "renewed nothing";
  - the voice handle every `HSD_AudioSFXStartParam` returned (axdriver.c). The handle is game
    state; the voice pool is not;
  - every answer the voice pool gave the game (axdriver.c `LAB_AUDIO`): key-off / set-pan /
    volume / pitch / mix results, `HSD_AudioSFXCheck` ("still playing?", which the crowd and stage
    code read), `AXDriverCheck` and the two voice counts.
  The RNG seed is game state, so the snapshot holds it. The pad queue, the rumble and the disc's
  async blocks are not simulation.
- **Going to frame F.** Load the newest keyframe K <= F. Only the pages that can differ are
  copied: those written since the live state was last a keyframe, plus the pages of the keyframes
  between. Then re-simulate K..F-1 **in one tick** on the logged input. The frames before the last
  run with the rollback's resimulation flag: sound is silent, and the scene loop renders without
  presenting (gmscene.c). The last frame runs as a real frame and is shown.
  - Frames between F and the log's head are then **replayed** from the log, at speed or step by
    step. Sounds play, but the game keeps the logged handles; a logged-to-real map lets them stop.
  - `G` (`gd.rewind_live()`) stops the replay and goes live at that frame.
  - Any write (`set_percent`, `set_stocks`, `set_motion`) forks the timeline: the log and the
    keyframes after the fork go, and a keyframe of the new timeline is kept at once.
- **Scalars only across the game/native boundary.** Native code writing through a pointer into
  game memory writes little-endian. The first version handed the handle back through an `int*`,
  and replayed handles came back byte-swapped; the exactness test caught it.

| | |
|---|---|
| memory, 10 s (600 frames, the default) | **49.4 MB** (the base image 24 MB + the globals x3 + the keyframe deltas 17 MB). The old ring needed 27 MB per *frame* |
| memory, 20 s (1200 frames) | **67.1 MB** (deltas 34.7 MB) |
| a keyframe | 1.8 ms, every 30 frames |
| step back 1 | **8-12 ms** measured: a keyframe load of 0.7-1.1 ms (~200 pages) plus 16-20 re-simulated frames at ~0.5 ms each. At most ~15 ms (29 frames) |
| scrub to -600 | **10.6 ms** (21 frames re-simulated). -1190 at 20 s: 12.8 ms. The distance does not matter: it is one load plus at most 29 frames |
| a library save / its file | ~40 ms / ~12.6 MB (the zero pages are left out) |

Measured on ACE: Wolf vs Fox (CPU level 0) on FD, P1 driven by a scripted input loop.

**Exactness** (`gd.rewind_test([frames [, keep_running]])`, `gd.rewind_test_result()`):
1. It copies the whole state (MEM1 + globals) at a frame T.
2. It runs `frames` more, then rewinds to T through a keyframe and the log.
3. It compares every byte and hashes both sides.
It does not count the disc's async blocks (the four stream command blocks at 0x80171160, and
devcom's nodes; a rewind load now leaves all four blocks alone), the pad-side globals SyncTest does
not compare either (pad statuses, rumble), the light list (walked in both images), or the
**render-owned** bytes it measures while it runs. Render-owned bytes are those that change between
a render pass's start and the next logic frame, word-granular, as SyncTest marks them. The render
pass advances particles and reuses list nodes per render, and a paused Lab renders without running
logic, so those bytes legitimately differ.

Result on ACE (Wolf vs Fox, FD, a scripted P1): **14 of 14 PASS** back to back, over 29..500 frames
and 0..29 re-simulated frames, many of them captured while a replay of an earlier timeline was
running. Each reads `PASS: ... 0 simulation bytes differ, hash X vs X`; between 4 and 99 bytes
were exempt per run. What the test found on the way (all fixed): handles byte-swapped through a
pointer; the fourth stream block; the voice pool's answers; the light list.

Debug read: `gd.lab_peek(addr [, n])` returns n (<= 64) raw MEM1 bytes as hex, read-only.
`gd.lab_leave("restart")` ends a LAB match into the same match again.

**The persistent state library** (`gd.state_save/list/load/delete/rename/gen`):
- **Storage.** One file per state, `scripts-data/geno-lab_lab/states/st_<time>_<n>.gdst`, plus
  `index.txt`, a readable list rewritten on every change (the files are the truth). A file holds
  a header, the non-zero MEM1 pages and the globals, with a 64-bit hash over all of it. It is
  written to a temp file and renamed.
- **The header.** Exe hash (the running exe's bytes), disc hash (the ISO's first 4 MB and its
  size), mods hash (`gw_Mods_Describe`), Geno hash (every profile id + `GENO_VERSION`), the
  fighters (character, costume and CPU per port), the stage, the match frame, the date, the name
  and "what" ("Wolf v Fox on FD").
- **Loading.** The header is checked when you ask. Any mismatch is **refused** with the reason:
  "saved by another build of the game", "... another disc", "... other mods", "... other Geno
  fighter data", "load it during a match", or "this state is Wolf v Fox on FD: start that match
  first". Only then is the whole file read and verified, before a byte of it is loaded; a damaged
  file is refused too. A load restarts the history (`on_loadstate(9)`).
- **In the menu (STATES).** Save to library, with an auto name like `Wolf v Fox – FD – f1234`
  (there is no text entry; `lab rename <file> <name>` renames from the console). Then the list:
  8 rows show and it scrolls. A loads; Y / DELETE asks, and A confirms the delete. Refused states
  show REFUSED and say why. F5 / F6 are still the quick memory slots.

**Hot reload** (`gd.hot_reload([seconds])`, F8, STATES > Hot reload; `on_hot_reload(ok)`,
`gd.hot_reload_status()`):
1. Rewind `seconds` (default 2) through the history. That is exact: the old data re-simulates to
   there.
2. Re-read every mounting mod's `geno.json` and the words files its overlays name
   (`gw_Geno_Reload`).
3. Re-apply the data to every live fighter (`GenoGame_LabReload`): refill the overlay pool and
   repoint the subaction rows (`Orig` keeps the vanilla script), rebuild the Geno state rows and
   parameters, and recompute the attributes (the file, then the Geno overrides, then the game's
   own modifiers: `ftCo_800D105C`).
4. Reload the Lab script.
5. Restart the history at that frame and **replay** the logged input on the new data, then go
   live.

What reloads cleanly: attributes, jumps, special attributes, hooks, on_land, change-action checks,
behaviour parameters, state next/land targets and landing/root-motion/counter parameters,
same-layout subaction overlay words (inline or words files), and the Lab script.

What does not:
- **Layout changes.** The profile set or attachment; state count/order/names, behavior or any
  anim/iasa/phys/coll callback; state subaction/like/flags/move id; article list/order; overlay
  list/order or script-slot lengths/offsets. Active fighters copy motion rows and callbacks, so
  rebuilding those rows alone cannot safely update them. Reload names the reason in
  `geno: hot reload: layout changed (...)` and **restarts the match**
  (`gd.lab_leave("restart")`: the loading screen, then the same fighters on the same stage).
- **Disc / file data** (Pl*.dat, animations, models): these need a new match.
- `on_init` hooks are not re-run.
- A `geno.json` that does not parse keeps the old data.

### 14.11 Stage E: the creator tools

Three new display modes (keys `6`/`7`/`8`, TAB cycles through all 8), a FRAMES toggle and a pause-menu
tab. The Lua is in `lab.lua` in one function scope, `stage_e()`. It needs its own scope because the main chunk is at Lua's
200-local limit. The art is `ico_lab_moves / launch / ab / export / diff / rollback / ko` (`lab_art.py`, 83
textures, every check passes).

**MOVES: the state browser (6).**
- **The list.** `gd.motion_list(port)` gives every action state the fighter has a row for:
  - `common`: the common states;
  - `special`: the decomp's special table for its kind;
  - `mex`: past that table, the m-ex MoveLogic table (`gw_Mex_MoveLogicEntriesForKind`);
  - `geno`: motion 0x400+n, named by geno.json (`gw_Geno_StateName`).

  On top of these sit eight **input** rows (B, B>, B^, Bv, standing and in the air).
- **Filters** (`V`): ALL / ATTACKS / COMMON / SPECIAL / M-EX / GENO. `lab moves <text>` filters by
  name.
- **Playing a state** (`UP` / `DOWN` pick, `ENTER` plays it):
  - `N` plays it from neutral: the match-start state (slot 4) each time, otherwise Wait.
  - `L` loops it.
  - `Z` / `X` set the speed: x1, 1/2, 1/4, 1/10. Below x1 it steps one frame every N ticks.
  - `S` stops it.
  - The timeline follows along the bottom (`T`), and the right panel shows the state's own script windows.
- **How states are entered.**
  - Common states: a plain `gd.set_motion`. States with no animation symbol are refused, e.g. Fox's
    Attack13: entering it trips the ground assert.
  - Geno states: `Geno_LabEnterState`, i.e. the behaviour's own entry routine.
  - Vanilla and m-ex specials: **only through their input.** A bare motion change skips the
    special's setup and crashed the game (Fox's blaster). The B rows press B, plus a direction, on
    the pad from a neutral stance, so the game enters the special itself.
- `gd.player().motion_name` now names Geno states (e.g. `Tornado`).

**LAUNCH: the knockback preview (7).**
- **What it previews.** The focused fighter's live hitbox, or, when none is live, the next window its
  script opens (the hit about to connect). It is applied to the victim (`V` cycles). `Q` / `E` pick the hitbox.
- **`gd.kb_preview(victim, {damage, angle, kbg, bkb, wbk, attacker, dir, di, percent, x, y})`
  returns:**
  - `kb`, `level`, `tumble`, `hitstun`;
  - `angle` and `angle_di`;
  - `points` (one per hitstun frame);
  - `blast` (the frame and side where it crosses a zone), and the `zones`.
- **The game's own code:**
  - Knockback is `ftColl_80079AB0`, the fighter-hit path: the stage factor, the attack and defence
    ratios, weight, and percent + damage. Then `ftCo_Damage_CalcKnockback`: crouch, ice, smash
    charge, Y scale, armour and the minimum.
  - For that call the game half (`ScriptGame_LabKnockback`) sets the percent and pending damage
    the formula reads, then restores them bit for bit.
  - The level is `ftCo_8008D8E8`.
- **The same expressions, with the loaded PlCo constants** (`ScriptGame_LabCommonF`):
  - the angle: `ftCo_Damage_CalcAngle`'s Sakurai rule;
  - the trajectory DI: `ftCo_8008E5A4`, 18 degrees;
  - the launch speed: x100 = 0.03;
  - the flight: DamageFly's gravity / terminal / aerial friction on the self velocity, plus
    fighter.c's 0.051 decay on the knockback velocity.
- **DI** (`D`): none / in / out / survival.
  - `in`: the perpendicular back toward the attacker's side.
  - `survival`: the perpendicular that lands nearer 45 or 135 degrees.
- **Percent.** `Z` / `X` set it (±10); `P` goes back to the live percent.
- **Drawing.** The arc runs to the end of hitstun, with a dot every 10 frames and a KO burst where
  it crosses a blast zone. The check runs past hitstun only while the launch still has velocity.
- **The check** (`K`, on by default): when a hit really lands, the Lab predicts it from that hit's own numbers
  and the victim's pre-hit percent. It then follows the real flight to the end of hitstun and
  reports the knockback, the hitstun and the flight error (`lab kbcheck:` in the log, `lab kb`).
  - Measured on ACE: Falco's fsmash on Fox at 80%, 75 frames of tumble.
    - Hitstun predicted 75, real 75.
    - Flight error: max **0.0001** units, mean 0.0000.
- **Not modelled:**
  - ground launches that stay grounded (slide), and bounces;
  - the air-motion multiplier x190;
  - an attacker's special hit directions (the default `dir` is away from the attacker);
  - landing during hitstun: the arc goes through the floor.

**A/B: two variants on the same inputs (8).**
- **The approach.** Sequential and exact: re-simulation through B's rewind machinery, not two live fighters.
  1. `R` records A live (the frames are captured on every frame) and stops at `R` or at the rewind window's edge.
  2. B re-runs the logged input from A's first frame:
     - `C` = the same data. This is a determinism check; 0 differences are expected.
     - `B` = reload. It runs through `gd.hot_reload`, so it uses whatever geno.json / overlay words are on disk now.
       A and the plan wait in `ab_a.txt` / `ab_pending.txt`, because the Lab script reloads too.
     - `M` = two fighters live: P2 gets P1's pad, and they are compared from their own start with x as
       faced. P2 must be a human port.
- **What is compared, per frame and per port:** position, action, the hitbox-id mask, and the first divergence.
- **What is drawn:**
  - ghost paths of A (cyan) and B (gold), with markers on the viewed frame (`Q` / `E`);
  - split tracks coloured by action;
  - a strip of the differing frames, and the first divergence marked.
- **Measured** (MK vs Fox CPU, ACE):
  - Same data: 389 frames, 0 differences on both ports.
  - Reload with geno.json unchanged: 251 frames, 0 differences.
  - Reload with `tornado.w02` (start_rate) 80 -> 60: they part at the tornado's end (action,
    hitboxes). Fox, who was hit, differs from the next frame.
- **Not built:** Geno vs its non-Geno base. That needs the profile switched off in a live match, which is a layout change.

**The frame-data export and diff (TOOLS tab, `lab export`, headless).**
- **What it runs.** Every common `Attack*` state (plain entry), the 8 input specials (the full chain,
  from the input to Wait / Fall / Landing / helpless) and every Geno state (Geno entry). Each starts
  from a neutral state saved in quick slot 3 and runs at up to 30 frames per tick.
- **Numbering.** The first frame the game runs in the state is frame 1, which matches the frame-data sites.
- **Per state:** startup, active windows, total, IASA, the script length, landing lag and L-cancel
  lag (attributes), autocancel windows (`cmd_var[0]`, `ftCo_LandingAir_EnterWithLag`), where it
  ended and the state chain.
- **Per hitbox:** damage, angle, KBG, BKB, WDSK, radius, bone, element, shield damage and frames.
- **Output.** `scripts-data/geno-lab_lab/framedata/<fighter>/<version>/moves.csv`, `hitboxes.csv` and
  `framedata.json`, plus `versions.txt`. `gd.data_write` names may now contain folders.
- **Headless:**

  ```
  MELEE_SCENE="mode=lab;p1=fox;stage=fd" MELEE_LAB_BATCH=v1 MELEE_LAB_BATCH_QUIT=1
  ```

  It starts once P1 stands in Wait. `MELEE_LAB_BATCH_PORT` picks the port and
  `MELEE_LAB_BATCH_VERBOSE` logs each state. Fox takes 1.8 s (29 states); Meta Knight 4.3 s
  (60 states).
- **A watchdog.** A Lua error or 5 s without progress ends the export (and quits in headless mode).
- **Diff.** `python pc/geno/tools/framedata_diff.py <exportA> <exportB>`, or in the game TOOLS > "Diff the
  last two" / `lab fdiff [fighter verA verB]` (also written to `diff_<a>_<b>.txt`). MK with
  `tornado.w02` 80 -> 60 lists exactly Neutral B, Air Neutral B and Tornado, each 13 frames shorter.
- **Fox on ACE vs the known values** (all match):

  | move | startup / active | total | landing |
  |---|---|---|---|
  | jab 1 | 2-3 | IASA 17 | |
  | dash attack | 4-17 | 39 | |
  | ftilt | 5-8 | 26 | |
  | utilt | 5-11 | 23 | |
  | dtilt | 7-9 | 29 | |
  | fsmash | 12-22 | 39 | |
  | usmash | 7-17 | 41 | |
  | nair | 4-31 | 49 | 15 / 7 |
  | fair | 6-8, 16-18, 24-26, 33-35, 43-45 | 59 | 22 |
  | bair | 4-19 | 39 | 20 |
  | uair | 8-9, 11-14 | 39 | 18 |
  | dair | 5-24 (7 hits) | 49 | 18 |
- Run the export with the fighter alone. An opponent in range is hit, and hitlag stretches the numbers.

**The rollback visualiser (FRAMES, `N`).**
- **What is recorded.** `gw_snap.c` keeps a ring of the last 600 rollbacks:
  - the frame, the first frame, the depth;
  - the cause: the port whose confirmed input differed (`gw_rb_submit_remote_input`), or SyncTest's own;
  - the kind: synctest / fake / netplay;
  - the load + re-simulation ms, and whether it mismatched.
- **The first mismatch.** SyncTest's `sn_compare` names the symbol or heap object, and inside a live
  **Fighter** the field (`ScriptGame_LabFighterAt/FieldName`). Inside `Geno_StateBlock` it names the
  port and the **GenoState field** (`GenoGame_StateFieldName`). A netplay checksum desync is recorded
  with both checksums.
- **API.** `gd.rollbacks([n])` and `gd.rollbacks_clear()`; console `lab rollbacks [n]`. It works in any session.
- **Drawn as** a strip of bars (depth; the colour is the kind; red = mismatch) with the last rollback's
  numbers, the deepest one, the mean cost and a count per cause, plus a MISMATCH panel.
- **Locally, without a second PC:**
  - `MELEE_SLP=<replay> MELEE_RB_FAKE=4,2,5` is the fake network. A Marth ditto gave 28 rollbacks, all
    caused by P2, depth 3-7, 3.4-8.5 ms. That equals the session's own count (28, max 7, 136 frames re-simulated).
  - `MELEE_SYNCTEST=2` on the same replay: 874 rollbacks recorded, 0 mismatches.
  - `_build/netplay_local.ps1` for real netplay.
- **Test.** `geno_lab_mismatch_fields` checks the naming.

**Other API:** `gd.lab_env(name)` (reads `MELEE_LAB_<name>` only), `gd.lab_now([long])` (the sandbox
has no `os.date`), `gd.player().kb_last` (the last launch's knockback, which is kept after
`kb_applied` clears), and `gd.attrs` gains `normal_landing_lag` and `landingair{n,f,b,hi,lw}_lag`.

### 14.12 Stage D: the player-training half (D1: frame data on screen)

Stage E built the creator tools; stage D is what players practise with. It has five parts: D1 frame
data on screen (14.12), D3 the dummy (14.13), D2 hitbox display (14.14), D4 combo analysis (14.15)
and D5 scenarios and drills (14.16). All five are built. The punish finder (D4) is left for later.

The Lua is in `lab.lua` in one function scope, `stage_d()`, like stage E's. No new art: it reuses
the stage C and E icons.

**TRAINING mode (9).** It has four panels, each a toggle (`A`, `M`, `I`, `K`), plus `B` for the
game's boxes and `C` to clear the readouts. They all only read the game.

**Frame advantage (`A`).**
- **An exchange** starts on a hit (`on_hit`, fighters only, no items) or a shield hit (the victim
  enters hitlag in `GuardOn` / `Guard` / `GuardSetOff` / `GuardReflect`). The shield hit's attacker
  is whoever entered hitlag on the same frame, else whoever has a live hitbox, else the nearest
  fighter.
- **From that frame,** each side's first actionable frame is taken. The advantage is the victim's
  minus the attacker's, from the attacker's side: `+3 on shield` means the attacker acts 3 frames
  first. Both clocks start on the same frame, so the shared hitlag cancels out.
- **Actionable** (`LD.actionable`):
  - never during hitlag;
  - the script's IASA flag (`player.iasa`);
  - a free state: Wait, walks, Turn, Dash, Run, the squats, the falls, JumpF/B, JumpAerialF/B,
    GuardOn, Guard, CliffWait and Ottotto;
  - `Landing` once the animation frame reaches `normal_landing_lag` (`ftCo_Landing_IASA`'s own test);
  - a damage state (`DamageHi/N/Lw/Air1-3`, `DamageFly*`, `DamageFall`) once hitstun is over.
- **Restarts and drops.** Another hit by the same attacker restarts the exchange, so multi-hit
  moves count from the last hit. A shield break ends it with "shield break". After 240 frames it
  is dropped.
- **On a load or a step back,** the exchanges after the frame you land on go, like the event log's
  lines. The same goes for the tech results and the combos. The tech hit rates stay, as totals.
- **A move's jump-cancel is not its IASA.** Shine's jump cancel, for one, is not the script's
  interrupt flag, so the attacker's side is read at the move's own end. `lab adv` measured Fox's
  shine on shield at +4 and jab 1 at -10 (ACE, a scripted human P2 holding shield). A published
  sheet's number may count the jump cancel.
- **On screen:** the latest result big at the top (green plus, red minus) with the move, and the
  last 5 in a row under it. It also goes to INSPECT's event log.

**The move card (`M`).** It shows the focused fighter's move, **measured as it runs, the way the
frame-data export measures** (`bx_frame`), so `lab card` and `lab export` agree. One convention for
both:
- **frame 1** is the frame the state starts: the frame its change is seen (`action_frame` 0). A move's
  frame-1 hitbox, like shine's, is already out then. The first fix counted from the frame after,
  which read a frame low (ftilt 4 / 4-7 / 25) on ACE;
- **startup / active** are the frames the move has a hitbox;
- **IASA** is the first frame where the interrupt flag **turns on** inside the state. It is none when
  it only comes at or after the end (Fox ftilt). The flag can come into a state already set from the
  one before; the Lab's own entry does not clear it, and on ACE every export row read IASA 1. So a
  flag that is on from frame 1 and never off is not an IASA, and the export falls back to the
  script's, bounded by the total;
- **total** is the frames the state lasts when nothing ends it early. A run cut short in its IASA
  window, or by a landing, gives no total, and the fighter's last full run's total stays.

Unlike the export, the card runs in real exchanges, so frames frozen in hitlag are not counted.
Each move's last measurement is kept per fighter, so the next time the card has the numbers from
the move's first frame ("(measuring)" until then). The landing lag, the L-cancelled lag
(`ftCo_LandingAir_EnterWithLag`: the lag / PlCo `xE8`, at least 1) and autocancel come from the
script (`LE.move_static`), as the export's do. The card stays up as LAST MOVE until the next move
with a hitbox or a landing lag.

**Why D1 changed this.** D1's card read the script statically (`analyse`, as FRAMES does). On ACE
that read one frame high against the export and the known values: ftilt 6-9 vs 5-8, nair 5-32 vs
4-31. Its "total" was the script's length, not the state's. The static reading is still what the
FRAMES timeline, the state browser and the export's `autocancel` and `total_script` use, so those
may be one frame high too. **Open:** check `gd.timeline`'s frame numbering against a measured move
before relying on those.

The export now takes the script's IASA only when it falls inside the state; before, ftilt's 28 was
reported past its 26-frame end.

**The input display (`I`).**
- **The controls:** both sticks (an octagon gate with the position), A B X Y Z, and the L / R
  analog bars, as the game saw them (`gd.pad`).
- **The log:** each input with its frame in the sequence, e.g. `jump f1 > R f4`. Buttons are named
  (X / Y = jump), and stick flicks past 0.8 of a full tilt are named by direction. A sequence ends
  after 20 frames with no input. The last 5 sequences are kept.

**Tech feedback (`K`).** Per port, from the action changes and the game's own counters, with hit
rates:

| tech | how it is read |
|---|---|
| L-cancel | An aerial lands in its `LandingAir*` state. The game's L / R / Z press age (`player.lr_age` = `fp->x67F`) is compared with its window (`gd.lab_common().lcancel_window`, PlCo `xE4`, 7). The same test the game makes decides hit or miss. A miss says "N f early" (for presses up to 20 frames early), "N f late" (a press during the landing lag), or "no press". A plain `Landing` (autocancel) is listed but not scored. |
| wavedash | KneeBend > (JumpF/B) > EscapeAir > LandingFallSpecial. It reports how many frames after the jump's first airborne frame the airdodge came. The fighter procs run the animation (proc 0) before the input (proc 3), so a frame-perfect airdodge goes straight out of KneeBend and JumpF is never seen. That is scored as 0, "frame-perfect". |
| waveland | EscapeAir > LandingFallSpecial with no jump in the 12 frames before it: the airdodge frame it landed on (and the mean). |
| ledgedash | Off CliffWait (a drop or a jump) into an airdodge that lands within 60 frames: the ledge intangibility left on landing (GALINT, `player.intangible`, the timer `ftCo_CliffWait` starts). Scored as a hit when above 0. |
| hops | KneeBend > JumpF/B: short or full, by the take-off speed against the fighter's `hop_v_initial_velocity` / `jump_v_initial_velocity`, and the jumpsquat's length. Counted, not scored: `lab tech` prints "short N  full M". |

Not in stage D: dash-back and shield-drop timing. They need the game's own stick thresholds and
windows (like `lr_age` for the L-cancel) before the Lab can say "early" or "late".

**API added:**
- `player.lr_age`: `fp->x67F`, the frames since L / R / Z was pressed (255 = none).
- `player.jump_age`: `fp->x67E`, the same for X / Y.
- `gd.lab_common()`: `{lcancel_window, lcancel_div, hitstun_mul, kb_speed, kb_decay}`, PlCo as loaded.

The game half is `ScriptGame_LabI` (`LAB_I_LR_AGE`, `LAB_I_JUMP_AGE`) and `ScriptGame_LabCommonF`
(`LAB_C_LCANCEL_WINDOW`, `LAB_C_LCANCEL_DIV`).

**Console:** `lab adv` (the exchanges), `lab card`, `lab tech [clear]`, `lab actionable` (each
fighter's state, the actionable reading, IASA, hitlag, hitstun and `lr_age`: the thing to check
when a number looks wrong), `lab mode training`.

**Check off the game:** `pc/geno/tools/lab_stage_d_check.lua` runs `lab.lua` against a stub `gd`
with scripted frames:
- a hit and a shield exchange;
- L-cancels pressed on time, 4 f early, 2 f late and not at all;
- a frame-perfect wavedash and one 2 f late;
- a short hop, a waveland and a ledgedash with GALINT 5;
- the move card for a tilt and an aerial;
- the drawing, before and after a step back.

All 15 checks pass. It needs any Lua 5.4. It checks the Lab's reading of states and counters, not
the game.

**Checked in the game (D1, ACE):**
- It builds and runs, and `lab.lua` raised no errors.
- L-cancel on time, 1 f early and no press all read right.
- Wavedash "1 f late" and "3 f late" match the scripted inputs.
- Shine and jab on shield read +4 / -10, as noted above.
- The card disagreed with the export. That is fixed above.

**Still to check in the game:**
- Fox shine on shield. Check it against our own export first; compare with a published sheet
  only once there is one we trust (decision 3).
- Fox jab 1 on shield, and Marth fsmash on shield.
- L-cancel early/late counts from scripted `gd.input` presses at known frames.
- `lab card` against `lab export` for the same fighter.

### 14.13 Stage D3: the dummy

**What it is.** The Lab plays one fighter, the **dummy**, through `gd.input`, one frame at a time.
- The dummy must be a **human** port, because CPUs ignore pads. Leave its controller unplugged,
  or its inputs are overridden anyway.
- It is **off until you turn it on**: DUMMY tab > Dummy, then A. Left / right picks the port. A
  human P2 keeps its pad.
- It is offline only. It stands down while the history replays (the replay feeds the logged pads)
  and while it is recording.
- The settings persist in `scripts-data/geno-lab_lab/dummy.txt`. The pause menu's DUMMY tab edits
  them, and so does the console: `lab dummy` lists them, `lab dummy <key> <value>` sets one.

| setting | what it does |
|---|---|
| Record slot (`R` in TRAINING, or the menu) | Your controller drives the dummy while it records, and your own fighter stands still. This is `gd.mirror_pad(from, to, true)`: the "take" flag, new in stage D, leaves `from` neutral. `R` again stops. There are 4 slots of up to 10 s, saved in `dummy_rec<n>.txt`. |
| Record from a state | Starting a recording saves quick slot 2, and each playback of that slot loads it first, so it loops from the same spot. |
| Playback (`P` in TRAINING) | Off, in order, or random by each slot's weight (`lab dummy w_slot 1,1,0,2`). |
| DI | Knockback DI on every hit: none / in / out / survival / a fixed angle / random (`w_di`). The launch comes from the knockback preview of the hit's own numbers, and the stick is set with the preview's own DI rule (`gs_apply_di`): "in" is the perpendicular back toward the attacker's side, "survival" the one that ends nearer 45 / 135 degrees. It is held through hitlag. Stage E's real-hit check (`lab kb`, LAUNCH's K) now predicts with this stick (`LE.di_for`), not with no DI. |
| Clean DI (on) | SDI (`ftCo_Damage_OnEveryHitlag`) needs the stick past `sdi_min` **and** an axis that crossed the stick line (PlCo `x8`, **0.25** on ACE) fewer than `sdi_window` frames ago: a fast flick. A DI flick from neutral during hitlag is exactly that, and on ACE it moved "in" up to 28 units. Clean DI moves in two steps. First, each axis the DI uses goes just past the line (25 of 80), so the stick is far under `sdi_min` and there is no SDI. Then, after `sdi_window` + 1 frames, it goes to the full DI; no axis crosses the line again, so again no SDI. A pad the script writes reaches the game on the **next** frame, so the full stick is sent while 2 hitlag frames are left and the game has it on hitlag's last frame, when DI is read (on ACE it arrived a frame after hitlag, and "in" / "out" missed their prediction by 3-4 units). The first fix instead capped the stick under the line, which on ACE's 0.25 was ±19: no DI at all, and no tech rolls. The control stick still gives ASDI at hitlag's end when the C-stick is neutral; that is Melee, and the ASDI setting overrides it with the C-stick. `lab kb` predicts from where that nudge puts the fighter (`x4BC` × the held stick, `asdi_scale` in `gd.lab_common()`), so the check is not off by it. The stick there, and the DI stick the check predicts with, is the fighter's own float, not pad units: the radius clamped to 80 (the s8 truncates), / 80, then each axis at or under its dead zone (PlCo `x0` / `x4`, `stick_dz_x` / `stick_dz_y` in `gd.lab_common()`, 0.275 on ACE) reads 0, so a DI of (-79, 14) is (-0.975, 0) to the game. The first version multiplied by 80 again (an ASDI of 240 units: every held-DI check was off by ~240), then the unzeroed 14 turned the predicted DI ~1° too far (usmash at 70% "in" / "out": mean 5.3 / 5.6). Now on ACE, Fox usmash at 70% over the full 83-frame flight: none / in / out mean 0.25 / 0.25 / 0.25; ftilt at 50% "in" 0.45 (it lands after 16 frames). On ACE with Fox's ftilt at 50%: "in" mean error 2.8 before that, "none" 0.56; "out" turned the launch into the ground at once (hitstun 1, a 1-frame flight): use a higher launch to check it. Off = a full flick (DI plus one SDI). |
| ASDI | The C-stick held through hitlag: away / toward / up / down. |
| SDI | N flicks (one every 2 frames: in, then neutral) at the start of hitlag, in one direction. |
| Tech | In tumble (`DamageFly*` / `DamageFall`) it presses R when `gd.floor_below` says the floor is 5 frames away (the game takes a press in the 20 frames before contact, `ftCo_800986B0`, with a 40-frame lockout). The stick picks in place / away / toward (`ftCo_80098928` reads it at the landing). Other options are miss and random (`w_tech`). The press is tumble-only: tumble's interrupts have no airdodge (`ftCo_DamageFall_IASA`), but an air damage state out of hitstun would airdodge. **The roll's stick:** on ACE a full ±80 dropped tumble to Fall before the landing (the "wiggle": x past `tumble_wiggle`, smashed within `tumble_window`, or UCF 0.84's raw jump of 75+ in two frames), so away / toward landed normally on BF's platforms. The roll's stick is held **only near the landing**: from `tumble_window` + 2 frames before the press, x goes just past the stick line, then to full, so there is no fresh crossing (no wiggle) and the roll gets the full stick. Earlier in the fall the stick is neutral: a full sideways stick through a whole fall is drift, and on ACE it carried a fighter off a platform's edge and past the stage (a ledge grab at 48-51%, a death at 54%). With no floor under the drift at all, the stick goes toward the stage (x = 0). **Where it lands:** the floor is looked for under where the drift will have taken it by then. **Steering stays under the wiggle:** every steering stick (no floor under the drift, or a landing within 6 units of a floor's end, which steers back inward) is `tumble_wiggle` x 80 - 4 (60 on ACE, whose `tumble_wiggle` is 0.8), never a full ±80, and a reversal goes through neutral; otherwise the stick is neutral in DamageFall. A full ±80 had dropped DamageFall to Fall and lost the tech (BF upsmash at 45-60% off the main stage's edge: ledge grabs and an untech'd landing). **The roll direction rule:** a roll must end on the floor it lands on. Just before the press the floor 40 units past the landing in the roll's direction is checked (same height, within 2); when it is gone (a stage edge, a platform's end) the tech is in place instead, or the inward roll when the fighter is already steering back from the edge. So "away" near an edge never rolls off. |
| Getup | From DownWait: stand / attack / roll away / roll toward / random (`w_getup`). |
| Ledge | From CliffWait: getup (toward x = 0) / roll / attack / jump / drop / ledgedash / random (`w_ledge`). The ledgedash is a fixed script (drop back, jump, 2 frames, airdodge down-in), so its timing is approximate and varies by fighter. |
| After hitstun / shieldstun / landing | What it does on the first actionable frame (the same reading as frame advantage), the frame GuardSetOff becomes Guard, or when a landing's lag ends. The options are shield, spotdodge, roll away / toward, jump, attack, nair (jump then A), grab, or a recorded slot. |
| Reaction min / max | Every response waits a random number of frames between the two. |
| Percent lock | Put back to this percent after a hit (`gd.set_percent`, only when it changed). |
| Infinite shield | The shield is refilled to 60 when it falls under 50. `gd.set_shield`, new in stage D, writes only then, because every write forks the rewind timeline. |
| Hold / Shield tilt | When nothing else is running: shield (with a tilt), crouch, or jump. |

**What wins when several apply** (per frame): SDI / DI in hitlag first, then the tech press, then a
triggered response (after its delay), then playback, then the hold. With none of them it releases
the pad.

**Native API:**
- `gd.lab_common()` gains the PlCo stick lines the dummy keeps under: `stick_smash_dz` (`x8`),
  `tumble_wiggle` / `tumble_window` (`x210` / `x214`), `tech_roll_stick` (`x254`), `sdi_min` /
  `sdi_window` (`x4B0` / `x4B4`), `asdi_scale` (`x4BC`), and the fighter stick's per-axis dead
  zones `stick_dz_x` / `stick_dz_y` (`x0` / `x4`).
- `gd.floor_below(x, y [, depth])`: `mpCheckFloor`, read-only. It clears the bounding flags it
  sets, so it is safe between frames and rewind-exact.
- `gd.set_shield(port, health)`: offline; forks the timeline.
- `gd.mirror_pad(from, to, take)`.
- `player.shield_on / shield_x / shield_y / shield_r`: `shield_hit` as the game has it this frame.

### 14.14 Stage D2: hitbox display

The Lab's own drawing over the game's.
- **Where:** HITBOXES gets four toggles: `W` swept hitboxes (on), `U` hurtbox states (off), `S`
  shield bubble (on), `C` grab boxes (on). TRAINING and COMBO draw the swept hitboxes, the shield
  and the grabs when `B` is on.
- **Swept hitboxes:** the capsule from last frame's position to this frame's (`px, py` to `x, y`),
  the shape Melee tests hits against. The last 4 frames stay as fading ghosts.
- **Hurtbox states:** the capsules coloured normal yellow, intangible blue, invincible green. A
  whole-body timer (`player.intangible` / `invincible`) or body state overrides the capsule's own,
  with a chip giving the frames left: ledge, respawn, airdodge and spotdodge each read clearly.
- **Shield:** the bubble at the game's position and radius, with its health, coloured by health.
- **Grabs:** hitboxes with the `catch` element, in purple.

### 14.15 Stage D4: combo analysis

COMBO mode (`0`).
- **The reading, per hit on a victim:** the victim's first actionable frame after it (the frame
  advantage reading), set against the next hit.
  - **TRUE:** the next hit landed first.
  - **Escapable:** otherwise, "escapable N f" (how many frames the victim could act), with what it
    could do then: in the air jump (if it has one) / airdodge / aerial, on the ground shield / jump
    / spotdodge.
- **DI does not change the reading.** Hitstun does not depend on DI; what DI changes is where the
  victim is. So the panel also shows the **DI fan** (`D`): the last hit's flight for none / in /
  out / survival from the knockback preview, with the angle and hitstun, drawn to the end of
  hitstun.
- **Throws count.** A throw has no hit event, so on ACE upthrow > uair started at the uair and was
  dropped as a one-hit combo. Now **the grab and the throw are one hit** by the grabber (the fighter
  in `Catch*` / `Throw*`), named for the throw. Its damage is the victim's percent at the grab
  (`Capture*`) subtracted from its percent once it has **stopped changing for 16 frames** after the
  let-go (leaving `Thrown*`), or 40 frames after it at most: Fox's upthrow is 2% at the let-go plus
  three blaster shots 9-13 frames later (7.46% in all), and both a read at the let-go (2.0%) and a
  fixed 2-frame wait missed the lasers. The hit is dated at the let-go. The grabber's own hit events in that time are folded in, and a grab that ends
  without a throw (an escape) is no hit.
- **Lying down is able to act.** `DownWaitU` / `DownWaitD` (a missed tech, getup open) are free
  states now, for the combo reading and frame advantage alike. On ACE a hit on a downed P2 read
  TRUE; it is now escapable by the frames P2 lay there. `DownBound` (the bounce) still is not.
- **A lone throw is listed** as a one-hit entry (before, a throw whose follow-up missed printed
  "combo: none yet").
- **A combo** is a run of hits by one attacker on one victim. It ends with the reason:
  - "dropped: P2 could act for 30 f (jump / airdodge / aerial)";
  - "P2 hit back";
  - "a new exchange".

  The last 5 are kept. Console: `lab combo`.
- **Not built: the punish finder** (decision 2). It would try the attacker's moves in B's rewind
  after an exchange and list the ones that connect.

### 14.16 Stage D5: scenarios and drills

**A drill is data.** It has these keys:

| key | meaning |
|---|---|
| `name`, `desc` | shown in the DRILLS tab |
| `rule` | how an attempt is scored (below) |
| `score` | `streak` (the longest run of hits) or `rate` (the percentage) |
| `attempts`, `seconds` | the limits (0 = none) |
| `state` | a library state file to load first; otherwise the match as it is |
| `window` | techchase only: frames to land the hit |
| `dummy.<setting>` | dummy settings for the drill (weights as `in place:1,away:1`), put back when it ends |

**The rules** (the only code):
- `tech:<kind>` scores each D1 tech result of that kind on your port: `lcancel`, `wavedash`,
  `waveland`, `ledgedash`, `hop`.
- `techchase`: the dummy techs, rolls or gets up (the `Passive*` / `Down*` state starts), and
  you must hit it within `window` frames. The reaction time is logged.

**Built in:**
- **L-cancel streak:** 60 s, the score is the streak.
- **Tech chase:** 20 attempts, window 40, random tech and getup, the rate.
- **Ledgedash consistency:** 20, GALINT above 0, the rate.
- **Wavedash timing:** 20, frame-perfect, the rate.

The last two exist only as data, on the `tech:` rule.

**Your own drills.** A file per drill, `scripts-data/geno-lab_lab/drills/<id>.txt`, one
`key = value` per line, listed by id in `drills/index.txt`. DRILLS > Reload reads them again.

**Running one:**
- **Starting:** from the pause menu's **DRILLS** tab (between TOOLS and EXIT; the tab strip
  narrows to fit), or `lab drill <id>`. `lab drill` lists them and `lab drill stop` ends one.
- **HUD:** the name, the score, the streak or rate, the attempts or time left, and the last
  attempt.
- **Results:** a panel for 10 s: the score, hits of attempts, the mean (GALINT, reaction), the
  best before and NEW BEST, and the last attempts. Each run adds a line to `drills/results.txt`,
  and the best per drill is read back from it.

**Next candidates** (each needs a rule): shield-drop punish, wavedash out of shield, edgeguard
(the dummy's ledge options), DI survival (you are the victim; the dummy's recorded slot hits you).

**The pause menu and the mouse.** Stage C read `gd.mouse()` as a table (`m.pressed`), from before
the mouse existed. The public mouse returns four numbers (x, y, buttons, wheel), so every tick with
the menu open raised an error, and after 20 the Lab turned itself off. It now reads the four
numbers: a click is the left button's press edge (x, y are -1000 off the picture), and the wheel
moves the selection.

**The draw budget.** Twice on ACE a draw ran past the 50 ms script budget: once in the pause menu,
once at startup in `draw_strip`. Both were a first draw, where the Lab's textures load from disk. They
are now drawn off-screen a few at a time before the Lab UI first appears, at most 12 ms a frame. The
menu's detail text is also word-wrapped once per text, not every frame.

**Checks off the game:** `pc/geno/tools/lab_stage_d_check.lua` covers D1-D5 in 42 checks (32 at first).
Each fix from an ACE run added a check that fails on the build it fixes:
- D1, as in 14.12;
- D3:
  - SDI flicks, then DI "in" as the right perpendicular;
  - the tech press about 5 frames out, with the away stick held;
  - a ledge jump after a 3-frame reaction;
  - a spotdodge out of shieldstun;
  - recording through the take-mirror, the slot saved, and its playback pressing A;
- D4: a true second hit and a third escapable by exactly 3 frames; upthrow counted as the opener;
  a hit on a downed P2 escapable;
- the pause menu ticked and drawn with `gd.mouse`'s four numbers. This check fails on `da6b6ce` with
  the ACE error;
- D5: the L-cancel drill's count, streak and saved result, and a tech-chase hit;
- the D2 drawing, in HITBOXES with a swept hitbox, a grab box and a shield.

All pass. The checks in the game are the ones in 14.12, plus:
- the dummy's DI angle against LAUNCH's arc for the same DI;
- the tech press with a real `mpCheckFloor` (Battlefield platforms included);
- the swept capsules against the game's own hitbox draw.

### 14.17 Debug movement: fly (noclip) and teleport

A dev tool for moving a fighter anywhere on a map, for building and checking large maps. It works
in **every offline mode** (VS, the 1P modes, Adventure maps, the LAB), not only in the LAB, and is
**refused during a netplay or rollback session** (console, hotkey and `gd.*` alike).

**Flying.** The fighter goes into Fall, and its state callbacks are replaced by the fly ones
(`pc/geno/geno_lab_mode.c`, the "debug movement" section):
- **Stick:** moves it at the fly speed (default 2 units a frame at full stick). A held is ×0.25,
  B held is ×4. A resting stick (under 0.1 on an axis) does not move it. It faces the way it moves.
- **Physics:** there is no gravity and no knockback.
- **Collision:** none. The fighter passes through the stage and does not grab ledges. Its collision
  data is moved to it every frame, so leaving the flight never sweeps across the stage.
- **Blast zones:** it cannot be KO'd by them (`ft_0D31.c`), and the off-screen damage counter is
  held at 0.
- **Hurtboxes:** off (intangible) unless solid is on.
- **Camera:** while any fighter flies, the camera frames only the flying fighters and ignores the
  stage's camera bounds (`cm/camera.c`), so it follows one anywhere.

**Stopping.** "Off" drops the fighter into Fall where it is. "Place" puts it on the floor straight
below (`mpCheckFloor`), or drops it where it is when there is no floor. Anything else that changes
its action ends the flight by itself: a KO, a respawn, a hit while solid, the match ending. Nothing
is left behind.

**Teleport.** `tp` / `gd.teleport` sets the position exactly. A flying fighter stays flying there. A
fighter on foot is put into Fall at that spot, because a grounded fighter would snap back along its
floor line.

**Refused states.** A fighter that is dead, asleep, respawning, held or thrown, holding someone, or
one of the Hands cannot start flying (error "cannot fly").

**Determinism.** Everything is kept either in the Fighter (its callbacks) or in snapshotted statics
of `pc_geno_geno_lab_mode.c` (the speed and the solid switch). So a savestate, the Lab's step-back
and the rewind restore it exactly. Every change forks the Lab's rewind timeline, as a gameplay write
does.

**Controls** (the fighter is the fly port, P1 unless `fly port N` says otherwise):

| where | |
|---|---|
| `F11` | toggle the fly port's flight (any offline match; the window focused, the console closed). F11 is used nowhere else (14.9's keys, F1 / F9 / F10, the examples' F2 / F5-F8) |
| LAB pause menu, PLAY tab | **Fly (noclip)** toggles the focused fighter (F11 in the detail panel), **Fly speed** (left / right ×1.5), **Land here** (place) |
| console `fly [port] [on\|off\|place\|toggle]` | toggle with no argument; `noclip` is the same command |
| console `fly speed <n>`, `fly solid on\|off`, `fly port <n>`, `fly readout on\|off` | the speed (0.05-200), hurtboxes while flying, the hotkey's port, the readout |
| console `tp [port] <x> <y>` | teleport |
| console `pos [port]` | print the position and copy `x y` to the clipboard |
| `gd.fly(port [, mode])`, `gd.teleport(port, x, y)`, `gd.fly_speed([n])`, `gd.fly_solid([bool])` | the script API (docs/scripting.md) |

**The readout.** While a fighter flies, a small kit panel at the bottom left shows `FLY P1  x  y
speed`, one line per flying port. The host overlay draws it in every mode (`gw_console.cpp`,
`draw_fly_readout`), and `fly readout off` hides it.

**Checked** (2026-09-27, ACE, one game at a time):
- **Headless** (`geno_fly`, suite 196/196): the phys callback gives stick × speed with A / B, clears
  knockback, faces the stick, is intangible, and ignores drift. Solid mid-flight gives the hurtboxes
  back. The coll callback makes the collision data follow. A flier past the blast zones is not KO'd.
  The speed clamps.
- **In game, LAB (Fox vs Marth on Battlefield, 25/25):**
  - 30 frames right = +60.000 x.
  - 60 up = +120.000 y, and 30 idle frames = no drift.
  - B = −80.000 in 10 frames; A = −10.000 in 20 frames.
  - Straight down through the main floor to y −60.
  - 60 frames past the top, right and bottom blast zones with no KO and no stock or percent change.
  - A savestate while flying, then a load: the same frame (f580) gives the same position bit for
    bit (45.7000, 31.2000).
  - Marth's jabs: none land on a flying P1; with solid on they land (0 → 11.7%) and end the flight.
  - Place from (0, 45): standing at y 0.00, hurtboxes back.
  - After the flight it jumps (y 10.62) and lands.
  - Teleport on foot falls from the spot and lands; fly off mid-air falls.
- **Console over the socket (13/13):** `fly speed`, `fly 1 on`, `tp 1 12.5 77`, `pos` (landed on
  the top platform at 54.40), `noclip 1 off`, `fly` toggle, `fly place`, `fly solid`. Bad port,
  empty port and a bad `tp` give errors.
- **VS:** 20 frames = +40.00. The camera keeps P1 on screen at (600, 300), (−700, −250) and
  (1500, 900). It is back on the stage after place. The readout draws `FLY P1 x 40.0 y 70.0 speed
  2.00`.
- **Rollback session** (`MELEE_RB_LIVETEST`): `gd.fly`, `gd.teleport` and `gd.fly_speed` error.
  `fly`, `tp`, `fly speed` and `noclip` on the console answer "offline-only". The fighter moved only
  by its own walk.
- **Not machine-checked:** the F11 key itself (a minimized test window has no focus) and the LAB
  menu rows in a live game (the Lab checker draws them with stubs).

### 14.18 Input timing and turbo

`gd.input(port, spec, frames)` claims the whole pad and holds it for completed logic frames,
not PADReads. `gw_Script_PadFrameConsumed` runs after live logic frames; paused reads and
rollback/rewind resimulation do not consume a live hold. Once the hold ends, the claimed port
stays connected and neutral until release/task completion/unload. `gd.release_pad(port)` and
its alias `gd.release(port)` release early. Legacy text pad scripts still count PADReads.

The LAB's input-driven frame-data export steps one frame per tick during an input move so the
script can supply each sample before the next frame. General multi-frame `gd.step(n)` still exists.

`MELEE_FPS=u` uncaps interpolated presentation, keeping realtime logic at 60 Hz. `MELEE_TURBO=1`
or game `--turbo` runs scripted/LAB batch logic on a virtual clock without pacing and mutes audio.
It requires `MELEE_PAD_SCRIPT` or `MELEE_LAB_BATCH` and refuses netplay/Slippi/fake rollback.
`MELEE_TURBO_RENDER=N` presents every Nth game frame (default 8, 0 never). Hidden/minimised
windows never present. Unpresented match frames execute render callbacks but skip display
lists/skinning unless `MELEE_TURBO_DRAWS=1`. Use logic frames for holds; wall-clock performance
and screenshots need their own realtime checks. Workspace `run.sh --test` requests turbo unless
`--realtime` is supplied, but the headless test runner has no paced frame loop.

### 14.19 Offline scripting additions

Full signatures, bounds and return values are in workspace `docs/scripting.md`, checked against
`pc/platform/gw_script.c` registration and game-side implementations.

- `gd.set_damage(port, n)` writes real fighter/player damage (integer 0-999), including stamina
  HP accounting. It does not itself deliver the hit needed to finish an HP boss.
- `gd.hit(port, {damage=, angle=, kbg=, bkb=, from=})` feeds Melee's collision result and hit
  processing. Required integers: damage 0-500, angle 0-361, kbg/bkb 0-1000; optional source
  fighter slot `from`, otherwise environment damage. Returns acceptance, forks rewind.
- `on_boss_defeated{kind, port, x, y}` observes the Hand bosses' completed death. Gameplay
  scripts can delay progression with `gd.boss_hold([seconds])` (default 60, range 1/60-600)
  and release it with `gd.boss_release()`. The hold timeout counts match logic frames.
- `gd.stage_add_platform`, `gd.stage_add_line`, `gd.stage_move`, `gd.stage_remove`,
  `gd.spawn_target` and `gd.stage_add_model` create offline stage content. Up to 200 collision
  lines (also bounded by the 256-joint stage budget), 32 targets, 64 model branches and 8 DATs.
  Targets use Mato's real `GrTMr.dat` model. `gd.stage_add_model{file=, platform=handle, ...}`
  can carry an existing floor; `gd.stage_add_platform` has no `model=` field in this revision.
  `gd.stage_view` controls geometry/debug-overlay drawing.
- `gd.spawn_enemy(kind, x, y [, opts])` and `gd.enemy_remove(handle)` manage up to 32 Adventure
  enemies: goomba, koopa, redead, like_like, octorok, polar_bear. Defeats deliver
  `on_enemy_defeated{kind, handle}`; Koopa's shell transition and explicit removal do not.
- Stage/enemy writes require a gameplay script in an active offline match; console calls are
  refused. Camera writes allow the offline gameplay console: `gd.camera_get`, `camera_detach`,
  `camera_set`, `camera_move`, `camera_path`, `camera_follow`, `camera_shake`, `camera_bounds`,
  `camera_attach`. One script owns the camera; completion calls `on_camera_complete(kind)`.
- `gd.items`, `gd.fx` and `gd.perf` expose item/effect/performance diagnostics. Performance is
  host state, not deterministic gameplay data. Effect packages hide article placeholder DObjs
  when attached; compare the installed effect visually before calling the conversion complete.

## 15. v1 script encodings (STABLE reference for the Meta Knight translator)

This section is the contract the Brawl -> Geno script translator (ports/halberd/ (workspace repo) )
emits against. A copy lives at `ports/halberd/ (workspace repo) geno_v1_encodings.md`; this section
wins if they differ. **Numbers here never change**: new features get new sub / value / condition
ids. Constants: `pc/geno/geno.h`. Everything is ftcmd opcode 59 (first byte 0xEC-0xEF), words
big-endian like any Pl file script:

```
word0  [31:26] 59   [25:20] sub   [19:16] len (total words incl. word0, 1-15)   [15:0] sub-specific
```

Unknown subs are skipped by `len`, so always set `len` correctly.

### 15.1 Variables (v0, unchanged)

A **var ref** is 8 bits: `[7:6] bank, [5:0] index`. Banks: `0` LA int, `1` RA int, `2` LA float,
`3` RA float (64 vars each). LA = kept across actions (reset at spawn/respawn), RA = cleared on
every action change. PSA bit vars are bits of int vars (translator's choice of packing, e.g.
`RA.Bit[n]` -> RA int `n / 32`, bit `n % 32`).

Variable-sub layout of word0 `[15:0]`: `[15:8] var A`, `[7] B is a var`, `[6:4] cmp`, `[3:0] 0`.
`B` (word1) is an immediate (int, or float bits when A is a float var) or, with `[7]` set, a var
ref in `[7:0]` (converted to A's type).

| sub | name | len | effect |
|---|---|---|---|
| 0x00 | NOP | any | nothing |
| 0x01 | SET | 2 | `A = B` |
| 0x02 | ADD | 2 | `A += B` |
| 0x03 | SUB | 2 | `A -= B` |
| 0x04 | MUL | 2 | `A *= B` |
| 0x05 | SETBIT | 2 | `A |= 1 << B` |
| 0x06 | CLRBIT | 2 | `A &= ~(1 << B)` |
| 0x07 | DIV | 2 | `A /= B` (B == 0: A unchanged) **v1** |
| 0x0A | RAND | 2 | int A: `A = random 0..B-1` (B <= 0: 0); float A: `A = random [0, B)`. Game RNG (rollback-safe) **v1** |
| 0x10 | IF | 3 | if `!(A cmp B)` skip `word2` words (counted from the end of this command) |
| 0x11 | SKIP | 2 | skip `word1` words forward |
| 0x20 | CALL | 3 | native hook `word1` with argument `word2` |

cmp: `0` EQ, `1` NE, `2` LT, `3` LE, `4` GT, `5` GE, `6` BIT (`A & (1 << B)`), `7` NOBIT.

`if (c) {T} else {E}` = `IF !c ->skip |T|+2 ; T ; SKIP |E| ; E`. Skips only go forward. For a
backward jump (a loop) use Melee's own goto/loop commands (they wait frames), guarded by an IF.

### 15.2 Engine values (v1)

| sub | name | len | layout |
|---|---|---|---|
| 0x08 | GET | 2 | word0 `[15:8]` var A; word1 = value id. `A = value` (converted to A's type) |
| 0x09 | PUT | 3 | word0 `[7]` B is a var; word1 = value id; word2 = B (immediate in the **value's** type, or a var ref). `value = B`. Read-only values ignore it (logged once) |
| 0x12 | IFV | 4 | word0 `[7]` B is a var, `[6:4]` cmp; word1 = value id; word2 = B (value's type or var); word3 = words to skip when `!(value cmp B)` |

Value ids (type: `f` float, `i` int; `W` = writable):

| id | name | type | W | meaning |
|---|---|---|---|---|
| 0x00 | AIR | i | W | 1 in the air, 0 on the ground. Writing 1 on the ground makes the fighter airborne (Melee's own "become airborne"); writing 0 is ignored (landing needs a floor: use the ground check / on_land) |
| 0x01 | FACING | f | W | +1 right, -1 left. Writing: sign sets the facing, **0 turns around** (PSA "Reverse Direction") |
| 0x02 | VEL_X | f | W | self velocity x (world) |
| 0x03 | VEL_Y | f | W | self velocity y |
| 0x04 | GROUND_VEL | f | W | ground velocity (along the floor) |
| 0x05 | FWD_VEL | f | W | self velocity x times facing (forward-positive; PSA "Set Horizontal Speed") |
| 0x06 | KB_VEL_X | f |  | knockback velocity x |
| 0x07 | KB_VEL_Y | f |  | knockback velocity y |
| 0x08 | STICK_X | f |  | control stick x, -1..1 (world) |
| 0x09 | STICK_Y | f |  | control stick y |
| 0x0A | STICK_FWD | f |  | stick x times facing (forward-positive) |
| 0x0B | CSTICK_X | f |  | C-stick x |
| 0x0C | CSTICK_Y | f |  | C-stick y |
| 0x0D | ANIM_FRAME | f |  | current animation frame (0-based, as Melee counts it) |
| 0x0E | ACTION_FRAME | i |  | frames spent in the current action (1 on its first frame) |
| 0x0F | MOTION | i |  | current Melee motion (action state) id |
| 0x10 | PERCENT | f |  | damage percent |
| 0x11 | JUMPS_USED | i | W | jumps used (ground jump counts; 1 in the air after leaving the ground) |
| 0x12 | JUMPS_MAX | i |  | max jumps |
| 0x13 | BUTTONS_HELD | i |  | Geno button mask (section 3) of held buttons |
| 0x14 | BUTTONS_PRESSED | i |  | Geno button mask of buttons pressed this frame |
| 0x15 | POS_X | f |  | position x |
| 0x16 | POS_Y | f |  | position y |
| 0x17-0x1A | CMD_VAR0-3 | i | W | Melee's script variables `fp->cmd_vars[0..3]` (what Melee's own "set cmd var" writes and special states read) |
| 0x1B | ANIM_RATE | f | W | animation speed (`fp->frame_speed_mul`). **v5.3: writable** - `ftAnim_8006F0FC` sets the clip rate and the script pace together (Ultimate FT_MOTION_RATE; values <= 0 ignored) |
| 0x1C | FAST_FALL | i |  | 1 while fast-falling |
| 0x1D | TRIGGER | f |  | analog shield trigger, 0..1 |
| 0x1000 + i | SPECIAL_F[i] | f |  | special attribute word i (`fp->dat_attrs`), read as float, i < 265 |
| 0x2000 + i | SPECIAL_I[i] | i |  | special attribute word i, read as int |

### 15.3 Change action (v1)

| sub | name | len | layout |
|---|---|---|---|
| 0x30 | CHG | 2-4 | word0 `[15:8]` condition, `[7]` B is a var, `[6:4]` cmp, `[3]` NOT, `[2]` ONCE; word1 = **target**; word2 = arg1; word3 = arg2 |
| 0x31 | CHGAND | 1-3 | word0 `[15:8]` condition, `[7]`, `[6:4]`, `[3]` NOT as CHG; word1 = arg1; word2 = arg2. Adds an AND condition to the most recent CHG of this action (PSA "Additional Change Action Requirement"). Max 3 conditions per CHG |
| 0x32 | CHGCLR | 1 | removes every change-action check of this action |

Semantics (Brawl's): a CHG **registers** a check. Without `ONCE` it is checked **every frame for
the rest of the action** (cleared by any action change). With `ONCE` it is checked once, at the end
of the frame it was registered in, then dropped. Checks are tested in registration order, first
match wins. Registered checks run every frame after the animation and script advance and **before
the state's own animation callback**, so an "animation end" check beats the state's own anim-end
transition. Up to 8 checks per fighter (a 9th is dropped, logged). `CHG ALWAYS ONCE` = "change
action now" (at the end of this frame's script pass - never in the middle of a script).

Landing: a registered check whose (first) condition is GROUND is also tested **at the moment of
landing** inside the collision callback, and wins over the state's own landing transition (it runs
right after the callback). Likewise AIR at the moment of leaving the ground there. So PSA
`Change Action X, requirement On Ground` is exactly `CHG GROUND -> X`.

Conditions (`[15:8]`):

| id | name | arg1 | arg2 |
|---|---|---|---|
| 0 | ALWAYS | | |
| 1 | ANIM_END | | | the animation has no frames left |
| 2 | GROUND | | | on the ground |
| 3 | AIR | | | in the air |
| 4 | PRESSED | Geno button mask | | any of the buttons pressed this frame |
| 5 | HELD | Geno button mask | | any of the buttons held |
| 6 | BIT | var ref | bit index | bit set (NOT for "bit clear") |
| 7 | VAR | var ref A | B (A's type, or var ref with `[7]`) | `A cmp B` |
| 8 | FRAME | frame N (int) | | animation frame >= N |
| 9 | VALUE | value id (section 2) | B (value's type, or var ref with `[7]`) | `value cmp B` |

Geno button mask: bit 0 ATTACK (A), bit 1 SPECIAL (B), bit 2 JUMP (X or Y), bit 3 SHIELD (L or
R, digital or the analog trigger past Melee's shield threshold), bit 4 GRAB (Z), bit 5 TAUNT
(D-pad up). Brawl's requirement button ids 0-5 are these bits (`1 << id`).

**Target word:**

```
[31:28] kind   0 MOTION  id = Melee motion id (common 0-340, fighter specials 341+)
               1 SPECIAL id = the fighter's special action n (motion = first special + n)
               2 GENO    id = Geno state (v2; v1 skips such a check - logged - and a later
                         matching check still fires, so emit a fallback CHG after it)
[27]    RAW        plain Fighter_ChangeMotionState, even for the common states below
[26]    KEEP_FRAME continue at the current animation frame with Melee's mid-move transition
                   flags (hitboxes, GFX, SFX kept) - Melee's way to swap the air/ground version of
                   a move, i.e. Brawl's "Change Subaction" to the other-situation variant. Implies RAW
[25:16] 0
[15:0]  id
```

Without RAW, these common states enter through Melee's own entry function (so they are set up
the way the game sets them up): Wait (on the ground; in the air it becomes Fall), Fall,
FallSpecial (helpless fall, default landing lag), Landing, LandingFallSpecial. Every other target
is a plain `Fighter_ChangeMotionState(id, 0, frame 0, speed 1, blend 0)` into that motion's row -
the state's callbacks run, its special entry code (if any) does not.

### 15.4 Hitbox helpers (v1)

| sub | name | len | layout |
|---|---|---|---|
| 0x38 | REHIT | 2 | word0 `[15:8]` hitbox mask (bit i = Melee hitbox id i, 0-3); word1 = N frames (0 = stop) |
| 0x39 | LINK | 2 | word0 `[15:8]` hitbox mask; word1 = mode: 0 off, 1 autolink direction, 2 direction + speed |

**REHIT** = Brawl's rehit rate: every N frames (counted from the command, frozen during the
attacker's hitlag) the victim lists of those hitboxes are cleared - exactly what Melee's
multi-hit moves get by re-creating a hitbox. Lives until the action ends or `REHIT mask 0`.
Emit it right after the hitbox creation (Brawl: `Offensive Collision ... rehit rate 6` ->
Melee hitbox + `REHIT mask(id) 6`).

**LINK** = Brawl angle 365 for these hitboxes (keep Melee's angle field at any value, e.g. 361 or
the Brawl number - LINK overrides it for Geno fighters only; without LINK a Melee hitbox angle is
used as Melee uses it). On a hit, the victim is launched along the attacker's momentum (its air
velocity, or its ground velocity along the floor); when the attacker is nearly still (speed below
0.05) the hitbox's own Melee angle applies unchanged. Mode 1 keeps the hitbox's Melee knockback magnitude (hitstun as Melee
computes it). Mode 2 also raises the knockback so the victim's launch speed is at least the
attacker's speed. This is the agreed approximation of Brawl's 365, not a bit-exact copy.
Lives until the action ends or `LINK mask 0`.

### 15.5 Script overlays and on_land (geno.json)

```json
"subactions": [ { "index": 87, "words": ["0xEC220001", 5, "0x08000014"] },
                { "index": 88, "file": "geno/mk_fair.txt" } ],
"special_attributes": [ { "index": 13, "float": 16.0 }, { "offset": "0x2C", "int": 10 } ],
"on_land": [ { "from": "special:4", "to": "special:6" },
             { "from": 66, "to": 43, "keep_frame": false } ],
"hooks": { "on_land": ["geno.count_frames:5"] }
```

- `subactions`: replace subaction (animation + script) `index`'s **script** with these words
  (numbers or "0x.." strings; a `file` is whitespace-separated words, `#` comments). Loaded once at
  boot, read-only. Inside an overlay, sub **0x13 ORIG** (len 1) continues with the fighter's
  original script of that subaction from its start - so an overlay can be "Geno prefix + ORIG".
- `special_attributes`: override the fighter's special attribute block (`dat_attrs`) word by word
  (`index` = word, or `offset` = bytes); the value is `float` or `int`. Applied before the
  fighter's own attribute setup copies/scales them, at every spawn and attribute re-apply.
- `on_land`: when the fighter lands while in action `from`, it goes to `to` (after any script
  GROUND check, which wins). Targets: a number (motion id), `"special:N"`, `"motion:N"`,
  `"geno:N"` (v2). `keep_frame` = the KEEP_FRAME bit.
- `hooks.on_land`: native hooks at the moment of landing.

### 15.6 Brawl PSA -> Geno cheat sheet

| PSA | Geno |
|---|---|
| Change Action X, req R | `CHG R -> X` |
| Additional Change Action Requirement R | `CHGAND R` |
| Change Subaction X, req On Ground / In Air | the Melee action whose subaction is X, `KEEP_FRAME` (or a plain target) |
| Change Action Status 10000/10002 (Wait/Fall group) | `CHG ... -> Wait` / `-> Fall` (common entry) |
| If On Ground / In Air | `IFV AIR EQ 0/1` |
| If Compare IC.x | `GET` the value into a var, then `IF`, or `IFV` directly |
| If Button Pressed n | `IFV BUTTONS_PRESSED BIT n` |
| Set Air/Ground (to air) | `PUT AIR 1` |
| Reverse Direction | `PUT FACING 0` |
| Set/Add Horizontal Speed | `PUT FWD_VEL` (add: GET, ADD, PUT) |
| Set Vertical Speed | `PUT VEL_Y` |
| Offensive Collision rehit N | Melee hitbox + `REHIT mask N` |
| Offensive Collision angle 365 | Melee hitbox + `LINK mask 1` |
| Roll A Die n | `RAND A n` |

### 15.7 How v1 hooks into the engine (and why it stays m-ex compatible)

| site | what | inert fighters |
|---|---|---|
| `fighter.c` `Fighter_8006A360`, before `anim_cb` | `Geno_PreAnim`: action frame +1, REHIT timers, registered CHG checks (first match changes the action; the old state's anim callback is then skipped, as when it changes state itself) | returns 0 at once |
| `fighter.c` `Fighter_procMap`, around `coll_cb` | `Geno_CollBegin` / `Geno_CollEnd`: a landing / take-off inside the callback may pick a target, performed right after it | return at once |
| `ftcommon.c` `ftCommon_8007D6A4` (land), `8007D5D4` / `8007D60C` (become airborne) | `Geno_GroundEdge`: on_land hooks; GROUND / AIR checks; geno.json `on_land` | return at once |
| `ftcoll.c` `ftColl_8007A06C` (fighter hitbox won the hit) | `Geno_Autolink`: LINK rewrites dir / angle (/ kb) | returns 0 at once |
| `ftchangeparam.c` (existing) | special attributes written with the common ones, before the fighter's own attribute code | profile -1: return |
| `Fighter_UnkInitReset_80067C98` (existing) | subaction overlays installed (idempotent) | profile -1: nothing |

"Inert" = no geno.json profile for the kind and no escape ever executed by its scripts - every
vanilla and m-ex fighter. Edges outside a collision callback (a hit launching a grounded fighter,
a state entry calling "become airborne") never pick a change target, so a GROUND/AIR check can
never hijack a damage state.

**Rollback / determinism.** Every v1 field is in `GenoState` (`pc/geno/geno_state.h`): action
frame, up to 8 checks (target + 3 conditions each), REHIT period/count and LINK mode per hitbox,
the collision-edge scratch. The overlay pool (`Geno_ScriptPool`) and the original-script table are
game globals too, written only with the same values (idempotent). RAND uses the game's RNG (not
drawn in the fast-forward pass). No host pointers, no host time. Tests `geno_state_savestate`
(v1 fields survive a gw_snap round trip) and the netplay_local run in section 15.9.

### 15.8 Approximations and limits (read before translating)

- **Autolink 365** is an approximation: direction = attacker's momentum (air velocity, or ground
  velocity along the floor), Melee knockback magnitude (mode 1) or at least the attacker's speed
  (mode 2). Hitstun, DI, SDI, meteor rules stay Melee's. Brawl's exact 365 math (victim velocity
  set relative to the attacker each frame) is not reproduced. Attacker speed below 0.05: the
  hitbox's own Melee angle.
- **REHIT** clears the victim list on a fixed period from the REHIT command (frozen in the
  attacker's hitlag). Brawl counts per victim from its last hit; for a victim hit on the first
  frame (the usual case) they agree; a victim entering mid-window can be rehit up to N-1 frames
  earlier than in Brawl.
- **Change action** into a special state enters its motion row (callbacks) but not its own entry
  function: a special that needs setup (m-ex state variables) should be entered with a script /
  MoveLogic that sets itself up on its first frame. The common Wait / Fall / FallSpecial /
  Landing / LandingFallSpecial go through Melee's entry functions. Special targets are bounded to
  256 past the first special; the row must exist. Geno-state targets (kind 2): v2, section 16.
- **Brawl "Change Subaction"** (same action, other animation) has no v1 opcode: use the Melee
  action that plays that subaction, with KEEP_FRAME for the air/ground-variant case.
- Overlays cannot hold Melee commands with **absolute pointers** (goto / subroutine into the Pl
  file); use Geno SKIP/IF for forward jumps, Melee's loop commands for loops, ORIG to continue with
  the original script.
- PRESSED SHIELD sees digital L/R only (HELD also sees the analog trigger).
- Not in v1: `on_hit` dispatch (v5, section 19), `air_vy` for non-multi-jump fighters, a "change
  subaction" op.


### 15.9 v1 verification

- Tests (`--test`, 149/149 on ACE, vanilla and Akaneia): `geno_registry_v1` (native parse of the
  v1 keys), `geno_v1_values`, `geno_v1_change_action`, `geno_v1_ground_edge`, `geno_v1_rehit`,
  `geno_v1_autolink`, `geno_v1_special_attrs`, `geno_v1_overlay`, `geno_v1_inert`, and the v1
  fields in `geno_state_savestate`.
- Demo mod `_build/agents/beta/mods-geno-v1/geno-v1-demo` (not committed): vanilla Kirby on the
  ACE disc. Nair (subaction 68) overlay = REHIT 4 + LINK + PUT VEL_Y hop + IFV STICK_FWD boost +
  GET special word + CHG GROUND -> LandingFallSpecial + CHG ANIM_END & AIR -> FallSpecial + CHG
  A & FRAME >= 12 -> fair, then ORIG; `on_land` fair -> ftilt; `special_attributes` word 5 (first
  air jump 3.0). Pad `geno_v1_pad.txt`; log lines `geno: ... rehit`, `autolink`, `change action`,
  `landed in motion 66 -> target 0x00000035`; shot `_build/agents/beta/shots/geno_v1_1500.png`.
- Rollback: `_build/netplay_local.ps1 -Disc ace` with the demo mod on both sides (EnvHost /
  EnvGuest `MELEE_MODS_DIR`), Kirby (pad-driven, v1 features) vs Wolf; results in NOTES.md.

## 16. v2: Geno action states, glide, native specials (STABLE reference)

Status: **built** (v2). This section is the contract for geno.json v2 and for the Meta Knight
translator; a copy of the MK part lives at `ports/halberd/ (workspace repo) geno_v2_encodings.md`
(this section wins). Numbers here never change. Code: `pc/geno/geno_game_v2.inc` (states, tables),
`geno_game_glide.inc`, `geno_game_specials.inc`; registry `pc/platform/geno_registry.c`.

### 16.1 Geno action states

A profile's `"states"` list declares new action states. **State n is Melee motion id
`0x400 + n`** (`GENO_MOTION_BASE`): past every fighter's own specials (Kirby's table, the largest,
ends at 0x220), so no vanilla or m-ex range check ever matches it. At spawn Geno builds one
`MotionState` row per state; `Fighter_ChangeMotionState` asks `Geno_MotionRow` for rows in that
range (one `#if TARGET_PC` branch in fighter.c; any other id takes the vanilla / m-ex path
untouched). The state is then an ordinary Melee action state: the subaction script runs (hitboxes,
GFX, SFX, Geno v1 commands), hurtboxes work, and damage, grabs, death, ledge grabs and landing
leave it through Melee's own code. Geno never edits the vanilla or m-ex tables.

```json
"states": [
  { "name": "GlideStart", "behavior": "geno.glide.start", "subaction": 57 },
  { "name": "Glide",      "behavior": "geno.glide",       "subaction": "motion:29" },
  { "name": "Plain",      "behavior": "geno.air", "subaction": 18, "like": "motion:65",
    "flags": "0x55", "move_id": 13, "next": "geno:Glide", "land": "helpless",
    "landing_lag": 12, "anim": "next", "iasa": "like", "phys": "auto", "coll": "air" }
]
```

| key | meaning |
|---|---|
| `name` | for targets `"geno:<name>"` (geno.json); scripts use the index |
| `behavior` | a native behaviour (16.2): the four callbacks + an entry routine + default targets |
| `subaction` | the animation + script: a subaction index of the fighter's own files, or `"motion:N"` / `"special:N"` = the subaction that motion plays. Omitted: the like motion's |
| `like` | the motion whose row gives the flags, move id and camera callback (default: Fall for air behaviours, Landing / LandingFallSpecial for ground ones, AttackAirN for GlideAttack) |
| `flags`, `move_id` | override the row's `x4_flags` / `move_id` (staling / attack id) |
| `anim` / `iasa` / `phys` / `coll` | override one callback by name (16.2), or `"like"` = the like motion's own callback |
| `next` | where the state goes when its animation ends (behaviours that use it) |
| `land` | where it goes when it lands (air collision); default Melee's Landing |
| `landing_lag` | with the default landing: LandingFallSpecial with this lag |

Numeric `subaction` and the LAB's `anim_id` are the same row number. A Geno state reuses that
row's animation and effective script, including any `subactions` overlay on it. `lab events`,
the timeline, move card and frame-data export all read it through `gd.timeline`.
Conditional scripts label the static fall-through path; exports include `script_conditional`
and omit conditional static IASA/autocancel estimates, while sampled frame data remains measured.

Targets (everywhere in geno.json: `next`, `land`, `on_land`, `specials`): a motion id, `"motion:N"`,
`"special:N"`, `"geno:N"`, `"geno:<name>"`, `"auto"` (Wait on the ground, Fall in the air),
`"helpless"` (Wait / FallSpecial).

**Script entry (v1 encoding, now live).** CHG target kind 2 `GENO` (`0x2000000n`) enters state n:
`CHG ANIM_END -> GENO(0)` + `CHGAND AIR` is Shuttle Loop's "Change Action Glide on animation end,
in the air". A GENO target the profile does not declare is still skipped (logged) and a later
check fires, so the translator's "Geno check first, fallback after" pattern keeps working on any
exe. KEEP_FRAME works on GENO targets too. GROUND / AIR edges and `on_land` accept GENO targets.

**Specials bound to states.** `"specials": {"n": "geno:Tornado", "s": "geno:Drill"}` (keys `n s hi
lw air_n air_s air_hi air_lw`; an `air_*` key defaults to its grounded one). The eight special
dispatch sites (ftCo_Attack100.c, ftCo_SpecialAir.c, ftCo_SpecialS.c) ask `Geno_SpecialEnter`
first; unbound specials (and every fighter without a profile) run their own / m-ex code.

**Engine values (v2):** `0x1E GENO_STATE` (i: current state index, -1 none), `0x20..0x27 MOVE_F0-7`
(f W) and `0x28..0x2F MOVE_I0-7` (i W): the behaviour's per-move variables (below), readable and
writable from scripts (GET / PUT / IFV / VALUE conditions).

### 16.2 Behaviours and callbacks

| behaviour | anim | iasa | phys | coll | next (default) | land |
|---|---|---|---|---|---|---|
| `geno.air` | next | none | air | air | auto | Landing |
| `geno.ground` | next | none | ground | ground | auto | - |
| `geno.glide.start` | glide.start | none | glide.start | air | the Glide state | GlideLanding |
| `geno.glide` | glide | glide | glide | glide | the GlideEnd state | GlideLanding |
| `geno.glide.attack` | next | none | glide.attack | air_noledge | Fall | Landing |
| `geno.glide.landing` | next | none | ground | ground | Wait | - |
| `geno.glide.end` | next | none | glide.end | air | auto | Landing |
| `geno.tornado` | tornado | none | tornado | both | `next`, else helpless | (stays in state) |
| `geno.drill.start` | next | none | drill.start | both | the Drill state | (stays) |
| `geno.drill` | drill | none | drill | drill | the DrillEnd state | (stays) |
| `geno.drill.end` | drill.end | none | drill.end | both | helpless unless it hit | (stays) |

"The Glide state" = the profile's first state with that behaviour. Callback names per slot:
anim `next loop hold glide.start glide tornado drill drill.end`; iasa `none interrupt glide`; phys `none air
air_nodrift ground auto glide.start glide glide.attack glide.end tornado drill.start drill
drill.end`; coll `none air air_noledge ground ground_stop both glide drill`; any slot `like`.
`air` = Melee's aerial physics (gravity, fast fall, drift) / air collision with ledge grab
(platforms drop-through like FallSpecial). `both` = ground and air without changing state (walk
off -> airborne, land -> grounded, same state). Stable ids in geno_game_v2.inc.

**IASA trap:** the default `iasa` is `none` for ordinary `geno.air` / `geno.ground` states.
An IASA script command sets the flag but does not provide an input callback. Use
`"iasa": "interrupt"` to make the flagged window accept normal interrupts. At fighter load,
Geno warns once per state per registry load if the effective script contains IASA while its
effective callback is none, naming the fighter and state. The diagnostic follows calls,
ORIG and both IF/IFV arms with bounds (2000 commands, 32 pending paths, 8 calls); it does not
execute the script. Callback edits require a reload restart (14.10).

Per-move variables (`GenoState.move_f[8]` / `move_i[8]`) are **not** cleared by action changes:
the states of one move hand them on (GlideStart -> Glide -> GlideAttack). Each behaviour's entry
sets what it needs. `hold_motion` / `hold_frames` are the glide entry's counter.

### 16.3 Parameters

Per profile, in family blocks; each family also takes its Brawl block word by word (`w00`..).

```json
"glide":   { "hold_frames": 16, "w07": 2.2, "end_buttons": 14 },
"tornado": { "tap_cooldown": 10 },
"drill":   { "angle_max": 75, "speed": 2.2 }
```

**glide** (Brawl Misc Glide block, MK values as defaults):

| word | name | MK | used for |
|---|---|---|---|
| w00 | angle_max | 80 | nose-up limit (deg) |
| w01 | angle_min | -70 | nose-down limit |
| w02 | start_vy | 0.75 | GlideStart: vy += w02 |
| w03 | start_gravity | 1.0 | GlideStart gravity multiplier |
| w04 | start_vx | 1.0 | GlideStart: vx *= w04 |
| w05 | speed | 1.7 | Glide's initial speed |
| w06 | speed_accel | 0.04 | speed change per frame at +-90 deg |
| w07 | max_speed | 2.2 | velocity magnitude cap |
| w08 | stall_speed | 0.7 | below it the glide stalls (ends) |
| w09 | sink_accel | 0.03 | sink growth per frame |
| w10 | max_sink | 0.6 | sink cap |
| w11 | recover_angle | 15 | Brawl's stall recovery (the stall ends the glide first) |
| w12 | dive_angle | -25 | steeper dives get a bonus |
| w13 | dive_bonus | 0.03 | max extra speed per frame (at angle_min) |
| w14 | - | 0.15 | not read by Brawl's code |
| w15 | deadzone | 0.25 | stick magnitude dead zone |
| w16 | pitch_up | 0.55 | angular accel nose up (deg/f^2) |
| w17 | pitch_down | 0.75 | angular accel nose down |
| w18 | max_pitch_rate | 7 | deg/f |
| w19 | stall_pitch | 1.0 | stall auto pitch-up |
| w20 | wing_node | 44 | Brawl's wing partial-animation joint (not used) |
| w21 | - | 0 | not read |

Geno extras: `hold_frames` 16 (jump held in an air jump), `from_ground_jump` 0, `entry` 1 (0 = only
scripts enter), `end_buttons` 14 (GENO_BTN mask: shield 8 + special 2 + jump 4), `max_frames` 0 (no
time limit, as Brawl), `end_helpless` 0, `pose_center` 0 (16.4), `landing_lag` (unused by the
defaults).

**tornado** (MK paramSpecialN; param ids 4000-4016 float, 24000-24001 int):
w00 entry_vy 1.0, w01 entry_vx_mul 0.7, w02 start_rate 80, w03 ground_accel 0.12, w04 ground_speed
2.0, w05 air_accel 0.1, w06 air_speed 1.7, w07 brake 0.008, w08 gravity -0.08, w09 max_fall 0.5,
w10 tap_vy 1.0, w11 tap_cooldown 10 (int), w12 max_rise 1.4, w13 tap_rate 16, w14 max_rate 80, w15
rate_decay 1.5, w16 spin_frames 70 (int), w17 late_decay 2.0, w18 end_rate 10, w19 -. Extras:
`max_speed` 2.5, `end_helpless` 1.

**drill** (MK paramSpecialS; ids 4017-4020, 24002): w00 start_vx_mul 0.5, w01 start_vy 1.0, w02
start_gravity -0.08, w03 steer 3.0 (deg per frame at full stick), w04 end_frames 10 (int), w05 -.
Extras: `speed` 2.0 (used when the clip has no root motion), `angle_max` 0 (0 = no limit, as
Brawl), `bounce` 0 (v3; was 7 in v2: 1 wall, 2 hit, 4 shield - Brawl has no such check, 17.5),
`pop_vx` 1.0, `pop_vy` 2.1, `end_helpless` 1 (v3: FallSpecial after an air end, hit or not).

The param-id numbering is now confirmed (fn_111_81A8's getters): float ids count only the float
words (4011 = N w12, 4012 = N w13, ..., 4015 = N w17, 4016 = N w18, 4017-4020 = S w00-w03), int
ids 24000 = N w11, 24001 = N w16, 24002 = S w04.

### 16.4 The recovered Brawl logic (what is exact, what is approximated)

Recovered by disassembling `sora_melee.rel` (ftStatusUniqProcessGlide init/exec/fixPos,
ftMetaknightStatusUniqProcessSpecialNSpin, MK kinetic types 0x64-0x68) and `ft_metaknight.rel`
(SpecialSRush, SpecialSEnd, the param accessor fn_111_81A8), with an annotated REL disassembler
over capstone (relocations and rodata resolved). The glide reaches its block through
`soValueAccesser::getConstantIndefinite(43019)`; common ids used: 3023 gravity, 3024 terminal
velocity, 3029 max air speed, 3030 air friction, 3032 hard x cap 2.5.

**Glide entry.** Jump (X/Y, or the stick past Melee's tap-jump threshold) held continuously for
`hold_frames` frames of an air jump (JumpAerialF/B, or a Kirby/Puff multi-jump state) enters
GlideStart. A release cancels it for that jump. Scripts enter it with CHG GENO.

**GlideStart** (Brawl 0x84): on entry `vy += w02`, `vx *= w04`; per frame Melee gravity x w03 to the
terminal velocity, vx braked toward 0 by the air friction, |vx| <= 2.5. Animation end -> Glide.

**Glide** (0x85), per frame (exact port of execStatus):
```
init:  speed = w05; sink = -vy(entry); angle = rate = 0
stick: a = stick angle mirrored into the facing frame; mag = |stick|
  if mag > w15:  acc = (a >= 45 or a < -135) ? +w16 : -w17     (back/up = nose up)
                 acc *= (mag - w15) / (1 - w15); if rate*acc < 0: rate = 0
                 rate = clamp(rate + acc, +-w18); angle += rate
  (neutral: angle and rate hold)
angle = clamp(angle, w01, w00)
speed -= w06 * angle / 90   (+ 0.01 penalty while touching a wall); speed >= 0
if angle < w12: speed += w13 * (w12 - angle) / (w12 - w01)
v = (facing * speed * cos(angle), speed * sin(angle)); sink = clamp(sink + w09, +-w10); v.y -= sink
if |v| > w07: v *= w07 / |v|
if |v| < w08 or speed <= 0: stall -> the glide ends (GlideEnd), as Brawl's action does
```
Exits: A -> GlideAttack; a pressed `end_buttons` button (Brawl's PSA: requirement 0x30 args 1 and
2, read as special / jump; shield added per GD) -> GlideEnd; stall -> GlideEnd; landing ->
GlideLanding; `max_frames` > 0 adds a time limit (Brawl has none). GlideAttack / GlideEnd -> Fall.

**Mach Tornado** (exact logic; Brawl units used unscaled):
entry vx *= w01, in the air vy += w00; spin rate r = w02, countdown w16. Per frame: frames-since-lift
+1; while the countdown runs: r -= w15, a B press arms a lift; if armed and >= w11 frames since the
last lift: lift (grounded -> airborne), r += w13; r in [0, w14]; after the countdown r -= w17.
Air: an accepted lift adds vy += w10, rise capped at w12, gravity w08 to a fall of w09; drift accel
stick*w05 (+ Melee's aerial drift base), stable |stick|*w06, brake w07 above it, |vx| <= 2.5.
Ground: accel stick*w03 to |stick|*w04, brake w07. Moving into a wall reverses vx and turns around.
The spin ends when r <= w18 -> `next` (MK: TornadoEnd, the final hit) -> helpless in the air.
Not ported: Brawl drives the spin animation's rate by r (Melee plays the clip at rate 1, looping).

**Drill Rush:** start: vx *= w00, air vy = w01, gravity w02 (optional DrillStart state). Rush:
pitch += stickY * w03 per frame (up climbs for either facing; `angle_max` optional limit, Brawl has
none - its 45-frame clip bounds it to +-135); velocity = the clip's root motion (Melee TransN) or
`speed`, rotated by the pitch; on the ground it can only pitch up (and takes off). End: pitch eases
to level (x(1 - 2/w04) per frame); in the air a pop back/up (pop_vx / pop_vy: Brawl's air-end
momentum -1 / +2.1), then helpless unless the rush hit something (Brawl's SpecialSEnd enters both
Fall and FallSpecial; which one when is inferred). **Bounce** (a Geno addition: Brawl's code has none
besides that end pop): wall / hit / shield (`bounce` mask) -> DrillEnd at once. **v3 corrects this
from Brawl's code: 17.5** (no bounce by default, the air end is always helpless, ends by situation).

**Approximations / not recovered:** the stall-recovery branch (w11, w19) is ported but the stall
ends the glide first (as Brawl's action does); Glide_Landing's extra thresholds (common params
3169/3170) are not applied (every landing -> GlideLanding); Brawl's wing partial animation (w20)
and Glide_Direction's pose-by-angle need MK's clips: `pose_center` (e.g. 90 with the 181-frame
Glide_Direction clip) scrubs the animation to `pose_center - angle` (untested until Phase 2 ships
the clip); the Drill's real speed is its SpecialSDrill TransN curve (needs the converted clip; else
`speed`); units are not rescaled (Brawl and Melee both use units/frame; tune per fighter).

### 16.5 How v2 hooks in (compatibility)

| site | what | fighters without a profile |
|---|---|---|
| fighter.c `Fighter_ChangeMotionState` | motion id >= 0x400 -> `Geno_MotionRow` | never reach it |
| ftCo_Attack100.c / ftCo_SpecialAir.c / ftCo_SpecialS.c | `Geno_SpecialEnter` before the special dispatch | returns 0 at once |
| geno_game.c `Geno_PreAnim` | after v1 checks: the glide's jump-hold entry | returns 0 at once (inert) |
| pc/gameworld/script_game.c (Lab) | motion rows for Geno ids | n/a |

Rollback: rows (`Geno_Rows`), parameters (`Geno_Params`) and row counts are game globals rebuilt
idempotently at every spawn from the registry (same bytes every time); all behaviour state is in
`GenoState` (move vars, hold counter) and the Fighter struct. No host state, time or randomness.
`GENO_ID_VERSION` stays 1: a v1 entry keeps its id; v2 keys are hashed like any key.

### 16.6 v2 verification

- Tests (`--test`, ACE): `geno_registry_v2` (parse: states, names, callbacks, targets by name,
  parameter blocks and Brawl words, specials), `geno_v2_states` (rows: subaction by index / by
  motion, like copy, flags, move id, like callbacks, fallback row), `geno_v2_change_to_state` (CHG
  GENO performed, undeclared falls through, GENO_STATE value, specials bound / unbound / no
  profile), `geno_v2_glide_entry` (16-frame hold, release cancels, not from Fall),
  `geno_v2_glide` (pitch limits, dive gains / climb loses, savestate of the move vars, A / shield
  exits, stall -> GlideEnd), `geno_v2_specials` (tornado sink / lift cooldown / rise cap / drift,
  drill steering limit, bounce -> DrillEnd moving back).
- Demo and netplay: `_build/agents/beta/NOTES.md` (Geno v2 entry).

## 17. v3: root-motion states, Dimensional Cape, Drill Rush from Brawl's code (STABLE reference)

Status: **built** (v3, `"geno": 3`; every v3 key is additive, a v2 exe reads the v2 keys and logs
the rest). Code: `pc/geno/geno_game_v2.inc` (geno.anim_motion, the KEEP_FRAME carry, the glide
start frame), `geno_game_specials.inc` (cape, drill), `geno_game_glide.inc` (helpless glide),
registry `pc/platform/geno_registry.c`. MK translator copy: `ports/halberd/ (workspace repo) 
geno_v2_encodings.md` section 6.

### 17.1 geno.anim_motion: the clip's root motion, on the ground and in the air

Brawl moves many specials by the clip's TransN (Shuttle Loop, the cape's reappears). Melee extracts
TransN only on subaction rows whose flags have `0x80000000` (`fp->x594_b0`: the model's TransN is
zeroed and the per-frame delta lands in `fp->x6A4_transNOffset`, z forward / y up, already
model-scaled), and only a few ground helpers use it. `geno.anim_motion` (anim `next`, phys and coll
`anim_motion`) uses it everywhere:

| situation | rule |
|---|---|
| ground | `gr_vel = forward x facing`; with `"liftoff"` (default on) an upward delta takes off and the air rule moves that frame; walking off an edge = airborne, same state |
| air | `self_vel = (forward x facing, up) + gravity`; `gravity` = the state's `"gravity"` multiplier of Melee's gravity, accumulated from the state's start (0 = pure root motion, Brawl's motion kinetics), overridden per action by `MOTION_GRAVITY` |
| no root motion (row flag clear) | ground: friction; air without gravity: holds still (Brawl's "stall" start frames) |
| first frame | `"origin"`: also moves by the clip's frame-0 offset (Brawl clips whose frame 0 already sits one frame into the motion: SpecialHi z 0.77, SpecialHiLoop y 6.3 / z 6.7) |
| facing | `"facing": "entry"`: the travel keeps the facing the state was entered with, so a mid-clip Reverse Direction turns the model and the hitboxes, not the path |
| ledge | this action's ledge grab: `LEDGE` value if a script set it (PSA Allow/Disallow Ledgegrab), else the state's `"ledge"`: `none` / `front` (Melee's own) / `both` (mpColl tests both ledges, dir 0; ftCliffCommon turns MK to the ledge). Melee only catches a ledge while falling |
| landing | the state's `"land"` (a Geno state, a motion; `"stay"` = only become grounded, the state goes on) or Melee's landing (`"landing_lag"` -> LandingFallSpecial with that lag) |

New state keys: `"ledge"`, `"liftoff"`, `"origin"`, `"gravity"`, `"facing"`; new target `"stay"`.

**KEEP_FRAME entries** (`CHG ... -> GENO(n) KEEP_FRAME`, and the engine's own ground / air swaps)
carry the root-motion progress (no second origin step, the same entry facing, the gravity so far,
the ledge setting) and set `GenoState.enter_keep`, so behaviours skip their entry setup on a swap.

### 17.2 Engine values (v3)

| id | name | type | W | meaning |
|---|---|---|---|---|
| 0x30 | LEDGE | i | W | this action's ledge grab: 0 none, 1 front, 2 front and back; -1 = the state's default. Cleared by every action change (Brawl's Allow/Disallow Ledgegrab values 1 / 2 map 1:1 - inferred) |
| 0x31 | HIDDEN | i | W | 1 = the fighter is not drawn (Melee's FighterVis flag `x221E_b5`: model, shadow). Kept across Geno states (re-applied on every Geno state entry: ChangeMotionState clears the flag), cleared by any non-Geno action (damage, death show the fighter) |
| 0x32 | TRANSN_FWD | f | | this frame's root motion, forward (0 without root motion) |
| 0x33 | TRANSN_UP | f | | this frame's root motion, up |
| 0x34 | MOTION_GRAVITY | f | W | this action's `geno.anim_motion` gravity multiplier; -1 = the state's (PSA Disable / Enable Horizontal Gravity) |

Melee already hides the whole fighter with FighterVis (script op 37 / `x221E_b5`: the body model,
shadow and nametag checks all read it); HIDDEN is Geno's way to keep it across a move of several
states. Both live in the Fighter struct / GenoState: rollback-safe.

### 17.3 Behaviours (v3)

| behaviour | anim | iasa | phys | coll | next | land |
|---|---|---|---|---|---|---|
| `geno.anim_motion` | next | none | anim_motion | anim_motion | auto | Landing |
| `geno.cape` | cape | none | cape | cape | (decides at `decide_frame`) | (stays) |
| `geno.cape.attack` | next | none | anim_motion | cape.after | helpless | stay |
| `geno.cape.end` | next | interrupt | anim_motion | cape.after | helpless | stay |

New callback names: anim `glide.after`, `cape`; iasa `interrupt` (Melee's IASA command opens
Wait's / Fall's interrupts, as Melee's attacks do); phys `anim_motion`, `cape`; coll
`anim_motion`, `cape`, `cape.after`, `drill.start`. Glide attack / end use `glide.after`.

**Order conventions.** Paired states are found by declaration order among the profile's states
with that behaviour: `geno.cape` (start ground, start air), `geno.cape.attack` (N, N air, F, F air,
B, B air), `geno.cape.end` (ground, air), `geno.drill.start` (ground, air), `geno.drill.end`
(ground, air). A ground state that leaves the ground, or an air one that lands, swaps to its
partner with the frame kept (Brawl's ground / air subaction loops).

### 17.4 Dimensional Cape (Brawl actions 0x115 / 0x11B / 0x11C, recovered)

Sources: the PSA (actions 277 / 283 / 284, subactions 472-481) and MK's kinetic types in
`sora_melee fn_27_359014` (0x69 / 0x6A read paramSpecialLw, ids 4021-4026; nothing else reads them).

- **Start** (`geno.cape`, 20-frame clip, the script runs to 26): the momentum is kept at w00 (x 0.5)
  / w01 (y 0.4) with no gravity (kinetic 0x69); frame 12 hides MK (HIDDEN) and switches to kinetic
  0x6A: each axis accelerates by w02 / w04 (0.5) toward stick x w03 / w05 (2.5), capped at 2.5, no
  gravity (on the ground only x steers; stick up takes off). Intangible from 17 (script). At
  `decide_frame` 26: Special or Attack **held** (`attack_buttons`, Brawl requirement 0x32) -> the
  slash reappear; the variant is the stick x relative to the facing at that frame: |x| <
  `neutral_x` (0.3, inferred) N, back -> Brawl's "F" clips, forward -> "B" clips; nothing held ->
  the plain reappear.
- **Slash reappear** (`geno.cape.attack`, 56 frames): root motion (the clips' TransN; MK's "F" and
  "B" clips travel 15 / 17 toward the stick), `"facing": "entry"`; visible and tangible at frame 1;
  N and B turn around at 1 (PUT FACING 0); 14% f6-7 (two TopN boxes). Ground -> Wait, air ->
  FallSpecial.
- **Plain reappear** (`geno.cape.end`, 36 frames): visible at 0, ledge grab both, ground IASA 28
  (iasa `interrupt`), air falls from 10 (MOTION_GRAVITY 0 then 1). Ground -> Wait, air -> FallSpecial.
- Parameters (`"cape"`): w00 keep_vx 0.5, w01 keep_vy 0.4, w02 steer_accel_x 0.5, w03 steer_max_x
  2.5, w04 steer_accel_y 0.5, w05 steer_max_y 2.5; steer_frame 12, decide_frame 26, neutral_x 0.3,
  attack_buttons 3 (special | attack).
- Not ported / open: Brawl's air-N PSA also has `Change Action Fall` on requirement 0x0F ("button
  tap?") after its Bit16 at frame 2 - read as "not always" (the slash would never hit otherwise);
  the vanish's kept stick energy into the reappear (0E04 at 5 / 20) is not modelled (the reappear
  moves by its clip only); the cape article's separate clips (the cape is MK's own merged mesh,
  ModelVis(0, 2)).

### 17.5 Drill Rush, from Brawl's code (v2 corrected)

Sources: `ft_metaknight.rel` SpecialSRush (exec C710, exit C820), SpecialSEnd (exec C9B8, exit
CB48), sora kinetic types 0x66-0x68 and the drill angle hook `fn_27_359CE4`, the motion energy
update `fn_27_15DC14`; neither class reacts to a hit (checkDamage returns 0, checkAttack is empty).

| | Brawl | Geno v2 | Geno v3 |
|---|---|---|---|
| hit / shield / wall during the rush | nothing: the rush always runs its 45-frame clip (hitlag pauses it), rehit 6 | ended the rush (bounce 7) | nothing (`bounce` default 0; the option stays) |
| pitch | stick Y x 3 deg a frame, no limit (<= 135 over the clip), rotates the TransN vector, up climbs for either facing | same | same |
| speed | the clip's TransN delta (about 2 u/f), no gravity | same (abs) | same (signed) |
| on the floor | situation forced to air; the floor blocks it (slides) | could only pitch up | pitch accumulates, slides along the floor, takes off when pitched up |
| rush ledge grab | Allow Ledgegrab 1 at 20 | none | PUT LEDGE 1 at 20 |
| end choice | at the floor -> SpecialSEnd, else SpecialAirSEnd | air end + CHG GROUND ONCE | the end state of the situation (ground first, then air) |
| air end | velocity (-1 x facing, +2.1), FallSpecial at the end, LandingFallSpecial on landing, ledge grab 2 | FallSpecial unless the rush hit | FallSpecial always; landing -> LandingFallSpecial; ledge both |
| ground end | Wait at the end, off an edge -> Fall | Wait | Wait; off an edge -> Fall |
| start | ground: vx x 0.5 (ground brake); air: vx x 0.5, vy = 1.0, gravity -0.08; ground / air swap with the frame kept, no second x 0.5 | one state per situation, no swap | swap with the frame kept (coll `drill.start`), speeds untouched |
| pitch ease in the end | x (1 - 2 / 10) a frame | same | same |

### 17.6 Glide fixes (v3)

- **Entry pop**: a Glide state now starts its clip at `pose_center - angle` (the glide always starts
  level: frame 90 of MK's 181-frame GlideDirection). Before, the entry frame showed clip frame 0
  (nose straight up) until the phys callback scrubbed it the next frame.
- **Helpless after a script-entered glide**: Brawl's Shuttle Loop sets LA-Bit61 on its way into the
  common Glide, and that glide ends helpless. `glide.script_entry_helpless` (default 1): a Glide
  entered from anything but the GlideStart state is marked (`move_i[4]`); its GlideEnd /
  GlideAttack go to FallSpecial at their end and land in LandingFallSpecial.

### 17.7 Limits and state count

`GENO_MAX_STATES` is 48 (was 16). Rows are game state (`Geno_Rows[32][48]` MotionState, about 48 KB
more per snapshot, rebuilt idempotently at spawn). MK uses 30.

### 17.8 v3 verification

- Tests (`--test`, ACE, 161/161): `geno_v3_anim_motion` (air both facings, ground, lift-off, origin,
  gravity, ledge default / PUT LEDGE / reset, savestate), `geno_v3_hidden_glide` (HIDDEN across
  states, the glide start frame, helpless script-entered glide), `geno_v3_many_states` (40 states),
  `geno_v3_cape` (momentum kept, steering, decision, partner swap keeps the counter, entry facing),
  `geno_v3_drill` (start swap, unlimited pitch, a hit does not end the rush, the end by situation,
  helpless air end), `geno_lab_stale_fighter`.
- In game (Meta Knight, `ports/halberd/ (workspace repo) tools/ingame_v2.py`, "v3" plans) and netplay:
  `_build/agents/beta/NOTES.md` (Geno v3 entry).

## 18. v4: the model follows Meta Knight's specials (STABLE reference)

Status: **built** (additive keys; a v3 exe logs them as unknown and keeps its behaviour). Code:
`pc/geno/geno_game_specials.inc` (`geno_drill_pose`, `geno_tornado_spin_*`). MK: `tools/build_mk.py`
sets the keys; checks `ports/halberd/ (workspace repo) tools/ingame_v4.py` (numeric, joints / hitboxes).

### 18.1 Drill Rush turns the model (`drill.pitch_model`, default 1)

Brawl keeps the rush pitch on the **model**, not only on the travel: SpecialSRush's execStatus
(ft_metaknight C710) is `rot = posture.getRot(); rot.x += -clamp(stickY) * param 4020 (3.0);
posture.setRot(rot)` (soPostureModule slots 0x40 / 0x44), and MK's kinetic angle hook
(sora fn_27_359CE4) derives the travel from it: `angle = -rot.x * lr * deg2rad`. The rush's exit
(C820) clears rot unless the next status is SpecialSEnd (0x11A); SpecialSEnd's exec (C9B8) eases
`rot.x -= 2 * rot.x / RA-Basic[2]` (the entry sets RA-Basic[2] once from a constant: `end_frames`, 10)
and its exit (CB48) clears it. SpecialSStart never touches it; nothing else in ft_metaknight.rel
does (Dimensional Cape and the tornado have no posture rotation).

Geno: TopN's rotation X (`ftPartSetRotX(fp, 0, -pitch)`, Melee's equivalent of the posture rot, as
Kirby's Final Cutter and the slope tilt use it) is set from `move_f[0]` whenever the rush or the end
changes the pitch and on every entry (rush, end, both situations). `-pitch` turns the nose up for
either facing (TopN's Y rotation holds the facing). Every joint, the hitboxes (the sword's) and the
hurtboxes turn with it, rigidly about TopN (the fighter's position). Any other action clears it
(Fighter_ChangeMotionState, Brawl's exit). Rollback: derived from snapshotted state every frame.
Measured in game: the model turn equals pitch x facing within 0.0004 deg, rigid within 0.0002 units,
hitboxes within 0.002 deg, the airborne travel along the nose within 0.004 deg (18.3).

### 18.2 Mach Tornado's body spins at the spin rate (`tornado.spin_anim`, `tornado.spin_period`)

Brawl's spin rate **is** the SpecialNSpin clip's playback rate: the spin action's entry sets
`Frame Speed Modifier = IC-Basic[4002]` (w02, 80), SpecialNSpin's execStatus (sora 358CF4) reads it
back with soMotionModule::getRate (slot 0x20, +0x4C), applies the decay / lift and writes it with
setRate (0x24), and the PSA ends the spin when IC-Basic[24] (the frame speed) is <= 10. MK's clip is
361 frames of YRotN rotY = frame degrees, so the body turns `rate` degrees a frame (80 at the start,
10 at the end). The whirlwind (EfBmData model 20, gfx 5019) is spawned once on TopN (bone 0) and keeps
its own 30-frame loop: Brawl never touches its rate, and neither does Geno.

Geno (`spin_anim` > 0): the joints' animations (the model and the blend skeleton, as ftAnim_8006F0FC)
play at `spin_anim x move_f[0]`, set on entry and every phys frame, looping (AOBJ_LOOP) at
`spin_period` (MK 360: frame 360 = frame 0) instead of ending. `fp->frame_speed_mul`, the subaction
script's clock, stays 1: the script's hitbox loop keeps its timing (Brawl's own script loop is 180
clip frames, i.e. a rehit every 180 / rate game frames - not ported, see 18.4). Hitlag skips the
fighter's animation step, so the spin pauses with it, as in Brawl. The clip frame lives in the joints'
AObjs (game heap, snapshotted); the rate is re-derived from move_f[0] every frame. `spin_anim` 0 = v3
(rate 1, the state restarts the clip when it ends).

| key | id | default | MK |
|---|---|---|---|
| `drill.pitch_model` | 0x4E | 1 | 1 |
| `tornado.spin_anim` | 0x3A | 0 | 1 |
| `tornado.spin_period` | 0x3B | 0 (the clip's end) | 360 |

### 18.3 v4 verification

- Tests (`--test`, ACE, 163/163): `geno_v4_drill_pose` (rot x = -pitch every rush frame, both
  facings, steering down, the end keeps and eases it, `pitch_model` 0 keeps the model level) and
  `geno_v4_tornado_spin` (joint rate = the spin rate from the entry, -1.5 a frame, +16 per lift with
  the cooldown; `spin_anim` 0 leaves the animation alone).
- In game (`ingame_v4.py`, 13 plans): each steered drill is compared with a straight drill at the
  same clip frame (joint offsets from TopN, hidden parts whose matrices are not recomputed left out);
  the tornado's clip frame equals the running sum of the rate rebuilt from the inputs (0.0000 frames)
  and the shoulders turn about YRotN by the rate every frame (within 0.1 deg). `ingame_v2.py` 34/34.

### 18.4 Open (not changed by v4)

- Tornado rehit: Brawl's SpecialNSpin script re-creates its 4 hitboxes every 180 clip frames (at rate
  80: every 2.25 game frames; at 10: every 18); the port rehits every 10 game frames.
- The special's own B press arms the tornado's first lift (the phys reads `pressed_buttons` on the
  entry frame): +16 at frame 10 (and, on the ground, the lift-off), about 10 frames more spin. Whether
  Brawl's first execStatus still sees the press as a trigger is not recovered.

## 19. v5: articles, the on_hit hook, counter windows (STABLE reference)

Status: **built** (`"geno": 5`; keys are additive, but an older build cannot be assumed to execute newer commands). Code:
`pc/geno/geno_game_articles.inc` (game half), `pc/platform/geno_registry.c` (parse, model load),
engine sites in `item.c` and `fighter.c` (19.5). First customer: Sora (Firaga / Blizzaga / Thundaga
are articles, Counter Attack is a counter window), but nothing here is Sora-specific.

### 19.1 Articles are Melee items

A profile's `"articles"` list declares projectiles. **Article a of profile p is item kind
`0x1000 + p * 8 + a` for a=0..7, or `0x1100 + p * 8 + (a - 8)` for a=8..15**
(`GENO_ART_KIND`, 16 articles per profile; the original first-eight ids are unchanged): above m-ex's custom kinds
(237 + the disc's count) and below m-ex's `5000 + n` "the spawner's own article n" remap, so no
vanilla or m-ex table is ever indexed with it. `Item_80267978` asks Geno for the kind's descriptor
(`Article`: `ItemAttr`, model desc, no hurtbox) and logic table; `Item_802674AC` gives it the
character-article hold kind (no item-count cap). From there it is an ordinary item.

**Reused, not reinvented:**

| what | Melee / m-ex machinery used |
|---|---|
| spawn | Fox's laser path (`it_8029C504`): `SpawnItem` + `Item_InitSpawn*`, `Item_80268B18`, `Item_80268E5C` (one state, `anim_id` -1), `db_80225DD8` |
| movement, drawing, blast zones | the item procs: `x40_vel` is added to the position by `Item_802697D4`; the generic item renderer (`it_8026EECC`, as m-ex custom items) |
| hitboxes | the item hitbox setup of the item script's hitbox command (`it_802790C0`: `it_8026FCF8`, `it_80272460` with the owner's staling / scale, `it_80275594`, `it_8027129C`), so they hit, clank, and are reflected / absorbed / countered like the laser |
| stage contact | the laser's ray test between the last and this position (`it_8026E9A4`) |
| despawn | the logic table's predicates (`dmg_dealt`, `hit_shield`, `shield_bounced`, `clanked`, `absorbed`) returning true, the anim callback at the lifetime's end |
| a dying owner | `it_8026B894` (the laser's reference cleanup) |

What Geno adds: the descriptor built from geno.json, the three state callbacks (life + hitbox
windows, motion, stage contact) and the reactions. Halberd's cape is **not** a precedent here: it was
merged into MK's model (section 17.4), it never was an item.

**Models.** `"model": {"file", "symbol"}` names an HSD archive (a costume-style `.dat` in the mod's
`files/`, or any disc file) and its joint symbol (default: the first public `*_joint`). The registry
loads it **once** into Geno's withheld guest region (`shim_os.c` `GW_GENO_PERSIST_SIZE`, 1 MB taken
off the main heap below m-ex's region - reserved only when a mounted mod's geno.json defines
`"articles"`, decided once at boot by `gw_Geno_ModelRegionWanted` and logged `geno: article model
region RESERVED` / `not reserved`; a mod enabled later by a hot reload gets no models until a restart) and relocates it there, so it outlives every scene heap (a
model loaded into a match's heap is freed with it). Read-only after the load. No model = an
invisible article (hitboxes only).

**A model with an `"fx"` package is hidden (v5.7 note).** When an article names an effect package, the package is its look and
the model is treated as a placeholder: the spawn hides every display object of the model's joint tree
(`geno_art_hide_model`; the joints keep following the article, so the effects still do) and logs
`geno: article N model: 2 (...)` (event 45). This is by design, not a draw bug: a model that **looks** never-drawn next to a retail model is
almost always one whose article also has an `"fx"`; any model draws as an article without one (a procedural
`fighterbuild` model and a retail `PlFx*.dat` alike; checked in the LAB, `_research/luigi-ultimate-grab-2026-10-07.md` section 7).
`"show_model": true` (v5.7, `GENO_AP_SHOW_MODEL`, default false) keeps both drawn: the model under the package (event 45, value 3).

### 19.2 geno.json

```json
"articles": [
  { "name": "Fire",
    "model": { "file": "GnSoraFire.dat", "symbol": "fire_joint" },
    "scale": 1.0, "spin": 0,
    "lifetime": 60,
    "spawn": [8, 8],
    "velocity": [2.5, 0], "gravity": 0, "max_fall": 0, "accel": 0, "max_speed": 0,
    "homing": { "turn": 0, "range": 0, "delay": 0 },
    "max_live": 4,
    "despawn": { "hit": true, "shield": true, "stage": true, "clank": true },
    "hitboxes": [ { "damage": 7, "size": 4, "offset": [0, 0, 0], "angle": 45, "kbg": 50, "wkb": 0,
                    "bkb": 30, "element": 1, "shield_damage": 0, "sfx_severity": 1, "sfx_kind": 0,
                    "start": 1, "end": 0,
                    "hits": { "ground": true, "air": true, "reflect": true, "absorb": true,
                              "counter": true } } ] }
]
```

| key | default | meaning (param id, `GENO_AP_*`) |
|---|---|---|
| `lifetime` | 60 | frames alive; then it despawns (0) |
| `velocity` | [0, 0] | initial [forward, up]; forward follows the owner's facing (1, 2) |
| `gravity`, `max_fall` | 0 | `vy -= gravity` a frame, not below `-max_fall` (0 = no cap) (3, 4) |
| `accel`, `max_speed` | 0 | speed change a frame along the travel; speed cap (0 = none) (5, 6) |
| `homing.turn` / `.range` / `.delay` | 0 | turn at most `turn` degrees a frame toward the nearest fighter that is not the owner (its position + 5 up), only within `range` (0 = any) and after `delay` frames (7, 8, 9) |
| `spawn` | [0, 0] | spawn offset from the owner's position, [forward, up] (10, 11) |
| `scale` | 1 | model scale (`ItemAttr` scale) (12) |
| `despawn` | all true | what ends it besides the lifetime: its hitbox hit something, it hit a shield, it touched the stage, it clanked (13, a mask). An absorber always takes it |
| `spin` | 0 | model roll, degrees a frame (visual) (14) |
| `max_live` | 4 | at most this many of this article per fighter; a spawn past it is refused (15) |

Hitboxes (up to 4, Melee's item limit; `GENO_AH_*` ids 0-15): `damage` (1), `size` (3), `offset`
(from the article's root joint), `angle` (361), `kbg` (100), `wkb`, `bkb`, `element` (Melee's:
0 normal, 1 fire, 2 electric, 3 slash, 5 ice ...), `shield_damage`, `sfx_severity` (1), `sfx_kind`,
`start` / `end` (the active frames of the article's life, 1-based; `end` 0 = to the end; a window
re-opening clears the victim list, so two windows = two hits), `hits` (who it hits and who can take
it). One hit per victim per window, as Melee's items.
v5.5: `stun` (0-255, default 0): extra hitstun frames a hit by this hitbox gives the victim (Ultimate's `set_add_reaction_frame_revised` on the article's attack; the fighter-side command is HBSTUN, section 20 table). Checked: Firaga with `stun` 15 on Fox: hitstun 18 -> 33.

**Spawning:** native hook **5 `geno.article.spawn`**, argument = the article's index: from a script
(`CALL 5 n`, section 15.1), from a geno.json dispatch point, or a subaction overlay at the frame the
move throws it. Reflected: the reflector owns it and the travel turns with the new facing (as the
laser). The spawn, the despawn reason and a refused spawn are logged (`geno: ... article`).

### 19.3 on_hit: dispatch point 4

`"hooks": {"on_hit": ["geno.log:77"]}` runs when the fighter is **hit**: in
`Fighter_ProcessHit_8006D1EC`, before Melee reacts to the frame's hit (a hit = knockback to apply or
damage to take). Dispatch points are stable numbers (`GENO_EV_*`): 0 on_init, 1 on_frame,
2 on_action, 3 on_land, **4 on_hit**. The hit's context is in engine values first, so hooks and
scripts read it:

| id | name | type | meaning |
|---|---|---|---|
| 0x35 | HIT_DAMAGE | f | the last hit's damage (before a counter dropped it) |
| 0x36 | HIT_PORT | i | its attacker's port 0-5 (an article counts as its owner), -1 unknown |
| 0x37 | HIT_COUNTER | i | 1 when it landed in a counter window |
| 0x38 | HIT_COUNT | i | hits taken since the spawn |
| 0x39 | ARTICLES | i | this fighter's Geno articles alive now |

All read-only; HIT_* persist until the next hit (reset at spawn / respawn).

### 19.4 Counter windows

A Geno state's `"counter": {"from": 5, "to": 25, "target": "geno:CounterAttack", "negate": true}`.
A hit taken in that state on an action frame in [`from`, `to`] (ACTION_FRAME, 1-based; defaults 1
and forever) is **countered**: with `negate` (default) the knockback and the damage are dropped
before Melee applies them (the hitlag both sides already got stays, as with Marth's counter), the
fighter goes to `target` (any target, section 16.1; none = stay), HIT_COUNTER is 1, then the on_hit
hooks run. `"negate": false` only flags the hit (the fighter takes it; a script / hook decides).
The attacker's hitbox is not asked whether it is counterable (`hits.counter` concerns Melee's
shield-bubble counters); a Brawl/Ultimate-style counter multiplier on HIT_DAMAGE is the move's
script: read HIT_DAMAGE, scale it, and apply it with HBDMG (sub 0x3A; section 19.12).

### 19.5 How v5 hooks in (compatibility)

| site | what | everyone else |
|---|---|---|
| `item.c` `Item_80267978` | kinds `0x1000..0x11FF` -> `Geno_ArticleDesc` / `Geno_ArticleLogic` | the vanilla / m-ex ranges, before the m-ex custom branch |
| `item.c` `Item_802674AC` | those kinds: hold kind 8 | unchanged |
| `fighter.c` `Fighter_ProcessHit_8006D1EC` | `Geno_OnHit` first | returns at once (no profile) |
| `shim_os.c` | 1 MB withheld under m-ex's region, only if a mounted mod defines articles | full main heap otherwise |

Rollback: descriptors, attributes and model descs are game globals written only with registry values
(idempotent); the model data is read-only; all per-frame state is the item itself (game heap,
snapshotted like every item: position, velocity, hitboxes, and Geno's vars in its per-kind block)
and the `GenoState` hit fields. `GENO_VERSION` is 5; the LAB state library checks that version as well as build/disc/mod/Geno
identity before loading. `GENO_ID_VERSION` stays 1.

### 19.6 Limits and approximations

- Straight / ballistic / homing travel only; no bounce off the floor, no ground-riding articles (Link's
  bomb, Samus's missile are future behaviours), no per-article animation (the model is static; `spin`
  rolls it). Articles have no hurtbox (they cannot be hit, only clank).
- One state per article. An article that changes behaviour mid-life (Thundaga's cloud then bolt)
  is two articles, the second spawned by the first's owner script, or a later `"phases"` key.
- Homing aims at the nearest non-owner fighter, not the nearest opponent by team.
- Spawn offsets can use the owner or a configured bone (section 19.8); check the source skeleton
  mapping when converting another game's article attachment.
- The model region is a 1 MiB bump allocation, reserved only when a mounted mod defines articles.
  Models are shared by file+symbol within a registry load; reload/allocation limits still matter.

### 19.7 v5 verification

- Tests (`--test`, ACE, 188/188): `geno_v5_registry` (article numbers and defaults, hitboxes, the
  `hits` / `despawn` masks, counter windows, the on_hit list, kinds -> descriptors, undeclared kinds),
  `geno_v5_article_motion` (a straight shot flies 2.5 u/f for 40 frames = 100 u; gravity to max_fall;
  homing turns exactly 5 degrees a frame after its delay; max_speed), `geno_v5_on_hit` (a hit before the
  window stands and runs the hook; in the window it is negated and goes to the target; a state without a
  window; a flag-only window; GET HIT_*; CALL `geno.article.spawn` from a script; no profile = inert).
- In game (ACE, demo mod `_build/agents/beta/mods-geno-v5/geno-v5-demo`, Lua pad
  `_build/agents/beta/geno_v5_pad.lua`, Wolf (P1, ck 34) vs Fox (P2), both driven, FD): neutral B ->
  state Shoot, whose overlay `CALL 5 0` at frame 5 spawns the article (Fox's costume, scale 0.35,
  355,712 bytes loaded once into the Geno region): item kind 4096, it hit Fox for **7.0%** and despawned
  (reason 1 hit). Shot away from Fox: despawned after 60 frames (reason 0 timeout). Down B -> Counter
  (motion 0x401); Fox's jab (4%) at action frame 4: **COUNTER**, Wolf took **0.0%**, went to CounterHit
  (0x402) whose jab hitbox hit Fox (+4.0%), on_hit hook ran (`geno.log 77`). A jab on Wolf standing:
  on_hit, not countered, Wolf took 3.6%.

### 19.8 v5.1: what Sora's magic needed (additive)

| key / id | meaning |
|---|---|
| articles 8-15 | `GENO_MAX_ARTICLES` 16; articles 8-15 are kinds `0x1100 + p * 8 + (a - 8)` (the v5 numbers of 0-7 unchanged; `GENO_ART_KIND(p, a)`) |
| hitbox `"slot"` (`GENO_AH_SLOT` 16) | up to 8 entries on Melee's 4 item hitbox slots (default: entry index mod 4). An entry taking over an active slot re-sets it in place and keeps its victim list - Ultimate's ATTACK re-issued on one id (Firaga's 5.6 / 5.2 / 4.8 decay) |
| `"spawns": [[fwd, up], ...]` (`GENO_AP_SPAWN_N` 19, `_V` 20-27) | spawn variants picked by the spawn arg (Thundaga's three clouds) |
| hook 5's arg | `[7:0]` article, `[15:8]` spawn variant, `[31:16]` signed extra angle, degrees (`GENO_SPAWN_ARG`); a plain 0-7 is v5 |
| `"angle"` (38) | the initial velocity turned up by this many degrees (in the facing) |
| `"bone"` (17) | spawn from that fighter part's world position (`lb_8000B1CC`) + the offset; -1 the fighter's position |
| `"min_speed"` (16) | a negative `accel` brakes to this speed and holds it (Ultimate's brake / stable_speed) |
| `"effect"` (18) | a Melee effect id attached to the article at spawn (`efAsync_Spawn` kind 1 on its root joint, as Mario's fireball) |
| `"children": [{"article", "frame", "every", "count", "spawn"}]` (28-37) | the article spawns articles from its own position and facing, owned by its fighter (a cloud dropping a bolt). Up to 2 |
| specials `{"select": "la_i:N" \| "ra_i:N", "targets": [...]}` | the special picks its target by an int variable; out of range = the fighter's own special. `air_*` inherits it. Sora's cycle: each cast sets LA int 0 to the next spell |

Event 34 logs every spawn position and velocity, 35 a stage contact.

**Sora's magic** (workspace `ports/ir/tools/trail_magic_geno.py`: our acmd dump + `vl.prc` -> a
geno.json; stand-in states on a host fighter's unused ItemScope subactions until Sora's own clips
and model exist):

| spell | from the dump (1:1) | inferred |
|---|---|---|
| Firaga | fire: speed 1.65, life 40, spawn (13.6, 7.4); hitbox 5.6 % / 361 / kbg 24 / bkb 42 / size 3.8, fire, decaying to 5.2 at f10 and 4.8 at f20; the cast's frames through its FT_MOTION_RATE segments (fire at game frame 16, 43 frames) | the re-press window (specialn12 loop) not ported |
| Blizzaga | 8 shards at game frames 15-37, angles 4 / 16 / -8 / 24 / -2 / 12 / -14 / 0; speed 3.6, brake 0.18 to 0.2; 2.4 % / 70 / kbg 12 / bkb 92 -> 86 -> 74, the last shard 3.6 % / 80 / kbg 100 / fkb 82; the body hit 1.8 % / 42 / 64 / 52 | shard life 30 |
| Thundaga | three clouds at game frames 26 / 40 / 54, each dropping a bolt at its frame 3 (generate_frame), speed -3.8, life length / speed = 24, despawn on the stage; 5.2 % / 10 then 50, the third 64 / kbg 140; air versions 344 / 68 | cloud positions (15 / 26 / 37 forward, 32 up) and life 30 |

Melee rounds hitbox damage to whole percent (5.6 -> 6, 2.4 -> 2).

### 19.9 v5.1 verification

- Test `geno_v51` (slots, variants, angle, bone, effect, children by name, the brake 3.6 -> 2.7 in 5
  frames and stopping at 0.2, select by LA0 incl. out of range); suite 189/189 on ACE.
- In game (ACE, Wolf standing in for Sora, Fox 29-31 units in front, both scripted,
  `_build/agents/beta/sora_magic_pad.lua`): B three times = Firaga (0x400, 43 frames, Fox +6.0 %),
  Blizzaga (0x401, 54 frames, 8 shards at the dump's angles, Fox +10.0 %), Thundaga (0x402, 70 frames,
  three clouds at x -45 / -34 / -23 dropping bolts that reach the floor and despawn there, Fox +10.0 %);
  the fourth B in the air = FiragaAir (0x403): the cycle wrapped.
- A bug the run found: `item.c` includes `geno.h` for the kind range; it must be rebuilt when the
  range changes (it was not, and article 8 fell into the m-ex branch - a panic, fixed by the rebuild).

### 19.10 Article models for Sora's magic

Ultimate draws Sora's spells with particle effects, not models: trail_fire / _ice / _cloud / _thunder
have no model folders and their effect scripts are .eff emitters (our acmd dump); the only trail
weapon meshes are the taunt's `flower` and the Final Smash `box`. So the magic's models are ORIGINAL
procedural shapes (workspace `ports/ir/tools/trail_magic_models.py`, no game data in them), sized from
the dump (fireball radius = hitbox size 3.8, shard 2.0, bolt 12 tall over its hitbox), built by
fighterbuild into costume-style .dat files (GnTrailFire / Ice / Cloud / Bolt, symbol `<name>_joint`)
that the mod ships in `files/`. The registry now loads a file + symbol once per profile and shares it
between articles (Ice and IceLast, the four clouds, the four bolts). Event 36 logs what each spawned
item builds from its model (joints, display objects, scale).

Verified (ACE, the magic pad): 4 model loads (5-5.2 KB each), every spawned article item has 2 joints
and 1 display object at scale 1.00; at the canonical spacing Firaga 6 %, Blizzaga 10 %, Thundaga 10 %,
the cycle wraps (unchanged by the models). Whether they look right is for GD's eyes (no screenshots).

### 19.11 v5.2: effects on articles (Ultimate particles, route (c) pilot)

`"effects": [6000, ...]` (param ids 40-47, up to 8; the v5.1 `"effect"` still works): effects attached to
the article's root joint at spawn. An id 5000..8999 is the OWNER fighter's own m-ex effect (models 5000+,
particle generators 6000+ of its bank) and is queued the way an m-ex article's item script does it
(`efAsync_MexSpawn`, iteffect.c), then flushed at once; other ids go through `efAsync_Spawn` as before.
The logic table's `destroyed` callback now stops the generators that follow the article's joint
(`hsd_8039D5DC`, as grlib does for a stage part): an m-ex "FollowJointPos" generator keeps the item's JObj
and `efLib_DestroyAll` only removes effect models. Events 37 (effects attached) and 38 / 39 (at article
frames 5 / 10 / 20 / 30: generators on the article, live particles from them, per generator) are the
numeric check.

The pilot (workspace `ports/ir/tools/trail_vfx_melee.py`, `_research/ultimate-particles.md`): Firaga's
in-flight effect `P_TrailFireBullet` from GD's ef_trail.eff, decoded with EffectLibrary, converted to a
Melee particle bank (EfUsData.dat, 7 generators, 3 texture groups) given to the Sora slot through an MxDt
effect row. Measured in game (ACE, Sora slot vs Fox): 7 generators on the fireball, 16 / 31 / 46 / 50 live
particles at frames 5 / 10 / 20 / 30; per generator at frame 30 {2, 6, 6, 6, 7, 9, 14} against the decoded
steady states rate x life {2 (flare1), 5.8 (fire1), 6.5 (fireline1), 6.6 (fire2), 7.5 (fire_rif1), 9.75
(fire3), 15 (spark2)}.
### 19.12 v5.3: what Sora's physical specials needed (additive)

| id | meaning |
|---|---|
| hook **6 `geno.lockon`** | arg `[15:0]` range (units, 0 = any), `[23:16]` max angle (deg), `[31:24]` stick angle (deg). The nearest fighter of another port within range: turn to face it, aim at it with the angle above / below the horizontal clamped to the max; none in range: keep the facing, the stick past 0.25 aims +- the stick angle; never down on the ground. Writes MOVE_F0 / MOVE_F1 (unit forward / up), MOVE_I0 (1 = locked on); scripts scale it (GET / MUL / PUT FWD_VEL, VEL_Y). Event 40. Pure part: `GenoGame_LockonAim` |
| phys **`air_drift`** (callback id 14) | Melee's horizontal air drift without gravity (the script owns VEL_Y, e.g. a per-frame rise profile); ground friction on the ground |
| hook **7 `geno.aim_stick`** | arg `[7:0]` stick threshold x100, `[15:8]` max angle up, `[23:16]` max angle down (deg), bit 24 the stick may turn the fighter. Stick vector length >= threshold: face its horizontal side (bit 24, when it points behind), heading = `atan2(sy, abs(sx))` clamped (never down on the ground); MOVE_F0/F1 = heading, also saved in MOVE_F2/F3, MOVE_I2 = 1. Below: MOVE_F0/F1 = the saved heading if MOVE_I2 is 1, else unchanged. Event 42. Pure part `GenoGame_AimStick`. Sora's Sonic Blade runs it after `geno.lockon` when nothing is locked on (MOVE_I0 = 0) |
| values **0x3B ATTACK_CONNECTED** (i, W) / **0x3C ATTACK_CONNECTED_PREV** (i) | 1 when a hitbox of this fighter won a hit on a fighter (set at `ftColl_8007A06C`'s Geno site, where Geno_Autolink runs: not shield hits, not articles) during the current action. **Reset point:** `Geno_OnActionChange` (inside `Fighter_ChangeMotionState`, right after `motion_id` is set - every action change, Geno states included): `_PREV` = the old value, then ATTACK_CONNECTED = 0. So a state reads its predecessor's result through `_PREV` at any frame; to carry it further (Sonic Blade: dash -> hover -> dash) copy it into an LA var. PUT writes ATTACK_CONNECTED (a script can re-arm it). Only fighters with a Geno profile or script track it. `GENO_VERSION` 5 (the GenoState layout grew) |
| sub **0x3A HBDMG** (len 2) | word0 `[15:8]` hitbox mask (Melee ids 0-3), `[7]` B is a var; word1 = damage (float immediate or var ref). Sets the active hitboxes' damage after their creation (after staling) - Ultimate counters scale their hit: Sora's Counter Attack is `GET RAF5 HIT_DAMAGE; MUL 1.5; clamp 9..30; HBDMG mask RAF5` after each hitbox command |
| sub **0x3B HBSTUN** (len 2, v5.5) | word0 `[15:8]` hitbox mask (Melee ids 0-3), `[7]` B is a var; word1 = extra hitstun frames (int immediate 0-255, or an int var ref). A hit by a masked hitbox id adds word1 frames to the victim's hitstun (the damage state's frame count, `mv.co.damage.x0`, after Melee's knockback x 0.4) - Ultimate's `AttackModule::set_add_reaction_frame_revised`. Kept for the rest of the action (cleared at the action change, like REHIT / LINK), so it may come before or after the hitbox command. Two bonused hits on one victim in one frame: the larger. Word0 = `0xEFB2mm00` (mm = mask; `| 0x80` for a var). Rollback: attacker per-hitbox values and the victim's pending bonus live in Geno_StateBlock (a game global). Checked (ACE): Sora jab (overlay on subaction 46: `0xEFB20F00, 20, 0xED310000`) on Fox: hitstun 11 -> 31 frames; `MELEE_GENO_STUNLOG=1` logs base + bonus |
| sub **0x3C HBFLAGS** (len 2, v5.5) | word0 `[15:8]` hitbox mask (Melee ids 0-3), `[7]` B is a var; word1 = contact flags (int): 1 NO_HITLAG (no hitlag for the victim or the attacker), 2 FLINCHLESS (the damage lands; no knockback, no damage state), 4 ZERO_DAMAGE (the hit adds no damage: a detector), 8 FORCE_REACTION (the victim reacts even with `no_kb` set; wins over FLINCHLESS). Ultimate's ATTACK `disable_hitlag` / `flinchless` / zero damage and `AttackModule::set_force_reaction`. Kept for the action (cleared at the action change); fighter hitboxes on fighters. The hit still registers (on_hit, ATTACK_CONNECTED, the victim list). Word0 = `0xEFC2mm00`. Checked (ACE): Sora jab with flags 7 on Fox: 3 hits registered, 0 % and no damage state (without: 11 hitstun frames and 8.2 % after 3) |
| value **0x1B ANIM_RATE writable** | `ftAnim_8006F0FC(gobj, rate)`: clip rate and `frame_speed_mul`; ftAction's timer steps by `frame_speed_mul`, so script waits stay in clip frames while game time stretches (Ultimate FT_MOTION_RATE r = PUT ANIM_RATE 1/r) |
| hook **8 `geno.dash.search`** (v5.4) | arg `[7:0]` stick threshold x100, `[23:8]` range (units, 0 = none). One frame of an aim window between two dashes. No target yet: the nearest fighter of another port within range locks (MOVE_I0 = 1, MOVE_I1 = its port). Still none: a stick sample whose vector length reaches the threshold is stored as a world unit vector (MOVE_F2/F3, MOVE_I2 = 1), the raw polar angle, the full circle, no clamp and no ground rule. A script clears MOVE_I0 / MOVE_I2 where the window opens. Event 43 |
| hook **9 `geno.dash.aim`** (v5.4) | arg `[7:0]` degrees from vertical inside which the facing is kept, `[15:8]` / `[23:16]` the "up" range in degrees. MOVE_F0/F1 = the heading as a **world** unit vector: at the locked target's position now, else the stored stick sample, else straight ahead. MOVE_I3 = 1 inside the up range, 2 inside the same range 180 degrees round, else 0. The fighter turns to the heading's side unless it is within the first argument of straight up or down. A target that is gone unlocks. Scripts multiply into a speed and PUT VEL_X / VEL_Y. Event 44 |
| hook **10 `geno.brake`** (v5.4) | arg `[15:0]` brake x1000, `[30:16]` speed cap x100 (0 = none), bit 31 horizontal only: cap the fighter's own speed, then take the brake off it along its own direction, never past rest. On the ground it acts on the ground speed. One call = one step |
| value **0x3D STICK_LEN** (f, v5.4) | the stick vector's length (the dead-zone test Ultimate's status code makes: `IFV STICK_LEN GE 0.25`) |
| value **0x3E FALL_LIMIT** (f, W, v5.4) | this action's fall-speed limit: while it is above the fighter's terminal velocity, `ftCommon_Fall` clamps the vertical speed to it instead (every aerial phys that falls goes through there). Cleared at each action change; 0 = the fighter's own. For a script-set dive faster than terminal velocity (Ultimate's `SET_SPEED_EX` on a down air); a fighter whose scripts never PUT it is unchanged |
| key **`motion_anims`** (v5.4) | `[{"motion": id, "subaction": row}]`, up to 8 per fighter: a common motion state plays this fighter's own row (clip and script) instead of the one Melee's state table names. Made for the grab pull-in: Melee's CatchPull (213) and CatchDashPull (215) carry the Catch clip on at its current frame, so a fighter with a pull clip of its own (every Ultimate fighter) looked like a whiff on a successful grab. With an entry for those states the grab-connect code (`fn_800D9CE8`) starts the row at frame 0; `Fighter_ChangeMotionState` applies the map to any state. Give the row an empty overlay script if the host row has one. No entry = unchanged |
| phys **`brake`** (callback id 15, v5.4) | once a game frame, whatever the clip's rate: MOVE_F5 comes off the speed along its own direction (the ground speed on the ground; MOVE_I4 bit 0 = the horizontal only in the air); in the air MOVE_F6 is the gravity per frame and MOVE_F7 the vertical speed's clamp either way (0 = none). The script PUTs them where the state begins. For states whose kinetics are a status's own numbers, not the fighter's attributes |
| coll **`both`**, landing (v5.4) | a state on a row whose clip drives the fighter (`x594_b0`) keeps the horizontal speed it had on landing; before, `ftCommon_8007D6A4` replaced it with the clip's root motion, so a scripted dash into the floor stopped dead |

Sora's side special is the status code's own state machine (workspace
`_research/ultimate-sonic-blade-spec-2026-10-03.md`): dash 1 straight and never aimed, then SEARCH
(hook 8 each frame, phys `brake`), TURN (level / up / down clip), and a dash along hook 9's heading;
the chain continues on stick deflection at a dash's last frame or the special button latched inside
its window. Test `geno_v54_dash` (the three pure cores, hook names). Hooks 6 and 7 are unchanged and
no longer used by that move.

Sora's side / up / down specials (workspace `ports/ir/tools/trail_specials_geno.py`, with `--magic` the
combine step that puts trail_magic_geno.py's neutral special first) are 13 Geno states; long overlays go
in word files (`"file"`), the registry's JSON reader has a node cap. Test `geno_v52_lockon` (the aim,
hook 6 by name, PUT ANIM_RATE, HBDMG). Counter scaling in game (ACE): Fox jab 3.64 % -> 9.0 (min), Ganondorf
jab 7.28 -> 10.9, fsmash 16.71 -> 25.1 (x1.5), fresh fsmash 22.0 -> 30.0 (max); Sora took 0 each time. In-game numbers: the workspace lane notes (`_build/agents/echo/NOTES.md`).

### 19.13 Standalone native items (source update 2026-10-03)

Fix1: material roots and canonical mesh paths share Windows drive/UNC namespace
normalization. Visual colour/part assets preload one job per engine tick, outside
Lua callbacks. Spawn refuses while its colour is preparing; optional-art errors
finish with the fallback, and spawn itself performs no loading. Failed/budget-
aborted creating callbacks remove their newly created items without awards.
The drive defaults to bob 0.6 and logs when its four-visible/four-hidden blink
phase starts. These corrections need native acceptance after integration.

Item hooks default to standalone Geno items. Each script explicitly enables
common vanilla/m-ex events using `gd.item_events{vanilla=true,mex=true}`. Vanilla
projectiles/stage enemies and fighter-owned Geno articles stay excluded. Payload
belongs only to standalone Geno rows/events; vanilla/m-ex payload options refuse.


Standalone definitions are independent of fighter profiles. Their native kinds are
`GENO_ITEM_KIND_BASE = GENO_ART_KIND_END = 4608` through `4671` (64 immutable
registry slots); existing article numbers remain unchanged. Vanilla/Pokemon/stage
kinds are below the native range. On mod discs, registration checks the active
m-ex item table for overlap and refuses conflicting definitions. These items do
not require `MxDt.dat`, PowerPC item code, a fighter owner, or a DAT model writer.

A mounting mod supplies `items/<name>/item.json`, with `version: 1` and a lowercase
name using letters, digits, `_` or `-`. Definitions register at mod boot; repeated
identical registration from the same origin returns the same definition, while
changed definitions or duplicate names from another origin are refused until
restart. Unknown/duplicate fields and invalid numeric ranges are refused. Item files use
an intentionally strict lexical subset: JSON objects, decimal numbers (including
fraction/exponent forms), booleans, and unescaped printable ASCII strings. Arrays,
null, comments, trailing commas, backslashes/escapes, non-ASCII and raw control
characters are refused; this does not change the older fighter-profile parser.

```json
{
  "version": 1,
  "name": "drive",
  "physics": {
    "gravity": 0.12, "terminal_velocity": 4,
    "bounce": 0, "friction": 0.15,
    "floor_rest": true, "stage_collision": true
  },
  "collection": "touch", "radius": 7, "ports": 1, "teams": 15,
  "lifetime": 300, "blink": 60,
  "count_against_cap": false, "held": false, "thrown": false,
  "payload": {"colour": "red", "amount": 20},
  "visual": {
    "asset_mod": "envoy_drives_sa2",
    "model": "models/drive_red", "glass": "models/drive_glass",
    "scale": 2.5, "spin": 6, "tilt": 12, "height": 4.55,
    "hover": 6, "bob": 0.6, "bob_speed": 3
  }
}
```

`collection` accepts `touch`, `press` or `none`; `grab`, held and thrown standalone items
are explicitly refused. `press` is collected by pressing A near it through the retail
item-pickup states: `ftpickupitem_80094790` (the attack entry states' item check, and the L/R+A catch
`ftCo_800D705C`) asks `ftpickupitem_800942A0` for an item in the fighter's pickup box, which also asks
`Geno_ItemPressDist` (`pc/geno/geno_game_items.inc`); the LightGet motion plays and its grab frame
(`ftpickupitem_Anim`) collects the item (the same event as a touch, then destroys it) instead of
attaching it to the hand. The `ports`/`teams` masks decide who may; a CPU-controlled fighter never may
(`ftCo_IsCpuControlled`), and the CPU item seeker never targets these items (`Item_IsGrabbable` is false
for them). Walking over a `press` item collects nothing. Grounded only, as retail's A pickup is. Source
and syntax only; not run in the game. Default hitboxes/hurtboxes are absent. `ports` is a six-bit
fighter-slot mask (bit 0 is slot 1); `teams` is a four-bit team mask. Lifetime and
blink count logic frames, and blink cannot exceed lifetime. Payload currently has
only `colour` (`red`, `green`, `blue`, `yellow`, `white`) and integer `amount`
(0..1000000), rather than arbitrary script values.

`count_against_cap` selects the `Item_802674AC` hold-kind policy: true uses
common-item class `ITEM_HOLD_0`, false uses character-item class `ITEM_HOLD_8`.
The default is false so mission drops do not consume the common random-item cap;
a hard limit of 128 live native standalone items also applies, alongside the
game's total item pool. This classification is a cap
policy, not permission to hold the item. No vanilla hold/throw procedures are
reused for standalone definitions.

Physics uses swept point collision against floor, wall and ceiling segments,
including script/mission collision and pass-through floors. Resting pickups track
the floor line and use mpLib speed for moving-floor carry. Bounce affects reflected
velocity and floor friction removes the specified fraction of horizontal velocity
(default 0.15). This is point collision, not a model-sized ECB. These source paths
have not been run in the game.

Visual paths are contained relative stems without extensions; the loader resolves
GXMS assets and their material sidecars, with an optional glass part and optional
asset mod. The fallback is an engine primitive, so optional converted art is not
required. `hover` (0..100, default 0) lifts the visual above the physics anchor;
`height` (0..1000, default 0) is unscaled model height and also sets fallback
primitive height when positive. `tilt` (-90..90 degrees, default 0) inclines the
long axis before Y spin. Spin and bob speed are degrees per logic frame. The
example turns at 360 degrees/second and bobs with a two-second period at 60 Hz;
its model height is 11.375 units before tilt. Visual Y is anchor Y plus hover
and the sinusoidal bob. Collision stays on the floor, following its moving line
and falling off edges. Collection uses a capsule from the anchor to the visual
top, with radius multiplied by visual scale; standing underneath and jumping
through it both collect. These fields reconstruct from snapshot-owned age and
position, without a second host-side trajectory. Their fixtures are source and
syntax checked; native execution and appearance remain pending.
Script model instances also accept X/Y/Z rotations, with inverse-transpose normals
and matching alpha sorting. Collision-bearing script model instances refuse X/Y
rotation because mpLib sidecars describe two-dimensional segments.

The per-item runtime belongs to snapshotted game memory. Immutable definitions
and visual assets remain host-side. Source fixtures cover parsing, lifecycle and
visual contracts; no game build or launch was performed for this source update.
Native acceptance still requires floor/pass-through-platform rest, collection,
expiry, thirty live items, and LAB save/load/rewind with items in flight. Full
netplay still needs item-definition fingerprints and deterministic replicated
spawn/despawn inputs; Lua gameplay writes remain offline-only.

## 20. Geno effects (`.gfx.json`, format v1): the effect IR

Status: **importer, simulation, game bindings and renderer built**. `pc/platform/gw_fx.c` loads,
simulates and supplies the census; `pc/geno/geno_game_articles.inc` drives article/fighter effects;
`pc/platform/gw_fx_render.cpp` renders them through Aurora. Article `fx` and fighter `fx_bindings`
are implemented (20.3). Code presence does not establish source-game visual parity. Design: the workspace's
`_research/geno-effects-runtime.md`. GD's decisions (2026-09-26): an effect from another game is rendered by
Geno's own runtime, not squeezed into Melee's particle bytecode (sections 19.8-19.11 remain the path for effects
Melee's particles express well); the goal is higher fidelity close to the source, **not a 1:1 shader port**.

**The pipeline**: source game -> importer -> **effect IR** (`.gfx.json`) -> Geno effect runtime -> Aurora, the same
shape as the character IR (schema: the workspace's `ports/ir/schema/effects.schema.json`). The IR is
game-neutral. Emission, motion, colour / alpha curves, textures and blend modes are carried faithfully; every
material is **a shader type from the runtime's library + parameters** (20.2). There is no IR of source shader
programs: importers read them only to choose the type. Importers: `ports/ir/tools/ultimate_vfx_geno.py`
(Ultimate eft2 / VFXB); Halberd's Brawl effects (REFF) could get one later.

A **Geno effect package** is a directory in a mod (`mods/<id>/fx/<name>/`): `<name>.gfx.json`, `tex/*.png`,
`mesh/*.json`. Nothing in it is interpreted by Melee's HSD particle system.

```json
{ "geno_fx": 1, "name": "P_TrailFireBullet",
  "source": { "format": "eft2/VFXB (Nintendo), via EffectLibrary", "set": "P_TrailFireBullet" },
  "textures": [ { "name": "ef_cmn_fire00", "file": "tex/ef_cmn_fire00.png", "w": 512, "h": 512,
                  "source_format": "0x1e01", "swizzle": "rrrg" } ],
  "meshes":   [ { "name": "P_TrailFireBullet_sphere1", "file": "mesh/P_TrailFireBullet_sphere1.json",
                  "verts": 327, "tris": 520 } ],
  "emitters": [ { "name": "fire1", "order": 4, "kind": "particle" | "mesh", "mesh": null | "<mesh name>",
                  "follow": "srt" | "none" | "translate", "transform": {...}, "emission": {...}, "shape": {...},
                  "particle": {...}, "color": {...}, "samplers": [...], "material": {...}, "wave": {...},
                  "inherit": {...}, "program": {...}, "extensions": {...} } ] }
```

| block | fields (units: game units, frames, radians; curves are up to 8 keys `[x, y, z, t]`, t = life ratio 0..1) |
|---|---|
| `transform` | emitter offset `translate` (+ `_random`), `rotate` (+ `_random`), `scale`, in the spawning object's frame |
| `emission` | `start` (frame delay), `duration`, `one_time`, `rate` (+ `rate_random`), `interval` (+ `_random`), `position_random`, `by_distance` {`unit`, `min`, `max`, `margin`, `max_particles`} (emit per distance travelled), `fade` {`on_stop`, `alpha_frames`, `fade_in_frames`, ...} |
| `shape` | `type` (point, circle, circle_fill, sphere, sphere_fill, cylinder, box, line, rectangle, primitive, ...), `radius[3]`, `form_scale[3]`, `caliber` (hollow ratio), `sweep` [start, longitude, latitude], `surface_random`, `line`, `divide` |
| `particle` | `life` (+ `life_random_pct`), `infinite`, `shape` (billboard, y_billboard, plate_xy, plate_xz, directional_y, directional_polygon, stripe, complex_stripe, primitive); `velocity` {`all_direction` (radial speed), `direction[3]` x `direction_scale`, `diffusion...`, `random_pct`, `inherit` (share of the emitter's own velocity), `momentum_random`}; `forces` {`gravity_dir[3]`, `gravity`, `gravity_world`, `air_resistance` (velocity x this a frame)}; `rotation` {`axes`, `init` + `init_random`, `add` + `add_random` (per frame), `regist`}; `scale` {`base[3]`, `random_pct`, `keys`, `loop`, `add_velocity`}; `param_keys` |
| `color` | `scale` (HDR multiplier), `emitter` {color0, color1}, `color0` / `alpha0` / `color1` / `alpha1`: {`kind`: constant / random / keys, `value`, `keys`}, `loop` per curve |
| `samplers` | per texture slot: `texture`, `wrap` [u, v], `filter`, `uv_channel`, `pattern` {`mode` fit_life / clamp / loop / random, `count`, `frequency`, `table`}, `uv` {`scroll`, `scroll_add`, `scale`, `scale_add`, `rotate`, `rotate_add`, `*_random`, `divide` [cols, rows] (atlas)} |
| `material` | `shader` {`type` + parameters, 20.2}, `bloom` {`threshold`, `intensity`} (20.2), `blend` (alpha, add, sub, mul, screen), depth test / write / func, `alpha_test` {func, threshold}, `display_side`, `draw_path` (render pass), `sort`, `soft_particle` {distance, volume}, `fresnel_alpha` [lo, hi], `near_alpha` / `far_alpha` [lo, hi], `decal` |
| `wave` | per-particle fluctuation: `type`, `amplitude`, `cycle`, `phase_random`, `phase_init`, `apply` [alpha, scale, scale_y] |
| `inherit` | child-emitter inheritance flags and rates |
| `program` | **optional, reference only** (the runtime never reads it): the source shader `kind`, `shader_index`, `fragment_listing` (the disassembly beside the package); with the importer's `--keep-programs` also `fragment` (op list) and `vertex_outputs` (varying -> source field) |
| `extensions` | emitter sub-sections the format does not map yet (`EPxx` plugins, `FSPN` spin field, `FCLN` collision field, `CADP` / `CSDP` ...), as raw words and floats |

Textures keep their source channel layout; `swizzle` says which source channel feeds r, g, b, a (the GPU's
component selector: Ultimate's BC5 effect textures read as `rrrg`: colour = R, alpha = G). Mesh JSON:
`position`, `normal`, `uv0`, `color0`, `indices` (triangles).

**Not in v1** (named so a runtime does not guess): child emitters' links, stripes' history, GPU (compute)
emitters (`spark2`-style `ComputeShader.bnsh` is noted, not translated), the plugins and fields in `extensions`.

### 20.2 The effect shader library (`material.shader`, `material.bloom`)

| `type` | draws | parameters |
|---|---|---|
| `sprite` | texture(s) x the colour ramps | `color`: `modulate` (color0 x tex) / `lerp` (lerp(color1, color0, tex)); `alpha`: `texture` / `texture_product` |
| `warp` | sampler `offset` displaces sampler `base`'s UVs | + `strength[2]`, `base`, `offset` |
| `distortion` | displaces the frame copy (heat haze) | + `strength[2]` |

All types: `fresnel`, `alpha_test` (booleans; their values are in `material`), the samplers' UV animation, the
blend mode, soft particles. `bloom`: the effect's colour above `threshold` goes to an effect-only bloom buffer
(blurred, added over the frame after the effects; Melee's own world never enters it) at `intensity`. A
missing `shader` means `sprite` / `modulate` / `texture`.

### 20.3 The runtime API (native, `gw_fx.c`)

`gw_Fx_Find(name)` loads `mods/<id>/fx/<name>/<name>.gfx.json` (the last mounting mod wins) and returns a package
index; `gw_Fx_Attach(pkg, jobj, mtx_offset, frame)` starts every emitter on a guest JObj (its world matrix read each
frame, read-only); `gw_Fx_Detach(jobj)` stops emission (live particles finish); `gw_Fx_Frame(frame)` steps once per
logic frame with the game's own frame counter: a frame not after the last one restores the state kept for the
frame before (16 frames) and re-simulates (rollback, LAB rewind); nothing enters a snapshot. `gw_Fx_Stat` /
`gw_Fx_Census` are the numeric census (live particles, instances, spawned / killed / refused, per instance).
Budget: 64 packages, 2000 particles, 256 emitter instances (a full pool refuses the newest package's lowest-priority emitters; `gw_Fx_Stat(5)` counts them). An idle state is kept in the rollback ring as its header only.

**Binding (geno.json v5.4)**: an article's `"fx": "<package>"` (GENO_AP_FX) attaches that package to the article's
root joint at spawn and detaches it when the article goes (every despawn path). The game half calls
`Geno_FxFramePost(frame)` once per scene frame after the scripts (`gmscene.c`), with the scene's frame counter;
`gw_Fx_Frame` logs a census line every 30 frames while anything is live (`fx: census frame N: ...`). Checked in
a real match (Sora magic, ACE): Firaga attached 10 emitters, census 8 live at +7 frames; 8 Blizzaga shots, 35
instances / 18 live at the peak, none refused, all dead after.

**Fighter bindings (geno.json v5.5)**: a fighter entry's `"fx_bindings": "fx/fx_bindings.json"` (a path in the mod;
format: the workspace's `ports/ir/schema/fx_bindings.schema.json`, written by `ports/ir/tools/trail_fx_bindings.py`)
attaches packages on the fighter's states. `Geno_FxFramePost` calls `gw_Fx_Drive` for each fighter whose profile has
a set, before `gw_Fx_Frame`, with its motion state, subaction (`anim_id`), animation frame, Geno's `action_time`,
situation, facing and parts table. A state is matched by `subaction`; a call fires once the state's clock (`animation`
= the animation frame, `game` = frames since the state began) reaches `frame`, on `joint` (a depth-first index in the model's tree; the runtime walks that tree rather than trusting a remapped parts slot) x T(`offset`) R(`rotation`, degrees, X then Y then Z)
S(`scale`), the package's axes taken as the joint's own. `follow: false` takes the matrix once (world-fixed). Ends:
`off` / `detach` at `end_frame`; `state_exit` (and `owner_destroy`) when the motion state or subaction changes or
the state is entered again; `emitter_life` instances end themselves when their emission is over (one whose emitter
never ends goes at state exit). `when`: a call is taken when every condition it lists holds (the other branch is
skipped); `situation` limits a call to ground / air. What fired in the current state lives in the effects state
(`fx_drv`, 8 fighters), so a rollback or LAB rewind restores it with the instances; a rewind past the 16-frame ring
starts clean. Checked in the LAB (Sora slot, ACE): jab KeybladeFlare on joint 78 at anim frame 8, off at 16; forward
smash KeybladeFlare + KeybladeLight at 14, off at 25 / 23; Sonic Blade SonicStart world-fixed on `top` + 11.5 up at 0,
SDash KeybladeFlare 1, SonicAttack on joint 2 at 2, SonicImpact world-fixed at 9; Counter CounterFlash at 8 (ground
and air); a LAB rewind restores and re-fires.

Importer check (GD's ef_trail.eff; shader types: Fire 7 warp / 2 sprite / 1 distortion, Ice 10 sprite / 1 warp,
Thunder sprite): P_TrailFireBullet 10 emitters, 9 textures, 4 meshes, 10 / 10 fragment programs read; P_TrailIceBullet 11 / 10 textures / 5 meshes / 10 of 11; P_TrailThunderCloud 5 / 4 / 4 /
4 of 5; P_TrailThunderBullet 4 / 3 / 4 / 4 of 4.


## 22. v6: native definitions, GF1 (integration pending)

The Geno engine accepts `define` beside the existing `attach` format. Slice 1 supports exactly:

```json
{"geno":6,"fighters":[{"define":{"key":"vanilla-hero","name":"Vanilla Hero","base":"mario","common":"melee.common.v1","resources":"retail:mario"},"attributes":{"walk_max_vel":1.8},"subactions":[{"index":46,"file":"moves/hero-jab.words"}]}]}
```

The authored Hero jab uses a 12-damage root-joint hitbox. All remaining scripts/actions inherit the Mario/common preset. Native callback and archive-name tables are inherited explicitly, then effective attributes, animation descriptors, common and special state rows are copied independently. Pristine Mario script descriptors are captured before Geno attachments can change them. Immutable model/clip/effect/audio descriptors remain installed-disc references. Other mods or an m-ex disc may replace the referenced Mario resources; GF1 does not certify them as vanilla-byte-identical. No disc bytes are distributed in Hero.

`common_states` overrides motion rows 0..350 (341 common rows and ten Mario-specific rows), with optional `like`, `subaction`, `flags`, `move_id`, and registered `anim`/`iasa`/`phys`/`coll` callbacks. Omitted values inherit. Explicit installed subaction indices must be 0..302 for the audited Mario preset. Existing Geno states/scripts/hooks remain available; new resources/articles/Lua are later slices. Attaching and defining in one entry is an error.

The native catalogue reuses the P5 roster catalogue's ordering/identity primitives independently of MxDt discovery. Definitions receive unused resident CK aliases 127 downward to 34 at boot. Occupied m-ex slots are preserved; at most 94 aliases exist, fewer with m-ex installed. Duplicate identities, allocation/budget failures and exhaustion refuse admission with logs rather than truncate. All active definitions must fit; GF1 has no on-demand resident swapping. This does not activate P5's wide m-ex clone manifests.

Native profile storage grows beyond 32 with a 65,535 id bound and a 256 MiB metadata budget. Game descriptor pointer tables and lazy allocations use a 2 MiB, explicitly budgeted static game-global arena, with snapshotted bytes/cursor/pointers and scene reset. GF1 follow-up 1 removes their dependency on an initialized HSD heap, including in headless attachment tests; runtime suite acceptance awaits the integrator rebuild. Historical article kinds for profiles 0..31 are unchanged; further profiles use a disjoint range starting at 0x20000. Overlay slots/word-pool budgets remain bounded and invalid/partial native overlays refuse the definition.

Native identities and interpreter-attempt counters are game globals; host registry/aliases are process-immutable. A guard at the PPC interpreter entry recognizes a native fighter owner passed as GObj or Fighter pointer, increments its counter and logs/refuses the attempt. Normal native fallback dispatch does not increment it. Load/spawn logs show the count. These are diagnostics, not a claim that a complete stock match has been executed.

Definitions are offline only, absent from online CSS, and refused by named/CK scene selection and the loader guard. Changing definitions requires restart; adding the first definition through hot reload is also refused. Online remains blocked pending resource identity/preset certification, negotiated admission and mapping, replay metadata, compatibility/hash checks and real deterministic resimulation validation. The source SHA-256 export manifest is packaging metadata, not an online certificate. Existing attachments retain their previous format and id behavior.

Author commands are in workspace `tools/geno/README.md`; packet 11 in `docs/learn/geno-fighters/` is an untested teaching stub. The catalogue companion requires the separate Hero folder and is excluded from automatic tours until accepted. Registered tests `geno_define_registry`, `geno_define_repeated_install` and `geno_define_snapshot` require the integrator rebuild. Full action coverage, mixed stock match, save/restore/repeated-input hashes, frontend/audio/results and second-match memory lifetime remain runtime acceptance gates.

### 22.1 v7: a define with its own move set (slice 2, built 2026-10-05)

`"geno": 7` (and `GENO_VERSION` 7) marks a define file that uses the widened attribute list, `special_attributes` or `fx_bindings`; a v6
file, the Hero included, keeps its v6 meaning and id (the id hashes the entry, not the version). A v7 state library is not read by an older
build (the header check compares `geno_version`). What a define can now do, with the fixture that proves it,
`pc/geno/mods/vanilla-striker/` ("Vanilla Striker", key `vanilla-striker`, Mario's model and clips, own behaviour):

| area | how | notes |
|---|---|---|
| attributes | all 67 common attributes (`GENO_MAX_ATTRS` 128), by their `geno_attrs[]` names (`landingairn_lag`, `initial_shield_size`, `jab_2_input_window`, ...) | a define is strict: an unknown name is a load error naming it; an attach stays lenient. Not in the table: dodge, roll and ledge scalars that live elsewhere than `ftCo_DatAttrs` |
| special attributes, fx bindings | `special_attributes`, `fx_bindings` (Geno effect packages, `fx/*.gfx.json` of the mod, not retail effects) | `articles` stayed refused until slice 3 |
| grounded and air attacks, grab, throws | overlays on subactions 0..302 plus `common_states` rows | 26 rows are the Striker's own (`python -m tools.geno.report`) |
| specials | `specials` bound to Geno states (`geno.ground`, `geno.air`, `geno.anim_motion`) for all eight entries | the Striker binds all eight to six states; `report` shows `DONOR` for an unbound entry (it runs Mario's code) |
| move tags | `move_tag` on states, overlays and `common_states`; **a define's declared tag wins over the tag derived from the motion id** (D6). Retail, m-ex and attach fighters keep the old order | verified in the game: hit rules and crits by tag class give the same multipliers for the Striker and Mario |
| rollback | `RB_GameHash` folds a digest of the fighter's `GenoState` (variables, change checks, move variables, action time, hold and hit words) for defined kinds only (`ftRb_GenoDefineWord`, `GenoDefine_StateDigest`); 0 for every other kind, so no retail hash changes | diagnostics counters are excluded |

Budgets, logged once per define at boot as `geno: define budgets: kind N overlays a/64, pool w words (all profiles t/16384), states s/48, arena b/2097152`: the Striker uses 32/64 overlays, 728 words (4.4%), 6/48 states; the pool was not raised.

**Known default arms.** Retail code that switches on `fp->kind` treats a define as an unknown fighter and takes the default arm: `ftCo_800C70D0.c:28`, `ftCo_800C7178.c:28`, `ftcpuattack.c:1106,1446` (CPU recovery and attack choice), `ftCo_0A01.c:4487`, `ftCo_Landing.c:49` (Mario's tornado/cape flags are not reset on landing), `ftkirby.c:3371,3932` (a define is not a copy-ability source), `player.c:52`. Nothing in the slice-2 sweep misbehaved on these (the table is in `docs/learn/geno-fighters/10-known-gaps.md`); AI hints are slice 6.

**Frame-timing readouts for a define** (measured): hitboxes at joint 0 map the offset x to world Z (sideways) and y to forward; a persistent savestate load does not restore a Lua `gd.input` hold, so a charge resumes only while the pad is held again.

### 22.2 v8: a define with its own articles, effect and named sounds (slice 3, built 2026-10-05)

`"geno": 8` (and `GENO_VERSION` 8) lets a define carry `articles` (the v5 machinery, section 19) and a `sounds` table. A v7 file with either key is refused
with `define articles and sounds need "geno": 8`; v6/v7 files, the Hero and the Striker, keep their ids. Fixture: `pc/geno/mods/vanilla-caster/`
("Vanilla Caster": its neutral special, ground and air, is a Geno state whose script runs `CALL geno.article.spawn 0`).

| what | how |
|---|---|
| article | the section 19 keys, unchanged: `lifetime`, `velocity`, `spawn`, `hitboxes`, `despawn`, `max_live`, `fx` (a Geno effect package of the mod's `fx/` folder), optional `model` (the Caster has none: the item is invisible and its look is the effect) |
| item kind | profile arithmetic (`GENO_ART_KIND(p, a)`): a define is a profile, so its articles take their own kind range (profiles 0..31 the historical ranges, 32 and up from `0x20000`); two defines never share a kind (test `geno_define_v8_parse`). The Mario fireball path (`ftmariospecialn.c:124`) only runs for Mario's own special entry, which a define binds to its own state |
| sounds (the resolver) | `"sounds": [{"name", "retail_sfx": <engine sound id 1..999999>, "volume": 0..127 (default 127)}]`, at most 16 per profile (`GENO_MAX_SOUNDS`). An article names them: `"spawn_sound"` (played when it spawns) and `"end_sound"` (when it goes: hit, timeout, stage, absorbed). Resolved at load into article params 64-67 (`GENO_AP_SPAWN_SFX` ...); played by the owner with `ft_PlaySFX(fp, id, volume, 64)`, logged `geno: article sound: spawn of article 0 plays sound 234 at volume 100` |
| strictness | for a define an unknown name, a duplicate name, an id or volume out of range, more than 16 sounds or a malformed table refuses the entry with the reason in the log; there is no donor fallback. An attach entry only logs and plays nothing |
| rollback | the item and its `GenoArtVars` live in the item heap (snapshotted); the descriptors in the snapshot-covered arena. `GenoDefine_StateDigest` (called by `ftRb_GenoDefineWord`, the only hash edit of this slice) now also folds the fighter's live articles (an order-independent sum of per-item words: profile, article, age, despawn flag, travel velocity, position, velocity, hitbox slots; folded only when one exists, so a define without articles keeps its slice-2 digest). `Geno_FxFramePost` keeps the "matrix dirty" bit of an effect owner's joint as the logic left it (that bit made `gd.rewind_test` report 1 simulation byte with an effect on an article) and only touches an owner that is still a live item root or fighter joint (a stale owner crashed the bench SyncTest at frame 864 once; cause class inferred, not proven) |

What the resolver supports and what a real character needs: it maps a NAME to an engine sound id the game's own code already plays (the numbers in `src/melee/ft/*.c`
such as 234 and 228); nothing reads or ships audio. Own audio (WAV converted offline to the mixer's representation, named events `(tick, owner, ordinal)` for rollback
suppression, a bank of the fighter's own, voice and announcer) is not built: it needs an audio container the engine can mount from a mod folder, which is open work for slice 6
(voice, announcer) and a follow-up of this slice. The Caster's two ids are placeholders for that; looks, sounds and feel are unreviewed.

### 22.3 Slice 4, first steps: the authored-fighter converter and a drawn stand-in (built 2026-10-05; the rest of slice 4 is NOT built)

**Done and run.** An original fighter (the Courier, `ports/vanilla-original/`: 2,580 triangles, 39 bones, 179 clips, 15 hurtboxes) goes
through `tools/geno/build_courier.sh` (one command) into engine files with no disc read at any point:

| stage | tool | output (under `_build/geno-slice4/courier/`, never committed) |
|---|---|---|
| validate | `python -m tools.geno.check_art ports/vanilla-original` | clear errors by file and field: unmapped motion row, hurtbox bone, inverted ECB, missing costume texture, glb vs manifest counts |
| plan + mesh | `ports/ir/tools/authored_fighter.py mesh` (reuses `plan_parts.py`) | `plan.json`: 42 joints (the 39 bones plus synthesized `TopN`, `XRotN`, `YRotN`, depth-first), the 54-slot Melee parts table, role -> joint, the `ftData` joint fields by role, hurtboxes and ECB in joint-local space; `mesh_<costume>.json` |
| model | `fighterbuild build <mesh> - <out> courier_joint courier_matanim --pc-palette 64` (template `-` = authored: no disc template; the material is built from scratch, `AuthoredMobj`) | `GnCourier_<default,red,blue,green>.dat`, 109,834 bytes each, ONE palette POBJ, `verify`: rest pose error 0, triangle and weight differences 0 |
| clips | `authored_fighter.py anim` (`figatree.py` encoder) | `GnCourierAJ.dat`: 179 figatree sub-archives (`PlyCourier_Share_ACTION_<Clip>_figatree`), 1.5 MB, largest clip 20 KB (buffer 128 KB), rotation as per-frame Euler (R = Rz Ry Rx, unwrapped), `trans`/`hips` translation, constant channels as one key, tree `frames` = key count - 1 (as `convert_ultimate_anim.py`); decoded back and compared to the glTF: max error 0.00012 rad |

Facts that cost time: glTF Y-up facing +Z is HSD model space, so no axis change and no flip (the winding drew correctly as `Cull_Outside`).
**A retail costume material is "the texture is the diffuse lightmap": TOBJ flags `0x00050010` (lightmap diffuse, colour operation REPLACE), material
ambient/diffuse 179.** The first authored material (MODULATE, no lightmap bit, ambient 255) drew the model as a flat white silhouette; changing only
those flags drew it textured. Two files in one process that share a name share one load (the article model cache is keyed by file name): variant
files need distinct names.

**Seen in the game** (vanilla disc, this build, LAB, a window capture of my own game, the repo fixture `pc/geno/mods/vanilla-courier/` mounted alone with geno-lab): its neutral special spawned the article; the model drew
textured (teal tunic, cream helmet, visor, crest, boots) in the rest A-pose at 1.0 scale through the palette path. (The white-material test drew identically through the plain and the palette path, so the material, not the path, was the fault.) Not run: any
clip on the skeleton, any fighter use of the files (the model is an article, not a costume), hurtboxes, performance.

**What is left (the engine side, none of it started).** Read from source, not run:
1. **Registry.** `gn_define_parse` (`pc/platform/geno_define_registry.inc`) accepts only `base: "mario"`, `common`, `resources: "retail:mario"`; `gw_Geno_DefineBaseKind` returns 0 for any define (line ~151) and `gw_Geno_DefineBaseCK` maps it to CK 8. `base: "none"` needs the parse, a per-define resource record (model files, bank file, `plan.json`) and the base lookup to answer "no donor".
2. **Loader.** `GenoDefine_Load` (`pc/geno/geno_define_data.inc`) copies Mario's `ftData`, `ftCo_DatAttrs`, `ftMario_DatAttrs`, the 303 `xC` rows and the specific state table. A none define builds `ftData` natively: attributes from the define alone (an authored default table is needed: zeroed gravity breaks the fighter), `xC` rows from `manifest.json` `motion_rows` (name `x0` = the clip symbol, flags low 6 bits = the define's kind, as `ftData_8008572C` does for m-ex), `x14` demo rows, and the `xC`/`x14` counts (`ftData_Table_Unk0[kind].count`), which are now copied from the base.
3. **Files.** `ftData_803C2360[kind]` (costume file names), `ftData_803C23E4[kind]` (animation file) and `ftData_803C1F40[kind]` (data file) are what `ftData_800855C8` loads; point them at `GnCourier_<costume>.dat` / `GnCourierAJ.dat` (mod `files/` resolve by name, as the article model already does).
4. **Skeleton roles.** `ftCommonData_ExtendKindTable` (`src/melee/ft/fighter.c:210`) builds `ftPartsTable[kind]` from PlCo.dat's table of the BASE kind; a define needs its own `FighterPartsTable` from `plan.json` `parts`. The `ftData` joint fields (`x8` part bytes, `x30` hurtbox bones, `x34`, `x38`, `x44` ECB bones, `x54`, `x58`) come from `plan.json` `ftdata`; the hurtbox capsules from `hurtboxes` (15, the engine's `hurt_capsules[15]`); the extra roles (grab/victim anchor, camera focus, absorb/reflect origin, scarf) are in `role_joint`. `LHaveN` has no bone (the Courier has one item socket, right): the converter reports it unresolved.
5. **Rollback.** New per-fighter gameplay state goes into `GenoDefine_StateDigest`; today the clips are not state (they are indexed by the motion id the hash already covers).
6. **Fixture, acceptance, performance.** The Striker's move set on the Courier's bank, the checks in the build brief (readback and captures), `bench.sh` against a retail fighter.

Not decided because not reached: how clip rate follows ground speed (retail's walk/run rate follows velocity; the Courier's reference speeds are 2 to 15 times below the Striker's gameplay speeds), the facing flip frames of `Turn`/`TurnRun`, the real ECB and hurtbox radii, and the model scale (1.0 was used; the Courier is about 11.7 units tall).

### 22.4 Slice 4, engine half so far: the Courier is a fighter (built 2026-10-05; see the open list below)

`base: "none"` (format 9) loads the define's own model, animation bank, parts table (42 joints), joint fields and 15 hurtboxes from its
`plan.json`. It stands in `Wait`, walks, jumps and attacks with its own clips, carrying the Striker's moves retargeted by role.

- **The pose bug (cause, measured).** Constant channels were written as ONE-key tracks. `FObjLoadData` (`fobj.c`) takes a key's interpolation
  op from the key before it (`op_intrp = op`, then the new op is parsed), so a one-key track reaches state 6 with `op_intrp == 0` and
  `FObjUpdateAnim` calls the joint update with an uninitialised value (0). Every constant channel zeroed its joint (the thigh's rest rotation
  of pi, the hips' height): legs up, hips at y 0. The converter now writes constants as two linear keys. Joint world positions agree with a
  forward-kinematics check of the same clips (five clips of five kinds, 42 joints, worst 0.14 units on an 11.7-unit fighter).
- **Rows.** `GenoDefine_BuildOwn`: the Wait state claims its row first (the idle picker, `ftCo_8008A7A8`, re-reads the row by number); any
  other state that shares a donor row but names a different clip gets a row of its own (rows `n..`, `ftData` `xC` and `x10` extended,
  `ftData_Table_Unk0[kind].count` raised, the per-kind motion-state copies remapped); a row no state names takes the clip its
  `ftCo_SM_` name gives (plan `row_clips`), else the Wait clip, because the engine plays some rows by number (`ftCo_Fall_Anim_Inner` blends toward
  `ftCo_SM_FallF`/`FallB`; an empty row was a NULL tree and a crash).
- **Hurtbox status words** (`ftColl_8007B128`): a borrowed script word naming a joint a none define has no hurtbox on is skipped (retail and
  every other define still assert). Moves retargeted by role (`tools/geno/courier_moves.py`) do not hit this.
- **Attributes.** The fixture authors 46 of the 67 named attributes; `model_scaling` is 1.0 and the walk / run reference speeds come from
  the art. The other 21 and the unnamed `ftCo_DatAttrs` fields are still the donor's.
- **Rollback.** No new gameplay state: rows and tables are static per kind. `gd.rewind_test` mid-jab: 0 simulation bytes differ; the bench
  SyncTest (k=12, Courier against Mario, pad bots, 700 frames): 7,200 checks, 0 mismatching.
- **Open.** ECB offsets, IK lengths, ledge snap and the hurtbox scale are the donor's numbers; the three donor fixtures are unchanged (same
  stable ids); Turn / TurnRun flip frames follow `standing_turn_frames` (4, as the art) and
  the dash-turn constant (donor's); costumes (four files exist), stage switching, six slots and a Classic run were not exercised.

### 22.5 Slice 4d: format 9 reference, the Courier's moves, the guard crash and the independence census (built 2026-10-05)

**Format 9 (`geno: 9`).** `define.base` is `"mario"` (retail Mario is the template, `resources: "retail:mario"`) or `"none"` (the package owns its
model, bank, parts table and joint fields, `resources: "mod:files"`). A `none` define carries a `fighter` block: `plan` (the converter's
`plan.json`: joints, parts table, role joints, `ftdata` joint fields, hurtboxes, motion rows), `animation` (the bank `.dat`) and `costumes`
(`file`, `joint`, `matanim` per costume). Its subaction rows are its own table (no 0..302 limit); `tools/geno check` and `report` accept it.

**The guard crash (fixed).** `ftCo_80091E78` (guard) blends the body from `ftData->x20->x0[2]`, a joint tree that starts at `FtPart_TransN`. A
`none` define's `ftData` was a copy of Mario's, so the tree was Mario's: walked over another skeleton it ran past the parts table and read a
NULL part (`ftAnim_80070010` / `80070108` -> `lb_8000C868`, read of NULL+0x38, the owner's crash entering Guard). Now a `none` define takes its
own costume model's joint tree below the root (`GenoDefine_RestPoseTree`), and after the blend the `ThrowN` joint's translation is cleared the way
`ftCo_800921DC` clears it, so the bubble is centred on its parent. Checked in the LAB: shield on, hold, drop, rolls, spot dodge, a real hit in
shield (`GuardSetOff`), shield held to break (`ShieldBreakFly`..`Furafura`).

**`ThrowN` is one joint with two jobs.** `ftData->x8->x11` is where a held victim attaches (`ftCo_CatchPull`: the victim's `XRotN` is moved onto it)
and the joint the shield bubble hangs from. The converter now maps the art's `grab_anchor` to it (it was `shield_origin`, which put the victim
inside the Courier). A split of the two (a separate shield origin) is not built.

**Courier hitboxes** are data in the fixture's `retarget.json` (`roles`, per-hitbox `joint`/`x`/`y`/`z`/`size`, and `delay`): sizes were tuned for
Mario's body (a 4.6 fist on an 11.7-unit fighter); the Courier's are 1.8-3.4 on the striking limb. `delay` shifts a move's first hitbox group
later by whole frames because the charge smashes' strike pose shows one frame after the clip's hit frame. All 32 scripts keep the Striker's
damage, angle, knockback growth and base (diffed), so knockback is the Striker's formula.

**Independence census.** `GenoDefine_Census` logs, per `none` kind, the `ftData` pointer fields that are still the donor's pointers; the pure
function `GenoDefine_FtDataSharedMask` is checked by `geno_none_independence`. Today: `x18` (demo blend table), `x1C` (per-row part lists, Mario's
part numbers), `x24`, `x2C` (dynamics), `x48` (item hat table) and `x5C` (the metal model's joint tree). Own storage (donor-valued defaults until
authored): `x20` (the guard tree table; the tree is the costume's), `x3C` camera box, `x40` pick-up offsets, `x50`, `x4C` sound-id struct. Common-state
code is the engine and stays shared by design.

**Donor requests a `none` define cannot honour (2026-10-07).** The define still runs the donor's action scripts and callbacks, which name the donor's body parts. Two requests used to fault on the Courier's skeleton and are now dropped, each logging once per kind: a hand-pose request (`ftAnim_ApplyPartAnim`; ftData x1C is Mario's table, so a define holds items with no grip pose) and an effect word on a part with no joint (`ftCo_8009F834`; the effect is skipped). Found by playing the Courier through Classic (`tools/geno/moves_check`); census fields x18, x24, x2C, x48, x5C are the same class and unproven.

### 22.6 Slice 6, first increments: costumes as colour sets, a define's own menu and HUD art (built 2026-10-07; the rest of slice 6 is NOT built)

Brief and ranking: workspace `docs/superpowers/plans/2026-10-07-geno-slice6-presentation.md`. Format: **still `"geno": 9`** (no bump: the keys below are
presentation, not simulation; an engine that does not know them ignores them; `presentation` is refused in a file below 9). The entry's content id
changes when the keys are present, and not otherwise.

```json
"fighter": { "costumes": [ {"file": "...", "joint": "...", "name": "Teal"}, {"file": "...", "joint": "...", "name": "Red", "team": "red"} ] },
"presentation": { "icon": "GnCourier_icon.gxtex", "portrait": ["GnCourier_csp_default.gxtex", "GnCourier_csp_red.gxtex"], "stock": "GnCourier_stock_default.gxtex" }
```

| key | meaning |
|---|---|
| `fighter.costumes[].name` | the label the Atlas select shows on the card (`Teal`) and on the stepper (`1 / 4 Teal`); printable ASCII, 1..23 characters. Without it the select says `Costume N` |
| `fighter.costumes[].team` | `red`, `blue` or `green`: the team battle colour this costume is (the first declared wins; a second is a `check` error). Undeclared: red is costume 0, blue 1, green 2, each wrapping to 0 when there are fewer costumes. `gm_GetNumCostumesForCKind`, `gm_80169264` (red), `gm_801692BC` (blue) and `gm_80169290` (green) answer a base `none` define's own list; a donor-based define keeps Mario's |
| `presentation.icon` | the select tile (64x56 is the retail size; any size up to 1024 is drawn fitted) |
| `presentation.portrait` | the portrait (retail 136x188), one file or one per costume; a costume past the list uses the first |
| `presentation.stock` | the HUD stock icon, one file or one per costume. **rgb5a3 or rgba8 only** (the swap carries no palette) |

Each name is a `.gxtex` of the mod's `files/` folder (`pc/tools/png2gx.py --allow-odd-size`; the container is the one the menus already use) and is read at run
time with the same lookup as `plan.json`, so name them with the fighter's prefix. A missing or unusable file logs once (`geno: <name>: presentation file X is not in the mod's files/`)
and the game falls back: the donor's art for a donor-based define, **letters and `DISC ART` for a base `none` define (never Mario's face)**, Mario's stock frame for the HUD.

What reads it: the Atlas select (`gmfrontend_atlas_select.inc`, shim `Ui_ArtGeno` -> `gw_Geno_DefineArtTex` -> `gw_Kit_TexAddGxtex`: the kit's disc-art pool); the HUD stock icon
(`ifstock.c`: `gm_GenoStockFrame` returns `20000 + ck * 32 + costume` and a wrapper that replaces `HSD_TObjReqAnimAll` in that file points the icon's own TObj image table at an
`HSD_ImageDesc` built from the package's bytes, `gw_Geno_DefineArtOpen` -> `gw_GxTex_OpenBlob`; cached per scene). The legacy kit select keeps the donor's tile for a donor-based define
and none for a `none` define; the retail CSS never lists a define. Logged: `geno: <name>: select icon|portrait|stock icon from its package: <file> as kit texture N`,
`geno: stock icon of ck N costume C: WxH format F from its package`, `frontend: character select - ck N icon|portrait from its package (kit texture T)`.

Also built here, each a small retail-path guard (all `TARGET_PC`, a define only): the **results name plate** is drawn from the define's name (`gmRst_DrawName`: it was blank, a define being no m-ex slot);
**records**: a define's matches write no retail row (`gm_CKindToSelKind` sends every m-ex or define CK to one retail selkind, Captain Falcon's, so a define's KOs and play time would have
been written into a retail fighter's record; `fn_80162068`, `fn_80162170`); **announcer**: a define is silent (`gm_80168C5C`; the donor's call named Mario). Not built (elevated, see the brief):
own announcer and voice audio, CPU AI hints, Kirby copy policy, a define's own records, the package emblem and the results stock icon. Tests: `geno_define_presentation`,
`geno_gxtex_art`, `geno_define_presentation_ck`.

### Frame-counting convention: the first `wait` of an authored script

A subaction script's `wait N` is the engine's own synchronous timer, run by retail's ftAction loop
for authored and retail scripts alike (a `define` overlay is installed as an ordinary script row; there
is no Geno-side first-event handling). Measured in the LAB with `gd.player(1).action_frame` and
`.hitboxes` sampled after `gd.wait(1)` following a one-frame A press: `wait 2` then a hitbox shows the
hitbox from action frame 0, `wait 5` from action frame 3 (the iasa shifts by the same amount), i.e.
**the hitbox is live from readout action frame N-2**. Retail Mario's jab shows the same offset: its
script places the hitbox at timeline frame 3 (`gd.timeline(1, 44)`) and it is live from action frame 1.
So this is a convention of the readout, not an off-by-N in `define`: read `gd.timeline` frames as
script frames, and subtract two for the sampled action frame.

### Declaring a move tag

A state (`states[].move_tag`), a subaction overlay (`subactions[].move_tag`) or a `define`
`common_states` row may declare `"move_tag"`: one of `jab dash_attack tilt smash aerial grab throw
special projectile`. Hit rules, `on_hit` info and echoes read it. An undeclared fighter-specific state
reports `special` when it was entered through a special input (any fighter, Geno or m-ex), keeps the
vanilla special-range answer for a `define` (Mario) or `attach` fighter, and is otherwise `unknown`.

## 23. Fighter Lua (slice 5, first increment built 2026-10-07)

A define can write a state's behaviour as Lua. This increment is deliberately small: two phases (`enter`, `frame`), a typed per-fighter state, four commands. Physics,
collision, landing and ledge rules stay the state's native callbacks. Brief and decisions: workspace `docs/superpowers/plans/2026-10-07-geno-slice5-fighter-lua.md`.
Fixture: `pc/geno/mods/vanilla-charger/` (neutral special, ground and air: hold B to charge up to 60 frames, release to strike; the dash speed and the hit's damage grow
with the charge). Demo and headless proof: `pc/scripts/examples/demos/geno-define-charger/`. Lesson stub: workspace `docs/learn/geno-fighters/13-fighter-lua.md`.

**This is not `gd` scripting.** A fighter's Lua runs inside the fighter's own state machine, in a separate `lua_State` with no `gd`, so the manifest flags `gameplay` and
`rollback_safe` (`docs/scripting.md`) do not apply to it. It is offline only because define fighters are (section 22). Whether this API shape and its limits are final is the
owner's call (listed as open in the brief).

### 23.1 The format (additive; the file says `"geno": 9`, no number was taken for it)

```json
"lua": { "script": "lua/charger.lua", "state": { "charge": "int", "charged": "bool" } },
"states": [ { "name": "Charge", "behavior": "geno.ground", "subaction": 295, "like": "motion:343",
              "lua": { "enter": "charge_enter", "frame": "charge_frame" } } ]
```

| key | meaning |
|---|---|
| `lua.script` | the module, a file of the mod (relative, no `..`, at most 64 KB); `lua.source` is the same text inline (tests). Exactly one of the two |
| `lua.state` | the typed state: slot name (1 to 23 letters, digits, underscores) to `int`, `float` or `bool`; at most 16 slots of 32 bits (`GENO_LUA_STATE_SLOTS`) |
| `states[].lua.enter` / `.frame` | module functions run once when the state is entered / once per logic frame. A state with a `frame` function gets the anim callback `lua` (unless it names its own `anim`), which also takes the state's `next` when the animation ends and no command changed the action |

Strict like the rest of a define: a `lua` key on an attach entry, in a file older than `"geno": 9`, a state that names a function the module lacks, a module the sandbox refuses
(23.3), more than 16 slots, a bad slot name or type, refuse the entry with the reason in the log (`geno: <mod>/geno.json: the Lua module was refused: ...`). The module text and the
slot layout are folded into the entry's content id; an entry without a `lua` block hashes exactly as before (the Hero, Striker and Caster ids were compared old exe against new:
unchanged). **An older engine ignores the key** and loads the Charger as a plain Mario define (measured: it logged id `08ed...` and ran without the special); that is why a format
number for the Lua keys is an open decision.

### 23.2 The API

```lua
local MAX = 60
local M = {}
function M.charge_enter(ctx) ctx.state.charge = 0 end
function M.charge_frame(ctx)
  local s = ctx.state
  if ctx.input.special_held and s.charge < MAX then s.charge = s.charge + 1 else ctx.go("Release") end
end
return M
```

A module is a chunk that returns a table of functions and nothing else. `ctx` is a fresh table every call.

| field | |
|---|---|
| `ctx.input` | `attack`, `special`, `jump`, `shield`, `grab` with `_held` and `_pressed` booleans; `stick_x`, `stick_y`, `stick_fwd` (the same buttons the script VM's HELD / PRESSED read) |
| `ctx.self` | `air`, `anim_ended` (booleans), `facing` (+1 / -1), `percent`, `x`, `y`, `vel_x`, `vel_y`, `action_frame`, `motion` |
| `ctx.state` | the declared slots, read and written by name; an undeclared name, a float into an `int` slot, an integer beyond 32 bits, a non-finite float, a number into a `bool` slot is a fault |
| `ctx.go(name)` | go to the Geno state of that name, or `"auto"` (Wait on the ground, Fall in the air) or `"helpless"` |
| `ctx.velocity(forward, up)` | facing-relative forward speed; on the ground the ground speed (`up` ignored), in the air both |
| `ctx.hitbox_damage(mask, damage)` | as the script word HBDMG: bit n of `mask` is hitbox slot n |
| `ctx.loop()` | restart the animation, the state's variables kept |

Commands are queued (at most 8 per call, `GENO_LUA_MAX_CMDS`) and applied in call order when the call returns, by the game half, before the next phase. The allowlist environment is
`type`, `select`, `ipairs`, `assert`, `error`, `freeze` and `math.{abs, min, max, floor, ceil, sqrt, tointeger, huge, pi}`; nothing else exists.

### 23.3 What is enforced, and where (`pc/platform/geno_lua_core.h`, tested by `build.sh --native-test geno-lua`)

- **No hidden state.** Module functions are scanned when the module loads: bytecode containing `OP_SETUPVAL` (assigns a captured variable) or `OP_SETTABUP` (assigns a global or a
  field of a captured table) is refused, in the function or any function nested in it; every captured value must be nil, boolean, number, string, a function that passes the same test,
  or a table made by `freeze{...}` (a deep read-only proxy; constants go there). The module table itself cannot be captured, so helpers are `local function`s. The environment is a
  frozen proxy, so a stray global write is an error at load or call time too.
- **No host dependence.** No `pairs`/`next` (iteration order follows a per-state string seed in 5.4), no `pcall` (a budget fault must not be catchable), no `string`, `table`,
  `os`, `io`, `debug`, `coroutine`, `load`, metatables, randomness or clock. Values are IEEE doubles narrowed to `s32`/`f32` at the state boundary. The collector is stopped during a call and a full
  collection runs after it, so the heap at the start of every call is the same; the test checks 50 calls leave it unchanged.
- **Budgets, deterministic.** 10,000 VM instructions per call (counted in steps of 50 by a count hook) and 64 KB of heap growth per call. The fault count of an endless loop was identical in three runs.
- **A fault** (error, either budget, bad command, bad state access, more than 8 commands) drops the call's state writes and commands, adds one to `Geno_LuaBlock.faults`
  (simulation state, in the hash), and sends the fighter to `auto` so it cannot sit in a state whose Lua no longer runs. The first eight faults of a profile are logged. The design
  asks for a hard match fault instead of this; that is an open decision (one line in `geno_lua_run`).

### 23.4 State, rollback and the hash

The typed state is `Geno_LuaBlock[6][2]` (`GenoLuaBlock` in `geno_state.h`: 16 words, the fault count, the last fault), a game global declared like `Geno_StateBlock`, so a savestate,
the LAB rewind and SyncTest cover it with no extra code. `GenoState` itself is untouched. Each call is transactional: the slots are copied into the call's io block, Lua works on the
copy, and only a call that returned 0 is copied back. `Geno_FighterReset` zeroes the block. `GenoDefine_StateDigest` folds the block (slots, fault count, last fault) **only for a
define whose profile declares a layout**; a define without a `lua` block keeps its digest, and a non-define's stays 0, so no existing fighter's hash changes. Both
`enter` (from `geno_enter_state`, after the behaviour's own entry; a chain of enters through `go` stops at depth 4 as a fault) and `frame` read only the fighter, the pad and the block.

### 23.5 Verified (this build, vanilla disc, headless; nothing seen on screen)

| check | result |
|---|---|
| native sandbox test `geno-lua` | all pass: 11 module refusals (global write, captured counter, nested capture, mutable table upvalue, module table upvalue, constant in the module table, no table, load-time global write, syntax error, load-time loop, bad freeze key), 19 run-time faults, allowlist arithmetic, stateless fresh ctx, heap steady |
| `run.sh --test` | 326 of 326 (321 before; new: `geno_lua_registry`, `geno_lua_call`, `geno_lua_charge`, `geno_lua_fault`, `geno_lua_digest`) |
| LAB proof (`scripts/proof.lua`, `mode=lab;p1=geno:vanilla-charger/hu;p2=mario/cpu0`) | charge 13 after 15 held frames; released at 25 with the charge kept; Release hitbox damage `9.750` = 6 + 0.15 x 25; `gd.savestate` at charge 10, `gd.loadstate` at 24 gave 11; `gd.rewind_test(120)` across charge, cap and release: `pass=true`, `diff_compared=0` ("0 simulation bytes differ"); `PROOF RESULT: PASS` |
| bench SyncTest (`cycle.lua`: B held 40 frames every 100, ~24 charge/release cycles; `MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1 MELEE_SYNCTEST=12`) | 27,600 curated checks, no Lua fault logged; 616 mismatching, **all in one word: the cmd-script frame counter of P2 (the idle Mario CPU), curated word 23, from frame 544**. The Charger's own record never mismatched. The same word mismatches for a retail Mario P1 driven by the same input (649 of 10,800), and an idle-Charger control had 0 of 8,400: this is the documented unexplained SyncTest class (`PORT_DEV_QUICKREF`, "sustained combat"), not shown to come from Lua, and not a certification |
| ids | Hero `47e0fa82...`, Striker `54334a53...`, Caster `ac4e4338...` identical in the old and the new exe |

`gd.rewind_test` restores a snapshot and compares bytes; it does not itself resimulate (its text says "0 frames re-simulated"), so resimulation is what the SyncTest line is for. Online remains
refused for defines (section 22); certifying a Lua fighter under real rollback is slice 7.

### 23.6 Not built (the road, in order)

`phys`/`coll`/`iasa` phases in Lua; `ctx.query` (opponents, stable order); `grab`, `throw_release`, `attribute`, article/FX/sound commands; per-article state; a LAB faults and budget
display (S5-7 of the road); `check` that understands Lua beyond function names and `ctx.state.<name>` spellings; a real match fault; Lua source and compiler version in an online handshake.

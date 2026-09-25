# Geno - GD's Melee's fighter-extension layer

Status: **v2** (v0 foundation, v1 Meta Knight script features (section 15), v2 action states, glide and MK specials (section 16)), on the public `pc-port` branch of the melee fork (made public 2026-09-24; later work: v3/v4, the LAB mode, see sections 17-18).

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
| v1 (built, section 15) | subaction script overlays; engine values GET/PUT/IFV; DIV, RAND; change action (Brawl requirements, persistent/once, CHGAND); REHIT; LINK (autolink 365); special-attribute overrides; `on_land` (script checks, geno.json map, hooks). Not done from the old v1 list: `air_vy` for non-multi-jump fighters, `on_hit` |
| v2 | Geno action states (section 11): glide, then crawl and wall cling; Meta Knight on top |
| v2.5 | HUD elements (section 11b): data-driven meters/icons bound to Geno variables |
| v3 | `define`: brand-new fighters with Geno-native registration (their own kind range and content ids, independent of m-ex's dense slots), CSS/SSS entries via gw_uigen |
| later | an IR emitter/loader for `melee.geno`; per-profile merge rules instead of "later mod wins" |

Constraints that stay: Melee's global rules (section 1), m-ex untouched (section 3), rollback-safe
(section 4), privacy of this branch until GD decides otherwise.

## 14. Geno Lab (inspection tool)

An in-engine, frame-steppable lab for seeing *why* a fighter feels the way it does (first use:
Brawl Kirby vs vanilla Kirby). The native part is thin: scalar read-only getters, the game's own
develop-mode drawing switches, a camera projection, engine event hooks and a snapshot history.
The tool itself is a Lua script mod. Everything here is private (this branch), so the API is
documented **here**, not in the public `docs/scripting.md`. It extends the `gd` table of scripting
API 1; `gd.lab_api == 1` says the build has it (nil on public builds).

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
| `gd.joints(port)` | per joint, list position = index + 1: `{index, parent (-1 root), valid, x, y, z, sx, sy, on}` - world position from the joint's last computed matrix (never recomputed, so reading cannot change the game), `sx, sy` projected |
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
| `gd.timeline(port [, motion])` | the subaction script of the current action (or of `motion`'s row), walked read-only the way ftAction times it: `wait` adds frames, `wait_until` jumps to an animation frame, loops / calls / gotos are followed; stops at `end`, `wait_anim` ("anim_end"), 2000 commands or frame 1000 ("limit"). Returns `{motion, motion_name, anim_id, anim_name, end_frame, length, stop, script, events}`; each event `{frame (1-based, as frame-data sites count), op, name, addr, words, ...}` with decoded fields for `hitbox` (`id, group, bone, damage, size, ox/oy/oz, angle, kbg, wbk, bkb, element, element_name, shield_damage, hit_ground, hit_air`), `gfx`, `sfx`, `hitbox_damage/size`, `hitbox_remove`, `cmd_var`, `body_state`, `hurtbox_state`, `visibility`, ... Names for opcodes 10-58 follow the community decoders, checked against the decomp's handlers; 59 is Geno's escape (skipped by its length) |
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
<motion id> [frame]`, `lab events [port]` (the decoded script).

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
port's fps readout (`shim_vi.c`, opt-in) is suppressed during a LAB match
(`GenoLab_InMatch()`). The F9 info panel and toasts are still there; they only appear when you
ask for them.

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

| tab | rows |
|---|---|
| PLAY | Resume; Step +1; Step -1; Step +10; Focus (left / right) |
| DISPLAY | Display mode (left / right); the current mode's toggles; Lab UI shown / hidden |
| DUMMY | Target (left / right); Damage (left / right steps of 10, A applies `gd.set_percent`); Lock-step; Replay move; Mirror my pad |
| STATES | Save to library; Quick save / Quick load (memory slot 1-3); Reset positions (slot 4, match start); History (2 / 5 / 10 / 20 s); Hot reload (replay 1 / 2 / 3 / 5 s); then the library, one row per saved state (A loads, Y / DELETE deletes after a confirm). Stage B, 14.10 |
| EXIT | Change fighters; Change stage; Quit (no contest) |

Pad: stick / d-pad up and down, left and right change a value, A, B or START close, L / R
switch tabs. Keyboard: the arrows, Enter or Space, Backspace or Esc, `Q` / `E`. While the menu
is open it takes every key, so no Lab shortcut fires underneath it.

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
behaviour parameters, Geno state rows (behaviour, callbacks, flags, landing, motion), subaction
overlay words (inline or words files), and the Lab script.

What does not:
- **Layout changes.** The profile set, what a profile attaches to, a profile's Geno state count,
  or its overlay list. The restored state would not fit, so it says so and **restarts the match**
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
| 0x1B | ANIM_RATE | f |  | animation speed |
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
- Not in v1: `on_hit` dispatch, `air_vy` for non-multi-jump fighters, a "change subaction" op.


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
anim `next loop hold glide.start glide tornado drill drill.end`; iasa `none glide`; phys `none air
air_nodrift ground auto glide.start glide glide.attack glide.end tornado drill.start drill
drill.end`; coll `none air air_noledge ground ground_stop both glide drill`; any slot `like`.
`air` = Melee's aerial physics (gravity, fast fall, drift) / air collision with ledge grab
(platforms drop-through like FallSpecial). `both` = ground and air without changing state (walk
off -> airborne, land -> grounded, same state). Stable ids in geno_game_v2.inc.

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

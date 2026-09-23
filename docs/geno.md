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

`gd.history(depth)` (offline, gameplay) keeps one snapshot per logic frame for the last `depth`
frames (max 40); `gd.history()` returns `{depth, stored, back, now, slot_mb}`. `gd.step_back([n])`
restores the state `n` frames back and stays paused -> `true`, or `false, why`. Implementation: the
ring lives in `gw_snap` slots after the 4 savestate slots (addressed by index, so a ring entry can
never evict a named savestate); a slot is a MEM1 copy plus the game globals, **26.6 MB** on ACE,
and dirty-page saving keeps the per-frame cost small (60 fps held with depth 20; process private
memory ~1.2 GB). Savestates, loads and step-backs requested while **paused** are applied at once
(at the loop top, between frames) instead of waiting for a frame that never comes; loading a
savestate clears the history (it belongs to the other timeline). `gd.player().action_frame` is now
restored by loads too. `on_loadstate(0)` reports a step-back.

### 14.6 The Lab mod

`pc/geno/mods/geno-lab` (mod.json: `"kind": "script"`, `"gameplay": true`, not rollback_safe). Put
it (or a junction to it) in the mods folder; script id `geno-lab/lab`. Settings persist in
`scripts-data/geno-lab_lab/settings.txt`. Keys (game window focused, console closed; none is a
keyboard play key):

| key | |
|---|---|
| `P` | pause / resume |
| `N` | step 1 frame; hold = slow play; `CTRL+N` 10 frames |
| `B` | step back 1 frame; hold = slow rewind; `CTRL+B` 10 frames |
| `TAB` | focus the next fighter (the info panel shows the focused one and the next side by side) |
| `1` .. `0` | hit/hurtboxes, model, skeleton, joint numbers, ECB, stage collision (cycles lines / ledges / terrain), info panel, event log, hitbox labels, attributes (differences in yellow) |
| `F5` / `F6` | save / load state 1 |
| `X` | Lab on/off (off restores the default drawing) |
| `F3` | help |

Console: `lab help | lab port N | lab history N | lab back [N] | lab dump [N] | lab set <key> <value>`.
Online the Lab only reads (the status bar says so).

### 14.7 Stage 2 (not built)

Move timeline from the subaction events with a scrubber; set any fighter to any motion/frame
(offline); lock-step comparison of two ports; a "Lab" entry in the frontend menu; live edits
written back to `experiment/brawl-kirby/tuning.json`.

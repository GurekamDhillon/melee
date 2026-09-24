# Geno - GD's Melee's fighter-extension layer

Status: **v2** (v0 foundation, v1 Meta Knight script features (section 15), v2 action states, glide and MK specials (section 16)), private branch `private/geno` in the melee fork. Working name, approved by GD.
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

## 15. v1 script encodings (STABLE reference for the Meta Knight translator)

This section is the contract the Brawl -> Geno script translator (experiment/brawl-metaknight/)
emits against. A copy lives at `experiment/brawl-metaknight/geno_v1_encodings.md`; this section
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
translator; a copy of the MK part lives at `experiment/brawl-metaknight/geno_v2_encodings.md`
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
registry `pc/platform/geno_registry.c`. MK translator copy: `experiment/brawl-metaknight/
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
- In game (Meta Knight, `experiment/brawl-metaknight/tools/ingame_v2.py`, "v3" plans) and netplay:
  `_build/agents/beta/NOTES.md` (Geno v3 entry).

## 18. v4: the model follows Meta Knight's specials (STABLE reference)

Status: **built** (additive keys; a v3 exe logs them as unknown and keeps its behaviour). Code:
`pc/geno/geno_game_specials.inc` (`geno_drill_pose`, `geno_tornado_spin_*`). MK: `tools/build_mk.py`
sets the keys; checks `experiment/brawl-metaknight/tools/ingame_v4.py` (numeric, joints / hitboxes).

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

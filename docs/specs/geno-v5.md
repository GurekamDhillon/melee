# Geno fighter-extension format, v5 (with the additive v5.1-v5.5 features)

Revision 2026-11-02 (rev 2: melee-unlocked's answers folded in; sections 3, 4.5, 4.6, 4.8, 8.2, 9, 11 changed). Standalone spec for an independent C reader/runtime (melee-unlocked's Source Port). Derived from our source only:
`docs/geno.md` (sections 1-20; 14 for the LAB), `pc/geno/geno.h`, `pc/geno/geno_state.h`, `pc/geno/geno_game*.c|inc`,
`pc/platform/geno_registry.c`, `pc/platform/gw_fx.c`, `src/melee/ft/fighter.c`, `src/melee/it/item.c`, the mods under `pc/geno/mods/`,
and the Rust side (`rust/melee/src/ident.rs`, `rust/melee_data/src/gdlp.rs`: only the Geno version/identity words; the Rust engine has no Geno runtime).
Companion files: `docs/specs/geno-v5.schema.json` (JSON Schema, draft 2020-12), `docs/specs/tools/check_geno_schema.py` (validates the shipped mods),
`docs/specs/tools/geno_id.py` (the stable id), `docs/specs/fixtures/geno-v5/` (conformance fixture), `docs/specs/nhv1.md` (the per-frame hash).
Marker **OPEN** = not pinned down by the source; do not guess.

## 1. Scope

| in | out |
|---|---|
| `geno.json` of `attach` fighters, keys v1-v5.5 (sections 3-4) | **No PowerPC.** Geno scripts are Melee subaction command words (big-endian 32-bit), interpreted; a reader never executes PPC code of its own. m-ex PPC hooks are a different layer |
| the ftcmd escape (opcode 59) and its sub-commands (section 6) | **No Lua.** `lua` blocks, per-state `lua` keys, fighter Lua (geno 10) and the LAB's Lua `gd.*` API (docs/geno.md 14, 23) are not part of v5; a reader ignores the keys |
| Geno action states, behaviours, parameters, specials binding, counter windows, articles, effects packages and bindings | `define` (native fighter definitions, geno 6-9), `combo` and `meter` (geno 11), `controls`, `presentation`, `ai`, `toon`, `kirby_copy`, `thrown_as`, `fighter` (base `none`), standalone items (`items/<name>/item.json`), `shader-demo`, the LAB mod |
| rollback state layout (section 8), error handling (section 9), identity (section 5) | rendering of effects (cosmetic; section 7), hot reload (LAB only) |

All six shipped mods in `pc/geno/mods/` are `define` fighters (geno 6-10). There is **no shipped `attach` mod**: the v5 key set is exercised through the shared keys they carry
(`attributes`, `subactions`, `states`, `specials`, `articles`, `jumps`, `special_attributes`, `fx_bindings`) and through the spec's own examples.

## 2. Versioning and compatibility

| number | where | value | meaning |
|---|---|---|---|
| `geno` | file, required, integer >= 1 | v5 writes 5 | the format version the author targeted |
| `GENO_VERSION` | reader (`geno.h`) | 11 in our build | the newest file version this reader knows. `file.geno > GENO_VERSION`: log, **keep reading what is known**. `file.geno < 1` or missing/non-number: file ignored |
| `GENO_ID_VERSION` | `geno.h` | 1 | salt of the stable ids; **never bumped by v2-v11**. Changing the canonical form is a deliberate break (a test pins it) |
| `GENO_LEVEL` | `geno.h` | 3 | stale comment-level constant; unused for gating |
| dispatch/hook/value/behaviour/callback/param ids | `geno.h`, `geno_game_v2.inc` | stable | **numbers never change**; new features get new ids |

Rules:

| rule | detail |
|---|---|
| keys are not gated by the file's `geno` | this build parses every v1-v5.5 key whatever number the file declares. A conforming v5 reader must do the same (a v3 file may carry v5 keys) |
| unknown keys | ignored, but hashed into the entry id (section 5) so two installs never agree on an id while behaving differently |
| unknown script sub-commands | skipped by their `len`, logged once: scripts for a newer Geno still walk |
| unknown hook name, behaviour, callback, attribute, parameter | logged and ignored (the entry stays); see section 9 |
| additive only | v5.1-v5.5 added keys, ids, hook numbers 6-10, engine values 0x35-0x3E, sub-commands 0x3A-0x3C; none changes an earlier meaning. A reader older than a feature "cannot be assumed to execute newer commands" (docs/geno.md 19): the unknown-sub rule keeps it walking, not behaving the same |
| opt-in | a fighter no `geno.json` names runs exactly as without Geno. The only global change is opcode 59 in ftcmd, which no shipped retail/m-ex script contains (highest retail opcode used is 58) |
| scope rule | Geno adds character-level abilities inside Melee's rules; it never changes air dodge, hitstun/knockback formulas, ledge rules, tripping (docs/geno.md 1) |

Feature levels inside "v5": v0 (registry, escape variables/if, 4 hooks, attributes, multi-jump) -> v1 (script overlays, engine values, change action, REHIT/LINK, `on_land`, special attributes)
-> v2 (states, behaviours, glide/tornado/drill, `specials`) -> v3 (root motion, cape, `LEDGE`/`HIDDEN` values, 48 states) -> v4 (`drill.pitch_model`, `tornado.spin_anim`) -> v5 (articles, `on_hit`, counter windows)
-> v5.1 (16 articles, hitbox `slot`, `spawns`, hook-5 argument packing, `bone`, `min_speed`, `effect`, `children`, `specials.select`) -> v5.2 (`effects`, `lockon`, `ANIM_RATE` writable, `HBDMG`)
-> v5.3 (`aim_stick`, `ATTACK_CONNECTED`, `HBDMG`, `phys air_drift`, `effects[].frame/joint`, `spins`) -> v5.4 (`dash.search/aim`, `brake`, `STICK_LEN`, `FALL_LIMIT`, `motion_anims`, `fx`)
-> v5.5 (`HBSTUN`, `HBFLAGS`, hitbox `stun`, `fx_bindings`). Sections 15-19 of docs/geno.md are labelled STABLE.

## 3. Files, discovery, loading

| item | rule |
|---|---|
| mods folder | `MELEE_MODS_DIR`, else `mods/` next to the executable (`gw_Mods_Dir`). A mod is a folder `<id>/` with a `mod.json` (`id`, `name`, `version`, `kind`; `kind` `fighter` for the shipped fighter mods) |
| which mods | the **active** (enabled) set in **Geno load order: sorted by mod id, bytewise** (`strcmp` on the raw id bytes, so `Zeta` < `alpha`: upper case before lower case, no case folding, no locale); a folder not in the set is not read. **What it was before 2026-11-02:** the global mount order (`gw_Mods_ActiveAt(0..n-1)`: `base`-kind mods first, then `misc`, then content mods; inside a rank by case-insensitive id; a mod after the mods it requires). That order equals the bytewise order only when all active mods share a kind, ids differ in more than case and no requirement points at a later id, which holds for the six `fighter`-kind mods shipped in `pc/geno/mods/` (all `vanilla-*`), so none of their profile indexes moved. The mount order is still used for disc-file overlays and for effects packages |
| the file | `<mods>/<id>/geno.json`, beside `mod.json`; never a disc file. Missing file = the mod has no Geno part (not an error). Size limit **1 MiB** (`gn_read_file` returns NULL above it: the file is treated as missing) |
| when | once, at install time, before the first fighter spawns (lazily on the first registry query); never changed during a match. Hot reload exists only in the LAB (docs/geno.md 14.10) and refuses layout changes |
| parse | one pass per file in Geno load order (above); fighter entries in file order. Entries are numbered by load order (profile index); articles/overlay slots are numbered by it, so **the profile index is part of the layout** and, through the article kinds (4.8), of what the rollback hash and the NHV1 `items` columns see. Two independent readers must therefore sort identically before they number |
| JSON dialect | the reader accepts, beyond strict JSON: `//` line comments, a trailing comma before `}`/`]`, a UTF-8 BOM; `\uXXXX` escapes decode to `?`. Limits: **32768** values per file, **512 KiB** of string data, nesting depth **32**. Writers should emit strict JSON (the schema assumes it) |
| words files | `subactions[].file`: `<mods>/<id>/<file>` (path containing `..` refused; absolute paths are not specially handled, **OPEN**). Content: numbers separated by whitespace or commas, `0x..` or decimal (`strtoull`, <= 0xFFFFFFFF), `#` to end of line is a comment. Appended to the shared pool |
| models | `articles[].model.file`: any HSD archive readable through the DVD file layer (a mod's `files/` overlay or a disc file), `model.symbol` the public symbol (default: the first public symbol ending `_joint`). Loaded once per (profile, file, symbol) into a withheld read-only region of 1 MiB; articles of one profile with the same file+symbol share the load. The region exists only if a mounted mod's `geno.json` text contains `"articles"` at boot |
| effects | package `<mods>/<id>/fx/<name>/<name>.gfx.json`; the **last** mounting mod wins; textures/meshes relative to the package folder. Bindings: `<mods>/<id>/<fx_bindings path>` |
| target resolution | `attach` name -> Pl file (`PlMr.dat`...); the install's FighterKind for a Pl file: retail by code, m-ex by scanning the dense slots. Not installed -> the entry is inactive (kind -1) |
| two entries, one fighter | **a refusal, never "later wins"** (changed 2026-11-02; before it the later entry in mount order won and the loser was logged). If two entries (from two mods, or twice in one file) attach to the same fighter (same resolved FighterKind), **neither applies**: both entries go inactive (kind -1) but keep their profile index, so the layout does not change, and one log line per extra claimant names both mod ids: `geno: REFUSED: mods <A> and <B> both attach to <Pl file> (fighter kind <k>); neither entry applies - remove one of them`, then `geno: <mod>/<name> refused: another mod attaches to <Pl file> too - inactive` for each claimant. The outcome does not depend on load order. Per-profile merging is not defined |

Retail names (aliases after `/`): `mario fox captain/falcon donkey/dk kirby koopa/bowser link seak/sheik ness peach popo nana pikachu samus yoshi purin/jigglypuff mewtwo luigi mars/marth zelda clink/younglink drmario falco pichu gamewatch/gnw ganon/ganondorf emblem/roy`
(27, FighterKind 0..26 in this order, Pl code `Mr Fx Ca Dk Kb Kp Lk Sk Ns Pe Pp Nn Pk Ss Ys Pr Mt Lg Ms Zd Cl Dr Fc Pc Gw Gn Fe`). Matching is case-insensitive; `attach` may also be a Pl file name ending `.dat`.

## 4. The JSON schema

Machine-readable: `docs/specs/geno-v5.schema.json`. `fighters[]` entry (only `attach` entries are in v5 scope; `define` entries are skipped by a v5 reader with a log line):

| key | type | default | limit / rule |
|---|---|---|---|
| `attach` | string | required (or `define`) | name/alias/`Pl*.dat`; unknown name -> entry refused |
| `name` | string | the Pl file name | display only, < 64 bytes |
| `attributes` | object name -> number | none | names = `ftCo_DatAttrs` fields (list below); unknown name or non-number -> ignored; **128** entries max; `max_jumps` is an integer, clamped to **1..250** (logged); int-typed fields are truncated to s32 |
| `scale` | number | 1.0 | 0.5..2.0 else ignored (logged); multiplies `x34_scale.x` at fighter load; a respawn keeps it |
| `jumps.max` | integer | fighter's own | clamped 1..250; wins over `attributes.max_jumps` |
| `jumps.air_vy` | number[] | none | <= **16**; per-air-jump vertical impulse for multi-jump fighters, last entry repeats |
| `hooks` | object event -> string[] | none | events `on_init on_frame on_action on_land on_hit`; **8** per event; each `"name"` or `"name:arg"` (`atoi`, default 0); unknown name -> ignored |
| `subactions` | object[] | none | <= **336** per profile, **2560** slots over all profiles, **65536** words over all profiles; see below |
| `special_attributes` | object[] | none | <= **64**; `{index 0..264 \| offset (multiple of 4), float \| int}` |
| `on_land` | object[] | none | <= **16**; `{from: target, to: target, keep_frame: bool}` |
| `motion_anims` | object[] | none | <= **8**; `{motion 0..1023, subaction 0..1023}` |
| `states` | object[] | none | <= **48**; state n is motion id `0x400 + n` |
| `glide` `tornado` `drill` `cape` | object | defaults below | parameter blocks |
| `specials` | object | none | keys `n s hi lw air_n air_s air_hi air_lw`; `air_*` defaults to the grounded key |
| `articles` | object[] | none | <= **16** |
| `fx_bindings` | string | none | mod-relative path |
| `sounds` | object[] | none | v8 name table, <= 16 (`name`, `retail_sfx` 1..999999, `volume` 0..127 default 127); only referenced by `articles[].spawn_sound/end_sound`. Listed because shipped mods use it; **not** v5 |

### 4.1 Targets

A target is: a number 0..65535 (motion id); `"motion:N"`; `"special:N"` (motion = the fighter's first special + N; bounded to 256); `"geno:N"` (Geno state index); `"geno:<state name>"` (a state of the same profile, resolved by name; only states declared in the same entry); `"auto"` (Wait on the ground, Fall in the air);
`"helpless"` (Wait / FallSpecial); `"stay"` (`land` only: landing grounds the fighter, the state goes on). Case-insensitive. A bad target is logged and replaced by "none" (`~0`, "the behaviour's / the engine's default"), or the entry key is ignored (section 9).
Internal word: `[31:28] kind (0 MOTION, 1 SPECIAL, 2 GENO)`, `[27] RAW`, `[26] KEEP_FRAME`, `[15:0] id`; `auto=0xFFFFFFFE`, `helpless=0xFFFFFFFD`, `stay=0xFFFFFFFC`, none `0xFFFFFFFF`.

### 4.2 `attributes` names (index order; the first 40 are v1-v5, the rest append-only from v7 and accepted by this build in any file)

Float unless marked `i`:
`walk_accel_mul walk_accel_base walk_max_vel slow_walk_max mid_walk_point fast_walk_min ground_friction dash_initial_velocity dash_accel_mul dash_accel_base dash_max_velocity run_animation_scaling max_run_brake_frames ground_max_horizontal_velocity jump_startup_time jump_h_initial_velocity jump_v_initial_velocity ground_to_air_jump_momentum_multiplier jump_h_max_velocity hop_v_initial_velocity air_jump_v_multiplier air_jump_h_multiplier max_jumps(i) gravity terminal_velocity air_drift_stick_mul aerial_drift_base air_drift_max aerial_friction fast_fall_velocity air_max_horizontal_velocity jab_2_input_window jab_3_input_window standing_turn_frames weight model_scaling initial_shield_size shield_break_initial_velocity rapid_jab_window(i) clank_animation_length`
then (v7) `hit_spark_variant(i) ledge_jump_horizontal_velocity ledge_jump_vertical_velocity item_throw_velocity_multiplier heavy_throw_velocity_multiplier specials_ground_speed_retention kirby_b_star_damage normal_landing_lag landingairn_lag landingairf_lag landingairb_lag landingairhi_lag landingairlw_lag name_tag_height passivewall_vel_x wall_jump_horizontal_velocity wall_jump_vertical_velocity passiveceil_vel_x trophy_scale screw_attack_launch_velocity wall_jump_min_approach_speed damageice_ice_size damageicejump_vel_y damageicejump_vel_x_mult respawn_platform_scale warp_star_hitbox_scale camera_zoom_target_bone(i) unused_0(i) xDC x12C x13C x144 x150_damageice_unk x154_damageice_unk x168 x17C`.
Each is a member of `ftCo_DatAttrs` (`src/melee/ft/ftchangeparam.h`); the value is written over the field the file copy set. Applied **right after the engine copies the attributes from the Pl file, before scale/metal/etc. modifiers**, at spawn, respawn and every re-apply (`ftchangeparam.c` -> `Geno_ApplyAttrs`).

### 4.3 `subactions[]` (script overlays, v1)

| key | type | rule |
|---|---|---|
| `index` | integer 0..1023 | the subaction row (animation + script) whose **script** is replaced; the animation is kept |
| `words` | (integer \| string)[] | each: integer in -2^31..2^32-1 (non-integer refused) or a `0x..`/decimal string |
| `file` | string | a words file (section 3); used when `words` is absent |
| `move_tag` | string | `jab dash_attack tilt smash aerial grab throw special projectile`; the hit-rule vocabulary (LAB/script layer; no simulation effect) |

An overlay with no usable words, or with `index` out of range, is ignored (logged). A word that does not parse, or a full pool, refuses that overlay. The registry appends **one `0` word (End)** after every overlay so a script that runs off its end stops. Overlays cannot hold Melee commands with absolute pointers (goto/subroutine into the Pl file) in `attach` entries.
Inside an overlay, sub `0x13 ORIG` (len 1) continues with the fighter's original script of that subaction **from its start**. The overlay installs into the fighter's subaction table rows at spawn (idempotent, read-only after boot).

### 4.4 `states[]` (v2/v3/v5)

| key | type | default | meaning |
|---|---|---|---|
| `name` | string (<= 31) | `""` | target `"geno:<name>"` |
| `behavior` | enum | none | one of section 4.5; unknown -> state kept with no behaviour (logged) |
| `subaction` | integer 0..1023 \| `"motion:N"` \| `"special:N"` | the like motion's | the animation + effective script (a `subactions` overlay on it applies) |
| `like` | target | Fall (air behaviours), Landing/LandingFallSpecial (ground), AttackAirN (GlideAttack) | the motion whose row gives flags, move id and camera callback |
| `flags` | word | the like row's | `MotionState.x4_flags` |
| `move_id` | integer | the like row's | staling/attack id, `& 0xFF` |
| `next` | target | the behaviour's | where the state goes when its animation ends |
| `land` | target | Melee's Landing | where it goes on landing in the air; `"stay"` allowed |
| `landing_lag` | number | 0 (default lag) | with the default landing: LandingFallSpecial with this lag |
| `anim` `iasa` `phys` `coll` | enum per slot | the behaviour's | callback names (4.5); `"like"` = the like motion's own |
| `ledge` | `none\|front\|both` or 0/1/2 | `none` (0) | default ledge grab for `geno.anim_motion` |
| `liftoff` | bool | true | ground upward root motion takes off |
| `origin` | bool | false | first frame also moves by the clip's frame-0 offset |
| `gravity` | number | 0 | root-motion gravity multiplier of Melee's gravity |
| `facing` | `"entry"` | unset | travel keeps the entry facing |
| `counter` | object | none | `{from: int (default 1), to: int (default 2^31-1), target: target (default stay), negate: bool (default true)}` |
| `move_tag` | string | none | as above |

State n becomes MotionState row for motion id `0x400 + n` (`GENO_MOTION_BASE`), built at fighter spawn, idempotently, from the registry.

### 4.5 Behaviours and callbacks (stable ids in `geno_game_v2.inc`)

| behaviour | anim | iasa | phys | coll | `next` default | `land` default |
|---|---|---|---|---|---|---|
| `geno.air` | next | none | air | air | auto | Landing |
| `geno.ground` | next | none | ground | ground | auto | - |
| `geno.anim_motion` | next | none | anim_motion | anim_motion | auto | Landing |
| `geno.glide.start` | glide.start | none | glide.start | air | first Glide state | GlideLanding |
| `geno.glide` | glide | glide | glide | glide | first GlideEnd state | GlideLanding |
| `geno.glide.attack` | glide.after | none | glide.attack | air_noledge | Fall | Landing |
| `geno.glide.landing` | next | none | ground | ground | Wait | - |
| `geno.glide.end` | glide.after | none | glide.end | air | auto | Landing |
| `geno.tornado` | tornado | none | tornado | both | `next`, else helpless | stays |
| `geno.drill.start` | next | none | drill.start | drill.start | the Drill state | stays |
| `geno.drill` | drill | none | drill | drill | the DrillEnd state | stays |
| `geno.drill.end` | drill.end | none | drill.end | both | helpless unless it hit | stays |
| `geno.cape` | cape | none | cape | cape | decided at `decide_frame` | stays |
| `geno.cape.attack` | next | none | anim_motion | cape.after | helpless | stay |
| `geno.cape.end` | next | interrupt | anim_motion | cape.after | helpless | stay |

Callback names by slot. anim: `next loop hold glide.start glide tornado drill drill.end glide.after cape`; iasa: `none interrupt glide`; phys: `none air air_nodrift air_drift brake ground auto anim_motion glide.start glide glide.attack glide.end tornado drill drill.end drill.start cape`;
coll: `none air air_noledge ground ground_stop both anim_motion glide drill drill.start cape cape.after`; any slot: `like`. `air` = Melee's aerial physics/collision with ledge grab; `both` = ground and air without changing state.
**IASA trap:** the default `iasa` is `none`; an IASA script command alone does not give an input callback; use `"iasa": "interrupt"`. Paired states are found by declaration order among the profile's states with that behaviour (cape start ground/air, cape.attack N/N-air/F/F-air/B/B-air, cape.end ground/air, drill.start ground/air, drill.end ground/air).
Behaviour procedures (glide, tornado, drill, cape, multi-dash, anim_motion) are being re-specified clean-room in geno-v5.1; their arithmetic will be defined there. Until then they are implementation-defined and excluded from cross-build parity.

### 4.6 Parameter blocks

The parameter blocks `glide`, `tornado`, `drill` and `cape` (float; bool accepted as 0/1; `wNN` = numbered word also accepted) configure the behaviours of 4.5. Their key names, default values and meaning belong to the behaviour procedures: Behaviour procedures (glide, tornado, drill, cape, multi-dash, anim_motion) are being re-specified clean-room in geno-v5.1; their arithmetic will be defined there. Until then they are implementation-defined and excluded from cross-build parity. A reader accepts the four keys, ignores unknown names (logged) and has **128** parameter slots.
Geno button mask: ATTACK 1, SPECIAL 2, JUMP 4, SHIELD 8, GRAB 0x10, TAUNT 0x20, BLAST 0x40 (the last is geno 11).

### 4.7 `specials`

`{"n": target, ..., "air_s": {"select": "la_i:N"|"ra_i:N", "targets": [target, <=4]}}`: the eight special dispatch sites ask `Geno_SpecialEnter` first; a bound special enters the target; unbound/`none` runs the fighter's own special.
With `select`, the int variable's value n picks `targets[n]`; out of range = the fighter's own special. `air_*` inherits the grounded binding.

### 4.8 `articles[]` (v5 + v5.1-v5.5)

| key | type | default | meaning |
|---|---|---|---|
| `name` | string (<= 31) | `""` | referenced by `children[].article` |
| `model` | `{file, symbol}` | none (invisible, hitboxes only) | section 3 |
| `lifetime` | number | 60 (a value <= 0 also gives 60) | frames alive |
| `velocity` | [fwd, up] | [0,0] | initial velocity; fwd follows the owner's facing |
| `spawn` | [fwd, up] | [0,0] | spawn offset from the owner position (or `bone`) |
| `spawns` | [[fwd, up]]x4 | none | spawn variants picked by hook-5 argument bits [15:8]; variant 0 / out of range uses the first |
| `gravity`, `max_fall` | number | 0 | `vy -= gravity`; `vy >= -max_fall` (0 = no cap) |
| `accel`, `max_speed`, `min_speed` | number | 0 | speed change along the travel; cap (0 none); a negative accel stops at `min_speed` |
| `homing` | `{turn, range, delay}` | 0 | degrees/frame toward the nearest non-owner fighter (position + 5 up), within `range` (0 = any), after `delay` frames |
| `scale` | number | 1 | model scale (<= 0 -> 1) |
| `spin` | number | 0 | model roll, degrees/frame (visual) |
| `max_live` | integer | 4 (values < 1 -> 4) | per owner and article; a spawn past it is refused |
| `despawn` | `{hit, shield, stage, clank}` | all true | what ends it besides the lifetime (an absorber always takes it) |
| `angle` | number | 0 | initial velocity turned up by this many degrees (in the facing) |
| `bone` | integer | -1 | spawn from that fighter part's world position (+ offset); -1 = the fighter position |
| `effect` | integer | 0 | a Melee effect id attached at spawn |
| `effects` | (int \| `{id, frame, joint, count}`)[] | none | <= 8; id 5000..8999 = the owner's own m-ex effect bank |
| `spins` | `{joint, z}`[] | none | <= 4; a model joint rotating about its Z, `z` radians/frame |
| `children` | `{article, frame, every, count, spawn}`[] | none | <= 2; the article spawns articles (by name or index) from its own position, owned by its fighter; `every` 0 = once at `frame` |
| `fx` | string | none | effects package (section 7), attached to the root joint at spawn, detached when it goes |
| `show_model` | bool | false | with `fx`, keep the model drawn too (otherwise the model is hidden as a placeholder) |
| `hitboxes` | object[] | none | <= 8 entries on Melee's **4** item hitbox slots |

Hitbox entry: `damage` 1, `size` 3 (radius), `offset` [0,0,0], `angle` 361, `kbg` 100, `wkb` 0, `bkb` 0, `element` 0 (0 normal 1 fire 2 electric 3 slash 4 coin 5 ice), `shield_damage` 0, `sfx_severity` 1, `sfx_kind` 0,
`start` 1 / `end` 0 (active frames of the article's life, 1-based; 0 = to the end), `slot` (entry index mod 4; entries sharing a slot hand over in place and keep the victim list), `stun` 0 (0..255, extra hitstun frames),
`hits` `{ground, air, reflect, absorb, counter}` all true.
**Kinds:** article a of profile p is item kind `0x1000 + 8p + a` for a < 8, `0x1100 + 8p + (a-8)` for a >= 8, for p < 32; for p >= 32, `0x20000 + (p-32)*16 + a`. All are above m-ex's custom kinds and below m-ex's `5000+n`. Profile index p = load order (section 3: **mods sorted by id, bytewise**, then file order; before 2026-11-02 it was the global mount order, which gives the same numbers for the six shipped mods), so **kinds are profile arithmetic**, and two implementations agree on them only if they sort identically; the rollback hash identifies an article by (profile id, article index) instead (`Geno_ArtStableKind`).
Despawn reasons (log/event only): 0 timeout, 1 hit or absorbed, 2 shield, 3 stage, 4 clank.

### 4.9 Hooks (native, stable numbers)

| # | name | arg | does |
|---|---|---|---|
| 1 | `geno.log` | any | log arg and LA int 0 |
| 2 | `geno.jumps.refill` | n | give back n air jumps (0 = all) |
| 3 | `geno.jumps.to_var` | var | LA int[arg] = air jumps left |
| 4 | `geno.count_frames` | var | LA int[arg] += 1 |
| 5 | `geno.article.spawn` | `[7:0]` article, `[15:8]` spawn variant, `[31:16]` signed extra angle (deg) | spawn the profile's article (plain 0-7 = v5) |
| 6 | `geno.lockon` | `[15:0]` range (0 any), `[23:16]` max angle, `[31:24]` stick angle | aim at the nearest other-port fighter -> MOVE_F0/F1 (unit forward/up), MOVE_I0 (1 = locked), turns to face it |
| 7 | `geno.aim_stick` | `[7:0]` threshold x100, `[15:8]` max up, `[23:16]` max down, bit 24 may turn | heading from the stick polar angle -> MOVE_F0/F1 (saved in F2/F3, I2 = 1) |
| 8 | `geno.dash.search` | `[7:0]` threshold x100, `[23:8]` range | one aim-window frame: lock target (I0, I1 = its port) or store a stick sample (F2/F3, I2) |
| 9 | `geno.dash.aim` | `[7:0]` keep-facing degrees, `[15:8]` / `[23:16]` up ranges | MOVE_F0/F1 = world unit heading, MOVE_I3 = 1/2 in the up ranges; turns to the heading's side |
| 10 | `geno.brake` | `[15:0]` brake x1000, `[30:16]` cap x100, bit 31 horizontal only | cap the fighter's own speed then decelerate along it, never past rest |

A hook runs from the escape's `CALL` (sub 0x20; EXEC and ANIM passes only) or from a bound event. Pure parts (`GenoGame_LockonAim`, `GenoGame_AimStick`, `GenoGame_DashStick`, `GenoGame_DashAim`, `GenoGame_Brake`, `GenoGame_ArticleStep`) are in `geno_game.c` / `geno_game_articles.inc`.
Events (stable): 0 `on_init` (spawn/respawn, after the state block reset), 1 `on_frame`, 2 `on_action`, 3 `on_land`, 4 `on_hit`.

## 5. Stable id and netplay identity

Each `fighters[]` entry gets a 64-bit id (`docs/specs/tools/geno_id.py` is the reference; its self-test reproduces the value pinned by our C test `geno_registry_stable_ids`, and its id for `vanilla-hero` equals the id our build logged for it in the fixture run).

```
mix(h, v):  h ^= v;  h *= 0x9E3779B97F4A7C15;  h ^= h >> 31;  h *= 0xBF58476D1CE4E5B9;  h ^= h >> 29        (u64)
bytes(h, s): h = mix(h, len(s)); for each byte b: h = mix(h, b)
node(h, x):  h = mix(h, 0x100 + type)        type: null 0, bool 1, number 2, string 3, array 4, object 5
             bool: mix(h, x);  number: bytes(h, printf("%.9g", x));  string: bytes(h, raw bytes);
             array/object: h = mix(h, count); for each child: (object: h = bytes(h, key)); h = node(h, child)   (file order, duplicates kept)
id = node(mix(0x47454E4F00000000 /* "GENO" */, GENO_ID_VERSION), entry)
for each overlay in subactions order: id = mix(id, 0x4F56000000000000 | index); for each stored word incl. the End 0: id = mix(id, word)
(an entry with a Lua block also folds the Lua hash: out of scope)   if id == 0: id = 1
```
Whitespace, number spelling (`9` vs `9.0`) and the mod folder name do not change the id; the target's dense m-ex slot does not either. Vector: `{"geno":1,"fighters":[{"attach":"kirby","jumps":{"max":9}}]}` -> entry id `39ebc1bd1de2fc58`; `max` 8 -> `79da9105587fa5d0`.
Netplay: an `attach` entry's id is folded into the **target fighter's** content identity (`gw_mexid.c`: only fighters with an overlay; every other identity and the global hash unchanged). Two peers with different overlays for one fighter see it greyed out for each other; peers must run the same executable.

## 6. The script escape (ftcmd opcode 59)

A subaction command's opcode is the top 6 bits of its first word (retail 0-58; 59 is past every retail table). Words are big-endian like any Pl script.

```
word0 [31:26]=59  [25:20]=sub  [19:16]=len (total words, 1..15; 0 reads as 1)  [15:0] sub-specific
variable subs: [15:8] var A, [7] B is a var, [6:4] compare, [3:0]=0;  word1 = B (immediate, float bits when A is a float var, or var ref in [7:0] converted to A's type);  word2 (IF) = words to skip forward on failure
var ref (8 bits): [7:6] bank (0 LA int, 1 RA int, 2 LA float, 3 RA float)  [5:0] index (64 per bank)
next command = (address of word0) + len + skip   (skip is capped at 0x10000 words; a missing word reads as 0)
```

| sub | name | len | effect |
|---|---|---|---|
| 0x00 | NOP | any | none |
| 0x01-0x04 | SET ADD SUB MUL | 2 | `A op= B` (int or float by A's bank) |
| 0x05 / 0x06 | SETBIT / CLRBIT | 2 | `A \|= 1<<B` / `A &= ~(1<<B)` |
| 0x07 | DIV | 2 | `A /= B` (B == 0: unchanged) |
| 0x08 | GET | 2 | `A = value[word1]` (converted to A's type) |
| 0x09 | PUT | 3 | `value[word1] = B` (word2: immediate in the value's type, or var ref when `[7]`); read-only values ignore it (logged once) |
| 0x0A | RAND | 2 | int A: random 0..B-1 (B <= 0: 0); float A: random in [0,B). Uses the game RNG (`HSD_Rand`); **not drawn in the SKIP pass** |
| 0x10 | IF | 3 | if `!(A cmp B)` skip word2 words. cmp: 0 EQ 1 NE 2 LT 3 LE 4 GT 5 GE 6 BIT (`A & 1<<B`) 7 NOBIT |
| 0x11 | SKIP | 2 | skip word1 words forward |
| 0x12 | IFV | 4 | word0 `[7]` B is var, `[6:4]` cmp; word1 value id; word2 B (value's type or var); word3 skip when `!(value cmp B)` |
| 0x13 | ORIG | 1 | continue with the overlaid subaction's original script |
| 0x20 | CALL | 3 | native hook word1 with argument word2 (EXEC/ANIM only) |
| 0x30 | CHG | 2-4 | register a change-action check: word0 `[15:8]` condition, `[7]` B is var, `[6:4]` cmp, `[3]` NOT, `[2]` ONCE; word1 target word; word2 arg1; word3 arg2 |
| 0x31 | CHGAND | 1-3 | add an AND condition to the last CHG (same flag bits; word1 arg1, word2 arg2); <= 3 conditions per check |
| 0x32 | CHGCLR | 1 | drop all checks of this action |
| 0x38 | REHIT | 2 | `[15:8]` hitbox mask (ids 0-3); word1 = N frames (0 = stop); clears the victim lists every N frames, frozen in the attacker's hitlag |
| 0x39 | LINK | 2 | `[15:8]` mask; word1 = 0 off, 1 autolink direction, 2 direction + speed (Brawl angle 365, approximated; attacker speed < 0.05 keeps the Melee angle) |
| 0x3A | HBDMG | 2 | `[15:8]` mask, `[7]` B is var; word1 = damage (float, clamped >= 0); sets active hitboxes' damage after staling |
| 0x3B | HBSTUN | 2 | `[15:8]` mask, `[7]` var; word1 = extra hitstun frames 0..255 (int) |
| 0x3C | HBFLAGS | 2 | `[15:8]` mask, `[7]` var; word1 = flags & 0xF: 1 NO_HITLAG, 2 FLINCHLESS, 4 ZERO_DAMAGE, 8 FORCE_REACTION |
| other | | | skipped by `len`, logged once |

Variables: LA (long-term: kept across actions, **reset at spawn/respawn**) and RA (cleared on **every action change**), 64 int + 64 float each. Passes: EXEC (the frame's normal pass), ANIM (first pass of a new animation), SKIP (fast-forward, control flow only). Variables, SET..IF..SKIP, IFV, ORIG, CHG (not ONCE in SKIP), CHGAND, CHGCLR, REHIT, LINK, HB* run in all three; **CALL and RAND do not run in SKIP**. Using the escape at all sets `GENO_SF_SCRIPT` on the fighter's block (the fighter is then not inert).
`if (c) {T} else {E}` compiles to `IF !c -> skip |T|+2 ; T ; SKIP |E| ; E`. Skips only go forward.

Change-action semantics: a CHG registers a check; without ONCE it is tested **every frame for the rest of the action** (cleared by any action change); with ONCE tested once at the end of the registration frame. Tested in registration order, first match wins; **8** per fighter (a 9th is dropped).
A check whose first condition is GROUND (AIR) is also tested at the moment of landing (take-off) inside the collision callback.
Conditions: 0 ALWAYS, 1 ANIM_END, 2 GROUND, 3 AIR, 4 PRESSED(mask), 5 HELD(mask), 6 BIT(var ref, bit), 7 VAR(var A cmp B), 8 FRAME(animation frame >= N), 9 VALUE(value id cmp B).
Target kinds: MOTION, SPECIAL (first special + n), GENO (state index; **a GENO target the profile does not declare is skipped (logged) and a later check still fires**); RAW (plain `Fighter_ChangeMotionState`), KEEP_FRAME (continue at the current frame with Melee's mid-move transition flags; implies RAW).
Without RAW the common states Wait (in air: Fall), Fall, FallSpecial, Landing, LandingFallSpecial enter through Melee's own entry functions; every other target is `Fighter_ChangeMotionState(id, 0, frame 0, speed 1, blend 0)`.

### 6.1 Engine values (GET / PUT / IFV / VALUE; type f float, i int, W writable)

| id | name | | meaning |
|---|---|---|---|
| 0x00 | AIR | i W | 1 airborne; writing 1 on the ground = become airborne; writing 0 ignored |
| 0x01 | FACING | f W | +1/-1; writing 0 turns around |
| 0x02-0x04 | VEL_X VEL_Y GROUND_VEL | f W | `self_vel.x`, `self_vel.y`, `gr_vel` |
| 0x05 | FWD_VEL | f W | `self_vel.x * facing` |
| 0x06 0x07 | KB_VEL_X KB_VEL_Y | f | knockback velocity |
| 0x08-0x0C | STICK_X STICK_Y STICK_FWD CSTICK_X CSTICK_Y | f | -1..1; STICK_FWD = x * facing |
| 0x0D | ANIM_FRAME | f | animation frame (0-based) |
| 0x0E | ACTION_FRAME | i | frames in this action (1 on its first frame) |
| 0x0F | MOTION | i | motion id |
| 0x10 | PERCENT | f | |
| 0x11 | JUMPS_USED | i W | |
| 0x12 | JUMPS_MAX | i | |
| 0x13 0x14 | BUTTONS_HELD BUTTONS_PRESSED | i | Geno button mask |
| 0x15 0x16 | POS_X POS_Y | f | |
| 0x17-0x1A | CMD_VAR0-3 | i W | `fp->cmd_vars[0..3]` |
| 0x1B | ANIM_RATE | f W | `ftAnim_8006F0FC`: clip rate and script pace together; values <= 0 ignored |
| 0x1C | FAST_FALL | i | |
| 0x1D | TRIGGER | f | analog shield trigger 0..1 |
| 0x1E | GENO_STATE | i | current Geno state index, -1 none |
| 0x20-0x27 / 0x28-0x2F | MOVE_F0-7 / MOVE_I0-7 | f W / i W | behaviour per-move variables (kept across the states of one move) |
| 0x30 | LEDGE | i W | this action's ledge grab: 0 none, 1 front, 2 both, -1 the state's default; cleared each action |
| 0x31 | HIDDEN | i W | 1 = not drawn; kept across Geno states, cleared by any non-Geno action |
| 0x32 0x33 | TRANSN_FWD TRANSN_UP | f | this frame's root motion |
| 0x34 | MOTION_GRAVITY | f W | root-motion gravity multiplier, -1 = the state's |
| 0x35-0x39 | HIT_DAMAGE HIT_PORT HIT_COUNTER HIT_COUNT ARTICLES | f i i i i | last hit taken (before a counter negated it), attacker port -1 unknown, 1 if countered, hits taken since spawn, live articles of this fighter |
| 0x3B 0x3C | ATTACK_CONNECTED (W), ATTACK_CONNECTED_PREV | i | 1 when a hitbox of this fighter won a hit on a **fighter** (not a shield, not an article) in this action; `PREV` = the value as the previous action ended |
| 0x3D | STICK_LEN | f | stick vector length |
| 0x3E | FALL_LIMIT | f W | this action's fall-speed limit above terminal velocity; 0 = the fighter's |
| 0x1000+i / 0x2000+i | SPECIAL_F[i] / SPECIAL_I[i] | f / i | special attribute word i (i < 265) of `fp->dat_attrs`, read-only |

(0x1F, 0x3A, 0x3F+ unassigned.) PUT to a read-only id is ignored. IFV/VALUE compare in the value's own type.

## 7. Effects (`.gfx.json`) and articles

Articles are **Melee items** (section 4.8): spawned through the laser's path (`SpawnItem`, `Item_80268B18`, one state with no animation), moved by the item procs (`x40_vel` added to `pos` by `Item_802697D4`), drawn by the generic item renderer, hit like the laser (clank, reflect, absorb, shield), destroyed by the logic table predicates. An article has no hurtbox. The Geno additions: the descriptor from `geno.json`, three state callbacks (life + hitbox windows + children + effects at `anim`, motion at `phys`, stage ray test at `coll`) and the reactions.

Per article frame (item procs, section 8 order): `anim` (priority 1): `frame += 1`; `frame > lifetime` -> despawn reason 0; hitbox windows; children; effects at their frame; model joint spins; `phys` (priority 4): `prev = pos`, then `GenoGame_ArticleStep` (accel/min_speed brake, homing turn clamp, `vy -= gravity`, `max_fall`, `max_speed` clamp), `x40_vel = (vx, vy, 0)`, the engine then adds it to `pos`; `coll` (priority 5): if `despawn` has `stage`, a ray from `prev` to `pos` against the stage (`it_8026E9A4`) despawns with reason 3. `hitbox on`: damage = `(u32)(staling * ((damage + 0.5) * owner_scale))` (Melee rounds to whole percent). Reflected: the reflector owns it and the travel turns with the new facing.
Per-article state: `GenoArtVars` (section 8). Spawn: `geno.article.spawn` -> `geno_art_spawn`: refuse if `p`/`a` invalid or `live(owner, a) >= max_live` (logged); position = base + facing*fwd, up; velocity = (fwd,up) rotated by `angle + argument angle`; max 2 children per article; `fx` attaches at the spawn position's matrix.

Effects are **cosmetic**: no field of a package or binding reaches the simulation, the rollback snapshot or `NHV1`. A reader may omit them entirely and remain gameplay-conformant. Format (docs/geno.md 20; loader `gw_fx.c`):

| item | rule |
|---|---|
| package | `{ "geno_fx": 1 (>=1), "name", "textures"[<=24], "meshes"[<=24], "space": {forward[3], up[3]} (default +X forward, +Y up), "emitters"[<=96 kept; extras dropped with a log line] }`; emitters ordered by `order`; `kind: "mesh"` selects mesh emitters, anything else particle; `follow` `srt` (default) / `none` / `translate`; keys `transform emission shape particle color samplers material wave inherit program extensions ribbon ring` (render features R1-R6 are render-only) |
| limits | 64 packages, 2000 particles, 512 emitter instances, rollback ring 16 frames (a rewind past it starts clean) |
| step | `gw_Fx_Frame(frame)` once per logic frame with the game's scene frame counter after the scripts (`Geno_FxFramePost`, called after `Script_FramePost`); a frame not after the last one restores the state kept for the frame before and re-simulates. Nothing enters the game snapshot |
| bindings | `{ "geno_fx_bindings": 1, "states": [{ "state", "subaction", "clock": "animation"\|"game", "calls": [{ "frame", "package", "joint" (depth-first index, default 0), "joint2" (default -1), "offset"[3], "rotation"[3] degrees, "scale", "follow" (default true), "when", "situation": "ground"\|"air", "end_event": "off"\|"detach"\|"emitter_life"\|"state_exit"\|"owner_destroy", "end_frame", "shake": {amp, frames, seed}, "flash": {color, alpha, frames} }] }] }`; <= 8 sets, 96 states per set, 384 calls per set, 32 per state; a call under a `when` condition with `holds: 0` is skipped; a call whose package is not found is dropped (logged) |
| bound fighters | <= 8 at once (`fx_drv`); fired-call bits are part of the effects ring, not the game snapshot |

## 8. Runtime model

### 8.1 Where Geno runs in a logic frame

Procs run by priority level 0..max across all objects (`HSD_GObj_RunProcs`). Fighter procs (set up in `Fighter_Create`): 0 `Fighter_8006A1BC`, **1 `Fighter_8006A360`**, **2 `Fighter_8006ABA0`**, **3 `Fighter_Spaghetti_8006AD10`**, **4 `Fighter_procUpdate`**, **6 `Fighter_procMap`**, 7, 8, 9, 0xC, 0xD, **0xE `Fighter_ProcessHit_8006D1EC`**, 0x10, 0x12, 0x16. Item procs (`Item_802693E4` 0, `Item_80269528` 1, `Item_802697D4` 4, `Item_80269978` 5, 9, 11-14, 16).

| order | where | Geno step | runs for |
|---|---|---|---|
| 0 (spawn, respawn, Zelda/Sheik swap) | `Fighter_UnkInitReset_80067C98` | `Geno_FighterReset`: zero the `GenoState`, `profile = ProfileForKind`, `hold_motion = hold_frames = -1`, `ledge = -1`, `enter_from = -1`, `hit_port = -1`, `last_check = -1`; install overlays; build state rows; prepare article descriptors; **`on_init` hooks** | profiled fighters |
| 0b | `ftchangeparam.c` attribute copy | `Geno_ApplyAttrs`: attribute overrides + special-attribute overrides, before the engine's scale/metal modifiers | profiled |
| 0c | fighter load | `Geno_FighterScale`: `x34_scale.x *= scale` | profiled |
| each action change | `Fighter_ChangeMotionState`, right after `motion_id` is set | `Geno_OnActionChange`: clear RA banks, `action_time`, checks, REHIT/LINK/HBSTUN/HBFLAGS tables, `ledge`, root-motion scratch; `atk_connected_prev = atk_connected; atk_connected = 0; fall_limit = 0`; `hidden = 0` for non-Geno motions; **`on_action` hooks**. For ids >= 0x400 the MotionState row comes from `Geno_MotionRow`; `motion_anims` may substitute a subaction row for a common motion | profiled or script-using |
| 1 | `Fighter_8006A360`, inside the `!x221F_b3` guard, **not** gated by hitlag | after the m-ex `onFrame` dispatch: `Geno_OnFrame` = **`on_frame` hooks** (run during hitlag too) | profiled |
| 1 | same proc, inside the `!hitlag (x2219_b5)` branch | animation + script advance (`ftAnim_8006EBA4`): the ftAction loops call `Geno_FtCmd` for opcode 59 in EXEC/ANIM/SKIP passes (**CALL hooks run here**) | any fighter whose script holds an escape |
| 1 | same, after `ftCo_800DEF38` and `pl_800411C4`, before `anim_cb` | `Geno_PreAnim`: `action_time++`, REHIT timers (clear the victim list every N frames, frozen in hitlag because this branch is), then registered change-action checks in order (first true wins -> change action and **skip the old state's `anim_cb`**), then the glide jump-hold entry | profiled/scripted |
| 2/3 | `Fighter_8006ABA0`, `Fighter_Spaghetti_8006AD10` | (geno 11 combo input: out of scope); the state's `iasa`/input callbacks; `Geno_SpecialEnter` at the special dispatch sites (`ftCo_Attack100`, `ftCo_SpecialAir`, `ftCo_SpecialS`) | profiled |
| 4 | `Fighter_procUpdate` | the state's `phys` callback (row callbacks); `Geno_FallLimit` inside `ftCommon_Fall` | Geno states |
| 6 | `Fighter_procMap` | `Geno_CollBegin` -> the state's `coll` callback -> `Geno_CollEnd`. Ground edges inside it (`ftCommon_8007D6A4` land, `8007D5D4`/`8007D60C` take-off) call `Geno_GroundEdge`: `on_land` hooks on landing, then the script's GROUND/AIR checks, then `on_land` map; the picked target is **performed right after the callback returns**. Edges outside a collision callback never pick a target | profiled/scripted |
| contact | `ftColl_8007A06C` (fighter hitbox won) | `Geno_Autolink` (LINK), `Geno_HitStunBonus`/`Geno_HitFlags` (HBSTUN/HBFLAGS, `ftcoll.c` ~779), `ATTACK_CONNECTED = 1` | attacker profiled/scripted |
| 0xE | `Fighter_ProcessHit_8006D1EC`, start of the `!x221F_b3` block, before Melee reacts to this frame's hit | `Geno_OnHit`: only if `profile >= 0` and (`kb_applied != 0` or `percentTemp > 0`): `hit_count++`, `hit_damage = percentTemp`, `hit_port = source player (0-5 else -1)`, `hit_counter = 0`; if the current Geno state has a counter window and `from <= action_time <= to`: `hit_counter = 1`, `counters++`, with `negate` zero `kb_applied` and `percentTemp`, go to `target`; then **`on_hit` hooks**. Then `Geno_HitReact` (HBFLAGS flinchless/forced reaction) and `Geno_NoHitlag` | profiled |
| item 1 (priority 1) | article `anim` | lifetime, hitbox windows, children, effects | articles |
| item 4 | article `phys`, then the engine's position update | `GenoGame_ArticleStep` | articles |
| item 5 | article `coll` | stage ray test | articles |
| item 11-16 | hit/shield/clank/reflect/absorb predicates | despawn by mask; reflect updates facing/velocity | articles |
| after procs | `gm_801A4D34`, after `Script_FramePost` | `Geno_FxFramePost(scene frame)`: effects bindings drive + `gw_Fx_Frame` (cosmetic) | effects |

"Inert" = no profile **and** no escape ever executed by the fighter's scripts: every Geno entry point returns at once. A fighter with a profile whose scripts never use the escape is not inert (hooks run).
Order inside one frame between fighters follows the engine (player slot order). `ProfileForKind` is stable for a match.

### 8.2 Rollback snapshot: exactly what is Geno state

All of it is **game memory** (a plain global or the item heap) and so is covered by the engine's snapshot (`gw_snap.c` takes MEM1 and every game-object `.data/.bss` symbol, including `pc_geno_*`):

| state | where | size / layout |
|---|---|---|
| `Geno_StateBlock[6][2]` | global; `[player slot][0 = main, 1 = sub-fighter (Nana, the inactive Zelda/Sheik form)]` | 12 x **1708** = 20,496 bytes; layout below |
| `GenoArtVars` | in the article item's `xDD4_itemVar` (item heap) | 44 bytes: `s32 profile, article, frame, why; f32 vx, vy; Vec3 prev (12); f32 spin; s8 cur[4]` |
| fighter and item structs | the engine's own | every field Geno writes: `motion_id`, `facing_dir`, velocities, hitboxes (`x914[]`), `cmd_vars`, `x34_scale`, `x221E_b5` (HIDDEN), animation AObjs (clip frame lives there), `dat_attrs` after override |
| `Geno_LuaBlock`, `Geno_ComboBlock` | globals | exist in our build (geno 10/11); all zero for a v5 profile; out of scope |
| derived, rebuilt idempotently at every spawn from the registry (same bytes every time): `Geno_ScriptPool[65536]`, `Geno_OverlayOrig[2560]`, state `MotionState` rows (`Geno_Rows`), parameter arrays (`Geno_Params`), article descriptors/attrs/model descs | globals | snapshotted because they are globals, but **never a source of truth**: a C port may keep them native and rebuild them |

**Not** in the snapshot (and must not be needed to resume): the registry and hook table (install-time, `const`), article model data (read-only after load, in a region that outlives scenes), the effects runtime (own 16-frame ring), logs, `GenoState.resets/hook_calls/state_entries/changes/art_spawned/counters/motion_land` are diagnostics but sit in the block and are saved with it.
No pointer is in `GenoState`; floats are IEEE binary32; the block is plain big-endian ints/floats in guest memory.

`GenoState` (offsets in bytes; all `s32`/`u32`/`f32` fields are 4 bytes; group = who clears it):

| offset | field | size | group |
|---|---|---|---|
| 0 | `profile` (registry profile of this fighter's kind, -1 none) | 4 | reset |
| 4 | `kind` | 4 | reset |
| 8 | `flags` (`GENO_SF_SCRIPT` = 1: the escape was used since reset) | 4 | reset |
| 12 | `resets` (diagnostic, survives the reset: incremented) | 4 | - |
| 16 / 272 | `la_i[64]` / `ra_i[64]` | 256 each | reset / per action |
| 528 / 784 | `la_f[64]` / `ra_f[64]` | 256 each | reset / per action |
| 1040 1044 | `hook_calls`, `extra_jumps` (air jumps past Melee's multi-jump table) | 4 | reset |
| 1048 | `action_time` (`ACTION_FRAME`) | 4 | per action |
| 1052 1056 | `nchecks`, `last_check` | 4 | per action |
| 1060 | `checks[8]` = `{u32 target, once, ncond; GenoCond cond[3] = {head, arg1, arg2}}` (60 bytes each) | 384 | per action |
| 1444 1460 1476 1492 | `rehit_period[4]`, `rehit_count[4]`, `link_mode[4]`, `stun_add[4]` | 16 each | per action |
| 1508 1512 | `stun_bonus`, `stun_bonus_frame` (victim side; a bonus is valid for its own frame) | 4 | not cleared by actions |
| 1516 | `hb_flags[4]` | 16 | per action |
| 1532 1536 | `react_flags`, `react_frame` | 4 | frame-scoped |
| 1540 1544 1548 | `in_coll`, `edge_pending`, `edge_target` (collision-edge scratch) | 4 | not cleared by actions |
| 1552 | `changes` (diagnostic) | 4 | reset |
| 1556 1560 | `hold_motion`, `hold_frames` (glide entry counter, -1 = released) | 4 | **not** cleared by actions |
| 1564 / 1596 | `move_i[8]` / `move_f[8]` (MOVE_I/F) | 32 each | **not** cleared by actions (a move's states hand them on) |
| 1628 1632 1636 | `state_entries`, `enter_from`, `hidden` | 4 | reset (`hidden`: also cleared by non-Geno actions) |
| 1640 1644 1648 1652 1656 1660 | `hit_count`, `hit_damage` (f32), `hit_port`, `hit_counter`, `counters`, `art_spawned` | 4 | reset (persist until the next hit) |
| 1664 | `fall_ledge` (kept across actions; cleared by the first non-Geno action that is not FallSpecial) | 4 | - |
| 1668 | `ledge` (-1 = state default) | 4 | per action |
| 1672 1676 1680 1684 1688 | `motion_started`, `motion_vy` (f32), `motion_land`, `motion_facing` (f32), `motion_gravity` (f32, -1 default) | 4 | per action |
| 1692 | `enter_keep` | 4 | per state entry |
| 1696 1700 | `atk_connected`, `atk_connected_prev` | 4 | per action (moved, then cleared, at the change) |
| 1704 | `fall_limit` (f32) | 4 | per action |
| 1708 | end | | |

Reset (`Geno_FighterReset`) zeroes the whole block and then sets the non-zero initial values listed in 8.1; `resets` is carried over and incremented.
The Geno-specific state is **not** folded into `NHV1` (`docs/specs/nhv1.md` section 6); it is folded into our rollback hash by `GenoDefine_StateDigest` for defines only. **Geno digest for `attach` fighters (added 2026-11-02):** an optional trailing NHV1 column group `geno0`..`geno3` (one word per port) hashes the LA/RA/MOVE variable banks and `action_time` with NHV1's mix; field order, hashing and inert rule are in `docs/specs/nhv1.md` 8.1, reference code `docs/specs/tools/nhv1_ref.py geno_digest()`. Our C build writes it when `MELEE_NEUTRAL_HASHLOG_GENO=1` is set beside `MELEE_NEUTRAL_HASHLOG`. The Rust engine has no Geno runtime, so it cannot write it.

Determinism rules (docs/geno.md 4): no host time or randomness (RNG = the game's `HSD_Rand` only); no intra-frame loops (skips go forward only); logs rate-limited (resimulation repeats frames); registry and hook table never change during a match; peers run the same executable.

## 9. Errors and refusals

| condition | result |
|---|---|
| file missing | no Geno part for that mod |
| file > 1 MiB, invalid JSON, > 32768 values, > 512 KiB strings, depth > 32 | file ignored (`geno: <mod>/geno.json is not valid JSON (...) - ignored`) |
| no/non-numeric/`< 1` `geno` | file ignored |
| `geno` > reader version | logged, known keys read |
| `fighters` missing / not an array | zero entries |
| entry not an object | skipped |
| neither `attach` nor `define` | `entry has no attach or define - refused` |
| `attach` names no known fighter | `unknown attach target - refused` |
| target fighter not installed | entry kept but **inactive** (kind -1) |
| two entries attach to one fighter | **both refused** (inactive, kind -1, profile index kept); log `geno: REFUSED: mods <A> and <B> both attach to <Pl file> ...` naming both mod ids (section 3) |
| `define` entry in a v5 reader | skipped (logged); in our build it parses the native-definition path |
| unknown attribute / non-number | that key ignored (logged) |
| `max_jumps` / `jumps.max` outside 1..250 | clamped (logged) |
| `scale` outside 0.5..2.0 | ignored (logged) |
| unknown hook name | ignored (logged); > 8 per event: rest dropped |
| overlay: bad index, no usable words, bad word, pool/slot full | that overlay ignored (logged); the rest continue; too many: the rest ignored |
| special attribute missing index/offset or value | ignored (logged) |
| `on_land` entry missing/bad `from`/`to` | ignored (logged) |
| `states` > 48 / `articles` > 16 / parameter slots > 128 | the rest ignored (logged) |
| unknown behaviour / callback / parameter name | state kept with none / the behaviour's callback / parameter ignored (logged) |
| bad target in `next`/`land`/`like`/`counter.target` | key ignored (logged) |
| `children` naming no article | that child ignored (logged) |
| article model missing / not an HSD archive / symbol missing / no room | article is invisible (logged); `fx` package missing: `fx` = none |
| `fx_bindings` missing or not `geno_fx_bindings` 1 | no bindings (logged) |
| runtime: spawn refused (`max_live`, bad index, no item) | logged, no article |
| runtime: unknown hook id | logged, ignored; unknown script sub: skipped |
| runtime: CHG to an undeclared GENO state | skipped (logged), later checks still run |
| runtime: article model region not reserved (mod enabled by hot reload) | no models until restart |

Nothing in the v5 path aborts the program; every refusal is a log line starting `geno:`. (The v6+ `define` path refuses whole entries on budget errors; out of scope.)

## 10. Conformance fixture

`docs/specs/fixtures/geno-v5/`: a headless run of the shipped `vanilla-hero` define (a `geno 6` Mario-based fighter with one authored jab overlay and `walk_max_vel` 1.8), the only existing mod fighter runnable headless in our build. It exercises the registry path (id, attributes, overlay install) and an overlay script; it does **not** exercise states, articles or the escape's variables.

| file | content |
|---|---|
| `nhv1_hero_jab.csv` | NHV1 per-frame hashes (`docs/specs/nhv1.md`), frames 0..300 |
| `state_hero_jab.csv` | per-frame fighter state logged by `pad_hero_jab.lua`: `frame,p1_kind,p1_action,p1_action_frame,p1_anim_frame,p1_x,p1_y,p1_vx,p1_vy,p1_percent,p1_facing,p1_airborne,p1_hitlag,p2_action,p2_percent,p2_x,p2_y,p2_hitlag`, 300 rows. Row `n` is read in `on_frame_pre` of iteration `n`, i.e. it is the end-of-logic state of NHV1 frame `n-1` (inferred, not probed) |
| `pad_hero_jab.lua` | the input: P1 idle until frame 100, half-stick right until 205, A at 215/245/275; P2 idle |
| `geno.json` | the mod's file |

Recorded: build root `_build/agents/specs` (private), exe sha256 prefix `9cca2879` (a copy of the `rustgold` build of branch `rust-port`), `MELEE_TURBO=1`, window off-screen, `MELEE_SCRIPTS=0`, `MELEE_TEST_SEED=12345`, scene
`mode=vs;at=match;p1=geno:vanilla-hero/human;p2=mario/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off`, `MELEE_MODS_DIR` = a folder holding only `vanilla-hero`. Two passes: both CSVs and both state logs byte-identical.
Logged by the run: `geno: fighter Vanilla Hero (PlMr.dat) -> kind 126: id 47e0fa8287c3c75f`, `subaction 46 script replaced by overlay slot 0`, `attributes overridden: 1 field(s), max_jumps 2`.
Observable: the jab (action 44) connects at state row 219: P2 percent 12.000000 (the overlay's authored 12% hitbox), P2 action 80 (damage), hitlag 7 on both. No disc-derived asset is in the files (hashes and numbers only).
Reproduce (`tools/port/run.sh` with a private `GW_BUILD_ROOT`; needs the user's disc via `.env`, never printed):

```
export GW_MELEE=<worktree> GW_BUILD_ROOT=<private root holding melee-pc.exe>; set -a; . <workspace>/.env; set +a
MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345 MELEE_TURBO=1 \
MELEE_MODS_DIR=<folder with vanilla-hero> MELEE_PAD_SCRIPT=<fixtures>/pad_hero_jab.lua MELEE_NEUTRAL_HASHLOG=<out>.csv \
MELEE_SCENE='<scene above>' bash tools/port/run.sh --max-seconds 200 <sandbox> --iso "$GW_ISO_VANILLA"
# the state rows are the GENOFIX lines of <sandbox>/melee-pc.log
python tools/rust-port/compare_golden.py docs/specs/fixtures/geno-v5/nhv1_hero_jab.csv <out>.csv
```
To produce a fixture for an `attach` v5 fighter (states, articles, escape variables): write a mod with `geno.json` (`{"geno":5,"fighters":[{"attach":"mario", ...}]}`), run the same scene with `p1=mario/human`, and log with a pad script. The LA/RA/MOVE variables and `action_time` are now visible per frame through the NHV1 trailing columns `geno0..geno3` (`MELEE_NEUTRAL_HASHLOG_GENO=1`, `docs/specs/nhv1.md` 8.1); `nhv1_hero_jab_geno.csv` in this folder is the hero-jab run with them. The checks, rehit tables and motion scratch are still not dumped (OPEN 4).

## 11. OPEN items

| # | item |
|---|---|
| 1 | behaviour procedures (glide, tornado, drill, cape, multi-dash, anim_motion) and their parameter defaults: Behaviour procedures (glide, tornado, drill, cape, multi-dash, anim_motion) are being re-specified clean-room in geno-v5.1; their arithmetic will be defined there. Until then they are implementation-defined and excluded from cross-build parity. Hooks 6-10 and `GenoGame_ArticleStep` arithmetic are also not restated |
| 2 | words-file path handling beyond `..` refusal (absolute paths, drive letters) |
| 3 | meaning of `x221F_b3` (the guard around `Fighter_8006A360` and `ProcessHit`'s body); assumed "fighter not frozen" |
| 4 | digest of `attach` state is specified and implemented (nhv1.md 8.1, columns `geno0..3`) for the variable banks and `action_time` only; the change-action checks, rehit/link/stun tables and motion scratch are not in it |
| 5 | **closed 2026-11-02 (melee-unlocked asked for it):** two mods attaching to one fighter are a refusal naming both mod ids, never later-wins (section 3, 9). Per-profile merging stays undefined |
| 6 | the fixture covers a `define` fighter only; no shipped `attach` mod exists to record |
| 7 | row/frame alignment of `state_hero_jab.csv` vs `nhv1_hero_jab.csv` is inferred |
| 8 | `GENO_LEVEL` is stale (3) and unused; do not read it as the feature level |

## 12. Agreed with melee-unlocked (hero88go, 2026-11-02)

| item | decision |
|---|---|
| 4.5 numeric procedures | **not written out**: re-specified clean-room in geno-v5.1 (provenance); implementation-defined and excluded from cross-build parity until then |
| 3 / 9 two mods, one fighter | refusal naming both mods; our C loader changed to match (`geno_registry.c`, test `geno_registry_attach_conflict`) |
| 3 / 4.8 load order | sorted by mod id, bytewise; previous order documented; our C loader changed (`gn_sorted_active`) |
| 8.2 Geno digest | NHV1 trailing columns `geno0..geno3` (nhv1.md 8.1); implemented in our C build behind `MELEE_NEUTRAL_HASHLOG` + `MELEE_NEUTRAL_HASHLOG_GENO=1` |

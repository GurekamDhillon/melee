# WebAssembly mod host (task #25)

Code mods for the native engine: sandboxed wasm modules behind hooks-v1 rows, implementing the
mod-c-v1 ABI (docs/specs/mod-c-v1.md, gdmod.h, hooks-v1.tsv). Crate `rust/melee_modhost`. Not to be confused with
`melee/src/mods.rs`, the gdmods *file* overlay (disc paths), which is unrelated and unchanged.

## Use

| what | how |
|---|---|
| enable | `MELEE_WASM_MODS=<dir>`: every `<dir>/*.wasm` is loaded at engine boot (`Engine::boot_host`). Unset / empty / `0` = off (default) |
| build the example | `bash tools/rust-port/modhost/build_mods.sh` -> `build/mods/hello_mod.wasm` (needs `rustup target add wasm32-unknown-unknown`) |
| determinism test | `bash tools/rust-port/modhost/determinism.sh [--skip-build]` -> `MODHOST OK` |
| unit tests | `cargo test -p melee_modhost` (loader refusals, table lookup, the built example loads) |
| trace | `MELEE_MODHOST_TRACE=1` logs every damage hook call |

A refused mod fails the boot with a message naming the check (`modhost: <file>: refused: <check>: ...`): never
silent, never partial. Loaded mods are logged with id, version, FNV-1a-64 of the module and hook count.

## Design

| topic | choice |
|---|---|
| runtime | **wasmi 2.0** (pure-Rust interpreter) with features `validate` + `deterministic` (NaN canonicalisation inside the VM). Chosen over wasmtime: no codegen, small, its float ops are the host's IEEE scalar ops, and its whole state is ordinary Rust heap that the snapshot covers. Deviation from spec section 10 ("AOT only"): documented here; the per-hook cost is fine for the hook counts involved |
| watchdog | fuel metering, `HOOK_FUEL` = 1,000,000 per hook call. Fuel is deterministic, so a runaway mod traps at the same point on every machine; a trap disables that mod for the rest of the session (logged). Closes mod-c-v1 OPEN 9 for this host |
| sandbox | imports only from module `gd`, functions only, and only the implemented subset (anything else fails instantiation, naming the import); no start section; memory exported, fixed (min = max) and at most 16 pages (1 MiB); no SIMD / memory64 / threads (features not compiled in); function table must be exported (`--export-table`) |
| floats at the boundary | every float handed to or taken from a mod is NaN-canonicalised (`0x7FC00000`); `gd_fighter_set_facing` accepts only +-1 |
| manifest | read as data before any hook runs: `gd_mod_manifest` / `gdmod_abi_version` are exported i32 globals holding the address (what both clang and rustc emit for exported statics). All section 7 checks: magic, struct_size 124, ABI major / abi_min, mod_id charset and uniqueness, required features (host offers only LOG), hook_table_version 1, state_bytes <= 256 KiB and <= memory, hook_count <= 256, every address in hooks-v1, mode, priority, flags, and the function's wasm signature against the row |
| ordering | per address and mode: (priority, mod_id bytewise, declaration index); post is the exact reverse. Files are read sorted, and the order is a total order, so load order never matters |
| handles | `gd_fighter` = `(1 << 16) + slot + 1`, re-resolved on every call through `Player_GetEntity`; stale handles read 0 / write nothing and are counted (`HostData::bad_handles`). Nana is not addressable (her damage runs no hook) |
| RNG | `gd_rand_u32/f32` draw from the game's logic RNG (`HSD_Randi/Randf`): game state, rolled back with it |
| `gd_match_frame` | frame.begin count since load, kept in the host (rolls back) |

### Wired rows

Every hooks-v1 address passes the table check, but only these rows are wired; a mod declaring any other row is refused
("not wired by this host yet"), never accepted as a silent no-op.

| rows | site | modes |
|---|---|---|
| `frame.run_procs` 0x80390CFC | `gm_801A4D34_tick` around `HSD_GObj_RunProcs` (`gmscene::MOD_FRAME`); post before `Script_FramePost` (OPEN 3 default). Frame begin also clears the per-frame handle tables | pre / post (no replace) |
| `damage.fighter_take_damage` 0x8006CC7C | `Fighter_TakeDamage_8006CC7C` (`melee_fighter::hooks::MOD_TAKE_DAMAGE`, which hands the generic dispatcher an `orig` closure) | pre / replace / post |
| `fighter.proc00/01/04/06` | `Fighter_8006A1BC` (read input there with `gd_fighter_get_input`), `Fighter_8006A360`, `Fighter_procUpdate`, `Fighter_procMap` | pre / replace / post |
| all 19 `action.*.enter` rows (action-state changes) | the `ftCo_*_Enter` definitions | pre / replace / post |
| `item.spawn`, `item.spawn_gobj`, `item.spawn_gobj2`, `item.spawn_void` | `Item_8026862C`, `Item_80268B18/5C/9C` | pre / post; replace refused (handle result). A pre `GD_SKIP` returns a null item (no spawn) |
| `item.destroy`, `item.change_state`, `item.update0`, `item.update5` | `Item_802680CC`, `Item_80268E5C`, `Item_802693E4`, `Item_80269978` | pre / replace / post |
| `stage.update`, `stage.on_start` | `Ground_801C0C2C`, `Ground_801C0FB8` | pre / replace / post |

Generic sites are `melee_core::mod_hook!(addr, [args], orig)` at the top of the function (the body moved to
`<name>__orig`). With no mod hooking that address the cost is a `None` load and branch, then the same body.
`melee_core::modhook::{DISPATCH, HOOKED}` are installed only for addresses some mod hooks.

**Dispatch** (`melee_modhost::dispatch`): the pre chain (ascending order; `GD_SKIP` stops it and skips the replace and
the original; `gd_set_return_*` sets a skipped non-void row's result), then the replace hook or the original, then the
post chain (reverse order; a non-void post gets `(args, ret)` and returns the new result; for handle results this host
ignores the returned value). Queued `gd_fighter_request_action_state` calls are applied after the pre/replace part and
after the post chain. Re-entrancy: depth 8 per address, deeper activations run the original only (`deep_calls`). A
sub-fighter (Nana) argument runs the original only. Each hook's wasm signature is checked at load against its row.

**Replace** (`GD_FEATURE_REPLACE` required): at most one per address across all mods (a second is a refusal), only on
`sig_src = decomp` rows, never on the frame row or a handle-returning row. `gd_orig_call_void/i32/f32` runs the
original once per activation (a second call is a no-op returning 0). While it runs, hooks of other addresses fire in
the caller's store context (wasmi re-entrancy through `Caller`; the outer fuel is restored afterwards). A trapping
replace falls back to the original unless it already ran.

**Accessors**: all of gdmod.h. Fighters (`get_input`, `request_action_state`, `gobj`, pos/vel/percent/facing, the
view), players (`stocks`, `character`, `present`), items (`count`, `at`, `kind`, `gobj`, `read`, `write`), spawn
requests (`get/set_kind`, `get/set_pos`; setters in pre hooks only, kinds limited to the common items), gobjs (`kind`,
`to_fighter`, `to_item`), `gd_stage_kind`, RNG, trig (`gd_sinf/cosf` = MSL as retail computes them, `gd_atan2f` =
lbtrigf, `gd_powf` = lb; closes OPEN 1 for this host), `gd_orig_call_*`, `gd_set_return_*`, `gd_log`. Item and spawn
writes need `GD_FEATURE_ITEM_WRITE`, action requests `GD_FEATURE_ACTION_REQUEST` (declared required or optional).

**Handles**: fighters `(1<<16)+slot+1`, re-resolved each call; items `(2<<16)+n` and other gobjs `(3<<16)+n` from
per-frame tables (valid in the logic frame they were issued in; an item only while it is on the item list); the spawn
request `(4<<16)+1` (the innermost spawn activation). Action requests accept only Wait / Squat / SquatWait on the
ground and Fall / FallSpecial in the air (no flags, frame 0, rate 1); anything else is refused and logged (OPEN 12).

## Mods off = unchanged hashes

With `MELEE_WASM_MODS` unset nothing is created and both hook pointers stay `None`; each site costs one `None` check
and the game path is the same code. Verified: `cpu_fox_falco_dl` matches its committed golden for all 3601 frames with
the host compiled in (determinism.sh step 1) and the gate (below).

## Rollback

The host (wasmi engine, store, instances, linear memories, fuel, chains) is allocated inside the engine's `GameScope`,
so all of it lives in the game arena / statics that `melee_engine::snapshot` saves and loads page by page. A rollback
restores mod state together with the game; mods have no save/load code (spec section 6). This relies on wasmi keeping
no state outside its heap objects and statics (true for 2.0: no threads, no OS handles). Rule for future work: never
create or grow the host outside a `GameScope`.

## Verified (2026-11-03, port/modhost2)

| check | result |
|---|---|
| mods off, `cpu_fox_falco_dl` vs golden (all generic sites compiled in) | 3601/3601 frames |
| hello + moves + world loaded, two runs | byte-identical NHV1 CSV (3602 rows) |
| effect | first differing row 119; the logs show every row firing: input reads, jumps (+10% vy), replaced squats, landings, 94 item spawns, 118 item state changes, 9 live items, a stage update every frame, `gd_sinf(1)` = 0.841471 |
| synctest depth 4, 600 frames, three mods | 2388 hash checks, 0 mismatches |
| loader refusals | as before, plus replace without the feature, replace on a handle row, a second replace on one address |
| unit: replace calling `gd_orig_call_void` twice (the original runs once), pre `GD_SKIP` on `stage.on_start`, unhooked rows stay off | pass |
| live `melee play` (retail menus, scripted pad, one VS match) with the three mods | loaded, hooks ran through title / CSS / SSS / match / results, exit 0 |

Not verified: the C build / melee-unlocked running the same mods (V3 needs a host there).

**Netplay**: `play_net::mods_identity` adds the wasm files (`file#fnv32`, sorted) to the handshake digest and its
description, so peers with different code mods are refused. The guest's refusal text lists both sides
(`melee_net::refusal_both_sides`, applied where the room shows it: "...host has: X; you have: Y"); the wire text stays
the C's (net_oracle).

## Example mods

`rust/mods/hello_mod` (a `no_std` cdylib, not a workspace member): the Rust twin of `docs/specs/examples/hello_mod.c`.
Post hook on damage: slot 0 takes 1% extra per hit, counted in a static; post hook on the frame: logs the count every
600 frames. Build flags (build_mods.sh): `--initial-memory=--max-memory=1048576`, `-zstack-size=16384`, `--export-table`,
`-simd128`.

`rust/mods/moves_mod` (`gd.moves`): input read on `fighter.proc00`, post on `action.Jump.enter`, replace on
`action.Squat.enter` through `gd_orig_call_void`, pre on `action.Landing.enter`, an action request at frame 900.
`rust/mods/world_mod` (`gd.world`): `item.spawn` pre / post with the spawn accessors, `item.change_state` post,
`stage.update` pre, `stage.on_start` post, an item walk and trig in the frame hook. build_mods.sh builds all three.

## Stage switching (port/stagesw, 2026-11-08)

Port of the C build's seamless stage switch (pc/gameworld/script_stage_slot_retail.inc, demo
pc/scripts/examples/stage_switch_demo): the live stage is torn down and the destination's complete retail init /
load / start runs mid-match, with no scene change.

| piece | where |
|---|---|
| native switch | `melee_modes::tu::stage_hop` (`check`, `switch_to`): reads the destination `GrXx.dat` from the disc into heap 0 at the switch, then the C's order through the ported helpers: `script_retail_items(1)` / `script_retail_destroy` (ownership ledger captured during the normal stage init), `Ground_StageSlotRetailInstall` (collision through `ScriptGame_StageSlotRetailCollision`), camera handover, `script_retail_carry`, `script_retail_items(0)` |
| imports | `gd_stage_grkind() -> i32`, `gd_stage_switch(grkind) -> i32` (0 queued, else the refusal: -1 not gameplay, -5 same, -9 fighter grabbed/thrown/respawning, -18 stage not one of FD/BF/YS/DL/FoD), `gd_stage_switch_result()` (1 pending, 0 done, < 0 refused at apply). The switch is applied after the `frame.run_procs` post chain, outside the procs |
| the mod | `rust/mods/stage_switch`: every 600 logic frames after `stage.on_start` (match start and each switch), the next of BF -> YS -> DL -> FoD -> FD after the live stage; -9 retries next frame |
| check | `tools/rust-port/modhost/stage_switch.sh` (two identical goldens with 5 switches; synctest depth 4 over 3601 frames with MELEE_SNAP_CHECK=1) |

Rollback: no state outside the arena/statics (the C kept a raw-bytes cache outside its snapshot and refused online;
here the file is re-read on resimulation). Verified: synctest 14392 hash checks, 0 mismatches, each switch resimulated
5 times; `play_pair.sh` 20 ms / 2 % loss with the mod on both peers: 3 switches, 398 rollbacks, 0 desyncs, replay equal.

Differences from the C mod: no logic freeze and no wipe/flash/morph cover or countdown indicator (a sim mod cannot
import `gd_ui`, and the presentation host has no read-only sim getters yet); the music does not change; no Lua script
geometry is carried (`script_stage.cap` is 0). Found on the way: Fountain's platform grounds carry `map_id` -1; the
C reads `map_gobjs[-1]`, the port panicked on the index (`Ground_801C4A08` now ignores an out-of-range id).

## `gd_hash_word`: sim mod state in the compared hash (Envoy M0 probe b)

Wasm linear memory lives in `GameScope`, so synctest's page hash already sees a local resimulation that differs; but
two peers compare only the session hash, and NHV1 (`neutral_row()[0]`) is the cross-engine format that leaves every
PC add-on out (fighter.c's NHV1 comment). `gd_hash_word(u32)` folds a word into the frame's mod word (`nh_Mix` steps,
in `HostData`, so it rolls back; cleared at frame begin). `melee_modhost::session_hash` mixes it into NHV1 word 0
only when some mod gave one this frame, as the C mixes `ScriptGame_BuildHashWord` only when nonzero: the netplay
session (`play_net`), the synctest (`netmatch`) and the LAB desync check (`lab.rs frame_hash`) compare it. NHV1, the
goldens, the oracles and the `.mrpl` tick hashes are unchanged. Probe mod: `rust/mods/hashword_probe` (C, clang).


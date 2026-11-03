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

| row | site | modes |
|---|---|---|
| `frame.run_procs` 0x80390CFC | `gm_801A4D34_tick` around `HSD_GObj_RunProcs` (`gmscene::MOD_FRAME`); post runs before `Script_FramePost`, i.e. before the NHV1 sample (OPEN 3 default) | pre `i32()`, post `void()` |
| `damage.fighter_take_damage` 0x8006CC7C | `Fighter_TakeDamage_8006CC7C` (`melee_fighter::hooks::MOD_TAKE_DAMAGE`); a pre `GD_SKIP` omits the original, the post chain still runs | pre `i32(fighter,f32)`, post `void(fighter,f32)` |

Replace (`GD_FEATURE_REPLACE`) is refused. Accessors implemented: `gd_abi_host_version`, `gd_match_frame`,
`gd_fighter_from_slot`, `gd_player_present`, `gd_fighter_{slot,kind,get_action_state}`, get/set `percent`, `facing`,
`pos`, `vel` (per axis), `gd_fighter_read/write` (the 80-byte view, mask POS/VEL/KB_VEL/FACING/PERCENT),
`gd_rand_u32/f32`, `gd_log`. Not yet: items, spawn, input, stage, stocks, trig (`gd_sinf...`, OPEN 1 says the game's
tables), `gd_orig_call_*`, `gd_set_return_*`, `gd_fighter_request_action_state`.

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

## Verified (2026-11-03, this branch)

| check | result |
|---|---|
| mods off, `cpu_fox_falco_dl` vs golden | 3601/3601 frames |
| hello_mod loaded, two runs | byte-identical NHV1 CSV (3602 rows) |
| hello_mod effect | first differing row 119 (slot 0's first damage + 1%); the mod's frame hook logs 6, 12 hits at frames 600, 1200 |
| synctest depth 4, 600 frames, mod loaded | 2388 hash checks, 0 mismatches |
| loader refusals (V6) | growable memory, abi_min, address not in table, replace, wrong signature, unwired row, start function, duplicate id, foreign import |

`pad_falcon_fox_fd` has no damage to slot 0, so the test uses the CPU scenario. Not verified: the C build / melee-unlocked
running the same mod (V3 needs a host there), netplay sessions carrying the mod list (OPEN 10; the FNV digest is logged
only), live play (`melee play`) with mods (it boots through the same `boot_host`, so it loads them, but no run was made).

## Example mod

`rust/mods/hello_mod` (a `no_std` cdylib, not a workspace member): the Rust twin of `docs/specs/examples/hello_mod.c`.
Post hook on damage: slot 0 takes 1% extra per hit, counted in a static; post hook on the frame: logs the count every
600 frames. Build flags (build_mods.sh): `--initial-memory=--max-memory=1048576`, `-zstack-size=16384`, `--export-table`,
`-simd128`.

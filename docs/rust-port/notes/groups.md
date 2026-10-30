# Port groups (fighter engine notes)

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 113-122 -->
## Group 3: fighter engine + ftCommon + pl (2026-10-21; detail: docs/rust-port/group-fighter.md)
- Native types with a GC offset record: `rust/melee_fighter/gen/gen_types.py` (libclang, ppc32 parse)
  -> `src/types/generated.rs`, 569 `#[repr(C)]` records each with a `GcLayout` table; Fighter 0x23EC.
- All 216 TUs compile as a mechanical native translation: `gen/fix_raw.py` over the raw c2rust
  output -> `src/c2r` (gitignored, 50 MB), `cargo build -p melee_fighter --features c2r`. Not linked/run.
- Hand ports verified bit-for-bit against the C (gen/cref/run.py compiles the game's own C natively,
  cut verbatim): NHV1 fighter hash, 33 ftcommon.c movement helpers, staling (plstale/plattack/ft_0881).
- Trap: `HitCapsule::hit_grabbed_victim_only` is a `:1` unioned with the owner pointer, i.e. the GC
  address MSB (1 for every heap pointer); see `melee_fighter::compat`. `ABS(-0.0)` is -0.0, not abs().

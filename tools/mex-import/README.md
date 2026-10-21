# mex-import (Rust port, platform track step 5/5)

Offline tool. Reads YOUR OWN vanilla Melee disc and an m-ex mod disc (Akaneia, ACE, or an
extracted Nucleus project), writes **gdmods** that the game mounts over the vanilla disc, and
inventories the mods' PowerPC code against `melee_types`' GameCube offset tables.

**Its output is disc-derived. Never commit it.** `--out` is refused when it is inside a git work
tree and git does not ignore it (the workspace's `_build/` is ignored).

```
cargo build --release -p mex_import            # from rust/
mex-import import --vanilla V.iso --mod M.iso --name akaneia --version 1.0.1 --out <mods dir>
              [--dry-run] [--no-split] [--no-code] [--enable id,id]
mex-import code   --mod M.iso [--vanilla V.iso] [--out DIR --name N]
mex-import tables --mod M.iso                  # MxDt.dat counts + per-fighter file claims
```
`--vanilla` defaults to `$GW_ISO_VANILLA`.

## gdmod = a mods/ folder entry

The format is the one the C build already mounts (`pc/platform/gw_mods.h`, `shim_dvd.c`):
`<out>/<id>/mod.json` + `<out>/<id>/files/<disc path>`. Point `MELEE_MODS_DIR` at `<out>`.
`import` is the Rust port of the workspace's `tools/mex_port/make_mod_from_disc.py` (delta: NEW
files and same-path files whose CONTENT differs) fused with `split_modpack.py` (one gdmod per
added fighter and per added stage from `MxDt.dat`, plus `<name>-base`; a file two entities claim
stays in the base). Beside the mods it writes `<name>.mex-import.json` (sha256 per file per mod),
`<name>.mex-code.json` and `<name>.mex-code.tsv` (plain files in the mods folder are not mounted).

## Code inventory

Every `ftFunction` / `grFunction` / `itFunction[n]` blob (m-ex `MEXFunction`) in the delta's
`.dat` files is relocated and analysed (`src/analyze.rs`): functions from the blob's debug
symbols, data tables (MotionState/ItemState tables inside the blob) set apart, a forward
register-type dataflow per function, and every D-form load/store classified by its base:
`Mapped` (a melee_types record + GC offset -> the native field, e.g. `Fighter.self_vel.y`,
`Fighter.anon_594`, `Fighter.bf_2101`), `ArrayElem` (past the record: candidate array element),
`Opaque` (through a pointer read from an untyped field such as `Fighter.dat_attrs`: a struct the
mod defines), `Unknown`, `Indexed` (lwzx...), `Stack`, `Global` (r2/r13 small data, lis/addi
constants). Coverage = Mapped / (all but Stack and Global).

Types are INFERRED; the seeds are assumptions, listed in `src/analyze.rs`: callback entry r3 =
the owner's HSD_GObj (override slots, functions whose address the blob takes), its user_data =
Fighter / Ground / Item by blob kind and its hsd_obj = HSD_JObj, pointer fields' declared types,
`T *` returns of vanilla functions (decomp headers + symbols.txt, `build.rs`), call arguments
across the blob's own functions. Nothing here was checked against a run of the code.

## Verified (2026-10-21, locally, on the user's discs; nothing committed)

- Akaneia 1.0.1 and ACE 2.0.0: every file of `import`'s output (all gdmods, including each
  `mod.json`) is byte-identical to `make_mod_from_disc.py` + `split_modpack.py --version ...`
  run on the same discs (sha256 of all 477 / 1168 files compared).
- Code inventory: Akaneia 78 blobs, 86k instructions, 59.5% of struct-offset accesses mapped to a
  native field; ACE 167 blobs, 112k instructions, 61.4%. Two ACE blobs fail to parse (empty
  `grFunction` in GrTDa.dat, a NULL-code `itFunction[2]` in PlNt.dat) and are reported.
- `cargo test -p mex_import` (unit: sha256 vectors, FST walk, slug/JSON/resolve parity, layout
  resolution, two synthetic PPC dataflow cases); `-- --ignored` partitions Akaneia's delta.

## Not done

- Mod PPC code is inventoried, not translated or executed natively (the C build interprets it).
- No m-ex DOL patch / Gecko code handling (the C build reimplements m-ex in C; see
  `docs/mods-packaging.md` in the workspace).
- Type inference is unproven per slot; `ArrayElem` mappings are candidates; character-private
  attribute structs (`dat_attrs`) have no vanilla record, so they cannot map to a native field.
- No nod: plain GCM images and extracted trees only (no RVZ/WIA/CISO).

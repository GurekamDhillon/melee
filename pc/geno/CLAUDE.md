# pc/geno: the Geno fighter-extension engine and the LAB

`docs/geno.md` is the reference: sections 1-13 the engine, 14 the LAB, 15-18 the stable script
encodings the Meta Knight translator (workspace `ports/halberd`) emits against. `geno.h` is the
contract between the two halves and holds only `#define`s and enums.

## Two halves

| half | file | compiled as |
|---|---|---|
| native | `pc/platform/geno_registry.c` | x86: reads `geno.json`, the registry, stable ids |
| game | `geno_game.c` (+ `_glide.inc`, `_specials.inc`, `_v2.inc`), `geno_lab_mode.c` | through gwtool: sees guest structs in their byte order |

They talk through scalar functions only. A layout change (a struct both halves see) is a hot-reload
"layout change" and restarts the match (`docs/geno.md` 14.10).

## Contracts that must hold

- **Opt-in per fighter.** A fighter no `geno.json` names runs the code it ran before. The escape
  opcode (59) is the only global change, and no shipped script uses it
  (`tools/scan_ftcmd_opcodes.py` is the census; re-run it if the claim is questioned).
- **Numbers in sections 15-18 never change.** New behaviour gets new sub / value / condition ids.
- **Deterministic and rollback-safe** (section 4): no native state the snapshot misses; the stable
  ids are salted by `GENO_ID_VERSION`, which v2/v3 did not bump.
- m-ex stays untouched: Geno layers on `gw_mex_*`, never edits it.

## Tests

`geno_tests.c` (headless, `run.sh --test`): registry, ids, the escape, the LAB mode and rules,
mismatch naming. Add a test with a feature; name it `geno_<what>`.

## The LAB

The Lab mod is `mods/geno-lab/` (its own `CLAUDE.md`). Its game half is
`pc/gameworld/script_game.c` (`ScriptGame_Lab*`) with field numbers in `script_lab.h`; its native
half is the `l_*` functions in `pc/platform/gw_script.c` registered in `gs_gd_funcs`. A new read
is: an enum in `script_lab.h`, a case in `script_game.c`, a `gs_set*` in `gw_script.c`. No bridge
list to edit: the getters are already exported.

`tools/`: `lab_stage_d_check.lua` (the Lab's logic against a stub `gd`, any Lua 5.4: run it after
every `lab.lua` change), `framedata_diff.py`, `gen_motion_names.py` (regenerates
`pc/platform/gw_motion_names.inc` from the decomp's enums), `parity.sh` (replay parity; the real
replay list is the git-ignored `parity.local.conf`).

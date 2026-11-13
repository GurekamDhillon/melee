# Per-clip animation overrides (`anim_overrides`), Rust only

With an override, a mod can replace one fighter animation clip without shipping `Pl??AJ.dat` and without editing
the offset or size words in `Pl??.dat`. Code: `rust/melee_fighter/src/anim_override.rs` (table, validation, rules),
`ftdata.rs` `load_figatree` / `override_for` / `load_override` (binding), `rust/melee_data/src/modreg.rs` (manifest),
`rust/melee/src/mods.rs` `Mods::anim_overrides` (loading), `rust/melee/src/anim_hot.rs` (hot reload).

## Manifest
```json
{ "name": "Marth new smash", "version": "1", "kind": "misc",
  "anim_overrides": [
    { "fighter": "marth", "clip": "PlyMars5K_Share_ACTION_AttackS4_figatree", "file": "anim/fsmash.dat" },
    { "fighter": "marth", "clip": { "row": 55 }, "file": "anim/ftilt.dat" }
  ] }
```
- `fighter`: the scenario names (`marth`, `falcon`, `ics`/`popo`, `nana`, `gw`, ...; `anim_override::kind_by_name`).
- `clip`: a FigaTree symbol name, which replaces EVERY motion row that plays that clip (and demo / result rows with that symbol),
  or `{"row": N}`, which replaces only row N of the fighter's motion table (`pose-dump` `index.json` lists the rows and symbols).
- `file`: a path inside the mod folder (next to mod.json; `..` and absolute paths are refused). Keep it outside `files/`:
  anything under the payload folder is also mounted as a disc path (harmless, but pointless).
- The file is ONE standalone HSD archive holding the FigaTree (the bytes a bank slice would hold: big-endian archive header).

## Rules
- Later wins: entries are searched from the last-mounted mod back, and within a mod from the last entry back. So the
  last matching entry is used, whether it is a symbol or a row entry.
- Public symbol: with exactly one public symbol in the file, it is bound whatever its name (rename freely). With several,
  the one named like the `clip` symbol is used. Anything else is an error.
- Ice Climbers: when Nana plays Popo's rows (outside demos, as in retail) the rows are Popo's, so `popo`/`ics` entries apply
  to her too. `nana` entries apply to rows in Nana's own table, and they come first. Other kinds own their tables. Geno-defined
  fighters use the define's kind and rows (not verified).
- Errors are raised at load, before the match: an unknown fighter, a bad entry, a missing or unreadable file, a bad archive
  (header size, tables, relocations, no or ambiguous public symbol), or a file over the buffer. The error names the mod and
  the file, and the boot stops (`melee: ...`) instead of failing in a match.
- Limit: a clip must fit the fighter's fixed animation buffer, `FT_ANIM_BUF_SIZE` = 131072 bytes (0x20000). A larger file
  is rejected with its size and the limit. The buffer is not grown: unmodded allocation stays bit-identical.
  The demo / result scenes (`ftdemo_obj_alloc_init`) size the same buffers at 0xB000 (45056) bytes: an override larger than
  that is not bound in those scenes (logged, no animation) but plays in matches. Nothing else assumes the size (searched:
  `FT_ANIM_BUF_SIZE`, `fighter_x59C_alloc_data`, `Fighter_x59C_t`, `HSD_ObjAlloc_fighter_x59C`, literal 0x20000).
- Bank rows over the buffer (a modded `Pl??AJ.dat`) used to panic ("fighter figatree over!"); they are now logged
  (kind, row, size, limit) and left without animation. Retail banks never reach it.
- Netplay identity: each entry adds `anim_override:<fighter>/<clip>=<file>` plus the file size to the mod's digest, so
  peers with different overrides do not match.
- Scripts: only the animation is replaced. The row's subaction script (hitbox timing, sounds) stays the one in `Pl??.dat`.
  Retimed scripts still need a patched `Pl??.dat` (script overrides are a later item).
- Without any override installed, `load_figatree` makes one test of a static and then runs exactly as before.

## Hot reload (offline LAB only)
While the LAB runs offline, the engine checks the override files every 30 frames. When one changes on disk (size or modified
time), it re-reads them all, then restarts every fighter's current motion (frame 0, no blend) so the new clip plays right away.
Save the file from your exporter and the change shows in about half a second. The reload is all or nothing: a broken file
keeps the old data and logs `anim override <mod>/<file>: ...; reload refused, previous data kept`.
- `MELEE_ANIM_HOTRELOAD=0` turns the check off.
- Refused online, in replays, while spectating, in synctest and outside the LAB (`anim_hot::Ctx`). It never runs on
  normal play paths.
- Proof / debugging: `MELEE_LAB_TRACE=1 MELEE_LAB_TRACE_JOINT=j` logs P1's motion, row, frame and joint j's world
  position every LAB frame (`labjoint` lines).
- Engine call for a LAB button: `melee::anim_hot::reload_now(Ctx)` (no wasm import is bound yet).
- Memory: a reload replaces each entry's byte buffer and parses into the existing per-fighter buffer, so repeated reloads do
  not grow memory (unit test `reload_replaces_and_does_not_grow`, and `MELEE_ANIM_RELOAD_STRESS=N` runs N reloads per
  trigger and logs the held bytes).

## Checking a clip
`melee pose-dump --fighter marth --row 62 --mods DIR --out OUT` dumps the clip as the engine plays it with the mods mounted.

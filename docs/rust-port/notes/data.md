# Data: DAT reader, mex-import, saves/settings/mods/replays

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 251-278 -->
## Step 3: DAT/HSD archive reader (done, synthetic-tested)
- `melee_runtime::dat`: `DatArchive::parse` (port of `HSD_ArchiveParse`: header, data block,
  relocation table, public roots, externs, symbol strings; the C's file_size "byte-order
  mismatch" check; bad relocations rejected), `public_offset` / `extern_name`
  (`HSD_ArchiveGetPublicAddress` / `HSD_ArchiveGetExtern`), `load::<T>(root)` -> `Loaded<T>`
  (native root + `GcArena` owning every reached object + issue list). `DatBuilder` writes
  synthetic archives. The file is never patched in place (the C's `Locate`).
- `melee_core::gcdecode`: `GcDecode` / `GcPointee` + `GcDecoder`. Generated for all 1618 records
  by gen_types.py: each field decoded at its GC offset (scalars byte-swapped, bitfield bytes
  copied as-is, nested records recursive, pointers followed only through the relocation set,
  each (offset, type) decoded once, so cycles share objects; a work queue, not recursion).
- Unions: `HSD_Joint.u` (JOBJ_SPLINE / JOBJ_PTCL / dobj) and `HSD_RObjDesc.u` (ROBJ_TYPE_MASK) are
  hand-written in `melee_types/src/decode.rs` (gen `HAND_DECODE`). Other unions: pointer member
  if the first word is relocated, else first scalar member (heuristic).
- Limits (left zero + reported in `issues`): `void*` targets (e.g. HSD_ImageDesc pixel data,
  display lists: need a typed hand decoder), function pointers, and "pointer to first element"
  arrays (only element 0 decodes; `GcDecoder::array(target, n)` when the count is known, e.g.
  `MtxPtr` = `f32 (*)[4]` decodes one row).
- Tests: `cargo test -p melee_runtime --test dat` (5 synthetic: header/tables, HSD_Joint graph
  with cycle/strings/dobj/matrix, shared decoder, malformed files, unrelocated word).
  Opt-in disc check, run locally on the user's vanilla ISO (nothing written or committed):
  `GW_ISO_VANILLA=... cargo test -p melee_runtime --test dat -- --ignored --nocapture`:
  PlFxNr.dat/PlCaNr.dat parse, joint trees decode (Fox 73 joints / 1300 objects, Falcon 63 /
  1290), no out-of-bounds; ~350 issues each, mostly void* image data and words that some
  pointer-to-array or mis-typed pointer reaches as the wrong type (e.g. an HSD_ExpDesc reached
  as an HSD_Joint). Values were not compared against the C loader's output: no C-vs-Rust oracle
  for loaded data yet. GrNBa/GrNLa parse but have no `*_joint` roots (stage roots are map_head etc).


<!-- RUST_PORT.md lines 418-431 -->
## Platform track, step 5: mex-import (2026-10-21)
- `tools/mex-import` (workspace member `../tools/mex-import`, bin `mex-import`; README there).
  `import --vanilla V --mod M --name N --out DIR` writes gdmods = the mods-folder entries the C
  build mounts (`gw_mods.h`: `<id>/mod.json` + `<id>/files/<disc path>`), one per m-ex fighter and
  stage plus `<name>-base`, and `<name>.mex-code.{json,tsv}`: every m-ex code blob's loads/stores
  mapped to native field paths (`Fighter.self_vel.y`, `Fighter.bf_2101`, `Fighter.anon_594`).
- Verified locally on the user's discs (nothing disc-derived committed; `--out` refuses a
  non-ignored path inside a git tree): Akaneia and ACE outputs byte-identical to the workspace's
  Python packer+splitter. Code coverage: 59.5% / 61.4% of non-stack, non-global accesses map to a
  native field; the rest are Opaque (pointers from `void *` fields: `dat_attrs`, other GObjs'
  user_data), Unknown, indexed, or ArrayElem candidates. Types are inferred from assumed callback
  seeds (r3 = owner GObj), not proven per slot.
- Not done: translating or running mod PPC natively, DOL/Gecko patches, RVZ/WIA images.


<!-- RUST_PORT.md lines 1470-1487 -->
## Platform track: saves, settings, mods, replays (melee_data), 2026-10-24
- `rust/melee_data` (no game logic): memory card GCI folder + CARD API (vs aurora_card), settings.cfg (vs gw_settings.c table code), mods registry/overlay (vs gw_mods.c), `.slp` reader, replay Player/Recorder and `.gdlp` container (vs gw_replay.c / gw_script_rec.inc), and now the logger and crash report (`crashlog.rs`, vs gw_log.c). Every module has an oracle that cuts the shipping C out at build time (`oracle/*.sh`) and a Rust test over its transcript.
- Crash log (new): `oracle/log_oracle.sh` drives gw_log.c's own `gl_*` code (sink replaced) with a 9000-operation script (categories, MELEE_LOG / settings `log=`, flood limit, repeat counting, OSReport pieces, disc/user redaction, scene/pad/watchdog/disc/mods noting, RESET = a fresh process); `tests/log_oracle.rs` matches every written line and the noted state (6940+ lines) and 56 compact crash reports (`gl_write_report`: header, fault section, map symbolization with 500+ resolved frames, rings, 64 KB cap) by length and FNV-64 (the `os:` and `build id:` lines are machine-specific and left out of the hash). Not ported: the Win32 stack scan and exception filter, which are platform plumbing.
- Replays in the native sim: `melee golden --slp FILE` loads a `.slp` with `melee_data::player::Player` (stage, fighters, seed from the file), then per frame does what gmscene.c / fighter.c do: `Replay_Tick`, the forced seed (`MELEE_SLP_RESYNC=1` rule), and the recorded stick/trigger/button words written into the fighter at the input step (`input::fighter_input`, `VsMatch::replay_in`). Proof: the C build recorded Falcon v Fox on FD (`tools/rust-port/golden/record_slp.sh`, `MELEE_SLP_RECORD`; the recording run equals `pad_falcon_fox_fd.csv` on all 1801 frames), played the .slp back itself (`MELEE_SLP`, resync), and the Rust sim's CSV equals the C playback on every fighter/item word of 1801 frames (`check_slp.sh`; `compare_golden.py --ignore-seed`: the seed column differs where the port draws different effect RNG, and replays reseed every frame). Negative control: the same file with the buttons zeroed from frame 100 diverges at frame 242. Files: `docs/rust-port/golden/slp/`.
- `.gdlp` (round 2): `record_gdlp.sh` made the C record Falcon v Fox on FD with `MELEE_GDLP_RECORD` (its recording run = `pad_falcon_fox_fd.csv`, 1801 frames) and play it back with `MELEE_GDLP` (identity matched; 1801 rows identical to the golden and to the `.slp` playback). `melee golden --slp x.gdlp` (a `.gdlp` is a valid `.slp` stream) equals the C's `MELEE_GDLP` playback on all 1801 frames; `check_slp.sh` runs it. Files: `slp/pad_falcon_fox_fd.gdlp`, `.gdlp.play.csv`. Not done: the Rust player does not run `gdlp::check_ident` or restore the embedded state chunk.
- Marth v Sheik on BF: the Rust sim from the C-recorded file equals the recording on its 162 playable frames (it stops at an unported move), but the C's own playback of that file differs from its recording from frame 0 (a C-side RNG/setup effect, not investigated), so there is no C playback golden for it; the files are committed for when it is.
- Windows used by the C runs are parked at 30000,30000 with `SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0` and no OS input is sent anywhere (`MELEE_WINDOW_HIDE=1` hangs the turbo watchdog on a cold shader cache).
- Mods in `melee` now run on `melee_data::modreg` (ModSet + Overlay, oracle-checked against gw_mods.c) instead of the earlier hand-rolled resolver (`melee/src/mods.rs` is an adapter; `ModSet::load_with` takes `--enable`). Not re-run against the Akaneia Wolf golden this session.
- Not done: `melee` does not use melee_data for settings/card yet; UCF toggles (`Replay_UcfCardinals` and the ftUcf code) are off in the playback (the golden replay has them off); `.gdlp` identity check (build/disc/mods/Geno) in the Rust player; m-ex runtime dispatch from a replay's mods; CPU players in a replay; stages and fighters beyond FD/BF and Falcon/Fox/Marth/Sheik.
- Round 3: `melee golden --gdlp FILE` runs the identity gate (`rust/melee/src/ident.rs` = `gs_identity` for this binary: exe hash, disc 4 MiB hash ^ size, mods describe, Geno seed) via `Player::load_gdlp`; the C-recorded golden `.gdlp` is correctly refused ("recorded by another build of the game", as the C refuses another exe's recording). `melee_data/tests/gdlp_ident.rs` proves the gate accepts a matching identity and refuses each of build / disc / mods / Geno with the C's text, and that the Player plays only on a match. `melee` now reads `skip_intro`, `audio`, `mods`, `mods_dir` through `melee_data::settings` (env `MELEE_<KEY>` > settings.cfg > default; `rust/melee/src/cfg.rs`). Still not done: embedded-state restore, a Rust-side `.gdlp` recorder in the bin, memory card use by the bin, UCF in playback, m-ex dispatch from replay mods, CPU players, Marth v Sheik C playback golden.

- Round 4 (plat-play, re-verify after merging port/platform-integ): full_loop.txt (1359 ticks, to results, time-up draw) and
  pause_quit_rematch.txt (1563 ticks, two matches) re-run hands-off with MELEE_PAD_IGNORE_ADAPTER=1, unfocused window, `--shots`; all
  screens captured (build/r2, disc art, not committed). Still stand-ins: ui.rs menus, 7 fighters, 4 stages, sim halts on unported fighter states.

- Round 5 (plat-play): vs_to_ko.txt --golden-pads re-run hands-off (MELEE_PAD_IGNORE_ADAPTER=1, unfocused window, --shots): title/CSS/match/live/results
  reached, 1875 ticks, time-up on Final Destination (port 2 wins, falls 5-0 on the timer). No OS-level input was sent. Menus are still ui.rs stand-ins.

## 2026-11-03 port/saves: the memory card in `melee` (Rust writes, C reads, and back)
- `melee_engine::card` = shim_card.c over `melee_data::card`; installed by the retail flow only (golden/synctest/scenario runs stay save-less). `MELEE_CARD_PATH` (folder; create it first: Aurora in the C build drops the last component of a missing folder and its C++ code throws), default `card/` beside the exe; `MELEE_CARD=0` off; `MELEE_SKIP_MEMCARD=1`; `MELEE_CARD_DIAG=1`.
- Save files stay GameCube bytes on the card: `lbcardgame::GC_IMAGE` stages GmSaveData + 7 NameTagDataBanks through the GcDecode byte permutation (`tests/save_image.rs`). Log lines `gw: card: load result N (per-file errors ...)` and `save image -> native: ...` are what tests check.
- `tools/rust-port/saves/roundtrip.sh` (C_EXE = a C build with the try/catch in aurora OSTime.cpp: older C exes die in chrono::current_zone under the "British Columbia Standard Time" zone): all checks pass, 2026-11-03.
- Known: tu_lbcardgame oracle fails on the base commit too (op 10 case 30); VS records count only human results (gm_SetupHumanResultsScreen), so CPU-only scripted matches leave them 0.

# Push audit: tracked test data and binary art on port/push-prep (from rust-port bcc8e1e9e)

Read-only audit; nothing was deleted. Question per group: does it hold disc-derived content
(values read from the ISO: model/texture/stage/collision/text/audio data, archive bytes,
decoded assets, game screenshots) or only values computed by code from synthetic inputs?

Method: for every generator/oracle under `rust/*/gen`, `rust/*/oracle*`, `rust/*/cref`,
`rust/*/tests/c_oracle`, `tools/mex-import/oracle`, `tools/rust-port` I grepped for ISO/.env/GW_ISO,
`fopen`/`open` of inputs, `.dat`/`.usd`/`GALE01` paths: none of the generators that write
`tests/data` read the disc. They compile the public decomp C (`src/`, `pc/`) with a seeded RNG or
a scripted scenario and print state hashes/words. Several headers say "computed from C source
only: no disc data". Contents were sampled (see "Spot checks"). Images were viewed.

## 1. rust/*/tests/data (505.7 MB in 979 files; 451 files > 200 KB = 477.2 MB)

All rows in this table: verdict **safe** (computed from code, synthetic inputs).

| crate (size; biggest files) | >200KB total | generator | notes |
|---|---|---|---|
| melee_world (125.9 MB; textlib_r6_ref 22.4, stage_ref 14.2, camera*_ref 16.2, mpisland_ref 8.4, mplib_*_ref ~19, grlib_ref, ground*_ref, leaves*_ref, tu_gr*, mtu_*) | ~124 MB | `rust/melee_world/cref/gen_*_ref.sh` + `*_ref.c`; `gen/mtu/run_tu.py`; `gen/oracle/gr_oracle.py`; `oracle/tuo` | random byte strings / floats (textlib "S" rows are random char codes, not game text; stage_ref "G" rows are random floats, not stage collision) |
| melee_fighter (118.5 MB; player*.txt 12.2, tu_ftCo_*, ft*.txt, stale, tu_fighter*) | ~113 MB | `rust/melee_fighter/gen/cref/run.py`, `cref/tu/run_tu.py`, `tu_shard*.py` (`*_ref.c` drivers) | states filled by `gen/mtu/fill.py` pick_* from a seed |
| melee_runtime (94.4 MB; oracle_hsd/anim/state_tev/lobj/cobj/texp/jpeg, mtu_*, lbtu_*, pure_*) | ~90 MB | `rust/melee_runtime/oracle/build_*.sh`, `driver_*.c`, `build_state_tev.py`; `gen/{mtu,lbtu,pure,sis_sobj}/run_*.py` | synthetic HSD objects built by the drivers; oracle_anim bytes are generated keyframe streams |
| melee_modes (53.8 MB; tu_gm_*, vectors*, s497_*, tuo_*, mtu_*) | ~50 MB | `rust/melee_modes/oracle_tu/tuoracle.py`, `oracle/build*.sh`, `oracle/s497/build.sh`, `oracle/tuo/build.sh`, `gen/cref/run_tu.py` | random vectors, FNV hashes |
| melee_items (58.2 MB; mtu_*, native_cobj_eye) | ~49 MB | `rust/melee_items/gen/{kinds,mtu,mtu649,mtu_it,mtu_it2}/run_*.py`, `gen/native/build_cobj_eye.sh` | "computed from C source only" |
| melee_data (25.7 MB; card_scenarios 16.5, settings_oracle 4.9, card_oracle, log_*, mods_*, replay_oracle) | ~24.6 MB | `rust/melee_data/oracle/{card_oracle.py,*_oracle.sh,*_scenarios.py,log_script.py,gen_slp.py}` | memcard/settings/log scripts with fabricated names; "PlWf.dat" etc. are only file-name strings |
| melee_chara (20.3 MB; tu_mpcoll 4.4, tu_*, oracle_*, chars.txt) | ~19 MB | `rust/melee_chara/gen/oracle/oracle.py` ("computed from C source only: no disc data"), `gen/turef/tu_*.py`, `gen/chref.py` | hashes |
| melee_audio (3.6 MB; ax_oracle 3.3) | 3.2 MB | `rust/melee_audio/oracle/build.sh` (oracle.c) | random PCM/ADPCM words from the AX shim |
| melee_net 1.7, melee_gx 1.6 (shader/shim/state/tex oracle), mex-import 2.0 (ppc_oracle: random PPC opcodes) | ~5 MB | `rust/melee_net/oracle/*.sh`, `rust/melee_gx/oracle/build.py`, `tools/mex-import/oracle/ppc_oracle.c` | random inputs |

Subtotal: ~477 MB safe, 0 MB disc-derived in this class. Files under 200 KB (28.5 MB) come from the
same generators.

Weak points inside this class (flagged unsure, small):

| file | size | generator | verdict | why |
|---|---|---|---|---|
| rust/melee_data/tests/data/replay_real_oracle.txt | 77 KB | `oracle/replay_oracle.sh` ($REAL replay list) | unsure | summaries of the author's own real .slp replays (not disc data; personal data) |
| rust/melee_data/tests/data/gdlp/*.gdlp (72 files) | 359 KB | recorded by the C game, damaged by `oracle/gdlp_variants.py` | unsure | whole-match input recordings from a game run on the retail disc; inputs/ids only, no assets seen |
| rust/melee_data/tests/data/slp/*.slp (99 files) | 586 KB | `oracle/gen_slp.py` | safe | synthetic |

## 2. Other tracked data / binary art

| file | size | generator | verdict | proposed fix |
|---|---|---|---|---|
| pc/assets_src/bf_platform/look/after-ingame-close.png | 948 KB | screenshot of the running game (Fox x2 on the platform, Fox model, HUD, retail stage background) | **disc-derived** | untrack and git-ignore; keep before/after of the custom texture only |
| pc/assets_src/bf_platform/look/after-ingame-wide.png | 854 KB | same | **disc-derived** | same |
| pc/docs/readme/match_hd.jpg | 45 KB | README screenshot (Fox/Marth, stage, HUD) | **disc-derived** | regenerate locally with menu/pipeline/readme.py from the user's ISO into a git-ignored path, or replace with art that has no game assets |
| pc/docs/readme/kit_hud.jpg | 34 KB | screenshot, Fox/Marth on stage | **disc-derived** | same |
| pc/docs/readme/css.png | 45 KB | character-select render with the roster portraits/stock icons | **disc-derived** | same |
| pc/docs/readme/lobby_strikes.png | 39 KB | stage-select thumbnails of retail stages | **disc-derived** | same |
| docs/rust-port/golden/slp/pad_*.slp, .gdlp, *.play.csv | 1.3 MB + 0.59 MB + ~5 MB csv | `tools/rust-port/golden/record_slp.sh`, `record_gdlp.sh` (game run with GW_ISO_VANILLA) | unsure | replays/state traces, no asset bytes seen; README says hashes only. Safest: keep only the hash CSVs (`golden/*.csv`) and regenerate .slp/.gdlp with `record_*.sh` locally into a git-ignored dir |
| docs/rust-port/golden/*.csv, docs/rust-port/trace/*.csv | ~4.5 MB | `tools/rust-port/golden/record.sh` | safe | per-frame hashes (heads checked) |
| pc/assets_src/bf_platform/look/{before,after,*.glow}.png, pc/scripts/examples/bf_platform/models/*.{png,gxtex} | ~3.1 MB | Blender pipeline for original platform art | safe | after.png viewed: original texture atlas |
| pc/assets_src/vk_concepts/** (reference renders, textures) | 2.2 MB | own Blender renders | safe | viewed |
| pc/geno/mods/geno-lab/art/preview/lab_sheet.png, geno-lab/ui/*.gxtex (83) | 0.9 + 0.2 MB | `art/lab_art.py` | safe | own UI art |
| pc/scripts/examples/demos/screen-models/*, envoy_drives/*, bench/*.gxtex | 0.7 MB | example scripts | safe (generator not verified; simple shapes by name) | |
| rust/melee_gx/tests/data/render_aurora.png, docs/rust-port/shots/gx_viewer_synthetic.png | 72 + 45 KB | `rust/melee_gx/oracle/build.py`, synthetic viewer | safe | random-colour test textures (viewed) |
| extern/aurora/assets/*.png, .github/assets/*.png, docs/img/*.png, pc/docs/readme/brand logos | ~1.3 MB | upstream (aurora, decomp tooling) | safe | not disc |
| config/GALE01/{symbols,splits}.txt (3 MB), docs/rust-port/compile_commands*.json | | upstream decomp tables / build metadata | safe | public in doldecomp/melee |

Not tracked on rust-port but present in history: rust/out_*, rust/r4*, docs/rust-port/shots/play_*
screenshots (untracked by 1cb9b4d93). See the scrub in part 3 (branch `port/platform-integ-clean`).

## 3. Totals

| verdict | files | size |
|---|---|---|
| safe | ~1000+ (all oracle tests/data, own art, tables) | ~495 MB |
| disc-derived | 6 | ~2.0 MB (all screenshots) |
| unsure | ~175 | ~7.3 MB (golden .slp/.gdlp/play csv, tests/data/gdlp, replay_real_oracle) |

## 4. Proposed fixes for the disc-derived set
1. `git rm --cached` the six screenshots; add them to .gitignore. The README art is rebuilt from the
   user's ISO by `menu/pipeline/readme.py` at build time (workspace repo).
2. Optionally move golden `.slp/.gdlp/*.play.csv` to a git-ignored `docs/rust-port/golden/local/`, produced by
   `tools/rust-port/golden/record_slp.sh` / `record_gdlp.sh`; keep the four hash CSVs.
3. Files already in history stay there until history is rewritten: rust-port itself still carries the
   screenshots listed above plus rust/out_*, rust/r4*, play_* (part 3 scrubs only platform-integ).
4. The 477 MB of synthetic oracle outputs need no removal for licensing; for size consider git LFS, or
   regenerating them locally/CI (all generators are in-tree).

## Spot checks done
Heads of textlib_r6_ref, stage_ref, mpisland_ref, mplib_joint_ref, camera_ref, player, card_scenarios,
settings_oracle, ax_oracle, oracle_hsd/anim/state_tev, tu_gm_1601, vectors_r10, tu_mpcoll, ppc_oracle:
all random/hashed words; no recognisable text, model or collision data. Not byte-scanned against the
ISO (no disc paths were read).

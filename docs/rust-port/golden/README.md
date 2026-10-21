# Golden corpus (C build = oracle)

Per-frame byte-order-neutral gameplay hashes (`MELEE_NEUTRAL_HASHLOG`, format NHV1,
`docs/rust-port/neutral-hash.md`). Hash CSVs only: no disc data. Recorded 2026-10-21 from
branch `rust-port` (build root `_build/agents/rustgold`), NTSC 1.02 vanilla disc, Windows x86.

| file | scenario | frames |
|---|---|---|
| `pad_falcon_fox_fd.csv` | VS, Falcon (P1) v Fox (P2), Final Destination, both driven by `pad_fixed.lua` | 0..1800 |
| `pad_marth_sheik_bf.csv` | VS, Marth v Sheik, Battlefield, `pad_fixed.lua` | 0..1800 |
| `cpu_fox_falco_dl.csv` | VS, Fox CPU9 v Falco CPU9, Dream Land, items off (lasers appear as items) | 0..3600 |
| `cpu_peach_samus_ys_items.csv` | VS, Peach CPU9 v Samus CPU9, Yoshi's Story, items freq 4 | 0..3600 |

Every scenario was run twice; both CSVs were identical before the first was copied here.
No replay scenario: there is no non-disc-derived `.slp` in the repo to use.

## Reproduce

```
export GW_MELEE="E:/Projects/Melee Workspace/worktrees/rust-port"
export GW_BUILD_ROOT="E:/Projects/Melee Workspace/_build/agents/rustgold"
bash "E:/Projects/Melee Workspace/tools/port/build.sh"
bash tools/rust-port/golden/record.sh            # all four, or name scenarios
```

`record.sh` holds the exact scene strings. Each run is, e.g.:

```
MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345 \
MELEE_TURBO=1 MELEE_PAD_SCRIPT=<worktree>/tools/rust-port/golden/pad_fixed.lua \
MELEE_SCENE='mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off' \
MELEE_NEUTRAL_HASHLOG=<out>.csv bash tools/port/run.sh --max-seconds 240 <sandbox> --iso "$GW_ISO_VANILLA"
```

Compare a candidate (the Rust port, or a rebuilt C exe):
`python tools/rust-port/compare_golden.py docs/rust-port/golden/<x>.csv <candidate>.csv`
(exit 0 = identical; it names the first diverging frame and column: `seed`, `items`, or slot `sN`).

## Known limits
- Goldens are tied to the C build's frame hook and the scene/pad path. A Rust frame driver must
  start counting `frame` at the same point (the script layer's match frame 0).
- Not yet cross-checked by a second implementation of NHV1 (no Rust struct reader exists yet).

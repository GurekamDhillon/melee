# Replay results (2026-11-21)

Full run of the Rust engine (`melee slpcheck`) over five Slippi replay sets, measured on rust-port
2026-11-21. A replay counts as a match when every frame of the engine's post-frame state equals the
data recorded in the `.slp` file. No replay names, hashes or player tags are kept in the repo; the
replay files are not committed.

| set | match | total | rate |
|---|---:|---:|---:|
| ext | 125 | 125 | 100% |
| hero | 426 | 428 | 99.5% |
| lunar | 348 | 351 | 99.1% |
| gang | 3,428 | 3,561 | 96.3% |
| lunar2 | 7,685 | 7,800 | 98.5% |
| **total** | **12,012** | **12,265** | **97.9%** |

Against the previous full run: 0 regressions, 132 replays newly matching.

## The 253 misses, by cluster

| cluster | count | class | summary |
|---|---:|---|---|
| A | 113 | recording artifact | Slippi 1.7.1 singles, late small drifts; the Dolphin oracle agrees with the engine on most reps (1.7.1 lacks the polling-drift fix) |
| B | 2 | recording artifact | 1.7.1, one stage, spawn-time x off by 1e-3; oracle matches the engine |
| C | 17 | unknown | 1.7.1 four-player teams, late drifts |
| D | 1 | engine | wrong spawn position with four Ice Climbers on Final Destination |
| E | 27 | engine | Slippi 3.20.0 only: a timer field, an RNG draw and Dream Land wind (E1 14, E2 7, E3 6) |
| F | 69 | engine | Slippi 3.19.1 float position off by a few ULPs, mostly early |
| G | 19 | unknown (lean engine) | Slippi 3.17.0 small facing / percent / position diffs; stick processing suspected |
| H | 5 | unknown | single replays with large diffs |

Totals: engine bug 97 (D 1, E 27, F 69); recording artifact 115 (A 113, B 2); unknown 41 (C 17, G 19, H 5).

Miss rate by Slippi version: 1.7.1 3.7%, 3.17.0 0.4%, 3.19.1 2.3%, 3.20.0 25% (27 of 106).

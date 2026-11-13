# Envoy: what "production ready" means, and where it stands (2026-11-13, port/envoy-ready1)

Audit table (every finding, how to reproduce, severity): `scratch/envoy-ready1/audit-table.md` (local, not committed).
Checks for tonight's fixes: `tools/rust-port/envoy/ready_checks.sh` (rows quit, seed, acecss; each fails on
melee-latest6 and passes on this branch).

## The checklist

| line | the bar | status after tonight |
|---|---|---|
| Stability | no crash, hang or soft-lock in 30+ min of turbo play, solo and co-op, both supported discs | met on vanilla and ACE in four 170k-tick runs, after the ACE character-select crash fix. Memory grows ~10 MB per fight (open, below) |
| Determinism | synctest 0 mismatches at depths 4 and 7; vanilla play bit-identical | integrator checks re-run on this branch (gate, torture, v2_drives, loop3); see the branch report |
| Progression | every way out of a fight does the expected thing to the run | met: win, loss, closing the game, and now pause-quit (keeps the run) |
| Variety | each run differs | met: runs are salted at New run (`MELEE_ENVOY_SEED=n` reproduces one). Wave-two repeats owned by envoy-fix1 |
| First run | a new player learns goal and buttons without outside help | partly: a "How to play" card in the camp. No in-fight tutorial prompt yet |
| Content | enough fighters, rooms, drives, bosses for a few evenings | the drive pool, 3 rooms + boss, endless loops; one boss (Petey, own lane), Subspace foes in their lane |
| Performance | no game-caused frame over 16 ms in any Envoy scene | not re-measured tonight (windowed profiler run still to do) |
| Online co-op | two machines | not started; local co-op only |
| Packaging | one launcher, no env vars needed | met for local play (play-*.cmd); not in the Qt launcher yet |

## What remains, in order

1. Memory growth per fight (~10 MB, linear; 0.7 GB to 2.15 GB over 141 fights): heap profile around match teardown. Medium.
2. Frame-time gate on Envoy scenes (camp, picks, shop, boss) with MELEE_PROFILE=1, windowed. Small to run, unknown to fix.
3. Merge the five overnight branches (fix1, petey2, subspace1, css1, ready1) and re-run the gate + Envoy checks. Small.
4. First-fight hint (one line on the HUD: "Hold Y to swap") for players who skip the card. Small.
5. Fighter coverage: every fighter as main and partner through room 1 (chain2b covered 7 pairs; the rest to run). Small.
6. Two-human co-op through the picker (owned by envoy-fix1). Small.
7. Envoy in the Qt launcher. Medium.
8. Online co-op (rollback over the network). Large.

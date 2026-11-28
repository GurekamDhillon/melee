# Regression replay set (2026-11-27)

A fixed list of ~1,550 replays to run after every engine change, about 15 minutes at 8 jobs, instead of the full
12,265-file corpus (~90 minutes, a third of it pre-3.0 files that cannot be scored). Fresh Lunar batches are for
finding new problems (a new Slippi release, a stage, a matchup); this set is for proving a fix and catching a
regression on the same files every time. New failures found in fresh batches get added.

- Build: `python -I tools/rust-port/replayset/regress.py` -> `scratch/replayset/regress.tsv` (not committed: it
  names replay files). Inputs: the full sweep's re-scored rows and the 3.20.0 batch's scores.
- Run: `LIST="E:/Projects/Melee Workspace/scratch/replayset/regress.tsv" MELEE_EXE=... JOBS=8
  bash tools/rust-port/slippi/hard_desync_sweep.sh OUT_DIR`, then compare OUT_DIR/rows.csv with the previous run.
- Contents (a replay can be in several groups): every replay that ever failed (272); all of the owner's online
  games (hero, 428); the external set (125); every 3.20.0 replay (236); high risk, capped per tag: characters with
  a second entity or transform, random specials or items, unusual hurtboxes (375), stages with randomness (120)
  and every non-tournament stage (29), more than two players (44), teams (30), the longest games (30); a baseline
  of up to 8 matching games per Slippi version x stage (210); 60 pre-3.0 games for the old format paths.

## Replay tiers (owner, 2026-11-27)

- **Tier 1, must match exactly:** Slippi 3.x replays (current era 3.17-3.20 first) and every replay recorded from now
  on, by Slippi or by us. Sweep counts, the endgame score and fix work count tier 1 only.
- **Tier 2, pinned:** pre-3.0 replays (the scorer's `old`: Post-Frame taken mid-frame, not exactly checkable) and odd
  old Gecko variants. They stay in the set as "must not get worse": a tier-2 replay that changes from match to
  diverged still blocks a merge, but a tier-2 divergence is not opened as fix work unless the owner asks.
- A row counts as hard only when `first_hard` is numeric (`old`, `gap`, `error` are scorer markers; referee.md 10.4).
- Playback routing: pre-3.0 replays are candidates for the original-code player (Hero's static recomp) in his
  launcher; that routing is his launcher's side.

Scorer (`tools/rust-port/slippi/hard_desync.py`) results that are not desyncs: `old` (Slippi < 3.0: Post-Frame
taken mid-frame), `gap` (the replay is missing frames, e.g. lunar2 ...6325d9385: 5267-8149), `error` (unreadable).
It scores only finalized frames (Frame Bookend) and compares like with like for Zelda/Sheik and a dead Nana.

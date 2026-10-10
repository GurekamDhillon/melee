
## Lane handoff (stress6, 2026-10-10 12:36)
Branch agent/stress6 (game worktree E:/Projects/Melee Workspace/worktrees/stress6, build root _build/agents/stress6). Commits: 73cf50448 (GENO_MAX_SLOTS 2560, GENO_POOL_WORDS 65536, shared by registry+game half), 42be33a14 (budget log shows slots, test geno_define_six_full). Suite 368 pass 0 fail (_build/agents/stress6/test3.out).
- R1 inventory: done in head only, needs writing: slots (geno_registry.c GN_MAX_SLOTS, geno_game.c Geno_OverlayOrig), pool words, JDOC_NODES per doc (ok), GPL_MAX_CLIPS per plan (ok), profile arena 2 MB (0 used), fighter heap 3 (stage heap) had only 139 KB free on FD with 6 Geno Falcons (six-slot admission, lbdvd.c:24-70), snapshot globals grew ~205 KB (pool+OverlayOrig) - netplay cost unmeasured.
- R2 done (above). R3: only captain has an engine template (geno.md 22.11), so 6 DIFFERENT retail fighters is not possible without new native templates; 6 distinct Geno Falcon defines (keys geno-falcon-1..6) built to _build/stress6/mods (313 overlays, 4806 words each).
- R5 smoke (turbo, FD, 1200 frames, 6 Geno CPUs) ran clean: log "define budgets ... all profiles 30714/65536"; realtime perf line frame_work mean 6.38 p95 8.34 p99 14.89 (turbo-render run, not a realtime measure). Realtime run, 10+ min turbo soak, screenshots, jev runparse/triage: OWED.
- R4 sweep (2-CPU fox v falco, uncapped, bench.py, _build/stress6/sweep.csv; vanilla ext 2-60 DONE, ACE ext 0-~72 of 371 done, killed at ext 73). Top by mean frame_work:
  VANILLA ext 2: frame_work mean 7.5967 ms, p95 9.6068, draws/frame 2501.4
  ACE ext 71: frame_work mean 5.8589 ms, p95 8.6225, draws/frame 526.1
  ACE ext 2: frame_work mean 5.7987 ms, p95 7.2654, draws/frame 2510.8
  ACE ext 22: frame_work mean 5.7521 ms, p95 7.6112, draws/frame 593.0
  VANILLA ext 22: frame_work mean 5.6423 ms, p95 7.1791, draws/frame 588.8
  VANILLA ext 45: frame_work mean 5.5417 ms, p95 9.3395, draws/frame 339.0
  VANILLA ext 3: frame_work mean 5.5082 ms, p95 7.9239, draws/frame 415.4
  VANILLA ext 8: frame_work mean 5.356 ms, p95 7.5918, draws/frame 449.7
  ACE ext 63: frame_work mean 5.3143 ms, p95 7.7704, draws/frame 507.6
  ACE ext 59: frame_work mean 5.285 ms, p95 7.8852, draws/frame 550.4
  (2 = Fountain-type heavy draws 2500/frame; errors on vanilla ext 21,26 etc = stage not loadable.)
Next: (1) resume sweep: bash _build/stress6/sweep_all.sh (resumable; vanilla part skips done rows; edit ACE ids to start at 73); (2) 6-Geno run on top stages: bash _build/stress6/run6.sh NAME DUR STAGE realtime|turbo [VANILLA|ACE]; scene uses ext/int via STAGE e.g. ext:71; check six-slot heap 3 admission lines; (3) soak 36000 frames turbo; (4) runparse/triage/check_claims/coverage; (5) agent_slim.py.
Traps: gw_mex_bridge.c is regenerated build output, left uncommitted on purpose; no processes of mine left running (killed PIDs 22132 34944 21036 46204 23764 41932).

# Codex packet: enlarge stage heap 3 for six-fighter matches (game repo, this worktree only)
Your shell is broken: edit files only; Claude builds and tests. Do not commit.
Context: stress test (branch agent/stress6) with 6 Geno Falcon CPUs: after load, heap 3 free = 139,856 B of 5,351,056 on
Fountain of Dreams but only 9,520 B on ACE ext stage 121. Owner decision: pre-enlarge, future-proof; do not gate on a sweep.
Heap sizes on load (log): heap 0 free ~7.2-7.7 MB, heap 4 ~18.4-18.9 MB, heap 5 25.0 MB.
Task:
1. Find where the PC port sizes the HSD/lb heaps (heap 3 is the stage/fighter-load heap; look under src/ and pc/ for the
   heap creation and any TARGET_PC overrides; grep the log text that prints per-heap free).
2. Under `#if defined(TARGET_PC)` (keep the original path), grow heap 3 so six full Geno fighters on the heaviest stage keep
   >= 4 MB free. Prefer taking the space from the existing PC arena / adding to the total rather than shrinking heaps 0/4/5.
   Check every place that assumes the old size (address range checks, savestate/rollback snapshot sizes, netplay snapshot
   hashing, gw_mex bridge); list them in the report.
3. Add a headless test (follow geno_define_six_full) asserting heap 3 free after a 6-fighter load is >= 4 MB.
Report to .omo/evidence/report-heap3.md: files changed, the new sizes and why, every dependent you checked, exact commands
for Claude: build.sh with the stress6 build root, the headless suite (MELEE_TURBO=1), and a 6-CPU run on ACE ext 121.

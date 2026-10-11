# modsync handoff (lead, 2026-10-10)
Branch agent/netplay-mod-sync. Loopback driver: pc/tests/modsync/np_modsync.py (a b c d). Run: GW_MELEE/GW_BUILD_ROOT exported, MELEE_NETPLAY_BIND=127.0.0.1 MELEE_TURBO=1 windows offscreen, one scenario per call (~1-3 min each).
Results on build 33b691635c59d909 (native tests 386/386): a PASS (8 checks), b PASS (7). c and d PASS on the previous build (before the wait-screen fix), not re-run since.
Found and fixed this session:
- host stranded on the waiting screen if the guest left during the 45-frame "found" beat (a guest restarting to match mods does exactly that): gmfrontend_online.inc fl_frame_wait now rejoins the room.
- the supervisor (tools/port/runs.py) runs the game in a kill-on-close job, so the game's own CreateProcess relaunch dies with run.sh. The game now tries CREATE_BREAKAWAY_FROM_JOB first; the test driver sets MELEE_MODSYNC_NO_RELAUNCH=1 and does the restart itself (same env, new boot). The real CreateProcess relaunch is NOT proven by this driver (owed: the launcher/exe started by hand).
- a script mod does not change the global hash, so scenario b is "listed + sync offered", not "blocked".
- ne_refetch_mod logs HTTP status + error when it fails (first-run failure of ra1 was against the main-checkout run.sh; not reproduced).
Room UI: Atlas lobby draws a "MODS DIFFER" panel (up to 5 rows + "+N more", gw_Netplay_ModsRow, Ui_RoomMod*), test in atlas_room_host_test.c. NOT seen on screen (owed). The legacy lobby still shows only the one-line status.
Workspace side branch ws/modsync-breakaway (GW_RUNS_BREAKAWAY in runs.py) is unused by the final driver; can be dropped.

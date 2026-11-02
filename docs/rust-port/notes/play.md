# `melee play`

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 1305-1328 -->
## Platform track: `melee play` (2026-10-24, port/plat-play)
- `melee play [--iso] [--time-secs N] [--shot-dir D] [--quit-at-results] [--skip-intro] [--golden-pads]` (rust/melee/src/play.rs, ui.rs):
  winit window at 60 Hz, pads via `PadSystem::read` (keyboard, GC adapter, gamepads, `MELEE_PAD_SCRIPT`), scene flow
  opening -> title -> main menu -> VS menu -> CSS -> SSS -> loading -> live VS match (pause by START, quit by L+R+A+START) -> results -> CSS.
- Verified by scripted runs (tools/rust-port/play/*.txt): `vs_to_ko.txt` (with `--golden-pads --time-secs 25`) reaches a match and finishes it
  to results; `pause_quit_rematch.txt` pauses, resumes, quits to CSS and plays a second match to results on live pads. Shots: docs/rust-port/shots/play_*.png.
- Stand-ins, not ports: every non-match screen is drawn by ui.rs (rects + bitmap font), not the retail MnTitle/MnMain/MnCharSel scenes or the
  gmfrontend kit; opening movie is a card; roster = 7 fighters, 4 stages; rules fixed (time match). The sim (vs_match.rs) halts with a reported
  reason on unported fighter code (~107 sites: most button presses in IASAs, attacks, shield variants), so free live play ends early on such input.
  START is masked from the fighters (pause layer owns it).

- Round 2: `tools/rust-port/play/full_loop.txt` runs the whole loop on live pads in one go (`--skip-intro --time-secs 12 --shots --quit-at-results`):
  title, main, VS, CSS, SSS (Battlefield picked), loading, match, pause (START), resume, clock out, results; 1359 ticks. `--shots` now also
  captures the pause (10 ticks in) and a live frame (tick 420). Match/pause shots hold disc art and are not committed; play_sss.png is.
  Not tried: keyboard and gamepad input by hand (the GC adapter did open in the run, but the script drove the pads).

- Round 3 (hands-off input): a scripted run (`MELEE_PAD_SCRIPT` set) never touches the user's keyboard or focus. The window is created
  with `with_active(false)` (`WindowConfig.active`, also view-golden and gx_viewer), the GC adapter is not opened (`MELEE_PAD_IGNORE_ADAPTER=1`),
  gamepads are a fixed empty set and port 1 gets no keyboard preset, so alt-tabbing cannot change the result. No tool in the repo sends
  OS-level input (grep for SendKeys/SendInput/keybd_event/AppActivate/pyautogui: none). Interactive `melee play` binds port 1 to the keyboard
  preset (WASD/IJKL/Space/Shift/Enter; `MELEE_KEYBOARD=0` off). `view` renders any stage's map groups, not just FD.
  Re-run: full_loop.txt (1360 ticks to results), vs_to_ko.txt --golden-pads (1875 ticks, winner port 2 by falls), pause_quit_rematch.txt (two matches, 1563 ticks), all with the window in the background.
  Still stand-ins: ui.rs menus (gmfrontend.c's .inc units and the retail mn* scenes are not driven), opening card, 7 fighters, 4 stages.

## Round 4: full roster, rules, repeatable matches (2026-11-02, port/parity1)
- CSS (stand-in, ui.rs): all 26 fighters in the retail order (Sheik has her own icon; Jigglypuff marked "SPECIALS INCOMPLETE (BLOCKER D)"),
  4 slots HMN / CPU LV1-9 / NA, costume (Y) or team in a team match, and a rules bar: TIME (minutes) or STOCK (1-99), ITEMS OFF / very low..very high,
  TEAMS. Port 1 moves its focus with Z (P1 -> P2 -> P3 -> P4 -> RULES); ports 2-4 edit their own slot. X cycles HMN/CPU/NA, L/R the CPU level.
  The setup becomes a `melee_engine::scenario::Scenario` (no string) and reaches the game through `SceneLaunch_SeedVs`: GameRules mode /
  stock_count / time_limit, StartMeleeRules is_teams, items via `SceneLaunch_ApplyItemFrequency`, per-player slot_type / cpu_level / color / team.
  The match ends the game's own way (TIME! / GAME!); results read Player_* (KOs, falls, stocks, percent) and pick the winner per team.
- SSS: all 29 retail VS stages (2..32 without Akaneia 21 and Icetop 26) plus RANDOM (play's own LCG, never the game RNG). Headless 1-minute
  fox v marth CPU9 sweep: 25 stages finish; Corneria, Great Bay (grgreatbay.rs:577 panic), Mushroom Kingdom II and Venom crash: marked on the SSS.
  All 26 fighters finish a 1-minute 4-CPU match (7 runs of 4).
- Rematch: `Engine::next_vs` leaves the scene like `gm_801A4014` (`gm_801A4D34_end`, `gm_801A4014_leave`: the VS scene's and state's on_exit) and
  runs `runGameMode_load(GM_VS)` -> `gm_801A4014_enter` again; the host, link table and `main`'s libraries stay. Fixes this needed:
  the gmMainLib_8046B0F0 reset-flag seams; gm_1B03's SceneLaunch_Active / EnemyTeamColors copies (teams); `pl_80038914_fp` returning player.c's
  loaded `pl_804D6470`; `pl_800386D8` reading past `by_attack_hi[65]` as the C does (ids 0x65..0x72); the particle bank memo
  (`dat_custom::forget_banks`: a file re-read at the same raw address got the already relocated bank, double relocation -> crash in match 3);
  and `archive::RECLAIM` (set by next_vs only): a file parsed over the raw bytes of older images frees them (they leaked ~0.4-0.8 GB per match).
- Memory (`play: mem after N matches` lines, working set): 5 consecutive 4-CPU matches Brinstar, Jungle Japes, Green Greens, Poke Floats,
  Kongo Jungle 64: 3.62, 4.19, 6.38, 6.39, 6.39 GB at match start (flat from match 3 on; one match was already 3.6 GB before this work).
  Same stage 5 times: 3.62, 4.42, 5.21, 5.21, 5.21 GB.
- melee_gx: a draw whose vertex data runs past the command list (prim 0xF8 fmt 7 count 65535, seen at frame 1905 of a falcon v fox CPU match
  on FD, windowed only) panicked the game; it now drops the rest of that list with one log line. The cause is not found.
- Tests (no OS input; self-closing): `bash tools/rust-port/play/rematch.sh [--windowed] [css|x5|all]` runs css_rematch_4cpu.txt (CSS ->
  4 CPUs, 1-minute time, items very low, Brinstar -> results -> CSS -> 2v2 teams, 1 stock, Onett -> results) and rematch_x5.txt.
  Headless and windowed runs give the same results (deterministic).

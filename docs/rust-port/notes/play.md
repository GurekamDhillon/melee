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

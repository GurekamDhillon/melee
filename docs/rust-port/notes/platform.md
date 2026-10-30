# Platform track: melee_platform and the platform summary

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 279-304 -->
## Platform track, step 1: melee_platform (2026-10-21)
- Crate `rust/melee_platform`: `pad` (PADStatus = `melee_types::PADStatus`, constants, Aurora's
  PADClamp/PADClampCircle), `adapter` (port of `pc/platform/gc_adapter.c`: reader thread holding the
  newest report, hotplug scanner, focus-following ownership, `MELEE_PAD_IGNORE_ADAPTER`/`MELEE_UNATTENDED`,
  recalibrate, rumble-on-change; transports WinUSB then HID like the C; `decode` = gc_adapter_decode.h),
  `aurora` (Aurora's PADRead per-port arithmetic: keyboard bindings, standard button map, default
  axis map + dead zones, trigger emulation), `gamepads` (gilrs -> SDL-convention raw values,
  connection-order ports), `script` (`MELEE_PAD_SCRIPT` .txt + `MELEE_PAD_CHANNELS`), `pad_read`
  (`PadSystem::read` = shim_pad.c `gw_PADRead` order: Aurora layer, mode-NONE/suspended disconnect,
  F9 recal, adapter override, idle port 0, script, L+R+Y+X+Start combo -> Aurora's no-op
  PADRecalibrate), `main_loop` (winit 0.30 window; exact 60 Hz fixed step in integer ns*60, max 5
  catch-up ticks; present gets alpha; 100 ms focus-loss debounce; `run_headless`).
- The game calls `PadSystem::read` itself inside `Game::tick` (as HSD_PadRenewRawStatus calls PADRead).
- `button` is native-order u16: the C's bswap16/gw_w16 exist only for big-endian game memory.
- Verified: `cargo test -p melee_platform` (oracle: 3 clamp functions over every input, decode over a
  4000-report LCG stream with recal, equal to `oracle/oracle.cpp` built from the shipping sources;
  pipeline/adapter-lifecycle/script/clock tests with a fake device). `cargo run -p melee_platform
  --example window -- 4` opened a window, ran 240 ticks in 4.03 s with 0 dropped, and read a real
  WUP-028 (Zadig/WinUSB) controller in port 1. Not verified: the HID transport on hardware (this
  machine's adapter is WinUSB-bound), gilrs gamepads and keyboard on hardware (no gamepad attached;
  keyboard map is unit-tested only), Aurora gamepad arithmetic against the C++ (hand-checked values
  only; SDL axis values vs gilrs floats are a conversion, not identical device readings).
- Not ported: Linux adapter (`gc_adapter_linux.inc`), Lua/live/bot pads and gd.input claims,
  ctl_poll remapping, stick dead-zone setting, gamepad rumble, Aurora per-device mapping files and
  per-type default maps, diagnostics/overlay/profiling. No rendering yet (step 2: wgpu).


<!-- RUST_PORT.md lines 1641-1666 -->
## Platform (summary, 2026-10-25, port/platform-integ)

Rule for every run below: no OS-level input and no focus stealing. Game input only through `MELEE_PAD_SCRIPT`, `MELEE_PAD_IGNORE_ADAPTER=1`,
the `PadSystem` script source, or in-process hooks. Scripted `melee play` / `view-golden` windows are created unfocused
(`WindowConfig.active = false`, `MELEE_WINDOW_UNFOCUSED=1`), the GC adapter is not opened, gamepads are an empty set, and port 1 has no keyboard
preset, so alt-tabbing or typing cannot change a result. Grep of tools/, pc/tools/, rust/ for SendKeys/SendInput/keybd_event/AppActivate/
SetForegroundWindow/pyautogui finds nothing. C-build comparisons park the window at 30000,30000 without activation.

### Status per track
| Track | Status | Proven (how) | Remains |
|---|---|---|---|
| melee_platform (window, 60 Hz loop, pads, GC adapter) | working | oracle tests vs the C clamp/decode/Aurora code; fake-device pipeline tests; window example ran 240 ticks with 0 dropped | HID transport, gilrs gamepads, keyboard on hardware; Linux adapter; rumble |
| melee_gx (GX on wgpu) | working for the FD scenario | `view-golden` vs realtime C screenshots (`tools/rust-port/shotcmp3.py`): mean abs error 2.43 of 255 averaged over frames 60-1500 (f60 4.13, f1500 2.84), 88-98% of pixels within 16 | entry sequence (trophy stand, light pillars, beam effect 36 with PATH track), KO frames (star, SCORE -1, percent digits; error 12-26 there), AppSRT particles (off by default), bone-attached effects, other stages unchecked, fog/shadows/flashes unprovable on FD |
| melee_audio (AX/synth/cpal) | proven headless | sample-exact replay vs the C mix: 1953920 samples 0 differ (match), pause/resume 2440 sub-frames 0 differ, time-up 12856 sub-frames identical; master volume | GAME! call, results/sudden-death/menu audio, positional/KO paths in live play, native sim's own SFX only equal up to the first game-driven sound; sounds are never judged by ear |
| melee_net (rollback, UDP) | working on loopback | synctest depth 12: 1802 frames, 0 hash mismatches, 1801/1801 golden-equal; host+guest with 70 ms lag and 5% loss: 78 rollbacks, 0 desyncs, golden-equal; lobby, room codes and random matchmaking paths too | UPnP, Envoy/mod-sync/identity messages, rematch, CPUs/items; real internet play; Rust vs C build interop only measured, see the netplay section |
| melee_script (Lua, `gd.*`) | partial | same Lua 5.4.7 sources as C; 82 standalone scripts identical to system lua; per-module mock tests green | ~140 of 349 `gd.*` names, recorder state machine, fighter Lua, native Geno `define` entries |
| melee_data (card, settings, mods, replays, crash log) | working | C-oracle transcripts per module; `.slp` and `.gdlp` playback in the native sim equals the C playback on 1801 frames | `melee` bin does not use the card; UCF in playback; embedded-state restore; Marth v Sheik C golden |
| mex-import | working | output byte-identical to the Python packer on two mods | running mod PPC natively, DOL/Gecko patches |
| `melee play` (menus, match, pause, results) | flow works, screens are stand-ins | three scripts (`tools/rust-port/play/*.txt`) run title to results hands-off: 1360, 1875 and 1563 ticks | retail mn* scenes and the gmfrontend .inc units are not ported (ui.rs rectangles + bitmap font draw every non-match screen); opening is a card; 7 fighters, 4 stages, fixed rules; sim halts on ~107 unported fighter sites, so free play can end early |

### What only the user can check
- Visuals: how it looks to a person (the numbers above are pixel error against the C build on FD only), menus, other stages and fighters.
- Audio by ear: nothing has been listened to; equality is against the C mix on fixed scenarios.
- A real controller: the WinUSB adapter read in port 1 once; HID transport, gamepads and the keyboard preset (WASD/IJKL/Space/Shift/Enter) in interactive `melee play` are untried by hand.
- Online against another person over the internet: only loopback with injected lag/loss and local server paths were run.

# The Melee Unlocked launcher seam (Rust port as a third engine)

Status 2026-11-03. Studied: Hero88go's melee-unlocked v0.9.86 (GPL-3.0, our clone
`scratch/ext/melee-unlocked`, used with his permission). Everything below is our description of his launcher; none
of his code is copied here. Nothing has been sent to him: the launcher-side patch lives only on the local branch
`ours/rust-engine` of the scratch clone.

## 1. How his launcher drives an engine

**Engines and where they are found.** One launcher (`MeleeUnlockedLauncher.exe`, `port/app/launcher.cpp`), two
engines picked on the GAME BUILD row and kept in `launcher.ini` as `engine=` (absent = Static Recomp, `1` = Source
Port; `launcher_engine.h` parses it and rejects anything out of range):

| engine | files | searched in |
|---|---|---|
| Static Recomp | `melee_port.exe` (or `melee_port_compat.exe` on CPUs without AVX2, `cpubuild=`) | the game folder, `Versions\<v>` when an older version is active, a checkout's `build-review`/`build-integration` |
| Source Port | `melee_source.exe` + `melee_game.dll` (both must exist) | the game folder, or a checkout's `build-sourceport\port\Release` |

The no-Slippi build (`MELEE_NO_SLIPPI`) has only the Source Port and forces `engine=1` at start. An engine
from another release is refused by comparing the exe's version resource (`FileVersion`) with the launcher's.
**Updates** come from the GitHub releases feed (`runtime/host/updater.cpp`): a whole release zip is extracted
over the folder, and older versions can be kept under `Versions\`. Nothing updates an engine on its own.

**Process start.** `CreateProcessW` (`launch_process.h`), no pipes, inherited environment, working directory =
the game folder (the one holding `Sys\codehandler.bin`; in a checkout, the repo root). The launcher minimises
itself, a thread waits for the process, and `WM_APP_GAME_DONE` gets the exit code.

**Command line** (plain PLAY):

```
<exe> --iso <retail 1.02 iso> --threaded-renderer --settings-path <game folder>\port-settings.ini
      [--sys-dir <dir>\Sys [--user-dir <dir>\User\Slippi] --replay-dir <dir> --card-dir <dir>\User\GC\CardA]
      [--slippi-menus off --vanilla-only [--vanilla-game]]         (the Vanilla segment, Source Port only)
      [<lobby args>] --lobby-status-file <dir>\lobby-game-status.json
```

The bracketed folder options appear only when `Sys\codehandler.bin` is there (a release folder). Variants:
- mod launch: Source Port `--mod-iso|--mod-gci <file>` (+ every enabled TE `--mod-gci`), Static Recomp
  `--iso <mod> --mod-base-iso <retail>`; card folder `User\GC\Mods\<key>`;
- lobby match: `--p2p-port/--p2p-peer/--p2p-slot/--p2p-chars/--p2p-stage/--p2p-seed/--p2p-delay/...`
  (no-Slippi P2P) or `--lobby-direct CODE --lobby-character N` (Slippi);
- replays: `--replay <slp> [--as-experienced|--trace]`.
- Test mode (`MELEE_LAUNCHER_TEST`): hidden window, `MELEE_LAUNCHER_TEST_GAME_ARGS` appended, the exact command
  appended to `MELEE_LAUNCHER_TEST_LAUNCH_LOG`. Engines read `MELEE_NO_GC_ADAPTER` (do not take the adapter).

**Settings.** The launcher does not pass video/audio/controller values on the command line: the game owns
`port-settings.ini` (`key value` per line, written by the in-game panel): `fullscreen`, `exclusivefullscreen`,
`vsync`, `widescreen`, `truewidescreen`, `window WxH|follow`, `scale` (EFB multiplier), `msaa`, `ssaa`, `fps`,
`backend`, `volume`, `music`, audio device/ASIO keys, `dlss*`, online delay, overlay/HUD keys, and per-device
bindings (`key_<action>`, `pad<n>_`, `gc<n>_`, `ds4<n>_`, `swpro<n>_` with his button masks), `rumble`,
`backgroundinput`. GC adapter use is automatic (libusb/WinUSB), off with `MELEE_NO_GC_ADAPTER`.
`launcher.ini` (launcher only): `iso=`, `engine=`, `vanilla=`, `cpubuild=`, replay folder, language, lobby
prefs; a copy of `iso=` also goes to `%LOCALAPPDATA%\MeleeUnlocked\launcher.ini`.

**Status back.** No stdout protocol (stdout is not read). Exit code: only shown to the player after a lobby match
(non-zero = "could not connect"). Lobby progress goes through the JSON `--lobby-status-file`. **Crashes:** after
*any* exit the launcher looks for `melee_port_crash.txt` in the working directory newer than the launch (first line
`CRASH: ...`), collects it with `melee_port.log` and `lobby.log`, writes `melee_crash_report.md/.zip`, and offers to
post the zip to his relay (`X-MU-Engine` header names the engine). The engines also write `melee_port_crash.dmp`.

## 2. Our side: `melee.exe` in launcher mode (`rust/melee/src/launcher_compat.rs`)

Detection: argv starts with a flag only the launcher sends (`--iso`, `--settings-path`, `--card-dir`, ...; a
`melee` subcommand never starts with `--`), or `melee play --launcher-compat`. The line becomes a `melee play` run
(retail menus):

| launcher sends | Rust engine does |
|---|---|
| `--iso` | the disc |
| `--settings-path` | reads `port-settings.ini`; `fullscreen`/`exclusivefullscreen` -> `MELEE_FULLSCREEN` off/borderless/exclusive, `widescreen`/`truewidescreen` -> `MELEE_WIDESCREEN`, `window WxH` -> `MELEE_WINDOW` (`follow` = ours), `scale` -> `MELEE_RENDER_SCALE` (<=0 auto), `msaa`, `vsync`, `volume` -> `MELEE_VOLUME` |
| `--volume`, `--fullscreen`, `--widescreen`, `--vsync`, `--window`, `--scale`, `--log-file` | override the file |
| `--card-dir <..>\User\GC\CardA` | `MELEE_CARD_PATH=<..>\User\GC\RustPort` (our `USA\Card A` .gci tree, kept apart from his folder) |
| `--threaded-renderer`, `--sys-dir`, `--user-dir`, `--lobby-status-file`, vanilla flags, `--fps`, `--backend`, `--dlss*`... | accepted, logged, no effect |
| `--mod-*`, `--replay`, `--lobby-*`, `--local-peer`, `--p2p-*` | refused: log line, exit code 3 |
| `MELEE_NO_GC_ADAPTER` | `MELEE_PAD_IGNORE_ADAPTER=1` |
| anything else | passed to `melee play` unchanged (so test args like `--skip-intro --quit-after-matches 1` work) |

Precedence: an already-set `MELEE_*` variable > the launcher's command line > `port-settings.ini` > settings.cfg >
default. Log: `melee_port.log` in the working directory (`MELEE_LOG_FILE`). Crash: besides `crashlogs\crash-*.log`,
`melee_port_crash.txt` (`MELEE_CRASH_COPY`, first line `CRASH: <kind>: <reason>, engine rust (melee)`). Exit 0 on a
clean quit. Unit tests: `cargo test -p melee --bin melee launcher_compat`.

## 3. Launcher-side patch (local only: scratch clone, branch `ours/rust-engine`, commit 8263f6e)

`ENGINE_RUST = 2` (`engine=2` in launcher.ini); exe = `MELEE_RUST_EXE` or `<game folder>\rust\melee.exe`; the
no-Slippi build keeps it when picked; Rust crashes are **not** posted to his relay (logged locally); test-only
`MELEE_LAUNCHER_TEST_AUTOPLAY` presses PLAY by itself, appends `game exit <code>` to the launch log, and quits the
launcher after the game. No GAME BUILD segment was added (it is picked through launcher.ini). Built with cl.exe and
no CMake by `tools/rust-port/launcher/build_mu_launcher.bat <clone> <out dir>`.

**Test** `bash tools/rust-port/launcher/launch_test.sh <folder>` (folder: the launcher exe, `launcher.ini` with
`iso=` and `engine=2`, an empty `Sys\codehandler.bin` so the full release argument set is sent, `port-settings.ini`).
Hidden launcher, `LOCALAPPDATA` redirected so the user's remembered ISO is untouched, pad script
`retail_full.txt`. Result 2026-11-03: the launcher ran the Rust exe with its full release command line, settings
mapped (window 640x480, scale 2, vsync, volume 3), title -> VS -> match -> results, `quit after 1 matches`,
`game exit 0`: **LAUNCH OK**.

## 4. Contract with his current launcher (re-audited 2026-11-22: melee-unlocked 1.0.7, main 6c9472f; branch port/mucompat)

Supersedes the 2026-11-04 gap list. OK = same result as the Source Port; PART = works with a stated difference;
MISSING = not there; REFUSED = we exit 3 on purpose rather than start a different game than the one asked for.

| launcher contract | Source Port | Rust Port | state |
|---|---|---|---|
| engine pick: GAME BUILD row, `launcher.ini engine=` | engines 0 / 1 | "Rust Port" segment, `engine=2`, only when `rust\melee.exe` exists (launcher PR branch) | OK with PR |
| PLAY: `--iso --threaded-renderer --settings-path [--sys-dir --user-dir --replay-dir --card-dir --sysgen-data-dir]` | yes | translated by launcher_compat.rs | OK |
| Vanilla segment `--slippi-menus off --vanilla-only [--vanilla-game]` | yes | accepted; the Rust Port is always vanilla | OK |
| Build tab / warm cache `--hidden --frames N --log-file F` | yes | headless N-frame run, log to F | OK |
| Settings button `--settings-window [--settings-standby PID] [--settings-tab T]` | own panel | our overlay in its own window, reads/writes port-settings.ini (section 7); the PR starts it | OK |
| port-settings.ini display / audio | all keys | fullscreen, exclusivefullscreen, widescreen, truewidescreen, window, scale, msaa, vsync, volume, music, backgroundinput; `fps` -> our fps cap (0 / unlocked = off, 30..2000 clamped to our 1000, -1 / monitor = follows the display, nothing set; a limit on presented frames only, no interpolation); `ssaa` 2..4 multiplies a numeric `scale` into our render scale (cap 8), with an automatic scale it is logged and left (auto already supersamples); `--fps` likewise; `audio_device` (WASAPI endpoint id) -> the output device by name: the id is resolved to its friendly name through MMDevice (`audio_endpoint.rs`), checked against the outputs we can open, else the system default, with a log line either way (a name already in our list is taken as is); not backend, dlss*, ASIO, HUD / overlay keys | PART |
| bindings `key_*`, `pad<n>_*`, `gc<n>_*` | yes | yes | OK |
| bindings `ds4<n>_*`, `swpro<n>_*` | yes | yes, by position through gilrs (this branch; a DualSense uses the DS4 table) | OK |
| bindings `hid<n>_*`, `hid<n>_invert` (generic HID pads, box controllers in DInput) | yes | yes: 32-bit masks (bit N = button N+1, hat on bits 28..31, a signed spelling of bit 31 read back unsigned) for a generic / box pad that gilrs gives as a raw controller (Windows WGI; Linux evdev by button order); an XInput-layout pad keeps `pad<n>`. Not tried on real hardware (no box here) | PART |
| box mode (a keyboard or HID table with Mod X / Mod Y bound), `stick_modifier_percent` + StickMod | yes | yes, through `profiles::box_sticks` (the port's `box_values` and SOCD 2IP-without-reactivation, so the coordinates are coarser than his B0XX angle table: no shield / B angle variants); Mod X + Mod Y = D-pad layer; light / mid shield = R travel; the modifier on keyboard, pad and HID stick buttons | PART |
| per-binding levels `<table>_<Action>_level` | yes | no (every analog source clicks at 200 / 255) | MISSING |
| `rumble` | gamepads and adapter | the game's motor commands (`PADControlMotor`, hooked in melee_engine) go to the pad that port is read from: gilrs force feedback (WGI `SetVibration` on Windows) on a gamepad, the adapter's motors on an adapter port. `rumble 0` stops it (a stop is always honoured); `MELEE_RUMBLE=0` and `MELEE_RUMBLE_PORTS=1,3` do the same from the environment; silent in a rollback re-simulation; capped at 10 s if the game never stops it. Output only: pad bytes, game state and checksums never see it. Before this the hook was a no-op, so no pad rumbled at all. Not felt on real hardware (no pad here), only the decision logic, the hook and the launch are tested | OK |
| `MELEE_NO_GC_ADAPTER` (his test runs) | yes | `MELEE_PAD_IGNORE_ADAPTER=1` | OK |
| replays into `--replay-dir` (his viewer lists `.slp`) | `.slp` | one `.slp` per match, plus `.mrpl` in `RustPort\` | OK |
| Replays page `--replay <slp> [--as-experienced]` | plays | plays (slp_play.rs); the PR routes the page to the Rust Port when picked; `--as-experienced` ignored | PART |
| lobby status `--lobby-status-file` | `{"status","stocks"}` | same schema (also says "In match" offline) | OK |
| lobby result mailbox `<status file>.results` (new since 11-04) | a line per lobby game | same line, `--lobby-direct` games only (this branch) | OK |
| Slippi Direct `--lobby-direct CODE --lobby-character N` | yes | Slippi wire path (real Slippi only with the player's login); PR tags the build `<version>:rust` | OK, not tried against a real peer |
| P2P Direct `--p2p-*` (mu_net) | yes | refused; the PR shows his "needs the Source Port" message | REFUSED |
| YouGame: `--online-mode`, `YOUGAME_BRIDGE` / `YOUGAME_TOKEN`, `MELEE_YOUGAME_*` | yes | `--online-mode` opens the online menus; bridge vars read by melee_platform/yougame.rs; extension channel untested | PART |
| mod disc `--mod-iso`, or `--iso M --mod-base-iso R` | yes | data-only discs (main.dol identical to retail); code mods refused | PART |
| TE packs `--mod-gci`, `--mod-dir`, `--mod-profile` | yes | refused; the PR keeps TE launches on the catalog's engine | REFUSED |
| saves `--card-dir` | his CardA | `User\GC\RustPort`, CardA imported once (card_import.rs) | PART |
| crash files `melee_port_crash.txt` (+ `.dmp`), `melee_port.log` | yes | yes; the PR sends them through `crash_report::offer` / Send logs named "Rust Port" | OK |
| exit codes | 0; non-zero after a lobby game = "could not connect" | 0; 3 = request not supported | OK |
| FileVersion check | must match the launcher | the PR skips it for the Rust exe | OK with PR |
| updates | release zips | none; the `package.sh` folder is extracted over his | MISSING |

**Not done here, with estimates:** P2P Direct / mu_net (1-2 weeks, notes/mu-net-feasibility.md); code mods with
their own main.dol (the PPC interpreter, on hold for sc00p); TE packs / .gci loader (2-4 days once the interpreter
exists, most TE content is PPC code); per-binding levels (0.5 day); backend choice and DLSS (not planned: wgpu only); an update channel (1 day once Hero agrees on a feed).

Launcher patch: one commit on his main 6c9472f, branch `rust-port-engine` (see the round-three report). The
test-only network skip (no YouGame refresh, no lobby/DHT in `MELEE_LAUNCHER_TEST` runs) is a separate local commit
and never part of the PR.

## 5. Drop-in round two (2026-11-03, port/launch2)

Closed the UI/packaging gaps of §4 (as it stood on 2026-11-03). Box mode has since landed in our own controller
profiles; the current list is §4.

- **GAME BUILD row** (patch commit 385e535 + test hooks, branch `ours/rust-engine`): a "Rust Port" segment when
  `rust\melee.exe` is there (no-Slippi build: Source Port | Rust Port; normal build: a fourth segment before
  Vanilla); picking it saves `engine=2`. Its exe is exempt from the `FileVersion` match (its own version).
  **Settings** with the Rust engine opens the Source Port's panel (it writes the `port-settings.ini` we read), or a
  message without one; `melee.exe --settings-window` exits 0 at once.
- **Build tab / warm cache**: `--hidden --frames N` -> `--headless --max-frames N`, `--log-file` honoured
  (`launcher_build.log`), no replay recorded.
- **Version resource**: `rust/melee/build.rs` writes a VS_VERSIONINFO .res (FileVersion = workspace version,
  Comments = git hash) and links it; no rc.exe.
- **Controls** (`melee_platform::bindings`, `MELEE_PORT_SETTINGS`): `key_<Action> <VK>` -> port 1's keyboard,
  `pad<n>_<Action> <XInput mask>` (0x400/0x800 = LT/RT) -> the gilrs pad on port n, `gc<n>_*` adapter remaps
  when not the identity, `backgroundinput` -> the adapter keeps reading unfocused. Without the file, input is unchanged.
- **Mods**: `--mod-iso` (or `--iso <mod> --mod-base-iso <retail>`) plays when the mod disc's main.dol is the retail
  one byte for byte (a data-only mod); otherwise refused (exit 3). `--mod-gci`, `--mod-dir`, profiles: refused.
- **Replays**: `--replay-dir D` -> our `.mrpl` recorded to `D\RustPort\` (his viewer lists .slp only).
- **Packaging**: `bash tools/rust-port/launcher/package.sh <out> [patched launcher exe]` -> `<out>\rust\melee.exe`,
  `version.txt`, `README.txt` (+ launcher), checked for disc-shaped files, and `<out>.zip`.
- **Tests**: `SEGMENT=3 BUILD=1 bash tools/rust-port/launcher/launch_test.sh <folder>` -> BUILD OK;
  `SEGMENT=3 bash ... <folder>` (launcher.ini engine=1 to start) -> engine=2 saved, LAUNCH OK. Both passed on a
  folder made by package.sh.

## 6. Mod launches and the test-mode exit (2026-11-03, port/modhost2; patch commit 1937c29)

- **Mods page -> Rust engine**: with the Rust Port picked, PLAY on an installed disc mod (any kind but TE `.gci`)
  starts `rust\melee.exe --iso <retail> --mod-iso <mod>` with the settings, `Sys`, replay and the mod's own card
  folder (`User\GC\Mods\<key>`, which launcher_compat maps to its `RustPort` sibling). A data-only disc plays; one
  with its own main.dol exits 3 (refused, logged). TE packs and lobby mod matches keep the engine his catalog picks.
- Test hook `MELEE_LAUNCHER_TEST_MOD_ISO=<disc>`: autoplay launches that disc as the Mods page does.
  `MELEE_LAUNCHER_TEST_MOD_ISO=<retail iso> bash tools/rust-port/launcher/launch_test.sh <folder>` gives the command
  line above, `mod disc ...: the retail game's code`, one match, game exit 0: **LAUNCH OK**.
- **The exit 127**: our test hooks ended the launcher with `PostQuitMessage`, which skips `WM_DESTROY`, so the lobby's
  worker `std::thread` was still joinable when statics were destroyed: `std::terminate`, CRT fail-fast, process exit
  `0xC0000409` (bash shows 127). Not a bug in his launcher (its close path goes through `WM_DESTROY`). The hooks now
  call `DestroyWindow`; the launcher exits 0 (LAUNCH OK and BUILD OK both print `launcher exit: 0`).

## 7. Our own settings window (2026-11-22, port/musettings)

`melee.exe --settings-window --settings-path <port-settings.ini> [--settings-tab <name>] [--settings-standby <pid>]`
(`rust/melee/src/settings_window.rs`) opens the in-game settings overlay (settings_overlay.rs, same pages and look,
panel centred) alone in an 800x600 window: no disc, no engine; text is the bundled fonts.

- **Files:** on open his ini's display/audio values (port_ini.rs, the reverse of `map_settings`) are copied into
  settings.cfg and audio.cfg; each change goes to settings.cfg (as in game) and the shared keys to his ini at once:
  `widescreen` (+ `truewidescreen 0`), `fullscreen` + `exclusivefullscreen`, `vsync`, `scale` (0 = auto), `msaa`,
  `volume`, `music`. Only those lines are rewritten; every other line (controls tables, unknown keys, comments, CRLF)
  is kept byte for byte. The controls tables are not edited (no bindings page).
- **Tabs:** his names; `video`, `audio`, `game`, `overlays`, `customize` match our pages, `mods` -> CUSTOMIZE (skins),
  `controls` / `gecko` keep VIDEO. SESSION holds only "Close settings".
- **Standby:** waits on `Local\MeleeUnlockedSettingsShow-<pid>` / `...Quit-<pid>` / the launcher's exit before any
  window or GPU device exists (no cost while waiting; the window takes a moment to appear after the click). Show:
  re-read the files, open; quit or launcher gone: exit 0. The GC adapter is not touched while waiting.
- **Input:** controllers (D-pad / stick, A, B, L/R pages) and keyboard (arrows, Enter, Escape, Tab). Closing (B,
  Escape, "Close settings", the title bar X) exits 0. Log `melee_settings.log` (never the game's `melee_port.log`).
- **Launcher patch:** `pr/rust-port-engine` commit 2ab6d6f3 (and the squashed `scratch/ext/0001-...patch`): Settings
  and its standby copy start `game_exe()`, i.e. `rust\melee.exe` with the Rust Port picked; the Source Port fallback
  and the "no settings window" message are gone.
- **Tests:** `cargo test -p melee --bin melee port_ini launcher_compat settings_overlay`;
  `bash tools/rust-port/launcher/settings_window_test.sh <melee.exe> [shot dir]` (hidden: open + script changes the
  volume + quit, the ini asserted; standby shown by the event; standby ended by the launcher's exit; with a shot dir,
  720p/1080p pictures via `MELEE_SETTINGS_SHOT`). Hooks: `MELEE_SETTINGS_SCRIPT`, `MELEE_SETTINGS_SHOT`,
  `MELEE_SETTINGS_MAX_TICKS`.

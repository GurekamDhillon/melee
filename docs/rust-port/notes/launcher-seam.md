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

## 4. Gaps for parity with what his launcher offers (re-checked against rust-port 5138e0ba5, 2026-11-04)

Sections 5 and 6 closed some of the original list; the merges since 2026-11-03 added engine-side features that his
launcher cannot reach yet. State per area (code: `rust/melee/src/launcher_compat.rs` unless named):

- **Online:** still no Slippi lobby (`--lobby-direct`, `--lobby-character`), no P2P Direct (`--p2p-*`, his mu_net
  protocol: mu-net-feasibility.md), no lobby status JSON: all refused (exit 3). Our own online (`melee netplay`, host
  / join from the retail CSS with rematches, read-only spectators, `melee_relay`; notes/net.md,
  notes/spectate-replay.md) is a different protocol, reachable from the in-game browser's Online section, not from
  his launcher.
- **Mods:** data-only `--mod-iso` plays (section 5); `--mod-gci`, `--mod-dir`, `--mod-profile` and TE packs are
  refused. Cosmetic imports now exist on our side (`melee cosmetics import`: costume .dat / .usd, mod ZIP, Nucleus
  vault ZIP, portrait / stock PNG; `cosmetics.rs`, `cosmetics_import.rs`) and skins are picked in the settings
  overlay or the browser, but nothing maps his cosmetic or music-pack folders. Gameplay codes (UCF and others) are
  opt-in match rules (`--codes` / `MELEE_CODES`, `melee_core::codes`, notes/gameplay-codes.md); his Gecko lists and
  code toggles are not read.
- **Replays:** every match is recorded (`.mrpl` in `--replay-dir`'s `RustPort` subfolder, `.slp` export in the
  folder). Playback exists in the engine (`melee play --replay`, the browser's Replays section, playback controls),
  but the launcher's `--replay <slp>` and `--as-experienced` / `--trace` are refused.
- **1P modes** through the retail menus as far as melee_modes has them; the coverage set (classic, adventure,
  all-star, target test) passes (STATUS.md).
- **Controller mapping:** `key_*`, `pad<n>_*`, `gc<n>_*` and `backgroundinput` are read (section 5). Not read: his
  `ds4<n>_*` / `swpro<n>_*` tables, `rumble`, PhobGCC tools. Our own profiles (deadzones, remaps, box mode + SOCD,
  `melee_platform::profiles`) are configured through `controllers.ini`, not his file.
- **Video / audio:** the engine now has an fps cap, render-only interpolation, widescreen, vsync and MSAA, several
  of them live from the in-game settings overlay (`settings_overlay.rs`, `display.rs`). From his side only
  `fullscreen`, `widescreen` / `truewidescreen`, `window`, `scale`, `msaa`, `vsync`, `volume` and
  `backgroundinput` are mapped; `--fps`, `--backend`, `--dlss*`, `--ssao`, `--frame-mode` and the `fps` key are
  accepted and ignored. Not present at all: backend choice, DLSS / frame generation, SSAA, CRT / shader presets,
  audio device / ASIO selection, a separate music volume.
- **Settings button:** `--settings-window` still exits 0 at once. The engine now has its own in-game overlay (F1, or
  hold D-pad Down and press Start) that writes our settings.cfg, not his `port-settings.ini`.
- **Saves:** separate card folder (`User\GC\RustPort`); importing his `CardA` .gci files is not done.
- **Crash flow:** we write `melee_port_crash.txt` but no `.dmp`; the patch keeps reports local. Posting Rust reports
  anywhere needs Hero's agreement and an engine name on his relay.
- **Packaging:** closed by section 5 (GAME BUILD segment, version resource, `package.sh`); no updater channel.

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

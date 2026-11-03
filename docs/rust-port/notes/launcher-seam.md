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
| `--threaded-renderer`, `--sys-dir`, `--user-dir`, `--replay-dir`, `--lobby-status-file`, vanilla flags, `--fps`, `--backend`, `--dlss*`... | accepted, logged, no effect |
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

## 4. Gaps for parity with what his launcher offers

- **Online:** no Slippi lobby (`--lobby-direct`), no P2P Direct (`--p2p-*`, his mu_net protocol), no lobby status
  JSON. Our netplay (`melee netplay`) is a different protocol.
- **Mods:** `--mod-iso`/`--mod-gci`/`--mod-base-iso` (20XX TE, m-ex discs like Akaneia/ACE), mod profiles,
  cosmetic imports, music packs, Gecko codes, the built-in UCF/unlock codes, training packs. Refused today.
- **Replays:** no `.slp` recording into `--replay-dir`, no `--replay` playback / as-experienced traces.
- **1P modes** through the retail menus as far as melee_modes has them (onep's work); his launcher assumes all.
- **Controller mapping:** his per-device binding tables (`pad/gc/ds4/swpro<n>_*` masks), rumble,
  background input, PhobGCC tools are not read; we use our own pad defaults.
- **Video/audio extras:** backend choice, fps caps/unlocked, interpolation, DLSS/frame generation, SSAA, CRT and
  shader presets, separate music volume, audio device/ASIO selection: ignored.
- **Saves:** separate card folder (`User\GC\RustPort`); importing his `CardA` .gci files is not done.
- **Crash flow:** we write `melee_port_crash.txt` but no `.dmp`; the patch keeps reports local. Posting Rust
  reports anywhere needs Hero's agreement and an engine name on his relay.
- **Launcher UI/packaging:** no GAME BUILD segment, no `FileVersion` resource on `melee.exe` (his version check
  skips it), no release-zip layout (`rust\melee.exe`) or updater channel, the in-game settings panel he opens
  (`--settings-window`) does not exist in ours.

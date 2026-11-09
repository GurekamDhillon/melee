# YouGame: the desktop app's bridge, and the Rust engine under it

Status 2026-11-08, branch `port/yougame`. Sources (read only; nothing copied, everything here is our own description):
Hero88go's melee-unlocked at tag v0.9.107 (`port/runtime/hle/yougame_*`, `port/runtime/host/window.cpp`,
`port/app/main.cpp`, `port/linux/embed`, `port/mac`), and Crunch's handoff of 2026-11-08
(`YOUGAME_FIXES_HANDOFF_1108.md`, Part 1, which quotes the app's side: `native.js`, `native-toplevel.js`,
`native-pad-relay.js`, `sandbox-policy.js`, app 0.13.1344). The app's own source was not seen: anything only the
handoff describes is marked **(handoff)**, anything neither source shows is marked **unverified**.

The YouGame plugin skills (`yougame:publish-to-yougame`, `yougame:yougame-sdk`) are for **browser** builds (static
upload, the JS SDK: leaderboards, saves, `YouGame.input`, multiplayer). None of it applies to a native exe: a native
build is described by the app's `yougame-native.json` manifest and talks over this stdio bridge. Nothing was uploaded
and no account was used.

## 1. Transport

- **Detection:** `YOUGAME_STDIO=1` in the environment. The app sets `YOUGAME_STDIO=1`, `YOUGAME_GAME` (the game id,
  read by neither engine) and `YOUGAME_DATA` (a writable per-game folder) on every launch (handoff, `native.js:3673`).
  Hero's engine also has a loopback-HTTP variant (`YOUGAME_BRIDGE` + `YOUGAME_TOKEN`, POST `<bridge>/sdk`,
  `Authorization: Bearer`), used by his own launcher (`MELEE_YOUGAME_LAUNCHER=1`); not needed for YouGame itself.
  He clears these variables after reading them so nothing the game starts inherits them.
- **Lines:** UTF-8, one per `\n` (a trailing `\r` is dropped), at most 1 MiB. **stdout carries bridge lines only**
  (the log goes to stderr or a file); each line is written with one write so nothing can land inside it.
- **Requests** (game -> app): `yg {"type":T,...,"id":N}`; **answers** (app -> game): `yg {"re":N,"ok":B,"result":{..}|"error":".."}`.
  A request with no answer by its timeout fails; stdin EOF fails them all.
- **Pushes** (app -> game): `yg {"push":{"type":T,...}}`.
- **EOF on stdin** = the app is gone: the game quits (exit 0).

## 2. Needed for standalone mode (own top-level window: where YouGame is going)

| line / variable | direction | meaning | Rust (this branch) |
|---|---|---|---|
| `yg {"caps":[...]}` | game -> app, first line | what the game supports. Hero always sends `ready, overlay, join, notice, pads, view, volume` (Windows/Linux). Crunch: when not embedded drop `pads, rumble, view, volume, fullscreen`; the app attaches its 125 Hz pad relay whenever caps contain `pads` (handoff, `native-pad-relay.js:227`) | `["ready"]` standalone; `["ready","overlay"]` embedded |
| `yg {"ready":true}` | game -> app | after the first **presented** frame (Hero: a visible game frame, never a loading panel). Without it the app stays on "Opening Melee Unlocked..." | sent once after the first successful present (our first present can be the black boot frame; see open points) |
| `embed quit` | app -> game | Stop. The app then kills after `QUIT_GRACE_MS` = 5 s (handoff). Hero honours it only through the player box's handler, so a non-embedded game ignores it today (Crunch's fix 2) | honoured always (also a bare `quit`); main loop exits, exit 0; a watchdog `process::exit(0)` at 4 s |
| stdin EOF | app -> game | the app quit / closed the pipe | same quit path |
| `yg {"key":"escape"}` | game -> app | Shift+Esc: the app brings its window forward for a separate-window game (handoff, `native.js:2862`) | Shift+Esc relayed, key swallowed |
| Shift+Tab | (key) | Hero: with stdio on, swallowed and sent as `yg {"overlay":"down"}` / `"up"` / `"cancel"` (focus lost). Crunch: when not embedded, open the game's own YouGame menu instead | own window: opens our settings overlay (one F1 press); embedded: overlay down/up/cancel |
| crash line | game -> app, **stderr** | last stderr line `crash: <reason>` (first clause, no paths, <=160 chars): the app's "stopped" notice | written on panic / fault when the bridge is on |
| exit code | | 0 clean; Hero exits 3 for a refused mod | unchanged: 0 clean, 3 refused launcher args, failure 1 |
| `YOUGAME_DATA` | env | writable per-game folder. Hero only reads test files from it; saves go to his `--card-dir`/`--user-dir`. **Unverified** that the app keeps it between runs | `MELEE_CARD_PATH=<data>\card`, `MELEE_SETTINGS_CFG=<data>\settings.cfg`, `MELEE_SLP_DIR=<data>\replays`, each only when not set already. Log stays on stderr |
| one V-sync setting | | Hero keeps a separate `vsync_yougame` (default off) for the player box; Crunch: drop it once standalone | we have only the player's own `vsync`: nothing to do |
| `yg {"type":"exitReason","reason":R,...}` | game -> app request | why the game is leaving; only `mod_outdated` seen (fields `mod`, `modVersion`: printable ASCII, no links, <=60 chars), 500 ms budget, any reply is enough | `yougame::exit_reason()` exists; **not called yet** (we have no mod-compat refusal through YouGame) |
| page pause | push `{"type":"pause","paused":B}`; game answers `yg {"event":{"type":"pause","pausable":B,"paused":B,"reason":R}}` | offline matches only; reasons `online`, `not-in-match`, `scene-changed` | not implemented (logged as an unhandled push) |

Manifest side (Crunch's fix 1, the app's `yougame-native.json`): no `"embed"` block and `"controllers":"none"`, and
the app spawning the exe directly, outside its AppContainer sandbox (app fix 1). Without the sandbox exemption a
game that drops `pads` gets no controllers at all (the AppContainer cannot open HID/USB). That is the app's and
Crunch's decision; our build only has to behave as above. The manifest's exact schema is **unverified** (not in
Hero's tree at v0.9.107).

## 3. Needed only for the player box (embedded, the current default)

- `YOUGAME_EMBED=1`. On Windows the app's **sandbox launcher** adopts the window from outside (`--embed-parent`,
  `--embed-class`, `--embed-title` are its arguments, not the game's, handoff `sandbox-policy.js:72-79`): it finds a
  window of class `MeleePortWindow` titled `Melee Unlocked`, sets `WS_CHILD` and `SetParent`. Hero's window is
  `WS_POPUP`, hidden until adopted / told `show`. Linux: `YOUGAME_EMBED_PARENT=<X11 window>`; macOS likewise
  (`YOUGAME_EMBED_PARENT`, an app older than 0.13.1048 sends none).
- Commands, `embed <cmd>` on stdin: `rect x y w h` (client rect in the parent), `view x y w h` (Windows: the
  visible part), `show` / `hide`, `focus` / `blur`, `volume v` (a master over the game's own), `quit`. Before the
  window exists only the latest of each kind is kept.
- `pads` cap -> the app's pad relay: `yg-pads {"seq":u32,"t":ms,"pads":[{"slot":0..3,"id","name","kind","buttons",
  "lx","ly","rx","ry","lt","rt"}]}` (XInput-like layout, <=2048 bytes, ~125 Hz), and a shared-memory GameCube adapter
  mirror announced by push `{"type":"pads","handle":N}`. Measured 3.9 ms median / 7.6 ms p95 added (handoff).
- `view`, `volume`, `rumble`, `fullscreen` caps: the page's controls over the box.
- D-pad + Start on a pad = the same overlay tap (100 ms down/up).

**Rust:** with `YOUGAME_EMBED=1` the window is created undecorated, hidden, class `MeleePortWindow` (winit
`with_class_name`), caption `Melee Unlocked`; `YOUGAME_EMBED_PARENT` (decimal HWND) makes it a child at creation
(**unverified**, the Windows app does not send it). `rect`, `show`, `hide`, `focus` are applied on the window thread;
`view`, `volume`, `blur` are logged and ignored. **Missing:** pad relay and adapter mirror (so no `pads` cap: inside
the sandbox an embedded Rust build has no controllers), `view` clipping, page volume, `WS_CHILD` adoption checks,
pad-chord overlay tap. Not worth doing while the direction is standalone.

## 4. Needed for online / ranked / friends (not implemented)

Requests seen (`yg {"type":T,...,"id":N}`): `hello` (answer: `player{id,name}`, `nameShared`, `netBroker{direct,...}`,
`behind{version}`), `mpCode` (connect code `TAG#123`), `mpTicket` (`players`, `mode` singles/teams, `lobby`,
`ranked`), `mpRank`, `mpFriend` (friend room picker), `directOpen` / `directAttach` / `directClose` (app-owned links
with virtual ENet addresses), `netOpen` / `netSend` / `netClose` (brokered datagrams), `muRestartOlder` (launcher
only). Pushes: `net`, `direct` (`link`, `event` up/close), `pads`, `pause`, and friends/room events. Room messages
over the app's WebSocket (or its broker): `matched`, `welcome`, `join`, `leave`, `away`, `back`, `msg` (`d.kind`
`mu-cand` v2 candidates), `start`, `result`, `ended`, `kick(ed)`, `error`. Match ids `yg-<hex64>`. Ranked = best of 3
per room, reported from the game-end record (winner, end method 1/2/7, LRAS port, placements, stage, frames,
characters, rng). Overlay notices (`notice`, `net`, `direct`, `join`/`back`/`welcome`) would be drawn by the game in
standalone mode (Crunch section 4, recommended). All of this targets Slippi's netplay inside Hero's engine; ours is
a different protocol (notes/net.md), so it is a separate project and needs Crunch's input on the broker.

## 5. Test

`python tools/rust-port/yougame/bridge_test.py [--exe build/cargo/release/melee.exe]`: plays the app over stdio
(no network, no account): stdout is bridge-only, caps first without pad caps, ready after the first present,
`embed quit` -> exit 0 within 5 s, stdin EOF -> exit within 5 s, data folders created. Unit tests:
`cargo test -p melee_platform --lib yougame`.

## 6. Open decisions

1. Window caption in YouGame standalone: ours stays `MELEE_WINDOW_TITLE` / the default; should it be "Melee Unlocked"
   (Hero's) or our own name? Taskbar grouping is by exe.
2. `ready` on the first present, which can be our black boot frame: fine for "Opening..." to end, but Hero waits
   for a visible game frame.
3. YOUGAME_DATA as the home of card/settings/replays: only if the app keeps it per player and across updates
   (unverified); else the install folder (may be read-only, hash-verified) is wrong too.
4. Which caps a Rust build should claim once we draw notices ourselves (`notice`, `join`).
5. Whether the Rust engine ever goes into YouGame's sandboxed player box at all, or only standalone (then section 3
   can be dropped).

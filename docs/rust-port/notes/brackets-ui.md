# Brackets: the in-game bracket flow (port/bracket-host, 2026-11-15)

Owner-approved flow, built in the bundled first-party mod `rust/mods/bracket` on the native bracket service
(`rust/melee/src/bracket`, wasm-ui-lab-api.md A6.2). Live set data needs the bracket server's (b) API; until then
`MELEE_BRACKET_MOCK=1` makes the host serve made-up data (`bracket/mock.rs`) in the same record shapes, so the screens
are done and the server only has to fill the lists.

## Look

The Party pack (envoy-v3-ui.md): sticker panels (white paper face, 3 px grape-ink outline, a hard grape drop shadow
5-6 px down-right, 16 px corners), sunny #FFD84A for "your move" and the hovered button, grape ink #2A1A6E for text
and outlines, cream #FFF6DC for quiet cards, coral #FF6B5B for struck stages and errors, lime #7ED957 for verified /
done. Bricolage Grotesque ExtraBold (`TEXT_DISPLAY`) for titles, labels and buttons, Source Sans 3 Bold (`TEXT_BODY`)
for body text, mono for codes and file names. Drawn with the host's shape ops (no baked art yet). 1280 x 720 layout,
scaled to the window; content column 820 wide.

## Screens and the UI_HOST data each needs

| # | screen | shows | lists read | requests sent |
|---|---|---|---|---|
| 1 | Main-menu tile | BRACKETS sticker bottom right with its Y key cap; a coral star badge and "Your set is up!" when a set is called | `BRACKET` field 8 (set called) | none (Y opens home, which sends STATUS + MY_SETS) |
| 2 | Brackets home (linked) | the next-thing card first: YOUR SET IS UP (event, round, opponent, [READY]); or YOUR NEXT MATCH (round, opponent, ETA, [REFRESH]); or "Nothing to play right now". Then MY EVENTS (name, status per event), then [BROWSE EVENTS] [ACCOUNT] | `BRACKET` (linked), `BRACKET_NEXT`, `BRACKET_SETS` (my events) | `MY_SETS` (refresh), `READY(set)`, `EVENTS` |
| 2b | Brackets home (signed out) | the "Link your account" card with the code entry instead of the next card and events; the two buttons stay | `BRACKET` (linked = 0) | `LINK(code)` |
| 3 | Account | key fingerprint, server, reachable, linked; Auto-upload toggle, Link code, Check server; the upload queue (file, state pill, target or reason) | `BRACKET`, `BRACKET_QUEUE` | `AUTO("1"/"0")`, `LINK(code)`, `STATUS` |
| 4 | Browse events | placeholder card until the server lists open events | (future: an events catalog) | `EVENTS` |
| 5 | Set: ready | the score bar (always on top in the set flow: round, both tags, score on a sunny pill, best of, game n); "READY! Waiting for your opponent" | `BRACKET_SET` | `READY(set)` (sent from home) |
| 6 | Set: character select | "Both players go to the normal character select" (the engine will drive the retail CSS; the mock's A stands in) | `BRACKET_SET` | mock: `MOCK_STEP("css")` |
| 7 | Set: stage strike (game 1) | the legal stages in a 3-wide grid, struck ones faded with a coral slash; "STRIKE 1 STAGE" / "STRIKE 2 STAGES" on sunny when it is your turn, "Peace is striking..." on cream when not; 1-2-1 | `BRACKET_SET` (turn, removals left, stages, gone) | `STAGE(i)` |
| 8 | Set: ban / pick (games 2+) | the winner bans 2, then the loser picks, same grid | `BRACKET_SET` | `STAGE(i)` |
| 9 | Set: game | "Game n on <stage>"; the engine starts the netplay game (mock: A stands in) | `BRACKET_SET` | mock: `MOCK_STEP("game")` |
| 10 | Between games | about 5 s result card: who took it, the score, the winner's stocks and percent, what's next ("You pick, Peace bans 2"); then 8 | `BRACKET_SET` (last game, next) | none (timed by the server / mock) |
| 11 | Set end | FINAL, SET WON / SET LOST, the score, "checking the replays..." then "✓ verified", NEXT (the next set and opponent with an ETA, or the drop to Losers), [BRACKETS HOME] | `BRACKET_SET` (verified, next), `BRACKET_NEXT` | none |

Every screen has the status strip at the bottom (the last request's answer, `STATUS`, coral when it is an error).

## Records (gd-ui-lab-v1.h)

- `BRACKET` (9): fingerprint, linked, reachable, server, auto-upload, event, message, error, **set called** (new, 8).
- `BRACKET_SETS` (11): my events: event id, name, status line.
- `BRACKET_NEXT` (12): kind (`called` / `playing` / `upcoming` / `none`), event name, round, opponent, ETA seconds,
  set id, best of.
- `BRACKET_SET` (13): phase (`called`, `wait_ready`, `css`, `strike`, `ban`, `pick`, `game`, `result`, `done`), my
  score, opponent score, best of, game n, my tag, opponent tag, my turn (0/1), removals left, stage names (`|`), gone
  per stage (`0`/`1`), stage picked (index), last game (`won|stocks|percent`), verified (0/1), what's next, round.
- Requests: `READY` (16, set id), `STAGE` (19, stage index: a strike, ban or pick by phase), `EVENTS` (20),
  `MOCK_STEP` (21, `css` / `game`, mock only).

## What the server needs to provide ((b) API), per screen

1. **Link**: a route that binds this key to an account by a short code (LINK), and "linked" in an answer the client
   can poll (or `/v1/me`).
2. **My events**: the events this key is entered in, each with a status line (signed up / checked in / playing round
   X / eliminated, placement).
3. **Next thing**: per event, or across events: called set (event, round name, opponent tag, set id, best of) or the
   next set with an **ETA** (seconds) and opponent when known.
4. **Ready**: `POST` ready for a set; the set's state shows when both are ready.
5. **Stage flow state** for the current game: the legal stage list for this game (starters for game 1, all legal for
   later games), whose turn, how many removals are left, which stages are gone, the stage picked; `POST`
   strike / ban / pick with a stage. Server-side rule: game 1 strike 1-2-1 on the starters (who strikes first: the
   higher seed, or a coin flip); games 2+ the previous game's winner bans 2, the loser picks. The server should
   refuse an out-of-turn or not-open stage (409).
6. **Game result** as soon as both replays are in: who won, the winner's stocks and percent, and what's next.
7. **Set end**: the final score, a **verified** flag once the re-simulation agrees, and the next set (round, opponent,
   ETA) or the drop to Losers / elimination.
8. Push or poll: the client polls (`/v1/me/next` pattern) every couple of seconds while the screen is open; a
   long-poll or an ETag would save requests.

## Decisions and cuts (to revisit)

- **The tile is the mod's sticker in the bottom-right corner of the main menu, not a retail menu row.** The Rust
  main menu has no ONLINE tile (retail rows plus Envoy's sixth row); a real BRACKETS row would be another
  `mnmain` sixth-row job like Envoy's (`envoy_main6`). The sticker reads as a tile and carries the star badge.
- **Character select and the game are stand-ins in the mock** (A advances). Driving the retail CSS and starting the
  netplay game from the set flow is engine work that needs the server's room / relay data per game.
- Game 1 strikes on five starters in the mock (Battlefield, Final Destination, Dreamland, Yoshi's Story, Pokemon
  Stadium; Fountain of Dreams a counterpick) so 1-2-1 ends on one stage. The real list is the event's
  `legal_stages` and whatever starter list the server sends.
- The result card's 5 s and the "verified" delay are the mock's timers; live, the server's state drives them.
- Opening the set flow: READY on home goes straight to the set screen; reopening the screens mid-set returns to it.

## Live data: built against the server's phase-2b contracts (melee-autobracket phase1 2fb5489)

This replaces the "what the server needs" list above (kept for the history). `rust/melee/src/bracket/ingame.rs` reads
`proto::ingame` (docs/api.md section 8) with the engine's JSON reader and builds the same records the mock serves:

| screen need | route | notes |
|---|---|---|
| linked, my tag | `GET /v1/me` (MeStatus) | every 60 s while a screen watches; badge / home / flow wait until the key is a player |
| LINK(code) | `POST /v1/me/link` (LinkReq -> LinkResult) | the answer's tag goes on the status strip |
| tile star | `GET /v1/me/badge` (Badge) | every 10 s on the main menu, `If-None-Match` with the last ETag (304 = unchanged); only once a key exists (a fresh install never calls the server until Brackets is opened) |
| home | `GET /v1/me/home` (Home) | every 3 s on home and in the set; `HomeAction` -> BRACKET_NEXT (`SetUp` -> called / playing by `ready[0]`, `Upcoming` with `eta_secs`, `Review`, `None`); `MyEvent` -> MY EVENTS with the reference client's status words |
| READY | `POST /v1/sets/{e}/{s}/ready` (ReadyReq, our engine build) | the set is the home's `SetUp` |
| set flow | `GET /v1/sets/{e}/{s}/flow` (SetFlow) | every second in the set; `phase` + `stage.turn` -> strike / ban / pick / game / judging / review / done; `games` -> the result card (held 5 s after a game is judged) with `winner_stocks` / `winner_percent`; `outcome` + home's next set -> NEXT; `relay` / `room_code` on the game card |
| STAGE(index) | `POST /v1/sets/{e}/{s}/games/{n}/stage` (StageMoveReq: `kind` from the turn, `stage` = the pool's stable id) | the mod sends an index into the pool; the host sends the id |
| auto-upload target | `GET /v1/me/home` `SetUp` (event, set, game) | when `bracket_event` is not set; `/v1/me/next?event=` otherwise |

`BRACKET_WATCH` (22, "0" none / "1" menu / "2" home / "3" set) tells the host what is on screen; the worker polls
only for that. Not wired yet: starting the netplay game from the set flow (relay + room code are shown), JOIN,
browse events (no public listing route in 2b for the client), the live relay's send side. Tests: `ingame.rs` on
api.md's examples; `bracket::tests::ingame_routes_local_server` against a loopback server built from those types
(link, ready, flow, a strike by id, the badge's If-None-Match / 304). Not yet run against the real server: its
handlers are wave 2.

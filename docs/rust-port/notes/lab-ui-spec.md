# LAB: product and UI spec (M0-M2 implemented, M4 HUD part)

This is the product and UI spec for the new LAB. The LAB is a wasm presentation mod on **gd-ui-v1 / gd-lab-v1**
(`wasm-ui-lab-api.md`, including its 2-player amendment). This is a fresh design. GD's old LAB
(`pc/geno/mods/geno-lab`) is used only as a feature checklist: none of its art, palette, icons or layout is
reused. The visual source is our own design canvas (Kit, Styles, Hud boards). It contains no disc data. Values
below are copied from it exactly; it is not copied into the repo.

## Status: the LAB mod (`rust/mods/lab`)

| milestone | state |
|---|---|
| M0 | GAP-0, 13, 14 were in the API already; added: `gd_lab_applied` (who applied which command, for "P2 rewound" toasts online), untimed lab matches (`melee_core::codes::lab_untimed`, cleared in `fn_8016E730`, unless `--time-secs`), START owned by the Lab (stripped from the sim's pads in `LabRun::tick` unless L+R+A are held). GAP-17 fonts: the parallel font work in `melee_gx` `ui.rs`; the mod uses only the text roles, so it picks the fonts up unchanged |
| M1 | parts 1 panel, 2 chip, 3 toast, 5 readout list, 7 button hint, theme A tokens, Clean preset |
| M2 | START freeze, frame step (D-left / D-right, hold to repeat; L / R 10 frames), A take over, B resume, scrubber (part 8: played part, playhead, save diamonds, hit dots from `gd_lab_events_range`), D-down save / D-up load to the current slot, Z + D-left / D-right slot pick, slot cards (part 12, no thumbnails: GAP-3) |
| M4 (part) | frame meter (part 4, stun hatch), move readout (from the observed frame data of P1's last move with a hitbox), state readout (action names from the decomp's ftCommon enum, `tools/rust-port/modhost/lab_names.py`), dummy panel (defaults until the M3 menu), combo panel, input display (part 6, numpad history); Frame data and Full presets; Z + D-up cycles the preset |
| online | freeze / rewind / step / load apply to both players (no consent prompt, open question 4 decided); the other player's action shows as a toast |

Layout: the presets are laid out on 1280x720 and scaled to fit the window (`SPACE_WINDOW`); a 4:3 window gets the
extra height, the frozen widgets sit above the percent-row keep-out. Measured in the game (1280x960 window, retail
flow, Full preset with the frozen scrubber part of the run): gd_ui present p50 63 us, p99 157 us (all UI host work plus
the mod), 1 UI draw call per present, about 1500 quads; 0 allocations in the mod (no_std, no alloc).

## 1. Product

| decision | meaning |
|---|---|
| audience | all players, from someone learning to L-cancel to a player labbing frame traps. Defaults are readable without reading the docs, and every number is one press away |
| bar | first-in-class: everything SF6 / Tekken 8 training mode, UnclePunch TM / TM-CE and the 20XX Training Pack offer, plus a two-player and online lab that none of them have |
| two players | P1 and P2 can both be humans (local, or online through `LabNetplay`), or one human and a dummy. Either player can open the menu; settings are per port |
| online | every lab action that touches the sim (save, load, rewind, step, record, play, CPU or reaction changes, percent and position writes, drill start) is a **lab command** sent in the lockstep input stream and applied by both peers on the same frame. Presentation-only actions (HUD, theme, menu navigation, readouts) stay local |
| v1 scope | everything in section 2 |
| performance | budgets from `wasm-ui-lab-api.md` section 0 are acceptance criteria: Lab overlay plus panels <= 0.25 ms CPU per present, one `gd_ui_submit` per present, <= 2 UI draw calls, zero steady-state allocations. All four themes go through the same batched quad + atlas path (section 7) |

## 2. Feature areas and API mapping

Legend for the API column: `ui.*` = `gd_ui_*`, `lab.*` = `gd_lab_*`. **GAP** = not in the API note yet (section 3).
"cmd" = becomes a lab command online.

### 2.1 Time control

| feature | behaviour | API |
|---|---|---|
| freeze | one button (default START) freezes the game and shows the timeline scrubber (part 8). Press again to resume | `lab.pause` (cmd) |
| scrub | drag or step the playhead over the last 600 frames; marks show hits, inputs, saves, L-cancels | `lab.ring(600)`, `lab.rewind(n)`, `lab.events` per frame cached by the mod; marks need **GAP-1** (event history) |
| frame advance | step +1 / +10, hold for slow advance; step back -1 / -10 | `lab.step(n)` (cmd) |
| take over (replay takeover) | from any scrubbed frame, press a button to stop replaying logged inputs and play live from there | `lab.go_live` (cmd) |
| slow motion | 1/2, 1/4 speed | **GAP-2** (no speed call) |
| full menu | a separate button (default hold START 0.5 s, or SELECT/Z+START on a keyboard) opens the tabbed menu; the game stays frozen while it is open | `ui.show`, `ui.capture(EXCLUSIVE)` on the opener's port only |

### 2.2 Save states and position reset

| feature | behaviour | API |
|---|---|---|
| save / load | D-pad down saves, D-pad up loads (the 20XX / UnclePunch habit). Slots 1-16 with slot cards (part 12) showing a thumbnail, name and length | `lab.save/load(slot)` (cmd), `lab.slot_frame`; thumbnails need **GAP-3** |
| position reset | SF6-style: a button resets both fighters to a chosen layout (centre, left ledge, right ledge, platform, swap sides), at chosen percents | `lab.set_pos`, `lab.set_percent` (cmd); facing and action reset need **GAP-4** |
| per-character / per-stage state | states remember character and stage; loading one with a different matchup offers to switch | `ui.match_start` |

### 2.3 Dummy

| setting | options | API |
|---|---|---|
| control | stand, crouch, jump, shield, CPU level 1-9, recording, human (2P) | `lab.cpu(port, behaviour, param)` (cmd) |
| block | none, all, after first hit, random (SF6/T8 style); shield hold time | `lab.reaction(SHIELD, ...)` - first-hit and random-block policies need **GAP-5** |
| after block / after hit / on wakeup | action from a list or a recording slot: OoS (shine, up-B, nair, grab, jump), spot-dodge, roll, jab, recording | `lab.reaction(OOS / SHIELD_DROP / GETUP, ...)`; "play recording N as reaction" needs **GAP-5** |
| DI | none, in, out, away, toward, random, survival, combo; strength 0-100% | `lab.reaction(DI, ...)`; strength and angle need a param, **GAP-5** |
| SDI | none, in, away, random; frequency (every frame, every 2, ...) | `lab.reaction(SDI, ...)` |
| tech | in place, roll in, roll away, miss, random; weights per option | `lab.reaction(TECH, ...)`; weights **GAP-5** |
| getup / ledge | getup attack, roll, jump, stand, ledgedash; ledge options with delay | `lab.reaction(GETUP / LEDGE, ...)` |
| counters | out of shield and out of hitstun, with a frame delay | `lab.reaction(OOS, ...)`, delay **GAP-5** |
| percent | start percent slider, lock percent, reset on load | `lab.set_percent` (cmd) |
| intangible / infinite shield | toggles | **GAP-6** |

### 2.4 Recording

| feature | behaviour | API |
|---|---|---|
| slots | 8 recording slots per port, each with optional start state | `lab.rec_start/stop/len/free` (cmd), `FROM_SLOT` |
| playback order | ordered, random, or weighted (weight 0-9 per slot, shown on the card as x2) | the mod picks the slot; random must draw from the game RNG online: **GAP-7** |
| mirror | play a recording for the other side | `lab.play(MIRROR)` |
| loop and reset | loop with a position reset between loops | `lab.play(LOOP)`, `lab.load` |
| take over | interrupt playback with your own input at any frame; record the rest | `lab.play_stop`, `lab.rec_start` |

### 2.5 Readouts and frame data

| feature | behaviour | API |
|---|---|---|
| frame meter (part 4) | one row per player, one cell per frame: idle, startup, active, recovery, stun (hitstun/shieldstun/hitlag), can act (IASA). The advantage number shows who is ahead | `lab.fighter` per frame, `lab.frame_adv`; a per-frame cell kind would avoid the mod re-deriving it: **GAP-8** |
| move readout | startup, active, landing lag (L-cancelled), damage, IASA, on-shield | `lab.framedata`, `lab.hitboxes`, `lab.attr` |
| state readout | action name, frame n/m, speed X/Y, hitlag, hitstun, shield HP, jumps, ECB | `lab.fighter`; action names need **GAP-9** |
| combo counter | hits, total damage, move chain | `lab.events` |
| input display (part 6) | stick, held buttons, scrolling history with frame counts, and timing feedback ("A 3f after landing") | `ui.capture(SHARED)` + `ui.pad`, `lab.events` |
| IASA / L-cancel flash | the fighter flashes for one frame when it can act, or on an L-cancel success/fail | `lab.overlay(IASA)`; a coloured fighter tint needs **GAP-10** |
| world overlays (part 15) | hitboxes, hurtboxes, ECB, ledge grab, DI line and knockback arc, blast zones | `lab.overlay`, `lab.overlay_style`; DI line/arc **GAP-11** |

### 2.6 Feedback pop-ups (toasts, part 3)

L-cancel (frame and window, `5/7f`), tech (early/late by n frames), OoS (frame acted), powershield, SDI count,
wavedash angle and airdodge frame, ledgedash GALINT. Source: `lab.events`. Powershield, wavedash, ledgedash and
GALINT events are not in the event list: **GAP-12**.

### 2.7 Drills

Each drill is a preset of dummy, position reset, success rule and a score panel; it starts as a cmd.

| drill | success rule | needs |
|---|---|---|
| L-cancel | L/R within 7 frames before landing an aerial | `lab.events` |
| ledgedash | GALINT >= 1, shown per attempt | GAP-12 |
| wavedash | airdodge frame after jumpsquat and angle | GAP-12 |
| SDI | count of SDI inputs per multi-hit move | `lab.events` SDI, GAP-12 |
| tech chase | dummy techs randomly (weighted); score punishes | `lab.reaction(TECH, RANDOM)` |
| edgeguard | dummy recovers with random option from a ledge reset | `lab.cpu`, GAP-5 (recovery options) |
| powershield | dummy fires a projectile or recording; hit the 2-frame window | `lab.play`, GAP-12 |
| amsah reaction | dummy hits at chosen percent; ASDI down + tech | `lab.reaction`, GAP-12 |
| punish training | SF6 style: dummy plays a random recording; the HUD marks punishable windows and scores the punish | recordings + `lab.frame_adv` |
| combo trials | ordered move list; each step ticks when hit in sequence | `lab.events`, `lab.framedata` |

### 2.8 Session

Character and stage toggles (switch either player's character or the stage without leaving the lab, via
`ui.match_start`), swap ports, theme, HUD preset, save/load settings per character, online status (peer, delay,
lockstep), help, quit to CSS.

## 3. API gaps

Closed by the gd-lab-v1 implementation (`wasm-ui-lab-api.md`, "Status: half B"): GAP-1, GAP-5, GAP-7, GAP-8, GAP-11
(`DI_LINE`; `KB_ARC` not drawn yet), GAP-12, GAP-13 (with GAP-0 and GAP-14 from half A). The signatures are in
`docs/specs/gd-ui-lab-v1.h`.

| id | gap | proposal |
|---|---|---|
| GAP-0 | **window-space panels.** The API's canvas is the 640x480 4:3 band, but the HUD presets place panels in the 16:9 side bars (x < 160 or x > 1120 at 1280x720) | add a `LAYER` flag or a `SPACE` record: `BAND` (default) or `WINDOW` (pixels of the window, with the band rect queryable through `ui.band_rect`) |
| GAP-1 | event history for timeline marks | `lab.events_range(from, to, out, max)` from the ring |
| GAP-2 | slow motion | `lab.speed(num, den)` (cmd) |
| GAP-3 | slot thumbnails | `lab.slot_thumb(slot) -> tex`, rendered at save time at 96x48 |
| GAP-4 | reset with facing and action | `lab.set_facing`, `lab.set_action(port, WAIT/FALL)` (cmd) |
| GAP-5 | reaction parameters | extend `lab.reaction(port, kind, option, param)` with policy (first hit, random %), delay frames, DI strength/angle, weighted option tables, "play recording N" |
| GAP-6 | intangible, infinite shield, infinite jumps | `lab.cheat(port, bits)` (cmd, offline + LabNetplay) |
| GAP-7 | lab-side random choices online | `lab.rng() -> u32` from a lab RNG seeded through the lockstep stream, separate from the game RNG so choosing a slot never changes a match hash path that is not a cmd |
| GAP-8 | frame cell kind | `gd_lab_fighter_v1.cell` (idle/startup/active/recovery/stun/can-act) |
| GAP-9 | action-state names | `lab.action_name(char, action, out, len)` |
| GAP-10 | fighter tint flash | `lab.tint(port, rgba, frames)` presentation-only |
| GAP-11 | DI line and knockback arc | `LAB_OVERLAY` bits `DI_LINE KB_ARC` |
| GAP-12 | technique events | `lab.events` kinds `POWERSHIELD WAVEDASH(angle, frame) LEDGEDASH(galint) SDI ASDI AIRDODGE` |
| GAP-13 | lab command transport | the amendment says commands go through the lockstep stream; the API has no `lab.cmd` encoding, version or max size. Propose: every cmd-marked call above is queued host-side as a fixed 16-byte record and applied at frame N+delay on both peers; the call returns the apply frame |
| GAP-14 | keyboard text input | `ui.capture(KEYBOARD)` is offline only; the command bar must work in `LabNetplay` too. Allow `KEYBOARD` in `LabNetplay` since it is local-only |
| GAP-15 | free-form widget drag (mouse) | the HUD editor needs pointer position and buttons: `ui.pointer(out)` |
| GAP-16 | layout persistence and sharing | `ui.store_get/put(key, ptr, len)` in a per-mod settings dir; share as a short text code |
| GAP-17 | theme fonts | Inter, JetBrains Mono, Rubik, Space Grotesk, IBM Plex Mono, Orbitron (all OFL) must be packaged and loadable: `ui.font_load(path) -> role` |

## 4. Pausing and the menu

- **Freeze** (default START tap): game frozen, scrubber at the bottom of the band, button hints update
  (part 7). D-pad left/right steps, L/R steps 10, A takes over, B resumes.
- **Full menu** (default START hold): tabs **Dummy, Record, Display, HUD, Drills, Session** (part 9, L/R to switch),
  setting rows (part 10: toggle, choice, slider, action) and a help pane (part 11) explaining the focused row.
  Built with `ui.node` (`TABS LIST TOGGLE CHOICE SLIDER BUTTON`), so focus, wrap and repeat come from the host.
- **Radial quick menu** (part 13, default hold Z): 8 actions on stick directions, release to run. Defaults
  clockwise from up: Save, Load, Reset, Record, Play, Mirror, Swap, Menu. Each of the 8 is customisable from a list
  of every menu action. Drawn with `RECT`/`TEXT` records, not nodes.
- **Command bar** (part 14, keyboard only, default `/`): type part of a setting or action, arrows pick, Enter runs.
  Every row and action has a searchable name ("dummy tech random").
- Online, both players can open menus independently. Only actions marked cmd are sent; a peer sees a toast
  "P2 loaded slot 3".

## 5. In-match HUD

### 5.1 Presets (1280x720, 4:3 game area x 160-1120)

| widget | Clean | Frame data | Full | position (x, y, w) |
|---|---|---|---|---|
| session chip (`LAB · preset`, online `2P online · 48 ms`) | yes | yes | yes, with net | 20, 20, auto |
| toasts | yes | yes | yes (+ secondary line) | 490, y 40 (Clean) / 120, 300 wide, centred |
| frame meter (50 cells) | - | yes | yes | 290, 20, 700 |
| move readout (6 rows) | - | yes | yes | right 20 (1040), 20, 220 |
| state readout (5 rows) | - | - | yes | 1040, 236, 220 |
| dummy panel (5 rows + playback line) | - | - | yes | 20, 70, 200 |
| combo panel | - | - | yes | 20, 300, 200 |
| input display | - | - | yes | 20, bottom 20 |
| button hints | START only | - | START, D-down, D-up, Z hold | right 20, bottom 22 |
| world overlays | off | off | hitboxes on | in world |

### 5.2 Keep-out zones

Panels never cover the retail percent/stock row or the timer. Keep-out rects in 1280x720 (to be checked against
the retail HUD positions in the Rust build):

- percent and stock row: x 160-1120, y 560-720;
- timer: x 520-760, y 0-110, only when the match has a timer. Lab matches are untimed by default; when a timer is
  on, the frame meter's default moves to y 120.

The HUD editor refuses to drop a widget into a keep-out rect (it snaps to the nearest legal cell).

### 5.3 HUD editor (part 16)

From the HUD tab: every widget gets a frame, a name tag, a move handle and a resize corner. Drag with the stick
(or mouse, GAP-15), resize with the C-stick, snap to an 8 px grid (toggle 4/8/16/off). A widget picker adds or
removes any of the widgets above, and more than one of a kind is allowed (e.g. a second readout for P2). Layouts
are saved per name and shared as a text code (GAP-16). The editor is local only.

## 6. The parts (UI kit)

All screens and the HUD are built from these 17 parts. The look comes only from the theme tokens (section 7).

| # | part | contents |
|---|---|---|
| 1 | Panel | see-through box holding parts; title and drag grip; draggable/resizable in the editor |
| 2 | Chip / badge | tiny status label: mode, preset, connection, slot in use (`LAB`, `Preset · Full`, `2P · 48 ms`, `● REC 2`) |
| 3 | Toast | short feedback after an action (success, miss, a number); stacks, then fades (opacity 1, 0.75, 0.5) |
| 4 | Frame meter | one row per player, one cell per frame (cells 9x14 HUD, 8x14 compact, gap 2); kinds idle, startup, active, recovery, stun, can act; advantage number |
| 5 | Readout list | name / value rows |
| 6 | Input display | stick circle (56 px, dot 10 px), A B X Y L Z keys, history with frame counts |
| 7 | Button hint | key cap + action; changes with the moment (playing, frozen, menu) |
| 8 | Timeline scrubber | 18 px track, played part, 2 px playhead, diamond (save) and dot (hit) marks, `-120f / frame N / now` |
| 9 | Tab bar | tabs between L and R key caps |
| 10 | Setting rows | toggle, choice (`◀ x ▶`), slider (80x4 track + value), action (`A ›`) |
| 11 | Help / detail pane | title and plain-words explanation of the focused row, optional preview |
| 12 | Slot cards | 96 px wide: thumbnail 48 px tall, name, length and weight (`2.1 s · ×2`); empty slot dashed at 0.55 opacity |
| 13 | Radial quick menu | 200 px ring, 8 labels at radius 78 (52 px wide), centre knob 32 px |
| 14 | Command bar | prompt line (`› text▍`), filtered result rows, highlighted current |
| 15 | World overlays | hitboxes, hurtboxes, ECB, DI line, ledges, drawn on the world, not in a panel |
| 16 | HUD editor handles | dashed frame, name tag, move handle, 10 px resize corner |
| 17 | Player marker | who controls what in 2P: `P1 · you`, `P2 · friend`, `P3 · dummy`, `P4 · ▶ rec 2` |

## 7. Themes

A theme is a token set applied to the same 17 parts. **A Instrument** is the default; **C Arcade**, **E Brutalist**
and **G Neon** ship as alternatives (B Broadcast, D Glass, F Retro CRT and H Paper were not picked).

### 7.1 Base tokens

| token | A Instrument | C Arcade | E Brutalist | G Neon |
|---|---|---|---|---|
| panel bg | `rgba(8,11,17,0.85)` (HUD 0.82) | `#ffffff` | `#ffffff` | `rgba(20,10,40,0.85)` |
| border | 1px `#2b3546` | 3px `#241d4a` | 3px `#000000` | 1px `#ff3ea5` |
| radius | 4px | 14px | 0 | 6px |
| shadow | none | `0 4px 0 #241d4a` | `6px 6px 0 #000000` | `0 0 14px rgba(255,62,165,0.45)` |
| text | `#e8edf5` | `#241d4a` | `#000000` | `#f6e9ff` |
| muted | `#8d9ab0` | `#6b63a8` | `#444444` | `#b59ad6` |
| accent | `#7ee0c3` | `#3ddc97` | `#ffe500` | `#00f0ff` |
| accent ink (text on accent) | `#06120e` | `#241d4a` | `#000000` | `#0b0716` |
| UI font | Inter | Rubik | Space Grotesk | Orbitron |
| number font | JetBrains Mono | Rubik | IBM Plex Mono | Orbitron |
| base size | 13px | 13px | 13px | 13px (Orbitron set small) |
| cell radius | 1px | 4px | 1px | 1px |
| toast border | 1px accent | none | 3px `#000` | none |
| key / button radius | = radius | 50% (round) | 0 | 6px |
| canvas bg (preview only) | `#1b2230` | `#3a3170` | `#d8d8d8` | `#0b0716` |

Theme A, HUD extras from the Hud board: row divider `#222b39`, key-cap border `#3a475c`, key text `#c7d0de`,
P1 colour `#c9504a`, P2 colour `#4a78c9`, panel padding 10-14 px.

Type scale (all themes, from base size `fs`): label = fs-2 uppercase, tracking 0.06-0.08em, muted; value and row
= fs-1; big number = fs+5 bold, accent (E: text colour); toast = fs bold, accent bg and accent-ink text, number
font. Active tab and focused row: accent bg / 4px accent left border. Input display stick ring 2px muted, dot
accent; pressed key accent bg.

### 7.2 Frame-meter cells

| kind | A Instrument | C Arcade | E Brutalist | G Neon |
|---|---|---|---|---|
| idle | `#2a3344` | `#e6e1f7` | `#eeeeee` | `#231638` |
| startup | `#e0b84a` | `#3ddc97` | `#000000` | `#ffd23f` |
| active | `#e0564e` | `#ff5d73` | `#ff2e2e` | `#ff3ea5` |
| recovery | `#4a7fe0` | `#4fb6ff` | `#777777` | `#00f0ff` |
| stun | `#9a6ee0` | `#b18cff` | `#ffe500` | `#9b5cff` |
| can act | `#7ee0c3` | accent `#3ddc97` + 1px border | accent `#ffe500` + 1px border | accent `#00f0ff` + 1px border |

Only A defines can-act on the canvas (`#7ee0c3`). For C, E and G it is the accent plus a 1px text-colour
outline so it stays distinct from startup (C) and stun (E). For colour-blind players, stun cells also get the
diagonal hatch from the Kit board (two-tone 2 px stripes) in every theme.

### 7.3 One render path

Every token is a colour, a number or a font role. Radius, borders and shadows are drawn as quads: rounded corners
come from a 9-slice corner texture per radius (4, 6, 14 px) in the shared atlas, borders are 4 thin quads (or
part of the 9-slice), offset shadows are one extra quad, and glows (G) are a pre-blurred 9-slice texture. No
theme uses `VECTOR`, so Vello is never needed, and a theme switch only changes record colours and slice ids.
Glyphs for both theme fonts go in the same atlas, so a frame is still one batch.

## 8. Performance

- One `gd_ui_submit` per present; the HUD encodes into a reused buffer. Readouts re-encode text only when a
  value changes (cached strings by value).
- Frame meter: 2 rows x 50 cells = 100 `RECT`s in a ring; per frame it shifts one column, no allocation.
- Queries per present: `lab.fighter` x2, `lab.events`, `lab.frame_adv`, `ui.pad` x2: 6, inside the ~10 limit.
  `lab.framedata` only on a new move.
- Menu uses the host node tree (laid out once, dirty-only).
- Bench: the API note's Lab scene (every overlay on, Full preset) in each of the four themes; all four must meet
  0.25 ms and <= 2 draw calls.

## 9. Open questions

1. Retail percent/stock and timer positions in the Rust build: confirm the keep-out rects in 5.2.
2. Default freeze vs menu button on a GameCube pad: START tap / START hold, or START / Z+START? Hold adds 0.5 s
   latency to the menu.
3. Should the radial use Z (conflicts with grab during play) or a D-pad direction? D-pad down/up are save/load.
4. Online: may one player freeze or rewind for both, or does it need the other's agreement (a "P2 wants to load
   slot 3" prompt)?
5. Lab commands share the lockstep delay; is rewind/scrub online acceptable at 48 ms, or is it local-preview then
   commit?
6. Ship the four theme fonts in the exe or the mod package (size)?
7. GAP-7: a separate lab RNG, or is drawing from the game RNG under a cmd acceptable?

## 10. Implementation order

| milestone | content | depends on |
|---|---|---|
| M0 | API amendments for GAP-0, 13, 14, 17 (window space, lab cmd transport, keyboard, fonts) in `gd-ui-lab-v1.h` | API A stubs |
| M1 | kit parts 1-3, 5, 7 and theme A tokens through the batched path; Clean preset; bench | M0 |
| M2 | freeze, step, scrubber (8), save/load on the D-pad, slot cards (12) | API B1 |
| M3 | full menu: tabs (9), rows (10), help (11); Dummy tab with B3 behaviours and reactions | API A2, B3, GAP-5 |
| M4 | frame meter (4), readouts, input display (6), toasts; Frame data and Full presets | B5, GAP-8, 9, 12 |
| M5 | recording slots, ordered/random/weighted playback, takeover, mirror | B2, GAP-7 |
| M6 | world overlays (15), IASA/L-cancel flash | B4, GAP-10, 11 |
| M7 | drills, punish training, combo trials | M4, M5 |
| M8 | two players local, then online (`LabNetplay`, player markers 17) | GAP-13 |
| M9 | radial (13), command bar (14), HUD editor (16), layout sharing | GAP-15, 16 |
| M10 | themes C, E, G; four-theme bench | M1 |

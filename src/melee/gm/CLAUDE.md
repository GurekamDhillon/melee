# src/melee/gm: game modes and the port's menus

Upstream's `gm*.c` are the decomp's game-mode and scene files; keep their style and put port changes
under `TARGET_PC`. The port's own menus are `gmfrontend.c` and its `.inc` files (one translation
unit): the 60-line comment at the top of `gmfrontend.c` explains how a frontend screen is placed in
Melee's mode flow and is the place to start. Design notes and the art still expected:
workspace `_research/frontend-menus.md`; the art itself: workspace `menu/`.

## The files

| file | what |
|---|---|
| `gmfrontend.c` | the toolkit: `FrontendItem` rows (action / choice / slider / toggle; read-only rows are a slider with `set = NULL` and a `format`), `FrontendScreen`, routing, `fe_switch_screen`, `fe_ol_notice` |
| `gmfrontend_menus.inc` | the menu tree (`fm_menus[]`, `FeMenuItem` with `FA_SUB / FA_MODE / FA_NATIVE / FA_MATCH_SETUP / FA_ONLINE / FA_PAGE`), confirm → `fm.pend_kind` → `fm_do_pending` after the fade; `fm_first_boot` |
| `gmfrontend_settings.inc` | SETTINGS pages: `FSP_*` enum, `fe_screen_settings[]`, one `fe_items_set_*[]` per page. A new page = an enum value, a table, a screen entry, a `FA_PAGE` menu row |
| `gmfrontend_controls.inc` | controller remap editor (included by settings): timed press/release capture, Swap/Also, profiles, presets and mapped tester; original layout navigates this page |
| `gmfrontend_online.inc` | the room screens (`art != 0`), the lobby, strikes |
| `gmfrontend_select.inc` | the kit's character and stage select |
| `gmfrontend_kit.inc`, `_kitlist.inc`, `_player.inc` | drawing: the font atlas and palettes (`kit.json`), the row list (`list_layout.json`, `widgets_layout.json`), the layout/motion player |
| `gmfrontend_mouse.inc` | the mouse in the menus |
| `gmfrontend_atlas_online.inc` | the Atlas drawing of the room screens: copies legacy predicates into the host's room view by name, turns mouse and keyboard intents into the same MenuInput bits; reads netplay state, writes none (`tools/port/check_atlas_online.sh`) |
| `gmfrontend_atlas.inc` | the Atlas adapter for the menu tree: `FeMenu.atlas_id`, `fa_frame` / `fa_sync`, the More strip, the scene policy's stand-in hook (see "Atlas" below) |
| `gmfrontend_atlas_data.inc` | the Atlas data screens (Event Match, the Name Entry tag list, Sound Test, Special Messages, Bonus, Misc. and VS. Records) and the Results stand-in: opened from the `FA_NATIVE` row that opens the retail screen (`fad_open_for` in `fm_confirm`), the adapter owns the cursor and a 32-row window, retail text is decoded per string at run time and never stored (see "Data screens" below) |
| `gmscmemcard.c` | the memory-card prompt, the boot blocker; the port's skip and auto-create are here |

## Rules

- A settings row needs no drawing code: the table is the screen. Values are re-read every frame
  (`fk_frame`), so a `format` callback can show live data (the CONTROLS port rows do).
- Native code is called through unprefixed externs declared at the top of the `.inc`
  (`Settings_Int`, `gc_adapter_present`, `Pad_Value`...); the shim defines `gw_<name>`.
- Strings the player sees are plain English in the tables; the launcher, not the game, is
  translated.
- `OSReport("frontend: ...")` on every navigation: the scene trace is how a headless run is read.
- Env switches: `MELEE_FRONTEND_MENUS` (the tree, default on), `MELEE_NATIVE_CSS`,
  `MELEE_FE_HUBDEMO`, `MELEE_NO_ONBOARD`, `MELEE_ATLAS` (`0` = the legacy menus and the retail title), `MELEE_ATLAS_SCENES`
  (`<kind>:<retail|overlay|replace>,...`, a development override of the scene policy).
- A change to what the online screens show goes in the adapter (`gmfrontend_atlas_online.inc`) or in `pc/platform/gw_ui_room.c`; a change to what they do goes in
  `gmfrontend_online.inc` and the netplay layer, never in the adapter. Run `tools/port/check_atlas_online.sh` after touching either. `fa_room_on()` means a room
  screen is attached in the host, not that Atlas is enabled: a refusal leaves the legacy drawing.
- Syntax-check off Windows as PowerPC (root `CLAUDE.md`); the `.inc` files compile only through
  `gmfrontend.c`.

## Atlas (the new menu system draws this tree)

Each `FeMenu` carries an `atlas_id` (`main`, `solo`, `solo.regular`, ... `more.data`); when `Ui_Ready()` (the host's Atlas fonts are up and
`MELEE_ATLAS` is not `0`) the host draws that menu from the record `gmfrontend_atlas.inc` submits every frame (`Ui_Begin` ... `Ui_Commit`) and the
legacy hub underneath is hidden by its opaque ground. **The adapter never keeps its own cursor** for the menu's items: a host focus event
becomes `fm_go(index)`, accept becomes `fm_confirm()`, back becomes `fm_back()`, and the focus the host is told is `fm.cursor`. So
`fm_position_for`, the pending actions, `FMF_PORT` and the native back-out are the legacy ones. Under Atlas the main menu is `fm_main_atlas`
(Solo, Versus, Online, Mods, Settings; More: Collection, Data, Credits) and Versus is `fm_vs_atlas`; `MELEE_ATLAS=0` keeps `fm_main` and `fm_vs`.
A mod's `mod.json` `menus` entries are tiles tagged MOD after a menu's own items (`Ui_EntryField`, `Ui_EntryActivate`). The legacy pad and mouse
blocks of `fm_scene_frame` are skipped under Atlas (the host reads the mouse and keyboard); `Ui_MenuBlocked()` stops the menu taking input while a
mod's screen is on top. The title is a scene-policy OVERLAY drawn by the host (`gw_ui_policy.c`); REPLACE has no user yet (`gmFrontend_AtlasStandIn`).
`tools/port/test_fe_atlas_positions.py` (workspace) checks every position the router can produce is an item of an Atlas menu.

## Data screens (Atlas step 8)

`gmfrontend_atlas_data.inc` draws the retail screens whose content is data the game already holds. Rules: (1) a screen opens from its `FA_NATIVE` row through `fad_open_for(kind, sel)` and no table row changes, so `MELEE_ATLAS=0`, a netplay session, or a screen left out of `MELEE_ATLAS_DATA` (`events,name,sound,messages,bonus,misc,vsrec`; `none`; `notext` never opens the disc's text archive) runs the retail screen as before; (2) the adapter owns the cursor: the host queues MOVE and PAGE events and never moves a data list's focus, up and down wrap over the whole list, the window slides with `Ui_DataFirst`, row ids are absolute; (3) a locked row is a disabled item and an accept on it does nothing (the host drops it, the adapter checks again); (4) read-only except two writes retail makes too: the selection an event start stores (`gm_801BEB74`, as `mnEvent_8024D864` does) and the heard-track flag Sound Test stores for a flagged track (`gmMainLib_8015ED68`, as `mnsoundtest.c` does; `toy.c` reads it for the Trophy Gallery music); Sound Test applies the saved volumes and sets none; Name Entry's list opens the retail screen for any edit; (5) words on the disc are decoded per string (`fad_sis_text`), drawn, never logged or stored, and a string with a glyph the Latin decoder does not know keeps the row's authored label; (6) Results is a `REPLACE` stand-in reached only by `MELEE_ATLAS_SCENES=5:replace`, never online, with its side-effect table in a comment above `far_enter`. `tools/port/test_fe_atlas_data.py`, `test_fe_atlas_results.py` and `check_no_disc_text.py` (workspace) pin these.


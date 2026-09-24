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
| `gmfrontend_online.inc` | the room screens (`art != 0`), the lobby, strikes |
| `gmfrontend_select.inc` | the kit's character and stage select |
| `gmfrontend_kit.inc`, `_kitlist.inc`, `_player.inc` | drawing: the font atlas and palettes (`kit.json`), the row list (`list_layout.json`, `widgets_layout.json`), the layout/motion player |
| `gmfrontend_mouse.inc` | the mouse in the menus |
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
  `MELEE_FE_HUBDEMO`, `MELEE_NO_ONBOARD`.
- Syntax-check off Windows as PowerPC (root `CLAUDE.md`); the `.inc` files compile only through
  `gmfrontend.c`.

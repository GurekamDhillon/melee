# Envoy fighter picker and the select bypass (v3h)

## What Envoy shows

After the main menu's Envoy row, nothing retail is shown: the camp's fighter picker (envoy_ui `picker.rs`) is
Envoy's character select, and the run's stage comes from the plan. The VS mode's CSS state still runs (the mode's
state machine needs it), but `envoy::bypass_css` (called from gmvsmode `onEnterCss`) fills it, so no
character-select scene is created:

- the run's players are written into `CSSData.vs.start` by `match_begin` (from `PICK` = the picker, or the saved
  `PLAN`);
- the preload cache is written the way the retail CSS exit leaves it (`game_cache.entries[0..4].char_id / color`,
  `game_cache.stkind`, then `lbDvd_80018254`);
- `pending_scene_change = 1` (confirmed);
- the state's scene becomes the loading screen (`GS_FRONTEND` + `gmFrontend_BeginLoading`).

`onExitCss` runs `gmVsMelee_ExitCss` as usual (it copies the CSS data to the VS data and loads the fighters' sound
banks), then loads the stage's sound bank (the SSS exit's job) and goes straight to the match (no SSS, no second
loading screen). This runs on every Envoy match: the first (Start run from the camp, `SKIP_SELECT` 3) and the road's
next fights (`SKIP_SELECT` 1). `MELEE_ENVOY_SELECT=1` brings the retail selects back.

What the retail CSS exit also does and the bypass does not need: freeing MnSlChr / MnExtAll (never loaded), the
name tag and costume UI.

## The old C build's custom select (what the picker takes from it)

Sources: `src/melee/gm/gmfrontend_select.inc` (logic), `gmfrontend_atlas_select.inc` (view),
`gmfrontend_fighter_art.inc` (art lookup), `mncharsel.c` 6513-6640 (Rust: mncharsel.rs ~5920), and
`_research/mex-css.md` in the workspace.

- **Roster.** Retail disc: the 25 icons of the static `icons[]`. m-ex disc (MnSlChr has the symbol `mexSelectChr`):
  `Mex_CssIconCount()` rows of `Mex_CssIconTable()` (`mexData.menu.css` +0xDC, 0x1C per row); byte +1 is the m-ex
  **external id**, turned into a port ckind with `Mex_ExtToPortCKind`. `fs_css_roster` (select.inc 375-474) skips
  ckind < 0 (not in the port), duplicates and locked fighters, then appends Geno defines and ModFighter_* fighters.
- **Id spaces.** ext 0-25 = ckind 0-25. Other ext ids map to an internal id (MxDt fighter +0x0C), a dense slot,
  ckind `CK_MEX0` (0x22) + slot, fighter kind 0x21 + slot, 94 slots (melee_engine `mex_host.rs` 267-328).
- **Costumes.** `fs_costumes(ck)`: Geno's own count, else `gm_GetNumCostumesForCKind` (m-ex patched; host
  `mex_host::FtCostumeCount`). The pick goes to `players[p].color`.
- **Art.** Icons and portraits come from the player's disc (or a mod overriding) `MnSlChr`, decoded at run time:
  `mnCharSel_PcArtIcon(ck)` / `PcArtPortrait(ck, costume)` give the image and TLUT, `gw_Kit_TexAddHsd` decodes them.
  When there is no art: two letters (initials of the first two words, else the first two letters; Random "?").
- **Locked.** Offline the old select leaves locked fighters out; online it greys unavailable ones.
- **Handing over.** `fs_css_finish` sets `slot_type`, `ckind`, `color`, `team`, `cpu_level`, then
  `fs_preload_players` writes the preload cache. The bypass above does the same.

## The picker (envoy_ui picker.rs)

- Roster: `gd_ui_envoy_roster(out, max)`: one word per fighter the build can load as a tag half
  (`envoy::taggable`: retail without Ice Climbers / Zelda / Sheik, plus every m-ex slot `Mex_PortCKindToExt`
  resolves), `ckind | state << 16`. Names of m-ex fighters: `gd_ui_envoy_fighter_name`.
- Unlocks: **everything the build can load is offered for now** (owner, Nov 11); the locked state (2) is drawn
  faded with a lock but the host does not send it yet. Apply `gm_IsCKindUnlocked` there when unlocks return.
- Art: not yet. Every cell is the lettered sticker (retail coral, m-ex lagoon). Next: a host import that decodes
  `mnCharSel_PcArtIcon` into a texture for the wasm, the way `gmresultplayer` reuses it.
- More than 27 fighters: the rows scroll with P1's cursor.
- Costumes: colour 0 for now.

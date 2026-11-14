# SSE stage slot (port/sse-stage)

The Brawl Subspace Emissary map (The Jungle step 3, `080103` + hut `080103a`, read at run time from the owner's own
Brawl files; `explore.rs`) is its own stage slot. Final Destination is Final Destination again.

## GD stage ids

- `explore::GD_STAGE_FIRST..=GD_STAGE_LAST` (0x7000..0x70FF): external stage ids owned by the explore loader. Retail
  external ids stop below 0x21; m-ex external ids start at 288 (`SSS_EXT_MEX_FIRST`), so the range clashes with neither.
- `explore::GD_STAGES`: the GD stage table (id, name, archives). One row today: `GD_STAGE_SSE_JUNGLE` (0x7001),
  "Jungle (SSE)".
- `explore::available(id)`: every archive of the row is in the Brawl folder (`MELEE_BRAWL_LOCAL`, else `_build/brawl`
  walking up). `MELEE_BRAWL_ABSENT=1` behaves as if they were missing.
- `explore::carrier(id)`: the retail stage id a match really loads. A GD id -> Final Destination (0x20) and `PICK` is
  set; any other id clears `PICK`. The rest of the game (preload cache, stage tables, replays, results) only ever sees
  a retail id. The stage load (`coll_override`) swaps in the room's collision when `PICK` (VS) or `WANT` (Envoy) is set.

## Stage select

`mnstagesel.rs` `gd_icons_add`: after the retail grid (or the m-ex icons on ACE) one row per available GD stage is
appended past `mnStageSel_NumStages`, so random never picks it. Its icon is a placeholder (the screen's own random
plate, bottom-right below the grid); hovering it hides the retail name plate and shows the row's name as an SIS text.
Picking it writes the carrier id into the rules via `carrier`. Without the Brawl files no row is added and the screen
is the retail one.

`--match falcon,fox,sse` (or `jungle`) picks the slot from the command line.

## Envoy

`envoy::match_begin` asks for `GD_STAGE_SSE_JUNGLE` for every non-boss fight when `MELEE_ENVOY_EXPLORE=1` and the
files are present (unchanged opt-in), and loads its carrier.

## Known limits

- Replays and netplay record the carrier (Final Destination); the pick itself is not in the replay header yet.
- The icon is a placeholder; no preview picture.
- Not the full m-ex stage tables: only the SSS row and the id range. m-ex stage *loading* (`grfunction`) is separate.

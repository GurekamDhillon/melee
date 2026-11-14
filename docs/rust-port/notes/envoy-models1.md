# Envoy models, chunk 1 (2026-11-13)

Twenty-six original models for Envoy's items, act-map nodes, hearts and pool frames, made the way the Party drive
models are: a part list per model, one generator, no hand-drawn or third-party data.

- Generator: `pc/scripts/examples/envoy_models/tools/make_models1.py` (imports the drives' `drive_parts.py`).
  `--check` compares the files on disk with a fresh run; `--sheet OUT.png` draws the contact sheet.
- Contact sheet (not committed): `E:\Projects\Melee Workspace\scratch\models1\contact-sheet.png`.
- Not built or run in the game: this chunk adds assets only. No Rust changed (see "Loading" and "Not wired").

## Files

Each model `<key>` is two files:

- `pc/scripts/examples/envoy_models/models/em_<key>.gxmesh`: the mesh, in the drives' format and frame.
- `rust/mods/envoy_ui/art/em_<key>.png`: the same model as a 96 x 96 sticker sprite with an ink edge.

Shared: `models/envoy_models_atlas.gxtex` and `.glow.gxtex` (128 x 256 colour ramps), `models/envoy_models.txt`
(the manifest, one key per line in the order below).

## The models

| key | name | concept | where to use it |
|---|---|---|---|
| `spark_ring` | Spark Ring | A charged hoop throwing eight sparks: the ring of sparks a nair hit pops. | pickup prompt, item strip chip, new-item flash, build page; item id 1 |
| `moon_drops` | Moon Drops | A crescent moon with a little moon riding its orbit: two moons that circle you. | pickup prompt, item strip, build page; item id 15 (also the orbital's own body if the sim's moons get a model) |
| `twin_shot` | Twin Shot | Two darts flying side by side, the second a paler copy: every shot gets a twin. | pickup prompt, item strip, build page; item id 39; the synergy banner for doubled shots |
| `bouncy` | Bouncy | A ball skipping along the floor, each hop higher than the last: shots that bounce. | pickup prompt, item strip, build page; item id 42 |
| `chain_spark` | Chain Spark | Two links joined by a lightning bolt in place of the middle link: a hit that arcs on to the next foe. | pickup prompt, item strip, build page; item id 38; the synergy banner for arcs |
| `ice_glass` | Ice Glass | A tall pane of ice with three splinters flying off it: shots that shatter in three. | pickup prompt, item strip, build page; item id 37 (the splinter is the PJ_SHARD shape) |
| `gravity_well` | Gravity Well | Three rings narrowing down a funnel to a dark core, two motes falling in: bursts that pull foes in. | pickup prompt, item strip, build page; item id 40 |
| `big_bang` | Big Bang | A white-hot core, eight fat rays and a broken shock ring: bursts twice as big. | pickup prompt, item strip, build page; item id 41; the devil-deal card |
| `spare_heart` | Spare Heart | A full heart with a plus on its shoulder: one more stock slot, filled. | pickup prompt, item strip, build page; item id 43; the shop's stock-slot row |
| `node_fight` | Fight | Two crossed swords: a plain fight (a duel or a crowd). | act map fight nodes (rows 0, 2, 4), the road screen, the pause Run page |
| `node_elite` | Elite | A pale skull with red horns: a fight that hits back. | act map elite nodes, the elite's card header ("ELITE CARRIES") |
| `node_boss` | Boss | A three-point crown set with red gems: the act's boss. | act map boss node (row 6), the boss card header, the boss-drop banner |
| `node_item` | Item room | A gem floating over a stone pedestal: a free item. | act map item-room nodes (row 1, 3), the item-room banner, the pedestal's stone in the world |
| `node_shop` | Shop | A striped awning over a counter with a coin on it: spend coins. | act map shop node, the shop screen header |
| `node_campfire` | Campfire | A flame on two crossed logs: refill your stocks. | act map campfire nodes (the row before the boss), the campfire banner and room |
| `node_devil` | Devil deal | A red arched door with horns and two burning eyes in its dark: a deal paid in stock slots. | act map door node (row 7) when the red door opens, the devil-deal banner, the door's chance tag |
| `node_angel` | Angel room | A pale glowing door with two wings and a halo over it: one free, strong item. | act map door node (row 7) when the white door opens, the angel-room banner |
| `node_secret` | Secret room | A brick wall split by a glowing crack: hit it three times. | act map marker on the fight node that holds the cracked wall, the secret-room banner, the wall itself |
| `heart_container` | Heart container | An empty heart outline: a stock slot with nothing in it. | the stock row (limit above stocks held), the devil deal's slot price, the shop's stock-slot row |
| `heart_full` | Heart | A plain full heart with a glint: one stock held. | the stock row, the campfire's refill tick, co-op per-player stocks |
| `frame_item` | Frame: item room | A plain cut-corner frame in stone: an item-room find. | behind an item on its card, pickup prompt and build page (pool `item`) |
| `frame_shop` | Frame: shop | The frame in teal with a coin set in its foot: bought. | behind a shop item (pool `shop`); the price tag's anchor |
| `frame_boss` | Frame: boss | The frame in gold with three crown points on top: a boss drop. | behind a boss-pool item; the boss-drop pedestals |
| `frame_devil` | Frame: devil | The frame in dark red with two horns and a spade point below: a deal. | behind a devil-pool item; the deal pedestal and its slot price |
| `frame_angel` | Frame: angel | The frame in cream with a halo over it and a feather at each shoulder: a gift. | behind an angel-pool item; the angel room's pedestal |
| `frame_secret` | Frame: secret | The frame in violet, broken into dashes: found behind a wall. | behind a secret-pool item; the secret room's pedestal |

The nine item keys are the item table's own keys (`envoy_data::items::Item::key`), so an item's sprite is
`em_<item.key>.png`. The item colours are the table's `rgba` values.

## Loading

**Sprites (works today, no host change).** `camp_run.sh` and the perf gate copy `rust/mods/envoy_ui/art` whole into
the mod folder, so `gd_ui_tex_load("em_<key>.png")` returns a handle like any other sticker. A sprite is 96 x 96 for
48 game pixels (the stickers' 2x rule). The load returns 0 for an item with no sprite yet: keep the coloured disc
as the fallback.

- port/envoy-pickup: the floating prompt and the build display can draw `em_<item.key>.png` where the strip and the
  new-item flash draw `S_DISC` now (`items_ui.rs`: `strip`, `flash`), with `em_frame_<pool>.png` behind it.
- port/envoy-synergy: the banners can pair two item sprites; `em_twin_shot`, `em_chain_spark` and `em_big_bang` are
  drawn to read at banner size.
- A frame goes behind its item at the same centre, 4/3 the item's size (item 48, frame 64). The frame to pick, in
  the aura's order (design 11.11): devil, angel, secret, then boss, shop, item.

**Meshes (needs a small host change, not made here).** The world draw only knows the models compiled into
`rust/melee_modes/src/tu/drive_draw.rs`. To draw these in the match (a pedestal's floating item, a map node):

1. add the 26 files to `MESH_FILES` after the Party set, in `envoy_models.txt` order, and raise `MODEL_COUNT`
   (75 to 101; the model id stays a `u8`);
2. decode them with `decode_mesh_in(.., Some(PARTY_SPAN))` against the new atlas pair (the Party set's shared
   frame: no recentring, so a frame and its item line up at one position, yaw and scale);
3. keep them out of `is_drive_body` ranges that assume the Uniques are last (`is_unique` counts back from
   `MODEL_COUNT`: change it to the Party range first);
4. extend the `models_decode_and_sprites_cover` test with the new manifest.

## Not wired

No placeholder was swapped in code. The obvious swaps (the item chip in `items_ui::strip` and `items_ui::flash`,
the pedestal's disc in `items_ui::pedestals`) are the pickup lane's files, and C: was under the 15 GB build floor,
so nothing here could have been built or run. The sprites are named so those swaps are a lookup by item key.

## Chunk 2 proposals

1. **The other 23 items**, same keys: Glide Wake, Shine Bell, Hop Shot, Ledge Spikes, Whiff Echo, Roll Mines, Smash
   Quake, Coin Magnet, Homing Ink, Fire Lace, Laser Prism, Crouch Thorns, Perfect Guard, Brimstone Jab, Demon Wings,
   Crimson Count, Bloodlust, Halo, Guardian Feather, Dodge Orb, Nair Ring, Dash Orb, Shield Orbs. After that every
   item has a sprite and the disc fallback can go.
2. **Keyword glyphs** (design 11.9), one small mark per category: timing, condition, move, tech, outcome, effect,
   status, stacking; and one per status (Burn, Chill, Mark). Flat sprites from the same part lists.
3. **Tag emblems for synergy banners**: projectile, burst, trail, orbital, spark, launch.
4. **World pieces**: the pedestal stone alone, the cracked wall in three damage states, the devil and angel doors
   at door size, a crossed-out pedestal for a locked deal.
5. **Projectile bodies** the sim already has (`PJ_*`): star, ice shard, feather, coin, mine, moon, prism bolt.
6. **Map furniture**: the path dot, the cursor, P2's wish flag, a visited tick, the door node before it is known.
7. **HUD bits**: the A-button prompt, a coin, quality pips 0 to 4, a stock slot being lost (the devil's price), and a
   curse-pool frame.

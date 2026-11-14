# Envoy models, chunks 1 and 2 (2026-11-13 / 14)

91 original models for Envoy: every item, the act-map nodes, hearts and HUD bits, pool frames, keyword and status
glyphs, tag emblems, projectile bodies, map furniture and world pieces. They are made the way the Party drive models
are: a part list per model, one generator, no hand-drawn or third-party data.

- Generator: `pc/scripts/examples/envoy_models/tools/make_models1.py` (helpers, palette, chunk 1's part lists, every
  writer) and `models2.py` beside it (chunk 2's part lists). It imports the drives' `drive_parts.py` for the
  primitives and the colour ramp. `--check` compares the files on disk with a fresh run; `--sheet OUT.png` draws
  the contact sheet.
- Contact sheet (not committed): `E:\Projects\Melee Workspace\scratch\models1\contact-sheet.png`.

## Files

Each model `<key>` is two files:

- `pc/scripts/examples/envoy_models/models/em_<key>.gxmesh`: the mesh, in the drives' format and frame.
- `rust/mods/envoy_ui/art/em_<key>.png`: the same model as a 96 x 96 sticker sprite with an ink edge.

Shared, in `models/`: `envoy_models_atlas.gxtex` and `.glow.gxtex` (128 x 256 colour ramps, 43 rows),
`envoy_models.txt` (the manifest: one key per line, in model order) and `envoy_models.inc.rs` (the same list as the
host's `include_bytes!` array).

New models are appended: a key's line in the manifest is its model id, so the order never changes. New colour rows
are appended to the atlas for the same reason.

## Loading

**Sprites.** `camp_run.sh` and the perf gate copy `rust/mods/envoy_ui/art` whole into the mod folder, so
`gd_ui_tex_load("em_<key>.png")` returns a handle like any other sticker. A sprite is 96 x 96 for 48 game pixels (the
stickers' 2x rule). An item's sprite is `em_<item.key>.png` (`envoy_data::items::Item::key`); all 32 items have one.
A pool frame goes behind its item at the same centre, 4/3 the item's size (item 48, frame 64). The frame to pick,
in the aura's order (design 11.11): devil, angel, secret, then boss, shop, item.

**Meshes in the world.** The host draws them like drive models: `gd_ui_world_model` / `gd_ui_world_model_ex` with
model id `75 + line` (`melee_modes::tu::drive_draw::EM_FIRST`, after the 10 legacy and 65 Party models;
`drive_draw::em_model(key)` on the host, `envoy_ui::fx::em_model(key)` in the UI mod, -1 when there is none).
`scale` is the frame's width in world units: a model that fills the parts box is about 0.45 of it across, so
`scale 16` draws an item about 7 units wide. Every model shares the Party frame (no recentring), so a pool frame
and its item line up at one position, yaw and scale. The host drops the ink edge (`MAT_OUTLINE`) for this set (see
Verified).
`gd_ui_model_tex` gives the turning 12-frame sprite sheet for any of them too.

**Wired here.** `items_ui::pedestals` draws the floating item as its model (rocking, on a soft disc of the
item's colour) and keeps the old discs for an id with no model. Nothing else is swapped: the item strip, the
new-item flash, the map nodes and the stock row still draw what they drew, for the pickup, synergy and UI lanes to
change with the sprites above.

## Verified

Built and run on Windows, 2026-11-14, hidden window (`MELEE_WINDOW_HIDDEN=1`), release exe from this branch:

- `cargo test -p melee_modes --lib drive_draw`: 2 passed (`models_decode_and_sprites_cover` checks the include list
  against the manifest and the files on disk, that all 91 decode, two atlas colours, and a sprite sheet of one).
- `envoy_sim` and `envoy_ui` built for wasm; `melee` built in release.
- A scripted Envoy run to the boss drop (`MELEE_ENVOY_START_FIGHT=4`, knobs `nodoor,*4000`, the player a CPU):
  the log has `envoy: boss drop, choose one`, no host errors, and the screenshot shows both pedestals with their
  items drawn as models in the world (Shine Bell and Roll Mines), in their own colours, at the stone.
  Kept at `E:\Projects\Melee Workspace\scratch\models1\verify-boss-drop.png` (not committed).
- An item-room run (`MELEE_ENVOY_START_FIGHT=2`, `forkitem`) opened the room and drew its pedestal too; in that run
  the pedestal stood on a lower floor than the idle CPU, so the camera kept it at the bottom edge of the picture.
- Found on the way: with `MAT_OUTLINE` at pedestal size the ink hull covers the whole body (the models came out
  solid ink). `drive_draw::is_drive_body` therefore excludes this set, so the host drops the flag; the pedestal draw
  uses `MAT_BODY` with no edge. The sprites keep their ink edge (it is drawn by the generator).
- Not run: the items_check synctests and the zero-lag gate. The change is presentation only (no sim file touched),
  but 91 more meshes are uploaded when the effects layer arms; the arm took 128 ms in the run's log, once, at run
  start.

## Items (32)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `spark_ring` | 75 | Spark Ring | A charged hoop throwing eight sparks: the ring of sparks a nair hit pops. | pickup prompt, item strip chip, new-item flash, build page; item id 1 |
| `moon_drops` | 76 | Moon Drops | A crescent moon with a little moon riding its orbit: two moons that circle you. | pickup prompt, item strip, build page; item id 15 (also the orbital's own body if the sim's moons get a model) |
| `twin_shot` | 77 | Twin Shot | Two darts flying side by side, the second a paler copy: every shot gets a twin. | pickup prompt, item strip, build page; item id 39; the synergy banner for doubled shots |
| `bouncy` | 78 | Bouncy | A ball skipping along the floor, each hop higher than the last: shots that bounce. | pickup prompt, item strip, build page; item id 42 |
| `chain_spark` | 79 | Chain Spark | Two links joined by a lightning bolt in place of the middle link: a hit that arcs on to the next foe. | pickup prompt, item strip, build page; item id 38; the synergy banner for arcs |
| `ice_glass` | 80 | Ice Glass | A tall pane of ice with three splinters flying off it: shots that shatter in three. | pickup prompt, item strip, build page; item id 37 (the splinter is the PJ_SHARD shape) |
| `gravity_well` | 81 | Gravity Well | Three rings narrowing down a funnel to a dark core, two motes falling in: bursts that pull foes in. | pickup prompt, item strip, build page; item id 40 |
| `big_bang` | 82 | Big Bang | A white-hot core, eight fat rays and a broken shock ring: bursts twice as big. | pickup prompt, item strip, build page; item id 41; the devil-deal card |
| `spare_heart` | 83 | Spare Heart | A full heart with a plus on its shoulder: one more stock slot, filled. | pickup prompt, item strip, build page; item id 43; the shop's stock-slot row |
| `glide_wake` | 101 | Glide Wake | Three flames rising along a streak on the floor: the burning line a wavedash leaves. | pickup prompt, item strip, build page; item id 2 |
| `shine_bell` | 102 | Shine Bell | The shine's hexagon ringing like a bell, sound arcs to both sides. | pickup prompt, item strip, build page; item id 3 |
| `hop_shot` | 103 | Hop Shot | A star spat upward off two hop marks: a short hop fires a star. | pickup prompt, item strip, build page; item id 4 |
| `ledge_spikes` | 104 | Ledge Spikes | Three ice spikes stabbing up from a ledge. | pickup prompt, item strip, build page; item id 5 |
| `whiff_echo` | 105 | Whiff Echo | A pale wisp with a swing arc beside it: the whiff swings again. | pickup prompt, item strip, build page; item id 6 |
| `roll_mines` | 106 | Roll Mines | A spiked mine with a red light and two blink arcs over it. | pickup prompt, item strip, build page; item id 7 |
| `smash_quake` | 107 | Smash Quake | Rock teeth thrown up along the floor, tallest at the middle, leaning outward: shockwaves from a smash. | pickup prompt, item strip, build page; item id 12 |
| `coin_magnet` | 108 | Coin Magnet | A horseshoe magnet pulling a coin up between its tips. | pickup prompt, item strip, build page; item id 18 |
| `homing_ink` | 109 | Homing Ink | An ink stroke curling round to an arrowhead that points at a dot: shots that curve to foes. | pickup prompt, item strip, build page; item id 35 |
| `fire_lace` | 110 | Fire Lace | Two loops of fire laced through each other, embers rising off them. | pickup prompt, item strip, build page; item id 36 |
| `laser_prism` | 111 | Laser Prism | A glass prism: one beam in, three beams fanning out. | pickup prompt, item strip, build page; item id 8 |
| `crouch_thorns` | 112 | Crouch Thorns | A low mound bristling with thorns: crouch under a hit and they strike back. | pickup prompt, item strip, build page; item id 21 |
| `perfect_guard` | 113 | Perfect Guard | A gold shield with the powershield flash on its face. | pickup prompt, item strip, build page; item id 26 |
| `brimstone_jab` | 114 | Brimstone Jab | A horned ball of fire at the head of a wide red beam. | pickup prompt, item strip, build page; item id 27 |
| `demon_wings` | 115 | Demon Wings | A pair of dark bat wings with scalloped edges: one more air jump. | pickup prompt, item strip, build page; item id 28 |
| `crimson_count` | 116 | Crimson Count | A red diamond sigil with an eye, three tally marks under it: one sigil a KO. | pickup prompt, item strip, build page; item id 29 |
| `bloodlust` | 117 | Bloodlust | A drop of blood with three tendrils whipping out of it. | pickup prompt, item strip, build page; item id 30 |
| `halo` | 118 | Halo | A broad gold ring seen from below, light falling from it. | pickup prompt, item strip, build page; item id 31 |
| `guardian_feather` | 119 | Guardian Feather | One large white feather held upright against a hit. | pickup prompt, item strip, build page; item id 32 |
| `dodge_orb` | 120 | Dodge Orb | An orb shooting down and forward, three streaks behind it: it leads the airdodge. | pickup prompt, item strip, build page; item id 44 |
| `nair_ring` | 121 | Nair Ring | Six orbs on a ring around a spinning mark: orbs that circle a nair. | pickup prompt, item strip, build page; item id 45 |
| `dash_orb` | 122 | Dash Orb | An orb running ahead of three speed lines: it leads the dash. | pickup prompt, item strip, build page; item id 46 |
| `shield_orbs` | 123 | Shield Orbs | A shield bubble with three small orbs standing guard around it. | pickup prompt, item strip, build page; item id 47 |

## Act-map nodes and room markers (9)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `node_fight` | 84 | Fight | Two crossed swords: a plain fight (a duel or a crowd). | act map fight nodes (rows 0, 2, 4), the road screen, the pause Run page |
| `node_elite` | 85 | Elite | A pale skull with red horns: a fight that hits back. | act map elite nodes, the elite's card header ("ELITE CARRIES") |
| `node_boss` | 86 | Boss | A three-point crown set with red gems: the act's boss. | act map boss node (row 6), the boss card header, the boss-drop banner |
| `node_item` | 87 | Item room | A gem floating over a stone pedestal: a free item. | act map item-room nodes (row 1, 3), the item-room banner, the pedestal's stone in the world |
| `node_shop` | 88 | Shop | A striped awning over a counter with a coin on it: spend coins. | act map shop node, the shop screen header |
| `node_campfire` | 89 | Campfire | A flame on two crossed logs: refill your stocks. | act map campfire nodes (the row before the boss), the campfire banner and room |
| `node_devil` | 90 | Devil deal | A red arched door with horns and two burning eyes in its dark: a deal paid in stock slots. | act map door node (row 7) when the red door opens, the devil-deal banner, the door's chance tag |
| `node_angel` | 91 | Angel room | A pale glowing door with two wings and a halo over it: one free, strong item. | act map door node (row 7) when the white door opens, the angel-room banner |
| `node_secret` | 92 | Secret room | A brick wall split by a glowing crack: hit it three times. | act map marker on the fight node that holds the cracked wall, the secret-room banner, the wall itself |

## Hearts and HUD bits (7)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `heart_container` | 93 | Heart container | An empty heart outline: a stock slot with nothing in it. | the stock row (limit above stocks held), the devil deal's slot price, the shop's stock-slot row |
| `heart_full` | 94 | Heart | A plain full heart with a glint: one stock held. | the stock row, the campfire's refill tick, co-op per-player stocks |
| `hud_a_button` | 141 | A button | A round green button with an A on it. | the pickup prompt over a pedestal in range (V_PED_NEAR); the map's "A takes it" |
| `hud_coin` | 142 | Coin | A gold coin with a slot mark. | the coin count, shop prices, the cash-out line |
| `hud_pip_on` | 143 | Quality pip, lit | A small gold diamond. | an item's quality (0 to 4 pips) on its card and in the shop |
| `hud_pip_off` | 144 | Quality pip, empty | The same diamond as an empty stone outline. | the unlit pips of an item's quality |
| `hud_slot_lost` | 145 | Stock slot lost | The heart container struck through with a red cross: a slot a deal takes. | the devil deal's price tag (one per slot), the stock row as a slot leaves |

## Pool frames (7)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `frame_item` | 95 | Frame: item room | A plain cut-corner frame in stone: an item-room find. | behind an item on its card, pickup prompt and build page (pool `item`) |
| `frame_shop` | 96 | Frame: shop | The frame in teal with a coin set in its foot: bought. | behind a shop item (pool `shop`); the price tag's anchor |
| `frame_boss` | 97 | Frame: boss | The frame in gold with three crown points on top: a boss drop. | behind a boss-pool item; the boss-drop pedestals |
| `frame_devil` | 98 | Frame: devil | The frame in dark red with two horns and a spade point below: a deal. | behind a devil-pool item; the deal pedestal and its slot price |
| `frame_angel` | 99 | Frame: angel | The frame in cream with a halo over it and a feather at each shoulder: a gift. | behind an angel-pool item; the angel room's pedestal |
| `frame_secret` | 100 | Frame: secret | The frame in violet, broken into dashes: found behind a wall. | behind a secret-pool item; the secret room's pedestal |
| `frame_curse` | 146 | Frame: curse | The frame in purple with a thorn at each corner pointing in: it comes with a drawback. | behind a curse-pool item (design 7, the `curse` pool) when that pool is built |

## Keyword and status glyphs (11)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `kw_timing` | 124 | Keyword: timing | A clock face with two hands: ON / WHEN / WHILE / AFTER. | before a timing keyword on an item card; the glossary's Timing header |
| `kw_condition` | 125 | Keyword: condition | One path forking in two: IF. | before a condition keyword; the glossary's Conditions header |
| `kw_move` | 126 | Keyword: move | A slash with an impact star at its end: NAIR, JAB, SMASH and the other attacks. | before a move keyword; the glossary's Moves header |
| `kw_tech` | 127 | Keyword: tech | Two chevrons skimming a ground line: WAVEDASH, SHORT HOP, L-CANCEL and the other tech. | before a tech keyword; the glossary's Tech header |
| `kw_outcome` | 128 | Keyword: outcome | A jagged impact burst: HIT, WHIFF, SHIELD, KO, TAKE HIT. | before an outcome keyword; the glossary's Outcomes header |
| `kw_effect` | 129 | Keyword: effect | An orb inside a ring: PROJECTILE, BURST, TRAIL, AURA, ORB, BEAM. | before an effect keyword; the glossary's Effects header |
| `kw_status` | 130 | Keyword: status | A drop with a mark in it: BURN, CHILL, MARK. | before a status keyword; the glossary's Statuses header |
| `kw_stack` | 131 | Keyword: stacking | Three slabs stacked, the top one bright: STACK, PER STACK. | before a stacking keyword; the stack count on a chip |
| `st_burn` | 132 | Status: Burn | A single flame with a hot core. | the BURN keyword, the Burning status tag over a foe |
| `st_chill` | 133 | Status: Chill | A six-armed ice crystal. | the CHILL keyword, the Chilled status tag over a foe |
| `st_mark` | 134 | Status: Mark | A crosshair: a ring, four ticks and a centre dot. | the MARK keyword, the Marked status tag over a foe |

## Tag emblems (synergy banners) (6)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `tag_projectile` | 135 | Tag: projectile | An arrow in flight. | synergy banners and modifier cards for TG_PROJECTILE |
| `tag_burst` | 136 | Tag: burst | A ring thrown outward in four pieces from a core. | synergy banners and modifier cards for TG_BURST |
| `tag_trail` | 137 | Tag: trail | A line of marks growing toward the front. | synergy banners and modifier cards for TG_TRAIL |
| `tag_orbital` | 138 | Tag: orbital | A body with a moon on a ring around it. | synergy banners and modifier cards for TG_ORBITAL |
| `tag_spark` | 139 | Tag: spark | A lightning bolt. | synergy banners and modifier cards for TG_SPARK |
| `tag_launch` | 140 | Tag: launch | A thick arrow going up off a line. | synergy banners and modifier cards for TG_LAUNCH |

## Projectile bodies (7)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `pj_star` | 147 | Shot: star | A plain five-point star. | PJ_STAR in the world (Hop Shot); a size that reads at 6 world units |
| `pj_shard` | 148 | Shot: ice shard | A long ice splinter. | PJ_SHARD in the world (Ice Glass); turn it to its velocity |
| `pj_feather` | 149 | Shot: feather | A dark feather with a red quill. | PJ_FEATHER in the world (Demon Wings) |
| `pj_coin` | 150 | Shot: coin | A coin with a round hole, for turning in the air. | PJ_COIN in the world (Coin Magnet): spin it by yaw |
| `pj_mine` | 151 | Shot: mine | The spiked mine with its red light. | the mine on the floor (Roll Mines); pulse the light with the blink |
| `pj_moon` | 152 | Shot: moon | A round moon with two craters. | the Moon Drops orbital's body |
| `pj_bolt` | 153 | Shot: prism bolt | A thin bright bolt, pointed at both ends. | PJ_PRISM in the world (Laser Prism's three bolts) |

## Map furniture (5)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `map_dot` | 154 | Map: path dot | A small stone bead. | the dotted links between act-map nodes |
| `map_cursor` | 155 | Map: cursor | A gold arrowhead pointing down at the chosen node, with a bright rim. | P1's node cursor on the road screen, the pause Run page and the in-match room choice |
| `map_flag` | 156 | Map: wish flag | A pennant on a pole: where P2 would like to go. | P2's wish flag on the act map |
| `map_tick` | 157 | Map: visited | A green tick. | a cleared node on the act map |
| `map_door_unknown` | 158 | Map: unknown door | A stone arch with a question mark in it: the door after the boss, before it is known. | the door node (row 7) until the devil or angel door opens; its chance tag |

## World pieces (7)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `w_pedestal` | 159 | World: pedestal | The stone pedestal alone: a foot, a tapered column and a bright top slab. | under every floating item (item room, boss drop, devil, angel, secret): replaces the FX_BEAM stone in items_ui::pedestals |
| `w_pedestal_locked` | 160 | World: locked pedestal | The pedestal crossed out in red: a deal you cannot afford. | a devil deal that would take the last stock slot (drawn crossed) |
| `w_wall_0` | 161 | World: wall, whole | The brick wall with a hairline crack. | the secret wall before any hit |
| `w_wall_1` | 162 | World: wall, cracked | The wall with the crack run top to bottom. | the secret wall after one hit |
| `w_wall_2` | 163 | World: wall, breaking | The wall with a brick knocked out and light pouring through the gap. | the secret wall after two hits (the third breaks it) |
| `w_door_devil` | 164 | World: devil door | The red door at full height on its step, horned, eyes lit. | the door standing in the held match when the devil deal opens |
| `w_door_angel` | 165 | World: angel door | The pale door at full height on its step, a halo over it, light in the doorway. | the door standing in the held match when the angel room opens |

## Chunk 3 proposals

1. **Items still to come** (design section 7's first 40: this set covers the 32 in the table). Each new item gets
   its model with its table row.
2. **Per-keyword marks** where a category mark is too coarse: one for each tech (wavedash, short hop, L-cancel,
   ledge grab, roll, airdodge, dash, crouch cancel, powershield) and each move class (nair, jab, smash, special,
   grab, throw, shine, laser).
3. **Boss and elite faces for the map**: a node mark per boss (Petey first), so the boss node shows who it is.
4. **Fight node variants**: duel, crowd and gauntlet marks (the node kind picks the fight).
5. **Effect meshes that are fx shapes today**: the gold beam, the brimstone beam, the quake teeth, thorns, the
   ghost, tendrils, sigils, the stance orbs' bodies.
6. **Synergy banner frames**: a ribbon and a plate the two item sprites sit on, in the six tag colours.
7. **Run-end and camp pieces**: a trophy, a tombstone for a lost run, a loop counter, the campfire at room size,
   a shop stall at room size, a stock-refill sparkle.

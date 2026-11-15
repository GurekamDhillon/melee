# Envoy models, chunks 1 to 4 (2026-11-13 / 14)

182 original models for Envoy: every item (the synergy lane's six included), act-map nodes and fight variants, hearts
and HUD bits, pool frames, keyword, status, tech and move marks, tag emblems, projectile bodies, map furniture,
world pieces, effect meshes and stance-orb bodies, boss and elite faces, synergy and evolution banner pieces,
run-end pieces, curse-pool marks, camp and road scenery, and 8 animated sprite strips. They are made the way the Party drive models are: a part list per model, one generator, no
hand-drawn or third-party data.

- Generator: `pc/scripts/examples/envoy_models/tools/make_models1.py` (helpers, palette, chunk 1's part lists, every
  writer) with `models2.py`, `models3.py` and `models4.py` beside it (chunks 2 to 4). It imports the drives' `drive_parts.py` for
  the primitives and the colour ramp. `--check` compares the files on disk with a fresh run; `--sheet OUT.png`
  draws the contact sheet.
- Contact sheet (not committed): `E:\Projects\Melee Workspace\scratch\models1\contact-sheet.png`.

## Files

Each model `<key>` is two files:

- `pc/scripts/examples/envoy_models/models/em_<key>.gxmesh`: the mesh, in the drives' format and frame.
- `rust/mods/envoy_ui/art/em_<key>.png`: the same model as a 96 x 96 sticker sprite with an ink edge.

Shared, in `models/`: `envoy_models_atlas.gxtex` and `.glow.gxtex` (128 x 256 colour ramps, 45 rows),
`envoy_models.txt` (the manifest: one key per line, in model order) and `envoy_models.inc.rs` (the same list as the
host's `include_bytes!` array).

New models are appended: a key's line in the manifest is its model id, so the order never changes. New colour rows
are appended to the atlas for the same reason. The model id is a `u16` on the host since chunk 4 (257 models in
all with the drives' 75).

**Animated sprites.** `rust/mods/envoy_ui/art/em_<key>_anim.png` is a strip of frames, each 96 x 96, side by side;
`models/envoy_models_anim.txt` lists each strip and its frame count. Frame 0 is the still sprite. The meshes are not
animated.

## Loading

**Sprites.** `camp_run.sh` and the perf gate copy `rust/mods/envoy_ui/art` whole into the mod folder, so
`gd_ui_tex_load("em_<key>.png")` returns a handle like any other sticker. A sprite is 96 x 96 for 48 game pixels (the
stickers' 2x rule). An item's sprite is `em_<item.key>.png` (`envoy_data::items::Item::key`); every item has one.
A pool frame goes behind its item at the same centre, 4/3 the item's size (item 48, frame 64). The frame to pick,
in the aura's order (design 11.11): devil, angel, secret, then boss, shop, item.

**Synergy lane.** A synergy's emblem is `em_syn_<Synergy::key>.png`, an evolution's `em_evo_<Evolution::key>.png`.
A banner is `em_banner_plate` (tinted with the synergy's `rgba`) with the two item sprites on it, `em_banner_plus`
between them (`em_banner_arrow` for an evolution), and `em_banner_ribbon` (tinted) under the name. The plate and
the ribbon are white so a tint gives the colour.

**Meshes in the world.** The host draws them like drive models: `gd_ui_world_model` / `gd_ui_world_model_ex` with
model id `75 + line` (`melee_modes::tu::drive_draw::EM_FIRST`, after the 10 legacy and 65 Party models;
`drive_draw::em_model(key)` on the host, `envoy_ui::fx::em_model(key)` in the UI mod, -1 when there is none).
`scale` is the frame's width in world units: a model that fills the parts box is about 0.45 of it across, so
`scale 16` draws an item about 7 units wide. Every model shares the Party frame (no recentring), so a pool frame
and its item line up at one position, yaw and scale. `gd_ui_model_tex` gives the turning 12-frame sprite sheet for
any of them too. `gd_ui_world_model_rot` is `gd_ui_world_model_ex` with a `roll` after the yaw (radians, a turn in
the model's own plane, applied before the yaw): a shot laid along its velocity is `roll = atan2(vy, vx)`.

**The ink edge.** `MAT_OUTLINE` works on this set. The ink pass (`melee_gx::fx`, pipeline "fx ink") now pushes the
hull behind its own body (a slope-scaled depth bias) and widens it with the model's scale (x1 up to scale 5, so
drives are unchanged, to x3 at scale 15 and over).

**Wired here.**

- `items_ui::pedestals` draws the floating item as its model in the pedestal's shader ring, and keeps the shader
  orb for an id with no model.
- `items_ui::proj2` draws these effect bodies as models (`fx::mesh`, ink edged), each with its old shapes kept as
  the fallback when the model is missing: Demon Wings' feathers (`pj_feather`) and worn wings (`fx_wings`), Ice
  Glass shards (`pj_shard`, along their velocity), Laser Prism's bolts (`pj_bolt`; they had no drawing before),
  Crimson Count's sigils (`fx_sigil`), the Guardian Feather (`guardian_feather`), the Halo (`fx_halo`), Crouch
  Thorns (`fx_thorns`), the four stance orbs (`orb_*`) and the Moon Drops orbital (`pj_moon`). Their trails and
  glows are kept. The first slice's effects (sparks, star, wake, ghost, mine, quake, coin) and every burst are the
  vfx lane's tuned shader quads and are NOT swapped: a static mesh would be a step back from those. The beams,
  the tendril and the gravity well are also left as they were (a beam needs a length, which one mesh cannot give).
- The synergy / evolution banner and the pause Combos card show `em_syn_<key>` / `em_evo_<key>`
  (`kit::em_spr`, loaded on first use); the campfire banner plays `em_node_campfire_anim` (`kit::em_anim`).
- Nothing else is swapped: the item strip and Items page keep slice 2's own icons (`icons.rs`), the map nodes and
  the stock row draw what they drew.

## Verified

**Chunk 4 is only partly verified.** The game runs were being made when the machine ran critically short of memory
and the session's background job was stopped (2026-11-14). The coordinator then asked for no new builds or game runs
until candidate10 is packaged, so they were not restarted.

Done, on Windows, hidden windows, release exe from this branch after merging `pub/port/envoy-slice2` (`03846fe7b`)
and `pub/port/envoy-synergy` (`72d52f625`):

- `cargo test -p melee_modes --lib drive_draw`: 2 passed.
- `envoy_sim` and `envoy_ui` built for wasm; `melee` built in release. The two merged lanes did not compile
  together (`envoy_data::keywords`: "an item has no card" for the synergy lane's six items); cards and trigger
  words were added.
- One `itemdemo` run at 1280 x 720 (every item given and fired in turn, 181 screenshots): no host errors in the
  log; the pictures show the wings, the halo, the feathers and moons as models, and the Blood Moon and Iron Maiden
  banners with their emblems. An index of that run is kept at
  `E:\Projects\Melee Workspace\scratch\models1\swap-after720-index.png` (not committed).

NOT done for chunk 4:

- `items_check` at depth 4 and 7 (asked for). The last green run is chunk 3's tree at depth 4 (20 rows, 0
  mismatches), before the synergy merge, the u16 / roll change and the mesh swaps. This branch's own commits change
  no sim file, but the merge brings the synergy lane's sim in beside slice 2's, and that combination has not been
  checked here.
- The frame-time check (asked for). The script is ready (`scratch/models1/perf.sh`: the itemdemo run in real time
  with the frame profiler, then `zero_hitch_check.py`).
- The before / after crops at 720p and 1080p (asked for). Only the "after" 720p run finished; the "before" build of
  the UI mod and the crop script are ready (`scratch/models1/demo.sh`, `crops.py`).
- The Combos card's emblem and the campfire strip were not seen on screen (the demo run does not open them).
- The last change to the wings' size (30 to 24 world units) was built into the UI mod and is in that 720p run.

## Items (40)

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
| `ember_seed` | 202 | Ember Seed | A dark seed with a small flame sprouting from its tip. | pickup prompt, item strip, build page; the synergy lane's item `ember_seed` |
| `static_coil` | 203 | Static Coil | A coil of four rings on a base, a spark at its top. | pickup prompt, item strip, build page; the synergy lane's item `static_coil` |
| `frost_bud` | 204 | Frost Bud | A closed bud of three ice petals on a short stem. | pickup prompt, item strip, build page; the synergy lane's item `frost_bud` |
| `kin_tether` | 205 | Kin Tether | Two orbs, coral and lagoon, joined by a taut cord. | pickup prompt, item strip, build page; the synergy lane's item `kin_tether` |
| `rally_banner` | 206 | Rally Banner | A red banner on a pole with a gold star on it. | pickup prompt, item strip, build page; the synergy lane's item `rally_banner` |
| `kindred_spark` | 207 | Kindred Spark | Two sparks facing each other, a thread of light between them. | pickup prompt, item strip, build page; the synergy lane's item `kindred_spark` |
| `haunted_shell` | 243 | Haunted Shell | A skull peering out of a spiral shell, a wisp trailing off it. | pickup prompt, item strip, build page; the curse pool's item 33 |
| `glass_fist` | 244 | Glass Fist | A fist of clear glass, four knuckles and a thumb, shards coming off it. | pickup prompt, item strip, build page; the curse pool's item 34 |

## Act-map nodes and room markers (12)

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
| `node_duel` | 214 | Fight: duel | Two swords meeting point to point, a spark where they touch. | act map MN_DUEL nodes (a gauntlet of duels) |
| `node_crowd` | 215 | Fight: crowd | Three round heads, the middle one in front: one against three. | act map MN_CROWD nodes (the 1v3 crowd) |
| `node_start` | 216 | Map: start | A footprint of two chevrons stepping up from a line. | act map MN_START (the foot of the map) |

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

## Keyword and status glyphs (29)

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
| `kw_wavedash` | 217 | Keyword: wavedash | A step down at a slant, then a long slide along the floor. | the WAVEDASH keyword |
| `kw_short_hop` | 218 | Keyword: short hop | A low arc over the floor. | the SHORT HOP keyword |
| `kw_l_cancel` | 219 | Keyword: L-cancel | A letter L on a shoulder tab, landing lines under it. | the L-CANCEL keyword |
| `kw_ukemi` | 220 | Keyword: tech (the move) | A body bouncing off the floor and away. | the TECH keyword (the ground tech; `kw_tech` is the category mark) |
| `kw_ledge_grab` | 221 | Keyword: ledge grab | A hook catching the corner of a ledge. | the LEDGE GRAB keyword |
| `kw_roll` | 222 | Keyword: roll | A wheel turning along the floor. | the ROLL keyword |
| `kw_airdodge` | 223 | Keyword: airdodge | A body inside a broken ring, sliding off at a slant. | the AIRDODGE keyword |
| `kw_dash` | 224 | Keyword: dash | An arrowhead ahead of three speed lines. | the DASH keyword |
| `kw_crouch_cancel` | 225 | Keyword: crouch cancel | A low block braced on the floor under a blow coming down. | the CROUCH CANCEL keyword |
| `kw_powershield` | 226 | Keyword: powershield | A shield bubble with a flash on its rim. | the POWERSHIELD keyword |
| `kw_nair` | 227 | Keyword: nair | A body with a ring all the way round it: the neutral air. | the NAIR keyword |
| `kw_jab` | 228 | Keyword: jab | A short blunt punch with two impact ticks. | the JAB keyword |
| `kw_smash` | 229 | Keyword: smash | A heavy wedge driven into a burst. | the SMASH keyword |
| `kw_special` | 230 | Keyword: special | A round red button with a B on it. | the SPECIAL keyword |
| `kw_grab` | 231 | Keyword: grab | Two brackets closing on a body. | the GRAB keyword |
| `kw_throw` | 232 | Keyword: throw | A body flung up and away from a cupped hand. | the THROW keyword |
| `kw_shine` | 233 | Keyword: shine | A hexagon outline with a bright centre. | the SHINE keyword |
| `kw_laser` | 234 | Keyword: laser | A thin beam leaving a muzzle flash. | the LASER keyword |

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

## World pieces (9)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `w_pedestal` | 159 | World: pedestal | The stone pedestal alone: a foot, a tapered column and a bright top slab. | under every floating item (item room, boss drop, devil, angel, secret): replaces the FX_BEAM stone in items_ui::pedestals |
| `w_pedestal_locked` | 160 | World: locked pedestal | The pedestal crossed out in red: a deal you cannot afford. | a devil deal that would take the last stock slot (drawn crossed) |
| `w_wall_0` | 161 | World: wall, whole | The brick wall with a hairline crack. | the secret wall before any hit |
| `w_wall_1` | 162 | World: wall, cracked | The wall with the crack run top to bottom. | the secret wall after one hit |
| `w_wall_2` | 163 | World: wall, breaking | The wall with a brick knocked out and light pouring through the gap. | the secret wall after two hits (the third breaks it) |
| `w_door_devil` | 164 | World: devil door | The red door at full height on its step, horned, eyes lit. | the door standing in the held match when the devil deal opens |
| `w_door_angel` | 165 | World: angel door | The pale door at full height on its step, a halo over it, light in the doorway. | the door standing in the held match when the angel room opens |
| `w_campfire` | 211 | World: campfire | The campfire at room size: a ring of stones, crossed logs, a tall flame. | standing in the campfire room; the camp screen |
| `w_shop_stall` | 212 | World: shop stall | The shop stall at room size: awning, posts, a counter with three wares on it. | standing in the shop room behind the wares |

## Effect meshes and stance-orb bodies (18)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `fx_beam_gold` | 166 | Effect: gold beam | A length of gold beam with a bright core and a pointed head. | Perfect Guard's beam: lay lengths end to end along the shot, the head last |
| `fx_beam_brim` | 167 | Effect: brimstone beam | A length of blood-red beam, ragged along both edges, with a hot core. | Brimstone Jab's beam across the stage |
| `fx_quake_tooth` | 168 | Effect: quake tooth | One leaning tooth of rock on a chip of floor. | Smash Quake's wave: a row of teeth, tallest first, each leaning the way the wave runs |
| `fx_thorns` | 169 | Effect: thorns | A fan of five green thorns from one root. | Crouch Thorns' strike from the back; Iron Maiden's bigger thorns |
| `fx_ghost` | 170 | Effect: echo ghost | The pale wisp with two dark eyes and a tail. | Whiff Echo's ghost, where the whiff was |
| `fx_tendril` | 171 | Effect: tendril | One red tendril curling to a barbed tip. | Bloodlust's lash at the attacker; three or four turned about the fighter |
| `fx_sigil` | 172 | Effect: sigil | The red diamond sigil with its eye. | Crimson Count's sigils orbiting the fighter, one a KO |
| `fx_ice_spike` | 173 | Effect: ice spike | One tall ice spike with a short one at its foot. | Ledge Spikes' stab; Frost Bud's burst |
| `fx_spark` | 174 | Effect: spark | A four-point spark, long up and down. | Spark Ring's sparks, Chain Spark's arc ends, any hit glint |
| `fx_ember` | 175 | Effect: ember | A small flame with a hot core. | Fire Lace's embers on everything; Glide Wake's line of flames |
| `fx_shock_hex` | 176 | Effect: shock hexagon | A hexagon outline, bright on its inner edge. | Shine Bell's ring as it grows; Thunder Bell's wider one |
| `fx_halo` | 177 | Effect: halo | A gold ring lying flat, for over a head. | the worn Halo (45 frames after a proc); the holy tint on shots |
| `fx_wings` | 178 | Effect: demon wings | The pair of dark bat wings, spread. | the worn Demon Wings behind the fighter on the extra jump |
| `orb_dodge` | 179 | Orb: dodge | An orb with a swept fin: it leads the airdodge. | the Dodge Orb's body (TR_WHILE_AIRDODGE) |
| `orb_nair` | 180 | Orb: nair | An orb in a ring of six short spikes. | the Nair Ring's orbs (TR_WHILE_NAIR) |
| `orb_dash` | 181 | Orb: dash | An orb with a chevron cut ahead of it. | the Dash Orb's body (TR_WHILE_DASH) |
| `orb_shield` | 182 | Orb: shield | A small faceted bubble with a bright band. | the Shield Orbs round the shield (TR_WHILE_SHIELD); they break on blocks |
| `fx_refill` | 213 | Effect: stock refill | A heart rising with two chevrons under it and a glint. | the campfire's refill tick over each player; a shop stock bought |

## Boss and elite faces (4)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `boss_petey` | 183 | Boss: Petey | A flower head: a red spotted bulb in a collar of yellow petals, the mouth wide with white teeth. | the boss node and the boss card when the act's boss is Petey (MELEE_ENVOY_BOSS=petey) |
| `boss_unknown` | 184 | Boss: unknown | A dark horned shape with a gold question mark. | the boss node before the act's boss is known, and any boss without a face yet |
| `elite_helm` | 185 | Elite: helm | A steel helm with a dark visor slit and a red crest. | an elite's card and node when the elite is a heavy (the lead foe's weight class) |
| `elite_mask` | 186 | Elite: mask | A violet mask with slanted bright eyes and two fangs. | an elite's card and node when the elite is a fast one |

## Synergy and evolution banner pieces (15)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `banner_ribbon` | 187 | Banner: ribbon | A white ribbon with forked tails folded behind it: tint it to the synergy's colour. | the synergy and evolution banners' name strip (tint with Synergy::rgba) |
| `banner_plate` | 188 | Banner: plate | A white cut-corner plate with a raised rim: the two items sit on it. | behind the two item sprites on a synergy banner (tint with Synergy::rgba, 40% toward white) |
| `banner_plus` | 189 | Banner: plus | A gold plus: this item and that one. | between the two item sprites on a synergy banner |
| `banner_arrow` | 190 | Banner: becomes | A gold arrow pointing right: this item becomes that one. | between the base item and its evolved form on an evolution banner |
| `syn_thunder_bell` | 191 | Synergy: Thunder Bell | The shine's hexagon struck through by a lightning bolt. | the synergy banner and the pause Items page for `thunder_bell` (envoy_data::procs::SYNERGIES) |
| `syn_spectrum` | 192 | Synergy: Spectrum | A prism throwing five bolts, each a different colour, under a halo. | the synergy banner and the pause Items page for `spectrum` (envoy_data::procs::SYNERGIES) |
| `syn_wildfire` | 193 | Synergy: Wildfire | A seed split open with three flames leaping from it. | the synergy banner and the pause Items page for `wildfire` (envoy_data::procs::SYNERGIES) |
| `syn_blood_moon` | 194 | Synergy: Blood Moon | A red crescent moon with one drop falling from its lower horn. | the synergy banner and the pause Items page for `blood_moon` (envoy_data::procs::SYNERGIES) |
| `syn_iron_maiden` | 195 | Synergy: Iron Maiden | A dark red casket standing upright, thorns out of both sides and a slit of light down it. | the synergy banner and the pause Items page for `iron_maiden` (envoy_data::procs::SYNERGIES) |
| `syn_tectonic` | 196 | Synergy: Tectonic | The floor broken in two slabs tilted apart, one tall tooth of rock between them. | the synergy banner and the pause Items page for `tectonic` (envoy_data::procs::SYNERGIES) |
| `syn_superconductor` | 197 | Synergy: Superconductor | A coil of three rings with frost on it and a bolt jumping from its top. | the synergy banner and the pause Items page for `superconductor` (envoy_data::procs::SYNERGIES) |
| `syn_seeker_stars` | 198 | Synergy: Seeker Stars | Two stars, each on a curling purple trail. | the synergy banner and the pause Items page for `seeker_stars` (envoy_data::procs::SYNERGIES) |
| `evo_storm_crown` | 199 | Evolution: Storm Crown | A crown whose three points are lightning bolts. | the evolution banner and the evolved item's chip for `storm_crown` (envoy_data::procs::EVOLUTIONS) |
| `evo_eclipse` | 200 | Evolution: Eclipse | A black moon in a gold rim, a flare breaking over one edge. | the evolution banner and the evolved item's chip for `eclipse` (envoy_data::procs::EVOLUTIONS) |
| `evo_inferno_seed` | 201 | Evolution: Inferno Seed | The seed wearing a crown of five flames. | the evolution banner and the evolved item's chip for `inferno_seed` (envoy_data::procs::EVOLUTIONS) |

## Run-end pieces (3)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `run_trophy` | 208 | Run: trophy | A gold cup with two handles on a dark base, a star on its bowl. | the run-end summary for a cleared run; the loop-cleared banner |
| `run_tomb` | 209 | Run: tombstone | A round-topped stone with a crack and a small heart outline cut in it. | the run-end summary for a lost run (RUN OVER) |
| `run_loop` | 210 | Run: loop | An arrow running round in a circle with a pip at its centre: one more loop. | the loop counter ("loop now N"), the act banner |

## Curse-pool marks (8)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `curse_mark` | 235 | Curse: mark | A purple skull under three thorns: this item is cursed. | the corner of a curse-pool item's card and chip; the curse pool's pedestal |
| `curse_shared` | 236 | Curse: shared | Two arrows passing each other: foes get it too. | the drawback line on a curse-pool card (design 7, the `curse` pool): the effect also works for foes (Haunted Shell) |
| `curse_fragile` | 237 | Curse: fragile | A heart split by a crack, a chip falling from it: you are launched further. | the drawback line on a curse-pool card (design 7, the `curse` pool): you take more knockback (Glass Fist) |
| `curse_weak` | 238 | Curse: weak | A sword snapped in two, the point hanging off to one side. | the drawback line on a curse-pool card (design 7, the `curse` pool): you deal less |
| `curse_slow` | 239 | Curse: slow | A snail: a spiral shell on a low body with two feelers. | the drawback line on a curse-pool card (design 7, the `curse` pool): you move or act slower |
| `curse_heavy` | 240 | Curse: heavy | An iron weight with a ring on top and a down arrow on its face. | the drawback line on a curse-pool card (design 7, the `curse` pool): you fall faster or jump lower |
| `curse_costly` | 241 | Curse: costly | A coin with a red arrow falling out of it. | the drawback line on a curse-pool card (design 7, the `curse` pool): it costs coins, or shops cost more |
| `curse_stock` | 242 | Curse: stock | The heart container with a red minus on its shoulder: a stock slot gone. | the drawback line on a curse-pool card (design 7, the `curse` pool): it takes a stock slot or a stock |

## Camp and road scenery (12)

| key | id | name | concept | where to use it |
|---|---|---|---|---|
| `camp_tent` | 245 | Camp: tent | A teal tent with a cream stripe, its door flap open on the dark inside. | the camp screen behind the party cards |
| `camp_lantern` | 246 | Camp: lantern | A lantern with a lit glass, a cap and a carrying ring. | the camp screen; a road stop at night |
| `camp_board` | 247 | Camp: notice board | A board on two posts with three notes pinned to it. | the camp's run record (best loop, last run); the pause Run page header |
| `camp_seat` | 248 | Camp: log seat | A log on its side with a mushroom growing by it. | the camp screen, beside the fire |
| `camp_pot` | 249 | Camp: cooking pot | A pot hung from a tripod over a small flame, steam curling up. | the campfire room; the camp screen |
| `road_sign` | 250 | Road: signpost | A post with two arrow boards pointing opposite ways. | the road screen at a fork; the act map's foot |
| `road_stone` | 251 | Road: milestone | A low round-topped stone with one pip cut in it, grass at its foot. | the road screen: one per fight cleared |
| `road_pine` | 252 | Road: pine | A pine of three stacked tiers on a short trunk. | the road and camp backdrops |
| `road_bush` | 253 | Road: bush | A bush of three round lobes with red berries. | the road and camp backdrops |
| `road_rock` | 254 | Road: rock | A boulder with a smaller stone leaning on it. | the road and camp backdrops; a secret wall's rubble |
| `road_fence` | 255 | Road: fence | Two posts and two rails. | the road screen's edge; the camp's edge |
| `road_gate` | 256 | Road: finish gate | Two poles with a string of red and gold pennants between them. | the end of the road (the boss ahead); the loop-cleared banner |

## Chunk 5 proposals

1. **Finish what chunk 4 could not check** (see Verified): the before / after crops, items_check, the frame-time
   run. Until then the mesh swaps are unproven against the shader bodies they replace.
2. **Beams as a stretched quad or a tiled mesh** (a length parameter), then the gold and brimstone beams.
3. **Use the map and HUD pieces**: nodes, cursor, flag and tick on the act map; hearts on the stock row; the A
   button on the pickup prompt once the pickup lane lands.
4. **Animated strips for more pieces** (the wish flag, the lantern, the loop arrow, the trophy's glint).
5. **More boss faces** as bosses are added, and a face per elite archetype.
6. **Items still to come** from the design's first 40 (two curse items are here already).

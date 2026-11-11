# Envoy V3 view: the words envoy_sim publishes for the UI (port/v3sim, 2026-11-10)

The contract between `rust/mods/envoy_sim` (writes, every frame, `gd_envoy_publish`) and `rust/mods/envoy_ui`
(reads, `gd_ui_envoy`). The code is `rust/mods/envoy_data/src/view.rs`: always use its constants, never raw numbers.
This note says what every word means. It follows `envoy-synergy-v3.md` 12.8 (port/synergy3); where a name differs
from the spec it says so. V2's words keep their names; many moved (the view grew 512 -> 768 and slots 4 -> 5).

Conventions: i32 words. Positions are f32 bits. Drive / keystone ids are pool index + 1 (0 none). "x1000" means the
value times 1000 (Heat 40 = 40000, 6% = 6000). A word for a player who is not present is 0.

## Layout at a glance

| range | what |
|---|---|
| 0-31 | the run |
| 32-191 | player 1 (`V_P0 + 0 * V_PW`), 160 words |
| 192-351 | player 2 (`V_P0 + 1 * V_PW`) |
| 352-431 | match slots 0-3 (`V_S0 + s * V_SW`), 20 words each |
| 432-528 | the fire log (12 x 8) and its count `V_LOG_N` (528) |
| 536-600 | the event ring (16 x 4) |
| 600-605 | the forked road and the bank: `V_ROAD_N`, `V_ROAD_CUR`, `V_ROAD_FLAG`, `V_ROAD_TAKEN`, `V_BANK_Q`, `V_BANK_KIND` |
| 608-679 | the roads on offer (`V_ROAD0 + r * V_ROADW`, 3 x 24) |
| 704-751 | the run record (48 words, version 8) |
| 760-765 | sessions: `V_SESS_NEXT`, `V_BANK`, `V_BANK_N`, `V_BANK_I`, `V_SESS_KINDS`, `V_SESS_FIGHT` (envoy_data view.rs) |

`V_MAGIC` = `0x454E5633` ("ENV3"). `VIEW_WORDS` = 768 (the host's `melee_modes::tu::envoy::VIEW_WORDS` matches).

## The run (0-31)

V2's 0-24 are unchanged in meaning. `V_RW` gains 3 = the shop (`RW_SHOP`); `V_OPEN` is 3 / 4 / 5.

| word | meaning |
|---|---|
| `V_COINS` 25 | whole coins in the party purse (one purse, both players) |
| `V_EV_N` 26 | events written so far; the newest is at ring index `(n - 1) % EV` |
| `V_REROLL_COST` 27 | the shop's reroll price now (10, then +10 per reroll this run) |
| `V_CHAIN_N` 28 | chain links fired this fight |
| `V_HEAT_ON` 29 | 1 on this build (lets a UI tell V3 from V2) |
| `V_SHOP_DUE` 30 | 1: this fight's opening screens end with a shop |

## A player (`b = V_P0 + p * V_PW`)

V2 words (moved): `P_PRESENT 0, P_MAIN 1, P_PARTNER 2, P_ACTIVE 3` (the fighter in play, 0 main / 1 partner),
`P_TAG_CD 4, P_TAG_MAX 5, P_PARTNER_KIND 6, P_X 7, P_Y 8, P_HAVE_POS 9, P_KEYS 10-12, P_LAST_PICK 13,
P_LAST_WEARER 14`.

**The rows** (5 slots per fighter, **in order: index 0 is the leftmost slot**; slots past `V_OPEN` are closed):

| word | meaning |
|---|---|
| `P_DRIVES` 16 (10) | `[fighter][slot]` drive id + 1 |
| `P_LEVELS` 26 (10) | level 1-3 |
| `P_CHARGE` 36 (10) | 0-1000, 1000 = ready. Damage-charged and timed drives alike. Insulate: its allowance left as 0-1000. Other non-triggers: 0 |
| `P_LIVE_SETS` 46 | bit f: fighter f's row is live (in play, or the bench with Twin Soul) |
| `P_SET` 47 (2) | per fighter: a completed set (`SET_INFERNO` = 1: three different RED drives), 0 none |

**The pick screen** (`V_RW` 1 drives, 2 keystones): `P_RW_DONE 49, P_CURSOR 50` (card 0-2), `P_STEP 51`
(0 card, 1 fighter, **2 the slot the new drive replaces**: the row is full and the card is not a duplicate),
`P_FCURSOR 52`, `P_OFFERS 53-55` (pool index, or `OFFER_STOCK`), `P_CAN_TAKE 56` (bit `i * 2 + f`: card i fits
fighter f without replacing anything), `P_SCURSOR 57` (the slot cursor in step 2, and in the shop's slot steps).

Pick input (the sim reads pads; the UI only draws): step 0: left / right moves, A takes. Step 1: left / right
toggles the fighter, A confirms, B back. Step 2 (full row): left / right moves `P_SCURSOR` over the open slots,
A replaces that drive (it is gone; the new drive takes its slot), B back to step 1 (or step 0). The timer still
auto-picks card 0 for the default fighter; a full row at time-out skips the card.

**Other V2 words**: `P_DROPS 58` (2 x state, id + 1, x100, y100), `P_HASTE 66` (2, frames left), `P_SHIELD 68` (2),
`P_BANKED_JUMP 70`, `P_GAIN 71` (6: frame, id + 1, fighter, level, source, count).

**Heat** (12.8):

| word | unit | meaning |
|---|---|---|
| `P_HEAT` 80 | Heat x1000 | the fill. Gains need no event: draw them from the change |
| `P_HEAT_CAP` 81 | Heat x1000 | 225000 |
| `P_HEAT_MARKS` 82-84 | Heat x1000 | Warm 40000, Hot 90000, Blazing 165000 (draw from these; hold no constants) |
| `P_HEAT_STATE` 85 | 0-3 | Cold, Warm, Hot, Blazing |
| `P_HEAT_FULL` 86 | 0/1 | at the cap |
| `P_HEAT_FROZEN` 87 | 0/1 | not stepping (tag cinematic, a pick / shop freeze, the collect hold) |
| `P_GAIN_MUL` 88 | x1000 | the gain multiplier live now (1000 = x1) |
| `P_GAIN_SRC` 89 | bits | bit `f * 5 + k`: that xGain drive is live now; bit 15: Glass Core |
| `P_FREE` 90, `P_FREE_MAX` 91 | percent x1000 | Insulate's allowance left / full (0 / 0 without Insulate) |
| `P_ACT_ID` 92 | id | the active: `ACTIVE_VOLLEY` = 1 (spec name `P_ACTIVE`; renamed, V2 uses that name) |
| `P_PIPS` 93, `P_PIPS_NEED` 94, `P_PIP_PART` 95 | | whole pips; pips a use costs (3); 0-999 toward the next |
| `P_MARKS` 96-98 | frames | live technique marks: L-cancel (20), wavedash (60), tech (30) |
| `P_EARNED` 99 | coins | this player's earned total |
| `P_HEAT_SCALE` 100 | x1000 | what the state does to fires now: 1000 / 1250 / 1500 / 2000 |
| `P_LCANCEL_STREAK` 101 | 0-3 | Perfect Form's count |
| `P_TALLY` 102-105 | percent x1000 | dealt to match slot s this stock; 10000 or more cashes out on its KO |
| `P_ACT_READY` 106 | 0/1 | D-pad right fires the active now (prompt) |
| `P_CHAIN` 107 | links | the running chain's link count on the fighter in play (0 none) |
| `P_HEAT_PEAK` 108 | Heat x1000 | highest this fight |
| `P_CASHOUTS` 109 | count | cash-outs this fight |

**The shop** (`V_RW` = 3; one list per player; both shop at once in co-op):

| word | meaning |
|---|---|
| `P_SHOP_OFFERS` 112-114 | pool index of each drive on sale, -1 sold |
| `P_SHOP_PRICE` 115-117 | its price in coins (Common 30, Rare 80, Epic 150) |
| `P_SHOP_CURSOR` 118 | the row: `SHOP_DRIVE0` 0-2, `SHOP_LEVEL` 3, `SHOP_STOCK` 4 (100), `SHOP_REROLL` 5, `SHOP_REORDER` 6, `SHOP_LEAVE` 7 |
| `P_SHOP_STEP` 119 | 0 the list; 1 choose a slot (to level, or to replace when buying onto a full row); 2 reorder: choose the drive; 3 reorder: moving it |
| `P_SHOP_FIGHTER` 120 | the fighter the shop acts on (0 main, 1 partner) |
| `P_SHOP_DONE` 121 | 1: this player left |
| `P_SHOP_MSG` 122 | the last result, `SM_*`: 1 bought, 2 not enough coins, 3 levelled, 4 stock restored, 5 rerolled, 6 moved, 7 full: choose a slot, 8 replaced, 9 can't |
| `P_SHOP_LEVEL_PRICE` 123-127 | per slot of the shop fighter: the price to level it (60 to level 2, 120 to level 3), 0 = empty or max |

Shop input: up / down (stick or D-pad) moves the row; **X or Y toggles the fighter** (pairs with a partner); A buys /
enters a step; B backs out of a step (B on the list does nothing: leave with the Leave row, or the timer).
Level: step 1, left / right over the slots, A levels. Reorder: step 2, left / right picks a drive, A lifts it;
step 3, left / right swaps it with its neighbour (each move is shown at once), A or B puts it down. Buying onto a
full row: step 1, choose the slot to replace, A replaces, B cancels (no coins spent). The shop has no timer
pressure beyond `V_TIMER` (it auto-leaves at 0).

## A match slot (`b = V_S0 + s * V_SW`)

V2: `S_ALIVE 0, S_X 1, S_Y 2, S_BURN 3` (stacks), `S_CHILL 4, S_MARK 5` (frames left), `S_FOE 6, S_BOSS 7,
S_DRIVES 8-9, S_LEVELS 10-11, S_CHARGE 12-13, S_HASTE 14, S_SHIELD 15`. New:

| word | meaning |
|---|---|
| `S_BREAKS` 16 | a boss with HP: quarters broken 0-4 (each is a cash-out) |
| `S_PCT` 17 | the sim's own percent of this fighter this stock x1000 (`foe_m`; the 150% overkill line) |
| `S_HP` 18, `S_HP_MAX` 19 | a boss with HP (Petey): HP left / full, x1000; 0 otherwise |

## The fire log (432; 12 entries of 8 words; `V_LOG_N` 528 = entries written)

Entry: frame, id + 1, owner (player * 2 + fighter 0-3; a foe slot + 4), target slot, x100, y100, **level word**,
flags. The level word is `level | damage_x1000 << 4` (level = `w & 0xF`, the fire's dealt percent x1000 = `w >> 4`,
already scaled by level and Heat: use it to size the effect). Flags: V2's `FL_ECHO 1, FL_VOLLEY 2, FL_LINK 4,
FL_LAUNCH 8, FL_PASSIVE 16`, plus `FL_ACTIVE 32` (fired by Volley), `FL_FLARE 64` (the cash-out flare: id is the
cashing player's first drive or 0; draw a flare), `FL_INFERNO 128` (a RED fire under the Inferno set), the chain
link number in bits 8-11 (1 = the chain's first fire; 0 not a chain), the parent's log index + 1 in bits 12-15 (0
none: draw the link flash from that entry's position to this one), the Heat state at the fire in bits 16-17.

## The event ring (536; 16 entries of frame, kind, who, amount; `V_EV_N` = count)

| kind | who | amount |
|---|---|---|
| `EV_SPARK` 1 | player | `MARK_LCANCEL` 1, `MARK_WAVEDASH` 2, `MARK_TECH` 3, `MARK_OFFSTAGE` 4 (written whether or not a drive paid) |
| `EV_HEAT_ADD` 2 | player | `slot << 24 \| Heat x1000` (a +Heat drive paid) |
| `EV_STATE_UP` 3 / `EV_STATE_DOWN` 4 | player | the new state 0-3 |
| `EV_CASHOUT` 5 | player | Heat cashed x1000 |
| `EV_PAY` 6 | player | coins paid (whole) |
| `EV_PIP` 7 | player | thousandths of a pip paid |
| `EV_FLARE` 8 | the foe slot hit | percent x1000 |
| `EV_BREAK` 9 | the boss slot | quarters broken 1-4 |
| `EV_ACTIVE` 10 | player | the active's id |
| `EV_LOSS` 11 | player | Heat lost x1000 this frame (one event per frame per player) |
| `EV_HALVED` 12 | player | Heat x1000 after a lost stock halved it |
| `EV_INFERNO` 13 | player * 2 + fighter | 1: the set is live |
| `EV_COINS` 14 | player | coins from a flat award (fight clear, interest) |

## The record (704; version 6)

Saved by the host; `REC_AT` = 704 there. Words 0-19 as V2; `R_DRIVES` 20-39 (`[4 sets][5]`, `(id + 1) << 4 | level`
in slot order; version 5: `<< 2`), `R_POOL_DIGEST` 40, `R_COINS` 41, `R_REROLLS` 42, `R_ACTIVE` 43, `R_PIPS` 44,
`R_EARNED` 45, `R_LOOP` 46 (the loop; 0 in a version-5 record reads as loop 1), 47 digest. Version 5 is still read
(`view::rec_drive`); a version-4 record is dropped (a new run).

Endless loops also use run word 31 `V_LOOP` (the loop, 1 first) and player word 128 `P_SHOP_STOCK_PRICE` (the
shop's restore-a-stock price, which rises per loop).

## The forked road (600-679; port/v3g, 2026-11-11; envoy-macro-loop.md M1, first slice)

At the road screen before a three-fight session (fight 1 of every loop) the run offers `V_ROAD_N` roads (2 now:
Open, Steep; 0 = no fork, draw the single poster as before). The road screen draws one poster per road, left to
right in index order.

| word | meaning |
|---|---|
| `V_ROAD_N` 600 | roads on offer now (0 outside a fork) |
| `V_ROAD_CUR` 601 | P1's cursor, 0-based. Left / right moves it (the sim reads the pad); A on the road screen ("Start the fight") takes that road |
| `V_ROAD_FLAG` 602 | co-op: P2's flag, road index + 1 (0 none). P2's left / right moves it. A wish, it decides nothing |
| `V_ROAD_TAKEN` 603 | the road kind of the session in play (or the last one taken); 0 = none, read as Open |
| `V_BANK_Q` 604 | the bank as tokens: `RK_*` 4 bits each, the next to open lowest (`view::token(q, i)`); `V_BANK` is the count |
| `V_BANK_KIND` 605 | at a stop, the token being opened now (`RK_PICK` "Reward i of n" cards; `RK_LEVEL` "Level up": the cards are drives you wear below the level cap and taking one raises it a level); 0 outside a stop |

A road, `b = V_ROAD0 + r * V_ROADW` (`r` < `V_ROAD_N`):

| word | meaning |
|---|---|
| `RD_KIND` +0 | `ROAD_OPEN` 1, `ROAD_STEEP` 2 (`ROAD_SWARM` 3 reserved). Names: "Open road", "Steep road" |
| `RD_DANGER` +1 | the danger word: `DANGER_CALM` 0 "calm", `DANGER_ROUGH` 1 "rough", `DANGER_STEEP` 2 "steep" |
| `RD_NF` +2 | fights in the session (3) |
| `RD_FIGHTS` +3 | `RD_NF` x `RD_FW` (4) words: fight kind + 1 (`FK_*` + 1, as `V_SESS_KINDS`), the lead foe's character id (CK), the stage id, foes in the fight |
| `RD_TWISTS` +15 | twist bits: `TW_FOE_DRIVE` "every foe carries one more drive" (the others reserved: `TW_MORE_FOES`, `TW_BIG_FOES`, `TW_FAST_SHIFT`) |
| `RD_PRIZE` +16 | what the road banks, as tokens (`RK_*`, 4 bits each, in the order they open) |
| `RD_PRIZE_N` +17 | tokens in `RD_PRIZE` (3) |
| `RD_FOE_DRIVES` +18 | drives the lead foe of each fight carries on this road, 4 bits per fight |

Open road: the fights as today, prize `RK_PICK` x3, no twist, calm. Steep road: the same fights with
`TW_FOE_DRIVE`, prize `RK_PICK, RK_PICK, RK_LEVEL`, steep.

Reward kinds (`RK_*`, envoy-macro-loop.md 2.2), all eight defined, two live: `RK_PICK` 1, `RK_FAMILY` 2, `RK_RARE` 3,
`RK_LEVEL` 4, `RK_COINS` 5, `RK_STOCK` 6, `RK_ELITE_DRIVE` 7, `RK_KEYSTONE` 8. The bank holds up to `BANK_MAX` 7.

The shop's level-up row sells one level per visit now: after one, that row reads as unavailable (`SM_CANT`).

The record is version 8: `R_LOOP` = loop (8 bits) | stop done << 8 | road taken << 9 (3 bits) | bank tokens << 12
(5 tokens). A version 7 record reads as the Open road with its bank count as picks.

## Drive protection (player words 129-147; port/v3g, 2026-11-11)

Nothing a player wears is replaced without a confirm; a drive can be pinned, scrapped and moved between the
player's two fighters at the stop's reorder screen.

| word | meaning |
|---|---|
| `P_PINS` 129 | pinned slots: fighter 0's five bits, fighter 1's `<< 8`. A pinned drive is never offered as the one to replace or scrap; it is kept in the run record |
| `P_SCRAP_PRICE` 130-139 | `[fighter][slot]`: the coins scrapping that drive pays (a third of its shop value: tier price plus its level steps, / 3); 0 for an empty or pinned slot |
| `P_CONFIRM` 140 | 1: a replacement waits for its confirm. Draw "Replace X with Y for good?" with Keep and Replace |
| `P_CONFIRM_CUR` 141 | 0 Keep (the cursor starts here), 1 Replace. Left / right or up / down move it; B keeps |
| `P_CONFIRM_OLD` 142 / `P_CONFIRM_NEW` 143 | the drive that would go and the one that would come in (id + 1). `P_CONFIRM_NEW` 0 = a scrap ("Scrap X for N coins?") |
| `P_CONFIRM_AT` 144 | fighter * 8 + slot |
| `P_REORDER_ROW` 145 | the reorder screen's cursor row (0 main, 1 partner): the screen shows both rows; up / down change row |
| `P_REORDER_LIFT` 147 | 1: a drive is lifted: left / right move it in its row, up / down move it to the same slot of the other row (a swap when that slot is filled; refused when the other fighter already wears that drive) |
| `P_DECLINE` 146 | coins "Leave it" (the pick screen's Skip row) pays for the offer now; 0 = Skip pays nothing. Uniques will use it |

Reorder screen buttons (shop row `SHOP_REORDER`, the cursor on a drive): A lifts / drops, Y pins or unpins, X
scraps (through the confirm). New shop messages: `SM_SCRAPPED` 10, `SM_PINNED` 11 (also "pinned: cannot be
replaced"), `SM_KEPT` 12.

Every path that can overwrite a worn drive goes through the confirm: a pick onto a full row (step 2 then the
confirm), a shop purchase onto a full row, a banked extra pick. Drops and the end-of-fight fly-in only fill a free
slot or level a duplicate; with no room they bank a pick.

Record v8: a drive word's bit 12 is its pin (`view::rec_drive` masks it; `view::rec_pin`).

## Drop gains for the HUD (port/v3g, 2026-11-11)

- `P_GAIN` (71, 6 words: frame, id + 1, fighter, level, source, count) gains source `G_BANKED` 6: a drop with no
  room (a full row, or its duplicate at the level cap) was banked as a pick. Its level word is 0. This is "Row full.
  Frost is banked."
- `P_GAIN_FROM` (148, 3 words) holds the drop behind the latest drop gain (worn, levelled, banked, or flown in): its
  world x and y (x100), then the owner's slot. A drop's gain sources are 1 (walked over), 4 (fly-in) and 6
  (banked). For gains of other sources these words are stale; ignore them.
- During the end-of-fight fly-in (30 frames after the last KO) each drop's `P_DROPS` x / y moves toward its owner
  every frame. The drop is taken on the last frame (source 4). The trail can be drawn from those positions.

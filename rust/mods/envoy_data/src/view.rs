// The run view envoy_sim publishes (`gd_envoy_publish`, 768 words) and envoy_ui reads (`gd_ui_envoy`): one layout,
// shared here so the two mods cannot disagree. Positions are f32 bits; ids are pool index + 1 (0: none).
// V3 (Heat): docs/rust-port/notes/envoy-v3-view.md is the prose contract for every word below.

pub const VIEW_WORDS: usize = 1536;
pub const VIEW_MAGIC: i32 = 0x454E_5633; // "ENV3"

// ---- the run
pub const V_MAGIC: usize = 0;
pub const V_STATE: usize = 1; // 0 none, 1 live, 2 clear, 3 over
pub const V_FIGHT: usize = 2;
pub const V_FIGHTS: usize = 3;
pub const V_POOL: usize = 4;
pub const V_LIVE: usize = 5;
pub const V_HUMANS: usize = 6; // 1 solo, 2 co-op
pub const V_SHARED: usize = 7;
pub const V_RW: usize = 8; // the reward screen: 0 none, 1 drives, 2 keystones, 3 shop (RW_*)
pub const V_TIMER: usize = 9;
pub const V_RW_TIME: usize = 10;
pub const V_FROZEN: usize = 11;
pub const V_PLAY: usize = 12;
pub const V_DIGEST: usize = 13;
pub const V_STAGE_IN: usize = 14; // retired with the timed switch: always -1 (switches are events: EV_SHIFT)
pub const V_FIRES: usize = 15;
pub const V_FIGHT_KIND: usize = 16; // FK_*
pub const V_OPEN: usize = 17; // drive slots open per fighter: 3, 4 after boss 1, 5 after boss 2
pub const V_KEYS_EARNED: usize = 18;
pub const V_SEQ: usize = 19;
pub const V_ARMED: usize = 20;
pub const V_NCPU: usize = 21;
pub const V_SINCE_PICK: usize = 22;
pub const V_BOSSES: usize = 23;
pub const V_OUTCOME: usize = 24;
// V3
pub const V_COINS: usize = 25; // whole coins in the party purse
pub const V_EV_N: usize = 26; // events written so far (ring index = n % EV)
pub const V_REROLL_COST: usize = 27; // the shop's reroll price now
pub const V_CHAIN_N: usize = 28; // chain links fired this fight (all players)
pub const V_HEAT_ON: usize = 29; // 1: the V3 rules (always 1 on this build)
pub const V_SHOP_DUE: usize = 30; // 1: a shop opens after this fight's picks
pub const V_LOOP: usize = 31; // the loop (1 first time through; endless NG+ loops after a clear)

pub const FK_DUEL: i32 = 0;
pub const FK_PACK: i32 = 1;
pub const FK_ELITE: i32 = 2;
pub const FK_BOSS: i32 = 3;

pub const RW_NONE: i32 = 0;
pub const RW_DRIVES: i32 = 1;
pub const RW_KEYS: i32 = 2;
pub const RW_SHOP: i32 = 3;

// ---- a player (human slot p): block at V_P0 + p * V_PW
pub const V_P0: usize = 32;
pub const V_PW: usize = 160;
pub const P_PRESENT: usize = 0;
pub const P_MAIN: usize = 1; // character kinds of the pair
pub const P_PARTNER: usize = 2;
pub const P_ACTIVE: usize = 3; // fighter in play: 0 main, 1 partner
pub const P_TAG_CD: usize = 4;
pub const P_TAG_MAX: usize = 5;
pub const P_PARTNER_KIND: usize = 6; // gd_tag_partner: -1 no tag
pub const P_X: usize = 7;
pub const P_Y: usize = 8;
pub const P_HAVE_POS: usize = 9;
pub const P_KEYS: usize = 10; // 3
pub const P_LAST_PICK: usize = 13;
pub const P_LAST_WEARER: usize = 14;
pub const P_DRIVES: usize = 16; // [2 fighters][SLOTS], in slot order (left = first)
pub const P_LEVELS: usize = 26; // [2][SLOTS]
pub const P_CHARGE: usize = 36; // [2][SLOTS] permille; 1000 = ready (timed drives too; Insulate: its allowance left)
pub const P_LIVE_SETS: usize = 46; // bit f: set f orbits and fires now (in play, or benched with Twin Soul)
pub const P_SET: usize = 47; // [2] a set the fighter completes (SET_* id, 0 none): Inferno = 1
pub const P_RW_DONE: usize = 49;
pub const P_CURSOR: usize = 50; // the card cursor (0-2)
pub const P_STEP: usize = 51; // 0 the card, 1 the fighter, 2 the slot a new drive replaces (full row)
pub const P_FCURSOR: usize = 52; // the fighter cursor (0 main, 1 partner)
pub const P_OFFERS: usize = 53; // 3: pool index or OFFER_STOCK
pub const P_CAN_TAKE: usize = 56; // bit i * 2 + f: offer i fits fighter f WITHOUT replacing (a free slot or a level-up)
pub const P_SCURSOR: usize = 57; // the slot cursor (step 2 here, and the shop's slot steps)
pub const P_DROPS: usize = 58; // DROPS x (state, id + 1, x100, y100)
pub const P_HASTE: usize = 66; // [2] frames left
pub const P_SHIELD: usize = 68; // [2] orb armed
pub const P_BANKED_JUMP: usize = 70;
pub const P_GAIN: usize = 71; // 6: frame, id + 1, fighter, level (0: banked), source (G: 1 drop, 2 pick, 3 keystone, 4 fly-in, 5 shop, 6 banked: no room), count
// ---- V3: Heat (envoy-synergy-v3.md 12.8)
pub const P_HEAT: usize = 80; // Heat x1000
pub const P_HEAT_CAP: usize = 81; // Heat x1000 (225000)
pub const P_HEAT_MARKS: usize = 82; // 3: Warm, Hot, Blazing, Heat x1000 (40000, 90000, 165000)
pub const P_HEAT_STATE: usize = 85; // 0 Cold, 1 Warm, 2 Hot, 3 Blazing
pub const P_HEAT_FULL: usize = 86; // 1: at the cap (damage is being discarded)
pub const P_HEAT_FROZEN: usize = 87; // 1: the meter is not stepping (tag cinematic, reward freeze, hold)
pub const P_GAIN_MUL: usize = 88; // x1000: the gain multiplier live right now
pub const P_GAIN_SRC: usize = 89; // bits: bit (f * SLOTS + k) set = that xGain drive is live; bit 15 Glass Core
pub const P_FREE: usize = 90; // percent x1000 left of Insulate's allowance (0 without Insulate)
pub const P_FREE_MAX: usize = 91; // percent x1000 (20000 with Insulate, else 0)
pub const P_ACT_ID: usize = 92; // the active drive: ACTIVE_* id (Volley = 1), 0 none   [spec name: P_ACTIVE]
pub const P_PIPS: usize = 93; // whole pips held
pub const P_PIPS_NEED: usize = 94; // pips one use costs (3)
pub const P_PIP_PART: usize = 95; // 0-999 toward the next pip
pub const P_MARKS: usize = 96; // 3: frames left of the L-cancel, wavedash, tech marks
pub const P_EARNED: usize = 99; // whole coins this player earned this run
pub const P_HEAT_SCALE: usize = 100; // x1000: what the state does to fires now (1000 / 1250 / 1500 / 2000)
pub const P_LCANCEL_STREAK: usize = 101; // 0-3: the last aerial landings that were L-cancelled (Perfect Form)
pub const P_TALLY: usize = 102; // [4] percent x1000 this player dealt to match slot s this stock (10000 cashes out)
pub const P_ACT_READY: usize = 106; // 1: D-pad right fires the active now
pub const P_CHAIN: usize = 107; // the fighter in play's chain: links fired so far in the running chain (0 none)
pub const P_HEAT_PEAK: usize = 108; // the highest Heat x1000 this fight
pub const P_CASHOUTS: usize = 109; // cash-outs this fight
pub const P_BREAK_CD: usize = 110; // the combo breaker's cooldown: frames left (0: ready)
pub const P_BREAK_MAX: usize = 111; // its full cooldown, frames
// the shop (V_RW == RW_SHOP): one list per player
pub const P_SHOP_OFFERS: usize = 112; // 3: pool index of the three drives on sale (-1 sold)
pub const P_SHOP_PRICE: usize = 115; // 3: their prices (coins)
pub const P_SHOP_CURSOR: usize = 118; // the item cursor: SHOP_* row
pub const P_SHOP_STEP: usize = 119; // 0 the list, 1 the slot (level / replace), 2 reorder: picking, 3 reorder: moving
pub const P_SHOP_FIGHTER: usize = 120; // the fighter the shop acts on (0 main, 1 partner)
pub const P_SHOP_DONE: usize = 121; // 1: left the shop
pub const P_SHOP_MSG: usize = 122; // the last result: SM_* (for a one-line message)
pub const P_SHOP_LEVEL_PRICE: usize = 123; // [SLOTS] the price to level the shop fighter's slot k (0: max level / empty)
pub const P_SHOP_STOCK_PRICE: usize = 128; // the price to restore a stock (rises per loop)
// ---- drive protection (port/v3g): nothing is replaced without a confirm; pin, scrap, move between the two fighters
pub const P_PINS: usize = 129; // pinned slots: fighter 0's 5 bits | fighter 1's << 8 (a pinned drive cannot be replaced or scrapped)
pub const P_SCRAP_PRICE: usize = 130; // [2 fighters][5 slots]: coins scrapping that worn drive pays (0: empty, pinned)
pub const P_CONFIRM: usize = 140; // a replacement waiting for its confirm: 0 none, 1 "Replace X with Y for good?"
pub const P_CONFIRM_CUR: usize = 141; // its cursor: 0 Keep (the start; B keeps), 1 Replace
pub const P_CONFIRM_OLD: usize = 142; // the worn drive's id + 1 that would go
pub const P_CONFIRM_NEW: usize = 143; // the drive's id + 1 that would come in
pub const P_CONFIRM_AT: usize = 144; // fighter * 8 + slot of the replacement
pub const P_REORDER_ROW: usize = 145; // the reorder screen's cursor row: the fighter (0 main, 1 partner); it spans both rows
pub const P_DECLINE: usize = 146; // coins "Leave it" pays on the offer now (0: the offer cannot be declined for coins)
pub const P_REORDER_LIFT: usize = 147; // 1: a drive is lifted (moves with left / right, up / down to the other row)
pub const P_TAG_BUF: usize = 151; // Quick Tag: frames a tag press is still remembered (0: none waiting)
pub const P_TAG_WINDOW: usize = 152; // frames left in the tag-cancel window after dealing damage (a tag now is a Handoff)
pub const P_TAG_BUTTON: usize = 153; // the tag button: 0 Y, 1 X, 2 D-pad down (classic cinematic tag)
pub const P_HANDOFF_ON: usize = 154; // 1: Handoff is on
pub const P_ASSIST: usize = 155; // Assist Strike: frames the partner's strike has run + 1 (0: no strike out)
pub const P_TAG_HOLD: usize = 156; // frames the Tag button has been held toward a swap (0: not held; P_TAG_HOLD_AT swaps)
pub const P_ASSIST_COST: usize = 157; // Heat a strike costs (the notch on the Heat bar), whole Heat
pub const P_TAG_HOLD_AT: usize = 158; // the hold that swaps, in frames (a release before it is a tap: a strike)
pub const P_GAIN_FROM: usize = 148; // 3: the drop behind the last drop gain (worn, levelled, banked, flown in): x, y (x100, world), owner slot

pub const DROPS: usize = 2;

// shop rows (P_SHOP_CURSOR)
pub const SHOP_DRIVE0: i32 = 0; // 0-2 the three items (envoy items: no level-ups, no reorder)
pub const SHOP_STOCK: i32 = 3;
pub const SHOP_SLOT: i32 = 4; // slice 2: a stock slot (+1 limit, +1 stock) for a high price, some shops (V_SHOP_SLOT_PRICE)
pub const SHOP_REROLL: i32 = 5;
pub const SHOP_LEAVE: i32 = 6;
pub const SHOP_ROWS: i32 = 7;
/// Retired with the drives (never a row now).
pub const SHOP_LEVEL: i32 = -10;
pub const SHOP_REORDER: i32 = -11;
// shop messages (P_SHOP_MSG)
pub const SM_NONE: i32 = 0;
pub const SM_BOUGHT: i32 = 1;
pub const SM_POOR: i32 = 2;
pub const SM_LEVELED: i32 = 3;
pub const SM_STOCK: i32 = 4;
pub const SM_REROLLED: i32 = 5;
pub const SM_MOVED: i32 = 6;
pub const SM_FULL: i32 = 7; // no room: choose a slot to replace
pub const SM_REPLACED: i32 = 8;
pub const SM_CANT: i32 = 9;
pub const SM_SCRAPPED: i32 = 10; // a drive scrapped for coins
pub const SM_PINNED: i32 = 11; // pinned / unpinned (and: a pinned drive cannot be replaced or scrapped)
pub const SM_KEPT: i32 = 12; // the confirm said Keep

// ---- every match slot (0-3): statuses, foe drives
pub const V_S0: usize = 352;
pub const V_SW: usize = 20;
pub const S_ALIVE: usize = 0;
pub const S_X: usize = 1;
pub const S_Y: usize = 2;
pub const S_BURN: usize = 3; // stacks
pub const S_CHILL: usize = 4; // frames left
pub const S_MARK: usize = 5;
pub const S_FOE: usize = 6;
pub const S_BOSS: usize = 7;
pub const S_DRIVES: usize = 8; // 2: a foe's drives
pub const S_LEVELS: usize = 10;
pub const S_CHARGE: usize = 12;
pub const S_HASTE: usize = 14;
pub const S_SHIELD: usize = 15;
pub const S_BREAKS: usize = 16; // 0-4: a boss's quarters broken (a Petey boss with HP; 0 otherwise)
pub const S_PCT: usize = 17; // the sim's own percent of this slot this stock, x1000 (foe_m)
pub const S_HP: usize = 18; // a boss with HP: HP left x1000 (0 none)
pub const S_HP_MAX: usize = 19; // a boss with HP: full HP x1000

// ---- the fire log: LOG entries of (frame, id + 1, owner, target slot, x100, y100, level, flags)
pub const V_LOG: usize = 432;
pub const LOG: usize = 12;
pub const LOG_W: usize = 8;
pub const V_LOG_N: usize = V_LOG + LOG * LOG_W; // 528
/// A fire's owner: a player's set (player * 2 + fighter, 0-3) or a foe's slot + OWNER_FOE.
pub const OWNER_FOE: i32 = 4;
pub const FL_ECHO: i32 = 1;
pub const FL_VOLLEY: i32 = 2;
pub const FL_LINK: i32 = 4;
pub const FL_LAUNCH: i32 = 8;
pub const FL_PASSIVE: i32 = 16;
/// V3 flags: the active (Volley) fired it (FL_ACTIVE), a cash-out flare (FL_FLARE), the Inferno set's look; the
/// chain link number 1-15 (bits 8-11), the parent's log index + 1 (bits 12-15), the Heat state at the fire (16-17).
pub const FL_ACTIVE: i32 = 32;
pub const FL_FLARE: i32 = 64;
pub const FL_INFERNO: i32 = 128;
pub const FL_LINK_SHIFT: i32 = 8;
pub const FL_PARENT_SHIFT: i32 = 12;
pub const FL_STATE_SHIFT: i32 = 16;
/// Log word 6 carries the level in bits 0-3 and the fire's damage (percent x1000) above: damage = word >> 4.
pub const LOG_LEVEL_MASK: i32 = 0xF;

// ---- the event ring (V3 12.8): EV entries of (frame, kind, who, amount)
pub const V_EV: usize = 536;
pub const EV: usize = 16;
pub const EV_W: usize = 4;
pub const EV_SPARK: i32 = 1; // a technique registered; who = player; amount = MARK_*
pub const EV_HEAT_ADD: i32 = 2; // a +Heat drive paid; who = player; amount = slot << 24 | Heat x1000
pub const EV_STATE_UP: i32 = 3; // who = player; amount = the new state
pub const EV_STATE_DOWN: i32 = 4; // who = player; amount = the new state
pub const EV_CASHOUT: i32 = 5; // who = player; amount = Heat cashed x1000
pub const EV_PAY: i32 = 6; // who = player; amount = coins paid (whole)
pub const EV_PIP: i32 = 7; // who = player; amount = thousandths of a pip paid
pub const EV_FLARE: i32 = 8; // who = the foe slot hit; amount = percent x1000
pub const EV_BREAK: i32 = 9; // who = the boss slot; amount = quarters broken (1-4)
pub const EV_ACTIVE: i32 = 10; // who = player; amount = the active's id
pub const EV_LOSS: i32 = 11; // who = player; amount = Heat lost x1000 this frame
pub const EV_HALVED: i32 = 12; // who = player; amount = Heat x1000 after a lost stock halved it
pub const EV_INFERNO: i32 = 13; // who = player * 2 + fighter; amount = 1: the set is complete (fight start / gained)
pub const EV_TAG: i32 = 15; // who = player; amount = the fighter now in play (0 main, 1 partner): a Quick Tag
pub const EV_SHIFT: i32 = 17; // a stage switch began; who = the reason (SH_*); amount = the destination ground kind
pub const EV_ASSIST: i32 = 18; // who = player; amount = the move (AS_*): the partner jumped in and strikes
pub const EV_ASSIST_END: i32 = 19; // who = player; amount = frames the strike ran: the partner left
pub const EV_HEAT_SHORT: i32 = 20; // who = player; amount = Heat x1000: a tap with under the strike's cost (shake the bar)
pub const EV_WAVE: i32 = 21; // who = the wave now starting (2..); amount = the room's waves: the next wave arrives
pub const EV_COMBOBREAK: i32 = 22; // who = player; amount = hits << 16 | the combo's percent (whole): the player burst free
pub const EV_HANDOFF: i32 = 16; // who = player; amount = the fired drive's id + 1 | 0x100 when Relay fired the row
pub const EV_COINS: i32 = 14; // who = player; amount = coins from a flat award (a fight cleared, interest)
// slice 2
pub const EV_DEAL: i32 = 23; // who = player; amount = item id | slots paid << 8: a devil deal taken
pub const EV_SLOT: i32 = 24; // who = player; amount = the new stock limit (a slot gained or paid)
pub const EV_SECRET: i32 = 25; // who = the player who broke the wall; amount = the fight (0-based)
pub const EV_DOOR: i32 = 26; // who = 0; amount = DOOR_*: the door after the boss opened
pub const EV_GUARD: i32 = 27; // who = player; amount = the copies left: the Guardian Feather ate a KO hit
pub const EV_ORB_BREAK: i32 = 29; // who = player; amount = item id: one of the player's orbs shattered (a foe's attack or projectile)
pub const EV_ORB_BLOCK: i32 = 30; // who = player; amount = item id: an orb blocked a foe projectile
pub const EV_CAMPFIRE: i32 = 28; // who = player; amount = stocks given back
/// Why the stage switched (EV_SHIFT's who).
pub const SH_BREAK: i32 = 1; // an elite or a stock-based boss lost a stock it can spare (Petey's arena never switches)
pub const SH_WAVE: i32 = 2; // a wave's last foe fell and the next wave arrives (reserved: waves)
pub const SH_KO: i32 = 3; // the Shifting road: a foe was KO'd (not the fight's last)
pub const SH_DAMAGE: i32 = 4; // retired (the damage trigger was cut after play); kept so the id is not reused
/// Assist Strike moves (EV_ASSIST's amount): ground forward / up / down smash, air forward / up / down air.
pub const AS_FSMASH: i32 = 0;
pub const AS_USMASH: i32 = 1;
pub const AS_DSMASH: i32 = 2;
pub const AS_FAIR: i32 = 3;
pub const AS_UAIR: i32 = 4;
pub const AS_DAIR: i32 = 5;
pub const MARK_LCANCEL: i32 = 1;
pub const MARK_WAVEDASH: i32 = 2;
pub const MARK_TECH: i32 = 3;
pub const MARK_OFFSTAGE: i32 = 4;

// ---- the run record (melee_modes::tu::envoy REC_*): saved by the host, handed back at arm
// ---- sessions (v3g; envoy-v3-rules.md "Sessions"): fights played back to back, picks banked for one stop
pub const V_SESS_NEXT: usize = 760; // 1: the fight was won and the next one is in the same session (no stop)
pub const V_BANK: usize = 761; // drive picks banked (taken at the next stop)
pub const V_BANK_N: usize = 762; // picks in this stop (for "Pick i of n"; 0 outside a stop)
pub const V_BANK_I: usize = 763; // the pick being made now, 1-based
pub const V_SESS_KINDS: usize = 764; // the fights of the session from the next fight on: kind + 1 per 4 bits, low first
pub const V_SESS_FIGHT: usize = 765; // the fight's place in its session, 1-based (1: the session's first)
pub const V_STOP_DONE: usize = 766; // 1: the session ended and its stop ran in the match: no results, the road next

// ---- the forked road (envoy-macro-loop.md M1, first slice): at the road before a three-fight session the run offers
// V_ROAD_N roads; P1's cursor picks one (left / right), A on the road screen takes it; P2 may plant a flag (no effect).
// Everything a poster shows is here, as data: more roads, twists and reward kinds need no new words.
pub const V_ROAD_N: usize = 600; // roads on offer at this road screen (0: no fork; the one session as before)
pub const V_ROAD_CUR: usize = 601; // P1's cursor, 0-based (the road that A takes)
pub const V_ROAD_FLAG: usize = 602; // P2's flag: road index + 1 (0: none). A wish only
pub const V_ROAD_TAKEN: usize = 603; // the road kind (ROAD_*) of the session in play or last chosen; 0 = none (open)
pub const V_BANK_Q: usize = 604; // the bank: reward tokens (RK_*) 4 bits each, the next one lowest; V_BANK = count
pub const V_BANK_KIND: usize = 605; // the token being opened at the stop now (RK_*; 0 outside a stop)
pub const V_ROAD0: usize = 608; // road r's words: V_ROAD0 + r * V_ROADW + RD_*
pub const V_ROADW: usize = 24;
pub const ROADS_MAX: usize = 3; // 608..680
pub const RD_KIND: usize = 0; // ROAD_*
pub const RD_DANGER: usize = 1; // the one danger word: DANGER_*
pub const RD_NF: usize = 2; // fights in the session (3)
pub const RD_FIGHTS: usize = 3; // [RD_NF][RD_FW]: fight kind + 1 (FK_*), the lead foe's character id, stage id, foes
pub const RD_FW: usize = 4;
pub const RD_TWISTS: usize = 15; // twist bits (TW_*)
pub const RD_PRIZE: usize = 16; // what it banks: tokens (RK_*) 4 bits each, the first lowest
pub const RD_PRIZE_N: usize = 17; // tokens in RD_PRIZE
pub const RD_FOE_DRIVES: usize = 18; // drives the lead foe of each fight carries, 4 bits per fight, the first lowest
pub const RD_SHIFT_COINS: usize = 19; // coins each stage switch pays on this road (0: none; the Shifting road)
pub const RD_SHIFT_CAP: usize = 20; // switches paid per fight at most
pub const RD_SHIFT_DMG: usize = 21; // retired: 0 (no damage trigger)

pub const ROAD_OPEN: i32 = 1; // the session as it is: banks three drive picks
pub const ROAD_STEEP: i32 = 2; // the same fights, every foe one more drive: banks two picks and a level-up
pub const ROAD_SWARM: i32 = 3; // reserved (waves, coins)
pub const ROAD_SHIFT: i32 = 4; // the same fights; the stage also switches on a foe KO (not the last), 2 a fight at most, 10 coins each
pub const DANGER_CALM: i32 = 0;
pub const DANGER_ROUGH: i32 = 1;
pub const DANGER_STEEP: i32 = 2;
/// Twists: the content pass's elite modifiers and M3's dares share this list. TW_FOE_DRIVE and TW_FAST_SHIFT are live.
pub const TW_FOE_DRIVE: i32 = 1 << 0; // every foe carries one more drive (to the 2 a foe can hold)
pub const TW_MORE_FOES: i32 = 1 << 1; // reserved: one more foe in every fight
pub const TW_BIG_FOES: i32 = 1 << 2; // reserved: foes are larger
pub const TW_FAST_SHIFT: i32 = 1 << 3; // the stage shifts on foe KOs (the Shifting road)

/// Reward kinds: what a bank token (and a road's prize) opens at the stop. Only RK_PICK and RK_LEVEL are live.
pub const RK_PICK: i32 = 1; // one of three drives
pub const RK_FAMILY: i32 = 2; // one of three, all from one family (reserved)
pub const RK_RARE: i32 = 3; // one of three, Rare or better (reserved)
pub const RK_LEVEL: i32 = 4; // raise one worn drive a level (the cards are the drives you wear below the cap)
pub const RK_COINS: i32 = 5; // coins (reserved)
pub const RK_STOCK: i32 = 6; // +1 stock to the pool (reserved)
pub const RK_ELITE_DRIVE: i32 = 7; // one of the drives the elite carried (reserved)
pub const RK_KEYSTONE: i32 = 8; // a keystone (bosses; today's RW_KEYS screen)
pub const BANK_MAX: i32 = 7; // tokens the bank holds (5 survive a save mid-stop; a stop opens them all)

/// The token `i` (0 first) of a packed token word (V_BANK_Q, RD_PRIZE).
pub fn token(q: i32, i: i32) -> i32 {
    if (0..8).contains(&i) { (q as u32 >> (4 * i as u32) & 15) as i32 } else { 0 }
}

// ---- rooms (waves of foes in one match)
pub const V_WAVE: usize = 680; // the wave in play, 1-based ("Wave 1 of 2")
pub const V_WAVES: usize = 681; // waves in this room (1: a plain fight)
pub const V_ROOM: usize = 682; // the room in play, 1-based ("Room 1 of 3"); 0 in the boss fight
pub const V_ROOMS: usize = 683; // rooms before the boss (3)
pub const V_REC: usize = 704;
pub const REC_WORDS: usize = 48;
pub const REC_MAGIC: i32 = 0x454E_5652;
pub const REC_VERSION: i32 = 9; // 9: envoy items (R_DRIVES = item copies, a byte per id, 10 words a player; R_KEYS[0] = the stock limit); older runs are dropped; 8: R_LOOP = loop | stop << 8 | road << 9 | bank tokens << 12; 7: the bank count << 16; 6: drive words << 4, R_LOOP; 5 and 6 still read
pub const R_SEED: usize = 2;
pub const R_RUNS: usize = 3;
pub const R_FIGHT: usize = 4;
pub const R_POOL: usize = 5;
pub const R_SHARED: usize = 6;
pub const R_COOP: usize = 7;
pub const R_BOSSES: usize = 8;
pub const R_KEYS: usize = 9; // [2 players][3]
pub const R_PAIRS: usize = 15; // main, partner, main 2, partner 2
pub const R_PICKS: usize = 19;
pub const R_DRIVES: usize = 20; // [4 sets][5] in slot order: (id + 1) << 4 | level (version 5: << 2)
pub const R_POOL_DIGEST: usize = 40;
pub const R_COINS: usize = 41; // whole coins (the purse's fraction is not kept)
pub const R_REROLLS: usize = 42;
pub const R_ACTIVE: usize = 43; // player 1's active id | player 2's << 8
pub const R_PIPS: usize = 44; // player 1's whole pips | player 2's << 8
pub const R_EARNED: usize = 45; // player 1's earned | player 2's << 16
pub const R_LOOP: usize = 46; // the loop (low 8 bits; 0 in a version 5 record: loop 1); v7: | bank count << 16 | stop << 24; v8: see REC_VERSION

/// The loop in a record's R_LOOP word (1 at least).
pub fn rec_loop(w: i32) -> i32 {
    (w & 0xFF).max(1)
}

/// A record's drive word as (id + 1, level), by the record's version (`r[1]`).
pub fn rec_drive(w: i32, version: i32) -> (i32, i32) {
    if version <= 5 { (w >> 2, w & 3) } else { (w >> 4 & 0xFF, w & 15) }
}

/// Whether a record's drive word is pinned (version 8: bit 12).
pub fn rec_pin(w: i32, version: i32) -> bool {
    version >= 8 && w >> 12 & 1 != 0
}

// ---- envoy items (envoy-items-design.md, first slice): the stock limit, pedestals, the item effects in the world
pub const V_POOL_LIM: usize = 768; // the stock limit (Isaac's heart containers: start 4, cap 8); V_POOL = stocks now
pub const V_PED_KIND: usize = 769; // pedestals out in this held match: 0 none, PED_ITEM_ROOM, PED_BOSS (P_DROPS hold them)
pub const V_PED_LEFT: usize = 770; // frames before the pedestals fly to their owners
pub const V_CAMP_AT: usize = 771; // the play frame the campfire refilled the stocks (0: none this match)
pub const V_CAMP_GAIN: usize = 772; // stocks the campfire gave back
pub const V_ITEMS_ON: usize = 773; // 1: the item rules (no drives, no Heat meter)
pub const V_STOCK_PRICE: usize = 774; // the shop's stock refill price now
pub const V_FOES: usize = 775; // foes on the field at once this room (1 with V_WAVES > 1: a gauntlet of duels)
pub const V_CROWD: usize = 776; // the crowd knockback foes take this fight (x1000; 1000 none)
pub const PED_ITEM_ROOM: i32 = 1;
pub const PED_BOSS: i32 = 2;
/// Slice 2 rooms (V_PED_KIND): the devil deal behind the red door (each pedestal costs V_PED_COST stock slots; walking
/// in buys it, nothing flies in at the end), the angel room behind the white door (one free item, flies in), the
/// secret room behind the cracked wall (one item or a choice of two, flies in).
pub const PED_DEVIL: i32 = 3;
pub const PED_ANGEL: i32 = 4;
pub const PED_SECRET: i32 = 5;

// ---- slice 2 (envoy-items-design.md 11): co-op stocks, the doors, the secret wall, the in-match fork
pub const V_PSTOCK: usize = 777; // [2] each player's stocks now (Isaac's red hearts; co-op: each their own)
pub const V_PLIM: usize = 779; // [2] each player's stock limit (heart containers: start 4, cap 8)
pub const V_DOOR: usize = 781; // the door after the boss this match: 0 none, DOOR_DEVIL, DOOR_ANGEL
pub const V_DOOR_AT: usize = 782; // the play frame it opened (0: not yet)
pub const V_DEALS: usize = 783; // [2] devil deals each player has taken this run
pub const V_PED_COST: usize = 785; // [2 players][DROPS] stock slots that pedestal costs (0 free) | 0x100 when the
                                   // player cannot pay it (it would take their last slot): drawn locked
pub const V_SECRET: usize = 789; // the cracked wall in this fight: 0 none, 1 standing, 2 broken (the room opens after
                                 // the fight), 3 the secret room is open / done
pub const V_SECRET_X: usize = 790; // the wall's foot, x100 (world)
pub const V_SECRET_Y: usize = 791; // y100
pub const V_SECRET_HITS: usize = 792; // hits it has taken (it breaks at V_SECRET_HP)
pub const V_SECRET_HP: usize = 793;
pub const V_FORK: usize = 794; // the in-match fork after the item room: 0 none, 1 open (Shop or Campfire)
pub const V_FORK_CUR: usize = 795; // P1's cursor: the lane of the room row above the fight (the act map)
pub const V_FORK_LEFT: usize = 796; // always 0: no timer on any choice (the owner)
pub const V_SHOP_SLOT_PRICE: usize = 797; // the shop's stock slot (+1 limit, +1 stock) on sale: its price; 0 not today
pub const V_DOOR_PCT: usize = 798; // the chance the door after this floor's boss opens, percent (the map shows it)
pub const V_SHAKE: usize = 799; // the play frame of the last Big Bang (screen shake; strength in the burst's size)
pub const DOOR_DEVIL: i32 = 1;
pub const DOOR_ANGEL: i32 = 2;
/// The cost word's lock bit (V_PED_COST).
pub const COST_LOCKED: i32 = 0x100;
/// The item projectiles / orbitals / ground effects, every frame: PROJ entries of PJ_W words.
pub const V_PROJ: usize = 800;
pub const PROJ: usize = 48;
pub const PJ_W: usize = 8; // kind (PJ_*; 0 free), owner player, x100, y100, age, life, a, b
pub const PJ_SPARK: i32 = 1;
pub const PJ_WAKE: i32 = 2; // a: half length x100
pub const PJ_STAR: i32 = 3;
pub const PJ_GHOST: i32 = 4; // a: facing (+1 / -1), b: swing done (1)
pub const PJ_MINE: i32 = 5;
pub const PJ_QUAKE: i32 = 6; // a: direction (+1 / -1), b: size x100
pub const PJ_COIN: i32 = 7; // a: target player
pub const PJ_MOON: i32 = 8; // published only (orbit from the play clock); a: index
// slice 2
pub const PJ_BEAM: i32 = 9; // a horizontal beam from (x, y): a = length x100 (signed: the direction), b = BEAM_* look
pub const PJ_FEATHER: i32 = 10; // a falling feather (Demon Wings); velocity in a, b x100 when published
pub const PJ_TENDRIL: i32 = 11; // a lash from (x, y) to the slot in a; b: the frame it strikes
pub const PJ_SHARD: i32 = 12; // an ice shard (Ice Glass); velocity in a, b x100
pub const PJ_WELL: i32 = 13; // a gravity well pulling in: a = radius x100; it bursts at the end of its life
pub const PJ_ARC: i32 = 14; // a lightning arc (Chain Spark) from (x, y) to (a, b) x100
pub const PJ_SIGIL: i32 = 15; // published only: a red sigil orbiting its owner (Crimson Count); a: index, b: count
pub const PJ_GUARD: i32 = 16; // published only: the Guardian Feather orbiting its owner; a: index, b: 1 spent
pub const PJ_HALO: i32 = 17; // published only: the halo over its owner's head; a: size x100
pub const PJ_WINGS: i32 = 18; // published only: black wings on its owner; a: facing, b: 1 the extra jump spent
pub const PJ_THORNS: i32 = 19; // thorns growing out of the owner's back (Crouch Thorns): a = facing, b = size x100
pub const PJ_PRISM: i32 = 20; // a thin laser bolt (Laser Prism); velocity in a, b x100 when published
pub const PJ_ORB: i32 = 21; // published only: an orbital (Moon Drops and the stance orbs): a = index, b = item id;
                            // life = 1 while it is shattered (respawning: draw it as shards / a ghost or not at all)
pub const BEAM_BRIMSTONE: i32 = 0;
pub const BEAM_GOLD: i32 = 1;
pub const BEAM_LASER: i32 = 2;
/// Look flags carried in a projectile's or burst's flag word: Homing Ink's trail, Fire Lace's embers.
pub const PF_INK: i32 = 1 << 20;
pub const PF_LACE: i32 = 1 << 21;
/// Slice 2 modifiers' looks: Ice Glass (ice shots), Chain Spark (crackle), Twin Shot (a twin), Bouncy (squash),
/// Gravity Well (an inward swirl first), Big Bang (twice the size, a shake), Halo (holy white, pierces).
pub const PF_ICE: i32 = 1 << 22;
pub const PF_CHAIN: i32 = 1 << 23;
pub const PF_TWIN: i32 = 1 << 24;
pub const PF_BOUNCY: i32 = 1 << 25;
pub const PF_WELL: i32 = 1 << 26;
pub const PF_BIG: i32 = 1 << 27;
pub const PF_HALO: i32 = 1 << 28;
pub const PF_MASK: i32 = PF_INK | PF_LACE | PF_ICE | PF_CHAIN | PF_TWIN | PF_BOUNCY | PF_WELL | PF_BIG | PF_HALO;
pub const V_INV: usize = 1184; // [2 players][ITEM_IDS = 48] copies held, by item id (1184..1280)
/// One-shot item visuals (a ring, a spike column, a swing, a pop): BURSTS entries of BU_W words.
pub const V_BURST: usize = 1280;
pub const BURSTS: usize = 16;
pub const BU_W: usize = 7; // frame, item id, owner player, x100, y100, size x100, flags (PF_*; low bits: direction + 1)
pub const V_BURST_N: usize = 1392;

// ---- the floor map (envoy-items-design.md 11.4): the loop's rooms as a branching map, made from the run seed and the
// loop, the same for both co-op players and on replay. MAP_N nodes of MAP_W words; a node's links are the nodes it
// leads to (bits). The camp's road screen, the in-match fork and the pause Run page all draw it.
pub const V_MAP_N: usize = 1400; // nodes in use
pub const V_MAP_AT: usize = 1401; // the node the run is at (the fight in play, or the room just done)
pub const V_MAP_LOOP: usize = 1402; // the loop (floor) the map is of
pub const V_MAP_SECRET: usize = 1403; // the node carrying the cracked wall (-1 none this floor)
pub const V_MAP0: usize = 1404; // node i: V_MAP0 + i * MAP_W + MP_*
pub const MAP_N: usize = 24;
pub const MAP_W: usize = 4; // 1404..1500
pub const MP_KIND: usize = 0; // MN_*
pub const MP_POS: usize = 1; // lane | row << 8 (row 0 the bottom; the boss row 6, the door row 7; the secret room: lane 3, its wall's row, the wall's lane << 16)
pub const MP_STATE: usize = 2; // MS_* bits
pub const MP_LINKS: usize = 3; // bit j: this node leads to node j
pub const MN_START: i32 = 1;
pub const MN_DUEL: i32 = 2; // a gauntlet of duels
pub const MN_CROWD: i32 = 3; // the 1v3 crowd
pub const MN_ELITE: i32 = 4; // the elite gauntlet
pub const MN_BOSS: i32 = 5;
pub const MN_ITEM: i32 = 6; // gold star
pub const MN_SHOP: i32 = 7; // coin
pub const MN_CAMPFIRE: i32 = 8; // orange flame
pub const MN_DOOR: i32 = 9; // the door after the boss, not yet rolled ("?"; V_DOOR_PCT the chance)
pub const MN_DEVIL: i32 = 10; // red door
pub const MN_ANGEL: i32 = 11; // white door
pub const MN_SECRET: i32 = 12; // the secret room (shown once its wall is broken)
pub const MS_DONE: i32 = 1;
pub const MS_HERE: i32 = 2;
pub const MS_NEXT: i32 = 4; // can be the next room (a fork shows two)
pub const MS_CHOSEN: i32 = 8; // the branch taken at a fork
pub const MS_SKIPPED: i32 = 16; // the branch not taken
pub const MS_WALL: i32 = 32; // a cracked wall is in this fight (the secret room's entrance)
pub const MS_FOUND: i32 = 64; // its wall was broken
pub const MS_UNKNOWN: i32 = 128; // not revealed yet
pub const MS_STEEP: i32 = 256; // the Steep road's twist is on this fight
pub const MS_SHIFT: i32 = 512; // the Shifting road's twist is on this fight
pub const MS_CURSOR: i32 = 1024; // P1's cursor is on it (the next fight on the road screen, or the room in the held match)
pub const MS_WISH: i32 = 2048; // P2's wish flag (co-op; no effect)
pub const MS_PLAN: i32 = 4096; // on the planned path ahead (the cursor's lane carried up the map)
pub const V_MAP_END: usize = V_MAP0 + MAP_N * MAP_W; // 1500
pub const V_MAP_ROWS: usize = 1500; // rows (8: six of three lanes, the boss, the door)
pub const V_MAP_LANES: usize = 1501; // lanes a row (3)
pub const V_PED_NEAR: usize = 1503; // [2] per player: pedestal k + 1 in pickup range (0 none): the UI's "A" prompt
pub const V_FLOURISH: usize = 1505; // [2] per player: frames left of the build flourish (fight intro, victory), 0 none
pub const V_AURA: usize = 1507; // [2] per player: the rarest item held (id; 0 none): the UI's subtle aura tint
pub const V_PROC_AGE: usize = 1509; // [2] per player: frames since that player's last item proc (capped 255)
pub const FLOURISH_LEN: i32 = 60;
pub const V_MAP_PICK: usize = 1502; // what P1 picks now: 0 nothing, 1 the next fight (road screen), 2 the room above the fight (held match)

/// An offer that is not a drive: one more shared stock.
pub const OFFER_STOCK: i32 = 1000;
pub const SLOTS: usize = 5;
pub const KEYSTONES: usize = 3;
/// The active drive ids (P_ACT_ID) and set ids (P_SET).
pub const ACTIVE_VOLLEY: i32 = 1;
pub const SET_INFERNO: i32 = 1;

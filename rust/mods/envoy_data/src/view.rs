// The run view envoy_sim publishes (`gd_envoy_publish`, 768 words) and envoy_ui reads (`gd_ui_envoy`): one layout,
// shared here so the two mods cannot disagree. Positions are f32 bits; ids are pool index + 1 (0: none).
// V3 (Heat): docs/rust-port/notes/envoy-v3-view.md is the prose contract for every word below.

pub const VIEW_WORDS: usize = 768;
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
pub const V_STAGE_IN: usize = 14;
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
pub const P_GAIN: usize = 71; // 6: frame, id + 1, fighter, level, source, count
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
// the shop (V_RW == RW_SHOP): one list per player
pub const P_SHOP_OFFERS: usize = 112; // 3: pool index of the three drives on sale (-1 sold)
pub const P_SHOP_PRICE: usize = 115; // 3: their prices (coins)
pub const P_SHOP_CURSOR: usize = 118; // the item cursor: SHOP_* row
pub const P_SHOP_STEP: usize = 119; // 0 the list, 1 the slot (level / replace), 2 reorder: picking, 3 reorder: moving
pub const P_SHOP_FIGHTER: usize = 120; // the fighter the shop acts on (0 main, 1 partner)
pub const P_SHOP_DONE: usize = 121; // 1: left the shop
pub const P_SHOP_MSG: usize = 122; // the last result: SM_* (for a one-line message)
pub const P_SHOP_LEVEL_PRICE: usize = 123; // [SLOTS] the price to level the shop fighter's slot k (0: max level / empty)

pub const DROPS: usize = 2;

// shop rows (P_SHOP_CURSOR)
pub const SHOP_DRIVE0: i32 = 0; // 0-2 the three drives
pub const SHOP_LEVEL: i32 = 3;
pub const SHOP_STOCK: i32 = 4;
pub const SHOP_REROLL: i32 = 5;
pub const SHOP_REORDER: i32 = 6;
pub const SHOP_LEAVE: i32 = 7;
pub const SHOP_ROWS: i32 = 8;
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
pub const EV_COINS: i32 = 14; // who = player; amount = coins from a flat award (a fight cleared, interest)
pub const MARK_LCANCEL: i32 = 1;
pub const MARK_WAVEDASH: i32 = 2;
pub const MARK_TECH: i32 = 3;
pub const MARK_OFFSTAGE: i32 = 4;

// ---- the run record (melee_modes::tu::envoy REC_*): saved by the host, handed back at arm
pub const V_REC: usize = 704;
pub const REC_WORDS: usize = 48;
pub const REC_MAGIC: i32 = 0x454E_5652;
pub const REC_VERSION: i32 = 6; // 6: drive words (id + 1) << 4 | level, R_LOOP; 5 still read
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
pub const R_LOOP: usize = 46; // the loop (0 in a version 5 record: loop 1)

/// A record's drive word as (id + 1, level), by the record's version (`r[1]`).
pub fn rec_drive(w: i32, version: i32) -> (i32, i32) {
    if version <= 5 { (w >> 2, w & 3) } else { (w >> 4, w & 15) }
}

/// An offer that is not a drive: one more shared stock.
pub const OFFER_STOCK: i32 = 1000;
pub const SLOTS: usize = 5;
pub const KEYSTONES: usize = 3;
/// The active drive ids (P_ACT_ID) and set ids (P_SET).
pub const ACTIVE_VOLLEY: i32 = 1;
pub const SET_INFERNO: i32 = 1;

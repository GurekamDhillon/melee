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
pub const P_GAIN_FROM: usize = 148; // 3: the drop behind the last drop gain (worn, levelled, banked, flown in): x, y (x100, world), owner slot

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
pub const EV_HANDOFF: i32 = 16; // who = player; amount = the fired drive's id + 1 | 0x100 when Relay fired the row
pub const EV_COINS: i32 = 14; // who = player; amount = coins from a flat award (a fight cleared, interest)
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

pub const ROAD_OPEN: i32 = 1; // the session as it is: banks three drive picks
pub const ROAD_STEEP: i32 = 2; // the same fights, every foe one more drive: banks two picks and a level-up
pub const ROAD_SWARM: i32 = 3; // reserved (waves, coins)
pub const DANGER_CALM: i32 = 0;
pub const DANGER_ROUGH: i32 = 1;
pub const DANGER_STEEP: i32 = 2;
/// Twists: the content pass's elite modifiers and M3's dares share this list. Only TW_FOE_DRIVE is live.
pub const TW_FOE_DRIVE: i32 = 1 << 0; // every foe carries one more drive (to the 2 a foe can hold)
pub const TW_MORE_FOES: i32 = 1 << 1; // reserved: one more foe in every fight
pub const TW_BIG_FOES: i32 = 1 << 2; // reserved: foes are larger
pub const TW_FAST_SHIFT: i32 = 1 << 3; // reserved: the stage shifts twice as often

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

pub const V_REC: usize = 704;
pub const REC_WORDS: usize = 48;
pub const REC_MAGIC: i32 = 0x454E_5652;
pub const REC_VERSION: i32 = 8; // 8: R_LOOP = loop | stop << 8 | road << 9 | bank tokens << 12; 7: the bank count << 16; 6: drive words << 4, R_LOOP; 5 and 6 still read
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

/// An offer that is not a drive: one more shared stock.
pub const OFFER_STOCK: i32 = 1000;
pub const SLOTS: usize = 5;
pub const KEYSTONES: usize = 3;
/// The active drive ids (P_ACT_ID) and set ids (P_SET).
pub const ACTIVE_VOLLEY: i32 = 1;
pub const SET_INFERNO: i32 = 1;

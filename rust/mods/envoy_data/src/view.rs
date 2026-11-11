// The run view envoy_sim publishes (`gd_envoy_publish`, 512 words) and envoy_ui reads (`gd_ui_envoy`): one layout,
// shared here so the two mods cannot disagree. Positions are f32 bits; ids are pool index + 1 (0: none).

pub const VIEW_WORDS: usize = 512;
pub const VIEW_MAGIC: i32 = 0x454E_5632; // "ENV2"

// ---- the run
pub const V_MAGIC: usize = 0;
pub const V_STATE: usize = 1; // 0 none, 1 live, 2 clear, 3 over
pub const V_FIGHT: usize = 2;
pub const V_FIGHTS: usize = 3;
pub const V_POOL: usize = 4;
pub const V_LIVE: usize = 5;
pub const V_HUMANS: usize = 6; // 1 solo, 2 co-op
pub const V_SHARED: usize = 7;
pub const V_RW: usize = 8; // the reward screen: 0 none, 1 drives, 2 keystones
pub const V_TIMER: usize = 9;
pub const V_RW_TIME: usize = 10;
pub const V_FROZEN: usize = 11;
pub const V_PLAY: usize = 12;
pub const V_DIGEST: usize = 13;
pub const V_STAGE_IN: usize = 14;
pub const V_FIRES: usize = 15;
pub const V_FIGHT_KIND: usize = 16; // FK_*
pub const V_OPEN: usize = 17;
pub const V_KEYS_EARNED: usize = 18;
pub const V_SEQ: usize = 19;
pub const V_ARMED: usize = 20;
pub const V_NCPU: usize = 21;
pub const V_SINCE_PICK: usize = 22;
pub const V_BOSSES: usize = 23;
pub const V_OUTCOME: usize = 24;

pub const FK_DUEL: i32 = 0;
pub const FK_PACK: i32 = 1;
pub const FK_ELITE: i32 = 2;
pub const FK_BOSS: i32 = 3;

// ---- a player (human slot p): block at V_P0 + p * V_PW
pub const V_P0: usize = 32;
pub const V_PW: usize = 96;
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
pub const P_DRIVES: usize = 16; // [2 fighters][4]
pub const P_LEVELS: usize = 24;
pub const P_CHARGE: usize = 32; // permille
pub const P_LIVE_SETS: usize = 40; // bit f: set f orbits and fires now (in play, or benched with Twin Soul)
pub const P_RW_DONE: usize = 48;
pub const P_CURSOR: usize = 49;
pub const P_STEP: usize = 50; // 0 the card, 1 the fighter
pub const P_FCURSOR: usize = 51;
pub const P_OFFERS: usize = 52; // 3: pool index or OFFER_STOCK
pub const P_CAN_TAKE: usize = 55; // bit i * 2 + f
pub const P_DROPS: usize = 56; // DROPS x (state, id + 1, x100, y100)
pub const P_HASTE: usize = 64; // [2] frames left
pub const P_SHIELD: usize = 66; // [2] orb armed
pub const P_BANKED_JUMP: usize = 68;
pub const P_GAIN: usize = 70; // frame, id + 1, fighter, level, source, count

pub const DROPS: usize = 2;

// ---- every match slot (0-3): statuses, foe drives
pub const V_S0: usize = 224;
pub const V_SW: usize = 16;
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

// ---- the fire log: LOG entries of (frame, id + 1, owner, target slot, x100, y100, level, flags)
pub const V_LOG: usize = 288;
pub const LOG: usize = 12;
pub const LOG_W: usize = 8;
pub const V_LOG_N: usize = V_LOG + LOG * LOG_W;
/// A fire's owner: a player's set (player * 2 + fighter, 0-3) or a foe's slot + OWNER_FOE.
pub const OWNER_FOE: i32 = 4;
pub const FL_ECHO: i32 = 1;
pub const FL_VOLLEY: i32 = 2;
pub const FL_LINK: i32 = 4;
pub const FL_LAUNCH: i32 = 8;
pub const FL_PASSIVE: i32 = 16;

// ---- the run record (melee_modes::tu::envoy REC_*): saved by the host, handed back at arm
pub const V_REC: usize = 448;
pub const REC_WORDS: usize = 48;
pub const REC_MAGIC: i32 = 0x454E_5652;
pub const REC_VERSION: i32 = 4;
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
pub const R_DRIVES: usize = 20; // [4 sets][4]: (id + 1) << 2 | level
pub const R_POOL_DIGEST: usize = 36;

/// An offer that is not a drive: one more shared stock.
pub const OFFER_STOCK: i32 = 1000;
pub const SLOTS: usize = 4;
pub const KEYSTONES: usize = 3;

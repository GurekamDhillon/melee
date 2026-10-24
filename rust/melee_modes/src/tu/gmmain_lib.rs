//! src/melee/gm/gmmain_lib.c: the game-mode library. Save-data and VS-state accessors over the
//! global `gmMainLib_804D3EE0` record, the persistent card-data reset paths, the unlock and
//! Mex-gated PC hooks, and the boot-time setup.
//!
//! Verification: NONE yet. Every function here reads or writes the global record, so the state
//! oracle (C harness over a seeded `gmm_x0`, Rust over the same bytes) is the check; that harness
//! is not written. Pure helpers reused from `crate::gm` (`bitset_mask`, `bitset64_mask`,
//! `selkind_mask`) carry their own vector oracle.
//!
//! Globals: `gmMainLib_804D3EE0` is the record pointer. The C initialises it to the static
//! `gmMainLib_8045A6C0`; the port does the same in `gmMainLib_init_statics`. Data the C defines
//! in other TUs (`HSD_PadLibData`, `GXNtsc480*`, `db_*`) is reached through `ext` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::addr_of_mut;

use melee_types as mt;
use mt::*;

pub use crate::gm::{bitset64_mask, bitset_mask, selkind_mask};
use crate::gm_ptr::GM_NAMETAG_COUNT;

/// `gmm_x0` record pointer. C: `struct gmm_x0* gmMainLib_804D3EE0 = &gmMainLib_8045A6C0;`
pub static mut gmMainLib_804D3EE0: *mut gmm_x0 = core::ptr::null_mut();
/// C: `struct gmMainLib_8046B0F0_t gmMainLib_8046B0F0;`
pub static mut gmMainLib_8046B0F0: gmMainLib_8046B0F0_t = unsafe { core::mem::zeroed() };
/// C: `struct gmm_x0 gmMainLib_8045A6C0;`
pub static mut gmMainLib_8045A6C0: gmm_x0 = unsafe { core::mem::zeroed() };
/// C: `GameRules gmMainLib_DefaultGameRules = {...}` (set by `gmMainLib_init_statics`).
pub static mut gmMainLib_DefaultGameRules: GameRules = unsafe { core::mem::zeroed() };
/// C: `struct GamePrefs gmMainLib_DefaultGamePrefs = {...}` (set by `gmMainLib_init_statics`).
pub static mut gmMainLib_DefaultGamePrefs: GamePrefs = unsafe { core::mem::zeroed() };
/// C: `GXRenderModeObj gmMainLib_803D4A80`, the deflicker render mode. Opaque here; its contents
/// are set by the GX shim (`ext::gmMainLib_803D4A80_init`).
pub static mut gmMainLib_803D4A80: [u8; 0] = [];
/// C: `static s8 gmMainLib_804D3EE4[] = { 0 };` the name terminator.
const gmMainLib_804D3EE4: [i8; 1] = [0];

/// Static initial values from gmmain_lib.c's `= {...}` initialisers. Call once at boot before
/// any accessor (the C does this with static data).
pub unsafe fn gmMainLib_init_statics() {
    gmMainLib_804D3EE0 = addr_of_mut!(gmMainLib_8045A6C0);
    let r = &mut *addr_of_mut!(gmMainLib_DefaultGameRules);
    *r = core::mem::zeroed();
    r.force_main_menu = 0;
    r.bgm = 0x34;
    r.mode = 0;
    r.time_limit = 2;
    r.stock_count = 3;
    r.handicap = 0;
    r.damage_ratio = 10;
    r.stage_sel = 0;
    r.stock_time_limit = 0;
    r.friendly_fire = 0;
    r.pause = 1;
    r.score_display = 0;
    r.unk_xc = 0;
    r.xD = 0;
    r.xE = 8;
    r.xF = 8;
    r.unk_x10 = 0;
    r.x11 = 0;
    r.x12 = 8;
    r.x13 = 0;
    r.unk_14 = -1;

    let p = &mut *addr_of_mut!(gmMainLib_DefaultGamePrefs);
    *p = core::mem::zeroed();
    p.item_freq = 2;
    p.item_mask = u64::MAX;
    p.rumble_enabled = [1, 1, 1, 1];
    p.sound_balance = 0;
    p.deflicker = 1;
    p.saved_language = LANG_JP_VALUE;
    p.stage_mask = u32::MAX;
}

/// Language 0 is LANG_JP; the constant is written out so the port does not depend on the enum.
const LANG_JP_VALUE: u8 = 0;

// ---- helpers mirroring the C static inlines -------------------------------------------------

#[inline]
unsafe fn card() -> *mut GmCardData {
    addr_of_mut!((*gmMainLib_804D3EE0).thing)
}

#[inline]
unsafe fn save() -> *mut GmSaveData {
    addr_of_mut!((*gmMainLib_804D3EE0).thing.save_data)
}

#[inline]
unsafe fn fighter_base() -> *mut FighterData {
    addr_of_mut!((*save()).x1F2C) as *mut FighterData
}

#[inline]
unsafe fn persistent_fighter(selkind: u8) -> *mut FighterData {
    fighter_base().add(selkind as usize)
}

#[inline]
unsafe fn nametag_slot(banks: *mut NameTagDataBank, idx: i32) -> *mut NameTagData {
    let bank = banks.add((idx / GM_NAMETAG_BANK_SIZE) as usize);
    addr_of_mut!((*bank).inner[(idx % GM_NAMETAG_BANK_SIZE) as usize])
}

#[inline]
fn bitset_set(words: &mut [u32], bit: u32) {
    words[(bit / 32) as usize] |= bitset_mask(bit);
}

#[inline]
fn bitset_clear(words: &mut [u32], bit: u32) {
    words[(bit / 32) as usize] &= !bitset_mask(bit);
}

#[inline]
fn bitset_test(words: &[u32], bit: u32) -> u32 {
    words[(bit / 32) as usize] & bitset_mask(bit)
}

/// C `bitset_test_word`: loads the word before forming the mask (same result).
#[inline]
fn bitset_test_word(words: &[u32], bit: u32) -> u32 {
    let word = words[(bit / 32) as usize];
    word & bitset_mask(bit)
}

#[inline]
fn selkind_bit(word: i32, kind: u8) -> i32 {
    word & selkind_mask(kind)
}

#[inline]
unsafe fn memzero<T>(p: *mut T, bytes: usize) {
    core::ptr::write_bytes(p as *mut u8, 0, bytes);
}

// ---- accessors -------------------------------------------------------------------------------

pub unsafe fn gmMainLib_GetGameRules() -> *mut GameRules {
    addr_of_mut!((*gmMainLib_804D3EE0).x1850)
}

pub unsafe fn gmMainLib_GetCardData() -> *mut GmCardData {
    card()
}

pub unsafe fn gmMainLib_GetNameTagDataBanks() -> *mut NameTagDataBank {
    addr_of_mut!((*card()).nametag_banks) as *mut NameTagDataBank
}

pub unsafe fn gmMainLib_GetGamePrefs() -> *mut GamePrefs {
    addr_of_mut!((*save()).x1CB0)
}

pub unsafe fn GetPersistentFighterData(selkind: SelectableCharacterKind) -> *mut FighterData {
    fighter_base().add(selkind as usize)
}

pub unsafe fn gmMainLib_GetTrophyFlags() -> *mut u16 {
    addr_of_mut!((*save()).trophy_flags) as *mut u16
}

pub unsafe fn gmMainLib_GetTrophyCategoryFlags() -> *mut u16 {
    addr_of_mut!((*save()).trophy_category_flags)
}

pub unsafe fn gmMainLib_GetTrophyCount() -> *mut i16 {
    addr_of_mut!((*save()).trophy_count)
}

pub unsafe fn GetPersistentNameData(arg0: i32) -> *mut NameTagData {
    nametag_slot(gmMainLib_GetNameTagDataBanks(), arg0)
}

pub unsafe fn gmMainLib_8015CCE4() -> *mut gmm_x0_44_t {
    addr_of_mut!((*gmMainLib_804D3EE0).unk_44)
}

pub unsafe fn gmMainLib_8015CCF0() -> *mut u32 {
    addr_of_mut!((*save()).x1A48)
}

pub unsafe fn gmMainLib_8015CCFC() -> *mut i32 {
    addr_of_mut!((*save()).x1A4C)
}

pub unsafe fn gmMainLib_GetVsPlayContestants() -> *mut i32 {
    addr_of_mut!((*save()).x1A30)
}

pub unsafe fn gmMainLib_GetVsPlayTime() -> *mut i32 {
    addr_of_mut!((*save()).x1A34)
}

pub unsafe fn gmMainLib_GetCombinedVSPlayTime() -> *mut i32 {
    addr_of_mut!((*save()).x1A38)
}

pub unsafe fn gmMainLib_GetTimeMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).time_matches)
}

pub unsafe fn gmMainLib_GetStockMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).stock_matches)
}

pub unsafe fn gmMainLib_GetCoinMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).coin_matches)
}

pub unsafe fn gmMainLib_GetBonusMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).bonus_matches)
}

pub unsafe fn gmMainLib_GetStaminaMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).stamina_matches)
}

pub unsafe fn gmMainLib_GetMatchResetCounter() -> *mut u32 {
    addr_of_mut!((*save()).match_resets)
}

pub unsafe fn gmMainLib_GetSingleplayerTime() -> *mut i32 {
    addr_of_mut!((*save()).x1A3C)
}

pub unsafe fn gmMainLib_8015CD80() -> *mut i32 {
    addr_of_mut!((*save()).x1A44)
}

pub unsafe fn gmMainLib_GetPowerCount() -> *mut u32 {
    addr_of_mut!((*save()).x1A50) as *mut u32
}

pub unsafe fn gm_GetPowerTime() -> *mut i32 {
    addr_of_mut!((*save()).x1A54)
}

pub unsafe fn gmMainLib_GetTotalDamage() -> *mut i32 {
    addr_of_mut!((*save()).x1A58)
}

pub unsafe fn gmMainLib_GetKOTotal() -> *mut i32 {
    addr_of_mut!((*save()).x1A5C)
}

pub unsafe fn gmMainLib_GetSelfDestructTotal() -> *mut i32 {
    addr_of_mut!((*save()).x1A60)
}

pub unsafe fn gmMainLib_8015CDC8() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_51C)
}

pub unsafe fn gmMainLib_8015CDD4() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_522)
}

pub unsafe fn gmMainLib_8015CDE0() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_528)
}

pub unsafe fn gmMainLib_8015CDEC() {
    for i in 0..Gm_Player_NumMax as i32 {
        let ptr = gmMainLib_8015CE44(i, GM_NAMETAG_COUNT as i32);
        if !ptr.is_null() {
            *ptr = 5;
        }
    }
}

const PAD_MAX_CONTROLLERS: i32 = 4;

pub unsafe fn gmMainLib_8015CE44(arg0: i32, arg1: i32) -> *mut i8 {
    if arg1 == GM_NAMETAG_COUNT as i32 {
        if arg0 < PAD_MAX_CONTROLLERS {
            return addr_of_mut!((*gmMainLib_804D3EE0).modes.nametags[arg0 as usize]);
        }
        core::ptr::null_mut()
    } else {
        addr_of_mut!((*nametag_slot(gmMainLib_GetNameTagDataBanks(), arg1)).x1A2)
    }
}

pub unsafe fn gmMainLib_8015CEB4(arg0: i32) {
    let s = &mut (*save()).x1A68;
    *s |= bitset64_mask(arg0 as u32);
}

pub unsafe fn gmMainLib_8015CEFC(arg0: i32) -> bool {
    (*gmMainLib_804D3EE0).thing.save_data.x1A68 & bitset64_mask(arg0 as u32) != 0
}

pub unsafe fn gmMainLib_8015CF5C(arg0: i32) -> i32 {
    (*save()).x1A70[arg0 as usize]
}

pub unsafe fn gmMainLib_8015CF70(arg0: i32, arg1: i32) {
    (*save()).x1A70[arg0 as usize] = arg1;
}

pub unsafe fn gmMainLib_8015CF84() {
    (*save()).x1B3C = 1;
}

pub unsafe fn gmMainLib_8015CF94() -> bool {
    (*save()).x1B3C != 0
}

pub unsafe fn gmMainLib_8015CFB4(arg0: u8) -> *mut i8 {
    addr_of_mut!((*persistent_fighter(arg0)).x78)
}

pub unsafe fn gmMainLib_8015CFCC(arg0: u8) -> i32 {
    selkind_mask(arg0) & (*gmMainLib_8015ED98()).xC
}

pub unsafe fn gmMainLib_8015D00C(arg0: u8) {
    (*persistent_fighter_base_x7a(arg0)).anon_0.set_b0(1);
    (*gmMainLib_8015ED98()).xC |= selkind_mask(arg0);
}

/// `GetPersistentFighterDataBase(card)[arg0].x7A`, the UnkFlagStruct at `FighterData.x7A`.
#[inline]
unsafe fn persistent_fighter_base_x7a(arg0: u8) -> *mut UnkFlagStruct {
    addr_of_mut!((*persistent_fighter(arg0)).x7A)
}

pub unsafe fn gmMainLib_8015D06C(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x84)
}

pub unsafe fn gmMainLib_8015D084(selkind: u8) -> *mut i32 {
    addr_of_mut!((*gmMainLib_8015EDBC()).xB0[selkind as usize])
}

pub unsafe fn gmMainLib_8015D0C0(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x88)
}

pub unsafe fn gmMainLib_8015D0D8(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b4() != 0
}

pub unsafe fn gmMainLib_8015D0F4(arg0: u8) -> i32 {
    selkind_bit((*gmMainLib_8015ED98()).x10, arg0)
}

pub unsafe fn gmMainLib_8015D134(arg0: u8) {
    (*persistent_fighter(arg0)).x7C.set_b4(1);
    (*gmMainLib_8015ED98()).x10 |= selkind_mask(arg0);
}

pub unsafe fn gmMainLib_8015D194(arg0: u8) -> *mut u8 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x80)
}

pub unsafe fn gmMainLib_8015D1AC(arg0: u8) -> i32 {
    (*persistent_fighter(arg0)).x7C.b789() as i32
}

pub unsafe fn gmMainLib_8015D1C8(arg0: u8, stocks: i32) {
    (*persistent_fighter(arg0)).x7C.set_b789(stocks as u16);
}

pub unsafe fn gmMainLib_8015D1E8(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x8C)
}

pub unsafe fn gmMainLib_8015D200(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b5() != 0
}

pub unsafe fn gmMainLib_8015D21C(arg0: u8) -> i32 {
    selkind_bit((*gmMainLib_8015ED98()).x14, arg0)
}

pub unsafe fn gmMainLib_8015D25C(arg0: u8) {
    (*persistent_fighter(arg0)).x7C.set_b5(1);
    (*gmMainLib_8015ED98()).x14 |= selkind_mask(arg0);
}

pub unsafe fn gmMainLib_8015D2BC(arg0: u8) -> *mut u8 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x81)
}

pub unsafe fn gmMainLib_8015D2D4(arg0: u8) -> i32 {
    (*persistent_fighter(arg0)).x7C.b10_to_12() as i32
}

pub unsafe fn gmMainLib_8015D2F0(arg0: u8, stocks: i32) {
    (*persistent_fighter(arg0)).x7C.set_b10_to_12(stocks as u16);
}

pub unsafe fn gmMainLib_8015D310(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x90)
}

pub unsafe fn gmMainLib_8015D328(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b6() != 0
}

pub unsafe fn gmMainLib_8015D344(arg0: u8) -> i32 {
    selkind_bit((*gmMainLib_8015ED98()).x18, arg0)
}

pub unsafe fn gmMainLib_8015D384(arg0: u8) {
    (*persistent_fighter(arg0)).x7C.set_b6(1);
    (*gmMainLib_8015ED98()).x18 |= selkind_mask(arg0);
}

pub unsafe fn gmMainLib_8015D3E4(arg0: u8) -> *mut u8 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x82)
}

pub unsafe fn gmMainLib_8015D3FC(arg0: u8) -> i32 {
    (*persistent_fighter(arg0)).x7C.b13_to_15() as i32
}

pub unsafe fn gmMainLib_8015D418(arg0: u8, stocks: i32) {
    (*persistent_fighter(arg0)).x7C.set_b13_to_15(stocks as u16);
}

pub unsafe fn gmMainLib_8015D438(arg0: u8) -> *mut u32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x94)
}

pub unsafe fn gmMainLib_8015D450(arg0: u8) -> *mut i32 {
    addr_of_mut!((*gmMainLib_8015EDBC()).x4C[arg0 as usize])
}

pub unsafe fn gmMainLib_8015D48C(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b0() != 0
}

pub unsafe fn gmMainLib_8015D4A8(arg0: u8) -> i32 {
    selkind_bit((*gmMainLib_8015ED98()).x1C, arg0)
}

pub unsafe fn gmMainLib_8015D4E8(arg0: u8, arg1: i32) {
    (*persistent_fighter(arg0)).x7C.set_b0(arg1 as u16);
}

pub unsafe fn gmMainLib_8015D508() -> bool {
    for i in 0..SELKIND_COUNT as usize {
        if (*fighter_base().add(i)).x7C.b0() == 0 {
            return false;
        }
    }
    true
}

pub unsafe fn gmMainLib_8015D5DC() -> bool {
    for i in 0..0x19 {
        if selkind_bit((*gmMainLib_8015EDB0()).x4, i as u8) == 0 {
            return false;
        }
    }
    true
}

pub unsafe fn gmMainLib_8015D640() -> bool {
    for i in 0..0x19 {
        if selkind_bit((*gmMainLib_8015EDBC()).x8, i as u8) == 0 {
            return false;
        }
    }
    true
}

pub unsafe fn gmMainLib_8015D6A4(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x98)
}

pub unsafe fn gmMainLib_8015D6BC(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b1() != 0
}

pub unsafe fn gmMainLib_8015D6D8(arg0: u8, arg1: i32) {
    (*persistent_fighter(arg0)).x7C.set_b1(arg1 as u16);
}

pub unsafe fn gmMainLib_8015D6F8(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x9C)
}

pub unsafe fn gmMainLib_8015D710(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b2() != 0
}

pub unsafe fn gmMainLib_8015D72C(arg0: u8, arg1: i32) {
    (*persistent_fighter(arg0)).x7C.set_b2(arg1 as u16);
}

pub unsafe fn gmMainLib_8015D74C(arg0: u8) -> *mut u16 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.xA0)
}

pub unsafe fn gmMainLib_8015D764(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b3() != 0
}

pub unsafe fn gmMainLib_8015D780(arg0: u8) {
    (*persistent_fighter(arg0)).x7C.set_b3(1);
}

pub unsafe fn gmMainLib_8015D7A4(arg0: u8) -> *mut u16 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.xA2)
}

pub unsafe fn gmMainLib_8015D7BC(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.xA4)
}

pub unsafe fn gmMainLib_8015D7D4(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.xA8)
}

pub unsafe fn gmMainLib_8015D7EC(arg0: u8) -> *mut u16 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x7E)
}

pub unsafe fn gmMainLib_8015D804(arg0: i32) -> *mut u32 {
    addr_of_mut!((*save()).x1B80[arg0 as usize]) as *mut u32
}

pub unsafe fn gmMainLib_8015D818(arg0: u32) -> i32 {
    if gmMainLib_8015D94C(arg0) == 0 {
        *gmMainLib_8015D804(arg0 as i32) = ext::lbTime_GetTimeInSeconds();
        gmMainLib_8015D888(arg0);
        gmMainLib_8015D8FC(arg0);
        return 1;
    }
    0
}

pub unsafe fn gmMainLib_8015D888(arg0: u32) {
    bitset_set(&mut (*save()).x1B40, arg0);
}

pub unsafe fn gmMainLib_8015D8B0(arg0: u32) {
    bitset_clear(&mut (*save()).x1B40, arg0);
}

pub unsafe fn gmMainLib_8015D8D8(arg0: u32) -> i32 {
    bitset_test(&(*save()).x1B40, arg0) as i32
}

pub unsafe fn gmMainLib_8015D8FC(arg0: u32) {
    bitset_set(&mut (*save()).x1B4C, arg0);
}

pub unsafe fn gmMainLib_8015D924(arg0: u32) {
    bitset_clear(&mut (*save()).x1B4C, arg0);
}

pub unsafe fn gmMainLib_8015D94C(arg0: u32) -> i32 {
    if ext::Mex_Enabled("unlock_special_messages") != 0 {
        return 1;
    }
    bitset_test_word(&(*save()).x1B4C, arg0) as i32
}

pub unsafe fn gmMainLib_8015D970(idx: isize) -> *mut u32 {
    addr_of_mut!((*gmMainLib_8015CCE4()).times[idx as usize]) as *mut u32
}

pub unsafe fn gmMainLib_8015D984(arg0: u32) -> bool {
    if gmMainLib_8015DA90(arg0) == 0 {
        *gmMainLib_8015D970(arg0 as isize) = ext::lbTime_GetTimeInSeconds();
        gmMainLib_8015D9F4(arg0);
        gmMainLib_8015DA40(arg0);
        return true;
    }
    if ext::Mex_Enabled("unlock_special_messages_2") != 0 {
        return true;
    }
    false
}

pub unsafe fn gmMainLib_8015D9F4(arg0: u32) {
    bitset_set(&mut (*gmMainLib_8015CCE4()).flags, arg0);
}

pub unsafe fn gmMainLib_8015DA1C(arg0: u32) -> i32 {
    bitset_test(&(*gmMainLib_8015CCE4()).flags, arg0) as i32
}

pub unsafe fn gmMainLib_8015DA40(arg0: u32) {
    bitset_set(&mut (*save()).x1B58, arg0);
}

pub unsafe fn gmMainLib_8015DA68(arg0: u32) {
    bitset_clear(&mut (*save()).x1B58, arg0);
}

pub unsafe fn gmMainLib_8015DA90(arg0: u32) -> i32 {
    bitset_test_word(&(*save()).x1B58, arg0) as i32
}

pub unsafe fn gmMainLib_8015DAB4(arg0: u32) {
    bitset_set(&mut (*save()).x1C88, arg0);
}

pub unsafe fn gmMainLib_8015DADC(arg0: u32) -> bool {
    bitset_test(&(*save()).x1C88, arg0) != 0
}

pub unsafe fn gmMainLib_8015DB00() -> u8 {
    (*gmMainLib_804D3EE0).unk_1
}

pub unsafe fn gmMainLib_8015DB0C(arg0: u8) {
    (*gmMainLib_804D3EE0).unk_1 = arg0;
}

pub unsafe fn gmMainLib_8015DB18() {
    (*gmMainLib_804D3EE0).unk_1 = (*gmMainLib_804D3EE0).unk_1.wrapping_add(1);
}

pub unsafe fn gmMainLib_8015DB2C(arg0: u8) {
    let x = (*gmMainLib_804D3EE0).x39[arg0 as usize];
    (*gmMainLib_804D3EE0).x39[arg0 as usize] = crate::lb::lbTime_8000AF74(x as u32, 1) as u8;
}

pub unsafe fn gmMainLib_8015DB6C(arg0: u8) -> u8 {
    (*gmMainLib_804D3EE0).x39[arg0 as usize]
}

pub unsafe fn gmMainLib_8015DB80() {
    for i in 0..11 {
        (*gmMainLib_804D3EE0).x39[i] = 0;
    }
}

/// C `gmMainLib_AdjustNameTags`: a VS mode's six player name-tag slots.
unsafe fn gmMainLib_AdjustNameTags(vmd: *mut VsModeData, tag: u8) {
    for i in 0..6 {
        let ptr = addr_of_mut!((*vmd).start.players[i].nametag);
        if (*vmd).start.players[i].nametag == tag {
            *ptr = GM_NAMETAG_COUNT;
        } else if *ptr > tag && *ptr != GM_NAMETAG_COUNT {
            *ptr -= 1;
        }
    }
}

/// C `gmMainLib_AdjustNameTag`.
unsafe fn gmMainLib_AdjustNameTag(tag_ptr: *mut u8, tag: u8) {
    if *tag_ptr == tag {
        *tag_ptr = GM_NAMETAG_COUNT;
    } else if *tag_ptr > tag && *tag_ptr != GM_NAMETAG_COUNT {
        *tag_ptr = *tag_ptr - 1;
    }
}

/// C `gmMainLib_ClearNameTag`: as `AdjustNameTag`, but clears the slot instead of marking it.
unsafe fn gmMainLib_ClearNameTag(tag_ptr: *mut u8, tag: u8) {
    if *tag_ptr == tag {
        *tag_ptr = 0;
    } else if *tag_ptr > tag && *tag_ptr != GM_NAMETAG_COUNT {
        *tag_ptr = *tag_ptr - 1;
    }
}

pub unsafe fn gmMainLib_8015DBF4(arg0: i32) {
    let arg0 = arg0 as u8;
    let g = gmMainLib_804D3EE0;
    gmMainLib_AdjustNameTag(addr_of_mut!((*gmMainLib_8015CDC8()).nametag), arg0);
    gmMainLib_AdjustNameTag(addr_of_mut!((*g).vs.unk_522.nametag), arg0);
    gmMainLib_AdjustNameTag(addr_of_mut!((*g).vs.unk_528.nametag), arg0);
    gmMainLib_AdjustNameTag(addr_of_mut!((*g).vs.unk_530.nametag), arg0);
    gmMainLib_AdjustNameTag(addr_of_mut!((*g).vs.unk_530.unk_584.unk_586), arg0);

    let hr = addr_of_mut!(crate::tu::gmhomerun::gmHomeRun_VsModeData);
    gmMainLib_AdjustNameTags(hr, arg0);
    let t = addr_of_mut!((*g).modes.table) as *mut VsModeData;
    for m in [
        GmVsMode_Multiman,
        GmVsMode_Training,
        GmVsMode_Melee,
        GmVsMode_SuperSuddenDeath,
        GmVsMode_Invisible,
        GmVsMode_Camera,
        GmVsMode_FixedCamera,
        GmVsMode_SingleButton,
        GmVsMode_Tiny,
        GmVsMode_Giant,
        GmVsMode_Stamina,
        GmVsMode_Slomo,
        GmVsMode_Lightning,
        GmVsMode_Multiman,
    ] {
        gmMainLib_AdjustNameTags(t.add(m as usize), arg0);
    }

    let gr = addr_of_mut!((*g).x1850);
    gmMainLib_ClearNameTag(addr_of_mut!((*gr).unk_x10), arg0);
    gmMainLib_ClearNameTag(addr_of_mut!((*gr).x11), arg0);
    gmMainLib_ClearNameTag(addr_of_mut!((*gr).x13), arg0);
}

/// C `SetDefaultHandicaps`.
unsafe fn SetDefaultHandicaps(mode: *mut VsModeData) {
    for i in 0..GM_MAX_PLAYERS as usize {
        (*mode).start.players[i].handicap = 9;
    }
}

pub unsafe fn gmMainLib_8015EA80() {
    let modes = addr_of_mut!((*gmMainLib_804D3EE0).modes);
    gmMainLib_8015CDEC();
    let t = addr_of_mut!((*modes).table) as *mut VsModeData;
    for m in [
        GmVsMode_Melee,
        GmVsMode_SuperSuddenDeath,
        GmVsMode_Invisible,
        GmVsMode_Camera,
        GmVsMode_FixedCamera,
        GmVsMode_SingleButton,
        GmVsMode_Tiny,
        GmVsMode_Giant,
        GmVsMode_Stamina,
        GmVsMode_Slomo,
        GmVsMode_Lightning,
        GmVsMode_Multiman,
    ] {
        SetDefaultHandicaps(t.add(m as usize));
    }
}

pub unsafe fn gmMainLib_8015ECB0() -> i32 {
    (*gmMainLib_804D3EE0).x1850.bgm as i32
}

pub unsafe fn gmMainLib_8015ECBC() {
    let rules = addr_of_mut!((*gmMainLib_804D3EE0).x1850);
    // m-ex BGM/MenuPlaylist.asm: a weighted draw over the menu playlist when it has weight.
    let n = ext::Mex_MenuPlaylistCount();
    let mut total = 0;
    for i in 0..n {
        if ext::Mex_MenuPlaylistBgm(i) >= 0 {
            total += ext::Mex_MenuPlaylistChance(i);
        }
    }
    if total > 0 {
        let roll = ext::HSD_Randi(total);
        let mut acc = 0;
        for i in 0..n {
            let bgm = ext::Mex_MenuPlaylistBgm(i);
            let chance = if bgm >= 0 { ext::Mex_MenuPlaylistChance(i) } else { 0 };
            if chance == 0 {
                continue;
            }
            acc += chance;
            if roll < acc {
                (*rules).bgm = bgm as u8;
                return;
            }
        }
    }
    if ext::gm_80164600() && ext::gm_80164ABC() {
        if ext::HSD_Randi(4) != 0 {
            (*rules).bgm = 0x34;
        } else {
            (*rules).bgm = 0x36;
        }
    } else {
        (*rules).bgm = 0x34;
    }
}

pub unsafe fn gmMainLib_8015ED30() -> i32 {
    (*gmMainLib_804D3EE0).x1850.unk_xc as i32
}

pub unsafe fn GetRumbleSettingOfPort(port: isize) -> i32 {
    (*save()).x1CB0.rumble_enabled[port as usize] as i32
}

pub unsafe fn gmMainLib_SetRumbleEnabled(port: isize, enabled: bool) {
    (*gmMainLib_GetGamePrefs()).rumble_enabled[port as usize] = enabled as u8;
}

pub unsafe fn gmMainLib_8015ED5C() -> i32 {
    (*gmMainLib_804D3EE0).x1850.unk_14
}

pub unsafe fn gmMainLib_8015ED68(port: isize) {
    (*gmMainLib_804D3EE0).x1850.unk_14 = port as i32;
}

pub unsafe fn gmMainLib_8015ED74() -> u8 {
    (*gmMainLib_GetGamePrefs()).sound_balance as u8
}

pub unsafe fn gmMainLib_8015ED80(arg0: i8) {
    (*gmMainLib_GetGamePrefs()).sound_balance = arg0;
}

/// Unlock-everything (MELEE_UNLOCK_ALL, forced on in netplay): the getters report everything
/// unlocked and hand out a scratch copy, so the game's unlock writes never change the card.
static mut gmMainLib_PcUnlockedChars: u16 = 0;
static mut gmMainLib_PcUnlockedStages: u16 = 0;

pub unsafe fn gmMainLib_GetUnlockedCharactersBitmaskPtr() -> *mut u16 {
    if ext::PcUnlockAll() != 0 {
        gmMainLib_PcUnlockedChars = (*save()).unlocked_characters | 0x7FF;
        return addr_of_mut!(gmMainLib_PcUnlockedChars);
    }
    addr_of_mut!((*save()).unlocked_characters)
}

pub unsafe fn gmMainLib_8015ED98() -> *mut gmm_retval_ED98 {
    addr_of_mut!((*save()).unk_8)
}

pub unsafe fn gmMainLib_8015EDA4() -> *mut u16 {
    if ext::PcUnlockAll() != 0 {
        gmMainLib_PcUnlockedStages = (*save()).x186A | 0x7FF;
        return addr_of_mut!(gmMainLib_PcUnlockedStages);
    }
    addr_of_mut!((*save()).x186A)
}

pub unsafe fn gmMainLib_8015EDB0() -> *mut gmm_retval_EDB0 {
    addr_of_mut!((*save()).unk_28)
}

pub unsafe fn gmMainLib_8015EDBC() -> *mut gmm_retval_EDBC {
    addr_of_mut!((*save()).unk_30)
}

pub unsafe fn gmMainLib_8015EDC8() -> *mut gmm_x1868_1A8_t {
    addr_of_mut!((*save()).unk_1A8)
}

pub unsafe fn gmMainLib_8015EDD4() -> i32 {
    if ext::Mex_Enabled("unlock_sound_test") != 0 || ext::PcUnlockAll() != 0 {
        return 1;
    }
    ((*save()).x186C & 4) as i32
}

pub unsafe fn gmMainLib_8015EDE4() {
    (*save()).x186C |= 4;
}

pub unsafe fn gmMainLib_8015EDF8() {
    (*save()).x186C &= !4;
}

pub unsafe fn gmMainLib_8015EE0C() -> i32 {
    if ext::Mex_Enabled("unlock_score_display") != 0 || ext::PcUnlockAll() != 0 {
        return 1;
    }
    ((*save()).x186C & 1) as i32
}

pub unsafe fn gmMainLib_8015EE1C() {
    (*save()).x186C |= 1;
}

pub unsafe fn gmMainLib_8015EE30() {
    (*save()).x186C &= !1;
}

pub unsafe fn gmMainLib_8015EE44() -> i32 {
    if ext::Mex_Enabled("unlock_random_stage_select") != 0 || ext::PcUnlockAll() != 0 {
        return 1;
    }
    ((*save()).x186C & 2) as i32
}

pub unsafe fn gmMainLib_8015EE54() {
    (*save()).x186C |= 2;
}

pub unsafe fn gmMainLib_8015EE68() {
    (*save()).x186C &= !2;
    (*gmMainLib_GetGamePrefs()).stage_mask = gmMainLib_DefaultGamePrefs.stage_mask;
}

pub unsafe fn gmMainLib_8015EE90() -> i32 {
    if ext::Mex_Enabled("unlock_all_star") != 0 || ext::PcUnlockAll() != 0 {
        return 1;
    }
    ((*save()).x186C & 8) as i32
}

pub unsafe fn gmMainLib_8015EEA0() {
    (*save()).x186C |= 8;
}

pub unsafe fn gmMainLib_8015EEB4() {
    (*save()).x186C &= !8;
}

pub unsafe fn gmMainLib_8015EEC8() {
    for i in 0..SELKIND_COUNT as usize {
        let data = fighter_base().add(i);
        memzero(addr_of_mut!((*data).x7C), core::mem::size_of::<FighterData_x7C>());
    }
    let s = save() as *mut u8;
    let start = offset_of!(GmSaveData, x1A68);
    let end = offset_of!(GmSaveData, x1B40);
    memzero(s.add(start), end - start);
}

pub unsafe fn gmMainLib_8015EF30(stats: *mut GmStats) {
    (*stats).sd_count = 0;
    (*stats).attacks_hit = 0;
    (*stats).attacks_total = 0;
    (*stats).damage_dealt = 0;
    (*stats).damage_taken = 0;
    (*stats).peak_damage = 0;
    (*stats).match_count = 0;
    (*stats).victories = 0;
    (*stats).losses = 0;
    (*stats).play_time = 0;
    (*stats).total_player_count = 0;
    (*stats).damage_recovered = 0;
    (*stats).walk_distance = 0;
    (*stats).run_distance = 0;
    (*stats).fall_distance = 0;
    (*stats).peak_height = 0;
    (*stats).coins_swiped = 0;
    (*stats).coins_lost = 0;
    (*stats).coins_collected = 0;
}

pub unsafe fn InitializePersistentNameData(arg0: i32) {
    let idx = arg0 as u8 as i32;
    let data = nametag_slot(gmMainLib_GetNameTagDataBanks(), idx);
    for i in 0..GM_NAMETAG_COUNT as usize {
        (*data).vs_kos[i] = 0;
    }
    gmMainLib_8015EF30(addr_of_mut!((*data).stats));
    for i in 0..SELKIND_COUNT as usize {
        (*data).play_time_by_fighter[i] = 0;
    }
    (*data).x1A2 = 5;
}

/// C `GetNameTagSlot`: the name-tag slot `idx` within the banks.
pub unsafe fn GetNameTagSlot(banks: *mut NameTagDataBank, idx: i32) -> *mut NameTagData {
    nametag_slot(banks, idx)
}

/// C `ResetAllPersistentFighterData`.
unsafe fn ResetAllPersistentFighterData() {
    for i in 0..SELKIND_COUNT as usize {
        let base = fighter_base();
        for j in 0..SELKIND_COUNT as usize {
            (*base.add(i)).fighter_kos[j] = 0;
        }
        gmMainLib_8015EF30(addr_of_mut!((*base.add(i)).stats));
    }
}

/// C `ResetPersistentFighterData`.
unsafe fn ResetPersistentFighterData(i: i32) {
    let base = fighter_base();
    let k = i as u8 as usize;
    for j in 0..SELKIND_COUNT as usize {
        (*base.add(k)).fighter_kos[j] = 0;
    }
    gmMainLib_8015EF30(addr_of_mut!((*base.add(k)).stats));
}

pub unsafe fn gmMainLib_8015F150() {
    for i in 0..SELKIND_COUNT as i32 {
        ResetPersistentFighterData(i);
    }
}

pub unsafe fn gmMainLib_8015F260() {
    for i in 0..GM_NAMETAG_COUNT as i32 {
        InitializePersistentNameData(i);
    }
}

pub unsafe fn gmMainLib_8015F464() {
    memzero(gmMainLib_8015ED98(), core::mem::size_of::<gmm_retval_ED98>());
}

pub unsafe fn gmMainLib_8015F490() {
    memzero(gmMainLib_8015EDB0(), core::mem::size_of::<gmm_retval_EDB0>());
}

pub unsafe fn gmMainLib_8015F4BC() {
    memzero(gmMainLib_8015EDBC(), core::mem::size_of::<gmm_retval_EDBC>());
}

pub unsafe fn gmMainLib_8015F4E8() -> u32 {
    (*gmMainLib_GetGamePrefs()).deflicker as u32
}

pub unsafe fn gmMainLib_8015F4F4(arg0: u8) {
    (*save()).x1CB0.deflicker = arg0;
}

pub unsafe fn gmMainLib_8015F500() {
    let mode = if gmMainLib_8046B0F0.progressive != 0 {
        if gmMainLib_8015F4E8() != 0 { ext::gmMainLib_803D4A80() } else { ext::GXNtsc480Prog() }
    } else if gmMainLib_8015F4E8() != 0 {
        ext::GXNtsc480IntDf()
    } else {
        ext::GXNtsc480Int()
    };
    ext::HSD_VISetConfigure(mode);
}

pub unsafe fn gmMainLib_8015F588(deflicker: bool) {
    let mode = if gmMainLib_8046B0F0.progressive != 0 {
        if deflicker { ext::gmMainLib_803D4A80() } else { ext::GXNtsc480Prog() }
    } else if deflicker {
        ext::GXNtsc480IntDf()
    } else {
        ext::GXNtsc480Int()
    };
    ext::HSD_VISetConfigure(mode);
}

pub unsafe fn gmMainLib_8015F600(arg0: i32, arg1: i32) {
    if arg0 == 1 {
        ResetAllPersistentFighterData();
        let s = save() as *mut u8;
        let start = offset_of!(GmSaveData, trophy_count);
        let end = offset_of!(GmSaveData, x1F2C);
        memzero(s.add(start), end - start);
        ext::Toy_80311960();

        // m-ex Fill Save Data.asm: a reset save starts with every trophy collected.
        if ext::Mex_Enabled("fill_trophy_save_data") != 0 {
            (*save()).trophy_count = 293;
            for ti in 0..TY_TROPHY_COUNT as usize {
                (*save()).trophy_flags[ti] = 99;
            }
        }

        if arg1 == 0 {
            ext::Toy_803124BC();
            ext::Toy_SetUnlockState(ext::Toy_80305058(2, 0x63, 0, 100.0) as i16 as i32, 1);
        }

        (*save()).x1CB0 = gmMainLib_DefaultGamePrefs;

        let lang = match ext::lbLang_GetLanguageSetting() {
            1 => 1,
            // C: `lang` is uninitialised for other values; 0 is the port's choice.
            _ => 0,
        };
        ext::lbLang_SetSavedLanguage(lang);

        memzero(card() as *mut u8, offset_of!(GmSaveData, x1CB0));
        ext::gm_801623FC(0x32);
        ext::gm_IncrementPowerCount();

        if arg1 == 0 && ext::Toy_803048C0(0xA5) > 0 && ext::gm_80164430(0x14) == 0 {
            ext::gm_80164504(0x14);
        }
    } else {
        let bank_offset = (arg0 - 2) * GM_NAMETAG_BANK_SIZE;
        for j in 0..GM_NAMETAG_BANK_SIZE {
            let idx = j + bank_offset;
            InitializePersistentNameData(idx);

            let bank = gmMainLib_GetNameTagDataBanks();
            let data = nametag_slot(bank, (idx as u8) as i32);
            let src = ext::mnName_8023749C((idx as u8) as i32);
            if !src.is_null() {
                let mut k = 0usize;
                let mut p = src as *const i8;
                loop {
                    let c = *p;
                    if gmMainLib_804D3EE4[0] == c {
                        break;
                    }
                    (*data).namedata[k] = c;
                    k += 1;
                    p = p.add(1);
                }
                (*data).namedata[k] = gmMainLib_804D3EE4[0];
            } else {
                (*data).namedata[0] = gmMainLib_804D3EE4[0];
            }
            (*data).rumble_enabled = 1;
        }
    }
}

pub unsafe fn gmMainLib_8015FA34(arg0: i32) {
    for i in 1..9 {
        if (arg0 != 0 && arg0 != 2) || ext::lb_8001B6E0(i) != 0 {
            gmMainLib_8015F600(i, 0);
        } else if i == 1 && gmMainLib_8046B0F0.skip_intro == 0 {
            ext::gm_IncrementPowerCount();
        }
    }
    if ext::DbLevel() > ext::DbLKind_DebugDevelop() && ext::db_804D6B20() {
        (*save()).x186C = 0xFF;
        ext::gm_80164F18();
        ext::gm_8016468C();
        ext::gm_8017297C();
        ext::gm_801741FC();
    }
    ext::lbAudioAx_80028690();
    gmMainLib_8015F500();
}

pub unsafe fn gmMainLib_8015FB68() {
    (*save()).x186C = 0;
    ext::gm_8016505C();
    ext::gm_801647D0();
    ext::gm_801729EC();
    ext::gm_80174238();
    ext::Toy_80311960();
}

/// C `gmMainLib_MexApplyTournamentDefaults` (PC only, opt-in through MELEE_MEX): applies the
/// tournament-standard defaults to the default rules and prefs before they are copied in.
pub unsafe fn gmMainLib_MexApplyTournamentDefaults() {
    if ext::Mex_Enabled("default_stock_mode") != 0 {
        gmMainLib_DefaultGameRules.mode = 1;
        gmMainLib_DefaultGameRules.time_limit = 0;
    }
    if ext::Mex_Enabled("default_4_stocks") != 0 {
        gmMainLib_DefaultGameRules.stock_count = 4;
    }
    if ext::Mex_Enabled("default_8_minutes") != 0 {
        gmMainLib_DefaultGameRules.stock_time_limit = 8;
        gmMainLib_DefaultGameRules.friendly_fire = 1;
    }
    if ext::Mex_Enabled("default_no_items") != 0 {
        gmMainLib_DefaultGamePrefs.item_freq = 0xFF;
    }
    if ext::Mex_Enabled("default_tournament_stages") != 0 {
        gmMainLib_DefaultGamePrefs.stage_mask = 0xE70000B0;
    }
}

pub unsafe fn gmMainLib_8015FBA4() {
    gmMainLib_MexApplyTournamentDefaults();

    memzero(gmMainLib_804D3EE0 as *mut u8, core::mem::size_of::<gmm_x0>());
    if ext::DVDConvertPathToEntrynum("/usa.ini") != -1 {
        ext::lbLang_SetLanguageSetting(1);
        ext::lbLang_SetSavedLanguage(1);
    } else {
        ext::lbLang_SetLanguageSetting(0);
        ext::lbLang_SetSavedLanguage(0);
    }

    gmMainLib_8045A6C0.x1850 = gmMainLib_DefaultGameRules;
    for i in 1..9 {
        gmMainLib_8015F600(i, 1);
    }
    ext::lbAudioAx_80028690();
    gmMainLib_8015F500();
}

pub unsafe fn gmMainLib_8015FC74() -> i32 {
    let prev = gmMainLib_8046B0F0.x10;
    gmMainLib_8046B0F0.x10 = ext::lbTime_GetTimeInSeconds() as i32;
    gmMainLib_8046B0F0.x10 - prev
}

pub unsafe fn gmMainLib_8015FCC0() {
    let tmp = addr_of_mut!(gmMainLib_8046B0F0);
    (*tmp).skip_intro = if ext::OSGetResetCode() == 0x8000_0000 { 1 } else { 0 };
    (*tmp).resetting = 0;
    (*tmp).progressive = 0;
    (*tmp).xC = 0;
    (*tmp).x10 = ext::lbTime_GetTimeInSeconds() as i32;
}

/// Callees in TUs that are not ported yet, plus PC-only shims and globals owned by other TUs
/// (`Mex_*`, `Toy_*`, `db_*`, `GXNtsc480*`). Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut LBTIME_GETTIMEINSECONDS: Option<fn() -> u32> = None;
            #[allow(non_snake_case)]
            pub fn lbTime_GetTimeInSeconds() -> u32 {
                match unsafe { LBTIME_GETTIMEINSECONDS } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { melee_runtime::tu::lbtime::lbTime_GetTimeInSeconds() as _ }; },
                }
            }
    pub static mut LB_8001B6E0: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001B6E0(i: i32) -> i32 {
                match unsafe { LB_8001B6E0 } {
                    Some(f) => unsafe { f(i) },
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lb_8001B6E0(i as _) as _ }; },
                }
            }
    pub static mut TOY_80311960: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn Toy_80311960()  {
                match unsafe { TOY_80311960 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_80311960(); } return; },
                }
            }
    pub static mut TOY_803124BC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn Toy_803124BC()  {
                match unsafe { TOY_803124BC } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_803124BC(); } return; },
                }
            }
    pub static mut TOY_80305058: Option<fn(i32, i32, i32, f32) -> i16> = None;
            #[allow(non_snake_case)]
            pub fn Toy_80305058(a: i32, b: i32, c: i32, d: f32) -> i16 {
                match unsafe { TOY_80305058 } {
                    Some(f) => unsafe { f(a, b, c, d) },
                    None => { return unsafe { melee_items::tu::toy::Toy_80305058(a as _, b as _, c as _, d as _) as _ }; },
                }
            }
    pub static mut TOY_SETUNLOCKSTATE: Option<fn(i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn Toy_SetUnlockState(a: i32, b: i32)  {
                match unsafe { TOY_SETUNLOCKSTATE } {
                    Some(f) => unsafe { f(a, b) },
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_SetUnlockState(a as _, b as _); } return; },
                }
            }
    pub static mut TOY_803048C0: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Toy_803048C0(a: i32) -> i32 {
                match unsafe { TOY_803048C0 } {
                    Some(f) => unsafe { f(a) },
                    None => { return unsafe { melee_items::tu::toy::Toy_803048C0(a as _) as _ }; },
                }
            }
    pub static mut GM_INCREMENTPOWERCOUNT: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_IncrementPowerCount()  {
                match unsafe { GM_INCREMENTPOWERCOUNT } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_IncrementPowerCount(); } return; },
                }
            }
    pub static mut GM_801623FC: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801623FC(a: i32)  {
                match unsafe { GM_801623FC } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_801623FC(a as _); } return; },
                }
            }
    pub static mut GM_80164430: Option<fn(u32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_80164430(a: u32) -> i32 {
                match unsafe { GM_80164430 } {
                    Some(f) => unsafe { f(a) },
                    None => { return unsafe { crate::tu::gm_1601::gm_80164430(a as _) as _ }; },
                }
            }
    pub static mut GM_80164504: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80164504(a: u32)  {
                match unsafe { GM_80164504 } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_80164504(a as _); } return; },
                }
            }
    pub static mut GM_80164F18: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80164F18()  {
                match unsafe { GM_80164F18 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_80164F18(); } return; },
                }
            }
    pub static mut GM_8016468C: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_8016468C()  {
                match unsafe { GM_8016468C } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_8016468C(); } return; },
                }
            }
    pub static mut GM_8017297C: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_8017297C()  {
                match unsafe { GM_8017297C } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_16f1::gm_8017297C(); } return; },
                }
            }
    pub static mut GM_801741FC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801741FC()  {
                match unsafe { GM_801741FC } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1736::gm_801741FC(); } return; },
                }
            }
    pub static mut GM_8016505C: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_8016505C()  {
                match unsafe { GM_8016505C } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_8016505C(); } return; },
                }
            }
    pub static mut GM_801647D0: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801647D0()  {
                match unsafe { GM_801647D0 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_801647D0(); } return; },
                }
            }
    pub static mut GM_801729EC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801729EC()  {
                match unsafe { GM_801729EC } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_16f1::gm_801729EC(); } return; },
                }
            }
    pub static mut GM_80174238: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80174238()  {
                match unsafe { GM_80174238 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1736::gm_80174238(); } return; },
                }
            }
    pub static mut GM_80164600: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn gm_80164600() -> bool {
                match unsafe { GM_80164600 } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_1601::gm_80164600() as _ }; },
                }
            }
    pub static mut GM_80164ABC: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn gm_80164ABC() -> bool {
                match unsafe { GM_80164ABC } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_1601::gm_80164ABC() as _ }; },
                }
            }
    hook!(HSD_RANDI, HSD_Randi, (max: i32) -> i32);
    pub static mut MNNAME_8023749C: Option<fn(i32) -> *const i8> = None;
            #[allow(non_snake_case)]
            pub fn mnName_8023749C(idx: i32) -> *const i8 {
                match unsafe { MNNAME_8023749C } {
                    Some(f) => unsafe { f(idx) },
                    None => { return unsafe { crate::tu::mnname::mnName_8023749C(idx as _) as _ }; },
                }
            }
    pub static mut LBLANG_GETLANGUAGESETTING: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbLang_GetLanguageSetting() -> i32 {
                match unsafe { LBLANG_GETLANGUAGESETTING } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_GetLanguageSetting() as _ }; },
                }
            }
    pub static mut LBLANG_SETSAVEDLANGUAGE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbLang_SetSavedLanguage(v: i32)  {
                match unsafe { LBLANG_SETSAVEDLANGUAGE } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { let _ = melee_runtime::tu::lblanguage::lbLang_SetSavedLanguage(v as _); } return; },
                }
            }
    pub static mut LBLANG_SETLANGUAGESETTING: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbLang_SetLanguageSetting(v: i32)  {
                match unsafe { LBLANG_SETLANGUAGESETTING } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { let _ = melee_runtime::tu::lblanguage::lbLang_SetLanguageSetting(v as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_80028690: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80028690()  {
                match unsafe { LBAUDIOAX_80028690 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80028690(); } return; },
                }
            }
    hook!(DVDCONVERTPATHTOENTRYNUM, DVDConvertPathToEntrynum, (path: &str) -> i32);
    hook!(OSGETRESETCODE, OSGetResetCode, () -> u32);
    pub static mut HSD_VISETCONFIGURE: Option<fn(*mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_VISetConfigure(mode: *mut c_void)  {
                match unsafe { HSD_VISETCONFIGURE } {
                    Some(f) => unsafe { f(mode) },
                    None => { unsafe { let _ = melee_runtime::tu::video::HSD_VISetConfigure(mode as _); } return; },
                }
            }
    hook!(GXNTSC480PROG, GXNtsc480Prog, () -> *mut c_void);
    hook!(GXNTSC480INTDF, GXNtsc480IntDf, () -> *mut c_void);
    hook!(GXNTSC480INT, GXNtsc480Int, () -> *mut c_void);
    hook!(GMMAINLIB_803D4A80, gmMainLib_803D4A80, () -> *mut c_void);
    hook!(MEX_ENABLED, Mex_Enabled, (name: &str) -> i32);
    hook!(PCUNLOCKALL, PcUnlockAll, () -> i32);
    hook!(MEX_MENUPLAYLISTCOUNT, Mex_MenuPlaylistCount, () -> i32);
    hook!(MEX_MENUPLAYLISTBGM, Mex_MenuPlaylistBgm, (i: i32) -> i32);
    hook!(MEX_MENUPLAYLISTCHANCE, Mex_MenuPlaylistChance, (i: i32) -> i32);
    hook!(DBLEVEL, DbLevel, () -> i32);
    hook!(DBLKIND_DEBUGDEVELOP, DbLKind_DebugDevelop, () -> i32);
    hook!(DB_804D6B20, db_804D6B20, () -> bool);
}

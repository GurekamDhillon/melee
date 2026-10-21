//! src/melee/ft/ftdata.c (+ its src/melee/ft/ftdata_costumes.inc): per-kind fighter data tables,
//! file loading, m-ex kind init (TARGET_PC), the Geno thrown stand-in banks (TARGET_PC).
//!
//! Tables: every per-kind table of the C is a `static mut` here, sized `Ft_Kind_Max` (127). The
//! numeric tables (anim counts, demo counts, effect banks) carry their retail values. The
//! retail rows that point at per-character code or data (callbacks, motion-state tables, file
//! name strings, costume lists, demo strings) live in group 4 (`melee_chara`), which this crate
//! cannot name: they start empty and are installed by `install_retail_row` (TODO: called by the
//! character crate at init), exactly where the C's static initialisers would have put them.
//!
//! Addresses: the C stores animation-bank addresses in u32 fields (`Fighter_WaitAnimData::x14`,
//! `Fighter::x5A4/x5A8` compared as u32). Natively these go through `ext::gc_addr32` /
//! `ext::gc_ptr` (TODO: the guest address model), not a truncating cast.
//!
//! Not ported: `ftData_PcTestCostumes` (a guest-heap self test of the installer, needs the OS heap
//! shims; `tests/tu_ftdata.rs` tests `ftData_PcInstallCostumes` directly instead) and the pc/geno
//! includes (geno_define_data.inc, geno_anim_bank_*.inc: group 6, stubbed in `ext`).
//! No float math except ft_8008521C's three subtractions (no fused site).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::{c_char, c_void};

pub type G = *mut HSD_GObj;
const KMAX: usize = Ft_Kind_Max as usize;
const MEX0: i32 = Ft_Kind_Mex0 as i32;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees / globals in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    use core::ffi::{c_char, c_void};
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        // guest address model (see module doc)
        GC_ADDR32 gc_addr32(p: *const c_void) -> u32;
        GC_PTR gc_ptr(a: u32) -> *mut c_void;
        OS_REPORT OSReport(msg: &str);
        // pc/geno (group 6)
        GENO_GAME_CLEAR_PROFILE_STORAGE GenoGame_ClearProfileStorage();
        GENO_DEFINE_RESET_DESCRIPTORS GenoDefine_ResetDescriptors();
        GENO_DEFINE_INIT_KINDS GenoDefine_InitKinds();
        GENO_DEFINE_LOAD GenoDefine_Load(kind: i32, base: i32);
        GENO_DEFINE_CAPTURE_BASE GenoDefine_CaptureBase(kind: i32);
        GENO_DEFINE_BASE_KIND Geno_DefineBaseKind(kind: i32) -> i32;
        GENO_DEFINE_IS_NONE Geno_DefineIsNone(kind: i32) -> i32;
        GENO_DEFINE_TEMPLATE_SKELETON Geno_DefineTemplateSkeleton(kind: i32) -> i32;
        GENO_DEFINE_THROWN_AS Geno_DefineThrownAs(kind: i32) -> i32;
        GENO_DEFINE_THROWN_AS_ROW Geno_DefineThrownAsRow(kind: i32, row: i32) -> i32;
        GENO_THROW_REGION Geno_ThrowRegion(size: &mut u32) -> *mut u8;
        GENO_THROW_BANK_LIMIT Geno_ThrowBankLimit(def: i32) -> i32;
        PC_COSTUME_ALLOC PcCostumeAlloc(bytes: usize) -> *mut c_void;
        // pc/platform m-ex + skins
        MEX_SLOT_INTERNAL Mex_SlotInternal(slot: i32) -> i32;
        MEX_FT_PL_FILE Mex_FtPlFile(k: i32) -> *mut c_char;
        MEX_FT_PL_SYMBOL Mex_FtPlSymbol(k: i32) -> *mut c_char;
        MEX_FT_ANIM_FILE Mex_FtAnimFile(k: i32) -> *mut c_char;
        MEX_FT_ANIM_COUNT Mex_FtAnimCount(k: i32) -> i32;
        MEX_FT_COSTUME_COUNT Mex_FtCostumeCount(k: i32) -> i32;
        MEX_FT_COSTUME_STRING Mex_FtCostumeString(k: i32, c: i32, which: i32) -> *const c_char;
        MEX_FT_DEMO_STRINGS Mex_FtDemoStrings(k: i32) -> *mut Fighter_DemoStrings;
        MEX_FT_FUNC Mex_FtFunc(slot: i32, k: i32) -> *mut c_void;
        MEX_FT_BASE_KIND Mex_FtBaseKind(k: i32) -> i32;
        MEX_FT_EFFECT_INDEX Mex_FtEffectIndex(k: i32) -> i32;
        MEX_EFFECT_STRING Mex_EffectString(i: i32, which: i32) -> *const c_char;
        MEX_FT_FUNCTION_INSTALL Mex_FtFunctionInstall(kind: i32, data: *mut u8, size: u32);
        PLAYER_MEX_SET_MAPPING Player_MexSetMapping(ckind: i32, fkind: i32);
        FTKB_MEX_COPY_KIND_DATA ftKb_MexCopyKindData(dst: i32, src: i32, internal: i32);
        FTKB_MEX_COPY_KIND_HAT ftKb_MexCopyKindHat(dst: i32, src: i32, internal: i32);
        SKINS_ADDED Skins_Added(fk: i32) -> i32;
        SKINS_BASE Skins_Base(fk: i32) -> i32;
        SKINS_STRING Skins_String(fk: i32, c: i32, which: i32) -> *const c_char;
        SKINS255_TEST_COUNT Skins255_TestCount() -> i32;
        EF_BANK_MAX_ EF_BANK_MAX() -> i32;
        // game
        HSD_JOBJ_GET_TRANSLATION_PTR jobj_translate(j: *mut HSD_JObj) -> Vec3;
        LBDVD_800178E8 lbDvd_800178E8(a: i32, name: *mut c_char, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32);
        EFASYNC_LOAD_ASYNC efAsync_LoadAsync(id: i32);
        LBARCHIVE_80017040_FT lbArchive_80017040_ftdata(out: *mut *mut HSD_Archive, file: *mut c_char, data: *mut *mut ftData, sym: *mut c_char);
        LBARCHIVE_80017040_COSTUME lbArchive_80017040_costume(out: *mut *mut HSD_Archive, file: *mut c_char, joint: *mut UnkCostumeStruct, jname: *mut c_char, matanim: *mut *mut HSD_MatAnimJoint, mname: *mut c_char);
        FTKB_SPECIALN_800EEC34 ftKb_SpecialN_800EEC34(arg0: i32, color: u8, n: u8);
        LBFILE_800168A0 lbFile_800168A0(heap: i32, name: *mut c_char, out: &mut *mut c_void, size: &mut usize);
        LBFILE_GET_SIZE lbFileGetSize(name: *mut c_char) -> usize;
        LBFILE_LOAD_INTO lbFile_LoadInto(name: *mut c_char, dst: *mut u8, cap: usize, size: &mut usize) -> i32;
        LBHEAP_80015BD0 lbHeap_80015BD0(heap: i32, size: usize) -> *mut u8;
        LBHEAP_80015CA8 lbHeap_80015CA8(heap: i32, p: *mut u8);
        LBHEAP_FREE lbHeap_Free(heap: i32) -> u32;
        HSD_OBJ_ALLOC_X59C HSD_ObjAlloc_fighter_x59C() -> *mut Fighter_x59C_t;
        LBARCHIVE_RELOCATE lbArchiveRelocate(ar: *mut HSD_Archive, data: *mut u8, size: i32, offset: isize) -> i32;
        LBARQ_80014BD0 lbArq_80014BD0(src: u32, dst: *mut Fighter_x59C_t, size: u32, a: i32, b: i32);
        HSD_ARCHIVE_PARSE HSD_ArchiveParse(ar: *mut HSD_Archive, data: *mut u8, size: i32) -> i32;
        HSD_ARCHIVE_GET_PUBLIC_ADDRESS HSD_ArchiveGetPublicAddress(ar: *mut HSD_Archive, name: *mut c_char) -> *mut c_void;
        PLAYER_GET_PLAYER_SLOT_TYPE Player_GetPlayerSlotType(slot: i32) -> i32;
        PLAYER_GET_ENTITY_AT_INDEX Player_GetEntityAtIndex(slot: i32, i: i32) -> G;
        // `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`.
        PLINK_FIGHTER_HEAD plink_fighter_head() -> G;
    }
}

// ---- tables -----------------------------------------------------------------------------------

/// `ft_8045993C_t` (ftdata.c-local, GC 8 bytes; `x6_b0:1`, `x6_b1_b2:2` MSB-first in the u16 at +6).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ft_8045993C_t {
    pub pad_x0: u32,
    pub pad_x4: [u8; 2],
    pub bf_6: [u8; 2],
}
impl ft_8045993C_t {
    pub fn x6_b0(&self) -> u8 { self.bf_6[0] >> 7 }
    pub fn set_x6_b0(&mut self, v: u8) { self.bf_6[0] = (self.bf_6[0] & 0x7F) | ((v & 1) << 7) }
    pub fn x6_b1_b2(&self) -> u8 { (self.bf_6[0] >> 5) & 3 }
    pub fn set_x6_b1_b2(&mut self, v: u8) { self.bf_6[0] = (self.bf_6[0] & !0x60) | ((v & 3) << 5) }
}

/// `struct StringPair`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StringPair {
    pub a: *mut c_char,
    pub b: *mut c_char,
}

/// `ftData_UnkCallbackPairs0` row.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CallbackPair {
    pub x0: HSD_GObjEvent,
    pub x4: Option<unsafe extern "C" fn(*mut HSD_GObj, i32, f32)>,
}

pub static mut gFtDataList: [*mut ftData; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ft_8045993C: [ft_8045993C_t; 6] = [ft_8045993C_t { pad_x0: 0, pad_x4: [0; 2], bf_6: [0; 2] }; 6];
pub static mut ft_8045996C: [i32; KMAX] = [0; KMAX];
static mut ftData_ThrownRows: [*mut Fighter_WaitAnimData; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_AnimReloadData: [*mut c_void; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_AnimBankBytes: [usize; KMAX] = [0; KMAX];

pub static mut CostumeListsForeachCharacter: [UnkCostumeList; KMAX] =
    [UnkCostumeList { costume_list: core::ptr::null_mut(), numCostumes: 0 }; KMAX];

const fn counts(v: [i32; 33]) -> [ftData_UnkCountStruct; KMAX] {
    let mut out = [ftData_UnkCountStruct { data: core::ptr::null_mut(), count: 0 }; KMAX];
    let mut i = 0;
    while i < 33 {
        out[i].count = v[i];
        i += 1;
    }
    out
}

pub static mut ftData_Table_Unk0: [ftData_UnkCountStruct; KMAX] = counts([
    303, 327, 318, 337, 479, 316, 314, 317, 326, 318, 321, 321, 320, 313, 314, 327, 314, 312,
    327, 311, 314, 303, 327, 320, 323, 318, 327, 345, 344, 295, 295, 316, 296,
]);
pub static mut ftData_UnkIntPairs: [ftData_UnkCountStruct; KMAX] = counts([
    16, 14, 14, 14, 18, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 16, 14, 14, 14, 14,
    14, 14, 14, 14, 14, 14, 14, 14, 14, 15, 14,
]);
const fn bytes(v: [u8; 33]) -> [u8; KMAX] {
    let mut out = [0u8; KMAX];
    let mut i = 0;
    while i < 33 {
        out[i] = v[i];
        i += 1;
    }
    out
}
/// `ftData_UnkBytePerCharacter` (`-1` entries are 0xFF).
pub static mut ftData_UnkBytePerCharacter: [u8; KMAX] = bytes([
    1, 3, 4, 8, 5, 12, 6, 17, 10, 15, 14, 14, 7, 2, 9, 11, 13, 18, 16, 17, 6, 1, 3, 7, 0xFF, 19,
    49, 0xFF, 0xFF, 0xFF, 0xFF, 12, 0xFF,
]);

pub static mut ftData_Table_Unk1: [Event; KMAX] = [None; KMAX];
pub static mut ftData_OnLoad: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnDeath: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnUserDataRemove: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_CharacterStateTables: [*mut MotionState; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_UnkMotionStates0: [*mut MotionState; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_SpecialS: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialAirHi: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialAirLw: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialAirS: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialAirN: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialN: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialLw: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_SpecialHi: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnAbsorb: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemPickupExt: [Fighter_ItemEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemInvisible: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemVisible: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemDropExt: [Fighter_ItemEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemPickup: [Fighter_ItemEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnItemDrop: [Fighter_ItemEvent; KMAX] = [None; KMAX];
pub static mut ftData_UnkMotionStates1: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_UnkMotionStates2: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnKnockbackEnter: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_OnKnockbackExit: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_UnkMotionStates3: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_UnkMotionStates4: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftKindCalcIndiviParamTable: [HSD_GObjEvent; KMAX] = [None; KMAX];
pub static mut ftData_803C1F40: [StringPair; KMAX] =
    [StringPair { a: core::ptr::null_mut(), b: core::ptr::null_mut() }; KMAX];
pub static mut ftData_UnkMotionStates5: [Event; KMAX] = [None; KMAX];
pub static mut ftData_UnkMtxFunc0: [Fighter_UnkMtxEvent; KMAX] = [None; KMAX];
pub static mut ftData_UnkIntBoolFunc0: ftData_UnkModelStruct =
    ftData_UnkModelStruct { model_events: [None; KMAX], getter: [None; KMAX] };
pub static mut ftData_UnkCallbackPairs0: [CallbackPair; KMAX] = [CallbackPair { x0: None, x4: None }; KMAX];
pub static mut ftData_803C2360: [*mut Fighter_CostumeStrings; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_803C23E4: [*mut c_char; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_803C2468: [*mut Fighter_DemoStrings; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut ftData_803C24EC: [Fighter_MotionFileStringGetter; KMAX] = [None; KMAX];
pub static mut ftData_UnkDemoCallbacks0: [Fighter_UnkPtrEvent; KMAX] = [None; KMAX];

pub static ftData_assert_msg_0: &str = "cant get corps model array!\n";
pub static ftData_assert_msg_1: &str = "HSD_ArchiveParse error!\n";

/// One retail row of the per-character tables (what the C's static initialisers hold for `kind`).
#[derive(Clone, Copy)]
pub struct RetailRow {
    pub costume_list: *mut UnkCostumeStruct,
    pub num_costumes: u8,
    pub table_unk1: Event,
    pub on_load: HSD_GObjEvent,
    pub on_death: HSD_GObjEvent,
    pub on_user_data_remove: HSD_GObjEvent,
    pub character_state_table: *mut MotionState,
    pub unk_motion_states0: *mut MotionState,
    pub special: [HSD_GObjEvent; 8], // S, AirHi, AirLw, AirS, AirN, N, Lw, Hi
    pub on_absorb: HSD_GObjEvent,
    pub on_item_pickup_ext: Fighter_ItemEvent,
    pub on_item_invisible: HSD_GObjEvent,
    pub on_item_visible: HSD_GObjEvent,
    pub on_item_drop_ext: Fighter_ItemEvent,
    pub on_item_pickup: Fighter_ItemEvent,
    pub on_item_drop: Fighter_ItemEvent,
    pub unk_motion_states: [HSD_GObjEvent; 4], // 1, 2, 3, 4
    pub on_knockback_enter: HSD_GObjEvent,
    pub on_knockback_exit: HSD_GObjEvent,
    pub calc_indivi_param: HSD_GObjEvent,
    pub dat: StringPair,
    pub unk_motion_states5: Event,
    pub unk_mtx_func0: Fighter_UnkMtxEvent,
    pub model_event: Fighter_ModelEvent,
    pub model_getter: Option<unsafe extern "C" fn(*mut HSD_GObj) -> *mut HSD_JObj>,
    pub callback_pair: CallbackPair,
    pub costume_strings: *mut Fighter_CostumeStrings,
    pub anim_file: *mut c_char,
    pub demo_strings: *mut Fighter_DemoStrings,
    pub motion_file_string: Fighter_MotionFileStringGetter,
    pub demo_callback: Fighter_UnkPtrEvent,
}

/// Install one retail kind's row (the C's static initialisers; TODO: called by melee_chara).
pub fn install_retail_row(kind: usize, r: &RetailRow) {
    unsafe {
        CostumeListsForeachCharacter[kind].costume_list = r.costume_list;
        CostumeListsForeachCharacter[kind].numCostumes = r.num_costumes;
        ftData_Table_Unk1[kind] = r.table_unk1;
        ftData_OnLoad[kind] = r.on_load;
        ftData_OnDeath[kind] = r.on_death;
        ftData_OnUserDataRemove[kind] = r.on_user_data_remove;
        ftData_CharacterStateTables[kind] = r.character_state_table;
        ftData_UnkMotionStates0[kind] = r.unk_motion_states0;
        ftData_SpecialS[kind] = r.special[0];
        ftData_SpecialAirHi[kind] = r.special[1];
        ftData_SpecialAirLw[kind] = r.special[2];
        ftData_SpecialAirS[kind] = r.special[3];
        ftData_SpecialAirN[kind] = r.special[4];
        ftData_SpecialN[kind] = r.special[5];
        ftData_SpecialLw[kind] = r.special[6];
        ftData_SpecialHi[kind] = r.special[7];
        ftData_OnAbsorb[kind] = r.on_absorb;
        ftData_OnItemPickupExt[kind] = r.on_item_pickup_ext;
        ftData_OnItemInvisible[kind] = r.on_item_invisible;
        ftData_OnItemVisible[kind] = r.on_item_visible;
        ftData_OnItemDropExt[kind] = r.on_item_drop_ext;
        ftData_OnItemPickup[kind] = r.on_item_pickup;
        ftData_OnItemDrop[kind] = r.on_item_drop;
        ftData_UnkMotionStates1[kind] = r.unk_motion_states[0];
        ftData_UnkMotionStates2[kind] = r.unk_motion_states[1];
        ftData_UnkMotionStates3[kind] = r.unk_motion_states[2];
        ftData_UnkMotionStates4[kind] = r.unk_motion_states[3];
        ftData_OnKnockbackEnter[kind] = r.on_knockback_enter;
        ftData_OnKnockbackExit[kind] = r.on_knockback_exit;
        ftKindCalcIndiviParamTable[kind] = r.calc_indivi_param;
        ftData_803C1F40[kind] = r.dat;
        ftData_UnkMotionStates5[kind] = r.unk_motion_states5;
        ftData_UnkMtxFunc0[kind] = r.unk_mtx_func0;
        ftData_UnkIntBoolFunc0.model_events[kind] = r.model_event;
        ftData_UnkIntBoolFunc0.getter[kind] = r.model_getter;
        ftData_UnkCallbackPairs0[kind] = r.callback_pair;
        ftData_803C2360[kind] = r.costume_strings;
        ftData_803C23E4[kind] = r.anim_file;
        ftData_803C2468[kind] = r.demo_strings;
        ftData_803C24EC[kind] = r.motion_file_string;
        ftData_UnkDemoCallbacks0[kind] = r.demo_callback;
    }
}

fn report(s: String) {
    ext::OSReport(&s);
}

fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        return "(null)".into();
    }
    unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

// ---- functions --------------------------------------------------------------------------------

/// `order_bss` (MUST_MATCH only: references the bss tables in order; no effect).
pub fn order_bss() {}

/// `ft_8008521C`: self_vel = joint translation - cur_pos.
pub fn ft_8008521C(g: G) {
    let f = fp(g);
    let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
    assert!(!j.is_null(), "jobj.h: jobj");
    let pos = unsafe { (*j).translate };
    f.self_vel.x = pos.x - f.cur_pos.x;
    f.self_vel.y = pos.y - f.cur_pos.y;
    f.self_vel.z = pos.z - f.cur_pos.z;
}

fn reset_8045993c(i: usize) {
    unsafe {
        ft_8045993C[i].pad_x0 = 0;
        ft_8045993C[i].set_x6_b0(0);
        ft_8045993C[i].set_x6_b1_b2(0);
    }
}

/// `ft_800852B0`: scene reset of every loaded-file pointer.
pub fn ft_800852B0() {
    ext::GenoGame_ClearProfileStorage();
    ext::GenoDefine_ResetDescriptors();
    unsafe {
        ftData_ThrownRows = [core::ptr::null_mut(); KMAX];
        ftData_ThrowBankReset();
        ftData_AnimReloadData = [core::ptr::null_mut(); KMAX];
        ftData_AnimBankBytes = [0; KMAX];
        for i in 0..KMAX {
            gFtDataList[i] = core::ptr::null_mut();
            let cl = &mut CostumeListsForeachCharacter[i];
            for c in 0..cl.numCostumes as usize {
                (*cl.costume_list.add(c)).joint = core::ptr::null_mut();
                (*cl.costume_list.add(c)).pad_x8 = 0;
            }
            ftData_Table_Unk0[i].data = core::ptr::null_mut();
            ftData_UnkIntPairs[i].data = core::ptr::null_mut();
        }
    }
    for i in 0..6 {
        reset_8045993c(i);
    }
}

/// `ft_8008549C`: clear the per-kind reference counters.
pub fn ft_8008549C() {
    unsafe { ft_8045996C = [0; KMAX] };
}

/// `ftData_MexMotionRowOk`: a real motion row of an m-ex file (name NULL or in the archive,
/// script in the archive).
pub fn ftData_MexMotionRowOk(row: &Fighter_WaitAnimData, lo: *const u8, hi: *const u8) -> bool {
    let name = row.x0 as *const u8;
    let cmd = row.xC as *const u8;
    (name.is_null() || (name >= lo && name < hi)) && !cmd.is_null() && cmd >= lo && cmd < hi
}

/// `ftData_MexAnimFlags`: the authoring-kind bits an m-ex motion gets.
pub fn ftData_MexAnimFlags(flags: u32, kind: i32) -> u32 {
    if flags & 0x3F == 0x21 {
        return flags;
    }
    if kind < 64 {
        return (flags & !0x3F) | kind as u32;
    }
    (flags & !0x3F) | FT_ANIM_KIND_SELF as u32
}

static mut ftData_PcFixtureString: Fighter_CostumeStrings = Fighter_CostumeStrings {
    dat_filename: core::ptr::null_mut(),
    joint_name: core::ptr::null_mut(),
    matanim_joint_name: core::ptr::null_mut(),
};

/// `ftData_PcFixtureAt`.
pub fn ftData_PcFixtureAt(_internal: i32, _c: i32, which: i32) -> *const c_char {
    unsafe {
        match which {
            0 => ftData_PcFixtureString.dat_filename,
            1 => ftData_PcFixtureString.joint_name,
            _ => ftData_PcFixtureString.matanim_joint_name,
        }
    }
}

pub type StringAt = fn(i32, i32, i32) -> *const c_char;

struct PcRows {
    fk: i32,
    native: i32,
    native_at: Option<StringAt>,
}
static mut ftData_PcRows: PcRows = PcRows { fk: 0, native: 0, native_at: None };

/// `ftData_PcRowString`: own rows first, then the skins registry's.
pub fn ftData_PcRowString(internal: i32, c: i32, which: i32) -> *const c_char {
    unsafe {
        if c < ftData_PcRows.native {
            return (ftData_PcRows.native_at.expect("native_at"))(internal, c, which);
        }
        ext::Skins_String(ftData_PcRows.fk, c, which)
    }
}

/// `ftData_PcInstallCostumes` (ftdata_costumes.inc): `count` exactly-sized descriptors, the
/// first `have` kept from the current lists.
pub fn ftData_PcInstallCostumes(kind: i32, count: i32, have: i32, string_at: StringAt, internal: i32,
                                allocate: fn(usize) -> *mut c_void) {
    assert!(count > 0 && count <= 255 && have <= count, "invalid costume descriptor count");
    let k = kind as usize;
    let n = count as usize;
    let strings = allocate(n * core::mem::size_of::<Fighter_CostumeStrings>()) as *mut Fighter_CostumeStrings;
    let lists = allocate(n * core::mem::size_of::<UnkCostumeStruct>()) as *mut UnkCostumeStruct;
    assert!(!strings.is_null() && !lists.is_null(), "costume descriptor allocation exhausted");
    unsafe {
        core::ptr::write_bytes(lists, 0, n);
        for c in 0..n {
            if (c as i32) < have {
                *strings.add(c) = *ftData_803C2360[k].add(c);
                *lists.add(c) = *CostumeListsForeachCharacter[k].costume_list.add(c);
            } else {
                (*strings.add(c)).dat_filename = string_at(internal, c as i32, 0) as *mut c_char;
                (*strings.add(c)).joint_name = string_at(internal, c as i32, 1) as *mut c_char;
                (*strings.add(c)).matanim_joint_name = string_at(internal, c as i32, 2) as *mut c_char;
            }
        }
        ftData_803C2360[k] = strings;
        CostumeListsForeachCharacter[k].costume_list = lists;
        CostumeListsForeachCharacter[k].numCostumes = count as u8;
    }
}

fn pc_costume_alloc(n: usize) -> *mut c_void {
    ext::PcCostumeAlloc(n)
}

/// `ftData_PcInstallRows`.
pub fn ftData_PcInstallRows(fk: i32, native: i32, have: i32, native_at: StringAt, internal: i32) {
    let added = ext::Skins_Added(fk);
    unsafe {
        ftData_PcRows = PcRows { fk, native, native_at: Some(native_at) };
    }
    ftData_PcInstallCostumes(fk, native + added, have, ftData_PcRowString, internal, pc_costume_alloc);
    unsafe {
        let rows = ftData_803C2360[fk as usize];
        for c in native..native + added {
            let row = &mut *rows.add(c as usize);
            if row.dat_filename.is_null() {
                row.dat_filename = (*rows).dat_filename;
                row.joint_name = (*rows).joint_name;
                row.matanim_joint_name = (*rows).matanim_joint_name;
            }
        }
    }
    if added > 0 {
        report(format!("skins: kind {fk}: {} costumes ({native} own, {added} from skins)\n", native + added));
    }
}

static mut MEX_INIT_DONE: bool = false;

/// Fill `table[fk]` from m-ex slot `idx`, or the clone base's row when m-ex left it empty.
unsafe fn mex_fill<T: Copy>(table: *mut [T; KMAX], fk: usize, base: usize, idx: i32, k: i32, is_null: fn(&T) -> bool) {
    let p = ext::Mex_FtFunc(idx, k);
    // SAFETY: every table element is a (nullable) pointer-sized fn or data pointer.
    let v: T = core::mem::transmute_copy(&p);
    (*table)[fk] = v;
    if is_null(&(*table)[fk]) {
        (*table)[fk] = (*table)[base];
    }
}
fn none<T>(v: &Option<T>) -> bool { v.is_none() }
fn nullp<T>(v: &*mut T) -> bool { v.is_null() }

/// `ftData_MexInitKinds`: fill the m-ex kind slots from MxDt.dat, once.
pub fn ftData_MexInitKinds() {
    unsafe {
        if MEX_INIT_DONE {
            return;
        }
        MEX_INIT_DONE = true;
        let mut n = 0;
        for slot in 0..(Ft_Kind_Max as i32 - MEX0) {
            let fk = (MEX0 + slot) as usize;
            let k = ext::Mex_SlotInternal(slot);
            if k < 0 {
                continue;
            }
            let base = ext::Mex_FtBaseKind(k) as usize;
            macro_rules! fill { ($t:ident, $i:expr, $n:ident) => { mex_fill(core::ptr::addr_of_mut!($t), fk, base, $i, k, $n) }; }
            fill!(ftData_OnLoad, 0, none);
            fill!(ftData_OnDeath, 1, none);
            fill!(ftData_OnUserDataRemove, 2, none);
            fill!(ftData_CharacterStateTables, 3, nullp);
            fill!(ftData_SpecialN, 4, none);
            fill!(ftData_SpecialAirN, 5, none);
            fill!(ftData_SpecialS, 6, none);
            fill!(ftData_SpecialAirS, 7, none);
            fill!(ftData_SpecialHi, 8, none);
            fill!(ftData_SpecialAirHi, 9, none);
            fill!(ftData_SpecialLw, 10, none);
            fill!(ftData_SpecialAirLw, 11, none);
            fill!(ftData_OnAbsorb, 12, none);
            fill!(ftData_OnItemPickupExt, 13, none);
            fill!(ftData_OnItemInvisible, 14, none);
            fill!(ftData_OnItemVisible, 15, none);
            fill!(ftData_OnItemDropExt, 16, none);
            fill!(ftData_OnItemPickup, 17, none);
            fill!(ftData_OnItemDrop, 18, none);
            fill!(ftData_UnkMotionStates1, 19, none);
            fill!(ftData_UnkMotionStates2, 20, none);
            fill!(ftData_OnKnockbackEnter, 21, none);
            fill!(ftData_OnKnockbackExit, 22, none);
            fill!(ftData_UnkMotionStates3, 23, none);
            fill!(ftData_UnkMotionStates4, 24, none);
            fill!(ftKindCalcIndiviParamTable, 25, none);
            fill!(ftData_UnkMtxFunc0, 26, none);
            fill!(ftData_803C24EC, 38, none);
            fill!(ftData_UnkDemoCallbacks0, 39, none);
            fill!(ftData_UnkMotionStates0, 40, nullp);

            ftData_803C1F40[fk].a = ext::Mex_FtPlFile(k);
            ftData_803C1F40[fk].b = ext::Mex_FtPlSymbol(k);
            ftData_803C23E4[fk] = ext::Mex_FtAnimFile(k);
            ftData_Table_Unk0[fk].count = ext::Mex_FtAnimCount(k);
            ftData_803C2468[fk] = ext::Mex_FtDemoStrings(k);
            if ftData_803C2468[fk].is_null() {
                ftData_803C2468[fk] = ftData_803C2468[base];
            }
            let ncost = ext::Mex_FtCostumeCount(k);
            if ncost > 0 {
                ftData_PcInstallRows(fk as i32, ncost, 0, ext::Mex_FtCostumeString, k);
                report(format!("gw: kind {fk}: {ncost} costume descriptors allocated\n"));
            }
            ftData_Table_Unk1[fk] = ftData_Table_Unk1[base];
            ftData_UnkMotionStates5[fk] = ftData_UnkMotionStates5[base];
            ftData_UnkIntPairs[fk].count = ftData_UnkIntPairs[base].count;
            let eb = ext::Mex_FtEffectIndex(k);
            let ef_max = ext::EF_BANK_MAX();
            if eb >= 0 && eb < ef_max && !ext::Mex_EffectString(eb, 0).is_null() {
                ftData_UnkBytePerCharacter[fk] = eb as u8;
            } else {
                if eb >= ef_max && eb != 255 {
                    report(format!("gw: ERROR m-ex fighter {k}'s effect bank {eb} is past the {ef_max} the particle system holds - it uses its clone base's effects\n"));
                }
                ftData_UnkBytePerCharacter[fk] = ftData_UnkBytePerCharacter[base];
            }
            ftData_UnkCallbackPairs0[fk] = ftData_UnkCallbackPairs0[base];
            ext::ftKb_MexCopyKindHat(fk as i32, base as i32, k);
            ext::ftKb_MexCopyKindData(fk as i32, base as i32, k);
            ext::Player_MexSetMapping(ChKind_Mex0 as i32 + slot, fk as i32);
            n += 1;
        }
        for slot in 0..Ft_Kind_MasterH as i32 {
            let mut ncost = ext::Mex_FtCostumeCount(slot);
            let have = CostumeListsForeachCharacter[slot as usize].numCostumes as i32;
            if ftData_803C2360[slot as usize].is_null() {
                continue;
            }
            if ncost < have {
                ncost = have;
            }
            if ncost <= have && ext::Skins_Added(slot) == 0 {
                continue;
            }
            ftData_PcInstallRows(slot, ncost, have, ext::Mex_FtCostumeString, slot);
            report(format!("gw: retail kind {slot}: {} costumes ({have} preserved)\n", ncost + ext::Skins_Added(slot)));
        }
        let count = ext::Skins255_TestCount();
        let have = CostumeListsForeachCharacter[Ft_Kind_Mario as usize].numCostumes as i32;
        if count > have {
            ftData_PcFixtureString = *ftData_803C2360[Ft_Kind_Mario as usize];
            ftData_PcInstallCostumes(Ft_Kind_Mario as i32, count, have, ftData_PcFixtureAt, 0, pc_costume_alloc);
            report(format!("skins255: coordinator fixture Mario has {count} costumes (extra rows repeat default assets)\n"));
        }
        if n != 0 {
            report(format!("gw: {n} m-ex fighter kinds from MxDt.dat (kinds {}..{})\n", MEX0, MEX0 + n - 1));
        }
    }
}

/// `ftData_PcRetailKindOfOnLoad`: the retail kind (0..26) whose onLoad is `native`, or -1.
pub fn ftData_PcRetailKindOfOnLoad(native: u32) -> i32 {
    for r in 0..=26 {
        let p = unsafe { ftData_OnLoad[r] }.map_or(0usize, |f| f as usize);
        if p as u32 == native {
            return r as i32;
        }
    }
    -1
}

/// `ftData_IsMexKind`.
pub fn ftData_IsMexKind(kind: i32) -> bool {
    ext::Geno_DefineBaseKind(kind) < 0 && kind >= MEX0 && kind < Ft_Kind_Max as i32
        && unsafe { !ftData_803C1F40[kind as usize].a.is_null() }
}

/// `ftData_80085560`: adjust a kind's reference count; negative is fatal.
pub fn ftData_80085560(idx: i32, increment: i32) {
    unsafe {
        ft_8045996C[idx as usize] = ft_8045996C[idx as usize].wrapping_add(increment);
        if ft_8045996C[idx as usize] < 0 {
            report("fighter reference counter error!\n".into());
            panic!("ftdata.c:1944");
        }
    }
}

/// `ftData_800855C8`: queue a kind's files (costume `color`, 0xFF = all) for async load.
pub fn ftData_800855C8(kind: i32, mut color: u8) {
    ftData_MexInitKinds();
    ext::GenoDefine_InitKinds();
    let k = kind as usize;
    unsafe {
        let n = CostumeListsForeachCharacter[k].numCostumes;
        if color != 0xFF && color >= n {
            color = 0;
        }
        if !ftData_803C1F40[k].a.is_null() {
            ext::lbDvd_800178E8(2, ftData_803C1F40[k].a, 4, 4, 0, 1, 4, 2, 0);
        }
        let (lo, hi) = if color == 0xFF { (0, n as i32) } else { (color as i32, color as i32 + 1) };
        for i in lo..hi {
            let s = &*ftData_803C2360[k].add(i as usize);
            if !s.dat_filename.is_null() {
                ext::lbDvd_800178E8(2, s.dat_filename, 4, 4, 0, 1, 3, 1, 0);
            }
        }
        // C: `u8 != (char) -1` compares 255 with -1 after promotion: always true (kept).
        if ftData_UnkBytePerCharacter[k] as i32 != -1 {
            ext::efAsync_LoadAsync(ftData_UnkBytePerCharacter[k] as i32);
        }
        if !ftData_803C23E4[k].is_null() {
            ext::lbDvd_800178E8(1, ftData_803C23E4[k], 5, 5, 0, 0, 1, 8, 0);
        }
    }
}

/// The nearest ftData pointer field above `lo` that bounds a table in the archive.
unsafe fn table_bound(fd: *mut ftData, lo: *mut u8, arch_lo: *mut u8, arch_hi: *mut u8) -> *mut u8 {
    let mut hi = arch_hi;
    if (fd as *mut u8) > lo && (fd as *mut u8) < hi {
        hi = fd as *mut u8;
    }
    let words = core::mem::size_of::<ftData>() / core::mem::size_of::<*mut u8>();
    for w in 0..words {
        let p = *(fd as *mut *mut u8).add(w);
        if p > lo && p < hi && p >= arch_lo && p < arch_hi {
            hi = p;
        }
    }
    hi
}

/// `ftData_8008572C`: load a kind's Pl file (m-ex: rewrite its motions' authoring kind).
pub fn ftData_8008572C(kind: i32) {
    ftData_MexInitKinds();
    ext::GenoDefine_InitKinds();
    let base = ext::Geno_DefineBaseKind(kind);
    if base >= 0 {
        ftData_8008572C(base);
        ext::GenoDefine_Load(kind, base);
        return;
    }
    let k = kind as usize;
    unsafe {
        if gFtDataList[k].is_null() {
            let mut ar: *mut HSD_Archive = core::ptr::null_mut();
            ext::lbArchive_80017040_ftdata(&mut ar, ftData_803C1F40[k].a, &mut gFtDataList[k], ftData_803C1F40[k].b);
            if ftData_IsMexKind(kind) {
                report(format!("gw: ftData_8008572C kind={kind} file={} sym={}\n",
                               cstr(ftData_803C1F40[k].a), cstr(ftData_803C1F40[k].b)));
                let fd = gFtDataList[k];
                let row_sz = core::mem::size_of::<Fighter_WaitAnimData>();
                let arch_lo = (*ar).data;
                let arch_hi = arch_lo.add((*ar).header.data_size as usize);
                {
                    let tab_lo = (*fd).xC as *mut u8;
                    let tab_hi = table_bound(fd, tab_lo, arch_lo, arch_hi);
                    let (mut n_main, mut n_holes) = (0, 0);
                    let cnt = ftData_Table_Unk0[k].count;
                    let mut i = 0;
                    while i < cnt {
                        let row = (*fd).xC.add(i as usize);
                        let entry = row as *mut u8;
                        if entry < arch_lo || entry.add(row_sz) > tab_hi {
                            break;
                        }
                        if !ftData_MexMotionRowOk(&*row, arch_lo, arch_hi) {
                            if (*row).x0.is_null() && (*row).xC.is_null() && (*row).x4 == 0 && (*row).x8 == 0 {
                                n_holes += 1;
                                i += 1;
                                continue;
                            }
                            break;
                        }
                        (*row).x10_animCurrFlags = ftData_MexAnimFlags((*row).x10_animCurrFlags as u32, kind) as i32;
                        n_main += 1;
                        i += 1;
                    }
                    if n_main + n_holes != cnt {
                        report(format!("gw: kind {kind}: motion table ends after {} entries (the base has {cnt}); the rest keep their authored flags\n", n_main + n_holes));
                    }
                    report(format!("gw: kind {kind}: {n_main} motion rows rewritten, {n_holes} empty rows skipped\n"));
                }
                if !(*fd).x14.is_null() {
                    let demo_lo = (*fd).x14 as *mut u8;
                    let demo_hi = table_bound(fd, demo_lo, arch_lo, arch_hi);
                    let mut n_demo = 0;
                    let cnt = ftData_UnkIntPairs[k].count;
                    for i in 0..cnt.max(0) as usize {
                        let row = (*fd).x14.add(i);
                        let entry = row as *mut u8;
                        if entry < arch_lo || entry.add(row_sz) > arch_hi || entry.add(row_sz) > demo_hi
                            || !ftData_MexMotionRowOk(&*row, arch_lo, arch_hi)
                        {
                            break;
                        }
                        (*row).x10_animCurrFlags = ftData_MexAnimFlags((*row).x10_animCurrFlags as u32, kind) as i32;
                        n_demo += 1;
                    }
                    if n_demo != cnt {
                        report(format!("gw: kind {kind} has {n_demo} demo motions, not the base's {cnt}\n"));
                        ftData_UnkIntPairs[k].count = n_demo;
                    }
                }
                ext::Mex_FtFunctionInstall(kind, (*ar).data, (*ar).header.data_size);
            }
        }
    }
    let kk = kind as u32;
    if kk == Ft_Kind_Mario || kk == Ft_Kind_Captain || kk == Ft_Kind_Fox || kk == Ft_Kind_Falco || kk == Ft_Kind_Mars
        || kk == Ft_Kind_Purin || kk == Ft_Kind_Seak || kk == Ft_Kind_Zelda
    {
        ext::GenoDefine_CaptureBase(kind);
    }
}

/// `ftData_8008578C`: Kirby's copy-hat costume load.
pub fn ftData_8008578C(arg0: i32, mut color: u8) {
    let n = unsafe { CostumeListsForeachCharacter[Ft_Kind_Kirby as usize].numCostumes };
    if color != 0xFF && color >= n {
        color = 0;
    }
    ext::ftKb_SpecialN_800EEC34(arg0, color, n);
}

/// `ftData_800857E0`.
pub fn ftData_800857E0(kind: i32) {
    if let Some(f) = unsafe { ftData_UnkMotionStates5[kind as usize] } {
        unsafe { f() };
    }
}

unsafe fn load_costume(k: usize, costume_id: usize, t: *mut UnkCostumeStruct) {
    if (*t).joint.is_null() {
        let s = &*ftData_803C2360[k].add(costume_id);
        if !s.matanim_joint_name.is_null() {
            ext::lbArchive_80017040_costume(&mut (*t).x14_archive, s.dat_filename, t, s.joint_name, &mut (*t).x4, s.matanim_joint_name);
        } else {
            ext::lbArchive_80017040_costume(&mut (*t).x14_archive, s.dat_filename, t, s.joint_name, core::ptr::null_mut(), s.matanim_joint_name);
            (*CostumeListsForeachCharacter[k].costume_list.add(costume_id)).x4 = core::ptr::null_mut();
        }
    }
}

/// `ftData_80085820`: load one costume (TARGET_PC: an id past the count falls back to 0).
pub fn ftData_80085820(kind: i32, mut costume_id: i32) {
    let k = kind as usize;
    unsafe {
        let n = CostumeListsForeachCharacter[k].numCostumes as i32;
        if costume_id < 0 || costume_id >= n {
            report(format!("gw: kind {kind} has no costume {costume_id} ({n} costumes) - using 0\n"));
            costume_id = 0;
        }
        let t = CostumeListsForeachCharacter[k].costume_list.add(costume_id as usize);
        if ext::Skins_Added(kind) > 0 && costume_id >= ext::Skins_Base(kind) && (*t).joint.is_null() {
            report(format!("skins: kind {kind} loads skin costume {costume_id}: {}\n",
                           cstr((*ftData_803C2360[k].add(costume_id as usize)).dat_filename)));
        }
        if ftData_IsMexKind(kind) {
            let s = &*ftData_803C2360[k].add(costume_id as usize);
            report(format!("gw: ftData_80085820 kind={kind} costume={costume_id} file={} joint={}\n",
                           cstr(s.dat_filename), cstr(s.joint_name)));
        }
        load_costume(k, costume_id as usize, t);
    }
}

/// `ftData_800858E4`: load one costume (no range check).
pub fn ftData_800858E4(kind: i32, costume_id: i32) {
    let k = kind as usize;
    unsafe {
        let t = CostumeListsForeachCharacter[k].costume_list.add(costume_id as usize);
        load_costume(k, costume_id as usize, t);
    }
}

/// `ftData_800859A8`: release `fp`'s x61C slot unless another fighter shares it.
pub fn ftData_800859A8(f: *mut Fighter) {
    let r6 = unsafe { (*f).x61C };
    if r6 == -1 {
        return;
    }
    let mut g = ext::plink_fighter_head();
    while !g.is_null() {
        let cur = unsafe { (*g).user_data } as *mut Fighter;
        if f != cur && r6 == unsafe { (*cur).x61C } {
            return;
        }
        g = unsafe { (*g).next };
    }
    unsafe { ft_8045993C[r6 as usize].set_x6_b0(0) };
}

/// `ftData_80085A14`: load a kind's animation bank and bind its motion rows.
pub fn ftData_80085A14(kind: i32) {
    let k = kind as usize;
    unsafe {
        let fd = gFtDataList[k];
        if ftData_Table_Unk0[k].data.is_null() {
            let mut sp18: *mut c_void = core::ptr::null_mut();
            let mut sp10: usize = 0;
            if !ftData_AnimReloadData[k].is_null() {
                sp18 = ftData_AnimReloadData[k];
                sp10 = ftData_AnimBankBytes[k];
                ftData_AnimReloadData[k] = core::ptr::null_mut();
            } else {
                ext::lbFile_800168A0(1, ftData_803C23E4[k], &mut sp18, &mut sp10);
            }
            ftData_AnimBankBytes[k] = sp10;
            report(format!("AllA anim: kind {kind} file {} size 0x{:X}\n", cstr(ftData_803C23E4[k]), sp10 as u32));
            let a_head = sp18;
            assert!(!a_head.is_null(), "ftdata.c:0x974");
            let base = ext::gc_addr32(a_head);
            let mut i: u32 = 0;
            while i < ftData_Table_Unk0[k].count as u32 {
                let row = &mut *(*fd).xC.add(i as usize);
                let r0 = row.x8 as u32;
                if r0 != 0 {
                    if r0 > FT_ANIM_BUF_SIZE as u32 {
                        panic!("fighter figatree over! {r0:x}");
                    }
                    row.x14 = base.wrapping_add(row.x4 as u32);
                }
                i += 1;
            }
            ftData_Table_Unk0[k].data = a_head;
        }
    }
}

// ---- Geno thrown stand-in banks (TARGET_PC) ---------------------------------------------------

const GENO_THROW_BANKS: i32 = 1;
const GENO_THROW_BANKS_MAX: usize = 6;
const GENO_THROW_ROWS_MAX: i32 = 512;
const GENO_THROW_ARENA_MIN: usize = 0x1E0000;

/// `ftData_ThrowBank`.
pub struct ThrowBank {
    pub kind: i32,
    pub buf: *mut u8,
    pub cap: usize,
    pub sz: usize,
    pub stamp: u32,
    pub rows: [Fighter_WaitAnimData; GENO_THROW_ROWS_MAX as usize],
}

pub struct ThrowState {
    pub banks: [ThrowBank; GENO_THROW_BANKS_MAX],
    pub clock: u32,
    pub loads: u32,
    pub evicts: u32,
    pub prepared: [u8; KMAX],
    pub region_cur: *mut u8,
    pub region_lo: *mut u8,
    pub region_end: *mut u8,
    pub limit: i32,
}

static mut THROW: Option<Box<ThrowState>> = None;

fn throw() -> &'static mut ThrowState {
    unsafe {
        THROW.get_or_insert_with(|| {
            // SAFETY: all-zero is valid for every field (null pointers, zero counts); kind is
            // fixed to -1 just below, as the C's first reset does.
            let mut b: Box<ThrowState> = Box::new(core::mem::zeroed());
            for bank in b.banks.iter_mut() {
                bank.kind = -1;
            }
            b
        })
    }
}

/// `ftData_ThrowArenaAlloc`: bump from the persist region, else heap 0.
pub fn ftData_ThrowArenaAlloc(mut need: usize) -> *mut u8 {
    let t = throw();
    if t.region_cur.is_null() {
        let mut size = 0u32;
        let base = ext::Geno_ThrowRegion(&mut size);
        t.region_cur = base;
        t.region_lo = base;
        t.region_end = base.wrapping_add(size as usize);
        report(format!("geno: throw-bank region 0x{:X}, {size} bytes (outside heap 0)\n", ext::gc_addr32(base as *const c_void)));
    }
    need = (need + 31) & !31usize;
    if !t.region_cur.is_null() && (t.region_end as usize - t.region_cur as usize) >= need {
        let p = t.region_cur;
        t.region_cur = unsafe { t.region_cur.add(need) };
        return p;
    }
    ext::lbHeap_80015BD0(0, need)
}

/// `ftData_ThrowBankLimit` (clamped 1..GENO_THROW_BANKS_MAX, read once).
pub fn ftData_ThrowBankLimit() -> i32 {
    let t = throw();
    if t.limit == 0 {
        let mut l = ext::Geno_ThrowBankLimit(GENO_THROW_BANKS);
        if l < 1 {
            l = 1;
        }
        if l > GENO_THROW_BANKS_MAX as i32 {
            l = GENO_THROW_BANKS_MAX as i32;
        }
        t.limit = l;
    }
    t.limit
}

/// `ftData_ThrowBankReset`.
pub fn ftData_ThrowBankReset() {
    let t = throw();
    for b in t.banks.iter_mut() {
        b.kind = -1;
        b.buf = core::ptr::null_mut();
        b.cap = 0;
        b.sz = 0;
    }
    t.loads = 0;
    t.evicts = 0;
    t.region_cur = core::ptr::null_mut();
    t.prepared = [0; KMAX];
}

/// `ftData_ThrowBankReport`.
pub fn ftData_ThrowBankReport(what: &str, kind: i32) {
    let lim = ftData_ThrowBankLimit();
    let t = throw();
    let (mut n, mut arenas, mut bytes) = (0, 0, 0u32);
    for b in t.banks.iter() {
        arenas += !b.buf.is_null() as i32;
        if b.kind >= 0 {
            n += 1;
            bytes = bytes.wrapping_add(b.sz as u32);
        }
    }
    report(format!("geno: define budgets: throw banks {n}/{lim}, {bytes} bytes, arenas {arenas}, loads {}, evictions {}, heap 0 free 0x{:X} ({what} kind {kind})\n",
                   t.loads, t.evicts, ext::lbHeap_Free(0)));
}

/// `ftData_ThrowBankDrop`: forget a bank; no fighter may keep an animation key into it.
pub fn ftData_ThrowBankDrop(b: &mut ThrowBank) {
    let mut g = ext::plink_fighter_head();
    let lo = b.buf as usize;
    let hi = lo + b.cap;
    while !g.is_null() {
        let f = unsafe { (*g).user_data } as *mut Fighter;
        if !f.is_null() {
            unsafe {
                let a = (*f).x5A4 as usize;
                if a >= lo && a < hi {
                    (*f).x5A4 = core::ptr::null_mut();
                }
                let a = (*f).x5A8 as usize;
                if a >= lo && a < hi {
                    (*f).x5A8 = core::ptr::null_mut();
                }
            }
        }
        g = unsafe { (*g).next };
    }
    unsafe {
        if b.kind >= 0 && ftData_ThrownRows[b.kind as usize] == b.rows.as_mut_ptr() {
            ftData_ThrownRows[b.kind as usize] = core::ptr::null_mut();
        }
    }
    b.kind = -1;
    b.sz = 0;
}

/// `ftData_ThrownStandInPrepare`: at define load, the stand-in's Pl file and the arenas.
pub fn ftData_ThrownStandInPrepare(kind: i32) {
    let lim = ftData_ThrowBankLimit();
    if kind < 0 || kind >= Ft_Kind_Max as i32 || kind == Ft_Kind_Nana as i32 {
        return;
    }
    ftData_8008572C(kind);
    let k = kind as usize;
    unsafe {
        if gFtDataList[k].is_null() || (*gFtDataList[k]).xC.is_null() || ftData_803C23E4[k].is_null()
            || ftData_Table_Unk0[k].count > GENO_THROW_ROWS_MAX
        {
            return;
        }
        let mut need = (ext::lbFileGetSize(ftData_803C23E4[k]) + 63) & !31usize;
        if need < GENO_THROW_ARENA_MIN {
            need = GENO_THROW_ARENA_MIN;
        }
        for i in 0..lim as usize {
            let t = throw();
            let b = &mut t.banks[i] as *mut ThrowBank;
            if !(*b).buf.is_null() && (*b).cap >= need {
                continue;
            }
            if !(*b).buf.is_null() {
                ftData_ThrowBankDrop(&mut *b);
                let t = throw();
                if !((*b).buf >= t.region_lo && (*b).buf < t.region_end) {
                    ext::lbHeap_80015CA8(0, (*b).buf);
                }
                (*b).buf = core::ptr::null_mut();
            }
            (*b).buf = ftData_ThrowArenaAlloc(need);
            (*b).cap = if (*b).buf.is_null() { 0 } else { need };
        }
        throw().prepared[k] = 1;
    }
    ftData_ThrowBankReport("prepared", kind);
}

/// `ftData_ThrownStandInLoad`: read the stand-in's bank into an arena on first use (LRU).
pub fn ftData_ThrownStandInLoad(kind: i32) {
    let lim = ftData_ThrowBankLimit() as usize;
    if kind < 0 || kind >= Ft_Kind_Max as i32 || kind == Ft_Kind_Nana as i32 || throw().prepared[kind as usize] == 0 {
        return;
    }
    let k = kind as usize;
    unsafe {
        let t = throw();
        let mut b: *mut ThrowBank = core::ptr::null_mut();
        for i in 0..lim {
            if t.banks[i].kind == kind {
                if ftData_ThrownRows[k] == t.banks[i].rows.as_mut_ptr() {
                    t.clock = t.clock.wrapping_add(1);
                    t.banks[i].stamp = t.clock;
                    return;
                }
                b = &mut t.banks[i];
                break;
            }
        }
        if b.is_null() && !ftData_ThrownRows[k].is_null() {
            return;
        }
        let fd = gFtDataList[k];
        if fd.is_null() || (*fd).xC.is_null() {
            return;
        }
        if b.is_null() && !ftData_Table_Unk0[k].data.is_null() {
            ftData_ThrownRows[k] = (*fd).xC;
            return;
        }
        if b.is_null() {
            for i in 0..lim {
                let c: *mut ThrowBank = &mut t.banks[i];
                if (*c).buf.is_null() {
                    continue;
                }
                if b.is_null() || ((*c).kind < 0 && (*b).kind >= 0) || ((*c).kind >= 0 && (*b).kind >= 0 && (*c).stamp < (*b).stamp) {
                    b = c;
                }
            }
            if b.is_null() {
                return;
            }
            if (*b).kind >= 0 {
                let old = (*b).kind;
                t.evicts = t.evicts.wrapping_add(1);
                ftData_ThrowBankDrop(&mut *b);
                ftData_ThrowBankReport("evicted", old);
            }
        } else {
            ftData_ThrowBankDrop(&mut *b);
        }
        let mut sz = 0usize;
        if ext::lbFile_LoadInto(ftData_803C23E4[k], (*b).buf, (*b).cap, &mut sz) == 0 {
            return;
        }
        let n = ftData_Table_Unk0[k].count as usize;
        core::ptr::copy_nonoverlapping((*fd).xC, (*b).rows.as_mut_ptr(), n);
        for i in 0..n {
            let r = &mut (*b).rows[i];
            r.x14 = if r.x8 != 0 { ext::gc_addr32((*b).buf.add(r.x4 as usize) as *const c_void) } else { 0 };
        }
        let t = throw();
        (*b).kind = kind;
        (*b).sz = sz;
        t.clock = t.clock.wrapping_add(1);
        (*b).stamp = t.clock;
        ftData_ThrownRows[k] = (*b).rows.as_mut_ptr();
        t.loads = t.loads.wrapping_add(1);
        report(format!("throw: stand-in kind {kind}: {} (0x{:X} bytes) read into a throw-bank arena for thrown victims\n",
                       cstr(ftData_803C23E4[k]), sz as u32));
    }
    ftData_ThrowBankReport("loaded", kind);
}

static mut STANDIN: Option<Box<Fighter>> = None;

/// `ftData_ThrownStandIn`: the stand-in Fighter a thrown victim reads its rows from, or NULL.
pub fn ftData_ThrownStandIn(kind: i32) -> *mut Fighter {
    if kind < 0 || kind >= Ft_Kind_Max as i32 || kind == Ft_Kind_Nana as i32 {
        return core::ptr::null_mut();
    }
    ftData_ThrownStandInLoad(kind);
    let k = kind as usize;
    unsafe {
        if ftData_ThrownRows[k].is_null() {
            return core::ptr::null_mut();
        }
        let fd = gFtDataList[k];
        if fd.is_null() {
            return core::ptr::null_mut();
        }
        let s = STANDIN.get_or_insert_with(|| Box::new(Fighter::zeroed()));
        s.kind = kind as FighterKind;
        s.x24 = ftData_ThrownRows[k];
        s.x28 = (*fd).x10;
        s.x58C = ftData_Table_Unk0[k].count as u32;
        &mut **s
    }
}

/// `ftData_80085B10`: per-fighter anim buffers + bank (+ a Geno define's thrown stand-ins).
pub fn ftData_80085B10(f: &mut Fighter) {
    let kind = f.kind as i32;
    f.x59C = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A0 = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A4 = core::ptr::null_mut();
    f.x5A8 = core::ptr::null_mut();
    f.x58C = unsafe { ftData_Table_Unk0[kind as usize].count } as u32;
    ftData_80085A14(kind);
    let as_ = ext::Geno_DefineThrownAs(kind);
    if as_ >= 0 && as_ != kind {
        ftData_ThrownStandInPrepare(as_);
        for row in 0..4 {
            let ak = ext::Geno_DefineThrownAsRow(kind, row);
            if ak >= 0 && ak != as_ && ak != kind {
                ftData_ThrownStandInPrepare(ak);
            }
        }
    }
}

/// `ftData_80085B98`: bind demo rows `arg1..=arg2` to the demo archive.
pub fn ftData_80085B98(f: &mut Fighter, arg1: i32, arg2: i32) {
    let k = f.kind as usize;
    let mut r30: u32 = unsafe { ext::gc_addr32(ftData_UnkIntPairs[k].data) };
    if ext::Geno_DefineIsNone(f.kind as i32) != 0 && ext::Geno_DefineTemplateSkeleton(f.kind as i32) == 0 {
        ftData_80085A14(f.kind as i32);
        r30 = unsafe { ext::gc_addr32(ftData_Table_Unk0[k].data) };
    }
    f.x59C = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A0 = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A4 = core::ptr::null_mut();
    f.x5A8 = core::ptr::null_mut();
    f.x58C = unsafe { ftData_UnkIntPairs[k].count } as u32;
    if arg2 as u32 >= f.x58C {
        // C: `int >= u32` compares as unsigned.
        panic!("Demo Status error! {arg2}");
    }
    if r30 != 0 {
        let mut i = arg1;
        while i <= arg2 {
            let r3 = unsafe { &mut *(*f.ft_data).x14.add(i as usize) };
            let r0 = r3.x8 as u32;
            if r0 != 0 {
                if r0 > FT_ANIM_BUF_SIZE as u32 {
                    panic!("fighter figatree over! {r0:x}");
                }
                r3.x14 = r30.wrapping_add(r3.x4 as u32);
            }
            i += 1;
        }
        unsafe { ftData_UnkIntPairs[k].data = core::ptr::null_mut() };
    }
}

#[inline]
fn round_up_32b(x: u32) -> u32 {
    x.wrapping_add(31) & !31
}

/// Shared body of ftData_80085CD8 / ftData_80085E50: (re)load motion `msid`'s figatree into
/// `buf`, keyed by `key`; returns the new figatree, or None when the key already matches.
fn load_figatree(f: *mut Fighter, src: *mut Fighter, msid: i32, key: *mut c_void, buf: *mut Fighter_x59C_t,
                 lines: (u32, u32)) -> Option<(*mut FigaTree, u32)> {
    unsafe {
        let r3 = ftData_80085FD4(&mut *src, msid);
        let x14 = (*r3).x14;
        if x14 == ext::gc_addr32(key) {
            return None;
        }
        if x14 == 0 {
            return Some((core::ptr::null_mut(), 0));
        }
        let mut ar: HSD_Archive = core::mem::zeroed();
        let other = ftData_80086060(&mut *f);
        let size = (*r3).x8;
        if !other.is_null() && (*r3).x14 == ext::gc_addr32((*other).x5A4) {
            core::ptr::copy_nonoverlapping((*other).x59C as *const u8, buf as *mut u8, size as usize);
            let off = (buf as isize).wrapping_sub((*other).x59C as isize);
            if ext::lbArchiveRelocate(&mut ar, (*buf).x0.as_mut_ptr(), size, off) == -1 {
                panic!("lbArchiveRelocate error! {msid:x} (ftdata.c:{:#X})", lines.0);
            }
        } else {
            let a = (*r3).x14;
            if a < 0x8000_0000 {
                ext::lbArq_80014BD0(a, buf, round_up_32b(size as u32), 0, 0);
            } else {
                core::ptr::copy_nonoverlapping(ext::gc_ptr(a) as *const u8, buf as *mut u8, size as usize);
            }
            if ext::HSD_ArchiveParse(&mut ar, (*buf).x0.as_mut_ptr(), size) == -1 {
                panic!("HSD_ArchiveParse error! {msid:x} (ftdata.c:{:#X})", lines.1);
            }
        }
        Some((ext::HSD_ArchiveGetPublicAddress(&mut ar, (*r3).x0) as *mut FigaTree, (*r3).x14))
    }
}

/// `ftData_80085CD8`: load motion `msid` (read from `arg1`'s table) as `fp`'s main figatree.
pub fn ftData_80085CD8(f: *mut Fighter, arg1: *mut Fighter, msid: i32) {
    unsafe {
        // C: `int < u32` compares as unsigned.
        if !((msid as u32) < (*arg1).x58C) {
            return;
        }
        if let Some((tree, x14)) = load_figatree(f, arg1, msid, (*f).x5A4, (*f).x59C, (0x9FA, 0xA0F)) {
            (*f).x590 = tree;
            (*f).x5A4 = if x14 == 0 { core::ptr::null_mut() } else { ext::gc_ptr(x14) };
        }
    }
}

/// `ftData_80085E50`: load motion `msid` as `arg0`'s secondary figatree; returns it.
pub fn ftData_80085E50(f: *mut Fighter, msid: i32) -> *mut FigaTree {
    unsafe {
        if !((msid as u32) < (*f).x58C) {
            return core::ptr::null_mut();
        }
        if let Some((tree, x14)) = load_figatree(f, f, msid, (*f).x5A8, (*f).x5A0, (0xA30, 0xA45)) {
            (*f).x598 = tree;
            (*f).x5A8 = if x14 == 0 { core::ptr::null_mut() } else { ext::gc_ptr(x14) };
        }
        (*f).x598
    }
}

/// `ftData_80085FD4`: motion row `msid` (Nana outside demos falls back to Popo's unbound rows).
/// Returns the row itself: the C's `ftData_80085FD4_ret*` is a cast of it (same GC layout; the
/// native `ftData_80085FD4_ret` does not share the row's layout, so callers read the row).
pub fn ftData_80085FD4(f: &mut Fighter, msid: i32) -> *mut Fighter_WaitAnimData {
    unsafe {
        if f.kind == Ft_Kind_Nana
            && ext::Player_GetPlayerSlotType(f.player_id as i32) != Gm_PKind_Demo as i32
            && (*f.x24.offset(msid as isize)).x14 == 0
        {
            return (*gFtDataList[Ft_Kind_Popo as usize]).xC.offset(msid as isize);
        }
        f.x24.offset(msid as isize)
    }
}

/// `ftData_80085FD4(...)->x8` for the hooks that read only the size word.
pub fn ftData_80085FD4_x8(f: &mut Fighter, msid: i32) -> usize {
    unsafe { (*ftData_80085FD4(f, msid)).x8 as u32 as usize }
}

/// `ftData_80086060`: Nana's Popo (outside demos), else NULL.
pub fn ftData_80086060(f: &mut Fighter) -> *mut Fighter {
    if f.kind == Ft_Kind_Nana && ext::Player_GetPlayerSlotType(f.player_id as i32) != Gm_PKind_Demo as i32 {
        let g = ext::Player_GetEntityAtIndex(f.player_id as i32, 0);
        if !g.is_null() {
            return unsafe { (*g).user_data } as *mut Fighter;
        }
    }
    core::ptr::null_mut()
}

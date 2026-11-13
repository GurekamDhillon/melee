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
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $d:expr)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(unreachable_code)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => { $( return $d; )? melee_core::unported!(concat!("TODO stub: ", stringify!($name))) }
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
        GENO_DEFINE_RESET_DESCRIPTORS GenoDefine_ResetDescriptors() = super::GenoDefine_ResetDescriptors();
        GENO_DEFINE_INIT_KINDS GenoDefine_InitKinds() = super::GenoDefine_InitKinds();
        GENO_DEFINE_LOAD GenoDefine_Load(kind: i32, base: i32) = super::GenoDefine_Load(kind, base);
        // `ftData_MotionStateList` (melee_link::retail_ms, which this crate cannot name): the common motion rows.
        GENO_COMMON_ROWS ftData_CommonRows() -> *mut MotionState;
        // `GenoGame_DefineApplyRows` (pc/geno/geno_define_rows.inc, the game half).
        GENO_DEFINE_APPLY_ROWS GenoGame_DefineApplyRows(kind: i32, common: *mut MotionState, specific: *mut MotionState, subactions: i32);
        // The define budgets line's figures: overlays / max, profile pool words, all pool words / max, slots / max,
        // states / max, arena bytes.
        GENO_DEFINE_BUDGETS Geno_DefineBudgets(kind: i32) -> [i32; 10] = [0; 10];
        GENO_DEFINE_CAPTURE_BASE GenoDefine_CaptureBase(kind: i32) = super::GenoDefine_CaptureBase(kind);
        // The Geno registry queries (pc/geno): unset = the default configuration, no Geno define registered.
        GENO_DEFINE_BASE_KIND Geno_DefineBaseKind(kind: i32) -> i32 = crate::platform::Geno_DefineBaseKind(kind);
        GENO_DEFINE_IS_NONE Geno_DefineIsNone(kind: i32) -> i32 = crate::platform::Geno_DefineIsNone(kind);
        GENO_DEFINE_TEMPLATE_SKELETON Geno_DefineTemplateSkeleton(kind: i32) -> i32 = crate::hooks::geno_define_template_skeleton(kind);
        GENO_DEFINE_COUNT Geno_DefineCount() -> i32 = 0;
        GENO_DEFINE_CK_AT Geno_DefineCKAt(i: i32) -> i32 = 0;
        GENO_DEFINE_COSTUME_COUNT Geno_DefineCostumeCount(kind: i32) -> i32 = 0;
        GENO_DEFINE_STRING Geno_DefineString(kind: i32, which: i32, costume: i32) -> *const c_char = core::ptr::null();
        GENO_DEFINE_PLAN_INT Geno_DefinePlanInt(kind: i32, what: i32, i: i32) -> i32 = if what == 0 { 0 } else { -1 };
        GENO_DEFINE_PLAN_TEXT Geno_DefinePlanText(kind: i32, which: i32, i: i32) -> *const c_char = core::ptr::null();
        GENO_DEFINE_PARTNER_KIND Geno_DefinePartnerKind(kind: i32) -> i32 = -1;
        GENO_DEFINE_IDENTITY_WORD Geno_DefineIdentityWord(kind: i32, word: i32) -> u32 = 0;
        PLAYER_MEX_SET_PARTNER Player_MexSetPartner(ckind: i32, partner_fk: i32) = crate::tu::player::Player_MexSetPartner(ckind, partner_fk);
        HSD_MEM_ALLOC HSD_MemAlloc(bytes: usize) -> *mut c_void = Box::leak(vec![0u8; bytes].into_boxed_slice()).as_mut_ptr() as *mut c_void;
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
pub static mut ftData_ThrownRows: [*mut Fighter_WaitAnimData; KMAX] = [core::ptr::null_mut(); KMAX];
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
pub use melee_runtime::shared::ftData_UnkBytePerCharacter;

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
    melee_core::ct!("install_retail_row");
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
pub fn order_bss() { melee_core::ct!("order_bss");}

/// `ft_8008521C`: self_vel = joint translation - cur_pos.
pub fn ft_8008521C(g: G) {
    melee_core::ct!("ft_8008521C");
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

// ---- pc/geno/geno_define_data.inc (included by ftdata.c): the Geno define tables the fighter engine reaches --------------

/// `GenoDefine_CommonRows` / `GenoDefine_SpecificRows`: a define's own motion-state rows, per kind.
pub static mut GenoDefine_CommonRows: [*mut MotionState; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut GenoDefine_SpecificRows: [*mut MotionState; KMAX] = [core::ptr::null_mut(); KMAX];
pub static mut GenoDefine_ContentIdentity: [[u32; 2]; KMAX] = [[0; 2]; KMAX];
pub static mut GenoDefine_InterpreterAttempts: [u32; KMAX] = [0; KMAX];
pub static mut GenoDefine_Costumes: [*mut UnkCostumeStruct; KMAX - MEX0 as usize] = [core::ptr::null_mut(); KMAX - MEX0 as usize];
pub static mut GenoDefine_CostumeStrings: [*mut Fighter_CostumeStrings; KMAX - MEX0 as usize] =
    [core::ptr::null_mut(); KMAX - MEX0 as usize];
/// `GENO_TMPL_COUNT`.
const GENO_TMPL_COUNT: usize = 8;
/// The pristine animation-row table of each template fighter, captured before an attach can touch it.
pub static mut GenoDefine_RowBaseline: [*mut Fighter_WaitAnimData; GENO_TMPL_COUNT] = [core::ptr::null_mut(); GENO_TMPL_COUNT];

/// `GenoDefine_TmplIndex`.
fn GenoDefine_TmplIndex(base: i32) -> usize {
    if base == Ft_Kind_Captain as i32 { 1 }
    else if base == Ft_Kind_Fox as i32 { 2 }
    else if base == Ft_Kind_Falco as i32 { 3 }
    else if base == Ft_Kind_Mars as i32 { 4 }
    else if base == Ft_Kind_Purin as i32 { 5 }
    else if base == Ft_Kind_Seak as i32 { 6 }
    else if base == Ft_Kind_Zelda as i32 { 7 }
    else { 0 }
}

/// `gd_data_copy`: a heap copy of a descriptor block (`HSD_MemAlloc` + `memcpy`).
unsafe fn gd_data_copy(source: *const c_void, bytes: usize) -> *mut c_void {
    let out = ext::HSD_MemAlloc(bytes);
    assert!(!out.is_null(), "Geno definition descriptor allocation exhausted");
    core::ptr::copy_nonoverlapping(source as *const u8, out as *mut u8, bytes);
    out
}

/// `GenoDefine_ResetDescriptors`.
pub fn GenoDefine_ResetDescriptors() {
    melee_core::ct!("GenoDefine_ResetDescriptors");
    unsafe {
        for t in 0..GENO_TMPL_COUNT {
            GenoDefine_RowBaseline[t] = core::ptr::null_mut();
        }
        GenoDefine_CommonRows = [core::ptr::null_mut(); KMAX];
        GenoDefine_SpecificRows = [core::ptr::null_mut(); KMAX];
    }
}

/// `GenoDefine_CaptureBase`: capture a template fighter's row table before the first attachment can change script pointers.
pub fn GenoDefine_CaptureBase(kind: i32) {
    melee_core::ct!("GenoDefine_CaptureBase");
    let t = GenoDefine_TmplIndex(kind);
    unsafe {
        if ext::Geno_DefineCount() != 0 && GenoDefine_RowBaseline[t].is_null() && !gFtDataList[kind as usize].is_null() {
            GenoDefine_RowBaseline[t] = gd_data_copy(
                (*gFtDataList[kind as usize]).xC as *const c_void,
                ftData_Table_Unk0[kind as usize].count as usize * core::mem::size_of::<Fighter_WaitAnimData>(),
            ) as *mut Fighter_WaitAnimData;
        }
    }
}

/// `GenoDefine_InitKinds`: give every registered define its kind's tables (inherited from its base kind).
pub fn GenoDefine_InitKinds() {
    melee_core::ct!("GenoDefine_InitKinds");
    let n = ext::Geno_DefineCount();
    for i in 0..n {
        let ck = ext::Geno_DefineCKAt(i);
        let fk = ck - 1;
        let base = ext::Geno_DefineBaseKind(fk);
        if base < 0 || fk < MEX0 || fk >= Ft_Kind_Max as i32 {
            continue;
        }
        let (f, b) = (fk as usize, base as usize);
        unsafe {
            macro_rules! inherit { ($($t:ident),*) => { $( $t[f] = $t[b]; )* } }
            inherit!(ftData_OnLoad, ftData_OnDeath, ftData_OnUserDataRemove, ftData_CharacterStateTables);
            if !GenoDefine_SpecificRows[f].is_null() {
                ftData_CharacterStateTables[f] = GenoDefine_SpecificRows[f];
            }
            inherit!(
                ftData_SpecialN, ftData_SpecialAirN, ftData_SpecialS, ftData_SpecialAirS, ftData_SpecialHi,
                ftData_SpecialAirHi, ftData_SpecialLw, ftData_SpecialAirLw, ftData_OnAbsorb, ftData_OnItemPickupExt,
                ftData_OnItemInvisible, ftData_OnItemVisible, ftData_OnItemDropExt, ftData_OnItemPickup,
                ftData_OnItemDrop, ftData_UnkMotionStates1, ftData_UnkMotionStates2, ftData_OnKnockbackEnter,
                ftData_OnKnockbackExit, ftData_UnkMotionStates3, ftData_UnkMotionStates4, ftKindCalcIndiviParamTable,
                ftData_UnkMtxFunc0, ftData_803C24EC, ftData_UnkDemoCallbacks0, ftData_UnkMotionStates0,
                ftData_803C1F40, ftData_803C23E4, ftData_803C2468, ftData_803C2360, ftData_UnkMotionStates5,
                ftData_UnkBytePerCharacter, ftData_UnkCallbackPairs0
            );
            let slot = (fk - MEX0) as usize;
            if ext::Geno_DefineIsNone(fk) != 0 {
                // Own files by name: no Pl*.dat file, the fighter's ftData is built natively in GenoDefine_Load.
                let nc = ext::Geno_DefineCostumeCount(fk);
                ftData_803C1F40[f].a = core::ptr::null_mut();
                ftData_803C1F40[f].b = core::ptr::null_mut();
                ftData_803C23E4[f] = ext::Geno_DefineString(fk, 0, 0) as *mut c_char;
                if GenoDefine_CostumeStrings[slot].is_null() {
                    GenoDefine_CostumeStrings[slot] =
                        ext::PcCostumeAlloc(nc as usize * core::mem::size_of::<Fighter_CostumeStrings>()) as *mut Fighter_CostumeStrings;
                }
                assert!(!GenoDefine_CostumeStrings[slot].is_null(), "costume strings exhausted");
                for c in 0..nc {
                    let cs = &mut *GenoDefine_CostumeStrings[slot].add(c as usize);
                    cs.dat_filename = ext::Geno_DefineString(fk, 2, c) as *mut c_char;
                    cs.joint_name = ext::Geno_DefineString(fk, 3, c) as *mut c_char;
                    cs.matanim_joint_name = ext::Geno_DefineString(fk, 4, c) as *mut c_char;
                }
                ftData_803C2360[f] = GenoDefine_CostumeStrings[slot];
                // `static int said; if (!said++) OSReport("geno: base none kind ...")`: one log line, no state.
            }
            // Do not invoke Mario's global init twice under a new id.
            ftData_Table_Unk1[f] = None;
            ftData_Table_Unk0[f].count = ftData_Table_Unk0[b].count;
            ftData_UnkIntPairs[f].count = ftData_UnkIntPairs[b].count;
            {
                let nc = if ext::Geno_DefineIsNone(fk) != 0 {
                    ext::Geno_DefineCostumeCount(fk)
                } else {
                    CostumeListsForeachCharacter[b].numCostumes as i32
                };
                if GenoDefine_Costumes[slot].is_null() {
                    GenoDefine_Costumes[slot] =
                        ext::PcCostumeAlloc(nc as usize * core::mem::size_of::<UnkCostumeStruct>()) as *mut UnkCostumeStruct;
                    assert!(!GenoDefine_Costumes[slot].is_null(), "costume runtime exhausted");
                }
                CostumeListsForeachCharacter[f].costume_list = GenoDefine_Costumes[slot];
            }
            CostumeListsForeachCharacter[f].numCostumes = if ext::Geno_DefineIsNone(fk) != 0 {
                ext::Geno_DefineCostumeCount(fk) as u8
            } else {
                CostumeListsForeachCharacter[b].numCostumes
            };
            ext::Player_MexSetMapping(ck, fk);
            // a Zelda or Sheik template define: the sub-fighter is its transform partner define, never retail Sheik / Zelda
            let pk = ext::Geno_DefinePartnerKind(fk);
            if pk >= 0 {
                ext::Player_MexSetPartner(ck, pk);
            }
            GenoDefine_ContentIdentity[f][0] = ext::Geno_DefineIdentityWord(fk, 0);
            GenoDefine_ContentIdentity[f][1] = ext::Geno_DefineIdentityWord(fk, 1);
        }
    }
}

/// `GenoDefine_SelfCount`: the template's own (non-common) motion-state count.
fn GenoDefine_SelfCount(base: i32) -> usize {
    (if base == Ft_Kind_Captain as i32 {
        ftCa_MS_SelfCount
    } else if base == Ft_Kind_Fox as i32 || base == Ft_Kind_Falco as i32 {
        ftFx_MS_SelfCount
    } else if base == Ft_Kind_Mars as i32 {
        ftMs_MS_SelfCount
    } else if base == Ft_Kind_Purin as i32 {
        ftPr_MS_SelfCount
    } else if base == Ft_Kind_Seak as i32 {
        ftSk_MS_SelfCount
    } else if base == Ft_Kind_Zelda as i32 {
        ftZd_MS_SelfCount
    } else {
        ftMr_MS_SelfCount
    }) as usize
}

/// `GenoDefine_ExtAttrBytes`: the size of the template's special-attribute block.
fn GenoDefine_ExtAttrBytes(base: i32) -> usize {
    if base == Ft_Kind_Captain as i32 {
        core::mem::size_of::<ftCaptain_DatAttrs>()
    } else if base == Ft_Kind_Fox as i32 || base == Ft_Kind_Falco as i32 {
        core::mem::size_of::<ftFox_DatAttrs>()
    } else if base == Ft_Kind_Zelda as i32 {
        core::mem::size_of::<ftZelda_DatAttrs>()
    } else if base == Ft_Kind_Mars as i32 || base == Ft_Kind_Purin as i32 || base == Ft_Kind_Seak as i32 {
        // MarsAttributes / ftPurinAttributes / ftSeakAttributes: only a base "none" define's templates (not hosted)
        melee_core::unported!("GenoDefine_ExtAttrBytes: a Marth / Jigglypuff / Sheik template (base \"none\" defines)")
    } else {
        core::mem::size_of::<ftMario_DatAttrs>()
    }
}

/// `GenoDefine_Load`: a define fighter's own ftData, copied from its template's (attributes, special attributes, the
/// pristine animation rows and the demo rows, each row's authoring kind rewritten to the define's), and its own common
/// and specific motion-state rows with the define's common_states applied. A base "none" define builds its model,
/// bank, parts and joint fields from its package (GenoDefine_BuildOwn): not hosted in this engine.
pub fn GenoDefine_Load(kind: i32, base: i32) {
    melee_core::ct!("GenoDefine_Load");
    let (k, b) = (kind as usize, base as usize);
    unsafe {
        if !gFtDataList[k].is_null() {
            return;
        }
        let source = gFtDataList[b];
        let n = ftData_Table_Unk0[b].count as usize;
        let demo = ftData_UnkIntPairs[b].count as usize;
        let row_sz = core::mem::size_of::<Fighter_WaitAnimData>();
        let out = gd_data_copy(source as *const c_void, core::mem::size_of::<ftData>()) as *mut ftData;
        (*out).x0 = gd_data_copy((*source).x0 as *const c_void, core::mem::size_of::<ftCo_DatAttrs>()) as *mut ftCo_DatAttrs;
        (*out).ext_attr = gd_data_copy((*source).ext_attr as *const c_void, GenoDefine_ExtAttrBytes(base)) as *mut ftData_ExtAttr;
        let baseline = GenoDefine_RowBaseline[GenoDefine_TmplIndex(base)];
        assert!(!baseline.is_null(), "Geno native template preset baseline unavailable");
        (*out).xC = gd_data_copy(baseline as *const c_void, n * row_sz) as *mut Fighter_WaitAnimData;
        (*out).x14 = gd_data_copy((*source).x14 as *const c_void, demo * row_sz) as *mut Fighter_WaitAnimData;
        for i in 0..n + demo {
            let row = if i < n { &mut *(*out).xC.add(i) } else { &mut *(*out).x14.add(i - n) };
            let flags = row.x10_animCurrFlags;
            // Paired thrown clips keep the generic victim skeleton author id.
            if flags & 0x3f != 0x21 {
                row.x10_animCurrFlags = (flags & !0x3f) | if kind < 64 { kind } else { FT_ANIM_KIND_SELF };
            }
        }
        let none = ext::Geno_DefineIsNone(kind) != 0;
        let tskel = ext::Geno_DefineTemplateSkeleton(kind) != 0;
        if none {
            GenoDefine_BuildOwn(kind, out, n);
            if tskel {
                // template "captain": checked, not trusted: the package's parts table must be the retail table, entry for entry
                let own = GenoDefine_PartsTable(kind);
                let pt = crate::hooks::FT_PARTS_TABLE;
                let tpl = if pt.is_null() { core::ptr::null_mut() } else { *pt.add(b) };
                assert!(
                    !own.is_null() && !tpl.is_null() && (*own).parts_num == (*tpl).parts_num
                        && core::slice::from_raw_parts((*own).joint_to_part, (*own).parts_num as usize)
                            == core::slice::from_raw_parts((*tpl).joint_to_part, (*own).parts_num as usize),
                    "Geno define.template: the package's joint tree is not the template's (joint count or parts table differ)"
                );
            } else {
                // the donor's demo rows name figatrees of the donor's skeleton: each becomes a copy of the own Wait row
                let wr = (*ext::ftData_CommonRows().add(ftCo_MS_Wait as usize)).anim_id;
                if wr >= 0 && (wr as usize) < n {
                    for i in 0..demo {
                        *(*out).x14.add(i) = core::ptr::read((*out).xC.add(wr as usize));
                        (*(*out).x14.add(i)).x14 = 0;
                    }
                    report(format!("geno: base none kind {kind}: {demo} demo (results) rows pose with the own Wait clip, not the donor's figatrees\n"));
                }
            }
            GenoDefine_OwnTables(kind, out);
            GenoDefine_Census(kind, out, source);
        }
        let nself = GenoDefine_SelfCount(base);
        let ms = core::mem::size_of::<MotionState>();
        GenoDefine_CommonRows[k] = gd_data_copy(ext::ftData_CommonRows() as *const c_void, ms * ftCo_MS_Count as usize) as *mut MotionState;
        GenoDefine_SpecificRows[k] = gd_data_copy(ftData_CharacterStateTables[b] as *const c_void, ms * nself) as *mut MotionState;
        if none {
            let slot = (kind - MEX0) as usize;
            for m in 0..(ftCo_MS_Count as usize + nself).min(512) {
                let msp = if m < ftCo_MS_Count as usize { GenoDefine_CommonRows[k].add(m) } else { GenoDefine_SpecificRows[k].add(m - ftCo_MS_Count as usize) };
                if GenoDefine_RowRemap[slot][m] >= 0 {
                    (*msp).anim_id = GenoDefine_RowRemap[slot][m] as _;
                }
            }
        }
        ext::GenoGame_DefineApplyRows(kind, GenoDefine_CommonRows[k], GenoDefine_SpecificRows[k], if none { ftData_Table_Unk0[k].count as i32 } else { n as i32 });
        ftData_CharacterStateTables[k] = GenoDefine_SpecificRows[k];
        gFtDataList[k] = out;
        let what = if !none {
            "retail:mario"
        } else if tskel {
            "base none, template (own model, bank, parts, joint fields, scripts, attributes; the retail fighter's native code and tables underneath)"
        } else {
            "base none (own model, bank, parts, joint fields; Mario's script words, callbacks and attribute defaults are still the template)"
        };
        report(format!(
            "geno: native define kind {kind} loaded: {what}; independent common/special rows; interpreter attempts {}\n",
            GenoDefine_InterpreterAttempts[k]
        ));
        let bud = ext::Geno_DefineBudgets(kind);
        report(format!(
            "geno: define budgets: kind {kind} overlays {}/{}, pool {} words (all profiles {}/{}), slots {}/{}, states {}/{}, arena {}/{}\n",
            bud[0], bud[1], bud[2], bud[3], bud[4], bud[5], bud[6], bud[7], bud[8], bud[9] as u32, 2u32 * 1024 * 1024
        ));
    }
}

// ---- slice 4 / 8: a base "none" define's own model data (GenoDefine_BuildOwn and its helpers) ----
const NSLOT: usize = KMAX - MEX0 as usize;
static mut GenoDefine_Parts: [FighterPartsTable; NSLOT] =
    [const { FighterPartsTable { joint_to_part: core::ptr::null_mut(), part_to_joint: core::ptr::null_mut(), parts_num: 0 } }; NSLOT];
static mut GenoDefine_PartsJ2P: [[u8; 128]; NSLOT] = [[0; 128]; NSLOT];
static mut GenoDefine_PartsP2J: [[u8; 56]; NSLOT] = [[0; 56]; NSLOT];
/// `GENO_OWN_EXTRA_ROWS`.
const GENO_OWN_EXTRA_ROWS: usize = 96;
/// motion -> its own animation row (>= n), -1: keeps the donor's number.
static mut GenoDefine_RowRemap: [[i16; 512]; NSLOT] = [[-1; 512]; NSLOT];
/// no visibility groups: every row NULL, model_num 0.
static mut GenoDefine_VisTable: [[*mut c_void; 4]; 255] = [[core::ptr::null_mut(); 4]; 255];
static mut GenoDefine_TobjZero: u16 = 0;
static mut GenoDefine_TobjTable: [*mut u16; 255] = [core::ptr::null_mut(); 255];
static mut GenoDefine_GuardTrees: [*mut HSD_Joint; 3] = [core::ptr::null_mut(); 3];
static mut GenoDefine_X20: ftData_x20 = ftData_x20 { x0: core::ptr::null_mut(), x8: 0.0 };
static mut GenoDefine_CensusMask: [u32; NSLOT] = [0; NSLOT];

/// `Geno_DefineCensusMask`.
pub fn Geno_DefineCensusMask(kind: i32) -> u32 {
    if kind >= MEX0 && kind < KMAX as i32 { unsafe { GenoDefine_CensusMask[(kind - MEX0) as usize] } } else { 0 }
}

/// `GenoDefine_PartsTable`: the fighter's parts table from its plan; NULL for a define that is not base "none" or
/// whose plan is unusable. Built once per kind, static storage.
pub fn GenoDefine_PartsTable(kind: i32) -> *mut FighterPartsTable {
    melee_core::ct!("GenoDefine_PartsTable");
    let slot = kind - MEX0;
    if slot < 0 || slot as usize >= NSLOT || ext::Geno_DefineIsNone(kind) == 0 {
        return core::ptr::null_mut();
    }
    if ext::Geno_DefinePlanInt(kind, 0, 0) == 0 {
        return core::ptr::null_mut();
    }
    let s = slot as usize;
    unsafe {
        if GenoDefine_Parts[s].parts_num == 0 {
            let nj = ext::Geno_DefinePlanInt(kind, 1, 0) as usize;
            let np = ext::Geno_DefinePlanInt(kind, 2, 0) as usize;
            GenoDefine_PartsJ2P[s] = [255; 128];
            GenoDefine_PartsP2J[s] = [255; 56];
            // the C memcpys nj bytes into a 128-byte table; capped here
            core::ptr::copy_nonoverlapping(ext::Geno_DefinePlanText(kind, 2, 0) as *const u8, GenoDefine_PartsJ2P[s].as_mut_ptr(), nj.min(128));
            core::ptr::copy_nonoverlapping(ext::Geno_DefinePlanText(kind, 3, 0) as *const u8, GenoDefine_PartsP2J[s].as_mut_ptr(), np.min(56));
            GenoDefine_Parts[s].joint_to_part = GenoDefine_PartsJ2P[s].as_mut_ptr();
            GenoDefine_Parts[s].part_to_joint = GenoDefine_PartsP2J[s].as_mut_ptr();
            GenoDefine_Parts[s].parts_num = nj as u32;
        }
        &raw mut GenoDefine_Parts[s]
    }
}

/// `GenoDefine_JointFields`: the ftData fields that name joints take the package's (plan "ftdata"), by role.
unsafe fn GenoDefine_JointFields(kind: i32, out: *mut ftData, x8: *mut ftData_x8) {
    let pi = |w: i32, i: i32| ext::Geno_DefinePlanInt(kind, w, i);
    let o = &mut *out;
    let x34 = gd_data_copy(o.x34 as *const c_void, core::mem::size_of::<ftData_x34>()) as *mut ftData_x34;
    let x38 = gd_data_copy(o.x38 as *const c_void, 2 * core::mem::size_of::<AbsorbDesc>()) as *mut AbsorbDesc;
    let x44 = gd_data_copy(o.x44 as *const c_void, core::mem::size_of::<ftData_x44_t>()) as *mut ftData_x44_t;
    let x54 = gd_data_copy(o.x54 as *const c_void, 5 * 4) as *mut i32;
    let x58 = gd_data_copy(o.x58 as *const c_void, core::mem::size_of::<ftData_x58_t>()) as *mut ftData_x58_t;
    let b = &raw mut (*x8).x10;
    for i in 0..5 {
        *b.add(i) = pi(20, i as i32) as u8;
    }
    (*x34).x0 = pi(21, 0) as _;
    for i in 0..2 {
        (*x38.add(i)).x0_bone_id = pi(22, i as i32);
    }
    let x = &mut *x44;
    x.unk0 = pi(23, 0) as i16;
    x.unk2 = pi(23, 1) as i16;
    x.unk4 = pi(23, 2) as i16;
    x.unk6 = pi(23, 3) as i16;
    x.unk8 = pi(23, 4) as i16;
    x.unkA = pi(23, 5) as i16;
    for i in 0..5 {
        *x54.add(i) = pi(24, i as i32);
    }
    // IK pairs: [0],[1] thigh R,L; [2],[3] knee; [4],[5] foot
    (*x58).x0 = pi(25, 0) as u8;
    (*x58).x1 = pi(25, 1) as u8;
    (*x58).x8 = pi(25, 2) as u8;
    (*x58).x9 = pi(25, 3) as u8;
    (*x58).x10 = pi(25, 4) as u8;
    (*x58).x11 = pi(25, 5) as u8;
    o.x34 = x34;
    o.x38 = x38;
    o.x44 = x44;
    o.x54 = x54;
    o.x58 = x58;
    let n = pi(5, 0);
    let x30 = gd_data_copy(o.x30 as *const c_void, core::mem::size_of::<ftData_x30>()) as *mut ftData_x30;
    let inits = gd_data_copy((*o.x30).inits as *const c_void, n as usize * core::mem::size_of::<ftHurtboxInit>()) as *mut ftHurtboxInit;
    let f = |i: i32| f32::from_bits(pi(40, i) as u32);
    for i in 0..n {
        let h = &mut *inits.add(i as usize);
        h.bone_idx = pi(30, i) as _;
        h.height = pi(31, i) as _;
        h.is_grabbable = pi(32, i) as u32;
        h.a_offset.x = f(i * 8);
        h.a_offset.y = f(i * 8 + 1);
        h.a_offset.z = f(i * 8 + 2);
        h.b_offset.x = f(i * 8 + 3);
        h.b_offset.y = f(i * 8 + 4);
        h.b_offset.z = f(i * 8 + 5);
        h.scale = f(i * 8 + 6);
    }
    (*x30).count = n;
    (*x30).inits = inits;
    o.x30 = x30;
    report(format!(
        "geno: base none kind {kind}: ftData joint fields from the plan: ECB joints {},{},{},{},{},{}; {n} hurtboxes\n",
        x.unk0, x.unk2, x.unk4, x.unk6, x.unk8, x.unkA
    ));
}

/// `GenoDefine_BuildOwn`: a base "none" define's own model data in its ftData (part-visibility and texture-animation
/// stand-ins, joint fields, and the animation rows: Mario's script words with the package's clips).
unsafe fn GenoDefine_BuildOwn(kind: i32, out: *mut ftData, n: usize) {
    let pi = |w: i32, i: i32| ext::Geno_DefinePlanInt(kind, w, i);
    let pt = |w: i32, i: i32| ext::Geno_DefinePlanText(kind, w, i) as *mut i8;
    let tbase = ext::Geno_DefineBaseKind(kind);
    let tself = GenoDefine_SelfCount(tbase);
    let tspec = ftData_CharacterStateTables[tbase as usize];
    let common = ext::ftData_CommonRows();
    let nmotions = pi(4, 0).max(0) as usize;
    let njoints = pi(1, 0);
    let nown = pi(6, 0).max(0) as usize;
    let nco = ftCo_MS_Count as usize;
    assert!(pi(0, 0) != 0, "Geno base none: the fighter plan is unusable (see geno: plan lines in the log)");
    assert!(n <= 512, "Geno base none: animation row table too large");
    let o = &mut *out;
    let x8 = gd_data_copy(o.x8 as *const c_void, core::mem::size_of::<ftData_x8>()) as *mut ftData_x8;
    if ext::Geno_DefineTemplateSkeleton(kind) == 0 {
        (*x8).x0.model_num = 0;
        (*x8).x0.vis_table = (&raw mut GenoDefine_VisTable) as *mut [*mut FtPartsVisLookup; 4];
        (*x8).x8.x8 = 0;
        for i in 0..255 {
            GenoDefine_TobjTable[i] = &raw mut GenoDefine_TobjZero;
        }
        (*x8).x8.xC = (&raw mut GenoDefine_TobjTable) as *mut *mut u16;
    }
    o.x8 = x8;
    GenoDefine_JointFields(kind, out, x8);
    let o = &mut *out;
    let mut pick = [-1i16; 512];
    let mut rank = [0u8; 512];
    let (mut conflicts, mut mapped) = (0, 0);
    let slot = (kind - MEX0) as usize;
    let mut ex_a: std::vec::Vec<i16> = std::vec::Vec::new();
    let mut ex_c: std::vec::Vec<i16> = std::vec::Vec::new();
    GenoDefine_RowRemap[slot] = [-1; 512];
    {
        // The Wait state claims its row first (ftCo_8008A7A8 re-picks the idle clip from the wait table BY ROW).
        let ws = &*common.add(ftCo_MS_Wait as usize);
        let wc = pi(13, ftCo_MS_Wait as i32);
        if ws.anim_id >= 0 && (ws.anim_id as usize) < n && wc >= 0 {
            pick[ws.anim_id as usize] = wc as i16;
            rank[ws.anim_id as usize] = 1;
        }
    }
    let mut m = 0;
    while m < nmotions && m < nco + tself {
        let st = if m < nco { &*common.add(m) } else { &*tspec.add(m - nco) };
        let a = st.anim_id;
        let c = pi(13, m as i32);
        if a < 0 || a as usize >= n || c < 0 {
            m += 1;
            continue;
        }
        let a = a as usize;
        if rank[a] < 1 {
            pick[a] = c as i16;
            rank[a] = 1;
        } else if pick[a] != c as i16 {
            // two states share this row but the package gives them different clips: the later one gets a row of its own
            let i = match (0..ex_a.len()).find(|&i| ex_a[i] == a as i16 && ex_c[i] == c as i16) {
                Some(i) => i,
                None => {
                    assert!(ex_a.len() < GENO_OWN_EXTRA_ROWS, "Geno base none: too many own animation rows");
                    ex_a.push(a as i16);
                    ex_c.push(c as i16);
                    ex_a.len() - 1
                }
            };
            GenoDefine_RowRemap[slot][m] = (n + nown + i) as i16;
            conflicts += 1;
        }
        m += 1;
    }
    {
        // a row no state points at is still played BY NUMBER: its ftCo_SM_ clip (plan row_clips), else the Wait clip
        let wc = pi(13, ftCo_MS_Wait as i32);
        let mut filled = 0;
        for i in 0..n {
            if pick[i] >= 0 {
                continue;
            }
            let rc = pi(14, i as i32);
            pick[i] = (if rc >= 0 { rc } else { wc }) as i16;
            if pick[i] >= 0 {
                filled += 1;
            }
        }
        report(format!("geno: base none kind {kind}: {filled} rows no state names take their ftCo_SM_ clip or Wait\n"));
    }
    let extras = ex_a.len();
    let total = n + nown + extras;
    let rsz = core::mem::size_of::<Fighter_WaitAnimData>();
    let rows2 = ext::HSD_MemAlloc(total * rsz) as *mut Fighter_WaitAnimData;
    let blend2 = ext::HSD_MemAlloc(total * 2) as *mut [u8; 2];
    assert!(!rows2.is_null() && !blend2.is_null(), "Geno definition descriptor allocation exhausted");
    core::ptr::copy_nonoverlapping(o.xC as *const u8, rows2 as *mut u8, n * rsz);
    core::ptr::copy_nonoverlapping(o.x10 as *const u8, blend2 as *mut u8, n * 2);
    let row = |r: usize| &mut *rows2.add(r);
    let bl = |r: usize| &mut *blend2.add(r);
    {
        // slice 8: the package's declared rows (fighter.rows), n..: a copy of the Wait row given the declared clip
        let wr = (*common.add(ftCo_MS_Wait as usize)).anim_id;
        let wc2 = pi(13, ftCo_MS_Wait as i32);
        assert!(wr >= 0 && (wr as usize) < n, "Geno base none: no Wait row to copy for the declared rows");
        let wr = wr as usize;
        for i in 0..nown {
            let mut cc = pi(15, i as i32);
            if cc < 0 {
                cc = wc2;
            }
            let r = n + i;
            *row(r) = core::ptr::read(row(wr));
            *bl(r) = *bl(wr);
            row(r).x0 = pt(1, cc);
            row(r).x4 = pi(10, cc);
            row(r).x8 = pi(11, cc);
            let b = pi(18, r as i32);
            if b >= 0 {
                *bl(r) = [(b & 255) as u8, (b >> 8) as u8];
            }
            if pi(17, r as i32) != 0 {
                row(r).x10_animCurrFlags = (row(r).x10_animCurrFlags & 0x3f) | pi(16, r as i32);
            }
            mapped += 1;
        }
    }
    for i in 0..extras {
        let r = n + nown + i;
        let src = ex_a[i] as usize;
        *row(r) = core::ptr::read(row(src));
        *bl(r) = *bl(src);
        row(r).x0 = pt(1, ex_c[i] as i32);
        row(r).x4 = pi(10, ex_c[i] as i32);
        row(r).x8 = pi(11, ex_c[i] as i32);
        mapped += 1;
    }
    {
        // a row the package gives no flags loses the donor's part-animation bits (x594_b3 / b4)
        let mut cleared = 0;
        for r in 0..total {
            if pi(17, r as i32) == 0 && (row(r).x10_animCurrFlags as u32 & 0x1800_0000) != 0 {
                row(r).x10_animCurrFlags = (row(r).x10_animCurrFlags as u32 & !0x1800_0000u32) as i32;
                cleared += 1;
            }
        }
        if cleared != 0 {
            report(format!("geno: base none kind {kind}: {cleared} donor rows lose their part-animation flags\n"));
        }
    }
    o.xC = rows2;
    o.x10 = blend2;
    ftData_Table_Unk0[kind as usize].count = total as _;
    for i in 0..n {
        let r = row(i);
        if pick[i] < 0 {
            r.x8 = 0;
            r.x4 = 0;
            continue;
        }
        let p = pick[i] as i32;
        r.x0 = pt(1, p);
        r.x4 = pi(10, p);
        r.x8 = pi(11, p);
        if pi(17, i as i32) != 0 && (r.x10_animCurrFlags & 0x3f) != 0x21 {
            r.x10_animCurrFlags = (r.x10_animCurrFlags & 0x3f) | pi(16, i as i32);
        }
        let b = pi(18, i as i32);
        if b >= 0 {
            *bl(i) = [(b & 255) as u8, (b >> 8) as u8];
        }
        mapped += 1;
    }
    report(format!(
        "geno: base none kind {kind}: {mapped} of {n} animation rows take the package's clips ({conflicts} states given a row of their own, {total} rows in all); own parts table of {njoints} joints\n"
    ));
}

/// `GenoDefine_OwnTables`: a base "none" define's own storage for the small ftData tables the donor's copy shares.
unsafe fn GenoDefine_OwnTables(kind: i32, out: *mut ftData) {
    let o = &mut *out;
    if ext::Geno_DefineTemplateSkeleton(kind) == 0 {
        GenoDefine_X20.x0 = (&raw mut GenoDefine_GuardTrees) as *mut HSD_Joint; // the pose-tree table read through `x0->child` (native word 2)
        GenoDefine_X20.x8 = if o.x20.is_null() { 0.0 } else { (*o.x20).x8 };
        o.x20 = &raw mut GenoDefine_X20;
    }
    o.x3C = gd_data_copy(o.x3C as *const c_void, core::mem::size_of::<UnkFloat6_Camera>()) as _;
    o.x40 = gd_data_copy(o.x40 as *const c_void, core::mem::size_of::<itPickup>()) as _;
    o.x50 = gd_data_copy(o.x50 as *const c_void, core::mem::size_of::<Vec2>()) as _;
    o.x4C_sfx = gd_data_copy(o.x4C_sfx as *const c_void, core::mem::size_of::<FtSFX>()) as _;
}

/// `GenoDefine_Census`: log which ftData pointer fields are still the donor's.
unsafe fn GenoDefine_Census(kind: i32, own: *const ftData, donor: *const ftData) {
    let mask = GenoDefine_FtDataSharedMask(own, donor);
    GenoDefine_CensusMask[(kind - MEX0) as usize] = mask;
    let list: String = (0..GENO_FTDATA_FIELDS.len()).filter(|i| mask & (1 << i) != 0).map(|i| format!(" {}", GENO_FTDATA_FIELDS[i])).collect();
    report(format!(
        "geno: census kind {kind}: ftData fields still the donor's pointers:{} ({})\n",
        if mask != 0 { list.as_str() } else { " none" },
        if mask != 0 { "see docs/geno.md 22.4 for what each is and why" } else { "independent" }
    ));
}

/// The ftData pointer fields, by name, in struct order (the census; docs/geno.md 22.4 "independence").
const GENO_FTDATA_FIELDS: [&str; 24] = [
    "x0", "ext_attr", "x8", "xC", "x10", "x14", "x18", "x1C", "x20", "x24", "x28", "x2C", "x30", "x34", "x38", "x3C", "x40",
    "x44", "x48", "x4C", "x50", "x54", "x58", "x5C",
];

/// `GenoDefine_FtDataSharedMask`: which ftData pointer fields of a define are the very (non-null) pointers of the
/// donor's (bit i = field i). Pure: the independence test runs it on synthetic structs.
pub fn GenoDefine_FtDataSharedMask(own: *const ftData, donor: *const ftData) -> u32 {
    let mut mask = 0u32;
    for i in 0..GENO_FTDATA_FIELDS.len() {
        unsafe {
            let a = *(own as *const *const c_void).add(i);
            let b = *(donor as *const *const c_void).add(i);
            if a == b && !a.is_null() {
                mask |= <u32 as melee_core::ppc::Ppc>::pshl(1, (i) as i64);
            }
        }
    }
    mask
}
/// `GenoDefine_FtDataFieldName`.
pub fn GenoDefine_FtDataFieldName(i: i32) -> &'static str {
    GENO_FTDATA_FIELDS.get(i as usize).copied().filter(|_| i >= 0).unwrap_or("?")
}

/// `GenoGame_DefinePPCViolation`: called only from the interpreter entry probe; counts attempted guest execution.
pub fn GenoGame_DefinePPCViolation(kind: i32) {
    if (0..Ft_Kind_Max as i32).contains(&kind) {
        unsafe { GenoDefine_InterpreterAttempts[kind as usize] += 1 };
    }
}

/// `GenoDefine_RestPoseTree`: a base "none" define's rest-pose joint tree (the guard's pose blend), NULL for any other fighter.
pub fn GenoDefine_RestPoseTree(fp: &mut Fighter) -> *mut HSD_Joint {
    melee_core::ct!("GenoDefine_RestPoseTree");
    if ext::Geno_DefineIsNone(fp.kind as i32) == 0 || ext::Geno_DefineTemplateSkeleton(fp.kind as i32) != 0 {
        return core::ptr::null_mut();
    }
    // the guard walks it from FtPart_TransN, so it is the tree below the root joint
    unsafe {
        (*(*CostumeListsForeachCharacter[fp.kind as usize].costume_list.add(fp.costume_id as usize)).joint).child
    }
}

/// `GenoDefine_BindFighter`: a define's fighter takes the define's own motion-state rows.
pub fn GenoDefine_BindFighter(fp: &mut Fighter) {
    melee_core::ct!("GenoDefine_BindFighter");
    if ext::Geno_DefineBaseKind(fp.kind as i32) < 0 {
        return;
    }
    unsafe {
        fp.x1C_actionStateList = GenoDefine_CommonRows[fp.kind as usize];
        fp.x20_actionStateList = GenoDefine_SpecificRows[fp.kind as usize];
        report(format!("geno: native define kind {} spawn: interpreter attempts {}\n", fp.kind, GenoDefine_InterpreterAttempts[fp.kind as usize]));
    }
}

/// `ft_800852B0`: scene reset of every loaded-file pointer.
pub fn ft_800852B0() {
    melee_core::ct!("ft_800852B0");
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
    melee_core::ct!("ft_8008549C");
    unsafe { ft_8045996C = [0; KMAX] };
}

/// `ftData_MexMotionRowOk`: a real motion row of an m-ex file (name NULL or in the archive,
/// script in the archive).
pub fn ftData_MexMotionRowOk(row: &Fighter_WaitAnimData, lo: *const u8, hi: *const u8) -> bool {
    melee_core::ct!("ftData_MexMotionRowOk");
    let name = row.x0 as *const u8;
    let cmd = row.xC as *const u8;
    (name.is_null() || (name >= lo && name < hi)) && !cmd.is_null() && cmd >= lo && cmd < hi
}

/// `ftData_MexAnimFlags`: the authoring-kind bits an m-ex motion gets.
pub fn ftData_MexAnimFlags(flags: u32, kind: i32) -> u32 {
    melee_core::ct!("ftData_MexAnimFlags");
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
    melee_core::ct!("ftData_PcFixtureAt");
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
    melee_core::ct!("ftData_PcRowString");
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
    melee_core::ct!("ftData_PcInstallCostumes");
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
    melee_core::ct!("ftData_PcInstallRows");
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


/// The value `ftData_MexInitKinds` would install from m-ex `fighter_function` slot `idx` for retail kind `fk`
/// (0..=26): the native table entry as a pointer. The host's m-ex bridge maps a vanilla guest address that MxDt.dat gives
/// a retail row to the native function of that row (gw_mex_bridge_lookup's role). Null for a slot the port does not fill.
pub fn ftData_MexRetailSlot(idx: i32, fk: usize) -> *mut c_void {
    if fk >= KMAX {
        return core::ptr::null_mut();
    }
    // SAFETY: every table element is a (nullable) pointer-sized fn or data pointer.
    unsafe {
        match idx {
            0 => core::mem::transmute_copy(&ftData_OnLoad[fk]),
            1 => core::mem::transmute_copy(&ftData_OnDeath[fk]),
            2 => core::mem::transmute_copy(&ftData_OnUserDataRemove[fk]),
            3 => core::mem::transmute_copy(&ftData_CharacterStateTables[fk]),
            4 => core::mem::transmute_copy(&ftData_SpecialN[fk]),
            5 => core::mem::transmute_copy(&ftData_SpecialAirN[fk]),
            6 => core::mem::transmute_copy(&ftData_SpecialS[fk]),
            7 => core::mem::transmute_copy(&ftData_SpecialAirS[fk]),
            8 => core::mem::transmute_copy(&ftData_SpecialHi[fk]),
            9 => core::mem::transmute_copy(&ftData_SpecialAirHi[fk]),
            10 => core::mem::transmute_copy(&ftData_SpecialLw[fk]),
            11 => core::mem::transmute_copy(&ftData_SpecialAirLw[fk]),
            12 => core::mem::transmute_copy(&ftData_OnAbsorb[fk]),
            13 => core::mem::transmute_copy(&ftData_OnItemPickupExt[fk]),
            14 => core::mem::transmute_copy(&ftData_OnItemInvisible[fk]),
            15 => core::mem::transmute_copy(&ftData_OnItemVisible[fk]),
            16 => core::mem::transmute_copy(&ftData_OnItemDropExt[fk]),
            17 => core::mem::transmute_copy(&ftData_OnItemPickup[fk]),
            18 => core::mem::transmute_copy(&ftData_OnItemDrop[fk]),
            19 => core::mem::transmute_copy(&ftData_UnkMotionStates1[fk]),
            20 => core::mem::transmute_copy(&ftData_UnkMotionStates2[fk]),
            21 => core::mem::transmute_copy(&ftData_OnKnockbackEnter[fk]),
            22 => core::mem::transmute_copy(&ftData_OnKnockbackExit[fk]),
            23 => core::mem::transmute_copy(&ftData_UnkMotionStates3[fk]),
            24 => core::mem::transmute_copy(&ftData_UnkMotionStates4[fk]),
            25 => core::mem::transmute_copy(&ftKindCalcIndiviParamTable[fk]),
            26 => core::mem::transmute_copy(&ftData_UnkMtxFunc0[fk]),
            38 => core::mem::transmute_copy(&ftData_803C24EC[fk]),
            39 => core::mem::transmute_copy(&ftData_UnkDemoCallbacks0[fk]),
            40 => core::mem::transmute_copy(&ftData_UnkMotionStates0[fk]),
            _ => core::ptr::null_mut(),
        }
    }
}

/// Per m-ex kind: the class of each main / demo motion row of its Pl file (1 real, 2 hole), found on the raw image by
/// the archive decoder (ftData_MexMotionRowOk's walk); the list stops at the first row that is neither.
pub static mut MEX_MOTION_ROWS: std::collections::BTreeMap<usize, (std::vec::Vec<u8>, std::vec::Vec<u8>)> = std::collections::BTreeMap::new();

/// The kind whose Pl file `ftData_8008572C` is loading (-1 otherwise): an m-ex fighter's file may name its data with
/// its clone base's symbol (Halberd's PlBm.dat has `ftDataKirby`), so the archive decoder cannot tell the kind by name.
pub static mut FTDATA_LOADING_KIND: i32 = -1;

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
    melee_core::ct!("ftData_MexInitKinds");
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
    melee_core::ct!("ftData_PcRetailKindOfOnLoad");
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
    melee_core::ct!("ftData_IsMexKind");
    ext::Geno_DefineBaseKind(kind) < 0 && kind >= MEX0 && kind < Ft_Kind_Max as i32
        && unsafe { !ftData_803C1F40[kind as usize].a.is_null() }
}

/// `ftData_80085560`: adjust a kind's reference count; negative is fatal.
pub fn ftData_80085560(idx: i32, increment: i32) {
    melee_core::ct!("ftData_80085560");
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
    melee_core::ct!("ftData_800855C8");
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
    let words = <usize as melee_core::ppc::Ppc>::pdiv(core::mem::size_of::<ftData>(), core::mem::size_of::<*mut u8>());
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
    melee_core::ct!("ftData_8008572C");
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
            FTDATA_LOADING_KIND = kind;
            ext::lbArchive_80017040_ftdata(&mut ar, ftData_803C1F40[k].a, &mut gFtDataList[k], ftData_803C1F40[k].b);
            FTDATA_LOADING_KIND = -1;
            if ftData_IsMexKind(kind) {
                report(format!("gw: ftData_8008572C kind={kind} file={} sym={}\n",
                               cstr(ftData_803C1F40[k].a), cstr(ftData_803C1F40[k].b)));
                let fd = gFtDataList[k];
                // the row classes the decoder found on the raw image (dat_custom mex_motion_rows: the C's
                // ftData_MexMotionRowOk bounds are archive addresses, which native rows do not have)
                let (main, demo) = MEX_MOTION_ROWS.get(&k).cloned().unwrap_or_default();
                {
                    let cnt = ftData_Table_Unk0[k].count;
                    let (mut n_main, mut n_holes) = (0, 0);
                    for (i, &c) in main.iter().enumerate() {
                        if c == 2 {
                            n_holes += 1;
                            continue;
                        }
                        let row = (*fd).xC.add(i);
                        (*row).x10_animCurrFlags = ftData_MexAnimFlags((*row).x10_animCurrFlags as u32, kind) as i32;
                        n_main += 1;
                    }
                    if n_main + n_holes != cnt {
                        report(format!("gw: kind {kind}: motion table ends after {} entries (the base has {cnt}); the rest keep their authored flags
", n_main + n_holes));
                    }
                    report(format!("gw: kind {kind}: {n_main} motion rows rewritten, {n_holes} empty rows skipped
"));
                }
                if !(*fd).x14.is_null() {
                    let cnt = ftData_UnkIntPairs[k].count;
                    let n_demo = demo.len() as i32;
                    for i in 0..n_demo as usize {
                        let row = (*fd).x14.add(i);
                        (*row).x10_animCurrFlags = ftData_MexAnimFlags((*row).x10_animCurrFlags as u32, kind) as i32;
                    }
                    if n_demo != cnt {
                        report(format!("gw: kind {kind} has {n_demo} demo motions, not the base's {cnt}
"));
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
    melee_core::ct!("ftData_8008578C");
    let n = unsafe { CostumeListsForeachCharacter[Ft_Kind_Kirby as usize].numCostumes };
    if color != 0xFF && color >= n {
        color = 0;
    }
    ext::ftKb_SpecialN_800EEC34(arg0, color, n);
}

/// `ftData_800857E0`.
pub fn ftData_800857E0(kind: i32) {
    melee_core::ct!("ftData_800857E0");
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
    melee_core::ct!("ftData_80085820");
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
    melee_core::ct!("ftData_800858E4");
    let k = kind as usize;
    unsafe {
        let t = CostumeListsForeachCharacter[k].costume_list.add(costume_id as usize);
        load_costume(k, costume_id as usize, t);
    }
}

/// `ftData_800859A8`: release `fp`'s x61C slot unless another fighter shares it.
pub fn ftData_800859A8(f: *mut Fighter) {
    melee_core::ct!("ftData_800859A8");
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

/// TARGET_PC, Envoy rooms: give back kind `kind`'s animation bank (heap 1) when no fighter of that kind is left (`gone`, a fighter being freed, does not count), so a
/// wave's new character fits. Only a bank this file loaded into heap 1 (not one served from the preload cache); the
/// rows are bound again by the next `ftData_80085A14`. True when freed. Statics and heaps are in the rollback
/// snapshot, so a resimulation sees the same frees.
pub fn envoy_bank_release(kind: i32, gone: G) -> bool {
    let k = kind as usize;
    unsafe {
        let data = ftData_Table_Unk0[k].data;
        if data.is_null() || !melee_runtime::tu::lbheap::lbHeap_Contains(1, data) {
            return false;
        }
        let mut g = ext::plink_fighter_head();
        while !g.is_null() {
            let cur = (*g).user_data as *mut Fighter;
            if g != gone && !cur.is_null() && (*cur).kind as i32 == kind {
                return false;
            }
            g = (*g).next;
        }
        melee_runtime::tu::lbheap::lbHeap_80015CA8(1, data);
        ftData_Table_Unk0[k].data = core::ptr::null_mut();
    }
    true
}

/// TARGET_PC, Envoy rooms: the size of kind `kind`'s animation bank file (0 unknown).
pub fn envoy_bank_bytes(kind: i32) -> usize {
    let k = kind as usize;
    unsafe {
        if k >= KMAX || ftData_803C23E4[k].is_null() {
            return 0;
        }
        melee_runtime::tu::lbfile::lbFileGetSize(ftData_803C23E4[k])
    }
}

/// `ftData_80085A14`: load a kind's animation bank and bind its motion rows.
pub fn ftData_80085A14(kind: i32) {
    melee_core::ct!("ftData_80085A14");
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
                    row.x14 = base.wrapping_add(row.x4 as u32) as usize;
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

/// The Geno thrown stand-in bank state (`ftData_ThrowBanks`, `ftData_ThrowBankLoads`, ...).
pub fn throw_state() -> &'static mut ThrowState {
    melee_core::ct!("throw_state");
    throw()
}

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
    melee_core::ct!("ftData_ThrowArenaAlloc");
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
    melee_core::ct!("ftData_ThrowBankLimit");
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
    melee_core::ct!("ftData_ThrowBankReset");
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
    melee_core::ct!("ftData_ThrowBankReport");
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
    melee_core::ct!("ftData_ThrowBankDrop");
    let mut g = ext::plink_fighter_head();
    let lo = b.buf as usize;
    let hi = lo + b.cap;
    while !g.is_null() {
        let f = unsafe { (*g).user_data } as *mut Fighter;
        if !f.is_null() {
            unsafe {
                let a = (*f).x5A4 as usize;
                if a >= lo && a < hi {
                    (*f).x5A4 = 0;
                }
                let a = (*f).x5A8 as usize;
                if a >= lo && a < hi {
                    (*f).x5A8 = 0;
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
    melee_core::ct!("ftData_ThrownStandInPrepare");
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
    melee_core::ct!("ftData_ThrownStandInLoad");
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
            r.x14 = if r.x8 != 0 { ext::gc_addr32((*b).buf.add(r.x4 as usize) as *const c_void) as usize } else { 0 };
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
    melee_core::ct!("ftData_ThrownStandIn");
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
    melee_core::ct!("ftData_80085B10");
    let kind = f.kind as i32;
    f.x59C = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A0 = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A4 = 0;
    f.x5A8 = 0;
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
    melee_core::ct!("ftData_80085B98");
    let k = f.kind as usize;
    let mut r30: u32 = unsafe { ext::gc_addr32(ftData_UnkIntPairs[k].data) };
    if ext::Geno_DefineIsNone(f.kind as i32) != 0 && ext::Geno_DefineTemplateSkeleton(f.kind as i32) == 0 {
        ftData_80085A14(f.kind as i32);
        r30 = unsafe { ext::gc_addr32(ftData_Table_Unk0[k].data) };
    }
    f.x59C = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A0 = ext::HSD_ObjAlloc_fighter_x59C();
    f.x5A4 = 0;
    f.x5A8 = 0;
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
                r3.x14 = r30.wrapping_add(r3.x4 as u32) as usize;
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
        // TARGET_PC (anim_override.rs): a mod's per-clip override replaces the bank slice. Without one installed this is
        // a single test of a static and the code below runs as before.
        if crate::anim_override::active() {
            if let Some(e) = override_for(src, r3) {
                return Some(load_override(buf, e));
            }
        }
        let x14 = (*r3).x14;
        if x14 == ext::gc_addr32(key) as usize {
            return None;
        }
        if x14 == 0 {
            return Some((core::ptr::null_mut(), 0));
        }
        let mut ar: HSD_Archive = core::mem::zeroed();
        let other = ftData_80086060(&mut *f);
        let size = (*r3).x8;
        // C: when the other Ice Climber already holds this file, `memcpy` its figatree buffer and `lbArchiveRelocate` the copy. The native
        // buffer holds the decoded FigaTree objects (not a relocatable GC image), so the copy is parsed from the file again instead: the same
        // data, the same public address, a private decode (a pure memory optimisation in the C).
        if false && !other.is_null() && (*r3).x14 == ext::gc_addr32((*other).x5A4 as *const c_void) as usize {
            core::ptr::copy_nonoverlapping((*other).x59C as *const u8, buf as *mut u8, size as usize);
            let off = (buf as isize).wrapping_sub((*other).x59C as isize);
            if ext::lbArchiveRelocate(&mut ar, (*buf).x0.as_mut_ptr(), size, off) == -1 {
                panic!("lbArchiveRelocate error! {msid:x} (ftdata.c:{:#X})", lines.0);
            }
        } else {
            let a = (*r3).x14;
            if a < 0x8000_0000 {
                let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || melee_core::frameprof::Text::new().s("figatree: disc read"));
                ext::lbArq_80014BD0(a as u32, buf, round_up_32b(size as u32), 0, 0);
            } else {
                core::ptr::copy_nonoverlapping(ext::gc_ptr(a as u32) as *const u8, buf as *mut u8, size as usize);
            }
            let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || melee_core::frameprof::Text::new().s("figatree: parse ").n(size as i64).s(" B"));
            if ext::HSD_ArchiveParse(&mut ar, (*buf).x0.as_mut_ptr(), size) == -1 {
                panic!("HSD_ArchiveParse error! {msid:x} (ftdata.c:{:#X})", lines.1);
            }
        }
        Some((ext::HSD_ArchiveGetPublicAddress(&mut ar, (*r3).x0) as *mut FigaTree, (*r3).x14 as u32))
    }
}

/// TARGET_PC: the key `load_figatree` hands back for an override clip. Real keys are bank addresses (32-byte aligned
/// archive starts) or 0, so this odd value never matches one: the next load of any row reloads, and a row without
/// animation after an override still clears the tree as it does after a bank clip.
pub const ANIM_OVERRIDE_KEY: u32 = 0xFFFF_FFFF;

/// TARGET_PC: the override for motion row `r3` as `src` reads it (anim_override.rs rules): the row's owner kind is the
/// table it lies in (`src`'s own, or Popo's for Nana borrowing it); rows of another table (demo / result) match by symbol.
unsafe fn override_for(src: *mut Fighter, r3: *mut Fighter_WaitAnimData) -> Option<&'static crate::anim_override::Entry> {
    let sym = if (*r3).x0.is_null() { None } else { Some(core::ffi::CStr::from_ptr((*r3).x0).to_bytes()) };
    let in_table = |base: *mut Fighter_WaitAnimData, n: usize| -> Option<i32> {
        if base.is_null() {
            return None;
        }
        // a scan, not pointer arithmetic: rows are few and only modded runs get here
        (0..n).position(|i| base.add(i) == r3).map(|i| i as i32)
    };
    let kind = (*src).kind as i32;
    let own = in_table((*src).x24, (*src).x58C as usize);
    let mut hit = crate::anim_override::find(kind, own, sym);
    if hit.is_none() && kind == Ft_Kind_Nana as i32 {
        // Nana outside demos plays Popo's rows (ftData_80085FD4): Popo's entries follow her there
        let p = Ft_Kind_Popo as usize;
        let fd = gFtDataList[p];
        if !fd.is_null() {
            if let Some(i) = in_table((*fd).xC, ftData_Table_Unk0[p].count as usize) {
                hit = crate::anim_override::find(p as i32, Some(i), sym);
            }
        }
    }
    hit
}

/// TARGET_PC: bind an override clip: its archive bytes into the fighter's animation buffer, parsed like a bank clip.
/// The file was validated at load (size within the buffer, archive structure, the public symbol), so a parse failure
/// here gives no animation (as a row without one) and a log line, never a panic.
unsafe fn load_override(buf: *mut Fighter_x59C_t, e: &crate::anim_override::Entry) -> (*mut FigaTree, u32) {
    let n = e.bytes.len();
    // the buffers' real size: FT_ANIM_BUF_SIZE in matches, 0xB000 in the demo / result scenes (ftdemo_obj_alloc_init)
    let cap = (*core::ptr::addr_of!(crate::tu::fighter::fighter_x59C_alloc_data)).size as usize;
    if n > cap.min(FT_ANIM_BUF_SIZE as usize) {
        eprintln!("anim override {}: {n} bytes is over this scene's animation buffer ({cap} bytes) - not bound here", e.label);
        return (core::ptr::null_mut(), 0);
    }
    core::ptr::copy_nonoverlapping(e.bytes.as_ptr(), buf as *mut u8, n);
    let mut ar: HSD_Archive = core::mem::zeroed();
    if ext::HSD_ArchiveParse(&mut ar, buf as *mut u8, n as i32) == -1 {
        eprintln!("anim override {}: the archive does not parse - not bound", e.label);
        return (core::ptr::null_mut(), 0);
    }
    let tree = ext::HSD_ArchiveGetPublicAddress(&mut ar, e.public.as_ptr() as *mut c_char) as *mut FigaTree;
    (tree, ANIM_OVERRIDE_KEY)
}

/// `ftData_80085CD8`: load motion `msid` (read from `arg1`'s table) as `fp`'s main figatree.
pub fn ftData_80085CD8(f: *mut Fighter, arg1: *mut Fighter, msid: i32) {
    melee_core::ct!("ftData_80085CD8");
    unsafe {
        // C: `int < u32` compares as unsigned.
        if !((msid as u32) < (*arg1).x58C) {
            return;
        }
        if let Some((tree, x14)) = load_figatree(f, arg1, msid, (*f).x5A4 as *mut c_void, (*f).x59C, (0x9FA, 0xA0F)) {
            (*f).x590 = tree;
            (*f).x5A4 = if x14 == 0 { 0 } else { ext::gc_ptr(x14) as usize };
        }
    }
}

/// `ftData_80085E50`: load motion `msid` as `arg0`'s secondary figatree; returns it.
pub fn ftData_80085E50(f: *mut Fighter, msid: i32) -> *mut FigaTree {
    melee_core::ct!("ftData_80085E50");
    unsafe {
        if !((msid as u32) < (*f).x58C) {
            return core::ptr::null_mut();
        }
        if let Some((tree, x14)) = load_figatree(f, f, msid, (*f).x5A8 as *mut c_void, (*f).x5A0, (0xA30, 0xA45)) {
            (*f).x598 = tree;
            (*f).x5A8 = if x14 == 0 { 0 } else { ext::gc_ptr(x14) as usize };
        }
        (*f).x598
    }
}

/// `ftData_80085FD4`: motion row `msid` (Nana outside demos falls back to Popo's unbound rows).
/// Returns the row itself: the C's `ftData_80085FD4_ret*` is a cast of it (same GC layout; the
/// native `ftData_80085FD4_ret` does not share the row's layout, so callers read the row).
pub fn ftData_80085FD4(f: &mut Fighter, msid: i32) -> *mut Fighter_WaitAnimData {
    melee_core::ct!("ftData_80085FD4");
    unsafe {
        if f.kind == Ft_Kind_Nana
            && ext::Player_GetPlayerSlotType(f.player_idx as i32) != Gm_PKind_Demo as i32
            && (*f.x24.offset(msid as isize)).x14 == 0
        {
            return (*gFtDataList[Ft_Kind_Popo as usize]).xC.offset(msid as isize);
        }
        f.x24.offset(msid as isize)
    }
}

/// `ftData_80085FD4(...)->x8` for the hooks that read only the size word.
pub fn ftData_80085FD4_x8(f: &mut Fighter, msid: i32) -> usize {
    melee_core::ct!("ftData_80085FD4_x8");
    unsafe { (*ftData_80085FD4(f, msid)).x8 as u32 as usize }
}

/// `ftData_80086060`: Nana's Popo (outside demos), else NULL.
pub fn ftData_80086060(f: &mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftData_80086060");
    if f.kind == Ft_Kind_Nana && ext::Player_GetPlayerSlotType(f.player_idx as i32) != Gm_PKind_Demo as i32 {
        let g = ext::Player_GetEntityAtIndex(f.player_idx as i32, 0);
        if !g.is_null() {
            return unsafe { (*g).user_data } as *mut Fighter;
        }
    }
    core::ptr::null_mut()
}

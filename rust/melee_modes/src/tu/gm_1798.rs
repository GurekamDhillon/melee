//! Native port of src/melee/gm/gm_1798.c: the results-screen player display (per-player
//! render-to-texture cameras, character spawn for the results scene, the display-state globals).
//! Verified against the C by tests/tu_gm_1798.rs (state oracle).
//!
//! Notes on the port:
//! * This is the `TARGET_PC` build of the C: the results state is the separate global
//!   `lbl_8046E3AC` (not a view past `lbl_8046E1B0`), the camera/scale tables go through
//!   `gm_MexVanillaKind` (`gmResult_TableKind`) and the m-ex "reduce result screen lag" hook
//!   (`gm_1798_MexSetResultModels`) is present.
//! * The inline helpers of `jobj.h` the C expands (`HSD_JObjSetTranslateX`, `HSD_JObjMtxIsDirty`,
//!   `HSD_ASSERT` -> `__assert`) are expanded here, calling the same externs in the same order.
//! * GameCube offsets of the TU-local records are noted in the comments (the native layout differs
//!   where a record holds pointers/`HSD_ImageDesc`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::ext_hook::memzero;
use crate::ext_fns;
use core::ffi::c_void;
use melee_types::*;

mod data;
pub use data::{lbl_804D3FD0, lbl_804D3FD8, lbl_804D3FE0, lbl_804D3FE8, lbl_804D3FF0, lbl_804D3FF8};

/// `CharScaleEntry` (gmresultplayer.static.h, size 0x30). With `TARGET_PC` the C reads the same table
/// as `CameraKindParams` (`x_off[4]` = `x0[0..4]`, `y_off[4]` = `x0[4..8]`, `z_scale[4]` = `x20`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharScaleEntry {
    /// 0x00
    pub x0: [f32; 8],
    /// 0x20
    pub x20: [f32; 4],
}

/// `ResultsRenderFuncs` (size 0x10 on the GameCube).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResultsRenderFuncs {
    pub funcs: [GObj_RenderFunc; 4],
}

/// `ResultsPlayerConfig` (gmresultplayer.static.h, GC size 0xA0).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResultsPlayerConfig {
    /// 0x00
    pub x00: [f32; 9],
    /// 0x24
    pub x24: Vec,
    /// 0x30
    pub x30: Vec,
    /// 0x3C
    pub x3C: ResultsRenderFuncs,
    /// 0x4C
    pub x4C: Vec,
    /// 0x58
    pub x58: Vec,
    /// 0x64
    pub x64: ResultsRenderFuncs,
    /// 0x74
    pub x74: f32,
    /// 0x78
    pub x78: f32,
    /// 0x7C
    pub x7C: f32,
    /// 0x80
    pub x80: f32,
    /// 0x84
    pub x84: f32,
    /// 0x88
    pub x88: f32,
    /// 0x8C
    pub x8C: f32,
    /// 0x90
    pub x90: f32,
    /// 0x94
    pub x94: f32,
    /// 0x98
    pub x98: f32,
    /// 0x9C
    pub x9C: f32,
}

/// `PackedS16x4` (8 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub union PackedS16x4 {
    pub h: [i16; 4],
    pub w: [u32; 2],
}

/// `U32Pair` (8 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct U32Pair {
    pub lo: u32,
    pub hi: u32,
}

/// `lbl_8046E3AC_t` (GC size 0x2314): the results-screen state.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_8046E3AC_t {
    /// 0x00: `x0_0:4` (bits 7..4), `x0_4:2` (bits 3..2), `x0_6:2` (bits 1..0), MSB-first as on the GameCube
    pub bf_0: u8,
    /// 0x01
    pub player_flags: [u8; 4],
    /// 0x05
    pub pad_x5: u8,
    /// 0x06
    pub x6: [u16; 4],
    /// 0x0E
    pub variant: [u8; 4],
    /// 0x12
    pub pad_12: [u8; 2],
    /// 0x14
    pub char_kind: [i32; 4],
    /// 0x24
    pub costume_override: [u8; 4],
    /// 0x28
    pub match_end: MatchEnd,
    /// 0x22A4
    pub scissor_x: [u16; 4],
    /// 0x22AC
    pub scissor_y: [u16; 4],
    /// 0x22B4
    pub dim_w1: [u16; 4],
    /// 0x22BC
    pub dim_w2: [u16; 4],
    /// 0x22C4
    pub dim_h1: [u16; 4],
    /// 0x22CC
    pub dim_h2: [u16; 4],
    /// 0x22D4
    pub score_tbl: [PackedS16x4; 4],
    /// 0x22F4
    pub x22F4: [PackedS16x4; 4],
}

impl lbl_8046E3AC_t {
    #[inline]
    pub fn x0_0(&self) -> u8 {
        (self.bf_0 >> 4) & 0xF
    }
    #[inline]
    pub fn set_x0_0(&mut self, v: u8) {
        self.bf_0 = (self.bf_0 & 0x0F) | ((v & 0xF) << 4);
    }
    #[inline]
    pub fn x0_4(&self) -> u8 {
        (self.bf_0 >> 2) & 0x3
    }
    #[inline]
    pub fn set_x0_4(&mut self, v: u8) {
        self.bf_0 = (self.bf_0 & !0x0C) | ((v & 0x3) << 2);
    }
    #[inline]
    pub fn x0_6(&self) -> u8 {
        self.bf_0 & 0x3
    }
    #[inline]
    pub fn set_x0_6(&mut self, v: u8) {
        self.bf_0 = (self.bf_0 & !0x03) | (v & 0x3);
    }
}

/// `ResultsDisplayData` (GC size 0x1DC): the image descriptors the per-player cameras render into.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResultsDisplayData {
    /// 0x000
    pub pad_000: [u8; 0x104],
    /// 0x104
    pub player_img1: [HSD_ImageDesc; 4],
    /// 0x164
    pub player_img2: [HSD_ImageDesc; 4],
    /// 0x1C4
    pub shared_img: HSD_ImageDesc,
}

/// `ResultsCharacterData` (gmresultplayer.static.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResultsCharacterData {
    /// 0x000
    pub scale: [f32; 32],
    /// 0x080
    pub slot_off: [[[f32; 4]; 3]; 32],
    /// 0x680
    pub x680: [[u8; 0x10]; 27],
    /// 0x830
    pub pad_830: [u8; 0x60],
}

pub static mut lbl_8046E1B0: ResultsDisplayData = unsafe { core::mem::zeroed() };
pub static mut lbl_8046E38C: [*mut HSD_GObj; 4] = [core::ptr::null_mut(); 4];
pub static mut lbl_8046E39C: [*mut HSD_JObj; 4] = [core::ptr::null_mut(); 4];
pub static mut lbl_8046E3AC: lbl_8046E3AC_t = unsafe { core::mem::zeroed() };

/// Callees (and data) in TUs that are not ported yet.
pub mod ext {
    use super::*;
    pub static mut lbl_8046DBE8: ResultsData = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = 0;
    pub static mut HSD_GObj_CameraKind: u8 = 0;
    pub static mut lbl_803B7B68: ResultsPlayerConfig = unsafe { core::mem::zeroed() };
    pub static mut gmResultCameraDesc: HSD_CameraDescPerspective = unsafe { core::mem::zeroed() };
    pub static mut gmResultPlayerColors: [u32; 4] = [0; 4];
    /// 32 rows (the C declares an unsized array; the results tables have 32 rows)
    pub static mut gmResultCharacterScaleData: [CharScaleEntry; 32] = unsafe { core::mem::zeroed() };
    pub static mut gmResultCharacterData: ResultsCharacterData = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
    pub static mut gmResultScoreTableInit: [u32; 8] = [0; 8];
    pub static mut gmResultX22F4Init: [u32; 8] = [0; 8];
    ext_fns! {
        fn Camera_800313E0(a0: *mut HSD_GObj, a1: u64);
        fn Camera_Init(a0: i32) = crate::tu::adapt::Camera_Init__cmc;
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8, a3: u32);
        fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u32);
        fn Ground_801C0378(a0: i32) = crate::tu::adapt::Ground_801C0378__874334;
        fn HSD_CObjEndCurrent() = crate::tu::adapt::HSD_CObjEndCurrent__d69e71;
        fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) = crate::tu::adapt::HSD_CObjEraseScreen__c961ec;
        fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjLoadDesc__66a547;
        fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = crate::tu::adapt::HSD_CObjSetCurrent__d18716;
        fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
        fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
        fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = crate::tu::adapt::HSD_CObjSetScissor__2d3b20;
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
        fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
        extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_ImageDescCopyFromEFB(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: GXBool, a4: i32) = crate::tu::adapt::HSD_ImageDescCopyFromEFB__464d65;
        fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
        fn HSD_Randi(a0: i32) -> i32;
        fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
        fn Item_80266FA8() = crate::tu::adapt::Item_80266FA8__f7386b;
        fn Item_80266FCC() = crate::tu::adapt::Item_80266FCC__dfa49f;
        fn Mex_Enabled(a0: *const i8) -> i32;
        fn Mex_ResultScaleForPortCKind(a0: i32) -> f32;
        fn Player_80032A04(a0: i32, a1: *mut Vec) = crate::tu::adapt::Player_80032A04__2b7dce;
        fn Player_800360D8(a0: i32) -> f32 = crate::tu::adapt::Player_800360D8__b9fa26;
        fn Player_80036E20(a0: CharacterKind, a1: *mut HSD_Archive, a2: i32) = crate::tu::adapt::Player_80036E20__54207b;
        fn Player_80036F34(a0: i32, a1: i32) = crate::tu::adapt::Player_80036F34__e8645c;
        fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntity__090538;
        fn Player_InitAllPlayers() = crate::tu::adapt::Player_InitAllPlayers__1d8abf;
        fn Player_SetCostumeId(a0: i32, a1: i32) = crate::tu::adapt::Player_SetCostumeId__920e80;
        fn Player_SetFacingDirection(a0: i32, a1: f32) = crate::tu::adapt::Player_SetFacingDirection__e0fc0a;
        fn Player_SetPlayerCharacter(a0: i32, a1: CharacterKind) = crate::tu::adapt::Player_SetPlayerCharacter__d99f81;
        fn Player_SetPlayerId(a0: i32, a1: i32) = crate::tu::adapt::Player_SetPlayerId__dba656;
        fn Player_SetScale(a0: i32, a1: f32) = crate::tu::adapt::Player_SetScale__3c042c;
        fn Player_SetSlottype(a0: i32, a1: Gm_PKind) = crate::tu::adapt::Player_SetSlottype__ff2974;
        fn Stage_802251E8(a0: StKind, a1: *mut i32) = crate::tu::adapt::Stage_802251E8__6e8910;
        fn Stage_8022524C() = crate::tu::adapt::Stage_8022524C__8c7585;
        fn __assert(a0: *const i8, a1: u32, a2: *const i8);
        fn efAsync_LoadSync(a0: i32);
        fn efLib_Init() = crate::tu::adapt::efLib_Init__0977f5;
        fn fn_80174274() -> *mut MatchEnd = crate::tu::adapt::fn_80174274__726c5b;
        fn fn_801795D4() -> i32 = crate::tu::adapt::fn_801795D4__0f9c8c;
        fn fn_801796F0(a0: i32) -> i32 = crate::tu::adapt::fn_801796F0__2a3c9d;
        fn ftDemo_ObjAllocInit();
        fn ftLib_800876B4(a0: *mut HSD_GObj) -> i32;
        fn gm_80160438(a0: i32) -> *mut i8 = crate::tu::adapt::gm_80160438__384bdb;
        fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32 = crate::tu::adapt::gm_80160854__387c80;
        fn gm_80160968(a0: u32) -> _GXColor = crate::tu::adapt::gm_80160968__e03b47;
        fn gm_MexVanillaKind(a0: i32, a1: i32) -> i32 = crate::tu::adapt::gm_MexVanillaKind__b3a17c;
        fn lbArchive_LoadArchive(a0: *const i8) -> *mut HSD_Archive = crate::tu::adapt::lbArchive_LoadArchive__471a59;
        fn lbBgFlash_800206D4(a0: *mut _GXColor, a1: *mut _GXColor, a2: i32) = crate::tu::adapt::lbBgFlash_800206D4__8952dd;
        fn lbBgFlash_800208EC(a0: i32) = crate::tu::adapt::lbBgFlash_800208EC__c1a64f;
        fn lb_8000FCDC() = crate::tu::adapt::lb_8000FCDC__a0c24e;
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj = crate::tu::adapt::lb_80011AC4__8bc38c;
        fn lb_800121FC(a0: *mut HSD_ImageDesc, a1: i32, a2: i32, a3: _GXTexFmt, a4: i16) -> *mut HSD_ImageDesc = crate::tu::adapt::lb_800121FC__096a83;
        fn memzero(a0: *mut c_void, a1: ssize_t) = super::memzero_ssize;
        fn mpColl_80041C78() = crate::tu::adapt::mpColl_80041C78__63449b;
    }
}

#[inline]
unsafe fn memzero_ssize(p: *mut c_void, n: ssize_t) {
    memzero(p, n)
}

const JOBJ_USER_DEF_MTX_: u32 = JOBJ_USER_DEF_MTX as u32;
const JOBJ_MTX_DIRTY_: u32 = JOBJ_MTX_DIRTY as u32;
const JOBJ_MTX_INDEP_SRT_: u32 = JOBJ_MTX_INDEP_SRT as u32;

/// `HSD_ASSERT(line, cond)` of jobj.h: `__assert` (never returns on the GameCube) when `cond` is false.
#[inline]
unsafe fn hsd_assert(line: u32, cond: bool, what: &'static [u8]) {
    if !cond {
        ext::__assert(b"jobj.h\0".as_ptr() as *const i8, line, what.as_ptr() as *const i8);
    }
}

/// jobj.h `HSD_JObjMtxIsDirty`.
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(564, !jobj.is_null(), b"jobj\0");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX_ == 0 && (*jobj).flags & JOBJ_MTX_DIRTY_ != 0 {
        result = true;
    }
    result
}

/// jobj.h `HSD_JObjSetMtxDirty` (the macro).
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// jobj.h `HSD_JObjSetTranslateX`.
unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(932, !jobj.is_null(), b"jobj\0");
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT_ == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        core::ptr::null_mut()
    } else {
        (*jobj).child
    }
}

unsafe fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        core::ptr::null_mut()
    } else {
        (*jobj).next
    }
}

/// The results camera/scale tables only have 32 rows; retail Popo and sentinels lie beyond them.
pub unsafe fn gmResult_TableKind(kind: i32) -> i32 {
    let row = ext::gm_MexVanillaKind(kind, 1);
    if (row as u32) < 32 {
        row
    } else {
        CKind_Mario as i32
    }
}

/// m-ex "reduce result screen lag": only the model for the slot being rendered stays visible
/// (`show_slot >= 0`), or every model (`show_slot < 0`). Opt-in: `MELEE_MEX=reduce_result_screen_lag`.
pub unsafe fn gm_1798_MexSetResultModels(show_slot: i32) {
    if ext::Mex_Enabled(b"reduce_result_screen_lag\0".as_ptr() as *const i8) == 0 {
        return;
    }
    for i in 0..6 {
        let entity = ext::Player_GetEntity(i);
        if entity.is_null() {
            continue;
        }
        let fp = (*entity).user_data as *mut Fighter;
        (*fp).x21FC_flag
            .anon_0
            .set_b0(if show_slot < 0 { 1 } else { (i == show_slot) as u8 });
    }
}

pub unsafe fn get_big_loser(slot: i32, match_end: *mut MatchEnd) -> i32 {
    (*match_end).player_standings[slot as usize].is_big_loser as i32
}

pub unsafe fn fn_80179990_img_at(imgs: *mut HSD_ImageDesc, slot: i32) -> *mut HSD_ImageDesc {
    imgs.add(slot as usize)
}

/// `match_end` big-loser lookup shared by most functions (team or player slot).
#[inline]
unsafe fn big_loser_of(match_end: *mut MatchEnd, slot: usize) -> i32 {
    if (*match_end).is_teams == 0 {
        (*match_end).player_standings[slot].is_big_loser as i32
    } else {
        (*match_end).team_standings[(*match_end).player_standings[slot].team as usize].is_big_loser as i32
    }
}

pub unsafe fn fn_80179854() {
    let mut color1 = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut color2 = _GXColor { r: 0, g: 0, b: 0, a: 0x3C };
    let match_end = core::ptr::addr_of_mut!(lbl_8046E3AC.match_end);
    let gobjs = lbl_8046E38C.as_mut_ptr();

    ext::lbBgFlash_800206D4(&mut color1, &mut color2, 0x1E);

    for i in 0..4usize {
        let lookup = big_loser_of(match_end, i);

        if (*match_end).player_standings[i].pkind != 3 && lookup != 0 {
            HSD_JObjSetTranslateX((**gobjs.add(i)).hsd_obj as *mut HSD_JObj, -300.0);
            lbl_8046E3AC.set_x0_6(1);
        }
    }
}

pub unsafe fn fn_80179990_set_erase_color(match_end: *mut MatchEnd, slot: i32) {
    let s = &(*match_end).player_standings[slot as usize];
    let color = ext::gm_80160968(ext::gm_80160854(slot as u8, s.team, ((*match_end).is_teams == 1) as u8, s.pkind));
    ext::HSD_SetEraseColor(color.r, color.g, color.b, color.a);
}

pub unsafe fn fn_80179990_copy_efb(imgs: *mut HSD_ImageDesc, slot: i32, w: *const u16, h: *const u16) {
    ext::HSD_ImageDescCopyFromEFB(
        imgs.add(slot as usize),
        (0x140 - (*w as i32 / 4) * 2) as u16,
        (0xF4 - (*h as i32 / 2) * 2) as u16,
        0,
        0,
    );
}

pub unsafe fn fn_80179990_copy_efb_at(imgs: *mut HSD_ImageDesc, slot: i32, w: *const u16, h: *const u16, x: *const u16, y: *const u16) {
    ext::HSD_ImageDescCopyFromEFB(
        imgs.add(slot as usize),
        (*x as i32 + (0x140 - (*w as i32 / 4) * 2)) as u16,
        (*y as i32 + (0xF4 - (*h as i32 / 2) * 2)) as u16,
        0,
        0,
    );
}

pub unsafe fn fn_80179990(arg0: *mut HSD_GObj, _arg1: i32, arg2: i32) {
    let disp = core::ptr::addr_of_mut!(lbl_8046E1B0);
    let match_end = core::ptr::addr_of_mut!(lbl_8046E3AC.match_end);
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);
    let mut child_jobj: *mut HSD_JObj = core::ptr::null_mut();

    gm_1798_MexSetResultModels(arg2);

    ext::fn_801795D4();
    ext::fn_801796F0(arg2);

    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;

    let lookup = big_loser_of(match_end, arg2 as usize);

    if lookup != 0 {
        let root = (*lbl_8046E38C[arg2 as usize]).hsd_obj as *mut HSD_JObj;
        child_jobj = if root.is_null() { core::ptr::null_mut() } else { (*root).child };
    }

    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        if lookup != 0 {
            fn_80179990_set_erase_color(match_end, arg2);
            ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
            ext::Camera_800313E0(arg0, 0);

            fn_80179990_copy_efb_at(
                (*disp).player_img2.as_mut_ptr(),
                arg2,
                (*st).dim_w1.as_ptr().add(lookup as usize),
                (*st).dim_h1.as_ptr().add(lookup as usize),
                (*st).scissor_x.as_ptr().add(lookup as usize),
                (*st).scissor_y.as_ptr().add(lookup as usize),
            );

            if (*st).x0_4() == 0 {
                fn_80179990_copy_efb(
                    (*disp).player_img1.as_mut_ptr(),
                    arg2,
                    (*st).dim_w1.as_ptr().add(lookup as usize),
                    (*st).dim_h1.as_ptr().add(lookup as usize),
                );
            }

            ext::HSD_CObjEraseScreen(cobj, 1, 1, 1);
            ext::HSD_ImageDescCopyFromEFB(core::ptr::addr_of_mut!((*disp).shared_img), 0x10E, 0x7C, 1, 0);
            ext::HSD_CObjEndCurrent();

            if (*st).x0_4() == 0 {
                let image_desc1 = (*disp).player_img1.as_mut_ptr();
                (*(*(*(*child_jobj).u.dobj).mobj).tobj).imagedesc = image_desc1.add(arg2 as usize);
            }

            if (*st).x0_4() != 0 {
                (*(*(*(*(*lbl_8046E39C[arg2 as usize]).u.dobj).next).mobj).tobj).imagedesc =
                    (*disp).player_img2.as_mut_ptr().add(arg2 as usize);
            }
        } else if ext::ftLib_800876B4(ext::Player_GetEntity(arg2)) == 0 {
            if (*st).player_flags[arg2 as usize] == 0 && (*st).x0_6() != 0 {
                fn_80179990_set_erase_color(match_end, arg2);
                ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
                ext::Camera_800313E0(arg0, 0);

                fn_80179990_copy_efb_at(
                    (*disp).player_img2.as_mut_ptr(),
                    arg2,
                    (*st).dim_w1.as_ptr().add(lookup as usize),
                    (*st).dim_h1.as_ptr().add(lookup as usize),
                    (*st).scissor_x.as_ptr().add(lookup as usize),
                    (*st).scissor_y.as_ptr().add(lookup as usize),
                );

                ext::HSD_CObjEraseScreen(cobj, 1, 1, 1);
                ext::HSD_ImageDescCopyFromEFB(core::ptr::addr_of_mut!((*disp).shared_img), 0x10E, 0x7C, 1, 0);
                ext::HSD_CObjEndCurrent();

                (*st).player_flags[arg2 as usize] = 1;
                (*(*(*(*(*lbl_8046E39C[arg2 as usize]).u.dobj).next).mobj).tobj).imagedesc =
                    (*disp).player_img2.as_mut_ptr().add(arg2 as usize);
            }
        }
    }
}

pub unsafe fn fn_80179D3C(gobj: *mut HSD_GObj, arg1: i32) {
    fn_80179990(gobj, arg1, 0);
}

pub unsafe fn fn_80179D60(gobj: *mut HSD_GObj, arg1: i32) {
    fn_80179990(gobj, arg1, 1);
}

pub unsafe fn fn_80179D84(gobj: *mut HSD_GObj, arg1: i32) {
    fn_80179990(gobj, arg1, 2);
}

pub unsafe fn fn_80179DA8(gobj: *mut HSD_GObj, arg1: i32) {
    fn_80179990(gobj, arg1, 3);
}

/// Shared body of fn_80179DCC / E34 / E9C / F04 (they differ only in the slot passed to the m-ex hook).
#[inline]
unsafe fn draw_result_cobj(gobj: *mut HSD_GObj, slot: i32) {
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    gm_1798_MexSetResultModels(slot);
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        ext::Camera_800313E0(gobj, 0);
        (*gobj).gxlink_prios = 0x80;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
    gm_1798_MexSetResultModels(-1);
}

pub unsafe fn fn_80179DCC(gobj: *mut HSD_GObj, _arg1: i32) {
    draw_result_cobj(gobj, 0);
}

pub unsafe fn fn_80179E34(gobj: *mut HSD_GObj, _arg1: i32) {
    draw_result_cobj(gobj, 1);
}

pub unsafe fn fn_80179E9C(gobj: *mut HSD_GObj, _arg1: i32) {
    draw_result_cobj(gobj, 2);
}

pub unsafe fn fn_80179F04(gobj: *mut HSD_GObj, _arg1: i32) {
    draw_result_cobj(gobj, 3);
}

pub unsafe fn fn_80179F6C(idx: i32, value: *mut HSD_GObj) {
    lbl_8046E38C[idx as usize] = value;
}

pub unsafe fn fn_80179F84(jobj: *mut HSD_JObj) {
    let mut child = HSD_JObjGetChild(jobj);

    for i in 0..4 {
        lbl_8046E39C[i] = child;
        child = HSD_JObjGetNext(child);
    }

    lbl_8046E3AC.set_x0_4(1);
}

pub unsafe fn fn_8017A004() {
    let data = core::ptr::addr_of_mut!(ext::lbl_8046DBE8);
    let gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*(*data).pnlsce).lights);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);
}

pub unsafe fn fn_8017A078(arg0: i32) {
    let config = core::ptr::addr_of!(ext::lbl_803B7B68);
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);
    let mut eye = (*config).x24;
    let mut interest = (*config).x30;
    let callbacks = (*config).x3C;

    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc(core::ptr::addr_of_mut!(ext::gmResultCameraDesc) as *mut HSD_CObjDesc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);

    eye.y = melee_core::fp::fmadds(eye.y, (arg0 + 1) as f32, 0.7f32 * ext::Player_800360D8(arg0));
    interest.y = melee_core::fp::fmadds(interest.y, (arg0 + 1) as f32, 0.7f32 * ext::Player_800360D8(arg0));

    let mode = ext::fn_801795D4();
    let idx = ext::fn_801796F0(arg0);
    let val = (*st).score_tbl[mode as usize].h[idx as usize];
    interest.x -= val as f32;
    eye.x -= val as f32;
    interest.y -= 10.0;
    eye.y -= 10.0;
    eye.z += melee_core::fp::fmadds(20.0, (mode - 2) as f32, 110.0);

    if mode == 0 {
        let kind = (*st).char_kind[arg0 as usize];
        if kind == CKind_Koopa as i32 && (*st).variant[arg0 as usize] == 1 {
            eye.z += 6.0;
        } else if kind == CKind_Mars as i32 && (*st).variant[arg0 as usize] == 1 {
            eye.z += 7.5;
        } else if kind == CKind_Captain as i32 && (*st).variant[arg0 as usize] == 2 {
            eye.z += 6.0;
        }
    }

    ext::HSD_CObjSetEyePosition(cobj, &mut eye);
    ext::HSD_CObjSetInterest(cobj, &mut interest);
    ext::GObj_SetupGXLinkMax(gobj, callbacks.funcs[arg0 as usize], 5);
}

pub unsafe fn fn_8017A318(arg0: i32) -> *mut HSD_GObj {
    let scissor_init = _Scissor { left: 270, right: 370, top: 124, bottom: 276 };
    let config = core::ptr::addr_of!(ext::lbl_803B7B68);
    let match_end = core::ptr::addr_of_mut!(lbl_8046E3AC.match_end);
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);

    ext::fn_801795D4();
    ext::fn_801796F0(arg0);

    let mut scissor = scissor_init;

    let mut eye = (*config).x4C;
    let mut interest = (*config).x58;
    let callbacks = (*config).x64;
    let slot = big_loser_of(match_end, arg0 as usize);

    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc(core::ptr::addr_of_mut!(ext::gmResultCameraDesc) as *mut HSD_CObjDesc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);

    {
        let n = arg0 + 1;
        eye.y *= n as f32;
        interest.y *= n as f32;
    }

    let variant = (*st).variant[arg0 as usize];
    let vi = |v: u8| -> usize { if (v as i32) <= 2 { v as usize } else { 3 } };

    let kind_data = gmResult_TableKind((*st).char_kind[arg0 as usize]) as usize;
    let tbl = core::ptr::addr_of!(ext::gmResultCharacterScaleData) as *const CharScaleEntry;
    let ent = &*tbl.add(kind_data);
    // CameraKindParams view: x_off = x0[0..4], y_off = x0[4..8], z_scale = x20[0..4]
    eye.y += ent.x0[4 + vi(variant)];
    interest.y += ent.x0[4 + vi(variant)];
    eye.x += ent.x0[vi(variant)];

    {
        let interest_x = interest.x + ent.x0[vi(variant)];
        interest.x = interest_x;
        // slot_off[kind][axis][slot], indexed like the C (slot is not range checked)
        let so = core::ptr::addr_of!(ext::gmResultCharacterData.slot_off) as *const f32;
        let x_off = *so.add(kind_data * 12 + slot as usize);
        eye.x += x_off;
        interest.x += x_off;

        let y_off = *so.add(kind_data * 12 + 4 + slot as usize);
        eye.y += y_off;
        interest.y += y_off;
    }

    if slot == 0 {
        eye.z += 320.0;
    }

    let z = ent.x20[vi(variant)];
    if (1.0f32 - z) < 0.0 {
        eye.z = melee_core::fp::fmadds(100.0, 1.0f32 - z, eye.z);
    } else {
        eye.z = melee_core::fp::fmadds(300.0, 1.0f32 - z, eye.z);
    }

    ext::HSD_CObjSetEyePosition(cobj, &mut eye);
    ext::HSD_CObjSetInterest(cobj, &mut interest);
    ext::HSD_CObjSetScissor(cobj, &mut scissor);
    ext::GObj_SetupGXLinkMax(gobj, callbacks.funcs[arg0 as usize], 0);

    if slot == 0 {
        fn_8017A078(arg0);
    }
    gobj
}

pub unsafe fn fn_8017A67C(mut kind: CharacterKind, arg1: i32, arg2: i32) -> *mut Fighter_GObj {
    let config = core::ptr::addr_of!(ext::lbl_803B7B68);
    let match_end = core::ptr::addr_of_mut!(lbl_8046E3AC.match_end);
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);
    let mut gobj: *mut HSD_GObj = core::ptr::null_mut();

    let slot_type = big_loser_of(match_end, arg2 as usize);

    if !ext::gm_80160438(kind as i32).is_null() {
        let mut pos: Vec = *(core::ptr::addr_of!((*config).x74) as *const Vec);

        if kind.wrapping_sub(0x12) <= 1 {
            if (*match_end).player_standings[arg2 as usize].ftkind as i32 == 7 {
                kind = CKind_Seak;
            } else {
                kind = CKind_Zelda;
            }
        }

        ext::Player_80036E20(kind, ext::lbArchive_LoadArchive(ext::gm_80160438(kind as i32)), 0);
        ext::Player_SetPlayerCharacter(arg2, kind);
        ext::Player_SetCostumeId(arg2, arg1);
        ext::Player_SetPlayerId(arg2, arg2);
        ext::Player_SetSlottype(arg2, Gm_PKind_Demo);

        if kind == CKind_GameWatch {
            ext::Player_SetFacingDirection(arg2, -1.0);
        } else {
            ext::Player_SetFacingDirection(arg2, 0.0);
        }

        let mut variant: i32;

        if slot_type == 0 {
            let buttons = ext::HSD_PadCopyStatus[(arg2 as u8) as usize].button;
            if buttons & 0x200 != 0 {
                variant = 0;
            } else if buttons & 0x800 != 0 {
                variant = 1;
            } else if buttons & 0x400 != 0 {
                variant = 2;
            } else {
                variant = ext::HSD_Randi(3);
            }
        } else {
            variant = 4;
        }

        if variant != 4 {
            let over = (*st).costume_override[arg2 as usize];
            if over != 0 {
                variant = over as i32 - 1;
            }
        }

        if slot_type == 0 {
            let mut pos2: Vec = *(core::ptr::addr_of!((*config).x80) as *const Vec);
            pos2.y = 100.0f32 * (arg2 + 1) as f32;
            ext::Player_80032A04(arg2, &mut pos2);
            if kind >= 32 {
                ext::Player_SetScale(arg2, 1.8f32 * ext::Mex_ResultScaleForPortCKind(kind as i32));
            } else {
                ext::Player_SetScale(arg2, 1.8f32 * ext::gmResultCharacterData.scale[kind as usize]);
            }
            ext::Player_80036F34(arg2, variant);
        } else {
            let sp: [f32; 4] = *(core::ptr::addr_of!((*config).x8C) as *const [f32; 4]);
            let var_idx = if variant <= 2 { variant } else { 3 } as usize;
            let scale = ext::gmResultCharacterScaleData[gmResult_TableKind(kind as i32) as usize].x20[var_idx];
            ext::Player_80036F34(arg2, variant);
            ext::Player_SetScale(arg2, scale * (sp[slot_type as usize] * (20.0f32 / ext::Player_800360D8(arg2))));
            pos.y = 100.0f32 * (arg2 + 1) as f32;
            ext::Player_80032A04(arg2, &mut pos);
        }

        (*st).variant[arg2 as usize] = variant as u8;
        (*st).char_kind[arg2 as usize] = kind as i32;
        gobj = ext::Player_GetEntity(arg2);
    }

    gobj as *mut Fighter_GObj
}

pub unsafe fn inline1(imgs: *mut HSD_ImageDesc, slot: i32, w: *const u16, h: *const u16) {
    (*imgs.add(slot as usize)).image_ptr = core::ptr::null_mut();
    ext::lb_800121FC(imgs.add(slot as usize), *w as i32, *h as i32, 5, 0);
}

pub unsafe fn fn_8017A9B4(slot: i32) {
    let disp = core::ptr::addr_of_mut!(lbl_8046E1B0);
    let match_end = core::ptr::addr_of_mut!(lbl_8046E3AC.match_end);
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);

    let lookup = big_loser_of(match_end, slot as usize);

    inline1(
        (*disp).player_img1.as_mut_ptr(),
        slot,
        (*st).dim_w1.as_ptr().add(lookup as usize),
        (*st).dim_h1.as_ptr().add(lookup as usize),
    );
    inline1(
        (*disp).player_img2.as_mut_ptr(),
        slot,
        (*st).dim_w2.as_ptr().add(lookup as usize),
        (*st).dim_h2.as_ptr().add(lookup as usize),
    );
}

pub unsafe fn fn_8017AA78(arg0: *const u8) {
    let st = core::ptr::addr_of_mut!(lbl_8046E3AC);

    ext::memzero(
        core::ptr::addr_of_mut!(lbl_8046E1B0.pad_000) as *mut c_void,
        core::mem::size_of_val(&lbl_8046E1B0.pad_000) as ssize_t,
    );
    ext::lbBgFlash_800208EC(6);
    ext::Camera_Init(8);
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(0x40);
    ext::Stage_802251E8(St_Kind_Dummy, core::ptr::null_mut());
    ext::Stage_8022524C();
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    lbl_8046E1B0.shared_img.image_ptr = core::ptr::null_mut();
    ext::lb_800121FC(core::ptr::addr_of_mut!(lbl_8046E1B0.shared_img), 0x64, 0x98, GX_TF_RGB5A3, 0);
    (*st).match_end = *ext::fn_80174274();

    (*st).set_x0_0(1);
    (*st).set_x0_4(0);
    (*st).set_x0_6(0);

    *((*st).dim_w1.as_mut_ptr() as *mut U32Pair) = lbl_804D3FD0;
    *((*st).dim_h1.as_mut_ptr() as *mut U32Pair) = lbl_804D3FD8;
    *((*st).dim_w2.as_mut_ptr() as *mut U32Pair) = lbl_804D3FE0;
    *((*st).dim_h2.as_mut_ptr() as *mut U32Pair) = lbl_804D3FE8;
    *((*st).scissor_y.as_mut_ptr() as *mut U32Pair) = lbl_804D3FF0;
    *((*st).scissor_x.as_mut_ptr() as *mut U32Pair) = lbl_804D3FF8;

    for i in 0..4usize {
        (*st).player_flags[i] = 0;
        (*st).costume_override[i] = *arg0.add(i);
        if (*st).match_end.outcome as u32 == OUTCOME_NO_CONTEST as u32 {
            (*st).match_end.player_standings[i].is_big_loser = 1;
            let team = (*st).match_end.player_standings[i].team as usize;
            (*st).match_end.team_standings[team].is_big_loser = 1;
        }
        (*st).x6[i] = 0;
        (*st).score_tbl[i] = *(core::ptr::addr_of!(ext::gmResultScoreTableInit) as *const PackedS16x4).add(i);
        (*st).x22F4[i] = *(core::ptr::addr_of!(ext::gmResultX22F4Init) as *const PackedS16x4).add(i);
    }
}

//! Port of src/melee/if/ifstock.c (built `TARGET_PC`, with ifstock_geno_art.inc and pc/geno/geno_art_{anim.inc,text.h}):
//! the stock-icon / coin / stock-count HUD. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifstock.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_assignments, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Geno_DefineArtBaseCK(a0: i32) -> i32;
    fn Geno_DefineArtOpen(a0: i32, a1: i32, a2: i32) -> i32;
    fn Geno_DefineBaseCK(a0: i32) -> i32;
    fn Geno_DefineName(a0: i32, a1: *mut i8, a2: i32) -> i32;
    fn Ground_801C1DE4(a0: *mut i32, a1: *mut i32) = |a0: *mut i32, a1: *mut i32| { unsafe { let _ = crate::tu::ground::Ground_801C1DE4(a0 as _, a1 as _); } };
    fn GxTex_Close(a0: i32);
    fn GxTex_CopyImage(a0: i32, a1: *mut c_void);
    fn GxTex_Format(a0: i32) -> i32;
    fn GxTex_Height(a0: i32) -> i32;
    fn GxTex_ImageSize(a0: i32) -> i32;
    fn GxTex_Width(a0: i32) -> i32;
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetRate(a0 as _, a1 as _); } };
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390CAC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_80390CAC(a0 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_TObjReqAnimAll(a0: *mut HSD_TObj, a1: f32) = |a0: *mut HSD_TObj, a1: f32| { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjReqAnimAll(a0 as _, a1 as _); } };
    fn Player_800325C8(a0: u32, a1: i32) -> i8 = |a0: u32, a1: i32| unsafe { crate::pl::Player_800325C8(a0 as _, a1 as _) as _ };
    fn Player_GetCoins(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetCoins(a0 as _) as _ };
    fn Player_GetStocks(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetStocks(a0 as _) as _ };
    fn Script_StatusHUDVisible() -> i32;
    fn UI_TextI4(a0: *const i8, a1: i32, a2: i32, a3: i32, a4: *mut c_void) -> i32;
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec3) -> *mut c_void;
    fn gmVs_GetSceneState() -> *mut VsSceneState = || unsafe { crate::gm::gmVs_GetSceneState() as _ };
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_80168B34(a0: u32, a1: i32, a2: i32) -> f32;
    fn gm_80168BF8(a0: i32) -> f32;
    fn gm_80169394() -> i32 = || unsafe { crate::gm::gm_80169394() as _ };
    fn gm_801694A0(a0: *mut HSD_GObj) -> i32;
    fn gm_80169520() -> *mut c_void = || unsafe { crate::gm::gm_80169520() as _ };
    fn gm_80169530() -> *mut c_void = || unsafe { crate::gm::gm_80169530() as _ };
    fn gm_8016A1F8() -> i32 = || unsafe { crate::gm::gm_8016A1F8() as _ };
    fn gm_8016A944() -> i32;
    fn gm_8016A97C() -> *mut c_void = || unsafe { crate::gm::gm_8016A97C() as _ };
    fn gm_8016A98C() -> *mut lbl_8046B668_t = || unsafe { crate::gm::gm_8016A98C() as _ };
    fn gm_8016B184() -> i32 = || unsafe { crate::gm::gm_8016B184() as _ };
    fn gm_8016B1A8() -> i32 = || unsafe { crate::gm::gm_8016B1A8() as _ };
    fn gm_GenoStockFrame(a0: i32, a1: i32) -> f32;
    fn gm_IsMultimanSmashMode() -> i32;
    fn gm_MexStockFrame(a0: i32, a1: i32) -> f32;
    fn gm_MexStockMatAnim() -> *mut HSD_MatAnimJoint;
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_GetPlayerHUDPosition(a0: i32) -> *mut Vec = |a0: i32| unsafe { crate::tu::ifall::ifAll_GetPlayerHUDPosition(a0 as _) as _ };
    fn ifStatus_GetHUDInfo() -> *mut HudIndex = super::hud_info_default;
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]) = |a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void]; for (s, n) in rest.iter() { v.push(*s as *mut core::ffi::c_void); v.push(*n as *mut core::ffi::c_void); } v.push(core::ptr::null_mut()); melee_runtime::tu::lbarchive::lbArchive_LoadSections(a0 as _, &v); } };
    fn lbVector_8000DE38(a0: *mut c_void, a1: *mut Vec, a2: f32) -> *mut Vec = |a0: *mut c_void, a1: *mut Vec, a2: f32| unsafe { melee_runtime::tu::lbvector::lbVector_8000DE38(a0 as _, a1 as _, a2 as _) as _ };
    fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, ids: &[i32]) -> i32;
    fn memzero(a0: *mut c_void, a1: i32) = super::memzero_default;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_TObjReqAnimAll(a0: *mut HSD_TObj, a1: f32) { super::HSD_TObjReqAnimAll(a0, a1) }
        pub unsafe extern "C" fn gm_80168BF8(a0: i32) -> f32 { super::gm_80168BF8(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::jobj_inlines!();

use melee_core::fp::{fmadds, fmsubs};
use melee_runtime::cabi::{f2i32, sdiv, shl, srem};

/// `AT_RE_HUD_STOCK` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_STOCK: i32 = 1;
/// `IFSTOCK_GENO_FRAME` (ifstock_geno_art.inc): the sentinel frame that names a package art.
const IFSTOCK_GENO_FRAME: f32 = 1000000.0;
const IFSTOCK_GENO_CACHE: usize = 12;
const IFSTOCK_GENO_BINDINGS: usize = 256;
const JOBJ_HIDDEN_U: u32 = JOBJ_HIDDEN as u32;

/// `memzero` (lb_00B0.c): the real behaviour, used unless a test installs a hook.
pub fn memzero_default(mem: *mut c_void, size: i32) {
    unsafe { core::ptr::write_bytes(mem as *mut u8, 0, size as usize) }
}

/// `memset` (libc): the real behaviour, used unless a test installs a hook.
pub fn memset_default(mem: *mut c_void, value: i32, size: u32) -> *mut c_void {
    unsafe { core::ptr::write_bytes(mem as *mut u8, value as u8, size as usize) };
    mem
}

/// `ifStatus_GetHUDInfo` (ifstatus.c, ported in `tu::ifstatus`).
pub fn hud_info_default() -> *mut HudIndex {
    unsafe { super::ifstatus::ifStatus_GetHUDInfo() }
}

/// `struct IfStockGenoArt` (ifstock_geno_art.inc): one cached package/letter art.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IfStockGenoArt {
    pub ck: i32,
    pub costume: i32,
    pub tried: i32,
    pub desc: *mut HSD_ImageDesc,
}

/// `struct IfStockGenoBinding` (ifstock_geno_art.inc): the original atlas of a stock TObj and its substitution table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IfStockGenoBinding {
    pub tobj: *mut HSD_TObj,
    pub original_image: *mut HSD_ImageDesc,
    pub original: *mut *mut HSD_ImageDesc,
    pub table: *mut *mut HSD_ImageDesc,
    pub count: i32,
}

/// 4A1378
pub static mut ifStock_804A1378: ifStock_804A1378 = unsafe { core::mem::zeroed() };
/// 4A1774
pub static mut ifStock_804A1774: ifStock_804A1774 = unsafe { core::mem::zeroed() };
/// 4A1A8C
pub static mut ifStock_804A1A8C: [*mut HSD_GObj; 16] = [null_mut(); 16];
/// 4A1ACC
pub static mut ifStock_804A1ACC: ifStock_804A1ACC = unsafe { core::mem::zeroed() };

pub static ifStock_SceneModels: [u8; 12] = *b"Stc_scemdls\0";

pub static mut ifStock_GenoArt: [IfStockGenoArt; IFSTOCK_GENO_CACHE] = unsafe { core::mem::zeroed() };
pub static mut ifStock_GenoBindings: [IfStockGenoBinding; IFSTOCK_GENO_BINDINGS] = unsafe { core::mem::zeroed() };

/// `x4[idx]` of a player record; `idx` may run past the eight `x4` slots into `x24..x44` (the
/// fields follow `x4` without padding in both layouts).
#[inline(always)]
unsafe fn x4(pp: *mut ifStock_804A1378_per_player, idx: usize) -> *mut HSD_JObj {
    *(pp as *mut u8).add(core::mem::offset_of!(ifStock_804A1378_per_player, x4)).cast::<*mut HSD_JObj>().add(idx)
}

/// `jobj->u.dobj->mobj->tobj`
#[inline(always)]
unsafe fn tobj_of(jobj: *mut HSD_JObj) -> *mut HSD_TObj {
    (*(*(*jobj).u.dobj).mobj).tobj
}

#[inline(always)]
unsafe fn GET_IFSTOCK(gobj: *mut HSD_GObj) -> *mut IfStockUserData {
    HSD_GObjGetUserData(gobj) as *mut IfStockUserData
}

#[inline(always)]
unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).user_data
}

// ---------------------------------------------------------------------------- pc/geno/geno_art_*

/// The first two words, or two letters of a single word. UI only.
pub unsafe fn genoArt_Initials(name: *const i8, out: *mut i8) {
    let mut n: i32 = 0;
    let mut start: i32 = 1;
    let mut second: i8 = 0;
    let mut i = 0usize;
    while *name.add(i) != 0 {
        let mut c = *name.add(i);
        let alnum = (c >= b'A' as i8 && c <= b'Z' as i8) || (c >= b'a' as i8 && c <= b'z' as i8) || (c >= b'0' as i8 && c <= b'9' as i8);
        if c >= b'a' as i8 && c <= b'z' as i8 {
            c = c.wrapping_sub((b'a' - b'A') as i8);
        }
        if alnum && start != 0 && n < 2 {
            *out.add(n as usize) = c;
            n += 1;
        } else if alnum && n == 1 && second == 0 {
            second = c;
        }
        start = !alnum as i32;
        i += 1;
    }
    if n == 1 && second != 0 {
        *out.add(n as usize) = second;
        n += 1;
    }
    if n == 0 {
        *out.add(n as usize) = b'?' as i8;
        n += 1;
    }
    *out.add(n as usize) = 0;
}

/// One key value of an FObj stream (fobj.c's own reader, restated): the format is the top 3 bits
/// of `frac`, the fixed-point shift the low 5.
pub unsafe fn genoArt_FObjValue(p: *mut *mut u8, frac: u8) -> f32 {
    let s = *p;
    let scale = 1i32.wrapping_shl((frac & 0x1F) as u32) as f32;
    match frac & 0xE0 {
        0x00 => {
            let u = ((*s as u32) << 24) | ((*s.add(1) as u32) << 16) | ((*s.add(2) as u32) << 8) | *s.add(3) as u32;
            *p = s.add(4);
            f32::from_bits(u)
        }
        0x20 => {
            *p = s.add(2);
            ((((*s as i32) << 8) | *s.add(1) as i32) as i16) as f32 / scale
        }
        0x40 => {
            *p = s.add(2);
            ((((*s as i32) << 8) | *s.add(1) as i32) as u16) as f32 / scale
        }
        0x60 => {
            *p = s.add(1);
            (*s as i8) as f32 / scale
        }
        _ => {
            *p = s.add(1);
            *s as f32 / scale
        }
    }
}

pub unsafe fn genoArt_FObjPacked(p: *mut *mut u8) -> u32 {
    let mut s = *p;
    let mut v: u32 = (*s & 0x7F) as u32;
    let mut shift: i32 = 7;
    loop {
        let cont = *s & 0x80 != 0;
        s = s.add(1);
        if !cont {
            break;
        }
        v |= shl((*s & 0x7F) as u32, shift as u32);
        shift += 7;
    }
    *p = s;
    v
}

/// Walk the texture-image track (HSD_A_T_TIMG) of `tobj`'s animation. Returns true when a key
/// sits exactly on `frame` (a frame with no key of its own just holds the previous name), and
/// stores the largest image index any key selects in *max_img (-1 when there is no track).
pub unsafe fn genoArt_ImageKeyAt(tobj: *mut HSD_TObj, frame: i32, max_img: *mut i32) -> bool {
    let mut hit = false;
    *max_img = -1;
    if tobj.is_null() || (*tobj).aobj.is_null() {
        return false;
    }
    let mut fo = (*(*tobj).aobj).fobj;
    while !fo.is_null() {
        if (*fo).obj_type as i32 != HSD_A_T_TIMG || (*fo).ad_head.is_null() {
            fo = (*fo).next;
            continue;
        }
        let mut p: *mut u8 = (*fo).ad_head;
        let end: *mut u8 = p.wrapping_add((*fo).length as usize);
        let mut t: f32 = (*fo).startframe as f32;
        while p < end {
            let mut b: u8 = *p;
            p = p.add(1);
            let op = (b & 0xF) as i32;
            let mut n: i32 = ((b >> 4) & 7) as i32;
            let mut shift: i32 = 3;
            while b & 0x80 != 0 {
                b = *p;
                p = p.add(1);
                n = n.wrapping_add(shl((b & 0x7F) as u32, shift as u32) as i32);
                shift += 7;
            }
            n = n.wrapping_add(1);
            while n > 0 {
                let mut v: f32 = 0.0;
                let mut wait: u32 = 0;
                let mut skip = false;
                match op {
                    HSD_A_OP_CON | HSD_A_OP_LIN | HSD_A_OP_SPL0 => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                        wait = genoArt_FObjPacked(&mut p);
                    }
                    HSD_A_OP_SPL => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                        genoArt_FObjValue(&mut p, (*fo).frac_slope);
                        wait = genoArt_FObjPacked(&mut p);
                    }
                    HSD_A_OP_SLP => {
                        genoArt_FObjValue(&mut p, (*fo).frac_slope);
                        skip = true;
                    }
                    HSD_A_OP_KEY => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                    }
                    _ => return hit,
                }
                if !skip {
                    if f2i32(t as f64) == frame {
                        hit = true;
                    }
                    if f2i32(v as f64) > *max_img {
                        *max_img = f2i32(v as f64);
                    }
                    t += wait as f32;
                }
                n -= 1;
            }
        }
        fo = (*fo).next;
    }
    hit
}

// ---------------------------------------------------------------------------- ifstock_geno_art.inc

pub unsafe fn ifStock_GenoReset() {
    memset_default(
        (&raw mut ifStock_GenoArt).cast(),
        0,
        core::mem::size_of::<[IfStockGenoArt; IFSTOCK_GENO_CACHE]>() as u32,
    );
    memset_default(
        (&raw mut ifStock_GenoBindings).cast(),
        0,
        core::mem::size_of::<[IfStockGenoBinding; IFSTOCK_GENO_BINDINGS]>() as u32,
    );
}

pub unsafe fn ifStock_GenoFind(ck: i32, costume: i32, current: *mut HSD_ImageDesc) -> *mut IfStockGenoArt {
    let mut handle: i32;
    let mut w: i32 = 0;
    let mut ht: i32 = 0;
    let mut size: i32 = 0;
    let mut fmt: i32 = -1;
    let mut a: *mut IfStockGenoArt = null_mut();
    let mut name: [i8; 48] = [0; 48];
    let mut abbr: [i8; 3] = [0; 3];
    for i in 0..IFSTOCK_GENO_CACHE {
        let e = &raw mut ifStock_GenoArt[i];
        if (*e).tried != 0 && (*e).ck == ck && (*e).costume == costume {
            return e;
        }
        if a.is_null() && (*e).tried == 0 {
            a = e;
        }
    }
    if a.is_null() {
        return null_mut();
    }
    (*a).tried = 1;
    (*a).ck = ck;
    (*a).costume = costume;
    handle = ext::Geno_DefineArtOpen(ck, 2, costume);
    if handle >= 0 {
        w = ext::GxTex_Width(handle);
        ht = ext::GxTex_Height(handle);
        size = ext::GxTex_ImageSize(handle);
        fmt = ext::GxTex_Format(handle);
        if fmt < 0 || fmt >= 8 || w < 1 || ht < 1 || w > 1024 || ht > 1024 || size < 1 {
            ext::GxTex_Close(handle);
            handle = -1;
        }
    }
    if handle < 0 {
        if ext::Geno_DefineArtBaseCK(ck) >= 0
            || current.is_null()
            || ext::Geno_DefineName(ck, name.as_mut_ptr(), core::mem::size_of_val(&name) as i32) == 0
        {
            return a;
        }
        w = (*current).width as i32;
        ht = (*current).height as i32;
        fmt = GX_TF_I4 as i32;
        if w < 1 || ht < 1 || w > 1024 || ht > 1024 {
            return a;
        }
        size = ((w + 7) / 8) * ((ht + 7) / 8) * 32;
        genoArt_Initials(name.as_ptr(), abbr.as_mut_ptr());
    }
    let allocation = ext::HSD_MemAlloc(size.wrapping_add(32)) as *mut u8;
    (*a).desc = ext::HSD_MemAlloc(core::mem::size_of::<HSD_ImageDesc>() as i32) as *mut HSD_ImageDesc;
    if allocation.is_null() || (*a).desc.is_null() {
        if !allocation.is_null() {
            ext::HSD_Free(allocation.cast());
        }
        if !(*a).desc.is_null() {
            ext::HSD_Free((*a).desc.cast());
        }
        (*a).desc = null_mut();
        if handle >= 0 {
            ext::GxTex_Close(handle);
        }
        return a;
    }
    let pix = ((allocation as usize).wrapping_add(31) & !31usize) as *mut u8;
    if handle >= 0 {
        ext::GxTex_CopyImage(handle, pix.cast());
        ext::GxTex_Close(handle);
    } else {
        memset_default(pix.cast(), 0, size as u32);
        ext::UI_TextI4(abbr.as_ptr(), w, ht, 0, pix.cast());
    }
    (*(*a).desc).image_ptr = pix.cast();
    (*(*a).desc).width = w as u16;
    (*(*a).desc).height = ht as u16;
    (*(*a).desc).format = fmt as _;
    (*(*a).desc).mipmap = 0;
    (*(*a).desc).minLOD = 0.0;
    (*(*a).desc).maxLOD = 0.0;
    // OSReport("geno: stock of ck %d costume %d: %s %dx%d\n", ...)
    a
}

/// Stock JObjs are reused (e.g. stock sharing). Restore the original atlas before
/// choosing a new fighter, and size substitutions from TIMG keys, not a fixed
/// 128-entry table which m-ex animations can index beyond.
pub unsafe fn ifStock_GenoBinding(tobj: *mut HSD_TObj, create: i32) -> *mut IfStockGenoBinding {
    let mut max_img: i32 = 0;
    let mut b: *mut IfStockGenoBinding = null_mut();
    for i in 0..IFSTOCK_GENO_BINDINGS {
        let e = &raw mut ifStock_GenoBindings[i];
        if (*e).tobj == tobj {
            b = e;
            break;
        }
        if b.is_null() && (*e).tobj.is_null() {
            b = e;
        }
    }
    if b.is_null() || (create == 0 && (*b).tobj.is_null()) {
        return null_mut();
    }
    if (*b).tobj.is_null() {
        (*b).tobj = tobj;
        (*b).original = (*tobj).imagetbl;
        (*b).original_image = (*tobj).imagedesc;
        genoArt_ImageKeyAt(tobj, -1, &mut max_img);
        if max_img < 0 {
            max_img = 0;
        }
        if max_img > 65535 {
            return b;
        }
        (*b).count = max_img + 1;
        (*b).table = ext::HSD_MemAlloc(
            (((*b).count + 1) as usize).wrapping_mul(core::mem::size_of::<*mut HSD_ImageDesc>()) as i32,
        ) as *mut *mut HSD_ImageDesc;
    }
    (*tobj).imagetbl = (*b).original;
    (*tobj).imagedesc = (*b).original_image;
    b
}

pub unsafe fn ifStock_ReqAnim(tobj: *mut HSD_TObj, mut frame: f32) {
    if tobj.is_null() {
        return;
    }
    let b = ifStock_GenoBinding(tobj, (frame >= IFSTOCK_GENO_FRAME) as i32);
    if frame >= IFSTOCK_GENO_FRAME {
        let v: i32 = f2i32((frame - IFSTOCK_GENO_FRAME) as f64);
        let ck = v / 256;
        let costume = v % 256;
        let base = if ext::Geno_DefineBaseCK(ck) >= 0 { ext::Geno_DefineArtBaseCK(ck) } else { ck };
        let a = ifStock_GenoFind(ck, costume, (*tobj).imagedesc);
        if !a.is_null() && !(*a).desc.is_null() && !b.is_null() && !(*b).table.is_null() {
            for i in 0..(*b).count as usize {
                *(*b).table.add(i) = (*a).desc;
            }
            *(*b).table.add((*b).count as usize) = null_mut();
            (*tobj).imagetbl = (*b).table;
            (*tobj).imagedesc = (*a).desc;
            frame = 0.0;
        } else {
            frame = if base >= 0 { ext::gm_80168B34(base as u32, 0, costume) } else { 0.0 };
        }
    }
    ext::HSD_TObjReqAnimAll(tobj, frame);
}

// ---------------------------------------------------------------------------- ifstock.c

pub unsafe fn ifStock_802F7EFC(arg0: i32, arg1: i32) -> i32 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let stock = &raw mut ifStock_804A1378;
    let arg0_data = (&raw mut (*stock).x204[arg0 as usize]) as *mut IfStockData;
    let arg1_data = (&raw mut (*stock).x204[arg1 as usize]) as *mut IfStockData;
    let slot: i32;
    if ext::Player_GetStocks(arg1) == 0 {
        return 1;
    }
    (*stock).player[arg1 as usize].stocks = ext::Player_GetStocks(arg1);
    if (*stock).player[arg1 as usize].stocks < 0 {
        (*stock).player[arg1 as usize].stocks = 1;
    }
    if (*arg1_data).x0[10] == 0 {
        slot = 5;
    } else if (*arg1_data).x0[11] == 0 {
        slot = 6;
    } else {
        return 2;
    }
    (*arg1_data).x0[(slot + 5) as usize] = 1;
    (*arg0_data).x0[2] = 0;
    (*arg1_data).x0[(slot - 2) as usize] = arg0 as u8;
    let j: i32;
    {
        let stocks = (*stock).player[arg1 as usize].stocks;
        if stocks <= 5 && stocks > 0 {
            (*arg1_data).x0[(stocks + 4) as usize] = 10;
        }
        let i = if stocks <= 5 && stocks > 0 { 1 } else { 0 };
        j = if i != 0 { stocks } else { 1 };
    }
    let pp1 = &raw mut (*stock).player[arg1 as usize];
    HSD_JObjGetTranslation(x4(pp1, 0), &mut pos);
    ext::HSD_JObjReqAnimAll(x4(pp1, j as usize), 0.0);
    ext::HSD_JObjAnimAll(x4(pp1, j as usize));
    let an = &raw mut (*arg1_data).anim[(slot - 5) as usize];
    HSD_JObjGetTranslation(x4(pp1, j as usize), &raw mut (*an).start);
    (*an).start.x += pos.x;
    (*an).start.y += pos.y;
    (*an).start.z += pos.z;
    let i = ext::Player_GetStocks(arg0) < 5 && ext::Player_GetStocks(arg0) > 0;
    let j: i32 = if i { ext::Player_GetStocks(arg0) } else { 0 };
    let pp0 = &raw mut (*stock).player[arg0 as usize];
    {
        let jobj = x4(pp0, 0);
        HSD_JObjGetTranslation(jobj, &mut pos);
    }
    ext::HSD_JObjReqAnimAll(x4(pp0, j as usize), 0.0);
    ext::HSD_JObjAnimAll(x4(pp0, j as usize));
    HSD_JObjGetTranslation(x4(pp0, 1), &raw mut (*an).end);
    (*an).end.x += fmadds(2.4, j as f32, pos.x);
    (*an).end.y = (*an).start.y;
    (*an).end.z += pos.z;
    (*an).mid.x = 0.5 * ((*an).end.x + (*an).start.x);
    (*an).mid.z = 0.5 * ((*an).end.z + (*an).start.z);
    (*an).mid.y = 10.0 + (*an).start.y;
    0
}

#[inline]
pub unsafe fn ifStock_802F8298_tobj_frame(player: u8) -> f32 {
    ext::gm_80168BF8(player as i32)
}

#[inline]
pub unsafe fn ifStock_802F8298_update_stocks(user_data: *mut IfStockUserData) -> *mut IfStockUserData {
    let stock = &raw mut ifStock_804A1378;
    let pl = &raw mut (*stock).player[(*user_data).player as usize];
    (*pl).stocks = ext::Player_GetStocks((*user_data).player as i32);
    if (*pl).stocks > 99 {
        (*pl).stocks = 99;
    }
    if (*pl).stocks < 0 {
        (*pl).stocks = 1;
    }
    user_data
}

#[inline]
pub unsafe fn ifStock_802F8298_step_anim(anim: *mut u8) {
    if *anim < 10 {
        *anim = (*anim).wrapping_add(1);
    }
}

pub unsafe extern "C" fn ifStock_802F8298(gobj: *mut HSD_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let stock = &raw mut ifStock_804A1378;
    let jobj_anim = jobj;
    let user_data = ifStock_802F8298_update_stocks((*gobj).user_data as *mut IfStockUserData);
    let player = (*user_data).player;
    let pl = &raw mut (*stock).player[player as usize];
    // ifStock_802F8298_data: the per-player animation element
    let data = &raw mut (*stock).x204[player as usize];
    let mut vec_a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec_b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec_c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec_d = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if (*pl).stocks <= 5 {
        ext::HSD_JObjSetFlagsAll((*pl).x3C, JOBJ_HIDDEN_U);
        for i in 0..5usize {
            let jobj2 = x4(pl, i + 1);
            if (i as i32) < (*pl).stocks {
                ext::HSD_JObjClearFlagsAll(jobj2, JOBJ_HIDDEN_U);
                if (*data).flag != 0 {
                    (*data).anim[i] = 0;
                } else {
                    (*data).anim[i] = 10;
                }
            } else {
                if (*data).flag == 0 {
                    (*data).anim[i] = 10;
                }
                if (*data).anim[i] == 0 {
                    HSD_JObjGetTranslation2(jobj, &mut vec_a);
                    HSD_JObjGetTranslation2(x4(pl, i + 1), &mut vec_b);
                    vec_b.x += vec_a.x;
                    vec_b.y += vec_a.y;
                    vec_b.z += vec_a.z;
                    ext::efSync_Spawn(0x474, gobj, &mut vec_b);
                }
                ifStock_802F8298_step_anim(&raw mut (*data).anim[i]);
            }
            ext::HSD_JObjReqAnimAll(jobj2, (*data).anim[i] as f32);
            ifStock_ReqAnim(tobj_of(jobj2), ifStock_802F8298_tobj_frame(player));
            ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
        }
    } else {
        ext::HSD_JObjClearFlagsAll((*pl).x3C, JOBJ_HIDDEN_U);
        if (*pl).stocks >= 10 {
            ext::HSD_JObjReqAnimAll((*pl).x44, ((*pl).stocks / 10) as f32);
            ext::HSD_JObjReqAnimAll((*pl).x40, ((*pl).stocks % 10) as f32);
        } else {
            ext::HSD_JObjReqAnimAll((*pl).x44, (*pl).stocks as f32);
            ext::HSD_JObjSetFlagsAll((*pl).x40, JOBJ_HIDDEN_U);
        }
        for i in 0..5usize {
            let jobj2 = x4(pl, i + 1);
            if i == 0 {
                (*data).anim[i] = 0;
            } else {
                (*data).anim[i] = 10;
            }
            ext::HSD_JObjReqAnimAll(jobj2, (*data).anim[i] as f32);
            ifStock_ReqAnim(tobj_of(jobj2), ifStock_802F8298_tobj_frame(player));
            ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
        }
    }
    for i in 5..=6usize {
        let steal_jobj = x4(pl, i + 1);
        if (*data).anim[i] == 0 {
            ext::HSD_JObjSetFlagsAll(steal_jobj, JOBJ_HIDDEN_U);
        } else {
            ext::HSD_JObjClearFlagsAll(steal_jobj, JOBJ_HIDDEN_U);
            let p = &raw mut (*data).anim[i];
            if *p <= 10 {
                ext::lbVector_8000DE38(
                    (&raw mut (*data).steal[i - 5]).cast(),
                    &mut vec_c,
                    0.1 * (*data).anim[i] as f32,
                );
                HSD_JObjGetTranslation2(x4(pl, 0), &mut vec_d);
                vec_c.x -= vec_d.x;
                vec_c.y -= vec_d.y;
                vec_c.z -= vec_d.z;
                HSD_JObjSetTranslate(steal_jobj, &mut vec_c);
                if (*data).anim[i] == 1 {
                    vec_c.x = (*data).steal[i - 5].start.x;
                    vec_c.y = (*data).steal[i - 5].start.y;
                    ext::efSync_Spawn(0x475, gobj, &mut vec_c);
                } else if (*data).anim[i] == 10 {
                    vec_c.x = (*data).steal[i - 5].end.x;
                    vec_c.y = (*data).steal[i - 5].end.y;
                    ext::efSync_Spawn(0x476, gobj, &mut vec_c);
                }
                (*data).anim[i] = (*data).anim[i].wrapping_add(1);
            } else {
                *p = 0;
                let other = &raw mut (*stock).x204[(*data).x3[i - 5] as usize];
                (*other).flag = 1;
            }
        }
        ifStock_ReqAnim(tobj_of(steal_jobj), ifStock_802F8298_tobj_frame(player));
        ext::HSD_AObjSetRate((*tobj_of(steal_jobj)).aobj, 0.0);
    }
    ext::HSD_JObjAnimAll(jobj_anim);
}

#[inline]
pub unsafe fn ifStock_802F89F8_get_data(gobj: *mut HSD_GObj) -> *mut IfStockUserData {
    GET_IFSTOCK(gobj)
}

#[inline]
pub fn ifStock_802F89F8_count(mut coins: i32) -> i32 {
    for i in 0..32 {
        if coins == 0 {
            return i;
        }
        coins /= 10;
    }
    0
}

#[inline]
pub fn ifStock_802F89F8_divisor(position: i32) -> i32 {
    let mut divisor: i32 = 1;
    if position != 0 {
        let mut i = 0;
        while i < position {
            divisor = divisor.wrapping_mul(10);
            i += 1;
        }
    }
    divisor
}

#[inline]
pub fn ifStock_802F89F8_digit(mut coins: i32, position: i32) -> i32 {
    let mut divisor: i32 = 1;
    if position - 1 == 0 {
        coins %= 10;
        return coins;
    }
    let mut i = 0;
    while i < position - 1 {
        divisor = divisor.wrapping_mul(10);
        i += 1;
    }
    srem(sdiv(coins, divisor), 10)
}

pub unsafe extern "C" fn ifStock_802F89F8(gobj: *mut HSD_GObj) {
    let user_data = ifStock_802F89F8_get_data(gobj);
    let player = (*user_data).player as i32;
    let pl = &raw mut ifStock_804A1378.player[player as usize];
    let jobj2 = x4(pl, 1);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Player_GetCoins(player);
    let pl = &raw mut ifStock_804A1378.player[(*user_data).player as usize];
    (*pl).coins = ext::Player_GetCoins((*user_data).player as i32);
    let mut coins = (*pl).coins;
    if coins as u32 > 99999 {
        coins = 99999;
    }
    let mut count = ifStock_802F89F8_count(coins);
    if count > 5 {
        count = 5;
    }
    ext::HSD_JObjReqAnimAll(jobj2, 0.0);
    ifStock_ReqAnim(tobj_of(jobj2), ext::gm_80168BF8((*user_data).player as i32));
    ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
    for i in 0..5i32 {
        if i < count {
            let _divisor = ifStock_802F89F8_divisor(i);
            ext::HSD_JObjClearFlagsAll(x4(pl, (13 - i) as usize), JOBJ_HIDDEN_U);
            let digit = ifStock_802F89F8_digit(coins, count - i);
            ext::HSD_JObjReqAnimAll(x4(pl, (13 - i) as usize), digit as f32);
        } else {
            ext::HSD_JObjSetFlagsAll(x4(pl, (13 - i) as usize), JOBJ_HIDDEN_U);
            if count == 0 && i == 0 {
                ext::HSD_JObjReqAnimAll((*pl).x38, 0.0);
                ext::HSD_JObjClearFlagsAll((*pl).x38, JOBJ_HIDDEN_U);
            }
        }
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_802F8DB0(gobj: *mut HSD_GObj) {
    let x = ext::gm_801694A0(gobj);
    if x != ifStock_804A1774.x108 {
        ifStock_802FA2D0(x);
    }
    ifStock_804A1774.x108 = x;
}

pub unsafe extern "C" fn fn_802F8E08(_gobj: *mut HSD_GObj) {
    let stock = &raw mut ifStock_804A1774;
    let mut counter: i32 = 0;
    let mut x: f32 = -21.0;
    let mut y: f32 = 11.0;
    let mut thing = ext::gm_80169520() as *mut i8;
    let mut i = 0usize;
    while i < 130 {
        let mut gobj = (*stock).x10C[i + 1];
        if gobj.is_null() {
            break;
        }
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        (*stock).x1[i] = *thing;
        if (*stock).x1[i] == -1 {
            HSD_JObjSetTranslateZ(jobj, 10000.0);
        } else if (*stock).x1[i] == -2 {
            for _j in i..130usize {
                gobj = (*stock).x10C[i + 1];
                if !gobj.is_null() {
                    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
                    HSD_JObjSetTranslateZ(jobj, 10000.0);
                }
            }
            return;
        } else {
            HSD_JObjSetTranslateZ(jobj, 0.0);
            HSD_JObjSetTranslateX(jobj, x);
            HSD_JObjSetTranslateY(jobj, y);
            counter += 1;
            x += 2.45;
            if counter % 5 == 0 {
                x = -21.0;
                y -= 2.45;
            }
        }
        i += 1;
        thing = thing.add(1);
    }
}

pub unsafe extern "C" fn fn_802F916C(_gobj: *mut HSD_GObj) {
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    let mut counter: i32 = 0;
    let mut x: f32 = -21.0;
    let mut y: f32 = 11.0;
    ext::Ground_801C1DE4(&mut a, &mut b);
    for i in 0..16i32 {
        if i >= b {
            break;
        } else {
            let gobj = ifStock_804A1A8C[i as usize];
            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            if i >= a {
                HSD_JObjSetTranslateZ(jobj, 10000.0);
            } else {
                HSD_JObjSetTranslateZ(jobj, 0.0);
                HSD_JObjSetTranslateX(jobj, x);
                HSD_JObjSetTranslateY(jobj, y);
                counter += 1;
                x += 2.45;
                if counter % 5 == 0 {
                    x = -21.0;
                    y -= 2.45;
                }
            }
        }
    }
}

#[inline]
pub unsafe fn fn_802F9410_inline(gobj: *mut HSD_GObj, p: *mut IfStockUserData) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let jobj2 = x4(&raw mut ifStock_804A1378.player[(*p).player as usize], 1);
    ext::HSD_JObjReqAnimAll(jobj2, 0.0);
    ifStock_ReqAnim(tobj_of(jobj2), ext::gm_80168BF8((*p).player as i32));
    ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_802F9410(gobj: *mut HSD_GObj) {
    let p = GET_IFSTOCK(gobj);
    match (*p).mode {
        0 => ifStock_802F8298(gobj),
        1 => fn_802F9410_inline(gobj, p),
        2 => ifStock_802F89F8(gobj),
        _ => {}
    }
}

pub unsafe extern "C" fn fn_802F94E0(gobj: *mut HSD_GObj, renderpass: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_STOCK) != 0 {
        return;
    }
    let p = GET_IFSTOCK(gobj);
    let x = ext::ifStatus_GetHUDInfo();
    if (*x).players[(*p).player as usize].flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, renderpass);
    }
}

pub unsafe extern "C" fn fn_802F9548(gobj: *mut HSD_GObj, renderpass: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_STOCK) != 0 {
        return;
    }
    let x = ext::ifStatus_GetHUDInfo();
    if (*x).players[0].flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, renderpass);
    }
}

pub unsafe extern "C" fn fn_802F9598(gobj: *mut HSD_GObj, renderpass: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_STOCK) != 0 {
        return;
    }
    let x = ext::ifStatus_GetHUDInfo();
    if (*x).players[0].flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, renderpass);
    }
}

pub unsafe extern "C" fn fn_802F95E8(gobj: *mut HSD_GObj, renderpass: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_STOCK) != 0 {
        return;
    }
    let x = ext::ifStatus_GetHUDInfo();
    if ext::gm_8016B184() != 0
        && ext::gm_8016A1F8() != 0
        && ((*x).players[0].flags.hide_all_digits()
            | (*x).players[1].flags.hide_all_digits()
            | (*x).players[2].flags.hide_all_digits()
            | (*x).players[3].flags.hide_all_digits())
            == 0
    {
        ext::HSD_GObj_JObjCallback(gobj, renderpass);
    }
}

pub unsafe extern "C" fn fn_802F9680(gobj: *mut HSD_GObj, renderpass: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_STOCK) != 0 {
        return;
    }
    let x = ext::ifStatus_GetHUDInfo();
    if (*x).players[0].flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, renderpass);
    }
}

/// m-ex "Replace ... Stock Matanim": point one stock-icon joint at Stc_icns (no-op on retail).
pub unsafe fn ifStock_MexAttach(jobj: *mut HSD_JObj) {
    let stc = ext::gm_MexStockMatAnim();
    if !jobj.is_null() && !stc.is_null() {
        ext::HSD_JObjAddAnimAll(jobj, null_mut(), stc, null_mut());
    }
}

/// m-ex "All Star / Multispawn - Change Stock Frame": retail passes arg1 = 0 (ignored by its
/// formula); m-ex needs the fighter, ftMapping_list[ckind].internal_id.
pub unsafe fn ifStock_MexFrame(ckind: i32, costume: i32) -> f32 {
    let geno = ext::gm_GenoStockFrame(ckind, costume);
    if geno >= 0.0 {
        return geno;
    }
    let mex = ext::gm_MexStockFrame(ext::Player_800325C8(ckind as u32, 0) as i32, costume);
    if mex >= 0.0 {
        mex
    } else {
        ext::gm_80168B34(ckind as u32, 0, costume)
    }
}

pub unsafe fn ifStock_802F96D0(a: i32, b: i32, x: f32, y: f32) -> *mut HSD_GObj {
    let q = &raw mut ifStock_804A1378;
    let gobj = ext::GObj_Create(14, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint((**(*q).x0).joint);
    let mut jobj2: *mut HSD_JObj = null_mut();
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(fn_802F9680), 11, 0);
    ext::gm_8016895C(jobj, *(*q).x0, 0);
    ext::lb_80011E24(jobj, &mut jobj2, &[1]);
    ifStock_MexAttach(jobj2);
    HSD_JObjSetTranslateX(jobj, x);
    HSD_JObjSetTranslateY(jobj, y);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN_U);
    ext::HSD_JObjClearFlags(jobj2, JOBJ_HIDDEN_U);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ifStock_ReqAnim(tobj_of(jobj2), ifStock_MexFrame(a, b));
    ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    gobj
}

#[inline]
pub unsafe fn ifStock_802F98E8_get_match_info(data: *mut VsSceneState, player: i32) -> *mut lbl_8046B6A0_FighterMatchInfoFlags {
    &raw mut (*data).fighters[player as usize].anon_4.flags
}

const ifStock_802F98E8_color1: GXColor = GXColor { r: 0x08, g: 0x08, b: 0x08, a: 0x80 };
const ifStock_802F98E8_color2: GXColor = GXColor { r: 0x3C, g: 0x3C, b: 0x46, a: 0x80 };

pub unsafe extern "C" fn ifStock_802F98E8(player: u8, b: u8) {
    let stock = &raw mut ifStock_804A1378;
    let pn = player as usize;
    if !(*stock).x0.is_null() {
        let data = &raw mut (*stock).x204[pn];
        // ifStock_802F98E8_get_data(stock, player)[0] = player;
        (*data).player = player;
        (*stock).x204[pn].mode = b;
        (*stock).x204[pn].flag = 1;
        if !(*stock).player[pn].x0.is_null() {
            ext::HSD_GObjFree((*stock).player[pn].x0);
        }
        let gobj = ext::GObj_Create(14, 15, 0);
        (*gobj).user_data = data.cast();
        if !gobj.is_null() {
            let jobj = ext::HSD_JObjLoadJoint((**(*stock).x0).joint);
            if !jobj.is_null() {
                ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
                ext::GObj_SetupGXLink(gobj, Some(fn_802F94E0), 11, 0);
                ext::gm_8016895C(jobj, *(*stock).x0, 0);
                ext::HSD_JObjReqAnimAll(jobj, 0.0);
                ext::HSD_GObj_SetupProc(gobj, Some(fn_802F9410), 17);
                HSD_JObjSetTranslate(jobj, ext::ifAll_GetPlayerHUDPosition(player as i32));
                let pl = &raw mut ifStock_804A1378.player[pn];
                ext::lb_80011E24(
                    jobj,
                    (*pl).x4.as_mut_ptr(),
                    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                );
                // m-ex: the seven stock-icon joints (x4[1..7]) use Stc_icns
                for i in 1..=7usize {
                    ifStock_MexAttach(x4(pl, i));
                }
                (*stock).player[pn].x0 = gobj;
                (*pl).coins = ext::Player_GetCoins(player as i32);
                (*pl).stocks = ext::Player_GetStocks(player as i32);
                if (*pl).stocks < 0 {
                    (*pl).stocks = 1;
                }
                match b {
                    0 => {
                        ext::HSD_JObjSetFlagsAll((*pl).x24, JOBJ_HIDDEN_U);
                        if (*pl).stocks <= 5 {
                            ext::HSD_JObjSetFlagsAll((*pl).x3C, JOBJ_HIDDEN_U);
                            for i in 0..7usize {
                                let icon_jobj = x4(pl, i + 1);
                                if (i as i32) < (*pl).stocks || i >= 5 {
                                    (*data).anim[i] = 0;
                                    if i >= 5 {
                                        ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                                    }
                                } else {
                                    (*data).anim[i] = 10;
                                    ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                                }
                                ext::HSD_JObjReqAnimAll(icon_jobj, (*data).anim[i] as f32);
                                ifStock_ReqAnim(tobj_of(icon_jobj), ext::gm_80168BF8(player as i32));
                                ext::HSD_AObjSetRate((*tobj_of(icon_jobj)).aobj, 0.0);
                            }
                        }
                    }
                    1 => {
                        ext::HSD_JObjSetFlagsAll((*pl).x24, JOBJ_HIDDEN_U);
                        ext::HSD_JObjSetFlagsAll((*pl).x3C, JOBJ_HIDDEN_U);
                        for i in 0..7usize {
                            let icon_jobj = x4(pl, i + 1);
                            if i == 0 {
                                (*data).anim[i] = 0;
                            } else {
                                (*data).anim[i] = 10;
                            }
                            if i < 5 {
                                ext::HSD_JObjReqAnimAll(icon_jobj, (*data).anim[i] as f32);
                                ifStock_ReqAnim(tobj_of(icon_jobj), ext::gm_80168BF8(player as i32));
                                ext::HSD_AObjSetRate((*tobj_of(icon_jobj)).aobj, 0.0);
                                if i != 0 {
                                    ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                                }
                            } else {
                                ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                            }
                        }
                    }
                    2 => {
                        ext::HSD_JObjSetFlagsAll((*pl).x28, JOBJ_HIDDEN_U);
                        ext::HSD_JObjSetFlagsAll((*pl).x2C, JOBJ_HIDDEN_U);
                        ext::HSD_JObjSetFlagsAll((*pl).x30, JOBJ_HIDDEN_U);
                        ext::HSD_JObjSetFlagsAll((*pl).x34, JOBJ_HIDDEN_U);
                        ext::HSD_JObjSetFlagsAll((*pl).x3C, JOBJ_HIDDEN_U);
                        for i in 0..7usize {
                            let icon_jobj = x4(pl, i + 1);
                            if i == 0 {
                                (*data).anim[i] = 0;
                            } else {
                                (*data).anim[i] = 10;
                            }
                            if i < 5 {
                                ext::HSD_JObjReqAnimAll(icon_jobj, (*data).anim[i] as f32);
                                ifStock_ReqAnim(tobj_of(icon_jobj), ext::gm_80168BF8(player as i32));
                                ext::HSD_AObjSetRate((*tobj_of(icon_jobj)).aobj, 0.0);
                                if i != 0 {
                                    ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                                }
                            } else {
                                ext::HSD_JObjSetFlagsAll(icon_jobj, JOBJ_HIDDEN_U);
                            }
                        }
                    }
                    _ => {}
                }
                ext::HSD_JObjAnimAll(jobj);
                let scene_state = ext::gmVs_GetSceneState();
                {
                    let match_info = ifStock_802F98E8_get_match_info(scene_state, player as i32);
                    if (*match_info).x4_b1() != 0 {
                        let mut c1 = ifStock_802F98E8_color1;
                        ifStock_802FB4EC(player as i32, &mut c1);
                    } else if (*match_info).x4_b0() != 0 {
                        let mut c2 = ifStock_802F98E8_color2;
                        ifStock_802FB4EC(player as i32, &mut c2);
                    }
                }
                fn_802F9410(gobj); // inlined
            } else {
                ext::HSD_GObjFree(gobj);
            }
        }
    }
}

#[inline]
pub unsafe fn ifStock_802F9F48_inline(arg: i32) -> *mut HSD_GObj {
    let q = &raw mut ifStock_804A1378;
    let gobj = ext::GObj_Create(14, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint((**(*q).x0).joint);
    let mut jobj2: *mut HSD_JObj = null_mut();
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(fn_802F95E8), 11, 0);
    ext::gm_8016895C(jobj, *(*q).x0, 0);
    ext::lb_80011E24(jobj, &mut jobj2, &[1]);
    ifStock_MexAttach(jobj2);
    if arg == 0 {
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802F8E08), 17);
    }
    HSD_JObjSetTranslateZ(jobj, 10000.0);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN_U);
    ext::HSD_JObjClearFlags(jobj2, JOBJ_HIDDEN_U);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ifStock_ReqAnim(
        tobj_of(jobj2),
        ifStock_MexFrame(ifStock_804A1774.x83[arg as usize] as i32, ifStock_804A1774.x1[arg as usize] as i32),
    );
    ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    gobj
}

pub unsafe fn ifStock_802F9F48(arg: i32) -> *mut HSD_GObj {
    ifStock_802F9F48_inline(arg)
}

pub unsafe fn ifStock_802FA118(arg: i32) -> *mut HSD_GObj {
    let q = &raw mut ifStock_804A1378;
    let gobj = ext::GObj_Create(14, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint((**(*q).x0).joint);
    let mut jobj2: *mut HSD_JObj = null_mut();
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(fn_802F9548), 11, 0);
    ext::gm_8016895C(jobj, *(*q).x0, 0);
    ext::lb_80011E24(jobj, &mut jobj2, &[1]);
    if arg == 0 {
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802F916C), 17);
    }
    HSD_JObjSetTranslateZ(jobj, 10000.0);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN_U);
    ext::HSD_JObjClearFlags(jobj2, JOBJ_HIDDEN_U);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ifStock_ReqAnim(tobj_of(jobj2), 57.0);
    ext::HSD_AObjSetRate((*tobj_of(jobj2)).aobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    gobj
}

pub unsafe fn ifStock_802FA2D0(mut arg: i32) {
    let s = &raw mut ifStock_804A1378;
    let jobj = (*s).jobj;
    if arg > 9999 {
        arg = 9999;
    } else if arg < 0 {
        arg = 0;
    }
    ext::HSD_JObjReqAnimAll(jobj, 10.0);
    if arg >= 1000 {
        ext::HSD_JObjReqAnimAll((*s).jobj_a, (arg % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_b, ((arg / 10) % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_c, ((arg / 100) % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_d, (arg / 1000) as f32);
    } else if arg >= 100 {
        ext::HSD_JObjReqAnimAll((*s).jobj_b, (arg % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_c, ((arg / 10) % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_d, (arg / 100) as f32);
    } else if arg >= 10 {
        ext::HSD_JObjReqAnimAll((*s).jobj_c, (arg % 10) as f32);
        ext::HSD_JObjReqAnimAll((*s).jobj_d, (arg / 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll((*s).jobj_d, arg as f32);
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe fn ifStock_802FA5BC(arg: i32) {
    let q = &raw mut ifStock_804A1378;
    let gobj = ext::GObj_Create(14, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint((*(*q).x4).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(fn_802F9598), 11, 0);
    if arg != 0 {
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802F8DB0), 17);
    }
    ext::gm_8016895C(jobj, (*q).x4, 0);
    ext::HSD_JObjReqAnimAll(jobj, 10.0);
    ext::HSD_JObjAnimAll(jobj);
    ifStock_804A1378.gobj = gobj;
    ext::lb_80011E24(jobj, &raw mut ifStock_804A1378.jobj, &[0, 1, 2, 3, 4, 5]);
}

pub unsafe extern "C" fn fn_802FA6C4(_arg: *mut HSD_GObj) {
    let s = &raw mut ifStock_804A1774;
    if ext::gm_8016B184() != 0 && ext::gm_8016A1F8() != 0 {
        if (*s).x0 == 0 {
            (*s).x0 = 1;
            let mut q = ext::gm_80169520() as *mut i8;
            let mut w = ext::gm_80169530() as *mut i8;
            for k in 0..130usize {
                (*s).x10C[k + 1] = null_mut();
                (*s).x1[k] = *q;
                q = q.add(1);
                (*s).x83[k] = *w;
                w = w.add(1);
            }
            let mut i = 0usize;
            while i < 130 {
                if !(*s).x10C[1 + i].is_null() {
                    ext::HSD_GObjFree((*s).x10C[1 + i]);
                }
                if (*s).x1[i] == -2 {
                    return;
                }
                (*s).x10C[1 + i] = ifStock_802F9F48(i as i32);
                i += 1;
            }
        }
    } else {
        if (*s).x0 == 1 {
            for j in (0..=129usize).rev() {
                if !(*s).x10C[1 + j].is_null() {
                    ext::HSD_GObjFree((*s).x10C[1 + j]);
                    (*s).x10C[1 + j] = null_mut();
                }
            }
        }
        (*s).x0 = 0;
    }
}

pub unsafe extern "C" fn fn_802FA8C0(_arg: *mut HSD_GObj) {
    let stock = &raw mut ifStock_804A1ACC;
    let mut k: i32 = 0;
    let mut x: f32 = -21.0;
    let mut y: f32 = 11.0;
    let mut a97c = ext::gm_8016A97C() as *mut i8;
    let mut a98c = ext::gm_8016A98C() as *mut i8;
    let mut i = 0usize;
    while i < 0x82 {
        let gobj = (*stock).x10C[i];
        if gobj.is_null() {
            return;
        }
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        (*stock).x83[i] = *a97c;
        (*stock).x1[i] = *a98c;
        if (*stock).x83[i] == -1 {
            HSD_JObjSetTranslateZ(jobj, 10000.0);
        } else if (*stock).x83[i] == -2 {
            for _j in i..0x82usize {
                let g = (*stock).x10C[i];
                if !g.is_null() {
                    HSD_JObjSetTranslateZ((*g).hsd_obj as *mut HSD_JObj, 10000.0);
                }
            }
            return;
        } else {
            HSD_JObjSetTranslateZ(jobj, 0.0);
            HSD_JObjSetTranslateX(jobj, x);
            HSD_JObjSetTranslateY(jobj, y);
            k += 1;
            x += 2.45;
            if k % 5 == 0 {
                x = -21.0;
                y -= 2.45;
            }
        }
        i += 1;
        a97c = a97c.add(1);
        a98c = a98c.add(1);
    }
}

pub unsafe extern "C" fn fn_802FAC34(_arg: *mut HSD_GObj) {
    let s = &raw mut ifStock_804A1ACC;
    if ext::gm_8016A944() != 0 {
        if (*s).x0 == 0 {
            (*s).x0 = 1;
            let mut q = ext::gm_8016A97C() as *mut i8;
            let mut w = ext::gm_8016A98C() as *mut i8;
            for i in 0..130usize {
                (*s).x10C[i] = null_mut();
                (*s).x83[i] = *q;
                q = q.add(1);
                (*s).x1[i] = *w;
                w = w.add(1);
            }
            let mut n: i32 = 0;
            while n < 130 {
                let nu = n as usize;
                if !(*s).x10C[nu].is_null() {
                    ext::HSD_GObjFree((*s).x10C[nu]);
                }
                if (*s).x83[nu] == -2 {
                    return;
                }
                if (*s).x1[nu] == -2 {
                    return;
                }
                (*s).x10C[nu] = ifStock_802F96D0(
                    (*s).x1[nu] as i32,
                    (*s).x83[nu] as i32,
                    fmadds(2.45, (n % 5) as f32, -21.0),
                    -fmsubs(2.45, (n / 5) as f32, 11.0),
                );
                if n == 0 {
                    ext::HSD_GObj_SetupProc((*s).x10C[nu], Some(fn_802FA8C0), 17);
                }
                n += 1;
            }
        }
    } else {
        if (*s).x0 == 1 {
            for i in (0..=129usize).rev() {
                if !(*s).x10C[i].is_null() {
                    ext::HSD_GObjFree((*s).x10C[i]);
                    (*s).x10C[i] = null_mut();
                }
            }
        }
        (*s).x0 = 0;
    }
}

#[inline]
pub unsafe fn ifStock_802FA5BC_noinline(arg: i32) {
    ifStock_802FA5BC(arg);
}

#[inline]
pub unsafe fn ifStock_ClearGObjSlots(slots: *mut *mut HSD_GObj, count: i32) {
    for i in 0..count.max(0) as usize {
        *slots.add(i) = null_mut();
    }
}

#[inline]
pub unsafe fn ifStock_CreateStockGObjs(mut slots: *mut *mut HSD_GObj) {
    let stock = &raw mut ifStock_804A1378;
    for i in 0..16i32 {
        *slots = null_mut();
        if ext::gm_8016B1A8() != 0 {
            *slots = ifStock_802FA118(i);
        }
        slots = slots.add(1);
    }
    if ext::gm_IsMultimanSmashMode() != 0 {
        let gobj = ext::GObj_Create(14, 15, 0);
        let jobj = ext::HSD_JObjLoadJoint((*(*stock).x4).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(gobj, Some(fn_802F9598), 11, 0);
        ext::gm_8016895C(jobj, (*stock).x4, 0);
        ext::HSD_JObjReqAnimAll(jobj, 10.0);
        ext::HSD_JObjAnimAll(jobj);
        (*stock).gobj = gobj;
        ext::lb_80011E24(jobj, &raw mut (*stock).jobj, &[0, 1, 2, 3, 4, 5]);
    }
}

pub unsafe fn ifStock_802FAEC4() {
    let stock = &raw mut ifStock_804A1378;
    let mut scene_models: *mut *mut DynamicModelDesc = null_mut();
    ifStock_GenoReset();
    ext::memzero(
        stock.cast(),
        (core::mem::size_of::<ifStock_804A1378>() - core::mem::size_of_val(&(*stock).x204)) as i32,
    );
    ext::memzero((&raw mut ifStock_804A1ACC).cast(), core::mem::size_of::<ifStock_804A1ACC>() as i32);
    ext::memzero((&raw mut ifStock_804A1A8C).cast(), core::mem::size_of::<[*mut HSD_GObj; 16]>() as i32);
    ext::memzero((&raw mut ifStock_804A1774).cast(), core::mem::size_of::<ifStock_804A1774>() as i32);
    ext::lbArchive_LoadSections(
        *ext::ifAll_GetArchive(),
        (&raw mut scene_models).cast(),
        ifStock_SceneModels.as_ptr(),
        &[],
    );
    (*stock).x0 = scene_models;
    (*stock).x4 = *scene_models.add(1);
    ifStock_804A1ACC.x108 = null_mut();
    ifStock_804A1ACC.x0 = 0;
    let mut gobj = ext::GObj_Create(14, 15, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_802FAC34), 17);
    ifStock_804A1ACC.x108 = gobj;
    ifStock_ClearGObjSlots(&raw mut ifStock_804A1ACC.x10C[0], 130);
    if ext::gm_80169394() == 0 {
        ifStock_804A1774.x10C[0] = null_mut();
        ifStock_804A1774.x0 = 0;
        gobj = ext::GObj_Create(14, 15, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802FA6C4), 17);
        ifStock_804A1774.x10C[0] = gobj;
        ifStock_ClearGObjSlots(&raw mut ifStock_804A1774.x10C[1], 130);
    } else if ext::gm_8016A1F8() != 0 {
        ifStock_804A1774.x108 = -1;
        ifStock_802FA5BC_noinline(1);
    }
    ifStock_CreateStockGObjs((&raw mut ifStock_804A1A8C).cast());
}

pub unsafe fn ifStock_802FB390() {
    if !ifStock_804A1774.x10C[0].is_null() {
        ext::HSD_GObjFree(ifStock_804A1774.x10C[0]);
        ifStock_804A1774.x10C[0] = null_mut();
    }
    for i in (0..=129usize).rev() {
        if !ifStock_804A1774.x10C[i + 1].is_null() {
            ext::HSD_GObjFree(ifStock_804A1774.x10C[i + 1]);
            ifStock_804A1774.x10C[i + 1] = null_mut();
        }
    }
}

pub unsafe fn ifStock_802FB41C() {
    for i in 0..16usize {
        if !ifStock_804A1A8C[i].is_null() {
            ext::HSD_GObjFree(ifStock_804A1A8C[i]);
            ifStock_804A1A8C[i] = null_mut();
        }
    }
}

pub unsafe fn ifStock_802FB484() {
    for i in 0..16usize {
        if !ifStock_804A1ACC.x10C[i].is_null() {
            ext::HSD_GObjFree(ifStock_804A1ACC.x10C[i]);
            ifStock_804A1ACC.x10C[i] = null_mut();
        }
    }
}

#[inline]
pub unsafe fn copyColor(q: *mut HSD_MObj, arg: *mut GXColor) {
    (*(*q).mat).diffuse.r = (*arg).r;
    (*(*q).mat).diffuse.g = (*arg).g;
    (*(*q).mat).diffuse.b = (*arg).b;
}

pub unsafe fn ifStock_802FB4EC(player: i32, arg: *mut GXColor) {
    for i in 0..7usize {
        copyColor(
            (*(*x4(&raw mut ifStock_804A1378.player[player as usize], i + 1)).u.dobj).mobj,
            arg,
        );
    }
}

pub unsafe fn ifStock_802FB650(player: i32) {
    let x = &raw mut ifStock_804A1378;
    if !(*x).player[player as usize].x0.is_null() {
        ext::HSD_GObjFree((*x).player[player as usize].x0);
    }
    ext::memzero(
        (&raw mut (*x).player[player as usize]).cast(),
        core::mem::size_of::<ifStock_804A1378_per_player>() as i32,
    );
}

pub unsafe fn ifStock_802FB6AC(player: i32) {
    let x = &raw mut ifStock_804A1378;
    if !(*x).player[player as usize].x0.is_null() {
        ext::HSD_GObj_80390CAC((*x).player[player as usize].x0);
    }
}

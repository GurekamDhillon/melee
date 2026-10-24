//! Typed adapters from the call shapes the character TUs declare (`tu/*/ext`, `stubs.rs`) to the
//! ports that already exist in the crates below `melee_chara`. A stub whose hook is unset calls the
//! adapter instead of panicking (see `port_hook!`). Every adapter only converts types; the
//! behaviour is the port's.
#![allow(non_snake_case, clippy::missing_safety_doc, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

type G = *mut HSD_GObj;

/// Run a callback-taking port with an `unsafe extern "C"` game callback. The ports take Rust
/// `fn(G)` values and call them before returning, so the C callback goes through a trampoline
/// that reads it from a slot (saved and restored, so nested uses are fine).
static mut EV_SLOT: HSD_GObjEvent = None;
fn ev_tramp(g: G) {
    unsafe {
        match EV_SLOT {
            Some(f) => f(g),
            None => panic!("adapt: null HSD_GObjEvent callback"),
        }
    }
}
fn with_ev<R>(cb: HSD_GObjEvent, f: impl FnOnce(fn(G)) -> R) -> R {
    unsafe {
        let prev = core::mem::replace(&mut EV_SLOT, cb);
        let r = f(ev_tramp);
        EV_SLOT = prev;
        r
    }
}

type LineCb = Option<unsafe extern "C" fn(G, i32) -> bool>;
static mut LINE_SLOT: LineCb = None;
fn line_tramp(g: G, i: i32) -> bool {
    unsafe {
        match LINE_SLOT {
            Some(f) => f(g, i),
            None => panic!("adapt: null line callback"),
        }
    }
}

// ---- melee_fighter ----------------------------------------------------------------------------

pub fn ft_80082C74(g: G, cb: HSD_GObjEvent) {
    with_ev(cb, |c| melee_fighter::tu::ft_081b::ft_80082c74(g, c))
}
pub fn ft_8008403C(g: G, cb: HSD_GObjEvent) {
    with_ev(cb, |c| melee_fighter::tu::ft_081b::ft_8008403c(g, c))
}
pub fn ft_800841B8(g: G, cb: HSD_GObjEvent) {
    with_ev(cb, |c| melee_fighter::tu::ft_081b::ft_800841b8(g, c))
}
pub fn ft_80083A48(g: G, cb: HSD_GObjEvent) {
    with_ev(cb, |c| melee_fighter::tu::ft_081b::ft_80083a48(g, c))
}
pub fn ft_800831CC(g: G, line_cb: LineCb, cb: HSD_GObjEvent) {
    unsafe {
        let prev = core::mem::replace(&mut LINE_SLOT, line_cb);
        with_ev(cb, |c| melee_fighter::tu::ft_081b::ft_800831cc(g, line_tramp, c));
        LINE_SLOT = prev;
    }
}
pub fn ftCommon_8007EFC8(g: G, cb: HSD_GObjEvent) {
    with_ev(cb, |c| melee_fighter::tu::ftcommon::ftcommon_8007efc8(g, c))
}
pub fn ftCo_8008D8E8(f: f32) -> i32 {
    melee_fighter::tu::ftCo_Damage::ftCo_8008D8E8(f)
}
pub fn ftCo_8008E908(g: G, f: f32) {
    melee_fighter::tu::ftCo_Damage::ftCo_8008E908(g, f)
}
pub fn ftCo_ScaleBy154(f: f32) -> f32 {
    melee_fighter::tu::ftCo_Damage::ftCo_ScaleBy154(f)
}
pub fn Player_80031DC8(cb: fn(i32, i32)) {
    melee_fighter::player::player_80031dc8(cb)
}

// ---- melee_items ------------------------------------------------------------------------------

pub unsafe fn it_8029DD58(owner: G, pos: &Vec3, x10: i32, part: i32, facing_dir: f32) -> G {
    melee_items::tu::itlinkbomb::it_8029DD58(owner as _, pos as *const Vec3 as *mut Vec3, x10 as _, part as _, facing_dir)
}
pub unsafe fn it_802A013C(facing_dir: f32, owner: G, pos: &Vec3, part: i32, x2c: i32) -> G {
    melee_items::tu::itlinkboomerang::it_802A013C(facing_dir, owner as _, pos as *const Vec3 as *mut Vec3, part as _, x2c)
}
pub unsafe fn it_802A0534(boomerang: G, pos: &Vec3, angle: f32) {
    melee_items::tu::itlinkboomerang::it_802A0534(boomerang as _, pos as *const Vec3 as *mut Vec3, angle)
}
pub unsafe fn it_802A8398(arrow: G, rpos: &Vec3, lpos: &Vec3) {
    melee_items::tu::itlinkarrow::it_802A8398(arrow as _, rpos as *const Vec3 as *mut Vec3, lpos as *const Vec3 as *mut Vec3)
}
pub unsafe fn it_802A83E0(facing_dir: f32, owner: G, pos: &Vec3, part: i32, xc: i32) -> G {
    melee_items::tu::itlinkarrow::it_802A83E0(facing_dir, owner as _, pos as *const Vec3 as *mut Vec3, part as _, xc)
}
pub unsafe fn it_802AF1A4(facing_dir: f32, owner: G, pos: &Vec3, part: i32, x10: i32) -> G {
    melee_items::tu::itlinkbow::it_802AF1A4(facing_dir, owner as _, pos as *const Vec3 as *mut Vec3, part as _, x10)
}
pub unsafe fn itLinkArrow_802A850C(arrow: G, rpos: &Vec3, lpos: &Vec3, angle: f32, charge: f32, max_charge: f32) {
    let _ = melee_items::tu::itlinkarrow::itLinkArrow_802A850C(
        arrow as _,
        rpos as *const Vec3 as *mut Vec3,
        lpos as *const Vec3 as *mut Vec3,
        angle,
        charge,
        max_charge,
    );
}
pub unsafe fn it_802C1590(gobj: G, pos: *mut Vec3, kind: i32, facing_dir: f32) -> G {
    melee_items::tu::itclimbersice::it_802C1590(gobj, pos, kind, facing_dir)
}
pub unsafe fn it_802C16F8(item: G) {
    melee_items::tu::itclimbersice::it_802C16F8(item)
}
pub unsafe fn it_802C17DC(item: G) {
    melee_items::tu::itclimbersice::it_802C17DC(item)
}
pub unsafe fn it_802C2750(item: G) {
    melee_items::tu::itclimbersstring::it_802C2750(item)
}
pub unsafe fn it_802C27D4(parent: G, pos: *mut Vec3, motion_id: u32, dir: f32) -> G {
    melee_items::tu::itclimbersstring::it_802C27D4(parent, pos, motion_id as i32, dir)
}
pub unsafe fn it_802C3810(item: G) {
    melee_items::tu::itclimbersstring::it_802C3810(item)
}
pub unsafe fn it_802C3864(item: G) {
    melee_items::tu::itclimbersstring::it_802C3864(item)
}
pub unsafe fn it_802C3950(item: G) {
    melee_items::tu::itclimbersstring::it_802C3950(item)
}
pub unsafe fn it_802C8B28(gobj: G, pos: *mut Vec3, bone: u32, facing: f32) -> G {
    melee_items::tu::itclinkmilk::it_802C8B28(gobj, pos, bone, facing)
}
pub unsafe fn it_802C8C34(item: G) {
    melee_items::tu::itclinkmilk::it_802C8C34(item)
}
pub unsafe fn it_802F0F6C(owner: G, prev_pos: *mut Vec3, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) {
    melee_items::tu::itcrazyhandbomb::it_802F0F6C(owner, prev_pos, pos, kind, facing_dir)
}
pub unsafe fn it_802F28C8(item: G, flag: i32, f: f32) {
    melee_items::tu::it_2F28::it_802F28C8(item, flag, f)
}
pub unsafe fn itDrMarioPill_802C0DBC(item: G) {
    melee_items::tu::itdrmariopill::itDrMarioPill_802C0DBC(item)
}
pub unsafe fn itDrMarioPill_Appeal_Spawn(gobj: G, pos: *mut Vec3, vitamin_idx: i32, kind: i32, facing_dir: f32) -> G {
    melee_items::tu::itdrmariopill::itDrMarioPill_Appeal_Spawn(gobj, pos, vitamin_idx, kind, facing_dir)
}
pub unsafe fn itDrMarioPill_Spawn(gobj: G, pos: *mut Vec3, pick: i32, kind: i32, facing_dir: f32) {
    melee_items::tu::itdrmariopill::itDrMarioPill_Spawn(gobj, pos, pick, kind, facing_dir)
}
pub unsafe fn Item_8026ABD8(gobj: G, pos: *mut Vec3, f: f32) {
    melee_items::tu::item::Item_8026ABD8(gobj, pos, f)
}

// ---- melee_runtime (lb / archive) -------------------------------------------------------------

pub unsafe fn lb_8000B1CC(jobj: *mut c_void, off: Option<&Vec3>, out: &mut Vec3) {
    melee_runtime::tu::lb_00B0::lb_8000B1CC(
        jobj as *mut HSD_JObj,
        off.map_or(core::ptr::null_mut(), |v| v as *const Vec3 as *mut Vec3),
        out,
    )
}
pub unsafe fn lbAnim_8001E8F8(tree: *mut FigaTree) -> f32 {
    melee_runtime::tu::lbanim::lbAnim_8001E8F8(tree.as_ref())
}
/// `lbArchive_80017040(NULL, filename, out, name, ..., 0)`: `rest` is the tail after `out`
/// without its terminator.
pub unsafe fn lbArchive_80017040(dst: *mut c_void, filename: *const i8, out: *mut c_void, rest: &[*mut c_void]) {
    let mut v: std::vec::Vec<*mut c_void> = std::vec::Vec::with_capacity(rest.len() + 2);
    v.push(out);
    v.extend_from_slice(rest);
    v.push(core::ptr::null_mut());
    let _ = melee_runtime::tu::lbarchive::lbArchive_80017040(dst as *mut *mut HSD_Archive, filename as _, &v);
}
pub unsafe fn lbDvd_800178E8(a: i32, filename: *const i8, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32) {
    melee_runtime::tu::lbdvd::lbDvd_800178E8(a, filename, b, c, d, e, f, g as u8, h)
}
pub unsafe fn HSD_ArchiveGetPublicAddress(archive: *mut HSD_Archive, name: &str) -> *mut c_void {
    let mut buf: std::vec::Vec<i8> = name.bytes().map(|b| b as i8).collect();
    buf.push(0);
    melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(archive, buf.as_ptr())
}

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

// ---- HSD baselib on raw pointers --------------------------------------------------------------

pub unsafe fn HSD_Randi(max: i32) -> i32 {
    melee_runtime::tu::random::HSD_Randi_global(max)
}
pub unsafe fn HSD_Randf() -> f32 {
    melee_runtime::tu::random::HSD_Randf_global()
}
pub unsafe fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) {
    melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(jobj)
}
pub unsafe fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    melee_runtime::tu::jobj_ptr::HSD_JObjGetDObj(jobj)
}
pub unsafe fn HSD_JObjAddDObj(jobj: *mut HSD_JObj, dobj: *mut HSD_DObj) {
    melee_runtime::tu::jobj_ptr::HSD_JObjAddDObj(jobj, dobj)
}
pub unsafe fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_runtime::tu::jobj_ptr::HSD_JObjSetFlagsAll(jobj, flags)
}
pub unsafe fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_runtime::tu::jobj_ptr::HSD_JObjClearFlagsAll(jobj, flags)
}
pub unsafe fn HSD_JObjRemoveAll(jobj: *mut HSD_JObj) {
    melee_runtime::tu::jobj::HSD_JObjRemoveAll_ptr(jobj)
}
pub unsafe fn HSD_JObjAnimAll(jobj: *mut HSD_JObj) {
    melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(jobj)
}
pub unsafe fn HSD_JObjReqAnimAll(jobj: *mut HSD_JObj, frame: f32) {
    melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(jobj, frame)
}
pub unsafe fn HSD_JObjAddAnimAll(jobj: *mut HSD_JObj, anim: *mut HSD_AnimJoint, mat: *mut HSD_MatAnimJoint, shape: *mut HSD_ShapeAnimJoint) {
    melee_runtime::tu::jobj_ptr::HSD_JObjAddAnimAll(jobj, anim, mat, shape)
}
/// The stub's `(jobj, mtx, rmode, flags)` names are the C's `(jobj, vmtx, flags, rendermode)` by position.
pub unsafe fn HSD_JObjDispAll(jobj: *mut HSD_JObj, mtx: MtxPtr, flags: u32, rendermode: u32) {
    melee_runtime::tu::jobj::HSD_JObjDispAll(jobj, &*(mtx as *const Mtx), flags, rendermode)
}
pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, v: *const Vec3) {
    melee_runtime::tu::jobj_ptr::HSD_JObjSetTranslate(jobj, v)
}
pub unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    melee_runtime::tu::jobj_ptr::HSD_JObjSetRotationZ(jobj, z)
}
pub unsafe fn HSD_JObjGetMtxPtr(jobj: *mut HSD_JObj) -> MtxPtr {
    melee_runtime::tu::jobj_ptr::HSD_JObjGetMtxPtr(jobj) as MtxPtr
}
pub unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: MtxPtr) {
    melee_runtime::tu::jobj_ptr::HSD_JObjCopyMtx(jobj, mtx as *const c_void)
}
pub unsafe fn HSD_DObjAddAnimAll(dobj: *mut HSD_DObj, matanim: *mut c_void, aobj: *mut c_void) {
    melee_runtime::tu::dobj::hsd::HSD_DObjAddAnimAll(dobj, matanim as *mut HSD_MatAnim, aobj as *mut HSD_ShapeAnimDObj)
}
pub unsafe fn HSD_DObjLoadDesc(desc: *mut HSD_DObjDesc) -> *mut HSD_DObj {
    melee_runtime::tu::dobj::hsd::HSD_DObjLoadDesc(desc)
}
pub unsafe fn HSD_DObjRemoveAll(dobj: *mut HSD_DObj) {
    melee_runtime::tu::dobj::hsd::HSD_DObjRemoveAll(dobj)
}
pub unsafe fn HSD_DObjResolveRefsAll(dobj: *mut HSD_DObj, desc: *mut HSD_DObjDesc) {
    melee_runtime::tu::dobj::hsd::HSD_DObjResolveRefsAll(dobj, desc)
}

// ---- effects ----------------------------------------------------------------------------------

/// `efSync_Spawn(gfx_id, gobj, ...)`: `args` is the variadic tail (pointers, in `va_arg` order).
pub unsafe fn efSync_Spawn(id: i32, gobj: G, args: &[*mut c_void]) -> *mut c_void {
    let mut va = melee_world::tu::efsync::Va::new(args);
    melee_world::tu::efsync::efSync_Spawn(id, gobj, &mut va)
}
/// Same, for the TUs that carry the tail as register-width integers.
pub unsafe fn efSync_Spawn_usize(id: i32, gobj: G, args: &[usize]) -> *mut c_void {
    let v: std::vec::Vec<*mut c_void> = args.iter().map(|&a| a as *mut c_void).collect();
    efSync_Spawn(id, gobj, &v)
}

type QPtr = *mut melee_types::EF_QueuedEffect;
pub unsafe fn efAsync_LoadAsync(id: i32) {
    melee_world::tu::efasync::efAsync_LoadAsync_native(id)
}
pub unsafe fn efAsync_LoadSync(id: i32) {
    melee_world::tu::efasync::efAsync_LoadSync_native(id)
}
/// `efAsync_Spawn(gobj, head, kind, gfx, jobj)` with no variadic tail.
pub unsafe fn efAsync_Spawn_plain(gobj: G, q: *mut c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) {
    melee_world::tu::efasync::efAsync_Spawn_native(gobj, q as QPtr, kind, gfx, jobj, &[])
}
/// ... with one pointer tail argument.
pub unsafe fn efAsync_Spawn_ptr<T>(gobj: G, q: *mut c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, arg: *const T) {
    melee_world::tu::efasync::efAsync_Spawn_native(gobj, q as QPtr, kind, gfx, jobj, &[arg as *mut c_void])
}
/// ... the `(gobj, head, i32 kind, i32 id, jobj, pos)` shape.
pub unsafe fn efAsync_Spawn_i32<T>(gobj: G, q: *mut c_void, kind: i32, id: i32, jobj: *mut HSD_JObj, arg: *mut T) {
    melee_world::tu::efasync::efAsync_Spawn_native(gobj, q as QPtr, kind as u32, id as u32, jobj, &[arg as *mut c_void])
}
/// ... the shape whose trailing slice starts with the JObj.
pub unsafe fn efAsync_Spawn_slice(gobj: G, q: *mut c_void, kind: u32, gfx: u32, args: &[*mut c_void]) {
    melee_world::tu::efasync::efAsync_Spawn_native(gobj, q as QPtr, kind, gfx, args[0] as *mut HSD_JObj, &args[1..])
}
/// ... the slice shape with the queue head passed as `*mut *mut c_void`.
pub unsafe fn efAsync_Spawn_slice_pp(gobj: G, q: *mut *mut c_void, kind: u32, gfx: u32, args: &[*mut c_void]) {
    efAsync_Spawn_slice(gobj, q as *mut c_void, kind, gfx, args)
}

// ---- cm/camera.c (game_camera) ----------------------------------------------------------------

fn subject_bone_pos(slot: i32) -> Option<Vec3> {
    unsafe {
        let g = melee_fighter::player::get_entity(slot);
        if g.is_null() {
            return None;
        }
        let s = melee_fighter::ftlib::ftlib_80086b74(g);
        if s.is_null() {
            None
        } else {
            Some((*s).bone_pos)
        }
    }
}
/// The game's `game_camera` with the fighter lookup installed.
fn cam() -> &'static mut melee_world::cm::camera::CameraWorld {
    let w = melee_world::cm::camera_boss::global();
    if w.subject_bone_pos.is_none() {
        w.subject_bone_pos = Some(subject_bone_pos);
    }
    w
}
pub fn Camera_8002E6FC(slot: i32) {
    cam().camera_8002e6fc(slot)
}
pub fn Camera_8002E948(cb: unsafe extern "C" fn(*mut Vec3) -> bool) {
    cam().camera_8002e948(cb)
}
pub fn Camera_8002EB5C(v: f32) {
    cam().camera_8002eb5c(v)
}
pub fn Camera_8002EC7C(v: f32) {
    cam().camera_8002ec7c(v)
}
pub fn Camera_8002ED9C(v: f32) {
    cam().camera_8002ed9c(v)
}
pub fn Camera_8002EEC8(fov: f32) {
    cam().camera_8002eec8(fov)
}
pub fn Camera_8002EF14() {
    cam().camera_8002ef14()
}
pub fn Camera_8002F0E4(frames: i32) {
    cam().camera_8002f0e4(frames)
}
pub fn Camera_8002F260() -> bool {
    cam().camera_8002f260()
}
pub fn Camera_8002F274() {
    cam().camera_8002f274()
}
pub fn Camera_SetModeToStandard() {
    cam().camera_set_mode_to_standard()
}
pub unsafe fn Camera_RequestQuake<P>(kind: impl Into<i64>, pos: *const P) {
    let p = if pos.is_null() { None } else { Some(*(pos as *const Vec3)) };
    cam().camera_request_quake(kind.into() as u32, p)
}
/// `Camera_8002E948` for TUs whose callback is a Rust `fn`: the camera keeps the callback and
/// calls it every frame, and there is one boss-intro anchor at a time, so a slot is enough.
static mut CAM_RS_CB: Option<fn(*mut Vec3) -> bool> = None;
unsafe extern "C" fn cam_rs_tramp(p: *mut Vec3) -> bool {
    match CAM_RS_CB {
        Some(f) => f(p),
        None => false,
    }
}
pub fn Camera_8002E948_rs(cb: fn(*mut Vec3) -> bool) {
    unsafe { CAM_RS_CB = Some(cb) };
    cam().camera_8002e948(cam_rs_tramp)
}

// ---- misc ports in lower crates / sibling modules ---------------------------------------------

pub fn ft_CheckGroundAndLedge(g: G, direction: i32) -> i32 {
    melee_fighter::tu::ft_081b::ft_check_ground_and_ledge(g, direction) as i32
}
pub fn ftCo_AirCatchHit_Coll(g: G) {
    melee_fighter::tu::ft_081b::ftco_air_catch_hit_coll(g)
}
pub fn ftKb_SpecialNPr_80100DE0(g: G) {
    crate::chars::r6::ftKirby::ftkirbyspecialdonkey::ftKb_SpecialNPr_80100DE0(g)
}
pub fn ftKb_Init_803B7548() -> *const ftLk_SpecialN_Vec3Group {
    crate::tu::ftkirby::ftKb_Init_803B7548.as_ptr() as *const ftLk_SpecialN_Vec3Group
}
pub fn OSReport(msg: &str) {
    melee_runtime::tu::os_report::OSReport(msg)
}
pub unsafe fn gm_8016B1D8() -> i32 {
    melee_runtime::tu::gmvs_state::gm_8016B1D8()
}
pub unsafe fn ScriptGame_CapsMaxJumps(fp: &Fighter) -> i32 {
    melee_fighter::script_caps::ScriptGame_CapsMaxJumps(fp as *const Fighter as *mut Fighter)
}
pub unsafe fn ScriptGame_HitRuleSpecial(hit: *mut HitCapsule) {
    melee_runtime::tu::script_hit::ScriptGame_HitRuleSpecial(hit)
}
pub fn Mex_OnFloatDispatch(kind: i32, gobj: G) -> i32 {
    melee_runtime::tu::mex_data::Mex_OnFloatDispatch(kind, gobj as *mut c_void)
}

/// `ft_80459B88_hat(kind)`: `ft_80459B88.hats[kind]` (ftkirby.c data).
pub unsafe fn ft_80459B88_hat(kind: i32) -> *mut KirbyHatStruct {
    crate::tu::ftkirby::hat_table(kind as usize)
}
/// `CostumeListsForeachCharacter[kind].costume_list` (ftdata.c data, owned by melee_fighter).
pub unsafe fn CostumeListsForeachCharacter_costume_list(kind: i32) -> *mut UnkCostumeStruct {
    (*(&raw const melee_fighter::tu::ftdata::CostumeListsForeachCharacter))[kind as usize].costume_list
}

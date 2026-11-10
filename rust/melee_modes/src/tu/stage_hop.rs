//! In-match stage switching for the wasm mod host (`gd_stage_switch`, rust/mods/stage_switch).
//!
//! The Rust twin of the C build's full retail visit (pc/gameworld/script_stage_slot_retail.inc,
//! `script_retail_visit`), without the Lua slot layer: the destination's `GrXx.dat` is read from the disc
//! into heap 0 at the moment of the switch, so there is no raw cache outside the game state. Everything
//! the visit touches (heap 0, the GObj pools, stage_info, the ownership ledger captured during the normal
//! stage init) is arena / static state, so a snapshot taken before a switch rolls back across it and a
//! resimulation repeats it exactly. The teardown, collision install, fighter/item carry and camera handover
//! are the ported C helpers (`script_retail_destroy`, `script_retail_items`, `script_retail_carry`,
//! `ScriptGame_StageSlotRetailCollision` through `Ground_801C0800`, `Ground_StageSlotRetailInstall`).
//!
//! Differences from the C: no script geometry is carried (no Lua host: `script_stage.cap` is 0), so the
//! private collision copy takes no extra room; the music keeps playing; there is no logic freeze / cover.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use crate::tu::script_game::ext as sg;
use melee_types::*;

/// Refusal codes (the C's numbering where one exists).
pub const HOP_NOT_GAMEPLAY: i32 = -1;
pub const HOP_READ: i32 = -2;
pub const HOP_HEAP: i32 = -4;
pub const HOP_SAME: i32 = -5;
pub const HOP_BUSY: i32 = -9;
pub const HOP_UNSUPPORTED: i32 = -18;

/// The five audited retail stages (FD, BF, Yoshi's Story, Dream Land, Fountain of Dreams).
pub fn supported(kind: i32) -> bool {
    (unsafe { sg::st_retail_supported(kind) != 0 }) || audit_extra(kind)
}

/// Audit knob (notes/envoy-stage-audit.md): `MELEE_STAGE_HOP_EXTRA=30,29,16` also allows those ground kinds, so a
/// candidate stage can be started on and shifted into / out of before it is added to `st_retail_supported`.
/// Unset (every normal run) it changes nothing; fixed for the process, so it is deterministic.
fn audit_extra(kind: i32) -> bool {
    static EXTRA: std::sync::OnceLock<std::vec::Vec<i32>> = std::sync::OnceLock::new();
    EXTRA
        .get_or_init(|| {
            std::env::var("MELEE_STAGE_HOP_EXTRA")
                .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect())
                .unwrap_or_default()
        })
        .contains(&kind)
}

/// The ground kind that is live now.
pub fn current() -> i32 {
    unsafe { sg::stage_info.grkind as i32 }
}

/// Whether a switch to `kind` could run now (0) or why not.
pub unsafe fn check(kind: i32) -> i32 {
    if sg::ScriptGame_StageGameplayScene() == 0 {
        return HOP_NOT_GAMEPLAY;
    }
    if !supported(kind) || !supported(current()) || sg::script_retail_live == 0 {
        return HOP_UNSUPPORTED;
    }
    if kind == current() {
        return HOP_SAME;
    }
    if sg::HSD_GObjPLinkHead.is_null() {
        return HOP_NOT_GAMEPLAY;
    }
    // the C's StageSwitchCheck: never mid-grab/throw or while (re)spawning
    let mut g = *sg::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !g.is_null() {
        let fp = sg::HSD_GObjGetUserData(g) as *mut Fighter;
        if !fp.is_null() {
            let m = (*fp).motion_id as i32;
            // TARGET_PC: a sleeping sub-fighter (a tag partner, the Zelda / Sheik other half) is not (re)spawning
            if m == ftCo_MS_Sleep as i32 {
                g = (*g).next as *mut HSD_GObj;
                continue;
            }
            if !(*fp).victim_gobj.is_null() || !(*fp).x1A5C.is_null() || m <= ftCo_MS_RebirthWait as i32
                || (m >= ftCo_MS_CapturePulledHi as i32 && m <= ftCo_MS_CaptureFoot as i32)
                || (m >= ftCo_MS_ThrownF as i32 && m <= ftCo_MS_ThrownlwWomen as i32)
            {
                return HOP_BUSY;
            }
        }
        g = (*g).next as *mut HSD_GObj;
    }
    0
}

/// `script_retail_prepare` with no reserved script-geometry room: a private, mutable copy of the
/// destination's `coll_data` in heap 0 (the archive's own tables are never written).
unsafe fn prepare(src: *mut MapCollData) -> *mut MapCollData {
    if src.is_null() || (*src).vert_count <= 0 || (*src).line_count <= 0 || (*src).joint_count <= 0
        || (*src).vert_count > 2048 || (*src).line_count > 1536 || (*src).joint_count > sg::SCRIPT_STAGE_JOINTS
    {
        return core::ptr::null_mut();
    }
    let dst = sg::lbHeap_StageSlotTryAlloc(core::mem::size_of::<MapCollData>() as u32) as *mut MapCollData;
    if dst.is_null() {
        return dst;
    }
    *dst = *src;
    let (nv, nl, nj) = ((*src).vert_count as usize, (*src).line_count as usize, (*src).joint_count as usize);
    let bv = nv * core::mem::size_of::<Vec2>();
    let bl = nl * core::mem::size_of::<MapLine>();
    let bj = nj * core::mem::size_of::<MapJoint>();
    (*dst).verts = sg::lbHeap_StageSlotTryAlloc(bv as u32) as _;
    (*dst).lines = sg::lbHeap_StageSlotTryAlloc(bl as u32) as _;
    (*dst).joints = sg::lbHeap_StageSlotTryAlloc(bj as u32) as _;
    if (*dst).verts.is_null() || (*dst).lines.is_null() || (*dst).joints.is_null() {
        for p in [(*dst).verts as *mut core::ffi::c_void, (*dst).lines as _, (*dst).joints as _] {
            if !p.is_null() {
                sg::lbHeap_80015CA8(0, p);
            }
        }
        sg::lbHeap_80015CA8(0, dst as _);
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping((*src).verts as *const u8, (*dst).verts as *mut u8, bv);
    core::ptr::copy_nonoverlapping((*src).lines as *const u8, (*dst).lines as *mut u8, bl);
    core::ptr::copy_nonoverlapping((*src).joints as *const u8, (*dst).joints as *mut u8, bj);
    dst
}

/// Switch the live stage to ground kind `kind` now. Must run between logic frames (outside
/// `HSD_GObj_RunProcs`): the mod host applies it after the `frame.run_procs` post chain.
/// Returns 0 on success; on a refusal nothing has changed.
pub unsafe fn switch_to(kind: i32) -> i32 {
    let rc = check(kind);
    if rc != 0 {
        return rc;
    }
    let file = sg::Ground_StageSlotDataFile(kind);
    if file.is_null() {
        return HOP_READ;
    }
    let mut path = [0i8; 262];
    path[0] = b'/' as i8;
    let mut f = file;
    while *f == b'/' as i8 {
        f = f.add(1);
    }
    let mut n = 1usize;
    let mut dot = false;
    while *f != 0 && n < 250 {
        dot |= *f == b'.' as i8;
        path[n] = *f;
        n += 1;
        f = f.add(1);
    }
    if !dot {
        for &b in b".dat" {
            path[n] = b as i8;
            n += 1;
        }
    }
    let entry = sg::DVDConvertPathToEntrynum(path.as_ptr());
    if entry < 0 {
        return HOP_READ;
    }
    let bytes = sg::lbFile_8001634C(entry) as u32;
    if !(32..=8 * 1024 * 1024).contains(&bytes) {
        return HOP_READ;
    }
    // the C's budget: the incoming archive and collision while the outgoing visit is still live
    let need = bytes as usize + core::mem::size_of::<HSD_Archive>() + 2048 * core::mem::size_of::<CollVtx>()
        + 1536 * core::mem::size_of::<CollLine>() + sg::SCRIPT_STAGE_JOINTS as usize * core::mem::size_of::<CollJoint>()
        + 2048 * core::mem::size_of::<Vec2>() + 1536 * core::mem::size_of::<MapLine>()
        + sg::SCRIPT_STAGE_JOINTS as usize * core::mem::size_of::<MapJoint>() + 2 * 1024 * 1024 + sg::ST_SLOT_HEADROOM as usize;
    if (sg::lbHeap_Free(0) as usize) < need {
        return HOP_HEAP;
    }
    let work = sg::lbHeap_StageSlotTryAlloc((bytes + 31) & !31);
    let archive = sg::lbHeap_StageSlotTryAlloc(core::mem::size_of::<HSD_Archive>() as u32) as *mut HSD_Archive;
    let free_both = |w: *mut core::ffi::c_void, a: *mut HSD_Archive| {
        if !a.is_null() {
            sg::lbHeap_80015CA8(0, a as _);
        }
        if !w.is_null() {
            sg::lbHeap_80015CA8(0, w);
        }
    };
    if work.is_null() || archive.is_null() {
        free_both(work, archive);
        return HOP_HEAP;
    }
    core::ptr::write_bytes(archive as *mut u8, 0, core::mem::size_of::<HSD_Archive>());
    let mut actual: usize = 0;
    sg::lbFile_8001668C(path.as_ptr(), work, &mut actual);
    // The engine decodes the archive's custom roots (`yakumono_param`: melee_engine::dat_custom, the per-stage layouts of
    // yakumono_layouts.rs) at parse time, picking the layout by `stage_info.grkind`: that must already name the destination,
    // or the record is decoded with the outgoing stage's layout (Kongo Jungle 64: all zeros, the barrel launcher asserts).
    let live_kind = sg::stage_info.grkind;
    sg::stage_info.grkind = kind as _;
    let parsed = actual == bytes as usize && sg::HSD_ArchiveParse(archive, work as *mut u8, bytes as usize) >= 0;
    sg::stage_info.grkind = live_kind;
    if !parsed {
        free_both(work, archive);
        return HOP_READ;
    }
    let mut i = 0;
    while !sg::HSD_ArchiveGetExtern(archive, i).is_null() {
        sg::HSD_ArchiveLocateExtern(archive, sg::HSD_ArchiveGetExtern(archive, i), core::ptr::null_mut());
        i += 1;
    }
    let map = prepare(sg::HSD_ArchiveGetPublicAddress(archive, c"coll_data".as_ptr()) as *mut MapCollData);
    if map.is_null() {
        free_both(work, archive);
        return HOP_HEAP;
    }
    let from = current();
    // commit: from here on the outgoing visit is destroyed (script_retail_visit's order)
    sg::ScriptGame_StageIsolationClear(0);
    sg::script_switch_detach();
    sg::script_retail_items(1);
    sg::script_retail_destroy();
    sg::script_retail.working = work;
    sg::script_retail.working_bytes = bytes;
    sg::script_retail.archive = archive;
    sg::script_retail.prepared = map;
    sg::script_retail_loading = 1;
    sg::script_retail_capture = 1;
    let mut pair: StageIdPair = core::mem::zeroed();
    pair.grkind = kind as GrKind;
    pair.stkind = sg::Stage_StageSlotSelect(kind) as StKind;
    sg::Camera_800311DC(0.1);
    sg::Camera_800311CC(16384.0);
    sg::Ground_StageSlotRetailInstall(&mut pair, archive);
    sg::Camera_80030730(sg::Ground_801C20D0());
    sg::Ground_EnableMatchCamera();
    sg::script_retail_loading = 0;
    sg::script_retail_capture = 0;
    sg::script_retail_carry();
    sg::script_retail_items(0);
    sg::script_switch.epoch += 1;
    melee_core::heaptrace::mark("stage switch");
    if std::env::var_os("MELEE_STAGE_HOP_QUIET").is_none() {
        eprintln!("stage switch: {from} -> {kind} ({bytes} bytes, {} owned objects, heap0 free {})",
            sg::script_retail_owned.count, sg::lbHeap_Free(0));
    }
    0
}

/// `ftCommon_UnlockECB` for the switch's contact reset (script_switch_contacts): the fighter crate's port.
pub unsafe fn unlock_ecb(fp: *mut Fighter) {
    if !fp.is_null() {
        melee_fighter::ftcommon2::unlock_ecb(&mut *fp);
    }
}

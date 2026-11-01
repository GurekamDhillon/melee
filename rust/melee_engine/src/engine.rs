//! The engine (consolidation plan 1.1, step 6): boots the game's own libraries on the host and runs the scene
//! loop one `gm_801A4D34` iteration at a time. No game logic here: `main`'s order of calls (gmmain.c, TARGET_PC),
//! `gm_801A4BD4` / `gm_801A4D34_begin` / `gm_801A4D34_tick` from `melee_modes::tu::gmscene`, and the pad feed.

use crate::gcm::Gcm;
use crate::host::Host;
use core::ffi::c_void;
use melee_modes::tu::{gm_1A3F, gmmain, gmmain_lib, gmscene};
use melee_platform::PadSystem;
use melee_runtime::tu::{controller, initialize};
use melee_types::{GameSceneInfo, PADStatus};

/// The pad source `PADRead` reads (a hook is a plain `fn`, so the system is a global like the C's SI registers).
static mut PADS: Option<PadSystem> = None;

/// A scripted pad source (the golden harness's `pad_fixed.lua` cycles): `(tick, port) -> PADStatus`. It stands in for
/// the Lua pad of the C golden run, i.e. it replaces what `PADRead` returns, before the game's own `HSD_PadRenewStatus`.
type PadFeed = Box<dyn Fn(u32, usize) -> PADStatus>;
static mut PAD_FEED: Option<PadFeed> = None;
/// The tick the next `PADRead` belongs to (the engine's frame counter).
static mut FEED_TICK: u32 = 0;
/// Inputs for the next tick set by a rollback session (wins over the feed). A plain static: a load restores it.
static mut DIRECT_PADS: Option<[PADStatus; 4]> = None;

fn pad_read(st: *mut PADStatus) -> u32 {
    unsafe {
        let arr = &mut *(st as *mut [PADStatus; 4]);
        let r = match PADS.as_mut() {
            Some(p) => p.read(arr),
            None => 0,
        };
        if let Some(d) = DIRECT_PADS {
            *arr = d;
        } else if let Some(f) = PAD_FEED.as_ref() {
            for (port, slot) in arr.iter_mut().enumerate() {
                *slot = f(FEED_TICK, port);
            }
        }
        r
    }
}

/// `MELEE_RAND_TRACE=<file>`: one line per `HSD_Rand*` draw (`frame draw# stream callers...`), for finding where the
/// engine's draw sequence leaves the C build's (the golden `seed` column counts them).
static mut RAND_TRACE: Option<std::fs::File> = None;
static mut RAND_FRAME: i32 = -1;
static mut RAND_N: u32 = 0;

fn rand_trace(_seed: u32, stream: i32) {
    use std::io::Write;
    unsafe {
        RAND_N += 1;
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let mut names: Vec<&str> = Vec::new();
        for l in bt.lines() {
            let l = l.trim();
            if let Some((_, rest)) = l.split_once(": ") {
                if rest.starts_with("melee_") && !rest.contains("random::") && !rest.contains("rand_trace") && !rest.contains("ext::") {
                    names.push(rest.trim_start_matches("melee_"));
                }
            }
        }
        names.truncate(6);
        if let Some(f) = (&raw mut RAND_TRACE).as_mut().and_then(|f| f.as_mut()) {
            let _ = writeln!(f, "{} {} s{} {}", RAND_FRAME, RAND_N, stream, names.join(" < "));
        }
    }
}

pub struct Engine {
    /// Frames ticked since `begin_scene`.
    pub frames: u32,
    /// The scene's `GameSceneInfo` (null: an empty scene, which is all step 6 runs).
    pub info: *mut GameSceneInfo,
    pub on_frame: Option<unsafe extern "C" fn()>,
    /// `gm_801A4D34_tick` returned false (the scene ended).
    pub ended: bool,
}

impl Engine {
    /// Host, link layer, host seams, the libraries `main` brings up (heap, pad, gmMainLib), then the scene
    /// setup `gm_801A4BD4` (which runs the real `HSD_GObjInit`) and the loop's pre-loop part.
    pub fn boot(disc: Gcm, mem_size: usize, pads: PadSystem) -> Result<Engine, String> {
        Self::boot_host(disc, mem_size, pads)?;
        unsafe {
            gmscene::gm_801A4BD4();
            gmscene::gm_801A4D34_begin(core::ptr::null_mut());
        }
        Ok(Engine { frames: 0, info: core::ptr::null_mut(), on_frame: None, ended: false })
    }

    /// Step 7: boot the host, then start the VS mode the way `main` -> `gm_801A4510` -> `runGameMode` -> `gm_801A4014` does
    /// (the VS mode's `on_load` seeds the match from `scn`, entry state 2 = the match), up to the first scene tick.
    pub fn boot_vs(disc: Gcm, mem_size: usize, pads: PadSystem, scn: crate::scenario::Scenario) -> Result<Engine, String> {
        let _scope = crate::arena::GameScope::enter();
        Self::boot_host(disc, mem_size, pads)?;
        crate::scenario::install(scn);
        unsafe {
            // The C boot runs the memory-card scene on a fresh save before the menus: `gmMainLib_8015F600(1, 0)` draws one
            // `HSD_Randi` (`Toy_80305058(2, 0x63, 0, 100.0f)` picks the new trophy). The golden seed, and every later draw
            // (grLast's sway rates, the item spawner), depend on that one RNG step. The trophy tables (ty, `Toy_803124BC`)
            // are not loaded on the engine path yet, so only the draw is made (the selected id is unused for the hash).
            let _ = melee_runtime::tu::random::global::HSD_Rand();
            gm_1A3F::gm_801A4510_init(Some(melee_types::GM_VS as u8));
            let mode = gm_1A3F::runGameMode_load(melee_types::GM_VS as u8);
            eprintln!("vs mode {:p} kind {} states {:p} curr_state {}", mode, (*mode).kind, (*mode).states, gm_1A3F::state_machine.routing.curr_state_id);
            { let mut j = 0; while j < 12 { let st = (*mode).states.add(j); eprintln!("  state[{j}] id {}", (*st).id); if (*st).id == 0xFF { break; } j += 1; } }
            let run = gm_1A3F::gm_801A4014_enter(mode);
            gmscene::gm_801A4D34_begin(run.info);
            // Diagnostic: draws the C makes before frame 0 that belong to subsystems the engine has not taken over yet
            // (the memory-card scene's trophy pick, the stage init: grLast 4 + the BGM pick). Only their COUNT matters
            // to the particle system's draws, so the seed column can be checked before those steps land.
            if let Some(n) = std::env::var("MELEE_ENGINE_PRE_DRAWS").ok().and_then(|v| v.parse::<u32>().ok()) {
                for _ in 0..n {
                    melee_runtime::tu::random::global::state().rand();
                }
            }
            Ok(Engine { frames: 0, info: run.info, on_frame: run.frame_fn, ended: false })
        }
    }

    fn boot_host(disc: Gcm, mem_size: usize, pads: PadSystem) -> Result<(), String> {
        crate::crash::install();
        Host::boot(disc, mem_size)?;
        unsafe {
            melee_runtime::tu::gobj_native::HEADLESS_RENDER = std::env::var_os("MELEE_ENGINE_HEADLESS").is_some();
            // render draws (particle generators created while drawing) use the private render seed, not the logic stream
            melee_runtime::tu::gobj_native::HSD_RandRenderBegin = Some(melee_runtime::tu::random::global::HSD_RandRenderBegin);
            melee_runtime::tu::gobj_native::HSD_RandRenderEnd = Some(melee_runtime::tu::random::global::HSD_RandRenderEnd);
            // GXProject is pure math whose result reaches the simulation (Camera_LogicToScreen -> the fighters' offscreen flag):
            // the SDK function, owned by the gx shim.
            melee_runtime::tu::lbvector::ext::hook::GXProject = Some(|x, y, z, mtx, pm, vp, sx, sy, sz| unsafe {
                let (a, b, c) = melee_gx::shim::gx_project(x, y, z, &*(mtx as *const [[f32; 4]; 3]), &*(pm as *const [f32; 7]), &*(vp as *const [f32; 6]));
                *sx = a;
                *sy = b;
                *sz = c;
            });
            melee_modes::tu::gmscene::ext::FRAME_POST = Some(|| unsafe {
                FRAME_ROW = Some(Engine::compute_row());
                if CAPTURE_FRAME == RAND_FRAME as i64 { CAPTURED = Some(Engine::capture_now()); }
            });
            melee_core::ct::RENDER_STUB.store(!melee_runtime::tu::gobj_native::HEADLESS_RENDER, core::sync::atomic::Ordering::Relaxed);
            melee_runtime::tu::particle::CMD_STREAM_BIG_ENDIAN = true;
            melee_items::tu::itmaterial::it_mobj_static_init(); // the C data initialiser of the item material class
            
        }
        melee_link::install();
        melee_link::install_retail();
        unsafe { melee_runtime::tu::generator::GEN_TRACE = std::env::var_os("MELEE_PS_TRACE").is_some(); }
        if let Some(path) = std::env::var_os("MELEE_RAND_TRACE") {
            unsafe {
                RAND_TRACE = std::fs::File::create(path).ok();
                melee_runtime::tu::random::global::state().trace = Some(rand_trace);
            }
        }
        crate::cdata::install_cdata();
        crate::install();
        unsafe {
            PADS = Some(pads);
            controller::ext::hook::PADRead = Some(pad_read);
            controller::ext::hook::PADInit = Some(|| 1);
            controller::ext::hook::PADReset = Some(|_| 1);
            controller::ext::hook::PADRecalibrate = Some(|_| 1);
            controller::ext::hook::OSGetResetSwitchState = Some(|| 0);
            controller::ext::hook::OSDisableInterrupts = Some(crate::dvd::OSDisableInterrupts);
            controller::ext::hook::OSRestoreInterrupts = Some(crate::dvd::OSRestoreInterrupts);

            crate::seams::install();
            // the render pass draws into melee_gx unless the run is headless (MELEE_ENGINE_HEADLESS): then the GX hooks answer nothing
            if RENDER && !melee_runtime::tu::gobj_native::HEADLESS_RENDER {
                crate::gxbridge_gen::install_gx();
            }
            crate::seams_gen::install_gen();
            gmmain_lib::gmMainLib_init_statics();
            melee_runtime::tu::perf::__OSBusClock = 162_000_000; // OS_BUS_CLOCK (low memory 0xF8 on a GameCube)
            initialize::HSD_OSInit();
            // *HSD_RandSeedPtr = InitialSeed() (MELEE_TEST_SEED=12345 for the goldens)
            melee_runtime::tu::random::global::state().set_logic_seed(
                std::env::var("MELEE_TEST_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(12345),
            );
            // lb_80019AAC(&gmMain_8015FD24): pad library setup and the sampling period
            melee_modes::tu::lb_0195::lb_80019AAC(Some(gmmain_pad_setup));
            // the rest of `main` after the pad setup (gm/gmmain.c)
            melee_items::tu::dbitem::db_ItemAndPokemonMenu.ItemSpawnsEnabled = 1; // db_EnableItemSpawns() in `main`
            melee_runtime::tu::lbmemory::lbMemory_8001564C();
            melee_runtime::tu::lbheap::lbHeap_80015F3C();
            melee_runtime::tu::lbdvd::lbDvd_80018F68();
            melee_runtime::tu::lbarq::lbArq_80014D2C();
            melee_runtime::tu::lbcardnew::lbCardNew_Init();
            melee_runtime::tu::lbcardgame::lbCardGame_Init();
            melee_runtime::tu::lbsnap::lbSnap_8001E290();
            gmmain_lib::gmMainLib_8015FCC0();
            melee_runtime::tu::lbmthp::lbMthp_8001F87C();
            melee_runtime::tu::sislib::HSD_SisLib_803A6048(0xC000);
            gmmain_lib::gmMainLib_8015FBA4();
        }
        Ok(())
    }

    /// The NHV1 golden row of the current state: `[hash, seed, items, itemsum, s0..s11]` (docs/rust-port/neutral-hash.md),
    /// from the real Fighter / Item records through the ported `RB_NeutralHash`.
    pub fn neutral_row(&self) -> [u32; 16] {
        // the C build hashes in Script_FramePost (after the logic procs, before the render pass)
        unsafe { if let Some(r) = FRAME_ROW { return r; } }
        Self::compute_row()
    }

    fn compute_row() -> [u32; 16] {
        unsafe {
            melee_fighter::tu::fighter::RB_NeutralHash();
            let mut r = [0u32; 16];
            for (i, w) in r.iter_mut().enumerate() { *w = melee_fighter::tu::fighter::RB_NeutralHashWord(i as i32); }
            r
        }
    }

    /// Debug: a slot's fighter JObj tree in preorder, "jN rot x y z w trans x y z" (bit patterns), like the legacy `--dump-state` tree lines.
    pub fn debug_tree(&self, slot: i32) -> String {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            if g.is_null() {
                return String::new();
            }
            let mut out = String::new();
            let mut stack = vec![(*g).hsd_obj as *mut melee_types::HSD_JObj];
            let mut n = 0;
            while let Some(j) = stack.pop() {
                if j.is_null() {
                    continue;
                }
                let r = &(*j).rotate;
                let t = &(*j).translate;
                out += &format!("j{n} rot {:08x} {:08x} {:08x} {:08x} trans {:08x} {:08x} {:08x}
", r.x.to_bits(), r.y.to_bits(), r.z.to_bits(), r.w.to_bits(), t.x.to_bits(), t.y.to_bits(), t.z.to_bits());
                n += 1;
                stack.push((*j).next);
                stack.push((*j).child);
            }
            out
        }
    }

    /// Debug: the live items (p_link 9): "itemN kind K msid M pos x y z vel x y z" (bit patterns), the fields the neutral hash sums.
    pub fn debug_items(&self) -> String {
        unsafe {
            let mut out = String::new();
            let ci = &melee_world::tu::ground::stage_info.cam_info;
            out += &format!("cam top {:08x} bottom {:08x} left {:08x} right {:08x} xoff {:08x} yoff {:08x}
", ci.cam_bounds.top.to_bits(), ci.cam_bounds.bottom.to_bits(),
                ci.cam_bounds.left.to_bits(), ci.cam_bounds.right.to_bits(), ci.cam_x_offset.to_bits(), ci.cam_y_offset.to_bits());
            let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(melee_types::HSD_GOBJ_PLINK_ITEM as usize);
            let mut n = 0;
            while !g.is_null() {
                let ip = (*g).user_data as *mut melee_types::Item;
                if ip.is_null() { out += &format!("gobj {:p} no user data
", g); }
                if !ip.is_null() {
                    let (p, v) = (&(*ip).pos, &(*ip).x40_vel);
                    out += &format!("item{n} kind {} msid {} pos {:08x} {:08x} {:08x} vel {:08x} {:08x} {:08x}
", (*ip).kind as i32, (*ip).msid as i32,
                        p.x.to_bits(), p.y.to_bits(), p.z.to_bits(), v.x.to_bits(), v.y.to_bits(), v.z.to_bits());
                    n += 1;
                }
                g = (*g).next;
            }
            out
        }
    }

    /// The live items (p_link 9, list order) as `(kind, msid, pos, vel)` for `melee diff --engine tu`.
    pub fn item_rows(&self) -> Vec<(i32, i32, [f32; 3], [f32; 3])> {
        unsafe {
            let mut out = Vec::new();
            let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(melee_types::HSD_GOBJ_PLINK_ITEM as usize);
            while !g.is_null() {
                let ip = (*g).user_data as *mut melee_types::Item;
                if !ip.is_null() {
                    let (p, v) = (&(*ip).pos, &(*ip).x40_vel);
                    out.push(((*ip).kind as i32, (*ip).msid as i32, [p.x, p.y, p.z], [v.x, v.y, v.z]));
                }
                g = (*g).next;
            }
            out
        }
    }

    /// Debug: the stage map gobjs (p_link 5): "map_id N [c4..e0 as f32 bits]" per gobj (compare with the legacy `last_bg`).
    pub fn debug_map_gobjs(&self) -> String {
        unsafe {
            let mut out = String::new();
            let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(5);
            while !g.is_null() {
                let gp = (*g).user_data as *const melee_types::Ground;
                if !gp.is_null() {
                    let l = &(*gp).u.last;
                    out += &format!("map_id {}: {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}
", (*gp).map_id as i32, l.xC4.to_bits(), l.xC8.to_bits(), l.xCC.to_bits(), l.xD0.to_bits(), l.xD4.to_bits(), l.xD8.to_bits(), l.xDC.to_bits());
                }
                g = (*g).next;
            }
            out
        }
    }

    /// The raw bytes of a slot's native `Fighter` record (debug dumps).
    pub fn fighter_bytes(&self, slot: i32) -> Option<Vec<u8>> {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            if g.is_null() || (*g).user_data.is_null() {
                return None;
            }
            let n = core::mem::size_of::<melee_types::Fighter>();
            Some(core::slice::from_raw_parts((*g).user_data as *const u8, n).to_vec())
        }
    }

    /// The second pose joint of every part (`parts[i].x4_jobj2`, the blend source tree) as the same 26-word records (zeros when absent).
    pub fn alt_joint_words(&self, slot: i32) -> Vec<[u32; 26]> {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            let mut out = Vec::new();
            if g.is_null() || (*g).user_data.is_null() {
                return out;
            }
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            let n = (**melee_fighter::hooks::ft_parts_table().add(fp.kind as usize)).parts_num as usize;
            for i in 0..n.min(128) {
                let j = (*fp.parts.add(i)).x4_jobj2;
                let mut w = [0u32; 26];
                if !j.is_null() {
                    let j = &*j;
                    w[0] = j.flags;
                    let r = &j.rotate;
                    for (k, v) in [r.x, r.y, r.z, r.w].iter().enumerate() { w[1 + k] = v.to_bits(); }
                    for (k, v) in [j.scale.x, j.scale.y, j.scale.z].iter().enumerate() { w[5 + k] = v.to_bits(); }
                    for (k, v) in [j.translate.x, j.translate.y, j.translate.z].iter().enumerate() { w[8 + k] = v.to_bits(); }
                    // words 11..: the AObj (curr_frame, rewind_frame, end_frame, framerate, flags) and its first three FObjs
                    // (time, p0, p1, d0, d1, flags | op_intrp << 8 | fterm << 16): the animation state of the blend tree
                    if !j.aobj.is_null() {
                        let a = &*j.aobj;
                        w[11] = a.curr_frame.to_bits(); w[12] = a.rewind_frame.to_bits(); w[13] = a.end_frame.to_bits(); w[14] = a.framerate.to_bits(); w[15] = a.flags;
                        let mut fo = a.fobj;
                        let mut k = 16;
                        while !fo.is_null() && k + 5 < 26 {
                            let f = &*fo;
                            w[k] = f.time.to_bits(); w[k + 1] = f.p0.to_bits(); w[k + 2] = f.p1.to_bits(); w[k + 3] = f.d0.to_bits(); w[k + 4] = f.d1.to_bits();
                            w[k + 5] = f.flags as u32 | (f.op_intrp as u32) << 8 | (f.fterm as u32) << 16;
                            k += 6;
                            fo = f.next;
                        }
                    }
                }
                out.push(w);
            }
            out
        }
    }

    /// A slot's fighter JObj tree in preorder as `[flags, rot x4, scale x3, translate x3, mtx 3x4]` bit patterns (`melee diff --engine tu`).
    pub fn joint_words(&self, slot: i32) -> Vec<[u32; 26]> {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            let mut out = Vec::new();
            if g.is_null() {
                return out;
            }
            let mut stack = vec![(*g).hsd_obj as *mut melee_types::HSD_JObj];
            while let Some(j) = stack.pop() {
                if j.is_null() || out.len() > 512 {
                    continue;
                }
                let j = &*j;
                let mut w = [0u32; 26];
                w[0] = j.flags;
                let r = &j.rotate;
                for (k, v) in [r.x, r.y, r.z, r.w].iter().enumerate() { w[1 + k] = v.to_bits(); }
                for (k, v) in [j.scale.x, j.scale.y, j.scale.z].iter().enumerate() { w[5 + k] = v.to_bits(); }
                for (k, v) in [j.translate.x, j.translate.y, j.translate.z].iter().enumerate() { w[8 + k] = v.to_bits(); }
                for row in 0..3 { for col in 0..4 { w[11 + row * 4 + col] = j.mtx[row][col].to_bits(); } }
                out.push(w);
                stack.push(j.next);
                stack.push(j.child);
            }
            out
        }
    }

    /// A slot's dynamic bone chains for `melee diff --engine tu`: per bone set, per node, the node's 35 words after its joint pointer
    /// (`lb_00F9_UnkDesc0` GC offsets 0x04..0x90; natively the record is 4 bytes longer because of the 8-byte pointer).
    pub fn dyn_words(&self, slot: i32) -> Vec<Vec<Vec<u32>>> {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            let mut out = Vec::new();
            if g.is_null() || (*g).user_data.is_null() {
                return out;
            }
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            for i in 0..(fp.dynamics_num.max(0) as usize).min(10) {
                let mut nodes = Vec::new();
                let mut n = fp.dynamic_bone_sets[i].dyn_desc.data;
                while !n.is_null() && nodes.len() < 64 {
                    let base = n as *const u8;
                    nodes.push((1..36usize).map(|k| core::ptr::read_unaligned(base.add(4 * k + 4) as *const u32)).collect());
                    n = (*n).next;
                }
                out.push(nodes);
            }
            out
        }
    }

    /// Debug (`MELEE_ENGINE_HURT=1`): a slot's hurt capsules: bone joint index (preorder), joint flags, state, skip flag, a/b positions.
    pub fn debug_hurt(&self, slot: i32) -> String {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            if g.is_null() || (*g).user_data.is_null() {
                return String::new();
            }
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            let mut order: Vec<*mut melee_types::HSD_JObj> = Vec::new();
            let mut stack = vec![(*g).hsd_obj as *mut melee_types::HSD_JObj];
            while let Some(j) = stack.pop() {
                if j.is_null() { continue; }
                order.push(j);
                stack.push((*j).next);
                stack.push((*j).child);
            }
            let mut out = String::new();
            for i in 0..fp.hurt_capsules_len as usize {
                let c = &fp.hurt_capsules[i].capsule;
                let idx = order.iter().position(|&j| j as *mut _ == c.bone as *mut _).map_or(-1, |x| x as i32);
                let fl = if c.bone.is_null() { 0 } else { (*c.bone).flags };
                out += &format!("hurt{i} bone j{idx} flags {fl:08x} state {} skip {} a ({:.4},{:.4},{:.4}) b ({:.4},{:.4},{:.4})
", c.state as i32, c.skip_update_pos(), c.a_pos.x, c.a_pos.y, c.a_pos.z, c.b_pos.x, c.b_pos.y, c.b_pos.z);
            }
            out
        }
    }

    /// Debug aid (`MELEE_ENGINE_FT=1` in `melee golden --engine tu`): motion id, position, velocity and ground/air of a slot's fighter.
    pub fn debug_fighter(&self, slot: i32) -> String {
        unsafe {
            let g = melee_fighter::player::get_entity(slot);
            if g.is_null() || (*g).user_data.is_null() {
                return format!("slot {slot}: none");
            }
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            let mut js = String::new();
            for i in 0..6 {
                let j = fp.coll_data.ecb_source.anon_4.anon_0.x10C_joint[i];
                let mut v: melee_types::Vec = core::mem::zeroed();
                if !j.is_null() {
                    melee_runtime::tu::lb_00B0::lb_8000B1CC(j, core::ptr::null_mut(), &mut v);
                }
                js += &format!(" j{i}({:.3},{:.3})", v.x, v.y);
            }
            format!(
                "{js} slot {slot}: msid {} pos ({:.4},{:.4}) vel ({:.4},{:.4}) goa {} env {:08X} x{} cmd {:?} frame {:.2}",
                fp.motion_id as i32, fp.cur_pos.x, fp.cur_pos.y, fp.self_vel.x, fp.self_vel.y,
                fp.ground_or_air as i32, fp.coll_data.env_flags, 0, fp.cmd_vars, fp.cur_anim_frame
            )
        }
    }

    /// Drive the pads from a script instead of `PadSystem` (`feed(tick, port)`; tick 0 is the first `tick()`).
    pub fn set_pad_feed(&mut self, feed: impl Fn(u32, usize) -> PADStatus + 'static) {
        unsafe { PAD_FEED = Some(Box::new(feed)) };
    }

    /// The pads of the next tick (a rollback session's inputs); `None` goes back to the feed / `PadSystem`.
    pub fn set_pads(&mut self, p: Option<[PADStatus; 4]>) {
        unsafe { DIRECT_PADS = p };
    }

    /// Exactly one iteration of the scene loop (pad queue, logic frame(s), `HSD_GObj_RunProcs`, render).
    /// Returns false once the scene has ended.
    pub fn tick(&mut self) -> bool {
        if self.ended {
            return false;
        }
        let _scope = crate::arena::GameScope::enter();
        unsafe { RAND_FRAME = self.frames as i32; }
        melee_core::ct::DBG_FRAME.store(self.frames as i32, core::sync::atomic::Ordering::Relaxed);
        unsafe { FEED_TICK = self.frames };
        crate::seams::fire_alarm(); // the periodic pad-sampling alarm (lb_80019628) fires once per frame
        let more = unsafe { gmscene::gm_801A4D34_tick(self.on_frame, self.info) };
        if unsafe { RENDER } && !unsafe { melee_runtime::tu::gobj_native::HEADLESS_RENDER } {
            crate::gxbridge::finish_frame();
        }
        self.frames += 1;
        if !more {
            self.ended = true;
        }
        more
    }
}

/// Draw into melee_gx (`gxbridge`): set before `boot_vs` by the viewers and `--shots`; goldens leave the GX hooks silent (faster).
pub static mut RENDER: bool = false;
static mut FRAME_ROW: Option<[u32; 16]> = None;
/// `melee diff --engine tu`: capture the state at `Script_FramePost` of this frame (the C snapshot point: after the logic
/// procs, before the render pass; joints are still matrix-dirty there).
pub static mut CAPTURE_FRAME: i64 = -1;
pub static mut CAPTURED: Option<Captured> = None;

/// The engine state read at the post-logic point.
pub struct Captured {
    pub row: [u32; 16],
    pub fighters: Vec<(i32, Vec<u8>)>,
    pub joints: Vec<(i32, Vec<[u32; 26]>)>,
    pub alt: Vec<(i32, Vec<[u32; 26]>)>,
    pub dyns: Vec<(i32, Vec<Vec<Vec<u32>>>)>,
    pub items: Vec<(i32, i32, [f32; 3], [f32; 3])>,
}

impl Engine {
    fn capture_now() -> Captured {
        let e = Engine { frames: 0, info: core::ptr::null_mut(), on_frame: None, ended: false };
        let mut fighters = Vec::new();
        for slot in 0..2 {
            if let Some(b) = e.fighter_bytes(slot) { fighters.push((slot, b)); }
        }
        Captured {
            row: unsafe { FRAME_ROW.unwrap_or([0; 16]) },
            fighters,
            joints: (0..2).map(|slot| (slot, e.joint_words(slot))).collect(),
            alt: (0..2).map(|slot| (slot, e.alt_joint_words(slot))).collect(),
            dyns: (0..2).map(|slot| (slot, e.dyn_words(slot))).collect(),
            items: e.item_rows(),
        }
    }
}

unsafe extern "C" fn gmmain_pad_setup() {
    gmmain::gmMain_8015FD24();
}

#[allow(dead_code)]
fn _unused(_: *mut c_void) {}

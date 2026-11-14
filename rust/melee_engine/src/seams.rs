//! Host seams the scene loop and the libraries `main` brings up call out to: the dolphin SDK hardware (pad
//! hardware, video, GX), the TARGET_PC port extensions (rollback, scripts, replay, determinism, overlay) and the
//! debug build helpers. A headless engine has none of them: each is the "feature off" answer. Anything here that
//! turns out to carry game logic belongs in its TU instead.

use melee_types::{OSAlarm, OSContext};
use core::ffi::c_void;

type AlarmHandler = unsafe extern "C" fn(*mut OSAlarm, *mut OSContext);

/// The periodic alarm `lb_80019628` arms (`fn_800195FC`: sample the pads into the raw queue). There is no timer
/// interrupt here: the engine fires it once per frame (`fire_alarm`), the 60 Hz case.
static mut ALARM: Option<AlarmHandler> = None;

/// The movie player's frame alarm (lb/lbmthp.c `lbMthp_8001F410`: `OSSetPeriodicAlarm(&alarm, OSSecondsToTicks(1/60), ...,
/// fn_8001F2A4)`, cancelled by `lbMthp_8001F800`). Its period is one 60 Hz frame, so it fires with the pad alarm. Without it
/// the display counter never advances and the opening movie (gm_Scene_Opening_OnFrame) never reaches its end.
static mut MOVIE_ALARM: (Option<AlarmHandler>, *mut OSAlarm) = (None, core::ptr::null_mut());

/// One alarm period elapsed.
pub fn fire_alarm() {
    unsafe {
        if let Some(h) = ALARM {
            h(core::ptr::null_mut(), core::ptr::null_mut());
        }
        if let (Some(h), a) = MOVIE_ALARM {
            h(a, core::ptr::null_mut());
        }
        // the heap compaction's RAM-copy alarm (lbmemory.rs COPY_ALARM): one step per frame when nothing waits on it
        melee_runtime::tu::lbmemory::lbMemory_FireCopyAlarm();
        // the movie's disc reads issued up to now complete (dvd.rs host_dev_com_deferred)
        crate::dvd::run_deferred_completions();
    }
}

/// The GX render mode objects (`GXNtsc480Prog` / `GXNtsc480IntDf` / `GXNtsc480Int`, `gmMainLib_803D4A80`), with the
/// Dolphin SDK's values (gmMainLib_803D4A80 is gm/gmmain_lib.c's). `HSD_VIData` keeps the chosen one; the HSD cameras
/// scale their viewports by `fbWidth / viWidth` and `efbHeight / viHeight` (cobj.c setupNormalCamera), so zeroed records
/// made every normal-camera viewport NaN (0 * inf): the Pokemon Stadium screen camera, the SIS text cameras and so on.
static mut RMODES: [melee_types::GXRenderModeObj; 4] = {
    const fn mode(tv: u32, xfb: u32, vfilter: [u8; 7]) -> melee_types::GXRenderModeObj {
        melee_types::GXRenderModeObj {
            viTVmode: tv,
            fbWidth: 640,
            efbHeight: 480,
            xfbHeight: 480,
            viXOrigin: 40,
            viYOrigin: 0,
            viWidth: 640,
            viHeight: 480,
            xFBmode: xfb,
            field_rendering: 0,
            aa: 0,
            sample_pattern: [[6, 6]; 12],
            vfilter,
        }
    }
    const NTSC_INT: u32 = 0; // VI_TVMODE_NTSC_INT
    const NTSC_PROG: u32 = 2; // VI_TVMODE_NTSC_PROG
    const SF: u32 = 0; // VI_XFBMODE_SF
    const DF: u32 = 1; // VI_XFBMODE_DF
    [
        mode(NTSC_PROG, SF, [0, 0, 21, 22, 21, 0, 0]),
        mode(NTSC_INT, DF, [8, 8, 10, 12, 10, 8, 8]),
        mode(NTSC_INT, DF, [0, 0, 21, 22, 21, 0, 0]),
        mode(NTSC_PROG, SF, [8, 8, 10, 12, 10, 8, 8]),
    ]
};

unsafe extern "C" fn mex_gr_bind_c(p: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    p
}

/// gw_TestTargetTestCKind with no MELEE_TARGET_TEST: not set.
unsafe extern "C" fn no_target_test_ckind() -> i32 {
    crate::scenario::target_test_ckind()
}

pub fn install() {
    install_vi_hooks();
    unsafe {
        melee_runtime::tu::lbmemory::ext::H_OSSetAlarm = Some(melee_runtime::tu::lbmemory::lbMemory_ArmCopyAlarm);
        // the shard 529-536 gm TUs' `tu_ext` hooks (the title's attract demo and the VS scene setup reach them):
        // by name through ext_real, the same path as the string-hook menus (ext_dispatch.rs, gen_ext_dispatch.py)
        crate::ext_dispatch::install_tu_ext();
        // gw_TestTargetTestCKind (gw_runtime.c): the dev boot-into-Target-Test hook; -1 = not set (the menu's flow)
        melee_modes::tu::gmmultiman::ext::H_TestTargetTestCKind = Some(no_target_test_ckind);
        // menuguard: the menu refuses the modes the port does not run yet (melee_modes/src/menuguard.rs)
        melee_modes::menuguard::ENABLED = true;
        {
            // ...except the GX calls of those TUs (the attract demo's own draws): the GX bridge, as every other
            // TU's GX calls, not the seams.txt "off" no-op the by-name path falls to
            use crate::gxbridge as g;
            use melee_modes::tu_ext as m;
            m::HOOK_GXBegin = Some(|p, f, n| g::GXBegin(p as u32, f as u32, n));
            m::HOOK_GXClearVtxDesc = Some(g::GXClearVtxDesc);
            m::HOOK_GXLoadPosMtxImm = Some(|mtx, id| unsafe { g::GXLoadPosMtxImm(mtx as *mut _, id) });
            m::HOOK_GXPosition3f32 = Some(g::GXPosition3f32);
            m::HOOK_GXSetAlphaCompare = Some(|c0, r0, op, c1, r1| g::GXSetAlphaCompare(c0 as u32, r0, op as u32, c1 as u32, r1));
            m::HOOK_GXSetBlendMode = Some(|t, s, d, o| g::GXSetBlendMode(t as u32, s as u32, d as u32, o as u32));
            m::HOOK_GXSetChanCtrl = Some(|c, e, a, mt, l, d, at| g::GXSetChanCtrl(c as u32, e, a as u32, mt as u32, l, d as u32, at as u32));
            m::HOOK_GXSetColorUpdate = Some(g::GXSetColorUpdate);
            m::HOOK_GXSetCullMode = Some(|c| g::GXSetCullMode(c as u32));
            m::HOOK_GXSetCurrentMtx = Some(g::GXSetCurrentMtx);
            m::HOOK_GXSetNumChans = Some(g::GXSetNumChans);
            m::HOOK_GXSetNumTevStages = Some(g::GXSetNumTevStages);
            m::HOOK_GXSetNumTexGens = Some(g::GXSetNumTexGens);
            m::HOOK_GXSetProjection = Some(|mtx, ty| unsafe { g::GXSetProjection(mtx as *mut _, ty as u32) });
            m::HOOK_GXSetTevOp = Some(|s, o| g::GXSetTevOp(s as u32, o as u32));
            m::HOOK_GXSetTevOrder = Some(|s, t, mp, c| g::GXSetTevOrder(s as u32, t as u32, mp as u32, c as u32));
            m::HOOK_GXSetVtxAttrFmt = Some(|f, a, c, t, fr| g::GXSetVtxAttrFmt(f as u32, a as u32, c as u32, t as u32, fr));
            m::HOOK_GXSetVtxDesc = Some(|a, t| g::GXSetVtxDesc(a as u32, t as u32));
            m::HOOK_GXSetZMode = Some(|e, f, u| g::GXSetZMode(e, f as u32, u));
        }
        {
            // gm_801A4014's leave path (a scene ending inside one run: `melee play`'s next match) reads the reset flag
            // of gmMainLib_8046B0F0 (gm/gmmain_lib.c); nothing on the engine path sets it (no reset button).
            use melee_modes::tu::gm_1A3F::ext as m;
            use melee_modes::tu::gmmain_lib::gmMainLib_8046B0F0 as st;
            m::GMMAINLIB_8046B0F0_RESETTING = Some(|| unsafe { st.resetting != 0 });
            m::SET_GMMAINLIB_8046B0F0_RESETTING = Some(|v| unsafe { st.resetting = v as i32 });
            m::SET_GMMAINLIB_8046B0F0_SKIP_INTRO = Some(|v| unsafe { st.skip_intro = v as i32 });
        }
        {
            use melee_modes::tu::gmmain_lib::ext as g;
            g::GXNTSC480PROG = Some(|| (&raw mut RMODES[0]) as *mut core::ffi::c_void);
            g::GXNTSC480INTDF = Some(|| (&raw mut RMODES[1]) as *mut core::ffi::c_void);
            g::GXNTSC480INT = Some(|| (&raw mut RMODES[2]) as *mut core::ffi::c_void);
            g::GMMAINLIB_803D4A80 = Some(|| (&raw mut RMODES[3]) as *mut core::ffi::c_void);
        }
        use melee_modes::tu::gmmain::ext as m;
        m::PADSETSPEC = Some(|_| {});
        // canonical statics the gen_link table does not reach (the `hook!` form of gmmain.rs)
        m::HSD_PADLIBDATA = Some(|| &raw mut melee_runtime::tu::controller::HSD_PadLibData);

        // sound-id lookups (pure table reads; they decide whether ft_PlaySFX draws from the RNG): the real functions
        melee_fighter::hooks::LBAUDIOAX_800233EC = Some(|id| melee_runtime::tu::lbaudio_ax::lbAudioAx_800233EC(id));
        melee_fighter::hooks::LBAUDIOAX_80023130 = Some(|id| melee_runtime::tu::lbaudio_ax::lbAudioAx_80023130(id));
        // a stage's callback table entry is a native pointer here (no guest code): the m-ex bind is the identity
        melee_world::tu_ext::HOOK_Mex_GrBind = Some(|p| p);
        // melee_modes::tu_ext's sound-bank calls (gm_1879's intro scenes import them directly): with no sound library, the
        // no-ops seams_gen gives the TU hooks (the engine audio bridge replaces them in post_link); unset they reach
        // melee_runtime's own lbaudio_ax, whose FGM heap nobody sized ("FGM load size is over")
        {
            use melee_modes::tu_ext as t;
            t::HOOK_lbAudioAx_80026E84 = Some(|_| 0);
            t::HOOK_lbAudioAx_80026F2C = Some(|_| ());
            t::HOOK_lbAudioAx_8002702C = Some(|_, _| ());
            t::HOOK_lbAudioAx_80027168 = Some(|| ());
            t::HOOK_lbAudioAx_80027648 = Some(|| ());
        }
        // the remaining `tu_ext` stage-proc hooks (their defaults stop on "not ported"): the engine's canonical bodies
        melee_world::tu_ext::HOOK_HSD_DObjSetFlags = Some(|d, f| unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjSetFlags(d, f) });
        melee_world::tu_ext::HOOK_HSD_DObjClearFlags = Some(|d, f| unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjClearFlags(d, f) });
        melee_world::tu_ext::HOOK_Player_80031790 = Some(|s| melee_fighter::tu::player::Player_80031790(s));
        melee_world::tu_ext::HOOK_Player_80031900 = Some(|| melee_fighter::tu::player::Player_80031900());
        melee_world::tu_ext::HOOK_ftLib_StartRumble = Some(|g, a, b| melee_fighter::tu::ftlib::ftlib_startrumble(g, a, b));
        melee_world::tu_ext::HOOK_gm_GetDbPauseFlag = Some(|b| unsafe { melee_modes::tu::gmscene::gm_GetDbPauseFlag(b) });
        melee_world::tu_ext::HOOK_gm_MexVanillaKind = Some(|k, c| unsafe { melee_modes::tu::gm_1601::gm_MexVanillaKind(k, c) });
        melee_world::tu_ext::HOOK_gm_801674C4 = Some(|a, b, c, d, e| unsafe { melee_modes::tu::gm_1601::gm_801674C4(a, b, c, d, e) });
        // the same identity for the TUs that carry their own `Mex_GrBind` hook (the generated default would return null, which `on_init` then calls)
        melee_world::tu::grfigure1::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grfigure2::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grfigure3::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grfigureget::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grflatzone::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grfourside::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::grgarden::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::groldkongo::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::groldyoshi::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::gronett::ext::hook::Mex_GrBind = Some(|p| p);
        melee_world::tu::gryorster::ext::H_Mex_GrBind = Some(mex_gr_bind_c);
        melee_world::tu::grzebesroute::ext::H_Mex_GrBind = Some(mex_gr_bind_c);
        // stage procs reach items and the camera through `tu_ext`, whose defaults are the legacy-state ports: run the engine's TUs
        melee_world::tu_ext::HOOK_it_8026B3C0 = Some(|k| melee_items::tu::it_26B1::it_8026B3C0(k as _) as i32);
        melee_world::tu_ext::HOOK_it_802D8618 = Some(|a, b, c, d| melee_items::tu::itheiho::it_802D8618(a, b, c, d));
        melee_world::tu_ext::HOOK_Camera_8003010C = Some(|| melee_world::tu::camera::Camera_8003010C());
        melee_world::tu_ext::HOOK_Camera_80030AE0 = Some(|a| melee_world::tu::camera::Camera_80030AE0(a));
        melee_world::tu_ext::HOOK_Camera_8003118C = Some(|a, b| melee_world::tu::camera::Camera_8003118C(a, b));
        melee_runtime::tu::memory::ext::hook::PcTraceHeapEnabled = Some(|| 0);
        melee_runtime::tu::memory::ext::hook::Snap_NoteMem = Some(|_, _, _, _| {});

        // ARAM queue (shim_ar.c): the "ARAM" of the engine is main memory, a request is a memcpy and completes at once
        {
            use melee_runtime::tu::lbarq::ext::hook as q;
            q::ARQPostRequest = Some(arq_post_request);
            q::DCInvalidateRange = Some(|_, _| {});
            q::wait_idle = Some(|| {});
        }

        // Variadic callees: the hook has fixed (symbols, name, terminator) parameters, the canonical takes the list.
        melee_modes::tu::lb_013B::ext::hook::lbArchive_80017040 = Some(|dst, file, sym, name, term| {
            melee_runtime::tu::lbarchive::lbArchive_80017040(
                dst, file, &[sym, name as *mut core::ffi::c_void, term as usize as *mut core::ffi::c_void]) as i32
        });

        {
            use melee_runtime::tu::lbsnap::ext::hook as sn; // hand-written hook block, not an ext_fns! list
            sn::CARDProbe = Some(|_| -3);
            sn::DCFlushRange = Some(|_, _| {});
            sn::OSGetTime = Some(os_get_time);
            sn::OSTicksToCalendarTime = Some(|_, _| {});
            sn::ft_GetFtKindMask = Some(|| 0);
            sn::gm_GetStKind = Some(|| 0);
        }

        melee_modes::tu::gm_1A36::ext::HSD_PADCOPYSTATUS =
            Some(|| (&raw mut melee_runtime::tu::controller::HSD_PadCopyStatus).cast());

        {
            use melee_runtime::tu::synth::ext::hook as sy; // AX/AI audio hardware: silent
            sy::AISetStreamVolLeft = Some(|_| {});
            sy::AISetStreamVolRight = Some(|_| {});
            sy::AISetDSPSampleRate = Some(|_| {});
            sy::AXInit = Some(|| {});
            sy::AXRegisterCallback = Some(|_| {});
            sy::OSGetSoundMode = Some(|| 1);
            sy::OSSetSoundMode = Some(|_| {});
            sy::wait_idle = Some(|| {});
            sy::watch_page = Some(|_, _| {});
        }

        // m-ex dispatch seams: with no override registered (vanilla game) each runs the per-kind vanilla table entry,
        // as gw_Mex_*Dispatch does (`vanilla != NULL`).
        {
            use melee_fighter::tu::ftdata as d;
            let k = |kind: i32| kind as usize;
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_KNOCKBACK_ENTER")) { melee_fighter::hooks::MEX_ON_KNOCKBACK_ENTER = Some(|kind, g| { if let Some(f) = d::ftData_OnKnockbackEnter[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_KNOCKBACK_EXIT")) { melee_fighter::hooks::MEX_ON_KNOCKBACK_EXIT = Some(|kind, g| { if let Some(f) = d::ftData_OnKnockbackExit[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ITEM_INVISIBLE")) { melee_fighter::hooks::MEX_ON_ITEM_INVISIBLE = Some(|kind, g| { if let Some(f) = d::ftData_OnItemInvisible[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ITEM_VISIBLE")) { melee_fighter::hooks::MEX_ON_ITEM_VISIBLE = Some(|kind, g| { if let Some(f) = d::ftData_OnItemVisible[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_REMOVE_HEAD_ITEM")) { melee_fighter::hooks::MEX_ON_REMOVE_HEAD_ITEM = Some(|kind, g| { if let Some(f) = d::ftData_UnkMotionStates2[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_APPLY_HEAD_ITEM")) { melee_fighter::hooks::MEX_ON_APPLY_HEAD_ITEM = Some(|kind, g| { if let Some(f) = d::ftData_UnkMotionStates1[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ITEM_DROP_EXT")) { melee_fighter::hooks::MEX_ON_ITEM_DROP_EXT = Some(|kind, g, a| { if let Some(f) = d::ftData_OnItemDropExt[kind as usize] { f(g as _, a) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ITEM_DROP")) { melee_fighter::hooks::MEX_ON_ITEM_DROP = Some(|kind, g, a| { if let Some(f) = d::ftData_OnItemDrop[kind as usize] { f(g as _, a) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ITEM_PICKUP2")) { melee_fighter::hooks::MEX_ON_ITEM_PICKUP2 = Some(|kind, g, a| { if let Some(f) = d::ftData_OnItemPickup[kind as usize] { f(g as _, a) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ACTION_STATE_CHANGE_DISPATCH")) { melee_fighter::tu::ftcolanim::ext::MEX_ON_ACTION_STATE_CHANGE_DISPATCH = Some(|kind, g, _vanilla| {
                // gw_Mex_OnActionStateChangeDispatch: an m-ex fighter's onActionStateChange (slot 24) replaces the kind's
                // ftData_UnkMotionStates4 entry, as m-ex installs it (Dedede's sets the sixth ECB joint on every state change)
                unsafe { melee_fighter::platform::Mex_GObjDispatch(melee_fighter::platform::MEX_EVENT_ON_ACTION_STATE_CHANGE, kind, g as _, d::ftData_UnkMotionStates4[kind as usize]) }
            }); }
            // gw_Mex_OnReapplyAttrDispatch: the m-ex onReapplyAttr (slot 25) at ftCo_800D105C's ftKindCalcIndiviParamTable call;
            // without an override the engine's seam stays as it was (seams_gen's no-op: the retail per-kind call is not made here)
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_REAPPLY_ATTR_DISPATCH")) { melee_fighter::tu::ftchangeparam::ext::Mex_OnReapplyAttrDispatch = Some(|kind, g| {
                unsafe { melee_fighter::platform::Mex_GObjDispatch(melee_fighter::platform::MEX_EVENT_ON_REAPPLY_ATTR, kind, g as _, None) }
            }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_MODEL_RENDER")) { melee_fighter::tu::ftdrawcommon::ext::MEX_ON_MODEL_RENDER = Some(|kind, g, a, m| { if let Some(f) = d::ftData_UnkMtxFunc0[kind as usize] { f(g as _, a, m as _) } }); }
            let _ = k;
        }

        {
            use melee_runtime::tu::lbmthp::ext as mt;
            mt::H_OSSetPeriodicAlarm = Some(|a, _, _, h| unsafe { MOVIE_ALARM = (h, a) });
            mt::H_OSCancelAlarm = Some(|_| unsafe { MOVIE_ALARM = (None, core::ptr::null_mut()) });
            // the frame's three I8 planes (lbMthp_8001F67C) go to GX like any texture; the decoder rewrites them in
            // place every frame and DCStoreRange's them (fn_8001EF5C), so that is where the cached copy goes stale
            use crate::gxbridge as gb;
            mt::H_GXInitTexObj = Some(|o, img, w, h, f, ws, wt, m| unsafe { gb::GXInitTexObj(o as _, img, w, h, f as u32, ws as u32, wt as u32, m) });
            mt::H_GXInitTexObjLOD = Some(|o, mn, mg, a, b, c, d, e, an| unsafe { gb::GXInitTexObjLOD(o as _, mn as u32, mg as u32, a, b, c, d, e, an as u32) });
            mt::H_GXLoadTexObj = Some(|o, id| unsafe { gb::GXLoadTexObj(o as _, id as u32) });
            mt::H_DCStoreRange = Some(|p, n| gb::forget_range(p as usize, p as usize + n as usize));
        }
        use melee_modes::tu::lb_0195::ext::hook as l;
        l::OSCancelAlarm = Some(|_| {});
        l::OSCreateAlarm = Some(|_| {});
        l::OSSetPeriodicAlarm = Some(|_, _, _, h| unsafe { ALARM = h });
        l::OSDisableInterrupts = Some(crate::dvd::OSDisableInterrupts);
        l::OSRestoreInterrupts = Some(crate::dvd::OSRestoreInterrupts);
        l::PADSetSamplingRate = Some(|_| {});
        l::OSResetSystem = Some(|_, _, _| {});
        l::VIFlush = Some(|| {});
        l::VISetBlack = Some(|_| {});
        l::VISetPostRetraceCallback = Some(|_| None);
        l::VISetPreRetraceCallback = Some(|_| None);
        l::VIWaitForRetrace = Some(|| {});
    }
}

/// Logic frames run since boot (every engine of the process), the base of the sim's clock ([`os_get_time`]).
pub static SIM_FRAMES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// `OSGetTime`: the console time base (`__OSBusClock / 4` = 40.5 MHz ticks), counted in logic frames, NOT the host's
/// wall clock. The game reads it into the simulation: the title burns `HSD_Rand()` once per second of the clock
/// (gmTitle_801A165C), so a wall clock gave a headless turbo run (title at ~0.3 s) and a windowed run (title at ~5 s)
/// different random streams from the title on (notes/run-modes.md). One frame = 40.5 MHz / 60 = 675000 ticks.
pub fn os_get_time() -> i64 {
    SIM_FRAMES.load(core::sync::atomic::Ordering::Relaxed) as i64 * 675_000
}

/// `GXGetTexBufferSize` (dolphin SDK GXTexture.c): bytes a texture of `format` occupies, tile padded, with the mip
/// chain when `mipmap` is set. The GX texture formats: I4 0, I8 1, IA4 2, IA8 3, RGB565 4, RGB5A3 5, RGBA8 6, C4 8, C8 9,
/// C14X2 10, CMPR 14; Z8 0x11, Z16 0x13, Z24X8 0x16; copy formats 0x20.. (R4 0x20, RA4 0x22, RA8 0x23, A8 0x27, ...).
pub fn gx_tex_buffer_size(width: u16, height: u16, format: u32, mipmap: u8, max_lod: u8) -> u32 {
    let (sx, sy) = match format {
        0 | 8 | 14 | 0x20 | 0x30 => (3, 3),                                    // I4 C4 CMPR R4 Z4: 8x8 texel tiles
        1 | 2 | 9 | 0x11 | 0x22 | 0x27..=0x2A | 0x21 | 0x28..=0x2F => (3, 2), // 8 bit formats: 8x4
        _ => (2, 2),                                                           // 16 / 32 bit formats: 4x4
    };
    let tile = if format == 6 || format == 0x16 { 64 } else { 32 };
    let tiles = |w: u32, h: u32| ((w + (1 << sx) - 1) >> sx) * ((h + (1 << sy) - 1) >> sy);
    let (mut w, mut h) = (width as u32, height as u32);
    if mipmap == 0 {
        return tiles(w, h) * tile;
    }
    let mut total = 0;
    let mut lod = 0u32;
    loop {
        total += tiles(w, h) * tile;
        if lod >= max_lod as u32 || (w == 1 && h == 1) {
            break;
        }
        w = (w >> 1).max(1);
        h = (h >> 1).max(1);
        lod += 1;
    }
    total
}

/// `gw_ARQPostRequest`: record the owner, copy `length` bytes from `source` to `dest` (addresses below 4 GiB: the
/// engine's heaps and archive images are low memory) and run the completion callback.
fn arq_post_request(
    request: *mut melee_types::ARQRequest,
    owner: u32,
    ty: u32,
    priority: u32,
    source: u32,
    dest: u32,
    length: u32,
    callback: Option<unsafe extern "C" fn(*mut melee_types::ARQRequest)>,
) {
    unsafe {
        if !request.is_null() {
            (*request).owner = owner as usize;
            (*request).r#type = ty;
            (*request).priority = priority;
            (*request).source = source;
            (*request).dest = dest;
            (*request).length = length;
        }
        if length != 0 && source != dest {
            let s = ar_addr(source, length as usize);
            let d = ar_addr(dest, length as usize);
            core::ptr::copy(s as *const u8, d, length as usize);
        }
        if let Some(cb) = callback {
            cb(request);
        }
    }
}

/// ARAM size of the port (shim_ar.c `gw_aram_size`): offsets below it address ARAM, anything else is a native pointer.
const ARAM_SIZE: u32 = 32 << 20;

/// `gw_ar_addr`: the host address of ARAM offset / pointer `addr` for a transfer of `len` bytes.
unsafe fn ar_addr(addr: u32, len: usize) -> *mut u8 {
    if addr < ARAM_SIZE {
        let aram = &mut crate::host::Host::get().aram;
        // grown to what the transfer touches (as dvd.rs grows it), not to all 32 MB at once: the first figatree read of
        // a fighter create zero-filled ~30 MB in its frame (zero-hitch); bytes past the old end are zero either way
        let end = (addr as usize + len).min(ARAM_SIZE as usize);
        if aram.len() < end {
            aram.resize(end, 0);
        }
        return aram.as_mut_ptr().add(addr as usize);
    }
    addr as usize as *mut u8
}

/// By-name `ext_fns!` callees (the `melee_modes::tu::ext_hook` form) that are real canonical functions, not "feature off":
/// `ext_fallback` asks here first. Arguments and results are the hook's 64-bit words (floats as their bits).
pub fn ext_real(name: &str, args: &[u64]) -> Option<u64> {
    // the retail scenes only: VS mode keeps the native character / stage select (the C build's MELEE_NATIVE_CSS=1),
    // never the port's own kit screens
    if name == "Frontend_NativeSelect" {
        return Some(1);
    }
    // the sound stack the `melee` crate runs (melee_audio's sim, as in the match), when on
    if name.starts_with("lbAudioAx_") {
        if let Some(v) = melee_modes::tu::ext_hook::audio_route(name, args) {
            return Some(v);
        }
    }
    // Geno's scalar API reached by name (script_game's and others' `ext_fns!` callees)
    if name.starts_with("Geno_") {
        if let Some(v) = crate::geno_host::by_name(name, args) {
            return Some(v);
        }
    }
    // the script host's side of script_game's callees (pc/platform/gw_script*.c's gw_Script_*)
    if name.starts_with("Script_") {
        if let Some(v) = crate::script_host::by_name(name, args) {
            return Some(v);
        }
    }
    if name.starts_with("Mex_") {
        if let Some(v) = crate::mex_host::by_name(name, args) {
            return Some(v);
        }
    }
    // a define's menu / results art (gmresultplayer_geno_art.inc: the package file as a .gxtex handle, the initials
    // over UI_TextI4) and the skin registry (gw_skins_boot.inc), as the C's gw_Geno_* / gw_GxTex_* / gw_Skins_* answer
    if name.starts_with("Geno_") || name.starts_with("GxTex_") || name.starts_with("Skins_") || name == "UI_TextI4" {
        if let Some(v) = crate::geno_host::by_name_art(name, args) {
            return Some(v);
        }
    }
    if let Some(v) = ext_hand(name, args) {
        return Some(v);
    }
    // gw_Script_CpuIdleDefault (gw_runtime.c): the scene's `pN=.../idle` slots stand still (2), else follow the retail AI
    if name == "Script_CpuIdleDefault" {
        return Some(crate::scenario::cpu_idle_default(args.first().copied().unwrap_or(0) as i32) as u64);
    }
    if let Some(v) = unsafe { crate::ext_dispatch::dispatch(name, args) } {
        return Some(v);
    }
    if let Some(v) = crate::ext_dispatch::seam_off(name) {
        log_stub(name, v);
        return Some(v);
    }
    None
}

/// The callees the generated table cannot spell (C varargs, inlines of other calls, the engine's DVD); checked first.
fn ext_hand(name: &str, args: &[u64]) -> Option<u64> {
    Some(match name {
        // shim_vi.c gw_View_SceneBegin / gw_View_AspectBits (view.rs)
        "View_SceneBegin" => { crate::view::scene_begin(args.first().copied().unwrap_or(u64::MAX) as u32 as i32); 0 }
        "View_AspectBits" => crate::view::aspect_bits() as u32 as u64,
        // GX calls reached by name (string-hook TUs): the bridge / the SDK size formula, not the "off" zero
        "GXSetScissor" => { crate::gxbridge::GXSetScissor(args[0] as u32, args[1] as u32, args[2] as u32, args[3] as u32); 0 }
        "GXGetTexBufferSize" => gx_tex_buffer_size(args[0] as u16, args[1] as u16, args[2] as u32, args[3] as u8, args[4] as u8) as u64,
        // ft/ftdemo.c (melee_fighter tu::ftdemo, snake_case names the dispatcher cannot match)
        "ftDemo_ObjAllocInit" => { melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init(); 0 }
        "ftDemo_CreateFighter" => unsafe {
            melee_fighter::tu::ftdemo::ftdemo_create_fighter(&*(args[0] as usize as *const melee_types::plAllocInfo2)) as usize as u64
        },
        "ftDemo_SetArchiveData" => {
            melee_fighter::tu::ftdemo::ftdemo_set_archive_data(args[0] as i32, args[1] as usize as *mut melee_types::HSD_Archive, args[2] as i32);
            0
        }
        "ftDemo_GetMotionFileString" => melee_fighter::tu::ftdemo::ftdemo_get_motion_file_string(args[0] as i32, args[1] as i32) as usize as u64,
        "ftDemo_SetFacingDirection" => {
            melee_fighter::tu::ftdemo::ftdemo_set_facing_direction(args[0] as usize as *mut melee_types::HSD_GObj, f32::from_bits(args[1] as u32));
            0
        }
        "lbArchive_LoadSymbols" => unsafe {
            let rest: Vec<*mut c_void> = args[1..].iter().map(|&w| w as usize as *mut c_void).collect();
            melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(args[0] as usize as *const core::ffi::c_char, &rest) as usize as u64
        },
        "lbArchive_LoadSections" => unsafe {
            let rest: Vec<*mut c_void> = args[1..].iter().map(|&w| w as usize as *mut c_void).collect();
            melee_runtime::tu::lbarchive::lbArchive_LoadSections(args[0] as usize as *mut melee_types::HSD_Archive, &rest);
            0
        },
        "HSD_SisLib_803A6B98" => unsafe {
            let s = c_format(args[3], &args[4..]);
            melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6B98_bytes(args[0] as usize as *mut melee_types::HSD_Text, f32::from_bits(args[1] as u32), f32::from_bits(args[2] as u32), Some(&s)) as i64 as u64
        },
        "OSReport" => 0,
        // gw_runtime.c gw_GxTex_OpenEnv: MELEE_MENUTEX names a replacement texture; unset (no mods) it is -1 and the
        // CSS keeps the disc's art. The handle queries are geno_host's table (`by_name_art`, checked before this).
        "GxTex_OpenEnv" | "GxTex_OpenUI" => -1i64 as u64,
        "HSD_SisLib_803A70A0" => unsafe {
            let s = c_format(args[2], &args[3..]);
            melee_runtime::tu::hsd_3A64::HSD_SisLib_803A70A0_bytes(args[0] as usize as *mut melee_types::HSD_Text, args[1] as i32, Some(&s)) as i64 as u64
        },
        // aobj.c HSD_ForeachAnim(obj, type, mask, func, arg_type, ...): the one variadic read by arg_type
        // (AOBJ_ARG_*F: a double, *V: a pointer, *U: a u32, A / AO / AOT: none)
        "HSD_ForeachAnim" => unsafe {
            use melee_runtime::tu::aobj::CallbackArg;
            let at = args[4] as u32;
            let w = args.get(5).copied().unwrap_or(0);
            let arg = match at {
                1 | 5 | 9 => CallbackArg { f: f64::from_bits(w) as f32 },
                2 | 6 | 10 => CallbackArg { v: w as usize as *mut c_void },
                3 | 7 | 11 => CallbackArg { d: w as u32 },
                _ => CallbackArg { d: 0 },
            };
            melee_runtime::tu::aobj::HSD_ForeachAnim(args[0] as usize as *mut c_void, args[1] as _, args[2] as _, args[3] as usize as *mut c_void, at as _, arg);
            0
        },
        "DVDConvertPathToEntrynum" => crate::dvd::DVDConvertPathToEntrynum(args[0] as usize as *const core::ffi::c_char) as i64 as u64,
        // baselib/jobj.c HSD_JObjReqAnim: HSD_JObjReqAnimByFlags(jobj, 0x7FF, frame)
        "HSD_JObjReqAnim" => unsafe {
            melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimByFlags(args[0] as usize as *mut melee_types::HSD_JObj, 0x7FF, f32::from_bits(args[1] as u32));
            0
        },
        "HSD_JObjRemoveAll" => unsafe { melee_runtime::tu::jobj::HSD_JObjRemoveAll_ptr(args[0] as usize as *mut melee_types::HSD_JObj); 0 },
        // lb_80011E24(root, out, idx, ..., -1): the variadic index list
        "lb_80011E24" => unsafe {
            let idx: Vec<i32> = args[2..].iter().map(|&w| w as i32).collect();
            melee_runtime::tu::lbspdisplay::lb_80011E24(args[0] as usize as *mut melee_types::HSD_JObj, args[1] as usize as *mut *mut melee_types::HSD_JObj, &idx) as i64 as u64
        },
        "fabsf" => f32::from_bits(args[0] as u32).abs().to_bits() as u64,
        "__frsqrte" => melee_runtime::tu::gekko_fp::__frsqrte(f64::from_bits(args[0])).to_bits(),
        "__fres" => melee_runtime::tu::gekko_fp::__fres(f64::from_bits(args[0])).to_bits(),
        _ => return None,
    })
}

/// A C `printf` of `fmt` (a NUL-terminated guest string) with its variadic words: %s %d %i %u %x %X %c %%, widths kept.
/// Byte-exact: the game's text is Shift-JIS, so nothing goes through a Rust `String`.
pub unsafe fn c_format(fmt: u64, va: &[u64]) -> Vec<u8> {
    let cstr = |p: u64| if p == 0 { Vec::new() } else { unsafe { std::ffi::CStr::from_ptr(p as usize as *const core::ffi::c_char) }.to_bytes().to_vec() };
    let f = cstr(fmt);
    let mut out = Vec::new();
    let mut it = f.iter().copied().peekable();
    let mut k = 0;
    while let Some(c) = it.next() {
        if c != b'%' { out.push(c); continue; }
        let mut spec = String::new();
        while let Some(&d) = it.peek() { if d.is_ascii_digit() || d == b'-' || d == b'.' || d == b'l' { spec.push(d as char); it.next(); } else { break; } }
        let width: usize = spec.trim_start_matches('-').trim_start_matches('0').split('.').next().unwrap_or("").trim_end_matches('l').parse().unwrap_or(0);
        let zero = spec.starts_with('0');
        let w = va.get(k).copied().unwrap_or(0);
        let body: Vec<u8> = match it.next() {
            Some(b'%') => { out.push(b'%'); continue; }
            Some(b's') => { k += 1; cstr(w) }
            Some(b'd') | Some(b'i') => { k += 1; (w as i32).to_string().into_bytes() }
            Some(b'u') => { k += 1; (w as u32).to_string().into_bytes() }
            Some(b'x') => { k += 1; format!("{:x}", w as u32).into_bytes() }
            Some(b'X') => { k += 1; format!("{:X}", w as u32).into_bytes() }
            Some(b'c') => { k += 1; vec![w as u8] }
            Some(o) => { k += 1; vec![o] }
            None => break,
        };
        if body.len() < width { out.extend(std::iter::repeat(if zero { b'0' } else { b' ' }).take(width - body.len())); }
        out.extend_from_slice(&body);
    }
    out
}

/// `MELEE_LOG_STUBS=1`: the first call of each host seam answered with its "off" value through the by-name table
/// (`engine: stub NAME -> VALUE`), so a run lists which ones a scene actually reached.
fn log_stub(name: &str, v: u64) {
    use std::sync::{Mutex, OnceLock};
    static ON: OnceLock<bool> = OnceLock::new();
    static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
    if !*ON.get_or_init(|| std::env::var_os("MELEE_LOG_STUBS").is_some_and(|v| v != "0")) {
        return;
    }
    let mut seen = SEEN.lock().unwrap();
    if !seen.iter().any(|s| s == name) {
        seen.push(name.to_string());
        eprintln!("engine: stub {name} -> {v:#x}");
    }
}

/// `OSTicksToCalendarTime` (extern/aurora/lib/dolphin/os/OSTime.cpp, which the C PC build links): `ticks` are UTC relative
/// to the GameCube epoch (2000-01-01 00:00:00), broken up in the local time zone. Written natively (the C byte-swaps for
/// its big-endian readers; the Rust TUs read the record natively).
pub fn os_ticks_to_calendar_time(ticks: i64, td: *mut melee_types::OSCalendarTime) {
    const OS_TIMER_CLOCK: i64 = 162_000_000 / 4;
    let us = ticks / (OS_TIMER_CLOCK / 125_000) * 8; // OSTicksToMicroseconds: ticks * 8 / (clock / 125000)
    let unix_us = us + 946_684_800i64 * 1_000_000;
    let secs = unix_us.div_euclid(1_000_000);
    let sub_us = unix_us.rem_euclid(1_000_000) as i32;
    #[repr(C)]
    #[derive(Default)]
    struct Tm { sec: i32, min: i32, hour: i32, mday: i32, mon: i32, year: i32, wday: i32, yday: i32, isdst: i32 }
    let mut tm = Tm::default();
    #[cfg(windows)]
    {
        extern "C" { fn _localtime64_s(tm: *mut Tm, t: *const i64) -> i32; }
        unsafe { _localtime64_s(&mut tm, &secs) };
    }
    // libc's own `struct tm`: glibc's has tm_gmtoff and tm_zone after the nine ints, so localtime_r into the Windows-shaped
    // `Tm` wrote 20 bytes past it (the zone offset landed in a saved register: the prize screen's date after a netplay
    // match segfaulted in gm_801692E8)
    #[cfg(not(windows))]
    {
        let mut lt: libc::tm = unsafe { core::mem::zeroed() };
        let t = secs as libc::time_t;
        unsafe { libc::localtime_r(&t, &mut lt) };
        tm = Tm { sec: lt.tm_sec, min: lt.tm_min, hour: lt.tm_hour, mday: lt.tm_mday, mon: lt.tm_mon, year: lt.tm_year, wday: lt.tm_wday,
                  yday: lt.tm_yday, isdst: lt.tm_isdst };
    }
    if td.is_null() {
        return;
    }
    unsafe {
        *td = melee_types::OSCalendarTime {
            sec: tm.sec, min: tm.min, hour: tm.hour, mday: tm.mday, mon: tm.mon, year: tm.year + 1900, wday: tm.wday,
            yday: tm.yday, msec: sub_us / 1000, usec: sub_us % 1000,
        };
    }
}

/// The Adventure cutscenes whose ext blocks are `todo_fns!` (no link-table hooks): vi0801 (F-Zero), vi1101 (Metal Mario),
/// vi1201v1 / vi1201v2 (Bowser's trophy, Giga Bowser), vi1202 (Giga Bowser defeated). Their callees in crates melee_items
/// cannot name, pointed at the canonical ports.
#[allow(clippy::unnecessary_cast)]
pub fn install_vi_hooks() {
    unsafe {
        // ftData_803C24EC (the demo motion file getters of Mario, Luigi, Giga Bowser): the link table's adapters cast their
        // `&str` / `Option<&str>` result to the C's `char*` and stop on the size check; NUL-terminated copies here
        let t = &raw mut melee_fighter::tu::ftdata::ftData_803C24EC;
        (*t)[0] = Some(demo_motion_mario);
        (*t)[17] = Some(demo_motion_luigi);
        (*t)[31] = Some(demo_motion_gkoopa);
        // the todo_fns! cutscenes' sound calls: no-ops with no sound library (their defaults reach melee_runtime's lbaudio_ax,
        // whose FGM heap nobody sized); melee's engine_audio post_link points them at the bridge
        melee_items::tu::vi0801::ext::hook::lbAudioAx_800236DC = Some(|| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80023F28 = Some(|_a0| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80024E50 = Some(|_a0| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80026F2C = Some(|_a0| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_8002702C = Some(|_a0, _a1| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80027168 = Some(|| ());
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80027648 = Some(|| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_800236DC = Some(|| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_800237A8 = Some(|_a0, _a1, _a2| 0);
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80023F28 = Some(|_a0| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80024E50 = Some(|_a0| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80026E84 = Some(|_a0| 0);
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80026F2C = Some(|_a0| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_8002702C = Some(|_a0, _a1| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80027168 = Some(|| ());
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80027648 = Some(|| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_800236DC = Some(|| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_800237A8 = Some(|_a0, _a1, _a2| 0);
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80023F28 = Some(|_a0| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80024E50 = Some(|_a0| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80026F2C = Some(|_a0| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_8002702C = Some(|_a0, _a1| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80027168 = Some(|| ());
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80027648 = Some(|| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80023F28 = Some(|_a0| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80024E50 = Some(|_a0| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80026F2C = Some(|_a0| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_8002702C = Some(|_a0, _a1| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80027168 = Some(|| ());
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80027648 = Some(|| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80023694 = Some(|| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_800236DC = Some(|| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80023F28 = Some(|_a0| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80024E50 = Some(|_a0| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80026F2C = Some(|_a0| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_8002702C = Some(|_a0, _a1| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80027168 = Some(|| ());
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80027648 = Some(|| ());
        // vi0102 / vi0401 / vi0501 / vi0502: their ftDemo_ObjAllocInit hook (the link table leaves it unresolved)
        melee_items::tu::vi0102::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi0401::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi0501::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi0502::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi0801::ext::hook::Camera_800310A0 = Some(|a0| melee_world::tu::camera::Camera_800310A0(a0 as _));
        melee_items::tu::vi0801::ext::hook::efAsync_LoadSync = Some(|a0| { melee_world::tu::efn::efasync::efAsync_LoadSync(a0 as _); 0 });
        melee_items::tu::vi0801::ext::hook::gm_801A4B60 = Some(|| melee_modes::tu::gmscene::gm_801A4B60());
        melee_items::tu::vi0801::ext::hook::gm_8016895C = Some(|a0, a1, a2| melee_modes::tu::gm_1601::gm_8016895C(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi0801::ext::hook::Stage_InitScene = Some(|a0, a1| melee_modes::tu::hsd::Stage_InitScene(a0 as _, a1 as _));
        melee_items::tu::vi0801::ext::hook::lb_80011E24 = Some(|a0, a1, a2, a3| melee_runtime::tu::lbspdisplay::lb_80011E24(a0, a1, &[a2, a3]));
        melee_items::tu::vi0801::ext::hook::lb_800145F4 = Some(|| melee_modes::tu::lb_013B::lb_800145F4());
        melee_items::tu::vi0801::ext::hook::Player_InitAllPlayers = Some(|| melee_fighter::tu::player::Player_InitAllPlayers());
        melee_items::tu::vi0801::ext::hook::GObj_Create = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _);
        melee_items::tu::vi0801::ext::hook::GObj_SetupGXLink = Some(|a0, a1, a2, a3| melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi0801::ext::hook::GObj_SetupGXLinkMax = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi0801::ext::hook::HSD_FogLoadDesc = Some(|a0| melee_runtime::tu::fog::HSD_FogLoadDesc(a0 as _) as _);
        melee_items::tu::vi0801::ext::hook::HSD_GObj_80390ED0 = Some(|a0, a1| melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _));
        melee_items::tu::vi0801::ext::hook::HSD_GObjObject_80390A70 = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi0801::ext::hook::HSD_JObjAnimAll = Some(|a0| melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(a0 as _));
        melee_items::tu::vi0801::ext::hook::HSD_JObjReqAnimAll = Some(|a0, a1| melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(a0 as _, a1 as _));
        melee_items::tu::vi0801::ext::hook::HSD_SetEraseColor = Some(|a0, a1, a2, a3| melee_runtime::tu::displayfunc_native::HSD_SetEraseColor(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1101::ext::hook::efAsync_LoadSync = Some(|a0| { melee_world::tu::efn::efasync::efAsync_LoadSync(a0 as _); 0 });
        melee_items::tu::vi1101::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi1101::ext::hook::gm_801A4B60 = Some(|| melee_modes::tu::gmscene::gm_801A4B60());
        melee_items::tu::vi1101::ext::hook::gm_8016895C = Some(|a0, a1, a2| melee_modes::tu::gm_1601::gm_8016895C(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1101::ext::hook::gm_IsCKindUnlocked = Some(|a0| melee_modes::tu::gm_1601::gm_IsCKindUnlocked(a0 as _) as _);
        melee_items::tu::vi1101::ext::hook::Stage_InitScene = Some(|a0, a1| melee_modes::tu::hsd::Stage_InitScene(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::lb_800145F4 = Some(|| melee_modes::tu::lb_013B::lb_800145F4());
        melee_items::tu::vi1101::ext::hook::Player_80032768 = Some(|a0, a1| melee_fighter::tu::player::Player_80032768(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_80036E20 = Some(|a0, a1, a2| melee_fighter::tu::player::Player_80036E20(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1101::ext::hook::Player_80036F34 = Some(|a0, a1| melee_fighter::tu::player::Player_80036F34(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_GetEntity = Some(|a0| melee_fighter::tu::player::Player_GetEntity(a0 as _) as _);
        melee_items::tu::vi1101::ext::hook::Player_InitAllPlayers = Some(|| melee_fighter::tu::player::Player_InitAllPlayers());
        melee_items::tu::vi1101::ext::hook::Player_SetCostumeId = Some(|a0, a1| melee_fighter::tu::player::Player_SetCostumeId(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_SetFacingDirection = Some(|a0, a1| melee_fighter::tu::player::Player_SetFacingDirection(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_SetFlagsBit5 = Some(|a0, a1| melee_fighter::tu::player::Player_SetFlagsBit5(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_SetPlayerCharacter = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerCharacter(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_SetPlayerId = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerId(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::Player_SetSlottype = Some(|a0, a1| melee_fighter::tu::player::Player_SetSlottype(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::GObj_Create = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _);
        melee_items::tu::vi1101::ext::hook::GObj_SetupGXLink = Some(|a0, a1, a2, a3| melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1101::ext::hook::GObj_SetupGXLinkMax = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1101::ext::hook::HSD_GObj_80390ED0 = Some(|a0, a1| melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::HSD_GObjObject_80390A70 = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1101::ext::hook::HSD_JObjAnimAll = Some(|a0| melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(a0 as _));
        melee_items::tu::vi1101::ext::hook::HSD_JObjReqAnimAll = Some(|a0, a1| melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(a0 as _, a1 as _));
        melee_items::tu::vi1101::ext::hook::HSD_SetEraseColor = Some(|a0, a1, a2, a3| melee_runtime::tu::displayfunc_native::HSD_SetEraseColor(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1201v1::ext::hook::efAsync_LoadSync = Some(|a0| { melee_world::tu::efn::efasync::efAsync_LoadSync(a0 as _); 0 });
        melee_items::tu::vi1201v1::ext::hook::efSync_Spawn = Some(|a0, a1, a2, a3, a4, a5, a6| melee_world::tu::efn::efsync::efSync_Spawn(a0, a1, &[a2 as *const c_void, a3 as *const c_void, a4 as *const c_void, a5 as usize as *const c_void, a6 as usize as *const c_void]));
        melee_items::tu::vi1201v1::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi1201v1::ext::hook::gm_801A4B60 = Some(|| melee_modes::tu::gmscene::gm_801A4B60());
        melee_items::tu::vi1201v1::ext::hook::gm_8016895C = Some(|a0, a1, a2| melee_modes::tu::gm_1601::gm_8016895C(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v1::ext::hook::gm_80160438 = Some(|a0| melee_modes::tu::gm_1601::gm_80160438(a0 as _) as _);
        melee_items::tu::vi1201v1::ext::hook::Stage_802251E8 = Some(|a0, a1| melee_world::tu::stage::Stage_802251E8(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::lb_80011E24 = Some(|a0, a1, a2, a3| melee_runtime::tu::lbspdisplay::lb_80011E24(a0, a1, &[a2, a3]));
        melee_items::tu::vi1201v1::ext::hook::lb_800145F4 = Some(|| melee_modes::tu::lb_013B::lb_800145F4());
        melee_items::tu::vi1201v1::ext::hook::mn_8022F298 = Some(|a0| melee_modes::tu::mn_22EC::mn_8022F298(a0 as _) as _);
        melee_items::tu::vi1201v1::ext::hook::Player_80032768 = Some(|a0, a1| melee_fighter::tu::player::Player_80032768(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_80036E20 = Some(|a0, a1, a2| melee_fighter::tu::player::Player_80036E20(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_80036F34 = Some(|a0, a1| melee_fighter::tu::player::Player_80036F34(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_GetEntity = Some(|a0| melee_fighter::tu::player::Player_GetEntity(a0 as _) as _);
        melee_items::tu::vi1201v1::ext::hook::Player_InitAllPlayers = Some(|| melee_fighter::tu::player::Player_InitAllPlayers());
        melee_items::tu::vi1201v1::ext::hook::Player_SetCostumeId = Some(|a0, a1| melee_fighter::tu::player::Player_SetCostumeId(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_SetFacingDirection = Some(|a0, a1| melee_fighter::tu::player::Player_SetFacingDirection(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_SetPlayerCharacter = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerCharacter(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_SetPlayerId = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerId(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::Player_SetSlottype = Some(|a0, a1| melee_fighter::tu::player::Player_SetSlottype(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::GObj_Create = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _);
        melee_items::tu::vi1201v1::ext::hook::GObj_SetupGXLink = Some(|a0, a1, a2, a3| melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1201v1::ext::hook::GObj_SetupGXLinkMax = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_FogLoadDesc = Some(|a0| melee_runtime::tu::fog::HSD_FogLoadDesc(a0 as _) as _);
        melee_items::tu::vi1201v1::ext::hook::HSD_GObj_80390ED0 = Some(|a0, a1| melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_GObjFree = Some(|a0| melee_runtime::tu::gobjplink::HSD_GObjFree(a0 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_GObjObject_80390A70 = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_JObjAnimAll = Some(|a0| melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(a0 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_JObjReqAnimAll = Some(|a0, a1| melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(a0 as _, a1 as _));
        melee_items::tu::vi1201v1::ext::hook::HSD_SetEraseColor = Some(|a0, a1, a2, a3| melee_runtime::tu::displayfunc_native::HSD_SetEraseColor(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1201v2::ext::hook::efAsync_LoadSync = Some(|a0| { melee_world::tu::efn::efasync::efAsync_LoadSync(a0 as _); 0 });
        melee_items::tu::vi1201v2::ext::hook::ftDemo_ObjAllocInit = Some(|| melee_fighter::tu::ftdemo::ftdemo_obj_alloc_init());
        melee_items::tu::vi1201v2::ext::hook::gm_801A4B60 = Some(|| melee_modes::tu::gmscene::gm_801A4B60());
        melee_items::tu::vi1201v2::ext::hook::gm_8016895C = Some(|a0, a1, a2| melee_modes::tu::gm_1601::gm_8016895C(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v2::ext::hook::Stage_InitScene = Some(|a0, a1| melee_modes::tu::hsd::Stage_InitScene(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::lb_80011E24 = Some(|a0, a1, a2, a3| melee_runtime::tu::lbspdisplay::lb_80011E24(a0, a1, &[a2, a3]));
        melee_items::tu::vi1201v2::ext::hook::lb_800145F4 = Some(|| melee_modes::tu::lb_013B::lb_800145F4());
        melee_items::tu::vi1201v2::ext::hook::mn_8022EAE0 = Some(|a0| melee_modes::tu::mnmain::mn_8022EAE0(a0 as _));
        melee_items::tu::vi1201v2::ext::hook::mn_8022F298 = Some(|a0| melee_modes::tu::mn_22EC::mn_8022F298(a0 as _) as _);
        melee_items::tu::vi1201v2::ext::hook::Player_80032768 = Some(|a0, a1| melee_fighter::tu::player::Player_80032768(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_80032BB0 = Some(|a0| melee_fighter::tu::player::Player_80032BB0(a0 as _) as _);
        melee_items::tu::vi1201v2::ext::hook::Player_80036E20 = Some(|a0, a1, a2| melee_fighter::tu::player::Player_80036E20(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_80036F34 = Some(|a0, a1| melee_fighter::tu::player::Player_80036F34(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_InitAllPlayers = Some(|| melee_fighter::tu::player::Player_InitAllPlayers());
        melee_items::tu::vi1201v2::ext::hook::Player_SetCostumeId = Some(|a0, a1| melee_fighter::tu::player::Player_SetCostumeId(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetFacingDirection = Some(|a0, a1| melee_fighter::tu::player::Player_SetFacingDirection(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetPlayerAndEntityFacingDirection = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerAndEntityFacingDirection(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetPlayerCharacter = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerCharacter(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetPlayerId = Some(|a0, a1| melee_fighter::tu::player::Player_SetPlayerId(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetScale = Some(|a0, a1| melee_fighter::tu::player::Player_SetScale(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::Player_SetSlottype = Some(|a0, a1| melee_fighter::tu::player::Player_SetSlottype(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::GObj_Create = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _);
        melee_items::tu::vi1201v2::ext::hook::GObj_SetupGXLink = Some(|a0, a1, a2, a3| melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1201v2::ext::hook::GObj_SetupGXLinkMax = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_FogInterpretAnim = Some(|a0| melee_runtime::tu::fog::HSD_FogInterpretAnim(a0 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_FogLoadDesc = Some(|a0| melee_runtime::tu::fog::HSD_FogLoadDesc(a0 as _) as _);
        melee_items::tu::vi1201v2::ext::hook::HSD_FogReqAnim = Some(|a0, a1| melee_runtime::tu::fog::HSD_FogReqAnim(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_Fog_8037DE7C = Some(|a0, a1| melee_runtime::tu::fog::HSD_Fog_8037DE7C(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_GObj_80390ED0 = Some(|a0, a1| melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_GObjFree = Some(|a0| melee_runtime::tu::gobjplink::HSD_GObjFree(a0 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_GObjObject_80390A70 = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_JObjAnimAll = Some(|a0| melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(a0 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_JObjReqAnimAll = Some(|a0, a1| melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(a0 as _, a1 as _));
        melee_items::tu::vi1201v2::ext::hook::HSD_SetEraseColor = Some(|a0, a1, a2, a3| melee_runtime::tu::displayfunc_native::HSD_SetEraseColor(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1202::ext::hook::efAsync_LoadSync = Some(|a0| { melee_world::tu::efn::efasync::efAsync_LoadSync(a0 as _); 0 });
        melee_items::tu::vi1202::ext::hook::gm_801A4B60 = Some(|| melee_modes::tu::gmscene::gm_801A4B60());
        melee_items::tu::vi1202::ext::hook::gm_8016895C = Some(|a0, a1, a2| melee_modes::tu::gm_1601::gm_8016895C(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1202::ext::hook::lb_80011E24 = Some(|a0, a1, a2, a3| melee_runtime::tu::lbspdisplay::lb_80011E24(a0, a1, &[a2, a3]));
        melee_items::tu::vi1202::ext::hook::lb_800145F4 = Some(|| melee_modes::tu::lb_013B::lb_800145F4());
        melee_items::tu::vi1202::ext::hook::GObj_Create = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _);
        melee_items::tu::vi1202::ext::hook::GObj_SetupGXLink = Some(|a0, a1, a2, a3| melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _));
        melee_items::tu::vi1202::ext::hook::GObj_SetupGXLinkMax = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1202::ext::hook::HSD_GObj_803910D8 = Some(|a0, a1| melee_runtime::tu::gobj_native::HSD_GObj_803910D8(a0 as _, a1 as _));
        melee_items::tu::vi1202::ext::hook::HSD_GObjObject_80390A70 = Some(|a0, a1, a2| melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _));
        melee_items::tu::vi1202::ext::hook::HSD_JObjAnimAll = Some(|a0| melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(a0 as _));
        melee_items::tu::vi1202::ext::hook::HSD_JObjReqAnimAll = Some(|a0, a1| melee_runtime::tu::jobj_ptr::HSD_JObjReqAnimAll(a0 as _, a1 as _));
    }
}

/// A `'static` name as the C string the demo loader reads (one leaked copy per name).
fn demo_cstr(name: Option<&'static str>) -> *mut i8 {
    static CACHE: std::sync::Mutex<Vec<(&'static str, usize)>> = std::sync::Mutex::new(Vec::new());
    let Some(name) = name else { return core::ptr::null_mut() };
    let mut c = CACHE.lock().unwrap();
    if let Some(&(_, p)) = c.iter().find(|(n, _)| *n == name) {
        return p as *mut i8;
    }
    let p = std::ffi::CString::new(name).unwrap().into_raw() as usize;
    c.push((name, p));
    p as *mut i8
}
unsafe extern "C" fn demo_motion_mario(k: melee_types::enum_t) -> *mut i8 {
    demo_cstr(Some(melee_chara::tu::ftmario::ftMr_Init_GetMotionFileString(k as u32)))
}
unsafe extern "C" fn demo_motion_luigi(k: melee_types::enum_t) -> *mut i8 {
    demo_cstr(melee_chara::tu::ftluigi::ftLg_Init_GetMotionFileString(k as i32))
}
unsafe extern "C" fn demo_motion_gkoopa(k: melee_types::enum_t) -> *mut i8 {
    demo_cstr(melee_chara::tu::ftgkoopa::ftGk_Init_GetMotionFileString(k as i32))
}

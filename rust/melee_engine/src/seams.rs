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

/// One alarm period elapsed.
pub fn fire_alarm() {
    unsafe {
        if let Some(h) = ALARM {
            h(core::ptr::null_mut(), core::ptr::null_mut());
        }
    }
}

/// The GX render mode objects (`GXNtsc480Prog` / `GXNtsc480IntDf` / `GXNtsc480Int`, `gmMainLib_803D4A80`): the
/// headless engine only stores them in `HSD_VIData`, so zeroed records stand in.
static mut RMODES: [melee_types::GXRenderModeObj; 4] = unsafe { core::mem::zeroed() };

/// The real render modes (libs/dolphin/src/dolphin/gx/GXFrameBuf.c `GXNtsc480Prog` / `GXNtsc480IntDf` /
/// `GXNtsc480Int`, gm/gmmain_lib.c `gmMainLib_803D4A80`): SDK constants, not disc data. The retail mode loop
/// (`Engine::boot_modes`) fills `RMODES` with them so `HSD_VISetConfigure` stores a real mode and the menu cameras'
/// `setupNormalCamera` scales their viewports by fbWidth / viWidth (zeroed records give 0 / 0 = NaN viewports).
/// The VS-only paths keep the zeroed records their goldens were recorded with.
pub fn fill_render_modes() {
    fn mode(tv: u32, xfb: u32, vf: [u8; 7]) -> melee_types::GXRenderModeObj {
        let mut m: melee_types::GXRenderModeObj = unsafe { core::mem::zeroed() };
        m.viTVmode = tv as _;
        m.fbWidth = 640;
        m.efbHeight = 480;
        m.xfbHeight = 480;
        m.viXOrigin = 40;
        m.viYOrigin = 0;
        m.viWidth = 640;
        m.viHeight = 480;
        m.xFBmode = xfb as _;
        m.field_rendering = 0;
        m.aa = 0;
        m.sample_pattern = [[6, 6]; 12];
        m.vfilter = vf;
        m
    }
    unsafe {
        RMODES[0] = mode(2, 0, [0, 0, 21, 22, 21, 0, 0]); // GXNtsc480Prog
        RMODES[1] = mode(0, 1, [8, 8, 10, 12, 10, 8, 8]); // GXNtsc480IntDf
        RMODES[2] = mode(0, 1, [0, 0, 21, 22, 21, 0, 0]); // GXNtsc480Int
        RMODES[3] = mode(2, 0, [8, 8, 10, 12, 10, 8, 8]); // gmMainLib_803D4A80
    }
}

unsafe extern "C" fn mex_gr_bind_c(p: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    p
}

pub fn install() {
    unsafe {
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
        // the remaining `tu_ext` stage-proc hooks (their defaults stop on "not ported"): the engine's canonical bodies
        melee_world::tu_ext::HOOK_HSD_DObjSetFlags = Some(|d, f| unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjSetFlags(d, f) });
        melee_world::tu_ext::HOOK_HSD_DObjClearFlags = Some(|d, f| unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjClearFlags(d, f) });
        melee_world::tu_ext::HOOK_Player_80031790 = Some(|s| melee_fighter::tu::player::Player_80031790(s));
        melee_world::tu_ext::HOOK_Player_80031900 = Some(|| melee_fighter::tu::player::Player_80031900());
        melee_world::tu_ext::HOOK_ftLib_80086C18 = Some(|g, a, b| melee_fighter::tu::ftlib::ftlib_80086c18(g, a, b));
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
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_ACTION_STATE_CHANGE_DISPATCH")) { melee_fighter::tu::ftcolanim::ext::MEX_ON_ACTION_STATE_CHANGE_DISPATCH = Some(|kind, g, _vanilla| { if let Some(f) = d::ftData_UnkMotionStates4[kind as usize] { f(g as _) } }); }
            if std::env::var("MELEE_MEX_OFF").map_or(true, |v| !v.split(',').any(|x| x == "MEX_ON_MODEL_RENDER")) { melee_fighter::tu::ftdrawcommon::ext::MEX_ON_MODEL_RENDER = Some(|kind, g, a, m| { if let Some(f) = d::ftData_UnkMtxFunc0[kind as usize] { f(g as _, a, m as _) } }); }
            let _ = k;
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

/// `OSGetTime`: the console time base (`__OSBusClock / 4` = 40.5 MHz ticks) off the host's monotonic clock.
pub fn os_get_time() -> i64 {
    use std::sync::OnceLock;
    static T0: OnceLock<std::time::Instant> = OnceLock::new();
    (T0.get_or_init(std::time::Instant::now).elapsed().as_nanos() as i64) * 81 / 2000
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
            (*request).owner = owner;
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
        if aram.len() < ARAM_SIZE as usize {
            aram.resize(ARAM_SIZE as usize, 0);
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
    if let Some(v) = ext_hand(name, args) {
        return Some(v);
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
            melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6B98(args[0] as usize as *mut melee_types::HSD_Text, f32::from_bits(args[1] as u32), f32::from_bits(args[2] as u32), Some(&s)) as i64 as u64
        },
        "OSReport" => 0,
        // gw_runtime.c gw_GxTex_OpenEnv: MELEE_MENUTEX names a replacement texture; unset (no mods) it is -1 and the
        // CSS keeps the disc's art. The handle queries answer like the C does for a bad handle.
        "GxTex_OpenEnv" | "GxTex_OpenUI" => -1i64 as u64,
        "GxTex_Width" | "GxTex_Height" | "GxTex_ImageSize" | "GxTex_TlutSize" | "GxTex_TlutEntries" => 0,
        "GxTex_Format" | "GxTex_TlutFormat" => -1i64 as u64,
        "GxTex_Close" | "GxTex_CopyImage" | "GxTex_CopyTlut" => 0,
        "HSD_SisLib_803A70A0" => unsafe {
            let s = c_format(args[2], &args[3..]);
            melee_runtime::tu::hsd_3A64::HSD_SisLib_803A70A0(args[0] as usize as *mut melee_types::HSD_Text, args[1] as i32, Some(&s)) as i64 as u64
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
pub unsafe fn c_format(fmt: u64, va: &[u64]) -> String {
    let cstr = |p: u64| if p == 0 { String::new() } else { unsafe { std::ffi::CStr::from_ptr(p as usize as *const core::ffi::c_char) }.to_string_lossy().into_owned() };
    let f = cstr(fmt);
    let mut out = String::new();
    let mut it = f.chars().peekable();
    let mut k = 0;
    while let Some(c) = it.next() {
        if c != '%' { out.push(c); continue; }
        let mut spec = String::new();
        while let Some(&d) = it.peek() { if d.is_ascii_digit() || d == '-' || d == '.' || d == 'l' { spec.push(d); it.next(); } else { break; } }
        let width: usize = spec.trim_start_matches('-').trim_start_matches('0').split('.').next().unwrap_or("").trim_end_matches('l').parse().unwrap_or(0);
        let zero = spec.starts_with('0');
        let w = va.get(k).copied().unwrap_or(0);
        let body = match it.next() {
            Some('%') => { out.push('%'); continue; }
            Some('s') => { k += 1; cstr(w) }
            Some('d') | Some('i') => { k += 1; (w as i32).to_string() }
            Some('u') => { k += 1; (w as u32).to_string() }
            Some('x') => { k += 1; format!("{:x}", w as u32) }
            Some('X') => { k += 1; format!("{:X}", w as u32) }
            Some('c') => { k += 1; ((w as u8) as char).to_string() }
            Some(o) => { k += 1; o.to_string() }
            None => break,
        };
        if body.len() < width { out.push_str(&(if zero { "0" } else { " " }).repeat(width - body.len())); }
        out.push_str(&body);
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
    #[cfg(not(windows))]
    {
        extern "C" { fn localtime_r(t: *const i64, tm: *mut Tm) -> *mut Tm; }
        unsafe { localtime_r(&secs, &mut tm) };
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

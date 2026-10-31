//! Host seams the scene loop and the libraries `main` brings up call out to: the dolphin SDK hardware (pad
//! hardware, video, GX), the TARGET_PC port extensions (rollback, scripts, replay, determinism, overlay) and the
//! debug build helpers. A headless engine has none of them: each is the "feature off" answer. Anything here that
//! turns out to carry game logic belongs in its TU instead.

use melee_types::{OSAlarm, OSContext};

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

pub fn install() {
    unsafe {
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
    Some(match name {
        "__frsqrte" => melee_runtime::tu::gekko_fp::__frsqrte(f64::from_bits(args[0])).to_bits(),
        "__fres" => melee_runtime::tu::gekko_fp::__fres(f64::from_bits(args[0])).to_bits(),
        _ => return None,
    })
}

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

        melee_runtime::tu::memory::ext::hook::PcTraceHeapEnabled = Some(|| 0);
        melee_runtime::tu::memory::ext::hook::Snap_NoteMem = Some(|_, _, _, _| {});

        // robj.c has no native port yet (tu/robj.rs keeps the owned model): only the NULL descriptor the DevText
        // camera and wobj loads pass is answered; a real rig stops here until robj.c is ported (step 7).
        fn robj_null(d: *mut melee_types::HSD_RObjDesc) -> *mut melee_types::HSD_RObj {
            assert!(d.is_null(), "HSD_RObjLoadDesc: robj.c is not ported natively (consolidation step 7)");
            core::ptr::null_mut()
        }
        macro_rules! robj_walks {
            ($m:ident) => {{
                use melee_runtime::tu::$m::ext::hook as h;
                h::HSD_RObjLoadDesc = Some(robj_null);
                h::HSD_RObjRemoveAll = Some(|r| assert!(r.is_null(), "robj.c is not ported natively"));
                h::HSD_RObjAddAnimAll = Some(|r, _| assert!(r.is_null(), "robj.c is not ported natively"));
                h::HSD_RObjAnimAll = Some(|r| assert!(r.is_null(), "robj.c is not ported natively"));
                h::HSD_RObjResolveRefsAll = Some(|r, _| assert!(r.is_null(), "robj.c is not ported natively"));
            }};
        }
        robj_walks!(wobj);
        robj_walks!(jobj);
        melee_runtime::tu::wobj::ext::hook::HSD_RObjRemoveAnimAll = Some(|r| assert!(r.is_null()));
        melee_runtime::tu::wobj::ext::hook::HSD_RObjReqAnimAll = Some(|r, _| assert!(r.is_null()));
        melee_runtime::tu::jobj::ext::hook::HSD_RObjRemoveAnimAllByFlags = Some(|r, _| assert!(r.is_null()));
        melee_runtime::tu::jobj::ext::hook::HSD_RObjReqAnimAllByFlags = Some(|r, _, _| assert!(r.is_null()));
        melee_runtime::tu::jobj::ext::hook::HSD_RObjUpdateAll = Some(|r, _, _| assert!(r.is_null()));
        melee_runtime::tu::jobj::ext::hook::HSD_RObjGetByType = Some(|r, _, _| { assert!(r.is_null()); r });
        melee_runtime::tu::jobj::ext::hook::HSD_RObjGetGlobalPosition = Some(|r, _, _| { assert!(r.is_null()); 0 });

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

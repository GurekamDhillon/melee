//! The engine (consolidation plan 1.1, step 6): boots the game's own libraries on the host and runs the scene
//! loop one `gm_801A4D34` iteration at a time. No game logic here: `main`'s order of calls (gmmain.c, TARGET_PC),
//! `gm_801A4BD4` / `gm_801A4D34_begin` / `gm_801A4D34_tick` from `melee_modes::tu::gmscene`, and the pad feed.

use crate::gcm::Gcm;
use crate::host::Host;
use core::ffi::c_void;
use melee_modes::tu::{gmmain, gmmain_lib, gmscene};
use melee_platform::PadSystem;
use melee_runtime::tu::{controller, initialize};
use melee_types::{GameSceneInfo, PADStatus};

/// The pad source `PADRead` reads (a hook is a plain `fn`, so the system is a global like the C's SI registers).
static mut PADS: Option<PadSystem> = None;

fn pad_read(st: *mut PADStatus) -> u32 {
    unsafe {
        let arr = &mut *(st as *mut [PADStatus; 4]);
        match PADS.as_mut() {
            Some(p) => p.read(arr),
            None => 0,
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
        Host::boot(disc, mem_size)?;
        melee_link::install();
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
            crate::seams_gen::install_gen();
            gmmain_lib::gmMainLib_init_statics();
            initialize::HSD_OSInit();
            // lb_80019AAC(&gmMain_8015FD24): pad library setup and the sampling period
            melee_modes::tu::lb_0195::lb_80019AAC(Some(gmmain_pad_setup));
            gmscene::gm_801A4BD4();
            gmscene::gm_801A4D34_begin(core::ptr::null_mut());
        }
        Ok(Engine { frames: 0, info: core::ptr::null_mut(), on_frame: None, ended: false })
    }

    /// Exactly one iteration of the scene loop (pad queue, logic frame(s), `HSD_GObj_RunProcs`, render).
    /// Returns false once the scene has ended.
    pub fn tick(&mut self) -> bool {
        if self.ended {
            return false;
        }
        crate::seams::fire_alarm(); // the periodic pad-sampling alarm (lb_80019628) fires once per frame
        let more = unsafe { gmscene::gm_801A4D34_tick(self.on_frame, self.info) };
        self.frames += 1;
        if !more {
            self.ended = true;
        }
        more
    }
}

unsafe extern "C" fn gmmain_pad_setup() {
    gmmain::gmMain_8015FD24();
}

#[allow(dead_code)]
fn _unused(_: *mut c_void) {}

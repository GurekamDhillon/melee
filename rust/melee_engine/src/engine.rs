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

fn pad_read(st: *mut PADStatus) -> u32 {
    unsafe {
        let arr = &mut *(st as *mut [PADStatus; 4]);
        match PADS.as_mut() {
            Some(p) => p.read(arr),
            None => 0,
        }
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
        Self::boot_host(disc, mem_size, pads)?;
        crate::scenario::install(scn);
        unsafe {
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
            melee_runtime::tu::gobj_native::HEADLESS_RENDER = true;
            melee_runtime::tu::particle::CMD_STREAM_BIG_ENDIAN = true;
            
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
        unsafe {
            melee_fighter::tu::fighter::RB_NeutralHash();
            let mut r = [0u32; 16];
            for (i, w) in r.iter_mut().enumerate() { *w = melee_fighter::tu::fighter::RB_NeutralHashWord(i as i32); }
            r
        }
    }

    /// Exactly one iteration of the scene loop (pad queue, logic frame(s), `HSD_GObj_RunProcs`, render).
    /// Returns false once the scene has ended.
    pub fn tick(&mut self) -> bool {
        if self.ended {
            return false;
        }
        unsafe { RAND_FRAME = self.frames as i32; }
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

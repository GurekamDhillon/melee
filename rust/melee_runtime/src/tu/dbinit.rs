//! Port of src/melee/db/dbinit.c (debug-ROM setup, the per-player debug button latches and the
//! every-frame debug-menu dispatcher).
//!
//! The C globals live in [`DbInit`]; calls out of the TU go through [`DbInitHost`], whose default
//! implementation [`ext::Ext`] is the unported-callee stub set (panics, named like the C).
//! Calls into the other db TUs (dbsound, dbitem, ...) also go through the host so each TU's state
//! stays in its own struct.
#![allow(non_snake_case)]

use core::ffi::c_void;
use melee_types::{HSD_GObj, PADStatus};

use super::dbeffect::pad;

/// `DbLKind` (db.h).
pub mod dblkind {
    pub const MASTER: i32 = 0;
    pub const NO_DEBUG_ROM: i32 = 1;
    pub const DEBUG_DEVELOP: i32 = 2;
    pub const DEBUG_ROM: i32 = 3;
    pub const DEVELOP: i32 = 4;
}

/// `db_build_timestamp` (0x803EA6C8).
pub const DB_BUILD_TIMESTAMP: &str = "DATE Feb 13 2002  TIME 22:06:27";

/// One entry of the C's `db_ButtonStates[4]` (GC +0 current, +4 prev, +8 pressed, +C released,
/// +10 repeat).
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbButtonState {
    pub current: u32,
    pub prev: u32,
    pub pressed: u32,
    pub released: u32,
    pub repeat: u32,
}

/// The pad fields of `HSD_PadMasterStatus[i]` that db_RunEveryFrame reads.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct DbPadIn {
    pub button: u32,
    pub repeat: u32,
    pub nml_sub_stick_x: f32,
    pub nml_sub_stick_y: f32,
}

/// Callees of dbinit.c.
pub trait DbInitHost {
    fn VIWaitForRetrace(&mut self);
    fn PADRead(&mut self, status: &mut [PADStatus; 4]);
    /// Returns the C's result; the out-params are unused by the caller.
    fn CARDProbeEx(&mut self, chan: i32) -> i32;
    /// `lbArchive_LoadSymbols(file, &out, sym, 0)`: returns `out`.
    fn lbArchive_LoadSymbols(&mut self, file: &str, sym: &str) -> *mut c_void;
    fn OSReport(&mut self, s: &str);
    fn HSD_GObjPLinkHead(&mut self, i: usize) -> *mut HSD_GObj;
    fn HSD_PadMasterStatus(&mut self, i: usize) -> DbPadIn;
    fn ftLib_IsMasterHandPresent(&mut self) -> bool;
    fn ftLib_IsCrazyHandPresent(&mut self) -> bool;
    /// `_stack_addr` / `_stack_end` linker symbols, and the byte at an address.
    fn stack_addr(&mut self) -> usize;
    fn stack_end(&mut self) -> usize;
    fn read_u8(&mut self, addr: usize) -> u8;
    // other db TUs
    fn fn_SetupCpuHandicapInfo(&mut self);
    fn fn_SetupAnimationInfo(&mut self);
    fn fn_SetupItemAndPokemonMenu(&mut self);
    fn fn_SetupSoundInfo(&mut self);
    fn fn_SetupMiscStageVisuals(&mut self);
    fn fn_SetupCameraInfo(&mut self);
    fn fn_SetupBonusInfo(&mut self);
    fn fn_SetupObjAllocLimiter(&mut self);
    fn fn_Setup5xSpeed(&mut self);
    fn fn_CheckMiscVisualEffects(&mut self, player: i32);
    fn fn_CheckCameraInfo(&mut self, player: i32, down: i32, pressed: i32, cx: f32, cy: f32);
    fn fn_CheckSoundInfo(&mut self, player: i32);
    fn fn_UpdateObjAllocLimiter(&mut self, player: i32);
    fn fn_CheckItemAndPokemonMenu(&mut self, player: i32);
    fn fn_CheckCpuHandicapInfo(&mut self, player: i32);
    fn fn_UpdateCpuHandicapInfo(&mut self);
    fn fn_CheckAnimationInfo(&mut self, player: i32);
    fn fn_UpdateAnimationInfo(&mut self);
    fn fn_CheckMiscStageEffects(&mut self, player: i32);
    fn fn_CheckBonusInfo(&mut self, player: i32);
    fn fn_Check5xSpeed(&mut self, player: i32);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl DbInitHost for Ext {
        fn VIWaitForRetrace(&mut self) { todo!("VIWaitForRetrace (pc/platform)") }
        fn PADRead(&mut self, _: &mut [PADStatus; 4]) { todo!("PADRead (pc/platform)") }
        fn CARDProbeEx(&mut self, _: i32) -> i32 { todo!("CARDProbeEx (pc/platform)") }
        fn lbArchive_LoadSymbols(&mut self, _: &str, _: &str) -> *mut c_void { todo!("lbArchive_LoadSymbols (lb/lbarchive.c)") }
        fn OSReport(&mut self, _: &str) { todo!("OSReport (pc/platform)") }
        fn HSD_GObjPLinkHead(&mut self, _: usize) -> *mut HSD_GObj { todo!("HSD_GObjPLinkHead (baselib/gobj.c)") }
        fn HSD_PadMasterStatus(&mut self, _: usize) -> DbPadIn { todo!("HSD_PadMasterStatus (baselib/controller.c)") }
        fn ftLib_IsMasterHandPresent(&mut self) -> bool { todo!("ftLib_IsMasterHandPresent (ft/ftlib.c)") }
        fn ftLib_IsCrazyHandPresent(&mut self) -> bool { todo!("ftLib_IsCrazyHandPresent (ft/ftlib.c)") }
        fn stack_addr(&mut self) -> usize { todo!("_stack_addr (linker)") }
        fn stack_end(&mut self) -> usize { todo!("_stack_end (linker)") }
        fn read_u8(&mut self, _: usize) -> u8 { todo!("stack byte read") }
        fn fn_SetupCpuHandicapInfo(&mut self) { todo!("fn_SetupCpuHandicapInfo (db/dbcpu)") }
        fn fn_SetupAnimationInfo(&mut self) { todo!("fn_SetupAnimationInfo (db)") }
        fn fn_SetupItemAndPokemonMenu(&mut self) { todo!("fn_SetupItemAndPokemonMenu (db/dbitem.c)") }
        fn fn_SetupSoundInfo(&mut self) { todo!("fn_SetupSoundInfo (db/dbsound.c)") }
        fn fn_SetupMiscStageVisuals(&mut self) { todo!("fn_SetupMiscStageVisuals (db)") }
        fn fn_SetupCameraInfo(&mut self) { todo!("fn_SetupCameraInfo (db)") }
        fn fn_SetupBonusInfo(&mut self) { todo!("fn_SetupBonusInfo (db)") }
        fn fn_SetupObjAllocLimiter(&mut self) { todo!("fn_SetupObjAllocLimiter (db)") }
        fn fn_Setup5xSpeed(&mut self) { todo!("fn_Setup5xSpeed (db/dbscreenshot.c)") }
        fn fn_CheckMiscVisualEffects(&mut self, _: i32) { todo!("fn_CheckMiscVisualEffects (db/dbeffect.c)") }
        fn fn_CheckCameraInfo(&mut self, _: i32, _: i32, _: i32, _: f32, _: f32) { todo!("fn_CheckCameraInfo (db)") }
        fn fn_CheckSoundInfo(&mut self, _: i32) { todo!("fn_CheckSoundInfo (db/dbsound.c)") }
        fn fn_UpdateObjAllocLimiter(&mut self, _: i32) { todo!("fn_UpdateObjAllocLimiter (db)") }
        fn fn_CheckItemAndPokemonMenu(&mut self, _: i32) { todo!("fn_CheckItemAndPokemonMenu (db/dbitem.c)") }
        fn fn_CheckCpuHandicapInfo(&mut self, _: i32) { todo!("fn_CheckCpuHandicapInfo (db)") }
        fn fn_UpdateCpuHandicapInfo(&mut self) { todo!("fn_UpdateCpuHandicapInfo (db)") }
        fn fn_CheckAnimationInfo(&mut self, _: i32) { todo!("fn_CheckAnimationInfo (db)") }
        fn fn_UpdateAnimationInfo(&mut self) { todo!("fn_UpdateAnimationInfo (db)") }
        fn fn_CheckMiscStageEffects(&mut self, _: i32) { todo!("fn_CheckMiscStageEffects (db)") }
        fn fn_CheckBonusInfo(&mut self, _: i32) { todo!("fn_CheckBonusInfo (db)") }
        fn fn_Check5xSpeed(&mut self, _: i32) { todo!("fn_Check5xSpeed (db/dbscreenshot.c)") }
    }
}

/// The globals of dbinit.c.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbInit {
    /// `db_gameLaunchButtonState` (0x804D6B30).
    pub game_launch_button_state: u16,
    /// `db_bonus_names` (0x804D6B2C).
    pub bonus_names: *mut *mut i8,
    /// `db_motionstate_names` (0x804D6B28).
    pub motionstate_names: *mut *mut i8,
    /// `db_submotion_names` (0x804D6B24).
    pub submotion_names: *mut *mut i8,
    /// `db_804D6B20`.
    pub db_804D6B20: bool,
    /// `DbLevel` (0x804D4A08).
    pub level: i32,
    /// `db_ButtonStates`.
    pub button_states: [DbButtonState; 4],
}

impl Default for DbInit {
    fn default() -> Self {
        Self {
            game_launch_button_state: 0,
            bonus_names: core::ptr::null_mut(),
            motionstate_names: core::ptr::null_mut(),
            submotion_names: core::ptr::null_mut(),
            db_804D6B20: false,
            level: dblkind::NO_DEBUG_ROM,
            button_states: [DbButtonState::default(); 4],
        }
    }
}

/// The d-pad diagonal filter db_RunEveryFrame applies to both `button` and `repeat`: a
/// diagonal drops both of its directions.
pub fn db_filter_dpad(mut b: i32) -> i32 {
    melee_core::ct!("db_filter_dpad");
    let (u, d, l, r) = (pad::DPADUP as i32, pad::DPADDOWN as i32, pad::DPADLEFT as i32, pad::DPADRIGHT as i32);
    if b & u != 0 {
        if b & l != 0 {
            b &= !(u | l);
        }
        if b & r != 0 {
            b &= !(u | r);
        }
    }
    if b & d != 0 {
        if b & l != 0 {
            b &= !(d | l);
        }
        if b & r != 0 {
            b &= !(d | r);
        }
    }
    b
}

impl DbInit {
    /// `db_GetGameLaunchButtonState` (0x80225374).
    pub fn db_GetGameLaunchButtonState<H: DbInitHost + ?Sized>(&mut self, h: &mut H) {
        let mut status: [PADStatus; 4] = unsafe { core::mem::zeroed() };
        loop {
            h.VIWaitForRetrace();
            h.PADRead(&mut status);
            let mut done = true;
            for s in &status {
                if s.err == -2 || s.err == -3 {
                    done = false;
                }
            }
            if done {
                break;
            }
        }
        let mut pad = 0;
        while pad < 4 {
            if status[pad].err == 0 {
                break;
            }
            pad += 1;
        }
        self.game_launch_button_state = if pad != 4 { status[pad].button } else { 0 };
        while h.CARDProbeEx(0) == -1 {
            h.VIWaitForRetrace();
        }
    }

    /// `db_Setup` (0x802254B8).
    pub fn db_Setup<H: DbInitHost + ?Sized>(&mut self, h: &mut H) {
        if self.level >= dblkind::DEBUG_ROM {
            for s in &mut self.button_states {
                s.repeat = 0;
                s.released = 0;
                s.pressed = 0;
                s.current = 0;
            }
            let common = h.lbArchive_LoadSymbols("DbCo.dat", "dbLoadCommonData") as *const *mut *mut i8;
            // SAFETY: the archive symbol is `{ char** bonus, motionstate, submotion }`.
            unsafe {
                self.bonus_names = *common;
                self.motionstate_names = *common.add(1);
                self.submotion_names = *common.add(2);
            }
            h.fn_SetupCpuHandicapInfo();
            h.fn_SetupAnimationInfo();
            h.fn_SetupItemAndPokemonMenu();
            h.fn_SetupSoundInfo();
            h.fn_SetupMiscStageVisuals();
            h.fn_SetupCameraInfo();
            h.fn_SetupBonusInfo();
            h.fn_SetupObjAllocLimiter();
            h.fn_Setup5xSpeed();
        }
    }

    /// `db_ButtonsDown` (0x8022558C).
    pub fn db_ButtonsDown(&self, player: i32) -> u32 {
        self.button_states[player as usize].current
    }

    /// `db_ButtonsPressed` (0x802255A4).
    pub fn db_ButtonsPressed(&self, player: i32) -> u32 {
        self.button_states[player as usize].pressed
    }

    /// `db_ButtonsRepeat` (0x802255BC).
    pub fn db_ButtonsRepeat(&self, player: i32) -> u32 {
        self.button_states[player as usize].repeat
    }

    /// `db_PrintEntityCounts` (0x802255D4).
    pub fn db_PrintEntityCounts<H: DbInitHost + ?Sized>(&self, h: &mut H) {
        h.OSReport("[all PLink num] -- Report --\n");
        for i in 0..10 {
            h.OSReport(&format!("{:5} ", i));
        }
        h.OSReport("\n");
        h.OSReport("------------------------------------------------------------\n");
        for i in 0..64usize {
            let mut count = 0i32;
            let mut g = h.HSD_GObjPLinkHead(i & 0xFF);
            while !g.is_null() {
                // SAFETY: the plink list is well formed (null-terminated `next` chain).
                g = unsafe { (*g).next };
                count += 1;
            }
            h.OSReport(&format!("{:5} ", count));
            if i % 10 == 9 {
                h.OSReport("\n");
            }
        }
        h.OSReport("\n");
    }

    /// `db_PrintThreadInfo` (0x802256CC).
    pub fn db_PrintThreadInfo<H: DbInitHost + ?Sized>(&self, h: &mut H) {
        let end = h.stack_end();
        let addr = h.stack_addr();
        let mut peak = end.wrapping_add(4);
        while h.read_u8(peak) == 0xAA {
            peak = peak.wrapping_add(1);
        }
        h.OSReport("------ Thread info ------\n");
        h.OSReport(&format!(
            "base:{:x}, end:{:x}, size:{} peak:{} \n",
            addr as u32,
            end as u32,
            addr.wrapping_sub(end) as i32,
            addr.wrapping_sub(peak) as i32
        ));
        h.OSReport("\n");
    }

    /// `db_RunEveryFrame` (0x80225754).
    pub fn db_RunEveryFrame<H: DbInitHost + ?Sized>(&mut self, h: &mut H) {
        if self.level < dblkind::DEBUG_ROM {
            return;
        }
        let num_players = if h.ftLib_IsMasterHandPresent() || h.ftLib_IsCrazyHandPresent() { 2 } else { 4 };
        let mut pads = [DbPadIn::default(); 4];
        for i in 0..num_players {
            let p = h.HSD_PadMasterStatus(i & 0xFF);
            pads[i] = p;
            let button = db_filter_dpad(p.button as i32) as u32;
            let repeat = db_filter_dpad(p.repeat as i32) as u32;
            let s = &mut self.button_states[i];
            s.prev = s.current;
            s.current = button;
            s.pressed = s.current & (s.prev ^ s.current);
            s.released = s.prev & (s.prev ^ s.current);
            s.repeat = repeat;
        }
        for i in 0..4 {
            h.fn_CheckMiscVisualEffects(i);
        }
        for i in 0..4i32 {
            let p = if (i as usize) < num_players { pads[i as usize] } else { h.HSD_PadMasterStatus(i as usize & 0xFF) };
            let s = self.button_states[i as usize];
            h.fn_CheckCameraInfo(i, s.current as i32, s.pressed as i32, p.nml_sub_stick_x, p.nml_sub_stick_y);
        }
        for i in 0..4 {
            h.fn_CheckSoundInfo(i);
        }
        for i in 0..4 {
            h.fn_UpdateObjAllocLimiter(i);
        }
        for i in 0..4 {
            h.fn_CheckItemAndPokemonMenu(i);
        }
        for i in 0..4 {
            h.fn_CheckCpuHandicapInfo(i);
        }
        h.fn_UpdateCpuHandicapInfo();
        for i in 0..4 {
            h.fn_CheckAnimationInfo(i);
        }
        h.fn_UpdateAnimationInfo();
        for i in 0..4 {
            h.fn_CheckMiscStageEffects(i);
        }
        for i in 0..4 {
            h.fn_CheckBonusInfo(i);
        }
        for i in 0..4 {
            h.fn_Check5xSpeed(i);
        }
    }
}

//! Shared smoke-test scaffolding for the Mr. Game & Watch / Ganondorf / Giga Bowser TU tests:
//! a zeroed fighter + gobj + part table + attribute block, and a call log that the recording
//! hooks append to. Hooks are process globals, so each test file has one #[test].
#![allow(static_mut_refs, dead_code)]

use melee_chara::stubs as st;
use melee_chara::tu::ftgamewatch::ext as gw;
use melee_types::*;

pub struct World {
    pub f: Box<Fighter>,
    pub g: Box<HSD_GObj>,
    pub parts: std::vec::Vec<FighterBone>,
    pub da: &'static mut ftGameWatchAttributes,
}

impl World {
    pub fn new() -> World {
        let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
        let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
        let da: &'static mut ftGameWatchAttributes = Box::leak(Box::new(ftGameWatchAttributes::zeroed()));
        g.user_data = &mut *f as *mut Fighter as *mut _;
        f.dat_attrs = da as *mut _ as *mut _;
        f.parts = parts.as_mut_ptr();
        World { f, g, parts, da }
    }
    pub fn gp(&mut self) -> *mut HSD_GObj {
        &mut *self.g
    }
}

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
pub fn log(s: impl Into<String>) {
    unsafe { LOG.push(s.into()) }
}
/// Drain the log as one comma-joined string.
pub fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

/// A fake item GObj pointer (never dereferenced: the item hooks only log it).
pub fn fake(n: usize) -> *mut HSD_GObj {
    n as *mut HSD_GObj
}

pub static mut FRAMES_REMAINING: i32 = 1;

/// Recording stand-ins for the group-3 calls and the shared `ext` hooks.
pub fn install_common() {
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, start, _, _, _| log(format!("ms {ms} {fl:x} {start}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim"));
        st::FT_80084104 = Some(|_| log("coll"));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_REMAINING);
        st::FT_8008A2BC = Some(|_| log("wait"));
        st::FTCO_FALL_ENTER = Some(|_| log("fall"));
        st::FT_80081D0C = Some(|_| 1);
        st::FT_80082708 = Some(|_| 0);
        st::FT_800827A0 = Some(|_| 0);
        st::FTCOMMON_8007D7FC = Some(|_| log("7D7FC"));
        gw::LB_8000B1CC = Some(|_, _, out| unsafe {
            (*out).x = 1.0;
            (*out).y = 2.0;
            (*out).z = 3.0;
        });
    }
}

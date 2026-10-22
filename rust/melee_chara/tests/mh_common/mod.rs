//! Shared rig for the ftMasterHand TU smoke tests: a fighter + gobj with a zeroed ftData and
//! ftMasterHand_SpecialAttrs, and recording hooks for the group-3 stubs.
#![allow(static_mut_refs, dead_code)]

use melee_chara::stubs as st;
use melee_types::*;

pub static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
pub fn log(s: impl Into<String>) {
    unsafe { LOG.push(s.into()) }
}
pub fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}
pub static mut FRAMES: i32 = 1;
pub static mut SLOT: i32 = 0;

pub struct Rig {
    pub f: Box<Fighter>,
    pub g: Box<HSD_GObj>,
    pub fd: Box<ftData>,
    pub da: Box<ftMasterHand_SpecialAttrs>,
}

impl Rig {
    pub fn new() -> Rig {
        unsafe {
            let mut r = Rig {
                f: Box::new(Fighter::zeroed()),
                g: Box::new(core::mem::zeroed()),
                fd: Box::new(core::mem::zeroed()),
                da: Box::new(core::mem::zeroed()),
            };
            r.fd.ext_attr = &mut *r.da as *mut _ as *mut _;
            r.f.ft_data = &mut *r.fd;
            r.g.user_data = &mut *r.f as *mut Fighter as *mut _;
            st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, st_, sp, bl, _| log(format!("cms {ms:#x} {fl} {st_} {sp} {bl}")));
            st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4"));
            st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
            st::FT_80085134 = Some(|_| log("85134"));
            st::FIGHTER_UNKSETFLAG_8006CFBC = Some(|_| log("setflag"));
            st::PLAYER_GETPLAYERSLOTTYPE = Some(|_| SLOT);
            r
        }
    }
    pub fn gp(&mut self) -> *mut HSD_GObj {
        &mut *self.g
    }
}

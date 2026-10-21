//! Shared rig for the ftCrazyHand TU smoke tests.
#![allow(dead_code, static_mut_refs)]
use melee_types::*;

pub static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
pub fn log(s: String) {
    unsafe { LOG.push(s) }
}
pub fn take_log() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

pub struct Rig {
    pub f: Box<Fighter>,
    pub g: Box<HSD_GObj>,
    pub fd: Box<ftData>,
    pub ext: Box<ftCrazyHand_DatAttrs>,
    pub backup: Box<ftCrazyHand_DatAttrs>,
    pub live: Box<ftCrazyHand_DatAttrs>,
}

impl Rig {
    pub fn new() -> Rig {
        let mut r = Rig {
            f: Box::new(Fighter::zeroed()),
            g: Box::new(unsafe { core::mem::zeroed() }),
            fd: Box::new(unsafe { core::mem::zeroed() }),
            ext: Box::new(unsafe { core::mem::zeroed() }),
            backup: Box::new(unsafe { core::mem::zeroed() }),
            live: Box::new(unsafe { core::mem::zeroed() }),
        };
        r.fd.ext_attr = &mut *r.ext as *mut _ as *mut _;
        r.f.ft_data = &mut *r.fd;
        r.f.dat_attrs_backup = &mut *r.backup as *mut _ as *mut _;
        r.f.dat_attrs = &mut *r.live as *mut _ as *mut _;
        r.g.user_data = &mut *r.f as *mut Fighter as *mut _;
        r
    }
    pub fn gp(&mut self) -> *mut HSD_GObj {
        &mut *self.g
    }
}

/// Install recording hooks for the group-3 calls the TUs make through `melee_chara::stubs`.
pub fn hook_stubs(frames_remaining: i32) {
    use melee_chara::stubs as st;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, s, sp, b, a3| {
            log(format!("cms {msid:#x} {flags} {s} {sp} {b} {}", a3.is_null()))
        });
        st::FTANIM_8006EBA4 = Some(|_| log("anim8006EBA4".into()));
        st::FT_80085134 = Some(|_| log("ft85134".into()));
        FRAMES = frames_remaining;
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
    }
}
pub static mut FRAMES: i32 = 0;

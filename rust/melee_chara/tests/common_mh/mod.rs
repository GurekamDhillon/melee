//! Shared rig for the ftMasterHand TU tests.
#![allow(dead_code, static_mut_refs)]
use melee_types::*;

pub static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
pub static mut FRAMES: i32 = 0;
pub fn log(s: String) {
    unsafe { LOG.push(s) }
}
pub fn take_log() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}
pub fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
pub fn fl(s: &str) -> f32 {
    f32::from_bits(hex(s))
}

pub struct Rig {
    pub f: Box<Fighter>,
    pub g: Box<HSD_GObj>,
    pub fd: Box<ftData>,
    pub da: Box<ftMasterHand_SpecialAttrs>,
}

impl Rig {
    pub fn new() -> Rig {
        let mut r = Rig {
            f: Box::new(Fighter::zeroed()),
            g: Box::new(unsafe { core::mem::zeroed() }),
            fd: Box::new(unsafe { core::mem::zeroed() }),
            da: Box::new(ftMasterHand_SpecialAttrs::zeroed()),
        };
        r.fd.ext_attr = &mut *r.da as *mut _ as *mut _;
        r.f.ft_data = &mut *r.fd;
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
            log(format!("cms {msid} {flags} {s} {sp} {b} {}", a3.is_null()))
        });
        st::FTANIM_8006EBA4 = Some(|_| log("anim8006EBA4".into()));
        st::FT_80085134 = Some(|_| log("ft85134".into()));
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::PLAYER_GETPLAYERSLOTTYPE = Some(|slot| slot);
        FRAMES = frames_remaining;
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
    }
}

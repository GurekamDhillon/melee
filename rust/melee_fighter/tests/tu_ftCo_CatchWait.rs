//! src/tu/ftCo_CatchWait.rs vs the C (gen/cref/tu/ftCo_CatchWait.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CatchWait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CatchWait as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JB: [u8; 64] = [0; 64];

fn jb_off(j: *mut HSD_JObj) -> u32 {
    if j.is_null() { 0xFF } else { (j as usize - unsafe { JB.as_ptr() } as usize) as u32 }
}

unsafe extern "C" fn dummy(_g: *mut HSD_GObj) {}

fn is_cb(cb: HSD_GObjEvent, want: unsafe extern "C" fn(*mut HSD_GObj)) -> u32 {
    (cb.map(|p| p as usize) == Some(want as usize)) as u32
}

#[test]
fn ftCo_CatchWait_matches_c() {
    init();
    let parts: &'static mut [FighterBone] = Box::leak((0..8).map(|_| FighterBone::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice());
    for (i, p) in parts.iter_mut().enumerate() {
        p.joint = unsafe { JB.as_mut_ptr().add(i * 4) } as *mut HSD_JObj;
    }
    let fd = Box::leak(Box::new(ftData::zeroed()));
    let x8 = Box::leak(Box::new(ftData_x8::zeroed()));
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::efAsync_Spawn = Some(|g, q, kind, gfx, j| {
            t(2); t(is_go(g)); t((q == &mut f().x60C as *mut _ as *mut core::ffi::c_void) as u32); t(kind); t(gfx); t(jb_off(j))
        });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(3); t((fr as *mut Fighter == F) as u32); t(a); t(bb) });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(4); t((fr as *mut Fighter == F) as u32); t(v as i32 as u32) });
        hk::fn_800DB6C8 = Some(|g| { t(5); t((g == go2()) as u32) });
        hk::fn_800DA054 = Some(|g| { t(6); t(is_go(g)) });
        hk::fn_800DA4FC = Some(|g| { t(7); t(is_go(g)) });
        hk::ftCo_800DD1E4 = Some(|g| { t(8); t(is_go(g)); sr(1) });
        hk::it_802A7AAC = Some(|g| { t(9); t(is_go(g)) });
        hk::it_802BAC3C = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(11); t((fr as *mut Fighter == F) as u32); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(12); t(is_go(g)) });
        hk::ft_800841B8 = Some(|g, cb| { t(13); t(is_go(g)); t(is_cb(cb, m::fn_800DA440_cb)); unsafe { (cb.unwrap())(g) } });
        hk::ftCo_800DC920 = Some(|g, v| { t(14); t(is_go(g)); t((v == go2()) as u32) });
        hk::ftCo_Fall_Enter = Some(|g| { t(15); t(is_go(g)); t((g == go2()) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CatchWait.txt"));
    for op in 0..10u32 {
        for k in 0..2000u32 {
            case(op, k);
            *x8 = ftData_x8::zeroed();
            x8.x11 = (rnd() % 8) as u8;
            fd.x8 = x8 as *mut _;
            let fr = f();
            fr.ft_data = fd as *mut _;
            fr.parts = parts.as_mut_ptr();
            fr.victim_gobj = go2();
            fr.gr_vel = rg();
            fr.kind = rnd() % 34;
            fr.input.pressed_buttons = rnd() & 0x1FF;
            cd().x64 = rg();
            fr.co_attrs.ground_friction = rg();
            fr.take_dmg_cb = if rnd() & 1 != 0 { Some(dummy) } else { None };
            fr.x221B.set_x221B_b7((rnd() & 1) as u8);
            let g = go();
            let ret = match op {
                0 => { m::fn_800DA1D8(g); 0 }
                1 => { m::fn_800DA2B0(g); 0 }
                2 => { m::ftCo_CatchWait_Anim(g); 0 }
                3 => { m::ftCo_CatchWait_IASA(g); 0 }
                4 => { m::ftCo_CatchWait_Phys(g); 0 }
                5 => { m::ftCo_CatchWait_Coll(g); 0 }
                6 => { m::fn_800DA440(g); 0 }
                7 => { m::fn_800DA490(g); 0 }
                8 => { m::fn_800DA4A0(g); 0 }
                _ => m::fn_800DA4C0(g) as u32,
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.gr_vel));
            h.m(is_cb(fr.accessory1_cb, m::fn_800DA4A0_cb));
            h.m(is_cb(fr.take_dmg_cb, m::fn_800DA490_cb));
            h.m(fr.take_dmg_cb.is_none() as u32);
            h.m(fr.x221B.x221B_b7() as u32);
            h.m(jb_off(unsafe { fr.mv.co.capturedamage.x18 }));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 20000);
}

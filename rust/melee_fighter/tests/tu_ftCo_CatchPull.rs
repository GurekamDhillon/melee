//! src/tu/ftCo_CatchPull.rs vs the C (gen/cref/tu/ftCo_CatchPull.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CatchPull`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CatchPull as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JB: [u8; 64] = [0; 64];
static mut POS: [Vec3; 2] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 2];
static mut NPOS: usize = 0;

fn jb_off(j: *mut HSD_JObj) -> u32 {
    if j.is_null() { 0xFF } else { (j as usize).wrapping_sub(unsafe { JB.as_ptr() } as usize) as u32 }
}
fn jb(off: usize) -> *mut HSD_JObj { unsafe { JB.as_mut_ptr().add(off) as *mut HSD_JObj } }

fn bones(stride: usize, base: usize) -> &'static mut [FighterBone] {
    let v: &'static mut [FighterBone] =
        Box::leak((0..8).map(|_| FighterBone::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice());
    for (i, p) in v.iter_mut().enumerate() {
        p.joint = jb(base + i * stride);
    }
    v
}

#[test]
fn ftCo_CatchPull_matches_c() {
    init();
    let parts = bones(4, 0);
    let parts2 = bones(2, 32);
    let fd = Box::leak(Box::new(ftData::zeroed()));
    let x8 = Box::leak(Box::new(ftData_x8::zeroed()));
    let ya = Box::leak(Box::new(ftYoshiAttributes::zeroed()));
    let it = Box::leak(Box::new(Item::zeroed()));
    let kt: [u32; 8] = [Ft_Kind_Link, Ft_Kind_CLink, Ft_Kind_Samus, Ft_Kind_Yoshi, 6, 13, 20, 0];
    unsafe {
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(1); t(is_go(g)); t(b(r)) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(2); t(is_go(g)) });
        hk::it_802A7840 = Some(|g| { t(3); t((g == igo()) as u32) });
        hk::it_802BAA94 = Some(|g| { t(4); t((g == igo()) as u32) });
        hk::Geno_MotionAnimRow = Some(|fr, ms| { t(5); t((fr as *mut Fighter == F) as u32); t(ms as u32); if sr(4) { 0 } else { -1 } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(6); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::fn_800DA1D8 = Some(|g| { t(7); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(8); t(is_go(g)); sr(1) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(9); t((fr as *mut Fighter == F) as u32); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(10); t(is_go(g)) });
        hk::ft_800841B8 = Some(|g, cb| {
            t(11); t(is_go(g));
            t((cb.map(|p| p as usize) == Some(m::fn_800DA004_cb as usize)) as u32);
            unsafe { (cb.unwrap())(g) }
        });
        hk::ftCo_800DC920 = Some(|g, v| { t(12); t(is_go(g)); t((v == go2()) as u32) });
        hk::ftCo_Fall_Enter = Some(|g| { t(13); t(is_go(g)); t((g == go2()) as u32) });
        hk::lb_8000B1CC = Some(|j, off, out| {
            t(14); t(jb_off(j)); t(off.is_null() as u32);
            unsafe { *out = POS[NPOS & 1]; NPOS += 1; }
        });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(15); t((fr as *mut Fighter == F2) as u32); t(p); SRV[10] % 8 });
        hk::ftCo_800DA698 = Some(|g, a| { t(16); t(is_go(g)); t(a as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CatchPull.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { NPOS = 0; }
            *x8 = ftData_x8::zeroed();
            *ya = ftYoshiAttributes::zeroed();
            *it = Item::zeroed();
            x8.x11 = (rnd() % 8) as u8;
            fd.x8 = x8 as *mut _;
            let fr = f();
            fr.ft_data = fd as *mut _;
            fr.parts = parts.as_mut_ptr();
            f2().parts = parts2.as_mut_ptr();
            fr.victim_gobj = go2();
            unsafe { (*igo()).user_data = it as *mut Item as *mut core::ffi::c_void; }
            fr.kind = if rnd() & 1 != 0 { kt[(rnd() % 8) as usize] } else { rnd() % 34 };
            fr.motion_id = if rnd() & 1 != 0 { 0xD4 } else { 0xD5 };
            fr.cur_anim_frame = (rnd() % 50) as f32 / 2.0;
            ya.x124 = (rnd() % 10) as f32;
            ya.x128 = ya.x124 + (rnd() % 12) as f32;
            for i in 0..12 { ya.x12C[i] = rnd() as u8; }
            fr.dat_attrs = ya as *mut _ as *mut core::ffi::c_void;
            fr.u.lk.xC = if rnd() & 1 != 0 || op == 0 { igo() } else { core::ptr::null_mut() };
            fr.u.ss.x223C = if rnd() & 1 != 0 || op == 0 { igo() } else { core::ptr::null_mut() };
            it.xDD4_itemVar.linkhookshot.xC = jb(60);
            it.xDD4_itemVar.linkhookshot.x14 = (rnd() & 1) as u8;
            unsafe { fr.anon_2210.anon_0.set_throw_flags_b3((rnd() & 1) as u8); }
            fr.anon_221b.set_x221B_b7((rnd() & 1) as u8);
            fr.gr_vel = rg();
            fr.facing_dir = rg();
            fr.x2170 = rw();
            cd().x34C = rw();
            cd().x350 = rw();
            fr.co_attrs.walk_max_vel = rw();
            cd().x64 = rg();
            fr.co_attrs.ground_friction = rg();
            f2().set_x2226_b2((rnd() % 4 == 0) as u8);
            unsafe {
                for i in 0..2 { POS[i].x = rw(); POS[i].y = rw(); POS[i].z = rw(); }
            }
            fr.mv.co.capturedamage.x18 = jb(40);
            let g = go();
            match op {
                0 => m::fn_800D9CE8(g),
                1 => m::ftCo_CatchPull_Anim(g),
                2 => m::ftCo_CatchPull_IASA(g),
                3 => m::ftCo_CatchPull_Phys(g),
                4 => m::ftCo_CatchPull_Coll(g),
                5 => m::fn_800DA004(g),
                6 => m::fn_800DA054(g),
                _ => m::fn_800DA190(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.gr_vel));
            unsafe { h.m(fr.anon_2210.anon_0.throw_flags_b3() as u32); h.m((fr.anon_2210.throw_flags == 0) as u32); }
            h.m(fr.anon_221b.x221B_b7() as u32);
            h.m((fr.accessory1_cb.map(|p| p as usize) == Some(m::fn_800DA190_cb as usize)) as u32);
            h.m(jb_off(unsafe { fr.mv.co.capturedamage.x18 }));
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 16000);
}

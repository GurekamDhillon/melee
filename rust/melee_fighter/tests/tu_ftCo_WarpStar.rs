//! src/tu/ftCo_WarpStar.rs vs the C (gen/cref/tu/ftCo_WarpStar.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_WarpStar`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_WarpStar as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut BZ: f32 = 0.0;

fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    unsafe {
        let fr = f();
        if p == 0 { return 0; }
        if p == go() as usize { return 1; }
        if p == F as usize { return 3; }
        if p == igo() as usize { return 5; }
        if p == JO as usize { return 6; }
        if p == &fr.mv.co.warpstar.ecb as *const _ as usize { return 7; }
        if p == &fr.coll_data as *const _ as usize { return 8; }
        if p == &fr.cur_pos as *const _ as usize { return 9; }
        if p == &fr.x60C as *const _ as usize { return 10; }
        if p == go2() as usize { return 11; }
    }
    99
}
fn pf(fr: &Fighter) -> u32 { pi(fr as *const Fighter) }

#[test]
fn ftCo_WarpStar_matches_c() {
    init();
    unsafe {
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
        hk::it_80294430 = Some(|i, a, bb| { t(1); t(pi(i)); t(b(a)); t(b(bb)) });
        hk::it_80294364 = Some(|i| { t(2); t(pi(i)); go2() as *mut HSD_AnimJoint });
        hk::it_802944AC = Some(|i, e| { t(3); t(pi(i)); t(pi(e)) });
        hk::it_802946B0 = Some(|i| { t(4); t(pi(i)) });
        hk::it_802947CC = Some(|i, p| { t(5); t(pi(i)); t(pi(p)) });
        hk::Item_8026ABD8 = Some(|i, p, v| { t(6); t(pi(i)); t(b((*p).x)); t(b((*p).y)); t(b((*p).z)); t(b(v)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(7); t(pf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(8); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftColl_8007B6A0 = Some(|g| { t(9); t(pi(g)) });
        hk::ftColl_8007B6EC = Some(|g| { t(10); t(pi(g)) });
        hk::ftCommon_8007E2FC = Some(|g| { t(11); t(pi(g)) });
        hk::HSD_JObjAddAnim = Some(|j, a, mm, s| { t(12); t(pi(j)); t(pi(a)); t(pi(mm)); t(pi(s)) });
        hk::ftAnim_80070734 = Some(|j, fr| { t(13); t(pi(j)); t(b(fr)) });
        hk::HSD_JObjAnim = Some(|j| { t(14); t(pi(j)) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(15); t(pi(j)) });
        hk::Stage_GetBlastZoneTopOffset = Some(|| { t(16); BZ });
        hk::ftCamera_800762F4 = Some(|g| { t(17); t(pi(g)) });
        hk::mpColl_80043680 = Some(|c, p| { t(18); t(pi(c)); t(pi(p)) });
        hk::ftCommon_Fall = Some(|fr, a, bb| { t(19); t(pf(fr)); t(b(a)); t(b(bb)) });
        hk::ftCommon_CalcSelfAccel_AccelToVelClampedFrom = Some(|fr, v, a, tv, fc| { t(20); t(pf(fr)); t(b(v)); t(b(a)); t(b(tv)); t(b(fc)) });
        hk::ft_80083E64 = Some(|g, e, cb| { t(21); t(pi(g)); t(pi(e)); t(cb.is_some() as u32); if sr(21) { (cb.unwrap())(g) } });
        hk::atan2f = Some(|y, x| { t(22); t(b(y)); t(b(x)); rg() });
        hk::efAsync_Spawn = Some(|g, q, kind, id, j, pos, param| {
            t(23); t(pi(g)); t(pi(q)); t(kind); t(id); t(pi(j)); t(b((*pos).x)); t(b((*pos).y)); t(b((*pos).z)); t(b(*param))
        });
        hk::Camera_RequestQuake = Some(|kd, p| { t(24); t(kd as u32); t(b((*p).x)); t(b((*p).y)); t(b((*p).z)) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(25); t(pf(fr)); t(a); t(bb) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_WarpStar.txt"));
    for op in 0..12u32 {
        for k in 0..2000u32 {
            case(op, k);
            let jo = unsafe {
                *JO = HSD_JObj::zeroed();
                (*go()).hsd_obj = JO as *mut _;
                &mut *JO
            };
            jo.flags = rnd() & 0x0200_0040;
            jo.translate.x = rw(); jo.translate.y = rw(); jo.translate.z = rw();
            unsafe { BZ = rw() };
            let fr = f();
            fr.item_gobj = igo() as *mut Item_GObj;
            fr.facing_dir = rg();
            fr.cur_pos.x = rw(); fr.cur_pos.y = rw(); fr.cur_pos.z = rw();
            fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.self_vel.z = rw();
            fr.x2C4.y = rw();
            fr.input.lstick[0].x = rg();
            fr.co_attrs.warp_star_hitbox_scale = rw();
            fr.co_attrs.ground_to_air_jump_momentum_multiplier = rg();
            fr.co_attrs.jump_h_initial_velocity = rw();
            fr.co_attrs.jump_v_initial_velocity = rw();
            fr.co_attrs.jump_h_max_velocity = rw();
            fr.coll_data.ecb.bottom.y = rw();
            fr.coll_data.floor.normal.x = rg(); fr.coll_data.floor.normal.y = rg();
            unsafe {
                let w = &mut fr.mv.co.warpstar;
                w.facing_dir = rg();
                w.x1C = (rnd() % 4) as i32;
                w.cur_pos.x = rw();
                w.self_vel.x = rw(); w.self_vel.y = rw(); w.self_vel.z = rw();
            }
            let d = cd();
            d.x694 = rg(); d.x698 = rg(); d.x6A8 = rg(); d.x438 = rg(); d.x78 = rg();
            d.warpstarfall_drift_scaling = rg(); d.warpstarfall_drift_flat = rg(); d.warpstarfall_drift_max = rw();
            if d.warpstarfall_drift_flat.is_nan() { d.warpstarfall_drift_flat = 0.5; }
            let g = go();
            match op {
                0 => m::ftCo_800C4724(g),
                1 => m::ftCo_800C4858(g),
                2 => m::ftCo_800C48AC(g),
                3 => m::ftCo_WarpStarJump_Anim(g),
                4 => m::ftCo_WarpStarJump_Phys(g),
                5 => {
                    m::ftCo_WarpStarJump_Cam(g); m::ftCo_WarpStarFall_Cam(g); m::ftCo_WarpStarJump_IASA(g);
                    m::ftCo_WarpStarJump_Coll(g); m::ftCo_WarpStarFall_Anim(g); m::ftCo_WarpStarFall_IASA(g);
                }
                6 => m::ftCo_800C4A38(g),
                7 => m::ftCo_WarpStarFall_Phys(g),
                8 => m::ftCo_WarpStarFall_Coll(g),
                _ => m::ftCo_800C4C60(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.facing_dir)); h.m(b(fr.cur_pos.x)); h.m(b(fr.cur_pos.y)); h.m(b(fr.cur_pos.z));
            h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z));
            unsafe {
                let w = &fr.mv.co.warpstar;
                h.m(b(w.cur_pos.x)); h.m(b(w.cur_pos.y)); h.m(b(w.cur_pos.z));
                h.m(b(w.self_vel.x)); h.m(b(w.self_vel.y)); h.m(b(w.self_vel.z));
                h.m(b(w.facing_dir)); h.m(w.x1C as u32);
            }
            h.m(fr.take_dmg_cb.is_some() as u32); h.m(fr.death2_cb.is_some() as u32);
            h.m(fr.x2219_b2() as u32); h.m(fr.x2222_b7() as u32); h.m(fr.x221E_b2() as u32);
            let jo = unsafe { &*JO };
            h.m(b(jo.rotate.x)); h.m(b(jo.rotate.y)); h.m(b(jo.rotate.z)); h.m(b(jo.rotate.w));
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 24000);
}

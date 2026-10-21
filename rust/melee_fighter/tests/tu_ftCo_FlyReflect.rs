//! src/tu/ftCo_FlyReflect.rs vs the C (gen/cref/tu/ftCo_FlyReflect.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_FlyReflect`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::types::*;
use melee_fighter::tu::ftCo_FlyReflect as m;
use m::ext::hook as hk;

static mut ATAN: f32 = 0.0;
fn tv(v: &Vec3) { t(b(v.x)); t(b(v.y)); t(b(v.z)); }
fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    unsafe {
        if p == 0 { 0 } else if p == GO as usize { 1 } else if p == F as usize { 3 }
        else if p == &(*F).x60C as *const _ as usize { 4 } else { 9 }
    }
}

fn fill() {
    let d = cd();
    d.x1B0 = rg(); d.x1B8 = rnd() as i32; d.x1BC = rw(); d.x1C0 = (rnd() % 20) as f32;
    unsafe { ATAN = rw(); }
    let fr = f();
    fr.x8c_kb_vel.x = rw(); fr.x8c_kb_vel.y = rw(); fr.x8c_kb_vel.z = rg();
    fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.self_vel.z = rg();
    fr.cur_pos.x = rw(); fr.cur_pos.y = rw(); fr.cur_pos.z = rg();
    fr.facing_dir = rg();
    fr.x68C_transNPos.y = rw(); fr.x68C_transNPos.z = rg();
    fr.co_attrs.weight = rg();
    fr.coll_data.env_flags = rnd() as i32;
    fr.coll_data.ecb.top.y = rw(); fr.coll_data.ecb.left.x = rg(); fr.coll_data.ecb.left.y = rw();
    fr.coll_data.ecb.right.x = rg(); fr.coll_data.ecb.right.y = rw();
    fr.coll_data.right_facing_wall.normal.x = rg(); fr.coll_data.right_facing_wall.normal.y = rg();
    fr.coll_data.left_facing_wall.normal.x = rg(); fr.coll_data.left_facing_wall.normal.y = rg();
    fr.coll_data.ceiling.normal.x = rg(); fr.coll_data.ceiling.normal.y = rg();
    unsafe {
        fr.mv.co.damage.x18 = (rnd() % 4) as u8; fr.mv.co.damage.x19 = (rnd() % 5) as u8;
    }
    fr.motion_id = if rnd() & 1 != 0 { ftCo_MS_FlyReflectWall as _ } else { ftCo_MS_FlyReflectCeil as _ };
}

fn hash() -> u32 {
    let fr = f();
    let mut h = Hs::new();
    h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z));
    h.m(b(fr.x8c_kb_vel.x)); h.m(b(fr.x8c_kb_vel.y)); h.m(b(fr.x8c_kb_vel.z));
    h.m(b(fr.cur_pos.x)); h.m(b(fr.cur_pos.y)); h.m(b(fr.cur_pos.z)); h.m(b(fr.facing_dir));
    unsafe { h.m(fr.mv.co.damage.x18 as u32); h.m(fr.mv.co.damage.x19 as u32); }
    h.0
}

#[test]
fn ftCo_FlyReflect_matches_c() {
    init();
    unsafe {
        hk::ftCo_800C23A0 = Some(|g| { t(1); t(pi(g)); sr(1) });
        hk::ftCo_800C1D38 = Some(|g| { t(2); t(pi(g)); sr(2) });
        hk::ftKb_SpecialN_800F1F1C = Some(|g, v| { t(3); t(pi(g)); tv(v); v.z = (SRV[7] % 5) as f32; });
        hk::efAsync_Spawn = Some(|g, q, kind, id, j, a1, a2| {
            t(4); t(pi(g)); t(pi(q)); t(kind); t(id); t(j.is_null() as u32);
            tv(&*(a1 as *const Vec3)); t(b(*(a2 as *const f32)));
        });
        hk::Camera_RequestQuake = Some(|k, v| { t(5); t(k as u32); tv(&*(v as *const Vec3)) });
        hk::atan2f = Some(|y, x| { t(6); t(b(y)); t(b(x)); ATAN });
        hk::lbVector_Add_xy = Some(|a, bb| { t(7); tv(a); tv(bb); a.x += bb.x; a.y += bb.y; });
        hk::lbVector_Mirror = Some(|a, n| { t(8); tv(a); tv(n); a.x = -a.x; a.y = a.y + n.y; });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o)) });
        hk::ft_80081F2C = Some(|g| { t(10); t(pi(g)); sr(3) });
        hk::ft_80082084 = Some(|g| { t(11); t(pi(g)); sr(4) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(12); t(pi(fr as *const Fighter)); t(a); t(bb) });
        hk::ftColl_8007B760 = Some(|g, v| { t(13); t(pi(g)); t(v as u32) });
        hk::ftCo_80097630 = Some(|fr, ids, th| {
            t(14); t(pi(fr as *const Fighter));
            t((ids as *const [i32; 4] == &melee_fighter::tu::ftCo_DownBound::ftCo_DownBound_SfxIds as *const [i32; 4]) as u32);
            t(ids[0] as u32); t(b(th))
        });
        hk::ft_80084DB0 = Some(|g| { t(15); t(pi(g)) });
        hk::ftCo_80090184 = Some(|g| { t(16); t(pi(g)) });
        hk::ftCo_DamageFly_Anim = Some(|g| { t(17); t(pi(g)) });
        hk::ftCo_DamageFly_IASA = Some(|g| { t(18); t(pi(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_FlyReflect.txt"));
    for op in 0..9u32 {
        for k in 0..4000u32 {
            case(op, k);
            fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C15F4(g) as u32,
                1 => m::ftCo_800C1718(g) as u32,
                2 => m::ftCo_800C17CC(g) as u32,
                3 => {
                    let n = Vec3 { x: rg(), y: rg(), z: rg() };
                    let o = Vec3 { x: rw(), y: rw(), z: rg() };
                    let ms = if rnd() & 1 != 0 { ftCo_MS_FlyReflectWall } else { ftCo_MS_FlyReflectCeil };
                    m::ftCo_800C18A8(g, ms as i32, &n, &o);
                    0
                }
                4 => { m::ftCo_FlyReflect_Anim(g); 0 }
                5 => { m::ftCo_FlyReflect_IASA(g); 0 }
                6 => { m::ftCo_FlyReflect_Phys(g); 0 }
                _ => { m::ftCo_FlyReflect_Coll(g); 0 }
            };
            c.line(op, k, ret, hash());
        }
    }
    assert_eq!(c.done(), 36000);
}

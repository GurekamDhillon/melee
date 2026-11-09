//! src/tu/ftCo_ItemScopeFire.rs vs the C (gen/cref/tu/ftCo_ItemScopeFire.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemScopeFire`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemScopeFire as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ItemScopeFire_matches_c() {
    init();
    unsafe {
        hk::fn_800D769C = Some(|p, ms| { t(1); t((p as *const Fighter == f() as *const Fighter) as u32); t(ms as u32); if sr(0) { ms + 8 } else { ms } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCommon_ClampAirDrift = Some(|_| t(3));
        hk::ftCommon_8007D7FC = Some(|_| t(4));
        hk::ftCommon_8007D5D4 = Some(|_| t(5));
        hk::ftCommon_8007D92C = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCommon_8007E7E4 = Some(|g, a| { t(7); t(is_go(g)); t(a as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(8); sr(1) });
        hk::ftCo_800CDE54 = Some(|_| t(9));
        hk::ftCo_800CDE74 = Some(|_| t(10));
        hk::ftCo_800CDE94 = Some(|fp, i, o| {
            t(11); t((fp as *const Fighter == f() as *const Fighter) as u32);
            t(b(i.x)); t(b(i.y)); t(b(i.z));
            o.x = i.x * 2.0; o.y = i.y - 1.0; o.z = i.z + 3.0;
        });
        hk::ft_800841B8 = Some(|g, cb| { t(12); t((fnsym::same(cb as usize, m::fn_800D82EC as usize)) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(13); t((fnsym::same(cb as usize, m::fn_800D8268 as usize)) as u32); cb(g) });
        hk::ft_PlaySFX = Some(|fp, id, v, p| { t(14); t((fp as *const Fighter == f() as *const Fighter) as u32); t(id as u32); t(v as u32); t(p as u32) });
        hk::it_80291CCC = Some(|g, pos| { t(15); t((g == igo()) as u32); pos.x = rg(); pos.y = rg(); pos.z = rg(); });
        hk::it_80291FA8 = Some(|g, pos, n, facing| {
            t(16); t((g == igo()) as u32); t(b(pos.x)); t(b(pos.y)); t(b(pos.z)); t(n as u32); t(b(facing));
            pos.x += 5.0; pos.z = rg();
        });
        hk::efSync_Spawn = Some(|id, g, p, fd| {
            t(17); t(id as u32); t(is_go(g)); t(b(p.x)); t(b(p.y)); t(b(p.z));
            if id == 0x430 {
                t((fd.map(|r| r as *mut f32 as usize) == Some(&mut f().facing_dir as *mut f32 as usize)) as u32);
            }
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemScopeFire.txt"));
    for op in 0..15u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.gobj = go();
            fr.item_gobj = if rnd() % 8 != 0 { igo() } else { core::ptr::null_mut() };
            fr.motion_id = (158 + rnd() % 16) as i32;
            fr.cur_anim_frame = rw();
            fr.frame_speed_mul = rw();
            fr.facing_dir = rg();
            let tf = (rnd() & 0xFF) as u8;
            {
                let a = unsafe { &mut fr.anon_2210.anon_0 };
                a.set_throw_flags_b0(tf & 1); a.set_throw_flags_b1((tf >> 1) & 1);
                a.set_throw_flags_b2((tf >> 2) & 1); a.set_throw_flags_b3((tf >> 3) & 1);
                a.set_throw_flags_b4((tf >> 4) & 1); a.set_throw_flags_b5((tf >> 5) & 1);
                a.set_throw_flags_b6((tf >> 6) & 1); a.set_throw_flags_b7((tf >> 7) & 1);
            }
            unsafe { fr.mv.co.common.x0 = (rnd() % 14) as i32 - 2 };
            let g = go();
            match op {
                0 => { let a = (rnd() % 14) as i32 - 2; m::fn_800D8140(g, a) }
                1 => { let a = (rnd() % 14) as i32 - 2; m::fn_800D81D0(g, a) }
                2 => m::fn_800D8268(g),
                3 => m::fn_800D82EC(g),
                4 => m::fn_800D8378(g),
                5 => m::ftCo_ItemScopeFire_Anim(g),
                6 => m::ftCo_ItemScopeAirFire_Anim(g),
                7 => m::ftCo_ItemScopeFire_IASA(g),
                8 => m::ftCo_ItemScopeAirFire_IASA(g),
                9 => m::ftCo_ItemScopeFire_Phys(g),
                10 => m::ftCo_ItemScopeAirFire_Phys(g),
                11 => m::ftCo_ItemScopeFire_Coll(g),
                12 => m::ftCo_ItemScopeAirFire_Coll(g),
                13 => { let x = unsafe { f().mv.co.common.x0 }; m::fn_800D84D4(g, x) }
                _ => m::fn_800D86B8(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(match fr.accessory4_cb { None => 0, Some(p) if fnsym::same(p as usize, m::fn_800D86B8_c as usize) => 1, _ => 9 });
            h.m(match fr.take_dmg_cb { None => 0, Some(p) if fnsym::same(p as usize, m::fn_800D8378_c as usize) => 2, _ => 9 });
            h.m(unsafe { fr.mv.co.common.x0 } as u32);
            {
                let a = unsafe { &fr.anon_2210.anon_0 };
                h.m(a.throw_flags_b0() as u32); h.m(a.throw_flags_b1() as u32); h.m(a.throw_flags_b2() as u32); h.m(a.throw_flags_b3() as u32);
                h.m(a.throw_flags_b4() as u32); h.m(a.throw_flags_b5() as u32); h.m(a.throw_flags_b6() as u32); h.m(a.throw_flags_b7() as u32);
            }
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 30000);
}

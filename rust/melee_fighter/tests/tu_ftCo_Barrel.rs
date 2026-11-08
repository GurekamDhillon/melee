//! src/tu/ftCo_Barrel.rs vs the C (gen/cref/tu/ftCo_Barrel.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Barrel`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Barrel as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut ARG3: [u8; 64] = [0; 64];
fn itid(p: *mut Item_GObj) -> u32 {
    if p.is_null() { 0 } else if p as *mut HSD_GObj == igo() { 1 } else { 99 }
}
fn cbid(e: HSD_GObjEvent) -> u32 {
    let Some(p) = e else { return 0 };
    let p = p as usize;
    if p == m::ftCo_Barrel_Death2_Cb as usize { 1 }
    else if p == m::ftCo_Barrel_Accessory1_Cb as usize { 2 }
    else if p == m::ftCo_Barrel_Take_Dmg_Cb as usize { 3 }
    else if p == m::fn_800C9290 as usize { 4 }
    else { 99 }
}

#[test]
fn ftCo_Barrel_matches_c() {
    init();
    unsafe {
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
        (*go()).hsd_obj = JO as *mut core::ffi::c_void as _;
        hk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        hk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        hk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::it_802960B8 = Some(|i| { t(6); t(itid(i)); (rnd() % 4) as i32 });
        hk::it_802960CC = Some(|i| { t(7); t(itid(i)); (rnd() % 4) as i32 });
        hk::it_80295F38 = Some(|i| { t(8); t(itid(i)) });
        hk::it_8026C334 = Some(|i, p| { t(9); t(itid(i)); t((p == &mut f().cur_pos as *mut Vec3) as u32) });
        hk::itGetOwner = Some(|i| { t(10); t(itid(i)); if sr(10) { go2() } else { core::ptr::null_mut() } });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(11); t((fr as *mut Fighter == f() as *mut Fighter) as u32) });
        hk::ftCommon_8007E2FC = Some(|g| { t(12); t(is_go(g)) });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(13); t((fr as *mut Fighter == f() as *mut Fighter) as u32); t(v as i32 as u32) });
        hk::ftCommon_8007EFC0 = Some(|fr, v| { t(14); t((fr as *mut Fighter == f() as *mut Fighter) as u32); t(v) });
        hk::ftColl_800787B4 = Some(|i, g, v| { t(15); t(itid(i)); t(is_go(g)); t(v as u32) });
        hk::ftLib_IsFighter = Some(|g| { t(16); t((g == go2()) as u32); sr(16) });
        hk::ftColl_8007B8CC = Some(|fr, o| { t(17); t((fr == f() as *mut Fighter) as u32); t((o == go2()) as u32) });
        hk::ftLib_GetCameraBonePos = Some(|g, v| { t(18); t(is_go(g)); v.x = rg(); v.y = rw(); v.z = rg(); });
        hk::mpColl_80043680 = Some(|c, v| { t(19); t((c == &mut f().coll_data as *mut CollData) as u32); t(v.is_null() as u32) });
        hk::lbColl_80008D30 = Some(|h, a| {
            t(20); t((a as *mut u8 == ARG3.as_mut_ptr()) as u32);
            (*h).unk_count = rnd() % 50; (*h).element = rnd() % 12;
        });
        hk::ftColl_80079EA8 = Some(|fr, _h, n| { t(21); t((fr == f() as *mut Fighter) as u32); t(n); rw() });
        hk::ftCo_8008DCE0 = Some(|g, a, d| { t(22); t(is_go(g)); t(a as u32); t(b(d)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Barrel.txt"));
    for op in 0..10u32 {
        for k in 0..1000u32 {
            case(op, k);
            let g = go();
            let fr = f();
            unsafe {
                (*JO).translate.x = rw(); (*JO).translate.y = rg(); (*JO).translate.z = rg();
            }
            fr.facing_dir = rg();
            fr.cur_pos.x = rw(); fr.cur_pos.y = rg(); fr.cur_pos.z = rg();
            fr.mv.co.barrel.x0 = (rnd() % 3) as i32;
            fr.mv.co.barrel.x4 = (rnd() % 3) as i32;
            fr.mv.co.barrel.x8 = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.input.pressed_buttons = rnd() & 0x3FF;
            fr.dmg.x184c_damaged_hurtbox = rnd() as i32;
            let a1 = Vec3 { x: rw(), y: rg(), z: rg() };
            let ang = rg() * 400.0;
            unsafe {
                match op {
                    0 => m::ftCo_Barrel_Death2_Cb(g),
                    1 => m::ftCo_Barrel_Enter(g, igo() as *mut Item_GObj),
                    2 => m::ftCo_Barrel_Take_Dmg_Cb(g),
                    3 => m::ftCo_Barrel_Anim(g),
                    4 => m::ftCo_Barrel_IASA(g),
                    5 => m::ftCo_Barrel_Phys(g),
                    6 => m::ftCo_Barrel_Coll(g),
                    7 => m::ftCo_Barrel_Accessory1_Cb(g),
                    8 => m::fn_800C9290(g),
                    _ => m::ftCo_800C92E4(g, &a1, core::ptr::null_mut(), ARG3.as_mut_ptr() as *mut lbColl_80008D30_arg1, ang),
                }
            }
            let fr = f();
            let mut h = Hs::new();
            unsafe {
                h.m(fr.mv.co.barrel.x0 as u32); h.m(fr.mv.co.barrel.x4 as u32); h.m(itid(fr.mv.co.barrel.x8));
            }
            h.m(cbid(fr.death2_cb)); h.m(cbid(fr.accessory1_cb)); h.m(cbid(fr.take_dmg_cb)); h.m(cbid(fr.x21EC));
            h.m(fr.x221D_b5() as u32); h.m(fr.invisible() as u32); h.m(fr.x1988 as u32);
            h.m(b(fr.cur_pos.x)); h.m(b(fr.cur_pos.y)); h.m(b(fr.cur_pos.z));
            h.m(b(fr.dmg.kb_applied)); h.m(fr.dmg.x1848_kb_angle as u32); h.m(b(fr.dmg.facing_dir_1));
            h.m(fr.dmg.x184c_damaged_hurtbox as u32); h.m(b(fr.dmg.x1854_collpos.x)); h.m(b(fr.dmg.x1854_collpos.y));
            h.m(b(fr.dmg.x1854_collpos.z)); h.m(fr.dmg.x1860_element);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}

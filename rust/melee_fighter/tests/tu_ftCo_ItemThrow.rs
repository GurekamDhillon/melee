//! src/tu/ftCo_ItemThrow.rs vs the C (gen/cref/tu/ftCo_ItemThrow.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemThrow`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemThrow as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut TBL: [f32; 90] = [0.0; 90];
static mut LBV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut JO: *mut HSD_JObj = core::ptr::null_mut();

fn cbid(cb: HSD_GObjEvent) -> u32 {
    let p = match cb { None => return 0, Some(p) => p as usize };
    if p == m::ftCo_80095EFC_c as usize { 1 }
    else if p == m::ftCo_800974C4_c as usize { 2 }
    else if p == m::ftCo_800961D0_c as usize { 3 }
    else if p == m::ftCo_80096498_c as usize { 4 }
    else { 9 }
}

fn v3(v: &Vec3) { t(b(v.x)); t(b(v.y)); t(b(v.z)); }

#[test]
fn ftCo_ItemThrow_matches_c() {
    init();
    let jo = Box::leak(Box::new(HSD_JObj::zeroed()));
    let mut boolv = false;
    let pv: *mut bool = &mut boolv;
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    unsafe {
        JO = jo as *mut _;
        hk::it_8026B30C = Some(|g| { t(1); t((g == igo()) as u32); sr(0) as i32 });
        hk::ftCo_GetLStickAngle = Some(|p| { t(2); p.input.lstick[0].y * 1.25 });
        hk::ftCo_GetCStickAngle = Some(|p| { t(3); p.input.cstick[0].y * -1.25 });
        hk::ftCo_800DF50C = Some(|_| { t(4); sr(1) });
        hk::ftGetParasolStatus = Some(|_| { t(5); if sr(2) { -1 } else { 1 } });
        hk::ftCo_800968C8 = Some(|_| t(6));
        hk::ftCo_Fall_Enter = Some(|_| t(7));
        hk::Item_8026ABD8 = Some(|_, p, x| { t(8); v3(p); t(b(x)) });
        hk::itGetDamageMultiplier = Some(|_| { t(9); if sr(3) { 0.5 } else { 3.0 } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(10); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftAnim_8006EBA4 = Some(|_| t(11));
        hk::ftCo_800974C4 = Some(|_| t(12));
        hk::ftCo_80096E68 = Some(|_| t(13));
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(14); sr(4) });
        hk::ftCommon_8007D92C = Some(|_| t(15));
        hk::it_80272C90 = Some(|_| { t(16); JO });
        hk::lb_8000B1CC = Some(|j, a, out| { t(17); t((j == JO) as u32); t(a.is_null() as u32); *out = LBV });
        hk::Fighter_804D6550 = Some(|| TBL.as_mut_ptr() as *mut u8);
        hk::pl_8003E978 = Some(|s, sub, it| { t(18); t(s as u32); t(sub as u32); t((it == igo()) as u32) });
        hk::Item_8026AC74 = Some(|_, p, vv, x| { t(19); v3(p); v3(vv); t(b(x)) });
        hk::Item_8026AD20 = Some(|_, p, vv, x, s| { t(20); v3(p); v3(vv); t(b(x)); t(s as u32) });
        hk::itIsHeavy = Some(|_| { t(21); sr(5) as i32 });
        hk::ftCommon_8007EBAC = Some(|_, a, bb| { t(22); t(a); t(bb) });
        hk::ft_80084F3C = Some(|_| t(23));
        hk::ft_80085030 = Some(|_, a, bb| { t(24); t(b(a)); t(b(bb)) });
        hk::ft_80084DB0 = Some(|_| t(25));
        hk::ft_80084104 = Some(|_| t(26));
        hk::ft_800841B8 = Some(|g, cb| { t(27); t((fnsym::same(cb as usize, m::ftCo_80096250 as usize)) as u32); t((fnsym::same(cb as usize, m::ext::ftCo_80096E68 as usize)) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(28); t((fnsym::same(cb as usize, m::ftCo_80096374 as usize)) as u32); cb(g) });
        hk::ft_80082708 = Some(|_| { t(29); sr(6) as u32 });
        hk::ft_80081D0C = Some(|_| { t(30); sr(7) as u32 });
        hk::ftCommon_8007D5D4 = Some(|_| t(31));
        hk::ftCommon_8007D7FC = Some(|_| t(32));
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemThrow.txt"));
    for op in 0..28u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                for i in 0..90 { TBL[i] = rg(); }
            }
            v.x = rw(); v.y = rw(); v.z = rw();
            unsafe { LBV.x = rw(); LBV.y = rw(); LBV.z = rw(); }
            unsafe { *pv = rnd() & 1 != 0 };
            let fr = f();
            fr.item_gobj = if rnd() % 8 != 0 { igo() } else { core::ptr::null_mut() };
            fr.input.pressed_buttons = rnd() & 0x80000F60;
            fr.input.held_buttons[0] = rnd() & 0x80000F60;
            fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg();
            fr.input.cstick[0].x = rg(); fr.input.cstick[0].y = rg();
            fr.active_sticky.lstick.x = (rnd() % 16) as u8;
            fr.active_sticky.lstick.y = (rnd() % 16) as u8;
            fr.facing_dir = rg(); fr.facing_dir1 = rg();
            fr.motion_id = if rnd() % 8 == 0 { ftCo_MS_EscapeF as i32 } else { (94 + rnd() % 30) as i32 };
            fr.player_idx = (rnd() % 6) as u8;
            fr.set_is_sub_fighter((rnd() & 1) as u8);
            fr.set_x2222_b0((rnd() & 1) as u8);
            unsafe {
                fr.x2210.x0.set_throw_flags_b3((rnd() & 1) as u8);
                fr.x2210.x0.set_throw_flags_b4((rnd() & 1) as u8);
            }
            fr.ground_or_air = rnd() & 1;
            fr.cur_anim_frame = rw(); fr.frame_speed_mul = rw(); fr.cmd_timer = rw();
            let a12 = rnd() & 0xFFF;
            fr.cmd_vars[0] = if rnd() & 1 != 0 { (a12 << 20) | ((rnd() & 0xFF) << 12) | a12 } else { 0 };
            fr.cmd_vars[1] = if rnd() & 1 != 0 { rnd() } else { 0 };
            fr.co_attrs.jump_startup_time = rw(); fr.co_attrs.item_throw_velocity_multiplier = rw();
            fr.co_attrs.heavy_throw_velocity_multiplier = rw(); fr.co_attrs.ground_friction = rw();
            unsafe {
                fr.mv.co.itemthrow.facing_dir = rg();
                fr.mv.co.itemthrow4.x8.x = rw(); fr.mv.co.itemthrow4.x8.y = rw(); fr.mv.co.itemthrow4.x8.z = rw();
                fr.mv.co.itemthrow.x20 = (rnd() % 3) as i32;
            }
            let cdd = cd();
            cdd.dash_smash_stick_threshold = rg(); cdd.dash_smash_window = (rnd() % 8) as i32; cdd.x44 = rg();
            cdd.xCC = rg(); cdd.xD0 = rg(); cdd.xD4 = rg(); cdd.xD8 = rg() * 8.0; cdd.x98 = rg();
            cdd.x20_radians = rg(); cdd.attackhi3_stick_threshold_y = rg(); cdd.xB0 = rg();
            cdd.xDC = rg(); cdd.xE0 = rg(); cdd.x3FC = (rnd() % 16) as i32; cdd.x400 = rw();
            cdd.x404 = rw(); cdd.x408 = rw(); cdd.x40C = rw();
            let g = go();
            let ret = match op {
                0 => m::ftCo_80094E54(f()) as u32,
                1 => m::ftCo_80094EA4(g) as u32,
                2 => m::ftCo_8009515C(g) as u32,
                3 => m::ftCo_800951D0(g) as u32,
                4 => m::ftCo_80095254(g) as u32,
                5 => m::ftCo_800952DC(g) as u32,
                6 => { let p = rnd() & 1 != 0; m::ftCo_80095328(g, if p { pv } else { core::ptr::null_mut() }) as u32 }
                7 => m::ftCo_8009563C(g) as u32,
                8 => { let a = (94 + rnd() % 26) as i32; m::ftCo_80095700(g, a); 0 }
                9 => { let p = rnd() & 1 != 0; m::ftCo_80095744(g, if p { pv } else { core::ptr::null_mut() }); 0 }
                10 => { let ms = (94 + rnd() % 30) as i32; m::ftCo_800957F4(g, ms); 0 }
                11 => { let ms = (94 + rnd() % 30) as i32; m::ftCo_800958FC(g, ms); 0 }
                12 => { m::ftCo_80095A30(g); 0 }
                13 => { if f().motion_id < 124 { m::ftCo_80095D5C(f(), &mut v); } 0 }
                14 => { m::ftCo_ItemThrow_Anim(g); 0 }
                15 => { if f().motion_id < 124 { m::ftCo_80095EFC(g); } 0 }
                16 => { m::ftCo_ItemThrow_IASA(g); 0 }
                17 => { m::ftCo_ItemThrow_Phys(g); 0 }
                18 => { m::ftCo_LightThrowDash_Phys(g); 0 }
                19 => { m::ftCo_LightThrowAir_Phys(g); 0 }
                20 => { m::ftCo_800961D0(g); 0 }
                21 => { m::ftCo_LightThrowDashDrop_Coll(g); 0 }
                22 => { m::ftCo_LightThrow_Coll(g); 0 }
                23 => { m::ftCo_80096250(g); 0 }
                24 => { m::ftCo_LightThrowAir_Coll(g); 0 }
                25 => { m::ftCo_80096374(g); 0 }
                26 => { m::ftCo_HeavyThrow_Coll(g); 0 }
                _ => { m::ftCo_80096498(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(cbid(fr.accessory4_cb)); h.m(cbid(fr.take_dmg_cb)); h.m(cbid(fr.phys_cb)); h.m(cbid(fr.coll_cb));
            h.m(fr.cmd_vars[0]); h.m(fr.cmd_vars[1]);
            unsafe { h.m(fr.x2210.x0.throw_flags_b3() as u32); h.m(fr.x2210.x0.throw_flags_b4() as u32); }
            h.m(b(fr.facing_dir)); h.m(fr.x2224_b1() as u32); h.m(unsafe { *pv } as u32);
            unsafe {
                h.m(b(fr.mv.co.itemthrow4.x8.x)); h.m(b(fr.mv.co.itemthrow4.x8.y)); h.m(b(fr.mv.co.itemthrow4.x8.z));
                h.m(b(fr.mv.co.itemthrow4.anim_spd)); h.m(fr.mv.co.itemthrow4.unk_timer as u32);
                h.m(b(fr.mv.co.itemthrow.facing_dir)); h.m(fr.mv.co.itemthrow.x20 as u32);
            }
            h.m(b(v.x)); h.m(b(v.y)); h.m(b(v.z));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 56000);
}

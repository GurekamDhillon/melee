//! src/tu/ftCo_Bury.rs vs the C (gen/cref/tu/ftCo_Bury.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Bury`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Bury as m;
use melee_fighter::tu::ftCo_Barrel::ext::hook as bk;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut [HSD_JObj; 5] = core::ptr::null_mut();
static mut PARTS: *mut [FighterBone; 4] = core::ptr::null_mut();
static mut DD: *mut DynamicsDesc = core::ptr::null_mut();
static mut SI: i32 = 0;
const KINDS: [i32; 8] = [4, 8, 30, 32, 39, 0, 1, 2];

fn jo(i: usize) -> &'static mut HSD_JObj { unsafe { &mut (*JO)[i] } }
fn jid(j: *mut HSD_JObj) -> u32 {
    for i in 0..5 { if j == jo(i) as *mut HSD_JObj { return i as u32 + 1; } }
    if j.is_null() { 0 } else { 99 }
}
fn isf(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }
fn isdd<T>(p: *mut T) -> u32 { (p as *mut DynamicsDesc == unsafe { DD }) as u32 }
fn bury() -> &'static mut ftCommon_MotionVars_bury { unsafe { &mut f().mv.co.bury } }

#[test]
fn ftCo_Bury_matches_c() {
    init();
    unsafe {
        JO = Box::leak(Box::new([HSD_JObj::zeroed(); 5]));
        PARTS = Box::leak(Box::new([FighterBone::zeroed(); 4]));
        DD = Box::leak(Box::new(DynamicsDesc::zeroed()));
        for i in 0..4 { (*PARTS)[i].joint = jo(i); }
        bk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        bk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        bk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftCo_8009EB18 = Some(|g| { t(4); t(is_go(g)) });
        hk::ftColl_800765F0 = Some(|fr, v, x| { t(5); t(isf(fr)); t(v.is_null() as u32); t(b(x)); rw() });
        hk::ftColl_80076640 = Some(|fr, x| { t(6); t(isf(fr)); t(b(*x)); *x = rg(); sr(6) });
        hk::ftColl_80076764 = Some(|a, bb, g, d, fr, h| {
            t(7); t(a as u32); t(bb as u32); t((g == go2()) as u32); t(g.is_null() as u32); t(isdd(d)); t(isf(fr));
            t((h == &mut f().hurt_capsules[0] as *mut FighterHurtCapsule) as u32)
        });
        hk::lbColl_80008D30 = Some(|_h, a| { t(8); t(isdd(a)) });
        hk::ftColl_80078384 = Some(|fr, h, _hit| { t(9); t(isf(fr)); t((h == &mut f().hurt_capsules[0] as *mut FighterHurtCapsule) as u32) });
        hk::pl_8003EC30 = Some(|s, sub, k, x| { t(10); t(s as u32); t(sub as u32); t(k as u32); t(b(x)) });
        hk::stage_info_grkind = Some(|| SI);
        hk::Ground_801C5700 = Some(|i| { t(11); t(i as u32); if sr(11 + (i as usize & 7) * 16) { DD } else { core::ptr::null_mut() } });
        hk::ftCo_8008DCE0 = Some(|g, a, d| { t(12); t(is_go(g)); t(a as u32); t(b(d)) });
        hk::ftCommon_8007D7FC = Some(|fr| { t(13); t(isf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(14); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_8007E2FC = Some(|g| { t(15); t(is_go(g)) });
        hk::ftCommon_InitGrab = Some(|fr, a, v| { t(16); t(isf(fr)); t(a as u32); t(b(v)) });
        hk::Player_GetHandicap = Some(|s| { t(17); t(s as u32); (rnd() % 10) as i32 });
        hk::Player_80033BB8 = Some(|s| { t(18); t(s as u32); (rnd() % 5) as i32 - 1 });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(19); t(isf(fr)); t(v as i32 as u32) });
        hk::ft_80084CB0 = Some(|fr, bx| {
            t(20); t(isf(fr)); t((bx as *mut ftCollisionBox == &mut bury().coll_box as *mut _) as u32);
            bx.bottom = rg(); bx.left.y = rw();
        });
        hk::lb_8000B1CC = Some(|j, p0, p1| { t(21); t(jid(j)); t(p0.is_null() as u32); (*p1).x = rg(); (*p1).y = rw(); (*p1).z = rg(); });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(22); t(isf(fr)); t(p as u32); (rnd() % 4) as i32 });
        hk::efSync_Spawn = Some(|id, g, p, s| {
            t(23); t(id as u32); t(is_go(g)); t((p == &mut f().cur_pos as *mut Vec3) as u32); t((s == &mut f().x34_scale.y as *mut f32) as u32)
        });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(24); t(isf(fr)); t(b(v)); fr.grab_timer = rg(); sr(24) });
        hk::mpLib_80054ED8 = Some(|l| { t(25); t(l as u32); sr(25) });
        hk::mpLineGetNormal = Some(|l, n| { t(26); t(l as u32); (*n).x = rg(); (*n).y = rg(); (*n).z = rg(); n });
        hk::atan2f = Some(|y, x| { t(27); t(b(y)); t(b(x)); rw() });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(28); t(jid(j)) });
        hk::mpGetSpeed = Some(|l, p, s| {
            t(29); t(l as u32); t((p == &mut bury().translate as *mut Vec3) as u32);
            (*s).x = rg(); (*s).y = rw(); (*s).z = rg(); sr(29)
        });
        hk::PSVECAdd = Some(|a, bb, c| {
            t(30); t((a == &mut bury().translate as *mut Vec3) as u32); t((c == a) as u32);
            (*c).x = (*a).x + (*bb).x; (*c).y = (*a).y + (*bb).y; (*c).z = (*a).z + (*bb).z;
        });
        hk::ft_80082888 = Some(|g, bx| { t(31); t(is_go(g)); t((bx == &mut bury().coll_box as *mut _) as u32); sr(31) });
        hk::ftCo_800C5240 = Some(|g| { t(32); t(is_go(g)); sr(32) });
        hk::ftCo_800C5A98 = Some(|g| { t(33); t(is_go(g)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(34); t(isf(fr)) });
        hk::ftColl_8007B7A4 = Some(|g, v| { t(35); t(is_go(g)); t(v as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(36); t(is_go(g)); sr(36) });
        hk::ftCo_Fall_Enter = Some(|g| { t(37); t(is_go(g)) });
        hk::ftCo_SpecialAir_CheckInput = Some(|g| { t(38); t(is_go(g)); sr(38) });
        hk::ftCo_80095328 = Some(|g, p| { t(39); t(is_go(g)); t(p.is_null() as u32); sr(39) });
        hk::ftCo_800D7100 = Some(|g| { t(40); t(is_go(g)); sr(40) });
        hk::ftCo_800C3B10 = Some(|g| { t(41); t(is_go(g)); sr(41) });
        hk::ftCo_80099A58 = Some(|g| { t(42); t(is_go(g)); sr(42) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|g| { t(43); t(is_go(g)); sr(43) });
        hk::ftCo_800D705C = Some(|g| { t(44); t(is_go(g)); sr(44) });
        hk::ftCo_800CB870 = Some(|g| { t(45); t(is_go(g)); sr(45) });
        hk::ftCommon_Fall = Some(|fr, a, bb| { t(46); t(isf(fr)); t(b(a)); t(b(bb)) });
        hk::ftCommon_CalcSelfAccel_Drift = Some(|fr| { t(47); t(isf(fr)) });
        hk::ft_80082D40 = Some(|g, v| { t(48); t(is_go(g)); t(b(v)) });
        hk::ftCo_AirCatchHit_Coll = Some(|g| { t(49); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Bury.txt"));
    let fl = (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_SRT) as u32;
    for op in 0..24u32 {
        for k in 0..1000u32 {
            case(op, k);
            let g = go();
            for i in 0..5 { *jo(i) = HSD_JObj::zeroed(); }
            for i in 0..5 { jo(i).flags = rnd() & fl; }
            unsafe { (*go2()).hsd_obj = jo(4) as *mut HSD_JObj as _; }
            let fr = f();
            fr.parts = unsafe { PARTS as *mut FighterBone };
            // the C reads `count` as GC word 1; natively the stage record is a lbColl_80008D30_arg1 whose word 1 is `damage` (ftCo_800C08A0)
            unsafe { (*(DD as *mut melee_types::lbColl_80008D30_arg1)).damage = rnd() % 100; SI = KINDS[(rnd() % 8) as usize]; }
            let cdm = cd();
            cdm.bury_timer_unk1 = rnd(); cdm.bury_timer_unk2 = rnd(); cdm.bury_timer_unk3 = rnd();
            cdm.x5F4 = (rnd() % 5) as i32;
            cdm.x5F8 = rw(); cdm.x5FC = rw(); cdm.x600 = rw(); cdm.x604 = rw(); cdm.x608 = rw();
            cdm.x60C = rw(); cdm.x610 = rg(); cdm.x614 = rg(); cdm.x618 = rw(); cdm.x61C = rg();
            cdm.x620 = rnd() as i32;
            fr.bury_timer_1 = rnd() % 3; fr.bury_timer_2 = rnd() % 3; fr.bury_grkind = KINDS[(rnd() % 8) as usize] as _;
            fr.motion_id = if rnd() & 1 != 0 { ftCo_MS_BarrelWait as _ } else { (rnd() % 400) as _ };
            fr.x1A6A = (rnd() & 0xFFFF) as u16;
            fr.set_stamina_dead((rnd() & 1) as u8); fr.set_x2227_b6((rnd() & 1) as u8); fr.set_is_sub_fighter((rnd() & 1) as u8);
            fr.dmg.x1860_element = rnd() % 12; fr.ground_or_air = (rnd() & 1) as _;
            fr.dmg.x1830_percent = rw();
            fr.player_idx = (rnd() % 4) as u8;
            fr.coll_data.env_flags = if rnd() & 1 != 0 { 0 } else { (rnd() & 0x1FFFF) as i32 };
            fr.coll_data.floor.index = (rnd() % 8) as i32; fr.coll_data.ceiling.index = (rnd() % 8) as i32;
            fr.coll_data.right_facing_wall.index = (rnd() % 8) as i32; fr.coll_data.left_facing_wall.index = (rnd() % 8) as i32;
            fr.grab_timer = rw();
            let bu = bury();
            bu.x0 = (rnd() % 3) as i32;
            bu.x20 = (rnd() % 8) as _;
            bu.coll_box.top = rg(); bu.coll_box.bottom = rw();
            bu.coll_box.left.x = rg(); bu.coll_box.left.y = rw();
            bu.coll_box.right.x = rg(); bu.coll_box.right.y = rw();
            bu.x1C = rg();
            bu.translate.x = rw(); bu.translate.y = rg(); bu.translate.z = rg();
            if rnd() & 1 != 0 { fr.mv.ca.specials.grav = rg(); }
            if rnd() & 1 != 0 { fr.mv.co.buryjump.x0 = rw(); }
            fr.self_vel.x = rg(); fr.self_vel.y = rg();
            fr.co_attrs.gravity = rg(); fr.co_attrs.terminal_velocity = rw();
            fr.cur_pos.x = rw(); fr.cur_pos.y = rg(); fr.cur_pos.z = rg();
            let arg = (rnd() % 5) as i32;
            let nul = core::ptr::null_mut();
            let ret = match op {
                0 => { m::ftCo_800C0874(g, nul, arg); 0 }
                1 => { let a1 = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() }; m::ftCo_800C08A0(g, a1, unsafe { DD }, arg); 0 }
                2 => { m::ftCo_800C09B4(g); 0 }
                3 => m::ftCo_800C0A28(g, nul, arg) as u32,
                4 => { m::ftCo_800C0A98(g); 0 }
                5 => { m::ftCo_800C0B20(g); 0 }
                6 => m::ftCo_800C0C88(arg) as u32,
                7 => m::ftCo_800C0CB8(g) as u32,
                8 => { m::ftCo_800C0D0C(g); 0 }
                9 => { m::ftCo_Bury_Anim(g); 0 }
                10 => { m::ftCo_Bury_IASA(g); 0 }
                11 => { m::ftCo_800C0FCC(go2(), g); 0 }
                12 => { m::ftCo_Bury_Phys(g); 0 }
                13 => { m::ftCo_Bury_Coll(g); 0 }
                14 => { m::ftCo_800C124C(g); 0 }
                15 => { m::ftCo_BuryWait_Anim(g); 0 }
                16 => { m::ftCo_BuryWait_IASA(g); 0 }
                17 => { m::ftCo_BuryWait_Phys(g); 0 }
                18 => { m::ftCo_BuryWait_Coll(g); 0 }
                19 => { m::ftCo_800C13BC(g); 0 }
                20 => { m::ftCo_BuryJump_Anim(g); 0 }
                21 => { m::ftCo_BuryJump_IASA(g); 0 }
                22 => { m::ftCo_BuryJump_Phys(g); 0 }
                _ => { m::ftCo_BuryJump_Coll(g); 0 }
            };
            let fr = f();
            let bu = bury();
            let mut h = Hs::new();
            h.m(fr.bury_timer_1); h.m(fr.bury_timer_2); h.m(fr.bury_grkind as u32);
            h.m(fr.x221D_b5() as u32); h.m(fr.x2220_b3() as u32); h.m(fr.x2224_b4() as u32); h.m(fr.x2219_b0() as u32);
            h.m(bu.x0 as u32); h.m(b(bu.x1C)); h.m(bu.x20 as u32);
            h.m(b(bu.coll_box.top)); h.m(b(bu.coll_box.bottom));
            h.m(b(bu.coll_box.left.x)); h.m(b(bu.coll_box.left.y));
            h.m(b(bu.coll_box.right.x)); h.m(b(bu.coll_box.right.y));
            h.m(b(bu.translate.x)); h.m(b(bu.translate.y)); h.m(b(bu.translate.z));
            h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(unsafe { fr.mv.co.buryjump.x0 })); h.m(b(fr.grab_timer));
            for i in 0..5 {
                let j = jo(i);
                h.m(b(j.rotate.z)); h.m(b(j.translate.x)); h.m(b(j.translate.y)); h.m(b(j.translate.z));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 24000);
}

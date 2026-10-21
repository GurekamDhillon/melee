//! src/tu/ftCo_DownBound.rs vs the C (gen/cref/tu/ftCo_DownBound.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DownBound`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DownBound as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut PARTS: *mut FighterBone = core::ptr::null_mut();
static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut ATAN: f32 = 0.0;
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }
fn jo(i: usize) -> *mut HSD_JObj { unsafe { JO.add(i) } }

#[test]
fn ftCo_DownBound_matches_c() {
    init();
    unsafe {
        PARTS = Box::leak(Box::new([FighterBone::zeroed(); 8])).as_mut_ptr();
        JO = Box::leak(Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed()])).as_mut_ptr();
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(1); t(isf(fr)); t(p as u32); (SRV[5] % 2 + 3) as i32 });
        hk::HSD_JObjSetupMatrixSub = Some(|j| { t(2); t((j == jo(0)) as u32); t((j == jo(1)) as u32) });
        hk::atan2f = Some(|y, x| { t(3); t(b(y)); t(b(x)); ATAN });
        hk::efAsync_Spawn = Some(|g, q, kind, id, j, p| {
            t(4); t(is_go(g)); t((q == &mut f().x60C as *mut _ as *mut core::ffi::c_void) as u32); t(kind); t(id);
            t((j == jo(0)) as u32); t((j == jo(1)) as u32); t(b(*p));
        });
        hk::ft_PlaySFX = Some(|fr, id, a, bb| { t(5); t(isf(fr)); t(id as u32); t(a as u32); t(bb as u32) });
        hk::ft_80084C74 = Some(|g, sfx, a2, a3| {
            t(6); t(is_go(g)); t(*a2 as u32);
            for i in 0..4 { sfx[i] = if sr(10 + i) { -1 } else { (SRV[6 + i] % 50) as i32 }; }
            if sr(16) { *a2 = sr(17) as i32; }
            *a3 = if sr(15) { -1 } else { (1000 + SRV[10] % 50) as i32 };
            sr(14)
        });
        hk::ftCo_8009F834 = Some(|g, id, p, a, bb, v1, v0, fl| {
            t(7); t(is_go(g)); t(id as u32); t(p as u32); t(a as u32); t(bb as u32);
            t(b(v1.x)); t(b(v1.y)); t(b(v1.z)); t(b(v0.x)); t(b(v0.y)); t(b(v0.z)); t(b(fl));
        });
        hk::Camera_RequestQuake = Some(|k, v| { t(8); t(k as u32); t((v == &mut f().cur_pos as *mut Vec) as u32) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(9); t(isf(fr)); t(a); t(bb) });
        hk::ftCommon_8007D7FC = Some(|fr| { t(10); t(isf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(11); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(12); t(isf(fr)); t(v as i32 as u32) });
        hk::ftCommon_SetGroundedKnockbackIfLanded = Some(|fr| { t(13); t(isf(fr)) });
        hk::ft_8008A2BC = Some(|g| { t(14); t(is_go(g)) });
        hk::ftSb_Init_8014FBA4 = Some(|g| { t(15); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(16); sr(3) });
        hk::ftCo_80098400 = Some(|_| { t(17); sr(4) });
        hk::ftCo_Down_CheckInput = Some(|_| { t(18); sr(5) });
        hk::ft_80084F3C = Some(|g| { t(19); t(is_go(g)) });
        hk::ft_80082708 = Some(|_| { t(20); sr(6) as i32 });
        hk::ftCo_Fall_Enter = Some(|g| { t(21); t(is_go(g)) });
        hk::ftCo_800C7CA0 = Some(|_| { t(22); sr(7) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(23); t(is_go(g)) });
        hk::ftCo_80098160 = Some(|g, ms| { t(24); t(is_go(g)); t(ms as u32) });
        hk::ftCo_800984D4 = Some(|_| { t(25); sr(8) });
        hk::ftCo_800980BC = Some(|_| { t(26); sr(9) });
        hk::ft_80083F88 = Some(|g| { t(27); t(is_go(g)) });
    }
    let ms = [ftCo_MS_DownBoundU, ftCo_MS_DownSpotU, ftCo_MS_DownWaitU, ftCo_MS_DownDamageU, ftCo_MS_DownWaitD, 0];
    let jf = [0u32, JOBJ_MTX_DIRTY as u32, JOBJ_USER_DEF_MTX as u32, (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX) as u32];
    let mut sfxa = [0i32; 4];
    let mut c = Check::new(include_str!("data/tu_ftCo_DownBound.txt"));
    for op in 0..21u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                for i in 0..8 { *PARTS.add(i) = FighterBone::zeroed(); }
                *jo(0) = HSD_JObj::zeroed();
                *jo(1) = HSD_JObj::zeroed();
                for i in 0..8 { (*PARTS.add(i)).joint = if rnd() & 7 != 0 { jo(i & 1) } else { core::ptr::null_mut() }; }
                (*PARTS.add(3)).joint = jo(0);
                (*PARTS.add(4)).joint = jo(1);
                for i in 0..2 {
                    (*jo(i)).flags = jf[(rnd() % 4) as usize];
                    for r in 0..3 { for cc in 0..4 { (*jo(i)).mtx[r][cc] = rg(); } }
                }
                ATAN = rg();
            }
            for i in 0..4 { sfxa[i] = (rnd() % 30) as i32; }
            let d = cd();
            d.x1F0 = rw(); d.x1F4 = rw(); d.x424 = rw();
            let fr = f();
            fr.parts = unsafe { PARTS };
            fr.set_x2226_b0((rnd() & 1) as u8); fr.set_x2226_b1((rnd() & 1) as u8); fr.set_x2224_b2((rnd() & 1) as u8);
            fr.set_is_sandbag((rnd() & 1) as u8);
            fr.ground_or_air = (rnd() & 1) as _;
            fr.motion_id = ms[(rnd() % 6) as usize] as _;
            fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.x8c_kb_vel.x = rw(); fr.x8c_kb_vel.y = rw();
            fr.co_attrs.weight = rw();
            fr.coll_data.floor.normal.x = rg(); fr.coll_data.floor.normal.y = rg();
            fr.facing_dir = rg();
            fr.x67C = (rnd() % 256) as u8; fr.x67D = (rnd() % 256) as u8;
            unsafe {
                fr.mv.co.downwait.x0 = rw();
                fr.mv.co.downspot.x4 = (rnd() % 256) as u8;
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_80097570(g) as u32,
                1 => { let th = rw(); m::ftCo_80097630(f(), &sfxa, th); 0 }
                2 => { m::ftCo_800976A4(g); 0 }
                3 => { m::ftCo_800978D4(g); 0 }
                4 => { m::ftCo_8009794C(g); 0 }
                5 => { m::ftCo_80097AF4(g); 0 }
                6 => { m::ftCo_80097D40(g); 0 }
                7 => { m::ftCo_80097D88(g); 0 }
                8 => { m::ftCo_DownBound_Anim(g); 0 }
                9 => { m::ftCo_DownBound_IASA(g); 0 }
                10 => { m::ftCo_DownBound_Phys(g); 0 }
                11 => { m::ftCo_DownBound_Coll(g); 0 }
                12 => { m::ftCo_80097E8C(g); 0 }
                13 => { m::ftCo_80097F38(g); 0 }
                14 => { m::ftCo_DownWait_Anim(g); 0 }
                15 => { m::ftCo_DownWait_IASA(g); 0 }
                16 => { m::ftCo_DownWait_Phys(g); 0 }
                17 => { m::ftCo_DownWait_Coll(g); 0 }
                18 => { let mut p = 0.0; m::ftCo_800978D4_inline(g, &mut p); b(p) }
                19 => { let mut dd = rw(); m::ftCo_800976A4_inline(f(), &mut dd); b(dd) }
                _ => { m::inlineA1(g); 0 }
            };
            let f = f();
            let mut h = Hs::new();
            h.m(f.x67C as u32); h.m(f.x67D as u32);
            unsafe { h.m(b(f.mv.co.downwait.x0)); h.m(f.mv.co.downspot.x4 as u32); }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 63000);
}

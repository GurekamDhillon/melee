//! src/tu/ftCo_YoshiEgg.rs vs the C (gen/cref/tu/ftCo_YoshiEgg.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_YoshiEgg`).
#![allow(static_mut_refs, non_snake_case, unused_unsafe)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_YoshiEgg as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JG: *mut HSD_JObj = core::ptr::null_mut();
static mut JA: *mut HSD_JObj = core::ptr::null_mut();
static mut JC: *mut HSD_JObj = core::ptr::null_mut();
static mut JP: *mut [HSD_JObj; 4] = core::ptr::null_mut();
static mut PARTS: *mut [FighterBone; 4] = core::ptr::null_mut();
static mut JD: *mut HSD_Joint = core::ptr::null_mut();

fn jid(j: *mut HSD_JObj) -> u32 {
    unsafe {
        if j == JG { return 1; }
        if j == JA { return 2; }
        if j == JC { return 3; }
        for i in 0..4 { if j == &mut (*JP)[i] as *mut _ { return 10 + i as u32; } }
        if j.is_null() { 0 } else { 99 }
    }
}
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == f() as *const Fighter) as u32 }
fn is_go2(g: *mut HSD_GObj) -> u32 { (g == go2()) as u32 }

#[test]
fn ftCo_YoshiEgg_matches_c() {
    init();
    unsafe {
        JG = Box::leak(Box::new(HSD_JObj::zeroed()));
        JA = Box::leak(Box::new(HSD_JObj::zeroed()));
        JC = Box::leak(Box::new(HSD_JObj::zeroed()));
        JP = Box::leak(Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed()]));
        PARTS = Box::leak(Box::new([FighterBone::zeroed(), FighterBone::zeroed(), FighterBone::zeroed(), FighterBone::zeroed()]));
        JD = Box::leak(Box::new(HSD_Joint::zeroed()));

        hk::ftColl_8007B62C = Some(|g, v| { t(1); t(is_go(g)); t(v as u32) });
        hk::Fighter_UpdateModelScale = Some(|g| { t(2); t(is_go(g)) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(3); t(jid(j)) });
        hk::ftColl_8007B0C0 = Some(|g, s| { t(4); t(is_go(g)); t(s as u32) });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(5); t(isf(fr)); t(p as u32); ((SRV[5] >> (p * 2)) & 3) as i32 });
        hk::ftColl_HurtboxInit = Some(|fr, c, h| {
            t(6); t(isf(fr)); t((c == f().hurt_capsules.as_mut_ptr()) as u32);
            t(h.bone_idx as u32); t(h.height as u32); t(h.is_grabbable as u32);
            t(b(h.a_offset.x)); t(b(h.a_offset.y)); t(b(h.a_offset.z));
            t(b(h.b_offset.x)); t(b(h.b_offset.y)); t(b(h.b_offset.z)); t(b(h.scale));
        });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(7); t(isf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(8); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(is_go2(o))
        });
        hk::ftCo_800BC438 = Some(|_| t(9));
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(10); t(isf(fr)); t(v as i32 as u32) });
        hk::ftCommon_SetAccessory = Some(|fr, j| { t(11); t(isf(fr)); t((j == JD) as u32) });
        hk::ftYs_SpecialN_8012CDD4 = Some(|g| { t(12); t(is_go2(g)); JD });
        hk::lb_8000C2F8 = Some(|a, bb| { t(13); t(jid(a)); t(jid(bb)) });
        hk::ftYs_SpecialN_SetupItemVel = Some(|g, out| {
            t(14); t(is_go2(g)); t((out as *mut Vec3 == &mut f().self_vel as *mut Vec3) as u32);
            out.x = rw(); out.y = rw(); out.z = rw();
        });
        hk::ftYs_SpecialN_GetFacingDir = Some(|g| { t(15); t(is_go2(g)); rw() });
        hk::ftYs_SpecialN_GetDatAttr18 = Some(|g| { t(16); t(is_go2(g)); rw() });
        hk::ftYs_SpecialN_GetDatAttr1C = Some(|g| { t(17); t(is_go2(g)); rw() });
        hk::ftYs_SpecialN_GetDatAttr20 = Some(|g| { t(18); t(is_go2(g)); rw() });
        hk::ftYs_SpecialN_GetDatAttr24 = Some(|g| { t(19); t(is_go2(g)); rw() });
        hk::ftCommon_8007EFC0 = Some(|fr, v| { t(20); t(isf(fr)); t(v) });
        hk::ftCommon_InitGrab = Some(|fr, a, tm| { t(21); t(isf(fr)); t(a as u32); t(b(tm)) });
        hk::ftYs_SpecialN_GetExtAttr28 = Some(|| { t(22); rw() });
        hk::ftYs_SpecialN_GetExtAttr2C = Some(|| { t(23); rw() });
        hk::ftYs_SpecialN_8012CD40 = Some(|| { t(24); rw() });
        hk::ftYs_SpecialN_GetExtAttr34 = Some(|| { t(25); rw() });
        hk::ftYs_SpecialN_8012CDB4 = Some(|| { t(26); rw() });
        hk::ftYs_SpecialN_GetExtAttr38 = Some(|| { t(27); sr(27) as i32 });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(28); t(isf(fr)); t(b(v)); sr(28) });
        hk::ft_PlaySFX = Some(|fr, id, v, p| { t(29); t(isf(fr)); t(id as u32); t(v as u32); t(p as u32) });
        hk::efAsync_Spawn = Some(|g, q, k, gfx, j, a| {
            t(30); t(is_go(g)); t((q == &mut f().x60C as *mut _ as *mut core::ffi::c_void) as u32);
            t(k); t(gfx); t(jid(j)); t((a == &f().co_attrs.xBC as *const _ as *const core::ffi::c_void) as u32);
        });
        hk::ftYs_SpecialN_8012CD88 = Some(|v| { t(31); t((v as *mut Vec3 == &mut f().self_vel as *mut Vec3) as u32) });
        hk::ftColl_8007B760 = Some(|g, v| { t(32); t(is_go(g)); t(v as u32) });
        hk::ftCo_Fall_Enter_YoshiEgg = Some(|g| { t(33); t(is_go(g)) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(34); t(is_go(g)); t(b(r)) });
        hk::ft_80084F3C = Some(|g| { t(35); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(36); t(is_go(g)) });
        hk::ft_8008403C = Some(|g, cb| { t(37); t(is_go(g)); t((fnsym::same(cb as usize, m::ftCo_800BC3AC as usize)) as u32) });
        hk::ft_80082C74 = Some(|g, cb| { t(38); t(is_go(g)); t((fnsym::same(cb as usize, m::ftCo_800BC388 as usize)) as u32) });
        hk::ftCommon_8007D7FC = Some(|fr| { t(39); t(isf(fr)) });
        hk::ftCo_800C0C88 = Some(|ty| { t(40); t(ty as u32); sr(40) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_YoshiEgg.txt"));
    let dirty_indep = (JOBJ_MTX_DIRTY | JOBJ_MTX_INDEP_SRT) as u32;
    for op in 0..10u32 {
        for k in 0..1000u32 { unsafe {
            case(op, k);
            unsafe {
                *JG = HSD_JObj::zeroed(); *JA = HSD_JObj::zeroed(); *JC = HSD_JObj::zeroed();
                for i in 0..4 { (*JP)[i] = HSD_JObj::zeroed(); (*PARTS)[i] = FighterBone::zeroed(); }
                (*go()).hsd_obj = JG as *mut core::ffi::c_void;
                (*JA).child = JC;
                for i in 0..4 { (*PARTS)[i].joint = &mut (*JP)[i] as *mut HSD_JObj as _; }
                f().parts = PARTS as *mut FighterBone;
                f().x20A0_accessory = JA;
                (*JG).flags = rnd() & dirty_indep;
                (*JA).flags = rnd() & dirty_indep;
                (*JG).scale.x = rw(); (*JG).scale.y = rw(); (*JG).scale.z = rw();
            }
            let fr = f();
            fr.ground_or_air = (rnd() & 1) as _;
            fr.grab_timer = rw();
            fr.mv.co.yoshiegg.x4 = (rnd() & 1) as i32;
            fr.mv.co.yoshiegg.x8 = rg() * 3.0;
            fr.mv.co.yoshiegg.xC = rg();
            fr.mv.co.yoshiegg.x10 = rw();
            fr.mv.co.yoshiegg.x14 = rg() * 4.0;
            fr.mv.co.yoshiegg.x18.x = rw(); fr.mv.co.yoshiegg.x18.y = rw(); fr.mv.co.yoshiegg.x18.z = rw();
            fr.mv.co.yoshiegg.scale.x = rw(); fr.mv.co.yoshiegg.scale.y = rw(); fr.mv.co.yoshiegg.scale.z = rw();
            fr.co_attrs.xBC.size = rw();
            fr.co_attrs.xBC.x4.x = rw(); fr.co_attrs.xBC.x4.y = rw(); fr.co_attrs.xBC.x4.z = rw();
            fr.co_attrs.xBC.x10.x = rw(); fr.co_attrs.xBC.x10.y = rw(); fr.co_attrs.xBC.x10.z = rw();
            fr.co_attrs.xBC.x1C = rw();
            fr.dmg.x1838_percentTemp = rw();
            fr.dmg.x18CC = (rnd() % 4) as i32;
            fr.dmg.x18D0 = (rnd() % 5) as i32;
            let g = go();
            match op {
                0 => m::ftCo_800BBC88(g),
                1 => m::ftCo_800BBCC0(g),
                2 => m::ftCo_800BBED4(g, go2()),
                3 => m::ftCo_YoshiEgg_Anim(g),
                4 => m::ftCo_YoshiEgg_IASA(g),
                5 => m::ftCo_YoshiEgg_Phys(g),
                6 => m::ftCo_YoshiEgg_Coll(g),
                7 => m::ftCo_800BC388(g),
                8 => m::ftCo_800BC3AC(g),
                _ => m::ftCo_800BC3D0(g),
            }
            let fr = f();
            let ye = fr.mv.co.yoshiegg;
            let mut h = Hs::new();
            // NaNs hash as one value (payload/sign not pinned: LLVM may commute fmul); see the C driver.
            let bn = |x: f32| if x.is_nan() { 0x7FC0_0000 } else { b(x) };
            let hv = |h: &mut Hs, v: &Vec3| { h.m(bn(v.x)); h.m(bn(v.y)); h.m(bn(v.z)); };
            h.m(b(fr.grab_timer)); h.m(fr.invisible() as u32); h.m(fr.x1828 as u32);
            h.m((ye.x0 == go2()) as u32); h.m(ye.x4 as u32); h.m(b(ye.x8));
            h.m(b(ye.xC)); h.m(b(ye.x10)); h.m(b(ye.x14));
            hv(&mut h, &ye.x18); hv(&mut h, &ye.scale); h.m(b(fr.mv.ca.specialhi.vel.y));
            hv(&mut h, &fr.self_vel); h.m(b(fr.facing_dir)); h.m(b(fr.dmg.x182c_behavior));
            h.m(fr.take_dmg_cb.map_or(false, |p| fnsym::same(p as usize, m::ftCo_800BC438_cb as usize)) as u32);
            h.m(fr.take_dmg_2_cb.map_or(false, |p| fnsym::same(p as usize, m::ftCo_800BC3D0_cb as usize)) as u32);
            h.m(match fr.accessory4_cb { None => 0, Some(p) if fnsym::same(p as usize, m::ftCo_800BBCC0_cb as usize) => 1, _ => 2 });
            unsafe { hv(&mut h, &(*JG).scale); hv(&mut h, &(*JA).scale); }
            c.line(op, k, 0, h.0);
        } }
    }
    assert_eq!(c.done(), 10000);
}

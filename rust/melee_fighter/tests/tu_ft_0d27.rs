//! Bit-exact check of the deterministic functions of src/tu/ft_0d27.rs (ftCo_800D27C4,
//! fn_800D2818, fn_800D2A3C fused, fn_800D2B30, fn_800D2BF8), ft_0cd1.rs (ftCo_800CD278) and
//! ft_0cdd.rs (ftCo_800CDE94) against the C cut verbatim and built with the shipping
//! contraction (gen/cref/tu_ft_0d27_ref.c, data from gen/cref/tu_ft_0d27_run.py), plus smoke
//! tests of the stateful ones that reach no unported callee.
#![allow(static_mut_refs)]
use melee_fighter::tu::{ft_0cd1, ft_0cdd, ft_0d27};
use melee_fighter::types::*;

static mut TR: u32 = 0;
fn t(v: u32) { unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) } }
fn b(f: f32) -> u32 { f.to_bits() }
static mut GOP: *mut HSD_GObj = core::ptr::null_mut();
static mut AFLAGS: u32 = 0;

fn set_scale(g: *mut HSD_GObj, s: f32) { t(1); t((g == unsafe { GOP }) as u32); t(b(s)); }
fn jobj_anim(_j: *mut HSD_JObj) { t(2); }
fn f84fa8(g: *mut HSD_GObj) { t(4); t((g == unsafe { GOP }) as u32); }
fn f85030(g: *mut HSD_GObj, fr: f32, dir: f32) { t(5); t((g == unsafe { GOP }) as u32); t(b(fr)); t(b(dir)); }
fn mtx(_j: *mut HSD_JObj, s: &Vec3, d: &mut Vec3) { t(7); t(b(s.x)); t(b(s.y)); t(b(s.z)); *d = *s; }
fn e2fc(g: *mut HSD_GObj) { t(8); t((g == unsafe { GOP }) as u32); }

struct R(u32);
impl R {
    fn n(&mut self) -> u32 { let mut s = self.0; s ^= s << 13; s ^= s >> 17; s ^= s << 5; self.0 = s; s }
    fn g(&mut self) -> f32 {
        match self.n() % 12 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => f32::from_bits(0x7FC0_0000),
            _ => { let a = self.n() & 0x807F_FFFF; let e = (0x70 + self.n() % 0x20) << 23; f32::from_bits(a | e) }
        }
    }
}
fn hv(p: *const u8, n: usize) -> u32 {
    let mut h = 7u32;
    for i in 0..n { h = h.wrapping_mul(131).wrapping_add(unsafe { *p.add(i) } as u32); }
    h
}
fn hvv(v: &Vec3) -> u32 { hv((v as *const Vec3).cast(), 12) }

#[test]
fn matches_c() {
    let data = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ft_0d27.txt")).unwrap();
    let mut lines = data.lines();
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd8: Box<ftData_x8> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: Box<[FighterBone; 4]> = Box::new(unsafe { core::mem::zeroed() });
    let mut jo: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut ao: Box<HSD_AObj> = Box::new(unsafe { core::mem::zeroed() });
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        GOP = gp;
        melee_fighter::hooks::FTLIB_SET_SCALE = Some(set_scale);
        ft_0d27::ext::HSD_JOBJ_ANIM = Some(jobj_anim);
        ft_0d27::FTCOMMON_8007E2FC = Some(e2fc);
        ft_0cd1::FT_80084FA8 = Some(f84fa8);
        ft_0cd1::FT_80085030 = Some(f85030);
        ft_0cdd::ext::MTX_MULT_VEC = Some(mtx);
        melee_fighter::hooks::P_FT_COMMON_DATA = &*cd;
    }
    let mut r = R(0x2545F491);
    let mut n = 0;
    for op in 0..7 {
        for k in 0..300 {
            let (mut ret, mut hsh) = (0u32, 0u32);
            unsafe {
                core::ptr::write_bytes(&mut *f as *mut Fighter, 0, 1);
                core::ptr::write_bytes(&mut *jo as *mut HSD_JObj, 0, 1);
                TR = 0;
            }
            g.user_data = (&mut *f as *mut Fighter).cast();
            f.gobj = gp;
            f.ft_data = &mut *fd;
            fd.x8 = &mut *fd8;
            fd8.x10 = (r.n() % 4) as u8;
            f.parts = parts.as_mut_ptr();
            parts[fd8.x10 as usize].joint = &mut *jo;
            f.x2184 = &mut *jo;
            jo.aobj = &mut *ao;
            f.motion_id = match r.n() % 4 {
                0 => 325,
                1 => 314 + r.n() % 8,
                2 => 313 + (r.n() & 1) * 9,
                _ => r.n() % 400,
            } as _;
            unsafe {
                f.x2070.anon_0.set_x2071_b0_3((r.n() & 15) as u8);
                f.x2070.anon_0.set_x2071_b4((r.n() & 1) as u8);
                AFLAGS = if r.n() & 0x4000_0000 != 0 { 0x4000_0000 } else { r.n() & 0x3FFF_FFFF };
                (*jo.aobj).flags = AFLAGS;
            }
            jo.scale.x = r.g();
            unsafe {
                f.mv.co.walk.middle_anim_frame = r.g();
                f.mv.co.walk.fast_anim_frame = r.g();
            }
            cd.x420 = r.g();
            f.co_attrs.ground_friction = r.g();
            f.facing_dir = r.g();
            f.x34_scale.y = if r.n() % 3 != 0 { r.g() } else { 1.0 };
            let inp = Vec3 { x: r.g(), y: r.g(), z: r.g() };
            let csz = core::mem::size_of::<ftCommon_MotionVars_common>();
            unsafe {
                let v = (&mut f.mv.co.common as *mut ftCommon_MotionVars_common).cast::<f32>();
                for i in 0..csz / 4 { *v.add(i) = r.g(); }
            }
            f.xE4_ground_accel_1 = r.g();
            f.xE8_ground_accel_2 = r.g();
            f.gr_vel = r.g();
            f.xF0_ground_kb_vel = r.g();
            f.xF4_ground_attacker_shield_kb_vel = r.g();
            f.x74_self_accel.x = r.g();
            f.self_vel.y = r.g();
            f.x8c_kb_vel.z = r.g();
            f.x98_atk_shield_kb.x = r.g();
            f.xD4_unk_vel.y = r.g();
            if op == 3 { unsafe { f.mv.co.swing.x4 = (r.n() % 6) as i32 } }
            match op {
                0 => ret = ft_0d27::ftco_800d27c4(&f) as u32,
                1 => ret = ft_0d27::fn_800d2818(&f) as u32,
                2 => {
                    // C: HSD_AObjGetFlags is a stub that records 3
                    ret = ft_0d27::fn_800d2a3c(gp) as u32;
                }
                3 => ft_0cd1::ftco_800cd278(gp),
                4 => {
                    let mut out = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    ft_0cdd::ftco_800cde94(&mut f, &inp, &mut out);
                    ret = b(out.x) ^ b(out.y).wrapping_mul(3) ^ b(out.z).wrapping_mul(7);
                }
                5 => {
                    ft_0d27::fn_800d2b30(gp);
                    hsh = hv((unsafe { &f.mv.co.common } as *const ftCommon_MotionVars_common).cast(), csz);
                }
                _ => {
                    ft_0d27::fn_800d2bf8(gp);
                    hsh = b(f.xE4_ground_accel_1) ^ b(f.xE8_ground_accel_2).wrapping_mul(3) ^ b(f.gr_vel).wrapping_mul(5)
                        ^ b(f.xF0_ground_kb_vel).wrapping_mul(7) ^ b(f.xF4_ground_attacker_shield_kb_vel).wrapping_mul(11)
                        ^ hvv(&f.x74_self_accel) ^ hvv(&f.self_vel).wrapping_mul(13) ^ hvv(&f.x8c_kb_vel).wrapping_mul(17)
                        ^ hvv(&f.x98_atk_shield_kb).wrapping_mul(19) ^ hvv(&f.xD4_unk_vel).wrapping_mul(23);
                }
            }
            let mut tr = unsafe { TR };
            if op == 2 {
                // the C trace has the AObj flags read (T(3)) after HSD_JObjAnim and ftLib_SetScale
                tr = tr.wrapping_mul(31).wrapping_add(3);
            }
            let got = format!("{op} {k} {ret:08X} {hsh:08X} {tr:08X}");
            assert_eq!(got, lines.next().unwrap(), "op {op} case {k}");
            n += 1;
        }
    }
    assert_eq!(n, 2100);
}

#[test]
fn smoke_stateful() {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    let gp: *mut HSD_GObj = &mut *g;
    // fall-state picker
    ft_0d27::fn_800d2890(gp, ftCo_MS_FallAerial as i32);
    assert!(unsafe { f.mv.co.unk_800D2890.x0 }.is_some());
    // no item: these callbacks do nothing
    assert!(!ft_0d27::ft_800d2d0c(gp));
    ft_0cd1::ft_800cd31c(gp);
    melee_fighter::tu::ft_0ce3::ftco_800ce620(gp);
    melee_fighter::tu::ft_0ce3::ftco_800ce650(gp);
    melee_fighter::tu::ft_0cdf::ftco_800ce1d4(gp);
    melee_fighter::tu::ft_0cd3::ft_800cd604(gp);
    melee_fighter::tu::ft_0cd3::ft_800cd65c(gp);
    melee_fighter::tu::ft_0ce3::ftco_fireflowershoot_anim(gp);
    // throw-flag inline helpers test-and-clear
    unsafe { f.anon_2210.anon_0.set_throw_flags_b3(1) };
    assert!(ft_0cd1::ft_check_throw_b3(&mut f));
    assert!(!ft_0cd1::ft_check_throw_b3(&mut f));
}

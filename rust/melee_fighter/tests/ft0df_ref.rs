//! Group 3 round 13 vs the C (gen/cref/ft0df_ref.c, cut verbatim, shipping contraction + gwtool
//! lowering; expected values in tests/data/ft0df.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ft0df`): ft_0DF0.c, ft_0DF1.c and the
//! ftcoll/ftdevice/ftparts leaves of src/ft0df.rs. Fill order matches the C driver exactly.
#![allow(static_mut_refs)]

use melee_fighter::ft0df as m;
use melee_fighter::hooks as h;
use melee_fighter::types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rg(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => f32::from_bits(0x7FC0_0000),
            3 => 1.0,
            4 => -1.0,
            _ => ((self.rnd() % 41) as i32 - 20) as f32 / 20.0,
        }
    }
}

static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
static mut G_GM: bool = false;
static mut G_IT: i32 = 0;
static mut G_P354: bool = false;
static mut GO_PTR: *mut HSD_GObj = core::ptr::null_mut();

struct Hs(u32);
impl Hs {
    fn m(&mut self, v: u32) {
        self.0 = self.0.wrapping_mul(0x0100_0193) ^ v;
    }
}

fn state_hash(f: &Fighter, gg: *mut HSD_GObj, hb: &HitCapsule) -> u32 {
    let a = &f.smash_attrs;
    let mut s = Hs(0x811C_9DC5);
    s.m(a.state as u32);
    s.m(a.x2118_frames.to_bits());
    s.m(a.x211C_holdFrame.to_bits());
    s.m(a.x2120_damageMul.to_bits());
    s.m(a.x2124_frameSpeedMul.to_bits());
    s.m(a.x2128 as u32);
    s.m(a.x212C as u32);
    s.m(a.x212D as u32);
    s.m(a.x2130_sfxBool as u32);
    s.m(f.shield_hit.skip_update_pos() as u32);
    s.m(f.reflect_hit.skip_update_pos() as u32);
    s.m(f.absorb_hit.skip_update_pos() as u32);
    for i in 0..15 {
        s.m(f.hurt_capsules[i].capsule.skip_update_pos() as u32);
    }
    let owner = unsafe { f.x1064_thrownHitbox.anon_134.owner };
    s.m((owner == gg) as u32);
    s.m(owner.is_null() as u32);
    s.m(f.x119C_teamUnk as u32);
    s.m(f.grabber_unk1 as u32);
    s.m(hb.jobj.is_null() as u32);
    s.m(hb.b_offset.x.to_bits());
    s.m(hb.b_offset.y.to_bits());
    s.m(hb.b_offset.z.to_bits());
    for i in 0..12 {
        s.m(f.x5F4_arr[i].prev as i32 as u32);
        s.m(f.x5F4_arr[i].idx as i32 as u32);
    }
    s.m(f.x221D_b2() as u32);
    s.0
}

#[test]
fn ft0df_matches_c() {
    unsafe {
        h::GM_8016B0FC = Some(|| { t(1); G_GM });
        h::IT_8026B30C = Some(|g| { t(2); t(!g.is_null() as u32); G_IT });
        h::FTANIM_SET_ANIM_RATE = Some(|g, r| { t(3); t((g == GO_PTR) as u32); t(r.to_bits()) });
        h::FTCO_800C0200 = Some(|_, i| { t(4); t(i as u32) });
        h::FTCO_800BFFD0 = Some(|_, k, b| { t(5); t(k as u32); t(b as u32); false });
        h::PLAYER_8003544C = Some(|s, u| { t(6); t(s as u32); t(u as u32); G_P354 });
        h::LB_RUMBLE_START = Some(|p, id, a, b| { t(7); t(p as u32); t(id as u32); t(a as u32); t(b as u32) });
        h::HSD_PAD_RUMBLE_REMOVE_ID = Some(|p, id| { t(8); t(p as u32); t(id as u32) });
        h::PLAY_SFX = Some(|_, id, vol, pan| { t(9); t(id as u32); t(vol as u8 as u32); t(pan as u8 as u32) });
    }
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut gf: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut gg: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut ig: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut jo: Box<HSD_JObj> = Box::new(HSD_JObj::zeroed());
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let mut shk: Box<[Vec2; 8]> = Box::new([Vec2 { x: 0.0, y: 0.0 }; 8]);
    let mut st: Box<Fighter_ShakeTable_t> = Box::new(Fighter_ShakeTable_t::zeroed());
    let gp: *mut HSD_GObj = &mut *go;
    let ggp: *mut HSD_GObj = &mut *gg;
    go.user_data = &mut *f as *mut Fighter as *mut core::ffi::c_void;
    gg.user_data = &mut *gf as *mut Fighter as *mut core::ffi::c_void;
    st.x0 = shk.as_mut_ptr();
    unsafe {
        GO_PTR = gp;
        h::P_FT_COMMON_DATA = &*cd;
        h::FIGHTER_SMASH_CHARGE_SHAKE_TABLE = &*st;
    }
    let expected = include_str!("data/ft0df.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..40u32 {
        for k in 0..600u32 {
            *f = Fighter::zeroed();
            *gf = Fighter::zeroed();
            *cd = ftCommonData::zeroed();
            let mut hb = HitCapsule::zeroed();
            unsafe { TR = 0 };
            let mut r = Rng(0xDF0u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            f.smash_attrs.state = (r.rnd() % 5) as _;
            f.smash_attrs.x2118_frames = r.rg();
            f.smash_attrs.x211C_holdFrame = r.rg();
            f.smash_attrs.x2120_damageMul = r.rg();
            f.smash_attrs.x2124_frameSpeedMul = r.rg();
            f.smash_attrs.x2128 = if r.rnd() & 1 != 0 { 0x7B } else { (r.rnd() % 200) as i32 };
            f.smash_attrs.x212C = (r.rnd() % 8) as u8;
            f.smash_attrs.x212D = (r.rnd() % 9) as u8;
            f.smash_attrs.x2130_sfxBool = (r.rnd() & 1) as i32;
            f.frame_speed_mul = r.rg();
            f.input.held_buttons[0] = r.rnd() & 0x300;
            f.input.cstick[0].x = r.rg();
            f.input.cstick[0].y = r.rg();
            f.input.cstick[1].x = r.rg();
            f.input.cstick[1].y = r.rg();
            f.facing_dir = r.rg();
            f.item_gobj = if r.rnd() & 1 != 0 { &mut *ig } else { core::ptr::null_mut() };
            unsafe {
                G_GM = r.rnd() & 1 != 0;
                G_IT = (r.rnd() % 3) as i32;
                G_P354 = r.rnd() & 1 != 0;
            }
            f.set_x221F_b3((r.rnd() & 1) as u8);
            f.set_x2224_b2((r.rnd() & 1) as u8);
            f.x618_player_id = (r.rnd() % 4) as u8;
            f.player_id = (r.rnd() % 4) as u8;
            f.set_is_sub_fighter((r.rnd() & 1) as u8);
            cd.dash_smash_stick_threshold = r.rg();
            cd.xCC = r.rg();
            cd.xD4 = r.rg();
            cd.xDC = r.rg();
            cd.xE0 = r.rg();
            cd.sdi_min_stick_mag = r.rg();
            cd.x7F4 = r.rg();
            cd.x248 = r.rg();
            cd.x20_radians = r.rg();
            cd.x7F8 = r.rg();
            cd.x7FC = r.rg();
            cd.x494 = r.rg();
            cd.x98 = r.rg();
            cd.attackhi3_stick_threshold_y = r.rg();
            cd.xB0 = r.rg();
            cd.x31C = r.rg();
            cd.x314 = r.rg();
            cd.tap_jump_threshold = r.rg();
            cd.x7C8 = r.rg();
            for i in 0..8 {
                shk[i].x = r.rg();
                shk[i].y = r.rg();
            }
            st.x4 = (r.rnd() % 9) as i32;
            f.x408.x28_colanim.i = if r.rnd() & 1 != 0 { 0 } else { r.rnd() as i32 };
            f.x488.x28_colanim.i = r.rnd() as i32;
            f.shield_hit.set_skip_update_pos((r.rnd() & 1) as u8);
            f.reflect_hit.set_skip_update_pos((r.rnd() & 1) as u8);
            f.absorb_hit.set_skip_update_pos((r.rnd() & 1) as u8);
            f.hurt_capsules_len = (r.rnd() % 16) as u8;
            for i in 0..15 {
                f.hurt_capsules[i].capsule.set_skip_update_pos((r.rnd() & 1) as u8);
            }
            gf.team = r.rnd() as u8;
            gf.player_id = r.rnd() as u8;
            hb.jobj = &mut *jo;
            let v = Vec3 { x: r.rg(), y: r.rg(), z: r.rg() };
            f.x5AC.model_num = r.rnd() % 13;
            for i in 0..12 {
                f.x5F4_arr[i].prev = r.rnd() as i8;
                f.x5F4_arr[i].idx = r.rnd() as i8;
            }
            f.set_x221D_b2((r.rnd() & 1) as u16);
            let (a, b, u) = (r.rg(), r.rg(), r.rnd());
            let i1 = (r.rnd() % 12) as i32;
            let i2 = r.rnd() as i32;
            let fr: &mut Fighter = unsafe { &mut *((*gp).user_data as *mut Fighter) };
            let which = |p: *mut ColorOverlay, f: &mut Fighter| -> u32 {
                if p == &mut f.x408 as *mut _ { 0 } else if p == &mut f.x488 as *mut _ { 1 } else { 2 }
            };
            let ret: u32 = match op {
                0 => { m::ftco_800dee84(gp, u, a, b); 0 }
                1 => { m::ftco_800deea8(gp); 0 }
                2 => m::ftco_800deeb8(fr, a).to_bits(),
                3 => match m::ftco_800deee8(fr) {
                    None => 0xFFFF_FFFF,
                    Some(s) => s.x.to_bits().wrapping_mul(31).wrapping_add(s.y.to_bits()) ^ 1,
                },
                4 => { m::ftco_800def38(gp); 0 }
                5 => { m::ftco_800df0d0(gp); 0 }
                6 => m::can_use_cstick(fr) as u32,
                7 => m::ftco_800df1c8(fr) as u32,
                8 => m::ftco_800df21c(fr) as u32,
                9 => m::ftco_800df2d8(fr) as u32,
                10 => m::ftco_800df30c(fr) as u32,
                11 => m::ftco_800df3a8(fr) as u32,
                12 => m::ftco_800df3dc(fr) as u32,
                13 => m::ftco_800df478(fr) as u32,
                14 => m::ftco_800df50c(fr) as u32,
                15 => m::ftco_800df608(fr) as u32,
                16 => m::ftco_800df644(fr) as u32,
                17 => m::ftco_800df678(fr) as u32,
                18 => m::ftco_800df6f8(fr) as u32,
                19 => m::ftco_800df72c(fr) as u32,
                20 => m::ftco_800df79c(fr) as u32,
                21 => m::ftco_800df7f4(fr) as u32,
                22 => m::ftco_800df844(fr) as u32,
                23 => m::ftco_800df878(fr) as u32,
                24 => m::ftco_800df8b0(fr) as u32,
                25 => m::ftco_800df8e8(fr) as u32,
                26 => m::ftco_800df910(fr) as u32,
                27 => { m::ftcoll_8007aee0(gp); 0 }
                28 => { m::ftcoll_8007aef8(gp); 0 }
                29 => { m::ftcoll_8007af10(gp); 0 }
                30 => { m::ftcoll_8007af28(gp); 0 }
                31 => { m::ftcoll_8007b8a8(&mut hb, &v); 0 }
                32 => { m::ftcoll_8007b8cc(fr, ggp); 0 }
                33 => { let p = m::ftco_800c0658(fr); which(p, fr) }
                34 => { let p = m::ftco_800c0674(gp); which(p, fr) }
                35 => m::ftco_800c0694(fr) as u32,
                36 => { m::ftparts_80074a4c(gp, i1, i2); 0 }
                37 => m::ftparts_80074a74(gp, i1) as u32,
                38 => { m::ftparts_80074a8c(gp); 0 }
                39 => { m::ftparts_80074acc(gp); 0 }
                _ => unreachable!(),
            };
            let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, state_hash(&f, ggp, &hb), unsafe { TR });
            let want = lines.next().expect("ft0df.txt too short");
            assert_eq!(got, want, "op {op} case {k}");
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "ft0df.txt has extra lines");
    assert_eq!(checked, 24000);
}

//! src/tu/ftCo_Damage.rs vs the C (gen/cref/tu/ftCo_Damage.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Damage`). Both Fighters
//! and the common data are filled word for word from the xorshift; the hash covers both Fighters.
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use core::ffi::c_void;
use m::ext::hook as hk;
use melee_fighter::tu::ftCo_Damage as m;
use melee_fighter::types::*;
use tu_ftco_harness::*;

/// Commutative trace event (see the C driver).
fn tc(v: u32) {
    unsafe { TR = TR.wrapping_add(v.wrapping_mul(0x9E37_79B1)) }
}
fn is_f(p: &Fighter) -> u32 {
    (p as *const Fighter == f() as *const Fighter) as u32
}
fn gk(g: *mut HSD_GObj) -> u32 {
    (g == go()) as u32 | ((g == go2()) as u32) << 1 | ((g == igo()) as u32) << 2
}
fn chk(id: u32) -> bool {
    unsafe { (SRV[(10 + id % 6) as usize] >> (id % 7)) & 1 != 0 }
}
fn bn(x: f32) -> u32 {
    if x.is_nan() { 0x7FC0_0000 } else { x.to_bits() }
}

macro_rules! dv {
    ($id:expr) => {
        Some(|g| {
            t($id);
            t(gk(g));
        })
    };
}
macro_rules! dchk {
    ($id:expr) => {
        Some(|g| {
            t($id);
            t(gk(g));
            chk($id)
        })
    };
}
macro_rules! dfv {
    ($id:expr) => {
        Some(|p| {
            t($id);
            t(is_f(p));
        })
    };
}

static mut D6524: *mut Fighter_804D6524_t = core::ptr::null_mut();
static mut SFXD: *mut FtSFX = core::ptr::null_mut();
static mut FTD: *mut ftData = core::ptr::null_mut();
static mut PARTS: *mut FighterBone = core::ptr::null_mut();
static JTB: [u8; 4] = [0; 4];
fn jtb() -> *mut HSD_JObj {
    JTB.as_ptr() as *mut HSD_JObj
}

unsafe extern "C" fn pre_cb(g: *mut HSD_GObj) {
    t(90);
    t(gk(g));
}

fn gf() -> f32 {
    match rnd() % 32 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(0x7FC0_0000),
        3 => 1.0,
        4 => -1.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 20.0,
    }
}
fn gw() -> u32 {
    match rnd() % 8 {
        0 | 1 | 2 => gf().to_bits(),
        3 => rw().to_bits(),
        4 => rnd() % 64,
        5 => 0,
        6 => rnd(),
        _ => (gf() * 3.0).to_bits(),
    }
}
fn fill_words<T>(p: *mut T) {
    let n = core::mem::size_of::<T>() / 4;
    let w = unsafe { core::slice::from_raw_parts_mut(p as *mut u32, n) };
    for x in w.iter_mut() {
        *x = gw();
    }
}
fn hw(w: u32) -> u32 {
    if (w & 0x7F80_0000) == 0x7F80_0000 && (w & 0x7F_FFFF) != 0 { 0x7FC0_0000 } else { w }
}
const MSL: [i32; 25] = [
    ftCo_MS_Squat,
    ftCo_MS_SquatWait,
    ftCo_MS_DamageIce,
    ftCo_MS_JumpF,
    ftCo_MS_JumpB,
    ftCo_MS_JumpAerialF,
    ftCo_MS_JumpAerialB,
    ftCo_MS_Fall,
    ftCo_MS_FallF,
    ftCo_MS_FallB,
    ftCo_MS_FallAerial,
    ftCo_MS_FallAerialF,
    ftCo_MS_FallAerialB,
    ftCo_MS_FallSpecial,
    ftCo_MS_FallSpecialF,
    ftCo_MS_FallSpecialB,
    ftCo_MS_DamageFall,
    ftCo_MS_EscapeAir,
    0xe0,
    0xe1,
    0xe3,
    0xe4,
    0x145,
    ftCo_MS_DamageFlyRoll,
    ftCo_MS_DamageFlyRoll,
];

fn fill_fighter(p: &mut Fighter, own: *mut HSD_GObj) {
    fill_words(p as *mut Fighter);
    unsafe {
        p.gobj = own;
        p.parts = PARTS;
        p.ft_data = FTD;
        p.victim_gobj = core::ptr::null_mut();
        p.item_gobj = core::ptr::null_mut();
        p.hitlag_cb = None;
        p.pre_hitlag_cb = None;
        p.post_hitlag_cb = None;
        p.x1064_thrownHitbox.x134.owner = core::ptr::null_mut();
        p.dmg.x190C = core::ptr::null_mut();
    }
    p.motion_id = if rnd() % 4 == 0 { (rnd() % 400) as _ } else { MSL[(rnd() % 25) as usize] as _ };
    p.ground_or_air = (rnd() & 1) as _;
    p.dmg.x1860_element = rnd() % 17;
    p.dmg.x184c_damaged_hurtbox = (rnd() % 3) as _;
    p.dmg.x1848_kb_angle = if rnd() & 1 != 0 { 361 } else { (rnd() % 400) as i32 - 20 };
    p.dmg.kb_applied = if rnd() % 5 == 0 { 0.0 } else { rw() };
    p.dmg.x1838_percentTemp = if rnd() % 4 == 0 { 0.0 } else { rw() };
    p.dmg.x18ac_time_since_hit = (rnd() % 12) as _;
    p.dmg.facing_dir_1 = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    p.facing_dir = if rnd() % 4 == 0 {
        rg()
    } else if rnd() & 1 != 0 {
        1.0
    } else {
        -1.0
    };
    p.pad_port = (rnd() % 4) as _;
    p.player_idx = (rnd() % 4) as _;
    p.active_timer.lstick.x = (rnd() % 8) as _;
    p.active_timer.lstick.y = (rnd() % 8) as _;
    p.active_sticky.lstick.x = (rnd() % 4) as _;
    p.active_sticky.lstick.y = (rnd() % 4) as _;
    p.x680 = (rnd() % 256) as _;
    p.x684 = (rnd() % 256) as _;
    p.smash_attrs.state = rnd() % 4;
    p.x34_scale.y = if rnd() & 1 != 0 { 1.0 } else { rg() };
    p.input.held_buttons[0] = if rnd() & 1 != 0 { HSD_PAD_LR } else { 0 };
    p.input.pressed_buttons = if rnd() & 1 != 0 { rnd() } else { 0 };
    p.x1828 = 0;
    p.set_allow_sdi((rnd() & 1) as _);
    p.set_is_metal((rnd() & 1) as _);
    p.set_is_sandbag((rnd() & 1) as _);
    p.set_is_sub_fighter((rnd() & 1) as _);
    p.set_x2219_b5((rnd() & 1) as _);
    p.set_x221A_b3((rnd() & 1) as _);
    p.x221B.set_x221B_b5((rnd() & 1) as _);
    p.set_x221C_b0((rnd() & 1) as _);
    p.set_x221C_b6((rnd() & 1) as _);
    p.set_x221D_b7((rnd() & 1) as _);
    p.set_x2220_b3((rnd() % 4 == 0) as u32 as _);
    p.set_x2220_b4((rnd() % 4 == 0) as u32 as _);
    p.set_x2221_b1((rnd() & 1) as _);
    p.set_x2222_b0((rnd() & 1) as _);
    p.set_stamina_dead((rnd() & 1) as _);
    p.set_x2227_b4((rnd() & 1) as _);
}

fn cb_code(c: Option<unsafe extern "C" fn(*mut HSD_GObj)>, own: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> u32 {
    if c.is_none() {
        0
    } else if c == own {
        1
    } else if c == Some(pre_cb as unsafe extern "C" fn(*mut HSD_GObj)) {
        2
    } else {
        3
    }
}

fn hash_fighter(h: &mut Hs, p: &Fighter) {
    let mut tmp: Box<Fighter> = Box::new(unsafe { core::ptr::read(p) });
    h.m(cb_code(p.hitlag_cb, Some(m::ftCo_Damage_OnEveryHitlag_cb)));
    h.m(cb_code(p.post_hitlag_cb, Some(m::ftCo_Damage_OnExitHitlag_cb)));
    h.m(cb_code(p.pre_hitlag_cb, None));
    h.m(p.dmg.x190C as usize as u32);
    unsafe {
        tmp.gobj = core::ptr::null_mut();
        tmp.parts = core::ptr::null_mut();
        tmp.ft_data = core::ptr::null_mut();
        tmp.victim_gobj = core::ptr::null_mut();
        tmp.item_gobj = core::ptr::null_mut();
        tmp.hitlag_cb = None;
        tmp.pre_hitlag_cb = None;
        tmp.post_hitlag_cb = None;
        tmp.x1064_thrownHitbox.x134.owner = core::ptr::null_mut();
        tmp.dmg.x190C = core::ptr::null_mut();
        let bf = core::mem::offset_of!(Fighter, bf_2218);
        core::ptr::write_bytes((&mut *tmp as *mut Fighter as *mut u8).add(bf), 0, 19);
    }
    h.m(p.allow_sdi() as u32);
    h.m(p.is_metal() as u32);
    h.m(p.is_sandbag() as u32);
    h.m(p.is_sub_fighter() as u32);
    h.m(p.x2219_b5() as u32);
    h.m(p.x221A_b3() as u32);
    h.m(p.x221B.x221B_b5() as u32);
    h.m(p.x221C_b0() as u32);
    h.m(p.x221C_b6() as u32);
    h.m(p.x221D_b7() as u32);
    h.m(p.x2220_b3() as u32);
    h.m(p.x2220_b4() as u32);
    h.m(p.x2221_b1() as u32);
    h.m(p.x2222_b0() as u32);
    h.m(p.stamina_dead() as u32);
    h.m(p.x2227_b4() as u32);
    let w = unsafe {
        core::slice::from_raw_parts(&*tmp as *const Fighter as *const u32, core::mem::size_of::<Fighter>() / 4)
    };
    for &x in w {
        h.m(hw(x));
    }
}

fn state_hash() -> u32 {
    let mut h = Hs::new();
    hash_fighter(&mut h, f());
    hash_fighter(&mut h, f2());
    h.0
}

fn hooks() {
    unsafe {
        hk::ScriptGame_ArmorResetReaction = Some(|p| {
            t(1);
            t(is_f(p));
        });
        hk::ScriptGame_ArmorSubtract = Some(|p, kb| {
            t(2);
            t(is_f(p));
            t(bn(kb));
            if rnd() & 1 != 0 { rg() } else { kb }
        });
        hk::ScriptGame_ArmorReact = Some(|p| {
            t(3);
            t(is_f(p));
            chk(3) as i32
        });
        hk::ScriptGame_CapsNoFlinch = Some(|p| {
            t(4);
            t(is_f(p));
            chk(4) as i32
        });
        hk::ScriptGame_ShockTake = Some(|p, base| {
            t(5);
            t(is_f(p));
            t(base as u32);
            (rnd() % 5) as i32 - 2
        });
        hk::Geno_TakeStunBonus = Some(|p| {
            t(6);
            t(is_f(p));
            (rnd() % 7) as i32 - 3
        });
        hk::diag_geno_stun = Some(|a, b, c| {
            t(7);
            t(a as u32);
            t(b as u32);
            t(c as u32);
        });
        hk::Mex_Enabled = Some(|_| {
            t(8);
            chk(8) as i32
        });
        hk::Replay_UcfVersion = Some(|p| {
            t(9);
            t(p as u32);
            if SRV[3] & 1 != 0 { 84 } else { 83 }
        });
        hk::ftUcf_PadRaw = Some(|p, w, bk| {
            t(10);
            t(p as u32);
            t(w as u32);
            t(bk as u32);
            (rnd() % 256) as i32 - 128
        });
        hk::ftColl_8007AC68 = Some(|a| {
            t(11);
            t(a);
            chk(11)
        });
        hk::ftCo_CalcYScaledKnockback = Some(|a, s, b2| {
            t(12);
            t(bn(a));
            t(bn(s));
            t(bn(b2));
            rg()
        });
        hk::ftCo_800BFFD0 = Some(|p, a, b2| {
            t(13);
            t(is_f(p));
            t(a as u32);
            t(b2 as u32);
            chk(13)
        });
        hk::lbBgFlash_80021C48 = Some(|a, b2| {
            t(14);
            t(a);
            t(b2);
        });
        hk::efAsync_Spawn = Some(|g, q, k, gfx, j, a| {
            t(15);
            t(gk(g));
            t((q == core::ptr::addr_of_mut!(f().x60C) as *mut c_void) as u32);
            t(k);
            t(gfx);
            t((j == jtb()) as u32);
            t(bn(*a));
        });
        hk::ftPartSetRotX = Some(|p, idx, v| {
            t(16);
            t(is_f(p));
            t(idx as u32);
            t(bn(v));
        });
        hk::ftParts_GetBoneIndex = Some(|p, part| {
            tc(17u32.wrapping_add(is_f(p).wrapping_mul(3)).wrapping_add((part as u32).wrapping_mul(131)));
            (((part as u32).wrapping_mul(7).wrapping_add(3)) % 20) as i32
        });
        hk::Fighter_8006CDA4 = Some(|p, v| {
            t(18);
            t(is_f(p));
            t(v as u32);
        });
        hk::pl_80040270 = Some(|a, b2, c| {
            t(19);
            t(a as u32);
            t(b2 as u32);
            t(bn(c));
        });
        hk::pl_800401F0 = Some(|a, b2, c, d| {
            t(20);
            t(a as u32);
            t(b2 as u32);
            t(bn(c));
            t(bn(d));
        });
        hk::lbVector_Angle = Some(|a, b2| {
            t(21);
            t((a as *const Vec3 == core::ptr::addr_of!(f().coll_data.floor.normal)) as u32);
            t(bn(a.x));
            t(bn(a.y));
            t(bn(b2.x));
            t(bn(b2.y));
            t(bn(b2.z));
            rg() * 2.0
        });
        hk::ftCommon_8007D5D4 = dfv!(22);
        hk::HSD_Randf = Some(|| {
            t(23);
            (rnd() % 1000) as f32 / 1000.0
        });
        hk::un_803222EC = Some(|a, b2| {
            t(24);
            t(bn(a));
            t(bn(b2));
            rg()
        });
        hk::un_8032233C = Some(|a, b2| {
            t(25);
            t(a);
            t(b2);
        });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, b2, c, o| {
            t(26);
            t(gk(g));
            t(ms as u32);
            t(fl);
            t(a.to_bits());
            t(b2.to_bits());
            t(c.to_bits());
            t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = dv!(27);
        hk::gm_8016B014 = Some(|| {
            t(28);
            chk(28)
        });
        hk::ftColl_8007B6A0 = dv!(29);
        hk::ftCommon_8007EFC0 = Some(|p, v| {
            t(30);
            t(is_f(p));
            t(v);
        });
        hk::Camera_RequestQuake = Some(|k, p| {
            t(31);
            t(k);
            t(bn(p.x));
            t(bn(p.y));
            t(bn(p.z));
        });
        hk::ftCommon_8007F824 = dv!(32);
        hk::ftCo_DamageIce_Init = dv!(33);
        hk::ftCo_800DF608 = Some(|p| {
            t(34);
            t(is_f(p));
            chk(34)
        });
        hk::ftColl_8007B7A4 = Some(|g, v| {
            t(35);
            t(gk(g));
            t(v as u32);
        });
        hk::ftCo_800C318C = Some(|g, v| {
            t(36);
            t(gk(g));
            t(v as u32);
        });
        hk::ftCo_800C8D00 = dv!(37);
        hk::ftCo_800DC284 = dv!(38);
        hk::ftCo_800DC3A4 = dv!(39);
        hk::ftCo_800C0408 = dv!(40);
        hk::ftCommon_800804FC = dfv!(41);
        hk::ftCo_800C3538 = dchk!(42);
        hk::ftCo_800C44CC = dchk!(43);
        hk::ftCo_800D2FA4 = dchk!(44);
        hk::ftCo_800DE854 = dv!(45);
        hk::ftCo_800DCE34 = Some(|a, b2| {
            t(46);
            t(gk(a));
            t(gk(b2));
        });
        hk::ftCommon_8007DB58 = dv!(47);
        hk::ftCommon_CalcHitlag = Some(|d, ms, mul| {
            t(48);
            t(d as u32);
            t(ms as u32);
            t(bn(mul));
            rg() * 10.0
        });
        hk::ftCo_800DE2F0 = dv!(49);
        hk::ftCo_800DCFD4 = dv!(50);
        hk::it_8026B2D8 = Some(|g| {
            t(51);
            t((g == igo()) as u32);
            chk(51)
        });
        hk::ftDk_MS_349_800E06D8 = dv!(52);
        hk::ftCo_800C0CB8 = dchk!(53);
        hk::ftCo_DamageIce_HitWhileFrozen = dv!(54);
        hk::ftCo_800C74F4 = dchk!(55);
        hk::ftCo_800C3598 = dv!(56);
        hk::ftCommon_8007F86C = dv!(57);
        hk::ftCo_800C5240 = dchk!(58);
        hk::ftCo_800C554C = dfv!(59);
        hk::ftCo_800CB870 = dchk!(60);
        hk::ftAnim_IsFramesRemaining = dchk!(61);
        hk::ftCo_80090780 = dv!(62);
        hk::ftCo_Fall_Enter = dv!(63);
        hk::ft_8008A2BC = dv!(64);
        hk::ftCo_800D69C4 = dchk!(65);
        hk::ftCo_800CB8E0 = dchk!(66);
        hk::ftCommon_8007EBAC = Some(|p, a, b2| {
            t(67);
            t(is_f(p));
            t(a);
            t(b2);
        });
        hk::ftCo_Jump_GetInput = Some(|g| {
            t(68);
            t(gk(g));
            if chk(68) { 1 + rnd() % 3 } else { 0 }
        });
        hk::ftCo_HammerFall_IASA = dv!(69);
        hk::ftCo_HammerWait_IASA = dv!(70);
        hk::ftCo_Fall_IASA_Inner = dchk!(71);
        hk::ftCo_Wait_IASA = dv!(72);
        hk::ft_80084DB0 = dv!(73);
        hk::ft_80084EEC = dv!(74);
        hk::ft_80084F3C = dv!(75);
        hk::ft_800848DC = Some(|g, cb| {
            t(76);
            t(gk(g));
            t((fnsym::same(cb as usize, m::ftCo_8008FC94 as usize)) as u32);
        });
        hk::ft_80081DD4 = dchk!(77);
        hk::ftCo_80097D40 = dv!(78);
        hk::ftCo_Landing_Enter_Basic = dv!(79);
        hk::ftCommon_8007D7FC = dfv!(80);
        hk::ftColl_8007AFF8 = dv!(81);
        hk::ftCo_800C1D38 = dchk!(82);
        hk::ftCo_800C23A0 = dchk!(83);
        hk::ftCo_800C17CC = dchk!(84);
        hk::ftCo_80098928 = dchk!(85);
        hk::ftCo_8009872C = dchk!(86);
        hk::ftCo_8009F0F0 = dchk!(87);
        hk::ftCo_DamageFall_IASA = dv!(88);
        hk::atan2f = Some(|y, x| {
            let h = bn(y).wrapping_mul(0x45D9F3B) ^ bn(x).wrapping_mul(0x27D4EB2F);
            tc(91u32.wrapping_add(h));
            (((h >> 8) % 41) as i32 - 20) as f32 / 20.0
        });
        hk::PSVECCrossProduct = Some(|a, b2| {
            t(89);
            t(bn(a.x));
            t(bn(a.y));
            t(bn(b2.x));
            t(bn(b2.y));
            Vec3 { x: 0.0, y: 0.0, z: rg() }
        });
        hk::Fighter_804D6524_x0 = Some(|| (*D6524).x0);
    }
}

#[test]
fn ftCo_Damage_matches_c() {
    init();
    unsafe {
        D6524 = Box::leak(Box::new(core::mem::zeroed()));
        SFXD = Box::leak(Box::new(core::mem::zeroed()));
        FTD = Box::leak(Box::new(core::mem::zeroed()));
        let parts: &mut [FighterBone; 8] = Box::leak(Box::new(core::mem::zeroed()));
        PARTS = parts.as_mut_ptr();
        (*PARTS).joint = jtb();
        (*FTD).x4C_sfx = SFXD;
    }
    hooks();
    let mut c = Check::new(include_str!("data/tu_ftCo_Damage.txt"));
    const NOPS: u32 = 36;
    for op in 0..NOPS {
        let n = if op == 17 {
            6000
        } else if op == 8 {
            4000
        } else {
            800
        };
        for k in 0..n {
            case(op, k);
            unsafe {
                fill_words(cd() as *mut ftCommonData);
                fill_words(D6524);
                fill_words(SFXD);
            }
            fill_fighter(f(), go());
            fill_fighter(f2(), go2());
            f().gobj = go();
            f2().gobj = go2();
            if rnd() & 1 != 0 {
                f().victim_gobj = go2();
            }
            if rnd() % 3 == 0 {
                f().item_gobj = igo() as *mut _;
            }
            if rnd() & 1 != 0 {
                f().x1064_thrownHitbox.x134.owner = go2();
            }
            if rnd() & 1 != 0 {
                f2().pre_hitlag_cb = Some(pre_cb);
            }
            if op != 5 && op != 8 && f().dmg.x1838_percentTemp == 0.0 {
                f().dmg.x1838_percentTemp = 1.0;
            }
            let g = go();
            let ret: u32 = match op {
                0 => bn(m::ftCo_Damage_CalcAngle(f(), rw())),
                1 => bn(m::ftCo_ScaleBy154(rw())),
                2 => m::ftCo_8008D8E8(rw()) as u32,
                3 => {
                    m::ftCo_Damage_CalcKnockback(f());
                    0
                }
                4 => {
                    let el = rnd() % 17;
                    let a = rnd() % 4;
                    m::ftCo_8008DA4C(g, el, a as i32) as u32
                }
                5 => {
                    let el = rnd() % 17;
                    let v = rw();
                    m::ftCo_8008DB10(g, el as i32, v);
                    0
                }
                6 => {
                    let x = rw();
                    let y = rw();
                    m::ftCo_Damage_CalcVel(f(), x, y);
                    0
                }
                7 => bn(m::calcAngle(rw())),
                8 => {
                    let a1 = if rnd() & 1 != 0 { -1 } else { (rnd() % 0x150) as i32 };
                    let fd = if rnd() & 1 != 0 { 0.0 } else { rg() };
                    m::ftCo_8008DCE0(g, a1, fd);
                    0
                }
                9 => m::ftCo_Damage_CheckAirMotion(f()) as u32,
                10 => {
                    m::ftCo_Damage_OnEveryHitlag(g);
                    0
                }
                11 => {
                    m::ftCo_8008E5A4(f());
                    0
                }
                12 => {
                    m::ftCo_Damage_OnExitHitlag(g);
                    0
                }
                13 => {
                    let fd = rg();
                    m::ftCo_8008E908(g, fd);
                    0
                }
                14 => m::ftCo_8008E984(f()) as u32,
                15 => {
                    m::ftCo_8008E9D0(g);
                    0
                }
                16 => {
                    m::ftCo_8008EB58(g);
                    0
                }
                17 => {
                    m::ftCo_8008EC90(g);
                    0
                }
                18 => {
                    m::ftCo_8008F744(g);
                    0
                }
                19 => {
                    m::ftCo_Damage_Anim(g);
                    0
                }
                20 => m::doIasa(g) as u32,
                21 => {
                    m::ftCo_Damage_IASA(g);
                    0
                }
                22 => {
                    m::ftCo_Damage_Phys(g);
                    0
                }
                23 => {
                    m::ftCo_Damage_Coll(g);
                    0
                }
                24 => {
                    m::ftCo_8008FC94(g);
                    0
                }
                25 => {
                    m::ftCo_Damage_SetMv8FromKbThreshold(f());
                    0
                }
                26 => {
                    m::ftCo_DamageFly_Anim(g);
                    0
                }
                27 => {
                    m::ftCo_DamageFly_IASA(g);
                    0
                }
                28 => {
                    m::ftCo_DamageFly_Phys(g);
                    0
                }
                29 => {
                    m::ftCo_DamageFly_Coll(g);
                    0
                }
                30 => {
                    m::ftCo_80090184(g);
                    0
                }
                31 => {
                    m::ftCo_DamageFlyRoll_Anim(g);
                    0
                }
                32 => {
                    m::ftCo_DamageFlyRoll_IASA(g);
                    0
                }
                33 => {
                    m::ftCo_DamageFlyRoll_Phys(g);
                    0
                }
                34 => {
                    m::ftCo_DamageFlyRoll_Coll(g);
                    0
                }
                _ => m::ftCo_Ucf084Sdi(f()) as u32,
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 37200);
}

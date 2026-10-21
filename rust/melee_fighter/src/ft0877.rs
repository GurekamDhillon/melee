//! Hand port, round 5: all of src/melee/ft/ft_0877.c (34 functions): fighter input / flag
//! queries, the CPU kind/level setters, the fighter-kind mask, sfx id adjustment (size
//! modifier, metal, Ice Climbers costume banks, m-ex relative ids) and `ft_PlaySFX`.
//! Signatures keep the C shape: `G` is the fighter gobj (`gobj->user_data` is the Fighter).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0877.c`): only ft_80087B34, five
//! f32 fmuladd, each `x*x + y*y` -> `fmadds(x, x, y*y)`. Calls out of the ported set go through
//! `crate::hooks` (lbAudioAx: group 1; itIsHeavy / itGetKind: group 5; ftCo: group 3 states).
//! TARGET_PC paths ported: m-ex 5000..9999 sfx ids in 80087D0C, Geno voice owner in PlaySFX.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0877_ref.rs.

use crate::ftcommon2 as c2;
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

pub fn ft_800877f8(g: G, a: i32) -> bool {
    fp(g).input.held_buttons[0] & a as u32 != 0
}
pub fn ft_80087818(g: G, a: i32) -> bool {
    fp(g).input.pressed_buttons & a as u32 != 0
}
pub fn ft_80087838(g: G) -> bool {
    fp(g).x221D_b7() & 1 != 0
}
pub fn ft_80087858(g: G) -> bool {
    fp(g).x2227_b1() & 1 != 0
}
pub fn ft_80087878(g: G, a: i32) -> bool {
    let f = fp(g);
    f.x1968_jumpsUsed <= 1 && f.motion_id == 0 && a != 0x23 && a != 0x24 && a != 0x25
}
pub fn ft_800878bc(g: G) -> bool {
    let f = fp(g);
    !f.item_gobj.is_null() && h::it_is_heavy(f.item_gobj)
}
pub fn ft_80087900(g: G) -> bool {
    let f = fp(g);
    !f.item_gobj.is_null() && h::it_get_kind(f.item_gobj) == It_Kind_ScBall
}
pub fn ft_80087944(g: G) -> bool {
    let f = fp(g);
    !f.item_gobj.is_null() && h::it_get_kind(f.item_gobj) == It_Kind_Dosei
}
/// Calls ftGetParasolStatus twice when the first is not 4, as the C does.
pub fn ft_80087988(g: G) -> bool {
    h::ft_get_parasol_status(g) == 4 || h::ft_get_parasol_status(g) == 5
}
pub fn ft_800879d8(g: G) -> bool {
    !fp(g).x1980.is_null()
}
pub fn ft_800879f8(g: G) -> bool {
    fp(g).x221D_b6() & 1 != 0
}
pub fn ft_80087a18(g: G) -> bool {
    let f = fp(g);
    if f.x2226_b4() == 0 {
        return false;
    }
    let v = h::ftco_800c06b4(f);
    (v == 0x7B || v == 0x80) && f.x2226_b5() != 0
}
pub fn ft_80087a80(g: G) -> i32 {
    fp(g).smash_attrs.x2134_vibrateFrame as i32
}
pub fn ft_80087a8c(g: G) -> f32 {
    fp(g).smash_attrs.x2138_smashSinceHitbox
}
pub fn ft_80087a98(g: G) -> i32 {
    fp(g).x213C
}
pub fn ft_80087aa4(g: G) -> bool {
    fp(g).x2227_b5() != 0
}
pub fn ft_80087ab4(g: G) -> u8 {
    fp(g).x1968_jumpsUsed
}
pub fn ft_80087ac0(g: G, a: i32) {
    fp(g).set_invisible((a == 0) as u8);
}
pub fn ft_80087aec(g: G) -> bool {
    let f = fp(g);
    f.x34_scale.y != f.x34_scale.x || f.x2226_b4() != 0 || f.is_metal() != 0 || !f.x197C.is_null()
}
/// 0: self velocity is the largest, 1: knockback, 2: shield knockback (ties go to 2).
pub fn ft_80087b34(g: G) -> i32 {
    let f = fp(g);
    let sx = f.x98_atk_shield_kb.x;
    let sy = f.x98_atk_shield_kb.y;
    let v1 = fmadds(f.self_vel.x, f.self_vel.x, f.self_vel.y * f.self_vel.y);
    let v2 = fmadds(sx, sx, sy * sy);
    let kb = |f: &Fighter| fmadds(f.x8c_kb_vel.x, f.x8c_kb_vel.x, f.x8c_kb_vel.y * f.x8c_kb_vel.y);
    if v1 > kb(f) {
        if v1 > fmadds(sx, sx, f.x98_atk_shield_kb.y * f.x98_atk_shield_kb.y) {
            return 0;
        }
        return 2;
    }
    if kb(f) > v2 {
        return 1;
    }
    2
}
pub fn ft_80087bac(g: G, a: i32) {
    fp(g).set_x2228_b5(a as u8);
}
pub fn ft_80087bc0(g: G, a: i32) {
    let f = fp(g);
    let (l, x) = (f.cpu.level, f.cpu.x14);
    h::ftco_800a101c(f, a, l, x);
}
pub fn ft_80087bec(g: G, a: i32) {
    let f = fp(g);
    let (k, x) = (f.cpu.kind as i32, f.cpu.x14);
    h::ftco_800a101c(f, k, a, x);
}
/// `ft_GetFtKindMask`: one bit per fighter kind below Master Hand in the fighter list.
pub fn get_ft_kind_mask() -> i32 {
    let mut r: u32 = 0;
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        let k = fp(cur).kind;
        if k < Ft_Kind_MasterH {
            r |= 1u32.wrapping_shl(k as u32);
        }
        cur = unsafe { (*cur).next };
    }
    r as i32
}
pub fn ft_80087c58(g: G) -> i32 {
    fp(g).x1988
}
pub fn ft_80087c64(g: G, a: i32) {
    fp(g).x1988 = a;
}
/// `Player_AdjustSFXIDForSizeModifier`.
pub fn ft_80087c70(f: &Fighter, a: i32) -> i32 {
    let mut r = a;
    match f.x2228_b3() {
        0 => {
            if f.x2220_b5() != 0 {
                r += 1;
            } else if f.x2220_b6() != 0 {
                r += 2;
            }
        }
        1 => {
            if f.x2220_b5() == 0 {
                r += 2;
            }
        }
        2 => {
            if f.x2220_b5() != 0 || f.x2220_b6() == 0 {
                r += 1;
            }
        }
        _ => {}
    }
    r
}

const SFX_METAL: [i32; 25] = [
    407, 467, 503, 515, 290051, 509, 485, 491, 479, 473, 437, 455, 461, 449, 443, 425, 431, 419,
    413, 377, 395, 401, 389, 383, 371,
];
const SFX_SIZE: [i32; 13] = [359, 368, 365, 362, 344, 353, 356, 350, 347, 338, 341, 335, 332];

pub fn ft_80087d0c(f: &Fighter, mut sfx_id: i32) -> i32 {
    if (5000..10000).contains(&sfx_id) {
        let ssm = h::mex_ssm_for_port_kind(f.kind as i32);
        if ssm >= 0 {
            sfx_id = sfx_id.wrapping_sub(5000).wrapping_add(ssm.wrapping_mul(10000));
        }
    }
    let mut sfx = h::lbaudioax_800233ec(sfx_id);
    let ssm_id = h::lbaudioax_80023130(sfx);
    let banked = |f: &Fighter, sfx: i32| -> i32 {
        let mut off = 0;
        let mut unused = 0;
        if h::lbaudioax_800230c8(ssm_id, &mut off, &mut unused) == 0
            && sfx >= off.wrapping_add(h::lbaudioax_80023220(ssm_id))
        {
            return ft_80087c70(f, sfx);
        }
        sfx
    };
    match ssm_id {
        0 => {
            if SFX_METAL.contains(&sfx) {
                let mut s = ft_80087c70(f, sfx);
                if f.is_metal() != 0 {
                    s += 3;
                }
                sfx = s;
            } else if SFX_SIZE.contains(&sfx) {
                sfx = ft_80087c70(f, sfx);
            }
        }
        13 => {
            if f.kind == Ft_Kind_Popo || f.kind == Ft_Kind_Nana {
                if (0x1FBFD..=0x1FC62).contains(&sfx) && c2::ftcommon_80080144(f) {
                    sfx += 0x66;
                }
                if (0x1FC63..=0x1FCC8).contains(&sfx) && !c2::ftcommon_80080144(f) {
                    sfx -= 0x66;
                }
            }
            sfx = banked(f, sfx); // C falls through into case 6..33
        }
        6..=12 | 14..=31 | 33 => sfx = banked(f, sfx),
        _ => {}
    }
    sfx
}
pub fn ft_8008805c(f: &Fighter, a: i32) {
    h::lbaudioax_80024dc4(ft_80087d0c(f, a));
}
pub fn ft_80088080(f: &mut Fighter) {
    f.x2164 = f.x2164.wrapping_add(1);
    h::lbaudioax_80024fdc();
}
pub fn ft_800880ac(f: &mut Fighter) {
    f.x2168 = f.x2168.wrapping_add(1);
    h::lbaudioax_80024ff4();
}
pub fn ft_800880d8(f: &mut Fighter) {
    h::lbaudioax_8002500c(f.x2164);
    f.x2164 = 0;
}
pub fn ft_80088110(f: &mut Fighter) {
    h::lbaudioax_80025038(f.x2168);
    f.x2168 = 0;
}
/// `ft_PlaySFX(fp, sfx_id, vol, pan)`.
pub fn play_sfx(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    if f.kind >= Ft_Kind_Mex0 && h::geno_define_voice_play_owner(f.kind as i32, sfx_id, f as *mut Fighter) != 0 {
        f.x2160 = -1;
        return;
    }
    let sfx_id = ft_80087d0c(f, sfx_id);
    f.x2160 = h::lbaudioax_800237a8(sfx_id, vol as i32, pan as i32);
    let rand = if (332..=370).contains(&sfx_id) { h::hsd_randi(200) - 100 } else { 0 };
    h::lbaudioax_80024b94(f.x2160, rand);
}

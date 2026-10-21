//! Item kinds, port round 6 (night 4), hand-ported part: per-kind functions whose bodies touch
//! only the Item, its `ItemAttr`, its kind's special attributes and item vars (or a plain struct
//! argument), and call only functions already ported. The one-statement forwarders and the
//! `return true;` / `return;` callbacks of this round are generated in `itfwd6.rs`. Each
//! function cites the C it mirrors; all are verified against verbatim C copies in
//! tests/c_oracle/round6_bodies.h (tests/oracle_round6.rs).
//!
//! Conventions as in `itkinds5.rs`: the `Item_GObj*` becomes the `Item`; a C `gobj != NULL`
//! guard becomes an `Option`; reading `xCC_item_attr` or the special attributes makes a function
//! `unsafe`; the `it_804D6D28` global is `&ItemCommonData`. File-local `static inline` helpers
//! are ported too (they are what the callers in later rounds inline).
//!
//! Fused multiply-adds (`tools/rust-port/fma_sites.py`, shipping contraction), all f32
//! (`fmadds`): `it_8028EDBC` 1, `it_802874F0` 3, `it_802962E0` 4, `test_comp` 3. Every other
//! function here has none. `M_PI` / `M_TAU` are `double` macros, so the expressions that use
//! them are evaluated in f64 (`it_2725_Logic101_Reflected`, `itMaril_UnkMotion1_Coll_inline`).
#![allow(non_snake_case)]

use crate::helpers::it_link_lerp;
use crate::itlib::{it_80271534, it_8026b3a8, it_8026b894, it_80274d6c, it_80274ecc, it_8027572c};
use crate::itlib3::{it_80272860, it_80274658, it_80274d04, it_802756e0, it_8027a344};
use crate::itkinds::itGShell_Attrs;
use crate::layout::{
    ftCollisionBox, itLinkHookshotAttributes, itMarilAttributes, itSScopeAttributes,
    itSword_UnkArticle1, itTaruAttributes, itTaruCann_DatAttrs, itToolsAttributes,
    itUnkAttributes, itWstarAttributes, Collide_LeftWallMask, Collide_RightWallMask, HSD_GObj,
    HurtCapsule, Item, ItemCommonData, ItemLink, Vec as GcVec, It_Kind_L_Gun_Ray,
};
use melee_core::fp::fmadds;
use std::ptr::null_mut;

const M_PI: f64 = 3.14159265358979323846;
const M_TAU: f64 = 6.283185307179586;

/// The kind's special attributes as a `T`.
/// # Safety
/// The article data and its special attributes must be valid as a `T` for `'a`.
unsafe fn sa<'a, T>(ip: &Item) -> &'a T {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const T)
}

/// `inline_fabsf` / the C `ABS(x)`: `x < 0 ? -x : x` (keeps the sign of a zero).
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// ---- item.c / ithitbox.c / maplib -----------------------------------------------------------

/// `it_8027574C` (ithitbox.c): `it_80274D04(gobj, &ip->xB54); it_80274D6C(gobj)`.
pub fn it_8027574c(ip: &mut Item) {
    let mut out = ip.xB54;
    it_80274d04(ip, &mut out);
    ip.xB54 = out;
    it_80274d6c(ip);
}

/// `it_802762B0` (itmaplib.c): `ground_or_air = GA_Ground`.
pub fn it_802762b0(ip: &mut Item) {
    ip.ground_or_air = 0;
}

/// `it_8026F3D4_check_kind` (itdrop.c, static inline): `kind >= It_Kind_L_Gun_Ray`.
pub fn it_8026f3d4_check_kind(_ip: Option<&Item>, it_kind: i32) -> bool {
    it_kind >= It_Kind_L_Gun_Ray
}

// ---- reference clears ------------------------------------------------------------------------

/// `it_2725_Logic70_EvtUnk` (itclimbersstring.c): `it_8026B894`, then forget `xC` if it is the
/// reference.
pub fn it_2725_logic70_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        it_8026b894(ip, ref_gobj);
        if ip.xDD4_itemVar.climbersstring.xC == ref_gobj {
            ip.xDD4_itemVar.climbersstring.xC = null_mut();
        }
    }
}

/// `it_2725_Logic59_EvtUnk` (itnessyoyo.c): `it_8026B894`; the volatile read after it has no
/// effect.
pub fn it_2725_logic59_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
}

/// `itCLinkMilk_Destroy` (itclinkmilk.c, static inline): clear `clinkmilk.x0` and `owner`.
pub fn it_clink_milk_destroy(ip: Option<&mut Item>) -> bool {
    match ip {
        Some(ip) => {
            ip.xDD4_itemVar.clinkmilk.x0 = null_mut();
            ip.owner = null_mut();
            true
        }
        None => false,
    }
}

/// `itNessbat_RemoveItem` (itnessbat.c, static inline): clear `nessbat.x0` and `owner`.
pub fn it_nessbat_remove_item(ip: Option<&mut Item>) -> bool {
    match ip {
        Some(ip) => {
            ip.xDD4_itemVar.nessbat.x0 = null_mut();
            ip.owner = null_mut();
            true
        }
        None => false,
    }
}

/// `it_802DB358` (itlikelike.c): remember the grab victim, clear `xDD0_flag.b5`,
/// `it_80274ECC(gobj, true)`.
pub fn it_802db358(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        ip.xDD4_itemVar.likelike.x50 = ip.grab_victim;
        ip.xDD0_flag.anon_0.set_b5(0);
        it_80274ecc(ip, 1);
    }
}

/// `it_802AE608` (itfoxblaster.c): if `xDD8 != 0`, `xDD8 = 3; xDDC = -1`.
pub fn it_802ae608(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let fb = &mut ip.xDD4_itemVar.foxblaster;
        if fb.xDD8 != 0 {
            fb.xDD8 = 3;
            fb.xDDC = -1;
        }
    }
}

/// `itFoxBlaster_SetShotOffset` (itfoxblaster.c, static inline): `xE14[0] = muzzle - pos`.
pub fn it_fox_blaster_set_shot_offset(ip: &mut Item, fighter_pos: &GcVec, muzzle: &GcVec) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let v = &mut ip.xDD4_itemVar.foxblaster.xE14[0];
        v.x = muzzle.x - fighter_pos.x;
        v.y = muzzle.y - fighter_pos.y;
        v.z = muzzle.z - fighter_pos.z;
    }
}

/// `itFoxBlaster_SetCommandVars` (itfoxblaster.c, static inline): all four command vars.
pub fn it_fox_blaster_set_command_vars(ip: &mut Item, value: i32) {
    let v = value as u32;
    ip.xDB8_itcmd_var3 = v;
    ip.xDB4_itcmd_var2 = v;
    ip.xDB0_itcmd_var1 = v;
    ip.xDAC_itcmd_var0 = v;
}

/// `itFoxBlaster_ClearShot` (itfoxblaster.c, static inline): zero shot `index`.
pub fn it_fox_blaster_clear_shot(ip: &mut Item, index: i32) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let fb = &mut ip.xDD4_itemVar.foxblaster;
        let i = index as usize;
        fb.xDE4[i] = 0;
        fb.xDFC[i] = 0;
        fb.xE14[i].z = 0.0;
        fb.xE14[i].y = 0.0;
        fb.xE14[i].x = 0.0;
        fb.angle[i] = 0.0;
    }
}

/// `itNesspkthunderball_ShiftAngles` (static inline): `angles[i + 1] = angles[i]`, i = 14..0.
pub fn it_nesspkthunderball_shift_angles(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let a = &mut ip.xDD4_itemVar.pkthunder.angles;
        for i in (0..15).rev() {
            a[i + 1] = a[i];
        }
    }
}

/// `itNesspkthunderball_ShiftPositions` (static inline): as the angles, for `positions`.
pub fn it_nesspkthunderball_shift_positions(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let a = &mut ip.xDD4_itemVar.pkthunder.positions;
        for i in (0..15).rev() {
            a[i + 1] = a[i];
        }
    }
}

// ---- physics ---------------------------------------------------------------------------------

/// `itDrMarioPill_UnkMotion0_Phys` (itdrmariopill.c, static): clear x15, fall, `it_80274658`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_dr_mario_pill_unk_motion0_phys(ip: &mut Item, common: &ItemCommonData) {
    it_8026b3a8(ip);
    let a = &*ip.xCC_item_attr;
    it_80272860(ip, a.x10_fall_speed, a.x14_fall_speed_max);
    it_80274658(ip, common.x68_float);
}

/// `itTogepy_UnkMotion6_Phys` (ittogepy.c): fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_togepy_unk_motion6_phys(ip: &mut Item) {
    if ip.ground_or_air == 1 {
        let a = &*ip.xCC_item_attr;
        it_80272860(ip, a.x10_fall_speed, a.x14_fall_speed_max);
    }
}

/// `itTools_UnkMotion4_Phys` (ittools.c): fall with motion `tools.x0`'s speeds.
/// # Safety
/// The special attributes must be a valid `itToolsAttributes` with motion `tools.x0`.
pub unsafe fn it_tools_unk_motion4_phys(ip: &mut Item) {
    let attrs = sa::<itToolsAttributes>(ip);
    let m = &*attrs.motions.as_ptr().offset(ip.xDD4_itemVar.tools.x0 as isize);
    it_80272860(ip, m.x4, m.x8);
}

/// `itHassam_UnkMotion1_Phys` (ithassam.c): `it_8027A344`, gravity on `x5C.y` clamped at
/// `-x10` while airborne, then `vel += x5C`.
/// # Safety
/// The special attributes must be a valid `itHassam_ItemVars` (sic, as the C).
pub unsafe fn it_hassam_unk_motion1_phys(ip: &mut Item) {
    let attr = sa::<crate::layout::itHassam_ItemVars>(ip);
    it_8027a344(ip);
    if ip.ground_or_air == 1 {
        let h = &mut ip.xDD4_itemVar.hassam;
        h.x5C.y -= attr.x14;
        if h.x5C.y < -attr.x10 {
            h.x5C.y = -attr.x10;
        }
    }
    let d = ip.xDD4_itemVar.hassam.x5C;
    ip.x40_vel.x += d.x;
    ip.x40_vel.y += d.y;
    ip.x40_vel.z += d.z;
}

/// `it_8028EDBC` (itfreeze.c): ground speed of the freezie (1 fused site).
/// # Safety
/// The special attributes must be a valid `itUnkAttributes`.
pub unsafe fn it_8028edbc(ip: &mut Item) {
    let attrs = sa::<itUnkAttributes>(ip);
    if ip.ground_or_air != 1 {
        let fr = &mut ip.xDD4_itemVar.freeze;
        if c_abs(fr.x4.x) > 0.1 {
            if fr.x0 == 0.0 {
                ip.facing_dir = if fr.x4.x > 0.0 { 1.0 } else { -1.0 };
                fr.x0 = attrs.x4_float * ip.facing_dir;
            } else {
                fr.x0 = fmadds(fr.x4.x, attrs.xC, fr.x0);
            }
        } else if c_abs(fr.x0) < attrs.x4_float {
            fr.x0 = attrs.x4_float * ip.facing_dir;
        } else if fr.x0 > 0.0 {
            fr.x0 -= attrs.x10;
        } else {
            fr.x0 += attrs.x10;
        }
        if fr.x0 > attrs.x8 {
            fr.x0 = attrs.x8;
        } else if fr.x0 < -attrs.x8 {
            fr.x0 = -attrs.x8;
        }
        if (fr.x0 > 0.0 && ip.facing_dir < 0.0) || (fr.x0 < 0.0 && ip.facing_dir > 0.0) {
            ip.facing_dir *= -1.0;
        }
        ip.x40_vel.x = fr.x0 * fr.x4.y;
    }
}

/// `it_802874F0` (ittaru.c): barrel roll speed and spin (3 fused sites).
/// # Safety
/// The special attributes must be a valid `itTaruAttributes`.
pub unsafe fn it_802874f0(ip: &mut Item) {
    let attr = sa::<itTaruAttributes>(ip);
    let t = &mut ip.xDD4_itemVar.taru;
    if ip.ground_or_air != 1 {
        let d = t.xDE8.x;
        if c_abs(d) > 0.1 {
            if c_abs(t.xDE4) < attr.x2C {
                t.xDE4 = fmadds(d, attr.x24, t.xDE4);
            }
        } else {
            let f1 = t.xDE4;
            let f2 = attr.x28;
            if c_abs(f1) < f2 {
                t.xDE4 = 0.0;
            } else if t.xDE4 > 0.0 {
                t.xDE4 -= f2;
            } else {
                t.xDE4 += f2;
            }
        }
        let s = t.xDE4;
        if (s > 0.0 && ip.facing_dir < 0.0) || (s < 0.0 && ip.facing_dir > 0.0) {
            ip.facing_dir *= -1.0;
        }
        let m = attr.x20;
        if c_abs(t.xDE4) > m {
            t.xDE4 = m * ip.facing_dir;
        }
        ip.x40_vel.x = t.xDE4 * t.xDE8.y;
        let v2 = c_abs(t.xDE4);
        t.xDE0 = fmadds(0.12217305f32 * v2, -ip.facing_dir, t.xDE0);
    } else {
        t.xDE0 = fmadds(0.10471976f32, -ip.facing_dir, t.xDE0);
    }
}

/// `it_802962E0` (ittarucann.c): barrel cannon roll speed and spin (4 fused sites).
/// # Safety
/// The special attributes must be a valid `itTaruCann_DatAttrs`.
pub unsafe fn it_802962e0(ip: &mut Item) {
    let da = sa::<itTaruCann_DatAttrs>(ip);
    let t = &mut ip.xDD4_itemVar.tarucann;
    if ip.ground_or_air != 1 {
        let d = t.x14.x;
        if c_abs(d) > 0.1 {
            if c_abs(t.x10) < da.x18 {
                t.x10 = fmadds(d, da.x10, t.x10);
            }
        } else {
            let f1 = t.x10;
            let mut f2 = da.x14;
            if c_abs(f1) < f2 {
                t.x10 = 0.0;
            } else {
                if f1 > 0.0 {
                    f2 = -f2;
                }
                t.x10 = f1 + f2;
            }
        }
        let s = t.x10;
        if (s > 0.0 && ip.facing_dir == -1.0) || (s < 0.0 && ip.facing_dir == 1.0) {
            ip.facing_dir = -ip.facing_dir;
        }
        let m = da.xC;
        if c_abs(t.x10) > m {
            t.x10 = m * ip.facing_dir;
        }
        ip.x40_vel.x = t.x28 * (t.x10 * t.x14.y);
        let v2 = c_abs(t.x10);
        t.xC = fmadds(t.x28, 0.12217305f32 * v2 * -ip.facing_dir, t.xC);
    } else {
        t.xC = fmadds(0.10471976f32 * -ip.facing_dir, t.x28, t.xC);
    }
    t.xC = fmadds(da.x50, -ip.facing_dir, t.xC);
}

/// `it_2725_Logic101_Reflected` (itmewtwoshadowball.c): turn the angle by pi and wrap it into
/// [0, tau] (in f64, as the `double` macros make the C). Returns false.
/// Precondition (as the C): the angle is finite, or the loops never end.
pub fn it_2725_logic101_reflected(ip: &mut Item) -> bool {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let a = &mut ip.xDD4_itemVar.mewtwoshadowball.x4.x;
        *a = (*a as f64 + M_PI) as f32;
        while *a < 0.0 {
            *a = (*a as f64 + M_TAU) as f32;
        }
        while *a as f64 > M_TAU {
            *a = (*a as f64 - M_TAU) as f32;
        }
        false
    }
}

/// `itMaril_UnkMotion1_Coll_inline` (itmaril.c, static): turn around once.
/// # Safety
/// The special attributes must be a valid `itMarilAttributes`.
pub unsafe fn it_maril_unk_motion1_coll_inline(ip: &mut Item) {
    let attr = sa::<itMarilAttributes>(ip);
    if ip.xDD4_itemVar.maril.x60.anon_0.x0() == 0 {
        let m = &mut ip.xDD4_itemVar.maril;
        m.x6C.y = attr.x8;
        m.x6C.x = ((M_PI * -ip.facing_dir as f64) / attr.x8 as f64) as f32;
        m.x60.anon_0.set_x0(1);
        ip.facing_dir = -ip.facing_dir;
        ip.xDC8_word.flags.set_x19(0);
        let m = &mut ip.xDD4_itemVar.maril;
        m.x64 = -m.x64;
    }
}

// ---- scope / wstar / sword -------------------------------------------------------------------

/// `it_80291DAC_level` (itsscope.c, static inline): charge level 0..9 of `arg1`.
/// # Safety
/// The special attributes must be a valid `itSScopeAttributes`.
pub unsafe fn it_80291dac_level(ip: &Item, arg1: i32) -> i32 {
    let attrs = sa::<itSScopeAttributes>(ip);
    if arg1 <= attrs.x4 {
        0
    } else if arg1 < attrs.x8.wrapping_mul(8) {
        arg1 / 8
    } else {
        9
    }
}

/// `it_802944AC` (itwstar.c): the attribute box scaled by `wstar.xDD4`.
/// # Safety
/// The special attributes must be a valid `itWstarAttributes`.
pub unsafe fn it_802944ac(ip: &Item, b: &mut ftCollisionBox) {
    let attr = sa::<itWstarAttributes>(ip);
    let s = ip.xDD4_itemVar.wstar.xDD4;
    b.top = attr.xC * s;
    b.bottom = attr.x10 * s;
    b.left.x = attr.x14 * s;
    b.left.y = attr.x18 * s;
    b.right.x = attr.x1C * s;
    b.right.y = attr.x20 * s;
}

/// `inlineA1` (itsword.c, static inline): start blend 0 towards 1.0 over 4 frames.
pub fn inline_a1(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        s.x4 = 4;
        s.x8 = 0;
        s.xC = -1;
        s.x14 = 1.0;
        s.x18 = (s.x14 - s.x10) / s.x4 as f32;
    }
}

/// `inlineA2` (itsword.c, static inline): as `inlineA1` for blend 1.
pub fn inline_a2(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        s.x1C = 4;
        s.x20 = 0;
        s.x24 = -1;
        s.x2C = 1.0;
        s.x30 = (s.x2C - s.x28) / s.x1C as f32;
    }
}

/// `inlineC1` (itsword.c, static inline): blend 1 from 1.0 to 2.0 over 4 frames.
pub fn inline_c1(ip: &mut Item) {
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        s.x1C = 0;
        s.x20 = 4;
        s.x24 = 1;
        s.x28 = 1.0;
        s.x2C = 2.0;
        s.x30 = (s.x2C - s.x28) / 4.0;
    }
}

/// `inlineA3` (itsword.c, static inline): blend 2 towards `attrs->x10` over `attrs->x18`.
/// # Safety
/// The special attributes must be a valid `itSword_UnkArticle1`.
pub unsafe fn inline_a3(ip: &mut Item) {
    let attrs = sa::<itSword_UnkArticle1>(ip);
    let s = &mut ip.xDD4_itemVar.sword;
    s.x34 = attrs.x18;
    s.x38 = 0;
    s.x3C = -1;
    s.x44 = attrs.x10;
    s.x48 = (attrs.x10 - s.x40) / attrs.x18 as f32;
}

/// `inlineD1` (itsword.c, static inline): blend 2 from `attrs->x10` to `attrs->x14`.
/// # Safety
/// The special attributes must be a valid `itSword_UnkArticle1`.
pub unsafe fn inline_d1(ip: &mut Item) {
    let attrs = sa::<itSword_UnkArticle1>(ip);
    let s = &mut ip.xDD4_itemVar.sword;
    s.x34 = 0;
    s.x38 = attrs.x18;
    s.x3C = 1;
    s.x40 = attrs.x10;
    s.x44 = attrs.x14;
    s.x48 = (attrs.x14 - attrs.x10) / attrs.x18 as f32;
}

// ---- zgshell ---------------------------------------------------------------------------------

/// The shared body of `it_802DDBE8` / `itZrshell_UnkMotion3_Anim`: count down `xDFC` while
/// `xE08_b0`, then re-arm and restore hurtbox state.
unsafe fn zgshell_tick(ip: &mut Item) {
    let attrs = sa::<itGShell_Attrs>(ip);
    if ip.xDD4_itemVar.zgshell.xE08_b0() != 0 {
        ip.xDD4_itemVar.zgshell.xDFC -= 1.0;
        if ip.xDD4_itemVar.zgshell.xDFC <= 0.0 {
            ip.xDD4_itemVar.zgshell.set_xE08_b0(0);
            ip.xDD4_itemVar.zgshell.xDFC = attrs.x28;
            it_8027572c(ip, 0);
            if ip.xDD4_itemVar.zgshell.xE08_b1() != 0 {
                it_802756e0(ip);
            }
        }
    }
}

/// `it_802DDBE8` (itzgshell.c).
/// # Safety
/// The special attributes must be a valid `itGShell_Attrs`.
pub unsafe fn it_802ddbe8(ip: &mut Item) {
    zgshell_tick(ip);
}

/// `itZrshell_UnkMotion3_Anim` (itzgshell.c): as `it_802DDBE8`, returns false.
/// # Safety
/// The special attributes must be a valid `itGShell_Attrs`.
pub unsafe fn it_zrshell_unk_motion3_anim(ip: &mut Item) -> bool {
    zgshell_tick(ip);
    false
}

/// `it_802DFF14` (itzgshell.c): set hurtbox 0 from the shell's capsule, raised and doubled if
/// `arg1`. (The C passes an `itGShell_HurtInit` as a `HurtCapsule*`; `it_80271534` reads only
/// the offsets and scale, which the two share.)
/// # Safety
/// `ip.xC4_article_data` must be valid.
pub unsafe fn it_802dff14(ip: &mut Item, arg1: i32) {
    let c = ip.xDD4_itemVar.zgshell.x58_hurtInit;
    let mut h: HurtCapsule = HurtCapsule::zeroed();
    h.a_offset = c.a_offset;
    h.b_offset = c.b_offset;
    h.scale = c.scale;
    if arg1 != 0 {
        h.a_offset.y = 6.0;
        h.scale *= 2.0;
    }
    it_80271534(ip, 0, &h);
}

// ---- leadead / links / pure helpers ---------------------------------------------------------

/// `it_802EA988` (itleadead.c): 8 if the item touches a left wall, 4 if a right wall (right
/// wins). The offset copy of the collision data is never used.
pub fn it_802ea988(ip: &Item, _x_offset: f32, _y_offset: f32) -> i32 {
    let f = ip.x378_itemColl.env_flags;
    let mut ret = 0;
    if f & Collide_LeftWallMask != 0 {
        ret = 8;
    }
    if f & Collide_RightWallMask != 0 {
        ret = 4;
    }
    ret
}

/// `it_802A43B8` (itlinkhookshot.c): `cur_pos = pos; last_pos = cur_pos`.
pub fn it_802a43b8(l: &mut ItemLink) {
    l.coll_data.cur_pos = l.pos;
    l.coll_data.last_pos = l.coll_data.cur_pos;
}

/// `it_802A43EC` (itlinkhookshot.c): `last_pos = cur_pos; cur_pos = pos`.
pub fn it_802a43ec(l: &mut ItemLink) {
    l.coll_data.last_pos = l.coll_data.cur_pos;
    l.coll_data.cur_pos = l.pos;
}

/// `it_802A4420` (itlinkhookshot.c): `pos += vel`.
pub fn it_802a4420(l: &mut ItemLink) {
    l.pos.x += l.vel.x;
    l.pos.y += l.vel.y;
    l.pos.z += l.vel.z;
}

/// `test_comp` (itlinkhookshot.c, static inline): `vec0 = vec2 * *arg2 + vec1` (3 fused).
pub fn test_comp(vec0: &mut GcVec, vec1: &GcVec, vec2: &GcVec, arg2: f32) {
    vec0.x = fmadds(vec2.x, arg2, vec1.x);
    vec0.y = fmadds(vec2.y, arg2, vec1.y);
    vec0.z = fmadds(vec2.z, arg2, vec1.z);
}

/// `it_link_attr_math` (itlinkhookshot.c, static inline): scale the hookshot attributes.
pub fn it_link_attr_math(attr: &mut itLinkHookshotAttributes, arg2: i32, arg8: f32) {
    let f0 = if arg2 != 0 { attr.x50 } else { attr.x4C };
    attr.x30 = attr.x10 * arg8;
    attr.x34 = attr.x14 * arg8;
    attr.x38 = f0 * (attr.x18 * arg8);
    attr.x3C = attr.x1C * arg8;
    attr.x40 = f0 * (attr.x20 * arg8);
    attr.x44 = f0 * (attr.x24 * arg8);
    attr.x48 = attr.x28 * arg8;
    let f1 = if arg8 > 1.0 {
        it_link_lerp(attr.xC as f32 * attr.x30, attr.xC as f32 * attr.x10, attr.x8)
    } else {
        attr.xC as f32 * attr.x30
    };
    attr.x2C = ((f1 * f0) / attr.x30) as i32;
}

/// `samus_grapple_fighter_compare` (itsamusgrapple.c, static inline): one of the four motions.
pub fn samus_grapple_fighter_compare(id: i32) -> bool {
    id == 0x165 || id == 0x166 || id == 0xD4 || id == 0xD6
}

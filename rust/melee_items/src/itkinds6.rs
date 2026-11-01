
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

pub(crate) const M_PI: f64 = 3.14159265358979323846;

pub(crate) const M_TAU: f64 = 6.283185307179586;

/// The kind's special attributes as a `T`.
/// # Safety
/// The article data and its special attributes must be valid as a `T` for `'a`.
pub(crate) unsafe fn sa<'a, T>(ip: &Item) -> &'a T {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const T)
}

/// `inline_fabsf` / the C `ABS(x)`: `x < 0 ? -x : x` (keeps the sign of a zero).
pub(crate) fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
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
    let attrs = sa::<itSword_UnkArticle1>(ip) as *const itSword_UnkArticle1 as *mut itSword_UnkArticle1;
    let s = &mut ip.xDD4_itemVar.sword;
    s.x34 = crate::tu::itsword::swi(attrs, 0x18);
    s.x38 = 0;
    s.x3C = -1;
    s.x44 = crate::tu::itsword::swf(attrs, 0x10);
    s.x48 = (crate::tu::itsword::swf(attrs, 0x10) - s.x40) / crate::tu::itsword::swi(attrs, 0x18) as f32;
}

/// `inlineD1` (itsword.c, static inline): blend 2 from `attrs->x10` to `attrs->x14`.
/// # Safety
/// The special attributes must be a valid `itSword_UnkArticle1`.
pub unsafe fn inline_d1(ip: &mut Item) {
    let attrs = sa::<itSword_UnkArticle1>(ip) as *const itSword_UnkArticle1 as *mut itSword_UnkArticle1;
    let s = &mut ip.xDD4_itemVar.sword;
    s.x34 = 0;
    s.x38 = crate::tu::itsword::swi(attrs, 0x18);
    s.x3C = 1;
    s.x40 = crate::tu::itsword::swf(attrs, 0x10);
    s.x44 = crate::tu::itsword::swf(attrs, 0x14);
    s.x48 = (crate::tu::itsword::swf(attrs, 0x14) - crate::tu::itsword::swf(attrs, 0x10)) / crate::tu::itsword::swi(attrs, 0x18) as f32;
}

// ---- zgshell ---------------------------------------------------------------------------------

/// The shared body of `it_802DDBE8` / `itZrshell_UnkMotion3_Anim`: count down `xDFC` while
/// `xE08_b0`, then re-arm and restore hurtbox state.
pub(crate) unsafe fn zgshell_tick(ip: &mut Item) {
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

/// `itZrshell_UnkMotion3_Anim` (itzgshell.c): as `it_802DDBE8`, returns false.
/// # Safety
/// The special attributes must be a valid `itGShell_Attrs`.
pub unsafe fn it_zrshell_unk_motion3_anim(ip: &mut Item) -> bool {
    zgshell_tick(ip);
    false
}

pub use crate::tu::itmewtwoshadowball::it_2725_logic101_reflected;
pub use crate::tu::itdrop::it_8026f3d4_check_kind;
pub use crate::tu::ithitbox::it_8027574c;
pub use crate::tu::itmaplib::it_802762b0_fp as it_802762b0;
pub use crate::tu::ittaru::it_802874f0;
pub use crate::tu::itfreeze::it_8028edbc;
pub use crate::tu::itsscope::it_80291dac_level;
pub use crate::tu::itwstar::it_802944ac;
pub use crate::tu::ittarucann::it_802962e0;
pub use crate::tu::itlinkhookshot::it_802a43b8;
pub use crate::tu::itlinkhookshot::it_802a43ec;
pub use crate::tu::itlinkhookshot::it_802a4420;
pub use crate::tu::itfoxblaster::it_802ae608;
pub use crate::tu::itlikelike::it_802db358;
pub use crate::tu::itzgshell::it_802ddbe8;
pub use crate::tu::itzgshell::it_802dff14;
pub use crate::tu::itleadead::it_802ea988;
pub use crate::tu::itlinkhookshot::it_link_attr_math_fp as it_link_attr_math;
pub use crate::tu::itsamusgrapple::samus_grapple_fighter_compare_fp as samus_grapple_fighter_compare;
pub use crate::tu::itlinkhookshot::test_comp_fp as test_comp;

#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DownAttack.c (getup attack, DownSpot check).
//! Checked against the C by tests/tu_ftCo_DownAttack.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftCo_800DF644(fp: &Fighter) -> bool = crate::ft0df::ftco_800df644;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80084104(g as _); } };
        // `HSD_GObjPLinkHead[link]`.
        HSD_GObjPLinkHead(link: i32) -> G = crate::hooks::plink_head;
        ftCo_800C5240(g: G) -> bool = crate::hooks::ftco_800c5240;
        ftCo_800987D0(g: G) = |g: G| { unsafe { crate::tu::ftCo_Passive::ftCo_800987D0(g as _); } };
    }
}

/// static `ecb_offset`: top, bottom, right, left.
pub const ECB_OFFSET: _itECB = _itECB { top: 14.0, bottom: 0.0, right: -3.0, left: 3.0 };

/// static inline `inlineA0`.
fn inlineA0(f: &Fighter) -> bool {
    f.input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0
}

pub fn ftCo_800984D4(g: G) -> bool {
    melee_core::ct!("ftCo_800984D4");
    let f = fp(g);
    if inlineA0(f) || ext::ftCo_800DF644(f) {
        // C declares `bool msid`, but `bool` is int in this build: the id survives intact.
        let msid = if f.motion_id == ftCo_MS_DownWaitU as _ { ftCo_MS_DownAttackU } else { ftCo_MS_DownAttackD };
        ftCo_8009856C(g, msid as i32);
        return true;
    }
    false
}

pub fn ftCo_8009856C(g: G, msid: i32) {
    melee_core::ct!("ftCo_8009856C");
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_DownAttack_Anim(g: G) {
    melee_core::ct!("ftCo_DownAttack_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_DownAttack_IASA(_g: G) { melee_core::ct!("ftCo_DownAttack_IASA");}

pub fn ftCo_DownAttack_Phys(g: G) {
    melee_core::ct!("ftCo_DownAttack_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_DownAttack_Coll(g: G) {
    melee_core::ct!("ftCo_DownAttack_Coll");
    ext::ft_80084104(g);
}

/// `ftCo_80098634`: one ECB offset and position per fighter, in plink order.
pub fn ftCo_80098634(track: &mut Item_FtTrack) {
    melee_core::ct!("ftCo_80098634");
    let mut cur = ext::HSD_GObjPLinkHead(HSD_GOBJ_PLINK_FIGHTER as i32);
    let mut i = 0usize;
    while !cur.is_null() {
        let pos = fp(cur).cur_pos;
        let e = &mut track.ecb_offset_arr[i];
        e.top = ECB_OFFSET.top;
        e.bottom = ECB_OFFSET.bottom;
        e.right = ECB_OFFSET.right;
        e.left = ECB_OFFSET.left;
        track.ft_pos_arr[i] = pos;
        cur = unsafe { (*cur).next };
        i += 1;
    }
    track.count = i as _;
}

pub fn ftCo_800986B0(g: G) -> bool {
    melee_core::ct!("ftCo_800986B0");
    if !ext::ftCo_800C5240(g) {
        let f = fp(g);
        if (f.x680 as f32) < cd().x250 && (f.x684 as i32) >= cd().x1C {
            return true;
        }
    }
    false
}

pub fn ftCo_8009872C(g: G) -> bool {
    melee_core::ct!("ftCo_8009872C");
    if ftCo_800986B0(g) {
        ext::ftCo_800987D0(g);
        return true;
    }
    false
}

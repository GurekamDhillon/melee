//! src/melee/ft/kinds/ftKirby/ftkirbyspecialpikachu.c: Kirby's Thunder Jolt (Pikachu / Pichu copy).
//! Fused sites (tools/rust-port/fma_sites.py, 4 per Anim): the jolt spawn position
//! `scale.y * (off.x * facing) + cur.x` and `off.y * scale.y + cur.y`, each an f32 `fmadds`.
//! The spawn position and landing-lag choice are checked against the C by tests/tu_ftkirbyspecialpikachu.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{fpp, G};
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialpikachu::{ftKb_PkSpecialAirN_Phys, ftKb_PkSpecialN_Phys};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    type G = *mut HSD_GObj;
    pub static mut ITPIKACHUTHUNDERJOLT_SPAWN: Option<fn(G, *const Vec3, f32, ItemKind) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itPikachuThunderJolt_Spawn(g: G, pos: *const Vec3, dir: f32, kind: ItemKind) -> *mut Item_GObj {
                match unsafe { ITPIKACHUTHUNDERJOLT_SPAWN } {
                    Some(f) => f(g, pos, dir, kind),
                    None => { return unsafe { melee_items::tu::itpikachutjoltground::itPikachuThunderJolt_Spawn(g as _, pos as _, dir as _, kind as _) as _ }; },
                }
            }
}

/// `coll_mf` (static const): the Coll ground<->air change's motion flags.
pub const coll_mf: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `doEnter` (static): enter the Pikachu or Pichu variant of the state, clearing the cmd vars.
pub fn doEnter(gobj: G, pk_msid: i32, pc_msid: i32) {
    melee_core::ct!("doEnter");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = pk_msid;
        let kind = (*fp).u.kb.hat.kind;
        if kind == Ft_Kind_Pichu {
            msid = pc_msid;
        }
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).cmd_vars[ftPk_SpecialN_Cmd0 as usize] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd1 as usize] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd2 as usize] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd3 as usize] = 0;
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftKb_SpecialNPk_800F9FD4(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPk_800F9FD4");
    doEnter(gobj, ftKb_MS_PkSpecialN as i32, ftKb_MS_PcSpecialN as i32);
}

pub fn ftKb_SpecialNPk_800FA064(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPk_800FA064");
    doEnter(gobj, ftKb_MS_PkSpecialAirN as i32, ftKb_MS_PcSpecialAirN as i32);
}

/// Jolt spawn position: `(scale.y * (off.x * facing)) + cur.x`, `(off.y * scale.y) + cur.y`, both
/// fused (f32 `fmadds`: the product of the first is rounded to f32 before it enters the fma).
pub fn jolt_pos(scale_y: f32, off: Vec2, facing: f32, cur: Vec3) -> Vec3 {
    melee_core::ct!("jolt_pos");
    Vec3 { x: fmadds(scale_y, off.x * facing, cur.x), y: fmadds(off.y, scale_y, cur.y), z: 0.0 }
}

/// The `cmd_vars[Cmd0]` jolt spawn shared by the ground and air Anim (offsets differ).
unsafe fn spawn_jolt(gobj: G, fp: *mut Fighter, pk_off: Vec2, pc_off: Vec2) {
    let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
    if (*fp).cmd_vars[ftPk_SpecialN_Cmd0 as usize] == 1 {
        (*fp).cmd_vars[ftPk_SpecialN_Cmd0 as usize] = 0;
        if (*fp).cmd_vars[ftPk_SpecialN_Cmd1 as usize] == 0 {
            (*fp).cmd_vars[ftPk_SpecialN_Cmd1 as usize] = 1;
            let kind = (*fp).u.kb.hat.kind;
            if kind == Ft_Kind_Pikachu {
                let it_pos = jolt_pos((*fp).x34_scale.y, pk_off, (*fp).facing_dir, (*fp).cur_pos);
                ext::itPikachuThunderJolt_Spawn(gobj, &it_pos, (*fp).facing_dir, da.specialn_pk_itkind);
                fh::play_sfx(&mut *fp, 240076, 127, 64);
            } else if kind == Ft_Kind_Pichu {
                let it_pos = jolt_pos((*fp).x34_scale.y, pc_off, (*fp).facing_dir, (*fp).cur_pos);
                ext::itPikachuThunderJolt_Spawn(gobj, &it_pos, (*fp).facing_dir, da.specialn_pc_itkind);
                fh::play_sfx(&mut *fp, 230067, 127, 64);
            }
        }
    }
}

pub fn ftKb_PkSpecialN_Anim(gobj: G) {
    melee_core::ct!("ftKb_PkSpecialN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        spawn_jolt(gobj, fp, da.specialn_pk_spawn_offset, da.specialn_pc_spawn_offset);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_PkSpecialAirN_Anim(gobj: G) {
    melee_core::ct!("ftKb_PkSpecialAirN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        spawn_jolt(gobj, fp, da.specialairn_pk_spawn_offset, da.specialairn_pc_spawn_offset);
        // `landing_lag` is uninitialised for other hat kinds in the C (-ftrivial-auto-var-init=zero).
        let mut landing_lag = 0.0f32;
        let kind = (*fp).u.kb.hat.kind;
        if kind == Ft_Kind_Pikachu {
            landing_lag = da.specialairn_pk_landing_lag;
        } else if kind == Ft_Kind_Pichu {
            landing_lag = da.specialairn_pc_landing_lag;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if landing_lag == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, landing_lag);
            }
        }
    }
}

pub fn ftKb_PkSpecialN_IASA(_gobj: G) { melee_core::ct!("ftKb_PkSpecialN_IASA");}
pub fn ftKb_PkSpecialAirN_IASA(_gobj: G) { melee_core::ct!("ftKb_PkSpecialAirN_IASA");}

/// Ground -> air; keeps the animation frame.
unsafe fn coll_change(gobj: G, pk_ms: i32, pc_ms: i32, fp: *mut Fighter) {
    let anim_frame = (*fp).cur_anim_frame;
    let mut msid = pk_ms;
    if (*fpp(gobj)).u.kb.hat.kind == Ft_Kind_Pichu {
        msid = pc_ms;
    }
    st::Fighter_ChangeMotionState(gobj, msid, coll_mf, anim_frame, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftKb_PkSpecialN_Coll(gobj: G) {
    melee_core::ct!("ftKb_PkSpecialN_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            let fp = fpp(gobj);
            st::ftCommon_8007D5D4(&mut *fp);
            coll_change(gobj, ftKb_MS_PkSpecialAirN as i32, ftKb_MS_PcSpecialAirN as i32, fp);
        }
    }
}

pub fn ftKb_PkSpecialAirN_Coll(gobj: G) {
    melee_core::ct!("ftKb_PkSpecialAirN_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) == GA_Air as i32 {
            let fp = fpp(gobj);
            st::ftCommon_8007D7FC(&mut *fp);
            (*fp).self_vel.y = 0.0;
            coll_change(gobj, ftKb_MS_PkSpecialN as i32, ftKb_MS_PcSpecialN as i32, fp);
        }
    }
}

//! src/melee/ft/ftcolanim.c: fighter colour-animation (ColorOverlay) list management and the
//! Sleep state entry. No float math. Callees outside this TU (lb_013B colanim interpreter,
//! ftAction subactions, Mex dispatch, PlCo colanim tables) are `ext` hooks that panic while
//! unset. The TARGET_PC path is ported (Mex_OnActionStateChangeDispatch replaces the direct
//! ftData_UnkMotionStates4 call).
#![allow(non_snake_case, static_mut_refs, non_upper_case_globals)]

use crate::types::*;

pub type G = *mut HSD_GObj;
pub type FtCmdFn = fn(G, *mut CommandInfo);
pub type ColAnimCb = fn(G, *mut CommandInfo, i32);

pub mod ext {
    use super::{ColAnimCb, G};
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: MotionFlags, anim_start: f32, rate: f32, blend: f32, obj: *mut HSD_GObj);
        PLAYER_GET_ENTITY_AT_INDEX Player_GetEntityAtIndex(slot: i32, i: i32) -> G;
        GM_80167320 gm_80167320(slot: i32, sub: i32);
        LB_80014498 lb_80014498(co: *mut ColorOverlay);
        LB_800144C8 lb_800144C8(co: *mut ColorOverlay, tbl: *mut Fighter_804D653C_t, idx: i32, b: i32) -> bool;
        LB_80014258 lb_80014258(g: G, co: *mut ColorOverlay, cb: ColAnimCb) -> bool;
        FTACTION_80071028 ftAction_80071028(g: G, c: *mut CommandInfo);
        FTACTION_800711DC ftAction_800711DC(g: G, c: *mut CommandInfo);
        FTACTION_80071B50 ftAction_80071B50(g: G, c: *mut CommandInfo);
        FTACTION_80071CA4 ftAction_80071CA4(g: G, c: *mut CommandInfo);
        FTACTION_800730B8 ftAction_800730B8(g: G, c: *mut CommandInfo);
        FTACTION_80073108 ftAction_80073108(g: G, c: *mut CommandInfo);
        FTCO_800C53E4 ftCo_800C53E4(fp: *mut Fighter) -> i32;
        MEX_ON_ACTION_STATE_CHANGE_DISPATCH Mex_OnActionStateChangeDispatch(kind: i32, gobj: G, vanilla: *mut core::ffi::c_void);
    }
    /// TODO(move-to-core ftdata) PlCo colanim tables (`Fighter_804D653C`, spycloak `Fighter_804D6538`).
    pub static mut Fighter_804D653C: *mut Fighter_804D653C_t = core::ptr::null_mut();
    pub static mut Fighter_804D6538: *mut Fighter_804D653C_t = core::ptr::null_mut();
    /// TODO(move-to-core ftdata) `ftData_UnkMotionStates4` (per-kind onActionStateChange).
    pub static mut ftData_UnkMotionStates4: *const *mut core::ffi::c_void = core::ptr::null();
    pub fn ftCo_800D4F24(g: G, i: i32) {
        crate::hooks::ftco_800d4f24(g, i)
    }
}

#[inline(always)]
fn fpg<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

fn t653c() -> *mut Fighter_804D653C_t {
    let p = unsafe { ext::Fighter_804D653C };
    assert!(!p.is_null(), "TODO Fighter_804D653C not loaded");
    p
}

/// `ftCo_803C6AD0`.
pub static ftCo_803C6AD0: [FtCmdFn; 3] = [ftco_800bfe74, ftco_800bfeb4, ftco_800bfef4];
/// `ftCo_803C6ADC`.
pub static ftCo_803C6ADC: [FtCmdFn; 3] = [ftco_800bfe94, ftco_800bfed4, ftco_800bff14];

/// `ftCo_DispatchActionStateChange` (TARGET_PC static).
fn dispatch_action_state_change(fp: &mut Fighter) {
    let vanilla = unsafe {
        let t = ext::ftData_UnkMotionStates4;
        assert!(!t.is_null(), "TODO ftData_UnkMotionStates4 not loaded");
        *t.add(fp.kind as usize)
    };
    ext::Mex_OnActionStateChangeDispatch(fp.kind as i32, fp.gobj, vanilla);
}

/// `ftCo_800BFD04`: enter Sleep (hidden, inactive camera).
pub fn ftco_800bfd04(g: G) {
    melee_core::ct!("ftco_800bfd04");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Sleep as i32, Ft_MF_None as MotionFlags, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let fp = fpg(g);
    fp.set_invisible(1);
    fp.set_x221E_b1(1);
    fp.set_x221E_b2(1);
    fp.set_x2219_b1(1);
    unsafe { (*fp.x890_cameraBox).state = CmSubjectState_Inactive as _ };
    fp.set_x221F_b3(1);
    fp.set_x221F_b1(1);
}

/// `ftCo_800BFD9C`.
pub fn ftco_800bfd9c(g: G) {
    melee_core::ct!("ftco_800bfd9c");
    ftco_800bfd04(g);
    let fp = fpg(g);
    if fp.x2222_b5() != 0 {
        let pl = ext::Player_GetEntityAtIndex(fp.player_id as i32, 1);
        if !pl.is_null() {
            ext::ftCo_800D4F24(pl, 1);
        }
    }
    let fp = fpg(g);
    ext::gm_80167320(fp.player_id as i32, fp.is_sub_fighter() as i32);
}

/// `ftCo_Sleep_Anim`: empty.
pub fn ftco_sleep_anim(_g: G) { melee_core::ct!("ftco_sleep_anim");}
/// `ftCo_Sleep_IASA`: empty.
pub fn ftco_sleep_iasa(_g: G) { melee_core::ct!("ftco_sleep_iasa");}

fn ftco_800bfe74(g: G, c: *mut CommandInfo) { ext::ftAction_80071028(g, c) }
fn ftco_800bfe94(g: G, c: *mut CommandInfo) { ext::ftAction_800711DC(g, c) }
fn ftco_800bfeb4(g: G, c: *mut CommandInfo) { ext::ftAction_80071B50(g, c) }
fn ftco_800bfed4(g: G, c: *mut CommandInfo) { ext::ftAction_80071CA4(g, c) }
fn ftco_800bfef4(g: G, c: *mut CommandInfo) { ext::ftAction_800730B8(g, c) }
fn ftco_800bff14(g: G, c: *mut CommandInfo) { ext::ftAction_80073108(g, c) }

/// `ft_800BFF34` (static): colanim command callback, table 803C6AD0.
pub fn ft_800bff34(g: G, c: *mut CommandInfo, arg2: i32) {
    melee_core::ct!("ft_800bff34");
    ftCo_803C6AD0[(arg2 - 0x15) as usize](g, c)
}

/// `ft_800BFF70` (static): colanim command callback, table 803C6ADC.
pub fn ft_800bff70(g: G, c: *mut CommandInfo, arg0: i32) {
    melee_core::ct!("ft_800bff70");
    ftCo_803C6ADC[(arg0 - 21) as usize](g, c)
}

/// `ftCo_800BFFAC`.
pub fn ftco_800bffac(fp: &mut Fighter) {
    melee_core::ct!("ftco_800bffac");
    ext::lb_80014498(&mut fp.x488);
}

/// `ftCo_800BFFD0`: start colanim `arg1` on the list it belongs to.
pub fn ftco_800bffd0(fp: &mut Fighter, arg1: i32, arg2: i32) -> bool {
    melee_core::ct!("ftco_800bffd0");
    if arg1 >= 0x7B {
        if ext::lb_800144C8(&mut fp.x508, unsafe { ext::Fighter_804D6538 }, arg1 - 0x7B, arg2) {
            return true;
        }
    } else if unsafe { (*t653c().offset(arg1 as isize)).unk5 } != 0 {
        if ext::lb_800144C8(&mut fp.x488, t653c(), arg1, arg2) {
            return true;
        }
    } else if ext::lb_800144C8(&mut fp.x408, t653c(), arg1, arg2) {
        return true;
    }
    false
}

/// `ftCo_800C0074`.
pub fn ftco_800c0074(fp: &mut Fighter) {
    melee_core::ct!("ftco_800c0074");
    ext::lb_80014498(&mut fp.x408);
}

/// `ft_800C0098`.
pub fn ft_800c0098(fp: &mut Fighter) {
    melee_core::ct!("ft_800c0098");
    ext::lb_80014498(&mut fp.x508);
    if fp.x2226_b4() != 0 {
        ext::lb_800144C8(&mut fp.x508, unsafe { ext::Fighter_804D6538 }, 0x80 - 0x7B, 0);
    }
}

/// `ftCo_800C0134`.
pub fn ftco_800c0134(fp: &mut Fighter) {
    melee_core::ct!("ftco_800c0134");
    ext::lb_80014498(&mut fp.x488);
    dispatch_action_state_change(fp);
    if ext::ftCo_800C53E4(fp) != 0 {
        let t = t653c();
        if unsafe { (*t.add(0x6A)).unk5 } != 0 {
            ext::lb_800144C8(&mut fp.x488, t, 0x6A, 0);
        } else {
            ext::lb_800144C8(&mut fp.x408, t, 0x6A, 0);
        }
    }
}

fn reapply_x408(fp: &mut Fighter) {
    if fp.x2224_b2() != 0 {
        ftco_800bffd0(fp, 0x7A, 0);
    }
    if fp.dmg.x18F0 != 0 {
        ftco_800bffd0(fp, 8, 0);
    }
    if fp.x221D_b6() != 0 {
        ftco_800bffd0(fp, 0x6B, 0);
    }
    if fp.x1994 != 0 || fp.x1990 != 0 || fp.x2221_b0() != 0 {
        ftco_800bffd0(fp, 9, 0);
    }
}

fn reapply_x488(fp: &mut Fighter) {
    dispatch_action_state_change(fp);
    if ext::ftCo_800C53E4(fp) != 0 {
        ftco_800bffd0(fp, 0x6A, 0);
    }
}

/// `ftCo_800C0200`: reset colanim `arg1`'s list and reapply the standing overlays.
pub fn ftco_800c0200(fp: &mut Fighter, arg1: i32) {
    melee_core::ct!("ftco_800c0200");
    if arg1 >= 0x7B {
        panic!("don't reset spycloak colanim!");
    }
    if unsafe { (*t653c().offset(arg1 as isize)).unk5 } != 0 {
        ext::lb_80014498(&mut fp.x488);
        reapply_x488(fp);
    } else {
        ext::lb_80014498(&mut fp.x408);
        reapply_x408(fp);
    }
}

/// `ftCo_800C0358`: copy the list of colanim `arg2` from `fp` to `arg1`.
pub fn ftco_800c0358(fp: &Fighter, arg1: &mut Fighter, arg2: i32) {
    melee_core::ct!("ftco_800c0358");
    if arg2 >= 0x7B {
        panic!("don't reset spycloak colanim!");
    }
    if unsafe { (*t653c().offset(arg2 as isize)).unk5 } != 0 {
        arg1.x488 = fp.x488;
    } else {
        arg1.x408 = fp.x408;
    }
}

/// `ftCo_800C0408`: run the three colanim lists for one frame.
pub fn ftco_800c0408(g: G) {
    melee_core::ct!("ftco_800c0408");
    let fp = fpg(g);
    while ext::lb_80014258(g, &mut fpg(g).x508, ft_800bff34) {
        ext::lb_80014498(&mut fpg(g).x508);
        if fpg(g).x2226_b4() != 0 {
            ftco_800bffd0(fpg(g), 0x80, 0);
        }
    }
    while ext::lb_80014258(g, &mut fpg(g).x408, ft_800bff34) {
        ext::lb_80014498(&mut fpg(g).x408);
        reapply_x408(fpg(g));
    }
    let cb: ColAnimCb = if unsafe { fp.x408.x28_colanim.i } == 0 { ft_800bff34 } else { ft_800bff70 };
    while ext::lb_80014258(g, &mut fpg(g).x488, cb) {
        ext::lb_80014498(&mut fpg(g).x488);
        reapply_x488(fpg(g));
    }
    fpg(g).set_x2221_b3(1);
}

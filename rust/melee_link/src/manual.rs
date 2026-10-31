//! Link entries the generator cannot make: the hook's signature differs from the canonical function in more than
//! a cast (a C-variadic call spread over fixed slots, a record argument split into scalars). Each is an adapter
//! with no logic of its own, listed in `docs/rust-port/notes/consolidation.md` (step 7). `install()` runs after
//! the generated table, so these win.
#![allow(clippy::missing_safety_doc)]
use core::ffi::c_void;
use melee_runtime::tu::aobj::CallbackArg;
use melee_types::*;

/// `HSD_ForeachAnim(obj, type, mask, func, arg_type, <variadic>)` from a hook that carries the variadic as an
/// `f64` and a `u32` (ground.c): `AF` takes the narrowed float, `AU` the integer.
unsafe fn foreach_anim_fu(obj: *mut c_void, ty: i32, mask: u32, func: *mut c_void, arg_type: i32, f: f64, u: u32) {
    let arg = match arg_type as u32 {
        t if t == AOBJ_ARG_AF as u32 => CallbackArg { f: f as f32 },
        _ => CallbackArg { d: u },
    };
    melee_runtime::tu::aobj::HSD_ForeachAnim(obj, ty as _, mask as _, func, arg_type as _, arg);
}

pub fn install() {
    unsafe {
        // texp.c's DAG calls take the texpdag.c host trait; the game's host is `texpdag::Ext`
        use melee_runtime::tu::{texp::ext::hook as te, texpdag};
        te::HSD_TExpMakeDag = Some(|root, list| unsafe { texpdag::HSD_TExpMakeDag(&mut texpdag::Ext, root, list) });
        te::HSD_TExpSchedule = Some(|num, list, result, res| unsafe { texpdag::HSD_TExpSchedule(&mut texpdag::Ext, num, list, result, res) });
        te::HSD_TExpSimplify = Some(|exp| unsafe { texpdag::HSD_TExpSimplify(&mut texpdag::Ext, exp) });
        te::HSD_TExpSimplify2 = Some(|exp| unsafe { texpdag::HSD_TExpSimplify2(&mut texpdag::Ext, exp) });
        melee_world::tu::ground::ext::hook::HSD_ForeachAnim = Some(|a0, a1, a2, a3, a4, a5, a6| unsafe { foreach_anim_fu(a0, a1, a2, a3, a4, a5, a6) });
        // ftanim.c: HSD_ForeachAnim(jobj, JOBJ_TYPE, mask, HSD_AObjSetFlags, AOBJ_ARG_AU, AOBJ_LOOP) and the AF rate form
        melee_fighter::tu::ftanim::ext::hook::HSD_ForeachAnim_SetLoop = Some(|j, mask| unsafe {
            melee_runtime::tu::aobj::HSD_ForeachAnim(
                j as *mut c_void, JOBJ_TYPE as _, mask as _,
                melee_runtime::tu::aobj::HSD_AObjSetFlags as usize as *mut c_void,
                AOBJ_ARG_AU as _, CallbackArg { d: AOBJ_LOOP as u32 },
            )
        });
        melee_fighter::tu::ftanim::ext::hook::HSD_ForeachAnim_SetRate = Some(|j, mask, rate| unsafe {
            melee_runtime::tu::aobj::HSD_ForeachAnim(
                j as *mut c_void, JOBJ_TYPE as _, mask as _,
                melee_runtime::tu::aobj::HSD_AObjSetRate as usize as *mut c_void,
                AOBJ_ARG_AF as _, CallbackArg { f: rate },
            )
        });
    }
}

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
        // efasync.c's `efSync_Spawn(id, gobj, ...)` passes its vararg words as a slice; efsync.c reads them through its own `Va`
        melee_world::tu::efn::efasync::ext::hook::efSync_Spawn = Some(|a0, a1, a2| unsafe { melee_world::tu::efsync::efSync_Spawn(a0, a1, &mut melee_world::tu::efsync::Va::new(core::slice::from_raw_parts(a2.as_ptr() as *const *mut core::ffi::c_void, a2.len()))) });
        // fighter.c: hooks spelled differently from their C functions (accessor / out-parameter / return-ness)
        {
            use melee_fighter::tu::fighter::ext::hook as f;
            f::HSD_JObjRemoveAll = Some(|j| unsafe { melee_runtime::tu::jobj::HSD_JObjRemoveAll_ptr(j as *mut melee_types::HSD_JObj) });
            f::stage_grkind = Some(|| unsafe { melee_world::tu::ground::stage_info.grkind as i32 });
            f::ftData_MotionStateList = Some(|| unsafe { (&raw mut crate::retail_ms::ftmotionstates::ftData_MotionStateList) as *mut melee_types::MotionState });
            f::pad_nml = Some(|port| unsafe {
                let s = &melee_runtime::tu::controller::HSD_PadGameStatus[port as usize];
                [s.nml_stickX, s.nml_stickY, s.nml_subStickX, s.nml_subStickY, s.nml_analogL, s.nml_analogR]
            });
            f::pad_button = Some(|port| unsafe { melee_runtime::tu::controller::HSD_PadGameStatus[port as usize].button });
            // `Fighter_LoadCommonData`: lbArchive_LoadSymbols("PlCo.dat", &pData, "ftLoadCommonData", 0)
            f::lbArchive_LoadSymbols_PlCo = Some(|| unsafe {
                static mut PDATA: *mut c_void = core::ptr::null_mut();
                melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(
                    c"PlCo.dat".as_ptr(),
                    &[(&raw mut PDATA) as *mut c_void, c"ftLoadCommonData".as_ptr() as *mut c_void, core::ptr::null_mut()],
                );
                PDATA as *mut *mut c_void
            });
            f::HSD_GObj_SetupProc = Some(|g, func, prio| unsafe {
                // a fighter proc is a plain `fn(*mut HSD_GObj)`: the same call on the x64 targets
                let _ = melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(g, core::mem::transmute(Some(func)), prio);
            });
        }
        {
            use melee_fighter::tu::ftchangeparam::ext as c;
            // `ftKindCalcIndiviParamTable[kind]` (ftdata.c) is the per-character attribute re-application; the
            // m-ex dispatcher hook runs the vanilla entry when no extension row owns the kind
            c::ftKindCalcIndiviParamTable_present = Some(|k| unsafe { melee_fighter::tu::ftdata::ftKindCalcIndiviParamTable[k as usize].is_some() });
            c::Mex_OnReapplyAttrDispatch = Some(|k, g| unsafe {
                if let Some(f) = melee_fighter::tu::ftdata::ftKindCalcIndiviParamTable[k as usize] {
                    f(g);
                }
            });
        }
        // ftcolanim.c reads ftdata.c's table through its own pointer-typed `ext` static
        melee_fighter::tu::ftcolanim::ext::ftData_UnkMotionStates4 = (&raw const melee_fighter::tu::ftdata::ftData_UnkMotionStates4) as *const *mut c_void;
        // ftdata.c: the kind tables' file loads (variadic lbArchive_80017040 spread over fixed slots) and the helpers
        // the stubs! block of tu/ftdata.rs names differently from their C functions
        {
            use melee_fighter::tu::ftdata::ext as e;
            e::LBARCHIVE_80017040_FT = Some(|out, file, data, sym| unsafe {
                melee_runtime::tu::lbarchive::lbArchive_80017040(out, file, &[data as *mut c_void, sym as *mut c_void, core::ptr::null_mut()]);
            });
            e::LBARCHIVE_80017040_COSTUME = Some(|out, file, joint, jname, matanim, mname| unsafe {
                melee_runtime::tu::lbarchive::lbArchive_80017040(
                    out,
                    file,
                    &[joint as *mut c_void, jname as *mut c_void, matanim as *mut c_void, mname as *mut c_void, core::ptr::null_mut()],
                );
            });
            // the guest address model: the banks live in the engine's low-memory heaps, so an address is the pointer
            e::GC_ADDR32 = Some(|p| {
                assert!((p as usize) <= u32::MAX as usize, "gc_addr32: {p:p} is not a guest (low memory) address");
                p as usize as u32
            });
            e::GC_PTR = Some(|a| a as usize as *mut c_void);
            // lbArq_80014BD0(src, dst, size, 0, 0): the C passes no completion callback here
            e::LBARQ_80014BD0 = Some(|src, dst, size, cb, arg| unsafe {
                assert!(cb == 0 && arg == 0, "ftdata.c passes no ARQ callback");
                melee_runtime::tu::lbarq::lbArq_80014BD0(src, dst as *mut c_void, size as usize, None, core::ptr::null_mut());
            });
            e::LBFILE_800168A0 = Some(|heap, name, out, size| unsafe {
                let _ = melee_runtime::tu::lbfile::lbFile_800168A0(heap, name, out as *mut *mut c_void, size as *mut usize);
            });
            e::EF_BANK_MAX_ = Some(|| 65);
            e::HSD_JOBJ_GET_TRANSLATION_PTR = Some(|j| unsafe { melee_types::Vec3 { x: (*j).translate.x, y: (*j).translate.y, z: (*j).translate.z } });
            e::PLINK_FIGHTER_HEAD = Some(|| unsafe { *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(melee_types::HSD_GOBJ_PLINK_FIGHTER as usize) });
            e::HSD_OBJ_ALLOC_X59C = Some(|| unsafe {
                melee_runtime::tu::objalloc::HSD_ObjAlloc(&raw mut melee_fighter::tu::fighter::fighter_x59C_alloc_data) as *mut melee_types::Fighter_x59C_t
            });
        }
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

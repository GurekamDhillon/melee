//! src/melee/ft/kinds/ftCommon/ftCo_09F7.c: `ftCo_8009F834`, the fighter effect spawner behind the
//! `ftAction` effect command (queues an `efAsync` effect on a body part, with the C's RNG draws for the
//! randomised offset), and `ftCo_800A0098`, the cmd-script defaults. Every effect goes through the
//! canonical `melee_world::tu::efn::efasync` queue (variadic tail passed as words, `Va`). Calls
//! `HSD_Randf` on the game's real RNG. Float sites: `2*r*(rand-0.5)+off` has no fused multiply-add
//! the compiler contracts except the `spA8.x += ...` adds (fmadds, see `fma`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_world::tu::efn::efasync as ef;

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn w<T>(p: *const T) -> *const c_void { p as *const c_void }

#[inline(always)]
unsafe fn rnd() -> f32 { melee_runtime::tu::random::global::HSD_Randf() }

/// `dst += 2 * range * (rand - 0.5)` for one axis: clang contracts the add into `fmadds(2*range, rand-0.5, dst)`.
#[inline(always)]
unsafe fn spread(dst: f32, range: f32) -> f32 {
    let r = rnd();
    fmadds(2.0f32 * range, r - 0.5f32, dst)
}

#[inline(always)]
unsafe fn floor_angle(fp: &Fighter) -> f32 {
    if fp.ground_or_air == GA_Ground as _ {
        melee_runtime::lbtrigf::atan2f(-fp.coll_data.floor.normal.x, fp.coll_data.floor.normal.y)
    } else {
        0.0
    }
}

/// `ftCo_8009F834(gobj, gfx_id, part, use_common_bone_id, destroy_on_state_change, offset, range, unused)`.
pub unsafe fn ftCo_8009F834(gobj: G, arg1: i32, mut part: i32, arg3: i32, arg4: i32, arg5: *const Vec3, arg6: *const Vec3, _arg7: f32) {
    melee_core::ct!("ftCo_8009F834");
    let gfx_id = arg1;
    let fp = &mut *((*gobj).user_data as *mut Fighter);
    if arg4 != 0 {
        fp.set_x2219_b0(1);
    }
    if part == 0x8D {
        let i = fp.x2220_b0() as isize;
        part = *(*fp.ft_data).x54.offset(i);
        let n = (fp.x2220_b0() + 1) & 7;
        fp.set_x2220_b0(n);
        if n >= 5 {
            fp.set_x2220_b0(0);
        }
    } else if part == 0x8E {
        part = (*(*fp.ft_data).x8).x10 as i32;
    } else if arg3 != 0 {
        part = crate::ftdmgfall::ftparts_get_bone_index(fp, part);
    }
    // TARGET_PC: a Geno "none" define has no joint for some donor parts.
    if crate::platform::Geno_DefineIsNone(fp.kind as i32) != 0 {
        let t = *crate::hooks::ft_parts_table().offset(fp.kind as isize);
        if (part as u32) >= (*t).parts_num as u32 || (*fp.parts.add(part as usize)).joint.is_null() {
            return;
        }
    }
    let q = &raw mut fp.x60C as *mut c_void;
    let joint = |fp: &Fighter| (*fp.parts.add(part as usize)).joint;
    let spawn = |kind: u32, gfx: u32, j: *mut HSD_JObj, args: &[*const c_void]| ef::efAsync_Spawn(gobj, q, kind, gfx, j, args);

    if gfx_id < 0x250 || gfx_id / 1000 == 0x1E {
        let mut a = *arg5;
        a.x = spread(a.x, (*arg6).x);
        a.y = spread(a.y, (*arg6).y);
        a.z = spread(a.z, (*arg6).z);
        spawn(2, gfx_id as u32, joint(fp), &[w(&a)]);
        return;
    }
    match gfx_id {
        0x402 | 0x403 | 0x409 | 0x412 | 0x413 | 0x414 | 0x422 | 0x487 | 0x4D1 | 0x4E5 | 0x4E6 | 0x4FE | 0x500 | 0x501 | 0x502 => {
            spawn(0, gfx_id as u32, joint(fp), &[]);
        }
        0x446 | 0x448 | 0x4E1 => spawn(7, gfx_id as u32, joint(fp), &[w(arg5)]),
        0x415 | 0x41E | 0x428 | 0x429 | 0x42A => {
            let attrs = &raw const (*(*fp.ft_data).x0).x168;
            spawn(3, gfx_id as u32, joint(fp), &[w(attrs)]);
        }
        0x495 => spawn(3, gfx_id as u32, joint(fp), &[w(&raw const fp.facing_dir)]),
        0x49D => {
            let sp9c = *arg5;
            let sp98 = 0.017453292f32 * (256.0f32 * (*arg6).x);
            spawn(6, gfx_id as u32, joint(fp), &[w(&sp9c), w(&raw const fp.facing_dir), w(&sp98)]);
        }
        0x438 | 0x439 => {
            let sp94 = fp.x34_scale.y * fp.co_attrs.x144;
            spawn(3, 0x438, joint(fp), &[w(&sp94)]);
        }
        0x423 | 0x424 | 0x4C1 | 0x4C2 | 0x4C4 | 0x4C5 => {
            let sp90 = floor_angle(fp);
            spawn(3, gfx_id as u32, joint(fp), &[w(&sp90)]);
        }
        _ => {
            let mut a = *arg5;
            a.x = spread(a.x, (*arg6).x);
            a.y = spread(a.y, (*arg6).y);
            a.z = spread(a.z, (*arg6).z);
            match gfx_id {
                0x3E8 => {
                    let data = melee_runtime::shared::p_ftCommonData;
                    spawn(5, gfx_id as u32, joint(fp), &[w(&a), w(&raw const (*data).fall_common.x564)]);
                }
                0x3E9 | 0x3EA | 0x3EB | 0x3EC | 0x3F3 | 0x3F4 | 0x3F6 | 0x3FA | 0x3FB | 0x3FC | 0x405 | 0x407 | 0x40C | 0x40D | 0x40E
                | 0x40F | 0x410 | 0x411 | 0x416 | 0x41C | 0x41D | 0x41F | 0x421 | 0x425 | 0x426 | 0x44B | 0x4E2 | 0x4E3 | 0x4E4 => {
                    spawn(2, gfx_id as u32, joint(fp), &[w(&a)]);
                }
                0x404 | 0x406 => {
                    let sp80 = floor_angle(fp);
                    spawn(5, gfx_id as u32, joint(fp), &[w(&a), w(&sp80)]);
                }
                0x513 => spawn(8, 2, joint(fp), &[w(&a)]),
                0x514 => spawn(8, 3, joint(fp), &[w(&a)]),
                0x515 => spawn(8, 4, joint(fp), &[w(&a)]),
                0x3ED | 0x3EF | 0x3F0 | 0x3F1 | 0x3F2 | 0x3F5 | 0x3FE | 0x400 | 0x401 | 0x4D9 => {
                    spawn(5, gfx_id as u32, joint(fp), &[w(&a), w(&raw const fp.facing_dir)]);
                }
                0x3F7 | 0x3F8 | 0x3F9 | 0x3FD | 0x3FF => {
                    let sp7c = floor_angle(fp);
                    spawn(6, gfx_id as u32, joint(fp), &[w(&a), w(&raw const fp.facing_dir), w(&sp7c)]);
                }
                _ => {
                    if melee_world::tu::efasync::EF_MEX_IS_CUSTOM(gfx_id) {
                        let orientation = floor_angle(fp);
                        ef::efAsync_MexSpawn(gobj, q, gfx_id, joint(fp), &mut a, fp.facing_dir, orientation);
                    }
                    // else: OSReport("no effect from animlist %d\n")
                }
            }
        }
    }
}

/// `ftCo_800A0098`: the default command-script setup.
pub unsafe fn ftCo_800A0098(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0098");
    use crate::tu::ftcmdscript::{ftCo_800B463C, ftCo_800B46B8};
    ftCo_800B46B8(fp, 0x80, 0);
    ftCo_800B46B8(fp, 0x81, 0);
    ftCo_800B46B8(fp, 0x8E, 1);
    ftCo_800B46B8(fp, 0x91, 0xB0);
    ftCo_800B46B8(fp, 0x8E, 1);
    ftCo_800B46B8(fp, 0x81, 0);
    ftCo_800B46B8(fp, 0x8E, 0xA);
    ftCo_800B46B8(fp, 0x80, 0);
    ftCo_800B463C(fp, 0x7F);
}

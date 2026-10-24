//! src/melee/ft/ftwaitanim.c: idle ("wait") animation selection: random weighted pick from a
//! wait table when the current loop ends, and the motion swap itself.
#![allow(non_snake_case)]

use crate::types::*;

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    pub fn ftData_80085CD8(fp: *mut Fighter, fp2: *mut Fighter, anim_id: i32) { { unsafe { let _ = crate::tu::ftdata::ftData_80085CD8(fp as _, fp2 as _, anim_id as _); } return; } }
    pub fn ftCo_8009E7B4(fp: *mut Fighter, blend: *mut [u8; 2]) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E7B4(&mut *(fp as *mut _), blend as _); } }
    pub fn ftAnim_8006EBE8(gobj: *mut HSD_GObj, a: f32, b: f32, c: f32) { { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBE8(gobj as _, a as _, b as _, c as _); } return; } }
    pub fn ftAnim_8006EBA4(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(gobj as _); } return; } }
    pub fn ftAnim_IsFramesRemaining(gobj: *mut HSD_GObj) -> bool { { return unsafe { crate::tu::ftanim::ftAnim_IsFramesRemaining(gobj as _) as _ }; } }
    pub fn HSD_Randi(max: i32) -> i32 { unsafe { crate::rng::HSD_Randi(max) } }
    pub fn itGetHoldKind(item: *mut Item_GObj) -> i32 {
        unsafe { melee_items::itlib::it_get_hold_kind(&*((*item).user_data as *const Item)) }
    }
}

use super::ftlipstickswing::fp;

/// `ftCo_8008A698(fp)`: holding an item that is not hold kind 2.
pub fn ftCo_8008A698(f: &Fighter) -> bool {
    !f.item_gobj.is_null() && ext::itGetHoldKind(f.item_gobj) != 2
}

/// `ftCo_8008A6D8(gobj, anim_id)`: switch to wait animation `anim_id` (-1: none).
pub fn ftCo_8008A6D8(gobj: *mut HSD_GObj, anim_id: i32) {
    if anim_id != -1 {
        set_wait_anim(gobj, anim_id);
    }
}

/// The body shared by ftCo_8008A6D8 and the manually inlined copy in ftCo_8008A7A8.
fn set_wait_anim(gobj: *mut HSD_GObj, anim_id: i32) {
    let f = fp(gobj);
    let anim = unsafe { &*f.x24.offset(anim_id as isize) };
    let blend = unsafe { f.x28.offset(anim_id as isize) };
    ext::ftData_80085CD8(f, f, anim_id);
    f.anim_id = anim_id;
    ext::ftCo_8009E7B4(f, blend);
    f.x3E4_fighterCmdScript.anon_8.u = anim.xC;
    f.x3E4_fighterCmdScript.loop_count = 0;
    if !f.x590.is_null() {
        // `fp->x594_s32 = ...`: a whole s32 over the bitfield word, GC byte order.
        f.anon_594.anon_0_0.bf_0 = (anim.x10_animCurrFlags as u32).to_be_bytes();
        ext::ftAnim_8006EBE8(gobj, 0.0, 1.0, unsafe { (*blend)[0] } as f32);
    }
    f.x3E4_fighterCmdScript.timer = 0.0;
    ext::ftAnim_8006EBA4(gobj);
}

/// static inline `inlineA0(fp)`: anim 2 or 31 may repeat.
fn inline_a0(f: &Fighter) -> bool {
    f.anim_id == 2 || f.anim_id == 31
}

/// static inline `getAnimID(table)`: weighted pick, `HSD_Randi(100) + 1` against the running
/// sum of `y` until the `x == -1` terminator (running off it is the C's fatal assert).
pub fn get_anim_id_with(mut table: *const WaitStruct, roll: i32) -> i32 {
    let max = roll + 1;
    let mut count = 0i32;
    unsafe {
        while (*table).u.i.x != -1 {
            count = count.wrapping_add((*table).u.i.y);
            if max <= count {
                // `(enum_t) u.p.x`: on the GC the pointer and int members are the same word.
                return (*table).u.i.x;
            }
            table = table.add(1);
        }
    }
    panic!("wait anim data illegal!! {max}");
}

/// `ftCo_8008A7A8(gobj, table)`: at the end of the wait loop, replay (holding an item, unless
/// Mewtwo or Fox-template) or pick a different wait animation from `table`.
pub fn ftCo_8008A7A8(gobj: *mut HSD_GObj, arg1: *const WaitStruct) {
    let f = fp(gobj);
    if ext::ftAnim_IsFramesRemaining(gobj) {
        return;
    }
    if arg1.is_null()
        || (!f.item_gobj.is_null()
            && f.kind != Ft_Kind_Mewtwo
            && crate::ft0892::template_kind(f) != Ft_Kind_Fox as i32)
    {
        ftCo_8008A6D8(gobj, f.anim_id);
        return;
    }
    let mut temp;
    loop {
        temp = get_anim_id_with(arg1, ext::HSD_Randi(100));
        if inline_a0(f) || f.anim_id != temp {
            break;
        }
    }
    if temp != -1 {
        set_wait_anim(gobj, temp);
    }
}

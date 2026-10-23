//! Native port of src/melee/it/itanimlist.c: the item animation-script commands (opcodes 10..25 of
//! an item's `x524_cmd` stream: effects, hitboxes, throw flags, colour overlays), the dispatcher
//! `it_802799E4` and the colour-overlay helpers. Callees in other TUs go through `ext`. Checked
//! against the C (the whole TU, shipping contraction) by tests/tu_itanimlist.rs.
//!
//! Script model. A command word is the 4 raw file bytes, big-endian, in the first 4 bytes of one
//! `CmdUnion` element (`cmd->u` steps by `size_of::<CmdUnion>()`, 8 natively). Bitfield members are
//! read through melee_types' GameCube-order accessors, which decode those bytes MSB first. The C's
//! raw `*(u16*)` / `*(s16*)` / `*(u32*)` reads of a word are big-endian reads of the same bytes
//! (what the PPC build sees), so [`rd_u16`], [`rd_s16`] and [`rd_u32`] swap; single bytes are
//! read as they are. The one place the C steps by something other than a word is
//! `it_8027978C`'s `(CmdUnion*) (ptr + 1)` with `ptr` an `itAnimlistCmdUnk*` (4 bytes): on the
//! console that is the next word, so it is `u.add(1)` here (the oracle compiles the C with that one
//! expression replaced the same way).
//!
//! Float math: `0.003906f * int` products and `xC3C * (damage * xC40)` only; no fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Command_Execute(a0: *mut CommandInfo, a1: u32) -> i32;
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026AF0C(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026AFA0(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026B034(a0: *mut Item);
    fn Item_8026B074(a0: *mut Item);
    fn ScriptGame_HitRuleForget(a0: *mut HitCapsule);
    fn ScriptGame_HitRuleItemCreate(a0: *mut Item, a1: *mut HitCapsule);
    fn it_8026FCF8(a0: *mut Item, a1: *mut HitCapsule);
    fn it_8027129C(a0: *mut HSD_GObj, a1: i32);
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj);
    fn it_80272560(a0: *mut HSD_GObj, a1: i32);
    fn it_802725D4(a0: *mut HSD_GObj);
    fn it_80273598(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_80273600(a0: *mut HSD_GObj);
    fn it_80273648(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_80275594(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn it_80278800(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: *mut Vec, a4: *mut Vec, a5: i32, a6: f32);
    fn lb_80014258(a0: *mut HSD_GObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut CommandInfo, i32)>) -> i32;
    fn lb_80014498(a0: *mut ColorOverlay);
    fn lb_800144C8(a0: *mut ColorOverlay, a1: *mut Fighter_804D653C_t, a2: i32, a3: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D04: *mut Fighter_804D653C_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `F32_MAX`
pub const F32_MAX: f32 = 3.4028235e38;

/// `typedef struct itAnimlistCmdUnk` (u16 bitfields, GameCube order: x0_b0:6, opcode:8, x0_b14:2, x2:16).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itAnimlistCmdUnk {
    pub bf_0: [u8; 2],
    pub x2: u16,
}

impl itAnimlistCmdUnk {
    /// `opcode:8`
    #[inline]
    pub fn opcode(&self) -> u32 {
        ((u16::from_be_bytes(self.bf_0) >> 2) & 0xFF) as u32
    }
}

/// Big-endian `((u16*) u)[k]`.
#[inline]
pub unsafe fn rd_u16(u: *const CmdUnion, k: usize) -> u16 {
    let b = u as *const u8;
    u16::from_be_bytes([*b.add(2 * k), *b.add(2 * k + 1)])
}
/// Big-endian `((s16*) u)[k]`.
#[inline]
pub unsafe fn rd_s16(u: *const CmdUnion, k: usize) -> i16 {
    rd_u16(u, k) as i16
}
/// Big-endian `*(u32*) u`.
#[inline]
pub unsafe fn rd_u32(u: *const CmdUnion) -> u32 {
    let b = u as *const u8;
    u32::from_be_bytes([*b, *b.add(1), *b.add(2), *b.add(3)])
}
/// `((u8*) u)[k]`.
#[inline]
pub unsafe fn rd_u8(u: *const CmdUnion, k: usize) -> u8 {
    *(u as *const u8).add(k)
}

#[inline]
unsafe fn item_of(g: *mut Item_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

#[inline]
unsafe fn next_cmd(cmd: *mut CommandInfo) {
    (*cmd).anon_8.u = (*cmd).anon_8.u.add(1);
}

/// `ItCmd it_803F22A8[16]`: the handler of opcode `10 + i`.
pub static it_803F22A8: [ItCmd; 16] = [
    Some(it_80278F2C),
    Some(it_802790C0),
    Some(it_80279544),
    Some(it_802795EC),
    Some(it_80279680),
    Some(it_802796C4),
    Some(it_8027978C),
    Some(it_802796FC),
    Some(it_80279720),
    Some(it_80279744),
    Some(it_80279768),
    Some(it_80279888),
    Some(it_802798D4),
    Some(it_8027990C),
    Some(it_80279958),
    Some(it_802799A8),
];

/// `static ItCmd it_804D51C8[2]` (4D51C8)
pub static it_804D51C8: [ItCmd; 2] = [Some(it_80279AF0), Some(it_80279B10)];

/// `void it_80278F2C(Item_GObj*, CommandInfo*)`: spawn an item effect from five command words.
pub unsafe extern "C" fn it_80278F2C(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let mut sp20 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp14 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut arg2: i32 = rd_u16((*cmd).anon_8.u, 0) as i32;
    arg2 &= 0x3FF;
    next_cmd(cmd);
    let arg6: i32 = (rd_u16((*cmd).anon_8.u, 1) as f32) as i32;
    let ef_id = rd_u16((*cmd).anon_8.u, 0);
    next_cmd(cmd);
    sp20.x = 0.003906f32 * rd_s16((*cmd).anon_8.u, 0) as f32;
    sp20.y = 0.003906f32 * rd_s16((*cmd).anon_8.u, 1) as f32;
    next_cmd(cmd);
    sp20.z = 0.003906f32 * rd_s16((*cmd).anon_8.u, 0) as f32;
    sp14.x = 0.003906f32 * rd_s16((*cmd).anon_8.u, 1) as f32;
    next_cmd(cmd);
    sp14.y = 0.003906f32 * rd_s16((*cmd).anon_8.u, 0) as f32;
    sp14.z = 0.003906f32 * rd_s16((*cmd).anon_8.u, 1) as f32;
    next_cmd(cmd);
    ext::it_80278800(item_gobj, ef_id as i32, arg2, &mut sp20, &mut sp14, 0, arg6 as f32);
}

/// `void it_802790C0(Item_GObj*, CommandInfo*)`: (re)create hitbox `id` from six command words.
pub unsafe extern "C" fn it_802790C0(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let item = item_of(item_gobj);
    let u = (*cmd).anon_8.u;
    let hitbox_idx = (*u).it_create_hitbox_0.id();
    let x4 = (*u).it_create_hitbox_0.hit_group();
    let hb = &raw mut (*item).x5D4_hitboxes[hitbox_idx as usize];
    let hit = &raw mut (*hb).hit;

    if (*hit).state == HitCapsule_Disabled || (*hit).x4 != x4 {
        (*hit).x4 = x4;
        (*hit).state = HitCapsule_Enabled;
        (*item).xDC8_word.flags.set_x16(1);
        (*item).anon_daa.xDAA_flag.anon_0.set_b2(0);
        ext::it_8026FCF8(item, hit);
    }

    let bone_idx = (*(*cmd).anon_8.u).it_create_hitbox_0.bone();
    if bone_idx != 0 {
        if (*item).xBBC_dynamicBoneTable.is_null() {
            panic!("itanimlist.c:0x8B: item can't set attack!");
        }
        (*hit).jobj = (*(*item).xBBC_dynamicBoneTable).bones[bone_idx as usize];
    } else {
        (*hit).jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    }
    ext::ScriptGame_HitRuleForget(hit);
    let dmg = (*(*cmd).anon_8.u).it_create_hitbox_0.damage();
    ext::it_80272460(hit, ((*item).xC3C * (dmg as f32 * (*item).xC40)) as u32, item_gobj);
    next_cmd(cmd);

    (*hit).scale = 0.003906f32 * (*(*cmd).anon_8.u).create_hitbox_1.size() as f32;
    (*item).x3C = (*hit).scale;
    ext::it_80275594(item_gobj, hitbox_idx as i32, 1.0f32 / (*item).scl);
    (*hit).b_offset.x = 0.003906f32 * (*(*cmd).anon_8.u).create_hitbox_1.z_offset() as f32;
    next_cmd(cmd);
    (*hit).b_offset.y = 0.003906f32 * (*(*cmd).anon_8.u).create_hitbox_2.y_offset() as f32;
    (*hit).b_offset.z = 0.003906f32 * (*(*cmd).anon_8.u).create_hitbox_2.x_offset() as f32;
    next_cmd(cmd);

    (*hit).kb_angle = (*(*cmd).anon_8.u).create_hitbox_3.angle() as i32;
    (*hit).x24 = (*(*cmd).anon_8.u).create_hitbox_3.knockback_growth();
    (*hit).x28 = (*(*cmd).anon_8.u).create_hitbox_3.weight_set_knockback();
    (*hit).set_x43_b1(0);
    next_cmd(cmd);

    let w4 = &(*(*cmd).anon_8.u).it_create_hitbox_4;
    (*hit).x2C = w4.base_knockback();
    (*hit).element = w4.element();
    (*hit).set_x40_b0(w4.x40_b0() as _);
    (*hit).set_x40_b1(0);
    (*hit).x34 = w4.shield_damage();
    (*hit).sfx_severity = w4.sfx_severity() as i32;
    (*hit).sfx_kind = w4.sfx_kind() as _;
    (*hit).set_x40_b2(w4.x40_b2() as _);
    (*hit).set_x40_b3(w4.x40_b3() as _);
    next_cmd(cmd);

    let w5 = &(*(*cmd).anon_8.u).create_hitbox_5;
    (*hit).set_x40_b4(w5.x0() as _);
    (*hit).set_x41_b4(w5.x1_b0() as _);
    (*hit).set_x41_b5(w5.x1_b1() as _);
    (*hit).set_x41_b6(w5.x1_b2() as _);
    (*hit).set_x41_b7(w5.x1_b3() as _);
    (*hit).set_x42_b0(w5.x1_b4() as _);
    (*hit).set_x42_b1(w5.x1_b5() as _);
    (*hit).set_x42_b2(w5.x1_b6() as _);
    (*hit).set_x42_b3(w5.x1_b7() as _);
    let b2 = rd_u8((*cmd).anon_8.u, 2);
    (*hit).set_x42_b4((b2 >> 7) & 1);
    (*hit).set_x42_b5((b2 >> 6) & 1);
    (*hit).set_x42_b6((b2 >> 5) & 1);
    (*hit).set_x42_b7((b2 >> 4) & 1);
    (*hit).set_x43_b0((b2 >> 3) & 1);
    (*hb).set_x138((b2 >> 2) & 1);
    next_cmd(cmd);

    (*hit).set_x43_b2(0);
    ext::ScriptGame_HitRuleItemCreate(item, hit);
    if !ext::HSD_GObj_CurrentInvokedProc.is_null() && (*ext::HSD_GObj_CurrentInvokedProc).s_link > 11 {
        ext::it_8027129C(item_gobj, hitbox_idx as i32);
    }
}

/// `void it_80279544(Item_GObj*, CommandInfo*)`: set a hitbox's damage (13-bit raw value).
pub unsafe extern "C" fn it_80279544(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let item = item_of(item_gobj);
    let hit = &raw mut (*item).x5D4_hitboxes[(*(*cmd).anon_8.u).set_hitbox_damage.idx() as usize].hit;
    let val = (rd_u16((*cmd).anon_8.u, 1) & 0x1FFF) as u32;
    ext::it_80272460(hit, ((*item).xC3C * (val as f32 * (*item).xC40)) as u32, item_gobj);
    next_cmd(cmd);
}

/// `void it_802795EC(Item_GObj*, CommandInfo*)`: set a hitbox's size.
pub unsafe extern "C" fn it_802795EC(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let idx = (*(*cmd).anon_8.u).set_hitbox_scale.idx() as i32;
    let item = item_of(item_gobj);
    let hit = &raw mut (*item).x5D4_hitboxes[idx as usize].hit;
    (*hit).scale = 0.003906f32 * (*(*cmd).anon_8.u).set_hitbox_scale.value() as f32;
    (*item).x3C = (*hit).scale;
    ext::it_80275594(item_gobj, idx, 1.0f32 / (*item).scl);
    next_cmd(cmd);
}

/// `void it_80279680(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_80279680(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    ext::it_80272560(item_gobj, (*(*cmd).anon_8.u).set_throw_flags.hit_idx() as i32);
    next_cmd(cmd);
}

/// `void it_802796C4(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_802796C4(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    ext::it_802725D4(item_gobj);
    next_cmd(cmd);
}

/// `void it_802796FC(Item_GObj*, CommandInfo*)`: `xDAC_itcmd_var0 = hit_idx`.
pub unsafe extern "C" fn it_802796FC(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    (*item_of(item_gobj)).xDAC_itcmd_var0 = (*(*cmd).anon_8.u).set_throw_flags.hit_idx();
    next_cmd(cmd);
}

/// `void it_80279720(Item_GObj*, CommandInfo*)`: the existing port in `itkinds14`, C ABI.
pub unsafe extern "C" fn it_80279720(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    crate::itkinds14::it_80279720(&mut *item_of(item_gobj), &mut *cmd);
}

/// `void it_80279744(Item_GObj*, CommandInfo*)`: the existing port in `itkinds14`, C ABI.
pub unsafe extern "C" fn it_80279744(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    crate::itkinds14::it_80279744(&mut *item_of(item_gobj), &mut *cmd);
}

/// `void it_80279768(Item_GObj*, CommandInfo*)`: the existing port in `itkinds14`, C ABI.
pub unsafe extern "C" fn it_80279768(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    crate::itkinds14::it_80279768(&mut *item_of(item_gobj), &mut *cmd);
}

/// `void it_8027978C(Item_GObj*, CommandInfo*)`: colour-overlay command (opcodes 0..2 set an
/// overlay from a following word, 10 / 11 clear one).
pub unsafe extern "C" fn it_8027978C(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let item = item_of(item_gobj);
    let ptr = (*cmd).anon_8.u as *const itAnimlistCmdUnk;
    let opcode = (*ptr).opcode() as i32;
    (*cmd).anon_8.u = (*cmd).anon_8.u.add(1);
    match opcode {
        0 | 1 | 2 => {
            let arg1 = rd_u32((*cmd).anon_8.u);
            next_cmd(cmd);
            let arg2 = rd_u8((*cmd).anon_8.u, 2);
            let arg3 = rd_u8((*cmd).anon_8.u, 3);
            match opcode {
                0 => ext::Item_8026AE84(item, arg1 as i32, arg2, arg3),
                1 => ext::Item_8026AF0C(item, arg1 as i32, arg2, arg3),
                _ => ext::Item_8026AFA0(item, arg1 as i32, arg2, arg3),
            }
        }
        10 | 11 => {
            next_cmd(cmd);
            match opcode {
                10 => ext::Item_8026B034(item),
                _ => ext::Item_8026B074(item),
            }
        }
        _ => {}
    }
    next_cmd(cmd);
}

/// `void it_80279888(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_80279888(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let w = &(*(*cmd).anon_8.u).unk33;
    ext::it_80273598(item_gobj, w.unk0() as i32, w.unk1() as i32);
    next_cmd(cmd);
}

/// `void it_802798D4(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_802798D4(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    ext::it_80273600(item_gobj);
    next_cmd(cmd);
}

/// `void it_8027990C(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_8027990C(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let w = &(*(*cmd).anon_8.u).unk33;
    ext::it_80273648(item_gobj, w.unk0() as i32, w.unk1() as i32);
    next_cmd(cmd);
}

/// `void it_80279958(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_80279958(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    let it = item_of(item_gobj);
    let w = &(*(*cmd).anon_8.u).unk13;
    it_80279B88(it, w.unk1() as i32, w.unk2() as i32);
    next_cmd(cmd);
}

/// `void it_802799A8(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_802799A8(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    it_80279BBC(item_of(item_gobj));
    next_cmd(cmd);
}

/// `void it_802799E4(Item_GObj*)`: runs the item's script: every command whose timer has run out.
pub unsafe extern "C" fn it_802799E4(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    let cmd = &raw mut (*item).x524_cmd;

    (*cmd).frame_count = (*item).x5CC_currentAnimFrame;
    (*item).anon_dbc.xDBC_itcmd_var4_word = 0;

    if (*cmd).anon_8.u.is_null() {
        return;
    }

    if (*cmd).timer != F32_MAX {
        (*cmd).timer -= (*item).x5D0_animFrameSpeed;
    }

    loop {
        if (*cmd).anon_8.u.is_null() {
            return;
        }
        if (*cmd).timer == F32_MAX {
            if (*cmd).frame_count >= (*item).x5D0_animFrameSpeed {
                return;
            }
            (*cmd).timer = -(*cmd).frame_count;
        } else if (*cmd).timer > 0.0 {
            return;
        }

        let mut opcode = (*(*cmd).anon_8.u).unk0.opcode();
        if ext::Command_Execute(cmd, opcode) == 0 {
            opcode = opcode.wrapping_sub(10);
            (it_803F22A8[opcode as usize].unwrap())(item_gobj, cmd);
        }
    }
}

/// `void it_80279AF0(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_80279AF0(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    it_80278F2C(item_gobj, cmd);
}

/// `void it_80279B10(Item_GObj*, CommandInfo*)`
pub unsafe extern "C" fn it_80279B10(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo) {
    it_8027978C(item_gobj, cmd);
}

/// `void fn_80279B30(Item_GObj*, CommandInfo*, int arg2)`: colour-overlay callback (opcodes 21, 22).
pub unsafe extern "C" fn fn_80279B30(item_gobj: *mut Item_GObj, cmd: *mut CommandInfo, arg2: i32) {
    let idx = arg2.wrapping_sub(21);
    (it_804D51C8[idx as usize].unwrap())(item_gobj, cmd);
}

/// `void it_80279B64(Item*)`
pub unsafe extern "C" fn it_80279B64(item: *mut Item) {
    ext::lb_80014498(&raw mut (*item).x548_colorOverlay);
}

/// `void it_80279B88(Item*, s32, s32)`
pub unsafe extern "C" fn it_80279B88(item: *mut Item, arg1: i32, arg2: i32) {
    ext::lb_800144C8(&raw mut (*item).x548_colorOverlay, ext::it_804D6D04, arg1, arg2);
}

/// `void it_80279BBC(Item*)`
pub unsafe extern "C" fn it_80279BBC(item: *mut Item) {
    ext::lb_80014498(&raw mut (*item).x548_colorOverlay);
}

/// `void it_80279BE0(Item_GObj*)`: runs the colour-overlay script until it stops yielding.
pub unsafe extern "C" fn it_80279BE0(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    while ext::lb_80014258(item_gobj, &raw mut (*item).x548_colorOverlay as *mut c_void, Some(fn_80279B30)) != 0 {
        ext::lb_80014498(&raw mut (*item).x548_colorOverlay);
    }
}

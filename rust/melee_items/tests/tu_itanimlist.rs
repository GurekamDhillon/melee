//! melee_items src/tu/itanimlist.rs vs the C (gen/mtu/tu/itanimlist.c: the whole TU, shipping
//! contraction). Written by rust/melee_world/gen/mtu/mk_itanimlist.py from the same description as
//! the C driver; regenerate the expected file with
//! `python rust/melee_world/gen/mtu/run_tu.py itanimlist`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itanimlist as m;
use melee_types::*;
use mtu_harness::*;
// The shared harness world (mtu_harness/world.rs) calls g::fill_*/hash_* that the newer generator emits;
// this older hand-driven TU does not use the world, so give it inert stand-ins.
mod g {
    use super::*;
    pub use super::g_inner::*;
    pub fn fill_HSD_GObj(_p: &mut HSD_GObj) {}
    pub fn hash_HSD_GObj(_p: &HSD_GObj) {}
    pub fn fill_Item(_p: &mut Item) {}
    pub fn hash_Item(_p: &Item) {}
    pub fn fill_Fighter(_p: &mut Fighter) {}
    pub fn hash_Fighter(_p: &Fighter) {}
    pub fn fill_HSD_JObj(_p: &mut HSD_JObj) {}
    pub fn hash_HSD_JObj(_p: &HSD_JObj) {}
    pub fn fill_ItemAttr(_p: &mut ItemAttr) {}
    pub fn hash_ItemAttr(_p: &ItemAttr) {}
    pub fn fill_Article(_p: &mut Article) {}
    pub fn hash_Article(_p: &Article) {}
    pub fn fill_ItemLogicTable(_p: &mut ItemLogicTable) {}
    pub fn hash_ItemLogicTable(_p: &ItemLogicTable) {}
    pub fn fill_DynamicBoneTable(_p: &mut DynamicBoneTable) {}
    pub fn hash_DynamicBoneTable(_p: &DynamicBoneTable) {}
    pub fn fill_CmSubject(_p: &mut CmSubject) {}
    pub fn hash_CmSubject(_p: &CmSubject) {}
    pub fn fill_ItemStateTable(_p: &mut ItemStateTable) {}
    pub fn hash_ItemStateTable(_p: &ItemStateTable) {}
    pub fn fill_ItemStateDesc(_p: &mut ItemStateDesc) {}
    pub fn hash_ItemStateDesc(_p: &ItemStateDesc) {}
    pub fn fill_ItemCommonData(_p: &mut ItemCommonData) {}
    pub fn hash_ItemCommonData(_p: &ItemCommonData) {}
    pub fn fill_CollData(_p: &mut CollData) {}
    pub fn hash_CollData(_p: &CollData) {}
    pub fn fill_SpawnItem(_p: &mut SpawnItem) {}
    pub fn hash_SpawnItem(_p: &SpawnItem) {}
    pub fn fillg_all() {}
}
#[path = "mtu/itanimlist_gen.rs"]
mod g_inner;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITEM: Item = unsafe { core::mem::zeroed() };
static mut JT: HSD_JObj = unsafe { core::mem::zeroed() };
static mut BONES: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut PROCX: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut W: [CmdUnion; 24] = unsafe { core::mem::zeroed() };
static mut CMDX: CommandInfo = unsafe { core::mem::zeroed() };
static mut FD: Fighter_804D653C_t = unsafe { core::mem::zeroed() };
static mut CE_RET: i32 = 0;
static mut GKIND: i32 = 0;

unsafe fn wbytes(w: usize, b0: u32, b1: u32, b2: u32, b3: u32) {
    let p = &raw mut W[w] as *mut u8;
    *p = b0 as u8;
    *p.add(1) = b1 as u8;
    *p.add(2) = b2 as u8;
    *p.add(3) = b3 as u8;
}

/// one typed command (opcode 10 + kind) at W[0], everything else zero
unsafe fn genw(kind: i32) {
    W = core::mem::zeroed();
    GKIND = kind;
    match kind {
        0 => {
            for w in 0..5usize {
                let v = rnd();
                let mut r0 = v >> 24;
                let mut r1 = (v >> 16) & 0xFF;
                let r2 = (v >> 8) & 0xFF;
                let r3 = v & 0xFF;
                if w == 0 {
                    r0 = (10u32 << 2) | (r0 & 3);
                    r1 = (r1 & 0xC0) | 10;
                } else {
                    let mut a = r0 >> 2;
                    if (10..=25).contains(&a) {
                        a += 16;
                    }
                    r0 = (a << 2) | (r0 & 3);
                    r1 = (r1 & 0xC0) | a;
                }
                wbytes(w, r0, r1, r2, r3);
            }
        }
        1 => {
            W[0].it_create_hitbox_0.set_opcode(11);
            W[0].it_create_hitbox_0.set_id(rnd() % 4);
            W[0].it_create_hitbox_0.set_hit_group(rnd() % 8);
            let bone = if rnd() % 3 == 0 { 0 } else { 1 + rnd() % 99 };
            W[0].it_create_hitbox_0.set_bone(bone);
            W[0].it_create_hitbox_0.set_damage(rnd() & 0x1FFF);
            W[1].create_hitbox_1.set_size(rnd() & 0xFFFF);
            W[1].create_hitbox_1.set_z_offset(rnd() as u16 as i16 as i32);
            W[2].create_hitbox_2.set_y_offset(rnd() as u16 as i16 as i32);
            W[2].create_hitbox_2.set_x_offset(rnd() as u16 as i16 as i32);
            W[3].create_hitbox_3.set_angle(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_knockback_growth(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_weight_set_knockback(rnd() & 0x1FF);
            W[3].create_hitbox_3.set_item_hit_interaction(rnd() & 1);
            W[3].create_hitbox_3.set_ignore_thrown_fighters(rnd() & 1);
            W[3].create_hitbox_3.set_ignore_fighter_scale(rnd() & 1);
            W[3].create_hitbox_3.set_clank(rnd() & 1);
            W[3].create_hitbox_3.set_rebound(rnd() & 1);
            W[4].it_create_hitbox_4.set_base_knockback(rnd() & 0x1FF);
            W[4].it_create_hitbox_4.set_element(rnd() & 0x1F);
            W[4].it_create_hitbox_4.set_x40_b0(rnd() & 1);
            W[4].it_create_hitbox_4.set_shield_damage(rnd() as u8 as i8 as i32);
            W[4].it_create_hitbox_4.set_sfx_severity(rnd() & 7);
            W[4].it_create_hitbox_4.set_sfx_kind(rnd() & 0xF);
            W[4].it_create_hitbox_4.set_x40_b3(rnd() & 1);
            W[4].it_create_hitbox_4.set_x40_b2(rnd() & 1);
            W[5].create_hitbox_5.set_x0(rnd() & 0xFF);
            W[5].create_hitbox_5.set_x1_b0(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b1(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b2(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b3(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b4(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b5(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b6(rnd() & 1);
            W[5].create_hitbox_5.set_x1_b7(rnd() & 1);
            *(&raw mut W[5] as *mut u8).add(2) = (rnd() & 0xFF) as u8;
        }
        2 => {
            W[0].set_hitbox_damage.set_opcdoe(12);
            W[0].set_hitbox_damage.set_idx(rnd() % 4);
            let hv = rnd() & 0xFFFF;
            let p = &raw mut W[0] as *mut u8;
            *p.add(2) = (hv >> 8) as u8;
            *p.add(3) = (hv & 0xFF) as u8;
        }
        3 => {
            W[0].set_hitbox_scale.set_opcode(13);
            W[0].set_hitbox_scale.set_idx(rnd() % 4);
            W[0].set_hitbox_scale.set_value(rnd() & 0x7FFFFF);
        }
        4 => {
            W[0].set_throw_flags.set_opcode(14);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        5 => W[0].set_throw_flags.set_opcode(15),
        6 => {
            const OPS: [u32; 7] = [0, 1, 2, 10, 11, 3, 200];
            let op8 = OPS[(rnd() % 7) as usize];
            let b14 = rnd() & 3;
            let x2 = rnd() & 0xFFFF;
            let h = (16u32 << 10) | (op8 << 2) | b14;
            wbytes(0, h >> 8, h & 0xFF, x2 >> 8, x2 & 0xFF);
            let v = rnd();
            let v = {
                let mut a = v >> 26;
                if (10..=25).contains(&a) {
                    a += 16;
                }
                (v & 0x03FF_FFC0) | (a << 26) | a
            };
            wbytes(1, v >> 24, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
            let v = rnd();
            let v = {
                let mut a = v >> 26;
                if (10..=25).contains(&a) {
                    a += 16;
                }
                (v & 0x03FF_FFFF) | (a << 26)
            };
            wbytes(2, v >> 24, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
        }
        7 => {
            W[0].set_throw_flags.set_opcode(17);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        8 => {
            W[0].set_throw_flags.set_opcode(18);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        9 => {
            W[0].set_throw_flags.set_opcode(19);
            W[0].set_throw_flags.set_hit_idx(rnd() & 0x3FFFFFF);
        }
        10 => W[0].set_throw_flags.set_opcode(20),
        11 => {
            W[0].unk33.set_opcode(21);
            W[0].unk33.set_unk0(rnd() & 0x1FFF);
            W[0].unk33.set_unk1(rnd() & 0x1FFF);
        }
        12 => W[0].set_throw_flags.set_opcode(22),
        13 => {
            W[0].unk33.set_opcode(23);
            W[0].unk33.set_unk0(rnd() & 0x1FFF);
            W[0].unk33.set_unk1(rnd() & 0x1FFF);
        }
        14 => {
            W[0].unk13.set_unk0(24);
            W[0].unk13.set_unk1(rnd() & 0xFF);
            W[0].unk13.set_unk2(rnd() & 0x3FFFF);
        }
        _ => W[0].set_throw_flags.set_opcode(25),
    }
}

unsafe fn setup() {
    G = core::mem::zeroed();
    ITEM = core::mem::zeroed();
    JT = core::mem::zeroed();
    BONES = core::mem::zeroed();
    DBT = core::mem::zeroed();
    PROCX = core::mem::zeroed();
    CMDX = core::mem::zeroed();
    FD = core::mem::zeroed();
    W = core::mem::zeroed();
    G.user_data = &raw mut ITEM as *mut c_void;
    G.hsd_obj = &raw mut JT as *mut c_void;
    for i in 0..100 {
        DBT.bones[i] = &raw mut BONES[i % 8];
    }
    ITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    ITEM.x5D4_hitboxes[0].hit.state = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.x4 = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.unk_count = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.x24 = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.x28 = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.x2C = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.element = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.sfx_kind = fv() as _;
    ITEM.x5D4_hitboxes[0].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[0].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[0].hit.kb_angle = fv() as i32;
    ITEM.x5D4_hitboxes[0].hit.x34 = fv() as i32;
    ITEM.x5D4_hitboxes[0].hit.sfx_severity = fv() as i32;
    ITEM.x5D4_hitboxes[0].hit.damage = rw();
    ITEM.x5D4_hitboxes[0].hit.scale = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[0].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[0].hit.set_x40_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x40_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x40_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x40_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x41_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x41_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x41_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x41_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x42_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x43_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[0].hit.set_x40_b4((rnd() & 0xFF) as _);
    ITEM.x5D4_hitboxes[0].hit.x44 = rnd() as u8; ITEM.x5D4_hitboxes[0].hit.x45 = rnd() as u8;
    ITEM.x5D4_hitboxes[0].hit.jobj = if rnd() & 1 != 0 { &raw mut BONES[(rnd() % 8) as usize] } else { null_mut() };
    ITEM.x5D4_hitboxes[0].set_x138((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.state = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.x4 = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.unk_count = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.x24 = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.x28 = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.x2C = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.element = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.sfx_kind = fv() as _;
    ITEM.x5D4_hitboxes[1].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[1].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[1].hit.kb_angle = fv() as i32;
    ITEM.x5D4_hitboxes[1].hit.x34 = fv() as i32;
    ITEM.x5D4_hitboxes[1].hit.sfx_severity = fv() as i32;
    ITEM.x5D4_hitboxes[1].hit.damage = rw();
    ITEM.x5D4_hitboxes[1].hit.scale = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[1].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[1].hit.set_x40_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x40_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x40_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x40_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x41_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x41_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x41_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x41_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x42_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x43_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[1].hit.set_x40_b4((rnd() & 0xFF) as _);
    ITEM.x5D4_hitboxes[1].hit.x44 = rnd() as u8; ITEM.x5D4_hitboxes[1].hit.x45 = rnd() as u8;
    ITEM.x5D4_hitboxes[1].hit.jobj = if rnd() & 1 != 0 { &raw mut BONES[(rnd() % 8) as usize] } else { null_mut() };
    ITEM.x5D4_hitboxes[1].set_x138((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.state = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.x4 = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.unk_count = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.x24 = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.x28 = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.x2C = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.element = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.sfx_kind = fv() as _;
    ITEM.x5D4_hitboxes[2].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[2].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[2].hit.kb_angle = fv() as i32;
    ITEM.x5D4_hitboxes[2].hit.x34 = fv() as i32;
    ITEM.x5D4_hitboxes[2].hit.sfx_severity = fv() as i32;
    ITEM.x5D4_hitboxes[2].hit.damage = rw();
    ITEM.x5D4_hitboxes[2].hit.scale = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[2].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[2].hit.set_x40_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x40_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x40_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x40_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x41_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x41_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x41_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x41_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x42_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x43_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[2].hit.set_x40_b4((rnd() & 0xFF) as _);
    ITEM.x5D4_hitboxes[2].hit.x44 = rnd() as u8; ITEM.x5D4_hitboxes[2].hit.x45 = rnd() as u8;
    ITEM.x5D4_hitboxes[2].hit.jobj = if rnd() & 1 != 0 { &raw mut BONES[(rnd() % 8) as usize] } else { null_mut() };
    ITEM.x5D4_hitboxes[2].set_x138((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.state = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.x4 = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.unk_count = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.x24 = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.x28 = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.x2C = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.element = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.sfx_kind = fv() as _;
    ITEM.x5D4_hitboxes[3].hit.state = rnd() & 1;
    ITEM.x5D4_hitboxes[3].hit.x4 = rnd() % 8;
    ITEM.x5D4_hitboxes[3].hit.kb_angle = fv() as i32;
    ITEM.x5D4_hitboxes[3].hit.x34 = fv() as i32;
    ITEM.x5D4_hitboxes[3].hit.sfx_severity = fv() as i32;
    ITEM.x5D4_hitboxes[3].hit.damage = rw();
    ITEM.x5D4_hitboxes[3].hit.scale = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.x = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.y = rw();
    ITEM.x5D4_hitboxes[3].hit.b_offset.z = rw();
    ITEM.x5D4_hitboxes[3].hit.set_x40_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x40_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x40_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x40_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x41_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x41_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x41_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x41_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x42_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b0((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b1((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b2((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b3((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b4((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b5((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b6((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x43_b7((rnd() & 1) as _);
    ITEM.x5D4_hitboxes[3].hit.set_x40_b4((rnd() & 0xFF) as _);
    ITEM.x5D4_hitboxes[3].hit.x44 = rnd() as u8; ITEM.x5D4_hitboxes[3].hit.x45 = rnd() as u8;
    ITEM.x5D4_hitboxes[3].hit.jobj = if rnd() & 1 != 0 { &raw mut BONES[(rnd() % 8) as usize] } else { null_mut() };
    ITEM.x5D4_hitboxes[3].set_x138((rnd() & 1) as _);
    ITEM.xC3C = rw();
    ITEM.xC40 = rw();
    ITEM.scl = rw();
    ITEM.x3C = rw();
    ITEM.x5CC_currentAnimFrame = rw();
    ITEM.x5D0_animFrameSpeed = rw();
    ITEM.xDAC_itcmd_var0 = fv();
    ITEM.xDB0_itcmd_var1 = fv();
    ITEM.xDB4_itcmd_var2 = fv();
    ITEM.xDC8_word.flags.set_x16(rnd() & 1);
    ITEM.xDAA.xDAA_flag.x0.set_b0((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b1((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b2((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b3((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b4((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b5((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b6((rnd() & 1) as u8);
    ITEM.xDAA.xDAA_flag.x0.set_b7((rnd() & 1) as u8);
    ITEM.xDBC.xDBC_itcmd_var4_word = rnd() & 1;
    ITEM.x524_cmd.timer = if rnd() % 4 == 0 { m::F32_MAX } else { rw() };
    ITEM.x524_cmd.frame_count = rw();
    ITEM.x524_cmd.anon_8.u = if rnd() % 5 == 0 { null_mut() } else { &raw mut W[0] };
    PROCX.s_link = (rnd() % 20) as u8;
    m::ext::HSD_GObj_CurrentInvokedProc = if rnd() & 1 != 0 { &raw mut PROCX } else { null_mut() };
    m::ext::it_804D6D04 = if rnd() & 1 != 0 { &raw mut FD } else { null_mut() };
    CMDX.timer = rw();
    CMDX.frame_count = rw();
    CMDX.anon_8.u = &raw mut W[0];
    CE_RET = 0;
}

unsafe fn hash_all() {
    hm(ITEM.x5D4_hitboxes[0].hit.state as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x4 as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.unk_count as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x24 as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x28 as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x2C as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.element as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.sfx_kind as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.kb_angle as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x34 as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.sfx_severity as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.damage.to_bits());
    hm(ITEM.x5D4_hitboxes[0].hit.scale.to_bits());
    hm(ITEM.x5D4_hitboxes[0].hit.b_offset.x.to_bits());
    hm(ITEM.x5D4_hitboxes[0].hit.b_offset.y.to_bits());
    hm(ITEM.x5D4_hitboxes[0].hit.b_offset.z.to_bits());
    hm(ITEM.x5D4_hitboxes[0].hit.x40_b0() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x40_b1() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x40_b2() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x40_b3() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x41_b4() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x41_b5() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x41_b6() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x41_b7() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b0() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b1() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b2() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b3() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b4() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b5() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b6() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x42_b7() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b0() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b1() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b2() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b3() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b4() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b5() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b6() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x43_b7() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x40_b4() as u32);
    hm(ITEM.x5D4_hitboxes[0].hit.x44 as u32); hm(ITEM.x5D4_hitboxes[0].hit.x45 as u32); hm(pi(ITEM.x5D4_hitboxes[0].hit.jobj)); hm(ITEM.x5D4_hitboxes[0].x138() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.state as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x4 as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.unk_count as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x24 as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x28 as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x2C as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.element as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.sfx_kind as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.kb_angle as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x34 as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.sfx_severity as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.damage.to_bits());
    hm(ITEM.x5D4_hitboxes[1].hit.scale.to_bits());
    hm(ITEM.x5D4_hitboxes[1].hit.b_offset.x.to_bits());
    hm(ITEM.x5D4_hitboxes[1].hit.b_offset.y.to_bits());
    hm(ITEM.x5D4_hitboxes[1].hit.b_offset.z.to_bits());
    hm(ITEM.x5D4_hitboxes[1].hit.x40_b0() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x40_b1() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x40_b2() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x40_b3() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x41_b4() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x41_b5() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x41_b6() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x41_b7() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b0() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b1() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b2() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b3() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b4() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b5() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b6() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x42_b7() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b0() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b1() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b2() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b3() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b4() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b5() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b6() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x43_b7() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x40_b4() as u32);
    hm(ITEM.x5D4_hitboxes[1].hit.x44 as u32); hm(ITEM.x5D4_hitboxes[1].hit.x45 as u32); hm(pi(ITEM.x5D4_hitboxes[1].hit.jobj)); hm(ITEM.x5D4_hitboxes[1].x138() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.state as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x4 as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.unk_count as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x24 as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x28 as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x2C as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.element as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.sfx_kind as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.kb_angle as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x34 as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.sfx_severity as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.damage.to_bits());
    hm(ITEM.x5D4_hitboxes[2].hit.scale.to_bits());
    hm(ITEM.x5D4_hitboxes[2].hit.b_offset.x.to_bits());
    hm(ITEM.x5D4_hitboxes[2].hit.b_offset.y.to_bits());
    hm(ITEM.x5D4_hitboxes[2].hit.b_offset.z.to_bits());
    hm(ITEM.x5D4_hitboxes[2].hit.x40_b0() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x40_b1() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x40_b2() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x40_b3() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x41_b4() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x41_b5() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x41_b6() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x41_b7() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b0() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b1() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b2() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b3() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b4() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b5() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b6() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x42_b7() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b0() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b1() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b2() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b3() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b4() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b5() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b6() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x43_b7() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x40_b4() as u32);
    hm(ITEM.x5D4_hitboxes[2].hit.x44 as u32); hm(ITEM.x5D4_hitboxes[2].hit.x45 as u32); hm(pi(ITEM.x5D4_hitboxes[2].hit.jobj)); hm(ITEM.x5D4_hitboxes[2].x138() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.state as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x4 as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.unk_count as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x24 as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x28 as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x2C as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.element as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.sfx_kind as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.kb_angle as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x34 as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.sfx_severity as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.damage.to_bits());
    hm(ITEM.x5D4_hitboxes[3].hit.scale.to_bits());
    hm(ITEM.x5D4_hitboxes[3].hit.b_offset.x.to_bits());
    hm(ITEM.x5D4_hitboxes[3].hit.b_offset.y.to_bits());
    hm(ITEM.x5D4_hitboxes[3].hit.b_offset.z.to_bits());
    hm(ITEM.x5D4_hitboxes[3].hit.x40_b0() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x40_b1() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x40_b2() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x40_b3() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x41_b4() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x41_b5() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x41_b6() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x41_b7() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b0() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b1() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b2() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b3() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b4() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b5() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b6() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x42_b7() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b0() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b1() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b2() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b3() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b4() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b5() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b6() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x43_b7() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x40_b4() as u32);
    hm(ITEM.x5D4_hitboxes[3].hit.x44 as u32); hm(ITEM.x5D4_hitboxes[3].hit.x45 as u32); hm(pi(ITEM.x5D4_hitboxes[3].hit.jobj)); hm(ITEM.x5D4_hitboxes[3].x138() as u32);
    hm(ITEM.x3C.to_bits());
    hm(ITEM.xDC8_word.flags.x16());
    hm(ITEM.xDAA.xDAA_flag.x0.b0() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b1() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b2() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b3() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b4() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b5() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b6() as u32);
    hm(ITEM.xDAA.xDAA_flag.x0.b7() as u32);
    hm(ITEM.xDAC_itcmd_var0);
    hm(ITEM.xDB0_itcmd_var1);
    hm(ITEM.xDB4_itcmd_var2);
    hm((ITEM.xDBC.xDBC_itcmd_var4_word != 0) as u32);
    hm(ITEM.x524_cmd.timer.to_bits());
    hm(ITEM.x524_cmd.frame_count.to_bits());
    hm(pi(ITEM.x524_cmd.anon_8.u));
    hm(CMDX.timer.to_bits());
    hm(CMDX.frame_count.to_bits());
    hm(pi(CMDX.anon_8.u));
    hm(CE_RET as u32);
}

#[test]
#[ignore = "pointer ids are byte offsets into W: the C oracle strides 8-byte CmdUnion elements, the port walks the raw 4-byte stream (consolidation step 8); needs the C oracle built with a 4-byte CmdUnion"]
fn itanimlist_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const JT, core::mem::size_of::<HSD_JObj>());
        preg(&raw const BONES, core::mem::size_of_val(&BONES));
        preg(&raw const DBT, core::mem::size_of::<DynamicBoneTable>());
        preg(&raw const PROCX, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const W, core::mem::size_of_val(&W));
        preg(&raw const CMDX, core::mem::size_of::<CommandInfo>());
        preg(&raw const FD, core::mem::size_of::<Fighter_804D653C_t>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itanimlist.txt"));
        const KIND_OF_OP: [i32; 16] = [0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 6, 11, 12, 13, 14, 15];
        for op in 0..24u32 {
            for k in 0..2000u32 {
                case(op, k);
                setup();
                let ret = 0u32;
                if op < 16 {
                    genw(KIND_OF_OP[op as usize]);
                    CMDX.anon_8.u = &raw mut W[0];
                }
                let g_ = &raw mut G;
                let cmd = &raw mut CMDX;
                match op {
                    0 => m::it_80278F2C(g_, cmd),
                    1 => m::it_802790C0(g_, cmd),
                    2 => m::it_80279544(g_, cmd),
                    3 => m::it_802795EC(g_, cmd),
                    4 => m::it_80279680(g_, cmd),
                    5 => m::it_802796C4(g_, cmd),
                    6 => m::it_802796FC(g_, cmd),
                    7 => m::it_80279720(g_, cmd),
                    8 => m::it_80279744(g_, cmd),
                    9 => m::it_80279768(g_, cmd),
                    10 => m::it_8027978C(g_, cmd),
                    11 => m::it_80279888(g_, cmd),
                    12 => m::it_802798D4(g_, cmd),
                    13 => m::it_8027990C(g_, cmd),
                    14 => m::it_80279958(g_, cmd),
                    15 => m::it_802799A8(g_, cmd),
                    16 => {
                        let kd = (rnd() % 16) as i32;
                        genw(kd);
                       
                        m::it_802799E4(g_);
                    }
                    17 => {
                        genw(0);
                        m::it_80279AF0(g_, cmd);
                    }
                    18 => {
                        genw(6);
                        m::it_80279B10(g_, cmd);
                    }
                    19 => {
                        let a2 = 21 + (rnd() & 1) as i32;
                        genw(if a2 == 21 { 0 } else { 6 });
                        m::fn_80279B30(g_, cmd, a2);
                    }
                    20 => m::it_80279B64(&raw mut ITEM),
                    21 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        m::it_80279B88(&raw mut ITEM, a, b);
                    }
                    22 => m::it_80279BBC(&raw mut ITEM),
                    _ => {
                        genw(if rnd() & 1 != 0 { 0 } else { 6 });
                       
                        m::it_80279BE0(g_);
                    }
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 24 * 2000);
    }
}

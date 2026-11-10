//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/item.c.
//! src/tu/item.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py item`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_world/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::item as m;
use mtu_harness::*;
#[path = "mtu/item_gen.rs"]
mod g;

static mut CUR_OP: i32 = -1;
const OP_Item_80266F3C: i32 = 0;
const OP_Item_80266F70: i32 = 1;
const OP_Item_80266FA8: i32 = 2;
const OP_Item_80266FCC: i32 = 3;
const OP_ItUnkHoldKind: i32 = 4;
const OP_HSD_JObjSetScaleItem: i32 = 5;
const OP_HSD_JObjSetFacingDirItem: i32 = 6;
const OP_Item_80267130: i32 = 7;
const OP_Item_80267454: i32 = 8;
const OP_Item_802674AC: i32 = 9;
const OP_Item_802675A8: i32 = 10;
const OP_Item_802676F4: i32 = 11;
const OP_Item_8026784C: i32 = 12;
const OP_Item_80267978: i32 = 13;
const OP_Item_80267AA8: i32 = 14;
const OP_Item_802680CC: i32 = 15;
const OP_Item_8026814C: i32 = 16;
const OP_Item_802682F0: i32 = 17;
const OP_Item_8026849C: i32 = 18;
const OP_Item_80268560: i32 = 19;
const OP_foobar: i32 = 20;
const OP_foobar2: i32 = 21;
const OP_foobar3: i32 = 22;
const OP_Item_8026862C: i32 = 23;
const OP_Item_ScriptSpawnAtCap: i32 = 24;
const OP_Item_ScriptSpawnCapTest: i32 = 25;
const OP_Item_80268B18: i32 = 26;
const OP_Item_80268B5C: i32 = 27;
const OP_Item_80268B9C: i32 = 28;
const OP_Item_80268BE0: i32 = 29;
const OP_Item_80268D34: i32 = 30;
const OP_Item_80268DD4: i32 = 31;
const OP_Item_80268E40: i32 = 32;
const OP_Item_80268E5C: i32 = 33;
const OP_Item_802693E4: i32 = 34;
const OP_Item_802694CC: i32 = 35;
const OP_Item_80269528: i32 = 36;
const OP_Item_802696CC: i32 = 37;
const OP_Item_802697D4: i32 = 38;
const OP_Item_80269978: i32 = 39;
const OP_Item_80269A9C: i32 = 40;
const OP_Item_80269B60: i32 = 41;
const OP_Item_80269BE4: i32 = 42;
const OP_Item_80269C5C: i32 = 43;
const OP_Item_80269CA0: i32 = 44;
const OP_Item_80269CC4: i32 = 45;
const OP_Item_80269DC8: i32 = 46;
const OP_Item_80269F14: i32 = 47;
const OP_Item_8026A0A0: i32 = 48;
const OP_Item_8026A0FC: i32 = 49;
const OP_func_8026A158_helper: i32 = 50;
const OP_Item_8026A158: i32 = 51;
const OP_func_8026A1E8_inline: i32 = 52;
const OP_Item_8026A1E8: i32 = 53;
const OP_processCallback: i32 = 54;
const OP_OnTakeDamageThink: i32 = 55;
const OP_OnClankThink: i32 = 56;
const OP_OnGiveDamageThink: i32 = 57;
const OP_EnterHitlagThink: i32 = 58;
const OP_checkHitLag: i32 = 59;
const OP_Item_8026A294: i32 = 60;
const OP_Item_ScriptCommitDamage: i32 = 61;
const OP_Item_8026A788: i32 = 62;
const OP_Item_8026A810: i32 = 63;
const OP_Item_8026A848: i32 = 64;
const OP_DestroyItemInline: i32 = 65;
const OP_ItemSwitch: i32 = 66;
const OP_RunGObjCallback: i32 = 67;
const OP_func_8026A8EC_inline1: i32 = 68;
const OP_func_8026A8EC_inline2: i32 = 69;
const OP_func_8026A8EC_inline3: i32 = 70;
const OP_Item_8026A8EC: i32 = 71;
const OP_Item_8026AB54: i32 = 72;
const OP_Item_8026ABD8: i32 = 73;
const OP_Item_8026AC74: i32 = 74;
const OP_Item_8026AD20: i32 = 75;
const OP_Item_8026ADC0: i32 = 76;
const OP_Item_OnUserDataRemove: i32 = 77;
const OP_Item_8026AE60: i32 = 78;
const OP_Item_MexSfx: i32 = 79;
const OP_Item_8026AE84: i32 = 80;
const OP_Item_8026AF0C: i32 = 81;
const OP_Item_8026AFA0: i32 = 82;
const OP_Item_8026B034: i32 = 83;
const OP_Item_8026B074: i32 = 84;
const OP_Item_8026B0B4: i32 = 85;
const OP_Item_IsGrabbable: i32 = 86;

const _: () = assert!(core::mem::size_of::<HSD_DObj>() <= 64);
static mut AJ: [HSD_AnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut MJ: [HSD_MatAnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut SJ: [HSD_ShapeAnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut MD: [ItemModelDesc; 2] = unsafe { core::mem::zeroed() };
static mut DY: [ItemDynamics; 2] = unsafe { core::mem::zeroed() };
static mut BDD: [BoneDynamicsDesc; 4] = unsafe { core::mem::zeroed() };
static mut STA: ItemStateArray = unsafe { core::mem::zeroed() };
static mut JNT: [HSD_Joint; 2] = unsafe { core::mem::zeroed() };
static mut A24: [*mut Article; 43] = [null_mut(); 43];
static mut A38: [*mut Article; 118] = [null_mut(); 118];
static mut A30: [*mut Article; 47] = [null_mut(); 47];
static mut CA04: [Fighter_804D653C_t; 4] = unsafe { core::mem::zeroed() };
fn pick_kind(spawn: bool) -> i32 {
    let r = rnd();
    match r & 7 {
        0 => 0x1000 + ((r >> 8) % 0x200) as i32,
        1 => 0x1200 + ((r >> 8) % 0x50) as i32,
        2 => if spawn { 5000 + ((r >> 8) % 40) as i32 } else { ((r >> 8) % 300) as i32 },
        3 => 0x20000 + ((r >> 8) % 20) as i32,
        _ => ((r >> 8) % 300) as i32,
    }
}
pub static mut DOBBUF: [[u64; 8]; 4] = [[0; 8]; 4];

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];

{
    AJ = core::mem::zeroed(); MJ = core::mem::zeroed(); SJ = core::mem::zeroed(); MD = core::mem::zeroed(); DY = core::mem::zeroed();
    BDD = core::mem::zeroed(); STA = core::mem::zeroed(); JNT = core::mem::zeroed(); DOBBUF = [[0; 8]; 4]; CA04 = core::mem::zeroed();
    for i in 0..4usize {
        AJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut AJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        AJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut AJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        MJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut MJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        MJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut MJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        SJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut SJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        SJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut SJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        (*((&raw mut DOBBUF[i]) as *mut HSD_DObj)).next = if i < 3 { (&raw mut DOBBUF[i + 1]) as *mut HSD_DObj } else { null_mut() };
        (*((&raw mut DOBBUF[i]) as *mut HSD_DObj)).mobj = if rnd() & 1 != 0 { (&raw mut U8A[0]) as *mut HSD_MObj } else { null_mut() };
        BDD[i].bone_id = (rnd() % 4) as _;
        CA04[i].unk5 = (rnd() & 1) as u8;
        STT[i].anim_id = (rnd() % 9) as i32 - 1;
    }
    for k in 0..8usize {
        STA.x0_itemStateDesc[k].x0_anim_joint = if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() };
        STA.x0_itemStateDesc[k].x4_matanim_joint = if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() };
        STA.x0_itemStateDesc[k].x8_parameters = if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() };
    }
    for i in 0..3usize {
        SDS[i].x0_anim_joint = if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() };
        SDS[i].x4_matanim_joint = if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() };
        SDS[i].x8_parameters = if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() };
    }
    for i in 0..2usize {
        let bc: i32 = if rnd() % 3 == 0 { 0 } else { (rnd() % 4) as i32 + 1 };
        ART[i].x0_common_attr = &raw mut ATTR[i];
        ART[i].xC_itemStates = &raw mut STA;
        ART[i].x10_modelDesc = &raw mut MD[i];
        MD[i].x0_joint = if rnd() & 1 != 0 { (&raw mut JNT[(rnd() & 1) as usize]) as *mut _ } else { null_mut() };
        MD[i].x4_bone_count = bc as _;
        DY[i].count = (rnd() % 4) as i32;
        DY[i].dyn_descs = ((&raw mut BDD) as *mut BoneDynamicsDesc) as *mut _;
        ART[i].x14_dynamics = if bc != 0 && (rnd() & 1) != 0 { &raw mut DY[i] } else { null_mut() };
    }
    for k in 0..43usize { A24[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..118usize { A38[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..47usize { A30[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..30usize { m::ext::it_804A0F60[k] = &raw mut ART[(rnd() % 2) as usize]; }
    m::ext::it_804D6D24 = (&raw mut A24) as *mut *mut Article;
    m::ext::it_804D6D38 = (&raw mut A38) as *mut *mut Article;
    m::ext::it_804D6D30 = (&raw mut A30) as *mut *mut Article;
    m::ext::it_804D6D04 = (&raw mut CA04) as *mut Fighter_804D653C_t;
    for i in 0..4usize {
        IT[i].kind = pick_kind(false) as _;
        IT[i].x374_dynamicBonesNum = (rnd() % 5) as i32;
        IT[i].atk_victim = pick_igo();
        IT[i].grab_victim = pick_igo();
        IT[i].grab_dealt = core::mem::transmute::<usize, _>(pick_cb(1));
        IT[i].grabbed_for_victim = core::mem::transmute::<usize, _>(pick_cb(3));
        IT[i].xC8_joint = if rnd() & 1 != 0 { &raw mut JNT[(rnd() & 1) as usize] } else { null_mut() };
        IT[i].x548_colorOverlay.x28_colanim.i = (rnd() % 4) as _;
        IT[i].xC6C = (rnd() % 5) as f32 * 0.5;
        IT[i].xC74 = ((rnd() & 0xFF) | ((rnd() & 0xFF) << 24)) as i32;
    }
    for i in 0..8usize { JO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
    SP.kind = pick_kind(true) as _;
    for i in 0..26usize { *((&raw mut m::Item_804A0C64) as *mut i32).add(i) = (fv() & 7) as i32; }
    m::Item_804A0CCC.count = rnd() as _; m::Item_804A0CCC.x154.x0.set_b0((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b1((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b2((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b3((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b4((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b5((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b6((rnd() & 1) as u8); m::Item_804A0CCC.x154.x0.set_b7((rnd() & 1) as u8); for i in 0..0xB0usize { *((&raw mut m::Item_804A0CCC) as *mut u8).add(i) = rnd() as u8; }
    m::Item_804A0E24.last_kind = rnd() as i32; m::Item_804A0E24.previous_kind = rnd() as i32; m::Item_804A0E24.rare_spawned = (rnd() & 1) as _;
}

}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
hm(rh(&m::Item_804A0C64)); hm(m::Item_804A0CCC.count as u32); hm(m::Item_804A0CCC.x154.x0.b0() as u32); hm(m::Item_804A0CCC.x154.x0.b1() as u32); hm(m::Item_804A0CCC.x154.x0.b2() as u32); hm(m::Item_804A0CCC.x154.x0.b3() as u32); hm(m::Item_804A0CCC.x154.x0.b4() as u32); hm(m::Item_804A0CCC.x154.x0.b5() as u32); hm(m::Item_804A0CCC.x154.x0.b6() as u32); hm(m::Item_804A0CCC.x154.x0.b7() as u32); hm(rh(&m::Item_804A0CCC.ecb_offset_arr)); hm(m::Item_804A0E24.last_kind as u32); hm(m::Item_804A0E24.previous_kind as u32); hm(m::Item_804A0E24.rare_spawned as u32);
}

#[test]
fn item_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const AJ, core::mem::size_of_val(&AJ)); preg(&raw const MJ, core::mem::size_of_val(&MJ)); preg(&raw const SJ, core::mem::size_of_val(&SJ)); preg(&raw const MD, core::mem::size_of_val(&MD)); preg(&raw const DY, core::mem::size_of_val(&DY)); preg(&raw const BDD, core::mem::size_of_val(&BDD)); preg(&raw const STA, core::mem::size_of_val(&STA)); preg(&raw const JNT, core::mem::size_of_val(&JNT)); preg(&raw const DOBBUF, core::mem::size_of_val(&DOBBUF)); preg(&raw const A24, core::mem::size_of_val(&A24)); preg(&raw const A38, core::mem::size_of_val(&A38)); preg(&raw const A30, core::mem::size_of_val(&A30)); preg(&raw const CA04, core::mem::size_of_val(&CA04)); preg(&raw const m::item_alloc_data, core::mem::size_of_val(&m::item_alloc_data)); preg(&raw const m::item_dynamic_bones_alloc_data, core::mem::size_of_val(&m::item_dynamic_bones_alloc_data)); preg(&raw const m::item_link_alloc_data, core::mem::size_of_val(&m::item_link_alloc_data)); preg(&raw const m::Item_804A0C64, core::mem::size_of_val(&m::Item_804A0C64));
        let mut c = Check::new(include_str!("data/mtu_item.txt"));
        let mut total = 0u32;
        for op in 0..87u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                5 => (300u32, false),
                6 => (300u32, false),
                7 => (300u32, false),
                8 => (300u32, false),
                9 => (300u32, false),
                10 => (300u32, false),
                11 => (300u32, false),
                12 => (300u32, false),
                13 => (300u32, false),
                14 => (300u32, false),
                15 => (300u32, false),
                16 => (300u32, false),
                17 => (300u32, false),
                18 => (300u32, false),
                19 => (300u32, false),
                20 => (300u32, false),
                21 => (300u32, false),
                22 => (300u32, false),
                23 => (300u32, false),
                24 => (300u32, false),
                25 => (300u32, false),
                26 => (300u32, false),
                27 => (300u32, false),
                28 => (300u32, false),
                29 => (300u32, false),
                30 => (300u32, false),
                31 => (300u32, false),
                32 => (300u32, false),
                33 => (300u32, false),
                34 => (300u32, false),
                35 => (300u32, false),
                36 => (300u32, false),
                37 => (300u32, false),
                38 => (300u32, false),
                39 => (300u32, false),
                40 => (300u32, false),
                41 => (300u32, false),
                42 => (300u32, false),
                43 => (300u32, false),
                44 => (300u32, false),
                45 => (300u32, false),
                46 => (300u32, false),
                47 => (300u32, false),
                48 => (300u32, false),
                49 => (300u32, false),
                50 => (300u32, false),
                51 => (300u32, false),
                52 => (300u32, false),
                53 => (300u32, false),
                54 => (300u32, false),
                55 => (300u32, false),
                56 => (300u32, false),
                57 => (300u32, false),
                58 => (300u32, false),
                59 => (300u32, false),
                60 => (300u32, false),
                61 => (300u32, false),
                62 => (300u32, false),
                63 => (300u32, false),
                64 => (300u32, false),
                65 => (300u32, false),
                66 => (300u32, false),
                67 => (300u32, false),
                68 => (300u32, false),
                69 => (300u32, false),
                70 => (300u32, false),
                71 => (300u32, false),
                72 => (300u32, false),
                73 => (300u32, false),
                74 => (300u32, false),
                75 => (300u32, false),
                76 => (300u32, false),
                77 => (300u32, false),
                78 => (300u32, false),
                79 => (300u32, false),
                80 => (300u32, false),
                81 => (300u32, false),
                82 => (300u32, false),
                83 => (300u32, false),
                84 => (300u32, false),
                85 => (300u32, false),
                86 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                CUR_OP = op as i32;
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { 
ret = m::Item_80266F3C() as u32;
 }
                    1 => { 
m::Item_80266F70();
 }
                    2 => { 
m::Item_80266FA8();
 }
                    3 => { 
m::Item_80266FCC();
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::ItUnkHoldKind(a0);
 }
                    5 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut JO[(rnd() % 8) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::HSD_JObjSetScaleItem(a0, a1, a2);
 }
                    6 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
m::HSD_JObjSetFacingDirItem(a0, a1);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut SP;
m::Item_80267130(a0, a1);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80267454(a0);
 }
                    9 => { let a0 = &raw mut SP;
m::Item_802674AC(a0);
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802675A8(a0);
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802676F4(a0);
 }
                    12 => { let a0 = fv() as _;
let a1 = fv() as _;
ret = m::Item_8026784C(a0, a1) as u32;
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80267978(a0);
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut SP;
m::Item_80267AA8(a0, a1);
 }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802680CC(a0);
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026814C(a0);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_802682F0(a0) as u32;
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026849C(a0);
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80268560(a0);
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::foobar(a0);
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::foobar2(a0);
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::foobar3(a0);
 }
                    23 => { let a0 = &raw mut SP;
ret = pi(m::Item_8026862C(a0));
 }
                    24 => { let a0 = &raw mut SP;
ret = m::Item_ScriptSpawnAtCap(a0) as u32;
 }
                    25 => { 
ret = m::Item_ScriptSpawnCapTest() as u32;
 }
                    26 => { let a0 = &raw mut SP;
ret = pi(m::Item_80268B18(a0));
 }
                    27 => { let a0 = &raw mut SP;
ret = pi(m::Item_80268B5C(a0));
 }
                    28 => { let a0 = &raw mut SP;
m::Item_80268B9C(a0);
 }
                    29 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
let a1 = if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() };
let a2 = if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() };
let a3 = if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() };
let a4 = &raw mut IT[(rnd() % 4) as usize];
m::Item_80268BE0(a0, a1, a2, a3, a4);
 }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut SDS[(rnd() % 3) as usize];
m::Item_80268D34(a0, a1);
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::Item_80268DD4(a0, a1);
 }
                    32 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut SDS[(rnd() % 3) as usize];
m::Item_80268E40(a0, a1);
 }
                    33 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 3) as _;
let a2 = fv() as _;
m::Item_80268E5C(a0, a1, a2);
 }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802693E4(a0);
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802694CC(a0);
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269528(a0);
 }
                    37 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_802696CC(a0) as u32;
 }
                    38 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_802697D4(a0);
 }
                    39 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269978(a0);
 }
                    40 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269A9C(a0);
 }
                    41 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269B60(a0);
 }
                    42 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269BE4(a0);
 }
                    43 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269C5C(a0);
 }
                    44 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = fv() as _;
m::Item_80269CA0(a0, a1);
 }
                    45 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_80269CC4(a0);
 }
                    46 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_80269DC8(a0) as u32;
 }
                    47 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_80269F14(a0) as u32;
 }
                    48 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A0A0(a0);
 }
                    49 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A0FC(a0);
 }
                    50 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::func_8026A158_helper(a0);
 }
                    51 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A158(a0);
 }
                    52 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::func_8026A1E8_inline(a0);
 }
                    53 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A1E8(a0);
 }
                    54 => { let a0 = unsafe { core::mem::transmute::<usize, _>(pick_cb(2)) };
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = &raw mut IT[(rnd() % 4) as usize];
ret = m::processCallback(a0, a1, a2) as u32;
 }
                    55 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
ret = m::OnTakeDamageThink(a0, a1) as u32;
 }
                    56 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
ret = m::OnClankThink(a0, a1) as u32;
 }
                    57 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
ret = m::OnGiveDamageThink(a0, a1) as u32;
 }
                    58 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
m::EnterHitlagThink(a0, a1);
 }
                    59 => { let a0 = rw();
let a1 = &raw mut IT[(rnd() % 4) as usize];
m::checkHitLag(a0, a1);
 }
                    60 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A294(a0);
 }
                    61 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_ScriptCommitDamage(a0) as u32;
 }
                    62 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A788(a0);
 }
                    63 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A810(a0);
 }
                    64 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A848(a0, a1);
 }
                    65 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
m::DestroyItemInline(a0, a1);
 }
                    66 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::ItemSwitch(a0);
 }
                    67 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
m::RunGObjCallback(a0, a1);
 }
                    68 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::func_8026A8EC_inline1(a0);
 }
                    69 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::func_8026A8EC_inline2(a0);
 }
                    70 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::func_8026A8EC_inline3(a0);
 }
                    71 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026A8EC(a0);
 }
                    72 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = fv() as _;
m::Item_8026AB54(a0, a1, a2);
 }
                    73 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = rw();
m::Item_8026ABD8(a0, a1, a2);
 }
                    74 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = rw();
m::Item_8026AC74(a0, a1, a2, a3);
 }
                    75 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = rw();
let a4 = fv() != 0;
m::Item_8026AD20(a0, a1, a2, a3, a4);
 }
                    76 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026ADC0(a0);
 }
                    77 => { let a0 = (&raw mut IT[(rnd() % 4) as usize]) as *mut c_void;
m::Item_OnUserDataRemove(a0);
 }
                    78 => { 
ret = m::Item_8026AE60() as u32;
 }
                    79 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::Item_MexSfx(a0, a1) as u32;
 }
                    80 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = (if rnd() % 4 == 0 { 540000 + (rnd() & 1) as i32 } else if rnd() % 3 == 0 { 5000 + (rnd() % 5000) as i32 } else { fv() as i32 }) as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::Item_8026AE84(a0, a1, a2, a3);
 }
                    81 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = (if rnd() % 4 == 0 { 540000 + (rnd() & 1) as i32 } else if rnd() % 3 == 0 { 5000 + (rnd() % 5000) as i32 } else { fv() as i32 }) as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::Item_8026AF0C(a0, a1, a2, a3);
 }
                    82 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = (if rnd() % 4 == 0 { 540000 + (rnd() & 1) as i32 } else if rnd() % 3 == 0 { 5000 + (rnd() % 5000) as i32 } else { fv() as i32 }) as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::Item_8026AFA0(a0, a1, a2, a3);
 }
                    83 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::Item_8026B034(a0);
 }
                    84 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
m::Item_8026B074(a0);
 }
                    85 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::Item_8026B0B4(a0);
 }
                    86 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::Item_IsGrabbable(a0) as u32;
 }
                    _ => unreachable!(),
                }
                hash_world(ft);
                hash_stem();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}

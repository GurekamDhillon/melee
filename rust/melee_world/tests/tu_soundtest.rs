//! src/tu/soundtest.rs vs the C (gen/mtu/tu/soundtest.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py soundtest`).
//! Written by gen/mtu/mk_soundtest.py: do not edit. Ops: "tables", "un_802FF7DC", "un_802FF884", "un_802FFCD0", "un_802FFD94", "un_802FFEE0", "un_802FFF2C", "un_80301BA8", "un_80301C64", "un_802FF88C", "un_802FF934", "un_802FF958", "un_802FF99C", "un_802FF9DC", "un_802FFB58", "un_802FFBAC", "un_802FFC30", "un_802FFC6C", "fn_802FFCC8", "fn_802FFE0C", "fn_802FFE6C", "un_802FFEA4", "un_803001DC", "un_80300218", "un_80300248", "un_80300290", "un_803002FC", "un_80300338", "un_80300378", "un_803003C4", "un_80300410", "un_80300480", "un_803004B4", "un_803004E8", "un_8030051C", "un_80300550", "un_80300584", "un_803005B8", "un_803005EC", "un_80300620", "un_80300654", "un_80300688", "un_803006BC", "un_803006F0", "un_80300724", "un_80300758", "un_80300790", "un_803007C8", "un_803007FC", "un_80300830", "un_80300864", "un_80300898", "un_803008CC", "un_80300900", "un_80300934", "un_80300968", "un_803009A4", "un_803009E0", "un_80300A88", "un_80300AB8", "un_80300AF4", "un_80300B58", "un_80300BBC", "un_80300BF8", "un_80300C3C", "un_80300C80", "fn_80300CC8", "un_80300D78", "fn_80300DE0", "un_80300E74", "fn_80300ED0", "un_80300F3C", "un_80300F98", "un_80300FEC", "un_80301028", "un_80301074", "un_803010C0", "fn_8030110C", "fn_803011EC", "un_80301280", "un_803012D4", "un_80301328", "un_8030136C", "un_803013B0", "un_803013EC", "un_80301420", "un_80301454", "un_80301490", "un_803014CC", "un_80301508", "un_80301544", "un_80301580", "un_803015BC", "un_803015F8", "un_80301634", "un_803016D8", "un_80301734", "un_8030178C", "un_80301800", "un_80301840", "gw_dev_unlock_all_chars", "gw_dev_unlock_all_stages", "un_803018BC", "un_8030191C", "un_80301964", "un_80301A64", "un_80301AD4", "un_80301B48", "un_80301C80", "un_80301CE0", "un_80301D40", "un_80301D7C", "un_80301DCC", "un_80301E08"
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, clippy::all)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::soundtest as m;
use mtu_harness::*;
#[path = "mtu/soundtest_gen.rs"]
mod g;

static mut OA: un_80304138_objalloc_t = unsafe { core::mem::zeroed() };
static mut DT: DevText = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut SOB: HSD_SObj = unsafe { core::mem::zeroed() };
static mut MEM: [u8; 64] = [0; 64];
static mut STR: [i8; 8] = [97, 98, 99, 0, 0, 0, 0, 0];
static mut U32S: [u32; 5] = [0; 5];
static mut I32S: [i32; 2] = [0; 2];
static mut GMM: [gmm_x0_528_t; 3] = unsafe { core::mem::zeroed() };
static mut ALLS: UnkAllstarData = unsafe { core::mem::zeroed() };
static mut SYMBUF: [u64; 16] = [0; 16];
static mut A5: [i32; 256] = [0; 256];
static mut A6: [i32; 64] = [0; 64];
static mut ARR: [i32; 16] = [0; 16];
static mut O0: m::UnkSoundTestData0 = unsafe { core::mem::zeroed() };
static mut O1: m::UnkSoundTestData1 = unsafe { core::mem::zeroed() };
static mut O2: un_80301C64_t = unsafe { core::mem::zeroed() };
static mut SMD: StartMeleeData = unsafe { core::mem::zeroed() };
static mut PRISTINE: Option<m::SoundTestMenuData> = None;
static mut PRISTINE_IMG: Option<HSD_ImageDesc> = None;

unsafe fn hash_258() {
    hm(m::un_803FA258.x0 as u32);
    hm(m::un_803FA258.x4[0] as u32);
    hm(m::un_803FA258.x4[1] as u32);
    hm(m::un_803FA258.x4[2] as u32);
    hm(m::un_803FA258.x4[3] as u32);
    hm(m::un_803FA258.x14[0] as u32);
    hm(m::un_803FA258.x14[1] as u32);
    hm(m::un_803FA258.x14[2] as u32);
    hm(m::un_803FA258.x14[3] as u32);
    hm(m::un_803FA258.x24[0] as u32);
    hm(m::un_803FA258.x24[1] as u32);
    hm(m::un_803FA258.x24[2] as u32);
    hm(m::un_803FA258.x24[3] as u32);
    hm(m::un_803FA258.x34 as u32);
    hm(m::un_803FA258.x38[0] as u32);
    hm(m::un_803FA258.x38[1] as u32);
    hm(m::un_803FA258.x38[2] as u32);
    hm(m::un_803FA258.x38[3] as u32);
    hm(m::un_803FA258.x48[0] as u32);
    hm(m::un_803FA258.x48[1] as u32);
    hm(m::un_803FA258.x48[2] as u32);
    hm(m::un_803FA258.x48[3] as u32);
    hm(m::un_803FA258.x58[0] as u32);
    hm(m::un_803FA258.x58[1] as u32);
    hm(m::un_803FA258.x58[2] as u32);
    hm(m::un_803FA258.x58[3] as u32);
    hm(m::un_803FA258.x68[0] as u32);
    hm(m::un_803FA258.x68[1] as u32);
    hm(m::un_803FA258.x68[2] as u32);
    hm(m::un_803FA258.x68[3] as u32);
    hm(m::un_803FA258.x78[0].to_bits());
    hm(m::un_803FA258.x78[1].to_bits());
    hm(m::un_803FA258.x78[2].to_bits());
    hm(m::un_803FA258.x78[3].to_bits());
    hm(m::un_803FA258.x88[0].to_bits());
    hm(m::un_803FA258.x88[1].to_bits());
    hm(m::un_803FA258.x88[2].to_bits());
    hm(m::un_803FA258.x88[3].to_bits());
    hm(m::un_803FA258.x98[0].to_bits());
    hm(m::un_803FA258.x98[1].to_bits());
    hm(m::un_803FA258.x98[2].to_bits());
    hm(m::un_803FA258.x98[3].to_bits());
    hm(m::un_803FA258.xA8[0] as u32);
    hm(m::un_803FA258.xA8[1] as u32);
    hm(m::un_803FA258.xA8[2] as u32);
    hm(m::un_803FA258.xA8[3] as u32);
    hm(m::un_803FA258.xB8[0] as u32);
    hm(m::un_803FA258.xB8[1] as u32);
    hm(m::un_803FA258.xB8[2] as u32);
    hm(m::un_803FA258.xB8[3] as u32);
    hm(m::un_803FA258.xC8 as u32);
    hm(m::un_803FA258.xCC[0] as u32);
    hm(m::un_803FA258.xCC[1] as u32);
    hm(m::un_803FA258.xCC[2] as u32);
    hm(m::un_803FA258.xCC[3] as u32);
    hm(m::un_803FA258.xDC[0] as u32);
    hm(m::un_803FA258.xDC[1] as u32);
    hm(m::un_803FA258.xDC[2] as u32);
    hm(m::un_803FA258.xDC[3] as u32);
    hm(m::un_803FA258.xEC.to_bits());
    hm(m::un_803FA258.xF0 as u32);
    hm(m::un_803FA258.xF4 as u32);
    hm(m::un_803FA258.xF5 as u32);
    hm(m::un_803FA258.xF6 as u32);
    hm(m::un_803FA258.xF7 as u32);
    hm(m::un_803FA258.xF8 as u32);
    hm(m::un_803FA258.xFC as u32);
    hm(m::un_803FA258.x100 as u32);
    hm(m::un_803FA258.x104 as u32);
    hm(m::un_803FA258.x108 as u32);
    hm(m::un_803FA258.x10C as u32);
    hm(m::un_803FA258.x110 as u32);
    hm(m::un_803FA258.x114 as u32);
    hm(m::un_803FA258.x118 as u32);
    hm(m::un_803FA258.x11C as u32);
    hm(m::un_803FA258.x120 as u32);
    hm(m::un_803FA258.x124 as u32);
    hm(m::un_803FA258.x128 as u32);
    hm(m::un_803FA258.x12C as u32);
    hm(m::un_803FA258.x130 as u32);
    hm(m::un_803FA258.x134 as u32);
    hm(pi(m::un_803FA258.x138));
    hm(m::un_803FA258.x13C as u32);
    hm(m::un_803FA258.x140 as u32);
    hm(m::un_803FA258.x144[0] as u32);
    hm(m::un_803FA258.x144[1] as u32);
    hm(m::un_803FA258.x144[2] as u32);
    hm(m::un_803FA258.x144[3] as u32);
    hm(m::un_803FA258.x144[4] as u32);
    hm(m::un_803FA258.x144[5] as u32);
    hm(m::un_803FA258.x144[6] as u32);
    hm(m::un_803FA258.x144[7] as u32);
    hm(m::un_803FA258.x144[8] as u32);
    hm(m::un_803FA258.x144[9] as u32);
    hm(m::un_803FA258.x144[10] as u32);
    hm(m::un_803FA258.x144[11] as u32);
    hm(m::un_803FA258.x144[12] as u32);
    hm(m::un_803FA258.x144[13] as u32);
    hm(m::un_803FA258.x144[14] as u32);
    hm(m::un_803FA258.x144[15] as u32);
    hm(m::un_803FA258.x144[16] as u32);
    hm(m::un_803FA258.x188 as u32);
    hm(m::un_803FA258.x18C as u32);
}

unsafe fn hash_tables() {
    hm((*(&raw const m::un_804D5850)) as u32);
    hm((*(&raw const m::un_804D5854)) as u32);
    hm((*(&raw const m::un_804D5858)) as u32);
    hm((*(&raw const m::un_804D585C)) as u32);
    hm(rh(&(*(&raw const m::un_804D5860))));
    hm(rh(&(*(&raw const m::un_804D5868))));
    hm(rh(&(*(&raw const m::un_804D5870))));
    hm(rh(&(*(&raw const m::un_804D5878))));
    hm(rh(&(*(&raw const m::un_804D587C))));
    for i in 0..2 { hm(pi((*(&raw const m::un_804D5880))[i])); }
    for i in 0..2 { hm(pi((*(&raw const m::un_804D5888))[i])); }
    hm(rh(&(*(&raw const m::un_804D5890))));
    hm(rh(&(*(&raw const m::un_804D5898))));
    hm(rh(&(*(&raw const m::un_804D58A0))));
    hm(rh(&(*(&raw const m::un_804D58A8))));
    hm(rh(&(*(&raw const m::un_804D58B0))));
    hm(rh(&(*(&raw const m::un_804D58B8))));
    hm(rh(&(*(&raw const m::un_804D58C0))));
    hm(rh(&(*(&raw const m::un_804D58C8))));
    hm(rh(&(*(&raw const m::un_804D58D0))));
    hm(rh(&(*(&raw const m::un_804D58D8))));
    hm(rh(&(*(&raw const m::un_804D58E0))));
    hm(rh(&(*(&raw const m::un_804D58E4))));
    hm(rh(&(*(&raw const m::un_804D58EC))));
    hm(rh(&(*(&raw const m::un_804D58F4))));
    for i in 0..2 { hm(pi((*(&raw const m::un_804D58FC))[i])); }
    hm((*(&raw const m::un_804D5908)) as u32);
    hm((*(&raw const m::un_804D590C)) as u32);
    hm((*(&raw const m::un_804D5910)) as u32);
    hm(rh(&(*(&raw const m::un_804D5914))));
    hm(rh(&(*(&raw const m::un_804D591C))));
    hm(rh(&(*(&raw const m::un_804D5924))));
    hm(rh(&(*(&raw const m::un_804D592C))));
    hm(rh(&(*(&raw const m::un_804D5934))));
    hm(rh(&(*(&raw const m::un_804D593C))));
    hm(rh(&(*(&raw const m::un_804D5944))));
    hm(rh(&(*(&raw const m::un_804D594C))));
    hm(rh(&(*(&raw const m::un_804D5954))));
    hm(rh(&(*(&raw const m::un_804D595C))));
    hm(rh(&(*(&raw const m::un_804D5964))));
    hm(rh(&(*(&raw const m::un_804D5968))));
    hm(rh(&(*(&raw const m::un_804D5970))));
    hm(rh(&(*(&raw const m::un_804D5978))));
    hm(rh(&(*(&raw const m::un_804D5980))));
    hm(rh(&(*(&raw const m::un_804D5988))));
    hm(pi((*(&raw const m::un_804D5990)).image)); hm(pi((*(&raw const m::un_804D5990)).tlut));
    hm(rh(&(*(&raw const m::un_804D5998))));
    hm(rh(&(*(&raw const m::un_804D59A0))));
    hm(rh(&(*(&raw const m::un_804D59A8))));
    hm(rh(&(*(&raw const m::un_804D59B0))));
    hm(rh(&(*(&raw const m::un_804D59B8))));
    hm(rh(&(*(&raw const m::un_804D59C0))));
    hm(rh(&(*(&raw const m::un_804D59C8))));
    hm(rh(&(*(&raw const m::un_804D59D0))));
    hm(rh(&(*(&raw const m::un_804D59D8))));
    hm(rh(&(*(&raw const m::un_804D59E0))));
    hm(rh(&(*(&raw const m::un_804D59E8))));
    hm(rh(&(*(&raw const m::un_804D59F0))));
    hm(rh(&(*(&raw const m::un_804D59F8))));
    hm(pi((*(&raw const m::un_804D6DA8))));
    hm((*(&raw const m::un_804D6DAC)) as u32);
    hm((*(&raw const m::un_804D6DB0)) as u32);
    hm((*(&raw const m::un_804D6DB4)) as u32);
    hm((*(&raw const m::un_804D6DB8)) as u32);
    hm((*(&raw const m::un_804D6DBC)) as u32);
    hm((*(&raw const m::un_804D6DC0)) as u32);
    hm((*(&raw const m::un_804D6DC8)) as u32);
    hm((*(&raw const m::un_804D6DCC)) as u32);
    hm((*(&raw const m::un_804D6DD0)).to_bits());
    hm((*(&raw const m::un_804D6DD4)) as u32);
    hm((*(&raw const m::un_804D6DD8)) as u32);
    hm((*(&raw const m::un_804D6DDC)) as u32);
    hm((*(&raw const m::un_804D6DE0)) as u32);
    hm((*(&raw const m::un_804D6DE4)) as u32);
    hm((*(&raw const m::un_804D6DE8)) as u32);
    hm((*(&raw const m::un_804D6DEC)) as u32);
    hm((*(&raw const m::un_804D6DF0)) as u32);
    hm((*(&raw const m::un_804D6DF4)) as u32);
    hm((*(&raw const m::un_804D6DF8)) as u32);
    hm((*(&raw const m::un_804D6DFC)) as u32);
    hm((*(&raw const m::un_804D6E00)) as u32);
    hm(pi((*(&raw const m::un_804D6E04))));
    hm(pi((*(&raw const m::un_804D6E08))));
    hm((*(&raw const m::un_804D6E0C)) as u32);
    hm((*(&raw const m::un_804D6E10)) as u32);
    hm((*(&raw const m::un_804D6E14)) as u32);
    hm(rh(&(*(&raw const m::gw_dev_unlock_chars_label))));
    hm(rh(&(*(&raw const m::gw_dev_unlock_stages_label))));
    hm(rh(&(*(&raw const m::lbl_803F9EF0))));
    hm(rh(&(*(&raw const m::lbl_803F9F10))));
    hm(rh(&(*(&raw const m::un_803F9F28))));
    hm(rh(&(*(&raw const m::un_803F9F38))));
    hm(rh(&(*(&raw const m::un_803F9F44))));
    hm(rh(&(*(&raw const m::un_803F9F50))));
    hm(rh(&(*(&raw const m::un_803F9F5C))));
    hm(rh(&(*(&raw const m::un_803F9F68))));
    hm(rh(&(*(&raw const m::un_803F9F74))));
    hm(rh(&(*(&raw const m::un_803F9F80))));
    hm(rh(&(*(&raw const m::un_803F9F8C))));
    hm(rh(&(*(&raw const m::un_803F9F98))));
    for i in 0..11 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803F9FA4)).entries[i]); } hm(rh(&(*(&raw const m::un_803F9FA4)).x160)); hm(rh(&(*(&raw const m::un_803F9FA4)).x16C));
    hm(rh(&(*(&raw const m::un_803FA128))));
    hash_258();
    for i in 0..3 { hm(pi((*(&raw const m::un_803FA3E8))[i])); }
    hm(rh(&(*(&raw const m::un_803FA3F4))));
    hm(rh(&(*(&raw const m::un_803FA404))));
    hm(rh(&(*(&raw const m::un_803FA414))));
    hm(rh(&(*(&raw const m::un_803FA424))));
    hm(rh(&(*(&raw const m::un_803FA434))));
    for i in 0..5 { hm(pi((*(&raw const m::un_803FA440))[i])); }
    hm(rh(&(*(&raw const m::un_803FA454))));
    hm(rh(&(*(&raw const m::un_803FA468))));
    hm(rh(&(*(&raw const m::un_803FA474))));
    hm(rh(&(*(&raw const m::un_803FA480))));
    hm(rh(&(*(&raw const m::un_803FA48C))));
    hm(rh(&(*(&raw const m::un_803FA498))));
    hm(rh(&(*(&raw const m::un_803FA4A8))));
    hm(rh(&(*(&raw const m::un_803FA4B8))));
    hm(rh(&(*(&raw const m::un_803FA4CC))));
    for i in 0..13 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FA4E0))[i]); }
    hm(rh(&(*(&raw const m::un_803FA640))));
    for i in 0..8 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FA658))[i]); }
    hm(rh(&(*(&raw const m::un_803FA758))));
    hm(rh(&(*(&raw const m::un_803FA768))));
    hm(rh(&(*(&raw const m::un_803FA77C))));
    for i in 0..4 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FA790))[i]); }
    hm(rh(&(*(&raw const m::un_803FA810))));
    hm(rh(&(*(&raw const m::un_803FA81C))));
    hm(rh(&(*(&raw const m::un_803FA828))));
    hm(rh(&(*(&raw const m::un_803FA834))));
    for i in 0..4 { hm(pi((*(&raw const m::un_803FA844))[i])); }
    hm(rh(&(*(&raw const m::un_803FA854))));
    for i in 0..6 { hm(pi((*(&raw const m::un_803FA860))[i])); }
    hm(rh(&(*(&raw const m::un_803FA878))));
    hm(rh(&(*(&raw const m::un_803FA884))));
    hm(rh(&(*(&raw const m::un_803FA894))));
    hm(rh(&(*(&raw const m::un_803FA8A4))));
    hm(rh(&(*(&raw const m::un_803FA8B4))));
    hm(rh(&(*(&raw const m::un_803FA8C4))));
    hm(rh(&(*(&raw const m::un_803FA8D8))));
    for i in 0..9 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FA8E8))[i]); }
    hm(rh(&(*(&raw const m::un_803FAA08))));
    hm(rh(&(*(&raw const m::un_803FAA14))));
    hm(rh(&(*(&raw const m::un_803FAA20))));
    hm(rh(&(*(&raw const m::un_803FAA2C))));
    hm(rh(&(*(&raw const m::un_803FAA38))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FAA44))[i]); }
    hm(rh(&(*(&raw const m::un_803FAB04))));
    hm(rh(&(*(&raw const m::un_803FAB10))));
    hm(rh(&(*(&raw const m::un_803FAB1C))));
    hm(rh(&(*(&raw const m::un_803FAB28))));
    hm(rh(&(*(&raw const m::un_803FAB34))));
    hm(rh(&(*(&raw const m::un_803FAB40))));
    hm(rh(&(*(&raw const m::un_803FAB4C))));
    hm(rh(&(*(&raw const m::un_803FAB58))));
    hm(rh(&(*(&raw const m::un_803FAB64))));
    hm(rh(&(*(&raw const m::un_803FAB70))));
    hm(rh(&(*(&raw const m::un_803FAB7C))));
    hm(rh(&(*(&raw const m::un_803FAB88))));
    hm(rh(&(*(&raw const m::un_803FAB94))));
    hm(rh(&(*(&raw const m::un_803FABA0))));
    hm(rh(&(*(&raw const m::un_803FABAC))));
    hm(rh(&(*(&raw const m::un_803FABB8))));
    hm(rh(&(*(&raw const m::un_803FABC4))));
    hm(rh(&(*(&raw const m::un_803FABD0))));
    hm(rh(&(*(&raw const m::un_803FABDC))));
    hm(rh(&(*(&raw const m::un_803FABE8))));
    hm(rh(&(*(&raw const m::un_803FABF4))));
    hm(rh(&(*(&raw const m::un_803FAC00))));
    hm(rh(&(*(&raw const m::un_803FAC0C))));
    hm(rh(&(*(&raw const m::un_803FAC18))));
    hm(rh(&(*(&raw const m::un_803FAC24))));
    hm(rh(&(*(&raw const m::un_803FAC30))));
    hm(rh(&(*(&raw const m::un_803FAC3C))));
    hm(rh(&(*(&raw const m::un_803FAC48))));
    hm(rh(&(*(&raw const m::un_803FAC54))));
    hm(rh(&(*(&raw const m::un_803FAC60))));
    hm(rh(&(*(&raw const m::un_803FAC6C))));
    hm(rh(&(*(&raw const m::un_803FAC78))));
    hm(rh(&(*(&raw const m::un_803FAC84))));
    hm(rh(&(*(&raw const m::un_803FAC90))));
    hm(rh(&(*(&raw const m::un_803FAC9C))));
    hm(rh(&(*(&raw const m::un_803FACA8))));
    hm(rh(&(*(&raw const m::un_803FACB4))));
    hm(rh(&(*(&raw const m::un_803FACC0))));
    hm(rh(&(*(&raw const m::un_803FACCC))));
    hm(rh(&(*(&raw const m::un_803FACD8))));
    hm(rh(&(*(&raw const m::un_803FACE4))));
    hm(rh(&(*(&raw const m::un_803FACF0))));
    hm(rh(&(*(&raw const m::un_803FACFC))));
    hm(rh(&(*(&raw const m::un_803FAD08))));
    hm(rh(&(*(&raw const m::un_803FAD14))));
    hm(rh(&(*(&raw const m::un_803FAD20))));
    hm(rh(&(*(&raw const m::un_803FAD2C))));
    hm(rh(&(*(&raw const m::un_803FAD38))));
    hm(rh(&(*(&raw const m::un_803FAD44))));
    hm(rh(&(*(&raw const m::un_803FAD50))));
    hm(rh(&(*(&raw const m::un_803FAD5C))));
    hm(rh(&(*(&raw const m::un_803FAD68))));
    hm(rh(&(*(&raw const m::un_803FAD74))));
    hm(rh(&(*(&raw const m::un_803FAD80))));
    hm(rh(&(*(&raw const m::un_803FAD8C))));
    hm(rh(&(*(&raw const m::un_803FAD98))));
    hm(rh(&(*(&raw const m::un_803FADA4))));
    hm(rh(&(*(&raw const m::un_803FADB0))));
    hm(rh(&(*(&raw const m::un_803FADBC))));
    hm(rh(&(*(&raw const m::un_803FADC8))));
    hm(rh(&(*(&raw const m::un_803FADD4))));
    hm(rh(&(*(&raw const m::un_803FADE0))));
    hm(rh(&(*(&raw const m::un_803FADEC))));
    hm(rh(&(*(&raw const m::un_803FADF8))));
    hm(rh(&(*(&raw const m::un_803FAE04))));
    hm(rh(&(*(&raw const m::un_803FAE10))));
    hm(rh(&(*(&raw const m::un_803FAE1C))));
    hm(rh(&(*(&raw const m::un_803FAE28))));
    hm(rh(&(*(&raw const m::un_803FAE34))));
    hm(rh(&(*(&raw const m::un_803FAE40))));
    hm(rh(&(*(&raw const m::un_803FAE4C))));
    hm(rh(&(*(&raw const m::un_803FAE58))));
    hm(rh(&(*(&raw const m::un_803FAE64))));
    hm(rh(&(*(&raw const m::un_803FAE70))));
    hm(rh(&(*(&raw const m::un_803FAE7C))));
    hm(rh(&(*(&raw const m::un_803FAE88))));
    hm(rh(&(*(&raw const m::un_803FAE94))));
    hm(rh(&(*(&raw const m::un_803FAEA0))));
    hm(rh(&(*(&raw const m::un_803FAEAC))));
    hm(rh(&(*(&raw const m::un_803FAEB8))));
    hm(rh(&(*(&raw const m::un_803FAEC4))));
    hm(rh(&(*(&raw const m::un_803FAED0))));
    hm(rh(&(*(&raw const m::un_803FAEDC))));
    hm(rh(&(*(&raw const m::un_803FAEE8))));
    hm(rh(&(*(&raw const m::un_803FAEF4))));
    hm(rh(&(*(&raw const m::un_803FAF00))));
    for i in 0..86 { hm(pi((*(&raw const m::un_803FAF0C))[i])); }
    hm(rh(&(*(&raw const m::un_803FB064))));
    hm(rh(&(*(&raw const m::un_803FB074))));
    hm(rh(&(*(&raw const m::un_803FB080))));
    hm(rh(&(*(&raw const m::un_803FB090))));
    hm(rh(&(*(&raw const m::un_803FB0A0))));
    hm(rh(&(*(&raw const m::un_803FB0B0))));
    hm(rh(&(*(&raw const m::un_803FB0C0))));
    hm(rh(&(*(&raw const m::un_803FB0D0))));
    hm(rh(&(*(&raw const m::un_803FB0E0))));
    hm(rh(&(*(&raw const m::un_803FB0F0))));
    hm(rh(&(*(&raw const m::un_803FB100))));
    hm(rh(&(*(&raw const m::un_803FB110))));
    hm(rh(&(*(&raw const m::un_803FB120))));
    hm(rh(&(*(&raw const m::un_803FB130))));
    hm(rh(&(*(&raw const m::un_803FB140))));
    hm(rh(&(*(&raw const m::un_803FB14C))));
    hm(rh(&(*(&raw const m::un_803FB158))));
    for i in 0..16 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FB168))[i]); }
    hm(rh(&(*(&raw const m::un_803FB368))));
    hm(rh(&(*(&raw const m::un_803FB378))));
    hm(rh(&(*(&raw const m::un_803FB388))));
    hm(rh(&(*(&raw const m::un_803FB394))));
    hm(rh(&(*(&raw const m::un_803FB3A4))));
    hm(rh(&(*(&raw const m::un_803FB3B0))));
    hm(rh(&(*(&raw const m::un_803FB3BC))));
    hm(rh(&(*(&raw const m::un_803FB3C8))));
    hm(rh(&(*(&raw const m::un_803FB3D4))));
    hm(rh(&(*(&raw const m::un_803FB3E0))));
    hm(rh(&(*(&raw const m::un_803FB3EC))));
    hm(rh(&(*(&raw const m::un_803FB3FC))));
    hm(rh(&(*(&raw const m::un_803FB408))));
    hm(rh(&(*(&raw const m::un_803FB414))));
    hm(rh(&(*(&raw const m::un_803FB424))));
    hm(rh(&(*(&raw const m::un_803FB434))));
    hm(rh(&(*(&raw const m::un_803FB440))));
    hm(rh(&(*(&raw const m::un_803FB44C))));
    hm(rh(&(*(&raw const m::un_803FB458))));
    hm(rh(&(*(&raw const m::un_803FB468))));
    hm(rh(&(*(&raw const m::un_803FB478))));
    hm(rh(&(*(&raw const m::un_803FB484))));
    hm(rh(&(*(&raw const m::un_803FB490))));
    hm(rh(&(*(&raw const m::un_803FB4A0))));
    hm(rh(&(*(&raw const m::un_803FB4B0))));
    hm(rh(&(*(&raw const m::un_803FB4BC))));
    hm(rh(&(*(&raw const m::un_803FB4C8))));
    hm(rh(&(*(&raw const m::un_803FB4D8))));
    hm(rh(&(*(&raw const m::un_803FB4E4))));
    hm(rh(&(*(&raw const m::un_803FB4F0))));
    hm(rh(&(*(&raw const m::un_803FB500))));
    hm(rh(&(*(&raw const m::un_803FB510))));
    hm(rh(&(*(&raw const m::un_803FB520))));
    hm(rh(&(*(&raw const m::un_803FB52C))));
    for i in 0..34 { hm(pi((*(&raw const m::un_803FB538))[i])); }
    hm(rh(&(*(&raw const m::un_803FB5C0))));
    hm(rh(&(*(&raw const m::un_803FB5D0))));
    hm(rh(&(*(&raw const m::un_803FB5DC))));
    hm(rh(&(*(&raw const m::un_803FB5E8))));
    hm(rh(&(*(&raw const m::un_803FB5F4))));
    hm(rh(&(*(&raw const m::un_803FB600))));
    for i in 0..7 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FB60C))[i]); }
    hm(rh(&(*(&raw const m::un_803FB6EC))));
    hm(rh(&(*(&raw const m::un_803FB6F8))));
    hm(rh(&(*(&raw const m::un_803FB704))));
    hm(rh(&(*(&raw const m::un_803FB710))));
    hm(rh(&(*(&raw const m::un_803FB71C))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FB728))[i]); }
    hm(rh(&(*(&raw const m::un_803FB7E8))));
    hm(rh(&(*(&raw const m::un_803FB7F4))));
    hm(rh(&(*(&raw const m::un_803FB800))));
    hm(rh(&(*(&raw const m::un_803FB80C))));
    for i in 0..4 { hm(pi((*(&raw const m::un_803FB818))[i])); }
    hm(rh(&(*(&raw const m::un_803FB828))));
    hm(rh(&(*(&raw const m::un_803FB834))));
    hm(rh(&(*(&raw const m::un_803FB840))));
    hm(rh(&(*(&raw const m::un_803FB84C))));
    hm(rh(&(*(&raw const m::un_803FB858))));
    hm(rh(&(*(&raw const m::un_803FB864))));
    for i in 0..7 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FB870))[i]); }
    hm(rh(&(*(&raw const m::un_803FB950))));
    hm(rh(&(*(&raw const m::un_803FB95C))));
    hm(rh(&(*(&raw const m::un_803FB968))));
    hm(rh(&(*(&raw const m::un_803FB974))));
    hm(rh(&(*(&raw const m::un_803FB980))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FB98C))[i]); }
    hm(rh(&(*(&raw const m::un_803FBA4C))));
    hm(rh(&(*(&raw const m::un_803FBA5C))));
    hm(rh(&(*(&raw const m::un_803FBA6C))));
    hm(rh(&(*(&raw const m::un_803FBA7C))));
    hm(rh(&(*(&raw const m::un_803FBA8C))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FBA9C))[i]); }
    hm(rh(&(*(&raw const m::un_803FBB5C))));
    hm(rh(&(*(&raw const m::un_803FBB68))));
    hm(rh(&(*(&raw const m::un_803FBB74))));
    hm(rh(&(*(&raw const m::un_803FBB80))));
    hm(rh(&(*(&raw const m::un_803FBB8C))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FBB98))[i]); }
    hm(rh(&(*(&raw const m::un_803FBC58))));
    hm(rh(&(*(&raw const m::un_803FBC6C))));
    hm(rh(&(*(&raw const m::un_803FBC7C))));
    hm(rh(&(*(&raw const m::un_803FBC8C))));
    hm(rh(&(*(&raw const m::un_803FBC9C))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FBCAC))[i]); }
    hm(rh(&(*(&raw const m::un_803FBD6C))));
    hm(rh(&(*(&raw const m::un_803FBD80))));
    hm(rh(&(*(&raw const m::un_803FBD90))));
    hm(rh(&(*(&raw const m::un_803FBDA0))));
    hm(rh(&(*(&raw const m::un_803FBDB0))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FBDC0))[i]); }
    hm(rh(&(*(&raw const m::un_803FBE80))));
    hm(rh(&(*(&raw const m::un_803FBE8C))));
    hm(rh(&(*(&raw const m::un_803FBE98))));
    hm(rh(&(*(&raw const m::un_803FBEA4))));
    hm(rh(&(*(&raw const m::un_803FBEB0))));
    hm(rh(&(*(&raw const m::un_803FBEBC))));
    hm(rh(&(*(&raw const m::un_803FBEC8))));
    hm(rh(&(*(&raw const m::un_803FBED4))));
    hm(rh(&(*(&raw const m::un_803FBEE4))));
    hm(rh(&(*(&raw const m::un_803FBEF4))));
    hm(rh(&(*(&raw const m::un_803FBF00))));
    hm(rh(&(*(&raw const m::un_803FBF10))));
    hm(rh(&(*(&raw const m::un_803FBF20))));
    hm(rh(&(*(&raw const m::un_803FBF30))));
    hm(rh(&(*(&raw const m::un_803FBF40))));
    hm(rh(&(*(&raw const m::un_803FBF50))));
    hm(rh(&(*(&raw const m::un_803FBF5C))));
    hm(rh(&(*(&raw const m::un_803FBF68))));
    for i in 0..18 { hm(pi((*(&raw const m::un_803FBF74))[i])); }
    hm(rh(&(*(&raw const m::un_803FBFBC))));
    hm(rh(&(*(&raw const m::un_803FBFCC))));
    hm(rh(&(*(&raw const m::un_803FBFD8))));
    hm(rh(&(*(&raw const m::un_803FBFE4))));
    hm(rh(&(*(&raw const m::un_803FBFF0))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FBFFC))[i]); }
    hm(rh(&(*(&raw const m::un_803FC0BC))));
    hm(rh(&(*(&raw const m::un_803FC0CC))));
    hm(rh(&(*(&raw const m::un_803FC0D8))));
    hm(rh(&(*(&raw const m::un_803FC0E4))));
    hm(rh(&(*(&raw const m::un_803FC0F0))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC0FC))[i]); }
    hm(rh(&(*(&raw const m::un_803FC1BC))));
    hm(rh(&(*(&raw const m::un_803FC1C8))));
    hm(rh(&(*(&raw const m::un_803FC1D4))));
    for i in 0..3 { hm(pi((*(&raw const m::un_803FC1E0))[i])); }
    hm(rh(&(*(&raw const m::un_803FC1EC))));
    hm(rh(&(*(&raw const m::un_803FC1FC))));
    hm(rh(&(*(&raw const m::un_803FC208))));
    hm(rh(&(*(&raw const m::un_803FC214))));
    hm(rh(&(*(&raw const m::un_803FC220))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC22C))[i]); }
    hm(rh(&(*(&raw const m::un_803FC2EC))));
    hm(rh(&(*(&raw const m::un_803FC420))));
    hm(rh(&(*(&raw const m::un_803FC42C))));
    hm(rh(&(*(&raw const m::un_803FC440))));
    hm(rh(&(*(&raw const m::un_803FC454))));
    hm(rh(&(*(&raw const m::un_803FC468))));
    hm(rh(&(*(&raw const m::un_803FC47C))));
    hm(rh(&(*(&raw const m::un_803FC490))));
    hm(rh(&(*(&raw const m::un_803FC4A4))));
    hm(rh(&(*(&raw const m::un_803FC4B8))));
    for i in 0..10 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC4CC))[i]); }
    hm(rh(&(*(&raw const m::un_803FC60C))));
    hm(rh(&(*(&raw const m::un_803FC61C))));
    hm(rh(&(*(&raw const m::un_803FC62C))));
    for i in 0..4 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC63C))[i]); }
    hm(rh(&(*(&raw const m::un_803FC6BC))));
    hm(rh(&(*(&raw const m::un_803FC6D0))));
    hm(rh(&(*(&raw const m::un_803FC6DC))));
    hm(rh(&(*(&raw const m::un_803FC6E8))));
    hm(rh(&(*(&raw const m::un_803FC6F4))));
    hm(rh(&(*(&raw const m::un_803FC700))));
    for i in 0..7 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC70C))[i]); }
    hm(rh(&(*(&raw const m::un_803FC7EC))));
    hm(rh(&(*(&raw const m::un_803FC804))));
    hm(rh(&(*(&raw const m::un_803FC810))));
    hm(rh(&(*(&raw const m::un_803FC81C))));
    for i in 0..5 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC828))[i]); }
    for i in 0..5 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FC8C8))[i]); }
    hm(rh(&(*(&raw const m::un_803FC968))));
    hm(rh(&(*(&raw const m::un_803FC97C))));
    hm(rh(&(*(&raw const m::un_803FC98C))));
    hm(rh(&(*(&raw const m::un_803FC998))));
    hm(rh(&(*(&raw const m::un_803FC9A8))));
    hm(rh(&(*(&raw const m::un_803FC9BC))));
    hm(rh(&(*(&raw const m::un_803FC9D0))));
    hm(rh(&(*(&raw const m::un_803FC9E8))));
    hm(rh(&(*(&raw const m::un_803FC9FC))));
    hm(rh(&(*(&raw const m::un_803FCA10))));
    hm(rh(&(*(&raw const m::un_803FCA20))));
    hm(rh(&(*(&raw const m::un_803FCA30))));
    for i in 0..13 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FCA40))[i]); }
    hm(rh(&(*(&raw const m::un_803FCBE0))));
    hm(rh(&(*(&raw const m::un_803FCBF4))));
    hm(rh(&(*(&raw const m::un_803FCC08))));
    hm(rh(&(*(&raw const m::un_803FCC18))));
    hm(rh(&(*(&raw const m::un_803FCC28))));
    for i in 0..6 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FCC38))[i]); }
    hm(rh(&(*(&raw const m::un_803FCCF8))));
    hm(rh(&(*(&raw const m::un_803FCD04))));
    for i in 0..3 { hm(pi((*(&raw const m::un_803FCD10))[i])); }
    hm(rh(&(*(&raw const m::un_803FCD1C))));
    for i in 0..5 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FCD34))[i]); }
    hm(rh(&(*(&raw const m::un_803FCDD4))));
    hm(rh(&(*(&raw const m::un_803FCDE8))));
    hm(rh(&(*(&raw const m::un_803FCDFC))));
    hm(rh(&(*(&raw const m::un_803FCE0C))));
    hm(rh(&(*(&raw const m::un_803FCE1C))));
    hm(rh(&(*(&raw const m::un_803FCE34))));
    for i in 0..7 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FCE4C))[i]); }
    hm(rh(&(*(&raw const m::un_803FCF2C))));
    hm(rh(&(*(&raw const m::un_803FCF38))));
    hm(rh(&(*(&raw const m::un_803FCF44))));
    hm(rh(&(*(&raw const m::un_803FCF50))));
    for i in 0..13 { hm(pi((*(&raw const m::un_803FCF5C))[i])); }
    hm(rh(&(*(&raw const m::un_803FCF90))));
    hm(rh(&(*(&raw const m::un_803FCFA0))));
    hm(rh(&(*(&raw const m::un_803FCFAC))));
    hm(rh(&(*(&raw const m::un_803FCFC0))));
    hm(rh(&(*(&raw const m::un_803FCFD4))));
    hm(rh(&(*(&raw const m::un_803FCFE0))));
    hm(rh(&(*(&raw const m::un_803FCFEC))));
    hm(rh(&(*(&raw const m::un_803FD000))));
    hm(rh(&(*(&raw const m::un_803FD00C))));
    hm(rh(&(*(&raw const m::un_803FD01C))));
    hm(rh(&(*(&raw const m::un_803FD034))));
    hm(rh(&(*(&raw const m::un_803FD044))));
    hm(rh(&(*(&raw const m::un_803FD058))));
    for i in 0..14 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FD064))[i]); }
    hm(rh(&(*(&raw const m::un_803FD224))));
    hm(rh(&(*(&raw const m::un_803FD230))));
    hm(rh(&(*(&raw const m::un_803FD23C))));
    hm(rh(&(*(&raw const m::un_803FD24C))));
    hm(rh(&(*(&raw const m::un_803FD258))));
    hm(rh(&(*(&raw const m::un_803FD264))));
    hm(pi((*(&raw const m::un_803FD274)).image_ptr)); hm((*(&raw const m::un_803FD274)).width as u32); hm((*(&raw const m::un_803FD274)).height as u32); hm((*(&raw const m::un_803FD274)).format as u32); hm((*(&raw const m::un_803FD274)).mipmap); hm((*(&raw const m::un_803FD274)).minLOD.to_bits()); hm((*(&raw const m::un_803FD274)).maxLOD.to_bits());
    hm(rh(&(*(&raw const m::un_803FD28C))));
    hm(rh(&(*(&raw const m::un_803FD29C))));
    hm(rh(&(*(&raw const m::un_803FD2AC))));
    hm(rh(&(*(&raw const m::un_803FD2BC))));
    hm(rh(&(*(&raw const m::un_803FD2D0))));
    hm(rh(&(*(&raw const m::un_803FD2DC))));
    hm(rh(&(*(&raw const m::un_803FD2E8))));
    hm(rh(&(*(&raw const m::un_803FD2F4))));
    hm(rh(&(*(&raw const m::un_803FD304))));
    for i in 0..13 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FD310))[i]); }
    for i in 0..5 { hm(pi((*(&raw const m::un_803FD4B0))[i])); }
    hm(rh(&(*(&raw const m::un_803FD4C4))));
    hm(rh(&(*(&raw const m::un_803FD4D0))));
    for i in 0..4 { hm(pi((*(&raw const m::un_803FD4DC))[i])); }
    hm(rh(&(*(&raw const m::un_803FD4EC))));
    hm(rh(&(*(&raw const m::un_803FD500))));
    hm(rh(&(*(&raw const m::un_803FD50C))));
    hm(rh(&(*(&raw const m::un_803FD51C))));
    hm(rh(&(*(&raw const m::un_803FD52C))));
    hm(rh(&(*(&raw const m::un_803FD53C))));
    hm(rh(&(*(&raw const m::un_803FD54C))));
    hm(rh(&(*(&raw const m::un_803FD55C))));
    hm(rh(&(*(&raw const m::un_803FD56C))));
    hm(rh(&(*(&raw const m::un_803FD57C))));
    for i in 0..16 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FD58C))[i]); }
    hm(rh(&(*(&raw const m::un_803FD78C))));
    hm(rh(&(*(&raw const m::un_803FD79C))));
    hm(rh(&(*(&raw const m::un_803FD7A8))));
    hm(rh(&(*(&raw const m::un_803FD7B4))));
    hm(rh(&(*(&raw const m::un_803FD7C0))));
    hm(rh(&(*(&raw const m::un_803FD7CC))));
    hm(rh(&(*(&raw const m::un_803FD7DC))));
    hm(rh(&(*(&raw const m::un_803FD7E8))));
    hm(rh(&(*(&raw const m::un_803FD7F4))));
    hm(rh(&(*(&raw const m::un_803FD800))));
    hm(rh(&(*(&raw const m::un_803FD80C))));
    hm(rh(&(*(&raw const m::un_803FD818))));
    hm(rh(&(*(&raw const m::un_803FD824))));
    hm(rh(&(*(&raw const m::un_803FD830))));
    hm(rh(&(*(&raw const m::un_803FD83C))));
    hm(rh(&(*(&raw const m::un_803FD848))));
    hm(rh(&(*(&raw const m::un_803FD854))));
    for i in 0..19 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FD864))[i]); }
    hm(rh(&(*(&raw const m::un_803FDAC4))));
    hm(rh(&(*(&raw const m::un_803FDAD4))));
    hm(rh(&(*(&raw const m::un_803FDAE4))));
    for i in 0..4 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FDAF8))[i]); }
    hm(rh(&(*(&raw const m::un_803FDB78))));
    hm(rh(&(*(&raw const m::un_803FDB88))));
    for i in 0..4 { g::hash_un_80304138_objalloc_t_x8(&(*(&raw const m::un_803FDB9C))[i]); }
}

fn pick() -> u32 {
    let r = rnd();
    if r & 7 == 7 {
        return fv() & 15;
    }
    match (r >> 3) % 3 {
        0 => 0,
        1 => 1,
        _ => 6,
    }
}

unsafe fn setup() {
    m::un_803F9FA4 = PRISTINE.unwrap();
    m::un_803FD274 = PRISTINE_IMG.unwrap();
    m::un_803FA258.x0 = fv() as i32;
    m::un_803FA258.x4[0] = fv() as i32;
    m::un_803FA258.x4[1] = fv() as i32;
    m::un_803FA258.x4[2] = fv() as i32;
    m::un_803FA258.x4[3] = fv() as i32;
    m::un_803FA258.x14[0] = fv() as i32;
    m::un_803FA258.x14[1] = fv() as i32;
    m::un_803FA258.x14[2] = fv() as i32;
    m::un_803FA258.x14[3] = fv() as i32;
    m::un_803FA258.x24[0] = fv() as i32;
    m::un_803FA258.x24[1] = fv() as i32;
    m::un_803FA258.x24[2] = fv() as i32;
    m::un_803FA258.x24[3] = fv() as i32;
    m::un_803FA258.x34 = fv() as i32;
    m::un_803FA258.x38[0] = fv() as i32;
    m::un_803FA258.x38[1] = fv() as i32;
    m::un_803FA258.x38[2] = fv() as i32;
    m::un_803FA258.x38[3] = fv() as i32;
    m::un_803FA258.x48[0] = fv() as i32;
    m::un_803FA258.x48[1] = fv() as i32;
    m::un_803FA258.x48[2] = fv() as i32;
    m::un_803FA258.x48[3] = fv() as i32;
    m::un_803FA258.x58[0] = fv() as i32;
    m::un_803FA258.x58[1] = fv() as i32;
    m::un_803FA258.x58[2] = fv() as i32;
    m::un_803FA258.x58[3] = fv() as i32;
    m::un_803FA258.x68[0] = fv() as i32;
    m::un_803FA258.x68[1] = fv() as i32;
    m::un_803FA258.x68[2] = fv() as i32;
    m::un_803FA258.x68[3] = fv() as i32;
    m::un_803FA258.x78[0] = rg();
    m::un_803FA258.x78[1] = rg();
    m::un_803FA258.x78[2] = rg();
    m::un_803FA258.x78[3] = rg();
    m::un_803FA258.x88[0] = rg();
    m::un_803FA258.x88[1] = rg();
    m::un_803FA258.x88[2] = rg();
    m::un_803FA258.x88[3] = rg();
    m::un_803FA258.x98[0] = rg();
    m::un_803FA258.x98[1] = rg();
    m::un_803FA258.x98[2] = rg();
    m::un_803FA258.x98[3] = rg();
    m::un_803FA258.xA8[0] = fv() as i32;
    m::un_803FA258.xA8[1] = fv() as i32;
    m::un_803FA258.xA8[2] = fv() as i32;
    m::un_803FA258.xA8[3] = fv() as i32;
    m::un_803FA258.xB8[0] = fv() as i32;
    m::un_803FA258.xB8[1] = fv() as i32;
    m::un_803FA258.xB8[2] = fv() as i32;
    m::un_803FA258.xB8[3] = fv() as i32;
    m::un_803FA258.xC8 = fv() as i32;
    m::un_803FA258.xCC[0] = fv() as i32;
    m::un_803FA258.xCC[1] = fv() as i32;
    m::un_803FA258.xCC[2] = fv() as i32;
    m::un_803FA258.xCC[3] = fv() as i32;
    m::un_803FA258.xDC[0] = fv() as i32;
    m::un_803FA258.xDC[1] = fv() as i32;
    m::un_803FA258.xDC[2] = fv() as i32;
    m::un_803FA258.xDC[3] = fv() as i32;
    m::un_803FA258.xEC = rg();
    m::un_803FA258.xF0 = (fv() & 0xFF) as u16;
    m::un_803FA258.xF4 = rnd() as u8;
    m::un_803FA258.xF5 = rnd() as u8;
    m::un_803FA258.xF6 = rnd() as u8;
    m::un_803FA258.xF7 = rnd() as u8;
    m::un_803FA258.xF8 = fv() as i32;
    m::un_803FA258.xFC = fv() as i32;
    m::un_803FA258.x100 = fv() as i32;
    m::un_803FA258.x104 = fv() as i32;
    m::un_803FA258.x108 = fv() as i32;
    m::un_803FA258.x10C = fv() as i32;
    m::un_803FA258.x110 = fv() as i32;
    m::un_803FA258.x114 = fv() as i32;
    m::un_803FA258.x118 = fv() as i32;
    m::un_803FA258.x11C = fv() as i32;
    m::un_803FA258.x120 = fv() as i32;
    m::un_803FA258.x124 = fv() as i32;
    m::un_803FA258.x128 = fv() as i32;
    m::un_803FA258.x12C = fv() as i32;
    m::un_803FA258.x130 = fv() as i32;
    m::un_803FA258.x134 = fv() as i32;
    m::un_803FA258.x138 = if rnd() & 1 != 0 { &raw mut MEM as *mut c_void } else { null_mut() };
    m::un_803FA258.x13C = fv() as i32;
    m::un_803FA258.x140 = fv() as i32;
    m::un_803FA258.x144[0] = fv() as i32;
    m::un_803FA258.x144[1] = fv() as i32;
    m::un_803FA258.x144[2] = fv() as i32;
    m::un_803FA258.x144[3] = fv() as i32;
    m::un_803FA258.x144[4] = fv() as i32;
    m::un_803FA258.x144[5] = fv() as i32;
    m::un_803FA258.x144[6] = fv() as i32;
    m::un_803FA258.x144[7] = fv() as i32;
    m::un_803FA258.x144[8] = fv() as i32;
    m::un_803FA258.x144[9] = fv() as i32;
    m::un_803FA258.x144[10] = fv() as i32;
    m::un_803FA258.x144[11] = fv() as i32;
    m::un_803FA258.x144[12] = fv() as i32;
    m::un_803FA258.x144[13] = fv() as i32;
    m::un_803FA258.x144[14] = fv() as i32;
    m::un_803FA258.x144[15] = fv() as i32;
    m::un_803FA258.x144[16] = fv() as i32;
    m::un_803FA258.x188 = fv() as i32;
    m::un_803FA258.x18C = fv() as i32;
    m::un_804D5850 = (fv()) as _;
    m::un_804D5854 = (fv()) as _;
    m::un_804D5858 = (fv()) as _;
    m::un_804D585C = (fv()) as _;
    m::un_804D5908 = (fv()) as _;
    m::un_804D590C = (fv()) as _;
    m::un_804D5910 = (fv()) as _;
    m::un_804D6DAC = (fv()) as _;
    m::un_804D6DB0 = (fv() % 56) as _;
    m::un_804D6DB4 = (fv() % 200) as _;
    m::un_804D6DB8 = (fv()) as _;
    m::un_804D6DBC = (fv()) as _;
    m::un_804D6DC0 = (fv()) as _;
    m::un_804D6DC8 = (fv()) as _;
    m::un_804D6DCC = (fv()) as _;
    m::un_804D6DD0 = ((fv() % 5000) as f32) / 4.0;
    m::un_804D6DD4 = (fv()) as _;
    m::un_804D6DD8 = (fv() % 5) as _;
    m::un_804D6DDC = (fv() % 1000) as _;
    m::un_804D6DE0 = (fv()) as _;
    m::un_804D6DE4 = (fv()) as _;
    m::un_804D6DE8 = (fv()) as _;
    m::un_804D6DEC = (fv()) as _;
    m::un_804D6DF0 = (fv()) as _;
    m::un_804D6DF4 = (fv()) as _;
    m::un_804D6DF8 = (fv()) as _;
    m::un_804D6DFC = (fv()) as _;
    m::un_804D6E00 = (fv() % 4) as _;
    m::un_804D6E0C = (fv()) as _;
    m::un_804D6E10 = (fv()) as _;
    m::un_804D6E14 = (fv()) as _;
    m::un_804D6E04 = if rnd() & 1 != 0 { &raw mut MEM as *mut c_void } else { null_mut() };
    m::un_804D6E08 = if rnd() & 1 != 0 { &raw mut GO[(rnd() % 4) as usize] } else { null_mut() };
    m::un_804D6DA8 = &raw mut SYMBUF as *mut i32;
    for i in 0..4 {
        SYMBUF[i] = fv64();
    }
    SYMBUF[4] = fv64();
    SYMBUF[7] = fv64();
    SYMBUF[5] = &raw mut A5 as usize as u64;
    SYMBUF[6] = &raw mut A6 as usize as u64;
    for i in 0..256 {
        A5[i] = fv() as i32;
    }
    for i in 0..64 {
        A6[i] = (rnd() % 4) as i32;
    }
    for i in 0..16 {
        ARR[i] = fv() as i32;
    }
    OA = core::mem::zeroed();
    DT = core::mem::zeroed();
    GO = core::mem::zeroed();
    SOB = core::mem::zeroed();
    g::fill_un_80304138_objalloc_t(&mut OA);
    OA.x4 = &raw mut DT;
    DT.scale_x = rw();
    DT.scale_y = rw();
    for i in 0..4 {
        g::fill_HSD_GObj(&mut GO[i]);
    }
    SOB.x10 = rw();
    SOB.x14 = rw();
    SOB.x40 = fv();
    for i in 0..5 {
        U32S[i] = fv();
    }
    for i in 0..2 {
        I32S[i] = fv() as i32;
    }
    for i in 0..3 {
        GMM[i].c_kind = rnd() as i8;
        GMM[i].stocks = rnd() as u8;
        GMM[i].color = rnd() as u8;
        GMM[i].cpu_level = rnd() as u8;
        GMM[i].nametag = rnd() as u8;
        GMM[i].x5 = rnd() as u8;
    }
    ALLS.x0.x0.ckind = rnd() as i8;
    ALLS.x0.x0.color = rnd() as u8;
    for i in 0..6656 {
        m::ext::gmMainLib_8045A6C0[i] = rnd() as u8;
    }
    O0.x0 = rnd() as u16;
    O0.x2 = rnd() as u16;
    O0.x4 = fv() as i32;
    O0.x8 = fv() as i32;
    for i in 0..32 {
        *(&raw mut O1 as *mut u8).add(i) = rnd() as u8;
    }
    O2.x0 = if rnd() & 1 != 0 { &raw mut MEM as *mut c_void } else { null_mut() };
    O2.x4 = fv() as i32;
    g::fill_StartMeleeData(&mut SMD);
}

unsafe fn hash_all() {
    hm(m::un_804D5850 as u32);
    hm(m::un_804D5854 as u32);
    hm(m::un_804D5858 as u32);
    hm(m::un_804D585C as u32);
    hm(m::un_804D5908 as u32);
    hm(m::un_804D590C as u32);
    hm(m::un_804D5910 as u32);
    hm(m::un_804D6DAC as u32);
    hm(m::un_804D6DB0 as u32);
    hm(m::un_804D6DB4 as u32);
    hm(m::un_804D6DB8 as u32);
    hm(m::un_804D6DBC as u32);
    hm(m::un_804D6DC0 as u32);
    hm(m::un_804D6DC8 as u32);
    hm(m::un_804D6DCC as u32);
    hm(m::un_804D6DD0.to_bits());
    hm(m::un_804D6DD4 as u32);
    hm(m::un_804D6DD8 as u32);
    hm(m::un_804D6DDC as u32);
    hm(m::un_804D6DE0 as u32);
    hm(m::un_804D6DE4 as u32);
    hm(m::un_804D6DE8 as u32);
    hm(m::un_804D6DEC as u32);
    hm(m::un_804D6DF0 as u32);
    hm(m::un_804D6DF4 as u32);
    hm(m::un_804D6DF8 as u32);
    hm(m::un_804D6DFC as u32);
    hm(m::un_804D6E00 as u32);
    hm(m::un_804D6E0C as u32);
    hm(m::un_804D6E10 as u32);
    hm(m::un_804D6E14 as u32);
    hm(pi(m::un_804D6E04));
    hm(pi(m::un_804D6E08));
    hm(pi(m::un_804D6DA8));
    hash_258();
    for i in 0..11 {
        g::hash_un_80304138_objalloc_t_x8(&m::un_803F9FA4.entries[i]);
    }
    hm(pi(m::un_803FD274.image_ptr));
    g::hash_un_80304138_objalloc_t(&OA);
    hm(DT.scale_x.to_bits());
    hm(DT.scale_y.to_bits());
    for i in 0..4 {
        g::hash_HSD_GObj(&GO[i]);
    }
    hm(SOB.x10.to_bits());
    hm(SOB.x14.to_bits());
    hm(SOB.x40);
    for i in 0..5 {
        hm(U32S[i]);
    }
    for i in 0..2 {
        hm(I32S[i] as u32);
    }
    for i in 0..3 {
        hm(GMM[i].c_kind as i32 as u32);
        hm(GMM[i].stocks as u32);
        hm(GMM[i].color as u32);
        hm(GMM[i].cpu_level as u32);
        hm(GMM[i].nametag as u32);
        hm(GMM[i].x5 as u32);
    }
    hm(ALLS.x0.x0.ckind as i32 as u32);
    hm(ALLS.x0.x0.color as u32);
    hm(rh(&*(&raw const m::ext::gmMainLib_8045A6C0 as *const [u8; 6656])));
    hm(rh(&ARR));
    hm(O0.x0 as u32);
    hm(O0.x2 as u32);
    hm(O0.x4 as u32);
    hm(O0.x8 as u32);
    hm(rh(&*(&raw const O1 as *const [u8; 32])));
    hm(pi(O2.x0));
    hm(O2.x4 as u32);
    g::hash_StartMeleeData(&SMD);
}

#[test]
fn soundtest_matches_c() {
    unsafe {
        g::install();
        const KC: [u32; 114] = [1, 300, 50, 300, 400, 300, 1500, 300, 100, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250];
        preg(&raw const m::un_804D5850, core::mem::size_of_val(&*(&raw const m::un_804D5850)));
    preg(&raw const m::un_804D5854, core::mem::size_of_val(&*(&raw const m::un_804D5854)));
    preg(&raw const m::un_804D5858, core::mem::size_of_val(&*(&raw const m::un_804D5858)));
    preg(&raw const m::un_804D585C, core::mem::size_of_val(&*(&raw const m::un_804D585C)));
    preg(&raw const m::un_804D5860, core::mem::size_of_val(&*(&raw const m::un_804D5860)));
    preg(&raw const m::un_804D5868, core::mem::size_of_val(&*(&raw const m::un_804D5868)));
    preg(&raw const m::un_804D5870, core::mem::size_of_val(&*(&raw const m::un_804D5870)));
    preg(&raw const m::un_804D5878, core::mem::size_of_val(&*(&raw const m::un_804D5878)));
    preg(&raw const m::un_804D587C, core::mem::size_of_val(&*(&raw const m::un_804D587C)));
    preg(&raw const m::un_804D5880, core::mem::size_of_val(&*(&raw const m::un_804D5880)));
    preg(&raw const m::un_804D5888, core::mem::size_of_val(&*(&raw const m::un_804D5888)));
    preg(&raw const m::un_804D5890, core::mem::size_of_val(&*(&raw const m::un_804D5890)));
    preg(&raw const m::un_804D5898, core::mem::size_of_val(&*(&raw const m::un_804D5898)));
    preg(&raw const m::un_804D58A0, core::mem::size_of_val(&*(&raw const m::un_804D58A0)));
    preg(&raw const m::un_804D58A8, core::mem::size_of_val(&*(&raw const m::un_804D58A8)));
    preg(&raw const m::un_804D58B0, core::mem::size_of_val(&*(&raw const m::un_804D58B0)));
    preg(&raw const m::un_804D58B8, core::mem::size_of_val(&*(&raw const m::un_804D58B8)));
    preg(&raw const m::un_804D58C0, core::mem::size_of_val(&*(&raw const m::un_804D58C0)));
    preg(&raw const m::un_804D58C8, core::mem::size_of_val(&*(&raw const m::un_804D58C8)));
    preg(&raw const m::un_804D58D0, core::mem::size_of_val(&*(&raw const m::un_804D58D0)));
    preg(&raw const m::un_804D58D8, core::mem::size_of_val(&*(&raw const m::un_804D58D8)));
    preg(&raw const m::un_804D58E0, core::mem::size_of_val(&*(&raw const m::un_804D58E0)));
    preg(&raw const m::un_804D58E4, core::mem::size_of_val(&*(&raw const m::un_804D58E4)));
    preg(&raw const m::un_804D58EC, core::mem::size_of_val(&*(&raw const m::un_804D58EC)));
    preg(&raw const m::un_804D58F4, core::mem::size_of_val(&*(&raw const m::un_804D58F4)));
    preg(&raw const m::un_804D58FC, core::mem::size_of_val(&*(&raw const m::un_804D58FC)));
    preg(&raw const m::un_804D5908, core::mem::size_of_val(&*(&raw const m::un_804D5908)));
    preg(&raw const m::un_804D590C, core::mem::size_of_val(&*(&raw const m::un_804D590C)));
    preg(&raw const m::un_804D5910, core::mem::size_of_val(&*(&raw const m::un_804D5910)));
    preg(&raw const m::un_804D5914, core::mem::size_of_val(&*(&raw const m::un_804D5914)));
    preg(&raw const m::un_804D591C, core::mem::size_of_val(&*(&raw const m::un_804D591C)));
    preg(&raw const m::un_804D5924, core::mem::size_of_val(&*(&raw const m::un_804D5924)));
    preg(&raw const m::un_804D592C, core::mem::size_of_val(&*(&raw const m::un_804D592C)));
    preg(&raw const m::un_804D5934, core::mem::size_of_val(&*(&raw const m::un_804D5934)));
    preg(&raw const m::un_804D593C, core::mem::size_of_val(&*(&raw const m::un_804D593C)));
    preg(&raw const m::un_804D5944, core::mem::size_of_val(&*(&raw const m::un_804D5944)));
    preg(&raw const m::un_804D594C, core::mem::size_of_val(&*(&raw const m::un_804D594C)));
    preg(&raw const m::un_804D5954, core::mem::size_of_val(&*(&raw const m::un_804D5954)));
    preg(&raw const m::un_804D595C, core::mem::size_of_val(&*(&raw const m::un_804D595C)));
    preg(&raw const m::un_804D5964, core::mem::size_of_val(&*(&raw const m::un_804D5964)));
    preg(&raw const m::un_804D5968, core::mem::size_of_val(&*(&raw const m::un_804D5968)));
    preg(&raw const m::un_804D5970, core::mem::size_of_val(&*(&raw const m::un_804D5970)));
    preg(&raw const m::un_804D5978, core::mem::size_of_val(&*(&raw const m::un_804D5978)));
    preg(&raw const m::un_804D5980, core::mem::size_of_val(&*(&raw const m::un_804D5980)));
    preg(&raw const m::un_804D5988, core::mem::size_of_val(&*(&raw const m::un_804D5988)));
    preg(&raw const m::un_804D5990, core::mem::size_of_val(&*(&raw const m::un_804D5990)));
    preg(&raw const m::un_804D5998, core::mem::size_of_val(&*(&raw const m::un_804D5998)));
    preg(&raw const m::un_804D59A0, core::mem::size_of_val(&*(&raw const m::un_804D59A0)));
    preg(&raw const m::un_804D59A8, core::mem::size_of_val(&*(&raw const m::un_804D59A8)));
    preg(&raw const m::un_804D59B0, core::mem::size_of_val(&*(&raw const m::un_804D59B0)));
    preg(&raw const m::un_804D59B8, core::mem::size_of_val(&*(&raw const m::un_804D59B8)));
    preg(&raw const m::un_804D59C0, core::mem::size_of_val(&*(&raw const m::un_804D59C0)));
    preg(&raw const m::un_804D59C8, core::mem::size_of_val(&*(&raw const m::un_804D59C8)));
    preg(&raw const m::un_804D59D0, core::mem::size_of_val(&*(&raw const m::un_804D59D0)));
    preg(&raw const m::un_804D59D8, core::mem::size_of_val(&*(&raw const m::un_804D59D8)));
    preg(&raw const m::un_804D59E0, core::mem::size_of_val(&*(&raw const m::un_804D59E0)));
    preg(&raw const m::un_804D59E8, core::mem::size_of_val(&*(&raw const m::un_804D59E8)));
    preg(&raw const m::un_804D59F0, core::mem::size_of_val(&*(&raw const m::un_804D59F0)));
    preg(&raw const m::un_804D59F8, core::mem::size_of_val(&*(&raw const m::un_804D59F8)));
    preg(&raw const m::un_804D6DA8, core::mem::size_of_val(&*(&raw const m::un_804D6DA8)));
    preg(&raw const m::un_804D6DAC, core::mem::size_of_val(&*(&raw const m::un_804D6DAC)));
    preg(&raw const m::un_804D6DB0, core::mem::size_of_val(&*(&raw const m::un_804D6DB0)));
    preg(&raw const m::un_804D6DB4, core::mem::size_of_val(&*(&raw const m::un_804D6DB4)));
    preg(&raw const m::un_804D6DB8, core::mem::size_of_val(&*(&raw const m::un_804D6DB8)));
    preg(&raw const m::un_804D6DBC, core::mem::size_of_val(&*(&raw const m::un_804D6DBC)));
    preg(&raw const m::un_804D6DC0, core::mem::size_of_val(&*(&raw const m::un_804D6DC0)));
    preg(&raw const m::un_804D6DC8, core::mem::size_of_val(&*(&raw const m::un_804D6DC8)));
    preg(&raw const m::un_804D6DCC, core::mem::size_of_val(&*(&raw const m::un_804D6DCC)));
    preg(&raw const m::un_804D6DD0, core::mem::size_of_val(&*(&raw const m::un_804D6DD0)));
    preg(&raw const m::un_804D6DD4, core::mem::size_of_val(&*(&raw const m::un_804D6DD4)));
    preg(&raw const m::un_804D6DD8, core::mem::size_of_val(&*(&raw const m::un_804D6DD8)));
    preg(&raw const m::un_804D6DDC, core::mem::size_of_val(&*(&raw const m::un_804D6DDC)));
    preg(&raw const m::un_804D6DE0, core::mem::size_of_val(&*(&raw const m::un_804D6DE0)));
    preg(&raw const m::un_804D6DE4, core::mem::size_of_val(&*(&raw const m::un_804D6DE4)));
    preg(&raw const m::un_804D6DE8, core::mem::size_of_val(&*(&raw const m::un_804D6DE8)));
    preg(&raw const m::un_804D6DEC, core::mem::size_of_val(&*(&raw const m::un_804D6DEC)));
    preg(&raw const m::un_804D6DF0, core::mem::size_of_val(&*(&raw const m::un_804D6DF0)));
    preg(&raw const m::un_804D6DF4, core::mem::size_of_val(&*(&raw const m::un_804D6DF4)));
    preg(&raw const m::un_804D6DF8, core::mem::size_of_val(&*(&raw const m::un_804D6DF8)));
    preg(&raw const m::un_804D6DFC, core::mem::size_of_val(&*(&raw const m::un_804D6DFC)));
    preg(&raw const m::un_804D6E00, core::mem::size_of_val(&*(&raw const m::un_804D6E00)));
    preg(&raw const m::un_804D6E04, core::mem::size_of_val(&*(&raw const m::un_804D6E04)));
    preg(&raw const m::un_804D6E08, core::mem::size_of_val(&*(&raw const m::un_804D6E08)));
    preg(&raw const m::un_804D6E0C, core::mem::size_of_val(&*(&raw const m::un_804D6E0C)));
    preg(&raw const m::un_804D6E10, core::mem::size_of_val(&*(&raw const m::un_804D6E10)));
    preg(&raw const m::un_804D6E14, core::mem::size_of_val(&*(&raw const m::un_804D6E14)));
    preg(&raw const m::gw_dev_unlock_chars_label, core::mem::size_of_val(&*(&raw const m::gw_dev_unlock_chars_label)));
    preg(&raw const m::gw_dev_unlock_stages_label, core::mem::size_of_val(&*(&raw const m::gw_dev_unlock_stages_label)));
    preg(&raw const m::lbl_803F9EF0, core::mem::size_of_val(&*(&raw const m::lbl_803F9EF0)));
    preg(&raw const m::lbl_803F9F10, core::mem::size_of_val(&*(&raw const m::lbl_803F9F10)));
    preg(&raw const m::un_803F9F28, core::mem::size_of_val(&*(&raw const m::un_803F9F28)));
    preg(&raw const m::un_803F9F38, core::mem::size_of_val(&*(&raw const m::un_803F9F38)));
    preg(&raw const m::un_803F9F44, core::mem::size_of_val(&*(&raw const m::un_803F9F44)));
    preg(&raw const m::un_803F9F50, core::mem::size_of_val(&*(&raw const m::un_803F9F50)));
    preg(&raw const m::un_803F9F5C, core::mem::size_of_val(&*(&raw const m::un_803F9F5C)));
    preg(&raw const m::un_803F9F68, core::mem::size_of_val(&*(&raw const m::un_803F9F68)));
    preg(&raw const m::un_803F9F74, core::mem::size_of_val(&*(&raw const m::un_803F9F74)));
    preg(&raw const m::un_803F9F80, core::mem::size_of_val(&*(&raw const m::un_803F9F80)));
    preg(&raw const m::un_803F9F8C, core::mem::size_of_val(&*(&raw const m::un_803F9F8C)));
    preg(&raw const m::un_803F9F98, core::mem::size_of_val(&*(&raw const m::un_803F9F98)));
    preg(&raw const m::un_803F9FA4, core::mem::size_of_val(&*(&raw const m::un_803F9FA4)));
    preg(&raw const m::un_803FA128, core::mem::size_of_val(&*(&raw const m::un_803FA128)));
    preg(&raw const m::un_803FA258, core::mem::size_of_val(&*(&raw const m::un_803FA258)));
    preg(&raw const m::un_803FA3E8, core::mem::size_of_val(&*(&raw const m::un_803FA3E8)));
    preg(&raw const m::un_803FA3F4, core::mem::size_of_val(&*(&raw const m::un_803FA3F4)));
    preg(&raw const m::un_803FA404, core::mem::size_of_val(&*(&raw const m::un_803FA404)));
    preg(&raw const m::un_803FA414, core::mem::size_of_val(&*(&raw const m::un_803FA414)));
    preg(&raw const m::un_803FA424, core::mem::size_of_val(&*(&raw const m::un_803FA424)));
    preg(&raw const m::un_803FA434, core::mem::size_of_val(&*(&raw const m::un_803FA434)));
    preg(&raw const m::un_803FA440, core::mem::size_of_val(&*(&raw const m::un_803FA440)));
    preg(&raw const m::un_803FA454, core::mem::size_of_val(&*(&raw const m::un_803FA454)));
    preg(&raw const m::un_803FA468, core::mem::size_of_val(&*(&raw const m::un_803FA468)));
    preg(&raw const m::un_803FA474, core::mem::size_of_val(&*(&raw const m::un_803FA474)));
    preg(&raw const m::un_803FA480, core::mem::size_of_val(&*(&raw const m::un_803FA480)));
    preg(&raw const m::un_803FA48C, core::mem::size_of_val(&*(&raw const m::un_803FA48C)));
    preg(&raw const m::un_803FA498, core::mem::size_of_val(&*(&raw const m::un_803FA498)));
    preg(&raw const m::un_803FA4A8, core::mem::size_of_val(&*(&raw const m::un_803FA4A8)));
    preg(&raw const m::un_803FA4B8, core::mem::size_of_val(&*(&raw const m::un_803FA4B8)));
    preg(&raw const m::un_803FA4CC, core::mem::size_of_val(&*(&raw const m::un_803FA4CC)));
    preg(&raw const m::un_803FA4E0, core::mem::size_of_val(&*(&raw const m::un_803FA4E0)));
    preg(&raw const m::un_803FA640, core::mem::size_of_val(&*(&raw const m::un_803FA640)));
    preg(&raw const m::un_803FA658, core::mem::size_of_val(&*(&raw const m::un_803FA658)));
    preg(&raw const m::un_803FA758, core::mem::size_of_val(&*(&raw const m::un_803FA758)));
    preg(&raw const m::un_803FA768, core::mem::size_of_val(&*(&raw const m::un_803FA768)));
    preg(&raw const m::un_803FA77C, core::mem::size_of_val(&*(&raw const m::un_803FA77C)));
    preg(&raw const m::un_803FA790, core::mem::size_of_val(&*(&raw const m::un_803FA790)));
    preg(&raw const m::un_803FA810, core::mem::size_of_val(&*(&raw const m::un_803FA810)));
    preg(&raw const m::un_803FA81C, core::mem::size_of_val(&*(&raw const m::un_803FA81C)));
    preg(&raw const m::un_803FA828, core::mem::size_of_val(&*(&raw const m::un_803FA828)));
    preg(&raw const m::un_803FA834, core::mem::size_of_val(&*(&raw const m::un_803FA834)));
    preg(&raw const m::un_803FA844, core::mem::size_of_val(&*(&raw const m::un_803FA844)));
    preg(&raw const m::un_803FA854, core::mem::size_of_val(&*(&raw const m::un_803FA854)));
    preg(&raw const m::un_803FA860, core::mem::size_of_val(&*(&raw const m::un_803FA860)));
    preg(&raw const m::un_803FA878, core::mem::size_of_val(&*(&raw const m::un_803FA878)));
    preg(&raw const m::un_803FA884, core::mem::size_of_val(&*(&raw const m::un_803FA884)));
    preg(&raw const m::un_803FA894, core::mem::size_of_val(&*(&raw const m::un_803FA894)));
    preg(&raw const m::un_803FA8A4, core::mem::size_of_val(&*(&raw const m::un_803FA8A4)));
    preg(&raw const m::un_803FA8B4, core::mem::size_of_val(&*(&raw const m::un_803FA8B4)));
    preg(&raw const m::un_803FA8C4, core::mem::size_of_val(&*(&raw const m::un_803FA8C4)));
    preg(&raw const m::un_803FA8D8, core::mem::size_of_val(&*(&raw const m::un_803FA8D8)));
    preg(&raw const m::un_803FA8E8, core::mem::size_of_val(&*(&raw const m::un_803FA8E8)));
    preg(&raw const m::un_803FAA08, core::mem::size_of_val(&*(&raw const m::un_803FAA08)));
    preg(&raw const m::un_803FAA14, core::mem::size_of_val(&*(&raw const m::un_803FAA14)));
    preg(&raw const m::un_803FAA20, core::mem::size_of_val(&*(&raw const m::un_803FAA20)));
    preg(&raw const m::un_803FAA2C, core::mem::size_of_val(&*(&raw const m::un_803FAA2C)));
    preg(&raw const m::un_803FAA38, core::mem::size_of_val(&*(&raw const m::un_803FAA38)));
    preg(&raw const m::un_803FAA44, core::mem::size_of_val(&*(&raw const m::un_803FAA44)));
    preg(&raw const m::un_803FAB04, core::mem::size_of_val(&*(&raw const m::un_803FAB04)));
    preg(&raw const m::un_803FAB10, core::mem::size_of_val(&*(&raw const m::un_803FAB10)));
    preg(&raw const m::un_803FAB1C, core::mem::size_of_val(&*(&raw const m::un_803FAB1C)));
    preg(&raw const m::un_803FAB28, core::mem::size_of_val(&*(&raw const m::un_803FAB28)));
    preg(&raw const m::un_803FAB34, core::mem::size_of_val(&*(&raw const m::un_803FAB34)));
    preg(&raw const m::un_803FAB40, core::mem::size_of_val(&*(&raw const m::un_803FAB40)));
    preg(&raw const m::un_803FAB4C, core::mem::size_of_val(&*(&raw const m::un_803FAB4C)));
    preg(&raw const m::un_803FAB58, core::mem::size_of_val(&*(&raw const m::un_803FAB58)));
    preg(&raw const m::un_803FAB64, core::mem::size_of_val(&*(&raw const m::un_803FAB64)));
    preg(&raw const m::un_803FAB70, core::mem::size_of_val(&*(&raw const m::un_803FAB70)));
    preg(&raw const m::un_803FAB7C, core::mem::size_of_val(&*(&raw const m::un_803FAB7C)));
    preg(&raw const m::un_803FAB88, core::mem::size_of_val(&*(&raw const m::un_803FAB88)));
    preg(&raw const m::un_803FAB94, core::mem::size_of_val(&*(&raw const m::un_803FAB94)));
    preg(&raw const m::un_803FABA0, core::mem::size_of_val(&*(&raw const m::un_803FABA0)));
    preg(&raw const m::un_803FABAC, core::mem::size_of_val(&*(&raw const m::un_803FABAC)));
    preg(&raw const m::un_803FABB8, core::mem::size_of_val(&*(&raw const m::un_803FABB8)));
    preg(&raw const m::un_803FABC4, core::mem::size_of_val(&*(&raw const m::un_803FABC4)));
    preg(&raw const m::un_803FABD0, core::mem::size_of_val(&*(&raw const m::un_803FABD0)));
    preg(&raw const m::un_803FABDC, core::mem::size_of_val(&*(&raw const m::un_803FABDC)));
    preg(&raw const m::un_803FABE8, core::mem::size_of_val(&*(&raw const m::un_803FABE8)));
    preg(&raw const m::un_803FABF4, core::mem::size_of_val(&*(&raw const m::un_803FABF4)));
    preg(&raw const m::un_803FAC00, core::mem::size_of_val(&*(&raw const m::un_803FAC00)));
    preg(&raw const m::un_803FAC0C, core::mem::size_of_val(&*(&raw const m::un_803FAC0C)));
    preg(&raw const m::un_803FAC18, core::mem::size_of_val(&*(&raw const m::un_803FAC18)));
    preg(&raw const m::un_803FAC24, core::mem::size_of_val(&*(&raw const m::un_803FAC24)));
    preg(&raw const m::un_803FAC30, core::mem::size_of_val(&*(&raw const m::un_803FAC30)));
    preg(&raw const m::un_803FAC3C, core::mem::size_of_val(&*(&raw const m::un_803FAC3C)));
    preg(&raw const m::un_803FAC48, core::mem::size_of_val(&*(&raw const m::un_803FAC48)));
    preg(&raw const m::un_803FAC54, core::mem::size_of_val(&*(&raw const m::un_803FAC54)));
    preg(&raw const m::un_803FAC60, core::mem::size_of_val(&*(&raw const m::un_803FAC60)));
    preg(&raw const m::un_803FAC6C, core::mem::size_of_val(&*(&raw const m::un_803FAC6C)));
    preg(&raw const m::un_803FAC78, core::mem::size_of_val(&*(&raw const m::un_803FAC78)));
    preg(&raw const m::un_803FAC84, core::mem::size_of_val(&*(&raw const m::un_803FAC84)));
    preg(&raw const m::un_803FAC90, core::mem::size_of_val(&*(&raw const m::un_803FAC90)));
    preg(&raw const m::un_803FAC9C, core::mem::size_of_val(&*(&raw const m::un_803FAC9C)));
    preg(&raw const m::un_803FACA8, core::mem::size_of_val(&*(&raw const m::un_803FACA8)));
    preg(&raw const m::un_803FACB4, core::mem::size_of_val(&*(&raw const m::un_803FACB4)));
    preg(&raw const m::un_803FACC0, core::mem::size_of_val(&*(&raw const m::un_803FACC0)));
    preg(&raw const m::un_803FACCC, core::mem::size_of_val(&*(&raw const m::un_803FACCC)));
    preg(&raw const m::un_803FACD8, core::mem::size_of_val(&*(&raw const m::un_803FACD8)));
    preg(&raw const m::un_803FACE4, core::mem::size_of_val(&*(&raw const m::un_803FACE4)));
    preg(&raw const m::un_803FACF0, core::mem::size_of_val(&*(&raw const m::un_803FACF0)));
    preg(&raw const m::un_803FACFC, core::mem::size_of_val(&*(&raw const m::un_803FACFC)));
    preg(&raw const m::un_803FAD08, core::mem::size_of_val(&*(&raw const m::un_803FAD08)));
    preg(&raw const m::un_803FAD14, core::mem::size_of_val(&*(&raw const m::un_803FAD14)));
    preg(&raw const m::un_803FAD20, core::mem::size_of_val(&*(&raw const m::un_803FAD20)));
    preg(&raw const m::un_803FAD2C, core::mem::size_of_val(&*(&raw const m::un_803FAD2C)));
    preg(&raw const m::un_803FAD38, core::mem::size_of_val(&*(&raw const m::un_803FAD38)));
    preg(&raw const m::un_803FAD44, core::mem::size_of_val(&*(&raw const m::un_803FAD44)));
    preg(&raw const m::un_803FAD50, core::mem::size_of_val(&*(&raw const m::un_803FAD50)));
    preg(&raw const m::un_803FAD5C, core::mem::size_of_val(&*(&raw const m::un_803FAD5C)));
    preg(&raw const m::un_803FAD68, core::mem::size_of_val(&*(&raw const m::un_803FAD68)));
    preg(&raw const m::un_803FAD74, core::mem::size_of_val(&*(&raw const m::un_803FAD74)));
    preg(&raw const m::un_803FAD80, core::mem::size_of_val(&*(&raw const m::un_803FAD80)));
    preg(&raw const m::un_803FAD8C, core::mem::size_of_val(&*(&raw const m::un_803FAD8C)));
    preg(&raw const m::un_803FAD98, core::mem::size_of_val(&*(&raw const m::un_803FAD98)));
    preg(&raw const m::un_803FADA4, core::mem::size_of_val(&*(&raw const m::un_803FADA4)));
    preg(&raw const m::un_803FADB0, core::mem::size_of_val(&*(&raw const m::un_803FADB0)));
    preg(&raw const m::un_803FADBC, core::mem::size_of_val(&*(&raw const m::un_803FADBC)));
    preg(&raw const m::un_803FADC8, core::mem::size_of_val(&*(&raw const m::un_803FADC8)));
    preg(&raw const m::un_803FADD4, core::mem::size_of_val(&*(&raw const m::un_803FADD4)));
    preg(&raw const m::un_803FADE0, core::mem::size_of_val(&*(&raw const m::un_803FADE0)));
    preg(&raw const m::un_803FADEC, core::mem::size_of_val(&*(&raw const m::un_803FADEC)));
    preg(&raw const m::un_803FADF8, core::mem::size_of_val(&*(&raw const m::un_803FADF8)));
    preg(&raw const m::un_803FAE04, core::mem::size_of_val(&*(&raw const m::un_803FAE04)));
    preg(&raw const m::un_803FAE10, core::mem::size_of_val(&*(&raw const m::un_803FAE10)));
    preg(&raw const m::un_803FAE1C, core::mem::size_of_val(&*(&raw const m::un_803FAE1C)));
    preg(&raw const m::un_803FAE28, core::mem::size_of_val(&*(&raw const m::un_803FAE28)));
    preg(&raw const m::un_803FAE34, core::mem::size_of_val(&*(&raw const m::un_803FAE34)));
    preg(&raw const m::un_803FAE40, core::mem::size_of_val(&*(&raw const m::un_803FAE40)));
    preg(&raw const m::un_803FAE4C, core::mem::size_of_val(&*(&raw const m::un_803FAE4C)));
    preg(&raw const m::un_803FAE58, core::mem::size_of_val(&*(&raw const m::un_803FAE58)));
    preg(&raw const m::un_803FAE64, core::mem::size_of_val(&*(&raw const m::un_803FAE64)));
    preg(&raw const m::un_803FAE70, core::mem::size_of_val(&*(&raw const m::un_803FAE70)));
    preg(&raw const m::un_803FAE7C, core::mem::size_of_val(&*(&raw const m::un_803FAE7C)));
    preg(&raw const m::un_803FAE88, core::mem::size_of_val(&*(&raw const m::un_803FAE88)));
    preg(&raw const m::un_803FAE94, core::mem::size_of_val(&*(&raw const m::un_803FAE94)));
    preg(&raw const m::un_803FAEA0, core::mem::size_of_val(&*(&raw const m::un_803FAEA0)));
    preg(&raw const m::un_803FAEAC, core::mem::size_of_val(&*(&raw const m::un_803FAEAC)));
    preg(&raw const m::un_803FAEB8, core::mem::size_of_val(&*(&raw const m::un_803FAEB8)));
    preg(&raw const m::un_803FAEC4, core::mem::size_of_val(&*(&raw const m::un_803FAEC4)));
    preg(&raw const m::un_803FAED0, core::mem::size_of_val(&*(&raw const m::un_803FAED0)));
    preg(&raw const m::un_803FAEDC, core::mem::size_of_val(&*(&raw const m::un_803FAEDC)));
    preg(&raw const m::un_803FAEE8, core::mem::size_of_val(&*(&raw const m::un_803FAEE8)));
    preg(&raw const m::un_803FAEF4, core::mem::size_of_val(&*(&raw const m::un_803FAEF4)));
    preg(&raw const m::un_803FAF00, core::mem::size_of_val(&*(&raw const m::un_803FAF00)));
    preg(&raw const m::un_803FAF0C, core::mem::size_of_val(&*(&raw const m::un_803FAF0C)));
    preg(&raw const m::un_803FB064, core::mem::size_of_val(&*(&raw const m::un_803FB064)));
    preg(&raw const m::un_803FB074, core::mem::size_of_val(&*(&raw const m::un_803FB074)));
    preg(&raw const m::un_803FB080, core::mem::size_of_val(&*(&raw const m::un_803FB080)));
    preg(&raw const m::un_803FB090, core::mem::size_of_val(&*(&raw const m::un_803FB090)));
    preg(&raw const m::un_803FB0A0, core::mem::size_of_val(&*(&raw const m::un_803FB0A0)));
    preg(&raw const m::un_803FB0B0, core::mem::size_of_val(&*(&raw const m::un_803FB0B0)));
    preg(&raw const m::un_803FB0C0, core::mem::size_of_val(&*(&raw const m::un_803FB0C0)));
    preg(&raw const m::un_803FB0D0, core::mem::size_of_val(&*(&raw const m::un_803FB0D0)));
    preg(&raw const m::un_803FB0E0, core::mem::size_of_val(&*(&raw const m::un_803FB0E0)));
    preg(&raw const m::un_803FB0F0, core::mem::size_of_val(&*(&raw const m::un_803FB0F0)));
    preg(&raw const m::un_803FB100, core::mem::size_of_val(&*(&raw const m::un_803FB100)));
    preg(&raw const m::un_803FB110, core::mem::size_of_val(&*(&raw const m::un_803FB110)));
    preg(&raw const m::un_803FB120, core::mem::size_of_val(&*(&raw const m::un_803FB120)));
    preg(&raw const m::un_803FB130, core::mem::size_of_val(&*(&raw const m::un_803FB130)));
    preg(&raw const m::un_803FB140, core::mem::size_of_val(&*(&raw const m::un_803FB140)));
    preg(&raw const m::un_803FB14C, core::mem::size_of_val(&*(&raw const m::un_803FB14C)));
    preg(&raw const m::un_803FB158, core::mem::size_of_val(&*(&raw const m::un_803FB158)));
    preg(&raw const m::un_803FB168, core::mem::size_of_val(&*(&raw const m::un_803FB168)));
    preg(&raw const m::un_803FB368, core::mem::size_of_val(&*(&raw const m::un_803FB368)));
    preg(&raw const m::un_803FB378, core::mem::size_of_val(&*(&raw const m::un_803FB378)));
    preg(&raw const m::un_803FB388, core::mem::size_of_val(&*(&raw const m::un_803FB388)));
    preg(&raw const m::un_803FB394, core::mem::size_of_val(&*(&raw const m::un_803FB394)));
    preg(&raw const m::un_803FB3A4, core::mem::size_of_val(&*(&raw const m::un_803FB3A4)));
    preg(&raw const m::un_803FB3B0, core::mem::size_of_val(&*(&raw const m::un_803FB3B0)));
    preg(&raw const m::un_803FB3BC, core::mem::size_of_val(&*(&raw const m::un_803FB3BC)));
    preg(&raw const m::un_803FB3C8, core::mem::size_of_val(&*(&raw const m::un_803FB3C8)));
    preg(&raw const m::un_803FB3D4, core::mem::size_of_val(&*(&raw const m::un_803FB3D4)));
    preg(&raw const m::un_803FB3E0, core::mem::size_of_val(&*(&raw const m::un_803FB3E0)));
    preg(&raw const m::un_803FB3EC, core::mem::size_of_val(&*(&raw const m::un_803FB3EC)));
    preg(&raw const m::un_803FB3FC, core::mem::size_of_val(&*(&raw const m::un_803FB3FC)));
    preg(&raw const m::un_803FB408, core::mem::size_of_val(&*(&raw const m::un_803FB408)));
    preg(&raw const m::un_803FB414, core::mem::size_of_val(&*(&raw const m::un_803FB414)));
    preg(&raw const m::un_803FB424, core::mem::size_of_val(&*(&raw const m::un_803FB424)));
    preg(&raw const m::un_803FB434, core::mem::size_of_val(&*(&raw const m::un_803FB434)));
    preg(&raw const m::un_803FB440, core::mem::size_of_val(&*(&raw const m::un_803FB440)));
    preg(&raw const m::un_803FB44C, core::mem::size_of_val(&*(&raw const m::un_803FB44C)));
    preg(&raw const m::un_803FB458, core::mem::size_of_val(&*(&raw const m::un_803FB458)));
    preg(&raw const m::un_803FB468, core::mem::size_of_val(&*(&raw const m::un_803FB468)));
    preg(&raw const m::un_803FB478, core::mem::size_of_val(&*(&raw const m::un_803FB478)));
    preg(&raw const m::un_803FB484, core::mem::size_of_val(&*(&raw const m::un_803FB484)));
    preg(&raw const m::un_803FB490, core::mem::size_of_val(&*(&raw const m::un_803FB490)));
    preg(&raw const m::un_803FB4A0, core::mem::size_of_val(&*(&raw const m::un_803FB4A0)));
    preg(&raw const m::un_803FB4B0, core::mem::size_of_val(&*(&raw const m::un_803FB4B0)));
    preg(&raw const m::un_803FB4BC, core::mem::size_of_val(&*(&raw const m::un_803FB4BC)));
    preg(&raw const m::un_803FB4C8, core::mem::size_of_val(&*(&raw const m::un_803FB4C8)));
    preg(&raw const m::un_803FB4D8, core::mem::size_of_val(&*(&raw const m::un_803FB4D8)));
    preg(&raw const m::un_803FB4E4, core::mem::size_of_val(&*(&raw const m::un_803FB4E4)));
    preg(&raw const m::un_803FB4F0, core::mem::size_of_val(&*(&raw const m::un_803FB4F0)));
    preg(&raw const m::un_803FB500, core::mem::size_of_val(&*(&raw const m::un_803FB500)));
    preg(&raw const m::un_803FB510, core::mem::size_of_val(&*(&raw const m::un_803FB510)));
    preg(&raw const m::un_803FB520, core::mem::size_of_val(&*(&raw const m::un_803FB520)));
    preg(&raw const m::un_803FB52C, core::mem::size_of_val(&*(&raw const m::un_803FB52C)));
    preg(&raw const m::un_803FB538, core::mem::size_of_val(&*(&raw const m::un_803FB538)));
    preg(&raw const m::un_803FB5C0, core::mem::size_of_val(&*(&raw const m::un_803FB5C0)));
    preg(&raw const m::un_803FB5D0, core::mem::size_of_val(&*(&raw const m::un_803FB5D0)));
    preg(&raw const m::un_803FB5DC, core::mem::size_of_val(&*(&raw const m::un_803FB5DC)));
    preg(&raw const m::un_803FB5E8, core::mem::size_of_val(&*(&raw const m::un_803FB5E8)));
    preg(&raw const m::un_803FB5F4, core::mem::size_of_val(&*(&raw const m::un_803FB5F4)));
    preg(&raw const m::un_803FB600, core::mem::size_of_val(&*(&raw const m::un_803FB600)));
    preg(&raw const m::un_803FB60C, core::mem::size_of_val(&*(&raw const m::un_803FB60C)));
    preg(&raw const m::un_803FB6EC, core::mem::size_of_val(&*(&raw const m::un_803FB6EC)));
    preg(&raw const m::un_803FB6F8, core::mem::size_of_val(&*(&raw const m::un_803FB6F8)));
    preg(&raw const m::un_803FB704, core::mem::size_of_val(&*(&raw const m::un_803FB704)));
    preg(&raw const m::un_803FB710, core::mem::size_of_val(&*(&raw const m::un_803FB710)));
    preg(&raw const m::un_803FB71C, core::mem::size_of_val(&*(&raw const m::un_803FB71C)));
    preg(&raw const m::un_803FB728, core::mem::size_of_val(&*(&raw const m::un_803FB728)));
    preg(&raw const m::un_803FB7E8, core::mem::size_of_val(&*(&raw const m::un_803FB7E8)));
    preg(&raw const m::un_803FB7F4, core::mem::size_of_val(&*(&raw const m::un_803FB7F4)));
    preg(&raw const m::un_803FB800, core::mem::size_of_val(&*(&raw const m::un_803FB800)));
    preg(&raw const m::un_803FB80C, core::mem::size_of_val(&*(&raw const m::un_803FB80C)));
    preg(&raw const m::un_803FB818, core::mem::size_of_val(&*(&raw const m::un_803FB818)));
    preg(&raw const m::un_803FB828, core::mem::size_of_val(&*(&raw const m::un_803FB828)));
    preg(&raw const m::un_803FB834, core::mem::size_of_val(&*(&raw const m::un_803FB834)));
    preg(&raw const m::un_803FB840, core::mem::size_of_val(&*(&raw const m::un_803FB840)));
    preg(&raw const m::un_803FB84C, core::mem::size_of_val(&*(&raw const m::un_803FB84C)));
    preg(&raw const m::un_803FB858, core::mem::size_of_val(&*(&raw const m::un_803FB858)));
    preg(&raw const m::un_803FB864, core::mem::size_of_val(&*(&raw const m::un_803FB864)));
    preg(&raw const m::un_803FB870, core::mem::size_of_val(&*(&raw const m::un_803FB870)));
    preg(&raw const m::un_803FB950, core::mem::size_of_val(&*(&raw const m::un_803FB950)));
    preg(&raw const m::un_803FB95C, core::mem::size_of_val(&*(&raw const m::un_803FB95C)));
    preg(&raw const m::un_803FB968, core::mem::size_of_val(&*(&raw const m::un_803FB968)));
    preg(&raw const m::un_803FB974, core::mem::size_of_val(&*(&raw const m::un_803FB974)));
    preg(&raw const m::un_803FB980, core::mem::size_of_val(&*(&raw const m::un_803FB980)));
    preg(&raw const m::un_803FB98C, core::mem::size_of_val(&*(&raw const m::un_803FB98C)));
    preg(&raw const m::un_803FBA4C, core::mem::size_of_val(&*(&raw const m::un_803FBA4C)));
    preg(&raw const m::un_803FBA5C, core::mem::size_of_val(&*(&raw const m::un_803FBA5C)));
    preg(&raw const m::un_803FBA6C, core::mem::size_of_val(&*(&raw const m::un_803FBA6C)));
    preg(&raw const m::un_803FBA7C, core::mem::size_of_val(&*(&raw const m::un_803FBA7C)));
    preg(&raw const m::un_803FBA8C, core::mem::size_of_val(&*(&raw const m::un_803FBA8C)));
    preg(&raw const m::un_803FBA9C, core::mem::size_of_val(&*(&raw const m::un_803FBA9C)));
    preg(&raw const m::un_803FBB5C, core::mem::size_of_val(&*(&raw const m::un_803FBB5C)));
    preg(&raw const m::un_803FBB68, core::mem::size_of_val(&*(&raw const m::un_803FBB68)));
    preg(&raw const m::un_803FBB74, core::mem::size_of_val(&*(&raw const m::un_803FBB74)));
    preg(&raw const m::un_803FBB80, core::mem::size_of_val(&*(&raw const m::un_803FBB80)));
    preg(&raw const m::un_803FBB8C, core::mem::size_of_val(&*(&raw const m::un_803FBB8C)));
    preg(&raw const m::un_803FBB98, core::mem::size_of_val(&*(&raw const m::un_803FBB98)));
    preg(&raw const m::un_803FBC58, core::mem::size_of_val(&*(&raw const m::un_803FBC58)));
    preg(&raw const m::un_803FBC6C, core::mem::size_of_val(&*(&raw const m::un_803FBC6C)));
    preg(&raw const m::un_803FBC7C, core::mem::size_of_val(&*(&raw const m::un_803FBC7C)));
    preg(&raw const m::un_803FBC8C, core::mem::size_of_val(&*(&raw const m::un_803FBC8C)));
    preg(&raw const m::un_803FBC9C, core::mem::size_of_val(&*(&raw const m::un_803FBC9C)));
    preg(&raw const m::un_803FBCAC, core::mem::size_of_val(&*(&raw const m::un_803FBCAC)));
    preg(&raw const m::un_803FBD6C, core::mem::size_of_val(&*(&raw const m::un_803FBD6C)));
    preg(&raw const m::un_803FBD80, core::mem::size_of_val(&*(&raw const m::un_803FBD80)));
    preg(&raw const m::un_803FBD90, core::mem::size_of_val(&*(&raw const m::un_803FBD90)));
    preg(&raw const m::un_803FBDA0, core::mem::size_of_val(&*(&raw const m::un_803FBDA0)));
    preg(&raw const m::un_803FBDB0, core::mem::size_of_val(&*(&raw const m::un_803FBDB0)));
    preg(&raw const m::un_803FBDC0, core::mem::size_of_val(&*(&raw const m::un_803FBDC0)));
    preg(&raw const m::un_803FBE80, core::mem::size_of_val(&*(&raw const m::un_803FBE80)));
    preg(&raw const m::un_803FBE8C, core::mem::size_of_val(&*(&raw const m::un_803FBE8C)));
    preg(&raw const m::un_803FBE98, core::mem::size_of_val(&*(&raw const m::un_803FBE98)));
    preg(&raw const m::un_803FBEA4, core::mem::size_of_val(&*(&raw const m::un_803FBEA4)));
    preg(&raw const m::un_803FBEB0, core::mem::size_of_val(&*(&raw const m::un_803FBEB0)));
    preg(&raw const m::un_803FBEBC, core::mem::size_of_val(&*(&raw const m::un_803FBEBC)));
    preg(&raw const m::un_803FBEC8, core::mem::size_of_val(&*(&raw const m::un_803FBEC8)));
    preg(&raw const m::un_803FBED4, core::mem::size_of_val(&*(&raw const m::un_803FBED4)));
    preg(&raw const m::un_803FBEE4, core::mem::size_of_val(&*(&raw const m::un_803FBEE4)));
    preg(&raw const m::un_803FBEF4, core::mem::size_of_val(&*(&raw const m::un_803FBEF4)));
    preg(&raw const m::un_803FBF00, core::mem::size_of_val(&*(&raw const m::un_803FBF00)));
    preg(&raw const m::un_803FBF10, core::mem::size_of_val(&*(&raw const m::un_803FBF10)));
    preg(&raw const m::un_803FBF20, core::mem::size_of_val(&*(&raw const m::un_803FBF20)));
    preg(&raw const m::un_803FBF30, core::mem::size_of_val(&*(&raw const m::un_803FBF30)));
    preg(&raw const m::un_803FBF40, core::mem::size_of_val(&*(&raw const m::un_803FBF40)));
    preg(&raw const m::un_803FBF50, core::mem::size_of_val(&*(&raw const m::un_803FBF50)));
    preg(&raw const m::un_803FBF5C, core::mem::size_of_val(&*(&raw const m::un_803FBF5C)));
    preg(&raw const m::un_803FBF68, core::mem::size_of_val(&*(&raw const m::un_803FBF68)));
    preg(&raw const m::un_803FBF74, core::mem::size_of_val(&*(&raw const m::un_803FBF74)));
    preg(&raw const m::un_803FBFBC, core::mem::size_of_val(&*(&raw const m::un_803FBFBC)));
    preg(&raw const m::un_803FBFCC, core::mem::size_of_val(&*(&raw const m::un_803FBFCC)));
    preg(&raw const m::un_803FBFD8, core::mem::size_of_val(&*(&raw const m::un_803FBFD8)));
    preg(&raw const m::un_803FBFE4, core::mem::size_of_val(&*(&raw const m::un_803FBFE4)));
    preg(&raw const m::un_803FBFF0, core::mem::size_of_val(&*(&raw const m::un_803FBFF0)));
    preg(&raw const m::un_803FBFFC, core::mem::size_of_val(&*(&raw const m::un_803FBFFC)));
    preg(&raw const m::un_803FC0BC, core::mem::size_of_val(&*(&raw const m::un_803FC0BC)));
    preg(&raw const m::un_803FC0CC, core::mem::size_of_val(&*(&raw const m::un_803FC0CC)));
    preg(&raw const m::un_803FC0D8, core::mem::size_of_val(&*(&raw const m::un_803FC0D8)));
    preg(&raw const m::un_803FC0E4, core::mem::size_of_val(&*(&raw const m::un_803FC0E4)));
    preg(&raw const m::un_803FC0F0, core::mem::size_of_val(&*(&raw const m::un_803FC0F0)));
    preg(&raw const m::un_803FC0FC, core::mem::size_of_val(&*(&raw const m::un_803FC0FC)));
    preg(&raw const m::un_803FC1BC, core::mem::size_of_val(&*(&raw const m::un_803FC1BC)));
    preg(&raw const m::un_803FC1C8, core::mem::size_of_val(&*(&raw const m::un_803FC1C8)));
    preg(&raw const m::un_803FC1D4, core::mem::size_of_val(&*(&raw const m::un_803FC1D4)));
    preg(&raw const m::un_803FC1E0, core::mem::size_of_val(&*(&raw const m::un_803FC1E0)));
    preg(&raw const m::un_803FC1EC, core::mem::size_of_val(&*(&raw const m::un_803FC1EC)));
    preg(&raw const m::un_803FC1FC, core::mem::size_of_val(&*(&raw const m::un_803FC1FC)));
    preg(&raw const m::un_803FC208, core::mem::size_of_val(&*(&raw const m::un_803FC208)));
    preg(&raw const m::un_803FC214, core::mem::size_of_val(&*(&raw const m::un_803FC214)));
    preg(&raw const m::un_803FC220, core::mem::size_of_val(&*(&raw const m::un_803FC220)));
    preg(&raw const m::un_803FC22C, core::mem::size_of_val(&*(&raw const m::un_803FC22C)));
    preg(&raw const m::un_803FC2EC, core::mem::size_of_val(&*(&raw const m::un_803FC2EC)));
    preg(&raw const m::un_803FC420, core::mem::size_of_val(&*(&raw const m::un_803FC420)));
    preg(&raw const m::un_803FC42C, core::mem::size_of_val(&*(&raw const m::un_803FC42C)));
    preg(&raw const m::un_803FC440, core::mem::size_of_val(&*(&raw const m::un_803FC440)));
    preg(&raw const m::un_803FC454, core::mem::size_of_val(&*(&raw const m::un_803FC454)));
    preg(&raw const m::un_803FC468, core::mem::size_of_val(&*(&raw const m::un_803FC468)));
    preg(&raw const m::un_803FC47C, core::mem::size_of_val(&*(&raw const m::un_803FC47C)));
    preg(&raw const m::un_803FC490, core::mem::size_of_val(&*(&raw const m::un_803FC490)));
    preg(&raw const m::un_803FC4A4, core::mem::size_of_val(&*(&raw const m::un_803FC4A4)));
    preg(&raw const m::un_803FC4B8, core::mem::size_of_val(&*(&raw const m::un_803FC4B8)));
    preg(&raw const m::un_803FC4CC, core::mem::size_of_val(&*(&raw const m::un_803FC4CC)));
    preg(&raw const m::un_803FC60C, core::mem::size_of_val(&*(&raw const m::un_803FC60C)));
    preg(&raw const m::un_803FC61C, core::mem::size_of_val(&*(&raw const m::un_803FC61C)));
    preg(&raw const m::un_803FC62C, core::mem::size_of_val(&*(&raw const m::un_803FC62C)));
    preg(&raw const m::un_803FC63C, core::mem::size_of_val(&*(&raw const m::un_803FC63C)));
    preg(&raw const m::un_803FC6BC, core::mem::size_of_val(&*(&raw const m::un_803FC6BC)));
    preg(&raw const m::un_803FC6D0, core::mem::size_of_val(&*(&raw const m::un_803FC6D0)));
    preg(&raw const m::un_803FC6DC, core::mem::size_of_val(&*(&raw const m::un_803FC6DC)));
    preg(&raw const m::un_803FC6E8, core::mem::size_of_val(&*(&raw const m::un_803FC6E8)));
    preg(&raw const m::un_803FC6F4, core::mem::size_of_val(&*(&raw const m::un_803FC6F4)));
    preg(&raw const m::un_803FC700, core::mem::size_of_val(&*(&raw const m::un_803FC700)));
    preg(&raw const m::un_803FC70C, core::mem::size_of_val(&*(&raw const m::un_803FC70C)));
    preg(&raw const m::un_803FC7EC, core::mem::size_of_val(&*(&raw const m::un_803FC7EC)));
    preg(&raw const m::un_803FC804, core::mem::size_of_val(&*(&raw const m::un_803FC804)));
    preg(&raw const m::un_803FC810, core::mem::size_of_val(&*(&raw const m::un_803FC810)));
    preg(&raw const m::un_803FC81C, core::mem::size_of_val(&*(&raw const m::un_803FC81C)));
    preg(&raw const m::un_803FC828, core::mem::size_of_val(&*(&raw const m::un_803FC828)));
    preg(&raw const m::un_803FC8C8, core::mem::size_of_val(&*(&raw const m::un_803FC8C8)));
    preg(&raw const m::un_803FC968, core::mem::size_of_val(&*(&raw const m::un_803FC968)));
    preg(&raw const m::un_803FC97C, core::mem::size_of_val(&*(&raw const m::un_803FC97C)));
    preg(&raw const m::un_803FC98C, core::mem::size_of_val(&*(&raw const m::un_803FC98C)));
    preg(&raw const m::un_803FC998, core::mem::size_of_val(&*(&raw const m::un_803FC998)));
    preg(&raw const m::un_803FC9A8, core::mem::size_of_val(&*(&raw const m::un_803FC9A8)));
    preg(&raw const m::un_803FC9BC, core::mem::size_of_val(&*(&raw const m::un_803FC9BC)));
    preg(&raw const m::un_803FC9D0, core::mem::size_of_val(&*(&raw const m::un_803FC9D0)));
    preg(&raw const m::un_803FC9E8, core::mem::size_of_val(&*(&raw const m::un_803FC9E8)));
    preg(&raw const m::un_803FC9FC, core::mem::size_of_val(&*(&raw const m::un_803FC9FC)));
    preg(&raw const m::un_803FCA10, core::mem::size_of_val(&*(&raw const m::un_803FCA10)));
    preg(&raw const m::un_803FCA20, core::mem::size_of_val(&*(&raw const m::un_803FCA20)));
    preg(&raw const m::un_803FCA30, core::mem::size_of_val(&*(&raw const m::un_803FCA30)));
    preg(&raw const m::un_803FCA40, core::mem::size_of_val(&*(&raw const m::un_803FCA40)));
    preg(&raw const m::un_803FCBE0, core::mem::size_of_val(&*(&raw const m::un_803FCBE0)));
    preg(&raw const m::un_803FCBF4, core::mem::size_of_val(&*(&raw const m::un_803FCBF4)));
    preg(&raw const m::un_803FCC08, core::mem::size_of_val(&*(&raw const m::un_803FCC08)));
    preg(&raw const m::un_803FCC18, core::mem::size_of_val(&*(&raw const m::un_803FCC18)));
    preg(&raw const m::un_803FCC28, core::mem::size_of_val(&*(&raw const m::un_803FCC28)));
    preg(&raw const m::un_803FCC38, core::mem::size_of_val(&*(&raw const m::un_803FCC38)));
    preg(&raw const m::un_803FCCF8, core::mem::size_of_val(&*(&raw const m::un_803FCCF8)));
    preg(&raw const m::un_803FCD04, core::mem::size_of_val(&*(&raw const m::un_803FCD04)));
    preg(&raw const m::un_803FCD10, core::mem::size_of_val(&*(&raw const m::un_803FCD10)));
    preg(&raw const m::un_803FCD1C, core::mem::size_of_val(&*(&raw const m::un_803FCD1C)));
    preg(&raw const m::un_803FCD34, core::mem::size_of_val(&*(&raw const m::un_803FCD34)));
    preg(&raw const m::un_803FCDD4, core::mem::size_of_val(&*(&raw const m::un_803FCDD4)));
    preg(&raw const m::un_803FCDE8, core::mem::size_of_val(&*(&raw const m::un_803FCDE8)));
    preg(&raw const m::un_803FCDFC, core::mem::size_of_val(&*(&raw const m::un_803FCDFC)));
    preg(&raw const m::un_803FCE0C, core::mem::size_of_val(&*(&raw const m::un_803FCE0C)));
    preg(&raw const m::un_803FCE1C, core::mem::size_of_val(&*(&raw const m::un_803FCE1C)));
    preg(&raw const m::un_803FCE34, core::mem::size_of_val(&*(&raw const m::un_803FCE34)));
    preg(&raw const m::un_803FCE4C, core::mem::size_of_val(&*(&raw const m::un_803FCE4C)));
    preg(&raw const m::un_803FCF2C, core::mem::size_of_val(&*(&raw const m::un_803FCF2C)));
    preg(&raw const m::un_803FCF38, core::mem::size_of_val(&*(&raw const m::un_803FCF38)));
    preg(&raw const m::un_803FCF44, core::mem::size_of_val(&*(&raw const m::un_803FCF44)));
    preg(&raw const m::un_803FCF50, core::mem::size_of_val(&*(&raw const m::un_803FCF50)));
    preg(&raw const m::un_803FCF5C, core::mem::size_of_val(&*(&raw const m::un_803FCF5C)));
    preg(&raw const m::un_803FCF90, core::mem::size_of_val(&*(&raw const m::un_803FCF90)));
    preg(&raw const m::un_803FCFA0, core::mem::size_of_val(&*(&raw const m::un_803FCFA0)));
    preg(&raw const m::un_803FCFAC, core::mem::size_of_val(&*(&raw const m::un_803FCFAC)));
    preg(&raw const m::un_803FCFC0, core::mem::size_of_val(&*(&raw const m::un_803FCFC0)));
    preg(&raw const m::un_803FCFD4, core::mem::size_of_val(&*(&raw const m::un_803FCFD4)));
    preg(&raw const m::un_803FCFE0, core::mem::size_of_val(&*(&raw const m::un_803FCFE0)));
    preg(&raw const m::un_803FCFEC, core::mem::size_of_val(&*(&raw const m::un_803FCFEC)));
    preg(&raw const m::un_803FD000, core::mem::size_of_val(&*(&raw const m::un_803FD000)));
    preg(&raw const m::un_803FD00C, core::mem::size_of_val(&*(&raw const m::un_803FD00C)));
    preg(&raw const m::un_803FD01C, core::mem::size_of_val(&*(&raw const m::un_803FD01C)));
    preg(&raw const m::un_803FD034, core::mem::size_of_val(&*(&raw const m::un_803FD034)));
    preg(&raw const m::un_803FD044, core::mem::size_of_val(&*(&raw const m::un_803FD044)));
    preg(&raw const m::un_803FD058, core::mem::size_of_val(&*(&raw const m::un_803FD058)));
    preg(&raw const m::un_803FD064, core::mem::size_of_val(&*(&raw const m::un_803FD064)));
    preg(&raw const m::un_803FD224, core::mem::size_of_val(&*(&raw const m::un_803FD224)));
    preg(&raw const m::un_803FD230, core::mem::size_of_val(&*(&raw const m::un_803FD230)));
    preg(&raw const m::un_803FD23C, core::mem::size_of_val(&*(&raw const m::un_803FD23C)));
    preg(&raw const m::un_803FD24C, core::mem::size_of_val(&*(&raw const m::un_803FD24C)));
    preg(&raw const m::un_803FD258, core::mem::size_of_val(&*(&raw const m::un_803FD258)));
    preg(&raw const m::un_803FD264, core::mem::size_of_val(&*(&raw const m::un_803FD264)));
    preg(&raw const m::un_803FD274, core::mem::size_of_val(&*(&raw const m::un_803FD274)));
    preg(&raw const m::un_803FD28C, core::mem::size_of_val(&*(&raw const m::un_803FD28C)));
    preg(&raw const m::un_803FD29C, core::mem::size_of_val(&*(&raw const m::un_803FD29C)));
    preg(&raw const m::un_803FD2AC, core::mem::size_of_val(&*(&raw const m::un_803FD2AC)));
    preg(&raw const m::un_803FD2BC, core::mem::size_of_val(&*(&raw const m::un_803FD2BC)));
    preg(&raw const m::un_803FD2D0, core::mem::size_of_val(&*(&raw const m::un_803FD2D0)));
    preg(&raw const m::un_803FD2DC, core::mem::size_of_val(&*(&raw const m::un_803FD2DC)));
    preg(&raw const m::un_803FD2E8, core::mem::size_of_val(&*(&raw const m::un_803FD2E8)));
    preg(&raw const m::un_803FD2F4, core::mem::size_of_val(&*(&raw const m::un_803FD2F4)));
    preg(&raw const m::un_803FD304, core::mem::size_of_val(&*(&raw const m::un_803FD304)));
    preg(&raw const m::un_803FD310, core::mem::size_of_val(&*(&raw const m::un_803FD310)));
    preg(&raw const m::un_803FD4B0, core::mem::size_of_val(&*(&raw const m::un_803FD4B0)));
    preg(&raw const m::un_803FD4C4, core::mem::size_of_val(&*(&raw const m::un_803FD4C4)));
    preg(&raw const m::un_803FD4D0, core::mem::size_of_val(&*(&raw const m::un_803FD4D0)));
    preg(&raw const m::un_803FD4DC, core::mem::size_of_val(&*(&raw const m::un_803FD4DC)));
    preg(&raw const m::un_803FD4EC, core::mem::size_of_val(&*(&raw const m::un_803FD4EC)));
    preg(&raw const m::un_803FD500, core::mem::size_of_val(&*(&raw const m::un_803FD500)));
    preg(&raw const m::un_803FD50C, core::mem::size_of_val(&*(&raw const m::un_803FD50C)));
    preg(&raw const m::un_803FD51C, core::mem::size_of_val(&*(&raw const m::un_803FD51C)));
    preg(&raw const m::un_803FD52C, core::mem::size_of_val(&*(&raw const m::un_803FD52C)));
    preg(&raw const m::un_803FD53C, core::mem::size_of_val(&*(&raw const m::un_803FD53C)));
    preg(&raw const m::un_803FD54C, core::mem::size_of_val(&*(&raw const m::un_803FD54C)));
    preg(&raw const m::un_803FD55C, core::mem::size_of_val(&*(&raw const m::un_803FD55C)));
    preg(&raw const m::un_803FD56C, core::mem::size_of_val(&*(&raw const m::un_803FD56C)));
    preg(&raw const m::un_803FD57C, core::mem::size_of_val(&*(&raw const m::un_803FD57C)));
    preg(&raw const m::un_803FD58C, core::mem::size_of_val(&*(&raw const m::un_803FD58C)));
    preg(&raw const m::un_803FD78C, core::mem::size_of_val(&*(&raw const m::un_803FD78C)));
    preg(&raw const m::un_803FD79C, core::mem::size_of_val(&*(&raw const m::un_803FD79C)));
    preg(&raw const m::un_803FD7A8, core::mem::size_of_val(&*(&raw const m::un_803FD7A8)));
    preg(&raw const m::un_803FD7B4, core::mem::size_of_val(&*(&raw const m::un_803FD7B4)));
    preg(&raw const m::un_803FD7C0, core::mem::size_of_val(&*(&raw const m::un_803FD7C0)));
    preg(&raw const m::un_803FD7CC, core::mem::size_of_val(&*(&raw const m::un_803FD7CC)));
    preg(&raw const m::un_803FD7DC, core::mem::size_of_val(&*(&raw const m::un_803FD7DC)));
    preg(&raw const m::un_803FD7E8, core::mem::size_of_val(&*(&raw const m::un_803FD7E8)));
    preg(&raw const m::un_803FD7F4, core::mem::size_of_val(&*(&raw const m::un_803FD7F4)));
    preg(&raw const m::un_803FD800, core::mem::size_of_val(&*(&raw const m::un_803FD800)));
    preg(&raw const m::un_803FD80C, core::mem::size_of_val(&*(&raw const m::un_803FD80C)));
    preg(&raw const m::un_803FD818, core::mem::size_of_val(&*(&raw const m::un_803FD818)));
    preg(&raw const m::un_803FD824, core::mem::size_of_val(&*(&raw const m::un_803FD824)));
    preg(&raw const m::un_803FD830, core::mem::size_of_val(&*(&raw const m::un_803FD830)));
    preg(&raw const m::un_803FD83C, core::mem::size_of_val(&*(&raw const m::un_803FD83C)));
    preg(&raw const m::un_803FD848, core::mem::size_of_val(&*(&raw const m::un_803FD848)));
    preg(&raw const m::un_803FD854, core::mem::size_of_val(&*(&raw const m::un_803FD854)));
    preg(&raw const m::un_803FD864, core::mem::size_of_val(&*(&raw const m::un_803FD864)));
    preg(&raw const m::un_803FDAC4, core::mem::size_of_val(&*(&raw const m::un_803FDAC4)));
    preg(&raw const m::un_803FDAD4, core::mem::size_of_val(&*(&raw const m::un_803FDAD4)));
    preg(&raw const m::un_803FDAE4, core::mem::size_of_val(&*(&raw const m::un_803FDAE4)));
    preg(&raw const m::un_803FDAF8, core::mem::size_of_val(&*(&raw const m::un_803FDAF8)));
    preg(&raw const m::un_803FDB78, core::mem::size_of_val(&*(&raw const m::un_803FDB78)));
    preg(&raw const m::un_803FDB88, core::mem::size_of_val(&*(&raw const m::un_803FDB88)));
    preg(&raw const m::un_803FDB9C, core::mem::size_of_val(&*(&raw const m::un_803FDB9C)));
    preg(&raw const m::ext::DbLevel, core::mem::size_of_val(&*(&raw const m::ext::DbLevel)));
    preg(&raw const m::ext::db_804D4AF8, core::mem::size_of_val(&*(&raw const m::ext::db_804D4AF8)));
    preg(&raw const m::ext::db_804D6B88, core::mem::size_of_val(&*(&raw const m::ext::db_804D6B88)));
    preg(&raw const m::ext::db_build_timestamp, core::mem::size_of_val(&*(&raw const m::ext::db_build_timestamp)));
    preg(&raw const m::ext::gmMainLib_8045A6C0, core::mem::size_of_val(&*(&raw const m::ext::gmMainLib_8045A6C0)));
        preg(&raw const OA, core::mem::size_of_val(&OA));
        preg(&raw const DT, core::mem::size_of_val(&DT));
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const SOB, core::mem::size_of_val(&SOB));
        preg(&raw const MEM, core::mem::size_of_val(&MEM));
        preg(&raw const STR, core::mem::size_of_val(&STR));
        preg(&raw const U32S, core::mem::size_of_val(&U32S));
        preg(&raw const I32S, core::mem::size_of_val(&I32S));
        preg(&raw const GMM, core::mem::size_of_val(&GMM));
        preg(&raw const ALLS, core::mem::size_of_val(&ALLS));
        preg(&raw const SYMBUF, core::mem::size_of_val(&SYMBUF));
        preg(&raw const A5, core::mem::size_of_val(&A5));
        preg(&raw const A6, core::mem::size_of_val(&A6));
        preg(&raw const ARR, core::mem::size_of_val(&ARR));
        preg(&raw const O0, core::mem::size_of_val(&O0));
        preg(&raw const O1, core::mem::size_of_val(&O1));
        preg(&raw const O2, core::mem::size_of_val(&O2));
        preg(&raw const SMD, core::mem::size_of_val(&SMD));
        g::freg_all();
        PRISTINE = Some(m::un_803F9FA4);
        PRISTINE_IMG = Some(m::un_803FD274);
        let mut c = Check::new(include_str!("data/mtu_soundtest.txt"));
        let mut total = 0;
        for op in 0..114u32 {
            for k in 0..KC[op as usize] {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { hash_tables(); }
                    1 => { m::un_802FF7DC(); }
                    2 => { { let p = if rnd() & 1 != 0 { &raw mut STR as *mut i8 } else { null_mut() }; ret = m::un_802FF884(p) as u32; } }
                    3 => { { let cnt = (rnd() % 9) as i32; m::un_802FFCD0(cnt, &raw mut ARR as *mut c_void); } }
                    4 => { { let a = (rnd() % 4) as i32; let t = if rnd() & 1 != 0 { &raw mut ARR as *mut c_void } else { null_mut() }; let w = rnd() % 8; let cb: soundtest_callback = match w { 0 => None, 1 => Some(m::fn_802FFE6C), 2 => Some(m::fn_802FFE0C), 3 => Some(m::un_80301B48), 4 => Some(m::fn_80300CC8), 5 => Some(m::fn_80300DE0), 6 => Some(m::fn_80300ED0), _ => Some(m::fn_803011EC) }; m::un_802FFD94(a, t, cb); } }
                    5 => { m::un_802FFEE0(&raw mut O0); }
                    6 => { m::un_802FFF2C(&raw mut SMD); }
                    7 => { m::un_80301BA8(&raw mut O1); }
                    8 => { m::un_80301C64(&raw mut O2); }
                    9 => { { let a = pick(); ret = m::un_802FF88C(a) as u32; } }
                    10 => { { let a = pick(); m::un_802FF934(a); } }
                    11 => { { let a = pick(); ret = m::un_802FF958(a) as u32; } }
                    12 => { { let a = pick(); ret = m::un_802FF99C(a) as u32; } }
                    13 => { { let a = pick(); ret = m::un_802FF9DC(a) as u32; } }
                    14 => { { let a = pick(); ret = m::un_802FFB58(a) as u32; } }
                    15 => { { let a = pick(); ret = m::un_802FFBAC(a) as u32; } }
                    16 => { { let a = pick(); ret = m::un_802FFC30(a) as u32; } }
                    17 => { { let a = pick(); ret = m::un_802FFC6C(a) as u32; } }
                    18 => { { let a = pick(); ret = m::fn_802FFCC8(a) as u32; } }
                    19 => { { let a = pick(); ret = m::fn_802FFE0C(a) as u32; } }
                    20 => { { let a = pick(); ret = m::fn_802FFE6C(a) as u32; } }
                    21 => { { let a = pick(); ret = m::un_802FFEA4(a) as u32; } }
                    22 => { { let a = pick(); ret = m::un_803001DC(a) as u32; } }
                    23 => { { let a = pick(); ret = m::un_80300218(a) as u32; } }
                    24 => { { let a = pick(); ret = m::un_80300248(a) as u32; } }
                    25 => { { let a = pick(); ret = m::un_80300290(a) as u32; } }
                    26 => { { let a = pick(); ret = m::un_803002FC(a) as u32; } }
                    27 => { { let a = pick(); ret = m::un_80300338(a) as u32; } }
                    28 => { { let a = pick(); ret = m::un_80300378(a) as u32; } }
                    29 => { { let a = pick(); ret = m::un_803003C4(a) as u32; } }
                    30 => { { let a = pick(); ret = m::un_80300410(a) as u32; } }
                    31 => { { let a = pick(); ret = m::un_80300480(a) as u32; } }
                    32 => { { let a = pick(); ret = m::un_803004B4(a) as u32; } }
                    33 => { { let a = pick(); ret = m::un_803004E8(a) as u32; } }
                    34 => { { let a = pick(); ret = m::un_8030051C(a) as u32; } }
                    35 => { { let a = pick(); ret = m::un_80300550(a) as u32; } }
                    36 => { { let a = pick(); ret = m::un_80300584(a) as u32; } }
                    37 => { { let a = pick(); ret = m::un_803005B8(a) as u32; } }
                    38 => { { let a = pick(); ret = m::un_803005EC(a) as u32; } }
                    39 => { { let a = pick(); ret = m::un_80300620(a) as u32; } }
                    40 => { { let a = pick(); ret = m::un_80300654(a) as u32; } }
                    41 => { { let a = pick(); ret = m::un_80300688(a) as u32; } }
                    42 => { { let a = pick(); ret = m::un_803006BC(a) as u32; } }
                    43 => { { let a = pick(); ret = m::un_803006F0(a) as u32; } }
                    44 => { { let a = pick(); ret = m::un_80300724(a) as u32; } }
                    45 => { { let a = pick(); m::un_80300758(a); } }
                    46 => { { let a = pick(); m::un_80300790(a); } }
                    47 => { { let a = pick(); ret = m::un_803007C8(a) as u32; } }
                    48 => { { let a = pick(); ret = m::un_803007FC(a) as u32; } }
                    49 => { { let a = pick(); ret = m::un_80300830(a) as u32; } }
                    50 => { { let a = pick(); ret = m::un_80300864(a) as u32; } }
                    51 => { { let a = pick(); ret = m::un_80300898(a) as u32; } }
                    52 => { { let a = pick(); ret = m::un_803008CC(a) as u32; } }
                    53 => { { let a = pick(); ret = m::un_80300900(a) as u32; } }
                    54 => { { let a = pick(); ret = m::un_80300934(a) as u32; } }
                    55 => { { let a = pick(); ret = m::un_80300968(a) as u32; } }
                    56 => { { let a = pick(); ret = m::un_803009A4(a) as u32; } }
                    57 => { { let a = pick(); ret = m::un_803009E0(a) as u32; } }
                    58 => { { let a = pick(); ret = m::un_80300A88(a) as u32; } }
                    59 => { { let a = pick(); ret = m::un_80300AB8(a) as u32; } }
                    60 => { { let a = pick(); ret = m::un_80300AF4(a) as u32; } }
                    61 => { { let a = pick(); ret = m::un_80300B58(a) as u32; } }
                    62 => { { let a = pick(); ret = m::un_80300BBC(a) as u32; } }
                    63 => { { let a = pick(); ret = m::un_80300BF8(a) as u32; } }
                    64 => { { let a = pick(); ret = m::un_80300C3C(a) as u32; } }
                    65 => { { let a = pick(); ret = m::un_80300C80(a) as u32; } }
                    66 => { { let a = pick(); m::fn_80300CC8(a); } }
                    67 => { { let a = pick(); ret = m::un_80300D78(a) as u32; } }
                    68 => { { let a = pick(); m::fn_80300DE0(a); } }
                    69 => { { let a = pick(); ret = m::un_80300E74(a) as u32; } }
                    70 => { { let a = pick(); m::fn_80300ED0(a); } }
                    71 => { { let a = pick(); ret = m::un_80300F3C(a) as u32; } }
                    72 => { { let a = pick(); ret = m::un_80300F98(a) as u32; } }
                    73 => { { let a = pick(); ret = m::un_80300FEC(a) as u32; } }
                    74 => { { let a = pick(); ret = m::un_80301028(a) as u32; } }
                    75 => { { let a = pick(); ret = m::un_80301074(a) as u32; } }
                    76 => { { let a = pick(); ret = m::un_803010C0(a) as u32; } }
                    77 => { { let a = pick(); ret = m::fn_8030110C(a) as u32; } }
                    78 => { { let a = pick(); m::fn_803011EC(a); } }
                    79 => { { let a = pick(); ret = m::un_80301280(a) as u32; } }
                    80 => { { let a = pick(); ret = m::un_803012D4(a) as u32; } }
                    81 => { { let a = pick(); ret = m::un_80301328(a) as u32; } }
                    82 => { { let a = pick(); ret = m::un_8030136C(a) as u32; } }
                    83 => { { let a = pick(); ret = m::un_803013B0(a) as u32; } }
                    84 => { { let a = pick(); ret = m::un_803013EC(a) as u32; } }
                    85 => { { let a = pick(); ret = m::un_80301420(a) as u32; } }
                    86 => { { let a = pick(); ret = m::un_80301454(a) as u32; } }
                    87 => { { let a = pick(); ret = m::un_80301490(a) as u32; } }
                    88 => { { let a = pick(); ret = m::un_803014CC(a) as u32; } }
                    89 => { { let a = pick(); ret = m::un_80301508(a) as u32; } }
                    90 => { { let a = pick(); ret = m::un_80301544(a) as u32; } }
                    91 => { { let a = pick(); ret = m::un_80301580(a) as u32; } }
                    92 => { { let a = pick(); ret = m::un_803015BC(a) as u32; } }
                    93 => { { let a = pick(); ret = m::un_803015F8(a) as u32; } }
                    94 => { { let a = pick(); ret = m::un_80301634(a) as u32; } }
                    95 => { { let a = pick(); ret = m::un_803016D8(a) as u32; } }
                    96 => { { let a = pick(); ret = m::un_80301734(a) as u32; } }
                    97 => { { let a = pick(); ret = m::un_8030178C(a) as u32; } }
                    98 => { { let a = pick(); ret = m::un_80301800(a) as u32; } }
                    99 => { { let a = pick(); ret = m::un_80301840(a) as u32; } }
                    100 => { { let a = pick(); ret = m::gw_dev_unlock_all_chars(a) as u32; } }
                    101 => { { let a = pick(); ret = m::gw_dev_unlock_all_stages(a) as u32; } }
                    102 => { { let a = pick(); ret = m::un_803018BC(a) as u32; } }
                    103 => { { let a = pick(); ret = m::un_8030191C(a) as u32; } }
                    104 => { { let a = pick(); ret = m::un_80301964(a) as u32; } }
                    105 => { { let a = pick(); ret = m::un_80301A64(a) as u32; } }
                    106 => { { let a = pick(); ret = m::un_80301AD4(a) as u32; } }
                    107 => { { let a = pick(); ret = m::un_80301B48(a) as u32; } }
                    108 => { { let a = pick(); ret = m::un_80301C80(a) as u32; } }
                    109 => { { let a = pick(); m::un_80301CE0(a); } }
                    110 => { { let a = pick(); ret = m::un_80301D40(a) as u32; } }
                    111 => { { let a = pick(); ret = m::un_80301D7C(a) as u32; } }
                    112 => { { let a = pick(); ret = m::un_80301DCC(a) as u32; } }
                    113 => { { let a = pick(); ret = m::un_80301E08(a) as u32; } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}

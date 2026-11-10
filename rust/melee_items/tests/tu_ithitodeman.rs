//! melee_items src/tu/ithitodeman.rs vs the C (gen/mtu/tu/ithitodeman.c: the whole TU, shipping
//! contraction). Written by rust/melee_world/gen/mtu/mk_itk.py from the same description as the C driver;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/mk_itk.py run ithitodeman`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::ithitodeman as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/ithitodeman_gen.rs"]
mod g;
pub static mut VIEWPUN: bool = false;
fn flg(f: &UnkFlagStruct) -> u32 { unsafe { (f.x0.b0() as u32) | (f.x0.b1() as u32) << 1 | (f.x0.b2() as u32) << 2 | (f.x0.b3() as u32) << 3 | (f.x0.b4() as u32) << 4 | (f.x0.b5() as u32) << 5 | (f.x0.b6() as u32) << 6 | (f.x0.b7() as u32) << 7 } }
static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut REFG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWNG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITEM: Item = unsafe { core::mem::zeroed() };
static mut SITEM: Item = unsafe { core::mem::zeroed() };
static mut JT: HSD_JObj = unsafe { core::mem::zeroed() };
static mut JC: HSD_JObj = unsafe { core::mem::zeroed() };
static mut SJT: HSD_JObj = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut SDBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut SART: Article = unsafe { core::mem::zeroed() };
static mut IATTR: ItemAttr = unsafe { core::mem::zeroed() };
static mut SIATTR: ItemAttr = unsafe { core::mem::zeroed() };
static mut VP: Vec = unsafe { core::mem::zeroed() };
static mut BONES: [HSD_JObj; 24] = unsafe { core::mem::zeroed() };
static mut FP1: f32 = 0.0;
static mut ATTR: itHitodemanAttributes = unsafe { core::mem::zeroed() };
static mut SATTR: itHitodemanAttributes = unsafe { core::mem::zeroed() };

unsafe fn setup() {
    G = core::mem::zeroed(); REFG = core::mem::zeroed(); OWNG = core::mem::zeroed(); SG = core::mem::zeroed();
    ITEM = core::mem::zeroed(); SITEM = core::mem::zeroed(); JT = core::mem::zeroed(); JC = core::mem::zeroed(); SJT = core::mem::zeroed();
    BONES = core::mem::zeroed(); DBT = core::mem::zeroed(); SDBT = core::mem::zeroed(); ART = core::mem::zeroed(); SART = core::mem::zeroed();
    IATTR = core::mem::zeroed(); SIATTR = core::mem::zeroed(); VIEWPUN = false;
    ATTR = core::mem::zeroed(); SATTR = core::mem::zeroed();
    g::fill_Item(&mut ITEM); g::fill_Item(&mut SITEM);
    g::fill_HSD_JObj(&mut JT); g::fill_HSD_JObj(&mut JC); g::fill_HSD_JObj(&mut SJT);
    g::fill_ItemAttr(&mut IATTR); g::fill_ItemAttr(&mut SIATTR);
    g::fill_itHitodemanAttributes(&mut ATTR); g::fill_itHitodemanAttributes(&mut SATTR);
    JT.flags &= !0x20000u32; JC.flags &= !0x20000u32; SJT.flags &= !0x20000u32;
    G.user_data = &raw mut ITEM as *mut c_void; G.hsd_obj = &raw mut JT as *mut c_void; SG.user_data = &raw mut SITEM as *mut c_void; SG.hsd_obj = &raw mut SJT as *mut c_void;
    JT.child = if rnd() & 1 != 0 { &raw mut JC } else { null_mut() };
    ITEM.xC4_article_data = &raw mut ART; SITEM.xC4_article_data = &raw mut SART;
    ART.x4_specialAttributes = &raw mut ATTR as *mut _; SART.x4_specialAttributes = &raw mut SATTR as *mut _;
    ITEM.xCC_item_attr = &raw mut IATTR; SITEM.xCC_item_attr = &raw mut SIATTR;
    ITEM.xBBC_dynamicBoneTable = &raw mut DBT; SITEM.xBBC_dynamicBoneTable = &raw mut SDBT;
    for i in 0..100usize { DBT.bones[i] = &raw mut BONES[i % 24]; SDBT.bones[i] = &raw mut BONES[(i + 5) % 24]; }
    ITEM.owner = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() }; SITEM.owner = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() };
    ITEM.xDD4_itemVar.hitodeman.x90 = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() };
}

unsafe fn hash_all() {
    g::hash_Item(&ITEM); g::hash_Item(&SITEM); g::hash_HSD_JObj(&JT); g::hash_HSD_JObj(&JC); g::hash_HSD_JObj(&SJT);
    g::hash_ItemAttr(&IATTR);
    g::hash_itHitodemanAttributes(&ATTR); g::hash_itHitodemanAttributes(&SATTR);
    hm(VP.x.to_bits()); hm(VP.y.to_bits()); hm(VP.z.to_bits()); hm(FP1.to_bits());
}

#[test]
fn ithitodeman_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of_val(&G));
        preg(&raw const REFG, core::mem::size_of_val(&REFG));
        preg(&raw const OWNG, core::mem::size_of_val(&OWNG));
        preg(&raw const SG, core::mem::size_of_val(&SG));
        preg(&raw const ITEM, core::mem::size_of_val(&ITEM));
        preg(&raw const SITEM, core::mem::size_of_val(&SITEM));
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const JC, core::mem::size_of_val(&JC));
        preg(&raw const SJT, core::mem::size_of_val(&SJT));
        preg(&raw const BONES, core::mem::size_of_val(&BONES));
        preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const SDBT, core::mem::size_of_val(&SDBT));
        preg(&raw const ART, core::mem::size_of_val(&ART));
        preg(&raw const SART, core::mem::size_of_val(&SART));
        preg(&raw const IATTR, core::mem::size_of_val(&IATTR));
        preg(&raw const SIATTR, core::mem::size_of_val(&SIATTR));
        preg(&raw const VP, core::mem::size_of_val(&VP));
        preg(&raw const FP1, core::mem::size_of_val(&FP1));
        preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const SATTR, core::mem::size_of_val(&SATTR));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ithitodeman.txt"));
        for op in 0..31u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::it_2725_Logic24_Spawned(&raw mut G); }
                    1 => { m::it_802D43AC(&raw mut G); }
                    2 => { m::it_802D43B0(&raw mut G, &raw mut REFG); }
                    3 => { m::it_802D43EC(&raw mut G); }
                    4 => { m::it_802D4494(&raw mut G); }
                    5 => { m::it_802D4510(&raw mut G); }
                    6 => { ret = (m::it_802D4564(&raw mut G)) as u32; }
                    7 => { m::it_802D472C(&raw mut G); }
                    8 => { ret = (m::it_802D48A8(&raw mut G)) as u32; }
                    9 => { FP1 = rg(); let a1: f32 = rw(); let a2: f32 = rw(); let a3: f32 = rw(); let a4: f32 = rw(); ret = (m::it_802D48B0(&raw mut FP1, a1, a2, a3, a4)).to_bits(); }
                    10 => { m::it_802D4990(&raw mut G); }
                    11 => { ret = (m::itHitodeman_UnkMotion1_Anim(&raw mut G)) as u32; }
                    12 => { m::itHitodeman_UnkMotion1_Phys(&raw mut G); }
                    13 => { ret = (m::itHitodeman_UnkMotion1_Coll(&raw mut G)) as u32; }
                    14 => { m::it_802D4B50(&raw mut G); }
                    15 => { m::it_802D4B54(&raw mut G); }
                    16 => { ret = (m::itHitodeman_UnkMotion2_Anim(&raw mut G)) as u32; }
                    17 => { m::itHitodeman_UnkMotion2_Phys(&raw mut G); }
                    18 => { ret = (m::itHitodeman_UnkMotion2_Coll(&raw mut G)) as u32; }
                    19 => { m::it_802D4C74(&raw mut G); }
                    20 => { m::it_2725_Logic43_Spawned(&raw mut G); }
                    21 => { ret = (m::it_802D4EF4(&raw mut G)) as u32; }
                    22 => { ret = (m::it_802D4F08(&raw mut G)) as u32; }
                    23 => { ret = (m::it_802D4F28(&raw mut G)) as u32; }
                    24 => { ret = (m::itHitodeman_Logic43_Absorbed(&raw mut G)) as u32; }
                    25 => { ret = (m::it_802D4F50(&raw mut G)) as u32; }
                    26 => { m::it_802D4F58(&raw mut G, &raw mut REFG); }
                    27 => { m::it_802D4F78(&raw mut G); }
                    28 => { ret = (m::it_802D4FFC(&raw mut G)) as u32; }
                    29 => { m::it_802D5044(&raw mut G); }
                    30 => { ret = (m::it_802D5048(&raw mut G)) as u32; }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 31 * 400);
    }
}

//! melee_items src/tu/itgamewatchparachute.rs vs the C (gen/mtu/tu/itgamewatchparachute.c: the whole TU, shipping
//! contraction). Written by rust/melee_world/gen/mtu/mk_gamewatch.py from the same description as the C
//! driver; regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py itgamewatchparachute`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itgamewatchparachute as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itgamewatchparachute_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut NEWG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWN: HSD_GObj = unsafe { core::mem::zeroed() };
static mut PAR: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITEM: Item = unsafe { core::mem::zeroed() };
static mut NEWITEM: Item = unsafe { core::mem::zeroed() };
static mut JT: [HSD_JObj; 3] = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut ATTRS: [*mut c_void; 2] = [null_mut(); 2];
static mut ATTR0: [i32; 4] = [0; 4];
static mut POS: Vec = Vec { x: 0.0, y: 0.0, z: 0.0 };

unsafe fn pickg() -> *mut HSD_GObj {
    match rnd() % 3 {
        0 => null_mut(),
        1 => &raw mut OWN,
        _ => &raw mut PAR,
    }
}

unsafe fn fillitem(ip: *mut Item) {
    let b = ip as *mut u8;
    for i in 0..core::mem::size_of::<Item>() {
        *b.add(i) = rnd() as u8;
    }
}

unsafe fn setup() {
    fillitem(&raw mut ITEM);
    fillitem(&raw mut NEWITEM);
    ITEM.xDCC_flag.bf_0 = [if rnd() & 1 != 0 { 0xFF } else { 0 }];
    NEWITEM.xDCC_flag.bf_0 = [if rnd() & 1 != 0 { 0xFF } else { 0 }];
    G = core::mem::zeroed();
    NEWG = core::mem::zeroed();
    OWN = core::mem::zeroed();
    PAR = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTRS = [null_mut(); 2];
    ATTR0 = [0; 4];
    for i in 0..3usize {
        JT[i] = core::mem::zeroed();
        JT[i].child = if i < 2 { &raw mut JT[i + 1] } else { null_mut() };
        JT[i].flags = rnd() & !(JOBJ_USE_QUATERNION as u32);
        JT[i].rotate.x = rw();
        JT[i].rotate.y = rw();
    }
    G.user_data = &raw mut ITEM as *mut c_void;
    G.hsd_obj = &raw mut JT[0] as *mut c_void;
    NEWG.user_data = &raw mut NEWITEM as *mut c_void;
    NEWG.hsd_obj = &raw mut JT[0] as *mut c_void;
    ATTRS[0] = &raw mut ATTR0[0] as *mut c_void;
    ART.x4_specialAttributes = &raw mut ATTRS as *mut c_void;
    ITEM.xC4_article_data = &raw mut ART;
    NEWITEM.xC4_article_data = &raw mut ART;
    ITEM.owner = pickg();
    NEWITEM.owner = pickg();
    ITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    NEWITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    ITEM.x5CC_currentAnimFrame = match rnd() % 4 {
        0 => 3.0,
        1 => 30.0,
        2 => rw(),
        _ => 0.0,
    };
    ITEM.x5D0_animFrameSpeed = match rnd() % 4 {
        0 => 0.0,
        1 => 1.0,
        2 => rw(),
        _ => -0.0,
    };
    ITEM.xDAC_itcmd_var0 = fv();
    ITEM.xDB0_itcmd_var1 = fv();
    ITEM.xDB4_itcmd_var2 = fv();
    ITEM.xDB8_itcmd_var3 = fv();
    NEWITEM.xDAC_itcmd_var0 = fv();
    NEWITEM.xDB0_itcmd_var1 = fv();
    NEWITEM.xDB4_itcmd_var2 = fv();
    NEWITEM.xDB8_itcmd_var3 = fv();
    ITEM.ground_or_air = fv() % 3;
    POS.x = rw();
    POS.y = rw();
    POS.z = rw();
}

unsafe fn hash_item(ip: *const Item) {
    let mut t: Item = core::ptr::read(ip);
    t.owner = pi((*ip).owner) as usize as *mut HSD_GObj;
    t.xDD4_itemVar.gamewatchrescue.xDD8 = pi((*ip).xDD4_itemVar.gamewatchrescue.xDD8) as usize as *mut HSD_GObj;
    t.xC4_article_data = pi((*ip).xC4_article_data) as usize as *mut Article;
    hm((*ip).xDCC_flag.b3() as u32);
    t.xDCC_flag.bf_0 = [0];
    hm(rh(&t));
}

unsafe fn hash_all() {
    hash_item(&raw const ITEM);
    hash_item(&raw const NEWITEM);
    for i in 0..3usize {
        hm(JT[i].flags);
        hm(JT[i].rotate.x.to_bits());
        hm(JT[i].rotate.y.to_bits());
        hm(JT[i].rotate.z.to_bits());
        hm(JT[i].rotate.w.to_bits());
        hm(pi(JT[i].child));
    }
    hm(POS.x.to_bits());
    hm(POS.y.to_bits());
    hm(POS.z.to_bits());
}

#[test]
fn itgamewatchparachute_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const NEWG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const OWN, core::mem::size_of::<HSD_GObj>());
        preg(&raw const PAR, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const NEWITEM, core::mem::size_of::<Item>());
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const ART, core::mem::size_of::<Article>());
        preg(&raw const ATTRS, core::mem::size_of_val(&ATTRS));
        preg(&raw const ATTR0, core::mem::size_of_val(&ATTR0));
        preg(&raw const POS, core::mem::size_of::<Vec>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itgamewatchparachute.txt"));
        for op in 0..10u32 {
            for k in 0..2000u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { let pa = pickg(); let d = rw(); let part = fv(); ret = pi(m::it_802C6C38(pa, &raw mut POS, part as i32, d)); }
                    1 => { m::itGameWatchParachute_Logic74_Destroyed(&raw mut G); }
                    2 => { m::it_802C6D6C(&raw mut G); }
                    3 => { m::it_802C6DB8(&raw mut G); }
                    4 => { m::it_802C6DD8(&raw mut G); }
                    5 => { m::itGameWatchParachute_Logic74_PickedUp(&raw mut G); }
                    6 => { m::it_802C6E50(&raw mut G); }
                    7 => { ret = m::itGamewatchparachute_UnkMotion1_Anim(&raw mut G) as u32; }
                    8 => { let o = pickg(); m::itGameWatchParachute_Logic74_EvtUnk(&raw mut G, o); }
                    _ => {
                        for e in 0..2usize {
                            let t = &(*(&raw const m::it_803F78F8))[e];
                            hm(t.anim_id as u32);
                            hm(fi(t.animated));
                            hm(fi(t.physics_updated));
                            hm(fi(t.collided));
                        }
                    }
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 20000);
    }
}

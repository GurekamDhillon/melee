//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/ityaku.c.
//! src/tu/ityaku.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py ityaku`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::ityaku as m;
use mtu_harness::*;
#[path = "kinds_gen/ityaku_gen.rs"]
mod g;

#[repr(C, align(16))]
struct SpecBuf([[u8; 128]; 3]);
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut IT: [Item; 3] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 48] = unsafe { core::mem::zeroed() };
static mut DBT: [DynamicBoneTable; 3] = unsafe { core::mem::zeroed() };
static mut ART: [Article; 3] = unsafe { core::mem::zeroed() };
static mut ATTR: [ItemAttr; 3] = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
static mut SPECB: SpecBuf = SpecBuf([[0; 128]; 3]);
static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };

static mut GRD: [u8; 64] = [0; 64];
static mut ARTP: [*mut Article; 118] = [null_mut(); 118];
unsafe extern "C" fn cb6(g: *mut HSD_GObj, gr: *mut Ground) { t(7001); t(pi(g)); t(pi(gr)); }
unsafe extern "C" fn cb7(g: *mut HSD_GObj, gr: *mut Ground, v: *mut Vec3, f: *mut HSD_GObj, x: f32) { t(7002); t(pi(g)); t(pi(gr)); tvec(v); t(pi(f)); t(x.to_bits()); }
unsafe extern "C" fn cb8(g: *mut HSD_GObj, gr: *mut Ground, tt: *mut HSD_GObj) { t(7003); t(pi(g)); t(pi(gr)); t(pi(tt)); }

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {

        }
        _ => unreachable!(),
    }
}
unsafe fn setup(prof: i32) {
    SPECB = SpecBuf([[0; 128]; 3]);
    GO = core::mem::zeroed();
    IT = core::mem::zeroed();
    JO = core::mem::zeroed();
    DBT = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTR = core::mem::zeroed();
    ICD = core::mem::zeroed();
    VA = core::mem::zeroed();
    for k in 0..3 {
        g::fill_HSD_GObj(&mut GO[k]);
        g::fill_Item(&mut IT[k]);
        g::fill_ItemAttr(&mut ATTR[k]);
    }
    for i in 0..48 {
        g::fill_HSD_JObj(&mut JO[i]);
        JO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 {
        VA[i].x = rw();
        VA[i].y = rw();
        VA[i].z = rw();
    }
    for k in 0..3 {
        GO[k].user_data = &raw mut IT[k] as *mut _;
        GO[k].hsd_obj = &raw mut JO[k] as *mut _;
        JO[k].child = &raw mut JO[3 + k];
        IT[k].entity = &raw mut GO[k];
        IT[k].xBBC_dynamicBoneTable = &raw mut DBT[k];
        IT[k].xC4_article_data = &raw mut ART[k];
        IT[k].xCC_item_attr = &raw mut ATTR[k];
        ART[k].x0_common_attr = &raw mut ATTR[k];
        ART[k].x4_specialAttributes = SPECB.0[k].as_mut_ptr() as *mut _;
        for i in 0..14 {
            DBT[k].bones[i] = &raw mut JO[6 + 14 * k + i];
        }
    }

    IT[0].owner = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    IT[0].xC64_reflectGObj = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    if rnd() & 1 != 0 {
        IT[0].msid = (rnd() % 14) as i32;
    }
    if rnd() & 1 != 0 {
        IT[0].facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    }
    IT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
    spec_fill(prof);
    ARTP = [null_mut(); 118]; ARTP[117] = &raw mut ART[0]; m::ext::it_804D6D38 = ARTP.as_mut_ptr();
    for k in 0..3 {
        let yk = &mut IT[k].xDD4_itemVar.yaku;
        yk.x2 = (1 + (rnd() & 1)) as i16;
        if yk.x2 == 1 { yk.anon_4.x4 = &raw mut JO[3 + (rnd() % 3) as usize]; } else { yk.anon_4.x4_vec.x = rw(); yk.anon_4.x4_vec.y = rw(); yk.anon_4.x4_vec.z = rw(); }
        yk.x10 = if rnd() & 1 != 0 { GRD.as_mut_ptr() as *mut Ground } else { null_mut() };
        yk.x14 = if rnd() & 1 != 0 { Some(cb6) } else { None };
        yk.x18 = if rnd() & 1 != 0 { Some(cb7) } else { None };
        yk.x1C = if rnd() & 1 != 0 { Some(cb8) } else { None };
        IT[k].toucher = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xCEC_fighterGObj = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
    }
}

unsafe fn hash_all() {
    hm(pi(ART[0].x0_common_attr));
    for k in 0..3 {
        let yk = &mut IT[k].xDD4_itemVar.yaku;
        hm(yk.x2 as u32);
        if yk.x2 == 1 { hm(pi(yk.anon_4.x4)); } else { hm(yk.anon_4.x4_vec.x.to_bits()); hm(yk.anon_4.x4_vec.y.to_bits()); hm(yk.anon_4.x4_vec.z.to_bits()); }
        hm(pi(yk.x10)); hm(fi(yk.x14)); hm(fi(yk.x18)); hm(fi(yk.x1C));
        yk.anon_4.x4 = null_mut(); yk.anon_4.x4_vec.x = 0.0; yk.anon_4.x4_vec.y = 0.0; yk.anon_4.x4_vec.z = 0.0;
        yk.x10 = null_mut(); yk.x14 = None; yk.x18 = None; yk.x1C = None;
    }
    for k in 0..3 {
        g::hash_HSD_GObj(&GO[k]);
        g::hash_Item(&IT[k]);
        g::hash_ItemAttr(&ATTR[k]);
    }
    for i in 0..48 {
        g::hash_HSD_JObj(&JO[i]);
    }
    for i in 0..4 {
        hm(VA[i].x.to_bits());
        hm(VA[i].y.to_bits());
        hm(VA[i].z.to_bits());
    }
}

#[test]
fn ityaku_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const GRD, 64); preg(&raw const m::it_803F8C08, core::mem::size_of::<ItemAttr>());
        freg(cb6 as *const () as usize); freg(cb7 as *const () as usize); freg(cb8 as *const () as usize);
        g::freg_all();
        const PROFTAB: [u8; 7] = [0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_ityaku.txt"));
        for op in 0..7u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = GRD.as_mut_ptr() as *mut Ground; let a1 = fv() as i32; let a2 = (fv() % 20) as i32; let a3: *mut HSD_JObj = if fv() & 1 != 0 { &raw mut JO[3 + (fv() % 3) as usize] } else { null_mut() }; let a4: *mut Vec3 = if fv() & 1 != 0 { &raw mut VA[0] } else { null_mut() }; let a5 = (fv() & 1) as i32; let a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)> = if fv() & 1 != 0 { Some(cb6) } else { None }; let a7: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)> = if fv() & 1 != 0 { Some(cb7) } else { None }; let a8: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)> = if fv() & 1 != 0 { Some(cb8) } else { None }; ret = pi(m::it_802E6AEC(a0, a1, a2, a3, a4, a5, a6, a7, a8)); } // it_802E6AEC
                    1 => {  m::it_2E6A_UnkMotion19_Phys(&raw mut GO[0]); } // it_2E6A_UnkMotion19_Phys
                    2 => {  ret = m::it_2E6A_Logic117_DmgDealt(&raw mut GO[0]) as u32; } // it_2E6A_Logic117_DmgDealt
                    3 => {  ret = m::it_2E6A_Logic117_DmgReceived(&raw mut GO[0]) as u32; } // it_2E6A_Logic117_DmgReceived
                    4 => {  ret = m::it_802E7054(&raw mut GO[0]) as u32; } // it_802E7054
                    5 => {  m::it_2E6A_Logic117_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // it_2E6A_Logic117_EvtUnk
                    6 => { for j in 0..20 { let row = &m::it_803F8C8C[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2800);
    }
}

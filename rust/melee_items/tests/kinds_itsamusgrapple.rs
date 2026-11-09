//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itsamusgrapple.c.
//! src/tu/itsamusgrapple.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itsamusgrapple`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itsamusgrapple as m;
use mtu_harness::*;
#[path = "kinds_gen/itsamusgrapple_gen.rs"]
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

static mut LK: [ItemLink; 24] = unsafe { core::mem::zeroed() };
static mut LGO: [HSD_GObj; 24] = unsafe { core::mem::zeroed() };
static mut FT: [Fighter; 2] = unsafe { core::mem::zeroed() };
static mut FB: [FighterBone; 256] = unsafe { core::mem::zeroed() };
static mut DA: ftSs_DatAttrs = unsafe { core::mem::zeroed() };
static mut FD: ftData = unsafe { core::mem::zeroed() };
static mut AG: [itSamusGrappleAttributes; 3] = unsafe { core::mem::zeroed() };
static mut HBD: itSamusGrapple_HitboxData = unsafe { core::mem::zeroed() };
static mut ANIMP: [*mut core::ffi::c_void; 8] = [null_mut(); 8];
static mut LKN: usize = 0;
static mut LGN: usize = 0;
static mut NL: usize = 0;


unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            
        }
        1 => {
            
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
    {
        core::ptr::write_bytes(&raw mut LK, 0, 1); core::ptr::write_bytes(&raw mut LGO, 0, 1); core::ptr::write_bytes(&raw mut FT, 0, 1);
        core::ptr::write_bytes(&raw mut FB, 0, 1); core::ptr::write_bytes(&raw mut DA, 0, 1); core::ptr::write_bytes(&raw mut FD, 0, 1);
        core::ptr::write_bytes(&raw mut AG, 0, 1); core::ptr::write_bytes(&raw mut HBD, 0, 1); core::ptr::write_bytes(&raw mut ANIMP, 0, 1);
        LKN = 0; LGN = 0;
        NL = 2 + (rnd() % 5) as usize;
        g::fill_Fighter(&mut FT[0]); g::fill_Fighter(&mut FT[1]); g::fill_ftSs_DatAttrs(&mut DA); g::fill_itSamusGrapple_HitboxData(&mut HBD);
        for j in 0..24 { g::fill_ItemLink(&mut LK[j]); g::fill_HSD_GObj(&mut LGO[j]); }
        for j in 0..8 { ANIMP[j] = if rnd() & 1 != 0 { &raw mut ANIMP[(j + 1) % 8] as *mut _ } else { null_mut() }; }
        for j in 0..256 { FB[j].joint = &raw mut JO[j % 48]; }
        for j in 0..8 { JO[16 + j].child = &raw mut JO[32 + j]; }
        for j in 0..NL {
            LK[j].next = if j > 0 { &raw mut LK[j - 1] } else { null_mut() };
            LK[j].prev = if j < NL - 1 { &raw mut LK[j + 1] } else { null_mut() };
            LK[j].gobj = &raw mut LGO[j];
            LK[j].jobj = &raw mut JO[3 + (rnd() % 3) as usize];
            LGO[j].hsd_obj = &raw mut JO[24 + j] as *mut _;
        }
        for j in 0..3 { ART[j].x4_specialAttributes = &raw mut AG[j] as *mut _; }
        for j in 0..4 { core::ptr::write_bytes(&raw mut FT[0].x914[j].anon_134, 0, 1); }
        IT[0].owner = &raw mut GO[1];
        GO[1].user_data = &raw mut FT[0] as *mut _;
        FT[0].gobj = &raw mut GO[1];
        FT[0].ft_data = &raw mut FD; FD.ext_attr = &raw mut DA as *mut _; FT[0].dat_attrs = &raw mut DA as *mut _; FT[0].parts = (&raw mut FB) as *mut FighterBone;
        DA.xA0 = (rnd() % 4) as i32; DA.xB0 = (rnd() % 4) as i32; DA.xC0 = (rnd() % 4) as i32;
        DA.xD0 = (rnd() % 40) as i32;
        FT[0].u.ss.x223C = if prof == 1 && (rnd() & 3) == 0 { null_mut() } else { &raw mut GO[0] };
        FT[0].u.ss.x2240 = (rnd() % 6) as u8;
        {
            const MIDS: [i32; 6] = [0xD4, 0xD6, 0x165, 0x166, 0x100, 0x14];
            FT[0].motion_id = MIDS[(rnd() % 6) as usize] as _;
        }
        FT[0].mv.ss.unk7.x0 = (rnd() % 4) as f32;
        FT[0].mv.ss.grapple.x4 = ((rnd() % 6) as i32 - 2) as f32;
        FT[0].input.held_buttons[0] = (rnd() & 0x4C) as _;
        FT[0].input.pressed_buttons = if rnd() & 1 != 0 { 0x100 } else { 0 };
        FT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        FT[0].x34_scale.y = (rnd() % 4 + 1) as f32 / 2.0;
        HBD.create_hitbox.create_hitbox_0.set_id(rnd() % 4);
        {
            let g = &mut IT[0].xDD4_itemVar.samusgrapple;
            g.x0 = &raw mut LK[NL - 1]; g.x4 = &raw mut LK[0]; g.x8 = if rnd() & 7 != 0 { &raw mut GO[1] } else { null_mut() }; g.xC = &raw mut JO[3];
            g.unk_10 = if rnd() & 1 != 0 { Some(m::fn_802B7E34 as unsafe extern "C" fn(*mut Item_GObj)) } else { None };
            g.x14 = (rnd() & 1) as u8; g.x15 = rnd() as u8; g.x16 = (rnd() % 3) as u8;
        }
        {
            let a = &raw mut AG[0];
            (*a).x0 = rg(); (*a).x4 = rw(); (*a).x8 = rg(); (*a).xC = (rnd() % 5) as i32;
            (*a).x10 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x14 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x18 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x1C = (rnd() % 8 + 1) as f32 / 4.0; (*a).x20 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x24 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x28 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x2C = (rnd() % 8 + 1) as f32 / 4.0; (*a).x30 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x34 = (rnd() % 6) as i32;
            (*a).x38 = (rnd() % 30 + 1) as f32 / 4.0; (*a).x3C = (rnd() % 10) as f32 / 4.0;
            (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48 = rw(); (*a).x4C = (rnd() % 8) as f32 / 2.0; (*a).x50 = rw(); (*a).x54 = (rnd() % 8) as f32 / 2.0; (*a).x58 = (rnd() % 8) as f32 / 4.0;
            (*a).x5C = (rnd() % 4 + 1) as f32 / 2.0; (*a).x60 = (rnd() % 4 + 1) as f32 / 2.0;
            (*a).x64 = &raw mut JO[40] as *mut _; (*a).x68 = &raw mut JO[41] as *mut _; (*a).x6C = &raw mut JO[42] as *mut _; (*a).x70 = &raw mut JO[43] as *mut _;
            (*a).x74 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x78 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x7C = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x80 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x84 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x88 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x8C = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x90 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x94 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x98 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).x9C = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).xA0 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).xA4 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).xA8 = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
            (*a).xAC = if rnd() % 3 != 0 { &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ } else { null_mut() };
        }
    }
}

unsafe fn hash_all() {
    {
        hm(pi(FT[0].u.ss.x223C)); hm(FT[0].u.ss.x2240 as u32); core::ptr::write_bytes(&raw mut FT[0].u, 0, 1);
        for j in 0..4 { hm(FT[0].x914[j].anon_134.hit_grabbed_victim_only() as u32); core::ptr::write_bytes(&raw mut FT[0].x914[j].anon_134, 0, 1); }
        for j in 0..2 { g::hash_Fighter(&FT[j]); }
        for j in 0..24 { g::hash_ItemLink(&LK[j]); g::hash_HSD_GObj(&LGO[j]); }
        for j in 0..256 { g::hash_FighterBone(&FB[j]); }
        g::hash_ftSs_DatAttrs(&DA); g::hash_itSamusGrappleAttributes(&AG[0]); g::hash_itSamusGrapple_HitboxData(&HBD);
        for j in 0..8 { hm(pi(ANIMP[j])); }
        hm(LKN as u32); hm(LGN as u32);
        {
            let g = &IT[0].xDD4_itemVar.samusgrapple;
            hm(pi(g.x0)); hm(pi(g.x4)); hm(pi(g.x8)); hm(pi(g.xC)); hm(fi(g.unk_10)); hm(g.x14 as u32); hm(g.x15 as u32); hm(g.x16 as u32);
        }
        for j in 0..3 { core::ptr::write_bytes(&raw mut IT[j].xDD4_itemVar, 0, 1); }
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
fn itsamusgrapple_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LGO, core::mem::size_of_val(&LGO)); preg(&raw const FT, core::mem::size_of_val(&FT));
        preg(&raw const FB, core::mem::size_of_val(&FB)); preg(&raw const DA, core::mem::size_of_val(&DA));
        preg(&raw const FD, core::mem::size_of_val(&FD)); preg(&raw const AG, core::mem::size_of_val(&AG)); preg(&raw const HBD, core::mem::size_of_val(&HBD)); preg(&raw const ANIMP, core::mem::size_of_val(&ANIMP));
        g::freg_all();
        const PROFTAB: [u8; 49] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itsamusgrapple.txt"));
        for op in 0..49u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::itSamusGrapple_Logic53_Spawned(&raw mut GO[0]); } // itSamusGrapple_Logic53_Spawned
                    1 => { m::it_802B7160(&raw mut GO[1], &raw mut HBD); } // it_802B7160
                    2 => { m::it_802B743C(&raw mut LGO[0], &raw mut IT[0], (fv() % 3) as i32); } // it_802B743C
                    3 => { let v2: i32 = (rnd() % 2) as i32 as i32; let v3: f32 = (rnd() % 4 + 1) as f32 / 2.0 as f32; ret = pi(m::it_802B75FC(&raw mut IT[0], &raw mut JO[3], v2, v3)); } // it_802B75FC
                    4 => {  m::it_802B7B84(&raw mut GO[0]); } // it_802B7B84
                    5 => { let v0: f32 = rw(); ret = pi(m::it_802B7C18(&raw mut GO[1], &raw mut VA[0], v0)); } // it_802B7C18
                    6 => {  m::fn_802B7E34(&raw mut GO[0]); } // fn_802B7E34
                    7 => {  m::itSamusgrapple_UnkMotion0_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion0_Phys
                    8 => {  m::fn_802B805C(&raw mut GO[0]); } // fn_802B805C
                    9 => {  m::itSamusgrapple_UnkMotion1_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion1_Phys
                    10 => {  m::fn_802B8384(&raw mut GO[0]); } // fn_802B8384
                    11 => {  m::itSamusgrapple_UnkMotion2_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion2_Phys
                    12 => {  m::fn_802B8524(&raw mut GO[0]); } // fn_802B8524
                    13 => {  m::itSamusgrapple_UnkMotion3_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion3_Phys
                    14 => {  m::fn_802B8684(&raw mut GO[0]); } // fn_802B8684
                    15 => {  m::itSamusgrapple_UnkMotion4_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion4_Phys
                    16 => {  m::fn_802B8814(&raw mut GO[0]); } // fn_802B8814
                    17 => {  m::itSamusgrapple_UnkMotion5_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion5_Phys
                    18 => {  m::fn_802B895C(&raw mut GO[0]); } // fn_802B895C
                    19 => {  m::itSamusgrapple_UnkMotion6_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion6_Phys
                    20 => {  m::fn_802B8B54(&raw mut GO[0]); } // fn_802B8B54
                    21 => {  m::itSamusgrapple_UnkMotion7_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion7_Phys
                    22 => {  m::fn_802B8D38(&raw mut GO[0]); } // fn_802B8D38
                    23 => {  m::itSamusgrapple_UnkMotion8_Phys(&raw mut GO[0]); } // itSamusgrapple_UnkMotion8_Phys
                    24 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; let v3: f32 = (rnd() % 40) as f32 / 4.0 as f32; m::it_802B900C(v0, &raw mut VA[0], &raw mut AG[0], v3); } // it_802B900C
                    25 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; let v3: f32 = (rnd() % 40) as f32 / 4.0 as f32; m::it_802B91C4(v0, &raw mut VA[0], &raw mut AG[0], v3); } // it_802B91C4
                    26 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; ret = m::it_802B9328(v0, &raw mut VA[0], &raw mut AG[0], &raw mut FT[0]) as u32; } // it_802B9328
                    27 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; ret = m::it_802B99A0(v0, &raw mut VA[0], &raw mut AG[0], &raw mut FT[0]) as u32; } // it_802B99A0
                    28 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; m::it_802B9CE8(v0, &raw mut VA[0], &raw mut AG[0], &raw mut FT[0]); } // it_802B9CE8
                    29 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; ret = m::it_802B9FD4(v0, &raw mut VA[0], &raw mut AG[0]) as u32; } // it_802B9FD4
                    30 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; let v3: f32 = (rnd() % 40) as f32 / 4.0 as f32; ret = m::it_802BA194(v0, &raw mut VA[0], &raw mut AG[0], v3) as u32; } // it_802BA194
                    31 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; let v3: f32 = (rnd() % 40) as f32 / 4.0 as f32; ret = m::it_802BA2D8(v0, &raw mut VA[0], &raw mut AG[0], v3) as u32; } // it_802BA2D8
                    32 => { let v4: f32 = (rnd() % 40) as f32 / 4.0 as f32; ret = m::it_802BA3BC(&raw mut LK[0], &raw mut LK[NL - 1], &raw mut VA[0], &raw mut AG[0], v4) as u32; } // it_802BA3BC
                    33 => {  m::it_802BA5DC(&raw mut LK[0], &raw mut LK[NL - 1], &raw mut VA[0], &raw mut AG[0]); } // it_802BA5DC
                    34 => { let v0: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize]; ret = m::it_802BA760(v0, &raw mut VA[0], &raw mut AG[0], &raw mut FT[0]) as u32; } // it_802BA760
                    35 => {  m::itSamusGrapple_Logic53_PickedUp(&raw mut GO[0]); } // itSamusGrapple_Logic53_PickedUp
                    36 => {  m::it_802BA9B8(&raw mut GO[0]); } // it_802BA9B8
                    37 => {  m::it_802BAA08(&raw mut GO[0]); } // it_802BAA08
                    38 => {  m::it_802BAA58(&raw mut GO[0]); } // it_802BAA58
                    39 => {  m::it_802BAA94(&raw mut GO[0]); } // it_802BAA94
                    40 => {  m::it_802BAAE4(&raw mut GO[0], &raw mut VA[0]); } // it_802BAAE4
                    41 => {  m::it_802BAB40(&raw mut GO[0]); } // it_802BAB40
                    42 => {  m::it_802BAB7C(&raw mut GO[0]); } // it_802BAB7C
                    43 => {  m::it_802BABB8(&raw mut GO[0]); } // it_802BABB8
                    44 => { m::it_802BAC3C(&raw mut GO[1]); } // it_802BAC3C
                    45 => { m::it_802BAC80(&raw mut GO[1]); } // it_802BAC80
                    46 => { m::it_802BACC4(&raw mut GO[1]); } // it_802BACC4
                    47 => {  m::itSamusGrapple_Logic53_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itSamusGrapple_Logic53_EvtUnk
                    48 => { for j in 0..9 { let row = &m::it_803F73A8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 19600);
    }
}

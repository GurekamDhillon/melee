//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lbspdisplay.c.
//! src/tu/lbspdisplay.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lbspdisplay`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbspdisplay as m;
use mtu_harness::*;
#[path = "lbtu_gen/lbspdisplay_gen.rs"]
mod g;

static mut JO: [HSD_JObj; 9] = unsafe { core::mem::zeroed() };
static mut DO: [HSD_DObj; 9] = unsafe { core::mem::zeroed() };
static mut MO: [HSD_MObj; 9] = unsafe { core::mem::zeroed() };
static mut IMG: [HSD_ImageDesc; 1] = unsafe { core::mem::zeroed() };
static mut TEX: [GXTexObj; 1] = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 1] = unsafe { core::mem::zeroed() };
static mut CO: [HSD_CObj; 1] = unsafe { core::mem::zeroed() };
static mut BD: [CameraBlurData; 1] = unsafe { core::mem::zeroed() };
static mut LL: [LightList; 4] = unsafe { core::mem::zeroed() };
static mut LO: [HSD_LObj; 4] = unsafe { core::mem::zeroed() };
static mut LD: [HSD_LightDesc; 4] = unsafe { core::mem::zeroed() };
static mut CD: [HSD_CameraDescPerspective; 1] = unsafe { core::mem::zeroed() };
static mut MEM: [u8; 256] = unsafe { core::mem::zeroed() };
static mut OUT: [*mut HSD_JObj; 6] = unsafe { core::mem::zeroed() };
static mut IX: [u16; 6] = unsafe { core::mem::zeroed() };
static mut LS: [*mut LightList; 5] = unsafe { core::mem::zeroed() };
unsafe extern "C" fn evt(p: *mut HSD_GObj) { t(9001); (*p).p_priority ^= 7; }
unsafe extern "C" fn draw(_: *mut HSD_GObj, pass: i32) { t(9002); t(pass as u32); }
static mut AN: [*mut HSD_LightAnim;1] = [core::ptr::null_mut()];

unsafe fn setup() {
    let _ = 0;
    JO = core::mem::zeroed();
    DO = core::mem::zeroed();
    MO = core::mem::zeroed();
    IMG = core::mem::zeroed();
    TEX = core::mem::zeroed();
    GO = core::mem::zeroed();
    CO = core::mem::zeroed();
    BD = core::mem::zeroed();
    LL = core::mem::zeroed();
    LO = core::mem::zeroed();
    LD = core::mem::zeroed();
    CD = core::mem::zeroed();
    MEM = core::mem::zeroed();
    OUT = core::mem::zeroed();
    IX = core::mem::zeroed();
    LS = core::mem::zeroed();
    for i in 0..9 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..9 { g::fill_HSD_DObj(&mut DO[i]); }
    for i in 0..9 { g::fill_HSD_MObj(&mut MO[i]); }
    for i in 0..1 { g::fill_HSD_ImageDesc(&mut IMG[i]); }
    for i in 0..1 { g::fill_GXTexObj(&mut TEX[i]); }
    for i in 0..1 { g::fill_HSD_GObj(&mut GO[i]); }
    for i in 0..1 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..1 { g::fill_CameraBlurData(&mut BD[i]); }
    for i in 0..4 { g::fill_LightList(&mut LL[i]); }
    for i in 0..4 { g::fill_HSD_LObj(&mut LO[i]); }
    for i in 0..4 { g::fill_HSD_LightDesc(&mut LD[i]); }
    for i in 0..1 { g::fill_HSD_CameraDescPerspective(&mut CD[i]); }
    let n=rnd()%5;
    for k in 0..4 { LL[k].desc=&raw mut LD[k]; LL[k].anims=if rnd()&1 !=0 { &raw mut AN as *mut _ } else { null_mut() }; LS[k]=if (k as u32)<n { &raw mut LL[k] } else { null_mut() }; }
    for k in 0..9 { JO[k].flags &= !((1u32<<5)|(1u32<<14)|(1u32<<12)); JO[k].flags |= if rnd()%8==0 {1u32<<12} else {0}; JO[k].u.dobj=if rnd()&1!=0 { &raw mut DO[k] } else { null_mut() }; DO[k].mobj=&raw mut MO[k]; DO[k].next=if k<8 && rnd()&1!=0 { &raw mut DO[k+1] } else { null_mut() }; }
    JO[0].child=&raw mut JO[1]; JO[1].parent=&raw mut JO[0]; JO[2].parent=&raw mut JO[0]; JO[3].parent=&raw mut JO[0];
    JO[1].next=&raw mut JO[2]; JO[2].next=&raw mut JO[3]; JO[1].child=&raw mut JO[4]; JO[4].next=&raw mut JO[5]; JO[4].parent=&raw mut JO[1]; JO[5].parent=&raw mut JO[1];
    JO[2].child=&raw mut JO[6]; JO[6].parent=&raw mut JO[2]; JO[6].child=&raw mut JO[7]; JO[7].parent=&raw mut JO[6]; JO[0].next=&raw mut JO[8];
    for k in 0..6 { IX[k]=(rnd()%14) as u16; }
    IMG[0].width=(rnd()%641) as u16; IMG[0].height=(rnd()%481) as u16; IMG[0].image_ptr=MEM.as_mut_ptr() as *mut _;
    TEX[0].dummy[0]=IMG[0].width as u32; TEX[0].dummy[1]=IMG[0].height as u32;
    BD[0].mode=(rnd()%3) as u8; BD[0].efb_copy=&raw mut IMG[0]; BD[0].callback=if rnd()&1!=0 { Some(evt) } else { None };
    GO[0].user_data=&raw mut BD[0] as *mut _; GO[0].hsd_obj=&raw mut CO[0] as *mut _;

}
unsafe fn hash_all() {
for k in 0..9 { hm(pi(JO[k].u.dobj)); JO[k].u.dobj=null_mut(); }
    for i in 0..9 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..9 { g::hash_HSD_DObj(&DO[i]); }
    for i in 0..9 { g::hash_HSD_MObj(&MO[i]); }
    for i in 0..1 { g::hash_HSD_ImageDesc(&IMG[i]); }
    for i in 0..1 { g::hash_GXTexObj(&TEX[i]); }
    for i in 0..1 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..1 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..1 { g::hash_CameraBlurData(&BD[i]); }
    for i in 0..4 { g::hash_LightList(&LL[i]); }
    for i in 0..4 { g::hash_HSD_LObj(&LO[i]); }
    for i in 0..4 { g::hash_HSD_LightDesc(&LD[i]); }
    for i in 0..1 { g::hash_HSD_CameraDescPerspective(&CD[i]); }
for k in 0..6 { hm(pi(OUT[k])); }
}
#[test]
fn lbspdisplay_matches_c() {
    unsafe {
        g::install();
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const DO, core::mem::size_of_val(&DO));
        preg(&raw const MO, core::mem::size_of_val(&MO));
        preg(&raw const IMG, core::mem::size_of_val(&IMG));
        preg(&raw const TEX, core::mem::size_of_val(&TEX));
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const BD, core::mem::size_of_val(&BD));
        preg(&raw const LL, core::mem::size_of_val(&LL));
        preg(&raw const LO, core::mem::size_of_val(&LO));
        preg(&raw const LD, core::mem::size_of_val(&LD));
        preg(&raw const CD, core::mem::size_of_val(&CD));
        preg(&raw const MEM, core::mem::size_of_val(&MEM));
        preg(&raw const OUT, core::mem::size_of_val(&OUT));
        preg(&raw const IX, core::mem::size_of_val(&IX));
        preg(&raw const LS, core::mem::size_of_val(&LS));
        g::freg_all();
freg(evt as *const () as usize); freg(draw as *const () as usize);
        let mut c = Check::new(include_str!("data/lbtu_lbspdisplay.txt"));
        for op in 0..20u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = pi(m::lb_80011AC4(LS.as_mut_ptr())); } // lb_80011AC4
                    1 => {  let v1: u32 = (fv()) as u32; m::lb_80011B74(&raw mut DO[0], v1); } // lb_80011B74
                    2 => { ret=m::checkJObjFlags(&raw mut JO[0]) as u32; } // checkJObjFlags
                    3 => {  let v1: u32 = (fv()) as u32; m::lb_80011C18(&raw mut JO[0], v1); } // lb_80011C18
                    4 => { ret=m::lb_80011E24(JO.as_mut_ptr(),OUT.as_mut_ptr(),&[IX[0] as i32,IX[1] as i32,IX[2] as i32,IX[3] as i32,IX[4] as i32,IX[5] as i32,-1]) as u32; } // lb_80011E24
                    5 => {  let v3: i32 = ((fv()%7) as i32 - 1) as i32; ret = m::lb_8001204C(&raw mut JO[0], OUT.as_mut_ptr(), IX.as_mut_ptr(), v3) as u32; } // lb_8001204C
                    6 => {  let v1: i16 = (fv() as i32) as i16; ret = pi(m::setImageFromPreloadedArchive(&raw mut IMG[0], v1)); } // setImageFromPreloadedArchive
                    7 => { IMG[0].image_ptr=null_mut(); let v1: i32 = (fv() as i32) as i32; let v2: i32 = (fv() as i32) as i32; let v3: GXTexFmt = (fv() as i32) as GXTexFmt; let v4: i16 = (fv() as i32) as i16; ret = pi(m::lb_800121FC(&raw mut IMG[0], v1, v2, v3, v4)); } // lb_800121FC
                    8 => {  let v1: u16 = (fv()) as u16; let v2: u16 = (fv()) as u16; let v3: i32 = (fv() as i32) as i32; m::lb_800122C8(&raw mut IMG[0], v1, v2, v3); } // lb_800122C8
                    9 => {  let v2: f32 = (rf()) as f32; m::lb_800122F0(&raw mut IMG[0], &raw mut TEX[0], v2); } // lb_800122F0
                    10 => {  let v1: f32 = (rf()) as f32; let v2: f32 = (rf()) as f32; let v3: f32 = (rf()) as f32; let v4: f32 = (rf()) as f32; let v5: f32 = (rf()) as f32; let v6: f32 = (rf()) as f32; m::lb_8001271C(&raw mut TEX[0], v1, v2, v3, v4, v5, v6); } // lb_8001271C
                    11 => {   m::lb_8001285C(&raw mut IMG[0], &raw mut TEX[0]); } // lb_8001285C
                    12 => { m::setTevAlpha(fv() as u8); } // setTevAlpha
                    13 => {  let v1: u8 = (fv()) as u8; let v2: u8 = (fv()) as u8; let v3: f32 = (rf()) as f32; let v4: f32 = (rf()) as f32; let v5: f32 = (rf()) as f32; let v6: f32 = (rf()) as f32; let v7: f32 = (rf()) as f32; m::lb_80012994(&raw mut IMG[0], v1, v2, v3, v4, v5, v6, v7); } // lb_80012994
                    14 => {   m::fn_80013614(&raw mut GO[0]); } // fn_80013614
                    15 => {   m::fn_800138AC(MEM.as_mut_ptr() as *mut core::ffi::c_void); } // fn_800138AC
                    16 => {  let v1: Option<unsafe extern "C" fn(*mut HSD_GObj)> = (if fv()&1 != 0 { Some(evt) } else { None }) as Option<unsafe extern "C" fn(*mut HSD_GObj)>; m::lb_800138CC(&raw mut GO[0], v1); } // lb_800138CC
                    17 => {  let v1: i8 = (fv() as i32) as i8; m::lb_800138D8(&raw mut GO[0], v1); } // lb_800138D8
                    18 => {  let v1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)> = (if fv()&1 != 0 { Some(draw) } else { None }) as Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>; let v2: i32 = (fv() as i32) as i32; let v3: i8 = (fv() as i32) as i8; let v4: f32 = (rf()) as f32; let v5: f32 = (rf()) as f32; let v6: f32 = (rf()) as f32; let v7: f32 = (rf()) as f32; m::lb_800138EC(&raw mut IMG[0], v1, v2, v3, v4, v5, v6, v7); } // lb_800138EC
                    19 => {   ret = pi(m::lb_80013B14(&raw mut CD[0])); } // lb_80013B14
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8000);
    }
}

//! tu::gm_1832 against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_1832_ref.c, data tests/data/tu_gm_1832.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_1832`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_1832 as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

static mut G_PROC: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut G_ENT: [*mut HSD_GObj; 6] = [core::ptr::null_mut(); 6];

unsafe fn mkjobj() -> *mut HSD_JObj {
    let j = az_t::<HSD_JObj>();
    (*j).translate.x = rf();
    (*j).translate.y = rf();
    (*j).translate.z = rf();
    (*j).scale.x = rf();
    (*j).scale.y = rf();
    (*j).scale.z = rf();
    j
}
unsafe fn mkjobjt() -> *mut HSD_JObj {
    let j = mkjobj();
    (*j).u.dobj = az_t::<HSD_DObj>();
    (*(*j).u.dobj).mobj = az_t::<HSD_MObj>();
    (*(*(*j).u.dobj).mobj).tobj = az_t::<HSD_TObj>();
    (*(*(*(*j).u.dobj).mobj).tobj).aobj = az_t::<HSD_AObj>();
    j
}
unsafe fn mkgobj() -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = mkjobj() as *mut _;
    g
}

include!("stubs/gm_1832.rs");

unsafe fn mktable() -> *mut core::ffi::c_void {
    let n = core::mem::size_of::<t::ClassicIntroTable>();
    let p = az(n) as *mut f32;
    for i in 0..n / 4 {
        *p.add(i) = rf();
    }
    p as *mut _
}
unsafe fn mkscene() -> *mut SceneDesc {
    let s = az_t::<SceneDesc>();
    (*s).models = az(16) as *mut *mut DynamicModelDesc;
    *(*s).models = az_t::<DynamicModelDesc>();
    (**(*s).models).joint = az(64) as *mut HSD_Joint;
    (*s).cameras = az(core::mem::size_of::<SceneCameraDesc>() * 2) as *mut SceneCameraDesc;
    (*(*s).cameras).desc = az(64) as *mut HSD_CObjDesc;
    (*s).lights = az(16) as *mut *mut LightList;
    (*s).fogs = az(core::mem::size_of::<SceneFogDesc>() * 2) as *mut SceneFogDesc;
    (*(*s).fogs).desc = az(64) as *mut HSD_FogDesc;
    s
}
fn t_lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_80016DBC");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            let name = core::ffi::CStr::from_ptr(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) =
                if name == c"gmIntroEasyTable" { mktable() } else { mkscene() as *mut _ };
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}
fn t_lb_80011E24(j: *mut HSD_JObj, o: *mut *mut HSD_JObj, id: i32, end: i32) -> i32 {
    unsafe {
        out("lb_80011E24");
        trp(j as *const u8);
        trp(o as *const u8);
        tri(id as i64);
        tri(end as i64);
        out("\n");
        *o = mkjobjt();
        1
    }
}
fn t_HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const core::ffi::c_char) -> i32 {
    out("HSD_SisLib_803A6B98");
    trp(text as *const u8);
    trf(x);
    trf(y);
    trs(fmt);
    out("\n");
    0
}
fn t_HSD_SisLib_803A70A0(text: *mut HSD_Text, idx: i32, fmt: *const core::ffi::c_char) -> i32 {
    out("HSD_SisLib_803A70A0");
    trp(text as *const u8);
    tri(idx as i64);
    trs(fmt);
    out("\n");
    0
}
fn t_GXLoadPosMtxImm(mtx: *const [f32; 4], id: u32) {
    out("GXLoadPosMtxImm");
    trp(mtx as *const u8);
    tru(id as u64);
    for r in 0..3 {
        for c in 0..4 {
            trf(unsafe { (*mtx.add(r))[c] });
        }
    }
    out("\n");
}
fn t_GXSetProjection(mtx: *const [f32; 4], ty: i32) {
    out("GXSetProjection");
    tri(ty as i64);
    for r in 0..4 {
        for c in 0..4 {
            trf(unsafe { (*mtx.add(r))[c] });
        }
    }
    out("\n");
}
fn t_MTXOrtho(mtx: *mut [f32; 4], tt: f32, b: f32, l: f32, r: f32, n: f32, f: f32) {
    out("MTXOrtho");
    trf(tt);
    trf(b);
    trf(l);
    trf(r);
    trf(n);
    trf(f);
    out("\n");
    for i in 0..16 {
        unsafe { *(mtx as *mut f32).add(i) = rf() };
    }
}

unsafe fn state() {
    let a = &t::lbl_804735A8;
    dump_t("x0", &a.x0);
    dump_t("x4", &a.x4);
    dump_t("x38", &a.x38);
    dump_t("x3A", &a.x3A);
    dump_t("x3C", &a.x3C);
    dump_t("x3E", &a.x3E);
    let b = &t::lbl_804735E8;
    dump_t("x40", &b.x40);
    dump_t("x88", &b.x88);
    dump_t("xD0", &b.xD0);
    dump_t("xDC", &b.xDC);
    dump_t("xE0", &b.xE0);
    dump_t("xE1", &b.xE1);
    dump_t("xE2", &b.xE2);
    dump_t("xE3", &b.xE3);
    dump_t("k", &t::lbl_8047368C);
    dump_t("f0", &t::lbl_804D65F0);
    dump_t("f4", &t::lbl_804D65F4);
    dump_t("f8", &t::lbl_804D65F8);
    dump_t("fc", &t::lbl_804D65FC);
    dump_t("600", &t::lbl_804D6600);
    dump_t("604", &t::lbl_804D6604);
    dump_t("608", &t::lbl_804D6608);
    out(&format!("proc {}\n", G_PROC.flags_3() as i32));
    dump_arena();
    state_end();
}
unsafe fn seed_state() {
    const X38S: [i32; 8] = [4, 9, 0x2C, 0x45, 0x63, 0x8B, 0x8C, 5];
    core::ptr::write_bytes(&raw mut t::lbl_804735A8, 0, 1);
    core::ptr::write_bytes(&raw mut t::lbl_804735E8, 0, 1);
    core::ptr::write_bytes(&raw mut t::lbl_8047368C, 0, 1);
    core::ptr::write_bytes(&raw mut G_PROC, 0, 1);
    let a = &mut t::lbl_804735A8;
    a.x0 = (rnd() % 2) as i32;
    for i in 0..13 {
        a.x4[i] = mkjobjt();
    }
    a.x38 = if rnd() % 3 != 0 { X38S[(rnd() % 8) as usize] as u16 } else { (rnd() % 0x90) as u16 };
    a.x3A = (rnd() % 0x34) as u16;
    a.x3C = (rnd() % 0x34) as u16;
    let k = &mut t::lbl_8047368C;
    k.model_scale_kind = (rnd() % 5) as i32;
    k.game_type = (rnd() % 4) as i32;
    k.xEC = (rnd() % 4) as u8;
    k.xED = if rnd() % 4 == 0 { 0x78 } else { (rnd() % 26) as u8 };
    k.xEE = (rnd() % 30) as u8;
    k.xEF = (1 + rnd() % 2) as u8;
    k.xF0 = (1 + rnd() % 2) as u8;
    for i in 0..3 {
        k.xF1[i] = if rnd() % 6 == 0 { 33 } else { (rnd() % 30) as u8 };
        k.xF4[i] = if rnd() % 6 == 0 {
            0xE
        } else if rnd() % 8 == 0 {
            29
        } else if rnd() % 8 == 0 {
            33
        } else {
            (rnd() % 30) as u8
        };
        k.xF7[i] = (rnd() % 4) as u8;
        k.xFA[i] = (rnd() % 4) as u8;
        k.xFD[i] = (rnd() % 2) as u8;
        k.x100[i] = (rnd() % 2) as u8;
    }
    let b = &mut t::lbl_804735E8;
    b.xDC = mkgobj();
    b.xE0 = (1 + rnd() % 3) as u8;
    b.xE1 = (rnd() % 2) as u8;
    for i in 0..10 {
        b.xD0[i] = (rnd() % b.xE0 as u32) as u8;
    }
    t::lbl_804D65F0 = mkgobj();
    t::lbl_804D65F4 = az_t::<HSD_Archive>();
    t::lbl_804D65F8 = az_t::<HSD_Archive>();
    t::lbl_804D65FC = mkscene();
    t::lbl_804D6600 = mkscene();
    t::lbl_804D6604 = mktable() as *mut t::ClassicIntroTable;
    t::lbl_804D6608 = (rnd() % 3) as i32 - 1;
    for i in 0..6 {
        G_ENT[i] = mkgobj();
    }
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HOOK_lb_80011E24 = Some(t_lb_80011E24);
        ext::HOOK_HSD_SisLib_803A6B98 = Some(t_HSD_SisLib_803A6B98);
        ext::HOOK_HSD_SisLib_803A70A0 = Some(t_HSD_SisLib_803A70A0);
        ext::HOOK_GXLoadPosMtxImm = Some(t_GXLoadPosMtxImm);
        ext::HOOK_GXSetProjection = Some(t_GXSetProjection);
        ext::HOOK_MTXOrtho = Some(t_MTXOrtho);
        ext::HSD_GObj_JObjKind = 0x2A;
        ext::HSD_GObj_CameraKind = 0x2B;
        ext::HSD_GObj_LightKind = 0x2C;
        ext::HSD_GObj_FogKind = 0x2D;
        ext::HSD_SObjLib_804D7960 = 0x2E;
        ext::HSD_GObj_804D783C = 2;
        t::MSL_TrigF_80400770[0] = 1.5;
        t::MSL_TrigF_80400774[0] = 2.5;
        size_t::<HSD_ImageDesc>("HSD_ImageDesc");
        size_t::<HSD_SObj>("HSD_SObj");
        size_t::<HSD_SObjDesc2>("HSD_SObjDesc2");
        size_t::<HSD_GObjProc>("HSD_GObjProc");
        size_t::<t::ClassicModeEnterData>("ClassicModeEnterData");
        size_t::<t::gm_1832_8047368C_t>("struct gm_1832_8047368C_t");
        size_t::<t::ClassicIntroTable>("*lbl_804D6604");
        reg("a0", &t::lbl_804735A8, core::mem::size_of::<t::gm_1832_804735A8_t>());
        reg("img", &t::lbl_804735E8, core::mem::size_of::<t::gm_1832_804735E8_t>());
        reg("k", &t::lbl_8047368C, core::mem::size_of::<t::gm_1832_8047368C_t>());
        reg("a5f0", &t::lbl_804D65F0, 8);
        reg("a5fc", &t::lbl_804D65FC, 8);
        reg("a600", &t::lbl_804D6600, 8);
        reg("a604", &t::lbl_804D6604, 8);
        reg("proc", &raw const G_PROC, core::mem::size_of::<HSD_GObjProc>());
        fnreg("fn_801849E0", t::fn_801849E0 as usize);
        fnreg("fn_80184A04", t::fn_80184A04 as usize);
        fnreg("fn_80184A28", t::fn_80184A28 as usize);
        fnreg("fn_80184A4C", t::fn_80184A4C as usize);
        fnreg("fn_80184A70", t::fn_80184A70 as usize);
        fnreg("fn_80184A94", t::fn_80184A94 as usize);
        fnreg("fn_80184AB8", t::fn_80184AB8 as usize);
        fnreg("fn_8018504C", t::fn_8018504C_cb as usize);
        fnreg("fn_801852FC", t::fn_801852FC as usize);
        fnreg("fn_8018564C", t::fn_8018564C as usize);
        fnreg("fn_8018569C", t::fn_8018569C as usize);
        fnreg("fn_8018575C", t::fn_8018575C as usize);
        fnreg("fn_801857C4", t::fn_801857C4 as usize);
        fnreg("fn_801859C8", t::fn_801859C8 as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);
        fnreg("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize);
        fnreg("HSD_GObj_FogCallback", ext::HSD_GObj_FogCallback as usize);
        fnreg("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize);
        fnreg("HSD_SObjLib_803A49E0", ext::HSD_SObjLib_803A49E0 as usize);
        dump_t("scale", &t::gm_1832_CharScale);
        dump_t("offset", &t::gm_1832_CharOffset);
        dump_t("samus", &t::gm_1832_SamusOffset);
        dump_t("sjis", &t::lbl_803D9414);
        dump_t("sjis2", &t::lbl_804D40A0);
        state_end();

        for c in 0..1200usize {
            let fns: [unsafe extern "C" fn(*mut HSD_GObj); 6] =
                [t::fn_801849E0, t::fn_80184A04, t::fn_80184A28, t::fn_80184A4C, t::fn_80184A70, t::fn_80184A94];
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed_state();
            let g = if c % 2 != 0 { G_ENT[(rnd() % 6) as usize] } else { mkgobj() };
            fns[c % 6](g);
            state();
        }
        for c in 0..1200usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80184AB8(mkgobj());
            state();
        }
        for c in 0..300usize {
            case(5000 + c);
            rseed(6000 + c as u32);
            arena_reset();
            seed_state();
            if c % 3 == 0 {
                t::lbl_804735A8.x38 = 0x2D;
            }
            t::fn_8018504C();
            state();
        }
        for c in 0..60usize {
            case(7000 + c);
            rseed(8000 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::fn_801851C0()));
            state();
        }
        for c in 0..80usize {
            case(8000 + c);
            rseed(9000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_801852FC(mkgobj());
            state();
        }
        for c in 0..3usize {
            case(9000 + c);
            rseed(10000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80185408(rnd() as i32, 0.0, 480.0, 200.0, 640.0);
            state();
        }
        {
            const VALS: [f64; 14] = [10.0, 0.0, 1.0, 2.0, 100.0, 0.25, 1e10, -1.0, 4.0, 9.0, 3.0, 1e-300, 7.5, 123456.789];
            let zz = std::hint::black_box(0.0f64);
            for i in 0..14 {
                out(&format!("sq {:016x}\n", t::fn_801855BC(VALS[i]).to_bits()));
            }
            out(&format!("sq nan {:016x}\n", t::fn_801855BC(zz / zz).to_bits()));
            out(&format!("sq -0 {:016x}\n", t::fn_801855BC(-zz).to_bits()));
            out(&format!("sq inf {:016x}\n", t::fn_801855BC(1.0 / zz).to_bits()));
        }
        for c in 0..20usize {
            case(10000 + c);
            rseed(11000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_8018564C(mkgobj(), 0);
            t::fn_8018569C(mkgobj(), 0);
            t::fn_8018575C(mkgobj(), 0);
            state();
        }
        for c in 0..80usize {
            case(11000 + c);
            rseed(12000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_801857C4(mkgobj());
            state();
        }
        for c in 0..50usize {
            case(12000 + c);
            rseed(13000 + c as u32);
            arena_reset();
            seed_state();
            let mut head: *mut HSD_SObj = core::ptr::null_mut();
            for _ in 0..4 {
                let s = az_t::<HSD_SObj>();
                (*s).x40 = rnd() % 16;
                (*s).x48 = rnd() % 4;
                (*s).next = head;
                head = s;
            }
            let g = mkgobj();
            (*g).hsd_obj = head as *mut _;
            t::fn_801859C8(g);
            state();
        }
        for c in 0..60usize {
            case(13000 + c);
            rseed(14000 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::fn_80185A0C()));
            state();
        }
        for c in 0..80usize {
            case(14000 + c);
            rseed(15000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80185D64();
            state();
        }
        for c in 0..150usize {
            case(15000 + c);
            rseed(16000 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::fn_80185E34()));
            t::fn_80185F5C((rnd() % 3) as i32);
            state();
        }
        for c in 0..60usize {
            case(16000 + c);
            rseed(17000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80186080();
            state();
            t::fn_801861B8();
            state();
        }
        for c in 0..40usize {
            case(17000 + c);
            rseed(18000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80186400();
            state();
        }
        for c in 0..150usize {
            case(18000 + c);
            rseed(19000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80186634(core::ptr::null_mut());
            state();
        }
        for c in 0..4usize {
            case(19000 + c);
            rseed(20000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_Scene_IntroEasy_OnFrame();
            state();
        }
        for c in 0..150usize {
            case(20000 + c);
            rseed(21000 + c as u32);
            arena_reset();
            seed_state();
            let d = az_t::<t::ClassicModeEnterData>();
            let b = d as *mut u8;
            core::ptr::write_bytes(b, 0, 0x20);
            (*d).x0 = (rnd() % 5) as i32;
            (*d).x4 = (rnd() % 4) as i32;
            *b.add(8) = (rnd() % 4) as u8;
            *b.add(9) = if rnd() % 4 == 0 { 0x78 } else { (rnd() % 26) as u8 };
            *b.add(10) = (rnd() % 30) as u8;
            *b.add(11) = (1 + rnd() % 2) as u8;
            *b.add(12) = (1 + rnd() % 2) as u8;
            for k in 13..32usize {
                *b.add(k) = if rnd() % 6 == 0 { 33 } else { (rnd() % 30) as u8 };
            }
            if c % 10 == 0 {
                *b.add(0x10) = 0x1A;
            }
            if c % 10 == 1 {
                *b.add(0x11) = 0x1A;
            }
            t::gm_Scene_IntroEasy_OnEnter(d as *mut _);
            state();
        }
    }
    check("tu_gm_1832");
}

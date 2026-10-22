//! tu::gm_186E against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_186E_ref.c, data tests/data/tu_gm_186E.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_186E`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_186E as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

unsafe fn mkjobj() -> *mut HSD_JObj {
    let j = az_t::<HSD_JObj>();
    (*j).flags = rnd();
    (*j).translate.x = rf();
    (*j).translate.y = rf();
    (*j).translate.z = rf();
    (*j).scale.x = rf();
    (*j).scale.y = rf();
    (*j).scale.z = rf();
    j
}
unsafe fn mkgobj() -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = mkjobj() as *mut _;
    g
}

include!("stubs/gm_186E.rs");

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
            *(pair[0] as *mut *mut core::ffi::c_void) = mkscene() as *mut _;
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
        *o = mkjobj();
        1
    }
}

unsafe fn state() {
    dump_t("a10", &t::lbl_804D6610);
    dump_t("a14", &t::lbl_804D6614);
    dump_t("a18", &t::lbl_804D6618);
    dump_t("b0", &t::lbl_804736B0);
    dump_arena();
    state_end();
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HOOK_lb_80011E24 = Some(t_lb_80011E24);
        ext::HSD_GObj_JObjKind = 0x2A;
        ext::HSD_GObj_CameraKind = 0x2B;
        ext::HSD_GObj_LightKind = 0x2C;
        ext::HSD_GObj_FogKind = 0x2D;
        size_t::<SceneDesc>("SceneDesc");
        size_t::<HSD_GObj>("HSD_GObj");
        size_t::<HSD_JObj>("HSD_JObj");
        size_t::<t::gm_804736B0_t>("gm_804736B0_t");
        reg("a10", &t::lbl_804D6610, 8);
        reg("a14", &t::lbl_804D6614, 8);
        reg("a18", &t::lbl_804D6618, 8);
        reg("b0", &t::lbl_804736B0, core::mem::size_of::<t::gm_804736B0_t>());
        reg("tbl", t::lbl_803D9498.as_ptr(), core::mem::size_of_val(&t::lbl_803D9498));
        fnreg("fn_80186EFC", t::fn_80186EFC as usize);
        fnreg("fn_80186F6C", t::fn_80186F6C as usize);
        fnreg("fn_80187494", t::fn_80187494 as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);
        fnreg("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize);
        fnreg("HSD_GObj_FogCallback", ext::HSD_GObj_FogCallback as usize);
        dump("tbl", t::lbl_803D9498.as_ptr() as *const u8, core::mem::size_of_val(&t::lbl_803D9498));
        state_end();

        for c in 0..400usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            let g = mkgobj();
            t::lbl_804736B0.x0 = (rnd() % 2) as i32;
            t::lbl_804736B0.x4 = if c % 5 == 0 { 0x8B + (rnd() % 3) as i32 } else { (rnd() % 0xA0) as i32 };
            t::lbl_804736B0.x8 = mkjobj();
            t::lbl_804736B0.xC = mkjobj();
            t::fn_80186EFC(g);
            state();
        }
        for c in 0..300usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            arena_reset();
            let g = mkgobj();
            t::lbl_804D6618.x0 = (c % 28) as i32;
            t::lbl_804D6618.x4 = (rnd() % 8) as i32;
            t::lbl_804736B0.x8 = mkjobj();
            t::fn_80186F6C(g);
            state();
        }
        for c in 0..28usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            t::lbl_804D6618.x0 = (rnd() % 30) as i32;
            t::lbl_804D6618.x4 = (rnd() % 8) as i32;
            t::lbl_804D6610 = az_t::<HSD_Archive>();
            out("r");
            trp(t::fn_801873F0() as *const u8);
            out("\n");
            state();
        }
        for c in 0..40usize {
            case(3000 + c);
            rseed(4000 + c as u32);
            arena_reset();
            let g = mkgobj();
            (*g).gxlink_prios = rnd() as _;
            t::fn_80187494(g, rnd() as i32);
            state();
        }
        for c in 0..8usize {
            case(4000 + c);
            rseed(5000 + c as u32);
            arena_reset();
            t::lbl_804D6614 = mkscene();
            t::lbl_804736B0.x8 = core::ptr::null_mut();
            t::lbl_804736B0.xC = core::ptr::null_mut();
            t::fn_801874FC();
            state();
        }
        for c in 0..2usize {
            case(5000 + c);
            rseed(6000 + c as u32);
            arena_reset();
            t::fn_80187714();
            state();
        }
        for c in 0..12usize {
            case(6000 + c);
            rseed(7000 + c as u32);
            arena_reset();
            t::lbl_804D6610 = az_t::<HSD_Archive>();
            t::lbl_804736B0.x0 = (rnd() % 2) as i32;
            t::gm_Scene_IntroAllstar_OnFrame();
            state();
        }
        for c in 0..12usize {
            case(7000 + c);
            rseed(8000 + c as u32);
            arena_reset();
            let d = az_t::<t::enterdata>();
            (*d).x0 = (rnd() % 28) as i32;
            (*d).x4 = (rnd() % 8) as i32;
            t::lbl_804736B0.x0 = 5;
            t::lbl_804736B0.x4 = 9;
            t::gm_Scene_IntroAllstar_OnEnter(d as *mut _);
            state();
        }
    }
    check("tu_gm_186E");
}

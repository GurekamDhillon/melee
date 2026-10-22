//! tu::gm_1879 against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_1879_ref.c, data tests/data/tu_gm_1879.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_1879`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_1879 as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

fn btn() -> u64 {
    let r = rnd();
    (if r & 1 != 0 { 0x100u64 } else { 0 }) | (if r & 2 != 0 { 0x1000u64 } else { 0 })
}

include!("stubs/gm_1879.rs");

unsafe fn mkscene() -> *mut SceneDesc {
    let s = az_t::<SceneDesc>();
    (*s).models = az(16 * 8) as *mut *mut DynamicModelDesc;
    for i in 0..13usize {
        let d = az_t::<DynamicModelDesc>();
        *(*s).models.add(i) = d;
        (*d).joint = az(64) as *mut HSD_Joint;
        if i == 12 || rnd() % 4 != 0 {
            (*d).anims = az(4 * 8) as *mut *mut HSD_AnimJoint;
            for k in 0..4usize {
                *(*d).anims.add(k) = if rnd() % 2 != 0 { az(64) as *mut HSD_AnimJoint } else { core::ptr::null_mut() };
            }
        }
    }
    (*s).cameras = az(core::mem::size_of::<SceneCameraDesc>() * 2) as *mut SceneCameraDesc;
    (*(*s).cameras).desc = az(64) as *mut HSD_CObjDesc;
    (*s).lights = az(16) as *mut *mut LightList;
    s
}
unsafe fn mkcam() -> *mut SceneCameraDesc {
    let c = az(core::mem::size_of::<SceneCameraDesc>() * 2) as *mut SceneCameraDesc;
    (*c).desc = az(64) as *mut HSD_CObjDesc;
    (*c).anims = az(2 * 8) as *mut *mut HSD_CameraAnim;
    *(*c).anims = az(64) as *mut HSD_CameraAnim;
    c
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
                if name == c"ScItrNormal_scene_data" { mkscene() as *mut _ } else { mkcam() as *mut _ };
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}

unsafe fn mkcobj(eye_ok: bool) -> *mut HSD_CObj {
    let c = az_t::<HSD_CObj>();
    (*c).eyepos = az_t::<HSD_WObj>();
    if eye_ok {
        (*(*c).eyepos).aobj = az_t::<HSD_AObj>();
        (*(*(*c).eyepos).aobj).curr_frame = rf();
    }
    (*c).interest = az_t::<HSD_WObj>();
    if rnd() % 2 != 0 {
        (*(*c).interest).aobj = az_t::<HSD_AObj>();
        (*(*(*c).interest).aobj).curr_frame = rf();
    }
    c
}
unsafe fn mkgobj(obj: *mut core::ffi::c_void) -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = obj;
    g
}
unsafe fn mkjobj() -> *mut HSD_JObj {
    az_t::<HSD_JObj>()
}

unsafe fn seed(full: bool) {
    let d = &mut t::lbl_804736C0;
    core::ptr::write_bytes(d as *mut t::gm_1832_804736C0_t, 0, 1);
    d.x0 = mkscene();
    d.x4 = mkcam();
    d.x8 = if full { mkgobj(mkcobj(rnd() % 2 != 0) as *mut _) } else { core::ptr::null_mut() };
    d.x34 = rnd() as u16;
    d.set_stage_index((rnd() % 12) as u8);
    d.set_done((rnd() % 2) as u8);
    d.set_flash((rnd() % 2) as u8);
    d.set_active((rnd() % 2) as u8);
    d.set_frame_counter((rnd() % 16) as u8);
    d.set_anim_state((rnd() % 4) as u8);
    d.set_state2((rnd() % 4) as u8);
    d.x38 = (rnd() % 4) as u8;
    t::lbl_804D6620 = az_t::<HSD_Archive>();
}
unsafe fn state() {
    dump_t("s", &t::lbl_804736C0);
    dump_t("a20", &t::lbl_804D6620);
    dump_arena();
    state_end();
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HSD_GObj_JObjKind = 0x2A;
        ext::HSD_GObj_CameraKind = 0x2B;
        ext::HSD_GObj_LightKind = 0x2C;
        ext::HSD_GObj_FogKind = 0x2D;
        size_t::<t::gm_1832_804736C0_t>("gm_1832_804736C0_t");
        size_t::<SceneDesc>("SceneDesc");
        size_t::<HSD_CObj>("HSD_CObj");
        size_t::<HSD_AObj>("HSD_AObj");
        size_t::<t::gm_80187F48_EnterData>("gm_80187F48_EnterData");
        reg("s", &t::lbl_804736C0, core::mem::size_of::<t::gm_1832_804736C0_t>());
        reg("a20", &t::lbl_804D6620, 8);
        fnreg("fn_80187910", t::fn_80187910 as usize);
        fnreg("fn_80187AB4", t::fn_80187AB4 as usize);
        fnreg("fn_80187C9C", t::fn_80187C9C as usize);
        fnreg("fn_80187CF4", t::fn_80187CF4 as usize);
        fnreg("Camera_800304E0", ext::Camera_800304E0 as usize);
        fnreg("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);
        fnreg("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize);
        dump("bgm", t::gm_1832_NormalBgmIds.as_ptr() as *const u8, core::mem::size_of_val(&t::gm_1832_NormalBgmIds));
        dump("masks", t::gm_1832_NormalAudioMasks.as_ptr() as *const u8, core::mem::size_of_val(&t::gm_1832_NormalAudioMasks));
        state_end();

        for c in 0..500usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed(false);
            let g = mkgobj(mkcobj(true) as *mut _);
            t::fn_80187910(g);
            state();
        }
        for c in 0..600usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            arena_reset();
            seed(false);
            let g = mkgobj(mkjobj() as *mut _);
            t::fn_80187AB4(g);
            state();
        }
        for c in 0..3usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            seed(false);
            t::fn_80187C9C(mkgobj(mkjobj() as *mut _), rnd() as i32);
            state();
        }
        for c in 0..600usize {
            case(3000 + c);
            rseed(4000 + c as u32);
            arena_reset();
            seed(false);
            let g = mkgobj(mkjobj() as *mut _);
            t::fn_80187CF4(g);
            state();
        }
        for c in 0..48usize {
            case(4000 + c);
            rseed(5000 + c as u32);
            arena_reset();
            seed(false);
            let d = az_t::<t::gm_80187F48_EnterData>();
            (*d).x0 = (rnd() % 4) as u8;
            (*d).stage_index = (c % 12) as u8;
            (*d).stkind = (rnd() % 40) as u16;
            t::gm_Scene_IntroNormal_OnEnter(d as *mut _);
            state();
        }
        for c in 0..4usize {
            case(5000 + c);
            rseed(6000 + c as u32);
            arena_reset();
            seed(false);
            t::gm_Scene_IntroNormal_OnLeave(core::ptr::null_mut());
            t::gm_Scene_IntroNormal_OnFrame();
            out(&format!("r {}\n", t::gm_8018841C() as i32));
            out(&format!("r {}\n", t::gm_8018841C() as i32));
            state();
        }
        for c in 0..40usize {
            case(6000 + c);
            rseed(7000 + c as u32);
            arena_reset();
            seed(c % 4 != 0);
            out(&format!("r {:08x}\n", t::gm_801883C0().to_bits()));
            state();
        }
    }
    check("tu_gm_1879");
}

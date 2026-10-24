//! State oracle for tu/mngallery.rs (src/oracle.rs, gen/tu/run.py).
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::mngallery as tu;
use melee_modes::tu::mngallery::ext;
use melee_modes::tu::mngallery::mnGallery_804D6C88_userdata as Ud;
use melee_types::*;
#[path = "mn_common/mod.rs"]
mod mn;
use mn::*;

unsafe fn flow(c: &mut Case) {
    tu::mn_804A04F0.cur_menu = c.rng.range(0, 40) as u8;
    tu::mn_804A04F0.prev_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.entering_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.buttons = c.rng.next();
    tu::mn_804D6BC8.cooldown = pick(c, &[0, 0, 0, 1, 5]);
    tu::mn_804D6BC8.x2 = c.rng.range(0, 99) as u16;
    tu::mn_804D6BC8.x4 = c.rng.range(0, 99) as i32;
    tu::HSD_GObj_804D783C = c.rng.range(0, 7) as i32;
    tu::HSD_GObj_JObjKind = c.rng.range(0, 255) as u8;
    tu::HSD_GObj_CameraKind = c.rng.range(0, 255) as u8;
    tu::rate_table = c.buf(16).cast();
    for i in 0..4 {
        let t = pick(c, &[0u32, 0, 0x100, 0x200, 0x1000, 0x20, 0x1300, 0x4]);
        tu::HSD_PadCopyStatus[i].trigger = t;
    }
    tu::gmMainLib_8046B0F0.xC = pick(c, &[0, 0, 1, 5]);
}

/// A cursor model: root jobj -> child jobj (u.dobj -> dobj -> next dobj, each with mobj/tobj).
unsafe fn cursor_jobj(c: &mut Case) -> *mut HSD_JObj {
    let root = jobj(c);
    let ch = jobj(c);
    let mut ds = [null_mut::<HSD_DObj>(); 2];
    for d in ds.iter_mut() {
        let dobj = c.zobj::<HSD_DObj>();
        let mobj = c.zobj::<HSD_MObj>();
        let tobj = c.zobj::<HSD_TObj>();
        (*mobj).tobj = tobj;
        (*dobj).mobj = mobj;
        *d = dobj;
    }
    (*ds[0]).next = ds[1];
    (*ch).u.dobj = ds[0];
    (*root).child = ch;
    root
}

/// A list model: root jobj with `n` children chained through `next`.
unsafe fn list_jobj(c: &mut Case) -> *mut HSD_JObj {
    let root = jobj(c);
    let n = c.rng.range(0, 3);
    let mut prev: *mut HSD_JObj = null_mut();
    for _ in 0..n {
        let ch = jobj(c);
        (*ch).translate.x = c.rng.f32_in(-5.0, 5.0);
        (*ch).translate.y = c.rng.f32_in(-5.0, 5.0);
        (*ch).translate.z = c.rng.f32_in(-5.0, 5.0);
        if prev.is_null() {
            (*root).child = ch;
        } else {
            (*prev).next = ch;
        }
        prev = ch;
    }
    root
}

unsafe fn plain_gobj(c: &mut Case) -> *mut HSD_GObj {
    let j = jobj(c);
    gobj(c, null_mut(), j.cast())
}

unsafe fn userdata(c: &mut Case) -> *mut Ud {
    let u = c.zobj::<Ud>();
    (*u).unk0 = pick(c, &[0, 0, 1]);
    (*u).unk1 = pick(c, &[0, 1, 0x31, 0x32, 0x33, 0x80]);
    (*u).unk2 = c.rng.range(0, 3) as u8;
    (*u).gobj4 = if c.rng.chance(2, 3) { plain_gobj(c) } else { null_mut() };
    (*u).gobj8 = if c.rng.chance(2, 3) { plain_gobj(c) } else { null_mut() };
    (*u).text = if c.rng.chance(2, 3) { text(c) } else { null_mut() };
    (*u).state = pick(c, &[0, 1, 1, 2, 3, 3, 4]);
    (*u).unk15 = pick(c, &[0, 0, 1, 1, 2]);
    (*u).frame = pick(c, &[0, 5, 0x12, 0x13, 0x1C, 0x1D, 0x30]);
    (*u).gobjs[0] = plain_gobj(c);
    (*u).gobjs[1] = plain_gobj(c);
    u
}

unsafe fn ud_gobj(c: &mut Case) -> *mut HSD_GObj {
    let u = userdata(c);
    let j = jobj(c);
    gobj(c, u.cast(), j.cast())
}

fn rets(c: &mut Case) {
    let sobjs = c.obj::<HSD_SObj>();
    c.ret_ptr("lbMthp_8001F624", sobjs);
    c.ret_int("lbMthp_8001F604", 0, 1);
    c.ret_int("HSD_CObjSetCurrent", 0, 1);
    c.ret_int("gmMainLib_8015ECB0", 0, 100);
    let p = c.buf(16);
    c.ret_ptr("gm_801ACC94", p);
    let btn = pick(c, &[0u64, 0x20, 0x1, 0x2, 0x100, 0x200, 0x101, 0x22]);
    c.ret_seq_u64("mn_80229624", &[btn]);
    c.ret_pool("HSD_SisLib_803A5ACC", core::mem::size_of::<HSD_Text>());
    let cobj = c.buf(0x80);
    c.ret_ptr("HSD_CObjAlloc", cobj);
    let gs: std::vec::Vec<*mut HSD_GObj> = (0..3).map(|_| unsafe { plain_gobj(c) }).collect();
    c.ret_seq("GObj_Create", &gs);
    let p = c.buf(0x40);
    c.ret_ptr("HSD_MemAlloc", p);
    procs(c, 3);
}

fn b_none(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
}

fn b_render(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnGallery_804D6C88 = ud_gobj(c);
    }
    rets(c);
    let cobj = c.buf(0x80);
    let g = unsafe { gobj(c, null_mut(), cobj.cast()) };
    c.arg(g);
    let pass = c.rng.range(0, 3) as i32;
    c.arg(pass);
}

fn b_camera(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { plain_gobj(c) };
    c.arg(g);
    let w = pick(c, &[0x280u32, 0x1C0, 0, 0xFFFF]);
    c.arg(w);
    let h = pick(c, &[0x1E0u32, 0x150, 0, 0x8000]);
    c.arg(h);
    let p = pick(c, &[1u32, 5, 0x80]);
    c.arg(p);
}

fn b_data(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnGallery_804D6C88 = ud_gobj(c);
    }
    rets(c);
    let u = unsafe { userdata(c) };
    c.arg(u);
}

fn b_gobj_data(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnGallery_804D6C88 = ud_gobj(c);
    }
    rets(c);
    let g = unsafe { plain_gobj(c) };
    let u = unsafe { userdata(c) };
    c.arg(g);
    c.arg(u);
}

fn b_global(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnGallery_804D6C88 = ud_gobj(c);
    }
    rets(c);
}

fn b_input(c: &mut Case) {
    b_global(c);
    let g = unsafe { plain_gobj(c) };
    c.arg(g);
}

fn b_ud_gobj(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { ud_gobj(c) };
    c.arg(g);
}

fn b_cursor(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnGallery_804D6C88 = ud_gobj(c);
        let pj = list_jobj(c);
        let parent = gobj(c, null_mut(), pj.cast());
        let cu = c.zobj::<mnGallery_child_userdata>();
        (*cu).parent_gobj = parent;
        (*cu).index = c.rng.range(0, 3) as i32;
        let cj = cursor_jobj(c);
        let g = gobj(c, cu.cast(), cj.cast());
        c.arg(g);
    }
    rets(c);
}

use melee_modes::tu::mngallery::mnGallery_child_userdata;

/// What GObj_Create / HSD_MemAlloc would have handed 8025963C (`enter`: and 80259868, which creates
/// a proc gobj first and the camera last).
fn create(c: &mut Case, enter: bool) {
    unsafe {
        flow(c);
        tu::mn_804D6BB8 = c.buf(16).cast();
        let u = c.zobj::<Ud>();
        let cu1 = c.zobj::<mnGallery_child_userdata>();
        let cu2 = c.zobj::<mnGallery_child_userdata>();
        c.ret_seq("HSD_MemAlloc", &[u.cast::<c_void>(), cu1.cast(), cu2.cast()]);
        let lj = list_jobj(c);
        let g0 = gobj(c, u.cast(), lj.cast());
        let j1 = cursor_jobj(c);
        let c1 = gobj(c, cu1.cast(), j1.cast());
        let j2 = cursor_jobj(c);
        let c2 = gobj(c, cu2.cast(), j2.cast());
        if enter {
            let a = plain_gobj(c);
            let cam = plain_gobj(c);
            c.ret_seq("GObj_Create", &[a, g0, c1, c2, cam]);
        } else {
            c.ret_seq("GObj_Create", &[g0, c1, c2]);
        }
        tu::mnGallery_804A0BA0.joint = c.buf(16).cast();
        tu::mnGallery_804A0BA0.animjoint = c.buf(16).cast();
        tu::mnGallery_804A0BB0.joint = c.buf(16).cast();
        tu::mnGallery_804A0BB0.animjoint = c.buf(16).cast();
    }
    c.ret_pool("HSD_SisLib_803A5ACC", core::mem::size_of::<HSD_Text>());
    procs(c, 4);
    let cobj = c.buf(0x80);
    c.ret_ptr("HSD_CObjAlloc", cobj);
}
fn b_create(c: &mut Case) {
    create(c, false);
}
fn b_enter(c: &mut Case) {
    create(c, true);
}

#[test]
fn mngallery_vs_c() {
    let funcs = [
        fd!("float_order_helper", "p", "v", 80, |c| { let p = c.obj::<HSD_SObj>(); c.arg(p); }, |a| { tu::float_order_helper(&mut *p(a, 0)); 0 }),
        fd!("mnGallery_80258940", "", "v", 4, b_none, |a| { tu::mnGallery_80258940(); 0 }),
        fd!("mnGallery_8025896C", "pi", "v", 40, b_render, |a| { tu::mnGallery_8025896C(p(a, 0), ai32(a, 1)); 0 }),
        fd!("mnGallery_80258A08", "piii", "v", 20, b_camera, |a| { tu::mnGallery_80258A08(p(a, 0), au32(a, 1) as u16, au32(a, 2) as u16, au32(a, 3)); 0 }),
        fd!("mnGallery_80258BC4", "p", "v", 120, b_data, |a| { tu::mnGallery_80258BC4(p(a, 0)); 0 }),
        fd!("mnGallery_80258D50", "p", "v", 60, b_data, |a| { tu::mnGallery_80258D50(p(a, 0)); 0 }),
        fd!("mnGallery_80258DBC", "pp", "v", 200, b_gobj_data, |a| { tu::mnGallery_80258DBC(p(a, 0), p(a, 1)); 0 }),
        fd!("fn_80258ED0_helper", "", "v", 60, b_global, |a| { tu::fn_80258ED0_helper(); 0 }),
        fd!("fn_80258ED0", "p", "v", 400, b_input, |a| { tu::fn_80258ED0(p(a, 0)); 0 }),
        fd!("fn_802590C4_inline", "p", "v", 20, b_ud_gobj, |a| { tu::fn_802590C4_inline(p(a, 0)); 0 }),
        fd!("fn_802590C4", "p", "v", 120, b_ud_gobj, |a| { tu::fn_802590C4(p(a, 0)); 0 }),
        fd!("mnGallery_802591BC", "p", "v", 200, b_cursor, |a| { tu::mnGallery_802591BC(p(a, 0)); 0 }),
        fd!("mnGallery_80259604", "p", "v", 20, b_data, |a| { tu::mnGallery_80259604(p(a, 0)); 0 }),
        fd!("mnGallery_8025963C", "", "v", 40, b_create, |a| { tu::mnGallery_8025963C(); 0 }),
        fd!("mnGallery_80259868", "", "v", 40, b_enter, |a| { tu::mnGallery_80259868(); 0 }),
    ];
    let fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("mnGallery_8025896C", tu::mnGallery_8025896C as usize),
        ("fn_80258ED0", tu::fn_80258ED0 as usize),
        ("fn_802590C4", tu::fn_802590C4 as usize),
        ("mnGallery_802591BC", tu::mnGallery_802591BC as usize),
        ("lbMthp_8001F67C", ext::lbMthp_8001F67C as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_Free", ext::HSD_Free as usize),
    ];
    let globals = tu::oracle_globals();
    run_tu(&TuOracle { tu: "mngallery", funcs: &funcs, globals, fn_table });
}

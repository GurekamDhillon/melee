//! src/melee/ft/ftdynamics.c: fighter dynamic bones (hair, capes, Kirby hats). No float math of
//! its own (copies only; no fused site), so the port is a pointer walk over native types; the
//! lb_00F9 dynamics engine, ftAnim, jobj.c and m-ex/Geno helpers are TODO stubs in `ext`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees / globals in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        HSD_JOBJ_SET_MTX_DIRTY_SUB HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj);
        HSD_JOBJ_SETUP_MATRIX_SUB HSD_JObjSetupMatrixSub(j: *mut HSD_JObj);
        HSD_JOBJ_CLEAR_FLAGS HSD_JObjClearFlags(j: *mut HSD_JObj, flags: u32);
        LB_8000FD18 lb_8000FD18(d: *mut DynamicsDesc);
        LB_8000FD48 lb_8000FD48(j: *mut HSD_JObj, d: *mut DynamicsDesc, count: usize);
        LB_8001044C lb_8001044C(d: *mut DynamicsDesc, x1670: *mut Fighter_x1670_t, x166c: i32, pos_y: f32, b: i32, part: i32, mode: i32, b0: i32);
        LB_80011710 lb_80011710(src: *mut DynamicsDesc, dst: *mut DynamicsDesc);
        LB_800119DC lb_800119DC(v: *mut Vec3, n: i32, a: f32, b: f32, c: f32);
        LB_80011ABC lb_80011ABC() -> i32;
        LB_8000B1CC lb_8000B1CC(j: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3);
        FTCOLL_8007AF60 ftColl_8007AF60(g: G);
        FTANIM_8006EED4 ftAnim_8006EED4(fp: *mut Fighter, part: i32, tree: *mut FigaTree, frame: f32, speed: f32);
        FTANIM_80070758 ftAnim_80070758(j: *mut HSD_JObj);
        FTANIM_8006EBE8 ftAnim_8006EBE8(g: G, start: f32, rate: f32, blend: f32);
        FTANIM_8006E9B4 ftAnim_8006E9B4(g: G);
        GROUND_801C4E20 Ground_801C4E20();
        // `stage_info.grkind` (gr/stage.c global).
        STAGE_INFO_GRKIND stage_info_grkind() -> i32;
        // `ft_80459B88.hats[kind]` (ftdata.c global; PC: array, never NULL).
        FT_80459B88_HAT ft_80459B88_hat(kind: i32) -> *mut KirbyHatStruct;
        FTKB_MEX_KIRBY_LIKE ftKb_MexKirbyLike(kind: i32) -> i32;
        GENO_DEFINE_TEMPLATE_SKELETON Geno_DefineTemplateSkeleton(kind: i32) -> i32;
        GENO_DEFINE_BASE_KIND Geno_DefineBaseKind(kind: i32) -> i32;
    }
}

const JOBJ_MTX_DIRTY: u32 = 1 << 6;
const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
const JOBJ_MTX_INDEP_SRT: u32 = 1 << 25;
const NUM_MAX: i32 = Ft_Dynamics_NumMax as i32;

fn mtx_is_dirty(j: &HSD_JObj) -> bool {
    j.flags & JOBJ_USER_DEF_MTX == 0 && j.flags & JOBJ_MTX_DIRTY != 0
}
/// jobj.h `HSD_JObjSetMtxDirty` macro.
fn set_mtx_dirty(j: *mut HSD_JObj) {
    if !j.is_null() && !mtx_is_dirty(unsafe { &*j }) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}
fn set_translate(j: *mut HSD_JObj, v: Vec3) {
    assert!(!j.is_null(), "jobj.h:481");
    unsafe { (*j).translate = v };
    if unsafe { (*j).flags } & JOBJ_MTX_INDEP_SRT == 0 {
        set_mtx_dirty(j);
    }
}
fn set_scale(j: *mut HSD_JObj, v: Vec3) {
    assert!(!j.is_null(), "jobj.h:382");
    unsafe { (*j).scale = v };
    if unsafe { (*j).flags } & JOBJ_MTX_INDEP_SRT == 0 {
        set_mtx_dirty(j);
    }
}
fn setup_matrix(j: *mut HSD_JObj) {
    if !j.is_null() && mtx_is_dirty(unsafe { &*j }) {
        ext::HSD_JObjSetupMatrixSub(j);
    }
}

/// `FTKB_IS_KIRBY(kind)` (TARGET_PC: m-ex Kirby clones count).
fn is_kirby(kind: i32) -> bool {
    kind == Ft_Kind_Kirby as i32
        || (kind >= Ft_Kind_Mex0 as i32 && kind < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(kind) != 0)
}
/// `ftGetTemplateKind` (inlines.h, TARGET_PC).
fn template_kind(f: &Fighter) -> i32 {
    let k = f.kind as i32;
    if k >= Ft_Kind_Mex0 as i32 && ext::Geno_DefineTemplateSkeleton(k) != 0 {
        return ext::Geno_DefineBaseKind(k);
    }
    k
}

#[inline(always)]
fn part<'a>(f: &Fighter, i: i32) -> &'a mut FighterBone {
    unsafe { &mut *f.parts.offset(i as isize) }
}
#[inline(always)]
fn pflag(b: &FighterBone) -> u8 {
    unsafe { b.anon_8.anon_0.flags_b0() }
}
#[inline(always)]
fn set_pflag(b: &mut FighterBone, v: u8) {
    unsafe { b.anon_8.anon_0.set_flags_b0(v) }
}
#[inline(always)]
fn dyn_bones<'a>(d: *mut ftDynamics) -> &'a mut ArticleDynamicBones {
    unsafe { &mut *(*d).anon_0.ftDynamicBones }
}

fn cb40_inline(data: *mut DynamicsData) {
    unsafe {
        let d = &mut (*data).desc.lb_unk0;
        let m = &(*d.jobj).mtx;
        d.unk_2C.x = m[0][3];
        d.unk_2C.y = m[1][3];
        d.unk_2C.z = m[2][3];
        d.unk_44 = 0.0;
        set_translate(d.jobj, d.translate);
        set_scale(d.jobj, d.scale);
    }
}

/// `ftCo_8009CB40` (static). `arg3` is a FigaTree* also used as an int count (truncated to s32).
pub fn ftCo_8009CB40(f: &mut Fighter, bone_idx: isize, arg2: i32, arg3: *mut FigaTree) {
    let dy = unsafe { (*f.ft_data).x2C };
    if unsafe { (*dy).anon_0.dynamicsNum } == 0 {
        return;
    }
    let mut bone_id = dyn_bones(dy).array[bone_idx as usize].bone_id;
    let flag: i32 = (arg2 != 0) as i32;
    let mut j = part(f, bone_id).joint;
    let mut k: i32 = 0;
    f.dynamic_bone_sets[bone_idx as usize].bone_id =
        if arg2 == 1 { arg3 as isize as i32 } else { 0x100 };
    let mut data = f.dynamic_bone_sets[bone_idx as usize].dyn_desc.data;
    let inverse = 1 - flag;
    let limit = arg3 as isize as i32;
    while !data.is_null() {
        let b = part(f, bone_id);
        let v = if k < limit { inverse } else { flag };
        if v != 0 && pflag(b) == 0 {
            cb40_inline(data);
        }
        set_pflag(b, v as u8);
        if pflag(b) != 0 {
            ext::HSD_JObjClearFlags(j, 0x20000);
        }
        j = if j.is_null() { core::ptr::null_mut() } else { unsafe { (*j).child } };
        data = unsafe { (*data).next };
        k += 1;
        bone_id += 1;
    }
}

fn num_over(f: &Fighter, line: u32) {
    if f.dynamics_num >= NUM_MAX {
        panic!("fighter dynamics num over! (ftdynamics.c:{line})");
    }
}

/// `ftCo_8009CF84`: set up the fighter's own dynamic bones.
pub fn ftCo_8009CF84(f: &mut Fighter) {
    let data = f.ft_data;
    let dy = unsafe { (*data).x2C };
    f.dynamics_num = unsafe { (*dy).anon_0.dynamicsNum };
    num_over(f, 109);
    let mut i: isize = 0;
    while i < unsafe { (*dy).anon_0.dynamicsNum } as isize {
        let bones = &mut dyn_bones(dy).array[i as usize];
        ext::lb_8000FD48(part(f, bones.bone_id).joint, &mut f.dynamic_bone_sets[i as usize].dyn_desc, bones.dyn_desc.count as usize);
        f.dynamic_bone_sets[i as usize].bone_id = 0;
        ftCo_8009CB40(f, i, 1, core::ptr::null_mut());
        ext::lb_80011710(&mut dyn_bones(dy).array[i as usize].dyn_desc, &mut f.dynamic_bone_sets[i as usize].dyn_desc);
        i += 1;
    }
}

/// Pre-order walk `n` steps from `cur` (child first, else next, climbing parents).
fn walk(mut cur: *mut HSD_JObj, n: i32) -> *mut HSD_JObj {
    let mut j = 0;
    while j < n {
        unsafe {
            if !(*cur).child.is_null() {
                cur = (*cur).child;
            } else {
                while (*cur).next.is_null() {
                    cur = (*cur).parent;
                }
                cur = (*cur).next;
            }
        }
        j += 1;
    }
    cur
}

/// `ftCo_SetupKirbyHatBone` (static inline).
fn setup_kirby_hat_bone(f: &mut Fighter, hat: *mut KirbyHatStruct, dyn_idx: usize, i: isize) {
    let hd = unsafe { (*hat).hat_dynamics[dyn_idx] };
    let article = &mut dyn_bones(hd).array[i as usize];
    let cur = walk(unsafe { f.u.kb.hat.jobj }, article.bone_id);
    ext::lb_8000FD48(cur, &mut f.dynamic_bone_sets[i as usize].dyn_desc, article.dyn_desc.count as usize);
    f.dynamic_bone_sets[i as usize].bone_id = 0;
    ext::lb_80011710(&mut dyn_bones(hd).array[i as usize].dyn_desc, &mut f.dynamic_bone_sets[i as usize].dyn_desc);
}

fn kirby_hat(f: &mut Fighter, kind: FighterKind, dyn_idx: usize, line: u32) {
    let hat = ext::ft_80459B88_hat(kind as i32);
    if hat.is_null() {
        return;
    }
    f.dynamics_num = unsafe { (*(*hat).hat_dynamics[dyn_idx]).anon_0.dynamicsNum };
    num_over(f, line);
    let mut i: isize = 0;
    while i < f.dynamics_num as isize {
        setup_kirby_hat_bone(f, hat, dyn_idx, i);
        i += 1;
    }
}

/// The Yoshi / Purin hat form: bones on the fighter's own parts, bone_id = FtPart_TopN.
fn kirby_hat_parts(f: &mut Fighter, kind: FighterKind, dyn_idx: usize, line: u32) {
    let hat = ext::ft_80459B88_hat(kind as i32);
    if hat.is_null() {
        return;
    }
    let hd = unsafe { (*hat).hat_dynamics[dyn_idx] };
    f.dynamics_num = unsafe { (*hd).anon_0.dynamicsNum };
    num_over(f, line);
    let mut i: isize = 0;
    while i < f.dynamics_num as isize {
        let bone_id = dyn_bones(hd).array[i as usize].bone_id;
        set_pflag(part(f, bone_id), 1);
        ext::lb_8000FD48(part(f, bone_id).joint, &mut f.dynamic_bone_sets[i as usize].dyn_desc,
                         dyn_bones(hd).array[i as usize].dyn_desc.count as usize);
        f.dynamic_bone_sets[i as usize].bone_id = FtPart_TopN as i32;
        ext::lb_80011710(&mut dyn_bones(hd).array[i as usize].dyn_desc, &mut f.dynamic_bone_sets[i as usize].dyn_desc);
        i += 1;
    }
}

pub fn ftCo_8009D074(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Koopa, 2, 135) }
pub fn ftCo_8009D18C(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Zelda, 2, 167) }
pub fn ftCo_8009D2A4(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Nana, 2, 199) }
pub fn ftCo_8009D3BC(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Falco, 2, 232) }
pub fn ftCo_8009D4D4(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Kirby, 1, 265) }
pub fn ftCo_8009D5EC(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Mars, 0, 298) }
pub fn ftCo_8009D704(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Link, 2, 331) }
pub fn ftCo_8009D81C(f: &mut Fighter) { kirby_hat_parts(f, Ft_Kind_Yoshi, 3, 364) }
pub fn ftCo_8009D920(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Luigi, 1, 388) }
pub fn ftCo_8009DA38(f: &mut Fighter) { kirby_hat(f, Ft_Kind_Ganon, 1, 421) }
pub fn ftCo_8009DB50(f: &mut Fighter) { kirby_hat_parts(f, Ft_Kind_Purin, 4, 455) }

/// `ftCo_8009DC54`: Jigglypuff costume 2/3 accessory dynamics (sets 1..2).
pub fn ftCo_8009DC54(f: &mut Fighter) {
    let idx: isize = match f.x619_costume_id {
        2 => 0,
        3 => 1,
        _ => return,
    };
    let data = f.ft_data;
    f.dynamics_num = 3;
    num_over(f, 490);
    let mut bone_idx = idx * 2 + 1;
    let mut dyn_idx: isize = 0;
    let mut i: isize = 0;
    while i < 2 {
        let dy = unsafe { (*data).x2C };
        let cur = walk(unsafe { f.u.pr.x223C }, dyn_bones(dy).array[bone_idx as usize].bone_id);
        let desc: *mut DynamicsDesc = &mut f.dynamic_bone_sets[(i + 1) as usize].dyn_desc;
        ext::lb_8000FD48(cur, desc, dyn_bones(dy).array[bone_idx as usize].dyn_desc.count as usize);
        f.dynamic_bone_sets[(dyn_idx + 1) as usize].bone_id = FtPart_TopN as i32;
        ext::lb_80011710(&mut dyn_bones(unsafe { (*data).x2C }).array[bone_idx as usize].dyn_desc, desc);
        i += 1;
        bone_idx += 1;
        dyn_idx += 1;
    }
}

/// `ftCo_8009DD94` (static): step the dynamics for one frame.
pub fn ftCo_8009DD94(g: G, arg1: i32) {
    let f = fp(g);
    let n = f.dynamics_num as isize;
    let mut scale_x = 0.0f32;
    let flat = n != 0 && ext::stage_info_grkind() == Gr_Kind_Flatzone as i32;
    if flat {
        let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
        unsafe {
            scale_x = (*j).scale.x;
            (*j).scale.x = f.x34_scale.y;
        }
        set_mtx_dirty(j);
    }
    let ret_b0: i32 = if f.x34_scale.y < 1.0 || f.x34_scale.y > 1.0 {
        0
    } else {
        let kind = f.kind as i32;
        if kind == Ft_Kind_Mewtwo as i32 && f.motion_id as i32 >= 0xDF && f.motion_id as i32 <= 0xE8 {
            1
        } else if f.ground_or_air as i32 == GA_Air as i32 || kind == Ft_Kind_Koopa as i32 || kind == Ft_Kind_GKoops as i32 {
            0
        } else {
            1
        }
    };
    ext::ftColl_8007AF60(g);
    ftCo_8009E614(fp(g));
    let f = fp(g);
    let x1670: *mut Fighter_x1670_t = f.x1670.as_mut_ptr();
    let call = |i: usize, mode: i32| {
        let f = fp(g);
        let bone = f.dynamic_bone_sets[i].bone_id;
        ext::lb_8001044C(&mut f.dynamic_bone_sets[i].dyn_desc, x1670, f.x166C as i32, f.cur_pos.y, arg1, bone, mode, ret_b0);
    };
    if f.kind as i32 == Ft_Kind_Peach as i32 {
        let mut i: isize = 0;
        while i < n - 1 {
            call(i as usize, 3);
            i += 1;
        }
        call(i as usize, 0);
    } else {
        let mode = if f.kind as i32 == Ft_Kind_Mewtwo as i32 { 8 } else { 0 };
        for i in 0..n.max(0) as usize {
            call(i, mode);
        }
    }
    if flat {
        let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
        unsafe { (*j).scale.x = scale_x };
        set_mtx_dirty(j);
        let f = fp(g);
        for i in 0..n.max(0) as usize {
            let mut cur = unsafe { (*f.dynamic_bone_sets[i].dyn_desc.data).desc.lb_unk0.jobj };
            unsafe {
                while !(*cur).child.is_null() {
                    cur = (*cur).child;
                }
            }
            setup_matrix(cur);
        }
    }
}

/// `ftCo_8009E0A8`.
pub fn ftCo_8009E0A8(g: G) {
    let b = fp(g).x2228_b1() as i32;
    ftCo_8009DD94(g, b);
}

/// `ftCo_UnloadDynamicBones`.
pub fn ftCo_UnloadDynamicBones(f: &mut Fighter) {
    let mut i = 0;
    while i < f.dynamics_num {
        ext::lb_8000FD18(&mut f.dynamic_bone_sets[i as usize].dyn_desc);
        i += 1;
    }
    f.dynamics_num = 0;
}

/// The repeated "all sets (Purin: set 0 only) to `flag`" tail; skipped for Kirby.
fn set_all(f: &mut Fighter, flag: i32) {
    if is_kirby(f.kind as i32) {
        return;
    }
    if template_kind(f) == Ft_Kind_Purin as i32 {
        ftCo_8009CB40(f, 0, flag, core::ptr::null_mut());
        return;
    }
    let mut i: isize = 0;
    while i < f.dynamics_num as isize {
        ftCo_8009CB40(f, i, flag, core::ptr::null_mut());
        i += 1;
    }
}
fn set_each(f: &mut Fighter, flag: i32) {
    let mut i: isize = 0;
    while i < f.dynamics_num as isize {
        ftCo_8009CB40(f, i, flag, core::ptr::null_mut());
        i += 1;
    }
}

/// `ftCo_8009E140`.
pub fn ftCo_8009E140(f: &mut Fighter, arg1: i32) {
    set_all(f, arg1);
}

/// `ftCo_8009E1D4` (static): find `arg2` in tree `arg0`, return the node at the same place in `arg1`.
pub fn ftCo_8009E1D4(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj) -> *mut HSD_JObj {
    if a0 == a2 {
        return a1;
    }
    unsafe {
        if !(*a0).child.is_null() {
            let t = ftCo_8009E1D4((*a0).child, (*a1).child, a2);
            if !t.is_null() {
                return t;
            }
        }
        if !(*a0).next.is_null() {
            let t = ftCo_8009E1D4((*a0).next, (*a1).next, a2);
            if !t.is_null() {
                return t;
            }
        }
    }
    core::ptr::null_mut()
}

/// `ftCo_8009E318`: toggle whether `part` is driven by dynamics; -1 if no set holds it.
pub fn ftCo_8009E318(g: G, arg1: i32, arg2: f32) -> i32 {
    let f = fp(g);
    let pj = part(f, arg1).joint;
    let mut i = 0usize;
    while (i as i32) < f.dynamics_num {
        let mut cur = f.dynamic_bone_sets[i].dyn_desc.data;
        let mut j: i32 = 0;
        while !cur.is_null() {
            if unsafe { (*cur).desc.lb_unk0.jobj } == pj {
                let b = part(f, arg1);
                if pflag(b) != 0 {
                    set_pflag(b, 0);
                    f.dynamic_bone_sets[i].bone_id = j + 1;
                    if !f.x590.is_null() {
                        let (t, s) = (f.x590, f.frame_speed_mul);
                        ext::ftAnim_8006EED4(f, arg1, t, arg2, s);
                    }
                } else {
                    set_pflag(b, 1);
                    f.dynamic_bone_sets[i].bone_id = j;
                    let w = ftCo_8009E1D4(unsafe { (*g).hsd_obj } as *mut HSD_JObj, f.x8AC_animSkeleton, pj);
                    ext::ftAnim_80070758(pj);
                    ext::ftAnim_80070758(w);
                }
                return 0;
            }
            cur = unsafe { (*cur).next };
            j += 1;
        }
        i += 1;
    }
    -1
}

fn x594_b4(f: &Fighter) -> bool { unsafe { f.anon_594.anon_0.x594_b4() != 0 } }
fn x594_b3(f: &Fighter) -> bool { unsafe { f.anon_594.anon_0.x594_b3() != 0 } }

fn reanim(f: &mut Fighter, i: usize, frame: f32, speed: f32) {
    if !f.x590.is_null() {
        let bone = unsafe { dyn_bones((*f.ft_data).x2C).array[i].bone_id };
        let t = f.x590;
        ext::ftAnim_8006EED4(f, bone, t, frame, speed);
    }
}

/// `ftCo_8009E4A8` (static).
pub fn ftCo_8009E4A8(f: &mut Fighter) {
    let frame = f.cur_anim_frame;
    let speed = f.frame_speed_mul;
    if x594_b4(f) {
        let tree = unsafe {
            let slot = (*f.x28.offset(f.anim_id as isize))[1];
            *(*(*f.ft_data).x2C).x10.offset(slot as isize)
        };
        if !tree.is_null() {
            let mut i = 0;
            while i < f.dynamics_num {
                ftCo_8009CB40(f, i as isize, 1, unsafe { *tree.offset(i as isize) });
                reanim(f, i as usize, frame, speed);
                i += 1;
            }
        }
    } else if x594_b3(f) {
        let mut i = 0;
        while i < f.dynamics_num {
            ftCo_8009CB40(f, i as isize, 0, core::ptr::null_mut());
            reanim(f, i as usize, frame, speed);
            i += 1;
        }
    }
}

fn is_marth_like(f: &Fighter) -> bool {
    template_kind(f) == Ft_Kind_Mars as i32 || template_kind(f) == Ft_Kind_Emblem as i32
}

/// `ftCo_8009E614` (static): Marth / Roy sword-trail dynamics vs lb_80011ABC's mode.
pub fn ftCo_8009E614(f: &mut Fighter) {
    if is_marth_like(f) && f.x2227_b6() == 0 {
        let x = ext::lb_80011ABC();
        if x == -1 {
            ftCo_8009E4A8(f);
            return;
        }
        if x == 1 {
            let data = f.ft_data;
            let frame = f.cur_anim_frame;
            let speed = f.frame_speed_mul;
            if x594_b4(f) || x594_b3(f) {
                let mut i: isize = 0;
                while i < unsafe { (*(*data).x2C).anon_0.dynamicsNum } as isize {
                    ftCo_8009CB40(f, i, 1, core::ptr::null_mut());
                    i += 1;
                }
                if !f.x590.is_null() {
                    ext::ftAnim_8006EBE8(f.gobj, frame, speed, 0.0);
                    ext::ftAnim_8006E9B4(f.gobj);
                }
            }
        }
    }
}

/// `ftCo_8009E714`: spawn a dynamics impulse at a point in `bone_id`'s space. Returns false.
pub fn ftCo_8009E714(g: G, bone_id: i32, arg2: i32, x: f32, y: f32, a5: f32, a6: f32, a7: f32) -> bool {
    let j = part(fp(g), bone_id).joint;
    let mut out = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut inv = Vec3 { x, y, z: 0.0 };
    ext::lb_8000B1CC(j, &mut inv, &mut out);
    let n = if arg2 < 0 { 120 } else { arg2 };
    ext::lb_800119DC(&mut out, n, a5, a6, a7);
    false
}

/// `ftCo_8009E7B4`: re-evaluate which sets are animation- vs physics-driven after a motion change.
/// `arg1` is the motion's `[u8; 2]` pair (only `[0][1]` is read).
pub fn ftCo_8009E7B4(f: &mut Fighter, arg1: *const [u8; 2]) {
    if f.anim_id != -1 {
        let b6 = f.x2227_b6();
        if b6 != 0 {
            if !is_kirby(f.kind as i32) {
                if template_kind(f) == Ft_Kind_Purin as i32 {
                    ftCo_8009CB40(f, 0, 0, core::ptr::null_mut());
                    return;
                }
                set_each(f, 0);
            }
            return;
        }
        let r3 = if !is_marth_like(f) { false } else { ext::lb_80011ABC() > 0 };
        if r3 {
            set_all(f, 1);
        } else if x594_b4(f) {
            let blend_slot = unsafe { (*arg1)[1] };
            let dy = unsafe { (*(*f.ft_data).x2C).x10 };
            if dy.is_null() {
                set_each(f, 0);
                return;
            }
            let tree = unsafe { *dy.offset(blend_slot as isize) };
            if tree.is_null() {
                set_each(f, 0);
                return;
            }
            let mut j = 0;
            while j < f.dynamics_num {
                ftCo_8009CB40(f, j as isize, 1, unsafe { *tree.offset(j as isize) });
                j += 1;
            }
        } else if x594_b3(f) {
            set_all(f, 0);
        } else {
            set_all(f, 1);
        }
    } else if f.x2227_b6() != 0 && !is_kirby(f.kind as i32) {
        if template_kind(f) == Ft_Kind_Purin as i32 {
            ftCo_8009CB40(f, 0, 0, core::ptr::null_mut());
            return;
        }
        set_each(f, 0);
    }
}

/// `ftCo_8009EAF8`.
pub fn ftCo_8009EAF8(_g: G) {
    ext::Ground_801C4E20();
}

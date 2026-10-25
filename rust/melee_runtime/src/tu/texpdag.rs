//! `texpdag.c`: TEV dependency scheduling and expression simplification.
//!
//! Pointer arguments retain the C graph's aliasing. Callers provide live generated records,
//! acyclic TEV graphs of at most 32 stages, and arrays large enough for `num` entries.
//! This integer-only TU has no fused sites; dependency shifts follow gwtool's zero-on-overshift rule.
#![allow(non_snake_case)]
use core::ptr::null_mut;
pub use melee_types::inline::{IsThroughAlpha, IsThroughColor};
use melee_types::{HSD_TExp, HSD_TExpDag, HSD_TExpRes, _HSD_TEArg};

/// `texp.c` operations used by the DAG compiler. The host preserves reference-count side effects.
pub trait TExpDagHost {
    unsafe fn HSD_TExpGetType(&mut self, exp: *mut HSD_TExp) -> u32;
    unsafe fn HSD_TExpRef(&mut self, exp: *mut HSD_TExp, sel: u8);
    unsafe fn HSD_TExpUnref(&mut self, exp: *mut HSD_TExp, sel: u8);
}
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_TExpGetType(a0: *mut HSD_TExp) -> u32;
    fn HSD_TExpRef(a0: *mut HSD_TExp, a1: u8);
    fn HSD_TExpUnref(a0: *mut HSD_TExp, a1: u8);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_TExpGetType(a0: *mut HSD_TExp) -> u32 { super::HSD_TExpGetType(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// The `texp.c` host of the game: the generated hooks (`HSD_TExpGetType` / `HSD_TExpRef` / `HSD_TExpUnref`).
pub struct Ext;
impl TExpDagHost for Ext {
    unsafe fn HSD_TExpGetType(&mut self, exp: *mut HSD_TExp) -> u32 {
        ext::HSD_TExpGetType(exp) as u32
    }
    unsafe fn HSD_TExpRef(&mut self, exp: *mut HSD_TExp, sel: u8) {
        ext::HSD_TExpRef(exp, sel)
    }
    unsafe fn HSD_TExpUnref(&mut self, exp: *mut HSD_TExp, sel: u8) {
        ext::HSD_TExpUnref(exp, sel)
    }
}

fn bit(i: u32) -> u32 {
    1u32.checked_shl(i).unwrap_or(0)
}

/// `assign_reg`: allocate the highest free color/alpha register after consuming each input.
pub unsafe fn assign_reg(
    h: &mut dyn TExpDagHost,
    num: i32,
    _: *mut u32,
    list: *mut HSD_TExpDag,
    order: *const i32,
) -> i32 {
    let mut color_refs = [0u8; 4];
    let mut alpha_refs = [0u8; 4];
    let (mut min_color, mut min_alpha) = (4, 4);
    for idx in (0..num).rev() {
        let dag = list.offset(*order.offset(idx as isize) as isize);
        let tev = (*dag).tev;
        for i in 0..4 {
            let c = (*tev).c_in[i];
            if h.HSD_TExpGetType(c.exp) == 1 {
                let refs = if c.sel == 1 {
                    &mut color_refs[(*c.exp).tev.c_dst as usize]
                } else {
                    &mut alpha_refs[(*c.exp).tev.a_dst as usize]
                };
                *refs = refs.wrapping_sub(1);
            }
            let a = (*tev).a_in[i];
            if h.HSD_TExpGetType(a.exp) == 1 {
                let refs = &mut alpha_refs[(*a.exp).tev.a_dst as usize];
                *refs = refs.wrapping_sub(1);
            }
        }
        let tev = (*dag).tev;
        if (*tev).c_ref > 0 {
            for i in (0..4).rev() {
                if color_refs[i] == 0 {
                    color_refs[i] = (*tev).c_ref as u8;
                    (*tev).c_dst = i as u8;
                    min_color = min_color.min(i as i32);
                    break;
                }
            }
        }
        if (*tev).a_ref > 0 {
            for i in (0..4).rev() {
                if alpha_refs[i] == 0 {
                    alpha_refs[i] = (*tev).a_ref as u8;
                    (*tev).a_dst = i as u8;
                    min_alpha = min_alpha.min(i as i32);
                    break;
                }
            }
        }
    }
    (4 - min_color) + (4 - min_alpha)
}

/// `order_dag`: enumerate ready stages and retain the first minimum-register order.
#[allow(clippy::too_many_arguments)]
pub unsafe fn order_dag(
    h: &mut dyn TExpDagHost,
    num: i32,
    dep: *mut u32,
    full_dep: *mut u32,
    list: *mut HSD_TExpDag,
    depth: i32,
    idx: i32,
    mut done_set: u32,
    mut ready_set: u32,
    order: *mut i32,
    min: *mut i32,
    min_order: *mut i32,
) {
    done_set |= bit(idx as u32);
    ready_set &= !bit(idx as u32);
    *order.offset(depth as isize) = idx as u8 as i32;
    let depth = depth + 1;
    if depth == num {
        let score = assign_reg(h, num, dep, list, order);
        if score < *min {
            *min = score;
            for i in 0..num {
                *min_order.offset(i as isize) = *order.offset(i as isize);
            }
        }
    } else {
        let dep_bits = *dep.offset(idx as isize);
        let blocked = ready_set | dep_bits;
        let mut full_bits = 0;
        for i in 0..num {
            if blocked & bit(i as u32) != 0 {
                full_bits |= *full_dep.offset(i as isize);
            }
        }
        let dag = list.offset(idx as isize);
        ready_set = blocked & !full_bits;
        if (*dag).nb_dep == 1 && ready_set & dep_bits != 0 {
            order_dag(
                h,
                num,
                dep,
                full_dep,
                list,
                depth,
                (*(*dag).depend[0]).idx as i32,
                done_set,
                ready_set,
                order,
                min,
                min_order,
            );
        } else {
            for i in 0..num {
                if ready_set & bit(i as u32) != 0 {
                    order_dag(
                        h, num, dep, full_dep, list, depth, i, done_set,
                        ready_set, order, min, min_order,
                    );
                }
            }
        }
    }
}

/// `CalcDistance`: propagate only an increased root-to-stage depth.
pub unsafe fn CalcDistance(
    tevs: *mut *mut HSD_TExp,
    dist: *mut i32,
    tev: *mut HSD_TExp,
    num: i32,
    depth: i32,
) {
    for idx in 0..num {
        if *tevs.offset(idx as isize) == tev {
            if *dist.offset(idx as isize) < depth {
                *dist.offset(idx as isize) = depth;
                for i in 0..4 {
                    if (*tev).tev.c_in[i].r#type == 1 {
                        CalcDistance(
                            tevs,
                            dist,
                            (*tev).tev.c_in[i].exp,
                            num,
                            depth.wrapping_add(1),
                        );
                    }
                    if (*tev).tev.a_in[i].r#type == 1 {
                        CalcDistance(
                            tevs,
                            dist,
                            (*tev).tev.a_in[i].exp,
                            num,
                            depth.wrapping_add(1),
                        );
                    }
                }
            }
            return;
        }
    }
}

/// `HSD_TExpMakeDag`: discover color inputs before alpha inputs and coalesce repeated edges.
pub unsafe fn HSD_TExpMakeDag(
    h: &mut dyn TExpDagHost,
    root: *mut HSD_TExp,
    list: *mut HSD_TExpDag,
) -> i32 {
    assert_eq!(h.HSD_TExpGetType(root), 1, "texpdag.c:238: TEV root");
    let mut tevs = [null_mut(); 32];
    let mut dist = [0; 32];
    tevs[0] = root;
    let mut n = 1;
    let mut j = 0;
    while j < n {
        assert!(j < 32, "texpdag.c:246: stage limit");
        let tmp = tevs[j];
        for alpha in [false, true] {
            for i in 0..4 {
                let arg = if alpha {
                    (*tmp).tev.a_in[i]
                } else {
                    (*tmp).tev.c_in[i]
                };
                if arg.r#type == 1 && !tevs[..n].contains(&arg.exp) {
                    tevs[n] = arg.exp;
                    n += 1;
                }
            }
        }
        j += 1;
    }
    dist[..n].fill(-1);
    CalcDistance(tevs.as_mut_ptr(), dist.as_mut_ptr(), root, n as i32, 0);
    // Retain the C's exact adjacent-swap traversal rather than substituting a different sort.
    for idx in 0..n {
        for j in idx + 1..n {
            if dist[j - 1] > dist[j] {
                tevs.swap(j - 1, j);
                dist.swap(j - 1, j);
            }
        }
    }
    for j in (0..n).rev() {
        let dag = list.add(j);
        let tmp = tevs[j];
        (*dag).idx = j as u8;
        (*dag).nb_ref = 0;
        (*dag).nb_dep = 0;
        (*dag).tev = core::ptr::addr_of_mut!((*tmp).tev);
        for alpha in [false, true] {
            for i in 0..4 {
                let arg = if alpha {
                    (*tmp).tev.a_in[i]
                } else {
                    (*tmp).tev.c_in[i]
                };
                if arg.r#type == 1 {
                    let l = (j..n)
                        .find(|&l| tevs[l] == arg.exp)
                        .expect("texpdag.c:325/347: dependency");
                    let dep = list.add(l);
                    if !(0..(*dag).nb_dep as usize)
                        .any(|k| (*dag).depend[k] == dep)
                    {
                        (*dag).depend[(*dag).nb_dep as usize] = dep;
                        (*dag).nb_dep = (*dag).nb_dep.wrapping_add(1);
                        (*dep).nb_ref = (*dep).nb_ref.wrapping_add(1);
                    }
                }
            }
        }
    }
    n as i32
}

/// C-static `make_dependancy_mtx`, exposed so the complete TU oracle can exercise it directly.
pub unsafe fn make_dependancy_mtx(
    num: i32,
    list: *mut HSD_TExpDag,
    dep_mtx: *mut u32,
) {
    for i in 0..num {
        *dep_mtx.offset(i as isize) = 0;
        let dag = list.offset(i as isize);
        for j in 0..(*dag).nb_dep as usize {
            *dep_mtx.offset(i as isize) |= bit((*(*dag).depend[j]).idx as u32);
        }
    }
}

pub unsafe fn make_full_dependancy_mtx(
    num: i32,
    dep: *const u32,
    full: *mut u32,
) {
    for i in 0..num {
        *full.offset(i as isize) = *dep.offset(i as isize);
    }
    loop {
        let mut changed = false;
        for j in 0..num {
            let flag = bit(j as u32);
            let bits = *full.offset(j as isize);
            for k in 0..num {
                let p = full.offset(k as isize);
                if flag & *p != 0 {
                    let old = *p;
                    *p |= bits;
                    changed |= old != *p;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

pub unsafe fn HSD_TExpSchedule(
    h: &mut dyn TExpDagHost,
    num: i32,
    list: *mut HSD_TExpDag,
    result: *mut *mut HSD_TExp,
    resource: *mut HSD_TExpRes,
) {
    use melee_types::{
        GX_CA_A0, GX_CA_A1, GX_CA_A2, GX_CA_APREV, GX_CC_A0, GX_CC_A1,
        GX_CC_A2, GX_CC_APREV, GX_CC_C0, GX_CC_C1, GX_CC_C2, GX_CC_CPREV,
    };
    let c_in = [GX_CC_C0, GX_CC_C1, GX_CC_C2, GX_CC_CPREV];
    let a_in = [GX_CC_A0, GX_CC_A1, GX_CC_A2, GX_CC_APREV];
    let args = [GX_CA_A0, GX_CA_A1, GX_CA_A2, GX_CA_APREV, 0];
    let (mut dep, mut full, mut order, mut best) =
        ([0u32; 32], [0u32; 32], [0i32; 32], [0i32; 32]);
    let mut min = 5;
    make_dependancy_mtx(num, list, dep.as_mut_ptr());
    make_full_dependancy_mtx(num, dep.as_ptr(), full.as_mut_ptr());
    order_dag(
        h,
        num,
        dep.as_mut_ptr(),
        full.as_mut_ptr(),
        list,
        0,
        0,
        0,
        0,
        order.as_mut_ptr(),
        &mut min,
        best.as_mut_ptr(),
    );
    for i in 0..num as usize {
        let exp = (*list.offset(best[i] as isize)).tev.cast::<HSD_TExp>();
        *result.add(i) = exp;
        if (*exp).tev.c_dst != 255 {
            (*resource).reg[(*exp).tev.c_dst as usize + 4].color = 3;
            for j in 0..4 {
                let a = (*exp).tev.c_in[j];
                if h.HSD_TExpGetType(a.exp) == 1 {
                    // The alpha-selected color path deliberately indexes c_dst, as in texpdag.c.
                    (*exp).tev.c_in[j].arg = if a.sel == 1 {
                        c_in[(*a.exp).tev.c_dst as usize]
                    } else {
                        a_in[(*a.exp).tev.c_dst as usize]
                    } as u8;
                }
            }
        }
        if (*exp).tev.a_dst != 255 {
            (*resource).reg[(*exp).tev.a_dst as usize + 4].alpha = 1;
            for j in 0..4 {
                let a = (*exp).tev.a_in[j];
                if h.HSD_TExpGetType(a.exp) == 1 {
                    (*exp).tev.a_in[j].arg =
                        args[(*a.exp).tev.a_dst as usize] as u8;
                }
            }
        }
    }
}

const CLEAR_ARG: _HSD_TEArg = _HSD_TEArg {
    r#type: 0,
    sel: 7,
    arg: 255,
    exp: null_mut(),
};
pub unsafe fn SimplifySrc(h: &mut dyn TExpDagHost, exp: *mut HSD_TExp) -> i32 {
    let mut result = false;
    for i in 0..4 {
        if (*exp).tev.c_in[i].r#type != 1 {
            continue;
        }
        let src = (*exp).tev.c_in[i].exp;
        let sel = (*exp).tev.c_in[i].sel;
        if HSD_TExpSimplify(h, src) != 0 {
            result = true;
        }
        if sel == 1 {
            match (*src).tev.c_op {
                255 => {
                    h.HSD_TExpUnref(src, sel);
                    result = true;
                    (*exp).tev.c_in[i] = CLEAR_ARG;
                }
                0 if IsThroughColor(&*src) => {
                    let a = (*src).tev.c_in[3];
                    match a.r#type {
                        1 if (*a.exp).tev.c_clamp != 0
                            || (*src).tev.c_clamp == 0 =>
                        {
                            (*exp).tev.c_in[i] = a;
                            h.HSD_TExpRef(a.exp, a.sel);
                            h.HSD_TExpUnref(src, sel);
                            result = true;
                        }
                        2 if ((*exp).tev.tex.is_null()
                            || (*exp).tev.tex == (*src).tev.tex)
                            && ((*exp).tev.tex_swap == 255
                                || (*src).tev.tex_swap == 255
                                || (*exp).tev.tex_swap
                                    == (*src).tev.tex_swap) =>
                        {
                            (*exp).tev.c_in[i] = a;
                            (*exp).tev.tex = (*src).tev.tex;
                            if (*exp).tev.tex_swap == 255 {
                                (*exp).tev.tex_swap = (*src).tev.tex_swap;
                            }
                            h.HSD_TExpUnref(src, sel);
                            result = true;
                        }
                        3 if ((*exp).tev.chan == 255
                            || (*exp).tev.chan == (*src).tev.chan)
                            && ((*exp).tev.ras_swap == 255
                                || (*src).tev.ras_swap == 255
                                || (*exp).tev.ras_swap
                                    == (*src).tev.tex_swap) =>
                        {
                            // Preserve the C's tex_swap comparison/copy in the raster path.
                            (*exp).tev.c_in[i] = a;
                            (*exp).tev.chan = (*src).tev.chan;
                            if (*exp).tev.tex_swap == 255 {
                                (*exp).tev.tex_swap = (*src).tev.tex_swap;
                            }
                            h.HSD_TExpUnref(src, sel);
                            result = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        } else if (*src).tev.a_op == 255 {
            h.HSD_TExpUnref(src, sel);
            result = true;
            (*exp).tev.c_in[i] = CLEAR_ARG;
        }
    }
    for i in 0..4 {
        if (*exp).tev.a_in[i].r#type != 1 {
            continue;
        }
        let src = (*exp).tev.a_in[i].exp;
        let sel = (*exp).tev.a_in[i].sel;
        // Unlike color inputs, this recursive result does not contribute to the return value.
        HSD_TExpSimplify(h, src);
        match (*src).tev.a_op {
            255 => {
                h.HSD_TExpUnref(src, sel);
                result = true;
                (*exp).tev.a_in[i] = CLEAR_ARG;
            }
            0 if IsThroughAlpha(&*src) => {
                let a = (*src).tev.a_in[3];
                match a.r#type {
                    1 => {
                        (*exp).tev.a_in[i] = a;
                        h.HSD_TExpRef(a.exp, a.sel);
                        h.HSD_TExpUnref(src, sel);
                        result = true;
                    }
                    2 if (*exp).tev.tex.is_null()
                        || (*exp).tev.tex == (*src).tev.tex =>
                    {
                        (*exp).tev.a_in[i] = a;
                        (*exp).tev.tex = (*src).tev.tex;
                        h.HSD_TExpUnref(src, sel);
                        result = true;
                    }
                    3 if (*exp).tev.chan == 255
                        || (*exp).tev.chan == (*src).tev.chan =>
                    {
                        (*exp).tev.a_in[i] = a;
                        (*exp).tev.chan = (*src).tev.chan;
                        h.HSD_TExpUnref(src, sel);
                        result = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    result as i32
}

unsafe fn unref_clear(h: &mut dyn TExpDagHost, a: *mut _HSD_TEArg) {
    h.HSD_TExpUnref((*a).exp, (*a).sel);
    *a = CLEAR_ARG;
}

pub unsafe fn SimplifyThis(
    h: &mut dyn TExpDagHost,
    exp: *mut HSD_TExp,
) -> i32 {
    let mut result = 0;
    let mut passes = 0u32;
    loop {
        let mut has_tex = false;
        let mut has_ras = false;
        for i in 0..4 {
            has_tex |= (*exp).tev.c_in[i].r#type == 2
                || (*exp).tev.a_in[i].r#type == 2;
            has_ras |= (*exp).tev.c_in[i].r#type == 3
                || (*exp).tev.a_in[i].r#type == 3;
        }
        if !has_tex {
            (*exp).tev.tex = null_mut();
            (*exp).tev.tex_swap = 255;
        }
        if !has_ras {
            (*exp).tev.chan = 255;
            (*exp).tev.ras_swap = 255;
        }
        let mut changed = false;
        for alpha in [false, true] {
            if !alpha && !matches!((*exp).tev.a_op, 0 | 1 | 14 | 15 | 255) {
                continue;
            }
            let (op, inputs, refs) = if alpha {
                (
                    core::ptr::addr_of_mut!((*exp).tev.a_op),
                    core::ptr::addr_of_mut!((*exp).tev.a_in)
                        .cast::<_HSD_TEArg>(),
                    (*exp).tev.a_ref,
                )
            } else {
                (
                    core::ptr::addr_of_mut!((*exp).tev.c_op),
                    core::ptr::addr_of_mut!((*exp).tev.c_in)
                        .cast::<_HSD_TEArg>(),
                    (*exp).tev.c_ref,
                )
            };
            if *op != 255 && refs == 0 {
                *op = 255;
                for i in 0..4 {
                    unref_clear(h, inputs.add(i));
                }
                changed = true;
            }
            match *op {
                0 | 1 => {
                    if (*inputs.add(2)).sel == 7 {
                        if (*inputs.add(1)).sel != 7 {
                            unref_clear(h, inputs.add(1));
                            changed = true;
                        }
                        if *op == 0 && (*inputs.add(3)).sel == 7 {
                            changed = true;
                            *inputs.add(3) = *inputs;
                            *inputs = CLEAR_ARG;
                            if !alpha {
                                (*exp).tev.c_clamp = 1;
                            }
                        }
                    }
                    if (*inputs.add(2)).sel == 8 {
                        if (*inputs).sel != 7 {
                            unref_clear(h, inputs);
                            changed = true;
                        }
                        if *op == 0 && (*inputs.add(3)).sel == 7 {
                            changed = true;
                            *inputs.add(3) = *inputs.add(1);
                            *inputs.add(1) = CLEAR_ARG;
                            *inputs.add(2) = CLEAR_ARG;
                        }
                    }
                    if (*inputs).sel == 7
                        && (*inputs.add(1)).sel == 7
                        && (*inputs.add(3)).sel == 7
                        && (alpha || (*exp).tev.c_bias == 0)
                    {
                        *op = 255;
                        if !alpha {
                            unref_clear(h, inputs.add(2));
                        }
                        changed = true;
                    }
                }
                8..=15 => {
                    if (*inputs.add(2)).sel == 7 {
                        *op = 0;
                        unref_clear(h, inputs);
                        unref_clear(h, inputs.add(1));
                        changed = true;
                    } else if ((!alpha && *op % 2 == 0)
                        || (alpha && *op == 14))
                        && (*inputs).sel == 7
                    {
                        *op = 0;
                        unref_clear(h, inputs.add(1));
                        unref_clear(h, inputs.add(2));
                        changed = true;
                    } else if ((!alpha && *op % 2 == 1)
                        || (alpha && *op == 15))
                        && (*inputs).sel == 7
                        && (*inputs.add(1)).sel == 7
                    {
                        *op = 0;
                        changed = true;
                        *inputs = *inputs.add(2);
                        *inputs.add(2) = CLEAR_ARG;
                    }
                }
                _ => {}
            }
        }
        if !changed {
            break;
        }
        passes += 1;
        assert!(passes <= 100, "texpdag.c: simplify loop does not converge");
        result = 1;
    }
    result
}

fn bias_to_int(bias: u8) -> i32 {
    match bias {
        1 => 1,
        2 => -1,
        _ => 0,
    }
}
fn int_to_bias(bias: i32) -> u8 {
    match bias {
        1 => 1,
        -1 => 2,
        _ => 0,
    }
}

unsafe fn resource_conflict(exp: *mut HSD_TExp, child: *mut HSD_TExp) -> bool {
    (!(*exp).tev.tex.is_null()
        && !(*child).tev.tex.is_null()
        && (*exp).tev.tex != (*child).tev.tex)
        || ((*exp).tev.chan != 255
            && (*child).tev.chan != 255
            && (*exp).tev.chan != (*child).tev.chan)
}

unsafe fn inherit_resources(
    exp: *mut HSD_TExp,
    child: *mut HSD_TExp,
    alpha: bool,
) {
    if (*exp).tev.tex.is_null() {
        (*exp).tev.tex = (*child).tev.tex;
    }
    if (*exp).tev.chan == 255 {
        (*exp).tev.chan = (*child).tev.chan;
    }
    if !alpha {
        if (*exp).tev.tex_swap == 255 {
            (*exp).tev.tex_swap = (*child).tev.tex_swap;
        }
        if (*exp).tev.ras_swap == 255 {
            (*exp).tev.ras_swap = (*child).tev.ras_swap;
        }
    }
}

pub unsafe fn SimplifyByMerge(
    h: &mut dyn TExpDagHost,
    exp: *mut HSD_TExp,
) -> i32 {
    let mut result = 0;
    let mut passes = 0u32;
    loop {
        let mut merged = false;
        if matches!((*exp).tev.a_op, 0 | 1 | 14 | 15 | 255) {
            for alpha in [false, true] {
                let (op, inputs, bias, clamp) = if alpha {
                    (
                        core::ptr::addr_of_mut!((*exp).tev.a_op),
                        core::ptr::addr_of_mut!((*exp).tev.a_in)
                            .cast::<_HSD_TEArg>(),
                        core::ptr::addr_of_mut!((*exp).tev.a_bias),
                        core::ptr::addr_of_mut!((*exp).tev.a_clamp),
                    )
                } else {
                    (
                        core::ptr::addr_of_mut!((*exp).tev.c_op),
                        core::ptr::addr_of_mut!((*exp).tev.c_in)
                            .cast::<_HSD_TEArg>(),
                        core::ptr::addr_of_mut!((*exp).tev.c_bias),
                        core::ptr::addr_of_mut!((*exp).tev.c_clamp),
                    )
                };
                if !matches!(*op, 0 | 1)
                    || (*inputs.add(1)).sel != 7
                    || (*inputs.add(2)).sel != 7
                    || h.HSD_TExpGetType((*inputs).exp) == 4
                    || h.HSD_TExpGetType((*inputs.add(3)).exp) == 4
                {
                    continue;
                }
                let d = *inputs.add(3);
                if *op == 0 && d.r#type == 1 {
                    let clamped = if alpha {
                        (*d.exp).tev.a_clamp != 0
                    } else {
                        (d.sel == 1 && (*d.exp).tev.c_clamp != 0)
                            || (d.sel == 5 && (*d.exp).tev.a_clamp != 0)
                    };
                    if clamped && matches!((*inputs).r#type, 2 | 3) {
                        core::ptr::swap(inputs, inputs.add(3));
                    }
                }
                match (*inputs).r#type {
                    1 if alpha || (*inputs).sel == 1 => {
                        let child = (*inputs).exp;
                        let child_sel = (*inputs).sel;
                        let (child_op, child_inputs, child_bias, child_scale) =
                            if alpha {
                                (
                                    (*child).tev.a_op,
                                    core::ptr::addr_of!((*child).tev.a_in)
                                        .cast::<_HSD_TEArg>(),
                                    (*child).tev.a_bias,
                                    (*child).tev.a_scale,
                                )
                            } else {
                                (
                                    (*child).tev.c_op,
                                    core::ptr::addr_of!((*child).tev.c_in)
                                        .cast::<_HSD_TEArg>(),
                                    (*child).tev.c_bias,
                                    (*child).tev.c_scale,
                                )
                            };
                        if matches!(child_op, 0 | 1)
                            && (*child_inputs.add(3)).sel == 7
                            && child_scale == 0
                            && !resource_conflict(exp, child)
                        {
                            let mut value = bias_to_int(child_bias);
                            if child_op == 1 {
                                value = -value;
                            }
                            value += bias_to_int(*bias);
                            // C overwrites `merged` here, even if the color channel already merged.
                            merged = (-1..=1).contains(&value);
                            if merged {
                                *bias = int_to_bias(value);
                                if child_op == 1 {
                                    *op = if *op == 0 { 1 } else { 0 };
                                }
                                for i in 0..3 {
                                    *inputs.add(i) = *child_inputs.add(i);
                                    h.HSD_TExpRef(
                                        (*inputs.add(i)).exp,
                                        (*inputs.add(i)).sel,
                                    );
                                }
                                inherit_resources(exp, child, alpha);
                                h.HSD_TExpUnref(child, child_sel);
                            }
                        }
                    }
                    0 => {
                        let d = *inputs.add(3);
                        if d.r#type != 1 || (!alpha && d.sel != 1) {
                            continue;
                        }
                        let child = d.exp;
                        let (
                            child_op,
                            child_inputs,
                            child_bias,
                            child_scale,
                            child_clamp,
                        ) = if alpha {
                            (
                                (*child).tev.a_op,
                                core::ptr::addr_of!((*child).tev.a_in)
                                    .cast::<_HSD_TEArg>(),
                                (*child).tev.a_bias,
                                (*child).tev.a_scale,
                                (*child).tev.a_clamp,
                            )
                        } else {
                            (
                                (*child).tev.c_op,
                                core::ptr::addr_of!((*child).tev.c_in)
                                    .cast::<_HSD_TEArg>(),
                                (*child).tev.c_bias,
                                (*child).tev.c_scale,
                                (*child).tev.c_clamp,
                            )
                        };
                        if child_scale == 0
                            && (*bias == 0 || *bias != child_bias)
                            && !resource_conflict(exp, child)
                        {
                            merged = true;
                            for i in 0..4 {
                                *inputs.add(i) = *child_inputs.add(i);
                                h.HSD_TExpRef(
                                    (*inputs.add(i)).exp,
                                    (*inputs.add(i)).sel,
                                );
                            }
                            *op = child_op;
                            let mut value = bias_to_int(child_bias);
                            if child_op == 1 {
                                value = -value;
                            }
                            value += bias_to_int(*bias);
                            *bias = int_to_bias(value);
                            if *clamp == 255 || *clamp == 0 {
                                *clamp = child_clamp;
                            }
                            inherit_resources(exp, child, alpha);
                            h.HSD_TExpUnref(child, d.sel);
                        }
                    }
                    _ => {}
                }
            }
        }
        if !merged {
            break;
        }
        passes += 1;
        assert!(passes <= 100, "texpdag.c: simplify loop does not converge");
        result = 1;
    }
    result
}

pub unsafe fn HSD_TExpSimplify(
    h: &mut dyn TExpDagHost,
    exp: *mut HSD_TExp,
) -> i32 {
    if h.HSD_TExpGetType(exp) != 1 {
        return 0;
    }
    let mut result = SimplifySrc(h, exp) != 0;
    result |= SimplifyThis(h, exp) != 0;
    result |= SimplifyByMerge(h, exp) != 0;
    result as i32
}

pub unsafe fn HSD_TExpSimplify2(
    h: &mut dyn TExpDagHost,
    exp: *mut HSD_TExp,
) -> i32 {
    for i in 0..4 {
        let src = (*exp).tev.c_in[i];
        if src.r#type == 1 && src.sel == 1 && IsThroughColor(&*src.exp) {
            let d = (*src.exp).tev.c_in[3];
            if d.r#type == 6 {
                if (*exp).tev.kcsel == 255 {
                    (*exp).tev.kcsel = (*src.exp).tev.kcsel;
                } else if (*exp).tev.kcsel != (*src.exp).tev.kcsel {
                    continue;
                }
            }
            if matches!(d.r#type, 5 | 6) {
                (*exp).tev.c_in[i] = d;
                h.HSD_TExpRef(d.exp, d.sel);
                h.HSD_TExpUnref(src.exp, src.sel);
            }
        }
    }
    for i in 0..4 {
        let src = (*exp).tev.a_in[i];
        if src.r#type == 1 && IsThroughAlpha(&*src.exp) {
            let d = (*src.exp).tev.a_in[3];
            if d.r#type == 6 {
                if (*exp).tev.kasel == 255 {
                    (*exp).tev.kasel = (*src.exp).tev.kasel;
                } else if (*exp).tev.kasel != (*src.exp).tev.kasel {
                    continue;
                }
            }
            if matches!(d.r#type, 5 | 6) {
                (*exp).tev.a_in[i] = d;
                h.HSD_TExpRef(d.exp, d.sel);
                h.HSD_TExpUnref(src.exp, src.sel);
            }
        }
    }
    0
}

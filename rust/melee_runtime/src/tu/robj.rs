//! Complete robj.c entry points, retaining the established owned object model.
#![allow(non_snake_case)]
pub use crate::robj::{add_anim as HSD_RObjAddAnim, anim as HSD_RObjAnim, bcexp_load_desc as bcexpLoadDesc, dummy_func, exp_evaluate as expEvaluate, exp_load_desc as expLoadDesc, get_by_type as HSD_RObjGetByType, get_by_type as inlined_HSD_RObjGetByType, get_constraint_type as HSD_RObjGetConstraintType, load_rvalue as loadRvalue, remove_anim_all as HSD_RObjRemoveAnimAll, remove_anim_by_flags as HSD_RObjRemoveAnimByFlags, req_anim_by_flags as HSD_RObjReqAnimByFlags, resolve_refs as HSD_RObjResolveRefs, robj_update_func as RObjUpdateFunc, rvalue_alloc as HSD_RvalueAlloc, rvalue_remove as HSD_RvalueRemove, rvalue_remove_all as HSD_RvalueRemoveAll, rvalue_resolve_refs as HSD_RvalueResolveRefs, rvalue_resolve_refs_all as HSD_RvalueResolveRefsAll, set_dirup_matrix};
use crate::{
    jobj::{JObjId, JObjTree},
    robj::{HsdRObj, RObjGlobals},
};
use melee_types::HSD_ObjAllocData;

/// Allocator initialization remains an external call until objalloc.c is ported.
pub mod ext {
    #![allow(non_upper_case_globals)]
    use melee_types::HSD_ObjAllocData;
    pub static mut HSD_ObjAllocInit: unsafe fn(
        *mut HSD_ObjAllocData,
        u32,
        u32,
    ) = missing;
    unsafe fn missing(_: *mut HSD_ObjAllocData, _: u32, _: u32) {
        panic!("TODO HSD_ObjAllocInit")
    }
}

pub unsafe fn HSD_RObjInitAllocData(
    robj: &mut HSD_ObjAllocData,
    rvalue: &mut HSD_ObjAllocData,
) {
    melee_core::ct!("HSD_RObjInitAllocData");
    unsafe {
        (ext::HSD_ObjAllocInit)(
            robj,
            core::mem::size_of::<melee_types::HSD_RObj>() as u32,
            4,
        );
        (ext::HSD_ObjAllocInit)(
            rvalue,
            core::mem::size_of::<melee_types::HSD_Rvalue>() as u32,
            4,
        );
    }
}
pub fn HSD_RObjGetAllocData(
    data: &mut HSD_ObjAllocData,
) -> &mut HSD_ObjAllocData {
    melee_core::ct!("HSD_RObjGetAllocData");
    data
}
pub fn HSD_RvalueObjGetAllocData(
    data: &mut HSD_ObjAllocData,
) -> &mut HSD_ObjAllocData {
    melee_core::ct!("HSD_RvalueObjGetAllocData");
    data
}
/// The native arena owns allocations; freeing transfers ownership to drop.
pub fn HSD_RObjFree(slot: &mut Option<HsdRObj>) {
    melee_core::ct!("HSD_RObjFree");
    drop(slot.take());
}
pub fn jobj_parent<D: Copy + PartialEq, R: Copy + PartialEq>(
    tree: &JObjTree<D, R>,
    jobj: Option<JObjId>,
) -> Option<JObjId> {
    melee_core::ct!("jobj_parent");
    tree.parent_of(jobj)
}
pub fn _HSD_RObjForgetMemory(g: &mut RObjGlobals, low: usize, high: usize) {
    melee_core::ct!("_HSD_RObjForgetMemory");
    let ptr = g.arg_buf.as_ref().map_or(0, |b| b.as_ptr() as usize);
    if low <= ptr && ptr < high {
        g.forget_memory();
    }
}

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    use crate::aobj::AObjGlobals;
    use crate::jobj::JObjHost;
    use crate::mtx::Vec3;
    #[allow(unused_imports)]
    use crate::robj::*;
    use super::*;
    /// `HSD_RObjAnimAll`: the C entry point over `crate::robj::anim_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjAnimAll(chain: &mut [HsdRObj], g: &mut AObjGlobals) {
        crate::robj::anim_all(chain, g)
    }
    /// `HSD_RObjLoadDesc`: the C entry point over `crate::robj::load_desc_list` (the owned object model).
    #[inline]
    pub fn HSD_RObjLoadDesc(descs: &[HsdRObjDesc]) -> Vec<HsdRObj> {
        crate::robj::load_desc_list(descs)
    }
    /// `HSD_RObjReqAnimAllByFlags`: the C entry point over `crate::robj::req_anim_all_by_flags` (the owned object model).
    #[inline]
    pub fn HSD_RObjReqAnimAllByFlags(chain: &mut [HsdRObj], startframe: f32, flags: u32) {
        crate::robj::req_anim_all_by_flags(chain, startframe, flags)
    }
    /// `HSD_RObjResolveRefsAll`: the C entry point over `crate::robj::resolve_refs_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjResolveRefsAll<X: RObjHost>(chain: &mut [HsdRObj],
        descs: &[HsdRObjDesc],
        lookup: &mut dyn FnMut(u32) -> Option<JObjId>,
        rhost: &mut X,) {
        crate::robj::resolve_refs_all(chain, descs, lookup, rhost)
    }
    /// `HSD_RObjAddAnimAll`: the C entry point over `crate::robj::add_anim_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjAddAnimAll(chain: &mut [HsdRObj],
        anim: &[HsdRObjAnimJoint],
        resolve_obj: &mut dyn FnMut(u32) -> Option<JObjId>,) {
        crate::robj::add_anim_all(chain, anim, resolve_obj)
    }
    /// `HSD_RObjRemoveAnimAllByFlags`: the C entry point over `crate::robj::remove_anim_all_by_flags` (the owned object model).
    #[inline]
    pub fn HSD_RObjRemoveAnimAllByFlags(chain: &mut [HsdRObj], flags: u32) {
        crate::robj::remove_anim_all_by_flags(chain, flags)
    }
    /// `HSD_RObjReqAnimAll`: the C entry point over `crate::robj::req_anim_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjReqAnimAll(chain: &mut [HsdRObj], startframe: f32) {
        crate::robj::req_anim_all(chain, startframe)
    }
    /// `HSD_RObjAlloc`: the C entry point over `crate::robj::alloc` (the owned object model).
    #[inline]
    pub fn HSD_RObjAlloc() -> HsdRObj {
        crate::robj::alloc()
    }
    /// `HSD_RObjGetGlobalPosition`: the C entry point over `crate::robj::get_global_position` (the owned object model).
    #[inline]
    pub fn HSD_RObjGetGlobalPosition<D, R, H>(chain: &[HsdRObj],
        ty: i32,
        tree: &mut JObjTree<D, R>,
        host: &mut H,) -> (i32, Option<Vec3>)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>, {
        crate::robj::get_global_position(chain, ty, tree, host)
    }
    /// `HSD_RObjSetConstraintObj`: the C entry point over `crate::robj::set_constraint_obj` (the owned object model).
    #[inline]
    pub fn HSD_RObjSetConstraintObj<X: RObjHost>(r: Option<&mut HsdRObj>, o: JObjId, rhost: &mut X) {
        crate::robj::set_constraint_obj(r, o, rhost)
    }
    /// `HSD_RObjSetFlags`: the C entry point over `crate::robj::set_flags` (the owned object model).
    #[inline]
    pub fn HSD_RObjSetFlags(robj: Option<&mut HsdRObj>, flags: u32) {
        crate::robj::set_flags(robj, flags)
    }
    /// `HSD_RObjUpdateAll`: the C entry point over `crate::robj::update_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjUpdateAll<D, R, H, X>(chain: &mut [HsdRObj],
        g: &mut RObjGlobals,
        tree: &mut JObjTree<D, R>,
        host: &mut H,
        rhost: &mut X,
        obj: JObjId,
        update: &mut RObjUpdate<'_, D, R>,) where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
        X: RObjHost, {
        crate::robj::update_all(chain, g, tree, host, rhost, obj, update)
    }
    /// `resolveCnsDirUp`: the C entry point over `crate::robj::resolve_cns_dir_up` (the owned object model).
    #[inline]
    pub fn resolveCnsDirUp<D, R, H>(chain: &[HsdRObj],
        tree: &mut JObjTree<D, R>,
        host: &mut H,
        obj: JObjId,
        update: &mut RObjUpdate<'_, D, R>,) where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>, {
        crate::robj::resolve_cns_dir_up(chain, tree, host, obj, update)
    }
    /// `resolveCnsOrientation`: the C entry point over `crate::robj::resolve_cns_orientation` (the owned object model).
    #[inline]
    pub fn resolveCnsOrientation<D, R, H>(chain: &[HsdRObj],
        tree: &mut JObjTree<D, R>,
        host: &mut H,
        obj: JObjId,
        update: &mut RObjUpdate<'_, D, R>,) where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>, {
        crate::robj::resolve_cns_orientation(chain, tree, host, obj, update)
    }
    /// `resolveLimits`: the C entry point over `crate::robj::resolve_limits` (the owned object model).
    #[inline]
    pub fn resolveLimits<D, R, H>(chain: &[HsdRObj], tree: &mut JObjTree<D, R>, host: &mut H, obj: JObjId) where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>, {
        crate::robj::resolve_limits(chain, tree, host, obj)
    }
    /// `HSD_RObjRemove`: the C entry point over `crate::robj::remove` (the owned object model).
    #[inline]
    pub fn HSD_RObjRemove<X: RObjHost>(r: HsdRObj, rhost: &mut X) {
        crate::robj::remove(r, rhost)
    }
    /// `HSD_RObjRemoveAll`: the C entry point over `crate::robj::remove_all` (the owned object model).
    #[inline]
    pub fn HSD_RObjRemoveAll<X: RObjHost>(chain: Vec<HsdRObj>, rhost: &mut X) {
        crate::robj::remove_all(chain, rhost)
    }
}
pub use cnames::*;

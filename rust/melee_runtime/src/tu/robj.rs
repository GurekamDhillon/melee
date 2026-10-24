//! Complete robj.c entry points, retaining the established owned object model.
#![allow(non_snake_case)]
pub use crate::robj::{
    add_anim as HSD_RObjAddAnim, add_anim_all as HSD_RObjAddAnimAll,
    alloc as HSD_RObjAlloc, anim as HSD_RObjAnim, anim_all as HSD_RObjAnimAll,
    bcexp_load_desc as bcexpLoadDesc, dummy_func, exp_evaluate as expEvaluate,
    exp_load_desc as expLoadDesc, get_by_type as HSD_RObjGetByType,
    get_by_type as inlined_HSD_RObjGetByType,
    get_constraint_type as HSD_RObjGetConstraintType,
    get_global_position as HSD_RObjGetGlobalPosition,
    load_desc_list as HSD_RObjLoadDesc, load_rvalue as loadRvalue,
    remove as HSD_RObjRemove, remove_all as HSD_RObjRemoveAll,
    remove_anim_all as HSD_RObjRemoveAnimAll,
    remove_anim_all_by_flags as HSD_RObjRemoveAnimAllByFlags,
    remove_anim_by_flags as HSD_RObjRemoveAnimByFlags,
    req_anim_all as HSD_RObjReqAnimAll,
    req_anim_all_by_flags as HSD_RObjReqAnimAllByFlags,
    req_anim_by_flags as HSD_RObjReqAnimByFlags,
    resolve_cns_dir_up as resolveCnsDirUp,
    resolve_cns_orientation as resolveCnsOrientation,
    resolve_limits as resolveLimits, resolve_refs as HSD_RObjResolveRefs,
    resolve_refs_all as HSD_RObjResolveRefsAll,
    robj_update_func as RObjUpdateFunc, rvalue_alloc as HSD_RvalueAlloc,
    rvalue_remove as HSD_RvalueRemove,
    rvalue_remove_all as HSD_RvalueRemoveAll,
    rvalue_resolve_refs as HSD_RvalueResolveRefs,
    rvalue_resolve_refs_all as HSD_RvalueResolveRefsAll,
    set_constraint_obj as HSD_RObjSetConstraintObj, set_dirup_matrix,
    set_flags as HSD_RObjSetFlags, update_all as HSD_RObjUpdateAll,
};
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
    data
}
pub fn HSD_RvalueObjGetAllocData(
    data: &mut HSD_ObjAllocData,
) -> &mut HSD_ObjAllocData {
    data
}
/// The native arena owns allocations; freeing transfers ownership to drop.
pub fn HSD_RObjFree(slot: &mut Option<HsdRObj>) {
    drop(slot.take());
}
pub fn jobj_parent<D: Copy + PartialEq, R: Copy + PartialEq>(
    tree: &JObjTree<D, R>,
    jobj: Option<JObjId>,
) -> Option<JObjId> {
    tree.parent_of(jobj)
}
pub fn _HSD_RObjForgetMemory(g: &mut RObjGlobals, low: usize, high: usize) {
    let ptr = g.arg_buf.as_ref().map_or(0, |b| b.as_ptr() as usize);
    if low <= ptr && ptr < high {
        g.forget_memory();
    }
}

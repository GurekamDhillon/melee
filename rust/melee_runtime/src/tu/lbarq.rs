//! Port of src/melee/lb/lbarq.c (the ARAM request queue: ten request nodes on three lists indexed
//! by state, free / pending / done). Verified against the C (whole TU) by tests/tu_lbarq.rs
//! (`python rust/melee_runtime/gen/mtu/run_tu.py lbarq`).
//!
//! The C finds its node from the ARAM callback's `lbArqHandle` (`handle->node`, which on the
//! console is the request's `owner` word) and the list for a state with `0x1E0 + state * 4` bytes
//! from the global. Both are 32-bit-layout arithmetic: here the node is found from the request
//! it embeds (`arq` sits at a fixed offset inside the node, and the request is what the callback
//! receives) and the list by index. Same nodes, same lists.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn ARQPostRequest(a0: *mut ARQRequest, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, a7: Option<unsafe extern "C" fn(*mut ARQRequest)>);
    fn DCInvalidateRange(a0: *mut c_void, a1: u32);
    fn OSDisableInterrupts() -> i32;
    fn OSRestoreInterrupts(a0: i32) -> i32;
    fn wait_idle();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub type lbArqCallback = Option<unsafe extern "C" fn(*mut c_void)>;

pub const LB_ARQ_STATE_FREE: u32 = 0;
pub const LB_ARQ_STATE_PENDING: u32 = 1;
pub const LB_ARQ_STATE_DONE: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbArqNode {
    pub next: *mut lbArqNode,
    pub state: u32,
    pub arq: ARQRequest,
    pub callback: lbArqCallback,
    pub callback_arg: *mut c_void,
}
melee_core::gc_layout!(lbArqNode, "lbArqNode", size 0x30, align 4, [
    next @ 0x0: 4 "lbArqNode*",
    state @ 0x4: 4 "lbArqState",
    arq @ 0x8: 32 "ARQRequest",
    callback @ 0x28: 4 "lbArqCallback",
    callback_arg @ 0x2C: 4 "void*",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbArqGlobal {
    pub nodes: [lbArqNode; 10],
    pub list: [*mut lbArqNode; 3],
}
melee_core::gc_layout!(lbArqGlobal, "lbArqGlobal", size 0x1EC, align 4, [
    nodes @ 0x0: 480 "lbArqNode[10]",
    list @ 0x1E0: 12 "lbArqNode*[3]",
]);

/// 4316C0
pub static mut lbArq_804316C0: lbArqGlobal = unsafe { core::mem::zeroed() };

pub unsafe extern "C" fn lbArq_80014ABC(arg0: *mut lbArqNode) -> u32 {
    (*arg0).state
}

/// The ARAM request's completion callback (`ARQCallback`): moves the node to the done list, runs
/// the node's callback if it has one and then returns the node to the free list.
pub unsafe extern "C" fn lbArq_80014AC4(handle: *mut ARQRequest) {
    let global = &raw mut lbArq_804316C0;
    let node = (handle as *mut u8).sub(offset_of!(lbArqNode, arq)) as *mut lbArqNode;

    let intr = ext::OSDisableInterrupts();

    // Remove from the current list (indexed by state)
    let mut prev = &raw mut (*global).list[(*node).state as usize];
    while *prev != node {
        prev = &raw mut (**prev).next;
    }
    *prev = (*node).next;

    // Add to the done list
    let mut tail = &raw mut (*global).list[LB_ARQ_STATE_DONE as usize];
    while !(*tail).is_null() {
        tail = &raw mut (**tail).next;
    }
    *tail = node;
    (*node).next = null_mut();
    (*node).state = LB_ARQ_STATE_DONE;

    ext::OSRestoreInterrupts(intr);

    // Call the callback if present
    if let Some(cb) = (*node).callback {
        cb((*node).callback_arg);

        let intr = ext::OSDisableInterrupts();

        // Remove from the current list again
        let mut prev = &raw mut (*global).list[(*node).state as usize];
        while *prev != node {
            prev = &raw mut (**prev).next;
        }
        *prev = (*node).next;

        // Add to the free list
        let mut tail = &raw mut (*global).list[LB_ARQ_STATE_FREE as usize];
        while !(*tail).is_null() {
            tail = &raw mut (**tail).next;
        }
        *tail = node;
        (*node).next = null_mut();
        (*node).state = LB_ARQ_STATE_FREE;

        ext::OSRestoreInterrupts(intr);
    }
}

pub unsafe extern "C" fn lbArq_80014BD0(source: u32, dest: *mut c_void, length: usize, callback: lbArqCallback, callback_arg: *mut c_void) {
    let global = &raw mut lbArq_804316C0;

    ext::DCInvalidateRange(dest, length as u32);
    let mut intr = ext::OSDisableInterrupts();
    let rp = (*global).list[LB_ARQ_STATE_FREE as usize];
    let mut free_head = &raw mut (*global).list[LB_ARQ_STATE_FREE as usize];
    assert!(!rp.is_null(), "lbarq.c:103 rp");
    *free_head = (*rp).next;
    (*rp).callback = callback;
    (*rp).callback_arg = callback_arg;

    let mut tail = &raw mut (*global).list[LB_ARQ_STATE_PENDING as usize];
    while !(*tail).is_null() {
        tail = &raw mut (**tail).next;
    }
    *tail = rp;
    (*rp).next = null_mut();
    (*rp).state = LB_ARQ_STATE_PENDING;

    ext::ARQPostRequest(&raw mut (*rp).arq, rp as usize as u32, 1, 0, source, dest as usize as u32, length as u32, Some(lbArq_80014AC4));

    if (*rp).callback.is_none() {
        ext::OSRestoreInterrupts(intr);
        while lbArq_80014ABC(rp) != LB_ARQ_STATE_DONE {
            ext::wait_idle();
        }
        intr = ext::OSDisableInterrupts();
        tail = &raw mut (*global).list[(*rp).state as usize];
        while *tail != rp {
            tail = &raw mut (**tail).next;
        }
        *tail = (*rp).next;
        while !(*free_head).is_null() {
            free_head = &raw mut (**free_head).next;
        }
        *free_head = rp;
        (*rp).next = null_mut();
        (*rp).state = LB_ARQ_STATE_FREE;
    }
    ext::OSRestoreInterrupts(intr);
}

pub unsafe extern "C" fn lbArq_80014D2C() {
    let global = &raw mut lbArq_804316C0;
    let nodes = (&raw mut (*global).nodes) as *mut lbArqNode;

    (*global).list[LB_ARQ_STATE_FREE as usize] = null_mut();
    (*global).list[LB_ARQ_STATE_PENDING as usize] = null_mut();
    (*global).list[LB_ARQ_STATE_DONE as usize] = null_mut();
    (*global).list[LB_ARQ_STATE_FREE as usize] = nodes;

    for i in 0..9usize {
        let node = nodes.add(i);
        (*node).next = node.add(1);
        (*node).state = LB_ARQ_STATE_FREE;
    }
    // `node` is still the last loop element, nodes[8]: nodes[9] is never linked.
    let node = nodes.add(8);
    (*node).next = null_mut();
    (*node).state = LB_ARQ_STATE_FREE;
}

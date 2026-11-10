"""lbarq spec (see lbspecs.py)."""
from lbspecs import add

C_DECL = r'''
static ARQRequest* PEND;
static void cbv1(void* a) { T(7101); T(PI(a)); }
static void cbv2(void* a) { T(7102); T(PI(a)); }
'''
RS_EXTRA = r'''
static mut PEND: *mut ARQRequest = null_mut();
pub unsafe extern "C" fn cbv1(a: *mut c_void) { t(7101); t(pi(a)); }
pub unsafe extern "C" fn cbv2(a: *mut c_void) { t(7102); t(pi(a)); }
'''
# lists: node 0 is always free; the rest go to a random list, in index order
SETUP_C = r'''{ int q; lbArqNode* last[3] = { 0, 0, 0 }; PEND = 0;
  lbArq_804316C0.list[0] = lbArq_804316C0.list[1] = lbArq_804316C0.list[2] = 0;
  for (q = 0; q < 10; q++) { lbArqNode* n = &lbArq_804316C0.nodes[q]; u32 s = q == 0 ? 0 : fv() % 3;
    n->state = s; n->next = 0; if (last[s]) last[s]->next = n; else lbArq_804316C0.list[s] = n; last[s] = n;
    n->callback = (fv() & 1) ? ((fv() & 1) ? cbv1 : cbv2) : 0; n->callback_arg = (fv() & 1) ? BUF + 8 * (fv() % 8) : 0; } }'''
SETUP_R = r'''{ PEND = null_mut(); let mut last: [*mut m::lbArqNode; 3] = [null_mut(); 3];
  m::lbArq_804316C0.list = [null_mut(); 3];
  for q in 0..10usize { let n = &raw mut m::lbArq_804316C0.nodes[q]; let s = if q == 0 { 0 } else { fv() % 3 };
    (*n).state = s; (*n).next = null_mut(); if !last[s as usize].is_null() { (*last[s as usize]).next = n; } else { m::lbArq_804316C0.list[s as usize] = n; } last[s as usize] = n;
    (*n).callback = if fv() & 1 != 0 { if fv() & 1 != 0 { Some(cbv1) } else { Some(cbv2) } } else { None };
    (*n).callback_arg = if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(8 * (fv() % 8) as usize) as *mut c_void } else { null_mut() }; } }'''
NODE = ("&lbArq_804316C0.nodes[fv() % 10]", "&raw mut m::lbArq_804316C0.nodes[(fv() % 10) as usize]")
HANDLE = ("(ARQRequest*) &lbArq_804316C0.nodes[fv() % 10].arq", "(&raw mut m::lbArq_804316C0.nodes[(fv() % 10) as usize].arq)")
CB = ("(lbArqCallback) ((fv() & 1) ? ((fv() & 1) ? (void*) cbv1 : (void*) cbv2) : 0)",
      "if fv() & 1 != 0 { if fv() & 1 != 0 { Some(cbv1 as unsafe extern \"C\" fn(*mut c_void)) } else { Some(cbv2 as unsafe extern \"C\" fn(*mut c_void)) } } else { None }")
add("lbarq", "src/melee/lb/lbarq.c", dict(
    c_pre='#include "types.h"' + chr(10),
    c_decl=C_DECL, rs_extra=RS_EXTRA,
    pregs=("freg((const void*) cbv1); freg((const void*) cbv2);", "freg(cbv1 as *const () as usize); freg(cbv2 as *const () as usize);"),
    setup=(SETUP_C, SETUP_R),
    ov={"lbArq_80014ABC": {0: NODE}, "lbArq_80014AC4": {0: HANDLE}, "lbArq_80014BD0": {3: CB}}),
    types=[], pools=[], own_hash=["lbArq_804316C0"],
    ptr32_args={"ARQPostRequest": [1, 5]},
    effects={
        "ARQPostRequest": ("if (fv() % 3 != 0) lbArq_80014AC4((ARQRequest*) a0); else PEND = a0;",
                           "if fv() % 3 != 0 { m::lbArq_80014AC4(a0); } else { PEND = a0; }"),
        "wait_idle": ("if (PEND) { ARQRequest* r = PEND; PEND = 0; lbArq_80014AC4((ARQRequest*) r); }",
                      "if !PEND.is_null() { let r = PEND; PEND = null_mut(); m::lbArq_80014AC4(r); }"),
    },
    # upstream derives the node from request->owner (the real ARQPostRequest stores it; the stub does not): take the
    # embedding node instead, the same node, as the port does (offset_of!(lbArqNode, arq))
    patch=[("    lbArqNode* node = (lbArqNode*) request->owner;",
            "    lbArqNode* node = (lbArqNode*) ((char*) request - offsetof(lbArqNode, arq));")])

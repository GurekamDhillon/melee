//! Native `jobj.c` completion. Arena algorithms remain the single implementations in
//! `crate::jobj` and `crate::jobj_rt`; exported below. The load/class boundary uses generated
//! little-endian records, never guest-memory offsets. See the shard coverage document.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]
pub use crate::jobj::{JObjTree, JObjHost, JObjId, JointSource};
pub use crate::jobj_rt::{JObjRt, JObjExt, sort_anim as JObjSortAnim, jobj_get_joint2, jobj_get_effector};
use crate::jobj::*;
use melee_types::{HSD_JObj, HSD_Joint, HSD_JObjInfo, HSD_DObj, HSD_DObjDesc, HSD_RObj,
    HSD_RObjDesc, HSD_AObj, HSD_CObj, _HSD_Class, _HSD_ClassInfo, _HSD_SList, Mtx};
use std::ptr::null_mut;
#[allow(unused_imports)]
use melee_types::*;
use crate::jobj::{JOBJ_HIDDEN, JOBJ_INSTANCE, JOBJ_MTX_DIRTY, JOBJ_PTCL, JOBJ_SPLINE};

pub static mut hsdJObj: HSD_JObjInfo = { let mut i: HSD_JObjInfo = unsafe { core::mem::zeroed() }; i.parent.parent.head.info_init = Some(JObjInfoInit); i };
pub static mut default_class: *mut _HSD_ClassInfo = null_mut();
pub static mut current_jobj: *mut HSD_JObj = null_mut();
pub static mut ufc_callbacks: *mut _HSD_SList = null_mut();

/// Exact external C boundaries. Defaults deliberately identify unavailable cross-TU services;
/// deterministic native hosts (including the C oracle mirror) install typed hooks.
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
    fn HSD_AObjInitEndCallBack();
    fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
    fn HSD_AObjInvokeCallBacks();
    fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj;
    fn HSD_AObjRemove(a0: *mut HSD_AObj);
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj;
    fn HSD_DObjAddAnimAll(a0: *mut HSD_DObj, a1: *mut _HSD_MatAnim, a2: *mut HSD_ShapeAnimDObj);
    fn HSD_DObjAnimAll(a0: *mut HSD_DObj);
    fn HSD_DObjLoadDesc(a0: *mut HSD_DObjDesc) -> *mut HSD_DObj;
    fn HSD_DObjRemoveAll(a0: *mut HSD_DObj);
    fn HSD_DObjRemoveAnimAllByFlags(a0: *mut HSD_DObj, a1: u32);
    fn HSD_DObjReqAnimAllByFlags(a0: *mut HSD_DObj, a1: f32, a2: u32);
    fn HSD_DObjResolveRefsAll(a0: *mut HSD_DObj, a1: *mut HSD_DObjDesc);
    fn HSD_IDGetDataFromTable(a0: *mut _HSD_IDTable, a1: u64, a2: *mut i32) -> *mut c_void;
    fn HSD_IDInsertToTable(a0: *mut _HSD_IDTable, a1: u64, a2: *mut c_void);
    fn HSD_IDRemoveByIDFromTable(a0: *mut _HSD_IDTable, a1: u64);
    fn HSD_JObjDisp(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjDispSub(a0: *mut HSD_JObj, a1: *mut c_void, a2: *mut c_void, a3: u32, a4: u32);
    fn HSD_JObjMakePositionMtx(a0: *mut HSD_JObj, a1: *mut c_void, a2: *mut c_void);
    fn HSD_MtxAlloc() -> *mut c_void;
    fn HSD_MtxFree(a0: *mut c_void);
    fn HSD_MtxGetRotation(a0: *mut c_void, a1: *mut Vec);
    fn HSD_MtxGetScale(a0: *mut c_void, a1: *mut Vec);
    fn HSD_MtxGetTranslate(a0: *mut c_void, a1: *mut Vec);
    fn HSD_MtxInverseConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn HSD_MtxSRT(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut Vec);
    fn HSD_MtxSRTQuat(a0: *mut c_void, a1: *mut Vec, a2: *mut Quaternion, a3: *mut Vec, a4: *mut Vec);
    fn HSD_RObjAddAnimAll(a0: *mut HSD_RObj, a1: *mut HSD_RObjAnimJoint);
    fn HSD_RObjAnimAll(a0: *mut HSD_RObj);
    fn HSD_RObjGetByType(a0: *mut HSD_RObj, a1: u32, a2: u32) -> *mut HSD_RObj;
    fn HSD_RObjGetGlobalPosition(a0: *mut HSD_RObj, a1: i32, a2: *mut Vec) -> i32;
    fn HSD_RObjLoadDesc(a0: *mut HSD_RObjDesc) -> *mut HSD_RObj;
    fn HSD_RObjRemoveAll(a0: *mut HSD_RObj);
    fn HSD_RObjRemoveAnimAllByFlags(a0: *mut HSD_RObj, a1: u32);
    fn HSD_RObjReqAnimAllByFlags(a0: *mut HSD_RObj, a1: f32, a2: u32);
    fn HSD_RObjResolveRefsAll(a0: *mut HSD_RObj, a1: *mut HSD_RObjDesc);
    fn HSD_RObjUpdateAll(a0: *mut HSD_RObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
    fn HSD_VecAlloc() -> *mut c_void;
    fn HSD_VecFree(a0: *mut c_void);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXIdentity(a0: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXIdentity(a0 as _);
    fn PSMTXInverse(a0: *mut c_void, a1: *mut c_void) -> u32;
    fn PSMTXMultVec(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec);
    fn PSMTXRotAxisRad(a0: *mut c_void, a1: *mut Vec, a2: f32);
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECDotProduct(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn PSVECNormalize(a0: *mut Vec, a1: *mut Vec);
    fn PSVECScale(a0: *mut Vec, a1: *mut Vec, a2: f32);
    fn PSVECSubtract(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn ProfActive() -> i32;
    fn ProfBegin(a0: i32, a1: i32);
    fn ProfEnd();
    fn acosf(a0: f32) -> f32;
    fn diag_jobj_mtx(a0: *const f32, a1: i32, a2: i32);
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32);
    fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32;
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void;
    fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo;
    fn splArcLengthPoint(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjDispSub(a0: *mut HSD_JObj, a1: *mut c_void, a2: *mut c_void, a3: u32, a4: u32) { super::HSD_JObjDispSub(a0, a1, a2, a3, a4) }
        pub unsafe extern "C" fn HSD_JObjMakePositionMtx(a0: *mut HSD_JObj, a1: *mut c_void, a2: *mut c_void) { super::HSD_JObjMakePositionMtx(a0, a1, a2) }
        pub unsafe extern "C" fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 { super::hsdIsDescendantOf(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdObj: HSD_ClassInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;
/// object.c `hsdObj`.
unsafe fn hsd_obj_ptr() -> *mut _HSD_ClassInfo {
    if use_ext_data { (&raw mut ext::hsdObj).cast() } else { (&raw mut crate::tu::object::hsdObj).cast() }
}
/// Cross-TU services of the arena adapters (not in jobj.c's own raw entry points).
pub mod ext_native {
    use super::*;
    pub fn HSD_ByteCodeEval(_b:Option<&[u8]>,_a:&[f32],_n:u32)->f32{todo!("cross-TU HSD_ByteCodeEval")}
    pub fn HSD_DObjRemoveAnimAllByFlags(_d:&[*mut HSD_DObj],_f:u32){todo!("cross-TU HSD_DObjRemoveAnimAllByFlags")}
    pub fn HSD_DObjReqAnimAllByFlags(_d:&[*mut HSD_DObj],_x:f32,_f:u32){todo!("cross-TU HSD_DObjReqAnimAllByFlags")}
    pub fn HSD_DObjAddAnimAll(_d:&[*mut HSD_DObj],_m:Option<&()>,_s:Option<&()>){todo!("cross-TU HSD_DObjAddAnimAll")}
    pub fn HSD_DObjAnimAll(_d:&[*mut HSD_DObj]){todo!("cross-TU HSD_DObjAnimAll")}
    pub fn splArcLengthPoint(_j:*mut HSD_JObj,_s:f32)->crate::mtx::Vec3{todo!("cross-TU splArcLengthPoint")}
}
/// class.h `hsdDelete` (static inline): release then destroy through the object class.
unsafe fn hsd_delete_ext(o:*mut HSD_JObj){
    if o.is_null(){return}
    let i=(*o).object.parent.class_info;
    ((*i).release.expect("hsdDelete: release"))(o.cast());
    ((*i).destroy.expect("hsdDelete: destroy"))(o.cast());
}
unsafe fn info(j:*mut HSD_JObj)->*mut HSD_JObjInfo { (*j).object.parent.class_info.cast() }
unsafe fn ref_inc(j:*mut HSD_JObj) {
    if !j.is_null() { (*j).object.ref_count=(*j).object.ref_count.wrapping_add(1); assert_ne!((*j).object.ref_count,65535); }
}
fn iref_dec(n:&mut u16)->bool { if *n==0 {true} else {*n-=1; *n==0} }
unsafe fn dobj_union(j:*mut HSD_JObj)->bool { (*j).flags&(JOBJ_PTCL|JOBJ_SPLINE)==0 }
pub unsafe fn has_scl(j:*mut HSD_JObj)->bool { !j.is_null()&&!(*j).scl.is_null() }
pub unsafe fn robj_set_next(r:*mut HSD_RObj,next:*mut HSD_RObj)->*mut HSD_RObj { if !r.is_null(){(*r).next=next;r}else{next} }

pub unsafe fn HSD_JObjSetDefaultClass(i:*mut _HSD_ClassInfo) {
    if !i.is_null(){assert!(ext::hsdIsDescendantOf(i.cast(), (&raw mut hsdJObj).cast())!=0);} default_class=i;
}
pub unsafe fn HSD_JObjAlloc()->*mut HSD_JObj {
    let j=ext::hsdNew(if default_class.is_null(){(&raw mut hsdJObj).cast()}else{default_class}).cast::<HSD_JObj>(); assert!(!j.is_null()); j
}
pub unsafe fn JObjLoadJointSub(d:*mut HSD_Joint,parent:*mut HSD_JObj)->*mut HSD_JObj {
    if d.is_null(){return null_mut()}
    let i=if (*d).class_name.is_null(){null_mut()}else{ext::hsdSearchClassInfo((*d).class_name)};
    let j=if i.is_null(){HSD_JObjAlloc()}else{ext::hsdNew(i).cast::<HSD_JObj>()};
    assert!(!j.is_null()); ((*info(j)).load.expect("JObj load method"))(j,d,parent); j
}
pub unsafe extern "C" fn JObjLoad(j:*mut HSD_JObj,d:*mut HSD_Joint,parent:*mut HSD_JObj)->i32 {
    if (*d).flags&JOBJ_INSTANCE==0 {(*j).child=JObjLoadJointSub((*d).child,j);}
    (*j).next=JObjLoadJointSub((*d).next,parent); (*j).parent=parent; (*j).flags|=(*d).flags;
    if (*j).flags&JOBJ_SPLINE!=0 {(*j).u.spline=(*d).u.spline;}
    else if (*j).flags&JOBJ_PTCL!=0 {
        (*j).u.ptcl=(*d).u.ptcl; let mut s=(*d).u.ptcl;
        while !s.is_null(){
            // The C updates the low 32-bit payload word, retaining native pointer upper bits.
            (*s).data=(((*s).data as usize)|0x80000000) as *mut _; s=(*s).next;
        }
    } else {(*j).u.dobj=ext::HSD_DObjLoadDesc((*d).u.dobjdesc);}
    (*j).robj=ext::HSD_RObjLoadDesc((*d).robjdesc);
    (*j).rotate.x=(*d).rotation.x; (*j).rotate.y=(*d).rotation.y; (*j).rotate.z=(*d).rotation.z;
    (*j).scale=(*d).scale; (*j).translate=(*d).position; ext::PSMTXIdentity((*j).mtx.as_mut_ptr().cast()); (*j).scl=null_mut();
    if !(*d).mtx.is_null(){(*j).envelopemtx=ext::HSD_MtxAlloc().cast();*((*j).envelopemtx as *mut Mtx)=*((*d).mtx as *const Mtx);}
    (*j).id=d as usize; ext::HSD_IDInsertToTable(null_mut(),(*j).id as u64,j.cast()); 0
}
pub unsafe fn HSD_JObjLoadJoint(d:*mut HSD_Joint)->*mut HSD_JObj {
    let j=JObjLoadJointSub(d,null_mut()); HSD_JObjResolveRefsAll(j,d); j
}
pub unsafe fn HSD_JObjResolveRefs(j:*mut HSD_JObj,d:*mut HSD_Joint) {
    if j.is_null()||d.is_null(){return}
    ext::HSD_RObjResolveRefsAll((*j).robj,(*d).robjdesc);
    if (*j).flags&JOBJ_INSTANCE!=0 {
        HSD_JObjUnref((*j).child);
        (*j).child=ext::HSD_IDGetDataFromTable(null_mut(),(*d).child as usize as u64,null_mut()).cast();
        assert!(!(*j).child.is_null()); ref_inc((*j).child);
    }
    if dobj_union(j){ext::HSD_DObjResolveRefsAll((*j).u.dobj,(*d).u.dobjdesc);}
}
pub unsafe fn HSD_JObjResolveRefsAll(mut j:*mut HSD_JObj,mut d:*mut HSD_Joint) {
    while !j.is_null()&&!d.is_null(){HSD_JObjResolveRefs(j,d);
        if (*j).flags&JOBJ_INSTANCE==0 {HSD_JObjResolveRefsAll((*j).child,(*d).child);}
        j=(*j).next; d=(*d).next;
    }
}
pub unsafe fn HSD_JObjUnref(j:*mut HSD_JObj) {
    if j.is_null(){return}
    let n=(*j).object.ref_count;
    if n!=65535 {(*j).object.ref_count=n.wrapping_sub(1);}
    if n==0||n==65535 {
        if (*j).object.ref_count_individual==0 {hsd_delete_ext(j);}
        else {(*j).object.ref_count_individual=(*j).object.ref_count_individual.wrapping_add(1);
            assert_ne!((*j).object.ref_count_individual,0);
            ((*info(j)).release_child.expect("release_child"))(j);
            if iref_dec(&mut (*j).object.ref_count_individual){hsd_delete_ext(j);}
        }
    }
}
pub unsafe fn HSD_JObjUnrefThis(j:*mut HSD_JObj) {
    if !j.is_null()&&iref_dec(&mut (*j).object.ref_count_individual)&&(*j).object.ref_count==65535 {hsd_delete_ext(j);}
}
pub unsafe fn HSD_JObjSetCurrent(j:*mut HSD_JObj){ref_inc(j);HSD_JObjUnref(current_jobj);current_jobj=j;}
pub unsafe fn HSD_JObjGetCurrent()->*mut HSD_JObj{current_jobj}
pub unsafe extern "C" fn JObjInit(o:*mut _HSD_Class)->i32 {
    let status=((*hsdJObj.parent.parent.head.parent).init.expect("parent init"))(o);
    if status<0{return status}
    let j=o.cast::<HSD_JObj>(); (*j).flags=JOBJ_MTX_DIRTY;
    (*j).scale=melee_types::Vec{x:1.,y:1.,z:1.}; 0
}
pub unsafe extern "C" fn JObjReleaseChild(j:*mut HSD_JObj) {
    if !(*j).child.is_null(){
        if (*j).flags&JOBJ_INSTANCE!=0 {HSD_JObjUnref((*j).child);}
        else {(*(*j).child).parent=null_mut(); native_remove_all((*j).child);}
        (*j).child=null_mut();
    }
    if !(*j).parent.is_null(){native_reparent(j,null_mut());}
    if dobj_union(j)&&!(*j).u.dobj.is_null(){ext::HSD_DObjRemoveAll((*j).u.dobj);(*j).u.dobj=null_mut();}
    if !(*j).robj.is_null(){ext::HSD_RObjRemoveAll((*j).robj);(*j).robj=null_mut();}
    if !(*j).aobj.is_null(){ext::HSD_AObjRemove((*j).aobj);(*j).aobj=null_mut();}
}
pub unsafe extern "C" fn JObjRelease(o:*mut _HSD_Class) {
    let j=o.cast::<HSD_JObj>(); ((*info(j)).release_child.expect("release_child"))(j);
    if ext::HSD_IDGetDataFromTable(null_mut(),(*j).id as u64,null_mut())==j.cast() {ext::HSD_IDRemoveByIDFromTable(null_mut(),(*j).id as u64);}
    if !(*j).scl.is_null(){ext::HSD_VecFree((*j).scl.cast());}
    if !(*j).envelopemtx.is_null(){ext::HSD_MtxFree((*j).envelopemtx.cast());}
    ((*hsdJObj.parent.parent.head.parent).release.expect("parent release"))(o);
}
pub unsafe extern "C" fn JObjAmnesia(i:*mut _HSD_ClassInfo) {
    if i==default_class{default_class=null_mut();}
    if i==(&raw mut hsdJObj).cast(){ufc_callbacks=null_mut();current_jobj=null_mut();}
    ((*hsdJObj.parent.parent.head.parent).amnesia.expect("parent amnesia"))(i);
}
unsafe extern "C" fn make_pmtx(j:*mut HSD_JObj,v:*mut [f32;4],p:*mut [f32;4]){ext::HSD_JObjMakePositionMtx(j,v.cast(),p.cast())}
unsafe extern "C" fn disp_sub(j:*mut HSD_JObj,v:*mut [f32;4],p:*mut [f32;4],f:u32,r:u32){ext::HSD_JObjDispSub(j,v.cast(),p.cast(),f,r)}
pub unsafe extern "C" fn JObjInfoInit() {
    ext::hsdInitClassInfo((&raw mut hsdJObj).cast(),hsd_obj_ptr(),c"sysdolphin_base_library".as_ptr().cast_mut(),c"hsd_jobj".as_ptr().cast_mut(),size_of::<HSD_JObjInfo>() as i32,size_of::<HSD_JObj>() as i32);
    hsdJObj.parent.parent.init=Some(JObjInit);hsdJObj.parent.parent.release=Some(JObjRelease);hsdJObj.parent.parent.amnesia=Some(JObjAmnesia);
    hsdJObj.make_mtx=Some(native_make_matrix);hsdJObj.make_pmtx=Some(make_pmtx);hsdJObj.disp=Some(disp_sub);hsdJObj.load=Some(JObjLoad);hsdJObj.release_child=Some(JObjReleaseChild);
}

// The tuple is adapter state, not a second C record. Every game record comes from melee_types.
type NativeTree = JObjTree<*mut HSD_DObj, *mut HSD_RObj>;
type NativeHost = (std::vec::Vec<*mut HSD_JObj>, std::vec::Vec<JObjId>);
fn vid(v:melee_types::Vec)->crate::mtx::Vec3{crate::mtx::Vec3{x:v.x,y:v.y,z:v.z}}
fn native_vec(v:crate::mtx::Vec3)->melee_types::Vec{melee_types::Vec{x:v.x,y:v.y,z:v.z}}
unsafe fn snapshot(roots:&[*mut HSD_JObj])->(NativeTree,NativeHost){
    let mut p=roots.iter().copied().filter(|j|!j.is_null()).collect::<std::vec::Vec<_>>(); p.dedup();
    let mut k=0;
    while k<p.len(){let j=p[k];
        let mut links=vec![(*j).parent,(*j).child,(*j).next];
        if !(*j).aobj.is_null(){links.push((*(*j).aobj).hsd_obj.cast());}
        let mut r=(*j).robj;
        while !r.is_null(){if (*r).flags&0x70000000==0x10000000{links.push((*r).u.jobj);}r=(*r).next;}
        for link in links {if !link.is_null()&&!p.contains(&link){p.push(link);}}
        k+=1;
    }
    let id=|j|p.iter().position(|q|*q==j).map(|n|JObjId(n as u32));
    let mut t=NativeTree::new();
    for &j in &p{
        let x=t.alloc();let o=t.get_mut(x);
        o.parent=id((*j).parent);o.child=id((*j).child);o.next=id((*j).next);o.flags=(*j).flags;
        o.scale=vid((*j).scale);o.translate=vid((*j).translate);
        o.rotate=crate::mtx::Quaternion{x:(*j).rotate.x,y:(*j).rotate.y,z:(*j).rotate.z,w:(*j).rotate.w};o.mtx=(*j).mtx;
        o.scl=(*j).scl.as_ref().map(|s|vid(*s));
        if !(*j).aobj.is_null(){let mut a=crate::aobj::HsdAObj::alloc();a.hsd_obj=id((*(*j).aobj).hsd_obj.cast());o.aobj=Some(a);}
        if dobj_union(j){let mut d=(*j).u.dobj;while !d.is_null(){o.dobj.push(d);d=(*d).next;}}
        let mut r=(*j).robj;while !r.is_null(){o.robj.push(r);r=(*r).next;}
    }
    (t,(p,vec![]))
}
unsafe fn flush(t:&NativeTree,p:&[*mut HSD_JObj]){
    let ptr=|id:Option<JObjId>|id.map_or(null_mut(),|i|p[i.0 as usize]);
    for (n,&j) in p.iter().enumerate(){let o=t.get(JObjId(n as u32));
        (*j).parent=ptr(o.parent);(*j).child=ptr(o.child);(*j).next=ptr(o.next);(*j).flags=o.flags;
        (*j).scale=native_vec(o.scale);(*j).translate=native_vec(o.translate);
        (*j).rotate=melee_types::Quaternion{x:o.rotate.x,y:o.rotate.y,z:o.rotate.z,w:o.rotate.w};(*j).mtx=o.mtx;
        match o.scl {Some(s)=>{if (*j).scl.is_null(){(*j).scl=Box::into_raw(Box::new(native_vec(s)));}else{*(*j).scl=native_vec(s);}},None=>{if !(*j).scl.is_null(){ext::HSD_VecFree((*j).scl.cast());(*j).scl=null_mut();}}}
        if dobj_union(j){(*j).u.dobj=o.dobj.first().copied().unwrap_or(null_mut());for (k,&d) in o.dobj.iter().enumerate(){(*d).next=o.dobj.get(k+1).copied().unwrap_or(null_mut());}}
        (*j).robj=o.robj.first().copied().unwrap_or(null_mut());for(k,&r)in o.robj.iter().enumerate(){(*r).next=o.robj.get(k+1).copied().unwrap_or(null_mut());}
    }
}
impl JObjHost<*mut HSD_DObj,*mut HSD_RObj> for NativeHost {
    fn ik_hint_bone_length(&mut self,r:&[*mut HSD_RObj])->Option<f32>{unsafe{r.iter().find(|&&r|(*r).flags&0xf0000000==0xc0000000).map(|&r|(*r).u.ik_hint.bone_length)}}
    fn robj_update_all(&mut self,t:&mut NativeTree,j:JObjId){unsafe{flush(t,&self.0);let p=self.0[j.0 as usize];ext::HSD_RObjUpdateAll((*p).robj,p.cast(),Some(crate::tu::jobj_ptr::JObjUpdateFunc));*t=snapshot(&self.0).0;}}
    fn resolve_ik_joint1(&mut self,t:&mut NativeTree,j:JObjId){unsafe{native_ik(t,&self.0,j,true)}}
    fn resolve_ik_joint2(&mut self,t:&mut NativeTree,j:JObjId){unsafe{native_ik(t,&self.0,j,false)}}
    fn unref(&mut self,_:&mut NativeTree,j:JObjId){self.1.push(j);}
}
unsafe fn finish(t:&NativeTree,h:NativeHost){flush(t,&h.0);for j in h.1{HSD_JObjUnref(h.0[j.0 as usize]);}}
unsafe fn native_remove_all(j:*mut HSD_JObj){if !j.is_null(){let(mut t,mut h)=snapshot(&[j]);t.remove_all(&mut h,Some(JObjId(0)));finish(&t,h);}}
unsafe fn native_reparent(j:*mut HSD_JObj,p:*mut HSD_JObj){let(mut t,h)=snapshot(&[j,p]);let pi=h.0.iter().position(|&x|x==p).map(|i|JObjId(i as u32));t.reparent(Some(JObjId(0)),pi);finish(&t,h);}
unsafe extern "C" fn native_make_matrix(j:*mut HSD_JObj){let(mut t,mut h)=snapshot(&[j]);t.make_matrix(&mut h,JObjId(0));finish(&t,h);}
unsafe fn native_setup(j:*mut HSD_JObj){if !j.is_null(){let(mut t,mut h)=snapshot(&[j]);t.setup_matrix(&mut h,Some(JObjId(0)));finish(&t,h);}}
pub unsafe fn HSD_JObjDispAll(j:*mut HSD_JObj,v:&Mtx,flags:u32,mode:u32){
    if j.is_null(){return}
    if (*j).flags&JOBJ_INSTANCE!=0{
        if (*j).flags&JOBJ_HIDDEN==0{
            native_setup(j);native_setup((*j).child);
            let inv=crate::mtx::inverse(&(*(*j).child).mtx).unwrap_or([[0.;4];3]);
            let m=crate::mtx::concat(&(*j).mtx,&inv);let c=ext::HSD_CObjGetCurrent();assert!(!c.is_null());
            let m=crate::mtx::concat(&(*c).view_mtx,&m);HSD_JObjDispAll((*j).child,&m,flags,mode);
        }
    }else{
        if (*j).flags&flags.wrapping_shl(18)!=0{ext::HSD_JObjDisp(j,(v as *const Mtx as *mut Mtx).cast(),flags,mode);}
        if (*j).flags&flags.wrapping_shl(28)!=0{let mut c=(*j).child;while !c.is_null(){HSD_JObjDispAll(c,v,flags,mode);c=(*c).next;}}
    }
}

/// Native constraint bridge calls the already-port-ed IK kernels. Cross-TU expression updates
/// still enter through HSD_RObjUpdateAll; IK hint/limit/joint records are copied by field.
unsafe fn native_ik(t:&mut NativeTree,p:&[*mut HSD_JObj],j:JObjId,first:bool){
    let mut nt=JObjTree::<*mut HSD_DObj,u32>::new();let mut rt=JObjRt::new(p.to_vec());
    for o in &t.nodes{
        let n=nt.alloc();let d=nt.get_mut(n);
        d.next=o.next;d.child=o.child;d.parent=o.parent;d.flags=o.flags;d.rotate=o.rotate;d.scale=o.scale;d.translate=o.translate;d.mtx=o.mtx;d.scl=o.scl;d.dobj=o.dobj.clone();d.aobj=o.aobj.clone();
        for &r in &o.robj{
            let mut v=crate::robj::HsdRObj::default();v.flags=(*r).flags;
            match v.flags&0x70000000{
                0x10000000=>v.u.jobj=p.iter().position(|&x|x==(*r).u.jobj).map(|i|JObjId(i as u32)),
                0x20000000=>v.u.limit=(*r).u.limit,
                0x40000000=>{v.u.ik_hint.bone_length=(*r).u.ik_hint.bone_length;v.u.ik_hint.rotate_x=(*r).u.ik_hint.rotate_x;},
                _=>panic!("TODO cross-TU: native HSD_RObj expression bridge"),
            }
            d.robj.push(rt.robjs.len() as u32);rt.robjs.push(v);
        }
    }
    if first{rt.resolve_ik_joint1(&mut nt,j)}else{rt.resolve_ik_joint2(&mut nt,j)}
    for (o,n) in t.nodes.iter_mut().zip(nt.nodes){o.flags=n.flags;o.mtx=n.mtx;o.scale=n.scale;o.translate=n.translate;o.rotate=n.rotate;o.scl=n.scl;}
}
impl crate::robj::RObjHost for std::vec::Vec<*mut HSD_JObj>{
    fn jobj_ref_this(&mut self,j:JObjId){unsafe{let o=&mut (*self[j.0 as usize]).object;o.ref_count_individual=o.ref_count_individual.wrapping_add(1);assert_ne!(o.ref_count_individual,0)}}
    fn jobj_unref_this(&mut self,j:Option<JObjId>){unsafe{HSD_JObjUnrefThis(j.map_or(null_mut(),|j|self[j.0 as usize]))}}
    fn byte_code_eval(&mut self,b:Option<&[u8]>,a:&[f32],n:u32)->f32{ext_native::HSD_ByteCodeEval(b,a,n)}
}
impl JObjExt for std::vec::Vec<*mut HSD_JObj>{
    type D=*mut HSD_DObj;type MatAnim=();type ShapeAnim=();
    fn dobj_remove_anim_all_by_flags(&mut self,d:&[Self::D],f:u32){ext_native::HSD_DObjRemoveAnimAllByFlags(d,f)}
    fn dobj_req_anim_all_by_flags(&mut self,d:&[Self::D],x:f32,f:u32){ext_native::HSD_DObjReqAnimAllByFlags(d,x,f)}
    fn dobj_add_anim_all(&mut self,d:&[Self::D],m:Option<&()>,s:Option<&()>){ext_native::HSD_DObjAddAnimAll(d,m,s)}
    fn dobj_anim_all(&mut self,d:&[Self::D]){ext_native::HSD_DObjAnimAll(d)}
    fn spline_point(&mut self,j:JObjId,s:f32)->crate::mtx::Vec3{ext_native::splArcLengthPoint(self[j.0 as usize],s)}
    fn aobj_resolve_obj(&mut self,id:u32)->Option<JObjId>{unsafe{let p=ext::HSD_IDGetDataFromTable(null_mut(),id as usize as u64,null_mut()).cast::<HSD_JObj>();self.iter().position(|&j|j==p).map(|i|JObjId(i as u32))}}
    fn jobj_ref(&mut self,j:Option<JObjId>){unsafe{ref_inc(j.map_or(null_mut(),|j|self[j.0 as usize]))}}
    fn jobj_unref(&mut self,j:Option<JObjId>){unsafe{HSD_JObjUnref(j.map_or(null_mut(),|j|self[j.0 as usize]))}}
}




// C-name adapters to the established arena API. They preserve explicit host ownership;
// no second implementation of tree, animation or IK arithmetic is introduced.
pub fn HSD_JObjCheckDepend<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>)->(){t.check_depend(j)}
pub fn HSD_JObjSetMtxDirtySub<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:JObjId)->(){t.set_mtx_dirty_sub(j)}
pub fn RecalcParentTrspBits<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>)->(){t.recalc_parent_trsp_bits(j)}
pub fn UpdateParentTrspBits<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,c:JObjId)->(){t.update_parent_trsp_bits(j,c)}
pub fn HSD_JObjAddChild<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,c:Option<JObjId>)->(){t.add_child(j,c)}
pub fn HSD_JObjReparent<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,p:Option<JObjId>)->Option<JObjId>{t.reparent(j,p)}
pub fn HSD_JObjAddNext<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,n:Option<JObjId>)->(){t.add_next(j,n)}
pub fn HSD_JObjGetPrev<D: Copy + PartialEq, R: Copy + PartialEq>(t:&JObjTree<D,R>,j:Option<JObjId>)->Option<JObjId>{t.get_prev(j)}
pub fn HSD_JObjGetDObj<D: Copy + PartialEq, R: Copy + PartialEq>(t:&JObjTree<D,R>,j:Option<JObjId>)->Option<D>{t.get_dobj(j)}
pub fn HSD_JObjAddDObj<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,d:Option<D>)->(){t.add_dobj(j,d)}
pub fn HSD_JObjPrependRObj<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,r:Option<R>)->(){t.prepend_robj(j,r)}
pub fn HSD_JObjDeleteRObj<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,r:Option<R>)->(){t.delete_robj(j,r)}
pub fn HSD_JObjGetFlags<D: Copy + PartialEq, R: Copy + PartialEq>(t:&JObjTree<D,R>,j:Option<JObjId>)->u32{t.get_flags(j)}
pub fn HSD_JObjSetFlags<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,f:u32)->(){t.set_flags(j,f)}
pub fn HSD_JObjSetFlagsAll<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,f:u32)->(){t.set_flags_all(j,f)}
pub fn HSD_JObjClearFlags<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,f:u32)->(){t.clear_flags(j,f)}
pub fn HSD_JObjClearFlagsAll<D: Copy + PartialEq, R: Copy + PartialEq>(t:&mut JObjTree<D,R>,j:Option<JObjId>,f:u32)->(){t.clear_flags_all(j,f)}
pub fn HSD_JObjWalkTree<D: Copy + PartialEq, R: Copy + PartialEq>(t:&JObjTree<D,R>,j:Option<JObjId>,cb:&mut dyn FnMut(JObjId,u32))->(){t.walk_tree(j,cb)}
pub fn HSD_JObjWalkTree0<D: Copy + PartialEq, R: Copy + PartialEq>(t:&JObjTree<D,R>,j:JObjId,cb:&mut dyn FnMut(JObjId,u32))->(){t.walk_tree0(j,cb)}
pub fn JObjResetRST<D: Copy + PartialEq, R: Copy + PartialEq,S:JointSource>(t:&mut JObjTree<D,R>,j:Option<JObjId>,s:&S,d:Option<S::Id>){t.reset_rst_one(j,s,d)}
pub fn HSD_JObjResetRST<D: Copy + PartialEq, R: Copy + PartialEq,S:JointSource>(t:&mut JObjTree<D,R>,j:Option<JObjId>,s:&S,d:Option<S::Id>){t.reset_rst(j,s,d)}
pub fn HSD_JObjMakeMatrix<D: Copy + PartialEq, R: Copy + PartialEq,H:JObjHost<D,R>>(t:&mut JObjTree<D,R>,h:&mut H,j:JObjId)->(){t.make_matrix(h,j)}
pub fn HSD_JObjSetupMatrixSub<D: Copy + PartialEq, R: Copy + PartialEq,H:JObjHost<D,R>>(t:&mut JObjTree<D,R>,h:&mut H,j:JObjId)->(){t.setup_matrix_sub(h,j)}
pub fn HSD_JObjRemove<D: Copy + PartialEq, R: Copy + PartialEq,H:JObjHost<D,R>>(t:&mut JObjTree<D,R>,h:&mut H,j:Option<JObjId>)->Option<JObjId>{t.remove(h,j)}
pub fn HSD_JObjRemoveAll<D: Copy + PartialEq, R: Copy + PartialEq,H:JObjHost<D,R>>(t:&mut JObjTree<D,R>,h:&mut H,j:Option<JObjId>)->(){t.remove_all(h,j)}
pub fn HSD_JObjRemoveAnimByFlags<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,f:u32){r.remove_anim_by_flags(t,j,f)}
pub fn HSD_JObjRemoveAnimAllByFlags<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,f:u32){r.remove_anim_all_by_flags(t,j,f)}
pub fn HSD_JObjRemoveAnim<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>){r.remove_anim(t,j)}
pub fn HSD_JObjRemoveAnimAll<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>){r.remove_anim_all(t,j)}
pub fn HSD_JObjReqAnimByFlags<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,f:u32,x:f32){r.req_anim_by_flags(t,j,f,x)}
pub fn HSD_JObjReqAnimAllByFlags<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,f:u32,x:f32){r.req_anim_all_by_flags(t,j,f,x)}
pub fn HSD_JObjReqAnim<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,x:f32){r.req_anim(t,j,x)}
pub fn HSD_JObjReqAnimAll<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,x:f32){r.req_anim_all(t,j,x)}
pub fn HSD_JObjAnim<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>){r.anim(t,j)}
pub fn JObjAnimAll<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>){r.anim_all_sub(t,j)}
pub fn HSD_JObjAnimAll<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>){r.anim_all(t,j)}
pub fn HSD_JObjAddAnim<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,a:Option<&crate::jobj_rt::HsdAnimJoint>,m:Option<&crate::jobj_rt::HsdSubAnimJoint<X::MatAnim>>,s:Option<&crate::jobj_rt::HsdSubAnimJoint<X::ShapeAnim>>){r.add_anim(t,j,a,m,s)}
pub fn HSD_JObjAddAnimAll<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,a:Option<&crate::jobj_rt::HsdAnimJoint>,m:Option<&crate::jobj_rt::HsdSubAnimJoint<X::MatAnim>>,s:Option<&crate::jobj_rt::HsdSubAnimJoint<X::ShapeAnim>>){r.add_anim_all(t,j,a,m,s)}
pub fn JObjUpdateFunc<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:Option<JObjId>,ty:u32,v:Option<&mut crate::jobj_rt::ObjData>){r.update_func(t,j,ty,v)}
pub fn resolveIKJoint1<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:JObjId){r.resolve_ik_joint1(t,j)}
pub fn resolveIKJoint2<X:JObjExt>(r:&mut JObjRt<X>,t:&mut JObjTree<X::D,u32>,j:JObjId){r.resolve_ik_joint2(t,j)}
pub fn jobj_get_effector_checked<X:JObjExt>(r:&JObjRt<X>,t:&JObjTree<X::D,u32>,j:Option<JObjId>)->Option<JObjId>{r.effector_checked(t,j)}
pub fn HSD_JObjSetDPtclCallback<X:JObjExt>(r:&mut JObjRt<X>,cb:Option<crate::jobj_rt::DPtclCallback>){r.set_dptcl_callback(cb)}

/// Raw-pointer entry points over the native bridge above (callers hold game `HSD_JObj*`).
/// `HSD_JObjSetupMatrix` (jobj.h inline): rebuild the matrix when dirty.
pub unsafe fn HSD_JObjSetupMatrix(j: *mut HSD_JObj) { native_setup(j) }
/// `HSD_JObjRemoveAll` on a raw tree.
pub unsafe fn HSD_JObjRemoveAll_ptr(j: *mut HSD_JObj) { native_remove_all(j) }
/// `HSD_JObjRemove` on a raw JObj: unlink it from its tree and drop its reference.
pub unsafe fn HSD_JObjRemove_ptr(j: *mut HSD_JObj) { if !j.is_null() { let (mut t, mut h) = snapshot(&[j]); t.remove(&mut h, Some(JObjId(0))); finish(&t, h); } }

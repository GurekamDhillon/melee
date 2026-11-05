//! `src/sysdolphin/baselib/pobj.c`, the display half: `HSD_PObjDisp`, `PObjSetupMtx`
//! (`SetupRigidModelMtx` / `SetupSharedVtxModelMtx` / `SetupEnvelopeModelMtx`), `setupArrayDesc`,
//! `setupVtxDesc`, `HSD_ClearVtxDesc`, `PObjDispSimplePrimitive`. (The logic half is the verified
//! port in `crate::pobj`.)
//!
//! Descriptor-driven like the rest of the draw path: `pobj` is the `HSD_PObjDesc` offset (+0x4
//! next, +0x8 verts, +0xC flags u16, +0xE n_display u16, +0x10 display, +0x14 jobj / shape set /
//! envelope list). Envelope joints are resolved through [`DispModel::by_desc`].
//!
//! Not ported: shape-animated PObjs (`PObjDispShapeAnim` / `drawShapeAnim`: counted in
//! `stats.shape_anim_skipped` and not drawn), the PC envelope cache / geno palette paths (both
//! produce the same matrices), perf counters.
use super::displayfunc::{DispModel, JOBJ_LIGHTING};
use super::state::gx::*;
use super::state::{GxBackend, HsdRender};
use super::tobj::{TEX_COORD_HILIGHT, TEX_COORD_REFLECTION};
use crate::hsd_mtx;
use crate::mtx::{self, Mtx};

// Native C entry points are independent of the descriptor renderer above/below.
pub use native::*;

#[allow(non_snake_case, non_upper_case_globals, unused_variables, unused_imports, static_mut_refs)]
pub mod native {
use melee_types::*;
use core::{ffi::c_void, mem::size_of, ptr::{null_mut, addr_of_mut}};
use melee_core::fp::fmadds;
type MP = *mut [f32; 4];
unsafe fn hsd_class_ptr() -> *mut HSD_ClassInfo { if use_ext_data { addr_of_mut!(ext::hsdClass) } else { addr_of_mut!(crate::tu::class::hsdClass) } }
/// The generated hooks take `Mtx` parameters as `void*`; the port keeps `MtxPtr`.
pub mod mx {
    use super::*;
    pub unsafe fn GXLoadNrmMtxImm(m: MP, n: u32) { ext::GXLoadNrmMtxImm(m.cast(), n) }
    pub unsafe fn GXLoadPosMtxImm(m: MP, n: u32) { ext::GXLoadPosMtxImm(m.cast(), n) }
    pub unsafe fn GXLoadTexMtxImm(m: MP, n: u32, t: u32) { ext::GXLoadTexMtxImm(m.cast(), n, t) }
    pub unsafe fn _HSD_mkEnvelopeModelNodeMtx(j: *mut HSD_JObj, m: MP) -> MP { ext::_HSD_mkEnvelopeModelNodeMtx(j, m.cast()).cast() }
}
/// perf.h `HSD_PerfCountMtxLoad` (static inline): `HSD_PerfCurrentStat.nb_mtx_load += 1`.
pub static mut use_ext_data: bool = false;
pub unsafe fn HSD_PerfCountMtxLoad() {
    melee_core::ct!("HSD_PerfCountMtxLoad");
    let st = if use_ext_data { addr_of_mut!(ext::HSD_PerfCurrentStat) } else { addr_of_mut!(crate::tu::perf::HSD_PerfCurrentStat) };
    (*st).nb_mtx_load = (*st).nb_mtx_load.wrapping_add(1);
}
/// jobj.h `HSD_JObjSetupMatrix` (inline): rebuild when dirty and not user-defined.
pub unsafe fn jobj_setup_matrix(j: *mut HSD_JObj) {
    melee_core::ct!("jobj_setup_matrix");
    if j.is_null() { return; }
    let fl = (*j).flags;
    if fl & (1 << 23) == 0 && fl & (JOBJ_MTX_DIRTY as u32) != 0 { ext::HSD_JObjSetupMatrixSub(j); }
}

/// Missing native callees retain their C names and fail explicitly until installed.
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
    fn GXAuroraEndPalette();
    fn GXAuroraLoadPalette(a0: u32, a1: u32, a2: *const f32);
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXCallDisplayList(a0: *mut c_void, a1: u32);
    fn GXClearVtxDesc();
    fn GXColor1u16(a0: u16);
    fn GXColor1x16(a0: u16);
    fn GXColor1x8(a0: u8);
    fn GXColor3u8(a0: u8, a1: u8, a2: u8);
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXLoadNrmMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadTexMtxImm(a0: *mut c_void, a1: u32, a2: u32);
    fn GXNormal3f32(a0: f32, a1: f32, a2: f32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetArray(a0: u32, a1: *const c_void, a2: u8);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXTexCoord1u8(a0: u8);
    fn GXTexCoord1x16(a0: u16);
    fn GXTexCoord1x8(a0: u8);
    fn GenoPalCacheGet(a0: u32, a1: u32, a2: u32, a3: u32, a4: *mut f32) -> i32;
    fn GenoPalCachePut(a0: u32, a1: u32, a2: u32, a3: u32, a4: *const f32);
    fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
    fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj;
    fn HSD_AObjRemove(a0: *mut HSD_AObj);
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32);
    fn HSD_Free(a0: *mut c_void);
    fn HSD_IDGetDataFromTable(a0: *mut _HSD_IDTable, a1: u64, a2: *mut i32) -> *mut c_void;
    fn HSD_Index2PosNrmMtx(a0: u32) -> i32;
    fn HSD_Index2TexMtx(a0: u32) -> u32;
    fn HSD_JObjGetCurrent() -> *mut HSD_JObj;
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_JObjUnrefThis(a0: *mut HSD_JObj);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn HSD_MtxInverseTranspose(a0: *mut c_void, a1: *mut c_void) = { let r = crate::hsd_mtx::inverse_transpose(&*(a0 as *const [[f32; 4]; 3])); (a1 as *mut [[f32; 4]; 3]).write(r); };
    fn HSD_MtxScaledAdd(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void, a3: f32) = { let r = crate::hsd_mtx::scaled_add(&*(a0 as *const [[f32; 4]; 3]), &*(a1 as *const [[f32; 4]; 3]), a3); (a2 as *mut [[f32; 4]; 3]).write(r); };
    fn HSD_PerfCountEnvelopeBlending(a0: i32);
    fn HSD_SListAlloc() -> *mut _HSD_SList;
    fn HSD_SListRemove(a0: *mut _HSD_SList) -> *mut _HSD_SList;
    fn HSD_StateSetCullMode(a0: i32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _);
    fn ProfActive() -> i32;
    fn ProfBegin(a0: i32, a1: i32);
    fn ProfCount(a0: i32);
    fn ProfEnd();
    fn _HSD_TObjGetCurrentByType(a0: *mut HSD_TObj, a1: u32) -> *mut HSD_TObj;
    fn _HSD_mkEnvelopeModelNodeMtx(a0: *mut HSD_JObj, a1: *mut c_void) -> *mut c_void = crate::tu::displayfunc_native::_HSD_mkEnvelopeModelNodeMtx(a0, a1 as _) as *mut c_void;
    fn diag_env_slot(a0: u32, a1: u32, a2: i32);
    fn diag_envelope(a0: i32, a1: f32);
    fn diag_geno_pal(a0: i32, a1: i32, a2: i32);
    fn diag_geno_pal_compare(a0: i32, a1: i32, a2: i32, a3: i32);
    fn diag_geno_pal_force() -> i32;
    fn diag_geno_pal_selftest() -> i32;
    fn diag_pobj_draw(a0: u32, a1: i32, a2: i32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn diag_pobj_enabled() -> i32;
    fn env_cache_compare(a0: u32, a1: u32, a2: u32, a3: u32, a4: *const c_void, a5: *const c_void, a6: i32);
    fn env_cache_get(a0: u32, a1: u32, a2: u32, a3: u32, a4: *mut c_void, a5: *mut c_void, a6: i32) -> i32;
    fn env_cache_mode() -> i32;
    fn env_cache_put(a0: u32, a1: u32, a2: u32, a3: u32, a4: *const c_void, a5: *const c_void, a6: i32);
    fn geno_stun_frame() -> i32;
    fn hsdAllocMemPiece(a0: i32) -> *mut c_void;
    fn hsdFreeMemPiece(a0: *mut c_void, a1: i32);
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32);
    fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32;
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void;
    fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo;
    fn memcmp(a0: *const c_void, a1: *const c_void, a2: u32) -> i32 = { let (x, y) = (core::slice::from_raw_parts(a0 as *const u8, a2 as usize), core::slice::from_raw_parts(a1 as *const u8, a2 as usize)); match x.cmp(y) { core::cmp::Ordering::Less => -1, core::cmp::Ordering::Equal => 0, core::cmp::Ordering::Greater => 1 } };
    fn strcmp(a0: *const i8, a1: *const i8) -> i32 = { match std::ffi::CStr::from_ptr(a0).to_bytes().cmp(std::ffi::CStr::from_ptr(a1).to_bytes()) { core::cmp::Ordering::Less => -1, core::cmp::Ordering::Equal => 0, core::cmp::Ordering::Greater => 1 } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 { super::hsdIsDescendantOf(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObj_804D7814;
    pub use crate::tu::perf::HSD_PerfCurrentStat;
    pub static mut gx_suppress_draws: i32 = unsafe { core::mem::zeroed() };
    pub use crate::tu::class::hsdClass;
}
// END GENERATED EXT
pub static mut hsdPObj:HSD_PObjInfo={let mut i:HSD_PObjInfo=unsafe {core::mem::zeroed()};i.parent.head.info_init=Some(PObjInfoInit);i};
pub static mut genoPalPObj:HSD_PObjInfo={let mut i:HSD_PObjInfo=unsafe {core::mem::zeroed()};i.parent.head.info_init=Some(GenoPalInfoInit);i};
pub static mut default_class:*mut HSD_PObjInfo=null_mut();
pub static mut vertex_buffer:*mut [f32;3]=null_mut();
pub static mut normal_buffer:*mut [f32;3]=null_mut();
pub static mut vertex_buffer_size:u32=0;
pub static mut normal_buffer_size:u32=0;
pub static mut prev_vtxdesclist_array:*mut HSD_VtxDescList=null_mut();
pub static mut prev_vtxdesc:*mut HSD_VtxDescList=null_mut();
#[repr(C)] #[derive(Clone,Copy)] pub struct MtxMark{pub obj:*mut c_void,pub mark:u32}
pub static mut mtx_mark:[MtxMark;2]=[MtxMark{obj:null_mut(),mark:0};2];
pub static mut geno_pal_selftest_on:i32=-1;
pub static mut geno_pal_legacy:[[f32;24];10]=[[0.;24];10];
pub static mut geno_pal_legacy_nrm:[u8;10]=[0;10];
pub static mut geno_pal_data:[f32;64*24]=[0.;64*24];
pub static mut geno_pal_force_on:i32=-1;
pub static mut geno_pal_forced:[*mut u8;512]=[null_mut();512];
pub static mut geno_pal_nforced:usize=0;
unsafe fn kind(p:*const HSD_PObj)->u16 {(*p).flags&0x3000}
unsafe fn method(p:*mut HSD_PObj)->*mut HSD_PObjInfo {(*p).parent.class_info.cast()}
pub unsafe extern "C" fn HSD_PObjGetFlags(p:*mut HSD_PObj)->u32 { melee_core::ct!("HSD_PObjGetFlags");if p.is_null(){0}else{(*p).flags as u32}}
pub unsafe extern "C" fn HSD_PObjRemoveAnimByFlags(p:*mut HSD_PObj,f:u32) { melee_core::ct!("HSD_PObjRemoveAnimByFlags");if !p.is_null()&&f&8!=0&&kind(p)==0x1000 {let s=(*p).u.shape_set;ext::HSD_AObjRemove((*s).aobj);(*s).aobj=null_mut();}}
pub unsafe extern "C" fn HSD_PObjRemoveAnimAllByFlags(mut p:*mut HSD_PObj,f:u32) { melee_core::ct!("HSD_PObjRemoveAnimAllByFlags");while !p.is_null(){HSD_PObjRemoveAnimByFlags(p,f);p=(*p).next;}}
pub unsafe extern "C" fn HSD_PObjAddAnim(p:*mut HSD_PObj,s:*mut HSD_ShapeAnim) { melee_core::ct!("HSD_PObjAddAnim");if p.is_null(){return}assert!(kind(p)==0x1000&&!(*p).u.shape_set.is_null());if s.is_null(){return}let st=(*p).u.shape_set;if !(*st).aobj.is_null(){ext::HSD_AObjRemove((*st).aobj);}(*st).aobj=ext::HSD_AObjLoadDesc((*s).aobjdesc);}
pub unsafe extern "C" fn HSD_PObjAddAnimAll(mut p:*mut HSD_PObj,mut s:*mut HSD_ShapeAnim) { melee_core::ct!("HSD_PObjAddAnimAll");if s.is_null(){return}while !p.is_null(){HSD_PObjAddAnim(p,s);p=(*p).next;s=if s.is_null(){null_mut()}else{(*s).next};}}
pub unsafe extern "C" fn HSD_PObjReqAnimByFlags(p:*mut HSD_PObj,f:f32,k:u32) { melee_core::ct!("HSD_PObjReqAnimByFlags");if !p.is_null()&&k&8!=0&&kind(p)==0x1000 {ext::HSD_AObjReqAnim((*(*p).u.shape_set).aobj,f);}}
pub unsafe extern "C" fn HSD_PObjReqAnimAllByFlags(mut p:*mut HSD_PObj,f:f32,k:u32) { melee_core::ct!("HSD_PObjReqAnimAllByFlags");while !p.is_null(){HSD_PObjReqAnimByFlags(p,f,k);p=(*p).next;}}
pub unsafe extern "C" fn ShapeSetSetAnimResult(s:*mut HSD_ShapeSet,k:u32,v:*mut HSD_ObjData) { melee_core::ct!("ShapeSetSetAnimResult");if (*s).flags&2!=0 {*(*s).blend.bp.offset(k.wrapping_sub(2) as i32 as isize)=(*v).fv;}else{(*s).blend.bl=(*v).fv;}}
pub unsafe extern "C" fn PObjUpdateFunc(p:*mut c_void,k:i32,v:*mut HSD_ObjData) { melee_core::ct!("PObjUpdateFunc");let p=p.cast::<HSD_PObj>();if !p.is_null()&&kind(p)==0x1000 {ShapeSetSetAnimResult((*p).u.shape_set,k as u32,v);}}
pub unsafe extern "C" fn HSD_PObjAnim(p:*mut HSD_PObj) { melee_core::ct!("HSD_PObjAnim");if !p.is_null()&&kind(p)==0x1000 {ext::HSD_AObjInterpretAnim((*(*p).u.shape_set).aobj,p.cast(),Some(PObjUpdateFunc));}}
pub unsafe extern "C" fn HSD_PObjAnimAll(mut p:*mut HSD_PObj) { melee_core::ct!("HSD_PObjAnimAll");while !p.is_null(){HSD_PObjAnim(p);p=(*p).next;}}
pub unsafe extern "C" fn HSD_EnvelopeAlloc()->*mut HSD_Envelope { melee_core::ct!("HSD_EnvelopeAlloc");let p=ext::hsdAllocMemPiece(size_of::<HSD_Envelope>() as i32).cast::<HSD_Envelope>();assert!(!p.is_null());p.write(HSD_Envelope::zeroed());p}
pub unsafe extern "C" fn HSD_EnvelopeFree(p:*mut HSD_Envelope){ melee_core::ct!("HSD_EnvelopeFree");if !p.is_null(){ext::hsdFreeMemPiece(p.cast(),size_of::<HSD_Envelope>() as i32);}}
pub unsafe extern "C" fn HSD_EnvelopeListFree(mut l:*mut HSD_SList){ melee_core::ct!("HSD_EnvelopeListFree");while !l.is_null(){let mut e=(*l).data.cast::<HSD_Envelope>();while !e.is_null(){let n=(*e).next;ext::HSD_JObjUnrefThis((*e).jobj);HSD_EnvelopeFree(e);e=n;}l=ext::HSD_SListRemove(l);}}
pub unsafe extern "C" fn loadEnvelopeDesc(mut ds:*mut *mut HSD_EnvelopeDesc)->*mut HSD_SList { melee_core::ct!("loadEnvelopeDesc");let mut list=null_mut();let mut lp=&mut list as *mut *mut HSD_SList;if ds.is_null(){return list}while !(*ds).is_null(){let mut e:*mut HSD_Envelope=null_mut();let mut ep=&mut e as *mut *mut HSD_Envelope;let mut d=*ds;while !(*d).joint.is_null(){*ep=HSD_EnvelopeAlloc();(**ep).weight=(*d).weight;ep=addr_of_mut!((**ep).next);d=d.add(1);}*lp=ext::HSD_SListAlloc();(**lp).data=e.cast();lp=addr_of_mut!((**lp).next);ds=ds.add(1);}list}
pub unsafe extern "C" fn HSD_ShapeSetFree(s:*mut HSD_ShapeSet){ melee_core::ct!("HSD_ShapeSetFree");if !s.is_null(){ext::hsdFreeMemPiece(s.cast(),size_of::<HSD_ShapeSet>() as i32);}}
pub unsafe extern "C" fn HSD_ShapeSetRemove(s:*mut HSD_ShapeSet){ melee_core::ct!("HSD_ShapeSetRemove");if s.is_null(){return}if (*s).flags&2!=0 {ext::HSD_Free((*s).blend.bp.cast());}ext::HSD_AObjRemove((*s).aobj);HSD_ShapeSetFree(s);}
pub unsafe extern "C" fn loadShapeSetDesc(d:*mut HSD_ShapeSetDesc)->*mut HSD_ShapeSet { melee_core::ct!("loadShapeSetDesc");let s=ext::hsdAllocMemPiece(size_of::<HSD_ShapeSet>() as i32).cast::<HSD_ShapeSet>();assert!(!s.is_null());s.write(HSD_ShapeSet::zeroed());(*s).flags=(*d).flags;(*s).nb_shape=(*d).nb_shape;(*s).nb_vertex_index=(*d).nb_vertex_index;(*s).vertex_desc=(*d).vertex_desc;(*s).vertex_idx_list=(*d).vertex_idx_list;(*s).nb_normal_index=(*d).nb_normal_index;(*s).normal_desc=(*d).normal_desc;(*s).normal_idx_list=(*d).normal_idx_list;if (*s).flags&2!=0 {(*s).blend.bp=ext::HSD_MemAlloc((*s).nb_shape as i32*4).cast();for i in 0..(*s).nb_shape{*(*s).blend.bp.add(i as usize)=0.;}}else{(*s).blend.bl=0.;}s}
pub unsafe extern "C" fn PObjLoad(p:*mut HSD_PObj,d:*mut HSD_PObjDesc)->i32 { melee_core::ct!("PObjLoad");(*p).next=HSD_PObjLoadDesc((*d).next);(*p).verts=(*d).verts;(*p).flags=(*d).flags;(*p).n_display=(*d).n_display;(*p).display=(*d).display;match kind(p){0x1000=>(*p).u.shape_set=loadShapeSetDesc((*d).u.shape_set),0x2000=>(*p).u.envelope_list=loadEnvelopeDesc((*d).u.envelope_p),0=>(),_=>panic!("pobj: unexpected type")};0}
pub unsafe extern "C" fn HSD_PObjLoadDesc(d:*mut HSD_PObjDesc)->*mut HSD_PObj { melee_core::ct!("HSD_PObjLoadDesc");if d.is_null(){return null_mut()}let p=if geno_pal_desc(d)!=0{ext::hsdNew(addr_of_mut!(genoPalPObj).cast()).cast()}else{let i=if (*d).class_name.is_null(){null_mut()}else{ext::hsdSearchClassInfo((*d).class_name)};if i.is_null(){HSD_PObjAlloc()}else{let p=ext::hsdNew(i).cast::<HSD_PObj>();assert!(!p.is_null());p}};((*method(p)).load.unwrap())(p,d);p}
pub unsafe extern "C" fn HSD_PObjRemove(p:*mut HSD_PObj){ melee_core::ct!("HSD_PObjRemove");if !p.is_null(){let i=method(p).cast::<HSD_ClassInfo>();((*i).release.unwrap())(p.cast());((*method(p)).parent.destroy.unwrap())(p.cast());}}
pub unsafe extern "C" fn HSD_PObjRemoveAll(mut p:*mut HSD_PObj){ melee_core::ct!("HSD_PObjRemoveAll");while !p.is_null(){let n=(*p).next;HSD_PObjRemove(p);p=n;}}
pub unsafe extern "C" fn HSD_PObjGetDefaultClass()->*mut HSD_PObjInfo { melee_core::ct!("HSD_PObjGetDefaultClass");if default_class.is_null(){addr_of_mut!(hsdPObj)}else{default_class}}
pub unsafe extern "C" fn HSD_PObjSetDefaultClass(i:*mut HSD_PObjInfo){ melee_core::ct!("HSD_PObjSetDefaultClass");if !i.is_null(){assert!(ext::hsdIsDescendantOf(i.cast(),addr_of_mut!(hsdPObj).cast())!=0);}default_class=i;}
pub unsafe extern "C" fn HSD_PObjAlloc()->*mut HSD_PObj { melee_core::ct!("HSD_PObjAlloc");let p=ext::hsdNew(HSD_PObjGetDefaultClass().cast()).cast::<HSD_PObj>();assert!(!p.is_null());p}
pub unsafe extern "C" fn HSD_PObjFree(p:*mut HSD_PObj){ melee_core::ct!("HSD_PObjFree");if !p.is_null(){((*method(p)).parent.destroy.unwrap())(p.cast());}}
unsafe fn ref_this(j:*mut HSD_JObj){if !j.is_null(){(*j).object.ref_count_individual=(*j).object.ref_count_individual.wrapping_add(1);assert_ne!((*j).object.ref_count_individual,0);}}
pub unsafe extern "C" fn resolveEnvelope(mut l:*mut HSD_SList,mut ds:*mut *mut HSD_EnvelopeDesc){ melee_core::ct!("resolveEnvelope");if ds.is_null(){return}while !l.is_null()&&!(*ds).is_null(){let mut e=(*l).data.cast::<HSD_Envelope>();let mut d=*ds;while !e.is_null()&&!(*d).joint.is_null(){ext::HSD_JObjUnrefThis((*e).jobj);(*e).jobj=ext::HSD_IDGetDataFromTable(null_mut(),(*d).joint as usize as u64,null_mut()).cast();assert!(!(*e).jobj.is_null());ref_this((*e).jobj);e=(*e).next;d=d.add(1);}l=(*l).next;ds=ds.add(1);}}
pub unsafe extern "C" fn HSD_PObjResolveRefs(p:*mut HSD_PObj,d:*mut HSD_PObjDesc){ melee_core::ct!("HSD_PObjResolveRefs");if p.is_null()||d.is_null(){return}match kind(p){0x2000=>resolveEnvelope((*p).u.envelope_list,(*d).u.envelope_p),0=>{ext::HSD_JObjUnrefThis((*p).u.jobj);(*p).u.jobj=null_mut();if !(*d).u.joint.is_null(){(*p).u.jobj=ext::HSD_IDGetDataFromTable(null_mut(),(*d).u.joint as usize as u64,null_mut()).cast();assert!(!(*p).u.jobj.is_null());ref_this((*p).u.jobj);}},_=>()}}
pub unsafe extern "C" fn HSD_PObjResolveRefsAll(mut p:*mut HSD_PObj,mut d:*mut HSD_PObjDesc){ melee_core::ct!("HSD_PObjResolveRefsAll");while !p.is_null()&&!d.is_null(){HSD_PObjResolveRefs(p,d);p=(*p).next;d=(*d).next;}}
pub unsafe extern "C" fn HSD_ClearVtxDesc(){ melee_core::ct!("HSD_ClearVtxDesc");ext::GXClearVtxDesc();prev_vtxdesclist_array=null_mut();prev_vtxdesc=null_mut();}
pub unsafe extern "C" fn setupArrayDesc(ds:*mut HSD_VtxDescList){ melee_core::ct!("setupArrayDesc");if prev_vtxdesclist_array!=ds{let mut d=ds;while (*d).attr!=255{if (*d).attr_type!=1{ext::GXSetArray((*d).attr,(*d).vertex,(*d).stride as u8);}d=d.add(1);}prev_vtxdesclist_array=ds;}}
pub unsafe extern "C" fn setupVtxDesc(p:*mut HSD_PObj){ melee_core::ct!("setupVtxDesc");if prev_vtxdesc!=(*p).verts{ext::GXClearVtxDesc();let mut d=(*p).verts;while (*d).attr!=255{ext::GXSetVtxDesc((*d).attr,(*d).attr_type);if (*d).attr>8{ext::GXSetVtxAttrFmt(0,(*d).attr,(*d).comp_cnt,(*d).comp_type,(*d).frac);}d=d.add(1);}prev_vtxdesc=(*p).verts;}}
pub unsafe extern "C" fn setupShapeAnimArrayDesc(mut d:*mut HSD_VtxDescList){ melee_core::ct!("setupShapeAnimArrayDesc");while (*d).attr!=255{if (*d).attr_type!=1&&!matches!((*d).attr,9|10|25){ext::GXSetArray((*d).attr,(*d).vertex,(*d).stride as u8);}d=d.add(1);}prev_vtxdesclist_array=null_mut();}
pub unsafe extern "C" fn setupShapeAnimVtxDesc(p:*mut HSD_PObj){ melee_core::ct!("setupShapeAnimVtxDesc");ext::GXClearVtxDesc();let mut d=(*p).verts;while (*d).attr!=255{let shape=matches!((*d).attr,9|10|25);ext::GXSetVtxDesc((*d).attr,if shape{1}else{(*d).attr_type});if (*d).attr>8{ext::GXSetVtxAttrFmt(0,(*d).attr,(*d).comp_cnt,if shape{4}else{(*d).comp_type},if shape{0}else{(*d).frac});}d=d.add(1);}prev_vtxdesc=null_mut();}
macro_rules! decode {($n:ident,$t:ty,$old:ident)=>{pub unsafe extern "C" fn $n(s:*mut c_void,d:*mut f32,scale:i32){let a=crate::pobj::$old(core::slice::from_raw_parts(s.cast::<$t>(),3),scale);core::ptr::copy_nonoverlapping(a.as_ptr(),d,3);}};}
decode!(decode_u8_xyz,u8,decode_u8_xyz);decode!(decode_s8_xyz,u8,decode_s8_xyz);decode!(decode_u16_xyz,u16,decode_u16_xyz);decode!(decode_s16_xyz,u16,decode_s16_xyz);
// GX vertex arrays stay big-endian (the disc layout melee_gx reads), so the shape keys are read as BE here; the C build's
// retargeted loads swap them implicitly.
unsafe fn get_shape(ds:*mut HSD_VtxDescList,ix:*mut *mut u8,shape:i32,index:i32,dst:*mut f32,n:usize){let a=*ix.offset(shape as isize);let idx=if (*ds).attr_type==3{((*a.offset(index as isize*2) as usize)<<8)|*a.offset(index as isize*2+1) as usize}else{*a.offset(index as isize) as usize};let src=(*ds).vertex.cast::<u8>().add(idx*(*ds).stride as usize);let scale=crate::cabi::shl(1,(*ds).frac as u32) as i32 as f32;for i in 0..n{*dst.add(i)=match (*ds).comp_type{0=>*src.add(i) as f32/scale,1=>*src.cast::<i8>().add(i) as f32/scale,2=>u16::from_be(src.cast::<u16>().add(i).read_unaligned()) as f32/scale,3=>i16::from_be(src.cast::<i16>().add(i).read_unaligned()) as f32/scale,4=>f32::from_bits(u32::from_be(src.cast::<u32>().add(i).read_unaligned())),_=>panic!("unexpected vertex type")};}}
pub unsafe extern "C" fn get_shape_vertex_xyz(s:*mut HSD_ShapeSet,k:i32,i:i32,d:*mut f32){ melee_core::ct!("get_shape_vertex_xyz");assert_eq!((*(*s).vertex_desc).comp_cnt,1);get_shape((*s).vertex_desc,(*s).vertex_idx_list,k,i,d,3);}
pub unsafe extern "C" fn get_shape_normal_xyz(s:*mut HSD_ShapeSet,k:i32,i:i32,d:*mut f32){ melee_core::ct!("get_shape_normal_xyz");assert_eq!((*(*s).normal_desc).comp_cnt,0);get_shape((*s).normal_desc,(*s).normal_idx_list,k,i,d,3);}
pub unsafe extern "C" fn get_shape_nbt_xyz(s:*mut HSD_ShapeSet,k:i32,i:i32,d:*mut f32){ melee_core::ct!("get_shape_nbt_xyz");assert_eq!((*(*s).normal_desc).attr,25);assert_eq!((*(*s).normal_desc).comp_cnt,0);get_shape((*s).normal_desc,(*s).normal_idx_list,k,i,d,9);}
pub unsafe extern "C" fn interpretShapeAnimDisplayList(p:*mut HSD_PObj,vertex:*mut [f32;3],normal:*mut [f32;3]){ melee_core::ct!("interpretShapeAnimDisplayList");let mut dl=(*p).display;let mut l=0;let length=(*p).n_display as i32*32;while l+3<length{let n=(*dl.add(1) as u16)<<8|*dl.add(2) as u16;if *dl&0xf8==0{break}ext::GXBegin((*dl&0xf8) as u32,(*dl&7) as u32,n);let mut m=3;for _ in 0..n{let mut d=(*p).verts;while (*d).attr!=255{let mut idx=*dl.add(m) as u16;m+=1;let a=(*d).attr;if a<=8{ext::GXTexCoord1u8(idx as u8);}else{if (*d).attr_type==3{idx=(idx<<8)|*dl.add(m) as u16;m+=1;}match a{9=>{let v=*vertex.add(idx as usize);ext::GXPosition3f32(v[0],v[1],v[2]);},10=>{let v=*normal.add(idx as usize);ext::GXNormal3f32(v[0],v[1],v[2]);},25=>{idx=idx.wrapping_mul(3);for k in 0..3{let v=*normal.add(idx as usize+k);ext::GXNormal3f32(v[0],v[1],v[2]);}},13..=20=>{if (*d).attr_type==3{ext::GXTexCoord1x16(idx)}else{ext::GXTexCoord1x8(idx as u8)}},11|12=>{if (*d).attr_type==3{ext::GXColor1x16(idx)}else if (*d).attr_type==2{ext::GXColor1x8(idx as u8)}else{match (*d).comp_type{0|3=>{ext::GXColor1u16((idx<<8)|*dl.add(m) as u16);m+=1;},1|4=>{ext::GXColor3u8(idx as u8,*dl.add(m),*dl.add(m+1));m+=2;},2|5=>{ext::GXColor4u8(idx as u8,*dl.add(m),*dl.add(m+1),*dl.add(m+2));m+=3;},_=>()}}},_=>crate::tu::lbext::OSReport("attr(%d) is not supported by sysdolphin
",&[crate::tu::lbext::OsArg::I(a as i32)])}}d=d.add(1);}}l+=m as i32;dl=dl.add(m);}}
pub unsafe extern "C" fn drawShapeAnim(p:*mut HSD_PObj){ melee_core::ct!("drawShapeAnim");let s=(*p).u.shape_set;if vertex_buffer_size==0{vertex_buffer_size=2000;vertex_buffer=ext::HSD_MemAlloc(2000*12).cast();}assert!(vertex_buffer_size>=(*s).nb_vertex_index as u32);if !(*s).normal_desc.is_null()&&normal_buffer_size==0{normal_buffer_size=2000;normal_buffer=ext::HSD_MemAlloc(2000*12).cast();}let nbt=!(*s).normal_desc.is_null()&&(*(*s).normal_desc).attr!=10;if !(*s).normal_desc.is_null(){assert!(normal_buffer_size>=((*s).nb_normal_index*if nbt{3}else{1}) as u32);}for normals in [false,true]{let count=if normals{(*s).nb_normal_index}else{(*s).nb_vertex_index};let n=if normals&&nbt{9}else{3};let buf=if normals{normal_buffer.cast::<f32>()}else{vertex_buffer.cast::<f32>()};let get:unsafe extern "C" fn(*mut HSD_ShapeSet,i32,i32,*mut f32)=if !normals{get_shape_vertex_xyz}else if nbt{get_shape_nbt_xyz}else{get_shape_normal_xyz};for i in 0..count{let dst=buf.add(i as usize*n);if (*s).flags&1!=0{let bl=(*s).blend.bl;let k=(melee_core::ppc::fctiwz_f32(bl)).max(0).min((*s).nb_shape as i32-1);let v=bl-k as f32;let b=if 0.0>v{0.0}else{v};let b=if b<1.0{b}else{1.0};let mut s0=[0.;9];let mut s1=[0.;9];get(s,k,i,s0.as_mut_ptr());get(s,(k+1).min((*s).nb_shape as i32-1),i,s1.as_mut_ptr());for j in 0..n{*dst.add(j)=fmadds(s1[j]-s0[j],b,s0[j]);}}else{get(s,0,i,dst);for k in 0..(*s).nb_shape{let v=*(*s).blend.bp.add(k as usize);let b=if 0.0>v{0.0}else{v};let mut tmp=[0.;9];get(s,k as i32+1,i,tmp.as_mut_ptr());for j in 0..n{*dst.add(j)=fmadds(tmp[j],b,*dst.add(j));}}}}}interpretShapeAnimDisplayList(p,vertex_buffer,normal_buffer);}

pub unsafe extern "C" fn HSD_PObjClearMtxMark(o:*mut c_void,m:u32){ melee_core::ct!("HSD_PObjClearMtxMark");mtx_mark=[MtxMark{obj:o,mark:m};2];}
pub unsafe extern "C" fn HSD_PObjSetMtxMark(i:i32,o:*mut c_void,m:u32){ melee_core::ct!("HSD_PObjSetMtxMark");if i>=2{return}if i<0{let e=addr_of_mut!(mtx_mark).cast::<MtxMark>().offset(i as isize);(*e).obj=o;(*e).mark=m;}}
pub unsafe extern "C" fn HSD_PObjGetMtxMark(i:i32,o:*mut *mut c_void,m:*mut u32){ melee_core::ct!("HSD_PObjGetMtxMark");assert!(!o.is_null()&&!m.is_null());if !(0..2).contains(&i){*o=null_mut();*m=0;}else{*o=mtx_mark[i as usize].obj;*m=mtx_mark[i as usize].mark;}}
pub unsafe extern "C" fn GetSetupFlags(j:*mut HSD_JObj,r:u32)->u32{ melee_core::ct!("GetSetupFlags");let mut f=0;if r&0x4000000==0{if (*j).flags&0x80!=0{f|=1;}if !ext::_HSD_TObjGetCurrentByType(null_mut(),1).is_null(){f|=3;}if !ext::_HSD_TObjGetCurrentByType(null_mut(),2).is_null(){f|=5;}}f}
unsafe fn matrix(p:MP)->[[f32;4];3]{p.cast::<[[f32;4];3]>().read()}
unsafe fn concat(a:MP,b:MP,d:MP){ext::PSMTXConcat(a.cast(),b.cast(),d.cast());}
unsafe fn inverse_transpose(a:MP,d:MP){ext::HSD_MtxInverseTranspose(a.cast(),d.cast());}
unsafe fn normal_upload(j:*mut HSD_JObj,f:u32,m:MP,pn:u32,tx:i32){if (*j).flags&0x80!=0{mx::GXLoadNrmMtxImm(m,pn);HSD_PerfCountMtxLoad();}if f&6!=0{let tx=if tx<0{ext::HSD_Index2TexMtx((-tx-1) as u32)}else{tx as u32};mx::GXLoadTexMtxImm(m,tx,0);HSD_PerfCountMtxLoad();}}
pub unsafe extern "C" fn SetupRigidModelMtx(p:*mut HSD_PObj,v:MP,m:MP,r:u32){ melee_core::ct!("SetupRigidModelMtx");let j=ext::HSD_JObjGetCurrent();if mtx_mark[0].obj==j.cast()&&mtx_mark[0].mark==1{return}HSD_PObjSetMtxMark(0,j.cast(),1);ext::GXSetCurrentMtx(0);mx::GXLoadPosMtxImm(m,0);crate::pose_capture::rigid(0,v,m,j);HSD_PerfCountMtxLoad();let f=GetSetupFlags(j,r);if f&1!=0{let mut n=[[0.;4];3];inverse_transpose(m,n.as_mut_ptr());normal_upload(j,f,n.as_mut_ptr(),0,30);}}
pub unsafe extern "C" fn SetupSharedVtxModelMtx(p:*mut HSD_PObj,v:MP,m:MP,r:u32){ melee_core::ct!("SetupSharedVtxModelMtx");let j=ext::HSD_JObjGetCurrent();let mut f=0;if mtx_mark[0].obj!=j.cast()&&mtx_mark[0].mark!=1{f|=1;}if mtx_mark[1].obj!=(*p).u.jobj.cast()&&mtx_mark[1].mark!=1{f|=2;}if f==0{return}f|=GetSetupFlags(j,r);ext::GXSetCurrentMtx(0);mx::GXLoadPosMtxImm(m,0);crate::pose_capture::rigid(0,v,m,j);HSD_PerfCountMtxLoad();if f&1!=0{let mut n=[[0.;4];3];inverse_transpose(m,n.as_mut_ptr());normal_upload(j,f,n.as_mut_ptr(),0,30);}jobj_setup_matrix((*p).u.jobj);let mut mm=[[0.;4];3];concat(v,(*(*p).u.jobj).mtx.as_mut_ptr(),mm.as_mut_ptr());mx::GXLoadPosMtxImm(mm.as_mut_ptr(),3);crate::pose_capture::rigid(3,v,mm.as_ptr(),(*p).u.jobj);HSD_PerfCountMtxLoad();if f&1!=0{let mut n=[[0.;4];3];inverse_transpose(mm.as_mut_ptr(),n.as_mut_ptr());normal_upload(j,f,n.as_mut_ptr(),3,33);}}
pub unsafe extern "C" fn env_hash_f32s(h0:*mut u32,h1:*mut u32,p:*const f32,n:i32){ melee_core::ct!("env_hash_f32s");for i in 0..n{let u=(*p.offset(i as isize)).to_bits();*h0=(*h0^u).wrapping_mul(16777619);*h1=(*h1).rotate_left(5).wrapping_mul(2246822519).wrapping_add(u.wrapping_mul(0x9e3779b1));}}
unsafe fn envelope_key(mut e:*mut HSD_Envelope,mut k0:u32,mut k1:u32,setup:bool)->(u32,u32,i32){let mut n=0;while !e.is_null(){if setup{jobj_setup_matrix((*e).jobj);}let j=(*e).jobj as usize as u32;let w=(*e).weight.to_bits();k0=(k0^j).wrapping_mul(16777619);k0=(k0^w).wrapping_mul(16777619);k1=k1.wrapping_add(j.wrapping_mul(31)).wrapping_add(w).wrapping_mul(2246822519);n+=1;e=(*e).next;}(k0,k1,n)}
pub unsafe extern "C" fn SetupEnvelopeModelMtx(p:*mut HSD_PObj,v:MP,pm:MP,r:u32){ melee_core::ct!("SetupEnvelopeModelMtx");if ext::ProfActive()!=0{ext::ProfBegin(14,0);}let j=ext::HSD_JObjGetCurrent();HSD_PObjClearMtxMark(null_mut(),2);let f=GetSetupFlags(j,r);let mut outer=[[0.;4];3];let right=mx::_HSD_mkEnvelopeModelNodeMtx(j,outer.as_mut_ptr());if geno_pal_selftest_on<0{geno_pal_selftest_on=ext::diag_geno_pal_selftest();}let mode=ext::env_cache_mode();let(mut c0,mut c1)=(0,0);if mode!=0{c0=(2166136261u32^ext::geno_stun_frame() as u32).wrapping_mul(16777619);c1=0x9e3779b9;env_hash_f32s(&mut c0,&mut c1,v.cast(),12);if !right.is_null(){env_hash_f32s(&mut c0,&mut c1,right.cast(),12);}else{c0^=0x5bd1e995;}}
let mut list=(*p).u.envelope_list;let mut idx=0;while idx<10&&!list.is_null(){let mut e=(*list).data.cast::<HSD_Envelope>();let mtx_no=ext::HSD_Index2PosNrmMtx(idx as u32) as u32;assert!(!e.is_null());let original=e;let(mut h0,mut h1,mut hit)=(0,0,0);let mut cached=[[0.;4];3];let mut cached_n=[[0.;4];3];let mut m=[[0.;4];3];let mut tmp=[[0.;4];3];ext::ProfCount(78);if ext::diag_pobj_enabled()!=0{let(k0,k1,n)=envelope_key(e,2166136261,0x9e3779b9,false);ext::diag_env_slot(k0,k1,n);}if mode!=0{(h0,h1,_)=envelope_key(e,c0^0x9e3779b9,c1.wrapping_add(0x85ebca6b),true);hit=ext::env_cache_get(h0,h1,c0,c1,cached.as_mut_ptr().cast(),cached_n.as_mut_ptr().cast(),(f&1!=0) as i32);}let use_cache=hit!=0&&mode==1;let have_n=hit==2;if use_cache{ext::ProfCount(79);tmp=cached;}else{let mut perf=0;let mp;if (*e).weight>=1.0-f32::EPSILON{jobj_setup_matrix((*e).jobj);if !right.is_null(){concat((*(*e).jobj).mtx.as_mut_ptr(),(*(*e).jobj).envelopemtx,m.as_mut_ptr());mp=m.as_mut_ptr();}else{mp=(*(*e).jobj).mtx.as_mut_ptr();}}else{let mut sum=0.;let mut n=0;while !e.is_null(){sum+=(*e).weight;n+=1;e=(*e).next;}ext::diag_envelope(n,sum);e=original;while !e.is_null(){let jp=(*e).jobj;assert!(!jp.is_null());jobj_setup_matrix(jp);assert!(!(*jp).envelopemtx.is_null());concat((*jp).mtx.as_mut_ptr(),(*jp).envelopemtx,tmp.as_mut_ptr());ext::HSD_MtxScaledAdd(tmp.as_mut_ptr().cast(),m.as_mut_ptr().cast(),m.as_mut_ptr().cast(),(*e).weight);perf+=1;e=(*e).next;}mp=m.as_mut_ptr();}ext::HSD_PerfCountEnvelopeBlending(perf);if !right.is_null(){concat(mp,right,m.as_mut_ptr());}concat(v,mp,tmp.as_mut_ptr());}mx::GXLoadPosMtxImm(tmp.as_mut_ptr(),mtx_no);if crate::pose_capture::on(){crate::pose_capture::envelope(mtx_no,v,tmp.as_ptr(),original,j,right);}HSD_PerfCountMtxLoad();if geno_pal_selftest_on>0{core::ptr::copy_nonoverlapping(tmp.as_ptr().cast::<f32>(),geno_pal_legacy[idx as usize].as_mut_ptr(),12);geno_pal_legacy_nrm[idx as usize]=0;}if f&1!=0{if use_cache&&have_n{m=cached_n;}else{inverse_transpose(tmp.as_mut_ptr(),m.as_mut_ptr());}if geno_pal_selftest_on>0{core::ptr::copy_nonoverlapping(m.as_ptr().cast::<f32>(),geno_pal_legacy[idx as usize].as_mut_ptr().add(12),12);geno_pal_legacy_nrm[idx as usize]=1;}normal_upload(j,f,m.as_mut_ptr(),mtx_no,-(idx as i32)-1);}if mode!=0{if hit!=0&&mode==2{ext::env_cache_compare(h0,h1,c0,c1,tmp.as_ptr().cast(),m.as_ptr().cast(),(f&1!=0) as i32);}else if hit==0||(f&1!=0&&!have_n){ext::env_cache_put(h0,h1,c0,c1,tmp.as_ptr().cast(),m.as_ptr().cast(),(f&1!=0) as i32);}}idx+=1;list=(*list).next;}if geno_pal_selftest_on>0{geno_pal_selftest_check(p,v,right,idx as i32);}ext::ProfEnd();}
pub unsafe extern "C" fn PObjSetupMtx(p:*mut HSD_PObj,v:MP,m:MP,r:u32){ melee_core::ct!("PObjSetupMtx");match kind(p){0=>if (*p).u.jobj.is_null(){SetupRigidModelMtx(p,v,m,r)}else{SetupSharedVtxModelMtx(p,v,m,r)},0x1000=>SetupRigidModelMtx(p,v,m,r),0x2000=>SetupEnvelopeModelMtx(p,v,m,r),_=>()}}
pub unsafe extern "C" fn PObjDispSimplePrimitive(p:*mut HSD_PObj,r:u32){ melee_core::ct!("PObjDispSimplePrimitive");setupArrayDesc((*p).verts);setupVtxDesc(p);ext::GXCallDisplayList((*p).display.cast(),(*p).n_display as u32*32);}
pub unsafe extern "C" fn PObjDispShapeAnim(p:*mut HSD_PObj,r:u32){ melee_core::ct!("PObjDispShapeAnim");setupShapeAnimArrayDesc((*p).verts);setupShapeAnimVtxDesc(p);assert!(!(*p).u.shape_set.is_null());drawShapeAnim(p);}
pub unsafe extern "C" fn HSD_PObjDisp(p:*mut HSD_PObj,v:MP,m:MP,r:u32){ melee_core::ct!("HSD_PObjDisp");ext::ProfCount(80);if ext::diag_pobj_enabled()!=0{let mut n=0;if kind(p)==0x2000{let mut l=(*p).u.envelope_list;while !l.is_null(){n+=1;l=(*l).next;}}let frame=ext::geno_stun_frame();let g=ext::HSD_GObj_804D7814;let ctx=if g.is_null(){u32::MAX}else{((*g).classifier as u32)<<16|((*g).gx_link as u32)<<8|(*g).p_link as u32};let flags=if ext::HSD_JObjGetCurrent().is_null(){u32::MAX}else{(*ext::HSD_JObjGetCurrent()).flags};ext::diag_pobj_draw(p as usize as u32,frame,n,ctx,r,0,flags);}match (*p).flags&0xc000{0=>ext::HSD_StateSetCullMode(0),0x4000=>ext::HSD_StateSetCullMode(1),0x8000=>ext::HSD_StateSetCullMode(2),_=>return}((*method(p)).setup_mtx.unwrap())(p,v,m,r);if kind(p)==0x1000{PObjDispShapeAnim(p,r)}else{PObjDispSimplePrimitive(p,r)}}
pub unsafe extern "C" fn PObjRelease(o:*mut HSD_Class){ melee_core::ct!("PObjRelease");let p=o.cast::<HSD_PObj>();match kind(p){0x1000=>HSD_ShapeSetRemove((*p).u.shape_set),0x2000=>HSD_EnvelopeListFree((*p).u.envelope_list),0=>ext::HSD_JObjUnrefThis((*p).u.jobj),_=>()}((*hsdPObj.parent.head.parent).release.unwrap())(o);}
pub unsafe extern "C" fn PObjAmnesia(i:*mut HSD_ClassInfo){ melee_core::ct!("PObjAmnesia");if i==default_class.cast(){default_class=null_mut();}if i==addr_of_mut!(hsdPObj).cast(){vertex_buffer=null_mut();normal_buffer=null_mut();vertex_buffer_size=0;normal_buffer_size=0;prev_vtxdesclist_array=null_mut();prev_vtxdesc=null_mut();}((*hsdPObj.parent.head.parent).amnesia.unwrap())(i);}
pub unsafe extern "C" fn PObjInfoInit(){ melee_core::ct!("PObjInfoInit");ext::hsdInitClassInfo(addr_of_mut!(hsdPObj).cast(),hsd_class_ptr(),c"sysdolphin_base_library".as_ptr().cast_mut(),c"hsd_pobj".as_ptr().cast_mut(),size_of::<HSD_PObjInfo>() as i32,size_of::<HSD_PObj>() as i32);hsdPObj.parent.release=Some(PObjRelease);hsdPObj.parent.amnesia=Some(PObjAmnesia);hsdPObj.disp=Some(HSD_PObjDisp);hsdPObj.setup_mtx=Some(PObjSetupMtx);hsdPObj.load=Some(PObjLoad);}
pub unsafe extern "C" fn GenoPalLoad(p:*mut HSD_PObj,d:*mut HSD_PObjDesc)->i32{ melee_core::ct!("GenoPalLoad");let rc=(hsdPObj.load.unwrap())(p,d);let mut n=0;let mut l=if kind(p)==0x2000{(*p).u.envelope_list}else{null_mut()};while !l.is_null(){n+=1;l=(*l).next;}let(mut pn,mut tex)=(0,0);let mut v=(*p).verts;while !v.is_null()&&(*v).attr!=255{if (*v).attr==0&&(*v).attr_type==1{pn=1;}if (1..=8).contains(&(*v).attr){tex=1;}v=v.add(1);}let gp=p.cast::<GenoPalPObj>();(*gp).n=n.min(255) as u8;(*gp).broken=(kind(p)!=0x2000||n<1||n>64||pn==0||tex!=0) as u8;ext::diag_geno_pal(((*gp).broken!=0) as i32,n,(kind(p)!=0x2000) as i32|((pn==0) as i32)<<1|tex<<2);rc}
pub unsafe extern "C" fn geno_pal_compute(p:*mut HSD_PObj,v:MP,right:MP,out:*mut f32,need_nrm:i32)->i32{ melee_core::ct!("geno_pal_compute");let frame=ext::geno_stun_frame() as u32;let vb=fmadds((*v.add(2))[3],7.,fmadds((*v.add(1))[3],3.,(*v)[0]));let cam=(v as usize as u32^vb.to_bits()^(ext::HSD_JObjGetCurrent() as usize as u32).wrapping_mul(2654435761)).wrapping_add((need_nrm!=0) as u32);let mut list=(*p).u.envelope_list;let mut i=0;while i<64&&!list.is_null(){let(k0,k1,_)=envelope_key((*list).data.cast(),2166136261,0x9e3779b9,false);let o=out.add(i as usize*24);if ext::GenoPalCacheGet(k0,k1,cam,frame,o)==0{let mut m=[[0.;4];3];let mut tmp=[[0.;4];3];let mut nrm=[[0.;4];3];let mut e=(*list).data.cast::<HSD_Envelope>();let mut mp;if (*e).weight>=1.-f32::EPSILON{jobj_setup_matrix((*e).jobj);if !right.is_null(){concat((*(*e).jobj).mtx.as_mut_ptr(),(*(*e).jobj).envelopemtx,m.as_mut_ptr());mp=m.as_mut_ptr();}else{mp=(*(*e).jobj).mtx.as_mut_ptr();}}else{while !e.is_null(){let j=(*e).jobj;jobj_setup_matrix(j);concat((*j).mtx.as_mut_ptr(),(*j).envelopemtx,tmp.as_mut_ptr());ext::HSD_MtxScaledAdd(tmp.as_mut_ptr().cast(),m.as_mut_ptr().cast(),m.as_mut_ptr().cast(),(*e).weight);e=(*e).next;}mp=m.as_mut_ptr();}if !right.is_null(){concat(mp,right,m.as_mut_ptr());mp=m.as_mut_ptr();}concat(v,mp,tmp.as_mut_ptr());if need_nrm!=0{inverse_transpose(tmp.as_mut_ptr(),nrm.as_mut_ptr());}for r in 0..3{for c in 0..4{*o.add(r*4+c)=tmp[r][c];*o.add(12+r*4+c)=if c<3{nrm[r][c]}else{0.};}}ext::GenoPalCachePut(k0,k1,cam,frame,o);}i+=1;list=(*list).next;}i}
pub unsafe extern "C" fn GenoPalSetupMtx(p:*mut HSD_PObj,v:MP,pm:MP,r:u32){ melee_core::ct!("GenoPalSetupMtx");let gp=p.cast::<GenoPalPObj>();if (*gp).broken!=0{return}let j=ext::HSD_JObjGetCurrent();HSD_PObjClearMtxMark(null_mut(),2);let f=GetSetupFlags(j,r);if f&6!=0{(*gp).broken=1;ext::diag_geno_pal(2,(*gp).n as i32,0);return}let mut m=[[0.;4];3];let right=mx::_HSD_mkEnvelopeModelNodeMtx(j,m.as_mut_ptr());if ext::gx_suppress_draws!=0{let mut l=(*p).u.envelope_list;let mut i=0;while i<64&&!l.is_null(){let mut e=(*l).data.cast::<HSD_Envelope>();while !e.is_null(){jobj_setup_matrix((*e).jobj);e=(*e).next;}l=(*l).next;i+=1;}return}if ext::ProfActive()!=0{ext::ProfBegin(50,0);}let n=geno_pal_compute(p,v,right,geno_pal_data.as_mut_ptr(),(f&1!=0) as i32);ext::ProfEnd();ext::GXAuroraLoadPalette(n as u32,p as usize as u32^j as usize as u32,geno_pal_data.as_ptr());}
pub unsafe extern "C" fn geno_pal_selftest_check(p:*mut HSD_PObj,v:MP,right:MP,slots:i32){ melee_core::ct!("geno_pal_selftest_check");let n=geno_pal_compute(p,v,right,geno_pal_data.as_mut_ptr(),1);if n!=slots{ext::diag_geno_pal_compare(-1,n,slots,0);return}let(mut pos,mut nrm,mut nrm_n)=(0,0,0);for i in 0..n as usize{pos+=(ext::memcmp(geno_pal_data.as_ptr().add(i*24).cast(),geno_pal_legacy[i].as_ptr().cast(),12*4)==0) as i32;if geno_pal_legacy_nrm[i]!=0{let mut eq=1;nrm_n+=1;for k in 0..12usize{if k&3!=3&&ext::memcmp(geno_pal_data.as_ptr().add(i*24+12+k).cast(),geno_pal_legacy[i].as_ptr().add(12+k).cast(),4)!=0{eq=0;}}nrm+=eq;}}ext::diag_geno_pal_compare(n,pos,nrm,nrm_n);}
pub unsafe extern "C" fn GenoPalDisp(p:*mut HSD_PObj,v:MP,m:MP,r:u32){ melee_core::ct!("GenoPalDisp");if (*p.cast::<GenoPalPObj>()).broken!=0{return}HSD_PObjDisp(p,v,m,r);if ext::gx_suppress_draws==0{ext::GXAuroraEndPalette();}}
pub unsafe extern "C" fn GenoPalInfoInit(){ melee_core::ct!("GenoPalInfoInit");ext::hsdInitClassInfo(addr_of_mut!(genoPalPObj).cast(),addr_of_mut!(hsdPObj).cast(),c"sysdolphin_base_library".as_ptr().cast_mut(),c"geno_pal_pobj_v1".as_ptr().cast_mut(),size_of::<HSD_PObjInfo>() as i32,size_of::<GenoPalPObj>() as i32);genoPalPObj.disp=Some(GenoPalDisp);genoPalPObj.setup_mtx=Some(GenoPalSetupMtx);genoPalPObj.load=Some(GenoPalLoad);}
pub unsafe extern "C" fn geno_pal_force_convert(d:*const HSD_PObjDesc)->i32{ melee_core::ct!("geno_pal_force_convert");if (*d).flags&0x3000!=0x2000||(*d).display.is_null()||(*d).verts.is_null(){return 0}let mut v=(*d).verts;let(mut stride,mut pn)=(0,0);while (*v).attr!=255{if (*v).attr==0{if (*v).attr_type!=1||stride!=0{return 0}pn=1;stride+=1;}else if (1..=8).contains(&(*v).attr){return 0}else if (*v).attr_type==2{stride+=1;}else if (*v).attr_type==3{stride+=2;}else{return 0}v=v.add(1);}if pn==0{return 0}for i in 0..geno_pal_nforced{if geno_pal_forced[i]==(*d).display{return 1}}if geno_pal_nforced>=512{return 0}let mut dl=(*d).display;let end=dl.add((*d).n_display as usize*32);while dl.wrapping_add(3)<=end&&*dl&0xf8!=0{let n=((*dl.add(1) as usize)<<8)|*dl.add(2) as usize;dl=dl.add(3);for _ in 0..n{if dl.wrapping_add(stride)>end{break}*dl/=3;dl=dl.add(stride);}}geno_pal_forced[geno_pal_nforced]=(*d).display;geno_pal_nforced+=1;1}
pub unsafe extern "C" fn geno_pal_desc(d:*const HSD_PObjDesc)->i32{ melee_core::ct!("geno_pal_desc");if !(*d).class_name.is_null()&&ext::strcmp((*d).class_name,c"geno_pal_pobj_v1".as_ptr())==0{return 1}if geno_pal_force_on<0{geno_pal_force_on=ext::diag_geno_pal_force();}(geno_pal_force_on>0&&(*d).class_name.is_null()&&geno_pal_force_convert(d)!=0) as i32}
}

pub const POBJ_SKIN: u32 = 0;
pub const POBJ_SHAPEANIM: u32 = 1 << 12;
pub const POBJ_ENVELOPE: u32 = 1 << 13;
pub const POBJ_TYPE_MASK: u32 = 0x3000;
pub const POBJ_CULLFRONT: u32 = 1 << 14;
pub const POBJ_CULLBACK: u32 = 1 << 15;

pub const HSD_MTX_RIGID: u32 = 0;
pub const HSD_MTX_ENVELOPE: u32 = 1;

const SETUP_NORMAL: u32 = 1;
const SETUP_REFLECTION: u32 = 2;
const SETUP_HIGHLIGHT: u32 = 4;
const SETUP_NORMAL_PROJECTION: u32 = SETUP_REFLECTION | SETUP_HIGHLIGHT;
const SETUP_JOINT0: u32 = 8;
const SETUP_JOINT1: u32 = 16;

const GX_VA_NRM: u32 = 10;
const GX_VA_NBT: u32 = 25;
const GX_VA_CLR0: u32 = 11;
const GX_VA_CLR1: u32 = 12;

/// One `HSD_VtxDescList` entry (0x18 bytes).
#[derive(Clone, Copy, Debug)]
pub struct VtxDesc {
    pub attr: u32,
    pub attr_type: u32,
    pub comp_cnt: u32,
    pub comp_type: u32,
    pub frac: u8,
    pub stride: u16,
    pub vertex: Option<u32>,
}

pub fn vtx_descs(m: &DispModel, verts: u32) -> Vec<VtxDesc> {
    melee_core::ct!("vtx_descs");
    let d = &m.dat;
    let mut out = Vec::new();
    let mut v = verts;
    while out.len() < 27 {
        let attr = d.u32_(v);
        if attr == GX_VA_NULL {
            break;
        }
        out.push(VtxDesc {
            attr,
            attr_type: d.u32_(v + 4),
            comp_cnt: d.u32_(v + 8),
            comp_type: d.u32_(v + 0xC),
            frac: d.u8_(v + 0x10),
            stride: d.u16_(v + 0x12),
            // An indexed attribute always has an array; offset 0 is valid there (the vertex
            // buffers usually open the DAT data section), so it is not read as NULL.
            vertex: Some(d.u32_(v + 0x14)).filter(|&p| (p as usize) < d.0.len()),
        });
        v += 0x18;
    }
    out
}

/// Bytes one attribute takes per vertex in a display list.
fn attr_size(a: &VtxDesc) -> usize {
    match a.attr_type {
        GX_INDEX8 => 1,
        GX_INDEX16 => 2,
        GX_DIRECT => {
            if a.attr <= GX_VA_TEX7MTXIDX {
                return 1;
            }
            let comp = match a.comp_type {
                0 | 1 => 1,
                2 | 3 => 2,
                _ => 4,
            };
            match a.attr {
                GX_VA_POS => comp * if a.comp_cnt == 0 { 2 } else { 3 },
                GX_VA_NRM | GX_VA_NBT => comp * if a.comp_cnt == 0 { 3 } else { 9 },
                GX_VA_CLR0 | GX_VA_CLR1 => match a.comp_type {
                    0 | 3 => 2,
                    1 | 4 => 3,
                    _ => 4,
                },
                _ => comp * if a.comp_cnt == 0 { 1 } else { 2 },
            }
        }
        _ => 0,
    }
}

/// The largest index each indexed attribute of the display list uses (to size the arrays the
/// backend snapshots). A malformed list stops the scan where it breaks.
pub fn scan_display_list(dl: &[u8], attrs: &[VtxDesc]) -> Vec<(u32, u32)> {
    melee_core::ct!("scan_display_list");
    let mut max: Vec<(u32, u32)> = Vec::new();
    let mut i = 0;
    'outer: while i < dl.len() {
        let op = dl[i];
        if op == 0 {
            i += 1;
            continue;
        }
        if op & 0x80 == 0 || i + 3 > dl.len() {
            break;
        }
        let n = u16::from_be_bytes([dl[i + 1], dl[i + 2]]) as usize;
        i += 3;
        for _ in 0..n {
            for a in attrs {
                let s = attr_size(a);
                if i + s > dl.len() {
                    break 'outer;
                }
                let idx = match a.attr_type {
                    GX_INDEX8 => Some(dl[i] as u32),
                    GX_INDEX16 => Some(u16::from_be_bytes([dl[i], dl[i + 1]]) as u32),
                    _ => None,
                };
                if let Some(x) = idx {
                    match max.iter_mut().find(|e| e.0 == a.attr) {
                        Some(e) => e.1 = e.1.max(x),
                        None => max.push((a.attr, x)),
                    }
                }
                i += s;
            }
        }
    }
    max
}

/// `HSD_Index2PosNrmMtx`.
fn index_to_pos_nrm_mtx(i: u32) -> u32 {
    GX_PNMTX0 + 3 * i
}
/// `HSD_Index2TexMtx`.
fn index_to_tex_mtx(i: u32) -> u32 {
    GX_TEXMTX0 + 3 * i
}

impl HsdRender {
    fn model_key(m: &DispModel) -> usize {
        m.dat.0.as_ptr() as usize
    }

    /// `HSD_ClearVtxDesc`.
    pub fn clear_vtx_desc<G: GxBackend>(&mut self, g: &mut G) {
        g.clear_vtx_desc();
        self.prev_array = None;
        self.prev_vtxdesc = None;
    }

    /// `GetSetupFlags` (the JObj is the one being drawn).
    fn get_setup_flags(&self, m: &DispModel, j: usize, rendermode: u32) -> u32 {
        let mut flags = 0;
        if rendermode & 0x400_0000 == 0 {
            if m.joints[j].flags & JOBJ_LIGHTING != 0 {
                flags |= SETUP_NORMAL;
            }
            if self.tobj_kinds & (1 << TEX_COORD_REFLECTION) != 0 {
                flags |= SETUP_NORMAL | SETUP_REFLECTION;
            }
            if self.tobj_kinds & (1 << TEX_COORD_HILIGHT) != 0 {
                flags |= SETUP_NORMAL | SETUP_HIGHLIGHT;
            }
        }
        flags
    }

    fn load_normal<G: GxBackend>(g: &mut G, lighting: bool, flags: u32, pm: &Mtx, pn: u32, tex: u32) {
        if flags & SETUP_NORMAL != 0 {
            let n = hsd_mtx::inverse_transpose(pm);
            if lighting {
                g.load_nrm_mtx_imm(&n, pn);
            }
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                g.load_tex_mtx_imm(&n, tex, GX_MTX3x4);
            }
        }
    }

    /// `SetupRigidModelMtx`.
    fn setup_rigid_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, pmtx: &Mtx, rendermode: u32) {
        if self.mtx_mark[0] == (Some(j), HSD_MTX_RIGID) {
            return;
        }
        self.mtx_mark[0] = (Some(j), HSD_MTX_RIGID);
        g.set_current_mtx(GX_PNMTX0);
        g.load_pos_mtx_imm(pmtx, GX_PNMTX0);
        let flags = self.get_setup_flags(m, j, rendermode);
        Self::load_normal(g, m.joints[j].flags & JOBJ_LIGHTING != 0, flags, pmtx, GX_PNMTX0, GX_TEXMTX0);
    }

    /// `SetupSharedVtxModelMtx` (`pobj->u.jobj` = joint `other`).
    fn setup_shared_vtx_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, other: usize, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        let mut flags = 0;
        // pobj.c: `obj != jobj && mark != RIGID` (sic: both conditions, as the C has them).
        if self.mtx_mark[0].0 != Some(j) && self.mtx_mark[0].1 != HSD_MTX_RIGID {
            flags |= SETUP_JOINT0;
        }
        if self.mtx_mark[1].0 != Some(other) && self.mtx_mark[1].1 != HSD_MTX_RIGID {
            flags |= SETUP_JOINT1;
        }
        if flags == 0 {
            return;
        }
        flags |= self.get_setup_flags(m, j, rendermode);
        let lighting = m.joints[j].flags & 0x80 != 0;
        g.set_current_mtx(GX_PNMTX0);
        g.load_pos_mtx_imm(pmtx, GX_PNMTX0);
        Self::load_normal(g, lighting, flags, pmtx, GX_PNMTX0, GX_TEXMTX0);
        let mm = mtx::concat(vmtx, &m.joints[other].mtx);
        g.load_pos_mtx_imm(&mm, GX_PNMTX1);
        Self::load_normal(g, lighting, flags, &mm, GX_PNMTX1, GX_TEXMTX1);
    }

    /// `SetupEnvelopeModelMtx`.
    fn setup_envelope_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, list: Option<u32>, vmtx: &Mtx, rendermode: u32) {
        // HSD_PObjClearMtxMark(NULL, HSD_MTX_ENVELOPE)
        self.mtx_mark = [(None, HSD_MTX_ENVELOPE); 2];
        let flags = self.get_setup_flags(m, j, rendermode);
        let right = m.mk_envelope_model_node_mtx(j);
        let d = &m.dat;
        let Some(mut lp) = list else { return };
        let ident = mtx::identity();
        let mut idx = 0u32;
        while idx < 10 {
            let Some(edesc0) = d.ptr(lp) else { break };
            // The envelope's (joint, weight) terms.
            let mut terms = Vec::new();
            let mut e = edesc0;
            while let Some(jd) = d.ptr(e) {
                terms.push((jd, d.f32_(e + 4)));
                e += 8;
                if terms.len() > 64 {
                    break;
                }
            }
            let joint = |jd: u32| m.by_desc(jd);
            let mut mtxv: Mtx;
            if terms.first().is_some_and(|t| t.1 >= 1.0 - f32::EPSILON) {
                let Some(ji) = joint(terms[0].0) else {
                    lp += 4;
                    idx += 1;
                    continue;
                };
                mtxv = if right.is_some() {
                    mtx::concat(&m.joints[ji].mtx, &m.envelope_mtx(ji).unwrap_or(ident))
                } else {
                    m.joints[ji].mtx
                };
            } else {
                mtxv = [[0.0; 4]; 3];
                for &(jd, w) in &terms {
                    let Some(ji) = joint(jd) else { continue };
                    let tmp = mtx::concat(&m.joints[ji].mtx, &m.envelope_mtx(ji).unwrap_or(ident));
                    mtxv = hsd_mtx::scaled_add(&tmp, &mtxv, w);
                }
            }
            if let Some(r) = &right {
                mtxv = mtx::concat(&mtxv, r);
            }
            let tmp = mtx::concat(vmtx, &mtxv);
            let mtx_no = index_to_pos_nrm_mtx(idx);
            g.load_pos_mtx_imm(&tmp, mtx_no);
            Self::load_normal(g, m.joints[j].flags & JOBJ_LIGHTING != 0, flags, &tmp, mtx_no, index_to_tex_mtx(idx));
            lp += 4;
            idx += 1;
        }
    }

    /// `setupArrayDesc` + `setupVtxDesc` + `GXCallDisplayList` (`PObjDispSimplePrimitive`).
    fn pobj_disp_simple_primitive<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, pobj: u32) {
        let d = &m.dat;
        let Some(verts) = d.ptr(pobj + 8) else { return };
        let Some(disp) = d.ptr(pobj + 0x10) else { return };
        let n_display = d.u16_(pobj + 0xE) as usize;
        let dl_full = d.tail(disp);
        let dl = &dl_full[..dl_full.len().min(n_display << 5)];
        let descs = vtx_descs(m, verts);
        let key = (Self::model_key(m), verts);
        let extents = self.extents.entry((key.0, pobj)).or_insert_with(|| scan_display_list(dl, &descs)).clone();
        // setupArrayDesc: the C caches by desc list; the array lengths here depend on the
        // display list too, so a list shared by two PObjs is re-bound per PObj.
        for a in &descs {
            if a.attr_type != GX_DIRECT {
                if let Some(base) = a.vertex {
                    let n = extents.iter().find(|e| e.0 == a.attr).map_or(0, |e| e.1) + 1;
                    let len = (n as usize * a.stride.max(1) as usize + 16).min(d.0.len() - base as usize);
                    g.set_array(a.attr, &d.0[base as usize..base as usize + len], a.stride as u8);
                }
            }
        }
        self.prev_array = Some(key);
        if self.prev_vtxdesc != Some(key) {
            g.clear_vtx_desc();
            for a in &descs {
                g.set_vtx_desc(a.attr, a.attr_type);
                if a.attr > GX_VA_TEX7MTXIDX {
                    g.set_vtx_attr_fmt(GX_VTXFMT0, a.attr, a.comp_cnt, a.comp_type, a.frac);
                }
            }
            self.prev_vtxdesc = Some(key);
        }
        g.call_display_list(dl);
    }

    /// `HSD_PObjDisp` for the PObj descriptor `pobj` of joint `j`.
    pub fn pobj_disp<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, pobj: u32, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        let flags = m.dat.u16_(pobj + 0xC) as u32;
        match flags & (POBJ_CULLFRONT | POBJ_CULLBACK) {
            0 => self.state_set_cull_mode(g, GX_CULL_NONE),
            POBJ_CULLFRONT => self.state_set_cull_mode(g, GX_CULL_FRONT),
            POBJ_CULLBACK => self.state_set_cull_mode(g, GX_CULL_BACK),
            _ => return,
        }
        let u = m.dat.ptr(pobj + 0x14);
        match flags & POBJ_TYPE_MASK {
            POBJ_SHAPEANIM => {
                self.stats.shape_anim_skipped += 1;
                return;
            }
            POBJ_ENVELOPE => {
                self.stats.envelope_pobjs += 1;
                self.setup_envelope_model_mtx(g, m, j, u, vmtx, rendermode);
            }
            _ => match u.and_then(|jd| m.by_desc(jd)) {
                None => self.setup_rigid_model_mtx(g, m, j, pmtx, rendermode),
                Some(other) => self.setup_shared_vtx_model_mtx(g, m, j, other, vmtx, pmtx, rendermode),
            },
        }
        self.stats.pobjs += 1;
        self.pobj_disp_simple_primitive(g, m, pobj);
    }
}

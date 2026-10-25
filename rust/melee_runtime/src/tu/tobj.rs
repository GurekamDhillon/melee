//! `src/sysdolphin/baselib/tobj.c`, the display half: `TObjLoad` (from the descriptor),
//! `HSD_TObjAssignResources`, `TObjSetupMtx`, `setupTextureCoordGen` /
//! `HSD_TObjSetupTextureCoordGen` and `HSD_TObjSetup` (TLUT naming + texture loads).
//! `MakeTextureMtx` and the GX id maps are the verified ports in `crate::tobj`.
//! Not ported: bump (TEX_BUMP) and toon texgens, the HILIGHT / SHADOW texture matrices (need the
//! current CObj / infinite LObj), texture animation, `tlut_no` / `imagetbl` (animated swaps).
use super::state::gx::*;
use super::state::{Dat, GxBackend, HsdRender, TexLoad};
use crate::mtx::{Mtx, Quaternion, Vec3};
use crate::tobj::{self as t, HsdTObj, HsdTexLodDesc};

pub const TEX_COORD_UV: u32 = 0;
pub const TEX_COORD_REFLECTION: u32 = 1;
pub const TEX_COORD_HILIGHT: u32 = 2;
pub const TEX_COORD_SHADOW: u32 = 3;
pub const TEX_COORD_TOON: u32 = 4;
pub const TEX_LIGHTMAP_DIFFUSE: u32 = 1 << 4;
pub const TEX_LIGHTMAP_SPECULAR: u32 = 1 << 5;
pub const TEX_LIGHTMAP_AMBIENT: u32 = 1 << 6;
pub const TEX_LIGHTMAP_EXT: u32 = 1 << 7;
pub const TEX_LIGHTMAP_SHADOW: u32 = 1 << 8;
pub const TEX_LIGHTMAP_MASK: u32 = 0x1F0;
pub const TEX_BUMP: u32 = 1 << 24;

/// A loaded `HSD_TObj` (the fields the display path reads), with its descriptor offset.
#[derive(Clone, Debug)]
pub struct TObjRt {
    pub desc: u32,
    pub id: u32,
    pub src: u32,
    pub mtxid: u32,
    pub coord: u32,
    pub mag_filt: u32,
    pub image: Option<u32>,
    pub tlut: Option<u32>,
    pub tev: Option<u32>,
    pub obj: HsdTObj<u32>,
}

impl TObjRt {
    pub fn flags(&self) -> u32 {
        self.obj.flags
    }
    /// `tobj_coord`.
    pub fn coord_kind(&self) -> u32 {
        self.obj.flags & 0xF
    }
    pub fn bump(&self) -> bool {
        self.obj.flags & TEX_BUMP != 0
    }
    /// `tobj_lightmap`.
    pub fn lightmap(&self) -> u32 {
        self.obj.flags & TEX_LIGHTMAP_MASK
    }
}

/// `HSD_TObjLoadDesc` over the descriptor chain starting at `td` (`TObjLoad`).
pub fn load_desc_list(d: &Dat, mut td: Option<u32>) -> Vec<TObjRt> {
    let mut v = Vec::new();
    while let Some(o) = td {
        if v.len() > 16 {
            break;
        }
        let vec3 = |at: u32| Vec3 { x: d.f32_(at), y: d.f32_(at + 4), z: d.f32_(at + 8) };
        let r = vec3(o + 0x10);
        let lod = d.ptr(o + 0x54).map(|l| HsdTexLodDesc {
            min_filt: d.u32_(l),
            lod_bias: d.f32_(l + 4),
            bias_clamp: d.u8_(l + 8),
            edge_lod_enable: d.u8_(l + 9),
            max_anisotropy: d.u32_(l + 0xC),
        });
        let obj = HsdTObj {
            rotate: Quaternion { x: r.x, y: r.y, z: r.z, w: 0.0 },
            scale: vec3(o + 0x1C),
            translate: vec3(o + 0x28),
            wrap_s: d.u32_(o + 0x34),
            wrap_t: d.u32_(o + 0x38),
            repeat_s: d.u8_(o + 0x3C),
            repeat_t: d.u8_(o + 0x3D),
            flags: d.u32_(o + 0x40) | t::TEX_MTX_DIRTY,
            blending: d.f32_(o + 0x44),
            imagedesc: d.ptr(o + 0x4C),
            lod,
            imagetbl: None,
            has_tluttbl: false,
            tluttbl_ptrs: Vec::new(),
            tlut_no: -1,
            mtx: crate::mtx::identity(),
            tev: None,
        };
        v.push(TObjRt {
            desc: o,
            id: d.u32_(o + 8),
            src: d.u32_(o + 0xC),
            mtxid: GX_IDENTITY,
            coord: 0,
            mag_filt: d.u32_(o + 0x48),
            image: d.ptr(o + 0x4C),
            tlut: d.ptr(o + 0x50),
            tev: d.ptr(o + 0x58),
            obj,
        });
        td = d.ptr(o + 4);
    }
    v
}

/// `HSD_TObjAssignResources`: texmap ids, PT texture matrices and texcoords. Returns the number
/// of texcoords used.
pub fn assign_resources(list: &mut [TObjRt]) -> u32 {
    if list.is_empty() {
        return 0;
    }
    let toon = list.iter().rposition(|o| o.coord_kind() == TEX_COORD_TOON);
    let bump = list.iter().rposition(|o| o.coord_kind() != TEX_COORD_TOON && o.bump());
    let mut limit = 8u32;
    if toon.is_some() {
        limit -= 1;
    }
    if bump.is_some() {
        limit -= 2;
    }
    let (mut texmap_no, mut texcoord_no) = (0u32, 0u32);
    for i in 0..list.len() {
        let o = &mut list[i];
        if o.coord_kind() == TEX_COORD_TOON {
            if Some(i) != toon {
                o.id = GX_TEXMAP_NULL;
            }
        } else if o.bump() {
            if Some(i) != bump {
                o.id = GX_TEXMAP_NULL;
            }
        } else if texmap_no < limit {
            o.id = t::index_to_tex_map(texmap_no);
            texmap_no += 1;
            o.mtxid = t::tex_map_id_to_pt_tex_mtx(o.id);
            if matches!(o.coord_kind(), TEX_COORD_REFLECTION | TEX_COORD_HILIGHT | TEX_COORD_SHADOW) {
                o.coord = t::index_to_tex_coord(texcoord_no);
                texcoord_no += 1;
            }
        } else {
            o.id = GX_TEXMAP_NULL;
        }
    }
    for o in list.iter_mut() {
        if o.id != GX_TEXMAP_NULL && !o.bump() && o.coord_kind() == TEX_COORD_UV {
            o.coord = t::index_to_tex_coord(texcoord_no);
            texcoord_no += 1;
        }
    }
    if let Some(b) = bump {
        let o = &mut list[b];
        o.id = t::index_to_tex_map(texmap_no);
        texmap_no += 1;
        o.mtxid = 57; // GX_TEXMTX9
        o.coord = t::index_to_tex_coord(texcoord_no);
        texcoord_no += 2;
    }
    if let Some(tn) = toon {
        let o = &mut list[tn];
        o.id = t::index_to_tex_map(texmap_no);
        o.coord = t::index_to_tex_coord(texcoord_no);
        texcoord_no += 1;
    }
    texcoord_no
}

/// `TObjSetupMtx` (static). REFLECTION and the UV default are ported; HILIGHT and SHADOW need the
/// current camera/light and load identity here (TODO: tobj.c:431 / :472).
fn setup_mtx<G: GxBackend>(g: &mut G, o: &mut TObjRt) {
    if o.coord_kind() == TEX_COORD_TOON {
        return;
    }
    if o.obj.flags & t::TEX_MTX_DIRTY != 0 {
        if o.obj.repeat_s == 0 || o.obj.repeat_t == 0 {
            // tobj.c:589 asserts; a zero-repeat descriptor never reaches a draw in the C.
            o.obj.mtx = crate::mtx::identity();
        } else {
            o.obj.make_texture_mtx();
        }
        o.obj.flags &= !t::TEX_MTX_DIRTY;
    }
    let m = o.obj.mtx;
    match o.coord_kind() {
        TEX_COORD_REFLECTION => {
            let mut r: Mtx = [[0.0; 4]; 3];
            for i in 0..3 {
                r[i][0] = 0.5 * m[i][0];
                r[i][1] = -0.5 * m[i][1];
                r[i][2] = 0.0;
                // Render-only (never reaches the sim), so fusion is not matched here.
                r[i][3] = 0.5 * m[i][0] + 0.5 * m[i][1] + m[i][2] + m[i][3];
            }
            g.load_tex_mtx_imm(&r, o.mtxid, GX_MTX3x4);
        }
        TEX_COORD_HILIGHT | TEX_COORD_SHADOW => {
            let id: Mtx = crate::mtx::identity();
            g.load_tex_mtx_imm(&id, o.mtxid, GX_MTX3x4);
        }
        _ => {
            g.load_tex_mtx_imm(&m, o.mtxid, if o.bump() { GX_MTX2x4 } else { GX_MTX3x4 });
        }
    }
}

/// `setupTextureCoordGen` (static).
fn setup_texture_coord_gen<G: GxBackend>(g: &mut G, o: &TObjRt) {
    match o.coord_kind() {
        TEX_COORD_SHADOW => g.set_tex_coord_gen2(o.coord, GX_TG_MTX3x4, GX_TG_POS, GX_PNMTX0, false, o.mtxid),
        TEX_COORD_REFLECTION | TEX_COORD_HILIGHT => {
            g.set_tex_coord_gen2(o.coord, GX_TG_MTX3x4, GX_TG_NRM, GX_TEXMTX0, true, o.mtxid)
        }
        _ => {
            if o.bump() {
                // GXSetTexCoordGen = GXSetTexCoordGen2(.., GX_FALSE, GX_PTIDENTITY).
                g.set_tex_coord_gen2(o.coord, GX_TG_MTX2x4, o.src, o.mtxid, false, GX_PTIDENTITY)
            } else {
                g.set_tex_coord_gen2(o.coord, GX_TG_MTX2x4, o.src, GX_IDENTITY, false, o.mtxid)
            }
        }
    }
}

impl HsdRender {
    /// `HSD_TObjSetupTextureCoordGen` (bump's second texgen and toon's SRTG are not ported).
    pub fn tobj_setup_texture_coord_gen<G: GxBackend>(&mut self, g: &mut G, list: &[TObjRt]) {
        for o in list {
            if o.id == GX_TEXMAP_NULL {
                continue;
            }
            if o.coord_kind() == TEX_COORD_TOON {
                g.set_tex_coord_gen2(o.coord, GX_TG_SRTG, o.src, GX_IDENTITY, false, GX_PTIDENTITY);
            } else {
                setup_texture_coord_gen(g, o);
            }
        }
    }

    /// `HSD_TObjSetup`.
    pub fn tobj_setup<G: GxBackend>(&mut self, g: &mut G, d: &Dat, list: &mut [TObjRt]) {
        if list.is_empty() {
            return;
        }
        let num = assign_resources(list);
        if num > 0 {
            self.state_register_tex_gen(t::index_to_tex_coord(num - 1));
        }
        // tluts[8]: (n_entries, name) of each TLUT loaded this call.
        let mut tluts: Vec<(u16, u32)> = Vec::new();
        let (mut tlut_name, mut big_tlut_name) = (GX_TLUT0, GX_BIGTLUT0);
        for o in list.iter_mut() {
            if o.id == GX_TEXMAP_NULL {
                continue;
            }
            setup_mtx(g, o);
            let Some(img) = o.image else { continue };
            let Some(image_ptr) = d.ptr(img) else { continue };
            let lod = o.obj.lod.unwrap_or(HsdTexLodDesc {
                min_filt: GX_LIN_MIP_LIN,
                lod_bias: 0.0,
                bias_clamp: 0,
                edge_lod_enable: 0,
                max_anisotropy: GX_ANISO_1,
            });
            let mut min_filter = lod.min_filt;
            let format = d.u32_(img + 8);
            let mipmap = d.u32_(img + 0xC) != 0;
            let mut tlut_sel = None;
            match format {
                GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2 => {
                    let animated = (o.obj.tlut_no >= 0).then(|| o.obj.tluttbl_ptrs.get(o.obj.tlut_no as usize).copied()).flatten();
                    let Some(tl) = animated.or(o.tlut) else { continue };
                    let n = d.u16_(tl + 0xC);
                    let name = if let Some(&(_, nm)) = tluts.iter().find(|(ne, _)| *ne == n) {
                        nm
                    } else if tluts.len() < 8 {
                        let nm = if n >= 256 {
                            big_tlut_name += 1;
                            big_tlut_name - 1
                        } else {
                            tlut_name += 1;
                            tlut_name - 1
                        };
                        if let Some(lut) = d.ptr(tl) {
                            let bytes = d.tail(lut);
                            let len = (n as usize * 2).min(bytes.len());
                            g.load_tlut(lut, &bytes[..len], d.u32_(tl + 4), n, nm);
                        }
                        tluts.push((n, nm));
                        nm
                    } else {
                        GX_TLUT0
                    };
                    tlut_sel = Some(name);
                    if min_filter == GX_LIN_MIP_LIN {
                        min_filter = GX_LIN_MIP_NEAR;
                    }
                }
                0..=6 | 0xE => {}
                _ => continue, // tobj.c:0x677 asserts
            }
            if !mipmap {
                min_filter &= 1;
            }
            self.stats.tobjs += 1;
            g.load_texture(&TexLoad {
                key: image_ptr,
                image: d.tail(image_ptr),
                width: d.u16_(img + 4),
                height: d.u16_(img + 6),
                format,
                wrap_s: o.obj.wrap_s,
                wrap_t: o.obj.wrap_t,
                mipmap,
                tlut_name: tlut_sel,
                min_filt: min_filter,
                mag_filt: o.mag_filt,
                min_lod: d.f32_(img + 0x10),
                max_lod: d.f32_(img + 0x14),
                lod_bias: lod.lod_bias,
                bias_clamp: lod.bias_clamp != 0,
                edge_lod: lod.edge_lod_enable != 0,
                max_aniso: lod.max_anisotropy,
                map: o.id,
            });
        }
    }
}

// Native pointer API, matching the complete C translation unit.
#[allow(non_snake_case, non_upper_case_globals, unused_variables, dead_code, static_mut_refs)]
mod native {
use melee_types::*;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null_mut, addr_of_mut};
use crate::tobj as logic;
use crate::mtx as math;
use melee_core::fp::fmadds;

pub use crate::tobj::{tex_map_id_to_pt_tex_mtx as HSD_TexMapID2PTTexMtx, tg_tex_to_index as HSD_TGTex2Index, tex_coord_id_to_tex_gen_src as HSD_TexCoordID2TexGenSrc, tex_coord_to_index as HSD_TexCoord2Index, index_to_tex_coord as HSD_Index2TexCoord, tex_mtx_to_index as HSD_TexMtx2Index, index_to_tex_map as HSD_Index2TexMap, tex_map_to_index as HSD_TexMap2Index};

pub static mut hsdTObj: HSD_TObjInfo = {
    let mut info: HSD_TObjInfo = unsafe { core::mem::zeroed() };
    info.parent.head.info_init = Some(TObjInfoInit);
    info
};
pub static mut default_class: *mut HSD_TObjInfo = null_mut();
pub static mut tobj_head: *mut HSD_TObj = null_mut();

pub unsafe extern "C" fn HSD_TObjRemoveAnim(p: *mut HSD_TObj) {
    if !p.is_null() { ext::HSD_AObjRemove((*p).aobj); (*p).aobj = null_mut(); }
}
pub unsafe extern "C" fn HSD_TObjRemoveAnimAll(mut p: *mut HSD_TObj) {
    while !p.is_null() { HSD_TObjRemoveAnim(p); p = (*p).next; }
}
pub unsafe extern "C" fn lookupTextureAnim(id: i32, mut a: *mut HSD_TexAnim) -> *mut HSD_TexAnim {
    while !a.is_null() { if (*a).id == id as u32 { return a; } a = (*a).next; } null_mut()
}
pub unsafe extern "C" fn HSD_TObjAddAnim(p: *mut HSD_TObj, anim: *mut HSD_TexAnim) {
    if p.is_null() { return; }
    let a = lookupTextureAnim((*p).id as i32, anim);
    if a.is_null() { return; }
    if !(*p).aobj.is_null() { ext::HSD_AObjRemove((*p).aobj); }
    (*p).aobj = ext::HSD_AObjLoadDesc((*a).aobjdesc);
    (*p).imagetbl = (*a).imagetbl;
    if !(*p).tluttbl.is_null() {
        let mut i=0;
        while !(*(*p).tluttbl.add(i)).is_null() { HSD_TlutRemove(*(*p).tluttbl.add(i)); i+=1; }
        ext::HSD_Free((*p).tluttbl.cast());
    }
    if (*a).n_tluttbl != 0 {
        let n=(*a).n_tluttbl as usize;
        (*p).tluttbl=ext::HSD_MemAlloc(((n+1)*size_of::<*mut HSD_Tlut>()) as i32).cast();
        for i in 0..n { *(*p).tluttbl.add(i)=HSD_TlutLoadDesc(*(*a).tluttbl.add(i)); }
        *(*p).tluttbl.add(n)=null_mut();
    } else { (*p).tluttbl=null_mut(); }
    (*p).tlut_no=-1;
}
pub unsafe extern "C" fn HSD_TObjAddAnimAll(mut p:*mut HSD_TObj,a:*mut HSD_TexAnim) {
    while !p.is_null() { HSD_TObjAddAnim(p,a);p=(*p).next; }
}
pub unsafe extern "C" fn HSD_TObjReqAnimByFlags(p:*mut HSD_TObj,frame:f32,flags:u32) {
    if !p.is_null() && flags & 0x10 != 0 { ext::HSD_AObjReqAnim((*p).aobj,frame); }
}
pub unsafe extern "C" fn HSD_TObjReqAnimAllByFlags(mut p:*mut HSD_TObj,frame:f32,flags:u32) {
    while !p.is_null() {HSD_TObjReqAnimByFlags(p,frame,flags);p=(*p).next;}
}
pub unsafe extern "C" fn HSD_TObjReqAnim(p:*mut HSD_TObj,frame:f32) {HSD_TObjReqAnimByFlags(p,frame,0x10);}
pub unsafe extern "C" fn HSD_TObjReqAnimAll(p:*mut HSD_TObj,frame:f32) {HSD_TObjReqAnimAllByFlags(p,frame,0x10);}

fn vec(v:Vec3)->math::Vec3 {math::Vec3{x:v.x,y:v.y,z:v.z}}
fn color(v:GXColor)->crate::hsd_util::GxColor {crate::hsd_util::GxColor{r:v.r,g:v.g,b:v.b,a:v.a}}
fn native_color(v:crate::hsd_util::GxColor)->GXColor {GXColor{r:v.r,g:v.g,b:v.b,a:v.a}}

/// Adapt the generated native record to the numerical port's already verified owned view.
/// No borrowed native pointers escape this call; only fields that an update changes are copied back.
unsafe fn logic_view(p:*mut HSD_TObj)->logic::HsdTObj<*mut HSD_ImageDesc> {
    let x=&*p;
    logic::HsdTObj{
        rotate:math::Quaternion{x:x.rotate.x,y:x.rotate.y,z:x.rotate.z,w:x.rotate.w},
        scale:vec(x.scale),translate:vec(x.translate),wrap_s:x.wrap_s,wrap_t:x.wrap_t,
        repeat_s:x.repeat_s,repeat_t:x.repeat_t,flags:x.flags,blending:x.blending,
        imagedesc:if x.imagedesc.is_null(){None}else{Some(x.imagedesc)},
        lod:x.lod.as_ref().map(|l|logic::HsdTexLodDesc{min_filt:l.minFilt,lod_bias:l.LODBias,bias_clamp:l.bias_clamp,edge_lod_enable:l.edgeLODEnable,max_anisotropy:l.max_anisotropy}),
        imagetbl:None,has_tluttbl:!x.tluttbl.is_null(),tluttbl_ptrs:std::vec::Vec::new(),tlut_no:x.tlut_no,mtx:x.mtx,
        tev:x.tev.as_ref().map(|v|logic::HsdTObjTev{konst:color(v.konst),tev0:color(v.tev0),tev1:color(v.tev1)}),
    }
}
pub unsafe extern "C" fn TObjUpdateFunc(obj:*mut c_void,kind:i32,value:*mut HSD_ObjData) {
    if obj.is_null(){return;}
    let p=obj.cast::<HSD_TObj>();
    let mut view=logic_view(p);
    let mut fv=(*value).fv;
    if kind==1 {
        assert!(!(*p).imagetbl.is_null(),"tobj.c:276 imagetbl");
        let image=*(*p).imagetbl.offset(crate::cabi::f2i32(fv as f64) as isize);
        view.imagetbl=Some(vec![if image.is_null(){None}else{Some(image)}]);
        fv=0.0; // The selected C slot is represented as slot zero in the owned view.
    }
    view.update_func(kind as u32,fv);
    let x=&mut *p;
    x.rotate=Quaternion{x:view.rotate.x,y:view.rotate.y,z:view.rotate.z,w:view.rotate.w};
    x.scale=Vec3{x:view.scale.x,y:view.scale.y,z:view.scale.z};
    x.translate=Vec3{x:view.translate.x,y:view.translate.y,z:view.translate.z};
    x.flags=view.flags;x.blending=view.blending;x.imagedesc=view.imagedesc.unwrap_or(null_mut());x.tlut_no=view.tlut_no;
    if let Some(l)=view.lod {(*x.lod).LODBias=l.lod_bias;}
    if let Some(t)=view.tev {(*x.tev).konst=native_color(t.konst);(*x.tev).tev0=native_color(t.tev0);(*x.tev).tev1=native_color(t.tev1);}
}
pub unsafe extern "C" fn HSD_TObjAnim(p:*mut HSD_TObj) {
    if !p.is_null(){ext::HSD_AObjInterpretAnim((*p).aobj,p.cast(),Some(TObjUpdateFunc));}
}
pub unsafe extern "C" fn HSD_TObjAnimAll(mut p:*mut HSD_TObj) {while !p.is_null(){HSD_TObjAnim(p);p=(*p).next;}}

pub unsafe extern "C" fn TObjLoad(p:*mut HSD_TObj,d:*mut HSD_TObjDesc)->i32 {
    (*p).next=HSD_TObjLoadDesc((*d).next);
    (*p).id=(*d).id;(*p).src=(*d).src;(*p).mtxid=GX_IDENTITY;
    (*p).rotate.x=(*d).rotate.x;(*p).rotate.y=(*d).rotate.y;(*p).rotate.z=(*d).rotate.z;
    (*p).scale=(*d).scale;(*p).translate=(*d).translate;(*p).wrap_s=(*d).wrap_s;(*p).wrap_t=(*d).wrap_t;
    (*p).repeat_s=(*d).repeat_s;(*p).repeat_t=(*d).repeat_t;(*p).flags=(*d).blend_flags;(*p).blending=(*d).blending;
    (*p).magFilt=(*d).magFilt;(*p).imagedesc=(*d).imagedesc;(*p).tlut=HSD_TlutLoadDesc((*d).tlutdesc);
    (*p).lod=(*d).lod;(*p).aobj=null_mut();(*p).flags|=logic::TEX_MTX_DIRTY;(*p).tlut_no=-1;(*p).tev=HSD_TObjTevLoadDesc((*d).tev);0
}
pub unsafe extern "C" fn HSD_TObjLoadDesc(d:*mut HSD_TObjDesc)->*mut HSD_TObj {
    if d.is_null(){return null_mut();}
    let ci=if (*d).class_name.is_null(){null_mut()}else{ext::hsdSearchClassInfo((*d).class_name)};
    let p=if ci.is_null(){HSD_TObjAlloc()}else{ext::hsdNew(ci).cast::<HSD_TObj>()};
    assert!(!p.is_null(),"tobj.c:468 tobj");
    let info=(*p).parent.parent.class_info.cast::<HSD_TObjInfo>();
    ((*info).load.expect("TObj load method"))(p,d);p
}
pub unsafe extern "C" fn HSD_TlutLoadDesc(d:*mut HSD_TlutDesc)->*mut HSD_Tlut {
    if d.is_null(){return null_mut();}let p=HSD_TlutAlloc();core::ptr::copy_nonoverlapping(d.cast::<u8>(),p.cast(),size_of::<HSD_Tlut>());p
}
pub unsafe extern "C" fn HSD_TObjTevLoadDesc(d:*mut HSD_TObjTevDesc)->*mut HSD_TObjTev {
    if d.is_null(){return null_mut();}let p=HSD_TObjTevAlloc();core::ptr::copy_nonoverlapping(d.cast::<u8>(),p.cast(),size_of::<HSD_TObjTev>());p
}
pub unsafe extern "C" fn _HSD_TObjGetCurrentByType(from:*mut HSD_TObj,mapping:u32)->*mut HSD_TObj {
    let mut p=if from.is_null(){tobj_head}else{(*from).next};
    while !p.is_null(){if (*p).flags&15==mapping{return p;}p=(*p).next;}null_mut()
}
/// tobj.c `MakeTextureMtx`: translate, rotate and scale matrices concatenated onto `tobj->mtx` through the
/// matrix library (callees are hooks, like the C).
pub unsafe extern "C" fn MakeTextureMtx(tobj: *mut HSD_TObj) {
    let t = &mut *tobj;
    assert!(t.repeat_s != 0 && t.repeat_t != 0, "tobj.c:589 tobj->repeat_s && tobj->repeat_t");
    let sx = if t.scale.x.abs() < f32::EPSILON { 0.0 } else { t.repeat_s as f32 / t.scale.x };
    let sy = if t.scale.y.abs() < f32::EPSILON { 0.0 } else { t.repeat_t as f32 / t.scale.y };
    let sz = t.scale.z;
    let mut rot = Quaternion { x: t.rotate.x, y: t.rotate.y, z: -t.rotate.z, w: 0.0 };
    let tx = -t.translate.x;
    let ty = -(t.translate.y + (if t.wrap_t == GX_MIRROR { 1.0 / (t.repeat_t as f32 / t.scale.y) } else { 0.0 }));
    let tz = t.translate.z;
    let mut m = [[0.0f32; 4]; 3];
    let mp = m.as_mut_ptr().cast::<c_void>();
    let mtxp = addr_of_mut!(t.mtx).cast::<c_void>();
    ext::PSMTXTrans(mtxp, tx, ty, tz);
    ext::HSD_MkRotationMtx(mp, addr_of_mut!(rot).cast());
    ext::PSMTXConcat(mp, mtxp, mtxp);
    ext::PSMTXScale(mp, sx, sy, sz);
    ext::PSMTXConcat(mp, mtxp, mtxp);
}
pub unsafe extern "C" fn TObjSetupMtx(p: *mut HSD_TObj) {
    let kind = (*p).flags & 15;
    if kind == super::TEX_COORD_TOON {
        return;
    }
    if (*p).flags & logic::TEX_MTX_DIRTY != 0 {
        let ci = (*p).parent.parent.class_info.cast::<HSD_TObjInfo>();
        ((*ci).make_mtx.expect("TObj make_mtx method"))(p);
        (*p).flags &= !logic::TEX_MTX_DIRTY;
    }
    let m = (*p).mtx;
    match kind {
        super::TEX_COORD_REFLECTION => {
            let mut r = [[0.0f32; 4]; 3];
            for i in 0..3 {
                r[i][0] = 0.5 * m[i][0];
                r[i][1] = -0.5 * m[i][1];
                r[i][2] = 0.0;
                r[i][3] = fmadds(0.5, m[i][0], 0.5 * m[i][1]) + m[i][2] + m[i][3];
            }
            ext::GXLoadTexMtxImm(r.as_mut_ptr().cast(), (*p).mtxid, GX_MTX3x4);
        }
        super::TEX_COORD_HILIGHT => {
            let light = ext::HSD_LObjGetCurrentByType(1);
            let mut r = [[0.0f32; 4]; 3];
            if !light.is_null() {
                let camera = ext::HSD_CObjGetCurrent();
                assert!(!camera.is_null(), "tobj.c:680 cobj");
                let vmtx = addr_of_mut!((*camera).view_mtx).cast::<c_void>();
                let mut ldir = Vec3::zeroed();
                let mut half = Vec3::zeroed();
                ext::HSD_LObjGetLightVector(light, &mut ldir);
                ext::PSMTXMultVecSR(vmtx, &mut ldir, &mut ldir);
                ldir.z += -1.0;
                ext::PSVECNormalize(&mut ldir, &mut half);
                half.x = (half.x as f64 * -0.5) as f32;
                half.y = (half.y as f64 * -0.5) as f32;
                half.z = (half.z as f64 * -0.5) as f32;
                r[0][0] = m[0][0] * half.x;
                r[0][1] = m[0][0] * half.y;
                r[0][2] = m[0][0] * half.z;
                r[0][3] = fmadds(m[0][0], 0.5, m[0][3]);
                r[1][0] = m[1][0] * half.x;
                r[1][1] = m[1][0] * half.y;
                r[1][2] = m[1][0] * half.z;
                r[1][3] = fmadds(m[1][0], 0.5, m[1][3]);
                r[2][0] = 0.0;
                r[2][1] = 0.0;
                r[2][2] = 0.0;
                r[2][3] = 1.0;
            } else {
                r[0][3] = 1.0;
                r[1][3] = 1.0;
            }
            ext::GXLoadTexMtxImm(r.as_mut_ptr().cast(), (*p).mtxid, GX_MTX3x4);
        }
        super::TEX_COORD_SHADOW => {
            let c = ext::HSD_CObjGetCurrent();
            let inv = ext::HSD_CObjGetInvViewingMtxPtrDirect(c);
            let mut r = [[0.0f32; 4]; 3];
            ext::PSMTXConcat(addr_of_mut!((*p).mtx).cast(), inv, r.as_mut_ptr().cast());
            ext::GXLoadTexMtxImm(r.as_mut_ptr().cast(), (*p).mtxid, GX_MTX3x4);
        }
        _ => ext::GXLoadTexMtxImm(addr_of_mut!((*p).mtx).cast(), (*p).mtxid, if (*p).flags & super::TEX_BUMP != 0 { GX_MTX2x4 } else { GX_MTX3x4 }),
    }
}
pub unsafe extern "C" fn setupTextureCoordGen(p:*mut HSD_TObj){
    let x=&*p;
    match x.flags&15{
        super::TEX_COORD_SHADOW=>ext::GXSetTexCoordGen2(x.coord,GX_TG_MTX3x4,GX_TG_POS,GX_PNMTX0,0,x.mtxid),
        super::TEX_COORD_REFLECTION|super::TEX_COORD_HILIGHT=>ext::GXSetTexCoordGen2(x.coord,GX_TG_MTX3x4,GX_TG_NRM,GX_TEXMTX0,1,x.mtxid),
        _=>if x.flags&super::TEX_BUMP!=0{ext::GXSetTexCoordGen2(x.coord,GX_TG_MTX2x4,x.src,x.mtxid,0,GX_PTIDENTITY)}
            else{ext::GXSetTexCoordGen2(x.coord,GX_TG_MTX2x4,x.src,GX_IDENTITY,0,x.mtxid)},
    }
}
pub unsafe extern "C" fn setupTextureCoordGenBump(p:*mut HSD_TObj){
    let mask=ext::HSD_LObjGetLightMaskDiffuse();let i=(mask&255).trailing_zeros();let i=if i>=8{0}else{i};
    ext::GXSetTexCoordGen2((*p).coord.wrapping_add(1),GX_TG_BUMP0+i,HSD_TexCoordID2TexGenSrc((*p).coord),GX_IDENTITY,0,GX_PTIDENTITY);
}
pub unsafe extern "C" fn setupTextureCoordGenToon(p:*mut HSD_TObj){ext::GXSetTexCoordGen2((*p).coord,GX_TG_SRTG,(*p).src,GX_IDENTITY,0,GX_PTIDENTITY);}
pub unsafe extern "C" fn HSD_TObjSetupTextureCoordGen(mut p:*mut HSD_TObj){
    while !p.is_null(){
        if (*p).id!=GX_TEXMAP_NULL{
            if (*p).flags&super::TEX_BUMP!=0{setupTextureCoordGen(p);setupTextureCoordGenBump(p);}
            else if (*p).flags&15==super::TEX_COORD_TOON{setupTextureCoordGenToon(p);}else{setupTextureCoordGen(p);}
        }p=(*p).next;
    }
}

/// The C statics contain only scalar TEV configuration; constructing the same initial value
/// per invocation is equivalent because every persistent field they modify is overwritten.
fn volatile_tev(bump:bool)->HSD_TevDesc{
    let mut d=HSD_TevDesc::zeroed();d.flags=TEVCONF_MODE as u32;d.stage=u32::MAX;d.coord=GX_TEXCOORD_NULL;d.map=GX_TEXMAP_NULL;
    d.color=if bump{GX_COLOR0A0}else{GX_COLOR_NULL};
    let mut c=_HSD_TevConf::zeroed();
    c.clr_op=if bump{GX_TEV_SUB}else{GX_TEV_ADD};c.clr_a=GX_CC_ZERO;c.clr_b=if bump{GX_CC_TEXC}else{GX_CC_CPREV};
    c.clr_c=if bump{GX_CC_RASC}else{GX_CC_TEXC};c.clr_d=if bump{GX_CC_CPREV}else{GX_CC_ZERO};
    c.clr_scale=GX_CS_SCALE_1;c.clr_bias=GX_TB_ZERO;c.clr_clamp=1;c.clr_out_reg=GX_TEVPREV;
    c.alpha_op=if bump{GX_TEV_SUB}else{GX_TEV_ADD};c.alpha_a=GX_CA_ZERO;c.alpha_b=GX_CA_ZERO;c.alpha_c=GX_CA_ZERO;c.alpha_d=GX_CA_APREV;
    c.alpha_scale=GX_CS_SCALE_1;c.alpha_bias=GX_TB_ZERO;c.alpha_clamp=1;c.alpha_out_reg=GX_TEVPREV;c.mode=GX_TC_LINEAR;
    d.u.tevconf=c;d
}
pub unsafe extern "C" fn TObjSetupTevModulateShadow(mut p:*mut HSD_TObj){
    let mut d=volatile_tev(false);
    while !p.is_null()&&(*p).flags&15==super::TEX_COORD_SHADOW{
        d.stage=ext::HSD_StateAssignTev() as u32;d.coord=(*p).coord;d.map=(*p).id;ext::HSD_SetupTevStage(&mut d);p=(*p).next;
    }
}
pub unsafe extern "C" fn SetupEmbossBumpTev(p:*mut HSD_TObj){
    let mut d=volatile_tev(true);d.stage=ext::HSD_StateAssignTev() as u32;d.coord=(*p).coord;d.map=(*p).id;
    d.u.tevconf.clr_op=GX_TEV_ADD;d.u.tevconf.alpha_op=GX_TEV_ADD;d.u.tevconf.clr_clamp=0;ext::HSD_SetupTevStage(&mut d);
    d.stage=ext::HSD_StateAssignTev() as u32;d.coord=(*p).coord.wrapping_add(1);
    d.u.tevconf.clr_op=GX_TEV_SUB;d.u.tevconf.alpha_op=GX_TEV_SUB;d.u.tevconf.clr_clamp=1;ext::HSD_SetupTevStage(&mut d);
}
pub unsafe extern "C" fn HSD_TObjSetupVolatileTev(mut p:*mut HSD_TObj,_rendermode:u32){
    while !p.is_null(){
        if (*p).id!=GX_TEXMAP_NULL{
            if (*p).flags&super::TEX_BUMP!=0{SetupEmbossBumpTev(p);}
            if (*p).flags&super::TEX_LIGHTMAP_SHADOW!=0{TObjSetupTevModulateShadow(p);break;}
        }p=(*p).next;
    }
}

pub unsafe extern "C" fn MakeColorGenTExp(_lightmap:u32,p:*mut HSD_TObj,c:*mut *mut HSD_TExp,a:*mut *mut HSD_TExp,list:*mut *mut HSD_TExp,_repeat:i32){
    let tev=(*p).tev;let mut used=[false;9];
    let colors=[(*tev).color_a,(*tev).color_b,(*tev).color_c,(*tev).color_d];
    let alphas=[(*tev).alpha_a,(*tev).alpha_b,(*tev).alpha_c,(*tev).alpha_d];
    for n in colors{if (128..=136).contains(&n){used[(n-128) as usize]=true;}}
    for n in alphas{match n{64..=67=>used[(n-63) as usize]=true,68=>used[6]=true,69=>used[8]=true,_=>{}}}
    let ptrs=[addr_of_mut!((*tev).konst).cast::<c_void>(),addr_of_mut!((*tev).konst.r).cast(),addr_of_mut!((*tev).konst.g).cast(),addr_of_mut!((*tev).konst.b).cast(),addr_of_mut!((*tev).konst.a).cast(),addr_of_mut!((*tev).tev0).cast(),addr_of_mut!((*tev).tev0.a).cast(),addr_of_mut!((*tev).tev1).cast(),addr_of_mut!((*tev).tev1.a).cast()];
    let mut constants=[null_mut();9];
    for i in 0..9{if used[i]{constants[i]=ext::HSD_TExpCnst(ptrs[i],if matches!(i,0|5|7){HSD_TE_RGB}else{HSD_TE_X},HSD_TE_U8,list);}}
    let e=ext::HSD_TExpTev(list);ext::HSD_TExpOrder(e,p,GX_COLOR_NULL);
    if (*tev).active&(1<<30)!=0{
        let mut sel=[HSD_TE_0;4];let mut exp=[null_mut();4];
        for i in 0..4{
            match colors[i] as u32{
                GX_CC_ZERO=>{},GX_CC_ONE=>sel[i]=HSD_TE_1,GX_CC_HALF=>sel[i]=HSD_TE_4_8,
                GX_CC_TEXC=>{sel[i]=HSD_TE_RGB;exp[i]=usize::MAX as *mut HSD_TExp;},
                GX_CC_TEXA=>{sel[i]=HSD_TE_A;exp[i]=usize::MAX as *mut HSD_TExp;},
                128..=132=>{let n=(colors[i]-128) as usize;sel[i]=if n==0{HSD_TE_RGB}else{HSD_TE_X};exp[i]=constants[n];},
                133..=136=>{
                    let n=(colors[i]-128) as usize;let tmp=ext::HSD_TExpTev(list);ext::HSD_TExpOrder(tmp,null_mut(),GX_COLOR_NULL);
                    ext::HSD_TExpColorOp(tmp,GX_TEV_ADD,GX_TB_ZERO,GX_CS_SCALE_1,1);
                    ext::HSD_TExpColorIn(tmp,HSD_TE_0,null_mut(),HSD_TE_0,null_mut(),HSD_TE_0,null_mut(),if n==5||n==7{HSD_TE_RGB}else{HSD_TE_X},constants[n]);
                    sel[i]=HSD_TE_RGB;exp[i]=tmp;
                }
                _=>panic!("tobj.c:1191 color selector"),
            }
        }
        ext::HSD_TExpColorOp(e,(*tev).color_op as u32,(*tev).color_bias as u32,(*tev).color_scale as u32,(*tev).color_clamp);
        ext::HSD_TExpColorIn(e,sel[0],exp[0],sel[1],exp[1],sel[2],exp[2],sel[3],exp[3]);*c=e;
    }
    if (*tev).active&(1<<31)!=0{
        let mut sel=[HSD_TE_0;4];let mut exp=[null_mut();4];
        for i in 0..4{match alphas[i] as u32{
            GX_CA_ZERO=>{},GX_CA_TEXA=>{sel[i]=HSD_TE_A;exp[i]=usize::MAX as *mut HSD_TExp;},
            64..=67=>{sel[i]=HSD_TE_X;exp[i]=constants[(alphas[i]-63) as usize];},
            68..=69=>{
                let n=if alphas[i]==68{6}else{8};let tmp=ext::HSD_TExpTev(list);ext::HSD_TExpOrder(tmp,null_mut(),GX_COLOR_NULL);
                ext::HSD_TExpAlphaOp(tmp,GX_TEV_ADD,GX_TB_ZERO,GX_CS_SCALE_1,1);
                ext::HSD_TExpAlphaIn(tmp,HSD_TE_0,null_mut(),HSD_TE_0,null_mut(),HSD_TE_0,null_mut(),HSD_TE_X,constants[n]);sel[i]=HSD_TE_A;exp[i]=tmp;
            }
            _=>panic!("tobj.c:1264 alpha selector"),
        }}
        ext::HSD_TExpAlphaOp(e,(*tev).alpha_op as u32,(*tev).alpha_bias as u32,(*tev).alpha_scale as u32,(*tev).alpha_clamp);
        ext::HSD_TExpAlphaIn(e,sel[0],exp[0],sel[1],exp[1],sel[2],exp[2],sel[3],exp[3]);*a=e;
    }
}
pub unsafe extern "C" fn TObjMakeTExp(p:*mut HSD_TObj,lightmap:u32,done:u32,c:*mut *mut HSD_TExp,a:*mut *mut HSD_TExp,list:*mut *mut HSD_TExp){
    let repeat=done&((*p).flags&super::TEX_LIGHTMAP_MASK);
    let mut cs=usize::MAX as *mut HSD_TExp;let mut als=cs;
    let e=ext::HSD_TExpTev(list);
    if !(*p).tev.is_null()&&(*(*p).tev).active&0xC0000000!=0{MakeColorGenTExp(lightmap,p,&mut cs,&mut als,list,repeat as i32);}
    ext::HSD_TExpOrder(e,p,GX_COLOR_NULL);
    let z=(HSD_TE_0,null_mut());let ci=(HSD_TE_RGB,*c);let src=(HSD_TE_RGB,cs);let alpha=(HSD_TE_A,als);
    let map=((*p).flags>>16)&15;
    let inp=match map{
        1=>[ci,src,alpha,z],2=>[ci,src,src,z],
        3=>{let cn=ext::HSD_TExpCnst(addr_of_mut!((*p).blending).cast(),HSD_TE_X,HSD_TE_F32,list);[ci,src,(HSD_TE_X,cn),z]},
        4=>[z,ci,src,z],5=>[z,z,z,src],0|6=>[z,z,z,ci],7|8=>[src,z,z,ci],_=>panic!("tobj.c:1372 colormap"),
    };
    ext::HSD_TExpColorOp(e,if map==8{GX_TEV_SUB}else{GX_TEV_ADD},GX_TB_ZERO,GX_CS_SCALE_1,1);
    ext::HSD_TExpColorIn(e,inp[0].0,inp[0].1,inp[1].0,inp[1].1,inp[2].0,inp[2].1,inp[3].0,inp[3].1);*c=e;
    if repeat==0{
        let ai=(HSD_TE_A,*a);let map=((*p).flags>>20)&15;
        let inp=match map{
            1=>[ai,alpha,alpha,z],
            2=>{let cn=ext::HSD_TExpCnst(addr_of_mut!((*p).blending).cast(),HSD_TE_X,HSD_TE_F32,list);[ai,alpha,(HSD_TE_X,cn),z]},
            3=>[z,ai,alpha,z],4=>[z,z,z,alpha],0|5=>[z,z,z,ai],6|7=>[alpha,z,z,ai],_=>panic!("tobj.c:1430 alphamap"),
        };
        ext::HSD_TExpAlphaOp(e,if map==7{GX_TEV_SUB}else{GX_TEV_ADD},GX_TB_ZERO,GX_CS_SCALE_1,1);
        ext::HSD_TExpAlphaIn(e,inp[0].0,inp[0].1,inp[1].0,inp[1].1,inp[2].0,inp[2].1,inp[3].0,inp[3].1);*a=e;
    }
}

pub unsafe extern "C" fn HSD_TObjAssignResources(mut p:*mut HSD_TObj)->i32{
    let mut pointers=std::vec::Vec::new();let mut objects=std::vec::Vec::new();
    while !p.is_null(){
        let x=&*p;pointers.push(p);
        objects.push(super::TObjRt{desc:0,id:x.id,src:x.src,mtxid:x.mtxid,coord:x.coord,mag_filt:x.magFilt,image:None,tlut:None,tev:None,
            obj:logic::HsdTObj{flags:x.flags,..Default::default()}});p=x.next;
    }
    let n=super::assign_resources(&mut objects);
    for (p,x) in pointers.into_iter().zip(objects){(*p).id=x.id;(*p).coord=x.coord;(*p).mtxid=x.mtxid;}n as i32
}
pub unsafe extern "C" fn DifferentTluts(a:*mut HSD_Tlut,b:*mut HSD_Tlut)->i32{
    logic::different_tluts(&logic::HsdTlut{n_entries:(*a).n_entries,..Default::default()},&logic::HsdTlut{n_entries:(*b).n_entries,..Default::default()}) as i32
}
pub unsafe extern "C" fn HSD_TObjSetup(mut p:*mut HSD_TObj){
    tobj_head=p;if p.is_null(){return;}
    let n=HSD_TObjAssignResources(p);if n>0{ext::HSD_StateRegisterTexGen(HSD_Index2TexCoord((n-1) as u32) as i32);}
    let mut tluts=[null_mut::<HSD_Tlut>();8];let mut count=0;let mut small=GX_TLUT0;let mut big=GX_BIGTLUT0;
    let default_lod=HSD_TexLODDesc{minFilt:GX_LIN_MIP_LIN,LODBias:0.0,bias_clamp:0,edgeLODEnable:0,max_anisotropy:GX_ANISO_1};
    let mut texobj=GXTexObj::zeroed();let mut tlutobj=GXTlutObj::zeroed();
    while !p.is_null(){
        let image=(*p).imagedesc;
        if (*p).id!=GX_TEXMAP_NULL{
            TObjSetupMtx(p);assert!(!image.is_null(),"tobj.c:1578 imagedesc");assert!(!(*image).image_ptr.is_null(),"tobj.c:1579 image_ptr");
            let lod=(*p).lod.as_ref().unwrap_or(&default_lod);let mut min=lod.minFilt;let im=&*image;
            match im.format{
                GX_TF_C4|GX_TF_C8|GX_TF_C14X2=>{
                    let tl=if (*p).tlut_no!=-1{*(*p).tluttbl.offset((*p).tlut_no as isize)}else{(*p).tlut};assert!(!tl.is_null(),"tobj.c:1595 tlut");
                    let mut i=0;while i<count&&DifferentTluts(tluts[i],tl)!=0{i+=1;}
                    if i<count{(*tl).tlut_name=(*tluts[i]).tlut_name;}
                    else if count<8{
                        (*tl).tlut_name=if (*tl).n_entries>=256{let n=big;big+=1;n}else{let n=small;small+=1;n};
                        ext::GXInitTlutObj(&mut tlutobj,(*tl).lut,(*tl).fmt,(*tl).n_entries);ext::GXLoadTlut(&mut tlutobj,(*tl).tlut_name);tluts[count]=tl;count+=1;
                    }else{(*tl).tlut_name=GX_TLUT0;}
                    ext::GXInitTexObjCI(&mut texobj,im.image_ptr,im.width,im.height,im.format,(*p).wrap_s,(*p).wrap_t,(im.mipmap!=0) as u8,(*tl).tlut_name);
                    if min==GX_LIN_MIP_LIN{min=GX_LIN_MIP_NEAR;}
                }
                GX_TF_I4|GX_TF_I8|GX_TF_IA4|GX_TF_IA8|GX_TF_RGB565|GX_TF_RGB5A3|GX_TF_RGBA8|GX_TF_CMPR=>
                    ext::GXInitTexObj(&mut texobj,im.image_ptr,im.width,im.height,im.format,(*p).wrap_s,(*p).wrap_t,(im.mipmap!=0) as u8),
                _=>panic!("tobj.c:1655 format"),
            }
            if im.mipmap==0{min&=1;}
            ext::GXInitTexObjLOD(&mut texobj,min,(*p).magFilt,im.minLOD,im.maxLOD,lod.LODBias,lod.bias_clamp,lod.edgeLODEnable,lod.max_anisotropy);ext::GXLoadTexObj(&mut texobj,(*p).id);
        }p=(*p).next;
    }
}
pub unsafe extern "C" fn HSD_TObjRemove(p:*mut HSD_TObj){
    if !p.is_null(){let ci=(*p).parent.parent.class_info;((*ci).release.expect("release"))(p.cast());let ci=(*p).parent.parent.class_info;((*ci).destroy.expect("destroy"))(p.cast());}
}
pub unsafe extern "C" fn HSD_TObjRemoveAll(mut p:*mut HSD_TObj){while !p.is_null(){let next=(*p).next;HSD_TObjRemove(p);p=next;}}
pub unsafe extern "C" fn HSD_TObjGetNext(p:*mut HSD_TObj)->*mut HSD_TObj{if p.is_null(){null_mut()}else{(*p).next}}
pub unsafe extern "C" fn HSD_TObjSetDefaultClass(info:*mut HSD_TObjInfo){
    if !info.is_null(){assert!(ext::hsdIsDescendantOf(info.cast(),addr_of_mut!(hsdTObj).cast())!=0,"tobj.c:2027 descendant");}default_class=info;
}
pub unsafe extern "C" fn HSD_TObjGetDefaultClass()->*mut HSD_TObjInfo{if default_class.is_null(){addr_of_mut!(hsdTObj)}else{default_class}}
pub unsafe extern "C" fn HSD_TObjAlloc()->*mut HSD_TObj{let p=ext::hsdNew(addr_of_mut!((*HSD_TObjGetDefaultClass()).parent)).cast::<HSD_TObj>();assert!(!p.is_null(),"tobj.c:2040 new");p}
pub unsafe extern "C" fn HSD_TObjFree(p:*mut HSD_TObj){if !p.is_null(){((*(*p).parent.parent.class_info).destroy.expect("destroy"))(p.cast());}}
pub unsafe extern "C" fn HSD_TlutAlloc()->*mut HSD_Tlut{let p=ext::hsdAllocMemPiece(size_of::<HSD_Tlut>() as i32).cast::<HSD_Tlut>();assert!(!p.is_null(),"tobj.c:2069 tlut");core::ptr::write_bytes(p,0,1);p}
pub unsafe extern "C" fn HSD_TlutFree(p:*mut HSD_Tlut){ext::hsdFreeMemPiece(p.cast(),size_of::<HSD_Tlut>() as i32);}
pub unsafe extern "C" fn HSD_TlutRemove(p:*mut HSD_Tlut){if !p.is_null(){ext::hsdFreeMemPiece(p.cast(),size_of::<HSD_Tlut>() as i32);}}
pub unsafe extern "C" fn HSD_TObjTevAlloc()->*mut HSD_TObjTev{let p=ext::hsdAllocMemPiece(size_of::<HSD_TObjTev>() as i32).cast::<HSD_TObjTev>();assert!(!p.is_null(),"tobj.c:2112 tev");core::ptr::write_bytes(p,0,1);p}
pub unsafe extern "C" fn HSD_TObjTevFree(p:*mut HSD_TObjTev){ext::hsdFreeMemPiece(p.cast(),size_of::<HSD_TObjTev>() as i32);}
pub unsafe extern "C" fn HSD_TObjTevRemove(p:*mut HSD_TObjTev){if !p.is_null(){HSD_TObjTevFree(p);}}
pub unsafe extern "C" fn HSD_ImageDescAlloc()->*mut HSD_ImageDesc{let p=ext::hsdAllocMemPiece(size_of::<HSD_ImageDesc>() as i32).cast::<HSD_ImageDesc>();assert!(!p.is_null(),"tobj.c:2155 idesc");core::ptr::write_bytes(p,0,1);p}
pub unsafe extern "C" fn HSD_ImageDescFree(p:*mut HSD_ImageDesc){ext::hsdFreeMemPiece(p.cast(),size_of::<HSD_ImageDesc>() as i32);}
pub unsafe extern "C" fn HSD_ImageDescCopyFromEFB(p:*mut HSD_ImageDesc,x:u16,y:u16,clear:u8,sync:i32){
    if p.is_null(){return;}ext::GXSetTexCopySrc(x,y,(*p).width,(*p).height);ext::GXSetTexCopyDst((*p).width,(*p).height,(*p).format,((*p).mipmap!=0) as u8);
    if clear!=0{ext::HSD_StateSetZMode(1,GX_LEQUAL as i32,1);}ext::GXCopyTex((*p).image_ptr,clear);if sync!=0{ext::GXPixModeSync();ext::GXInvalidateTexAll();}
}
pub unsafe extern "C" fn TObjRelease(o:*mut HSD_Class){
    let p=o.cast::<HSD_TObj>();ext::HSD_AObjRemove((*p).aobj);HSD_TlutRemove((*p).tlut);HSD_TObjTevRemove((*p).tev);
    if !(*p).tluttbl.is_null(){let mut i=0;while !(*(*p).tluttbl.add(i)).is_null(){HSD_TlutRemove(*(*p).tluttbl.add(i));i+=1;}ext::HSD_Free((*p).tluttbl.cast());}
    ((*hsdTObj.parent.head.parent).release.expect("parent release"))(o);
}
pub unsafe extern "C" fn TObjAmnesia(info:*mut HSD_ClassInfo){
    if info==default_class.cast(){default_class=null_mut();}if info==addr_of_mut!(hsdTObj).cast(){tobj_head=null_mut();}
    ((*hsdTObj.parent.head.parent).amnesia.expect("parent amnesia"))(info);
}
pub unsafe extern "C" fn TObjInfoInit(){
    ext::hsdInitClassInfo(addr_of_mut!(hsdTObj.parent),addr_of_mut!(ext::hsdObj),c"sysdolphin_base_library".as_ptr().cast_mut(),c"hsd_tobj".as_ptr().cast_mut(),size_of::<HSD_TObjInfo>() as i32,size_of::<HSD_TObj>() as i32);
    hsdTObj.parent.release=Some(TObjRelease);hsdTObj.parent.amnesia=Some(TObjAmnesia);hsdTObj.load=Some(TObjLoad);hsdTObj.make_texp=Some(TObjMakeTExp);hsdTObj.make_mtx=Some(MakeTextureMtx);
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
    fn GXCopyTex(a0: *mut c_void, a1: u8);
    fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8);
    fn GXInitTexObjCI(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8, a8: u32);
    fn GXInitTexObjLOD(a0: *mut _GXTexObj, a1: u32, a2: u32, a3: f32, a4: f32, a5: f32, a6: u8, a7: u8, a8: u32);
    fn GXInitTlutObj(a0: *mut _GXTlutObj, a1: *mut c_void, a2: u32, a3: u16);
    fn GXInvalidateTexAll();
    fn GXLoadTexMtxImm(a0: *mut c_void, a1: u32, a2: u32);
    fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32);
    fn GXLoadTlut(a0: *mut _GXTlutObj, a1: u32);
    fn GXPixModeSync();
    fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTexCopyDst(a0: u16, a1: u16, a2: u32, a3: u8);
    fn GXSetTexCopySrc(a0: u16, a1: u16, a2: u16, a3: u16);
    fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
    fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj = { let (a0,) = (a0,); crate::tu::aobj::HSD_AObjLoadDesc(a0) };
    fn HSD_AObjRemove(a0: *mut HSD_AObj) = { let (a0,) = (a0,); crate::tu::aobj::HSD_AObjRemove(a0) };
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) = { let (a0, a1) = (a0, a1); crate::tu::aobj::HSD_AObjReqAnim(a0, a1) };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjGetCurrent();
    fn HSD_CObjGetInvViewingMtxPtrDirect(a0: *mut HSD_CObj) -> *mut c_void;
    fn HSD_Free(a0: *mut c_void) = { let (a0,) = (a0,); crate::tu::memory::HSD_Free(a0) };
    fn HSD_LObjGetCurrentByType(a0: u16) -> *mut HSD_LObj;
    fn HSD_LObjGetLightMaskDiffuse() -> u32;
    fn HSD_LObjGetLightVector(a0: *mut HSD_LObj, a1: *mut Vec);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = { let (a0,) = (a0,); crate::tu::memory::HSD_MemAlloc(a0) };
    fn HSD_MkRotationMtx(a0: *mut c_void, a1: *mut Vec);
    fn HSD_SetupTevStage(a0: *mut _HSD_TevDesc);
    fn HSD_StateAssignTev() -> i32;
    fn HSD_StateRegisterTexGen(a0: i32);
    fn HSD_StateSetZMode(a0: i32, a1: i32, a2: i32);
    fn HSD_TExpAlphaIn(a0: *mut HSD_TExp, a1: u32, a2: *mut HSD_TExp, a3: u32, a4: *mut HSD_TExp, a5: u32, a6: *mut HSD_TExp, a7: u32, a8: *mut HSD_TExp);
    fn HSD_TExpAlphaOp(a0: *mut HSD_TExp, a1: u32, a2: u32, a3: u32, a4: u8);
    fn HSD_TExpCnst(a0: *mut c_void, a1: u32, a2: u32, a3: *mut *mut HSD_TExp) -> *mut HSD_TExp;
    fn HSD_TExpColorIn(a0: *mut HSD_TExp, a1: u32, a2: *mut HSD_TExp, a3: u32, a4: *mut HSD_TExp, a5: u32, a6: *mut HSD_TExp, a7: u32, a8: *mut HSD_TExp);
    fn HSD_TExpColorOp(a0: *mut HSD_TExp, a1: u32, a2: u32, a3: u32, a4: u8);
    fn HSD_TExpOrder(a0: *mut HSD_TExp, a1: *mut HSD_TObj, a2: u32);
    fn HSD_TExpTev(a0: *mut *mut HSD_TExp) -> *mut HSD_TExp = { let (a0,) = (a0,); crate::tu::texp::HSD_TExpTev(a0) };
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _);
    fn PSMTXMultVecSR(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec) = crate::tu::mtx_pc::raw::PSMTXMultVecSR(a0 as _, a1, a2);
    fn PSMTXScale(a0: *mut c_void, a1: f32, a2: f32, a3: f32) = crate::tu::mtx_pc::raw::PSMTXScale(a0 as _, a1, a2, a3);
    fn PSMTXTrans(a0: *mut c_void, a1: f32, a2: f32, a3: f32) = crate::tu::mtx_pc::raw::PSMTXTrans(a0 as _, a1, a2, a3);
    fn PSVECNormalize(a0: *mut Vec, a1: *mut Vec) = crate::tu::mtx_pc::raw::PSVECNormalize(a0, a1);
    fn hsdAllocMemPiece(a0: i32) -> *mut c_void = { let (a0,) = (a0,); crate::tu::class::hsdAllocMemPiece(a0) };
    fn hsdFreeMemPiece(a0: *mut c_void, a1: i32) = { let (a0, a1) = (a0, a1); crate::tu::class::hsdFreeMemPiece(a0, a1) };
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32) = { let (a0, a1, a2, a3, a4, a5) = (a0, a1, a2, a3, a4, a5); crate::tu::class::hsdInitClassInfo(a0 as _, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 = { let (a0, a1) = (a0, a1); crate::tu::class::hsdIsDescendantOf(a0, a1) as _ };
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void = { let (a0,) = (a0,); crate::tu::class::hsdNew(a0 as _) };
    fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo = { let (a0,) = (a0,); crate::tu::class::hsdSearchClassInfo(a0 as _) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 { super::hsdIsDescendantOf(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdObj: HSD_ClassInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

}
pub use native::*;

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    #[allow(unused_imports)]
    use crate::tobj::*;
    use super::*;
    /// `HSD_Index2TexMtx`: the C entry point over `crate::tobj::index_to_tex_mtx` (the owned object model).
    #[inline]
    pub fn HSD_Index2TexMtx(index: u32) -> u32 {
        crate::tobj::index_to_tex_mtx(index)
    }
}
pub use cnames::*;

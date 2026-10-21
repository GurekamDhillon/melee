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
    let toon = list.iter().position(|o| o.coord_kind() == TEX_COORD_TOON);
    let bump = list.iter().position(|o| o.coord_kind() != TEX_COORD_TOON && o.bump());
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
                    let Some(tl) = o.tlut else { continue };
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

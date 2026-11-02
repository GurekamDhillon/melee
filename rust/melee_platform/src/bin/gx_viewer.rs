//! Debug viewer: a stage model from the user's disc, drawn through the GX shim (melee_gx's
//! `GxShim` -> FIFO -> decode -> wgpu) into the EFB and presented in a winit window.
//!
//!   cargo run -p melee_platform --bin gx_viewer -- [--iso DISC.iso] [--stage GrNLa.dat]
//!       [--frames N] [--shot-frame K] [--shot PATH] [--synthetic] [--fit]
//!
//! The disc defaults to $GW_ISO_VANILLA. Nothing read from it is written anywhere except the
//! screenshot, which defaults to rust/target/gx-viewer/<stage>.png (disc art: never commit it).
//! `--synthetic` draws a built-in procedural stage instead (no disc; generated geometry and
//! texture), whose screenshot defaults to docs/rust-port/shots/gx_viewer_synthetic.png.
//!
//! This is a viewer, not the game's renderer: it walks the stage's `map_head` model groups
//! (HSD_Joint -> DObj -> MObj/TObj/PObj, the raw descriptors in the DAT file) with a bind-pose
//! SRT matrix per joint, and sets up a simple material per PObj (first texture modulated by the
//! vertex/material colour, one light). The real draw path (HSD JObj/DObj/MObj/TObj/PObj disp) is
//! group 1's to port; it will call the same `GxShim`.
use melee_gx::api::{GxColor, GxLightObj};
use melee_gx::consts::*;
use melee_gx::shim::GxShim;
use melee_gx::state::{TexObj, TlutObj};
use melee_gx::GxData;
use melee_platform::gfx::{write_png, Presenter, EFB_HEIGHT, EFB_WIDTH};
use melee_platform::{run_windowed, Game, PadSystem, WindowConfig};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use winit::window::Window;

type Mtx = [[f32; 4]; 3];

// ------------------------------------------------------------------ math (viewer only)

fn mtx_identity() -> Mtx {
    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]]
}
fn mtx_concat(a: &Mtx, b: &Mtx) -> Mtx {
    let mut o = [[0f32; 4]; 3];
    for r in 0..3 {
        for c in 0..4 {
            o[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c] + if c == 3 { a[r][3] } else { 0.0 };
        }
    }
    o
}
/// HSD's joint matrix: T * Rz * Ry * Rx * S.
fn mtx_srt(s: [f32; 3], r: [f32; 3], t: [f32; 3]) -> Mtx {
    let (sx, cx) = r[0].sin_cos();
    let (sy, cy) = r[1].sin_cos();
    let (sz, cz) = r[2].sin_cos();
    let rot = [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
        [-sy, sx * cy, cx * cy],
    ];
    let mut m = [[0f32; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = rot[i][j] * s[j];
        }
        m[i][3] = t[i];
    }
    m
}
fn xform(m: &Mtx, p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|r| m[r][0] * p[0] + m[r][1] * p[1] + m[r][2] * p[2] + m[r][3])
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(1e-12);
    [a[0] / l, a[1] / l, a[2] / l]
}
/// MTXLookAt.
fn look_at(eye: [f32; 3], up: [f32; 3], at: [f32; 3]) -> Mtx {
    let look = norm(sub(eye, at));
    let right = norm(cross(up, look));
    let up2 = cross(look, right);
    [
        [right[0], right[1], right[2], -dot(right, eye)],
        [up2[0], up2[1], up2[2], -dot(up2, eye)],
        [look[0], look[1], look[2], -dot(look, eye)],
    ]
}
/// MTXPerspective (fovy degrees).
fn perspective(fovy: f32, aspect: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    let cot = 1.0 / (fovy.to_radians() * 0.5).tan();
    [
        [cot / aspect, 0.0, 0.0, 0.0],
        [0.0, cot, 0.0, 0.0],
        [0.0, 0.0, -n / (f - n), -(f * n) / (f - n)],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

// ------------------------------------------------------------------ model (viewer IR)

#[derive(Clone)]
struct VtxAttr {
    attr: u32,
    ty: u32,
    cnt: u32,
    comp: u32,
    frac: u8,
    stride: u8,
    array: Option<GxData>,
}

struct Tex {
    obj: TexObj,
    tlut: Option<TlutObj>,
    src: u32,
    scale: [f32; 2],
}

struct Mesh {
    /// World matrix (rigid) or the model root's (envelope: vertices are model-space).
    world: Mtx,
    attrs: Vec<VtxAttr>,
    dl: Vec<u8>,
    cull: u32,
    xlu: bool,
    ambient: GxColor,
    diffuse: GxColor,
    tex: Option<usize>,
}

struct Model {
    meshes: Vec<Mesh>,
    textures: Vec<Tex>,
    bmin: [f32; 3],
    bmax: [f32; 3],
    skipped: Vec<String>,
}

/// Bytes of one vertex attribute in a display list.
fn attr_size(a: &VtxAttr) -> usize {
    match a.ty {
        GX_INDEX8 => 1,
        GX_INDEX16 => 2,
        GX_DIRECT => {
            if a.attr <= GX_VA_TEX7MTXIDX {
                return 1;
            }
            let comp = match a.comp {
                0 | 1 => 1,
                2 | 3 => 2,
                _ => 4,
            };
            match a.attr {
                GX_VA_POS => comp * if a.cnt == 0 { 2 } else { 3 },
                GX_VA_NRM | GX_VA_NBT => comp * if a.cnt == 0 { 3 } else { 9 },
                0xB | 0xC => match a.comp {
                    GX_RGB565 | GX_RGBA4 => 2,
                    GX_RGB8 | GX_RGBA6 => 3,
                    _ => 4,
                },
                _ => comp * if a.cnt == 0 { 1 } else { 2 },
            }
        }
        _ => 0,
    }
}

/// Walk a display list: (max index per attribute, vertex count). None on a malformed list.
fn scan_dl(dl: &[u8], attrs: &[VtxAttr]) -> Option<(HashMap<u32, u32>, usize)> {
    let vsize: usize = attrs.iter().map(attr_size).sum();
    let mut max = HashMap::new();
    let mut i = 0;
    let mut verts = 0;
    while i < dl.len() {
        let op = dl[i];
        if op == 0 {
            i += 1;
            continue;
        }
        if op & 0x80 == 0 || i + 3 > dl.len() {
            return None;
        }
        let n = u16::from_be_bytes([dl[i + 1], dl[i + 2]]) as usize;
        i += 3;
        for _ in 0..n {
            for a in attrs {
                let s = attr_size(a);
                if i + s > dl.len() {
                    return None;
                }
                let idx = match a.ty {
                    GX_INDEX8 => Some(dl[i] as u32),
                    GX_INDEX16 => Some(u16::from_be_bytes([dl[i], dl[i + 1]]) as u32),
                    _ => None,
                };
                if let Some(x) = idx {
                    let e = max.entry(a.attr).or_insert(0);
                    *e = (*e).max(x);
                }
                i += s;
            }
            verts += 1;
        }
        let _ = vsize;
    }
    Some((max, verts))
}

/// Decode position `idx` of a POS array (big-endian, as the GX reads it).
fn read_pos(a: &VtxAttr, data: &[u8], idx: u32) -> Option<[f32; 3]> {
    let n = if a.cnt == 0 { 2 } else { 3 };
    let cs = match a.comp {
        0 | 1 => 1,
        2 | 3 => 2,
        _ => 4,
    };
    let base = idx as usize * a.stride as usize;
    let mut out = [0f32; 3];
    let scale = 1.0 / (1u32 << a.frac) as f32;
    for (k, o) in out.iter_mut().enumerate().take(n) {
        let p = base + k * cs;
        let b = data.get(p..p + cs)?;
        *o = match a.comp {
            0 => b[0] as f32 * scale,
            1 => b[0] as i8 as f32 * scale,
            2 => u16::from_be_bytes([b[0], b[1]]) as f32 * scale,
            3 => i16::from_be_bytes([b[0], b[1]]) as f32 * scale,
            _ => f32::from_be_bytes([b[0], b[1], b[2], b[3]]),
        };
    }
    Some(out)
}

// ------------------------------------------------------------------ DAT walker

struct Dat {
    data: Vec<u8>,
    relocs: HashSet<u32>,
    publics: Vec<(u32, String)>,
    /// Shared copies of array/texture data by (offset, len), so each GxData is one Arc.
    slices: HashMap<(u32, u32), GxData>,
}

impl Dat {
    fn parse(file: &[u8]) -> Result<Self, String> {
        let r32 = |o: usize| -> Result<u32, String> {
            file.get(o..o + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap())).ok_or("truncated DAT".to_string())
        };
        let data_size = r32(4)? as usize;
        let nrel = r32(8)? as usize;
        let npub = r32(12)? as usize;
        let next = r32(16)? as usize;
        let data = file.get(0x20..0x20 + data_size).ok_or("truncated data")?.to_vec();
        let rel_at = 0x20 + data_size;
        let mut relocs = HashSet::new();
        for i in 0..nrel {
            relocs.insert(r32(rel_at + 4 * i)?);
        }
        let pub_at = rel_at + 4 * nrel;
        let str_at = pub_at + 8 * (npub + next);
        let mut publics = Vec::new();
        for i in 0..npub {
            let off = r32(pub_at + 8 * i)?;
            let so = str_at + r32(pub_at + 8 * i + 4)? as usize;
            let end = file[so..].iter().position(|&c| c == 0).unwrap_or(0);
            publics.push((off, String::from_utf8_lossy(&file[so..so + end]).into_owned()));
        }
        Ok(Self { data, relocs, publics, slices: HashMap::new() })
    }
    fn u8_(&self, o: u32) -> u8 {
        self.data.get(o as usize).copied().unwrap_or(0)
    }
    fn u16_(&self, o: u32) -> u16 {
        self.data.get(o as usize..o as usize + 2).map_or(0, |b| u16::from_be_bytes([b[0], b[1]]))
    }
    fn u32_(&self, o: u32) -> u32 {
        self.data.get(o as usize..o as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
    }
    fn f32_(&self, o: u32) -> f32 {
        f32::from_bits(self.u32_(o))
    }
    fn vec3(&self, o: u32) -> [f32; 3] {
        [self.f32_(o), self.f32_(o + 4), self.f32_(o + 8)]
    }
    /// A relocated pointer at `o`, or None (null / not a pointer).
    fn ptr(&self, o: u32) -> Option<u32> {
        if self.relocs.contains(&o) {
            Some(self.u32_(o))
        } else {
            None
        }
    }
    fn slice(&mut self, off: u32, len: u32) -> GxData {
        let end = (off as usize + len as usize).min(self.data.len());
        let off_c = (off as usize).min(end);
        let key = (off_c as u32, (end - off_c) as u32);
        let d = &self.data;
        self.slices.entry(key).or_insert_with(|| Arc::from(&d[off_c..end])).clone()
    }
    fn color(&self, o: u32) -> GxColor {
        GxColor::new(self.u8_(o), self.u8_(o + 1), self.u8_(o + 2), self.u8_(o + 3))
    }
}

struct Loader<'a> {
    dat: Dat,
    shim: &'a mut GxShim,
    model: Model,
    tex_by_desc: HashMap<u32, usize>,
    joint_world: HashMap<u32, Mtx>,
    visited: HashSet<u32>,
}

impl Loader<'_> {
    fn joint_tree(&mut self, j: u32, parent: &Mtx, depth: u32) {
        let mut cur = Some(j);
        while let Some(j) = cur {
            if depth > 64 || !self.visited.insert(j) {
                return;
            }
            let world = mtx_concat(parent, &mtx_srt(self.dat.vec3(j + 0x20), self.dat.vec3(j + 0x14), self.dat.vec3(j + 0x2C)));
            self.joint_world.insert(j, world);
            if let Some(c) = self.dat.ptr(j + 8) {
                self.joint_tree(c, &world, depth + 1);
            }
            cur = self.dat.ptr(j + 0xC);
        }
    }

    fn joint_dobjs(&mut self, root: u32, root_world: &Mtx) {
        let joints: Vec<u32> = {
            let mut v: Vec<u32> = self.visited.iter().copied().collect();
            v.sort();
            v
        };
        for j in joints {
            let flags = self.dat.u32_(j + 4);
            // JOBJ_HIDDEN, JOBJ_PTCL, JOBJ_SPLINE: no DObjs to draw.
            if flags & ((1 << 4) | (1 << 5) | (1 << 14)) != 0 {
                continue;
            }
            let world = self.joint_world[&j];
            let mut d = self.dat.ptr(j + 0x10);
            let mut guard = 0;
            while let Some(dobj) = d {
                guard += 1;
                if guard > 4096 {
                    break;
                }
                let mobj = self.dat.ptr(dobj + 8);
                let mut p = self.dat.ptr(dobj + 0xC);
                while let Some(pobj) = p {
                    self.pobj(pobj, mobj, &world, root_world);
                    p = self.dat.ptr(pobj + 4);
                }
                d = self.dat.ptr(dobj + 4);
            }
        }
        let _ = root;
    }

    fn texture(&mut self, tobj: u32) -> Option<usize> {
        if let Some(&i) = self.tex_by_desc.get(&tobj) {
            return Some(i);
        }
        let img = self.dat.ptr(tobj + 0x4C)?;
        let image = self.dat.ptr(img)?;
        let (w, h, fmt) = (self.dat.u16_(img + 4), self.dat.u16_(img + 6), self.dat.u32_(img + 8));
        let mip = self.dat.u32_(img + 0xC) != 0;
        let max_lod = self.dat.f32_(img + 0x14);
        let size = melee_gx::shim::get_tex_buffer_size(w, h, fmt, mip, if mip { max_lod.max(1.0) as u8 } else { 0 });
        let data = self.dat.slice(image, size.max(32));
        let ws = self.dat.u32_(tobj + 0x34).min(2);
        let wt = self.dat.u32_(tobj + 0x38).min(2);
        let mut obj = TexObj::default();
        let mut tlut = None;
        if matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2) {
            let td = self.dat.ptr(tobj + 0x50)?;
            let lut = self.dat.ptr(td)?;
            let (tfmt, n) = (self.dat.u32_(td + 4), self.dat.u16_(td + 0xC));
            let ldata = self.dat.slice(lut, n as u32 * 2);
            let mut t = TlutObj::default();
            self.shim.init_tlut_obj(&mut t, Some(ldata), tfmt, n);
            tlut = Some(t);
            self.shim.init_tex_obj_ci(&mut obj, Some(data), w, h, fmt, ws, wt, mip, GX_TLUT0);
        } else {
            self.shim.init_tex_obj(&mut obj, Some(data), w, h, fmt, ws, wt, mip);
        }
        let mag = self.dat.u32_(tobj + 0x48).min(1);
        GxShim::init_tex_obj_lod(&mut obj, if mip { GX_LIN_MIP_LIN } else { GX_LINEAR }, mag, 0.0, if mip { max_lod } else { 0.0 }, 0.0, false, false, GX_ANISO_1);
        let sc = self.dat.vec3(tobj + 0x1C);
        let rep = [self.dat.u8_(tobj + 0x3C) as f32, self.dat.u8_(tobj + 0x3D) as f32];
        let scale = [0, 1].map(|k| {
            let r = if rep[k] == 0.0 { 1.0 } else { rep[k] };
            if sc[k].abs() < f32::EPSILON { r } else { r / sc[k] }
        });
        let src = self.dat.u32_(tobj + 0xC);
        self.model.textures.push(Tex { obj, tlut, src: if (GX_TG_TEX0..=GX_TG_TEX0 + 7).contains(&src) { src } else { GX_TG_TEX0 }, scale });
        let i = self.model.textures.len() - 1;
        self.tex_by_desc.insert(tobj, i);
        Some(i)
    }

    fn pobj(&mut self, pobj: u32, mobj: Option<u32>, world: &Mtx, root_world: &Mtx) {
        let flags = self.dat.u16_(pobj + 0xC) as u32;
        let kind = flags & 0x3000;
        if kind == 0x1000 {
            self.model.skipped.push(format!("pobj @{pobj:#x}: shape-anim"));
            return;
        }
        let (Some(verts), Some(disp)) = (self.dat.ptr(pobj + 8), self.dat.ptr(pobj + 0x10)) else { return };
        let n_display = self.dat.u16_(pobj + 0xE) as u32;
        let mut attrs = Vec::new();
        let mut v = verts;
        loop {
            let attr = self.dat.u32_(v);
            if attr == GX_VA_NULL || attrs.len() > 26 {
                break;
            }
            attrs.push((v, VtxAttr {
                attr,
                ty: self.dat.u32_(v + 4),
                cnt: self.dat.u32_(v + 8),
                comp: self.dat.u32_(v + 0xC),
                frac: self.dat.u8_(v + 0x10),
                stride: self.dat.u16_(v + 0x12) as u8,
                array: None,
            }));
            v += 0x18;
        }
        let dl = self.dat.slice(disp, n_display * 32).to_vec();
        let plain: Vec<VtxAttr> = attrs.iter().map(|(_, a)| a.clone()).collect();
        let Some((max_idx, _verts)) = scan_dl(&dl, &plain) else {
            self.model.skipped.push(format!("pobj @{pobj:#x}: malformed display list"));
            return;
        };
        // Arrays: exactly as long as the display list's largest index needs.
        let mut out = Vec::new();
        for (vd, mut a) in attrs {
            if a.ty == GX_INDEX8 || a.ty == GX_INDEX16 {
                if let Some(base) = self.dat.ptr(vd + 0x14) {
                    let n = max_idx.get(&a.attr).copied().unwrap_or(0) + 1;
                    a.array = Some(self.dat.slice(base, n * a.stride.max(1) as u32 + 16));
                }
            }
            out.push(a);
        }
        let m = if kind == 0x2000 {
            *root_world
        } else if let Some(j) = self.dat.ptr(pobj + 0x14) {
            self.joint_world.get(&j).copied().unwrap_or(*world)
        } else {
            *world
        };
        // Bounds from the positions this display list references.
        if let Some(pa) = out.iter().find(|a| a.attr == GX_VA_POS && a.array.is_some()) {
            let data = pa.array.clone().unwrap();
            let n = max_idx.get(&GX_VA_POS).copied().unwrap_or(0);
            for i in 0..=n {
                if let Some(p) = read_pos(pa, &data, i) {
                    let w = xform(&m, p);
                    if w.iter().all(|c| c.is_finite() && c.abs() < 1e5) {
                        for k in 0..3 {
                            self.model.bmin[k] = self.model.bmin[k].min(w[k]);
                            self.model.bmax[k] = self.model.bmax[k].max(w[k]);
                        }
                    }
                }
            }
        }
        let (mut ambient, mut diffuse, mut xlu, mut tex) = (GxColor::new(128, 128, 128, 255), GxColor::new(255, 255, 255, 255), false, None);
        if let Some(mo) = mobj {
            let rm = self.dat.u32_(mo + 4);
            xlu = rm & (1 << 30) != 0;
            if let Some(mat) = self.dat.ptr(mo + 0xC) {
                ambient = self.dat.color(mat);
                diffuse = self.dat.color(mat + 4);
                let a = self.dat.f32_(mat + 0xC);
                diffuse.a = (a.clamp(0.0, 1.0) * 255.0) as u8;
            }
            if rm & (0xFF << 4) != 0 {
                if let Some(t) = self.dat.ptr(mo + 8) {
                    tex = self.texture(t);
                }
            }
        }
        let cull = match flags & 0xC000 {
            0x4000 => GX_CULL_FRONT,
            0x8000 => GX_CULL_BACK,
            0xC000 => GX_CULL_ALL,
            _ => GX_CULL_NONE,
        };
        self.model.meshes.push(Mesh { world: m, attrs: out, dl, cull, xlu, ambient, diffuse, tex });
    }
}

fn load_stage(file: &[u8], shim: &mut GxShim) -> Result<Model, String> {
    let dat = Dat::parse(file)?;
    let head = dat.publics.iter().find(|(_, n)| n == "map_head").map(|p| p.0).ok_or("no map_head symbol")?;
    let groups = dat.ptr(head + 8).ok_or("map_head has no model groups")?;
    let count = dat.u32_(head + 0xC);
    let mut l = Loader {
        dat,
        shim,
        model: Model { meshes: Vec::new(), textures: Vec::new(), bmin: [f32::MAX; 3], bmax: [f32::MIN; 3], skipped: Vec::new() },
        tex_by_desc: HashMap::new(),
        joint_world: HashMap::new(),
        visited: HashSet::new(),
    };
    for g in 0..count.min(64) {
        // UnkStageDat_x8_t (gr/types.h), 0x34 bytes: +0 HSD_Joint* root.
        let Some(root) = l.dat.ptr(groups + g * 0x34) else { continue };
        l.visited.clear();
        let id = mtx_identity();
        l.joint_tree(root, &id, 0);
        l.joint_dobjs(root, &id);
    }
    Ok(l.model)
}

// ------------------------------------------------------------------ synthetic stage (no disc)

/// RGB565 texture in GC 4x4 tiles.
fn encode_rgb565(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..h).step_by(4) {
        for tx in (0..w).step_by(4) {
            for y in ty..ty + 4 {
                for x in tx..tx + 4 {
                    let [r, g, b] = f(x, y);
                    let v = ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3);
                    out.extend_from_slice(&v.to_be_bytes());
                }
            }
        }
    }
    out
}

/// Battlefield-like layout: a main platform (a box with a tapered underside) and three floating
/// platforms, all generated here. Positions/normals F32, texcoords F32, INDEX16 into arrays,
/// colours DIRECT RGBA8; drawn as quads from display lists.
fn synthetic_stage(shim: &mut GxShim) -> Model {
    let mut model = Model { meshes: Vec::new(), textures: Vec::new(), bmin: [f32::MAX; 3], bmax: [f32::MIN; 3], skipped: Vec::new() };
    let tex_data: GxData = Arc::from(encode_rgb565(64, 64, |x, y| {
        let c = ((x / 8) + (y / 8)) % 2 == 0;
        let edge = x % 8 == 0 || y % 8 == 0;
        if edge { [40, 60, 90] } else if c { [190, 200, 215] } else { [150, 165, 190] }
    }));
    let mut obj = TexObj::default();
    shim.init_tex_obj(&mut obj, Some(tex_data), 64, 64, GX_TF_RGB565, GX_REPEAT, GX_REPEAT, false);
    model.textures.push(Tex { obj, tlut: None, src: GX_TG_TEX0, scale: [1.0, 1.0] });

    // (x0, x1, y_top, thickness, taper) per platform.
    let plats: [(f32, f32, f32, f32, f32); 4] =
        [(-68.0, 68.0, 0.0, 14.0, 30.0), (-57.0, -20.0, 27.0, 2.0, 0.0), (20.0, 57.0, 27.0, 2.0, 0.0), (-18.5, 18.5, 54.0, 2.0, 0.0)];
    for (pi, &(x0, x1, yt, th, taper)) in plats.iter().enumerate() {
        let (z0, z1) = (-12.0f32, 12.0f32);
        let yb = yt - th;
        let (bx0, bx1) = (x0 + taper * 0.5, x1 - taper * 0.5);
        // 8 corners: top face then bottom face (bottom pulled in by the taper).
        let c = [
            [x0, yt, z0], [x1, yt, z0], [x1, yt, z1], [x0, yt, z1],
            [bx0, yb, z0], [bx1, yb, z0], [bx1, yb, z1], [bx0, yb, z1],
        ];
        // Faces (CW from outside, as GX front faces are), each with its own 4 vertices.
        let faces: [[usize; 4]; 6] = [[0, 1, 2, 3], [7, 6, 5, 4], [4, 5, 1, 0], [6, 7, 3, 2], [7, 4, 0, 3], [5, 6, 2, 1]];
        let mut pos = Vec::new();
        let mut nrm = Vec::new();
        let mut uv = Vec::new();
        let mut dl = Vec::new();
        dl.push((GX_QUADS | GX_VTXFMT0) as u8);
        dl.extend_from_slice(&((faces.len() * 4) as u16).to_be_bytes());
        for (fi, f) in faces.iter().enumerate() {
            let p: Vec<[f32; 3]> = f.iter().map(|&i| c[i]).collect();
            let n = norm(cross(sub(p[2], p[0]), sub(p[1], p[0])));
            let shade = if fi == 0 { [235u8, 240, 255, 255] } else if fi == 1 { [70, 80, 110, 255] } else { [150, 160, 190, 255] };
            for (k, v) in p.iter().enumerate() {
                let idx = (pos.len() / 12) as u16;
                for comp in v {
                    pos.extend_from_slice(&comp.to_be_bytes());
                }
                for comp in n {
                    nrm.extend_from_slice(&comp.to_be_bytes());
                }
                let (u, w) = if n[1].abs() > 0.5 { (v[0] / 24.0, v[2] / 24.0) } else { ((v[0] + v[2]) / 24.0, -v[1] / 24.0) };
                uv.extend_from_slice(&u.to_be_bytes());
                uv.extend_from_slice(&w.to_be_bytes());
                // GX attribute order: POS, NRM, CLR0, TEX0.
                dl.extend_from_slice(&idx.to_be_bytes());
                dl.extend_from_slice(&idx.to_be_bytes());
                dl.extend_from_slice(&shade);
                dl.extend_from_slice(&idx.to_be_bytes());
                let _ = k;
                for k3 in 0..3 {
                    model.bmin[k3] = model.bmin[k3].min(v[k3]);
                    model.bmax[k3] = model.bmax[k3].max(v[k3]);
                }
            }
        }
        dl.resize(dl.len().div_ceil(32) * 32, 0);
        let arr = |d: Vec<u8>| Some(GxData::from(d));
        let a = |attr, ty, cnt, comp, stride, array| VtxAttr { attr, ty, cnt, comp, frac: 0, stride, array };
        let attrs = vec![
            a(GX_VA_POS, GX_INDEX16, GX_POS_XYZ, GX_F32, 12, arr(pos)),
            a(GX_VA_NRM, GX_INDEX16, GX_NRM_XYZ, GX_F32, 12, arr(nrm)),
            a(GX_VA_TEX0, GX_INDEX16, GX_TEX_ST, GX_F32, 8, arr(uv)),
            a(GX_VA_CLR0, GX_DIRECT, GX_CLR_RGBA, GX_RGBA8, 0, None),
        ];
        model.meshes.push(Mesh {
            world: mtx_identity(),
            attrs,
            dl,
            cull: GX_CULL_BACK,
            xlu: false,
            ambient: GxColor::new(90, 90, 110, 255),
            diffuse: GxColor::new(255, 255, 255, 255),
            tex: if pi == 0 || pi == 3 { Some(0) } else { Some(0) },
        });
    }
    model
}

// ------------------------------------------------------------------ drawing

fn draw_model(shim: &mut GxShim, model: &mut Model, view: &Mtx, proj: &[[f32; 4]; 4], clear: GxColor) {
    shim.set_copy_clear(clear, 0xFF_FFFF);
    shim.set_viewport(0.0, 0.0, EFB_WIDTH as f32, EFB_HEIGHT as f32, 0.0, 1.0);
    shim.set_scissor(0, 0, EFB_WIDTH, EFB_HEIGHT);
    shim.set_projection(proj, GX_PERSPECTIVE);
    // One light, up and toward the camera, in view space.
    let mut light = GxLightObj::default();
    let lp = xform(view, [3000.0, 6000.0, 8000.0]);
    GxShim::init_light_pos(&mut light, lp[0], lp[1], lp[2]);
    GxShim::init_light_color(&mut light, GxColor::new(255, 255, 255, 255));
    GxShim::init_light_attn(&mut light, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    shim.load_light_obj_imm(&light, GX_LIGHT0);
    for pass_xlu in [false, true] {
        for i in 0..model.meshes.len() {
            if model.meshes[i].xlu != pass_xlu {
                continue;
            }
            let mv = mtx_concat(view, &model.meshes[i].world);
            let mesh = &model.meshes[i];
            for id in (GX_PNMTX0..=GX_PNMTX9).step_by(3) {
                shim.load_pos_mtx_imm(&mv, id);
                shim.load_nrm_mtx_imm(&mv, id);
            }
            shim.set_current_mtx(GX_PNMTX0);
            shim.clear_vtx_desc();
            for a in &mesh.attrs {
                shim.set_vtx_desc(a.attr, a.ty);
                if a.attr > GX_VA_TEX7MTXIDX {
                    shim.set_vtx_attr_fmt(GX_VTXFMT0, a.attr, a.cnt, a.comp, a.frac);
                }
                if let Some(arr) = &a.array {
                    shim.set_array(a.attr, Some(arr), a.stride);
                }
            }
            let has_clr = mesh.attrs.iter().any(|a| a.attr == GX_VA_CLR0);
            let has_nrm = mesh.attrs.iter().any(|a| a.attr == GX_VA_NRM || a.attr == GX_VA_NBT);
            shim.set_num_chans(1);
            shim.set_chan_amb_color(GX_COLOR0A0, mesh.ambient);
            shim.set_chan_mat_color(GX_COLOR0A0, mesh.diffuse);
            shim.set_chan_ctrl(
                GX_COLOR0A0,
                has_nrm,
                GX_SRC_REG,
                if has_clr { GX_SRC_VTX } else { GX_SRC_REG },
                if has_nrm { GX_LIGHT0 } else { GX_LIGHT_NULL },
                if has_nrm { GX_DF_CLAMP } else { GX_DF_NONE },
                GX_AF_NONE,
            );
            shim.set_num_ind_stages(0);
            shim.set_tev_direct(GX_TEVSTAGE0);
            shim.set_num_tev_stages(1);
            let tex = mesh.tex.and_then(|t| model.textures.get_mut(t));
            if let Some(t) = tex {
                shim.set_num_tex_gens(1);
                let tm: Mtx = [[t.scale[0], 0.0, 0.0, 0.0], [0.0, t.scale[1], 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
                shim.load_tex_mtx_imm(&tm, GX_TEXMTX0, GX_MTX2x4);
                shim.set_tex_coord_gen2(GX_TEXCOORD0, GX_TG_MTX2x4, t.src, GX_TEXMTX0, false, GX_PTIDENTITY);
                if let Some(tl) = &t.tlut {
                    shim.load_tlut(tl, GX_TLUT0);
                }
                shim.load_tex_obj(&mut t.obj, GX_TEXMAP0);
                shim.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
                shim.set_tev_op(GX_TEVSTAGE0, GX_MODULATE);
            } else {
                shim.set_num_tex_gens(0);
                shim.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
                shim.set_tev_op(GX_TEVSTAGE0, GX_PASSCLR);
            }
            if mesh.xlu {
                shim.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
                shim.set_alpha_compare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
                shim.set_z_mode(true, GX_LEQUAL, false);
            } else {
                shim.set_blend_mode(GX_BM_NONE, GX_BL_ONE, GX_BL_ZERO, GX_LO_CLEAR);
                shim.set_alpha_compare(GX_GEQUAL, 128, GX_AOP_AND, GX_ALWAYS, 0);
                shim.set_z_mode(true, GX_LEQUAL, true);
            }
            shim.set_z_comp_loc(false);
            shim.set_cull_mode(mesh.cull);
            shim.call_display_list(&mesh.dl);
        }
    }
    shim.copy_disp(true);
}

// ------------------------------------------------------------------ app

struct Viewer {
    shim: GxShim,
    model: Model,
    presenter: Option<Presenter>,
    frame: u64,
    frames: Option<u64>,
    shot_frame: u64,
    shot: PathBuf,
    shot_done: bool,
    quit: bool,
    failed: Option<String>,
    /// Frame the model's bounds (the synthetic stage, --fit) instead of the game-like camera.
    fit: bool,
}

impl Viewer {
    fn camera(&self) -> (Mtx, [[f32; 4]; 4]) {
        let (bmin, bmax) = (self.model.bmin, self.model.bmax);
        if !self.fit {
            // A game-like camera on the stage's centre (stages span roughly x in [-150, 150]):
            // backgrounds reach tens of thousands of units, so the bounds would frame those.
            let fov = 30.0f32;
            let t = (self.frame as f32 / 240.0).sin() * 0.25;
            let at = [0.0, 15.0, 0.0];
            let d = 300.0;
            let eye = [d * t.sin(), 45.0, d * t.cos()];
            let view = look_at(eye, [0.0, 1.0, 0.0], at);
            return (view, perspective(fov, EFB_WIDTH as f32 / EFB_HEIGHT as f32, 2.0, 80000.0));
        }
        let ok = bmin.iter().zip(&bmax).all(|(a, b)| a <= b);
        let (c, r) = if ok {
            let c = [0, 1, 2].map(|k| (bmin[k] + bmax[k]) * 0.5);
            let e = sub(bmax, bmin);
            (c, (dot(e, e).sqrt() * 0.5).max(1.0))
        } else {
            ([0.0; 3], 100.0)
        };
        let fov = 30.0f32;
        let d = r / (fov.to_radians() * 0.5).tan() * 0.9;
        let t = (self.frame as f32 / 240.0).sin() * 0.35;
        let eye = [c[0] + d * t.sin(), c[1] + d * 0.22, c[2] + d * t.cos()];
        let view = look_at(eye, [0.0, 1.0, 0.0], c);
        (view, perspective(fov, EFB_WIDTH as f32 / EFB_HEIGHT as f32, (d * 0.02).max(0.5), d * 4.0 + r * 2.0))
    }
}

impl Game for Viewer {
    fn tick(&mut self, _pads: &mut PadSystem) {}
    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => {
                    eprintln!(
                        "gx_viewer: adapter {} ({:?}), swapchain {:?}",
                        p.renderer.adapter_info.name,
                        p.renderer.adapter_info.backend,
                        p.format()
                    );
                    self.presenter = Some(p);
                }
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        let (view, proj) = self.camera();
        draw_model(&mut self.shim, &mut self.model, &view, &proj, GxColor::new(24, 28, 44, 255));
        let frame = self.shim.take_frame();
        let p = self.presenter.as_mut().unwrap();
        p.present(&self.shim.gx.state, frame.as_deref());
        for l in self.shim.log.drain(..) {
            eprintln!("{l}");
        }
        self.frame += 1;
        if self.frame == 1 {
            let n = frame.as_ref().map_or(0, |f| f.len());
            eprintln!("gx_viewer: first frame {n} draws, warnings {:?}", self.shim.gx.state.warnings.iter().take(4).collect::<Vec<_>>());
            if !p.renderer.shader_errors.is_empty() {
                eprintln!("gx_viewer: {} shader errors, first:\n{}", p.renderer.shader_errors.len(), p.renderer.shader_errors[0]);
            }
        }
        if !self.shot_done && self.frame >= self.shot_frame {
            self.shot_done = true;
            let px = p.screenshot();
            match write_png(&self.shot, p.renderer.width, p.renderer.height, &px) {
                Ok(()) => eprintln!("gx_viewer: frame {} saved to {}", self.frame, self.shot.display()),
                Err(e) => eprintln!("gx_viewer: screenshot failed: {e}"),
            }
        }
        if self.frames.is_some_and(|n| self.frame >= n) {
            self.quit = true;
        }
    }
    fn resized(&mut self, w: u32, h: u32) {
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn read_disc_file(iso: &str, want: &str) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(iso).map_err(|e| format!("{iso}: {e}"))?;
    let mut rd = |off: u64, n: usize| -> Result<Vec<u8>, String> {
        let mut v = vec![0; n];
        f.seek(SeekFrom::Start(off)).and_then(|_| f.read_exact(&mut v)).map_err(|e| format!("disc read @{off:#x}: {e}"))?;
        Ok(v)
    };
    let be = |b: &[u8], at: usize| u32::from_be_bytes(b[at..at + 4].try_into().unwrap());
    let h = rd(0x424, 8)?;
    let fst = rd(be(&h, 0) as u64, be(&h, 4) as usize)?;
    let n = be(&fst, 8) as usize;
    let names = &fst[n * 12..];
    for i in 1..n {
        let e = &fst[i * 12..i * 12 + 12];
        if e[0] != 0 {
            continue;
        }
        let no = (be(e, 0) & 0xFF_FFFF) as usize;
        let end = names[no..].iter().position(|&c| c == 0).unwrap_or(0);
        if names[no..no + end].eq_ignore_ascii_case(want.as_bytes()) {
            return rd(be(e, 4) as u64, be(e, 8) as usize);
        }
    }
    Err(format!("{want}: not on the disc"))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let synthetic = args.iter().any(|a| a == "--synthetic");
    let stage = get("--stage").unwrap_or_else(|| "GrNLa.dat".into());
    let rust_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let shot = get("--shot").map(PathBuf::from).unwrap_or_else(|| {
        if synthetic {
            rust_dir.join("../docs/rust-port/shots/gx_viewer_synthetic.png")
        } else {
            rust_dir.join("target/gx-viewer").join(stage.replace(".dat", ".png"))
        }
    });
    let mut shim = GxShim::new();
    let model = if synthetic {
        synthetic_stage(&mut shim)
    } else {
        let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
            eprintln!("gx_viewer: no disc (--iso or GW_ISO_VANILLA); --synthetic draws a built-in stage");
            std::process::exit(2);
        };
        let file = match read_disc_file(&iso, &stage) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("gx_viewer: {e}");
                std::process::exit(1);
            }
        };
        match load_stage(&file, &mut shim) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("gx_viewer: {stage}: {e}");
                std::process::exit(1);
            }
        }
    };
    eprintln!(
        "gx_viewer: {}: {} meshes, {} textures, bounds {:?}..{:?}, skipped {}{}",
        if synthetic { "synthetic" } else { &stage },
        model.meshes.len(),
        model.textures.len(),
        model.bmin,
        model.bmax,
        model.skipped.len(),
        model.skipped.first().map(|s| format!(" (first: {s})")).unwrap_or_default()
    );
    let viewer = Viewer {
        shim,
        model,
        presenter: None,
        frame: 0,
        frames: get("--frames").and_then(|v| v.parse().ok()),
        shot_frame: get("--shot-frame").and_then(|v| v.parse().ok()).unwrap_or(60),
        shot,
        shot_done: false,
        quit: false,
        failed: None,
        fit: synthetic || args.iter().any(|a| a == "--fit"),
    };
    let cfg = WindowConfig { title: format!("gx_viewer - {}", if synthetic { "synthetic" } else { &stage }), width: 960, height: 720, active: std::env::var_os("MELEE_PAD_SCRIPT").is_none() };
    match run_windowed(viewer, PadSystem::headless(), cfg) {
        Ok((ticks, _)) => eprintln!("gx_viewer: closed after {ticks} ticks"),
        Err(e) => {
            eprintln!("gx_viewer: event loop: {e}");
            std::process::exit(1);
        }
    }
}

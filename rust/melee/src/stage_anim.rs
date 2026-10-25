//! Stage model animation for the golden-match viewer: the JObj / MObj / TObj animation the C runs on a
//! map group's GObj (`Ground_801C1CD0` -> `HSD_JObjAnimAll`), driven through melee_runtime's ported
//! animation machinery (`jobj_rt::JObjRt`, `aobj`, `fobj`, `mobj::HsdMObj::update_func`,
//! `tobj::HsdTObj::update_func`), plus the request side of `gr/granime.c`
//! (`grAnime_801C8138`, `grAnime_801C7C1C`, `grAnime_801C752C`) over the stage file's
//! `map_head->unk8[map_id]` animation tables.
//!
//! The stage DAT stays big-endian bytes (offset addressed); the animated state lives here (joint SRTs in
//! the tree, material / TObj state per DObj descriptor) and is handed to the renderer as
//! `melee_runtime::tu::mobj::MatOverride`s.
//!
//! NOT ported: shape animation (FD has none), RObj animation (`robj_anim` is asserted empty),
//! TObj TLUT tables and TEV-konst colour tracks (the renderer's TEV is built from the descriptor),
//! `grMaterial` colour-overlay scripts.
fn be16(d: &[u8], at: u32) -> u16 {
    d.get(at as usize..at as usize + 2).map_or(0, |b| u16::from_be_bytes(b.try_into().unwrap()))
}
fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}
use melee_runtime::aobj::{HsdAObj, HsdAObjDesc, AOBJ_LOOP};
use melee_runtime::fobj::HsdFObjDesc;
use melee_runtime::hsd_util::GxColor;
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree, JOBJ_CLASSICAL_SCALE, JOBJ_INSTANCE, JOBJ_PTCL, JOBJ_SPLINE};
use melee_runtime::jobj_rt::{sort_anim, HsdAnimJoint, JObjExt, JObjRt};
use melee_runtime::mobj::{HsdMObj, HsdMaterial, HsdPeDesc};
use melee_runtime::mtx::Vec3;
use melee_runtime::robj::{RObjHost, HsdRObjAnimJoint};
use melee_runtime::tobj::{HsdTObj, HsdTObjTev};
use melee_runtime::tu::displayfunc::DispJoint;
use melee_runtime::tu::mobj::{MatDesc, MatOverride};
use melee_runtime::tu::state::Dat;
use melee_runtime::tu::tobj::load_desc_list;
use std::collections::HashMap;
use std::sync::Arc;

pub type Tree = JObjTree<u32, u32>;

// ---------------------------------------------------------------------------------------------
// DAT loaders
// ---------------------------------------------------------------------------------------------

fn load_fobj_chain(d: &[u8], mut fo: u32) -> Vec<HsdFObjDesc> {
    let mut v = Vec::new();
    while fo != 0 && v.len() < 64 {
        let (len, ty) = (be32(d, fo + 4), d[fo as usize + 12]);
        let ad = be32(d, fo + 16) as usize;
        let end = (ad + len as usize).min(d.len());
        v.push(HsdFObjDesc {
            length: len,
            startframe: bef(d, fo + 8),
            type_: ty,
            frac_value: d[fo as usize + 13],
            frac_slope: d[fo as usize + 14],
            ad: Arc::from(&d[ad.min(end)..end]),
        });
        fo = be32(d, fo);
    }
    v
}

pub fn load_aobj_desc(d: &[u8], at: u32) -> Option<HsdAObjDesc> {
    if at == 0 {
        return None;
    }
    Some(HsdAObjDesc { flags: be32(d, at), end_frame: bef(d, at + 4), fobjdesc: load_fobj_chain(d, be32(d, at + 8)), obj_id: be32(d, at + 12) })
}

/// An `HSD_AnimJoint` tree (child / next links).
pub fn load_anim_joint(d: &[u8], at: u32, depth: u32) -> Option<Box<HsdAnimJoint>> {
    if at == 0 || depth > 400 {
        return None;
    }
    assert!(be32(d, at + 0xC) == 0, "stage anim joint {at:#x}: robj_anim not supported");
    Some(Box::new(HsdAnimJoint {
        child: load_anim_joint(d, be32(d, at), depth + 1),
        next: load_anim_joint(d, be32(d, at + 4), depth + 1),
        aobjdesc: load_aobj_desc(d, be32(d, at + 8)),
        robj_anim: Vec::<HsdRObjAnimJoint>::new(),
        flags: be32(d, at + 0x10),
    }))
}

/// An `HSD_MatAnimJoint` tree whose payload is the `HSD_MatAnim` descriptor offset.
#[derive(Clone, Debug, Default)]
pub struct MatJoint {
    pub child: Option<Box<MatJoint>>,
    pub next: Option<Box<MatJoint>>,
    pub matanim: u32,
}

pub fn load_mat_joint(d: &[u8], at: u32, depth: u32) -> Option<Box<MatJoint>> {
    if at == 0 || depth > 400 {
        return None;
    }
    Some(Box::new(MatJoint { child: load_mat_joint(d, be32(d, at), depth + 1), next: load_mat_joint(d, be32(d, at + 4), depth + 1), matanim: be32(d, at + 8) }))
}

// ---------------------------------------------------------------------------------------------
// MObj / TObj animation state
// ---------------------------------------------------------------------------------------------

struct TObjState {
    desc: u32,
    id: u32,
    obj: HsdTObj<u32>,
    aobj: Option<HsdAObj<u32>>,
}

struct MObjState {
    mobj: HsdMObj<u32>,
    aobj: Option<HsdAObj<u32>>,
    tobjs: Vec<TObjState>,
}

fn rgba(c: GxColor) -> [u8; 4] {
    [c.r, c.g, c.b, c.a]
}
fn gxc(c: [u8; 4]) -> GxColor {
    GxColor { r: c[0], g: c[1], b: c[2], a: c[3] }
}

impl MObjState {
    /// `MObjLoad` + the TObj list of the MObj descriptor `mobj`.
    fn load(d: &Dat, mobj: u32) -> Self {
        let (rendermode, mat, pe) = melee_runtime::tu::mobj::mobj_load(d, mobj);
        let pe = pe.map(|b| HsdPeDesc {
            flags: b[0],
            ref0: b[1],
            ref1: b[2],
            dst_alpha: b[3],
            ty: b[4],
            src_factor: b[5],
            dst_factor: b[6],
            logic_op: b[7],
            z_comp: b[8],
            alpha_comp0: b[9],
            alpha_op: b[10],
            alpha_comp1: b[11],
        });
        let tobjs = load_desc_list(d, d.ptr(mobj + 8))
            .into_iter()
            .map(|t| {
                let mut obj = t.obj;
                if t.tev.is_some() {
                    obj.tev = Some(HsdTObjTev::default());
                }
                TObjState { desc: t.desc, id: t.id, obj, aobj: None }
            })
            .collect();
        MObjState {
            mobj: HsdMObj {
                rendermode,
                tobj: None,
                mat: Some(HsdMaterial { ambient: gxc(mat.ambient), diffuse: gxc(mat.diffuse), specular: gxc(mat.specular), alpha: mat.alpha, shininess: mat.shininess }),
                pe,
            },
            aobj: None,
            tobjs,
        }
    }

    fn over(&self, base_pe: Option<[u8; 12]>) -> MatOverride {
        let m = self.mobj.mat.as_ref().unwrap();
        let pe = base_pe.map(|mut b| {
            if let Some(p) = &self.mobj.pe {
                b[0] = p.flags;
                b[1] = p.ref0;
                b[2] = p.ref1;
                b[3] = p.dst_alpha;
            }
            b
        });
        MatOverride {
            mat: MatDesc { ambient: rgba(m.ambient), diffuse: rgba(m.diffuse), specular: rgba(m.specular), alpha: m.alpha, shininess: m.shininess },
            pe,
            tobjs: self.tobjs.iter().map(|t| (t.desc, t.obj.clone())).collect(),
        }
    }
}

/// The not-ported DObj layer of `JObjExt`: per-DObj material animation state.
pub struct MatExt {
    dat: Arc<Vec<u8>>,
    mobjs: HashMap<u32, MObjState>,
    base_pe: HashMap<u32, Option<[u8; 12]>>,
}

impl MatExt {
    fn mobj_of(&self, dobj: u32) -> Option<u32> {
        let m = be32(&self.dat, dobj + 8);
        (m != 0).then_some(m)
    }
}

impl RObjHost for MatExt {
    fn jobj_ref_this(&mut self, _: JObjId) {}
    fn jobj_unref_this(&mut self, _: Option<JObjId>) {}
    fn byte_code_eval(&mut self, _: Option<&[u8]>, _: &[f32], _: u32) -> f32 {
        panic!("stage animation: byte code expression")
    }
}

impl JObjExt for MatExt {
    type D = u32;
    type MatAnim = u32;
    type ShapeAnim = u32;
    fn dobj_remove_anim_all_by_flags(&mut self, _: &[u32], _: u32) {}
    /// `HSD_DObjReqAnimAllByFlags` -> `HSD_MObjReqAnimByFlags`: MObj tracks with flag 4, TObj tracks with 0x10.
    fn dobj_req_anim_all_by_flags(&mut self, dobj: &[u32], frame: f32, flags: u32) {
        for d in dobj {
            if let Some(m) = self.mobjs.get_mut(d) {
                if flags & 0x4 != 0 {
                    if let Some(a) = m.aobj.as_mut() {
                        a.req_anim(frame);
                    }
                }
                if flags & 0x10 != 0 {
                    for t in &mut m.tobjs {
                        if let Some(a) = t.aobj.as_mut() {
                            a.req_anim(frame);
                        }
                    }
                }
            }
        }
    }
    fn dobj_add_anim_all(&mut self, _: &[u32], _: Option<&u32>, _: Option<&u32>) {
        unreachable!("stage animation adds material animation through StageAnim::add_*")
    }
    /// `HSD_DObjAnimAll` -> `HSD_MObjAnim`: the MObj's AObj, then every TObj's.
    fn dobj_anim_all(&mut self, dobj: &[u32]) {
        let mut ag = melee_runtime::aobj::AObjGlobals::default();
        for d in dobj {
            if let Some(m) = self.mobjs.get_mut(d) {
                if let Some(mut a) = m.aobj.take() {
                    {
                        let mo = &mut m.mobj;
                        let mut f = |ty: u8, fv: f32| mo.update_func(ty as u32, fv);
                        a.interpret_anim(&mut ag, &mut Some(&mut f));
                    }
                    m.aobj = Some(a);
                }
                for t in &mut m.tobjs {
                    if let Some(mut a) = t.aobj.take() {
                        {
                            let to = &mut t.obj;
                            let mut f = |ty: u8, fv: f32| to.update_func(ty as u32, fv);
                            a.interpret_anim(&mut ag, &mut Some(&mut f));
                        }
                        t.aobj = Some(a);
                    }
                }
            }
        }
    }
    fn spline_point(&mut self, _: JObjId, _: f32) -> Vec3 {
        panic!("stage animation: PATH track")
    }
    fn aobj_resolve_obj(&mut self, _: u32) -> Option<JObjId> {
        None
    }
    fn jobj_ref(&mut self, _: Option<JObjId>) {}
    fn jobj_unref(&mut self, _: Option<JObjId>) {}
}

// ---------------------------------------------------------------------------------------------
// The stage
// ---------------------------------------------------------------------------------------------

/// What `grAnime_801C752C` applies to every selected AObj.
#[derive(Clone, Copy, Debug)]
pub enum AOp {
    SetRate(f32),
    ClearFlags(u32),
    SetFlags(u32),
}

fn apply_op<O>(a: &mut HsdAObj<O>, op: AOp) {
    match op {
        AOp::SetRate(r) => a.set_rate(r),
        AOp::ClearFlags(f) => a.clear_flags(f),
        AOp::SetFlags(f) => a.set_flags(f),
    }
}

/// `anim_flags(arg2)` of granime.c: the object classes a `kinds` mask (1 joint, 2 material, 4 shape) selects.
pub fn anim_flags(kinds: u32) -> u32 {
    let mut f = 0;
    if kinds & 1 != 0 {
        f |= 0x220;
    }
    if kinds & 2 != 0 {
        f |= 0x7484;
    }
    if kinds & 4 != 0 {
        f |= 0x100;
    }
    f
}

pub struct GroupAnim {
    pub map_id: u32,
    pub tree: Tree,
    /// Joint descriptor of each JObj (allocation = preorder), as `Drawable::descs`.
    pub descs: Vec<u32>,
    pub root: JObjId,
}

pub struct StageAnim {
    dat: Arc<Vec<u8>>,
    pub rt: JObjRt<MatExt>,
    pub groups: Vec<GroupAnim>,
    /// `map_head` group table (0x34 bytes per group).
    table: u32,
    /// Scene models (`{joint, anims, matanims, shapeanims}` records, e.g. IfAll's HUD): the entry of
    /// each group id, in place of the `map_head` table. Empty for a stage.
    entries: Vec<u32>,
}

fn load_tree(d: &[u8], at: u32, tree: &mut Tree, parent: Option<JObjId>, descs: &mut Vec<u32>, depth: u32) -> Option<JObjId> {
    if at == 0 || depth > 200 || at as usize + 0x40 > d.len() {
        return None;
    }
    let j = tree.alloc();
    descs.push(at);
    let flags = be32(d, at + 4);
    if flags & JOBJ_INSTANCE == 0 {
        let c = load_tree(d, be32(d, at + 8), tree, Some(j), descs, depth + 1);
        tree.get_mut(j).child = c;
    }
    let n = load_tree(d, be32(d, at + 0xC), tree, parent, descs, depth + 1);
    let v = |o: u32| Vec3 { x: bef(d, at + o), y: bef(d, at + o + 4), z: bef(d, at + o + 8) };
    let r = v(0x14);
    // The DObj chain of a model joint (the union holds DObjs unless PTCL / SPLINE).
    let mut dobjs = Vec::new();
    if flags & (JOBJ_PTCL | JOBJ_SPLINE) == 0 {
        let mut x = be32(d, at + 0x10);
        while x != 0 && dobjs.len() < 256 {
            dobjs.push(x);
            x = be32(d, x + 4);
        }
    }
    let o = tree.get_mut(j);
    o.next = n;
    o.parent = parent;
    o.flags |= flags;
    o.rotate.x = r.x;
    o.rotate.y = r.y;
    o.rotate.z = r.z;
    o.scale = v(0x20);
    o.translate = v(0x2C);
    o.dobj = dobjs;
    Some(j)
}

impl StageAnim {
    pub fn new(dat: Arc<Vec<u8>>, map_head: u32, ids: &[u32]) -> Self {
        let table = be32(&dat, map_head + 8);
        Self::build(dat, table, Vec::new(), ids)
    }

    /// A scene-model animation host: group `i` is the `DynamicModelDesc` at `entries[i]`.
    pub fn new_scene(dat: Arc<Vec<u8>>, entries: Vec<u32>) -> Self {
        let ids: Vec<u32> = (0..entries.len() as u32).collect();
        Self::build(dat, 0, entries, &ids)
    }

    fn build(dat: Arc<Vec<u8>>, table: u32, entries: Vec<u32>, ids: &[u32]) -> Self {
        let d: &[u8] = &dat;
        let mut groups = Vec::new();
        let mut mobjs = HashMap::new();
        let mut base_pe = HashMap::new();
        for &g in ids {
            let root_desc = be32(d, if entries.is_empty() { table + g * 0x34 } else { entries[g as usize] });
            if root_desc == 0 {
                continue;
            }
            let mut tree = Tree::new();
            let mut descs = Vec::new();
            let Some(root) = load_tree(d, root_desc, &mut tree, None, &mut descs, 0) else { continue };
            for n in &tree.nodes {
                for &dobj in &n.dobj {
                    if let Some(m) = (be32(d, dobj + 8) != 0).then(|| be32(d, dobj + 8)) {
                        mobjs.entry(dobj).or_insert_with(|| MObjState::load(&Dat(d), m));
                        base_pe.entry(dobj).or_insert_with(|| melee_runtime::tu::mobj::mobj_load(&Dat(d), m).2);
                    }
                }
            }
            groups.push(GroupAnim { map_id: g, tree, descs, root });
        }
        let rt = JObjRt::new(MatExt { dat: dat.clone(), mobjs, base_pe });
        StageAnim { dat, rt, groups, table, entries }
    }

    fn gi(&self, map_id: u32) -> usize {
        self.groups.iter().position(|g| g.map_id == map_id).unwrap_or_else(|| panic!("stage group {map_id} not loaded"))
    }

    /// `Ground_801C3FA4(gobj, idx)`: the idx'th JObj of the group in depth-first order (allocation order).
    fn nth(&self, gi: usize, idx: u32) -> Option<JObjId> {
        ((idx as usize) < self.groups[gi].tree.nodes.len()).then_some(JObjId(idx))
    }

    fn entry(&self, map_id: u32) -> u32 {
        if self.entries.is_empty() { self.table + map_id * 0x34 } else { self.entries[map_id as usize] }
    }

    // --- adding animation ---------------------------------------------------------------------

    /// `HSD_MObjAddAnim` (`hsd`) / `grAnime_801C67A8`: the MObj's AObj and its TObjs'.
    fn add_mobj_anim(&mut self, dobj: u32, matanim: u32, hsd: bool) {
        let dat = self.dat.clone();
        let d: &[u8] = &dat;
        let ext = self.rt.ext.clone();
        let mut ext = ext.borrow_mut();
        let Some(m) = ext.mobjs.get_mut(&dobj) else { return };
        if matanim == 0 {
            return;
        }
        let ad = be32(d, matanim + 4);
        if hsd || ad != 0 {
            m.aobj = load_aobj_desc(d, ad).map(|x| HsdAObj::load_desc(&x, |_| None));
        }
        // HSD_TObjAddAnimAll / grAnime_801C6710: the texanim whose id matches the TObj's.
        for t in &mut m.tobjs {
            let mut ta = be32(d, matanim + 8);
            while ta != 0 && be32(d, ta + 4) != t.id {
                ta = be32(d, ta);
            }
            if ta == 0 {
                continue;
            }
            let aobjdesc = be32(d, ta + 8);
            if !hsd && aobjdesc == 0 {
                continue;
            }
            t.aobj = load_aobj_desc(d, aobjdesc).map(|x| HsdAObj::load_desc(&x, |_| None));
            let (tbl, n) = (be32(d, ta + 12), d[ta as usize + 0x14] as u32 * 256 + d[ta as usize + 0x15] as u32);
            t.obj.imagetbl = (tbl != 0).then(|| (0..n).map(|i| (be32(d, be32(d, tbl + 4 * i)) != 0 || true).then(|| be32(d, tbl + 4 * i)).filter(|&p| p != 0)).collect());
            let (ttbl, tn) = (be32(d, ta + 0x10), d[ta as usize + 0x16] as u32 * 256 + d[ta as usize + 0x17] as u32);
            t.obj.has_tluttbl = ttbl != 0 && tn != 0;
            t.obj.tluttbl_ptrs = if t.obj.has_tluttbl { (0..tn).map(|i| be32(d, ttbl + 4 * i)).collect() } else { Vec::new() };
            if hsd {
                t.obj.tlut_no = -1;
            }
        }
    }

    /// `HSD_JObjAddAnim` (`hsd`) / `grAnime_801C6A54` on one JObj.
    fn add_anim_one(&mut self, gi: usize, j: JObjId, aj: Option<&HsdAnimJoint>, mj: Option<&MatJoint>, hsd: bool) {
        if let Some(a) = aj {
            if a.aobjdesc.is_some() || hsd {
                let mut o = a.aobjdesc.as_ref().map(|x| HsdAObj::load_desc(x, |_| None));
                sort_anim(o.as_mut());
                self.groups[gi].tree.get_mut(j).aobj = o;
            }
            if hsd {
                let n = self.groups[gi].tree.get_mut(j);
                if a.flags & 1 != 0 {
                    n.flags |= JOBJ_CLASSICAL_SCALE;
                } else {
                    n.flags &= !JOBJ_CLASSICAL_SCALE;
                }
            }
        }
        let dobjs = self.groups[gi].tree.get(j).dobj.clone();
        if !dobjs.is_empty() {
            // HSD_DObjAddAnimAll pairs the DObj chain with the MatAnim chain (`next` at +0).
            let mut ma = mj.map_or(0, |m| m.matanim);
            for dobj in dobjs {
                self.add_mobj_anim(dobj, ma, hsd);
                ma = if ma != 0 { be32(&self.dat, ma) } else { 0 };
            }
        }
    }

    fn add_anim_rec(&mut self, gi: usize, j: JObjId, aj: Option<&HsdAnimJoint>, mj: Option<&MatJoint>, hsd: bool) {
        self.add_anim_one(gi, j, aj, mj, hsd);
        if self.groups[gi].tree.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = self.groups[gi].tree.get(j).child;
            let mut a = aj.and_then(|a| a.child.as_deref());
            let mut m = mj.and_then(|m| m.child.as_deref());
            while let Some(x) = c {
                self.add_anim_rec(gi, x, a, m, hsd);
                c = self.groups[gi].tree.get(x).next;
                a = a.and_then(|a| a.next.as_deref());
                m = m.and_then(|m| m.next.as_deref());
            }
        }
    }

    // --- requests / flag ops --------------------------------------------------------------------

    /// `HSD_JObjReqAnimAllByFlags` / `HSD_JObjReqAnimByFlags` on `j`.
    fn req(&mut self, gi: usize, j: JObjId, flags: u32, frame: f32, all: bool) {
        let g = &mut self.groups[gi];
        if all {
            self.rt.req_anim_all_by_flags(&mut g.tree, Some(j), flags, frame);
        } else {
            self.rt.req_anim_by_flags(&mut g.tree, Some(j), flags, frame);
        }
    }

    /// `grAnime_801C752C`: `op` on every AObj of the classes `flags` selects, on `j` and (if `rec`) below.
    fn for_aobjs(&mut self, gi: usize, j: JObjId, flags: u32, rec: bool, op: AOp) {
        let (jf, dobjs) = {
            let n = self.groups[gi].tree.get(j);
            (n.flags, n.dobj.clone())
        };
        if flags & 0x20 != 0 {
            if let Some(a) = self.groups[gi].tree.get_mut(j).aobj.as_mut() {
                apply_op(a, op);
            }
        }
        if jf & 0x4020 == 0 {
            let ext = self.rt.ext.clone();
            let mut ext = ext.borrow_mut();
            for dobj in dobjs {
                if let Some(m) = ext.mobjs.get_mut(&dobj) {
                    if flags & 0x80 != 0 {
                        if let Some(a) = m.aobj.as_mut() {
                            apply_op(a, op);
                        }
                    }
                    if flags & 0x400 != 0 {
                        for t in &mut m.tobjs {
                            if let Some(a) = t.aobj.as_mut() {
                                apply_op(a, op);
                            }
                        }
                    }
                }
            }
        }
        if rec && jf & JOBJ_INSTANCE == 0 {
            let mut c = self.groups[gi].tree.get(j).child;
            while let Some(x) = c {
                self.for_aobjs(gi, x, flags, true, op);
                c = self.groups[gi].tree.get(x).next;
            }
        }
    }

    /// `grAnime_801C7C1C` (via `grAnime_801C7FF8` when `all`, `grAnime_801C8098` otherwise) plus the
    /// `grAnime_801C7980` / `grAnime_801C7A94` follow-ups the caller makes through `after`.
    /// `idx` is the DFS index of the JObj, `kinds` the 1 joint / 2 material / 4 shape mask, `set` the
    /// animation set.
    pub fn grdo_anim(&mut self, map_id: u32, idx: u32, kinds: u32, set: i32, all: bool, frame: f32, rate: f32) {
        if set < 0 {
            return;
        }
        let gi = self.gi(map_id);
        let Some(j) = self.nth(gi, idx) else { return };
        let dat = self.dat.clone();
        let d: &[u8] = &dat;
        let e = self.entry(map_id);
        let (ajp, mjp) = (be32(d, e + 4), be32(d, e + 8));
        let mut req_flags = 0;
        let mut aj = None;
        let mut mj = None;
        if kinds & 1 != 0 && ajp != 0 && be32(d, ajp + 4 * set as u32) != 0 {
            let base = be32(d, ajp + 4 * set as u32) + idx * 0x14;
            aj = load_anim_joint(d, base, 0);
            req_flags |= 0x81;
        }
        if kinds & 2 != 0 && mjp != 0 && be32(d, mjp + 4 * set as u32) != 0 {
            let base = be32(d, mjp + 4 * set as u32) + idx * 0xC;
            mj = load_mat_joint(d, base, 0);
            req_flags |= 0x416;
        }
        let af = anim_flags(kinds) & (if aj.is_some() { 0x220 } else { 0 } | if mj.is_some() { 0x7484 } else { 0 } | if kinds & 4 != 0 { 0x100 } else { 0 });
        // Note the C passes the full kind flags to grAnime_801C752C below; computed from the sets that exist.
        if all {
            self.add_anim_rec(gi, j, aj.as_deref(), mj.as_deref(), false);
        } else {
            self.add_anim_one(gi, j, aj.as_deref(), mj.as_deref(), false);
        }
        self.req(gi, j, req_flags, frame, all);
        // grAnime_801C752C(.., HSD_AObjSetRate, 1, rate) calls the setter through `Callback1`
        // (aobj, obj, flags, float): on the native x64 build the float travels in the 4th argument
        // slot but HSD_AObjSetRate(aobj, f32) reads its 2nd, so the AObj's framerate becomes 0 (the
        // PPC build had f1 for both). Measured against the C build's screenshots: the FD backdrop
        // (gobjs 4-8) never moves. `rate` is kept for callers that want the PPC behaviour.
        let _ppc_rate = rate;
        self.for_aobjs(gi, j, af, all, AOp::SetRate(if std::env::var_os("MELEE_VIEW_PPC_RATE").is_some() { rate } else { 0.0 }));
        // eflags: the per-set "loop" byte (`x28`).
        let ef = if self.entries.is_empty() { be32(d, e + 0x28) } else { 0 };
        if ef != 0 && d[ef as usize + set as usize] != 0 {
            self.for_aobjs(gi, j, af, all, AOp::SetFlags(AOBJ_LOOP));
        }
    }

    /// `grAnime_801C7980(gobj, idx, kinds)`: clear the loop flag.
    pub fn grdo_clear_loop(&mut self, map_id: u32, idx: u32, kinds: u32) {
        let gi = self.gi(map_id);
        if let Some(j) = self.nth(gi, idx) {
            self.for_aobjs(gi, j, anim_flags(kinds), false, AOp::ClearFlags(AOBJ_LOOP));
        }
    }

    /// `grAnime_801C8138(gobj, map_id, set)`: reset to the base pose and start animation set `set`.
    pub fn start_map_anim(&mut self, map_id: u32, set: i32) {
        self.start_anim_set(map_id, set, true);
    }

    /// `start_map_anim` with `req == false`: only attach the set (`lb_8000C07C` without the request).
    pub fn start_anim_set(&mut self, map_id: u32, set: i32, req: bool) {
        if set < 0 {
            return;
        }
        let gi = self.gi(map_id);
        let dat = self.dat.clone();
        let d: &[u8] = &dat;
        let e = self.entry(map_id);
        let root = self.groups[gi].root;
        // lb_8000B804: the joint descriptors' SRT back onto the tree (preorder).
        let descs = self.groups[gi].descs.clone();
        for (i, &desc) in descs.iter().enumerate() {
            let n = self.groups[gi].tree.get_mut(JObjId(i as u32));
            n.rotate.x = bef(d, desc + 0x14);
            n.rotate.y = bef(d, desc + 0x18);
            n.rotate.z = bef(d, desc + 0x1C);
            n.scale = Vec3 { x: bef(d, desc + 0x20), y: bef(d, desc + 0x24), z: bef(d, desc + 0x28) };
            n.translate = Vec3 { x: bef(d, desc + 0x2C), y: bef(d, desc + 0x30), z: bef(d, desc + 0x34) };
            n.flags |= melee_runtime::jobj::JOBJ_MTX_DIRTY;
        }
        let pick = |tbl: u32, size: u32| -> u32 {
            if tbl != 0 && be32(d, tbl + 4 * set as u32) != 0 {
                be32(d, tbl + 4 * set as u32)
            } else {
                0
            }
            .min(if size == 0 { 0 } else { u32::MAX })
        };
        let aj = load_anim_joint(d, pick(be32(d, e + 4), 1), 0);
        let mj = load_mat_joint(d, pick(be32(d, e + 8), 1), 0);
        // HSD_JObjRemoveAnimAll: drop whatever was there.
        for n in &mut self.groups[gi].tree.nodes {
            n.aobj = None;
        }
        {
            let ext = self.rt.ext.clone();
            let mut ext = ext.borrow_mut();
            for m in ext.mobjs.values_mut() {
                m.aobj = None;
                for t in &mut m.tobjs {
                    t.aobj = None;
                }
            }
        }
        self.add_anim_rec(gi, root, aj.as_deref(), mj.as_deref(), true);
        if !req {
            return;
        }
        self.req(gi, root, 0x7FF, 0.0, true);
        let ef = if self.entries.is_empty() { be32(d, e + 0x28) } else { 0 };
        if ef != 0 && d[ef as usize + set as usize] != 0 {
            self.for_aobjs(gi, root, 0x77A4, true, AOp::SetFlags(AOBJ_LOOP));
        }
        self.anim_group(gi);
    }

    fn anim_group(&mut self, gi: usize) {
        let g = &mut self.groups[gi];
        let root = g.root;
        self.rt.anim_all(&mut g.tree, Some(root));
        // The root's `next` chain (the C parents siblings under the same wrapper, so they animate too).
        let mut x = self.groups[gi].tree.get(root).next;
        while let Some(s) = x {
            let g = &mut self.groups[gi];
            self.rt.anim_all(&mut g.tree, Some(s));
            x = self.groups[gi].tree.get(s).next;
        }
    }

    /// One frame of `Ground_801C1CD0` (`HSD_JObjAnimAll`) for every group.
    pub fn step(&mut self) {
        for gi in 0..self.groups.len() {
            self.anim_group(gi);
        }
    }

    /// World matrices of a group's joints (preorder from its roots), `HSD_JObjSetupMatrix` on the tree.
    pub fn disp_joints(&mut self, map_id: u32) -> Vec<DispJoint> {
        let gi = self.gi(map_id);
        let mut out: Vec<DispJoint> = Vec::new();
        let mut index: HashMap<u32, usize> = HashMap::new();
        let g = &mut self.groups[gi];
        let mut roots = vec![g.root];
        let mut x = g.tree.get(g.root).next;
        while let Some(s) = x {
            roots.push(s);
            x = g.tree.get(s).next;
        }
        for r in roots {
            let mut order = Vec::new();
            g.tree.walk_tree(Some(r), &mut |j, _| order.push(j));
            for j in order {
                if index.contains_key(&j.0) {
                    continue;
                }
                g.tree.setup_matrix(&mut self.rt, Some(j));
                let o = g.tree.get(j);
                let parent = o.parent.and_then(|p| index.get(&p.0).copied());
                index.insert(j.0, out.len());
                out.push(DispJoint { desc: g.descs[j.0 as usize], flags: o.flags, mtx: o.mtx, parent });
            }
        }
        out
    }

    /// The animated materials for the renderer.
    pub fn overrides(&self) -> HashMap<u32, MatOverride> {
        let ext = self.rt.ext.borrow();
        let skip: Vec<u32> = std::env::var("MELEE_VIEW_ALPHA0").map(|v| v.split(',').filter_map(|x| u32::from_str_radix(x.trim().trim_start_matches("0x"), 16).ok()).collect()).unwrap_or_default();
        ext.mobjs
            .iter()
            .map(|(&d, m)| {
                let mut o = m.over(ext.base_pe.get(&d).copied().flatten());
                if skip.contains(&d) {
                    o.mat.alpha = 0.0;
                }
                (d, o)
            })
            .collect()
    }
}

#[allow(dead_code)]
fn _host_check<H: JObjHost<u32, u32>>() {}
#[allow(dead_code)]
fn _is_host() {
    _host_check::<JObjRt<MatExt>>();
}

impl StageAnim {
    /// Debug: one line per JObj of a group (index, flags, T, S, DObj material alphas, anim presence).
    pub fn dump(&self, map_id: u32) {
        let gi = self.gi(map_id);
        let g = &self.groups[gi];
        let ext = self.rt.ext.borrow();
        for (i, n) in g.tree.nodes.iter().enumerate() {
            let mats: Vec<String> = n
                .dobj
                .iter()
                .map(|d| {
                    ext.mobjs.get(d).map_or("-".into(), |m| {
                        let mm = m.mobj.mat.as_ref().unwrap();
                        format!("a{:.2}/d{:02x}{:02x}{:02x}{}{}", mm.alpha, mm.diffuse.r, mm.diffuse.g, mm.diffuse.b, if m.aobj.is_some() { "M" } else { "" }, m.tobjs.iter().filter(|t| t.aobj.is_some()).count())
                    })
                })
                .collect();
            if std::env::var_os("MELEE_VIEW_TOBJDUMP").is_some() {
                for d in &n.dobj {
                    if let Some(m) = ext.mobjs.get(d) {
                        if let Some(a) = m.aobj.as_ref() {
                            eprintln!("    dobj {d:#x} mobj aobj f{:.1}/{:.1} fobjs {:?}", a.curr_frame, a.end_frame, a.fobj.iter().map(|f| (f.obj_type, f.p0, f.p1)).collect::<Vec<_>>());
                        }
                        for t in &m.tobjs {
                            eprintln!("    dobj {d:#x} tobj id {} flags {:#x} blend {} image {:?} {:?} tbl {:?} aobj {}", t.id, t.obj.flags, t.obj.blending, t.obj.imagedesc, t.obj.imagedesc.map(|i| (be16(&self.dat, i + 4), be16(&self.dat, i + 6), be32(&self.dat, i + 8))), t.obj.imagetbl.as_ref().map(|v| v.iter().map(|i| i.map(|i| (i, be16(&self.dat, i + 4), be16(&self.dat, i + 6), be32(&self.dat, i + 8)))).collect::<Vec<_>>()), t.aobj.as_ref().map_or("-".into(), |a| format!("f{:.1}/{:.1} fobjs {:?}", a.curr_frame, a.end_frame, a.fobj.iter().map(|f| (f.obj_type, f.op, f.p0, f.p1, f.time, f.startframe, f.nb_pack, f.frac_value, format!("{:02x?}", &f.ad_head[..f.ad_head.len().min(f.length as usize)]))).collect::<Vec<_>>())));
                        }
                    }
                }
            }
            if let Some(a) = n.aobj.as_ref() {
                for f in &a.fobj {
                    eprintln!("      fobj ty {} op {} intrp {} len {} time {:.1} p0 {:.3} p1 {:.3} d0 {:.3} d1 {:.3} start {} nb {}", f.obj_type, f.op, f.op_intrp, f.length, f.time, f.p0, f.p1, f.d0, f.d1, f.startframe, f.nb_pack);
                }
            }
            eprintln!("  g{map_id} j{i} flags {:#x} T {:.1},{:.1},{:.1} S {:.2},{:.2},{:.2} aobj {} dobj {:?}", n.flags, n.translate.x, n.translate.y, n.translate.z, n.scale.x, n.scale.y, n.scale.z, n.aobj.as_ref().map_or("-".into(), |a| format!("f{:.0}/{:.0}{} {:?}", a.curr_frame, a.end_frame, if a.flags & AOBJ_LOOP != 0 { "L" } else { "" }, a.fobj.iter().map(|f| f.obj_type).collect::<Vec<_>>())), mats);
        }
    }
}

impl StageAnim {
    /// Debug: per group, the number of JObjs with the HIDDEN flag.
    pub fn hidden_summary(&self) -> String {
        self.groups.iter().map(|g| format!("g{}:{}/{}", g.map_id, g.tree.nodes.iter().filter(|n| n.flags & melee_runtime::jobj::JOBJ_HIDDEN != 0).count(), g.tree.nodes.len())).collect::<Vec<_>>().join(" ")
    }
}

impl StageAnim {
    /// Debug: hide one JObj (and its subtree draw) of a group.
    pub fn hide(&mut self, map_id: u32, j: u32) {
        let gi = self.gi(map_id);
        self.groups[gi].tree.get_mut(JObjId(j)).flags |= melee_runtime::jobj::JOBJ_HIDDEN;
    }
}

/// Scene-model (HUD) helpers: the parts of `ifstatus.c` / `ifall.c` that poke single JObjs.
impl StageAnim {
    pub fn jobj_count(&self, map_id: u32) -> usize {
        self.groups[self.gi(map_id)].tree.nodes.len()
    }

    /// `lb_8000B1CC(jobj, NULL, &out)`: the world position of the idx'th JObj.
    pub fn joint_world_pos(&mut self, map_id: u32, idx: u32) -> Vec3 {
        let gi = self.gi(map_id);
        let g = &mut self.groups[gi];
        let j = JObjId(idx);
        g.tree.setup_matrix(&mut self.rt, Some(j));
        let m = g.tree.get(j).mtx;
        Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] }
    }

    pub fn translate_of(&self, map_id: u32, idx: u32) -> Vec3 {
        self.groups[self.gi(map_id)].tree.get(JObjId(idx)).translate
    }

    /// `HSD_JObjSetTranslate` (+ matrix dirty).
    pub fn set_translate(&mut self, map_id: u32, idx: u32, t: Vec3) {
        let gi = self.gi(map_id);
        let n = self.groups[gi].tree.get_mut(JObjId(idx));
        n.translate = t;
        n.flags |= melee_runtime::jobj::JOBJ_MTX_DIRTY;
    }

    pub fn set_scale(&mut self, map_id: u32, idx: u32, s: Vec3) {
        let gi = self.gi(map_id);
        let n = self.groups[gi].tree.get_mut(JObjId(idx));
        n.scale = s;
        n.flags |= melee_runtime::jobj::JOBJ_MTX_DIRTY;
    }

    /// `HSD_JObjSetFlagsAll` / `ClearFlagsAll` of JOBJ_HIDDEN on the idx'th JObj and its subtree.
    pub fn set_hidden_all(&mut self, map_id: u32, idx: u32, hidden: bool) {
        let gi = self.gi(map_id);
        let mut stack = vec![JObjId(idx)];
        let mut first = true;
        while let Some(j) = stack.pop() {
            let n = self.groups[gi].tree.get_mut(j);
            if hidden {
                n.flags |= melee_runtime::jobj::JOBJ_HIDDEN;
            } else {
                n.flags &= !melee_runtime::jobj::JOBJ_HIDDEN;
            }
            if n.flags & JOBJ_INSTANCE == 0 {
                if let Some(c) = n.child {
                    stack.push(c);
                }
            }
            if !first {
                if let Some(x) = n.next {
                    stack.push(x);
                }
            }
            first = false;
        }
    }

    pub fn dobj_of(&self, map_id: u32, idx: u32) -> Option<u32> {
        self.groups[self.gi(map_id)].tree.get(JObjId(idx)).dobj.first().copied()
    }

    /// `mobj->mat->diffuse.rgb = rgb` on the idx'th JObj's first DObj.
    pub fn set_diffuse(&mut self, map_id: u32, idx: u32, rgb: [u8; 3]) {
        let Some(d) = self.dobj_of(map_id, idx) else { return };
        let ext = self.rt.ext.clone();
        let mut ext = ext.borrow_mut();
        if let Some(m) = ext.mobjs.get_mut(&d) {
            if let Some(mat) = m.mobj.mat.as_mut() {
                mat.diffuse.r = rgb[0];
                mat.diffuse.g = rgb[1];
                mat.diffuse.b = rgb[2];
            }
        }
    }

    /// The `HSD_MatAnim` at `path` (child = 0 / next = 1 steps from the root of animation set `set`'s
    /// `HSD_MatAnimJoint` tree) -- C's `(*matanims)[set]->child->next->matanim`. 0 if the path ends.
    pub fn matanim_at(&self, map_id: u32, set: u32, path: &[u32]) -> u32 {
        let d: &[u8] = &self.dat;
        let e = self.entry(map_id);
        let mjp = be32(d, e + 8);
        if mjp == 0 {
            return 0;
        }
        let mut j = be32(d, mjp + 4 * set);
        for &step in path {
            if j == 0 {
                return 0;
            }
            j = be32(d, j + 4 * step);
        }
        if j == 0 { 0 } else { be32(d, j + 8) }
    }

    /// `HSD_TObjAddAnimAll(tobj, matanim->texanim)` + `HSD_TObjReqAnimAll(tobj, frame)` +
    /// `HSD_AObjSetRate(tobj->aobj, rate)` + `HSD_TObjAnim(tobj)` on the idx'th JObj's first DObj's
    /// MObj. The TObj chain pairs with the texanim chain positionally.
    pub fn tex_anim(&mut self, map_id: u32, idx: u32, matanim: u32, frame: f32, rate: f32) {
        let Some(dobj) = self.dobj_of(map_id, idx) else { return };
        if matanim == 0 {
            return;
        }
        let dat = self.dat.clone();
        let d: &[u8] = &dat;
        let ext = self.rt.ext.clone();
        let mut ext = ext.borrow_mut();
        let Some(m) = ext.mobjs.get_mut(&dobj) else { return };
        let mut ta = be32(d, matanim + 8);
        for t in &mut m.tobjs {
            if ta == 0 {
                break;
            }
            t.aobj = load_aobj_desc(d, be32(d, ta + 8)).map(|x| HsdAObj::load_desc(&x, |_| None));
            let (tbl, n) = (be32(d, ta + 12), d[ta as usize + 0x14] as u32 * 256 + d[ta as usize + 0x15] as u32);
            t.obj.imagetbl = (tbl != 0).then(|| (0..n).map(|i| Some(be32(d, tbl + 4 * i)).filter(|&p| p != 0)).collect());
            t.obj.tlut_no = -1;
            let (ttbl, tn) = (be32(d, ta + 0x10), d[ta as usize + 0x16] as u32 * 256 + d[ta as usize + 0x17] as u32);
            t.obj.has_tluttbl = ttbl != 0 && tn != 0;
            t.obj.tluttbl_ptrs = if t.obj.has_tluttbl { (0..tn).map(|i| be32(d, ttbl + 4 * i)).collect() } else { Vec::new() };
            if let Some(a) = t.aobj.as_mut() {
                a.req_anim(frame);
                a.set_rate(rate);
            }
            ta = be32(d, ta);
        }
        ext.dobj_anim_all(&[dobj]);
    }
}

impl StageAnim {
    /// `lb_8000B09C(root)`: some JObj still has a running joint AObj.
    pub fn joint_anim_running(&self, map_id: u32) -> bool {
        self.groups[self.gi(map_id)].tree.nodes.iter().any(|n| n.aobj.as_ref().is_some_and(|a| a.flags & melee_runtime::aobj::AOBJ_NO_ANIM == 0))
    }

    /// `HSD_JObjReqAnimAll(jobj, frame)` on the idx'th JObj.
    pub fn req_joint(&mut self, map_id: u32, idx: u32, frame: f32) {
        let gi = self.gi(map_id);
        self.req(gi, JObjId(idx), 0x7FF, frame, true);
    }

    /// `HSD_TObjReqAnimAll(tobj, frame)` + `HSD_AObjSetRate(tobj->aobj, rate)` on the idx'th JObj's
    /// first DObj, then `HSD_TObjAnim` (the animation must already be attached).
    pub fn tobj_req(&mut self, map_id: u32, idx: u32, frame: f32, rate: f32) {
        let Some(dobj) = self.dobj_of(map_id, idx) else { return };
        let ext = self.rt.ext.clone();
        let mut ext = ext.borrow_mut();
        if let Some(m) = ext.mobjs.get_mut(&dobj) {
            for t in &mut m.tobjs {
                if let Some(a) = t.aobj.as_mut() {
                    a.req_anim(frame);
                    a.set_rate(rate);
                }
            }
        }
        ext.dobj_anim_all(&[dobj]);
    }
}

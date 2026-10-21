//! The fighter's JObj skeleton and its figatree animation, as the native match needs them for the
//! bone-driven ECB (`mpColl_LoadECB_JObj` reads six joints' world positions).
//!
//! Ports (each cited at its function): `HSD_JObjLoadJoint` / `JObjLoad` (the SRT and flag half;
//! the DObj/RObj/envelope half is render-only and the fighter models checked here have no RObj
//! descriptors), `ftParts_SetupParts` (part index = DFS order, depth `xC`), the figatree attach of
//! `ftAnim_8006EBE8` with no blend (`ftAnim_80070758` -> `HSD_JObjRemoveAnimAllByFlags(.., 1)`,
//! `ftAnim_8006FA58` with `lb_8000B4FC` / `lb_8000B5DC` + `ftCommon_8007F6A4`, `ftAnim_8006F4C8` ->
//! `lbAnim_8001E6D8` / `lbAnim_InitFrames` / `lbAnim_JObjSortAnim`, `ftAnim_80070710` ->
//! `HSD_JObjReqAnimAllByFlags(.., 1, frame)`, `HSD_ForeachAnim` AObjSetFlags(LOOP) / SetRate),
//! the per-frame `ftAnim_8006E9B4` no-blend path (`HSD_JObjClearFlagsAll(QUATERNION)` ->
//! `ftAnim_8006E7B8(TopN)` -> `HSD_JObjAnim`), `ftAnim_8006F3DC` and `lb_8000B1CC(j, NULL)`.
//! Matrices are melee_runtime's jobj.rs (`HSD_JObjSetupMatrix`).
//!
//! Not modelled (none reached by pad_falcon_fox_fd before the first blended change at golden
//! frame 95, and none read by the ECB joints): animation blending (`x8AC_animSkeleton`,
//! `ftAnim_8006FE9C`), the TransN root-motion path (`x594_b0`, `ftAnim_8006E054`), dynamics bones
//! (`flags_b0`, ftdynamics.c: Falcon's scarf / Fox's tail are physics-driven there; here they keep
//! their animated pose), cross-kind figatrees (`ftAnim_8006FCE4`), DObj (material) animation.
//! GameCube records: `melee_types::HSD_Joint` (0x40), `FigaTree` (0x14), `FigaTrack` (0xC).

use melee_runtime::aobj::{AObjGlobals, HsdAObj, AOBJ_LOOP};
use melee_runtime::fobj::HsdFObj;
use melee_runtime::jobj::{self as hj, JObjHost, JObjId, JObjTree};
use melee_runtime::jobj_rt::{self, ObjData, UfcEvent};
use melee_runtime::mtx::Vec3;
use std::sync::Arc;

pub type Tree = JObjTree<(), ()>;

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
}
fn bef(b: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(b, at))
}
fn be16(b: &[u8], at: u32) -> u16 {
    u16::from_be_bytes(b[at as usize..at as usize + 2].try_into().unwrap())
}
fn vec3(b: &[u8], at: u32) -> Vec3 {
    Vec3 { x: bef(b, at), y: bef(b, at + 4), z: bef(b, at + 8) }
}

/// One `HSD_Joint` descriptor (DAT offsets; 0 = NULL).
#[derive(Clone, Copy, Debug)]
pub struct JointDesc {
    pub flags: u32,
    pub child: u32,
    pub next: u32,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub position: Vec3,
}

impl JointDesc {
    pub fn read(d: &[u8], at: u32) -> Self {
        JointDesc {
            flags: be32(d, at + 4),
            child: be32(d, at + 8),
            next: be32(d, at + 0xC),
            rotation: vec3(d, at + 0x14),
            scale: vec3(d, at + 0x20),
            position: vec3(d, at + 0x2C),
        }
    }
}

/// `FighterBone` (ft/types.h, 0x10): the bits this port reads.
#[derive(Clone, Copy, Debug)]
pub struct Bone {
    pub joint: JObjId,
    /// `xC`: tree depth.
    pub depth: u32,
    /// `flags_b0` (dynamics) and `flags_b5`: both clear here.
    pub b0: bool,
    pub b5: bool,
}

/// A loaded figatree (`HSD_FigaTree`): `nodes` (track count per part, -1 ended) and the tracks.
#[derive(Clone, Debug)]
pub struct FigaTree {
    pub type_: i32,
    pub flags: u32,
    pub frames: f32,
    pub nodes: Vec<i8>,
    pub tracks: Vec<FigaTrack>,
}

#[derive(Clone, Debug)]
pub struct FigaTrack {
    pub length: u16,
    pub startframe: u16,
    pub obj_type: u8,
    pub frac_value: u8,
    pub frac_slope: u8,
    pub ad_head: Arc<[u8]>,
}

impl FigaTree {
    /// The AJ row's DAT: its root is the `FigaTree` (`ftData_80085CD8`).
    pub fn parse(d: &[u8], root: u32) -> Result<Self, String> {
        let nodes_at = be32(d, root + 0xC);
        let tracks_at = be32(d, root + 0x10);
        let mut nodes = Vec::new();
        let mut n_tracks = 0usize;
        let mut at = nodes_at as usize;
        loop {
            let v = *d.get(at).ok_or("figatree: nodes past the end")? as i8;
            if v == -1 {
                break;
            }
            nodes.push(v);
            n_tracks += v as u8 as usize;
            at += 1;
        }
        let mut tracks = Vec::with_capacity(n_tracks);
        for k in 0..n_tracks as u32 {
            let t = tracks_at + k * 0xC;
            let ad = be32(d, t + 8) as usize;
            tracks.push(FigaTrack {
                length: be16(d, t),
                startframe: be16(d, t + 2),
                obj_type: d[t as usize + 4],
                frac_value: d[t as usize + 5],
                frac_slope: d[t as usize + 6],
                ad_head: Arc::from(&d[ad..]),
            });
        }
        Ok(FigaTree { type_: be32(d, root) as i32, flags: be32(d, root + 4), frames: bef(d, root + 8), nodes, tracks })
    }
}

/// The matrix host: the fighter skeletons checked here carry no RObj / IK joints.
struct NoRObj;
impl JObjHost<(), ()> for NoRObj {
    fn ik_hint_bone_length(&mut self, _: &[()]) -> Option<f32> {
        None
    }
    fn robj_update_all(&mut self, _: &mut Tree, _: JObjId) {}
    fn resolve_ik_joint1(&mut self, _: &mut Tree, _: JObjId) {
        panic!("resolveIKJoint1 on a fighter joint: not wired (TODO group1 robj)")
    }
    fn resolve_ik_joint2(&mut self, _: &mut Tree, _: JObjId) {
        panic!("resolveIKJoint2 on a fighter joint: not wired (TODO group1 robj)")
    }
    fn unref(&mut self, _: &mut Tree, _: JObjId) {}
}

pub struct Skeleton {
    pub tree: Tree,
    pub root: JObjId,
    pub parts: Vec<Bone>,
    /// The costume joint descriptors in `ftAnim_GetNextJointInTree` order (DFS, no INSTANCE check).
    joints: Vec<JointDesc>,
    /// `ftData->x8->x10` (the part `ftCommon_8007F6A4` rescales).
    x8_x10: usize,
    /// `1 / ftData->x0->model_scaling`.
    inv_model_scaling: f32,
}

impl Skeleton {
    /// `HSD_JObjLoadJoint(costume_joint)` + `ftParts_SetupParts`. `parts_num` checks the walk
    /// (`ftPartsTable[kind]->parts_num`; the kinds wired have no `Fighter_804D6540` skip list).
    pub fn load(d: &[u8], root_joint: u32, x8_x10: usize, model_scaling: f32, parts_num: u32) -> Result<Self, String> {
        let mut tree = Tree::new();
        let root = load_joint_sub(&mut tree, d, root_joint, None)?.ok_or("costume joint is NULL")?;
        // ftParts_SetupParts.
        let mut parts = Vec::new();
        let mut j = Some(root);
        let mut depth = 0u32;
        while let Some(x) = j {
            parts.push(Bone { joint: x, depth, b0: false, b5: false });
            let o = tree.get(x);
            if o.flags & hj::JOBJ_INSTANCE == 0 && o.child.is_some() {
                j = o.child;
                depth += 1;
                continue;
            }
            if o.next.is_some() {
                j = o.next;
                continue;
            }
            let mut y = x;
            j = loop {
                let Some(p) = tree.get(y).parent else { break None };
                if let Some(n) = tree.get(p).next {
                    depth -= 1;
                    break Some(n);
                }
                y = p;
                depth -= 1;
            };
        }
        if parts.len() as u32 != parts_num {
            return Err(format!("fighter parts num not match ({} joints, table {parts_num})", parts.len()));
        }
        // ftAnim_GetNextJointInTree order over the descriptors.
        let mut joints = Vec::new();
        let mut stack = vec![root_joint];
        while let Some(at) = stack.pop() {
            if at == 0 {
                continue;
            }
            let jd = JointDesc::read(d, at);
            joints.push(jd);
            stack.push(jd.next);
            stack.push(jd.child);
        }
        Ok(Skeleton { tree, root, parts, joints, x8_x10, inv_model_scaling: 1.0 / model_scaling })
    }

    pub fn part(&self, i: usize) -> JObjId {
        self.parts[i].joint
    }

    /// `HSD_JObjSetTranslate`.
    pub fn set_translate(&mut self, j: JObjId, t: Vec3) {
        let o = self.tree.get_mut(j);
        o.translate = t;
        if o.flags & hj::JOBJ_MTX_INDEP_SRT == 0 {
            self.tree.set_mtx_dirty(Some(j));
        }
    }

    /// `HSD_JObjSetScale`.
    pub fn set_scale(&mut self, j: JObjId, s: Vec3) {
        let o = self.tree.get_mut(j);
        o.scale = s;
        if o.flags & hj::JOBJ_MTX_INDEP_SRT == 0 {
            self.tree.set_mtx_dirty(Some(j));
        }
    }

    /// `ftPartSetRotX/Y/Z(fp, part, v)` on a non-quaternion joint (`HSD_JObjSetRotationX/Y/Z`).
    pub fn set_part_rot(&mut self, part: usize, axis: u8, v: f32) {
        let j = self.part(part);
        let o = self.tree.get_mut(j);
        assert!(o.flags & hj::JOBJ_USE_QUATERNION == 0, "ftPartSetRot on a quaternion joint: x4_jobj2 path not ported");
        match axis {
            0 => o.rotate.x = v,
            1 => o.rotate.y = v,
            _ => o.rotate.z = v,
        }
        if o.flags & hj::JOBJ_MTX_INDEP_SRT == 0 {
            self.tree.set_mtx_dirty(Some(j));
        }
    }

    /// `ftAnim_8006EBE8(gobj, anim_start, anim_rate, 0)` (no blend, same-kind figatree).
    pub fn attach(&mut self, tree: &FigaTree, anim_start: f32, anim_rate: f32, looping: bool) {
        // ftAnim_80070758(root) (and the anim skeleton's: no blend skeleton here).
        self.remove_anim_all();
        // ftAnim_8006FA58(fp, FtPart_TransN, costume_joint->child).
        let joints = std::mem::take(&mut self.joints);
        for (k, jd) in joints.iter().skip(1).enumerate() {
            let i = 1 + k;
            let b = self.parts[i];
            if i == self.x8_x10 {
                if b.b0 {
                    unreachable!("lb_8000B760 on a dynamics part");
                } else if !b.b5 {
                    lb_8000b5dc(&mut self.tree, b.joint, jd);
                }
                // ftCommon_8007F6A4: HSD_JObjSetScale(1 / model_scaling).
                let v = self.inv_model_scaling;
                self.set_scale(b.joint, Vec3 { x: v, y: v, z: v });
            } else if b.b0 {
                unreachable!("lb_8000B6A4 on a dynamics part");
            } else if !b.b5 {
                lb_8000b4fc(&mut self.tree, b.joint, jd);
            }
        }
        self.joints = joints;
        // ftAnim_8006FE08 -> ftAnim_8006F4C8(fp, false, x590).
        let mut i = 0usize;
        let mut track = 0usize;
        for &n in &tree.nodes {
            let b = self.parts[i];
            if !b.b0 && !b.b5 {
                lbanim_8001e6d8(&mut self.tree, b.joint, tree, &tree.tracks[track..], n);
            }
            i += 1;
            track += n as u8 as usize;
        }
        // ftAnim_80070710: HSD_JObjReqAnimAllByFlags(root, 1, anim_start); LOOP; rate.
        for o in &mut self.tree.nodes {
            if let Some(a) = &mut o.aobj {
                a.req_anim(anim_start);
                if looping {
                    a.set_flags(AOBJ_LOOP);
                }
                a.set_rate(anim_rate);
            }
        }
    }

    /// `HSD_JObjRemoveAnimAllByFlags(root, 1)`: every JObj's AObj (the tree has no INSTANCE).
    pub fn remove_anim_all(&mut self) {
        for o in &mut self.tree.nodes {
            o.aobj = None;
        }
    }

    /// The no-blend half of `ftAnim_8006E9B4`: `HSD_JObjClearFlagsAll(root, QUATERNION)` then
    /// `ftAnim_8006E7B8(fp, FtPart_TopN)` (every part's `HSD_JObjAnim`). Err: an animation event
    /// (particle / sound / user callback key) that is not wired.
    pub fn anim(&mut self, ag: &mut AObjGlobals) -> Result<(), String> {
        self.tree.clear_flags_all(Some(self.root), hj::JOBJ_USE_QUATERNION);
        ag.init_end_callback();
        for i in 0..self.parts.len() {
            let b = self.parts[i];
            if !b.b0 && !b.b5 {
                jobj_anim(&mut self.tree, b.joint, ag)?;
            }
        }
        ag.invoke_callbacks();
        Ok(())
    }

    /// `ftAnim_8006F3DC`, no blend: the first animated part's AObj frame.
    pub fn curr_frame(&self) -> Option<f32> {
        self.parts.iter().filter(|b| !b.b0 && !b.b5).find_map(|b| self.tree.get(b.joint).aobj.as_ref().map(|a| a.curr_frame))
    }

    /// `ftAnim_IsFramesRemaining`, no blend: some animated part's AObj still runs
    /// (`lb_8000B074`: an AObj without AOBJ_NO_ANIM).
    pub fn frames_remaining(&self) -> bool {
        self.parts.iter().filter(|b| !b.b0 && !b.b5).any(|b| {
            self.tree.get(b.joint).aobj.as_ref().is_some_and(|a| a.flags & melee_runtime::aobj::AOBJ_NO_ANIM == 0)
        })
    }

    /// `lb_8000B1CC(jobj, NULL, &out)`.
    pub fn world_pos(&mut self, j: JObjId) -> Vec3 {
        if self.tree.get(j).parent.is_some() {
            self.tree.setup_matrix(&mut NoRObj, Some(j));
            let m = self.tree.get(j).mtx;
            Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] }
        } else {
            self.tree.get(j).translate
        }
    }
}

/// `JObjLoadJointSub` + `JObjLoad`: child first, then next; flags |= desc flags.
fn load_joint_sub(tree: &mut Tree, d: &[u8], at: u32, parent: Option<JObjId>) -> Result<Option<JObjId>, String> {
    if at == 0 {
        return Ok(None);
    }
    let jd = JointDesc::read(d, at);
    if be32(d, at + 0x3C) != 0 {
        return Err(format!("joint @{at:#x} has an RObj descriptor: not wired"));
    }
    let j = tree.alloc();
    if jd.flags & hj::JOBJ_INSTANCE == 0 {
        let c = load_joint_sub(tree, d, jd.child, Some(j))?;
        tree.get_mut(j).child = c;
    }
    let n = load_joint_sub(tree, d, jd.next, parent)?;
    let o = tree.get_mut(j);
    o.next = n;
    o.parent = parent;
    o.flags |= jd.flags;
    o.rotate.x = jd.rotation.x;
    o.rotate.y = jd.rotation.y;
    o.rotate.z = jd.rotation.z;
    o.scale = jd.scale;
    o.translate = jd.position;
    o.mtx = melee_runtime::mtx::identity();
    o.scl = None;
    Ok(Some(j))
}

/// `lb_8000B4FC`: rotation, scale and translation from the descriptor.
fn lb_8000b4fc(t: &mut Tree, j: JObjId, jd: &JointDesc) {
    let o = t.get_mut(j);
    o.rotate.x = jd.rotation.x;
    o.rotate.y = jd.rotation.y;
    o.rotate.z = jd.rotation.z;
    o.scale = jd.scale;
    o.translate = jd.position;
    t.clear_flags(Some(j), hj::JOBJ_USE_QUATERNION);
    if t.get(j).flags & hj::JOBJ_MTX_INDEP_SRT == 0 {
        t.set_mtx_dirty(Some(j));
    }
}

/// `lb_8000B5DC`: rotation and translation only.
fn lb_8000b5dc(t: &mut Tree, j: JObjId, jd: &JointDesc) {
    let o = t.get_mut(j);
    o.rotate.x = jd.rotation.x;
    o.rotate.y = jd.rotation.y;
    o.rotate.z = jd.rotation.z;
    o.translate = jd.position;
    t.clear_flags(Some(j), hj::JOBJ_USE_QUATERNION);
    if t.get(j).flags & hj::JOBJ_MTX_INDEP_SRT == 0 {
        t.set_mtx_dirty(Some(j));
    }
}

/// `lbAnim_8001E6D8` (`lbAnim_InitFrames` + `lbAnim_JObjSortAnim`).
fn lbanim_8001e6d8(t: &mut Tree, j: JObjId, tree: &FigaTree, tracks: &[FigaTrack], frames: i8) {
    if frames == 0 {
        return;
    }
    let mut a: HsdAObj<JObjId> = HsdAObj::alloc();
    a.set_flags(tree.flags);
    a.set_rewind_frame(0.0);
    a.set_end_frame(tree.frames);
    let fobj = tracks[..frames as u8 as usize]
        .iter()
        .map(|tr| HsdFObj {
            ad: 0,
            ad_head: tr.ad_head.clone(),
            length: tr.length as u32,
            flags: 0,
            op: 0,
            op_intrp: 0,
            obj_type: tr.obj_type,
            frac_value: tr.frac_value,
            frac_slope: tr.frac_slope,
            nb_pack: 0,
            startframe: tr.startframe as i16,
            fterm: 0,
            time: 0.0,
            p0: 0.0,
            p1: 0.0,
            d0: 0.0,
            d1: 0.0,
        })
        .collect();
    a.set_fobj(fobj);
    jobj_rt::sort_anim(Some(&mut a));
    t.get_mut(j).aobj = Some(a);
    if tree.type_ & 1 != 0 {
        t.set_flags(Some(j), hj::JOBJ_CLASSICAL_SCALE);
    } else {
        t.clear_flags(Some(j), hj::JOBJ_CLASSICAL_SCALE);
    }
}

/// `HSD_JObjAnim` (no RObj chain; DObj material animation is render-only and not run).
fn jobj_anim(t: &mut Tree, j: JObjId, ag: &mut AObjGlobals) -> Result<(), String> {
    t.check_depend(Some(j));
    let Some(mut a) = t.get_mut(j).aobj.take() else { return Ok(()) };
    let mut events: Vec<UfcEvent> = Vec::new();
    {
        let mut f = |ty: u8, fv: f32| {
            let mut v = ObjData::from_f(fv);
            jobj_rt::update_func_core(t, Some(j), ty as u32, Some(&mut v), &mut |_, _| panic!("PATH track on a fighter joint"), &mut events);
        };
        a.interpret_anim(ag, &mut Some(&mut f));
    }
    t.get_mut(j).aobj = Some(a);
    if let Some(e) = events.first() {
        return Err(format!("fighter animation event {e:?}: not wired"));
    }
    Ok(())
}

//! Fighter dynamic bones (Fox's tail, scarves, hair) for the native match's `Skeleton`.
//!
//! `ftCo_8009CF84` / `ftCo_8009CB40` (ftdynamics.c: set up the chains from `ftData->x2C`),
//! `lb_8000FD48` / `lb_80011710` (lb_00F9.c: build the node list, load the per-node constants),
//! `ftCo_8009DD94` (the per-frame driver, `Fighter_8006D9AC` at s_link 0x10), `ftColl_8007AF60`
//! (the body colliders' world positions) and `lb_8001044C` (the chain step). The ports of these in
//! melee_runtime / melee_fighter work on raw `HSD_JObj` pointers; the match's skeleton is a
//! `JObjTree`, so this is the same code over the tree (each line transcribed from
//! `melee_runtime::tu::lb_00F9::lb_8001044C`, which is checked against the C), with the pool
//! `DynamicsData` replaced by a `Vec<Node>` per chain.
//!
//! Not modelled: the wind / force effect list (`lb_804D63B0`, empty outside the aesthetic wind
//! effects), Flatzone's scale tweak, `ftCo_8009E614` (Marth's / Roy's hair under that effect list).

use crate::fighter_data::skeleton::Skeleton;
use melee_core::fp::{fmadds, fmsubs};
use melee_runtime::gekko::{sinf, sqrtf};
use melee_runtime::jobj::JObjId;
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector as lv;
use melee_runtime::mtx::{self, Mtx, Quaternion, Vec3};
use melee_runtime::quatlib;

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
}
fn bef(b: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(b, at))
}
fn bev(b: &[u8], at: u32) -> Vec3 {
    Vec3 { x: bef(b, at), y: bef(b, at + 4), z: bef(b, at + 8) }
}
fn beq(b: &[u8], at: u32) -> Quaternion {
    Quaternion { x: bef(b, at), y: bef(b, at + 4), z: bef(b, at + 8), w: bef(b, at + 12) }
}

const JOBJ_MTX_INDEP_SRT: u32 = 1 << 25;
const JOBJ_USE_QUATERNION: u32 = 1 << 17;

/// `struct DynamicsData` (`lb_00F9_UnkDesc0` half): one joint of a chain.
#[derive(Clone, Copy)]
struct Node {
    jobj: JObjId,
    translate: Vec3,
    scale: Vec3,
    /// `unk_2C`: the joint's world position.
    p2c: Vec3,
    /// `unk_38`: the angular velocity's axis.
    axis: Vec3,
    /// `unk_44`: the angular velocity.
    v44: f32,
    /// `unk_48`: the length to the next joint.
    len: f32,
    v4c: f32,
    v50: f32,
    /// `unk_54`: the dominant axis of the next joint's translate (-1 z, 0 x, 1 y).
    i54: i32,
    q58: Quaternion,
    v68: f32,
    v84: f32,
    v88: f32,
    v8c: f32,
}

/// One `BoneDynamicsDesc` (`fp->dynamic_bone_sets[i]`).
struct BoneSet {
    /// `ftData->x2C->ftDynamicBones->array[i].bone_id`: the first part of the chain.
    bone0: usize,
    /// `dynamic_bone_sets[i].bone_id` (the `part` argument of `lb_8001044C`): 0 while the chain is
    /// simulated, 0x100 (`ftCo_8009CB40` off) while the animation drives it.
    part: i32,
    /// `desc.pos`.
    pos: Vec3,
    nodes: Vec<Node>,
}

/// `fp->x1670[i]` (`Fighter_x1670_t`).
struct Collider {
    part: usize,
    v1: Vec3,
    radius: f32,
    pos: Vec3,
}

pub struct Dynamics {
    sets: Vec<BoneSet>,
    colliders: Vec<Collider>,
}

/// What `ftCo_8009DD94` reads of the fighter.
pub struct FrameIn {
    pub kind: i32,
    pub motion_id: i32,
    pub grounded: bool,
    pub cur_pos_y: f32,
    pub scale_y: f32,
    pub x2228_b1: bool,
    pub is_peach: bool,
    pub is_mewtwo: bool,
    pub is_koopa: bool,
    pub mewtwo_motion_range: bool,
}

impl Dynamics {
    /// `ftCo_8009CF84` over `ftData->x2C` of PlXx.dat (None: the fighter has no dynamics).
    pub fn load(d: &[u8], ft_data: u32, skel: &mut Skeleton) -> Option<Dynamics> {
        let x2c = be32(d, ft_data + 0x2C);
        if x2c == 0 {
            return None;
        }
        let num = be32(d, x2c) as usize;
        if num == 0 {
            return None;
        }
        let bones = be32(d, x2c + 4);
        let mut sets = Vec::new();
        for i in 0..num as u32 {
            let b = bones + i * 0x18;
            let bone_id = be32(d, b) as usize;
            let data = be32(d, b + 4);
            let count = be32(d, b + 8) as usize;
            let pos = bev(d, b + 0xC);
            // lb_8000FD48(parts[bone_id].joint, &set.dyn_desc, count).
            let mut nodes: Vec<Node> = Vec::new();
            let mut jobj = Some(skel.part(bone_id));
            while nodes.len() < count {
                let j = jobj.expect("dynamics chain shorter than its count");
                skel.tree.set_mtx_dirty(Some(j));
                let m = skel.mtx_of(j);
                let o = skel.tree.get(j);
                nodes.push(Node {
                    jobj: j,
                    translate: o.translate,
                    scale: o.scale,
                    p2c: Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] },
                    axis: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                    v44: 0.0,
                    len: 0.0,
                    v4c: 0.0,
                    v50: 0.0,
                    i54: 0,
                    q58: o.rotate,
                    v68: 0.0,
                    v84: 0.0,
                    v88: 0.0,
                    v8c: 0.0,
                });
                jobj = o.child;
            }
            for k in 0..nodes.len().saturating_sub(1) {
                let (a, bn) = (nodes[k], nodes[k + 1]);
                let dx = a.p2c.x - bn.p2c.x;
                let dy = a.p2c.y - bn.p2c.y;
                let dz = a.p2c.z - bn.p2c.z;
                nodes[k].len = sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)));
                let tx = bn.translate.x.abs();
                let ty = bn.translate.y.abs();
                let tz = bn.translate.z.abs();
                nodes[k].i54 = if tz > ty {
                    if tz > tx {
                        -1
                    } else {
                        0
                    }
                } else if ty > tx {
                    1
                } else {
                    0
                };
            }
            // ftCo_8009CB40(fp, i, 1, NULL): the chain's parts become dynamics-driven.
            for (k, n) in nodes.iter_mut().enumerate() {
                let part = bone_id + k;
                if !skel.parts[part].b0 {
                    // ftCo_8009CB40_inline
                    let m = skel.tree.get(n.jobj).mtx;
                    n.p2c = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
                    n.v44 = 0.0;
                    skel.set_translate(n.jobj, n.translate);
                    skel.set_scale(n.jobj, n.scale);
                }
                skel.parts[part].b0 = true;
                skel.tree.clear_flags(Some(n.jobj), 0x20000);
            }
            // lb_80011710(&src, &set.dyn_desc).
            for (k, n) in nodes.iter_mut().enumerate() {
                let a = data + k as u32 * 0x3C;
                n.v4c = bef(d, a);
                n.v50 = bef(d, a + 4);
                n.q58 = beq(d, a + 8);
                n.v68 = bef(d, a + 0x18);
                n.v84 = bef(d, a + 0x34);
                n.v88 = bef(d, a + 0x38);
                n.v8c = if n.len as f64 != 0.0 { pos.z / n.len } else { 0.0 };
            }
            sets.push(BoneSet { bone0: bone_id, part: 0, pos, nodes });
        }
        // ftColl_8007B320's collider table.
        let ncol = be32(d, x2c + 8) as usize;
        let tab = be32(d, x2c + 0xC);
        let colliders = (0..ncol as u32)
            .map(|i| {
                let a = tab + i * 0x14;
                Collider { part: be32(d, a) as usize, v1: bev(d, a + 4), radius: bef(d, a + 0x10), pos: Vec3::default() }
            })
            .collect();
        Some(Dynamics { sets, colliders })
    }

    /// `ftCo_8009CB40(fp, idx, on, NULL)`: the chain `idx` is simulated (`on`) or driven by the animation.
    pub fn cb40(&mut self, skel: &mut Skeleton, idx: usize, on: bool) {
        let Some(set) = self.sets.get_mut(idx) else { return };
        set.part = if on { 0 } else { 0x100 };
        for (k, n) in set.nodes.iter_mut().enumerate() {
            let part = set.bone0 + k;
            if on && !skel.parts[part].b0 {
                // ftCo_8009CB40_inline
                let m = skel.tree.get(n.jobj).mtx;
                n.p2c = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
                n.v44 = 0.0;
                skel.set_translate(n.jobj, n.translate);
                skel.set_scale(n.jobj, n.scale);
            }
            skel.parts[part].b0 = on;
            if on {
                skel.tree.clear_flags(Some(n.jobj), 0x20000);
            }
        }
    }

    /// `ftCo_8009E7B4(fp, blend_data)` for a fighter with an animation row (`anim_id != -1`) that is neither
    /// Kirby nor a Marth / Roy under wind effects: `b3` = `x594_b3` (the row's "no dynamics" flag), `b4` =
    /// `x594_b4` (per-chain dynamics animations: not ported). Jigglypuff only drives chain 0.
    pub fn e7b4(&mut self, skel: &mut Skeleton, x2227_b6: bool, b3: bool, b4: bool, trees: bool, purin: bool) -> Result<(), String> {
        let n = if purin { 1.min(self.sets.len()) } else { self.sets.len() };
        if x2227_b6 {
            for i in 0..n {
                self.cb40(skel, i, false);
            }
            return Ok(());
        }
        if b4 {
            // x594_b4: every chain gets `tree[j]` as `ftCo_8009CB40`'s FigaTree (a guest address, i.e.
            // negative as the `(s32)arg3` it is compared with, or NULL): in both cases the chain is
            // simulated from node 0 (`part` below 0 / 0 runs the same), so `on`. Without the table or
            // its entry for the row's slot, the chains are off.
            for i in 0..self.sets.len() {
                self.cb40(skel, i, trees);
            }
            return Ok(());
        }
        for i in 0..n {
            self.cb40(skel, i, !b3);
        }
        Ok(())
    }

    /// `ftCo_8009DD94(gobj, arg1)`. `floor(ax, ay, bx, by, y_offset)` is `mpCheckFloor(.., -1, -1,
    /// -1, NULL, NULL)`'s hit point.
    pub fn step(&mut self, skel: &mut Skeleton, f: &FrameIn, floor: &mut dyn FnMut(f32, f32, f32, f32, f32) -> Option<Vec3>) {
        let ret_b0 = if f.scale_y < 1.0 || f.scale_y > 1.0 {
            false
        } else if f.is_mewtwo && f.mewtwo_motion_range {
            true
        } else {
            !(!f.grounded || f.is_koopa)
        };
        // ftColl_8007AF60.
        for c in &mut self.colliders {
            let j = skel.part(c.part);
            c.pos = skel.world_point(j, c.v1);
        }
        if std::env::var_os("MELEE_GOLDEN_DN").is_some() {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let dn = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if dn <= 3 {
                for (cc, c) in self.colliders.iter().enumerate() {
                    eprintln!("DC n{} c{} {:08X} {:08X} {:08X} r {:08X}", dn, cc, c.pos.x.to_bits(), c.pos.y.to_bits(), c.pos.z.to_bits(), c.radius.to_bits());
                }
                for (ii, set) in self.sets.iter().enumerate() {
                    for (kk, nd) in set.nodes.iter().enumerate() {
                        let r = skel.tree.get(nd.jobj).rotate;
                        eprintln!("DN n{} s{} k{} r {:08X} {:08X} {:08X} p2c {:08X} {:08X} {:08X} v44 {:08X} l48 {:08X} 4c {:08X} 50 {:08X} 54 {} 8c {:08X} 84 {:08X} 88 {:08X} 68 {:08X} pos {:08X}", dn, ii, kk,
                            r.x.to_bits(), r.y.to_bits(), r.z.to_bits(), nd.p2c.x.to_bits(), nd.p2c.y.to_bits(), nd.p2c.z.to_bits(), nd.v44.to_bits(), nd.len.to_bits(), nd.v4c.to_bits(), nd.v50.to_bits(), nd.i54, nd.v8c.to_bits(), nd.v84.to_bits(), nd.v88.to_bits(), nd.v68.to_bits(), set.pos.x.to_bits());
                    }
                }
            }
        }
        let n = self.sets.len();
        for i in 0..n {
            let mode = if f.is_peach {
                if i + 1 < n {
                    3
                } else {
                    0
                }
            } else if f.is_mewtwo {
                8
            } else {
                0
            };
            let set = &mut self.sets[i];
            lb_8001044c(skel, set, &self.colliders, f.cur_pos_y, f.x2228_b1, set.part, mode, ret_b0, floor);
        }
    }
}

/// `lb_800103D8`.
fn lb_800103d8(x0: f32, x1: f32, x2: f32, x3: f32, offset: f32) -> Option<Vec3> {
    let dist0 = x1 - offset;
    let dist1 = x3 - offset;
    if dist0 == dist1 {
        return if dist0 > 1e-10f32 { Some(Vec3::default()) } else { None };
    }
    if dist0 as f64 > 0.0 && (dist1 as f64) < 0.0 {
        return Some(Vec3 { x: fmadds(-dist1 / (dist0 - dist1), x0 - x2, x2), y: offset, z: 0.0 });
    }
    None
}

fn approx_zero(v: &Vec3) -> bool {
    !(v.x >= 0.00001 || v.x <= -0.00001 || v.y >= 0.00001 || v.y <= -0.00001 || v.z >= 0.00001 || v.z <= -0.00001)
}

fn quat3(v: &Vec3) -> Quaternion {
    Quaternion { x: v.x, y: v.y, z: v.z, w: 0.0 }
}

fn set_rot_axis(skel: &mut Skeleton, j: JObjId, axis: i32, v: f32) {
    let o = skel.tree.get_mut(j);
    match axis {
        0 => o.rotate.x = v,
        1 => o.rotate.y = v,
        _ => o.rotate.z = v,
    }
    if o.flags & JOBJ_MTX_INDEP_SRT == 0 {
        skel.tree.set_mtx_dirty(Some(j));
    }
}

/// `lb_8001044C(desc, colliders, num_colliders, pos_y, use_floor_fn, part, first_active, ground_check)`.
#[allow(clippy::too_many_arguments)]
fn lb_8001044c(
    skel: &mut Skeleton,
    set: &mut BoneSet,
    colliders: &[Collider],
    pos_y: f32,
    use_floor_fn: bool,
    part: i32,
    first_active: i32,
    ground_check: bool,
    floor: &mut dyn FnMut(f32, f32, f32, f32, f32) -> Option<Vec3>,
) {
    if part > 0xFF || set.nodes.is_empty() {
        return;
    }
    let mut cur = 0usize;
    let mut loop_index: i32 = 0;
    for _ in 0..part {
        cur += 1;
        loop_index += 1;
    }
    let pos_x = set.pos.x;
    let mut on_ground = false;
    let mut angle_quat = Quaternion::default();
    let mut euler_quat;
    let mut result_quat;

    let parent = skel.tree.get(set.nodes[cur].jobj).parent;
    let mut parent_mtx: Mtx = match parent {
        Some(p) => {
            skel.tree.set_mtx_dirty(Some(p));
            skel.mtx_of(p)
        }
        None => mtx::identity(),
    };

    let mut jobj = set.nodes[cur].jobj;
    let last = set.nodes.len() - 1;

    while cur < last {
        let nx_p2c = set.nodes[cur + 1].p2c;
        let (jt, jr, js, jchild) = {
            let o = skel.tree.get(jobj);
            (o.translate, o.rotate, o.scale, o.child.expect("dynamics joint has no child"))
        };
        let child_t = skel.tree.get(jchild).translate;
        let d = &mut set.nodes[cur];

        let trans_mtx = mtx::trans(jt.x, jt.y, jt.z);
        let mut bone_mtx = mtx::concat(&parent_mtx, &trans_mtx);

        let temp = lv::create_euler_matrix(&d.q58);
        let mut constrained_mtx = mtx::concat(&bone_mtx, &temp);

        let temp = lv::create_euler_matrix(&jr);
        bone_mtx = mtx::concat(&bone_mtx, &temp);

        let scale_mtx = mtx::scale(js.x, js.y, js.z);
        bone_mtx = mtx::concat(&bone_mtx, &scale_mtx);
        constrained_mtx = mtx::concat(&constrained_mtx, &scale_mtx);

        d.p2c = Vec3 { x: bone_mtx[0][3], y: bone_mtx[1][3], z: bone_mtx[2][3] };

        let mut natural_dir = mtx::mult_vec(&constrained_mtx, &child_t);
        natural_dir.x -= d.p2c.x;
        natural_dir.y -= d.p2c.y;
        natural_dir.z -= d.p2c.z;

        let mut current_dir = mtx::mult_vec(&bone_mtx, &child_t);
        current_dir.x -= d.p2c.x;
        current_dir.y -= d.p2c.y;
        current_dir.z -= d.p2c.z;

        let mut link_dir = Vec3 { x: nx_p2c.x - d.p2c.x, y: nx_p2c.y - d.p2c.y, z: nx_p2c.z - d.p2c.z };

        lv::normalize(&mut natural_dir);
        lv::normalize(&mut current_dir);
        lv::normalize(&mut link_dir);

        let mut saved_dir = link_dir;

        // stiffness blend
        if d.v4c * pos_x < 1.0 {
            let stiff_angle = lv::angle(&link_dir, &current_dir).abs();
            if stiff_angle != 0.0 {
                let mut axis = mtx::vec_cross(&link_dir, &current_dir);
                lv::normalize(&mut axis);
                lv::rotate_about_unit_axis(&mut link_dir, &axis, stiff_angle * ((1.0f64 - (d.v4c * pos_x) as f64) as f32));
            }
        }

        // gravity
        {
            let grav_dir = Vec3 { x: 0.0, y: -1.0, z: 0.0 };
            let grav_angle = lv::angle(&link_dir, &grav_dir);
            if grav_angle != 0.0 {
                let mut grav_rot = d.v8c * sinf(grav_angle);
                if grav_rot < 0.0 {
                    grav_rot = -grav_rot;
                }
                let mut axis = mtx::vec_cross(&link_dir, &grav_dir);
                lv::normalize(&mut axis);
                lv::rotate_about_unit_axis(&mut link_dir, &axis, grav_rot);
            }
        }

        // interaction force: the force list (lb_804D63B0) is empty.
        let force_mag = 0.0f32;

        // angular velocity rotation
        if 0.0 != d.v44 {
            lv::rotate_about_unit_axis(&mut link_dir, &d.axis, d.v44);
        }

        // max angle constraint
        if force_mag < d.v8c * d.len {
            let mut clamp_dir = saved_dir;
            if lv::angle(&clamp_dir, &link_dir) > d.v88 {
                let mut axis = mtx::vec_cross(&clamp_dir, &link_dir);
                lv::normalize(&mut axis);
                lv::rotate_about_unit_axis(&mut clamp_dir, &axis, d.v88);
                link_dir = clamp_dir;
            }
        }

        // convergence limit
        if d.v50 > 0.0 {
            if lv::angle(&natural_dir, &link_dir) < d.v50 {
                link_dir = natural_dir;
            } else {
                let mut axis = mtx::vec_cross(&link_dir, &natural_dir);
                lv::normalize(&mut axis);
                lv::rotate_about_unit_axis(&mut link_dir, &axis, d.v50);
            }
        }

        // max angle deviation
        {
            let dev_angle = lv::angle(&natural_dir, &link_dir);
            if dev_angle > d.v68 {
                let mut axis = mtx::vec_cross(&link_dir, &natural_dir);
                lv::normalize(&mut axis);
                lv::rotate_about_unit_axis(&mut link_dir, &axis, dev_angle - d.v68);
            }
        }

        // collider avoidance
        if !colliders.is_empty() {
            lv::normalize(&mut link_dir);
            for c in colliders {
                let next_bone_pos = Vec3 {
                    x: fmadds(link_dir.x, d.len, d.p2c.x),
                    y: fmadds(link_dir.y, d.len, d.p2c.y),
                    z: fmadds(link_dir.z, d.len, d.p2c.z),
                };
                let mut coll_dir = Vec3 { x: c.pos.x - d.p2c.x, y: c.pos.y - d.p2c.y, z: c.pos.z - d.p2c.z };
                let coll_dist = sqrtf(fmadds(coll_dir.z, coll_dir.z, fmadds(coll_dir.x, coll_dir.x, coll_dir.y * coll_dir.y)));
                let mut collision_point = Vec3::default();
                if coll_dist > c.radius
                    && unsafe {
                        melee_runtime::tu::lbcollision::lbColl_80005C44(&d.p2c as *const Vec3 as *const _, &next_bone_pos as *const Vec3 as *const _, &c.pos as *const Vec3 as *const _, &mut collision_point as *mut Vec3 as *mut _, 0.1, c.radius)
                    }
                {
                    let coll_angle = lv::angle(&coll_dir, &link_dir);
                    if 0.0 != coll_angle {
                        let adj_radius = (0.1f64 + c.radius as f64) as f32;
                        let side_sq = sqrtf(fmsubs(coll_dist, coll_dist, adj_radius * adj_radius));
                        let mut avoidance_angle = atan2f(adj_radius, side_sq);
                        if avoidance_angle < 0.0 {
                            avoidance_angle = -avoidance_angle;
                        }
                        avoidance_angle -= coll_angle;
                        if avoidance_angle > 0.0 {
                            let mut axis = mtx::vec_cross(&coll_dir, &link_dir);
                            lv::normalize(&mut axis);
                            lv::rotate_about_unit_axis(&mut link_dir, &axis, avoidance_angle);
                        }
                    }
                }
                let _ = &mut coll_dir;
            }
        }

        // ground collision
        if ground_check {
            if on_ground {
                let mut gnd_norm = Vec3 { x: 0.0, y: -1.0, z: 0.0 };
                lv::normalize(&mut link_dir);
                let end_x = fmadds(link_dir.x, d.len, d.p2c.x);
                let end_y = fmadds(link_dir.y, d.len, d.p2c.y);
                let floor_hit = if use_floor_fn {
                    lb_800103d8(end_x, (1.0f64 + end_y as f64) as f32, end_x, (end_y as f64 - 1.0) as f32, pos_y)
                } else {
                    floor(end_x, (1.0f64 + end_y as f64) as f32, end_x, (end_y as f64 - 1.0) as f32, 0.1)
                };
                let mut floor_cross = mtx::vec_cross(&gnd_norm, &link_dir);
                lv::normalize(&mut floor_cross);
                lv::rotate_about_unit_axis(&mut gnd_norm, &floor_cross, 1.5707964);
                link_dir = gnd_norm;
                on_ground = floor_hit.is_some();
            } else {
                let mut gnd_norm2 = Vec3 { x: 0.0, y: -1.0, z: 0.0 };
                lv::normalize(&mut link_dir);
                let floor_hit2 = if use_floor_fn {
                    lb_800103d8(d.p2c.x, (1.0f64 + d.p2c.y as f64) as f32, d.p2c.x, d.p2c.y, pos_y)
                } else {
                    floor(d.p2c.x, (1.0f64 + d.p2c.y as f64) as f32, d.p2c.x, d.p2c.y, 0.1)
                };
                if floor_hit2.is_some() {
                    let mut floor_cross = mtx::vec_cross(&gnd_norm2, &link_dir);
                    lv::normalize(&mut floor_cross);
                    lv::rotate_about_unit_axis(&mut gnd_norm2, &floor_cross, 1.5707964);
                    on_ground = true;
                    link_dir = gnd_norm2;
                } else {
                    let end_x2 = fmadds(link_dir.x, d.len, d.p2c.x);
                    let end_y2 = fmadds(link_dir.y, d.len, d.p2c.y);
                    let floor_hit3 = if use_floor_fn {
                        lb_800103d8(d.p2c.x, d.p2c.y, end_x2, end_y2, pos_y)
                    } else {
                        floor(d.p2c.x, d.p2c.y, end_x2, end_y2, 0.1)
                    };
                    if let Some(floor_point) = floor_hit3 {
                        let height_diff = (d.p2c.y - floor_point.y).abs();
                        let horiz_dist = sqrtf(-fmsubs(height_diff, height_diff, d.len * d.len));
                        let ground_angle = atan2f(horiz_dist, height_diff).abs();
                        let mut floor_cross = mtx::vec_cross(&gnd_norm2, &link_dir);
                        lv::normalize(&mut floor_cross);
                        lv::rotate_about_unit_axis(&mut gnd_norm2, &floor_cross, ground_angle);
                        on_ground = true;
                        link_dir = gnd_norm2;
                    } else {
                        on_ground = false;
                    }
                }
            }
        }

        // angular difference and rotation axis
        let angle_diff = lv::angle(&current_dir, &link_dir);
        let mut rotation_axis = mtx::vec_cross(&current_dir, &link_dir);
        lv::normalize(&mut rotation_axis);

        let mut adj_dir = current_dir;
        lv::rotate_about_unit_axis(&mut adj_dir, &rotation_axis, angle_diff);
        link_dir = adj_dir;

        // store angular velocity
        if on_ground {
            d.v44 = 0.0;
        } else {
            d.v44 = lv::angle(&saved_dir, &link_dir);
        }
        {
            let mut ang_vel = d.v44;
            if ang_vel < 0.0 {
                ang_vel = -ang_vel;
            }
            if ang_vel > 0.0 {
                d.axis = mtx::vec_cross(&saved_dir, &link_dir);
                lv::normalize(&mut d.axis);
            }
        }
        {
            let ang_vel2 = d.v44;
            if 0.0 != ang_vel2 {
                let damping = d.v84;
                if ang_vel2 > damping {
                    d.v44 = ang_vel2 - damping;
                } else if ang_vel2 < -damping {
                    d.v44 = ang_vel2 + damping;
                } else {
                    d.v44 = 0.0;
                }
            }
        }
        saved_dir = link_dir;
        let _ = saved_dir;

        // update JObj rotation via quaternion
        if !(angle_diff < 0.00001 && angle_diff > -0.00001) {
            bone_mtx = mtx::transpose(&parent_mtx);
            let local_axis = mtx::mult_vec(&bone_mtx, &rotation_axis);
            if !approx_zero(&local_axis) {
                let mut rotation = Quaternion::default();
                if let Some(q) = quatlib::from_axis_angle(&local_axis, angle_diff) {
                    angle_quat = q;
                }
                let o = skel.tree.get(jobj);
                let euler_angles = Vec3 { x: o.rotate.x, y: o.rotate.y, z: o.rotate.z };
                euler_quat = quatlib::euler_to_quat(&euler_angles);
                result_quat = quatlib::mul(&angle_quat, &euler_quat);
                bone_mtx = mtx::quat(&result_quat);
                let e = quatlib::mtx_to_euler(&bone_mtx);
                rotation.x = e.x;
                rotation.y = e.y;
                rotation.z = e.z;
                // HSD_JObjSetRotation
                skel.tree.get_mut(jobj).rotate = rotation;
                if skel.tree.get(jobj).flags & JOBJ_MTX_INDEP_SRT == 0 {
                    skel.tree.set_mtx_dirty(Some(jobj));
                }
                skel.tree.clear_flags_all(Some(jobj), 0x20000);
            }
        }

        // dominant axis damping
        {
            let o = skel.tree.get(jobj).rotate;
            let axis = set.nodes[cur].i54;
            if axis == -1 {
                set_rot_axis(skel, jobj, 2, o.z * 0.9);
            } else if axis == 0 {
                set_rot_axis(skel, jobj, 0, o.x * 0.9);
            } else {
                set_rot_axis(skel, jobj, 1, o.y * 0.9);
            }
        }

        // update parent matrix for next iteration
        parent_mtx = mtx::concat(&parent_mtx, &trans_mtx);
        let jr2 = skel.tree.get(jobj).rotate;
        let temp = lv::create_euler_matrix(&jr2);
        parent_mtx = mtx::concat(&parent_mtx, &temp);
        parent_mtx = mtx::concat(&parent_mtx, &scale_mtx);

        jobj = jchild;
        loop_index += 1;
        cur += 1;
    }
    let _ = (loop_index, first_active, quat3);

    // tail node
    let (jt, jr, js) = {
        let o = skel.tree.get(jobj);
        (o.translate, o.rotate, o.scale)
    };
    let temp = mtx::trans(jt.x, jt.y, jt.z);
    parent_mtx = mtx::concat(&parent_mtx, &temp);
    let temp = lv::create_euler_matrix(&jr);
    parent_mtx = mtx::concat(&parent_mtx, &temp);
    let temp = mtx::scale(js.x, js.y, js.z);
    parent_mtx = mtx::concat(&parent_mtx, &temp);
    set.nodes[cur].p2c = Vec3 { x: parent_mtx[0][3], y: parent_mtx[1][3], z: parent_mtx[2][3] };
}

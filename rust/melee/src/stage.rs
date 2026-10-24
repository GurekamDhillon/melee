//! Stage spawn points read from the stage file on the user's disc (`GrXx.dat`, root `map_head`).
//!
//! The C fills `stage_info.x280[id]` (`Ground_801C3680`-style walk at src/melee/gr/ground.c
//! ~2640-2712): `map_head->unk0` is an array of `{ HSD_Joint* joint; s16* pairs; s32 count; }`
//! (12 bytes, GC), each pair `(dfs_index, point_id)` names the JObj reached by a depth-first walk of
//! that map GObj's tree (children skipped under `JOBJ_INSTANCE`). `Ground_801C2D24(id)` then returns
//! that JObj's world translation (`lb_8000B1CC`), and `Stage_80224E64(slot)` uses it for slot `slot`.
//!
//! Here the world translation is composed from the joint descriptors (T * Rz*Ry*Rx * S per level)
//! WITHOUT the JObj system or animation: exact (pure f32 adds) only when every ancestor has zero
//! rotation and unit scale, which `SpawnPoint::exact` reports. TODO(group1 jobj): use the ported
//! HSD_JObjSetupMatrix once JObjs are instantiated from disc data.

use melee_runtime::dat::DatArchive;

const JOBJ_INSTANCE: u32 = 1 << 12;

#[derive(Clone, Copy, Debug)]
pub struct SpawnPoint {
    pub id: i32,
    pub pos: [f32; 3],
    /// Every joint on the path had zero rotation and unit scale (no float rounding beyond the C's).
    pub exact: bool,
}

fn be32(b: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(b[at as usize..at as usize + 4].try_into().unwrap())
}
fn bef(b: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(b, at))
}

/// One HSD_Joint level: (rotation, scale, translation).
fn srt(d: &[u8], j: u32) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let v = |o: u32| [bef(d, j + o), bef(d, j + o + 4), bef(d, j + o + 8)];
    (v(0x14), v(0x20), v(0x2C))
}

/// All points of the stage file's `map_head` table.
pub fn spawn_points(dat: &[u8]) -> Result<Vec<SpawnPoint>, String> {
    let a = DatArchive::parse(dat).map_err(|e| format!("stage dat: {e}"))?;
    let head = a.public_offset("map_head").ok_or("stage dat: no map_head")?;
    let d = &a.data;
    let (table, count) = (be32(d, head), be32(d, head + 4));
    let mut out = Vec::new();
    for e in 0..count {
        let ent = table + e * 12;
        let (root, pairs, n) = (be32(d, ent), be32(d, ent + 4), be32(d, ent + 8));
        // Depth-first order with each joint's ancestor chain, as the C's walk (ground.c ~2683).
        let mut order: Vec<(u32, Vec<u32>)> = Vec::new();
        let mut stack: Vec<(u32, Vec<u32>)> = vec![(root, vec![])];
        while let Some((j, chain)) = stack.pop() {
            if j == 0 {
                continue;
            }
            let mut c = chain.clone();
            c.push(j);
            order.push((j, c.clone()));
            let (flags, child, next) = (be32(d, j + 4), be32(d, j + 8), be32(d, j + 0xC));
            // next after the subtree: push next first so child is visited first.
            stack.push((next, chain));
            if flags & JOBJ_INSTANCE == 0 && a.is_reloc(j + 8) {
                stack.push((child, c));
            }
        }
        for p in 0..n {
            let idx = (be32(d, pairs + p * 4) >> 16) as i16 as usize;
            let id = (be32(d, pairs + p * 4) & 0xFFFF) as i16 as i32;
            let Some((_, chain)) = order.get(idx) else { continue };
            let (pos, exact) = world_translation(d, chain);
            out.push(SpawnPoint { id, pos, exact });
        }
    }
    Ok(out)
}

fn world_translation(d: &[u8], chain: &[u32]) -> ([f32; 3], bool) {
    // Column-major 3x4 affine, composed root first.
    let mut m = [[1.0f32, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
    let mut exact = true;
    for &j in chain {
        let (r, s, t) = srt(d, j);
        if r != [0.0; 3] || s != [1.0; 3] {
            exact = false;
        }
        let (sx, cx) = r[0].sin_cos();
        let (sy, cy) = r[1].sin_cos();
        let (sz, cz) = r[2].sin_cos();
        // Rz*Ry*Rx (HSD_MtxSRT order) scaled per column.
        let l = [
            [cy * cz * s[0], (sx * sy * cz - cx * sz) * s[1], (cx * sy * cz + sx * sz) * s[2], t[0]],
            [cy * sz * s[0], (sx * sy * sz + cx * cz) * s[1], (cx * sy * sz - sx * cz) * s[2], t[1]],
            [-sy * s[0], sx * cy * s[1], cx * cy * s[2], t[2]],
        ];
        let mut n = [[0.0f32; 4]; 3];
        for i in 0..3 {
            for k in 0..4 {
                n[i][k] = m[i][0] * l[0][k] + m[i][1] * l[1][k] + m[i][2] * l[2][k] + if k == 3 { m[i][3] } else { 0.0 };
            }
        }
        m = n;
    }
    ([m[0][3], m[1][3], m[2][3]], exact)
}

/// The stage file's particle bank: `map_ptcl` / `map_texg`, located once
/// (`grDatFiles_801C5FC0` -> `psInitDataBankLocate`). `Ground` then loads it into banks 0x40
/// (`grDatFiles_801C6038`) and 0x1E (ground.c ~873).
pub fn particle_bank(a: &DatArchive) -> Result<Option<melee_runtime::particle::PsBankData>, String> {
    match (a.public_offset("map_ptcl"), a.public_offset("map_texg")) {
        (Some(c), Some(t)) => {
            let data: std::sync::Arc<[u8]> = std::sync::Arc::from(&a.data[..]);
            melee_runtime::particle::PsBankData::locate(&data, c, t).map(Some)
        }
        _ => Ok(None),
    }
}

/// A joint animation track that spawns particles (FObj type 0x28, `HSD_A_J_PTCL`): its joint
/// (DAT offset of the `HSD_Joint`, with the ancestor chain for the world matrix) and its AObj.
pub struct DptclTrack {
    pub joint: u32,
    pub chain: Vec<u32>,
    pub aobj: melee_runtime::aobj::HsdAObjDesc,
}

/// Every particle track of map gobj `gobj`'s animation `anim` (`map_head->unk8[gobj].unk4[anim]`
/// walked in parallel with the joint tree `unk8[gobj].unk0`). Only the 0x28 FObjs are kept: the
/// others move the JObj, which the frame hash never reads.
pub fn dptcl_tracks(a: &DatArchive, gobj: u32, anim: u32) -> Vec<DptclTrack> {
    let d = &a.data;
    let Some(head) = a.public_offset("map_head") else { return Vec::new() };
    let (gobjs, n) = (be32(d, head + 8), be32(d, head + 12));
    if gobj >= n {
        return Vec::new();
    }
    let e = gobjs + gobj * 0x34;
    let (root, anims) = (be32(d, e), be32(d, e + 4));
    if anims == 0 || !a.is_reloc(e + 4) {
        return Vec::new();
    }
    let aj_root = be32(d, anims + 4 * anim);
    let mut tracks = dptcl_tracks_of(a, root, aj_root);
    // grAnime_801C8138: `unk8[gobj].x28` is a per-animation loop table; a nonzero entry sets
    // AOBJ_LOOP (0x20000000, grAnime_801C752C) on every AObj of the animation.
    let flags = be32(d, e + 0x28);
    if flags != 0 && d[(flags + anim) as usize] != 0 {
        for t in &mut tracks {
            t.aobj.flags |= 0x2000_0000;
        }
    }
    tracks
}

/// Every particle track (FObj 0x28) of animjoint tree `aj_root` walked in parallel with joint tree
/// `root` (`HSD_JObjAddAnimAll`'s pairing: child with child, next with next).
pub fn dptcl_tracks_of(a: &DatArchive, root: u32, aj_root: u32) -> Vec<DptclTrack> {
    use melee_runtime::aobj::HsdAObjDesc;
    use melee_runtime::fobj::HsdFObjDesc;
    let d = &a.data;
    let mut out = Vec::new();
    // (joint, animjoint, chain)
    let mut stack = vec![(root, aj_root, Vec::<u32>::new())];
    while let Some((j, aj, chain)) = stack.pop() {
        if j == 0 || aj == 0 {
            continue;
        }
        let mut c = chain.clone();
        c.push(j);
        let aobj = be32(d, aj + 8);
        if aobj != 0 {
            let mut fobjs = Vec::new();
            let mut fo = be32(d, aobj + 8);
            while fo != 0 {
                let (len, ty) = (be32(d, fo + 4), d[fo as usize + 12]);
                if ty == 0x28 {
                    let ad = be32(d, fo + 16) as usize;
                    fobjs.push(HsdFObjDesc {
                        length: len,
                        startframe: bef(d, fo + 8),
                        type_: ty,
                        frac_value: d[fo as usize + 13],
                        frac_slope: d[fo as usize + 14],
                        ad: std::sync::Arc::from(&d[ad..(ad + len as usize).min(d.len())]),
                    });
                }
                fo = be32(d, fo);
            }
            if !fobjs.is_empty() {
                out.push(DptclTrack {
                    joint: j,
                    chain: c.clone(),
                    aobj: HsdAObjDesc { flags: be32(d, aobj), end_frame: bef(d, aobj + 4), fobjdesc: fobjs, obj_id: be32(d, aobj + 12) },
                });
            }
        }
        let flags = be32(d, j + 4);
        stack.push((be32(d, j + 0xC), be32(d, aj + 4), chain));
        if flags & JOBJ_INSTANCE == 0 {
            stack.push((be32(d, j + 8), be32(d, aj), c));
        }
    }
    out
}

/// The world matrix of a joint chain from the joint descriptors (bind pose: no animation, the
/// `HSD_MtxSRT` order; see `world_translation` for exactness). TODO(group1 jobj): the animated
/// JObj matrix.
pub fn bind_pose_matrix(d: &[u8], chain: &[u32]) -> melee_runtime::mtx::Mtx {
    use melee_runtime::mtx::{self, Vec3};
    let mut m = mtx::identity();
    for &j in chain {
        let (r, s, t) = srt(d, j);
        let v = |a: [f32; 3]| Vec3 { x: a[0], y: a[1], z: a[2] };
        let l = melee_runtime::hsd_mtx::srt(&v(s), &v(r), &v(t), None);
        m = mtx::concat(&m, &l);
    }
    m
}

/// `grGroundParam` (for the music rows).
pub fn ground_param(a: &DatArchive) -> Option<u32> {
    a.public_offset("grGroundParam")
}

/// As [`bind_pose_matrix`], with the root joint's translation and scale replaced (an effect root
/// positioned by its RObj constraint, `lb_8000C1C0`, and scaled by `efAsync_SetEffectScaleXYZ`).
pub fn bind_pose_matrix_rooted(d: &[u8], chain: &[u32], root_t: [f32; 3], root_s: [f32; 3], rot_y: Option<f32>, rot_z: Option<f32>) -> melee_runtime::mtx::Mtx {
    use melee_runtime::mtx::{self, Vec3};
    let v = |a: [f32; 3]| Vec3 { x: a[0], y: a[1], z: a[2] };
    let mut m = mtx::identity();
    for (i, &j) in chain.iter().enumerate() {
        let (mut r, mut s, mut t) = srt(d, j);
        if i == 0 {
            s = root_s;
            t = root_t;
            if let Some(y) = rot_y {
                r[1] = y;
            }
            if let Some(z) = rot_z {
                r[2] = z;
            }
        }
        let l = melee_runtime::hsd_mtx::srt(&v(s), &v(r), &v(t), None);
        m = mtx::concat(&m, &l);
    }
    m
}

/// The world translation (stage-local, before the `grGroundParam->y` root scale) of the `dfs`-th
/// JObj of map gobj `gobj`'s tree (`map_head->unk8[gobj].unk0`, depth first, child before sibling:
/// `Ground_801C3FA4(gobj, dfs)` + `lb_8000B1CC`). `None` when the tree is shorter.
pub fn map_joint_world(a: &DatArchive, gobj: u32, dfs: usize) -> Option<([f32; 3], bool)> {
    let d = &a.data;
    let head = a.public_offset("map_head")?;
    let (gobjs, n) = (be32(d, head + 8), be32(d, head + 12));
    if gobj >= n {
        return None;
    }
    let root = be32(d, gobjs + gobj * 0x34);
    let mut stack: Vec<(u32, Vec<u32>)> = vec![(root, vec![])];
    let mut k = 0usize;
    while let Some((j, chain)) = stack.pop() {
        if j == 0 {
            continue;
        }
        let mut c = chain.clone();
        c.push(j);
        if k == dfs {
            return Some(world_translation(d, &c));
        }
        k += 1;
        let (flags, child, next) = (be32(d, j + 4), be32(d, j + 8), be32(d, j + 0xC));
        stack.push((next, chain));
        if flags & JOBJ_INSTANCE == 0 && a.is_reloc(j + 8) {
            stack.push((child, c));
        }
    }
    None
}

/// `yakumono_param` (a stage file's public symbol): its words, big endian.
pub fn yakumono_words(a: &DatArchive, n: usize) -> Option<Vec<u32>> {
    let at = a.public_offset("yakumono_param")?;
    Some((0..n).map(|i| be32(&a.data, at + 4 * i as u32)).collect())
}

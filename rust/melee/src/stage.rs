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

//! Render-only pose records: what the render pass of one tick knew about the matrices it loaded, so the presenter can
//! draw that frame at a fractional time between ticks (frame interpolation, docs/rust-port/notes/display.md).
//!
//! The capture side (melee_runtime `pose_capture`) fills a [`PoseFrame`] while the game's draw callbacks run; the GX
//! command stream tags each draw's matrix slots and camera with indices into it; the presenter (melee_gx `interp`)
//! re-poses the draws. Nothing here is ever read by the simulation: a frame is plain data, copied out of the game's
//! objects as they were drawn.

/// A 3x4 row-major matrix (the GX / HSD `Mtx`).
pub type Mtx34 = [[f32; 4]; 3];

/// Sub-steps of one tick the authored channels are sampled at (`samples` holds `SAMPLES + 1` rows: phase 0..=1).
pub const SAMPLES: usize = 4;

/// One camera that was current while drawing (`HSD_CObjSetCurrent`).
#[derive(Clone, Debug)]
pub struct CamRec {
    /// Identity across frames (the CObj's address).
    pub key: usize,
    /// World to view, as `HSD_CObjSetupViewingMtx` left it.
    pub view: Mtx34,
    /// Perspective (true) or orthographic / frustum.
    pub perspective: bool,
}

/// How a joint's world matrix can be rebuilt between ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointKind {
    /// `world = parent_world * SRT(scale, rotate, translate)`, checked against the matrix the game built (Euler).
    Srt,
    /// The same with a quaternion rotation (`JOBJ_USE_QUATERNION`): its rotation is held, scale and translation move.
    SrtQuat,
    /// Anything else (constraints, IK, user matrices, billboards of the tree): the local matrix relative to the parent
    /// is held, so it follows its parent.
    Opaque,
}

/// One joint reached by a tagged draw (and every ancestor of one).
#[derive(Clone, Debug)]
pub struct JointRec {
    /// Identity across frames (the JObj's address).
    pub key: usize,
    /// Index of the parent in [`PoseFrame::joints`], or `u32::MAX` for a root.
    pub parent: u32,
    pub kind: JointKind,
    /// Rotation (x, y, z), translation, scale: the values the matrix was built from.
    pub srt: [f32; 9],
    /// The quaternion (x, y, z, w) of a [`JointKind::SrtQuat`] joint.
    pub quat: [f32; 4],
    /// The parent's accumulated scale the build compensated (`jobj->parent->scl`), if any.
    pub pscl: Option<[f32; 3]>,
    /// The joint's world matrix this tick.
    pub world: Mtx34,
    /// Bit `c` set: component `c` of `srt` follows its authored animation over the next tick, sampled in `samples`.
    pub authored: u16,
    /// `SAMPLES + 1` rows of the 9 components at phase k / SAMPLES of the next tick (only authored bits are meaningful).
    pub samples: Option<Box<[[f32; 9]; SAMPLES + 1]>>,
}

/// One weighted bone of a skinned matrix slot: `world(joint) * envelope * weight`.
#[derive(Clone, Debug)]
pub struct BoneRec {
    pub joint: u32,
    pub weight: f32,
    /// The inverse bind matrix (`jobj->envelopemtx`); None: the joint's world alone (single-bone slots).
    pub envelope: Option<Mtx34>,
}

/// The right-hand factor of a skinned slot (`_HSD_mkEnvelopeModelNodeMtx`).
#[derive(Clone, Debug)]
pub enum RightRec {
    None,
    /// A constant matrix (`inverse(x->envelopemtx)`).
    Fixed(Mtx34),
    /// `inverse(world(x)) * world(m)`.
    InvWorld { x: u32, m: u32 },
    /// `inverse(world(x) * env) * world(m)`.
    InvWorldEnv { x: u32, env: Mtx34, m: u32 },
}

/// What one loaded position matrix was made of.
#[derive(Clone, Debug)]
pub enum SlotRec {
    /// `view * world(joint)`.
    Rigid { cam: u32, joint: u32 },
    /// `view * (sum of bones) * right`.
    Skinned { cam: u32, bones: std::ops::Range<u32>, right: RightRec },
}

/// Everything one tick's render pass recorded. Tags in the GX stream are 1-based indices (0: none).
#[derive(Clone, Debug, Default)]
pub struct PoseFrame {
    pub cams: Vec<CamRec>,
    pub joints: Vec<JointRec>,
    pub slots: Vec<SlotRec>,
    pub bones: Vec<BoneRec>,
    /// Joint records whose animation could not be sampled for the next tick (diagnostics).
    pub declined: u32,
}

impl PoseFrame {
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty() && self.cams.is_empty()
    }
}

/// `a * b` (3x4, the bottom row implied 0 0 0 1).
pub fn concat(a: &Mtx34, b: &Mtx34) -> Mtx34 {
    let mut o = [[0.0f32; 4]; 3];
    for r in 0..3 {
        for c in 0..4 {
            let mut v = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
            if c == 3 {
                v += a[r][3];
            }
            o[r][c] = v;
        }
    }
    o
}

/// The inverse of an affine 3x4 (None when singular).
pub fn inverse(m: &Mtx34) -> Option<Mtx34> {
    let a = m;
    let det = a[0][0] * (a[1][1] * a[2][2] - a[2][1] * a[1][2]) - a[0][1] * (a[1][0] * a[2][2] - a[2][0] * a[1][2])
        + a[0][2] * (a[1][0] * a[2][1] - a[2][0] * a[1][1]);
    if !det.is_finite() || det.abs() < 1e-20 {
        return None;
    }
    let d = 1.0 / det;
    let mut o = [[0.0f32; 4]; 3];
    o[0][0] = (a[1][1] * a[2][2] - a[2][1] * a[1][2]) * d;
    o[0][1] = -(a[0][1] * a[2][2] - a[2][1] * a[0][2]) * d;
    o[0][2] = (a[0][1] * a[1][2] - a[1][1] * a[0][2]) * d;
    o[1][0] = -(a[1][0] * a[2][2] - a[2][0] * a[1][2]) * d;
    o[1][1] = (a[0][0] * a[2][2] - a[2][0] * a[0][2]) * d;
    o[1][2] = -(a[0][0] * a[1][2] - a[1][0] * a[0][2]) * d;
    o[2][0] = (a[1][0] * a[2][1] - a[2][0] * a[1][1]) * d;
    o[2][1] = -(a[0][0] * a[2][1] - a[2][0] * a[0][1]) * d;
    o[2][2] = (a[0][0] * a[1][1] - a[1][0] * a[0][1]) * d;
    for r in 0..3 {
        o[r][3] = -(o[r][0] * a[0][3] + o[r][1] * a[1][3] + o[r][2] * a[2][3]);
    }
    Some(o)
}

/// The largest absolute element difference.
pub fn max_diff(a: &Mtx34, b: &Mtx34) -> f32 {
    let mut d = 0.0f32;
    for r in 0..3 {
        for c in 0..4 {
            let x = (a[r][c] - b[r][c]).abs();
            if !(x <= d) {
                d = x; // NaN propagates as "different"
            }
        }
    }
    d
}

/// The local matrix `HSD_MtxSRT` builds from Euler angles (render-side copy; the capture checks it against the game's
/// own build before a joint is marked [`JointKind::Srt`]).
pub fn srt(s: [f32; 3], r: [f32; 3], t: [f32; 3], ps: Option<[f32; 3]>) -> Mtx34 {
    let (sx, cx) = r[0].sin_cos();
    let (sy, cy) = r[1].sin_cos();
    let (sz, cz) = r[2].sin_cos();
    let (x2, mut x1, mut x0) = (s[0], s[0], s[0]);
    let (mut y2, y1, mut y0) = (s[1], s[1], s[1]);
    let (mut z2, mut z1, z0) = (s[2], s[2], s[2]);
    if let Some(p) = ps {
        let (t1, t2, t3) = (1.0 / p[0], 1.0 / p[1], 1.0 / p[2]);
        y2 *= p[1] * t1;
        z2 *= p[2] * t1;
        x1 *= p[0] * t2;
        z1 *= p[2] * t2;
        x0 *= p[0] * t3;
        y0 *= p[1] * t3;
    }
    [
        [cz * (x2 * cy), y2 * (cz * (sx * sy) - cx * sz), z2 * (cz * (cx * sy) + sx * sz), t[0]],
        [sz * (x1 * cy), y1 * (sz * (sx * sy) + cx * cz), z1 * (sz * (cx * sy) - sx * cz), t[1]],
        [-x0 * sy, cy * (y0 * sx), cy * (z0 * cx), t[2]],
    ]
}

/// `HSD_MtxSRTQuat` (render-side copy).
pub fn srt_quat(s: [f32; 3], q: [f32; 4], t: [f32; 3], ps: Option<[f32; 3]>) -> Mtx34 {
    let scale = |x: f32, y: f32, z: f32| -> Mtx34 { [[x, 0.0, 0.0, 0.0], [0.0, y, 0.0, 0.0], [0.0, 0.0, z, 0.0]] };
    let mut m = scale(s[0], s[1], s[2]);
    if let Some(p) = ps {
        m = concat(&scale(p[0], p[1], p[2]), &m);
    }
    m = concat(&quat_mtx(q), &m);
    if let Some(p) = ps {
        m = concat(&scale(1.0 / p[0], 1.0 / p[1], 1.0 / p[2]), &m);
    }
    m[0][3] += t[0];
    m[1][3] += t[1];
    m[2][3] += t[2];
    m
}

/// The rotation matrix of a unit quaternion (x, y, z, w), as `PSMTXQuat`.
pub fn quat_mtx(q: [f32; 4]) -> Mtx34 {
    let [x, y, z, w] = q;
    let n = x * x + y * y + z * z + w * w;
    let s = if n > 0.0 { 2.0 / n } else { 0.0 };
    let (xs, ys, zs) = (x * s, y * s, z * s);
    let (wx, wy, wz) = (w * xs, w * ys, w * zs);
    let (xx, xy, xz) = (x * xs, x * ys, x * zs);
    let (yy, yz, zz) = (y * ys, y * zs, z * zs);
    [
        [1.0 - (yy + zz), xy - wz, xz + wy, 0.0],
        [xy + wz, 1.0 - (xx + zz), yz - wx, 0.0],
        [xz - wy, yz + wx, 1.0 - (xx + yy), 0.0],
    ]
}

/// The inverse transpose of the upper 3x3 (the normal matrix `HSD_MtxInverseTranspose` loads), translation zero.
pub fn inverse_transpose(m: &Mtx34) -> Option<Mtx34> {
    let i = inverse(m)?;
    Some([[i[0][0], i[1][0], i[2][0], 0.0], [i[0][1], i[1][1], i[2][1], 0.0], [i[0][2], i[1][2], i[2][2], 0.0]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_and_concat() {
        let m = srt([1.5, 2.0, 0.5], [0.3, -1.1, 2.0], [10.0, -4.0, 7.0], None);
        let i = inverse(&m).unwrap();
        let id = concat(&m, &i);
        let e: Mtx34 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        assert!(max_diff(&id, &e) < 1e-5, "{id:?}");
    }

    #[test]
    fn quat_matches_euler_about_one_axis() {
        let a = 0.7f32;
        let q = [0.0, (a / 2.0).sin(), 0.0, (a / 2.0).cos()];
        let m = srt_quat([1.0; 3], q, [1.0, 2.0, 3.0], None);
        let e = srt([1.0; 3], [0.0, a, 0.0], [1.0, 2.0, 3.0], None);
        assert!(max_diff(&m, &e) < 1e-5);
    }
}

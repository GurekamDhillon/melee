//! Render-only frame interpolation and true widescreen: per-draw rewrites of a prepared frame's uniform blocks.
//!
//! The simulation runs at 60 Hz; a present between two ticks draws the newest frame with its matrices re-posed at a
//! fractional time (docs/rust-port/notes/display.md):
//! * joints the render pass tagged (melee_core `pose`) are rebuilt from their scale / rotation / translation: an
//!   animated channel is re-sampled from its authored keyframes, any other channel is interpolated between the two
//!   ticks (or extrapolated from the last delta in the predictive mode);
//! * cameras are interpolated as an eye position plus a rotation, with cut detection (a jump drops the whole frame
//!   pair to "show the newest frame as is");
//! * a perspective draw nothing tagged moves with its camera only; orthographic layers and, in a match, the HUD's
//!   own cameras are held.
//! Nothing here is ever read by the game: the frames are copies.
//!
//! Widescreen (renderer side): perspective draws get their projection's first row scaled by `(73/60) / aspect`
//! (Melee's cameras are authored at 73:60, cm/camera.c), except draws that fill a 4:3 frustum on purpose (screen
//! flashes, backdrop planes), draws into a smaller viewport (render-to-texture), corner-pinned perspective layers;
//! the orthographic HUD is not widened, except the P1/P2 nametag layer (ifnametag.c), which follows the world.
use crate::draw::DrawCmd;
use melee_core::pose::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Interpolate between the last two ticks (one tick of latency, exact endpoints), or extrapolate past the newest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Interpolate,
    Extrapolate,
}

impl Mode {
    pub fn parse(v: &str) -> Option<Self> {
        match v.trim().to_ascii_lowercase().as_str() {
            "interpolate" | "interp" | "i" => Some(Self::Interpolate),
            "extrapolate" | "predict" | "extrap" | "e" => Some(Self::Extrapolate),
            _ => None,
        }
    }
}

/// A camera eye jump (world units) or view rotation (radians) between ticks beyond these is a cut.
pub const CUT_MOVE: f32 = 120.0;
pub const CUT_TURN: f32 = 1.2;
/// A channel or root delta beyond this is not motion (a teleport, a respawn): held.
const MAX_STEP: f32 = 30.0;

/// One tick's frame as the presenter keeps it.
pub struct Frame {
    pub draws: Vec<DrawCmd>,
    pub pose: Option<Arc<PoseFrame>>,
    /// The decode thread's packing of `draws`, until the frame is prepared (it travels with the frame, so a frame
    /// prepared later than it arrived, or one whose EFB copies are rendered after it was skipped, never has to be
    /// packed again on the presenting thread).
    pub packed: Option<crate::render::Packed>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub draws: u32,
    pub posed: u32,
    pub carried: u32,
    pub held: u32,
    pub widened: u32,
    pub cuts: u32,
    pub joints: u32,
    pub authored: u32,
}

impl Stats {
    pub fn add(&mut self, o: &Stats) {
        self.draws += o.draws;
        self.posed += o.posed;
        self.carried += o.carried;
        self.held += o.held;
        self.widened += o.widened;
        self.cuts += o.cuts;
        self.joints = o.joints;
        self.authored = o.authored;
    }
}

struct CamPlan {
    prev: Option<u32>,
    held: bool,
    proj: Option<([[f32; 4]; 4], [[f32; 4]; 4])>,
}

struct JointPlan {
    prev: Option<u32>,
    /// Channel bits that follow the previous tick's authored samples (interpolate) / the current one's (extrapolate).
    authored: u16,
}

/// Uniform block layout of a draw (`shader::fill_uniform`): header, [line], projection, 10 position, 10 texture, 10
/// normal 3x4 matrices.
fn base(d: &DrawCmd) -> usize {
    16 + if d.info.line_mode != 0 { 16 } else { 0 }
}

fn rd(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}
fn wr(b: &mut [u8], o: usize, v: f32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn proj_of(d: &DrawCmd) -> Option<[[f32; 4]; 4]> {
    let b = base(d);
    if d.uniform.len() < b + 64 {
        return None;
    }
    let mut p = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            p[r][c] = rd(&d.uniform, b + (r * 4 + c) * 4);
        }
    }
    Some(p)
}

fn is_ortho(p: &[[f32; 4]; 4]) -> bool {
    p[3][3] == 1.0
}

fn mtx_at(u: &[u8], o: usize) -> Mtx34 {
    let mut m = [[0.0; 4]; 3];
    for r in 0..3 {
        for c in 0..4 {
            m[r][c] = rd(u, o + (r * 4 + c) * 4);
        }
    }
    m
}
fn put_mtx(u: &mut [u8], o: usize, m: &Mtx34) {
    for r in 0..3 {
        for c in 0..4 {
            let v = m[r][c];
            if v.is_finite() {
                wr(u, o + (r * 4 + c) * 4, v);
            }
        }
    }
}

// ---------------------------------------------------------------- rotations

type Quat = [f32; 4];

fn mtx_to_quat(m: &Mtx34) -> Option<(Quat, [f32; 3])> {
    let s = [0, 1, 2].map(|c| (m[0][c] * m[0][c] + m[1][c] * m[1][c] + m[2][c] * m[2][c]).sqrt());
    if s.iter().any(|&x| !(x > 1e-12)) {
        return None;
    }
    let r = |i: usize, j: usize| m[i][j] / s[j];
    let tr = r(0, 0) + r(1, 1) + r(2, 2);
    let q = if tr > 0.0 {
        let k = (tr + 1.0).sqrt() * 2.0;
        [(r(2, 1) - r(1, 2)) / k, (r(0, 2) - r(2, 0)) / k, (r(1, 0) - r(0, 1)) / k, 0.25 * k]
    } else if r(0, 0) > r(1, 1) && r(0, 0) > r(2, 2) {
        let k = (1.0 + r(0, 0) - r(1, 1) - r(2, 2)).sqrt() * 2.0;
        [0.25 * k, (r(0, 1) + r(1, 0)) / k, (r(0, 2) + r(2, 0)) / k, (r(2, 1) - r(1, 2)) / k]
    } else if r(1, 1) > r(2, 2) {
        let k = (1.0 + r(1, 1) - r(0, 0) - r(2, 2)).sqrt() * 2.0;
        [(r(0, 1) + r(1, 0)) / k, 0.25 * k, (r(1, 2) + r(2, 1)) / k, (r(0, 2) - r(2, 0)) / k]
    } else {
        let k = (1.0 + r(2, 2) - r(0, 0) - r(1, 1)).sqrt() * 2.0;
        [(r(0, 2) + r(2, 0)) / k, (r(1, 2) + r(2, 1)) / k, 0.25 * k, (r(1, 0) - r(0, 1)) / k]
    };
    // a reflection (negative determinant) is not a rotation
    let det = m[0][0] * (m[1][1] * m[2][2] - m[2][1] * m[1][2]) - m[0][1] * (m[1][0] * m[2][2] - m[2][0] * m[1][2])
        + m[0][2] * (m[1][0] * m[2][1] - m[2][0] * m[1][1]);
    (det > 0.0 && q.iter().all(|v| v.is_finite())).then_some((q, s))
}

fn qdot(a: &Quat, b: &Quat) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Slerp from `a` to `b` at `t` (t outside 0..1 extrapolates along the same arc).
fn slerp(a: &Quat, b: &Quat, t: f32) -> Quat {
    let mut b = *b;
    let mut d = qdot(a, &b);
    if d < 0.0 {
        b = b.map(|x| -x);
        d = -d;
    }
    let d = d.min(1.0);
    let th = d.acos();
    let (wa, wb) = if th < 1e-4 { (1.0 - t, t) } else { (((1.0 - t) * th).sin() / th.sin(), (t * th).sin() / th.sin()) };
    let q = [0, 1, 2, 3].map(|i| a[i] * wa + b[i] * wb);
    let n = qdot(&q, &q).sqrt();
    if n > 0.0 { q.map(|x| x / n) } else { *a }
}

fn angle(a: &Quat, b: &Quat) -> f32 {
    2.0 * qdot(a, b).abs().min(1.0).acos()
}

fn compose(q: &Quat, s: [f32; 3], t: [f32; 3]) -> Mtx34 {
    let mut m = quat_mtx(*q);
    for r in 0..3 {
        for c in 0..3 {
            m[r][c] *= s[c];
        }
        m[r][3] = t[r];
    }
    m
}

/// `a` -> `b` at `t` as rotation (slerp), scale and translation (linear); None when either is not decomposable or the
/// step is beyond the limits.
fn blend_mtx(a: &Mtx34, b: &Mtx34, t: f32, max_move: f32, max_turn: f32) -> Option<Mtx34> {
    let (qa, sa) = mtx_to_quat(a)?;
    let (qb, sb) = mtx_to_quat(b)?;
    let mv = ((b[0][3] - a[0][3]).powi(2) + (b[1][3] - a[1][3]).powi(2) + (b[2][3] - a[2][3]).powi(2)).sqrt();
    if !(mv <= max_move) || !(angle(&qa, &qb) <= max_turn) {
        return None;
    }
    let q = slerp(&qa, &qb, t);
    let s = [0, 1, 2].map(|i| sa[i] + (sb[i] - sa[i]) * t);
    let tr = [0, 1, 2].map(|i| a[i][3] + (b[i][3] - a[i][3]) * t);
    Some(compose(&q, s, tr))
}

/// A view matrix (world -> camera) at `t` from `a` to `b`: the eye moves on a line, the rotation on an arc.
fn blend_view(a: &Mtx34, b: &Mtx34, t: f32) -> Option<Mtx34> {
    let ia = inverse(a)?;
    let ib = inverse(b)?;
    let w = blend_mtx(&ia, &ib, t, f32::INFINITY, f32::INFINITY)?;
    inverse(&w)
}

/// Whether a view change between ticks is a cut.
fn view_cut(a: &Mtx34, b: &Mtx34) -> bool {
    let (Some(ia), Some(ib)) = (inverse(a), inverse(b)) else { return true };
    let mv = ((ib[0][3] - ia[0][3]).powi(2) + (ib[1][3] - ia[1][3]).powi(2) + (ib[2][3] - ia[2][3]).powi(2)).sqrt();
    let (Some((qa, _)), Some((qb, _))) = (mtx_to_quat(&ia), mtx_to_quat(&ib)) else { return true };
    !(mv <= CUT_MOVE) || !(angle(&qa, &qb) <= CUT_TURN)
}

fn wrap_pi(d: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut d = d % tau;
    if d > std::f32::consts::PI {
        d -= tau;
    } else if d < -std::f32::consts::PI {
        d += tau;
    }
    d
}

/// The samples row interpolated at phase `t` (0..1).
fn sample(s: &[[f32; 9]; SAMPLES + 1], c: usize, t: f32) -> f32 {
    let x = (t.clamp(0.0, 1.0)) * SAMPLES as f32;
    let i = (x.floor() as usize).min(SAMPLES - 1);
    let f = x - i as f32;
    s[i][c] + (s[i + 1][c] - s[i][c]) * f
}

// ---------------------------------------------------------------- widescreen

/// Renderer-side widescreen: the x scale applied to perspective draws (1: off).
#[derive(Clone, Copy, Debug)]
pub struct Widen {
    pub factor: f32,
}

impl Widen {
    /// Melee's cameras are authored for a 73:60 picture (cm/camera.c `1.2173333`, ty/toy.c); a content aspect `a` shows
    /// `(73/60) / a` of the authored x range per unit (16:9: 219/320).
    pub fn for_aspect(a: f32) -> Option<Self> {
        if a > 4.0 / 3.0 + 1e-3 { Some(Widen { factor: (73.0 / 60.0) / a }) } else { None }
    }
}

/// The P1/P2 tag layer's orthographic camera (if/ifnametag.c `nametag_CObjDesc`: a 640x480 box, top 0, bottom -480,
/// near 0.1, far 32768).
fn nametag_layer(p: &[[f32; 4]; 4]) -> bool {
    let near = 0.1f32;
    let far = 32768.0f32;
    (p[0][0] - 2.0 / 640.0).abs() < 1e-6
        && (p[0][3] + 1.0).abs() < 1e-4
        && (p[1][0]).abs() < 1e-9
        && (p[1][1] - 2.0 / 480.0).abs() < 1e-6
        && (p[1][3] - 1.0).abs() < 1e-4
        && (p[2][2].abs() - 1.0 / (far - near)).abs() < 1e-7
}

/// A perspective frustum authored to be filled by a 4:3 quad (lb/lbbgflash.c's screen flash and the wipes on it): no
/// off-centre shift and y over x scale exactly 4/3 (the game's own cameras are 73:60).
fn fullscreen_4x3(p: &[[f32; 4]; 4]) -> bool {
    p[0][2] == 0.0 && p[1][2] == 0.0 && p[0][0] > 0.0 && ((p[1][1] / p[0][0]) - 4.0 / 3.0).abs() < 1e-4
}

/// A perspective layer pinned to a screen corner: the frustum's off-centre shifts at +-1 on both axes.
fn corner_pinned(p: &[[f32; 4]; 4]) -> bool {
    (p[0][2].abs() - 1.0).abs() < 1e-3 && (p[1][2].abs() - 1.0).abs() < 1e-3
}

/// The draw's viewport covers the whole logical 640x480 screen (header: render w, h, logical w, h).
fn full_viewport(d: &DrawCmd) -> bool {
    if d.uniform.len() < 16 {
        return false;
    }
    let (w, h) = (rd(&d.uniform, 8).abs(), rd(&d.uniform, 12).abs());
    w >= 600.0 && h >= 440.0
}

/// Where a perspective draw's vertices land, as an NDC box (decoding only direct or indexed f32 / s16 positions of a
/// small draw); None when it cannot tell.
fn ndc_box(d: &DrawCmd, p: &[[f32; 4]; 4]) -> Option<[f32; 4]> {
    use crate::consts::*;
    if d.vtx_count == 0 || d.vtx_count > 64 || d.palette.is_some() {
        return None;
    }
    let attrs = &d.config.shader.attrs;
    let pos = &attrs[GX_VA_POS as usize];
    if attrs[GX_VA_PNMTXIDX as usize].attr_type != GX_NONE as u8 {
        return None;
    }
    let (ty, frac) = (pos.comp_type as u32, pos.frac as u32);
    let ncomp = pos.cnt as usize;
    if !(2..=3).contains(&ncomp) {
        return None;
    }
    let csize = match ty {
        GX_F32 => 4,
        GX_S16 | GX_U16 => 2,
        GX_S8 | GX_U8 => 1,
        _ => return None,
    };
    let vsize = d.config.shader.vtx_stride as usize;
    let off = pos.offset as usize;
    let slot = d.current_pn_mtx as usize / 3;
    if slot >= 10 || vsize == 0 {
        return None;
    }
    let m = mtx_at(&d.uniform, base(d) + 64 + slot * 48);
    let mut bx = [f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY];
    let direct = pos.attr_type as u32 == GX_DIRECT;
    let le = !direct && pos.le;
    for v in 0..d.vtx_count as usize {
        let o = v * vsize + off;
        let src: &[u8] = if direct {
            d.verts.get(o..o + ncomp * csize)?
        } else {
            let idx = if pos.attr_type as u32 == GX_INDEX8 { *d.verts.get(o)? as usize } else { u16::from_be_bytes([*d.verts.get(o)?, *d.verts.get(o + 1)?]) as usize };
            let a = d.arrays[0].as_ref()?;
            let stride = if pos.stride == 0 { ncomp * csize } else { pos.stride as usize };
            let o = idx * stride;
            if o + ncomp * csize > a.needed as usize {
                return None;
            }
            let o = o + a.offset as usize;
            a.data.get(o..o + ncomp * csize)?
        };
        let comp = |i: usize| -> f32 {
            let b = &src[i * csize..i * csize + csize];
            let sc = (1u32 << frac.min(31)) as f32;
            match ty {
                GX_F32 => {
                    let w = [b[0], b[1], b[2], b[3]];
                    f32::from_bits(if le { u32::from_le_bytes(w) } else { u32::from_be_bytes(w) })
                }
                GX_S16 => (if le { i16::from_le_bytes([b[0], b[1]]) } else { i16::from_be_bytes([b[0], b[1]]) }) as f32 / sc,
                GX_U16 => (if le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) }) as f32 / sc,
                GX_S8 => b[0] as i8 as f32 / sc,
                _ => b[0] as f32 / sc,
            }
        };
        let x = [comp(0), comp(1), if ncomp == 3 { comp(2) } else { 0.0 }];
        let e = [0, 1, 2].map(|r| m[r][0] * x[0] + m[r][1] * x[1] + m[r][2] * x[2] + m[r][3]);
        let cw = p[3][0] * e[0] + p[3][1] * e[1] + p[3][2] * e[2] + p[3][3];
        if !(cw > 1e-6) {
            return None;
        }
        let cx = (p[0][0] * e[0] + p[0][1] * e[1] + p[0][2] * e[2] + p[0][3]) / cw;
        let cy = (p[1][0] * e[0] + p[1][1] * e[1] + p[1][2] * e[2] + p[1][3]) / cw;
        bx = [bx[0].min(cx), bx[1].min(cy), bx[2].max(cx), bx[3].max(cy)];
    }
    Some(bx)
}

/// Per draw: whether the widening applies.
fn widen_class(d: &DrawCmd) -> bool {
    let Some(p) = proj_of(d) else { return false };
    if d.copy.is_some() {
        return false;
    }
    if is_ortho(&p) {
        return nametag_layer(&p);
    }
    if !full_viewport(d) || fullscreen_4x3(&p) || corner_pinned(&p) {
        return false;
    }
    // a backdrop plane that covers the whole 4:3 frustum keeps covering the screen
    if let Some(b) = ndc_box(d, &p) {
        if b[0] <= -0.995 && b[2] >= 0.995 && b[1] <= -0.995 && b[3] >= 0.995 {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------- the solver

/// The plan for one (previous, current) frame pair, and the per-present evaluation buffers.
#[derive(Default)]
pub struct Solver {
    pub mode: Mode,
    /// A match is on screen: the HUD's own perspective cameras are held like the orthographic layers.
    pub match_scene: bool,
    cut: bool,
    cams: Vec<CamPlan>,
    joints: Vec<JointPlan>,
    primary: u32,
    widen_ok: Vec<bool>,
    /// Per present.
    views: Vec<Option<Mtx34>>,
    inv_views: Vec<Option<Mtx34>>,
    worlds: Vec<Mtx34>,
    slots: Vec<Option<(Mtx34, Mtx34)>>,
    projs: Vec<Option<[[f32; 4]; 4]>>,
    pub stats: Stats,
    planned: bool,
}

impl Solver {
    pub fn with_mode(mode: Mode) -> Self {
        Self { mode, ..Default::default() }
    }

    /// Pair `cur` with `prev` (a new frame became current).
    pub fn plan(&mut self, prev: Option<&Frame>, cur: &Frame, widen: bool) {
        self.planned = true;
        self.cut = false;
        self.cams.clear();
        self.joints.clear();
        self.widen_ok.clear();
        if widen {
            self.widen_ok.extend(cur.draws.iter().map(widen_class));
        }
        let Some(cp) = cur.pose.as_deref() else {
            self.cut = prev.is_none();
            return;
        };
        let pp = prev.and_then(|p| p.pose.as_deref());
        // camera pairing by object, projection from the first draw of each camera
        let pcam: HashMap<usize, u32> = pp.map(|p| p.cams.iter().enumerate().map(|(i, c)| (c.key, i as u32)).collect()).unwrap_or_default();
        let mut first_proj: Vec<Option<[[f32; 4]; 4]>> = vec![None; cp.cams.len()];
        let mut count = vec![0u32; cp.cams.len()];
        for d in &cur.draws {
            if d.cam != 0 && (d.cam as usize) <= cp.cams.len() {
                let i = d.cam as usize - 1;
                if first_proj[i].is_none() {
                    first_proj[i] = proj_of(d);
                }
                if d.pose.iter().any(|&t| t != 0) {
                    count[i] += 1;
                }
            }
        }
        let mut prev_proj: HashMap<u32, [[f32; 4]; 4]> = HashMap::new();
        if let (Some(p), Some(pf)) = (pp, prev) {
            for d in &pf.draws {
                if d.cam != 0 && (d.cam as usize) <= p.cams.len() && !prev_proj.contains_key(&(d.cam - 1)) {
                    if let Some(x) = proj_of(d) {
                        prev_proj.insert(d.cam - 1, x);
                    }
                }
            }
        }
        self.primary = (0..cp.cams.len()).filter(|&i| cp.cams[i].perspective).max_by_key(|&i| count[i]).map_or(u32::MAX, |i| i as u32);
        for (i, c) in cp.cams.iter().enumerate() {
            let prev = pcam.get(&c.key).copied();
            let held = self.match_scene && i as u32 != self.primary;
            let proj = match (prev.and_then(|j| prev_proj.get(&j)), first_proj[i]) {
                (Some(a), Some(b)) if !is_ortho(a) && !is_ortho(&b) => Some((*a, b)),
                _ => None,
            };
            if i as u32 == self.primary {
                if let Some(j) = prev {
                    if view_cut(&pp.unwrap().cams[j as usize].view, &c.view) {
                        self.cut = true;
                    }
                } else if pp.is_some_and(|p| !p.cams.is_empty()) {
                    self.cut = true; // the main camera is new: a scene change
                }
            }
            self.cams.push(CamPlan { prev, held, proj });
        }
        if pp.is_none() {
            self.cut = true;
        }
        if self.cut {
            self.stats.cuts += 1;
        }
        let pj: HashMap<usize, u32> = pp.map(|p| p.joints.iter().enumerate().map(|(i, j)| (j.key, i as u32)).collect()).unwrap_or_default();
        let mut authored = 0;
        for j in &cp.joints {
            let mut prev = pj.get(&j.key).copied();
            // the same address with a different parent is another object
            if let (Some(p), Some(pf)) = (prev, pp) {
                let pr = &pf.joints[p as usize];
                let same_parent = match (pr.parent, j.parent) {
                    (u32::MAX, u32::MAX) => true,
                    (a, b) if a != u32::MAX && b != u32::MAX => pf.joints[a as usize].key == cp.joints[b as usize].key,
                    _ => false,
                };
                if !same_parent || pr.kind != j.kind {
                    prev = None;
                }
            }
            let bits = match self.mode {
                Mode::Interpolate => match prev.map(|p| &pp.unwrap().joints[p as usize]) {
                    // the previous tick's samples must land where this tick actually is
                    Some(pr) => match pr.samples.as_deref() {
                        Some(s) => {
                            let mut b = 0u16;
                            for c in 0..9 {
                                if pr.authored & (1 << c) != 0 && (s[SAMPLES][c] - j.srt[c]).abs() <= 1e-4 * (1.0 + j.srt[c].abs()) {
                                    b |= 1 << c;
                                }
                            }
                            b
                        }
                        None => 0,
                    },
                    None => 0,
                },
                Mode::Extrapolate => j.authored,
            };
            if bits != 0 {
                authored += 1;
            }
            self.joints.push(JointPlan { prev, authored: bits });
        }
        self.stats.joints = cp.joints.len() as u32;
        self.stats.authored = authored;
    }

    /// The frame pair is a cut (or has no pose): present the newest frame as is.
    pub fn is_cut(&self) -> bool {
        self.cut
    }

    /// Present the newest frame as is until the next pair: the two frames are not consecutive ticks (ticks ran in a
    /// burst after a stall, so the older one is several ticks back and a blend would smear across them).
    pub fn mark_cut(&mut self) {
        if !self.cut {
            self.cut = true;
            self.stats.cuts += 1;
        }
    }

    /// The main (fight) camera's view at the last evaluated phase; None when it is held, still or the pair is a cut.
    pub fn primary_view(&self) -> Option<Mtx34> {
        if self.cut {
            return None;
        }
        self.views.get(self.primary as usize).copied().flatten()
    }

    /// Evaluate cameras, joints and slots at phase `t` (0..1 between the two ticks; extrapolate: past the newest).
    pub fn evaluate(&mut self, prev: Option<&Frame>, cur: &Frame, t: f32) {
        self.views.clear();
        self.inv_views.clear();
        self.worlds.clear();
        self.slots.clear();
        self.projs.clear();
        let Some(cp) = cur.pose.as_deref() else { return };
        if self.cut {
            return;
        }
        let pp = prev.and_then(|p| p.pose.as_deref());
        let interp = self.mode == Mode::Interpolate;
        for (i, c) in cp.cams.iter().enumerate() {
            let plan = &self.cams[i];
            let v = if plan.held {
                None
            } else {
                match plan.prev.map(|j| &pp.unwrap().cams[j as usize]) {
                    Some(pc) if pc.view == c.view => None,
                    Some(pc) if !view_cut(&pc.view, &c.view) => {
                        if interp { blend_view(&pc.view, &c.view, t) } else { blend_view(&pc.view, &c.view, 1.0 + t) }
                    }
                    _ => None,
                }
            };
            self.inv_views.push(v.and_then(|_| inverse(&c.view)));
            self.views.push(v);
            let p = plan.proj.and_then(|(a, b)| {
                if a == b {
                    return None;
                }
                let k = if interp { t } else { 1.0 + t };
                let mut o = b;
                for r in 0..4 {
                    for cc in 0..4 {
                        o[r][cc] = a[r][cc] + (b[r][cc] - a[r][cc]) * k;
                    }
                }
                // a big change (a zoom cut) is held
                ((o[1][1] - b[1][1]).abs() <= 0.25 * b[1][1].abs()).then_some(o)
            });
            self.projs.push(p);
        }
        // joints, parents first (the capture records a parent before its children)
        for (i, j) in cp.joints.iter().enumerate() {
            let plan = &self.joints[i];
            let pr = plan.prev.map(|p| &pp.unwrap().joints[p as usize]);
            let parent_w = (j.parent != u32::MAX).then(|| self.worlds[j.parent as usize]);
            let w = match j.kind {
                JointKind::Srt | JointKind::SrtQuat => {
                    let mut v = j.srt;
                    for c in 0..9 {
                        if j.kind == JointKind::SrtQuat && c < 3 {
                            continue;
                        }
                        v[c] = if plan.authored & (1 << c) != 0 {
                            if interp {
                                sample(pr.unwrap().samples.as_deref().unwrap(), c, t)
                            } else {
                                sample(j.samples.as_deref().unwrap(), c, t)
                            }
                        } else if let Some(pr) = pr {
                            let mut d = j.srt[c] - pr.srt[c];
                            if c < 3 {
                                d = wrap_pi(d);
                            }
                            if d.abs() > MAX_STEP || (c < 3 && d.abs() > 1.5) {
                                j.srt[c]
                            } else if interp {
                                j.srt[c] - d * (1.0 - t)
                            } else {
                                j.srt[c] + d * t
                            }
                        } else {
                            j.srt[c]
                        };
                    }
                    let local = if j.kind == JointKind::SrtQuat {
                        let q = match pr {
                            Some(pr) if pr.kind == JointKind::SrtQuat => slerp(&pr.quat, &j.quat, if interp { t } else { 1.0 + t }),
                            _ => j.quat,
                        };
                        srt_quat([v[6], v[7], v[8]], q, [v[3], v[4], v[5]], j.pscl)
                    } else {
                        srt([v[6], v[7], v[8]], [v[0], v[1], v[2]], [v[3], v[4], v[5]], j.pscl)
                    };
                    match parent_w {
                        Some(pw) => concat(&pw, &local),
                        None => local,
                    }
                }
                JointKind::Opaque => match (parent_w, j.parent) {
                    (Some(pw), p) => {
                        let par = &cp.joints[p as usize];
                        match inverse(&par.world) {
                            Some(ip) => concat(&pw, &concat(&ip, &j.world)),
                            None => j.world,
                        }
                    }
                    (None, _) => match pr {
                        Some(pr) => blend_mtx(&pr.world, &j.world, if interp { t } else { 1.0 + t }, MAX_STEP, 1.5).unwrap_or(j.world),
                        None => j.world,
                    },
                },
            };
            self.worlds.push(w);
        }
        for s in &cp.slots {
            let r = match s {
                SlotRec::Rigid { cam, joint } => self.view_of(cp, *cam).map(|v| concat(&v, &self.worlds[*joint as usize])),
                SlotRec::Skinned { cam, bones, right } => self.view_of(cp, *cam).and_then(|v| {
                    let mut sum = [[0.0f32; 4]; 3];
                    for b in &cp.bones[bones.start as usize..bones.end as usize] {
                        let w = &self.worlds[b.joint as usize];
                        let m = match &b.envelope {
                            Some(e) => concat(w, e),
                            None => *w,
                        };
                        for r in 0..3 {
                            for c in 0..4 {
                                sum[r][c] += m[r][c] * b.weight;
                            }
                        }
                    }
                    let rm = match right {
                        RightRec::None => None,
                        RightRec::Fixed(m) => Some(*m),
                        RightRec::InvWorld { x, m } => inverse(&self.worlds[*x as usize]).map(|i| concat(&i, &self.worlds[*m as usize])),
                        RightRec::InvWorldEnv { x, env, m } => inverse(&concat(&self.worlds[*x as usize], env)).map(|i| concat(&i, &self.worlds[*m as usize])),
                    };
                    if !matches!(right, RightRec::None) {
                        sum = concat(&sum, &rm?);
                    }
                    Some(concat(&v, &sum))
                }),
            };
            self.slots.push(r.and_then(|m| inverse_transpose(&m).map(|n| (m, n))));
        }
    }

    /// The view at the evaluated phase of camera `cam` (1-based), or its own view when it does not move.
    fn view_of(&self, cp: &PoseFrame, cam: u32) -> Option<Mtx34> {
        let i = cam.checked_sub(1)? as usize;
        if self.cams.get(i)?.held {
            return None; // held layers keep the game's matrices
        }
        Some(self.views.get(i).copied().flatten().unwrap_or(cp.cams.get(i)?.view))
    }

    /// Rewrite draw `i`'s uniform block `out` (already a copy of the draw's own) for the evaluated phase, plus the
    /// widening. Writes only what it changes, from its own inputs.
    pub fn patch(&mut self, cur: &Frame, i: usize, out: &mut [u8], widen: Option<Widen>, posed: bool) {
        let d = &cur.draws[i];
        if d.copy.is_some() {
            return;
        }
        let b = base(d);
        let pos = b + 64;
        let nrm = b + 64 + 480 + 480;
        if out.len() < nrm + 480 {
            return;
        }
        self.stats.draws += 1;
        let ortho = rd(out, b + 60) == 1.0;
        // whether this call wrote the projection rows from the solver (else `out` may still hold the previous
        // present's widened values: the renderer copies the uniforms once per prepared frame)
        let mut proj_set = false;
        if posed && !self.cut && !ortho && !self.slots.is_empty() || (posed && !self.cut && !ortho && d.cam != 0) {
            let cam = d.cam.checked_sub(1).map(|c| c as usize);
            let held = cam.is_some_and(|c| self.cams.get(c).is_some_and(|x| x.held));
            if held {
                self.stats.held += 1;
            } else {
                let mut any = false;
                let indexed = d.config.shader.attrs[0].attr_type != crate::consts::GX_NONE as u8;
                let cur_slot = d.current_pn_mtx as usize / 3;
                let carry = cam.and_then(|c| Some((self.views.get(c).copied().flatten()?, self.inv_views.get(c).copied().flatten()?)));
                for k in 0..10 {
                    let tag = d.pose[k];
                    if tag != 0 {
                        if let Some(Some((m, n))) = self.slots.get(tag as usize - 1) {
                            put_mtx(out, pos + k * 48, m);
                            put_mtx(out, nrm + k * 48, n);
                            any = true;
                            continue;
                        }
                    }
                    if (indexed || k == cur_slot) && d.palette.is_none() {
                        if let Some((v, iv)) = carry {
                            let m = mtx_at(&d.uniform, pos + k * 48);
                            let m2 = concat(&v, &concat(&iv, &m));
                            put_mtx(out, pos + k * 48, &m2);
                            if let Some(n) = inverse_transpose(&m2) {
                                put_mtx(out, nrm + k * 48, &n);
                            }
                            if tag == 0 {
                                self.stats.carried += 1;
                            }
                        }
                    }
                }
                if any {
                    self.stats.posed += 1;
                }
                if let Some(Some(p)) = cam.and_then(|c| self.projs.get(c)) {
                    proj_set = true;
                    for r in 0..4 {
                        for c in 0..4 {
                            // row 2 is the depth row the uniform stores reversed (state::USE_REVERSED_Z): keep it
                            if r != 2 {
                                wr(out, b + (r * 4 + c) * 4, p[r][c]);
                            }
                        }
                    }
                }
            }
        }
        if let Some(w) = widen {
            if self.widen_ok.get(i).copied().unwrap_or(false) {
                // from the draw's own row (never `out`'s: a frame presented N times would be widened N times, the
                // picture shrinking toward a vertical line between ticks at high frame rates)
                for c in 0..4 {
                    let o = b + c * 4;
                    let v = if proj_set { rd(out, o) } else { rd(&d.uniform, o) };
                    wr(out, o, v * w.factor);
                }
                self.stats.widened += 1;
            }
        }
    }

    /// Whether `plan` ran for the current pair.
    pub fn planned(&self) -> bool {
        self.planned
    }
    pub fn reset(&mut self) {
        self.planned = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_blend_endpoints() {
        let a = srt([1.0; 3], [0.1, 0.2, 0.0], [0.0, 0.0, -100.0], None);
        let b = srt([1.0; 3], [0.1, 0.3, 0.0], [5.0, 0.0, -110.0], None);
        assert!(max_diff(&blend_view(&a, &b, 0.0).unwrap(), &a) < 1e-3);
        assert!(max_diff(&blend_view(&a, &b, 1.0).unwrap(), &b) < 1e-3);
        assert!(!view_cut(&a, &b));
        let c = srt([1.0; 3], [0.1, 0.3, 0.0], [500.0, 0.0, -110.0], None);
        assert!(view_cut(&a, &c));
    }

    #[test]
    fn widen_factor() {
        let w = Widen::for_aspect(16.0 / 9.0).unwrap();
        assert!((w.factor - 219.0 / 320.0).abs() < 1e-6);
        assert!(Widen::for_aspect(4.0 / 3.0).is_none());
    }
}

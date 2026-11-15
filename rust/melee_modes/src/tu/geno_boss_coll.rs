//! Boss-carried collision (docs/specs/geno-v5.2-boss.md §8 "moving collision players stand on";
//! notes `docs/rust-port/notes/geno-boss.md`, lane C).
//!
//! Brawl's bosses that players ride (Porky's back, Duon, the Falcon Flyer's deck) carry their walkable surface on
//! bones: the boss file's `grColDataSet` maps collision point i to a model bone, and the surface is the polyline
//! through those points, moved by the animation. Here a part's [`PartColl`] names the joints; at each stage load the
//! line set is appended to the stage's collision as its own joint (one floor line per segment, pass-through, linked
//! as one surface: `ground::BOSS_COLL`), and every frame the host hands the world points to [`set_points`], which
//! moves the joint exactly as explore.rs moves the Subspace water-wheel platforms (`movers_tick`): the previous place
//! kept for the riders (`mpLib_80057424`), the vertices written, the joint's bounds refreshed
//! (`mpJointUpdateBounding`). The points live in `geno_boss::State` (hashed), the vertices in the game heap: both
//! roll back.
//!
//! Without an actor host for a riding boss yet, `MELEE_BOSS_PLATFORM=porky|duon|falconflyer` runs a harness on
//! Final Destination: the boss's bones from the owner's local files, one clip looping, the body walking back and
//! forth across the stage, its surface live. Nothing here names a disc path or prints one.

use super::geno_boss::{MAX_COLL_PTS, S};
use super::geno_boss_ir::J;
use crate::tu::petey_model as pm;
use melee_types::{MapCollData, MapJoint, MapLine, MapLineRange, Vec2, LINE_FLAG_ENABLED, LINE_FLAG_PLATFORM};
const MAP_LINE_GROUP_FLOOR: usize = melee_types::MapLineGroup_Floor as usize;

/// A part's line set: the joints carrying its points, in point order (IR `platforms[].point_joints`). A name
/// `boneN` is the model's bone of index N.
#[derive(Clone, Debug, PartialEq)]
pub struct PartColl {
    pub id: String,
    pub joints: Vec<String>,
    pub pass_through: bool,
}

/// The IR's platform on part `part_id` (`boss.platforms[]`), if any.
pub fn platform_of(boss: &J, part_id: &str) -> Option<PartColl> {
    let p = boss.get("platforms").arr().iter().find(|p| p.get("part").str() == part_id)?;
    let joints: Vec<String> = p.get("point_joints").arr().iter().map(|j| j.str().to_string()).filter(|s| !s.is_empty()).collect();
    if joints.len() < 2 || joints.len() > MAX_COLL_PTS {
        return None;
    }
    Some(PartColl { id: p.get("id").str().to_string(), joints, pass_through: p.get("kind").str() != "solid" })
}

// ------------------------------------------------------------------------------------------------- collision

/// The appended joint of the current stage (-1: none) and its first vertex / line.
static mut JOINT: i32 = -1;
static mut VTX0: i32 = 0;
static mut LINE0: i32 = 0;
/// Points wanted at the next stage load (0: none) and their kind.
static mut WANT_N: usize = 0;
static mut WANT_PASS: bool = true;

/// Ask for a line set of `n` points at the next stage load (a host calls it before the match; 0 = none).
pub fn want(n: usize, pass_through: bool) {
    unsafe {
        WANT_N = n.min(MAX_COLL_PTS);
        WANT_PASS = pass_through;
        if WANT_N >= 2 {
            melee_world::tu::ground::BOSS_COLL = Some(append);
            melee_world::tu::ground::BOSS_COLL_INIT = Some(init_lines);
        }
    }
}

/// Is a line set in place on this stage.
pub fn present() -> bool {
    unsafe { JOINT >= 0 }
}

/// `ground::BOSS_COLL`: the stage's collision with the line set appended as the last joint (a fresh copy in the
/// game heap, as `Ground_TTMod_BuildCollData`). The points start spaced on a far-off row so that no segment is empty
/// (`mpPruneEmptyLines` would cut it) until the host's first [`set_points`].
unsafe fn append(orig: *mut MapCollData) -> *mut MapCollData {
    JOINT = -1;
    S.coll_n = 0;
    let n = WANT_N;
    if n < 2 {
        return core::ptr::null_mut();
    }
    let (ovc, olc, ojc) = ((*orig).vert_count as usize, (*orig).line_count as usize, (*orig).joint_count as usize);
    let nl = n - 1;
    let alloc = |b: usize| melee_runtime::tu::memory::HSD_MemAlloc(b as _);
    let cd = alloc(core::mem::size_of::<MapCollData>()) as *mut MapCollData;
    let vs = alloc(core::mem::size_of::<Vec2>() * (ovc + n)) as *mut Vec2;
    let ls = alloc(core::mem::size_of::<MapLine>() * (olc + nl)) as *mut MapLine;
    let js = alloc(core::mem::size_of::<MapJoint>() * (ojc + 1)) as *mut MapJoint;
    if cd.is_null() || vs.is_null() || ls.is_null() || js.is_null() {
        eprintln!("geno_boss_coll: no heap for the boss collision; none");
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping((*orig).verts, vs, ovc);
    core::ptr::copy_nonoverlapping((*orig).lines, ls, olc);
    core::ptr::copy_nonoverlapping((*orig).joints, js, ojc);
    for i in 0..n {
        *vs.add(ovc + i) = Vec2 { x: -9000.0 + 10.0 * i as f32, y: -9000.0 };
    }
    let flags = (MapLineGroup_Floor_bit() | if WANT_PASS { LINE_FLAG_PLATFORM as u32 } else { 0 }) as u16;
    for i in 0..nl {
        let li = olc + i;
        *ls.add(li) = MapLine {
            v0_idx: (ovc + i) as u16,
            v1_idx: (ovc + i + 1) as u16,
            // one surface: a rider walks from segment to segment (floor: prev / next = left / right)
            prev_id0: if i > 0 { (li - 1) as i16 } else { -1 },
            next_id0: if i + 1 < nl { (li + 1) as i16 } else { -1 },
            prev_id1: -1,
            next_id1: -1,
            hi_flags: flags,
            lo_flags: 0,
        };
    }
    let mut j: MapJoint = core::mem::zeroed();
    j.ranges[MAP_LINE_GROUP_FLOOR] = MapLineRange { start: olc as i16, count: nl as i16 };
    j.left_bound = -9010.0;
    j.right_bound = -9000.0 + 10.0 * n as f32 + 10.0;
    j.bottom_bound = -9010.0;
    j.top_bound = -8990.0;
    j.vtx_start = ovc as i16;
    j.vtx_count = n as i16;
    *js.add(ojc) = j;
    let mut c: MapCollData = core::mem::zeroed();
    c.verts = vs;
    c.vert_count = (ovc + n) as i32;
    c.lines = ls;
    c.line_count = (olc + nl) as i32;
    c.ranges = (*orig).ranges;
    c.joints = js;
    c.joint_count = (ojc + 1) as i32;
    c.x2C = (*orig).x2C;
    *cd = c;
    JOINT = ojc as i32;
    VTX0 = ovc as i32;
    LINE0 = olc as i32;
    eprintln!("geno_boss_coll: boss line set in place ({} points, joint {})", n, ojc);
    cd
}

/// The floor kind bit of a MapLine's hi_flags (`CollLine_Floor`).
#[allow(non_snake_case)]
fn MapLineGroup_Floor_bit() -> u32 {
    melee_types::CollLine_Floor as u32
}

/// `ground::BOSS_COLL_INIT`: after `mpLibLoad`, the appended lines on (they sit outside the global line groups,
/// as the TT platforms' do: `Ground_TTMod_InitLines`).
unsafe fn init_lines(c: *mut MapCollData) {
    use melee_world::tu::mplib as mp;
    if JOINT < 0 {
        return;
    }
    let g = mp::groundCollLine;
    for i in LINE0..(*c).line_count {
        let l = (*c).lines.add(i as usize);
        (*g.add(i as usize)).inner = l;
        (*g.add(i as usize)).flags = (*l).hi_flags as u32 | LINE_FLAG_ENABLED as u32;
    }
}

/// Move the line set to `pts` (world x y, point order) for this frame. Points left of their predecessor are
/// held at it (a floor line must run left to right); the whole set is mirrored to run left to right when the boss
/// faces the other way. The state keeps the points (hashed into NHV1 while an actor or the harness is live).
pub unsafe fn set_points(pts: &[[f32; 2]]) {
    use melee_world::tu::mplib as mp;
    if JOINT < 0 || pts.len() < 2 {
        return;
    }
    let n = pts.len().min(MAX_COLL_PTS);
    let rev = pts[n - 1][0] < pts[0][0];
    let mut last = f32::MIN;
    for i in 0..n {
        let mut p = if rev { pts[n - 1 - i] } else { pts[i] };
        if p[0] < last + 0.5 {
            p[0] = last + 0.5;
        }
        last = p[0];
        S.coll_pts[i] = p;
    }
    S.coll_n = n as u8;
    mp::mpLib_80057424(JOINT);
    for i in 0..n {
        let v = mp::groundCollVtx.add(VTX0 as usize + i);
        (*v).pos.x = S.coll_pts[i][0];
        (*v).pos.y = S.coll_pts[i][1];
    }
    mp::mpJointUpdateBounding(JOINT);
}

// ------------------------------------------------------------------------------------------------- bones

/// One riding boss's surface over its clips: per clip, per frame, the walkable points in side-view x y (model
/// units), left to right.
pub struct Surface {
    pub name: &'static str,
    /// the model's horizontal axis in the side view (0 = x, 2 = z: Duon and the Falcon Flyer face along z)
    pub axis: usize,
    /// the IR point indices on the walkable top, left to right
    pub order: Vec<usize>,
    pub clips: Vec<(String, Vec<Vec<[f32; 2]>>)>,
}

/// The walkable top of a point set seen from the side (`pts` = (h, y)): the indices, left to right. Running the IR
/// showed the points are an *outline* (Duon: ground, side, top, side, ground; the Falcon Flyer: hull with an underside
/// point; Porky: top / bottom pairs), not a left-to-right polyline, so a rider stands on its upper envelope: sorted by
/// h, of points within 1 unit in h the highest kept, then any point more than `DIP` below the line through its
/// neighbours dropped (underside / ground points), repeated until none is.
pub fn upper_envelope(pts: &[[f32; 2]]) -> Vec<usize> {
    const DIP: f32 = 12.0;
    let mut ix: Vec<usize> = (0..pts.len()).collect();
    ix.sort_by(|&a, &b| pts[a][0].partial_cmp(&pts[b][0]).unwrap_or(core::cmp::Ordering::Equal).then(a.cmp(&b)));
    let mut out: Vec<usize> = vec![];
    for i in ix {
        match out.last() {
            Some(&l) if (pts[i][0] - pts[l][0]).abs() < 1.0 => {
                if pts[i][1] > pts[l][1] {
                    *out.last_mut().unwrap() = i;
                }
            }
            _ => out.push(i),
        }
    }
    loop {
        let mut cut = None;
        let mut worst = DIP;
        for k in 1..out.len().saturating_sub(1) {
            let (a, p, b) = (pts[out[k - 1]], pts[out[k]], pts[out[k + 1]]);
            let t = if (b[0] - a[0]).abs() > 1e-3 { (p[0] - a[0]) / (b[0] - a[0]) } else { 0.5 };
            let under = a[1] + t * (b[1] - a[1]) - p[1];
            if under > worst {
                worst = under;
                cut = Some(k);
            }
        }
        // the ends too: an end point far below its one neighbour is a ground / underside corner
        let n = out.len();
        if cut.is_none() && n > 2 {
            if pts[out[1]][1] - pts[out[0]][1] > 2.0 * DIP {
                cut = Some(0);
            } else if pts[out[n - 2]][1] - pts[out[n - 1]][1] > 2.0 * DIP {
                cut = Some(n - 1);
            }
        }
        match cut {
            Some(k) if out.len() > 2 => {
                out.remove(k);
            }
            _ => break,
        }
    }
    out
}

/// (harness name, archive, model, IR file) of the bosses with a carried surface.
const BOSSES: [(&str, &str, &str, &str); 3] = [
    ("porky", "090101.pac", "EnmPorky", "porky.brawl.ir.json"),
    ("duon", "350001.pac", "EnmDuon", "duon.brawl.ir.json"),
    ("falconflyer", "370301.pac", "EnmFalconflyer", "falconflyer.brawl.ir.json"),
];

/// The joints of `name`'s surface from its IR.
pub fn ir_joints(name: &str) -> Result<PartColl, String> {
    let (_, _, _, file) = BOSSES.iter().find(|b| b.0 == name).ok_or("unknown boss")?;
    let dir = super::geno_boss_ir::ir_dir().ok_or("no IR folder")?;
    let t = std::fs::read_to_string(dir.join(file)).map_err(|e| format!("IR not readable ({})", e.kind()))?;
    // the analyst files carry bare `NaN` numbers (Python's json), which the tiny reader does not take
    let ir = super::geno_boss_ir::parse_json(&t.replace("NaN", "null"))?;
    let boss = ir.get("boss");
    boss.get("platforms").arr().first().and_then(|p| platform_of(boss, p.get("part").str())).ok_or_else(|| "no platform in the IR".into())
}

/// Build `name`'s surface: the model's bones posed at every frame of every clip. Call it on a helper thread (the
/// tables must live outside the game arena).
pub fn build_surface(name: &'static str) -> Result<Surface, String> {
    let (_, pac, mdl, _) = *BOSSES.iter().find(|b| b.0 == name).ok_or("unknown boss")?;
    let pc = ir_joints(name)?;
    let rigs = pm::load_rigs(&["stage", "adventure", pac], &[(mdl, mdl, "Wait1")])?;
    let r = rigs.into_iter().next().flatten().ok_or("model not found")?;
    let mut slots = vec![];
    for j in &pc.joints {
        let s = match j.strip_prefix("bone").and_then(|k| k.parse::<u32>().ok()) {
            Some(k) => r.bone_by_index(k),
            None => r.bone_named(j),
        };
        slots.push(s.ok_or_else(|| format!("joint {j} not in the model"))?);
    }
    // the side view: the horizontal axis with the larger spread in the reference pose (Wait / GunWait, else bind)
    let refa = r.anims.iter().find(|(n, _)| n.ends_with("Wait")).map(|(_, a)| a);
    let p0: Vec<[f32; 3]> = slots.iter().map(|&s| r.bone_point(refa, 0.0, s, [0.0; 3])).collect();
    let spread = |k: usize| p0.iter().map(|p| p[k]).fold(f32::MIN, f32::max) - p0.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
    let axis = if spread(2) > spread(0) { 2 } else { 0 };
    let order = upper_envelope(&p0.iter().map(|p| [p[axis], p[1]]).collect::<Vec<_>>());
    if order.len() < 2 {
        return Err("no walkable top".into());
    }
    let mut clips = vec![];
    for (an, a) in r.anims.iter() {
        let n = (a.frames.max(1.0) as usize).min(3600);
        let mut v = Vec::with_capacity(n);
        for f in 0..n {
            v.push(order.iter().map(|&i| {
                let p = r.bone_point(Some(a), f as f32, slots[i], [0.0; 3]);
                [p[axis], p[1]]
            }).collect::<Vec<_>>());
        }
        clips.push((an.clone(), v));
    }
    if clips.is_empty() {
        clips.push(("bind".into(), vec![order.iter().map(|&i| { let p = r.bone_point(None, 0.0, slots[i], [0.0; 3]); [p[axis], p[1]] }).collect()]));
    }
    Ok(Surface { name, axis, order, clips })
}

// ------------------------------------------------------------------------------------------------- harness

/// `MELEE_BOSS_PLATFORM` (fixed for the process): the boss whose surface the harness runs, if any.
pub fn harness_boss() -> Option<&'static str> {
    static V: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        let v = std::env::var("MELEE_BOSS_PLATFORM").ok()?;
        BOSSES.iter().find(|b| b.0.eq_ignore_ascii_case(v.trim())).map(|b| b.0)
    })
}

struct Harness {
    s: Surface,
    clip: usize,
    /// model -> world: scale, y offset (the lowest point of the clip sits at `floor`)
    scale: f32,
    dy: f32,
    half: f32,
}

fn harness() -> Option<&'static Harness> {
    static H: std::sync::OnceLock<Option<Harness>> = std::sync::OnceLock::new();
    H.get_or_init(|| {
        let name = harness_boss()?;
        let r = std::thread::spawn(move || -> Result<Harness, String> {
            let s = build_surface(name)?;
            let want = std::env::var("MELEE_BOSS_PLATFORM_CLIP").ok();
            let clip = want.as_deref().and_then(|w| s.clips.iter().position(|(n, _)| n == w))
                .or_else(|| s.clips.iter().position(|(n, _)| n.contains("Walk")))
                .or_else(|| s.clips.iter().position(|(n, _)| n.contains("Wait")))
                .unwrap_or(0);
            let v = &s.clips[clip].1;
            let (mut lo, mut x0, mut x1) = (f32::MAX, f32::MAX, f32::MIN);
            for f in v {
                for p in f {
                    lo = lo.min(p[1]);
                    x0 = x0.min(p[0]);
                    x1 = x1.max(p[0]);
                }
            }
            // a surface wider than 3/4 of FD is scaled down to fit (the boss arenas are wider than Melee's stage)
            let w = (x1 - x0).max(1.0);
            let scale = std::env::var("MELEE_BOSS_PLATFORM_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(if w > 120.0 { 120.0 / w } else { 1.0 });
            let floor: f32 = std::env::var("MELEE_BOSS_PLATFORM_Y").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0);
            Ok(Harness { dy: floor - lo * scale, half: (x1 - x0) * 0.5 * scale, s, clip, scale })
        })
        .join()
        .unwrap_or_else(|_| Err("loader panicked".into()));
        match r {
            Ok(h) => {
                eprintln!("geno_boss_coll: harness {} clip {} ({} frames, {} points, scale {:.3})", h.s.name, h.s.clips[h.clip].0, h.s.clips[h.clip].1.len(), h.s.clips[h.clip].1[0].len(), h.scale);
                Some(h)
            }
            Err(e) => {
                eprintln!("geno_boss_coll: harness off ({e})");
                None
            }
        }
    })
    .as_ref()
}

/// Startup (main.rs): with `MELEE_BOSS_PLATFORM` set, load the surface now and hook the stage loads and frames.
pub fn install() {
    if harness_boss().is_none() {
        return;
    }
    let Some(h) = harness() else { return };
    want(h.s.clips[h.clip].1[0].len(), true);
    unsafe {
        melee_world::tu::grlast::BOSS_COLL_FRAME = Some(harness_tick);
        if melee_fighter::tu::fighter::BOSS_HASH.is_none() {
            melee_fighter::tu::fighter::BOSS_HASH = Some(harness_hash);
        }
    }
}

/// Speed of the harness's walk across the stage (units / frame) and its turnaround margin.
const WALK: f32 = 0.35;
const EDGE: f32 = 80.0;

/// Once per logic frame on Final Destination: the clip advances, the body walks to an edge and back, the surface
/// follows.
unsafe fn harness_tick() {
    let Some(h) = harness() else { return };
    if !present() {
        return;
    }
    let v = &h.s.clips[h.clip].1;
    let c = S.coll_clock;
    // x: a triangle wave between the edges (the body turns at an edge without mirroring: the deck stays one way)
    let span = 2.0 * (EDGE - h.half).max(1.0);
    let period = (2.0 * span / WALK) as i32;
    let t = c.rem_euclid(period.max(1)) as f32 * WALK;
    let x = if t < span { -span / 2.0 + t } else { span / 2.0 - (t - span) };
    S.coll_x = x;
    let f = &v[(c as usize) % v.len()];
    let mut pts = [[0.0f32; 2]; MAX_COLL_PTS];
    let n = f.len().min(MAX_COLL_PTS);
    for i in 0..n {
        pts[i] = [x + f[i][0] * h.scale, h.dy + f[i][1] * h.scale];
    }
    set_points(&pts[..n]);
    if c % log_every() == 0 && std::env::var_os("MELEE_BOSS_PLATFORM_LOG").is_some() {
        let p = &S.coll_pts[..n];
        let mut riders = String::new();
        each_rider(|slot, on, px, py| riders += &format!(" p{slot}:{}({px:.1},{py:.1})", if on { "ON" } else { "off" }));
        eprintln!("geno_boss_coll: frame {c} x {x:.1} surface ({:.1},{:.1})..({:.1},{:.1}){riders}", p[0][0], p[0][1], p[n - 1][0], p[n - 1][1]);
    }
    S.coll_clock = c + 1;
}

fn log_every() -> i32 {
    static V: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_BOSS_PLATFORM_LOG").ok().and_then(|v| v.parse().ok()).filter(|&n: &i32| n > 1).unwrap_or(60))
}

/// Each in-play fighter: (slot, standing on the boss's lines, x, y).
unsafe fn each_rider(mut f: impl FnMut(i32, bool, f32, f32)) {
    use crate::tu::script_game::ext as sg;
    use melee_types::*;
    let mut g = *sg::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !g.is_null() {
        let next = (*g).next as *mut HSD_GObj;
        let fp = sg::HSD_GObjGetUserData(g) as *mut Fighter;
        if !fp.is_null() && (*fp).is_sleeping() == 0 {
            let line = (*fp).coll_data.floor.index;
            let on = (*fp).ground_or_air == GA_Ground && JOINT >= 0 && line >= LINE0;
            f((*fp).player_idx as i32, on, (*fp).cur_pos.x, (*fp).cur_pos.y);
        }
        g = next;
    }
}

/// NHV1 word of the harness (the surface points and its clock).
fn harness_hash() -> Option<u32> {
    let s = unsafe { &S };
    if s.coll_n == 0 {
        return None;
    }
    let mut h: u32 = 0x811C_9DC5;
    let mut mix = |v: u32| {
        for b in v.to_le_bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    };
    mix(s.coll_clock as u32);
    for p in &s.coll_pts[..s.coll_n as usize] {
        mix(p[0].to_bits());
        mix(p[1].to_bits());
    }
    Some(h)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// With the local files: each riding boss's surface builds; prints the clips and the point spread (the
    /// polyline reading's sanity: in order, left to right, spread like a deck).
    #[test]
    fn geno_boss_coll_surfaces() {
        for (name, ..) in BOSSES {
            match std::thread::spawn(move || build_surface(name)).join().unwrap() {
                Ok(s) => {
                    println!("surface {name}: axis {} top {:?}", s.axis, s.order);
                    println!("surface {name}: {} clips: {:?}", s.clips.len(), s.clips.iter().map(|(n, v)| format!("{n}:{}", v.len())).collect::<Vec<_>>());
                    for (cn, v) in s.clips.iter().filter(|(n, _)| n.contains("Wait") || n.contains("Move") || n == "Rush").take(4) {
                        let f = &v[0];
                        println!("surface {name} {cn} f0: {}", f.iter().map(|p| format!("({:.1},{:.1})", p[0], p[1])).collect::<Vec<_>>().join(" "));
                    }
                }
                Err(e) => println!("surface {name}: skipped ({e})"),
            }
        }
    }
}

//! `melee view-golden`: the native golden match (vs_match.rs) stepped one frame per tick and drawn
//! every frame through the ported HSD draw path (melee_runtime `tu::displayfunc` -> dobj -> mobj /
//! tobj / tev / state -> pobj) into melee_gx's `GxShim`, presented in a winit window.
//!
//! ```text
//! melee view-golden [--iso DISC.iso] [--scenario pad_falcon_fox_fd] [--frames N]
//!                   [--shots 30,60,90] [--shot-dir rust/target/shots] [--hidden]
//! ```
//!
//! What is drawn: the stage's `map_head` model groups (bind pose: stage animation is not run) and
//! each fighter's costume tree (PlXxNr.dat) with the simulation's joint SRTs, matrices from the
//! ported `HSD_JObjSetupMatrix` on a copy of the tree (the sim's own tree is not touched, so
//! drawing cannot change the simulation). Lights are the stage's light set (`map_head + 0x10`).
//!
//! NOT the game's frame: the camera is a fixed approximation of the match camera (cm/camera.c's
//! CObj path is not ported), no HUD, no effects/particles, no fog, no background CObj, no
//! shadows, no zsort. Screenshots (disc art: never commit) go under rust/target/shots/.
use crate::gx_backend::{DataCache, ShimBackend};
use crate::vs_match::VsMatch;
use crate::Sim;
use melee_gx::api::GxColor;
use melee_gx::consts::*;
use melee_gx::shim::GxShim;
use melee_platform::gfx::{write_png, Presenter, EFB_HEIGHT, EFB_WIDTH};
use melee_platform::{run_windowed, Game, PadSystem, WindowConfig};
use melee_runtime::dat::DatArchive;
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::mtx::{self, Mtx, Vec3};
use melee_runtime::tu::displayfunc::{DispJoint, DispModel, HSD_TRSP_OPA, HSD_TRSP_TEXEDGE, HSD_TRSP_XLU};
use melee_runtime::tu::lobj::stage_light_descs;
use melee_runtime::tu::state::{Dat, HsdRender};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use winit::window::Window;

type Tree = JObjTree<(), ()>;

/// Matrices only: these trees have no RObjs and are never freed here.
struct NoRObj;
impl JObjHost<(), ()> for NoRObj {
    fn ik_hint_bone_length(&mut self, _: &[()]) -> Option<f32> {
        None
    }
    fn robj_update_all(&mut self, _: &mut Tree, _: JObjId) {}
    fn resolve_ik_joint1(&mut self, _: &mut Tree, _: JObjId) {}
    fn resolve_ik_joint2(&mut self, _: &mut Tree, _: JObjId) {}
    fn unref(&mut self, _: &mut Tree, _: JObjId) {}
}

fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}
fn vec3(d: &[u8], at: u32) -> Vec3 {
    Vec3 { x: bef(d, at), y: bef(d, at + 4), z: bef(d, at + 8) }
}

/// `HSD_JObjLoadJoint` for drawing: the tree (child first, then next, INSTANCE children skipped)
/// plus, per JObj, its descriptor. Allocation order matches skeleton.rs's `load_joint_sub`.
fn load_tree(d: &[u8], at: u32, tree: &mut Tree, parent: Option<JObjId>, descs: &mut Vec<u32>, depth: u32) -> Option<JObjId> {
    if at == 0 || depth > 200 || at as usize + 0x40 > d.len() {
        return None;
    }
    let j = tree.alloc();
    descs.push(at);
    let flags = be32(d, at + 4);
    if flags & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
        let c = load_tree(d, be32(d, at + 8), tree, Some(j), descs, depth + 1);
        tree.get_mut(j).child = c;
    }
    let n = load_tree(d, be32(d, at + 0xC), tree, parent, descs, depth + 1);
    let r = vec3(d, at + 0x14);
    let o = tree.get_mut(j);
    o.next = n;
    o.parent = parent;
    o.flags |= flags;
    o.rotate.x = r.x;
    o.rotate.y = r.y;
    o.rotate.z = r.z;
    o.scale = vec3(d, at + 0x20);
    o.translate = vec3(d, at + 0x2C);
    Some(j)
}

/// The descriptors of an already-loaded tree (skeleton.rs's costume tree), by walking the
/// descriptors in the same allocation order.
fn descs_of(d: &[u8], at: u32, depth: u32, out: &mut Vec<u32>) {
    if at == 0 || depth > 200 {
        return;
    }
    out.push(at);
    if be32(d, at + 4) & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
        descs_of(d, be32(d, at + 8), depth + 1, out);
    }
    descs_of(d, be32(d, at + 0xC), depth + 1, out);
}

/// A drawable: DAT data section, a tree and each JObj's descriptor (index = JObjId.0 order).
struct Drawable {
    dat: Arc<Vec<u8>>,
    roots: Vec<JObjId>,
    descs: Vec<u32>,
}

impl Drawable {
    /// World matrices (ported `HSD_JObjSetupMatrix`) on `tree`, joints in preorder.
    fn disp_joints(&self, tree: &mut Tree) -> Vec<DispJoint> {
        let mut out = Vec::new();
        let mut index = std::collections::HashMap::new();
        for &root in &self.roots {
            let mut order = Vec::new();
            // The root and its `next` chain (walk_tree visits one root and its children).
            let mut x = Some(root);
            while let Some(r) = x {
                tree.walk_tree(Some(r), &mut |j, _| order.push(j));
                x = tree.get(r).next;
            }
            for j in order {
                if index.contains_key(&j.0) {
                    continue;
                }
                tree.setup_matrix(&mut NoRObj, Some(j));
                let o = tree.get(j);
                let parent = o.parent.and_then(|p| index.get(&p.0).copied());
                index.insert(j.0, out.len());
                out.push(DispJoint { desc: self.descs[j.0 as usize], flags: o.flags, mtx: o.mtx, parent });
            }
        }
        out
    }
}

struct Stage {
    draw: Drawable,
    tree: Tree,
    map_head: u32,
}

fn load_stage(file: &[u8]) -> Result<Stage, String> {
    let a = DatArchive::parse(file).map_err(|e| format!("stage: {e}"))?;
    let head = a.public_offset("map_head").ok_or("no map_head")?;
    let d = a.data;
    let groups = be32(&d, head + 8);
    let count = be32(&d, head + 0xC);
    let mut tree = Tree::new();
    let mut descs = Vec::new();
    let mut roots = Vec::new();
    for g in 0..count.min(64) {
        // UnkStageDat_x8_t, 0x34 bytes: +0 HSD_Joint* root. Each group is its own GObj's tree:
        // load the root alone (its `next` chain belongs to the same tree in the C too).
        let root = be32(&d, groups + g * 0x34);
        if root == 0 {
            continue;
        }
        if let Some(r) = load_tree(&d, root, &mut tree, None, &mut descs, 0) {
            roots.push(r);
        }
    }
    Ok(Stage { draw: Drawable { dat: Arc::new(d), roots, descs }, tree, map_head: head })
}

struct FighterModel {
    dat: Arc<Vec<u8>>,
    descs: Vec<u32>,
}

pub struct View {
    sim: VsMatch,
    sim_alive: bool,
    stage: Stage,
    fighters: Vec<FighterModel>,
    shim: GxShim,
    cache: DataCache,
    render: HsdRender,
    presenter: Option<Presenter>,
    frame: u64,
    frames: Option<u64>,
    shots: Vec<u64>,
    shot_dir: PathBuf,
    quit: bool,
    pub failed: Option<String>,
}

/// The fixed camera (see the module note): the FD match-start framing, looking down -z.
fn camera() -> (Mtx, [[f32; 4]; 4]) {
    let eye = Vec3 { x: 0.0, y: 25.0, z: 230.0 };
    let at = Vec3 { x: 0.0, y: 15.0, z: 0.0 };
    // look-at (MTXLookAt)
    let f = mtx::vec_normalize(&mtx::vec_sub(&eye, &at));
    let up = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let cross = |a: &Vec3, b: &Vec3| Vec3 { x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x };
    let dot = |a: &Vec3, b: &Vec3| a.x * b.x + a.y * b.y + a.z * b.z;
    let l = mtx::vec_normalize(&cross(&up, &f));
    let u = cross(&f, &l);
    let view = [
        [l.x, l.y, l.z, -dot(&l, &eye)],
        [u.x, u.y, u.z, -dot(&u, &eye)],
        [f.x, f.y, f.z, -dot(&f, &eye)],
    ];
    // MTXPerspective
    let (fovy, aspect, n, fz) = (30.0f32, EFB_WIDTH as f32 / EFB_HEIGHT as f32, 1.0f32, 50000.0f32);
    let c = 1.0 / (fovy.to_radians() * 0.5).tan();
    let s = 1.0 / (fz - n);
    let proj = [
        [c / aspect, 0.0, 0.0, 0.0],
        [0.0, c, 0.0, 0.0],
        [0.0, 0.0, -n * s, -fz * n * s],
        [0.0, 0.0, -1.0, 0.0],
    ];
    (view, proj)
}

impl View {
    pub fn draw_frame(&mut self) {
        let (vmtx, proj) = camera();
        let shim = &mut self.shim;
        shim.set_copy_clear(GxColor::new(0, 0, 0, 255), 0xFF_FFFF);
        shim.set_viewport(0.0, 0.0, EFB_WIDTH as f32, EFB_HEIGHT as f32, 0.0, 1.0);
        shim.set_scissor(0, 0, EFB_WIDTH, EFB_HEIGHT);
        shim.set_projection(&proj, GX_PERSPECTIVE);
        let r = &mut self.render;
        r.invalidate_all();
        let mut g = ShimBackend { shim: &mut self.shim, cache: &mut self.cache };
        let sd = self.stage.draw.dat.clone();
        r.lobj_setup_init(&mut g, &Dat(&sd), &stage_light_descs(&Dat(&sd), self.stage.map_head), &vmtx);
        if self.frame == 0 {
            eprintln!("view-golden: lights {:?} diffuse mask {:#x}", r.lights, r.lightmask_diffuse);
        }
        // Models: the stage, then each fighter's costume (sim joint SRTs on a copy of the tree).
        let mut models: Vec<(Arc<Vec<u8>>, Vec<DispJoint>)> = Vec::new();
        let mut st = self.stage.tree.clone();
        models.push((sd.clone(), self.stage.draw.disp_joints(&mut st)));
        for (p, f) in self.sim.players.iter().zip(&self.fighters) {
            let mut t = p.skel.tree.clone();
            let d = Drawable { dat: f.dat.clone(), roots: vec![p.skel.root], descs: f.descs.clone() };
            models.push((f.dat.clone(), d.disp_joints(&mut t)));
        }
        let dms: Vec<DispModel> = models.iter().map(|(d, j)| DispModel::new(d, j.clone())).collect();
        // HSD_JObjDispAll per pass: opaque and texedge, then translucent.
        for mask in [HSD_TRSP_OPA | HSD_TRSP_TEXEDGE, HSD_TRSP_XLU] {
            for m in &dms {
                r.clear_vtx_desc(&mut g);
                r.jobj_disp_all(&mut g, m, &vmtx, mask, 0);
            }
        }
        self.shim.copy_disp(true);
    }
}

impl Game for View {
    fn tick(&mut self, _pads: &mut PadSystem) {}
    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => self.presenter = Some(p),
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        // One sim frame per presented frame (the golden's frame index = self.frame).
        if self.sim_alive {
            self.sim_alive = self.sim.step();
            if !self.sim_alive {
                eprintln!("view-golden: simulation stopped after frame {}: {:?}", self.sim.frame, self.sim.stopped);
            }
        }
        self.draw_frame();
        let frame = self.shim.take_frame();
        let p = self.presenter.as_mut().unwrap();
        p.present(&self.shim.gx.state, frame.as_deref());
        self.shim.log.clear();
        if self.frame == 0 {
            let s = self.render.stats;
            eprintln!(
                "view-golden: first frame {} draws, {} pobjs ({} envelope, {} shape-anim skipped), {} shader errors",
                frame.as_ref().map_or(0, |f| f.len()),
                s.pobjs,
                s.envelope_pobjs,
                s.shape_anim_skipped,
                p.renderer.shader_errors.len()
            );
            if let Some(e) = p.renderer.shader_errors.first() {
                eprintln!("{e}");
            }
        }
        if self.shots.contains(&self.frame) {
            let px = p.screenshot();
            let path = self.shot_dir.join(format!("native_{:04}.png", self.frame));
            match write_png(&path, EFB_WIDTH, EFB_HEIGHT, &px) {
                Ok(()) => eprintln!("view-golden: frame {} (sim frame {}) -> {}", self.frame, self.sim.frame, path.display()),
                Err(e) => eprintln!("view-golden: screenshot: {e}"),
            }
        }
        self.frame += 1;
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

pub fn view_golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee view-golden --iso DISC.iso [--frames N] [--shots 30,60] [--shot-dir DIR]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    let (stage_file, names): (&str, &[&str]) = match scenario.as_str() {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"]),
        s => {
            eprintln!("view-golden: scenario {s:?} not wired");
            return ExitCode::from(2);
        }
    };
    let built = (|| -> Result<View, String> {
        let mut disc = crate::disc::Disc::open(&iso)?;
        let sim = VsMatch::new(&mut disc, stage_file, names, 12345, 3)?;
        let stage = load_stage(&disc.read(stage_file)?)?;
        let mut fighters = Vec::new();
        for n in names {
            let (nr, sym) = crate::fighter_data::costume_for(n).ok_or("no costume")?;
            let a = DatArchive::parse(&disc.read(nr)?).map_err(|e| format!("{nr}: {e}"))?;
            let at = a.public_offset(sym).ok_or(format!("{nr}: no {sym}"))?;
            let mut descs = Vec::new();
            descs_of(&a.data, at, 0, &mut descs);
            fighters.push(FighterModel { dat: Arc::new(a.data), descs });
        }
        let shot_dir = PathBuf::from(get("--shot-dir").unwrap_or_else(|| "target/shots".into()));
        std::fs::create_dir_all(&shot_dir).map_err(|e| format!("{}: {e}", shot_dir.display()))?;
        Ok(View {
            sim,
            sim_alive: true,
            stage,
            fighters,
            shim: GxShim::new(),
            cache: DataCache::default(),
            render: HsdRender::new(),
            presenter: None,
            frame: 0,
            frames: get("--frames").and_then(|v| v.parse().ok()),
            shots: get("--shots").map_or(vec![30, 60, 90], |s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()),
            shot_dir,
            quit: false,
            failed: None,
        })
    })();
    let view = match built {
        Ok(v) => v,
        Err(e) => {
            eprintln!("view-golden: {e}");
            return ExitCode::FAILURE;
        }
    };
    let cfg = WindowConfig { title: format!("melee view-golden - {scenario}"), width: 1280, height: 960 };
    match run_windowed(view, PadSystem::headless(), cfg) {
        Ok((ticks, _)) => {
            eprintln!("view-golden: closed after {ticks} ticks");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("view-golden: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}

//! Retail menu scenes, ported from src/melee/gm and src/melee/mn and drawn through the ported HSD
//! path (the same way `hud_view` draws IfAll). The scene logic here follows the C line by line; the
//! GObj / JObj arenas are `StageAnim` (melee_runtime's JObj runtime on the DAT's descriptors).
//!
//! `Scene` is the seam the frontend plays through: `melee play` owns one retail scene at a time,
//! ticks it with the pad state, draws it, and reads its exit. The port's own kit menus
//! (gmfrontend*, gw_kit, online lobby/settings) are NOT here; `MenuHook` is where they plug in.
use crate::menu_support::CamPose;
use crate::gx_backend::{DataCache, ShimBackend};
use crate::stage_anim::StageAnim;
use melee_gx::shim::GxShim;
use melee_runtime::dat::DatArchive;
use melee_runtime::tu::displayfunc::DispModel;
use melee_runtime::tu::state::{Dat, HsdRender};
use std::sync::Arc;

pub mod mn_tables;
pub mod sis;
mod sis_tables;
pub mod title;
pub mod mainmenu;

pub use title::Title;

pub fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
pub fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}

/// What a retail scene asks of the frontend when it ends (`gm_801A4B60` + `gm_GetCurrentSceneExitData`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// Still running.
    None,
    /// Title: START pressed (exit data = the triggered buttons) -> main menu.
    TitleStart(u32),
    /// Title: other button (PC build: Y or B) -> the port's own menu (the kit hook).
    TitleKit(u32),
    /// Title: 600 frames without input -> the opening movie (attract).
    TitleTimeout,
}

/// Where the port's own menus (the user is redesigning them) plug in: the frontend calls this when a
/// retail scene exits towards a kit page. Default: nothing there.
pub trait MenuHook {
    fn open_kit(&mut self, _from: &'static str, _buttons: u32) -> bool {
        false
    }
}
pub struct NoKit;
impl MenuHook for NoKit {}

/// Append a `{joint, animjoint, matanim_joint, shapeanim_joint}` model (`StaticModelDesc`, found by its
/// four `<base>_*` symbols) to the data copy as a scene record whose animation set 0 is the animjoint,
/// and return the record's address (the `entries` of `StageAnim::new_scene`).
pub fn add_model(dat: &mut Vec<u8>, a: &DatArchive, base: &str) -> Result<u32, String> {
    let sym = |s: &str| a.public_offset(&format!("{base}_{s}"));
    let j = sym("joint").ok_or_else(|| format!("no {base}_joint"))?;
    let (aj, mj, sj) = (sym("animjoint").unwrap_or(0), sym("matanim_joint").unwrap_or(0), sym("shapeanim_joint").unwrap_or(0));
    while dat.len() % 4 != 0 {
        dat.push(0);
    }
    let at = dat.len() as u32;
    for x in [j, at + 0x10, at + 0x14, at + 0x18, aj, mj, sj] {
        dat.extend_from_slice(&x.to_be_bytes());
    }
    Ok(at)
}

pub fn draw_models(r: &mut HsdRender, shim: &mut GxShim, cache: &mut DataCache, dat: &Arc<Vec<u8>>, models: &mut [&mut StageAnim], pose: &CamPose, light: &[u32]) {
    let (vmtx, proj) = crate::menu_support::camera(pose);
    shim.set_projection(&proj, melee_gx::consts::GX_PERSPECTIVE);
    let mut g = ShimBackend { shim, cache };
    r.invalidate_all();
    r.lobj_setup_init(&mut g, &Dat(dat), light, &vmtx);
    let parts: Vec<_> = models.iter_mut().map(|m| (m.overrides(), m.disp_joints(0))).collect();
    let dms: Vec<DispModel> = parts.iter().map(|(o, j)| DispModel::new(dat, j.clone()).with_mat_anim(o)).collect();
    for m in &dms {
        r.clear_vtx_desc(&mut g);
        r.jobj_disp_all(&mut g, m, &vmtx, 7, 0);
    }
}

/// `AnimLoopSettings`: `{start_frame, end_frame, loop_frame}` (loop_frame -0.1 = do not loop).
#[derive(Clone, Copy)]
pub struct AnimLoop {
    pub start: f32,
    pub end: f32,
    pub loop_to: f32,
}

/// `mn_8022ED6C(jobj, &loop)` on the group's root: advance the animation one frame, looping per `s`.
pub fn anim_loop(m: &mut StageAnim, s: &AnimLoop) -> f32 {
    let g = m.groups.first().map_or(0, |g| g.map_id);
    anim_loop_at(m, g, 0, s)
}

/// `mn_8022ED6C(jobj, &loop)` on node `idx` of group `g`; returns the frame.
pub fn anim_loop_at(m: &mut StageAnim, g: u32, idx: u32, s: &AnimLoop) -> f32 {
    let mut cur = m.frame_at(g, idx);
    if !(s.start <= cur && cur <= s.end) {
        m.req_joint(g, idx, s.start);
    }
    if s.loop_to == -0.1 {
        cur = m.frame_at(g, idx);
        if cur < s.end {
            m.anim_all_at(g, idx);
            cur = m.frame_at(g, idx);
            if cur > s.end {
                cur = s.end;
                m.req_joint(g, idx, cur);
                m.anim_all_at(g, idx);
            }
        }
    } else {
        m.anim_all_at(g, idx);
        cur = m.frame_at(g, idx);
        if cur >= s.end {
            cur = s.loop_to + (cur - s.end);
            m.req_joint(g, idx, cur);
            m.anim_all_at(g, idx);
        }
    }
    cur
}

/// `melee dat-dump [--iso DISC] FILE [JOINT_SYMBOL]`: the public symbols of a disc file; with a joint
/// symbol, its JObj tree (DFS index, flags, SRT, DObj count, MObj/TObj counts) for porting work.
pub fn dat_dump(args: &[String]) -> std::process::ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let pos: Vec<&String> = args.iter().skip(1).filter(|a| !a.starts_with("--") && Some(a.to_string()) != get("--iso")).collect();
    let (Some(file), iso) = (pos.first(), get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok())) else {
        eprintln!("usage: melee dat-dump [--iso DISC] FILE [JOINT_SYMBOL]");
        return std::process::ExitCode::from(2);
    };
    let Some(iso) = iso else {
        eprintln!("dat-dump: no disc (--iso or GW_ISO_VANILLA)");
        return std::process::ExitCode::from(2);
    };
    let r = (|| -> Result<(), String> {
        let mut disc = crate::disc::Disc::open(&iso)?;
        let bytes = disc.read(file)?;
        let a = DatArchive::parse(&bytes).map_err(|e| format!("{file}: {e}"))?;
        match pos.get(1) {
            None => {
                for s in &a.publics {
                    println!("{:#08x} {}", s.offset, s.name);
                }
            }
            Some(sym) => {
                let at = a.public_offset(sym).ok_or(format!("no symbol {sym}"))?;
                let d = &a.data;
                fn walk(d: &[u8], at: u32, depth: usize, n: &mut u32) {
                    if at == 0 || depth > 64 {
                        return;
                    }
                    let flags = be32(d, at + 4);
                    let mut dobjs = 0;
                    let mut x = if flags & 0x2000_0000 == 0 && flags & 0x4000 == 0 && flags & 0x20 == 0 { be32(d, at + 0x10) } else { 0 };
                    let mut tobjs = 0;
                    while x != 0 && dobjs < 64 {
                        dobjs += 1;
                        let m = be32(d, x + 8);
                        let mut t = if m != 0 { be32(d, m + 8) } else { 0 };
                        while t != 0 && tobjs < 256 {
                            tobjs += 1;
                            t = be32(d, t + 4);
                        }
                        x = be32(d, x + 4);
                    }
                    println!(
                        "{:>3} {}flags {:#010x} T {:8.3} {:8.3} {:8.3} S {:6.3} {:6.3} {:6.3} R {:6.3} {:6.3} {:6.3} dobj {dobjs} tobj {tobjs}",
                        *n,
                        "  ".repeat(depth),
                        flags,
                        bef(d, at + 0x2C),
                        bef(d, at + 0x30),
                        bef(d, at + 0x34),
                        bef(d, at + 0x20),
                        bef(d, at + 0x24),
                        bef(d, at + 0x28),
                        bef(d, at + 0x14),
                        bef(d, at + 0x18),
                        bef(d, at + 0x1C)
                    );
                    *n += 1;
                    if flags & 0x1000 == 0 {
                        walk(d, be32(d, at + 8), depth + 1, n);
                    }
                    walk(d, be32(d, at + 0xC), depth, n);
                }
                let mut n = 0;
                walk(d, at, 0, &mut n);
            }
        }
        Ok(())
    })();
    match r {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("dat-dump: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn push32(d: &mut Vec<u8>, v: u32) {
    d.extend_from_slice(&v.to_be_bytes());
}
fn set32(d: &mut [u8], at: u32, v: u32) {
    d[at as usize..at as usize + 4].copy_from_slice(&v.to_be_bytes());
}

/// State of a deep copy of a model's descriptors.
#[derive(Default)]
struct Cloner {
    /// original joint descriptor -> its copy
    joints: std::collections::HashMap<u32, u32>,
    /// copied PObj descriptors whose joint / envelope references still name the originals
    pobjs: Vec<u32>,
}

impl Cloner {
    fn joint(&mut self, d: &mut Vec<u8>, at: u32, depth: u32) -> u32 {
        if at == 0 || depth > 400 {
            return 0;
        }
        while d.len() % 4 != 0 {
            d.push(0);
        }
        let new = d.len() as u32;
        self.joints.insert(at, new);
        let src: Vec<u8> = d[at as usize..at as usize + 0x40].to_vec();
        d.extend_from_slice(&src);
        let flags = be32(d, at + 4);
        if flags & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
            let c = self.joint(d, be32(d, at + 8), depth + 1);
            set32(d, new + 8, c);
        }
        let n = self.joint(d, be32(d, at + 0xC), depth);
        set32(d, new + 0xC, n);
        if flags & (melee_runtime::jobj::JOBJ_PTCL | melee_runtime::jobj::JOBJ_SPLINE) == 0 {
            // the DObj chain: copy each 0x10-byte descriptor (and its PObj chain), relinking `next` (+4)
            let mut srcs = Vec::new();
            let mut x = be32(d, at + 0x10);
            while x != 0 && srcs.len() < 256 {
                srcs.push(x);
                x = be32(d, x + 4);
            }
            let mut news: Vec<u32> = Vec::new();
            for s in &srcs {
                let at2 = d.len() as u32;
                let b: Vec<u8> = d[*s as usize..*s as usize + 0x10].to_vec();
                d.extend_from_slice(&b);
                let p = self.pobj_chain(d, be32(d, *s + 0xC));
                set32(d, at2 + 0xC, p);
                news.push(at2);
            }
            for (k, &nw) in news.iter().enumerate() {
                set32(d, nw + 4, news.get(k + 1).copied().unwrap_or(0));
            }
            set32(d, new + 0x10, news.first().copied().unwrap_or(0));
        }
        new
    }

    /// Copy a PObj descriptor chain (0x18 bytes each); the references are fixed by `finish`.
    fn pobj_chain(&mut self, d: &mut Vec<u8>, at: u32) -> u32 {
        let mut srcs = Vec::new();
        let mut x = at;
        while x != 0 && srcs.len() < 256 {
            srcs.push(x);
            x = be32(d, x + 4);
        }
        let mut news = Vec::new();
        for s in &srcs {
            let n = d.len() as u32;
            let b: Vec<u8> = d[*s as usize..*s as usize + 0x18].to_vec();
            d.extend_from_slice(&b);
            news.push(n);
        }
        for (k, &nw) in news.iter().enumerate() {
            set32(d, nw + 4, news.get(k + 1).copied().unwrap_or(0));
            self.pobjs.push(nw);
        }
        news.first().copied().unwrap_or(0)
    }

    /// Point every copied PObj's joint / envelope references at the copied joints.
    fn finish(&mut self, d: &mut Vec<u8>) {
        for p in std::mem::take(&mut self.pobjs) {
            let flags = (be32(d, p + 0xC) >> 16) as u16;
            let u = be32(d, p + 0x14);
            match flags & 0x3000 {
                0x2000 if u != 0 => {
                    // HSD_EnvelopeDesc** : NULL-terminated array of {joint, weight} arrays ending in joint == 0
                    let mut lists = Vec::new();
                    let mut k = 0;
                    while be32(d, u + 4 * k) != 0 && k < 64 {
                        lists.push(be32(d, u + 4 * k));
                        k += 1;
                    }
                    let mut new_lists = Vec::new();
                    for l in lists {
                        let at = d.len() as u32;
                        let mut e = 0;
                        loop {
                            let (j, w) = (be32(d, l + 8 * e), be32(d, l + 8 * e + 4));
                            if j == 0 {
                                break;
                            }
                            push32(d, self.joints.get(&j).copied().unwrap_or(j));
                            push32(d, w);
                            e += 1;
                        }
                        push32(d, 0);
                        push32(d, 0);
                        new_lists.push(at);
                    }
                    let arr = d.len() as u32;
                    for l in new_lists {
                        push32(d, l);
                    }
                    push32(d, 0);
                    set32(d, p + 0x14, arr);
                }
                0 if u != 0 => {
                    let nj = self.joints.get(&u).copied().unwrap_or(u);
                    set32(d, p + 0x14, nj);
                }
                _ => {}
            }
        }
    }
}

/// `n` independent instances of the `<base>_*` model as scene records for `StageAnim::new_scene`
/// (`{joint, anim sets, mat sets, shape sets}`, set 0 = the animjoint). Each instance has its own joint,
/// DObj and PObj-envelope descriptors (state is keyed by descriptor address); mesh, material and texture
/// data stay shared.
pub fn clone_model(dat: &mut Vec<u8>, a: &DatArchive, base: &str, n: usize) -> Result<Vec<u32>, String> {
    let sym = |s: &str| a.public_offset(&format!("{base}_{s}"));
    let j = sym("joint").ok_or_else(|| format!("no {base}_joint"))?;
    let (aj, mj, sj) = (sym("animjoint").unwrap_or(0), sym("matanim_joint").unwrap_or(0), sym("shapeanim_joint").unwrap_or(0));
    let mut out = Vec::new();
    for _ in 0..n {
        let mut c = Cloner::default();
        let root = c.joint(dat, j, 0);
        c.finish(dat);
        while dat.len() % 4 != 0 {
            dat.push(0);
        }
        let at = dat.len() as u32;
        for x in [root, at + 0x10, at + 0x14, at + 0x18, aj, mj, sj] {
            push32(dat, x);
        }
        out.push(at);
    }
    Ok(out)
}

/// The menus' controller view: `HSD_PadRenewMasterStatus` per port (the ported `input::PadStatus`, so
/// the stick's digital bits and the edges are the game's) and `gm_EvaluateAllControllerInputs`' maps
/// (A/START -> CONFIRM, B -> CANCEL, direction pad / stick -> ANY_UP..) with `fn_801A396C`'s repeat
/// timer (20 frames, then 8 / 4 / 2 apart as the hold grows).
pub struct MenuPad {
    st: [crate::pads::PadStatus; 4],
    held: [u32; 4],
    trigger: [u32; 4],
    repeat2: [u32; 4],
    timer: [u32; 4],
    x2c: [u32; 4],
}

pub const PAD_A: u32 = 0x100;
pub const PAD_B: u32 = 0x200;
pub const PAD_X: u32 = 0x400;
pub const PAD_Y: u32 = 0x800;
pub const PAD_START: u32 = 0x1000;
pub const PAD_L: u32 = 0x40;
pub const PAD_R: u32 = 0x20;
pub const MP_CONFIRM: u32 = 1 << 24;
pub const MP_CANCEL: u32 = 1 << 25;
pub const MP_UP: u32 = 1 << 26;
pub const MP_DOWN: u32 = 1 << 27;
pub const MP_LEFT: u32 = 1 << 28;
pub const MP_RIGHT: u32 = 1 << 29;

fn gm_map(b: u32) -> u32 {
    let mut m = b;
    if b & (PAD_A | PAD_START) != 0 {
        m |= MP_CONFIRM;
    }
    if b & PAD_B != 0 {
        m |= MP_CANCEL;
    }
    if b & (0x8 | 0x10000) != 0 {
        m |= MP_UP;
    }
    if b & (0x4 | 0x20000) != 0 {
        m |= MP_DOWN;
    }
    if b & (0x1 | 0x40000) != 0 {
        m |= MP_LEFT;
    }
    if b & (0x2 | 0x80000) != 0 {
        m |= MP_RIGHT;
    }
    m
}

impl MenuPad {
    pub fn new() -> Self {
        Self { st: Default::default(), held: [0; 4], trigger: [0; 4], repeat2: [0; 4], timer: [20; 4], x2c: [0; 4] }
    }

    /// One frame's samples (`None` = no controller): `HSD_PadRenewStatus` then `gm_EvaluateAllControllerInputs`.
    pub fn update(&mut self, raw: &[Option<crate::pads::RawPad>; 4]) {
        for i in 0..4 {
            let r = raw[i].unwrap_or_default();
            self.st[i].renew(&r);
            let prev = self.held[i];
            let button = gm_map(self.st[i].button);
            let trig_raw = gm_map(self.st[i].trigger);
            // the maps fold into trigger/release exactly as gm_801A3714 does: a mapped bit is set when
            // its source bits are in trigger
            self.held[i] = button;
            self.trigger[i] = trig_raw;
            let release = self.st[i].release != 0;
            let _ = prev;
            if self.trigger[i] != 0 || release {
                self.repeat2[i] = self.trigger[i];
                self.timer[i] = 20;
                self.x2c[i] = 0;
                continue;
            }
            if self.x2c[i] < 100 {
                self.x2c[i] += 1;
            }
            if self.timer[i] != 0 {
                self.timer[i] -= 1;
                self.repeat2[i] = 0;
                continue;
            }
            self.repeat2[i] = self.held[i];
            self.timer[i] = if self.x2c[i] >= 100 {
                2
            } else if self.x2c[i] >= 40 {
                4
            } else {
                8
            };
        }
    }

    /// `gm_GetButtonsTriggered(port)`; port 4 = all ports.
    pub fn triggered(&self, port: usize) -> u32 {
        if port >= 4 { self.trigger.iter().fold(0, |a, b| a | b) } else { self.trigger[port] }
    }
    /// `gm_801A36C0(port)`: the repeating input.
    pub fn repeated(&self, port: usize) -> u32 {
        if port >= 4 { self.repeat2.iter().fold(0, |a, b| a | b) } else { self.repeat2[port] }
    }
    /// The C-stick of a port, -1..1 (`HSD_PadCopyStatus[port].nml_subStick*`).
    pub fn sub_stick(&self, port: usize) -> (f32, f32) {
        (self.st[port].nml_sub_x, self.st[port].nml_sub_y)
    }
}

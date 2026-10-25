//! The match HUD for `melee view-golden` (ifall.c / ifstatus.c / ifstock.c / if_2F6E.c over IfAll.usd):
//! the HUD camera (`ScInfDmg_scene_data`), per-player damage digits, emblem and stock icon, and the
//! Ready / GO element, all drawn with melee_runtime's HSD display path like the stage and fighters.
//!
//! Driven from the native golden match: `Hud::step` once per sim frame (the gobj procs), `Hud::draw`
//! after the fighters. Matches the retail VS stock/time HUD for 1-4 players; NOT modelled: the death
//! "explode" digits, stamina mode, KO stock-loss / steal animations (`ScInfStc`), name tags, the
//! timer, the magnifier, coin / score modes, team colours.
use crate::cm_drive::CamPose;
use crate::gx_backend::ShimBackend;
use crate::stage_anim::StageAnim;
use melee_runtime::dat::DatArchive;
use melee_runtime::mtx::{Mtx, Vec3};
use melee_runtime::tu::displayfunc::{DispJoint, DispModel};
use melee_runtime::tu::mobj::MatOverride;
use melee_runtime::tu::state::{Dat, HsdRender};
use std::collections::HashMap;
use std::sync::Arc;

fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}

pub struct HudFile {
    pub dat: Arc<Vec<u8>>,
    pub scene: u32,
    pub num_desc: u32,
    pub mark_desc: u32,
    pub stock_desc: u32,
    pub ready_desc: u32,
    pub ready_end: Option<f32>,
    pub go_desc: u32,
    pub go_end: Option<f32>,
    /// `ScInfPnm_scene_models[0]`: the player name tag / arrow (ifnametag.c).
    pub nametag_desc: u32,
}

impl HudFile {
    pub fn load(ifall: &[u8]) -> Result<Self, String> {
        let a = DatArchive::parse(ifall).map_err(|e| format!("IfAll: {e}"))?;
        let sym = |n: &str| a.public_offset(n).ok_or(format!("IfAll: no {n}"));
        let scene = sym("ScInfDmg_scene_data")?;
        let (num, mrk, stc, cnt) = (sym("DmgNum_scene_models")?, sym("DmgMrk_scene_models")?, sym("Stc_scemdls")?, sym("ScInfCnt_scene_models")?);
        let pnm = sym("ScInfPnm_scene_models")?;
        if std::env::var_os("MELEE_VIEW_READYLOG").is_some() {
            for i in 0..8 {
                eprintln!("ScInfCnt model {i}: end {:?}", crate::vs_match::input::ready_hud_end_frame_of(ifall, i));
            }
        }
        let ready_end = crate::vs_match::input::ready_hud_end_frame_of(ifall, 3)?;
        let dat = Arc::new(a.data);
        Ok(Self {
            scene,
            num_desc: be32(&dat, num),
            mark_desc: be32(&dat, mrk),
            stock_desc: be32(&dat, stc),
            ready_desc: be32(&dat, cnt + 12),
            go_desc: be32(&dat, cnt + 16),
            go_end: crate::vs_match::input::ready_hud_end_frame_of(ifall, 4)?,
            nametag_desc: be32(&dat, pnm),
            ready_end,
            dat,
        })
    }
}

// IfDamagePlace: child index of each digit JObj under the DmgNum root.
const HUNDREDS: usize = 0;
const TENS: usize = 1;
const PERCENT: usize = 3;

struct PlayerHud {
    slot: usize,
    num: StageAnim,
    mark: StageAnim,
    stock: StageAnim,
    /// DFS index of the four digit JObjs.
    digit: [u32; 4],
    tx: [f32; 4],
    ty: [f32; 4],
    damage: i32,
    old_damage: i32,
    shake_frames: u8,
    last_hit: i32,
    force_shake: bool,
    stock_frame: f32,
}

pub struct Hud {
    file: HudFile,
    pose: CamPose,
    light: Vec<u32>,
    players: Vec<PlayerHud>,
    ready: StageAnim,
    ready_frames: f32,
    /// `fn_8016B7F8` -> `ifStatus_802F6EA4(4, ..)`: the "Go!" element, created when Ready ends.
    go: Option<(StageAnim, f32)>,
    rng: u32,
    /// ifnametag.c: per slot the arrow model and its screen position (`None` = hidden).
    tags: Vec<(StageAnim, Option<(f32, f32)>)>,
}

/// `ifAll_804A0FD8.x18[slot]` for `n` players (`ifAll_802F343C` cases 1-4; every JObj by DFS index).
fn layout_index(_n: usize, slot: usize) -> u32 {
    // VS matches lay the HUD out for four slots whatever the player count (the C build's 2-player
    // screenshots use the 4-player anchors: ifStatus_802F665C(4)).
    2 + slot as u32
}

/// `lbl_803B7864` port colours (`gm_80160968`), by port.
const PORT_COLOR: [[u8; 3]; 4] = [[0x99, 0x1A, 0x1A], [0x33, 0x33, 0x80], [0x80, 0x66, 0x00], [0x1A, 0x66, 0x1A]];

/// The stock-icon / emblem frame of a retail fighter (`gm_80168B34`, costume 0).
fn emblem_frame(name: &str) -> f32 {
    match name {
        "falcon" => 0.0,
        "fox" => 2.0,
        "falco" => 20.0,
        _ => 0.0,
    }
}

impl Hud {
    pub fn new(file: HudFile, names: &[&str]) -> Result<Self, String> {
        let d = file.dat.clone();
        let cd = be32(&d, be32(&d, file.scene + 4));
        let (e, i) = (be32(&d, cd + 0x18), be32(&d, cd + 0x1C));
        let pose = CamPose {
            eye: [bef(&d, e + 4), bef(&d, e + 8), bef(&d, e + 12)],
            interest: [bef(&d, i + 4), bef(&d, i + 8), bef(&d, i + 12)],
            fov: bef(&d, cd + 0x30),
            near: bef(&d, cd + 0x28),
            far: bef(&d, cd + 0x2C),
            aspect: bef(&d, cd + 0x34),
        };
        let light = {
            let ll = be32(&d, be32(&d, file.scene + 8));
            let desc = be32(&d, ll);
            if desc != 0 {
                vec![desc]
            } else {
                vec![]
            }
        };
        // ifAll_802F370C: the layout model's anchor joints.
        let layout_desc = be32(&d, be32(&d, file.scene));
        let mut layout = StageAnim::new_scene(d.clone(), vec![layout_desc]);
        let n = names.len();
        let mut players = Vec::new();
        for (slot, name) in names.iter().enumerate() {
            let pos = layout.joint_world_pos(0, layout_index(n, slot));
            let mut mark = StageAnim::new_scene(d.clone(), vec![file.mark_desc]);
            let mut num = StageAnim::new_scene(d.clone(), vec![file.num_desc]);
            let mut stock = StageAnim::new_scene(d.clone(), vec![file.stock_desc]);
            // ifStatus_802F61FC: the emblem (anim attached, not requested), frame 0.5 + ckind, rate 0.1,
            // port colour on the emblem's diffuse.
            mark.start_anim_set(0, 0, false);
            let ma = mark.matanim_at(0, 0, &[0]);
            mark.tex_anim(0, 1, ma, 0.5 + emblem_frame(name), 0.1);
            mark.set_translate(0, 0, Vec3 { x: pos.x + 0.25, y: pos.y, z: pos.z });
            mark.set_diffuse(0, 1, PORT_COLOR[slot.min(3)]);
            // ifStatus_802F5EC0: the damage number.
            num.start_map_anim(0, 0);
            // The percent sign: texanim of the fourth matanim child, frame 0 (frame 1 = the "HP" sign of stamina).
            let pm = num.matanim_at(0, 0, &[0, 1, 1, 1]);
            num.tex_anim(0, 4, pm, 0.0, 0.0);
            num.set_translate(0, 0, pos);
            let digit = [1u32, 2, 3, 4];
            let (mut tx, mut ty) = ([0.0f32; 4], [0.0f32; 4]);
            for k in 0..4 {
                let t = num.translate_of(0, digit[k]);
                tx[k] = t.x;
                ty[k] = t.y;
            }
            // ifStock_802F98E8 mode 1 (one icon): icons 2-7, joints 8 and 14 hidden.
            stock.start_map_anim(0, 0);
            stock.set_translate(0, 0, pos);
            for h in [2u32, 3, 4, 5, 6, 7, 8, 14] {
                stock.set_hidden_all(0, h, true);
            }
            // gm_80168B34: ckind + costume * 30 (the costume is the slot for the golden scenarios).
            let frame = emblem_frame(name) + slot as f32 * 30.0;
            let mut p = PlayerHud { slot, num, mark, stock, digit, tx, ty, damage: 0, old_damage: 1, shake_frames: 0, last_hit: 0, force_shake: false, stock_frame: frame };
            p.stock_update();
            p.num_update(&[0.0; 8]);
            players.push(p);
        }
        // ifStatus_802F6EA4(1, ..): the Ready / GO element.
        let mut ready = StageAnim::new_scene(d.clone(), vec![file.ready_desc]);
        ready.start_map_anim(0, 0);
        // NameTag_Create: scale 10, the matanim/anim set attached, ReqAnimAll(frame) picking the label
        // (`un_802FC9B4`: 5 * port for a human slot of a free-for-all; `inlineA1`'s 20.. is for custom name tags), one AnimAll.
        let mut tags = Vec::new();
        for slot in 0..n {
            let mut t = StageAnim::new_scene(d.clone(), vec![file.nametag_desc]);
            t.start_anim_set(0, 0, false);
            t.set_scale(0, 0, Vec3 { x: 10.0, y: 10.0, z: 10.0 });
            t.req_joint(0, 0, 5.0 * slot as f32);
            t.step();
            if std::env::var_os("MELEE_VIEW_TAGDUMP").is_some() && slot == 0 {
                t.dump(0);
            }
            tags.push((t, None));
        }
        Ok(Self { file, pose, light, players, ready, ready_frames: 1.0, go: None, rng: 0x2545F491, tags })
    }

    fn rand(&mut self) -> f32 {
        // `offset_rand`: a render-side generator (the sim's HSD_Randf stream is not touched).
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng >> 8) as f32 / (1u32 << 24) as f32 - 0.5
    }

    /// `fn_802FCC44`: each slot's tag screen position (`lbVector_WorldToScreen` of the fighter's head, 640x480),
    /// or `None` when the tag is hidden. The tags are removed when the Go element is created (`un_802FD428`).
    /// `rebirth[i]` is `Player_80036058` (`ftLib_80086F4C`: the fighter is on its rebirth platform): after the
    /// Go element the tag stays visible only then (`un_804D6D70` cleared, no custom name tag).
    pub fn set_nametags(&mut self, pos: &[Option<(f32, f32)>], rebirth: &[bool]) {
        let live = self.go.is_none();
        if std::env::var_os("MELEE_VIEW_TAGLOG").is_some() { eprintln!("SETTAGS live {live} pos {pos:?} rebirth {rebirth:?}"); }
        for (i, (t, p)) in self.tags.iter_mut().enumerate() {
            *p = if live || rebirth.get(i).copied().unwrap_or(false) { pos.get(i).copied().flatten() } else { None };
            if let Some((x, y)) = *p {
                t.set_translate(0, 0, Vec3 { x, y: -y, z: 0.0 });
            }
        }
    }

    /// One logic frame of the HUD gobj procs. `damage[slot]` is the sim's `Player_GetDamage`.
    pub fn step(&mut self, damage: &[i32]) {
        self.ready.step();
        self.ready_frames += 1.0;
        if self.go.is_none() && self.ready_frames > self.file.ready_end.unwrap_or(f32::MAX) && self.ready_frames <= self.file.ready_end.unwrap_or(0.0) + 1.5 {
            let mut g = StageAnim::new_scene(self.file.dat.clone(), vec![self.file.go_desc]);
            g.start_map_anim(0, 0);
            // The new gobj's anim proc runs later in the frame it was created in (the C shot shows the
            // element one animation frame in).
            g.step();
            self.go = Some((g, 2.0));
        } else if let Some((g, n)) = &mut self.go {
            g.step();
            *n += 1.0;
        }
        for i in 0..self.players.len() {
            let r: Vec<f32> = (0..8).map(|_| self.rand()).collect();
            let p = &mut self.players[i];
            // ifStatus_802F5B48
            p.old_damage = p.damage;
            p.damage = damage.get(p.slot).copied().unwrap_or(0).clamp(0, 999);
            p.force_shake = false;
            if p.damage > p.old_damage {
                p.force_shake = true;
                p.last_hit = (p.old_damage - p.damage).abs();
            }
            p.num_update(&r);
            p.stock_update();
        }
    }

    /// The HUD pass: camera, light, then every gobj's JObj in creation order, each transparency class
    /// in turn (like the stage's gx-link schedule).
    pub fn draw(&mut self, r: &mut HsdRender, g: &mut ShimBackend, camera: impl Fn(&CamPose) -> (Mtx, [[f32; 4]; 4])) {
        let (vmtx, proj) = camera(&self.pose);
        g.shim.set_projection(&proj, melee_gx::consts::GX_PERSPECTIVE);
        let dat = self.file.dat.clone();
        r.invalidate_all();
        r.lobj_setup_init(g, &Dat(&dat), &self.light, &vmtx);
        let mut models: Vec<(HashMap<u32, MatOverride>, Vec<DispJoint>)> = Vec::new();
        for p in &mut self.players {
            for a in [&mut p.mark, &mut p.num, &mut p.stock] {
                models.push((a.overrides(), a.disp_joints(0)));
            }
        }
        if self.file.ready_end.map_or(true, |e| self.ready_frames <= e) {
            models.push((self.ready.overrides(), self.ready.disp_joints(0)));
        }
        if let Some((g, n)) = &mut self.go {
            if self.file.go_end.map_or(true, |e| *n <= e) {
                models.push((g.overrides(), g.disp_joints(0)));
            }
        }
        let dms: Vec<DispModel> = models.iter().map(|(o, j)| DispModel::new(&dat, j.clone()).with_mat_anim(o)).collect();
        for code in [0u32, 1, 2] {
            for m in &dms {
                r.clear_vtx_desc(g);
                r.jobj_disp_all(g, m, &vmtx, 1 << code, 0);
            }
        }
        // The name tags: their own orthographic camera (`nametag_CObjDesc`: 640x480, eye z 300), gx link 9.
        let mut tag_models: Vec<(HashMap<u32, MatOverride>, Vec<DispJoint>)> = Vec::new();
        for (t, p) in &mut self.tags {
            if p.is_some() {
                let dj = t.disp_joints(0);
                if std::env::var_os("MELEE_VIEW_TAGLOG").is_some() { eprintln!("TAGMTX {:?}", dj.iter().map(|j| (j.mtx[0][3], j.mtx[1][3], j.mtx[0][0])).collect::<Vec<_>>()); }
                tag_models.push((t.overrides(), dj));
            }
        }
        if std::env::var_os("MELEE_VIEW_TAGLOG").is_some() { eprintln!("TAGDRAW {}", tag_models.len()); }
        if !tag_models.is_empty() {
            let (n, f) = (0.1f32, 32768.0f32);
            let (top, bottom, left, right) = (0.0f32, -480.0f32, 0.0f32, 640.0f32);
            let proj = [
                [2.0 / (right - left), 0.0, 0.0, -(right + left) / (right - left)],
                [0.0, 2.0 / (top - bottom), 0.0, -(top + bottom) / (top - bottom)],
                [0.0, 0.0, -1.0 / (f - n), -f / (f - n)],
                [0.0, 0.0, 0.0, 1.0],
            ];
            g.shim.set_projection(&proj, melee_gx::consts::GX_ORTHOGRAPHIC);
            let tview: Mtx = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, -300.0]];
            let tdms: Vec<DispModel> = tag_models.iter().map(|(o, j)| DispModel::new(&dat, j.clone()).with_mat_anim(o)).collect();
            for code in [0u32, 1, 2] {
                for m in &tdms {
                    r.clear_vtx_desc(g);
                    r.jobj_disp_all(g, m, &tview, 1 << code, 0);
                }
            }
        }
    }
}

impl PlayerHud {
    /// `fn_802F9410` mode 1.
    fn stock_update(&mut self) {
        self.stock.req_joint(0, 1, 0.0);
        self.stock.tobj_req(0, 1, self.stock_frame, 0.0);
        self.stock.step();
    }

    /// `ifStatus_802F4EDC`, normal (not exploding, not healing) path.
    fn num_update(&mut self, rnd: &[f32]) {
        let n = &mut self.num;
        n.step();
        let d = self.damage;
        let digits = [(d % 1000) / 100, (d % 100) / 10, d % 10];
        let ma = n.matanim_at(0, 0, &[0]);
        for (k, &dg) in digits.iter().enumerate() {
            n.tex_anim(0, self.digit[k], ma, 2.0 * dg as f32, 0.0);
        }
        n.set_hidden_all(0, self.digit[TENS], digits[0] == 0 && digits[1] == 0);
        // ifStatus_UpdateDamageDisplay
        n.set_hidden_all(0, self.digit[HUNDREDS], digits[0] == 0);
        if self.old_damage != d {
            let f = (d.clamp(0, 300) as f32) / 300.0;
            let c = |s: f32, e: f32| ((f * (e - s) + s) as i32) as u8;
            let rgb = [c(255.0, 80.0), c(255.0, 0.0), c(255.0, 0.0)];
            for k in 0..4 {
                n.set_diffuse(0, self.digit[k], rgb);
            }
        }
        if n.joint_anim_running(0) {
            for k in 0..4 {
                let t = n.translate_of(0, self.digit[k]);
                self.tx[k] = t.x;
                self.ty[k] = t.y;
            }
        }
        let one = |v: i32| if v == 1 { 0.5069f32 } else { 0.0 };
        let (ones_off, tens_off, hund_off) = (one(d % 10), one((d % 100) / 10), one((d % 1000) / 100));
        let set_x = |n: &mut StageAnim, idx: u32, x: f32| {
            let mut t = n.translate_of(0, idx);
            t.x = x;
            n.set_translate(0, idx, t);
        };
        set_x(n, self.digit[PERCENT], self.tx[PERCENT] - ones_off);
        let mut off = ones_off + tens_off;
        set_x(n, self.digit[TENS], self.tx[TENS] + off);
        off += tens_off + hund_off;
        set_x(n, self.digit[HUNDREDS], self.tx[HUNDREDS] + off);
        // Shake (ifStatus_802F4B84).
        if self.force_shake {
            self.shake_frames = 10;
        }
        if self.shake_frames != 0 {
            if self.shake_frames == 1 {
                for k in 0..4 {
                    let mut t = n.translate_of(0, self.digit[k]);
                    t.x = self.tx[k];
                    t.y = self.ty[k];
                    n.set_translate(0, self.digit[k], t);
                }
                self.shake_frames = 0;
            } else {
                let mag = (0.1014 * self.last_hit as f32).clamp(0.1014, 1.5207);
                for k in 0..4 {
                    let mut ox = mag * 2.0 * rnd.get(2 * k).copied().unwrap_or(0.0);
                    let mut oy = mag * 2.0 * rnd.get(2 * k + 1).copied().unwrap_or(0.0);
                    ox += if ox < 0.0 { -0.2028 } else { 0.2028 };
                    oy += if oy < 0.0 { -0.2028 } else { 0.2028 };
                    let mut t = n.translate_of(0, self.digit[k]);
                    t.x = self.tx[k] + ox;
                    t.y = self.ty[k] + oy;
                    n.set_translate(0, self.digit[k], t);
                }
                self.shake_frames -= 1;
            }
        }
    }
}

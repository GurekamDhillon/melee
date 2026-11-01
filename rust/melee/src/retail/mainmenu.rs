//! mnmain.c: the retail main menu and its list screens (Main, 1P, VS, Trophies, Options, Data,
//! Regular Match, Stadium, Special Melee, Records) over MnMaAll.usd.
//!
//! The C is a set of GObjs; here each is a field and `tick` runs their procs in the C's process-link
//! order (think 1, camera 3, light 4, back 5, panel 6, con 7). A proc created during a frame does not
//! run until the next (`HSD_GObj_80390CD4`'s `flags_3` stamp), which `fresh` reproduces. The model
//! instances are `StageAnim` groups: one `MenMainConTop` per menu screen (`mn_8022B3A0`) with one
//! `MenMainCursor` per unlocked option attached under that screen's option JObjs.
//!
//! NOT here: the description text (HSD_SisLib), the C-stick camera swing, sounds, the screens that
//! hang off the list items (Event, Rules, Name entry, Snapshots, Archives, Sound Test, Info, Records
//! pages, Settings pages): those end this scene with an `Exit::Screen`.
use super::mn_tables as T;
use super::*;
use crate::menu_support::AnimLights;
use super::sis::{GlyphCache, SisArchive, SisEnv, Text};

const BACK: u32 = 0;
const PANEL: u32 = 1;
const CON0: u32 = 2;
const CON_POOL: u32 = 6;
const CUR0: u32 = 8;
const CUR_POOL: u32 = 64;

pub const KIND_MAIN: u8 = 0;
pub const KIND_1P: u8 = 1;
pub const KIND_VS: u8 = 2;
pub const KIND_TOY: u8 = 3;
pub const KIND_SETTINGS: u8 = 4;
pub const KIND_DATA: u8 = 5;
pub const KIND_REG: u8 = 6;
pub const KIND_STADIUM: u8 = 9;
pub const KIND_SPECIAL: u8 = 12;
pub const KIND_RECORDS: u8 = 28;

const S_IDLE: u8 = 0;
const S_ENTER_TO: u8 = 1;
const S_EXIT_FROM: u8 = 2;
const S_EXIT_TO: u8 = 3;
const S_ENTER_FROM: u8 = 4;
const S_5: u8 = 5;

/// `MenuKindData`: the per-screen table `mn_803EB6B0`.
struct KindData {
    loops: &'static [AnimLoop],
    start: f32,
    count: u8,
    /// SIS string ids of the hover description, per selection
    desc: &'static [u16],
}
fn kind_data(k: u8) -> KindData {
    let none: &'static [AnimLoop] = &[];
    match k {
        0 => KindData { loops: &T::L_803EB3FC, start: 0.0, count: 5, desc: &T::D_803EB660 },
        1 => KindData { loops: &T::L_803EB444, start: 20.0, count: 5, desc: &T::D_803EB66C },
        2 => KindData { loops: &T::L_803EB48C, start: 40.0, count: 5, desc: &T::D_803EB678 },
        3 => KindData { loops: &T::L_803EB4C8, start: 60.0, count: 4, desc: &T::D_804D4B38 },
        4 => KindData { loops: &T::L_803EB4F8, start: 80.0, count: 6, desc: &T::D_803EB684 },
        5 => KindData { loops: &T::L_803EB540, start: 100.0, count: 5, desc: &T::D_803EB690 },
        6 => KindData { loops: &T::L_803EB57C, start: 120.0, count: 3, desc: &T::D_804D4B30 },
        9 => KindData { loops: &T::L_803EB5C4, start: 160.0, count: 3, desc: &T::D_804D4B48 },
        12 => KindData { loops: &T::L_803EB5E8, start: 200.0, count: 10, desc: &T::D_803EB69C },
        13 => KindData { loops: none, start: 0.0, count: 7, desc: &[] },
        15 => KindData { loops: none, start: 0.0, count: 6, desc: &[] },
        28 => KindData { loops: &T::L_803EB5A0, start: 240.0, count: 3, desc: &[] },
        _ => KindData { loops: none, start: 0.0, count: 1, desc: &[] },
    }
}

/// `mn_8022BFBC`: the light colour of a colour index.
const LIGHT_COLORS: [[u8; 3]; 5] = [[0x5A, 0x73, 0xFF], [0xFF, 0x5A, 0x41], [0x0E, 0xD2, 0x41], [0xF0, 0xC8, 0x5A], [0x9B, 0x41, 0xFF]];

/// `mn_8022C010`: the light colour index of a screen.
fn light_index(kind: u8, sel: u16) -> usize {
    if kind == KIND_MAIN {
        return sel as usize;
    }
    match kind {
        1 | 6 | 7 | 8 | 9 | 10 | 33 => 0,
        2 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 => 1,
        3 => 2,
        4 | 19 | 20 | 21 | 22 | 23 | 24 => 3,
        _ => 4,
    }
}

/// What the main menu hands back when it ends (`MenuExitData.pending_mode` through `gm_801A4B60`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    None,
    /// B on the main menu: back to the title (GM_TITLE).
    Title,
    /// A game mode of the list (`pending_mode`): "Vs" (Melee), "Tournament", "Classic", "Adventure",
    /// "AllStar", "Training", "TargetTest", "HomeRun", "ToyGallery", "ToyLottery", "ToyCollection", ...
    Mode(&'static str),
    /// A screen the list opens inside this scene's mode (Event, Rules, Name entry, a Settings page...).
    Screen(&'static str),
}

#[derive(Clone, Copy)]
enum Think {
    List(u8),
}

struct Con {
    g: u32,
    kind: u8,
    hovered: u16,
    state: u8,
    tree: [u32; 42],
    /// created this frame: its proc waits for the next one.
    fresh: bool,
    /// proc is `fn_8022AF10` (exit only) rather than `fn_8022AFEC`.
    exit_only: bool,
    /// the hover description text (`MainMenuData.description`)
    description: Option<Text>,
}

struct Panel {
    cur: u8,
    prev: u8,
    state: u8,
    n4: u32,
    n41: u32,
}

struct Flow {
    cur: u8,
    prev: u8,
    hovered: u16,
    entering: bool,
    light_color: usize,
    light_lerp: i32,
}

pub struct MainMenu {
    dat: Arc<Vec<u8>>,
    sa: StageAnim,
    lights: AnimLights,
    /// the first POINT light's colour, which the menu drives (`LerpLightColor`)
    pose: CamPose,
    erase: [u8; 4],
    /// the scene fog (`ScMenMain_fog`): type, start, end
    fog: (u32, f32, f32),
    flow: Flow,
    cooldown: i32,
    panel: Panel,
    cons: Vec<Con>,
    think: Option<Think>,
    think_fresh: bool,
    con_next: u32,
    cur_next: u32,
    pad: MenuPad,
    pub exit: Exit,
    unlock_all: bool,
    render: HsdRender,
    cache: DataCache,
    pub frame: u64,
    sis: Option<SisArchive>,
    atlas: Vec<u8>,
    glyphs: GlyphCache,
}

impl MainMenu {
    /// `mnMain_Scene_OnEnter(MenuEnterData{menu_kind, hovered_selection, load_assets})`.
    pub fn new(mnmaall: &[u8], sis: Option<(&[u8], Vec<u8>)>, kind: u8, hovered: u16, unlock_all: bool) -> Result<Self, String> {
        let a = DatArchive::parse(mnmaall).map_err(|e| format!("MnMaAll: {e}"))?;
        let mut data = a.data.clone();
        let mut entries = Vec::new();
        entries.extend(clone_model(&mut data, &a, "MenMainBack_Top", 1)?);
        entries.extend(clone_model(&mut data, &a, "MenMainPanel_Top", 1)?);
        entries.extend(clone_model(&mut data, &a, "MenMainConTop_Top", CON_POOL as usize)?);
        entries.extend(clone_model(&mut data, &a, "MenMainCursor_Top", CUR_POOL as usize)?);
        let dat = Arc::new(data);
        let sym = |n: &str| a.public_offset(n).ok_or(format!("MnMaAll: no {n}"));
        let cd = sym("ScMenMain_cam_int1_camera")?;
        let (e, i) = (be32(&dat, cd + 0x18), be32(&dat, cd + 0x1C));
        let pose = CamPose {
            eye: [bef(&dat, e + 4), bef(&dat, e + 8), bef(&dat, e + 12)],
            interest: [bef(&dat, i + 4), bef(&dat, i + 8), bef(&dat, i + 12)],
            fov: bef(&dat, cd + 0x30),
            near: bef(&dat, cd + 0x28),
            far: bef(&dat, cd + 0x2C),
            aspect: bef(&dat, cd + 0x34),
        };
        let fog = sym("ScMenMain_fog")?;
        let c = fog + 0x10;
        let erase = [dat[c as usize], dat[c as usize + 1], dat[c as usize + 2], dat[c as usize + 3]];
        let list = be32(&dat, sym("ScMenMain_scene_lights")?);
        let _ = list;
        let lights = AnimLights::new_scene(&dat, sym("ScMenMain_scene_lights")?);
        let dat_for_fog = dat.clone();
        let mut sa = StageAnim::new_scene(dat.clone(), entries);
        // mn_80229B2C: the background
        sa.start_anim_set(BACK, 0, false);
        sa.req_joint(BACK, 0, 0.0);
        sa.anim_all_at(BACK, 0);
        // mn_80229DC0: the panel
        sa.start_anim_set(PANEL, 0, false);
        let n4 = sa.dfs_node(PANEL, 0, 4).ok_or("panel: no joint 4")?;
        let n41 = sa.dfs_node(PANEL, 0, 0x29).ok_or("panel: no joint 41")?;
        let mut m = Self {
            dat,
            sa,
            lights,
            pose,
            erase,
            fog: (be32(&dat_for_fog, fog), bef(&dat_for_fog, fog + 8), bef(&dat_for_fog, fog + 0xC)),
            flow: Flow { cur: kind, prev: kind, hovered, entering: false, light_color: 0, light_lerp: 0 },
            cooldown: 0x14,
            panel: Panel { cur: kind, prev: kind, state: 0, n4, n41 },
            cons: Vec::new(),
            think: None,
            think_fresh: false,
            con_next: 0,
            cur_next: 0,
            pad: MenuPad::new(),
            exit: Exit::None,
            unlock_all,
            render: HsdRender::new(),
            cache: DataCache::default(),
            frame: 0,
            sis: None,
            atlas: Vec::new(),
            glyphs: GlyphCache::default(),
        };
        if let Some((bytes, atlas)) = sis {
            m.sis = Some(SisArchive::load(bytes, "SIS_MenuData")?);
            m.atlas = atlas;
        }
        m.sa.req_joint(PANEL, m.panel.n4, T::L_803EB354[0].start);
        m.sa.anim_all_at(PANEL, m.panel.n4);
        if kind == KIND_MAIN {
            m.sa.req_joint(PANEL, n41, T::PANEL[kind as usize * 3].start);
        } else {
            m.sa.req_joint(PANEL, n41, T::PANEL[kind as usize * 3 + 2].start);
        }
        m.sa.anim_all_at(PANEL, n41);
        // mnMain_Scene_OnEnter: the light, then the think and the screen (mn_8022B3A0(0))
        m.flow.light_lerp = 0;
        m.flow.light_color = light_index(m.flow.cur, m.flow.hovered);
        m.lerp_light();
        m.think = Some(Think::List(kind));
        m.make_con(S_IDLE);
        // OnEnter's objects are not "fresh": their procs run from the first frame
        for c in &mut m.cons {
            c.fresh = false;
        }
        m.think_fresh = false;
        Ok(m)
    }

    // ---- selection lock tables (mn_80229938, mn_80229A04)

    fn unlocked(&self, kind: u8, sel: u16) -> bool {
        match (kind, sel) {
            (KIND_REG, 2) => self.unlock_all,
            (KIND_DATA, 2) => self.unlock_all,
            (KIND_SETTINGS, 3) => false,
            (KIND_1P, 2) => false,
            (KIND_TOY, 2) => false,
            _ => true,
        }
    }
    /// `mn_80229A04`: how many of the selections below `sel` are unlocked.
    fn unlocked_before(&self, kind: u8, sel: u16) -> usize {
        (0..sel).filter(|&i| self.unlocked(kind, i)).count()
    }

    // ---- lights

    /// `LerpLightColor` on the first POINT light.
    fn lerp_light(&mut self) {
        let target = LIGHT_COLORS[self.flow.light_color.min(4)];
        let div = self.flow.light_lerp;
        if let Some(l) = self.lights.lights.iter_mut().find(|l| l.flags & 3 == 2) {
            if div != 0 {
                for k in 0..3 {
                    let diff = target[k] as i32 - l.color[k] as i32;
                    if diff != 0 {
                        l.color[k] = (l.color[k] as i32 + diff / div) as u8;
                    }
                }
            } else {
                l.color[..3].copy_from_slice(&target);
            }
        }
    }

    // ---- mn_8022B3A0: a new screen

    fn make_con(&mut self, state: u8) {
        let kind = self.flow.cur;
        let kd = kind_data(kind);
        let hovered = self.flow.hovered;
        let g = CON0 + self.con_next % CON_POOL;
        self.con_next += 1;
        let option_count = kd.count as usize;
        let unlocked_count = (0..kd.count as u16).filter(|&i| self.unlocked(kind, i)).count();
        self.sa.reset_group(g);
        self.sa.start_anim_set(g, 0, false);
        self.sa.req_joint(g, 0, 0.0);
        let mut tree = [0u32; 42];
        for (i, t) in tree.iter_mut().enumerate() {
            *t = self.sa.dfs_node(g, 0, i as u32).unwrap_or(0);
        }
        if state != S_IDLE {
            match state {
                S_ENTER_TO => self.sa.req_one(g, tree[3], T::L_803EB39C[0].start),
                S_EXIT_TO => self.sa.req_one(g, tree[3], T::L_803EB3B4[0].start),
                _ => {}
            }
            self.sa.anim_one(g, tree[3]);
        }
        let opt: Vec<u32> = (0..option_count).map(|i| tree[T::D_803EAE68[i] as usize]).collect();
        for i in 0..option_count {
            if !self.unlocked(kind, i as u16) {
                continue;
            }
            let ui = self.unlocked_before(kind, i as u16);
            let oj = opt[ui];
            self.sa.req_one(g, oj, unlocked_count as f32);
            self.sa.anim_one(g, oj);
            // the cursor model: AddAnimAll, then its parts
            let cg = CUR0 + self.cur_next % CUR_POOL;
            self.cur_next += 1;
            self.sa.reset_group(cg);
            self.sa.start_anim_set(cg, 0, false);
            let part = |sa: &StageAnim, n: usize| sa.dfs_node(cg, 0, T::D_803EAE7C[n] as u32).unwrap();
            let p: Vec<u32> = (0..7).map(|n| part(&self.sa, n)).collect();
            let hov = i as u16 == hovered;
            self.sa.req_one(cg, p[0], T::L_803EB360[hov as usize].start);
            self.sa.anim_one(cg, p[0]);
            self.sa.req_joint(cg, p[1], if hov { 1.0 } else { 0.0 });
            self.sa.anim_all_at(cg, p[1]);
            self.sa.req_one(cg, p[1], kd.start + 2.0 * i as f32);
            for ty in 0xC..=0x13u8 {
                self.sa.stop_tobj_tracks(cg, p[1], ty);
            }
            self.sa.anim_one(cg, p[1]);
            if hov {
                self.sa.req_joint(cg, p[2], T::L_803EB378[1].start);
            } else {
                self.sa.req_joint(cg, p[2], T::L_803EB378[0].end);
            }
            self.sa.anim_all_at(cg, p[2]);
            if hov {
                self.sa.req_one(cg, p[3], T::L_803EB390[0].start);
            } else {
                self.sa.set_hidden_all(cg, p[3], true);
            }
            self.sa.anim_one(cg, p[3]);
            if !hov {
                self.sa.set_hidden_all(cg, p[4], true);
            }
            self.sa.attach_group(g, oj, cg);
        }
        let hover_anim = self.hover_anim(kind, hovered);
        self.sa.req_joint(g, tree[14], hover_anim.start);
        self.sa.anim_all_at(g, tree[14]);
        self.cons.push(Con { g, kind, hovered, state, tree, fresh: true, exit_only: false, description: None });
        let last = self.cons.len() - 1;
        self.make_description(last, kind, hovered);
    }

    /// `mn_80229A7C`: (re)create the hover description of a screen's selection.
    fn make_description(&mut self, ci: usize, kind: u8, selection: u16) {
        self.cons[ci].description = None;
        let kd = kind_data(kind);
        let (Some(sis), Some(&idx)) = (self.sis.as_ref(), kd.desc.get(selection as usize)) else { return };
        let mut t = Text::new([-9.5, 9.1, 17.0], [364.68332, 38.38772]);
        t.font_size = [0.0521, 0.0521];
        t.set_string(sis, idx as u32);
        self.cons[ci].description = Some(t);
    }

    /// The `tree[14]` (hover description backdrop) animation of a screen/selection.
    fn hover_anim(&self, kind: u8, hovered: u16) -> AnimLoop {
        if kind == KIND_MAIN && hovered == 4 && !self.unlock_all {
            T::L_803EB438[0]
        } else if kind == KIND_1P && hovered == 0 && !self.unlock_all {
            T::L_803EB480[0]
        } else {
            kind_data(kind).loops[hovered as usize]
        }
    }

    /// DFS node of cursor-part `n` (0..7) of the cursor under option `ui` of `con`.
    fn cursor_parts(&self, c: &Con, ui: usize) -> Option<[u32; 7]> {
        let oj = c.tree[T::D_803EAE68[ui] as usize];
        let root = self.sa.child_of(c.g, oj)?;
        let mut out = [0u32; 7];
        for (n, o) in out.iter_mut().enumerate() {
            *o = self.sa.dfs_node(c.g, root, T::D_803EAE7C[n] as u32)?;
        }
        Some(out)
    }

    // ---- hover in / out (mn_80229F60 / mn_8022A440)

    fn hover_in(&mut self, ci: usize, sel: u16) {
        let kd = kind_data(self.flow.cur);
        let c = &self.cons[ci];
        let (g, t41) = (c.g, c.tree[41]);
        let ui = self.unlocked_before(self.flow.cur, sel);
        let Some(p) = self.cursor_parts(&self.cons[ci], ui) else { return };
        let sa = &mut self.sa;
        let jv = p[1];
        sa.req_joint(g, jv, 1.0);
        sa.anim_all_at(g, jv);
        sa.req_one(g, jv, kd.start + sel as f32 * 2.0);
        for ty in 0xC..=0x13u8 {
            sa.stop_tobj_tracks(g, jv, ty);
        }
        sa.anim_one(g, jv);
        let f = sa.frame_at(g, p[0]);
        sa.req_joint(g, p[0], T::L_803EB360[1].start + (f - T::L_803EB360[0].start));
        sa.anim_all_at(g, p[0]);
        let f = sa.frame_at(g, p[2]);
        sa.req_joint(g, p[2], T::L_803EB378[1].start + (f - T::L_803EB378[0].start));
        sa.anim_all_at(g, p[2]);
        sa.req_joint(g, p[3], T::L_803EB390[0].start);
        sa.anim_all_at(g, p[3]);
        sa.set_hidden_all(g, p[3], false);
        sa.set_hidden_all(g, p[4], false);
        let mut s5 = sa.scale_of(g, p[5]);
        s5.y = 2.0;
        sa.set_scale(g, p[5], s5);
        let mut s6 = sa.scale_of(g, p[6]);
        s6.y = 2.0;
        sa.set_scale(g, p[6], s6);
        let t5 = sa.translate_of(g, p[5]);
        sa.set_translate(g, p[6], t5);
        let a = sa.joint_world_pos(g, p[5]);
        let b = sa.joint_world_pos(g, t41);
        let rot = melee_runtime::lbtrigf::atan2f(b.y - a.y, b.x - a.x);
        sa.set_rotate_yz(g, p[5], None, Some(rot));
        sa.set_rotate_yz(g, p[6], None, Some(rot));
    }

    fn hover_out(&mut self, ci: usize, sel: u16) {
        let kd = kind_data(self.flow.cur);
        let g = self.cons[ci].g;
        let ui = self.unlocked_before(self.flow.cur, sel);
        let Some(p) = self.cursor_parts(&self.cons[ci], ui) else { return };
        let sa = &mut self.sa;
        let r = p[1];
        sa.req_joint(g, r, 0.0);
        sa.anim_all_at(g, r);
        sa.req_one(g, r, kd.start + sel as f32 * 2.0);
        for ty in 0xC..=0x13u8 {
            sa.stop_tobj_tracks(g, r, ty);
        }
        sa.anim_one(g, r);
        let f = sa.frame_at(g, p[0]);
        sa.req_joint(g, p[0], (f - T::L_803EB360[1].start) + T::L_803EB360[0].start);
        sa.anim_all_at(g, p[0]);
        sa.req_joint(g, p[2], T::L_803EB378[0].start);
        sa.anim_all_at(g, p[2]);
        sa.set_hidden_all(g, p[4], true);
    }

    // ---- mn_8022A5D0: the per-frame cursor update

    fn cursor_update(&mut self, ci: usize, selection: u16) {
        let (g, kind, t41) = (self.cons[ci].g, self.cons[ci].kind, self.cons[ci].tree[41]);
        let kd = kind_data(kind);
        for var in 0..kd.count as u16 {
            if !self.unlocked(self.flow.cur, var) {
                continue;
            }
            let ui = self.unlocked_before(self.flow.cur, var);
            let Some(p) = self.cursor_parts(&self.cons[ci], ui) else { continue };
            let sel = (var == selection) as usize;
            let sa = &mut self.sa;
            if std::env::var_os("MELEE_MENU_LOG").is_some() && self.frame % 10 == 0 {
                eprintln!("menu: f{} con{} var{} sel{} p0 frame {} p2 {} p3 {}", self.frame, ci, var, selection, sa.frame_at(g, p[0]), sa.frame_at(g, p[2]), sa.frame_at(g, p[3]));
            }
            anim_loop_at(sa, g, p[0], &T::L_803EB360[sel]);
            anim_loop_at(sa, g, p[2], &T::L_803EB378[sel]);
            if anim_loop_at(sa, g, p[3], &T::L_803EB390[0]) >= T::L_803EB390[0].end {
                sa.set_hidden_all(g, p[3], true);
            }
            if selection == var {
                if sa.scale_of(g, p[6]).y == 2.0 {
                    let a = sa.joint_world_pos(g, p[5]);
                    let b = sa.joint_world_pos(g, t41);
                    let d = [b.x - a.x, b.y - a.y, b.z - a.z];
                    let mut t = sa.translate_of(g, p[6]);
                    t.x += d[0] / 10.0;
                    t.y += d[1] / 10.0;
                    t.z += d[2] / 10.0;
                    sa.set_translate(g, p[6], t);
                    if (d[0] >= 0.0 && t.x >= d[0]) || (d[0] < 0.0 && t.x <= d[0]) {
                        sa.set_translate(g, p[6], melee_runtime::mtx::Vec3 { x: d[0], y: d[1], z: d[2] });
                        let mut s6 = sa.scale_of(g, p[6]);
                        s6.y += -0.2;
                        sa.set_scale(g, p[6], s6);
                        let mut s5 = sa.scale_of(g, p[5]);
                        s5.y += -0.2;
                        sa.set_scale(g, p[5], s5);
                    }
                } else if sa.scale_of(g, p[6]).y > 0.0 {
                    let mut s6 = sa.scale_of(g, p[6]);
                    s6.y += -0.2;
                    sa.set_scale(g, p[6], s6);
                    let mut s5 = sa.scale_of(g, p[5]);
                    s5.y += -0.2;
                    sa.set_scale(g, p[5], s5);
                    if sa.scale_of(g, p[6]).y <= 0.0 {
                        let mut s6 = sa.scale_of(g, p[6]);
                        s6.y = 0.0;
                        sa.set_scale(g, p[6], s6);
                        let mut s5 = sa.scale_of(g, p[5]);
                        s5.y = 0.0;
                        sa.set_scale(g, p[5], s5);
                        sa.set_hidden_all(g, p[4], true);
                    }
                }
            }
        }
    }

    // ---- mn_8022ADD8: the hover backdrop animation

    fn backdrop(&mut self, ci: usize, selection_changed: bool) {
        let (g, state, kind, hov, t14) = {
            let c = &self.cons[ci];
            (c.g, c.state, c.kind, c.hovered, c.tree[14])
        };
        match state {
            S_EXIT_FROM | S_ENTER_FROM => self.sa.set_hidden_all(g, t14, true),
            S_IDLE | S_ENTER_TO | S_EXIT_TO | S_5 => {
                if kind_data(kind).loops.is_empty() {
                    return;
                }
                let al = self.hover_anim(kind, hov);
                let cur = self.sa.frame_at(g, t14);
                if selection_changed || !(al.start <= cur) || !(cur <= al.end) {
                    self.sa.req_joint(g, t14, al.start);
                }
                anim_loop_at(&mut self.sa, g, t14, &al);
            }
            _ => {}
        }
    }

    // ---- the con proc: fn_8022AFEC / fn_8022AF10

    fn con_proc(&mut self, ci: usize) -> bool {
        // returns false when the con was freed
        let g = self.cons[ci].g;
        if self.cons[ci].exit_only {
            let (state, t3) = (self.cons[ci].state, self.cons[ci].tree[3]);
            let al = if state == S_EXIT_FROM { T::L_803EB3A8[0] } else { T::L_803EB3C0[0] };
            if self.sa.frame_at(g, t3) >= al.end {
                return false;
            }
            self.sa.anim_one(g, t3);
            return true;
        }
        let mut selection_changed = false;
        let mut set_kind = false;
        let st = self.cons[ci].state;
        if (st == S_5 || st <= S_ENTER_TO || st == S_EXIT_TO) && self.cons[ci].kind != self.flow.cur {
            self.cons[ci].state = if self.flow.entering { S_ENTER_FROM } else { S_EXIT_FROM };
            let al = if self.cons[ci].state == S_EXIT_FROM { T::L_803EB3A8[0] } else { T::L_803EB3C0[0] };
            let t3 = self.cons[ci].tree[3];
            self.sa.req_one(g, t3, al.start);
            self.sa.anim_one(g, t3);
            let s = self.cons[ci].state;
            if s == S_5 || s <= S_ENTER_TO || s == S_EXIT_TO {
                set_kind = true;
                selection_changed = true;
            } else {
                // HSD_GObjProc_RemoveProc + SetupProc(fn_8022AF10): the rest of this call still runs
                self.cons[ci].exit_only = true;
            }
        }
        let state = self.cons[ci].state;
        if state == S_ENTER_TO || state == S_EXIT_TO {
            let t3 = self.cons[ci].tree[3];
            let al = if state == S_ENTER_TO { T::L_803EB39C[0] } else { T::L_803EB3B4[0] };
            if self.sa.frame_at(g, t3) >= al.end {
                self.cons[ci].state = S_IDLE;
            }
            self.sa.anim_one(g, t3);
        }
        let state = self.cons[ci].state;
        let mut selection = self.cons[ci].hovered;
        if state == S_IDLE || state == S_ENTER_TO || state == S_EXIT_TO {
            if self.cons[ci].hovered != self.flow.hovered {
                selection_changed = true;
            }
            if selection_changed {
                let new_sel = self.flow.hovered;
                self.hover_in(ci, new_sel);
                let old_sel = self.cons[ci].hovered;
                self.hover_out(ci, old_sel);
            }
            selection = if selection_changed { self.flow.hovered } else { self.cons[ci].hovered };
            self.cursor_update(ci, selection);
        }
        self.backdrop(ci, selection_changed);
        match self.cons[ci].state {
            S_EXIT_FROM | S_ENTER_FROM => self.cons[ci].description = None,
            S_IDLE => {
                if selection_changed {
                    let k = self.cons[ci].kind;
                    self.make_description(ci, k, selection);
                }
                if let Some(t) = self.cons[ci].description.as_mut() {
                    t.hidden = false;
                }
            }
            _ => {}
        }
        if set_kind {
            self.cons[ci].kind = self.flow.cur;
        }
        if selection_changed {
            self.cons[ci].hovered = self.flow.hovered;
        }
        true
    }

    // ---- the panel proc: fn_80229BF4

    fn panel_proc(&mut self) {
        let (n4, n41) = (self.panel.n4, self.panel.n41);
        let sa = &mut self.sa;
        if self.panel.cur != self.flow.cur {
            self.panel.prev = self.panel.cur;
            self.panel.cur = self.flow.cur;
            if self.flow.entering {
                sa.req_joint(PANEL, n41, T::PANEL[self.panel.cur as usize * 3].start);
                self.panel.state = 0;
            } else {
                sa.req_joint(PANEL, n41, T::PANEL[self.panel.prev as usize * 3 + 1].start);
                self.panel.state = 1;
            }
        }
        match self.panel.state {
            2 => {
                anim_loop_at(sa, PANEL, n41, &T::PANEL[self.panel.cur as usize * 3 + 2]);
            }
            0 | 1 => {
                let f = sa.frame_at(PANEL, n41);
                let sp = if self.panel.state == 0 { T::PANEL[self.panel.cur as usize * 3] } else { T::PANEL[self.panel.prev as usize * 3 + 1] };
                if f >= sp.end {
                    self.panel.state = 2;
                    sa.req_joint(PANEL, n41, T::PANEL[self.panel.cur as usize * 3 + 2].start);
                }
                sa.anim_all_at(PANEL, n41);
            }
            _ => {}
        }
        anim_loop_at(sa, PANEL, n4, &T::L_803EB354[0]);
    }

    // ---- the light proc: fn_8022C128

    fn light_proc(&mut self) {
        self.lights.step();
        let color = light_index(self.flow.cur, self.flow.hovered);
        if color != self.flow.light_color {
            self.flow.light_lerp = 0x10;
            self.flow.light_color = color;
        }
        if self.flow.light_lerp != 0 {
            self.flow.light_lerp -= 1;
            self.lerp_light();
        }
    }

    // ---- input (mn_80229624) and the think procs

    /// `mn_80229624(4)`: the menu input bits, or 0 while the cooldown runs.
    fn menu_input(&mut self) -> (bool, bool, bool, bool) {
        if self.cooldown != 0 {
            self.cooldown -= 1;
            return (false, false, false, false);
        }
        let t = self.pad.triggered(4);
        let r = self.pad.repeated(4);
        (t & MP_CONFIRM != 0, t & MP_CANCEL != 0, r & MP_UP != 0, r & MP_DOWN != 0)
    }

    /// The common "enter a sub screen / go back" step of the thinks: `mn_8022B3A0(state)` and the new think.
    fn goto_menu(&mut self, kind: u8, hovered: u16, state: u8) {
        self.flow.prev = self.flow.cur;
        self.flow.cur = kind;
        self.flow.hovered = hovered;
        self.make_con(state); // fresh: its proc starts next frame
        self.think = Some(Think::List(kind));
        self.think_fresh = true;
    }

    fn step_sel(&mut self, kind: u8, count: u8, up: bool, wrap_locked: bool) {
        loop {
            let h = &mut self.flow.hovered;
            if up {
                *h = if *h != 0 { *h - 1 } else { count as u16 - 1 };
            } else {
                *h = if *h == count as u16 - 1 { 0 } else { *h + 1 };
            }
            if !wrap_locked || self.unlocked(kind, self.flow.hovered) {
                break;
            }
        }
    }

    fn think_proc(&mut self, kind: u8) {
        let (confirm, back, up, down) = self.menu_input();
        if std::env::var_os("MELEE_MENU_LOG").is_some() && (confirm || back || up || down) {
            eprintln!("menu: frame {} think kind {kind} input c{confirm} b{back} u{up} d{down} hov {}", self.frame, self.flow.hovered);
        }
        let count = kind_data(kind).count;
        let hov = self.flow.hovered;
        if confirm {
            self.cooldown = 5;
            self.flow.entering = true;
            match (kind, hov) {
                (KIND_MAIN, h) => {
                    let nk = [KIND_1P, KIND_VS, KIND_TOY, KIND_SETTINGS, KIND_DATA][h as usize % 5];
                    self.flow.prev = self.flow.cur;
                    self.goto_menu(nk, 0, S_ENTER_TO);
                }
                (KIND_1P, 0) => self.goto_menu(KIND_REG, 0, S_ENTER_TO),
                (KIND_1P, 3) => self.goto_menu(KIND_STADIUM, 0, S_ENTER_TO),
                (KIND_1P, 1) => self.exit = Exit::Screen("event"),
                (KIND_1P, 4) => self.exit = Exit::Mode("Training"),
                (KIND_VS, 0) => self.exit = Exit::Mode("Vs"),
                (KIND_VS, 1) => self.exit = Exit::Mode("Tournament"),
                (KIND_VS, 2) => self.goto_menu(KIND_SPECIAL, 0, S_ENTER_TO),
                (KIND_VS, 3) => self.exit = Exit::Screen("rules"),
                (KIND_VS, 4) => self.exit = Exit::Screen("name"),
                (KIND_TOY, 0) => self.exit = Exit::Mode("ToyGallery"),
                (KIND_TOY, 1) => self.exit = Exit::Mode("ToyLottery"),
                (KIND_TOY, 3) => self.exit = Exit::Mode("ToyCollection"),
                (KIND_SETTINGS, 0) => self.exit = Exit::Screen("rumble"),
                (KIND_SETTINGS, 1) => self.exit = Exit::Screen("sound"),
                (KIND_SETTINGS, 2) => self.exit = Exit::Screen("display"),
                (KIND_SETTINGS, 4) => self.exit = Exit::Screen("language"),
                (KIND_SETTINGS, 5) => self.exit = Exit::Screen("erase"),
                (KIND_DATA, 0) => self.exit = Exit::Screen("snapshots"),
                (KIND_DATA, 1) => self.exit = Exit::Screen("archives"),
                (KIND_DATA, 2) => self.exit = Exit::Screen("soundtest"),
                (KIND_DATA, 3) => self.goto_menu(KIND_RECORDS, 0, S_ENTER_TO),
                (KIND_DATA, 4) => self.exit = Exit::Screen("info"),
                (KIND_REG, 0) => self.exit = Exit::Mode("Classic"),
                (KIND_REG, 1) => self.exit = Exit::Mode("Adventure"),
                (KIND_REG, 2) => self.exit = Exit::Mode("AllStar"),
                (KIND_STADIUM, 0) => self.exit = Exit::Mode("TargetTest"),
                (KIND_STADIUM, 1) => self.exit = Exit::Mode("HomeRunContest"),
                (KIND_STADIUM, 2) => self.exit = Exit::Screen("multiman"),
                (KIND_SPECIAL, h) => {
                    const M: [&str; 10] = ["CameraMode", "StaminaVs", "SuperSuddenDeathVs", "GiantVs", "TinyVs", "InvisibleVs", "CameraVs", "SingleButtonVs", "LightningVs", "SlomoVs"];
                    self.exit = Exit::Mode(M[h as usize % 10]);
                }
                (KIND_RECORDS, 0) => self.exit = Exit::Screen("records_vs"),
                (KIND_RECORDS, 1) => self.exit = Exit::Screen("records_bonus"),
                (KIND_RECORDS, 2) => self.exit = Exit::Screen("records_misc"),
                _ => {}
            }
        } else if back {
            self.flow.entering = false;
            self.cooldown = 5;
            match kind {
                KIND_MAIN => self.exit = Exit::Title,
                KIND_1P => self.goto_menu(KIND_MAIN, 0, S_EXIT_TO),
                KIND_VS => self.goto_menu(KIND_MAIN, 1, S_EXIT_TO),
                KIND_TOY => self.goto_menu(KIND_MAIN, 2, S_EXIT_TO),
                KIND_SETTINGS => self.goto_menu(KIND_MAIN, 3, S_EXIT_TO),
                KIND_DATA => self.goto_menu(KIND_MAIN, 4, S_EXIT_TO),
                KIND_REG => self.goto_menu(KIND_1P, 0, S_EXIT_TO),
                KIND_STADIUM => self.goto_menu(KIND_1P, 3, S_EXIT_TO),
                KIND_SPECIAL => self.goto_menu(KIND_VS, 2, S_EXIT_TO),
                KIND_RECORDS => self.goto_menu(KIND_DATA, 3, S_EXIT_TO),
                _ => {}
            }
        } else if up {
            match kind {
                // the 1P and VS thinks wrap without checking locks the same way the table does
                KIND_VS | KIND_RECORDS | KIND_SPECIAL => self.step_sel(kind, count, true, false),
                _ => self.step_sel(kind, count, true, true),
            }
        } else if down {
            match kind {
                KIND_VS | KIND_RECORDS | KIND_SPECIAL => self.step_sel(kind, count, false, false),
                _ => self.step_sel(kind, count, false, true),
            }
        }
    }

    /// One logic frame with this frame's controller samples.
    pub fn tick(&mut self, raw: &[Option<crate::pads::RawPad>; 4]) {
        self.frame += 1;
        if let Ok(v) = std::env::var("MELEE_MENU_DUMP") {
            // "frame:group"
            if let Some((f, g)) = v.split_once(':') {
                if f.parse::<u64>().ok() == Some(self.frame) {
                    self.sa.dump(g.parse().unwrap_or(1));
                }
            }
        }
        if let Ok(v) = std::env::var("MELEE_MENU_HIDE") {
            // "group:node,node": hide those subtrees every frame
            if let Some((g, js)) = v.split_once(':') {
                let g: u32 = g.parse().unwrap_or(1);
                for j in js.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
                    self.sa.set_hidden_all(g, j, true);
                }
            }
        }
        self.pad.update(raw);
        // gobj procs by process link: think(1), light(4), back(5), panel(6), cons(7)
        if let Some(Think::List(k)) = self.think {
            if !self.think_fresh {
                self.think_proc(k);
            }
        }
        self.light_proc();
        self.sa.anim_all_at(BACK, 0);
        self.panel_proc();
        let mut i = 0;
        while i < self.cons.len() {
            if self.cons[i].fresh {
                i += 1;
                continue;
            }
            if self.con_proc(i) {
                i += 1;
            } else {
                self.cons.remove(i);
            }
        }
        self.think_fresh = false;
        for c in &mut self.cons {
            c.fresh = false;
        }
    }

    /// The scene's draw: erase to the fog colour, then link 2 (back), 3 (panel), 4 (screens) through the camera.
    pub fn draw(&mut self, shim: &mut GxShim) {
        use melee_gx::api::GxColor;
        shim.set_copy_clear(GxColor::new(self.erase[0], self.erase[1], self.erase[2], self.erase[3]), 0xFF_FFFF);
        shim.set_viewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
        shim.set_scissor(0, 0, 640, 480);
        let (vmtx, proj) = crate::menu_support::camera(&self.pose);
        shim.set_projection(&proj, melee_gx::consts::GX_PERSPECTIVE);
        // HSD_FogSet (gx link 1): GXSetFog(type, start, end, cobj near, far, colour)
        shim.set_fog(self.fog.0, self.fog.1, self.fog.2, self.pose.near, self.pose.far, GxColor::new(self.erase[0], self.erase[1], self.erase[2], self.erase[3]));
        let dat = self.dat.clone();
        let mut g = ShimBackend { shim, cache: &mut self.cache };
        self.render.invalidate_all();
        self.render.lobj_setup_init_with(&mut g, &Dat(&dat), &self.lights.ins(), &vmtx);
        let mut groups: Vec<u32> = vec![BACK, PANEL];
        groups.extend(self.cons.iter().map(|c| c.g));
        if let Ok(v) = std::env::var("MELEE_MENU_ONLY") {
            groups.retain(|g| v.split(',').any(|x| x.trim().parse::<u32>().ok() == Some(*g)));
        }
        let parts: Vec<_> = groups.iter().map(|&gid| (self.sa.overrides(), self.sa.disp_joints(gid))).collect();
        for (o, j) in &parts {
            let dm = DispModel::new(&dat, j.clone()).with_mat_anim(o);
            self.render.clear_vtx_desc(&mut g);
            self.render.jobj_disp_all(&mut g, &dm, &vmtx, 7, 0);
        }
        // gx link 7 (the text camera, same view): each screen's description
        if let Some(sis) = self.sis.as_ref() {
            let env = SisEnv { sis, atlas: &self.atlas };
            for c in &mut self.cons {
                if let Some(t) = c.description.as_mut() {
                    t.draw(g.shim, &env, &vmtx, &mut self.glyphs);
                }
            }
        }
    }
}

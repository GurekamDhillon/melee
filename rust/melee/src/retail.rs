//! Retail menu scenes, ported from src/melee/gm and src/melee/mn and drawn through the ported HSD
//! path (the same way `hud_view` draws IfAll). The scene logic here follows the C line by line; the
//! GObj / JObj arenas are `StageAnim` (melee_runtime's JObj runtime on the DAT's descriptors).
//!
//! `Scene` is the seam the frontend plays through: `melee play` owns one retail scene at a time,
//! ticks it with the pad state, draws it, and reads its exit. The port's own kit menus
//! (gmfrontend*, gw_kit, online lobby/settings) are NOT here; `MenuHook` is where they plug in.
use crate::cm_drive::CamPose;
use crate::gx_backend::{DataCache, ShimBackend};
use crate::stage_anim::StageAnim;
use melee_gx::shim::GxShim;
use melee_runtime::dat::DatArchive;
use melee_runtime::tu::displayfunc::DispModel;
use melee_runtime::tu::state::{Dat, HsdRender};
use std::sync::Arc;

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
    let (vmtx, proj) = crate::view::camera(pose);
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

/// The current frame of a model's animation (`mn_8022F298`; the first AObj found, -1 if none).
fn cur_frame(m: &StageAnim) -> f32 {
    m.first_frame().unwrap_or(-1.0)
}

/// `mn_8022ED6C(jobj, &loop)`: advance the animation one frame, looping per `s`; returns the frame.
pub fn anim_loop(m: &mut StageAnim, s: &AnimLoop) -> f32 {
    let mut cur = cur_frame(m);
    if !(s.start <= cur && cur <= s.end) {
        m.req_joint(0, 0, s.start);
    }
    if s.loop_to == -0.1 {
        cur = cur_frame(m);
        if cur < s.end {
            m.step();
            cur = cur_frame(m);
            if cur > s.end {
                cur = s.end;
                m.req_joint(0, 0, cur);
                m.step();
            }
        }
    } else {
        m.step();
        cur = cur_frame(m);
        if cur >= s.end {
            cur = s.loop_to + (cur - s.end);
            m.req_joint(0, 0, cur);
            m.step();
        }
    }
    cur
}

// ------------------------------------------------------------------------------------------------
// gmtitle.c
// ------------------------------------------------------------------------------------------------

const HSD_PAD_A: u32 = 0x100;
const HSD_PAD_B: u32 = 0x200;
const HSD_PAD_Y: u32 = 0x800;
const HSD_PAD_START: u32 = 0x1000;

/// `loop_settings_0` / `loop_settings_1` of gmtitle.c.
const LOGO_LOOP: AnimLoop = AnimLoop { start: 0.0, end: 1600.0, loop_to: 400.0 };
const BG_LOOP: AnimLoop = AnimLoop { start: 0.0, end: 1330.0, loop_to: 130.0 };

/// The title scene (GM_TITLE): `gm_Scene_Title_OnEnter` / `OnFrame` over GmTtAll.usd.
pub struct Title {
    dat: Arc<Vec<u8>>,
    logo: StageAnim,
    bg: StageAnim,
    pose: CamPose,
    light: Vec<u32>,
    erase: [u8; 4],
    countdown_timer: i32,
    frame_count: u32,
    pub exit: Exit,
    render: HsdRender,
    cache: DataCache,
}

impl Title {
    /// `emblem_unlocked`: `gm_IsCKindUnlocked(Mars) || (Emblem)`; the logo's JObj 7 is hidden otherwise.
    pub fn new(gmttall: &[u8], emblem_unlocked: bool) -> Result<Self, String> {
        let a = DatArchive::parse(gmttall).map_err(|e| format!("GmTtAll: {e}"))?;
        let mut data = a.data.clone();
        let moji = add_model(&mut data, &a, "TtlMoji_Top")?;
        let bgm = add_model(&mut data, &a, "TtlBg_Top")?;
        let dat = Arc::new(data);
        let sym = |n: &str| a.public_offset(n).ok_or(format!("GmTtAll: no {n}"));
        let cd = sym("ScTitle_cam_int1_camera")?;
        let (e, i) = (be32(&dat, cd + 0x18), be32(&dat, cd + 0x1C));
        let pose = CamPose {
            eye: [bef(&dat, e + 4), bef(&dat, e + 8), bef(&dat, e + 12)],
            interest: [bef(&dat, i + 4), bef(&dat, i + 8), bef(&dat, i + 12)],
            fov: bef(&dat, cd + 0x30),
            near: bef(&dat, cd + 0x28),
            far: bef(&dat, cd + 0x2C),
            aspect: bef(&dat, cd + 0x34),
        };
        let ll = be32(&dat, be32(&dat, sym("ScTitle_scene_lights")?));
        let light = if be32(&dat, ll) != 0 { vec![be32(&dat, ll)] } else { vec![] };
        let fog = sym("ScTitle_fog")?;
        let c = fog + 0x10;
        let erase = [dat[c as usize], dat[c as usize + 1], dat[c as usize + 2], dat[c as usize + 3]];
        // gmTitle_801A165C (not the active-title branch: GM_TITLE): logo at frame 400.
        let mut logo = StageAnim::new_scene(dat.clone(), vec![moji]);
        logo.start_anim_set(0, 0, false);
        logo.req_joint(0, 0, 400.0);
        logo.step();
        if !emblem_unlocked {
            logo.set_hidden_all(0, 7, true);
        }
        // fn_801A1498_inline: the background, frame 130 (not the active-title branch).
        let mut bg = StageAnim::new_scene(dat.clone(), vec![bgm]);
        bg.start_anim_set(0, 0, false);
        bg.req_joint(0, 0, 130.0);
        bg.step();
        Ok(Self { dat, logo, bg, pose, light, erase, countdown_timer: 20, frame_count: 0, exit: Exit::None, render: HsdRender::new(), cache: DataCache::default() })
    }

    /// One logic frame: the gobj procs (`gmTitle_801A1630` on the logo, `gmTitle_801A146C` on the
    /// background), then `gm_Scene_Title_OnFrame` with the triggered buttons of any port.
    pub fn tick(&mut self, trig: u32) {
        anim_loop(&mut self.logo, &LOGO_LOOP);
        anim_loop(&mut self.bg, &BG_LOOP);
        if self.countdown_timer != 0 {
            self.countdown_timer -= 1;
            return;
        }
        self.frame_count += 1;
        if self.frame_count > 600 {
            self.exit = Exit::TitleTimeout;
        } else if trig & HSD_PAD_START != 0 {
            self.exit = Exit::TitleStart(trig);
        } else if trig & (HSD_PAD_Y | HSD_PAD_B) != 0 {
            self.exit = Exit::TitleKit(trig);
        }
    }

    /// The scene's draw: erase to the fog colour (`gmTitle_801A18D4`), then link 3 (background) and
    /// link 9 (logo) through the scene camera (`gmTitle_801A1814`).
    pub fn draw(&mut self, shim: &mut GxShim) {
        use melee_gx::api::GxColor;
        shim.set_copy_clear(GxColor::new(self.erase[0], self.erase[1], self.erase[2], self.erase[3]), 0xFF_FFFF);
        shim.set_viewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
        shim.set_scissor(0, 0, 640, 480);
        let dat = self.dat.clone();
        let mut ms = [&mut self.bg, &mut self.logo];
        draw_models(&mut self.render, shim, &mut self.cache, &dat, &mut ms, &self.pose, &self.light);
    }
}

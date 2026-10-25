//! gmtitle.c: the title scene (GM_TITLE).
use super::*;

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


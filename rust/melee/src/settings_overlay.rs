//! The in-game settings overlay (`melee play`, retail flow): F1, or hold D-pad Down and press Start on any controller.
//!
//! Pages: VIDEO, AUDIO, OVERLAYS, GAME, CUSTOMIZE (menu theme, costume skins). Every change is written to settings.cfg
//! at once (`MELEE_SETTINGS_CFG` moves the file; a `MELEE_<KEY>` in the environment still wins, and the row says so).
//! Rows marked LIVE apply on the spot (volume, the practice overlays, the performance HUD, the theme, skins offline);
//! the others say "next launch".
//!
//! Drawing: the gd-ui-v1 host renderer (melee_modhost `Batch` over the shared `Atlas`, melee_gx ui.rs): with UI mods
//! loaded the overlay is drawn into their batch (`PresHost::present_with`, after the mods, layer MODAL), else into its
//! own; either way one instanced draw per present, and nothing at all while closed.
//!
//! Sim safety: the overlay is harness-owned (a field of `RetailPlay`, never a static, never in the arena). Its only
//! path into the game is the input boundary: while open it zeroes the owning port's pad (as an exclusive gd-ui
//! capture does), before the recorder and the rollback submit, so a replay or a netplay peer sees exactly what this
//! side played. It never masks a replay feed. None of its settings reach the simulation (cosmetics.rs's rule for
//! skins).
//!
//! Reference (behaviour only, reimplemented): Hero's melee-unlocked PC settings overlay (`pc_settings.cpp`: F1 /
//! Start + D-pad Down, category pages, restart notices).
use melee_gx::ui::Atlas;
use melee_modhost::ui::cmd::{self as cmd, Batch, Pen};
use melee_modhost::ui::theme;
use melee_platform::pad::*;
use melee_types::PADStatus;

// gd-ui-v1 text roles (cmd::ROLES): label, value, body, big
const ROLE_VALUE: i32 = 1;
const ROLE_BODY: i32 = 2;
const ROLE_BIG: i32 = 3;

#[derive(Clone, Debug)]
enum Kind {
    Toggle,
    /// (shown, stored) choices
    Choice(Vec<(String, String)>),
    Slider { lo: i32, hi: i32, step: i32 },
}

#[derive(Clone, Debug)]
pub struct Row {
    pub key: String,
    label: String,
    kind: Kind,
    /// Toggle: 0/1; Choice: index; Slider: value.
    pub value: i32,
    live: bool,
    help: String,
    /// Set from outside (a skin's verdict).
    pub note: String,
}

impl Row {
    /// A live on/off row (`dflt` when settings.cfg has no value; `off` / `0` read as off).
    pub fn toggle_row(key: &str, label: &str, dflt: bool, help: &str) -> Row {
        toggle(key, label, dflt, true, help)
    }
    fn shown(&self) -> String {
        match &self.kind {
            Kind::Toggle => if self.value != 0 { "ON" } else { "OFF" }.into(),
            Kind::Choice(c) => c.get(self.value as usize).map_or(String::new(), |c| c.0.clone()),
            Kind::Slider { .. } => format!("{}", self.value),
        }
    }
    /// The settings.cfg value.
    pub fn stored(&self) -> String {
        match &self.kind {
            Kind::Toggle => (self.value != 0 as i32).then_some("1").unwrap_or("0").into(),
            Kind::Choice(c) => c.get(self.value as usize).map_or(String::new(), |c| c.1.clone()),
            Kind::Slider { .. } => self.value.to_string(),
        }
    }
    fn step(&mut self, d: i32) -> bool {
        let old = self.value;
        match &self.kind {
            Kind::Toggle => self.value = (self.value == 0) as i32,
            Kind::Choice(c) => {
                let n = c.len() as i32;
                if n > 0 {
                    self.value = (self.value + d).rem_euclid(n);
                }
            }
            Kind::Slider { lo, hi, step } => self.value = (self.value + d * step).clamp(*lo, *hi),
        }
        self.value != old
    }
}

pub struct Page {
    name: &'static str,
    pub rows: Vec<Row>,
}

/// A change the caller applies (live ones) after the overlay saved it.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub key: String,
    pub value: String,
    pub live: bool,
}

pub struct Overlay {
    pub open: bool,
    pub pages: Vec<Page>,
    page: usize,
    row: usize,
    /// The port that opened it (its pad is masked while open); None: opened from the keyboard.
    owner: Option<usize>,
    prev: [u16; 4],
    held: u32,
    stick_prev: i8,
    f1_prev: bool,
    /// Per port: buttons still held from a close (B / Start): kept from the game until released, so the close never
    /// becomes a press on the screen underneath.
    swallow: [u16; 4],
    pub theme: i32,
    /// The palette (0: the style's own colours).
    pub palette: i32,
    /// Changes since the last `take_changes`.
    changes: Vec<Change>,
    /// Write settings.cfg (off in the unit tests).
    pub save: bool,
    notice: Option<(String, u32)>,
    /// Own renderer state when no UI mod host exists (created on the first open).
    pub own: Option<(Batch, Atlas)>,
    /// The own batch is non-empty on the GPU (a closed overlay clears it once).
    pub shown: bool,
    /// Quads of the last drawn frame (log / tests).
    pub last_quads: usize,
}

fn cfg_val(key: &str) -> Option<String> {
    crate::cfg::text(key)
}

fn choice(key: &str, label: &str, opts: &[(&str, &str)], dflt: &str, live: bool, help: &str) -> Row {
    let cur = cfg_val(key).unwrap_or_else(|| dflt.to_string()).to_ascii_lowercase();
    let c: Vec<(String, String)> = opts.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let value = c.iter().position(|o| o.1 == cur).unwrap_or(0) as i32;
    Row { key: key.into(), label: label.into(), kind: Kind::Choice(c), value, live, help: help.into(), note: String::new() }
}

/// MSAA: only the counts the renderer's device supports for every EFB attachment format (all four before a renderer
/// exists); a saved count it lacks shows as the one the renderer clamped it to.
fn msaa_row() -> Row {
    let all = [("Off", "1"), ("2x", "2"), ("4x", "4"), ("8x", "8")];
    let mask = melee_gx::render::SUPPORTED_MSAA.load(std::sync::atomic::Ordering::Relaxed);
    let opts: Vec<(&str, &str)> = all.iter().copied().filter(|(_, v)| mask == 0 || mask >> v.parse::<u32>().unwrap() & 1 != 0).collect();
    let mut r = choice("msaa", "Anti-aliasing", &opts, "1", false, "MSAA of the EFB pass (pipelines are built for it: next launch).");
    if mask != 0 {
        let want = cfg_val("msaa").and_then(|v| v.trim().trim_end_matches(['x', 'X']).parse::<u32>().ok()).unwrap_or(1);
        let got = melee_gx::render::clamp_msaa(want, mask).to_string();
        if let Kind::Choice(c) = &r.kind {
            r.value = c.iter().position(|o| o.1 == got).unwrap_or(0) as i32;
        }
    }
    r
}

impl Overlay {
    /// Rebuild the MSAA row once the renderer knows the device's sample counts (the overlay may be built first).
    pub fn refresh_msaa(&mut self) {
        let r = msaa_row();
        for page in self.pages.iter_mut() {
            if let Some(x) = page.rows.iter_mut().find(|x| x.key == "msaa") {
                *x = r;
                return;
            }
        }
    }
}

fn toggle(key: &str, label: &str, dflt: bool, live: bool, help: &str) -> Row {
    let v = cfg_val(key).map_or(dflt, |v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "on" | "yes" | "true"));
    Row { key: key.into(), label: label.into(), kind: Kind::Toggle, value: v as i32, live, help: help.into(), note: String::new() }
}

impl Overlay {
    /// The pages from the current settings; `volume` the live master volume.
    pub fn new(volume: i32) -> Overlay {
        let onoff = [("Auto", "auto"), ("On", "on"), ("Off", "off")];
        // Live: the presenter and the pacer take these at once (presentation only). Widescreen is live in the renderer
        // mode (the game's view hooks keep answering 4:3); the camera mode is fixed for the run. MSAA is baked into
        // every pipeline (the sample count), so it waits for the next launch.
        let wide_live = !melee_engine::view::camera_mode();
        let video = vec![
            toggle("widescreen", "Widescreen 16:9", false, wide_live, "Hor+ widescreen in matches (renderer-side: the sim is a 4:3 run's)."),
            choice("fullscreen", "Fullscreen", &[("Off", "off"), ("Borderless", "borderless"), ("Exclusive", "exclusive")], "off", false, "Alt+Enter toggles at any time."),
            toggle("vsync", "VSync", true, true, "Off: present as fast as the cap allows."),
            choice("fps_cap", "Frame cap", &[("Off", "off"), ("60", "60"), ("120", "120"), ("144", "144"), ("165", "165"), ("240", "240")], "off", true, "Presented frames per second; the game always runs at 60."),
            choice("interp", "Frame interpolation", &onoff, "auto", true, "Render-only in-between frames (auto: on above 60 fps)."),
            choice("interp_mode", "Interpolation mode", &[("Interpolate", "interpolate"), ("Extrapolate", "extrapolate")], "interpolate", true, "Extrapolate adds no latency."),
            choice("render_scale", "Internal resolution", &[("Auto", "auto"), ("1x", "1"), ("2x", "2"), ("3x", "3"), ("4x", "4"), ("6x", "6"), ("8x", "8")], "auto", true, "x 480 lines."),
            msaa_row(),
        ];
        let audio = vec![Row {
            key: "volume".into(),
            label: "Master volume".into(),
            kind: Kind::Slider { lo: 0, hi: 100, step: 5 },
            value: volume,
            live: true,
            help: "Output gain only; the game's mix is untouched.".into(),
            note: String::new(),
        }];
        let overlays = vec![
            toggle("overlay", "Performance HUD", false, true, "fps, frame time, sim rate; netplay ping / delay / rollbacks (F3)."),
            toggle("input_display", "Input display", false, true, "Sticks, triggers and buttons per port (F6)."),
            toggle("lcancel_indicator", "L-cancel indicator", false, true, "Per player: L-cancel timing of the last landing (F5)."),
            toggle("no_screen_shake", "No screen shake", false, true, "The rendered camera only; the game's camera is untouched (F7)."),
        ];
        let game = vec![
            toggle("skip_intro", "Skip intro movie", false, false, "Boot straight to the title."),
            choice("input_delay", "Offline input delay", &[("0", "0"), ("1", "1"), ("2", "2"), ("3", "3"), ("4", "4"), ("5", "5"), ("6", "6"), ("7", "7"), ("8", "8")], "0", false, "Frames, offline matches only: practise netplay's delay."),
            toggle("early_inputs", "Netplay early inputs", true, false, "Send stick and button changes before the frame they are for."),
        ];
        let mut custom = vec![
            choice("ui_theme", "Appearance", &[("Instrument", "0"), ("Arcade", "1"), ("Brutalist", "2"), ("Neon", "3")], "0", true, "This overlay's style (gd-ui-v1 themes)."),
            choice("ui_palette", "Palette", &PALETTES.iter().enumerate().map(|(i, p)| (p.0, ["0", "1", "2", "3"][i])).collect::<Vec<_>>(), "0", true, "The style's accent colours; each style keeps its own."),
        ];
        let skins = crate::cosmetics::list(&crate::cosmetics::dir());
        let pick = crate::cosmetics::chosen(&skins);
        let mut slots: Vec<String> = skins.iter().map(|s| s.slot.clone()).collect();
        slots.dedup();
        for slot in slots {
            let mut opts = vec![("Vanilla".to_string(), "none".to_string())];
            opts.extend(skins.iter().filter(|s| s.slot == slot).map(|s| (s.name.clone(), s.name.clone())));
            let value = pick.get(&slot).and_then(|n| opts.iter().position(|o| o.1.eq_ignore_ascii_case(n))).unwrap_or(0) as i32;
            custom.push(Row {
                key: format!("skin_{slot}"),
                label: format!("Skin {}", crate::cosmetics::slot_file(&slot).trim_end_matches(".dat")),
                kind: Kind::Choice(opts),
                value,
                live: true,
                help: "Exact-skeleton skins only: hashes cannot change.".into(),
                note: String::new(),
            });
        }
        crate::cosmetics::portrait_rows(&mut custom);
        if custom.len() == 2 {
            custom.push(Row { key: String::new(), label: "No skins (melee cosmetics import FILE)".into(), kind: Kind::Choice(vec![]), value: 0, live: false, help: format!("Catalog: {}", crate::cosmetics::dir().display()), note: String::new() });
        }
        let theme = custom[0].stored().parse().unwrap_or(0);
        let palette = custom[1].stored().parse().unwrap_or(0);
        Overlay {
            open: false,
            pages: vec![Page { name: "VIDEO", rows: video }, Page { name: "AUDIO", rows: audio }, Page { name: "OVERLAYS", rows: overlays }, Page { name: "GAME", rows: game }, Page { name: "CUSTOMIZE", rows: custom }],
            page: 0,
            row: 0,
            owner: None,
            prev: [0; 4],
            held: 0,
            stick_prev: 0,
            f1_prev: false,
            swallow: [0; 4],
            theme,
            palette,
            changes: Vec::new(),
            save: true,
            notice: None,
            own: None,
            shown: false,
            last_quads: 0,
        }
    }

    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// A row's current stored value (the caller syncs rows the keyboard shortcuts changed).
    pub fn set_value(&mut self, key: &str, on: bool) {
        for p in &mut self.pages {
            for r in &mut p.rows {
                if r.key == key && matches!(r.kind, Kind::Toggle) {
                    r.value = on as i32;
                }
            }
        }
    }

    pub fn set_note(&mut self, key: &str, note: String) {
        for p in &mut self.pages {
            for r in &mut p.rows {
                if r.key == key {
                    r.note = note.clone();
                }
            }
        }
    }

    fn toggle_open(&mut self, owner: Option<usize>) {
        self.open = !self.open;
        self.owner = if self.open { owner } else { None };
        self.held = 0;
        eprintln!("settings overlay: {}{}", if self.open { "open" } else { "closed" }, owner.map_or(String::new(), |p| format!(" (port {})", p + 1)));
    }

    fn rows(&self) -> &[Row] {
        &self.pages[self.page].rows
    }

    fn change(&mut self, d: i32) {
        let (pi, ri) = (self.page, self.row);
        let r = &mut self.pages[pi].rows[ri];
        if r.key.is_empty() || !r.step(d) {
            return;
        }
        let (key, value, live) = (r.key.clone(), r.stored(), r.live);
        let env = format!("MELEE_{}", key.to_ascii_uppercase());
        let env_set = std::env::var_os(&env).is_some_and(|v| !v.is_empty());
        if self.save && key != "volume" {
            let mut s = melee_data::settings::Settings::new(melee_data::settings::default_path());
            s.set_str(&key, &value);
        }
        if key == "ui_theme" {
            self.theme = value.parse().unwrap_or(0);
        }
        if key == "ui_palette" {
            self.palette = value.parse().unwrap_or(0);
        }
        let n = if env_set { format!("{env} is set: it wins over settings.cfg") } else if live { "applied".into() } else { "saved: next launch".into() };
        self.notice = Some((n, 150));
        eprintln!("settings overlay: {key} = {value}{}", if live { " (live)" } else { " (next launch)" });
        self.changes.push(Change { key, value, live });
    }

    /// The input boundary. `live`: this tick's controller pads (not a replay's); `four`: what the game gets, masked
    /// on the owner's port while open when `mask` (never for a replay feed). `keys`: gd-ui keyboard events (F1 handled
    /// by the caller as `f1`). Returns whether the keyboard should be captured.
    pub fn input(&mut self, live: &[PADStatus; 4], four: &mut [PADStatus; 4], mask: bool, f1: bool, keys: &[u32]) -> bool {
        let mut edges = [0u16; 4];
        for p in 0..4 {
            edges[p] = live[p].button & !self.prev[p];
            self.prev[p] = live[p].button;
        }
        if f1 && !self.f1_prev {
            self.toggle_open(None);
        }
        self.f1_prev = f1;
        let mut opened_now = false;
        for p in 0..4 {
            if live[p].err == PAD_ERR_NONE && edges[p] & PAD_BUTTON_START != 0 && live[p].button & PAD_BUTTON_DOWN != 0 {
                self.toggle_open(Some(p));
                opened_now = true;
                if mask {
                    // the Start of the combo never reaches the game (no pause)
                    let err = four[p].err;
                    four[p] = PADStatus::zeroed();
                    four[p].err = err;
                }
                break;
            }
        }
        if let Some((_, t)) = self.notice.as_mut() {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.notice = None;
            }
        }
        for p in 0..4 {
            self.swallow[p] &= live[p].button;
            if mask && self.swallow[p] != 0 && !self.open {
                four[p].button &= !self.swallow[p];
            }
        }
        if !self.open {
            return false;
        }
        // navigation from the owner's pad (or any port when opened from the keyboard), and the keyboard
        let mut btn = 0u16;
        let mut ed = 0u16;
        let mut sy = 0i8;
        let mut sx = 0i8;
        for p in 0..4 {
            if self.owner.is_none_or(|o| o == p) {
                btn |= live[p].button;
                ed |= edges[p];
                if live[p].stickY.unsigned_abs() > sy.unsigned_abs() { sy = live[p].stickY }
                if live[p].stickX.unsigned_abs() > sx.unsigned_abs() { sx = live[p].stickX }
            }
        }
        if opened_now {
            ed = 0;
        }
        for &k in keys {
            if k >> 24 != 1 {
                continue;
            }
            ed |= match k & 0xFF_FFFF {
                1 => PAD_BUTTON_A,
                2 => PAD_BUTTON_B,
                4 => PAD_BUTTON_UP,
                5 => PAD_BUTTON_DOWN,
                6 => PAD_BUTTON_LEFT,
                7 => PAD_BUTTON_RIGHT,
                8 => PAD_TRIGGER_R,
                _ => 0,
            };
        }
        // the stick as a d-pad, with key repeat (15 then every 4 frames, the gd-ui node tree's)
        let dir = |v: i8| if v > 50 { 1 } else if v < -50 { -1 } else { 0 };
        let mut sdir = 0u16;
        if dir(sy) > 0 { sdir |= PAD_BUTTON_UP }
        if dir(sy) < 0 { sdir |= PAD_BUTTON_DOWN }
        if dir(sx) > 0 { sdir |= PAD_BUTTON_RIGHT }
        if dir(sx) < 0 { sdir |= PAD_BUTTON_LEFT }
        let dpad = (btn & 0xF) | sdir;
        let snew = sdir & !(self.stick_prev as u16);
        self.stick_prev = sdir as i8;
        let mut nav = (ed & 0xF) | snew;
        if dpad != 0 {
            self.held += 1;
            if self.held > 15 && (self.held - 15) % 4 == 0 {
                nav |= dpad;
            }
        } else {
            self.held = 0;
        }
        let n = self.rows().len().max(1);
        if nav & PAD_BUTTON_UP != 0 {
            self.row = (self.row + n - 1) % n;
        }
        if nav & PAD_BUTTON_DOWN != 0 {
            self.row = (self.row + 1) % n;
        }
        let np = self.pages.len();
        if ed & PAD_TRIGGER_R != 0 || ed & PAD_TRIGGER_Z != 0 {
            self.page = (self.page + 1) % np;
            self.row = 0;
        }
        if ed & PAD_TRIGGER_L != 0 {
            self.page = (self.page + np - 1) % np;
            self.row = 0;
        }
        if nav & PAD_BUTTON_RIGHT != 0 {
            self.change(1);
        }
        if nav & PAD_BUTTON_LEFT != 0 {
            self.change(-1);
        }
        if ed & PAD_BUTTON_A != 0 {
            self.change(1);
        }
        // the owner before a close: the B / Start that closes is the overlay's too (it must not reach the game)
        let owner = self.owner;
        if ed & (PAD_BUTTON_B | PAD_BUTTON_START) != 0 && self.open {
            self.toggle_open(None);
            for p in 0..4 {
                if owner.is_none_or(|o| o == p) {
                    self.swallow[p] = live[p].button;
                }
            }
        }
        // while open the owner's port is the overlay's (an exclusive capture); keyboard-opened: no port masked
        if mask {
            if let Some(o) = owner {
                let err = four[o].err;
                four[o] = PADStatus::zeroed();
                four[o].err = err;
            }
        }
        self.open
    }

    /// Draw into `b` (gd-ui-v1 batch, band space 640x480, layer MODAL).
    pub fn draw(&mut self, b: &mut Batch, a: &Atlas) {
        if !self.open {
            self.last_quads = 0;
            return;
        }
        let q0 = b.len();
        let t = &palette_theme(self.theme, self.palette);
        let mut p = Pen::new(0xFFFF);
        p.layer = cmd::LAYER_MODAL;
        let (x, y, w, h) = (276.0f32, 36.0f32, 350.0f32, 408.0f32);
        let fill = t[theme::TOK_SLICE_FILL as usize] as usize;
        let border = t[theme::TOK_SLICE_BORDER as usize] as usize;
        let sh = t[theme::TOK_SHADOW as usize];
        if sh & 0xFF != 0 {
            let (dx, dy) = (t[theme::TOK_SHADOW_X as usize] as f32, t[theme::TOK_SHADOW_Y as usize] as f32);
            b.rect(&p, a, x + dx, y + dy, w, h, sh, 0.0);
        }
        if t[theme::TOK_GLOW as usize] != 0 {
            b.slice(&p, a, t[theme::TOK_GLOW as usize] as usize, x, y, w, h, t[theme::TOK_BORDER as usize] & 0xFFFF_FF00 | 0x70);
        }
        if fill != 0 { b.slice(&p, a, fill, x, y, w, h, t[theme::TOK_PANEL_BG as usize]) } else { b.rect(&p, a, x, y, w, h, t[theme::TOK_PANEL_BG as usize], 0.0) }
        if border != 0 {
            b.slice(&p, a, border, x, y, w, h, t[theme::TOK_BORDER as usize]);
        }
        let (text, muted, accent, ink, div) = (t[theme::TOK_TEXT as usize], t[theme::TOK_MUTED as usize], t[theme::TOK_ACCENT as usize], t[theme::TOK_ACCENT_INK as usize], t[theme::TOK_DIVIDER as usize]);
        b.text(&p, a, x + 14.0, y + 12.0, ROLE_BIG, text, cmd::ALIGN_LEFT, 0.0, "SETTINGS");
        b.text(&p, a, x + w - 14.0, y + 16.0, cmd::ROLE_LABEL, muted, cmd::ALIGN_RIGHT, 0.0, "L/R page  B close");
        // tabs
        let mut tx = x + 10.0;
        for (i, pg) in self.pages.iter().enumerate() {
            let tw = a.measure(pg.name, cmd::style(0)) + 10.0;
            if i == self.page {
                b.rect(&p, a, tx, y + 40.0, tw, 18.0, accent, 0.0);
            }
            b.text(&p, a, tx + 5.0, y + 43.0, cmd::ROLE_LABEL, if i == self.page { ink } else { muted }, cmd::ALIGN_LEFT, 0.0, pg.name);
            tx += tw + 3.0;
        }
        b.rect(&p, a, x + 10.0, y + 62.0, w - 20.0, 1.0, div, 0.0);
        let rows = &self.pages[self.page].rows;
        let mut ry = y + 70.0;
        for (i, r) in rows.iter().enumerate() {
            let sel = i == self.row;
            if sel {
                b.rect(&p, a, x + 8.0, ry - 3.0, w - 16.0, 30.0, accent & 0xFFFF_FF00 | 0x38, 0.0);
                b.rect(&p, a, x + 8.0, ry - 3.0, 3.0, 30.0, accent, 0.0);
            }
            b.text(&p, a, x + 18.0, ry, ROLE_BODY, if sel { text } else { muted.max(text & 0xFFFF_FFC0) }, cmd::ALIGN_LEFT, 190.0, &r.label);
            let v = r.shown();
            if !v.is_empty() {
                let s = if sel { format!("◀ {v} ▶") } else { v };
                b.text(&p, a, x + w - 16.0, ry, ROLE_VALUE, if sel { accent } else { text }, cmd::ALIGN_RIGHT, 120.0, &s);
            }
            let tag = if !r.note.is_empty() { r.note.as_str() } else if r.live { "live" } else { "next launch" };
            b.text(&p, a, x + 18.0, ry + 14.0, cmd::ROLE_LABEL, muted & 0xFFFF_FFA0, cmd::ALIGN_LEFT, w - 36.0, tag);
            if let Kind::Slider { lo, hi, .. } = r.kind {
                let f = (r.value - lo) as f32 / (hi - lo).max(1) as f32;
                b.rect(&p, a, x + 200.0, ry + 18.0, 100.0, 3.0, div, 0.0);
                b.rect(&p, a, x + 200.0, ry + 18.0, 100.0 * f, 3.0, accent, 0.0);
            }
            ry += 34.0;
        }
        b.rect(&p, a, x + 10.0, y + h - 46.0, w - 20.0, 1.0, div, 0.0);
        if let Some(r) = rows.get(self.row) {
            b.text(&p, a, x + 14.0, y + h - 38.0, cmd::ROLE_LABEL, muted, cmd::ALIGN_LEFT, w - 28.0, &r.help);
        }
        if let Some((n, _)) = &self.notice {
            b.text(&p, a, x + 14.0, y + h - 20.0, ROLE_VALUE, accent, cmd::ALIGN_LEFT, w - 28.0, n);
        }
        self.last_quads = b.len() - q0;
    }

    /// The overlay alone as a frame (no UI mod host): (re)builds the own batch. None while closed.
    pub fn own_frame(&mut self) -> Option<(&melee_gx::ui::UiFrame, &mut Atlas)> {
        if !self.open && !self.shown {
            return None;
        }
        let mut own = self.own.take().unwrap_or_else(|| (Batch::with_capacity(2048), Atlas::new()));
        own.0.begin();
        self.draw(&mut own.0, &own.1);
        own.0.finish(false);
        self.shown = self.open;
        self.own = Some(own);
        let o = self.own.as_mut().unwrap();
        Some((&o.0.frame, &mut o.1))
    }
}

/// Palettes: (name, accent, border) RGBA; the first keeps the style's own colours.
pub const PALETTES: [(&str, u32, u32); 4] = [("Style", 0, 0), ("Cobalt", 0x3D7BFF_FF, 0x8FB4FF_FF), ("Emerald", 0x1FBF75_FF, 0x7CE3B0_FF), ("Crimson", 0xE0344A_FF, 0xFF8C99_FF)];

/// The style's theme tokens with the palette's accent and border.
pub fn palette_theme(style: i32, palette: i32) -> [u32; theme::N_TOKENS as usize] {
    let mut t = theme::THEMES[style.clamp(0, 3) as usize];
    let (_, accent, border) = PALETTES[palette.clamp(0, 3) as usize];
    if accent != 0 {
        t[theme::TOK_ACCENT as usize] = accent;
        t[theme::TOK_BORDER as usize] = border;
    }
    t
}

/// CPU raster of a UI frame over a dark background (`MELEE_OVERLAY_SHOT`, headless checks): band space at 2x.
pub fn raster(frame: &melee_gx::ui::UiFrame, a: &Atlas) -> (u32, u32, Vec<u8>) {
    let (w, h, s) = (1280u32, 960u32, 2.0f32);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for c in px.chunks_mut(4) {
        c.copy_from_slice(&[40, 52, 70, 255]);
    }
    let at = melee_gx::ui::ATLAS as f32;
    for q in &frame.quads {
        let [qx, qy, qw, qh] = q.rect;
        let (x0, y0, x1, y1) = ((qx * s).max(0.0) as i32, (qy * s).max(0.0) as i32, (((qx + qw) * s).min(w as f32)) as i32, (((qy + qh) * s).min(h as f32)) as i32);
        let c = q.rgba.to_be_bytes();
        for yy in y0..y1 {
            for xx in x0..x1 {
                let u = q.uv[0] + (q.uv[2] - q.uv[0]) * ((xx as f32 + 0.5) / s - qx) / qw;
                let v = q.uv[1] + (q.uv[3] - q.uv[1]) * ((yy as f32 + 0.5) / s - qy) / qh;
                let (tu, tv) = (((u * at) as usize).min(1023), ((v * at) as usize).min(1023));
                let tx = &a.px[(tv * 1024 + tu) * 4..][..4];
                let al = c[3] as f32 / 255.0 * tx[3] as f32 / 255.0;
                let o = ((yy as u32 * w + xx as u32) * 4) as usize;
                for k in 0..3 {
                    let src = c[k] as f32 * tx[k] as f32 / 255.0;
                    px[o + k] = (src * al + px[o + k] as f32 * (1.0 - al)) as u8;
                }
            }
        }
    }
    (w, h, px)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(b: u16) -> PADStatus {
        let mut p = PADStatus::zeroed();
        p.button = b;
        p
    }

    #[test]
    fn open_navigate_change_close() {
        let mut o = Overlay::new(40);
        o.save = false;
        let idle = [PADStatus::zeroed(); 4];
        let mut four = idle;
        // hold down, then start on port 2: opens, the Start is masked
        let mut l = idle;
        l[1] = pad(PAD_BUTTON_DOWN);
        o.input(&l, &mut four, true, false, &[]);
        l[1] = pad(PAD_BUTTON_DOWN | PAD_BUTTON_START);
        let mut four = l;
        assert!(o.input(&l, &mut four, true, false, &[]));
        assert_eq!(four[1].button, 0);
        // R twice: OVERLAYS; A toggles the HUD
        for b in [PAD_TRIGGER_R, 0, PAD_TRIGGER_R, 0, PAD_BUTTON_A, 0] {
            let mut l = idle;
            l[1] = pad(b);
            let mut four = l;
            o.input(&l, &mut four, true, false, &[]);
            assert_eq!(four[1].button, 0, "the owner's port is masked while open");
        }
        let c = o.take_changes();
        assert_eq!(c, vec![Change { key: "overlay".into(), value: "1".into(), live: true }]);
        // other ports reach the game untouched
        let mut l = idle;
        l[0] = pad(PAD_BUTTON_A);
        let mut four = l;
        o.input(&l, &mut four, true, false, &[]);
        assert_eq!(four[0].button, PAD_BUTTON_A);
        // a replay feed is never masked
        let mut l = idle;
        l[1] = pad(PAD_BUTTON_X);
        let mut four = l;
        o.input(&l, &mut four, false, false, &[]);
        assert_eq!(four[1].button, PAD_BUTTON_X);
        // draw: quads, then B closes
        let mut b = Batch::with_capacity(256);
        let a = Atlas::new();
        o.draw(&mut b, &a);
        assert!(o.last_quads > 50);
        let mut l = idle;
        l[1] = pad(PAD_BUTTON_B);
        let mut four = l;
        assert!(!o.input(&l, &mut four, true, false, &[]));
        assert_eq!(four[1].button, 0, "the closing B never reaches the game");
        let mut four = l;
        o.input(&l, &mut four, true, false, &[]);
        assert_eq!(four[1].button, 0, "nor while it stays held");
        let mut four = idle;
        o.input(&idle, &mut four, true, false, &[]);
        let mut l = idle;
        l[1] = pad(PAD_BUTTON_B);
        let mut four = l;
        o.input(&l, &mut four, true, false, &[]);
        assert_eq!(four[1].button, PAD_BUTTON_B, "a new press after the release does");
        // keyboard: F1 opens, Escape closes
        o.input(&idle, &mut idle.clone(), true, true, &[]);
        assert!(o.open);
        o.input(&idle, &mut idle.clone(), true, false, &[1 << 24 | 2]);
        assert!(!o.open);
    }
}

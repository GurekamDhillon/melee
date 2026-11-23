//! The windowed view policy: widescreen (16:9 Hor+) the way the C build does it (pc/platform/shim_vi.c
//! `gw_video_sync_viewport_policy`, the maths in gw_view_math.h).
//!
//! When widescreen is on and the current scene is one of the wide scenes (`gw_view_scene_wide`: VS, sudden death,
//! training, the camera VS scene and the kit frontend), the content aspect is the window's, clamped to 4:3..32:9;
//! everything else stays 4:3. Every perspective camera's authored aspect is scaled by
//! `gw_view_projection_scale(aspect, wide) = aspect * 60/73` (`HSD_CObjUpdateView` reads it through
//! `View_ProjectionScaleBits`), orthographic cameras keep their 0..640 logical range and the renderer maps it across the
//! wider target (Aurora's AURORA_VIEWPORT_ASPECT), and the text canvases that ask for it (`HSD_SisLib_SetWideCanvas`)
//! widen with `View_AspectBits`.
//!
//! The simulation DOES read the hooks' answers (Camera_LogicToScreen's offscreen flags, Pokemon Stadium's display
//! procs), so what they answer is a session setting, the *sim view* ([`set_sim_view`]): 4:3, or a fixed wide content
//! aspect (camera widescreen). It is decided once per session, never by the live window: the local default comes from
//! the display options when `melee play` starts ([`local_sim_view`]); a netplay guest adopts the host's (the agreed
//! scene's `np_rules` `v` field); a recording (.mrpl `sim_view`, .slp metadata `simView`) carries it to its playback.
//! The window then only decides how the picture is fitted (letterbox / pillarbox, the renderer-side widening).
//! Runs with no sim view (golden / gate / slpcheck of console replays) install the C recordings' fixed values instead
//! of these hooks (engine.rs `boot_host`).
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering::Relaxed};

pub const MIN_ASPECT: f32 = 4.0 / 3.0;
pub const MAX_ASPECT: f32 = 32.0 / 9.0;

static WIDE: AtomicBool = AtomicBool::new(false);
/// The window's client aspect as f32 bits (0: no window yet).
static WINDOW: AtomicU32 = AtomicU32::new(0);
static SCENE: AtomicI32 = AtomicI32::new(-1);
/// Widescreen through the cameras (the C build's way, `widescreen_mode=camera`) instead of the renderer. Off: the
/// game's hooks keep answering 4:3 and only the presenter widens perspective draws (melee_gx `interp::Widen`), so a
/// widescreen run simulates exactly what a 4:3 one does.
static CAMERA_MODE: AtomicBool = AtomicBool::new(false);
/// The session's sim view: [`SIM_UNSET`], 0 (4:3) or the wide content aspect as f32 bits.
static SIM: AtomicU32 = AtomicU32::new(SIM_UNSET);
pub const SIM_UNSET: u32 = u32::MAX;

/// Set the session's sim view (0 = 4:3, else the wide content aspect's f32 bits, clamped to 4:3..32:9).
pub fn set_sim_view(bits: u32) {
    let b = if bits == 0 || bits == SIM_UNSET {
        bits
    } else {
        let a = f32::from_bits(bits);
        if !a.is_finite() || a <= MIN_ASPECT { 0 } else { a.min(MAX_ASPECT).to_bits() }
    };
    SIM.store(b, Relaxed);
}

/// The session's sim view, None before one was set (golden / gate / console-replay runs).
pub fn sim_view() -> Option<u32> {
    let b = SIM.load(Relaxed);
    (b != SIM_UNSET).then_some(b)
}

/// The sim view as the hooks use it: 0 (4:3) when none was set.
fn sim_bits() -> u32 {
    sim_view().unwrap_or(0)
}

/// This machine's own sim view from its display options: the camera widescreen mode with widescreen on gives the
/// configured window's content aspect (16:9 without a size); anything else is 4:3 (renderer-side widescreen leaves
/// the simulation a 4:3 run's).
pub fn local_sim_view() -> u32 {
    if !(camera_mode() && widescreen()) {
        return 0;
    }
    let win = f32::from_bits(WINDOW.load(Relaxed));
    let a = if win > 0.0 { content_aspect(win, 1.0, true) } else { 16.0 / 9.0 };
    if a <= MIN_ASPECT { 0 } else { a.to_bits() }
}

/// The text form (replay headers, .slp metadata): `4:3` or `wide:<f32 bits hex>`.
pub fn encode_sim_view(bits: u32) -> String {
    if bits == 0 { "4:3".into() } else { format!("wide:{bits:08x}") }
}

pub fn parse_sim_view(s: &str) -> Option<u32> {
    match s.trim() {
        "4:3" => Some(0),
        t => {
            let b = u32::from_str_radix(t.strip_prefix("wide:")?, 16).ok()?;
            let a = f32::from_bits(b);
            (a.is_finite() && a > MIN_ASPECT && a <= MAX_ASPECT).then_some(b)
        }
    }
}

/// Renderer-side widening applies (the presenter widens perspective draws): widescreen in the renderer mode, and the
/// session's cameras are not already wide.
pub fn renderer_widen() -> bool {
    widescreen() && !camera_mode() && sim_bits() == 0
}

pub fn set_camera_mode(on: bool) {
    CAMERA_MODE.store(on, Relaxed);
}
pub fn camera_mode() -> bool {
    CAMERA_MODE.load(Relaxed)
}

/// `gw_view_scene_wide`: GameSceneKind values whose content may be wide (matches and the kit frontend). The native
/// menus (title, main menu, CSS, SSS, results) keep their authored 4:3 and are pillarboxed.
pub fn scene_wide(scene: i32) -> bool {
    matches!(scene, 2 | 3 | 4 | 44 | 46)
}

/// `gw_view_projection_scale`.
pub fn projection_scale(aspect: f32, wide: bool) -> f32 {
    if wide {
        aspect * (60.0 / 73.0)
    } else {
        1.0
    }
}

/// `gw_view_aspect`: the content aspect of a `w` x `h` window.
pub fn content_aspect(w: f32, h: f32, wide: bool) -> f32 {
    let a = if wide && w > 0.0 && h > 0.0 { w / h } else { MIN_ASPECT };
    a.clamp(MIN_ASPECT, MAX_ASPECT)
}

/// `gw_view_default_wide`: the automatic default (a window wider than 4:3); `None` without a size.
pub fn default_wide(w: u32, h: u32) -> Option<bool> {
    (w > 0 && h > 0).then(|| w as u64 * 3 > h as u64 * 4)
}

/// Widescreen on or off (the `--widescreen` option / `MELEE_WIDESCREEN` / settings.cfg `widescreen`).
pub fn set_widescreen(on: bool) {
    WIDE.store(on, Relaxed);
}

pub fn widescreen() -> bool {
    WIDE.load(Relaxed)
}

/// The window's client area (any unit: only the ratio is used).
pub fn set_window(w: u32, h: u32) {
    if w > 0 && h > 0 {
        WINDOW.store((w as f32 / h as f32).to_bits(), Relaxed);
    }
}

/// `View_SceneBegin` (gm_801A4B88): the GameSceneKind of the scene being entered.
pub fn scene_begin(scene: i32) {
    SCENE.store(scene, Relaxed);
}

/// The current GameSceneKind: the scene loop's own `gm_804D6720` (what gm_801A4B88 passes to View_SceneBegin; the
/// port's View_SceneBegin seam is not routed outside the state oracle), else the last [`scene_begin`].
pub fn scene() -> i32 {
    let info = unsafe { melee_modes::tu::gmscene::gm_804D6720 };
    if !info.is_null() {
        return unsafe { (*info).scene_kind } as i32;
    }
    SCENE.load(Relaxed)
}

/// Widescreen applies to the current scene.
pub fn wide_now() -> bool {
    widescreen() && scene_wide(scene())
}

/// The content aspect for the current scene and window (4:3 unless [`wide_now`]).
pub fn aspect() -> f32 {
    let s = sim_bits();
    if s != 0 {
        // the session's cameras are wide: the content is exactly that aspect (fitted into whatever window)
        return if scene_wide(scene()) { f32::from_bits(s) } else { MIN_ASPECT };
    }
    if camera_mode() {
        return MIN_ASPECT; // a 4:3 session in the camera mode: pillarboxed
    }
    let win = f32::from_bits(WINDOW.load(Relaxed));
    let wide = wide_now();
    if wide && win > 0.0 {
        content_aspect(win, 1.0, true)
    } else {
        MIN_ASPECT
    }
}

/// The content aspect a widened scene (a match) will have in this window: the presenter makes its EFB targets ahead.
pub fn wide_aspect() -> f32 {
    let s = sim_bits();
    if s != 0 {
        return f32::from_bits(s);
    }
    if camera_mode() {
        return MIN_ASPECT;
    }
    let win = f32::from_bits(WINDOW.load(Relaxed));
    if widescreen() && win > 0.0 {
        content_aspect(win, 1.0, true)
    } else {
        MIN_ASPECT
    }
}

/// `gw_View_ProjectionScaleBits`.
pub fn projection_scale_bits() -> i32 {
    let s = sim_bits();
    if s == 0 || !scene_wide(scene()) {
        return 1.0f32.to_bits() as i32;
    }
    projection_scale(f32::from_bits(s), true).to_bits() as i32
}

/// `gw_View_AspectBits`.
pub fn aspect_bits() -> i32 {
    let s = sim_bits();
    if s == 0 || !scene_wide(scene()) {
        return MIN_ASPECT.to_bits() as i32;
    }
    s as i32
}

/// Route the game's view hooks here (windowed play only; see the module doc).
pub fn install() {
    unsafe {
        melee_runtime::tu::cobj::ext::hook::View_ProjectionScaleBits = Some(projection_scale_bits);
        melee_runtime::tu::sislib::ext::hook::View_AspectBits = Some(aspect_bits);
        melee_world::tu::ifmagnify::ext::hook::View_AspectBits = Some(aspect_bits);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_view_math() {
        // pc/tests/view_aspect_cases.h: the 16:9 scale is 320/219
        assert!((projection_scale(16.0 / 9.0, true) - 320.0 / 219.0).abs() < 1e-6);
        assert_eq!(projection_scale(16.0 / 9.0, false), 1.0);
        assert_eq!(content_aspect(1280.0, 960.0, true), MIN_ASPECT);
        assert_eq!(content_aspect(1920.0, 1080.0, false), MIN_ASPECT);
        assert!((content_aspect(1920.0, 1080.0, true) - 16.0 / 9.0).abs() < 1e-6);
        assert_eq!(content_aspect(10000.0, 1000.0, true), MAX_ASPECT);
        assert_eq!(content_aspect(600.0, 800.0, true), MIN_ASPECT);
        assert_eq!(default_wide(800, 600), Some(false));
        assert_eq!(default_wide(1280, 720), Some(true));
        assert_eq!(default_wide(0, 0), None);
        assert!(scene_wide(2) && scene_wide(4) && !scene_wide(8) && !scene_wide(5) && !scene_wide(1));
    }

    #[test]
    fn hooks_follow_the_session_not_the_window() {
        set_camera_mode(true);
        set_window(1920, 1080);
        set_widescreen(false);
        assert_eq!(local_sim_view(), 0);
        set_widescreen(true);
        let wide = local_sim_view();
        assert!((f32::from_bits(wide) - 16.0 / 9.0).abs() < 1e-6);
        set_sim_view(wide);
        scene_begin(2);
        assert_eq!(f32::from_bits(projection_scale_bits() as u32), projection_scale(16.0 / 9.0, true));
        assert_eq!(aspect_bits(), wide as i32);
        // a resize changes nothing the simulation sees
        set_window(800, 600);
        assert_eq!(f32::from_bits(projection_scale_bits() as u32), projection_scale(16.0 / 9.0, true));
        scene_begin(8); // the CSS stays 4:3
        assert_eq!(projection_scale_bits(), 1.0f32.to_bits() as i32);
        assert_eq!(aspect(), MIN_ASPECT);
        // a 4:3 session (e.g. the host's) in a wide camera-mode window: the hooks answer 4:3
        scene_begin(2);
        set_sim_view(0);
        set_window(1920, 1080);
        assert_eq!(projection_scale_bits(), 1.0f32.to_bits() as i32);
        assert_eq!(aspect_bits(), MIN_ASPECT.to_bits() as i32);
        assert_eq!(aspect(), MIN_ASPECT);
        // renderer mode: the game sees 4:3, the presenter widens
        set_camera_mode(false);
        assert_eq!(local_sim_view(), 0);
        assert!((aspect() - 16.0 / 9.0).abs() < 1e-6 && renderer_widen());
        // ... unless the session's cameras are wide (a camera-mode host): no double widening
        set_sim_view(wide);
        assert!(!renderer_widen() && (aspect() - 16.0 / 9.0).abs() < 1e-6);
        assert_eq!(parse_sim_view(&encode_sim_view(wide)), Some(wide));
        assert_eq!(parse_sim_view("4:3"), Some(0));
        assert_eq!(parse_sim_view("wide:3f800000"), None);
        set_widescreen(false);
        set_sim_view(SIM_UNSET);
        scene_begin(-1);
    }
}

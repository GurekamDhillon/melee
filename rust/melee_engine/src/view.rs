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
//! The simulation never reads this: the C keeps logic on the authored aspect (`Camera_LogicToScreen`,
//! `HSD_CObjGetAuthoredAspect`), and the headless engine (golden / gate runs) installs its own fixed values instead
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
    let win = f32::from_bits(WINDOW.load(Relaxed));
    let wide = wide_now();
    if wide && win > 0.0 {
        content_aspect(win, 1.0, true)
    } else {
        MIN_ASPECT
    }
}

/// `gw_View_ProjectionScaleBits`.
pub fn projection_scale_bits() -> i32 {
    if !camera_mode() {
        return 1.0f32.to_bits() as i32;
    }
    projection_scale(aspect(), wide_now()).to_bits() as i32
}

/// `gw_View_AspectBits`.
pub fn aspect_bits() -> i32 {
    if !camera_mode() {
        return MIN_ASPECT.to_bits() as i32;
    }
    aspect().to_bits() as i32
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
    fn hooks_follow_scene_and_setting() {
        set_camera_mode(true);
        set_window(1920, 1080);
        set_widescreen(false);
        scene_begin(2);
        assert_eq!(aspect_bits(), MIN_ASPECT.to_bits() as i32);
        assert_eq!(projection_scale_bits(), 1.0f32.to_bits() as i32);
        set_widescreen(true);
        assert!((aspect() - 16.0 / 9.0).abs() < 1e-6);
        assert_eq!(f32::from_bits(projection_scale_bits() as u32), projection_scale(16.0 / 9.0, true));
        scene_begin(8); // the CSS stays 4:3
        assert_eq!(aspect(), MIN_ASPECT);
        assert_eq!(projection_scale_bits(), 1.0f32.to_bits() as i32);
        set_widescreen(false);
        scene_begin(-1);
        // renderer mode: the game always sees 4:3
        set_camera_mode(false);
        set_widescreen(true);
        scene_begin(2);
        assert!((aspect() - 16.0 / 9.0).abs() < 1e-6);
        assert_eq!(projection_scale_bits(), 1.0f32.to_bits() as i32);
        assert_eq!(aspect_bits(), MIN_ASPECT.to_bits() as i32);
        set_widescreen(false);
        scene_begin(-1);
    }
}

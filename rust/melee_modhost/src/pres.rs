//! Presentation interface stubs (gd-ui-v1 / gd-lab-v1, docs/rust-port/notes/wasm-ui-lab-api.md section C).
//! Shared between the UI half (A) and the Lab half (B). Nothing here is wired into the host yet.
//! Constants mirror docs/specs/gd-ui-lab-v1.h.

use melee_gx::draw::DrawCmd;
use melee_types::PADStatus;
use std::any::Any;

/// gd-ui-v1 / gd-lab-v1 ABI (major<<16 | minor); a mod's `gdmod_ui_abi` / `gdmod_lab_abi` must match the major
/// and may have a lower minor.
pub const UI_ABI_VERSION: u32 = 1 << 16;
pub const LAB_ABI_VERSION: u32 = 1 << 16;

pub const FEATURE_UI_DRAW: u32 = 1 << 4;
pub const FEATURE_UI_MENU: u32 = 1 << 5;
pub const FEATURE_UI_INPUT: u32 = 1 << 6;
pub const FEATURE_UI_SCENE: u32 = 1 << 7;
pub const FEATURE_UI_SIM_INPUT: u32 = 1 << 8;
pub const FEATURE_UI_HTML: u32 = 1 << 9; // reserved, refused by v1
pub const FEATURE_LAB: u32 = 1 << 10;
pub const FEATURE_LAB_STATE: u32 = 1 << 11;
pub const FEATURE_LAB_INPUT: u32 = 1 << 12;
pub const FEATURE_LAB_CPU: u32 = 1 << 13;
pub const FEATURE_LAB_OVERLAY: u32 = 1 << 14;
pub const FEATURE_UI_VECTOR: u32 = 1 << 15;
pub const FEATURES_UI: u32 = FEATURE_UI_DRAW | FEATURE_UI_MENU | FEATURE_UI_INPUT | FEATURE_UI_SCENE
    | FEATURE_UI_SIM_INPUT | FEATURE_UI_HTML | FEATURE_UI_VECTOR;
pub const FEATURES_LAB: u32 = FEATURE_LAB | FEATURE_LAB_STATE | FEATURE_LAB_INPUT | FEATURE_LAB_CPU | FEATURE_LAB_OVERLAY;

pub const GD_OK: i32 = 0;
pub const GD_EINVAL: i32 = -1;
pub const GD_ECAP: i32 = -2;
pub const GD_EOFFLINE: i32 = -3;
pub const GD_ESTATE: i32 = -4;
pub const GD_EFULL: i32 = -5;

/// The play session the presentation layer runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Session {
    Offline,
    /// Two-player online Lab: Lab actions travel as synced lockstep commands and apply on the same frame on both
    /// peers; Lab mods take part in the netplay mod check.
    LabNetplay,
    Netplay,
    Synctest,
    Replay,
}

impl Session {
    /// Whether `gd_lab_*` calls are allowed; otherwise they return `GD_EOFFLINE`.
    pub fn lab_allowed(self) -> bool {
        matches!(self, Session::Offline | Session::LabNetplay)
    }
}

pub struct PresFrame {
    pub session: Session,
    pub scene: (i32, i32),
    pub match_frame: i32,
}

/// A implements; B calls it (record, play, synthesise).
pub trait InputBoundary {
    /// This frame's local pad after captures.
    fn read(&self, port: usize) -> PADStatus;
    /// B2/B3 writes; session-checked (Offline / LabNetplay as a synced command).
    fn set(&mut self, port: usize, pad: PADStatus, src: u32);
}

/// B implements; A's host calls it.
pub trait PresExtension {
    /// LAB..LAB_OVERLAY.
    fn feature_bits(&self) -> u32;
    fn link(&self, l: &mut wasmi::Linker<PresData>) -> Result<(), String>;
    fn on_session(&mut self, s: Session);
    /// Inject inputs, pending load.
    fn before_logic(&mut self, f: &PresFrame, io: &mut dyn InputBoundary);
    /// Ring capture.
    fn after_logic(&mut self, f: &PresFrame);
    /// B4 into the GX pass.
    fn overlay(&mut self, f: &PresFrame, out: &mut Vec<DrawCmd>);
}

/// Per-mod presentation store data (stub).
#[derive(Default)]
pub struct PresData {
    pub features: u32,
    pub captures: u32,
    pub draws: Vec<DrawCmd>,
    /// B's per-mod state.
    pub ext: Option<Box<dyn Any>>,
}

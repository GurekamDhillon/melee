//! Platform layer, step 1: window + 60 Hz main loop (winit), PAD input, headless mode.
//!
//! * [`pad`]: `PADStatus` (melee_types, with its GC layout), constants, `PADClamp`.
//! * [`adapter`]: the raw GameCube USB adapter (port of `pc/platform/gc_adapter.c`).
//! * [`aurora`]: Aurora's `PADRead` arithmetic over keyboard bindings and SDL-style gamepads.
//! * [`profiles`], [`socd`]: per-controller profiles (deadzones, remaps, box mode + SOCD).
//! * [`gamepads`]: gilrs -> SDL-style raw gamepads.
//! * [`script`]: the txt pad script (`MELEE_PAD_SCRIPT`).
//! * [`pad_read`]: the game-facing `PADRead` (port of `shim_pad.c` `gw_PADRead`).
//! * [`main_loop`]: fixed-step logic decoupled from presentation; headless runner.
//! * [`gfx`]: swapchain + present of melee_gx's EFB on the window.
pub mod log;
pub mod pad;
pub mod adapter;
pub mod aurora;
pub mod bindings;
pub mod socd;
pub mod profiles;
pub mod gamepads;
pub mod script;
pub mod pad_read;
pub mod main_loop;
pub mod gfx;
pub mod overlay;

pub use main_loop::{run_headless, run_windowed, FixedStep, FullscreenMode, Game, WindowConfig};
pub use pad::PADStatus;
pub use pad_read::{InputMode, PadSystem, Source};

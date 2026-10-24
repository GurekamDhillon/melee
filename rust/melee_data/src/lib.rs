//! Saves, settings, mods and replays: the data-file half of pc/platform.
//!
//! * [`settings`]: `settings.cfg` (pc/platform/gw_settings.c).
//!
//! Everything here is checked against the C build's own code or files; see tests/.

pub mod replay;
pub mod settings;

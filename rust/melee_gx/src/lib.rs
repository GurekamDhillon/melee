//! GX on wgpu (platform track, step 2): a native port of the GX implementation the C build gets
//! from Aurora (`extern/aurora/lib/gx`, `lib/dolphin/gx`, `lib/gfx/texture_convert.cpp`).
//!
//! * [`api::Gx`]: the GX API (GXSetTevOrder, GXLoadTexObj, GXBegin...), encoding register writes
//!   into a FIFO exactly as Aurora's encoders do, plus display-list record / call.
//! * [`state::GxState`]: the command processor (BP/CP/XF decode) and the decoded GX state.
//! * [`draw`]: draws resolved against that state (vertex bytes, index buffer, arrays, uniforms).
//! * [`shader`]: TEV/vertex pipeline -> WGSL, byte-identical to Aurora's generator.
//! * [`texture`]: GameCube texture formats -> RGBA8.
//! * [`render::Renderer`]: wgpu backend drawing [`draw::DrawCmd`]s into an offscreen EFB, blitted
//!   to a swapchain by [`render::Renderer::blit_to`].
//! * [`shim::GxShim`]: port of `pc/platform/shim_gx.c`, the layer game GX calls go through.
//!
//! Verified against Aurora itself (oracle/ + tests/oracle.rs): texture decode and WGSL text bit for
//! bit, GX state after API sequences, and an offscreen render of a test display list against
//! Aurora's own screenshot. See RUST_PORT.md "Platform track, step 2".
#![allow(non_upper_case_globals)]

pub mod api;
pub mod consts;
#[cfg(feature = "corev1")]
pub mod corev1;
pub mod draw;
pub mod fx;
pub mod interp;
pub mod render;
pub mod resources;
pub mod shader;
pub mod shim;
pub mod state;
pub mod tex_override;
pub mod texture;
pub mod ui;
pub mod vfx;
pub mod ui_font;

pub use api::{Gx, GxColor, GxLightObj};
pub use resources::GxData;

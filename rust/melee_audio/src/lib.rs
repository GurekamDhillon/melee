//! Audio platform layer: a native port of `pc/platform/shim_ax.c`.
//!
//! - [`adpcm`]: the per-voice decoder (Nintendo DSP-ADPCM, PCM16, PCM8) over nibble-addressed ARAM.
//! - [`fx`]: the AXFX reverb-std and delay processors on the aux buses.
//! - [`ax`]: the 64-voice AXVPB pool, the AX entry points (parameters arrive as big-endian game
//!   memory blocks, exactly as the shim reads them), the 5 ms sub-frame mixer, de-pop, native
//!   clips, and the game-visible VPB writeback.
//! - [`output`]: the frame driver (device-clock pacing, wall clock without a device), master volume
//!   (`MELEE_VOLUME` / `audio.cfg`), `MELEE_AUDIO_DUMP` WAV capture and the cpal device on a
//!   lock-free SPSC ring.
//!
//! Everything up to the 160-sample s16 output of one sub-frame is checked bit for bit against the
//! shim itself, compiled as the shipping build compiles it (`oracle/build.sh`, `tests/oracle.rs`).

pub mod adpcm;
pub mod ax;
pub mod fx;
pub mod layout;
pub mod output;

/// 64 voices, like the shim's pool.
pub const NUM_VOICES: usize = 64;
/// 5 ms at 32 kHz.
pub const FRAME_SAMPLES: usize = 160;
/// Output rate (Hz).
pub const RATE: u32 = 32000;

pub use ax::Ax;

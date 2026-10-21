//! The AX voice pool and mixer: shim_ax.c's 35 AX entry points, `gw_ax_run_frame` and the native
//! clips (Geno slice 6), as one owned [`Ax`] value instead of file statics.
//!
//! Parameter blocks (`AXPBMIX`, `AXPBVE`, `AXPBADDR`, `AXPBADPCM`, `AXPBSRC`, `AXPBADPCMLOOP`, the
//! AXFX structs) are taken as big-endian game-memory bytes, read exactly where the shim reads them
//! (offsets in [`crate::layout`]). Each voice's 0x1F8-byte AXVPB is kept in game byte order too, so
//! the fields the synth reads back (pb.state, pb.itd.flag, pb.ve.currentVolume,
//! pb.addr.currentAddress, index...) are where synth.c expects them. A voice is named by its pool
//! index (the C's `voice - gw_ax_voices`); game pointers (callbacks, AXFX owners) are opaque u32
//! game addresses, 0 meaning NULL.

use std::sync::Arc;

use crate::adpcm::Decoder;
use crate::fx::{Delay, ReverbStd};
use crate::layout::*;
use crate::{FRAME_SAMPLES, NUM_VOICES};

const ITD_MAX: usize = 32; // lbl_8040806C in synth.c tops out at a 0x1F-sample shift
const MIX_COUNT: usize = 9;
const MIX_L: usize = 0;
const MIX_R: usize = 1;
const MIX_S: usize = 2;
const MIX_AL: usize = 3;
const MIX_AR: usize = 4;
const MIX_AS: usize = 5;
const MIX_BL: usize = 6;
const MIX_BR: usize = 7;
const MIX_BS: usize = 8;
const BUS_SAMPLES: usize = FRAME_SAMPLES * 3;
const FX_SLOTS: usize = 4;
const CLIPS: usize = 8;

/// Host-side per-voice mixer state (shim_ax.c `gw_ax_vstate`), never game-visible.
#[derive(Clone, Debug)]
pub struct Voice {
    /// Decoder; also carries `active` and `state`.
    pub dec: Decoder,
    /// AXSetVoiceAddr + AXSetVoiceAdpcm seen.
    pub configured: bool,
    /// 16.16 resampling ratio and the phase between `s_prev` and `s_next`.
    pub ratio: u32,
    pub frac: u32,
    pub s_prev: i16,
    pub s_next: i16,
    pub primed: bool,
    /// ve.currentVolume 0..32767 and its per-sample delta.
    pub volume: i32,
    pub volume_delta: i16,
    pub mix: [i32; MIX_COUNT],
    pub mix_delta: [i16; MIX_COUNT],
    pub mix_ramping: bool,
    pub itd_on: bool,
    pub itd_hist: [i16; ITD_MAX],
    pub itd_pos: i32,
    pub itd_l: i32,
    pub itd_r: i32,
    pub itd_tl: i32,
    pub itd_tr: i32,
    /// Last main L/R/S contribution, for the de-pop ramp.
    pub last_out: [i32; 3],
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            dec: Decoder::default(),
            configured: false,
            ratio: 0,
            frac: 0,
            s_prev: 0,
            s_next: 0,
            primed: false,
            volume: 0,
            volume_delta: 0,
            mix: [0; MIX_COUNT],
            mix_delta: [0; MIX_COUNT],
            mix_ramping: false,
            itd_on: false,
            itd_hist: [0; ITD_MAX],
            itd_pos: 0,
            itd_l: 0,
            itd_r: 0,
            itd_tl: 0,
            itd_tr: 0,
            last_out: [0; 3],
        }
    }
}

/// One aux bus, laid out like the DSP's (AXAux.c): L at [0], R at [160], S at [320].
#[derive(Clone, Debug)]
struct Aux {
    /// An AXFX*Callback is registered (the bus is on).
    on: bool,
    /// Its context: the game address of the AXFX struct.
    ctx: u32,
    bus: [i32; BUS_SAMPLES],
}

#[derive(Clone, Debug, Default)]
enum FxKind {
    #[default]
    None,
    Reverb(ReverbStd),
    Delay(Delay),
}

#[derive(Clone, Debug, Default)]
struct FxSlot {
    /// Game address of the AXFX_* struct, 0 = free.
    owner: u32,
    kind: FxKind,
}

#[derive(Clone, Debug, Default)]
struct Clip {
    pcm: Option<Arc<[i16]>>,
    frames: u32,
    pos: u32,
    channels: usize,
    gain: i32,
    age: u32,
    owner: u32,
}

/// The whole AX state.
pub struct Ax {
    vpb: Box<[[u8; VPB_SIZE]; NUM_VOICES]>,
    in_use: [bool; NUM_VOICES],
    voices: Box<[Voice]>,
    aux_a: Aux,
    aux_b: Aux,
    fx: [FxSlot; FX_SLOTS],
    /// MELEE_AUDIO_NOFX: bypass the aux processors (the sends still sum in dry).
    pub nofx: bool,
    acc: [[i32; FRAME_SAMPLES]; 3],
    out: [i16; FRAME_SAMPLES * 2],
    peak: i32,
    dpop: [i32; 3],
    dpop_step: [i32; 3],
    clips: [Clip; CLIPS],
    clip_age: u32,
    clips_started: u32,
    clip_frames_mixed: u32,
    frames_run: u32,
    reported_first: bool,
    /// AISetStreamVol / AISetDSPSampleRate state (shim_misc.c). Deliberately not applied: the
    /// music volume is already folded into every voice's ve.currentVolume (synth.c:986, :1345).
    pub ai_stream_vol_left: u8,
    pub ai_stream_vol_right: u8,
    pub ai_dsp_sample_rate: u32,
    /// Underrun/overrun counts for the heartbeat log (set by the output side).
    pub underruns: u32,
    pub overruns: u32,
    /// gw_log stand-in.
    pub log: Option<Box<dyn FnMut(&str) + Send>>,
}

impl Default for Ax {
    fn default() -> Self {
        Self::new()
    }
}

/// What [`Ax::acquire_voice`] stole, when the pool was full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stolen {
    pub index: usize,
    /// The previous owner's AXVPB.callback (game address, non-zero).
    pub callback: u32,
}

fn depop_voice(v: &mut Voice, dpop: &mut [i32; 3], step: &mut [i32; 3]) {
    for c in 0..3 {
        let x = v.last_out[c];
        v.last_out[c] = 0;
        if x == 0 {
            continue;
        }
        dpop[c] = dpop[c].wrapping_add(x);
        step[c] = dpop[c] / FRAME_SAMPLES as i32;
        if step[c] == 0 {
            step[c] = if dpop[c] > 0 { 1 } else { -1 };
        }
    }
}

fn stop_voice(v: &mut Voice, dpop: &mut [i32; 3], step: &mut [i32; 3]) {
    if v.dec.active {
        v.dec.active = false;
        depop_voice(v, dpop, step);
    }
}

#[inline]
fn send(x: i32, m: i32) -> i32 {
    x.wrapping_mul(m) >> 15
}

/// shim_ax.c `gw_ax_mix_voice`.
fn mix_voice(
    s: &mut Voice,
    aram: &[u8],
    acc: &mut [[i32; FRAME_SAMPLES]; 3],
    bus_a: &mut [i32; BUS_SAMPLES],
    bus_b: &mut [i32; BUS_SAMPLES],
    dpop: &mut [i32; 3],
    dpop_step: &mut [i32; 3],
) {
    let mut ended = false;
    const F: usize = FRAME_SAMPLES;

    if !s.primed {
        s.s_prev = s.dec.decode_one(aram);
        s.s_next = if s.dec.active { s.dec.decode_one(aram) } else { 0 };
        s.frac = 0;
        s.primed = true;
        if !s.dec.active {
            ended = true;
        }
    }

    let mut i = 0;
    while i < F && !ended {
        // Linear interpolation (AX_SRC_TYPE_LINEAR).
        let sample = (s.s_prev as i32).wrapping_add(
            (((s.s_next as i32 - s.s_prev as i32) as i64 * s.frac as i64) >> 16) as i32,
        );

        s.volume = s.volume.wrapping_add(s.volume_delta as i32).clamp(0, 32767);

        if s.mix_ramping {
            for t in 0..MIX_COUNT {
                s.mix[t] = s.mix[t].wrapping_add(s.mix_delta[t] as i32).clamp(0, 0x8000);
            }
        }

        let v = sample.wrapping_mul(s.volume) >> 15;

        // ITD: the main L and R sends delayed independently by up to 31 samples.
        s.itd_hist[s.itd_pos as usize] = v as i16;
        let (vl, vr) = if s.itd_on {
            (
                s.itd_hist[((s.itd_pos - s.itd_l) & (ITD_MAX as i32 - 1)) as usize] as i32,
                s.itd_hist[((s.itd_pos - s.itd_r) & (ITD_MAX as i32 - 1)) as usize] as i32,
            )
        } else {
            (v, v)
        };
        s.itd_pos = (s.itd_pos + 1) & (ITD_MAX as i32 - 1);

        s.last_out[0] = send(vl, s.mix[MIX_L]);
        s.last_out[1] = send(vr, s.mix[MIX_R]);
        s.last_out[2] = send(v, s.mix[MIX_S]);
        acc[0][i] = acc[0][i].wrapping_add(s.last_out[0]);
        acc[1][i] = acc[1][i].wrapping_add(s.last_out[1]);
        if s.mix[MIX_S] != 0 {
            acc[2][i] = acc[2][i].wrapping_add(s.last_out[2]);
        }
        for (t, o) in [(MIX_AL, 0), (MIX_AR, F), (MIX_AS, 2 * F)] {
            if s.mix[t] != 0 {
                bus_a[o + i] = bus_a[o + i].wrapping_add(send(v, s.mix[t]));
            }
        }
        for (t, o) in [(MIX_BL, 0), (MIX_BR, F), (MIX_BS, 2 * F)] {
            if s.mix[t] != 0 {
                bus_b[o + i] = bus_b[o + i].wrapping_add(send(v, s.mix[t]));
            }
        }

        // Advance the source phase.
        s.frac = s.frac.wrapping_add(s.ratio);
        while s.frac >= 0x10000 {
            s.s_prev = s.s_next;
            s.s_next = s.dec.decode_one(aram);
            s.frac -= 0x10000;
            if !s.dec.active {
                ended = true;
                break;
            }
        }
        i += 1;
    }

    if ended {
        depop_voice(s, dpop, dpop_step);
        s.primed = false;
    }

    // Step the ITD shifts one sample per sub-frame towards their targets.
    s.itd_l += (s.itd_tl - s.itd_l).signum();
    s.itd_r += (s.itd_tr - s.itd_r).signum();
}

impl Ax {
    pub fn new() -> Self {
        let aux = Aux { on: false, ctx: 0, bus: [0; BUS_SAMPLES] };
        Ax {
            vpb: Box::new([[0; VPB_SIZE]; NUM_VOICES]),
            in_use: [false; NUM_VOICES],
            voices: vec![Voice::default(); NUM_VOICES].into_boxed_slice(),
            aux_a: aux.clone(),
            aux_b: aux,
            fx: Default::default(),
            nofx: false,
            acc: [[0; FRAME_SAMPLES]; 3],
            out: [0; FRAME_SAMPLES * 2],
            peak: 0,
            dpop: [0; 3],
            dpop_step: [0; 3],
            clips: Default::default(),
            clip_age: 0,
            clips_started: 0,
            clip_frames_mixed: 0,
            frames_run: 0,
            reported_first: false,
            ai_stream_vol_left: 255,
            ai_stream_vol_right: 255,
            ai_dsp_sample_rate: 0,
            underruns: 0,
            overruns: 0,
            log: None,
        }
    }

    fn log(&mut self, s: &str) {
        if let Some(l) = self.log.as_mut() {
            l(s);
        }
    }

    /// The voice's AXVPB in game byte order (0x1F8 bytes).
    pub fn vpb(&self, voice: usize) -> &[u8; VPB_SIZE] {
        &self.vpb[voice]
    }
    pub fn vpb_mut(&mut self, voice: usize) -> &mut [u8; VPB_SIZE] {
        &mut self.vpb[voice]
    }
    pub fn voice(&self, voice: usize) -> &Voice {
        &self.voices[voice]
    }
    pub fn in_use(&self, voice: usize) -> bool {
        self.in_use[voice]
    }
    /// The last sub-frame's interleaved stereo s16 output.
    pub fn output(&self) -> &[i16; FRAME_SAMPLES * 2] {
        &self.out
    }

    // ---- entry points --------------------------------------------------------------------

    /// AXAcquireVoice. Never fails while the pool has a voice (the synth reads voice->index at
    /// once). When the pool is full the lowest-priority voice (first on ties) is stolen and, if it
    /// had a callback, `on_steal` runs with it before the voice is reinitialised, as in the C.
    pub fn acquire_voice(
        &mut self,
        priority: u32,
        callback: u32,
        user_context: u32,
        on_steal: impl FnOnce(&mut Ax, Stolen),
    ) -> Option<usize> {
        let i = match self.in_use.iter().position(|u| !u) {
            Some(i) => i,
            None => {
                let mut best: Option<usize> = None;
                for i in 0..NUM_VOICES {
                    if self.in_use[i]
                        && best.is_none_or(|b| {
                            (r32(&self.vpb[i], VPB_PRIORITY) as i32) < (r32(&self.vpb[b], VPB_PRIORITY) as i32)
                        })
                    {
                        best = Some(i);
                    }
                }
                let best = best?;
                stop_voice(&mut self.voices[best], &mut self.dpop, &mut self.dpop_step);
                let cb = r32(&self.vpb[best], VPB_CALLBACK);
                if cb != 0 {
                    on_steal(self, Stolen { index: best, callback: cb });
                }
                best
            }
        };
        let v = &mut self.vpb[i];
        w32(v, VPB_PRIORITY, priority);
        w32(v, VPB_CALLBACK, callback);
        w32(v, VPB_USER_CONTEXT, user_context);
        w32(v, VPB_INDEX, i as u32);
        w16(v, VPB_ITD_FLAG, 0);
        self.in_use[i] = true;
        self.voices[i] = Voice::default();
        Some(i)
    }

    /// AXFreeVoice.
    pub fn free_voice(&mut self, voice: usize) {
        if voice >= NUM_VOICES {
            return;
        }
        stop_voice(&mut self.voices[voice], &mut self.dpop, &mut self.dpop_step);
        self.in_use[voice] = false;
        self.voices[voice].dec.state = 0;
    }

    pub fn set_voice_priority(&mut self, voice: usize, priority: u32) {
        if voice < NUM_VOICES {
            w32(&mut self.vpb[voice], VPB_PRIORITY, priority);
        }
    }

    /// AXSetVoiceState: 1 gates mixing on (if configured), anything else stops the voice.
    pub fn set_voice_state(&mut self, voice: usize, state: u16) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.dec.state = state;
        if state == 1 {
            s.dec.active = s.configured;
        } else {
            stop_voice(s, &mut self.dpop, &mut self.dpop_step);
        }
        w16(&mut self.vpb[voice], VPB_PB_STATE, state);
    }

    /// AXSetVoiceMix (an AXPBMIX block).
    pub fn set_voice_mix(&mut self, voice: usize, mix: &[u8]) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        let mut ramping = false;
        for t in 0..MIX_COUNT {
            s.mix[t] = r16(mix, PBMIX_OFF[t]) as i32;
            s.mix_delta[t] = r16(mix, PBMIX_OFF[t] + 2) as i16;
            ramping |= s.mix_delta[t] != 0;
        }
        s.mix_ramping = ramping;
    }

    /// AXSetVoiceItdOn. Sets pb.itd.flag like AX does (synth.c HSD_SynthSFXUpdateMix tests it).
    pub fn set_voice_itd_on(&mut self, voice: usize) {
        if voice >= NUM_VOICES {
            return;
        }
        self.voices[voice].itd_on = true;
        w16(&mut self.vpb[voice], VPB_ITD_FLAG, 1);
    }

    pub fn set_voice_itd_target(&mut self, voice: usize, left_shift: u16, right_shift: u16) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.itd_tl = (left_shift as usize & (ITD_MAX - 1)) as i32;
        s.itd_tr = (right_shift as usize & (ITD_MAX - 1)) as i32;
    }

    /// AXSetVoiceVe (an AXPBVE block).
    pub fn set_voice_ve(&mut self, voice: usize, ve: &[u8]) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.volume = r16(ve, 0) as i32;
        s.volume_delta = r16(ve, 2) as i16;
    }

    pub fn set_voice_ve_delta(&mut self, voice: usize, delta: i16) {
        if voice < NUM_VOICES {
            self.voices[voice].volume_delta = delta;
        }
    }

    /// AXSetVoiceAddr (an AXPBADDR block).
    pub fn set_voice_addr(&mut self, voice: usize, addr: &[u8]) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.dec.loop_flag = r16(addr, 0);
        s.dec.format = r16(addr, 2);
        s.dec.loop_addr = r32(addr, 4);
        s.dec.end_addr = r32(addr, 8);
        s.dec.cur_addr = r32(addr, 0xC);
        s.primed = false;
        s.frac = 0;
    }

    pub fn set_voice_loop(&mut self, voice: usize, lp: u16) {
        if voice < NUM_VOICES {
            self.voices[voice].dec.loop_flag = lp;
        }
    }
    pub fn set_voice_loop_addr(&mut self, voice: usize, addr: u32) {
        if voice < NUM_VOICES {
            self.voices[voice].dec.loop_addr = addr;
        }
    }
    pub fn set_voice_end_addr(&mut self, voice: usize, addr: u32) {
        if voice < NUM_VOICES {
            self.voices[voice].dec.end_addr = addr;
        }
    }

    /// AXSetVoiceCurrentAddr: a seek; restores the AXPBADPCM initial decoder state.
    pub fn set_voice_current_addr(&mut self, voice: usize, addr: u32) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.dec.cur_addr = addr;
        s.dec.pred_scale = s.dec.init_pred_scale;
        s.dec.yn1 = s.dec.init_yn1;
        s.dec.yn2 = s.dec.init_yn2;
        s.primed = false;
        s.frac = 0;
    }

    /// AXSetVoiceAdpcm (an AXPBADPCM block). Marks the voice configured.
    pub fn set_voice_adpcm(&mut self, voice: usize, adpcm: &[u8]) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        for i in 0..8 {
            s.dec.coef[i][0] = r16(adpcm, i * 4) as i16;
            s.dec.coef[i][1] = r16(adpcm, i * 4 + 2) as i16;
        }
        s.dec.pred_scale = r16(adpcm, 0x22) as u8;
        s.dec.yn1 = r16(adpcm, 0x24) as i16;
        s.dec.yn2 = r16(adpcm, 0x26) as i16;
        s.dec.init_pred_scale = s.dec.pred_scale;
        s.dec.init_yn1 = s.dec.yn1;
        s.dec.init_yn2 = s.dec.yn2;
        s.primed = false;
        s.configured = true;
    }

    /// AXSetVoiceSrc (an AXPBSRC block: ratioHi, ratioLo).
    pub fn set_voice_src(&mut self, voice: usize, src: &[u8]) {
        if voice < NUM_VOICES {
            self.voices[voice].ratio = ((r16(src, 0) as u32) << 16) | r16(src, 2) as u32;
        }
    }

    /// AXSetVoiceSrcRatio. Matches the C for ratios in 0..65536; beyond that the C conversion is
    /// undefined and this saturates.
    pub fn set_voice_src_ratio(&mut self, voice: usize, ratio: f32) {
        if voice >= NUM_VOICES {
            return;
        }
        let r = if ratio < 0.0 { 0.0 } else { ratio };
        self.voices[voice].ratio = (r * 65536.0f32) as u32;
    }

    /// AXSetVoiceAdpcmLoop (an AXPBADPCMLOOP block).
    pub fn set_voice_adpcm_loop(&mut self, voice: usize, lp: &[u8]) {
        if voice >= NUM_VOICES {
            return;
        }
        let s = &mut self.voices[voice];
        s.dec.loop_pred_scale = r16(lp, 0) as u8;
        s.dec.loop_yn1 = r16(lp, 2) as i16;
        s.dec.loop_yn2 = r16(lp, 4) as i16;
    }

    /// AXRegisterAuxACallback: `on` = a non-NULL callback; `ctx` = the AXFX struct's game address.
    pub fn register_aux_a(&mut self, on: bool, ctx: u32) {
        self.aux_a.on = on;
        self.aux_a.ctx = ctx;
    }
    pub fn register_aux_b(&mut self, on: bool, ctx: u32) {
        self.aux_b.on = on;
        self.aux_b.ctx = ctx;
    }

    // ---- AXFX ----------------------------------------------------------------------------

    fn fx_find(&self, owner: u32) -> Option<usize> {
        if owner == 0 {
            return None;
        }
        self.fx.iter().position(|s| s.owner == owner)
    }

    fn fx_claim(&mut self, owner: u32) -> Option<usize> {
        let i = self.fx_find(owner).or_else(|| self.fx.iter().position(|s| s.owner == 0))?;
        self.fx[i] = FxSlot { owner, kind: FxKind::None };
        Some(i)
    }

    /// AXFXReverbStdInit: `owner` = the struct's game address, `p` = its bytes (>= 0x154).
    /// Returns 1 when ready, 0 when it stays unregistered (axdriver.c tests `== 1`).
    pub fn fx_reverb_std_init(&mut self, owner: u32, p: &[u8]) -> i32 {
        if owner == 0 {
            return 0;
        }
        let col = rf32(p, REVSTD_COLORATION);
        let mix = rf32(p, REVSTD_MIX);
        let time = rf32(p, REVSTD_TIME);
        let damp = rf32(p, REVSTD_DAMPING);
        let pre = rf32(p, REVSTD_PREDELAY);
        let Some(i) = self.fx_claim(owner) else {
            self.log("gw: AX: no AXFX slot free for reverb-std");
            return 0;
        };
        match ReverbStd::new(col, time, mix, damp, pre) {
            Some(rv) => {
                self.fx[i].kind = FxKind::Reverb(rv);
                self.log(&format!(
                    "gw: AX: AXFX reverb-std ready (col={col:.3} time={time:.3} mix={mix:.3} damp={damp:.3} pre={pre:.4})"
                ));
                1
            }
            None => {
                self.fx[i] = FxSlot::default();
                self.log(&format!(
                    "gw: AX: AXFX reverb-std init failed (col={col:.3} time={time:.3} mix={mix:.3} damp={damp:.3} pre={pre:.4})"
                ));
                0
            }
        }
    }

    pub fn fx_reverb_std_shutdown(&mut self, owner: u32) -> i32 {
        if let Some(i) = self.fx_find(owner) {
            if matches!(self.fx[i].kind, FxKind::Reverb(_)) {
                self.fx[i] = FxSlot::default();
            }
        }
        1
    }

    /// AXFXDelayInit: `p` = the AXFX_DELAY bytes (>= 0x60).
    pub fn fx_delay_init(&mut self, owner: u32, p: &[u8]) -> i32 {
        if owner == 0 {
            return 0;
        }
        let rd = |o: usize| [r32(p, o), r32(p, o + 4), r32(p, o + 8)];
        let (ms, fb, out) = (rd(DELAY_DELAY), rd(DELAY_FEEDBACK), rd(DELAY_OUTPUT));
        let Some(i) = self.fx_claim(owner) else {
            self.log("gw: AX: no AXFX slot free for delay");
            return 0;
        };
        self.fx[i].kind = FxKind::Delay(Delay::new(ms, fb, out));
        self.log(&format!("gw: AX: AXFX delay ready ({ms:?} ms, fb {fb:?}%, out {out:?}%)"));
        1
    }

    pub fn fx_delay_shutdown(&mut self, owner: u32) -> i32 {
        if let Some(i) = self.fx_find(owner) {
            if matches!(self.fx[i].kind, FxKind::Delay(_)) {
                self.fx[i] = FxSlot::default();
            }
        }
        1
    }

    /// AXFXReverbHiInit / AXFXChorusInit: not implemented (Melee does not use them); 0 = aux off.
    pub fn fx_reverb_hi_init(&mut self, _owner: u32) -> i32 {
        self.log("gw: AX: AXFX reverb-hi is not implemented (Melee does not use it); aux left off");
        0
    }
    pub fn fx_chorus_init(&mut self, _owner: u32) -> i32 {
        self.log("gw: AX: AXFX chorus is not implemented (Melee does not use it); aux left off");
        0
    }

    /// Run one aux bus's effect in place. The callback can only be one of the AXFX entry points,
    /// so the bus goes straight to the processor owning `ctx`; an unknown context is silenced
    /// (passing it through would fold the raw send back in as a second dry copy).
    fn run_aux(fx: &mut [FxSlot; FX_SLOTS], aux: &mut Aux, nofx: bool) {
        if !aux.on || nofx {
            return;
        }
        let slot = if aux.ctx == 0 { None } else { fx.iter_mut().find(|s| s.owner == aux.ctx) };
        match slot.map(|s| &mut s.kind) {
            Some(FxKind::Reverb(rv)) => rv.run(&mut aux.bus),
            Some(FxKind::Delay(d)) => d.run(&mut aux.bus),
            _ => aux.bus.fill(0),
        }
    }

    // ---- native clips --------------------------------------------------------------------

    /// gw_Audio_PlayOwnedClip: 32 kHz s16 PCM (`channels` 1 or 2, interleaved), gain 0..127,
    /// summed into the main mix after the voices and aux buses. Up to 8 at once; past that the
    /// oldest is replaced. Returns 1 if started.
    pub fn play_owned_clip(&mut self, pcm: Arc<[i16]>, frames: i32, channels: i32, volume: i32, owner: u32) -> i32 {
        if frames < 1 || !(1..=2).contains(&channels) {
            return 0;
        }
        let mut slot = 0;
        for i in 0..CLIPS {
            if self.clips[i].pcm.is_none() {
                slot = i;
                break;
            }
            if self.clips[i].age < self.clips[slot].age {
                slot = i;
            }
        }
        let frames = (frames as usize).min(pcm.len() / channels as usize) as u32;
        self.clip_age = self.clip_age.wrapping_add(1);
        self.clips[slot] = Clip {
            pcm: Some(pcm),
            frames,
            pos: 0,
            channels: channels as usize,
            gain: volume.clamp(0, 127),
            age: self.clip_age,
            owner,
        };
        self.clips_started = self.clips_started.wrapping_add(1);
        1
    }

    pub fn play_clip(&mut self, pcm: Arc<[i16]>, frames: i32, channels: i32, volume: i32) -> i32 {
        self.play_owned_clip(pcm, frames, channels, volume, 0)
    }

    /// Stop every instance of `pcm` that `owner` started.
    pub fn stop_owned_clips(&mut self, pcm: &Arc<[i16]>, owner: u32) -> i32 {
        let mut n = 0;
        for c in self.clips.iter_mut() {
            if c.pcm.as_ref().is_some_and(|p| Arc::ptr_eq(p, pcm)) && c.owner == owner {
                c.pcm = None;
                n += 1;
            }
        }
        n
    }

    /// Stop every playing instance of `pcm`.
    pub fn stop_clips(&mut self, pcm: &Arc<[i16]>) -> i32 {
        let mut n = 0;
        for c in self.clips.iter_mut() {
            if c.pcm.as_ref().is_some_and(|p| Arc::ptr_eq(p, pcm)) {
                c.pcm = None;
                n += 1;
            }
        }
        n
    }

    /// gw_Audio_ClipStat: 0 = clips started, else 32 kHz frames mixed.
    pub fn clip_stat(&self, which: i32) -> u32 {
        if which == 0 {
            self.clips_started
        } else {
            self.clip_frames_mixed
        }
    }

    fn mix_clips(&mut self) {
        for c in self.clips.iter_mut() {
            let Some(pcm) = c.pcm.as_ref() else { continue };
            let mut i = 0;
            while i < FRAME_SAMPLES && c.pos < c.frames {
                let f = c.pos as usize * c.channels;
                let l = pcm[f] as i32;
                let r = if c.channels == 2 { pcm[f + 1] as i32 } else { l };
                self.acc[0][i] = self.acc[0][i].wrapping_add(l * c.gain / 127);
                self.acc[1][i] = self.acc[1][i].wrapping_add(r * c.gain / 127);
                self.clip_frames_mixed = self.clip_frames_mixed.wrapping_add(1);
                i += 1;
                c.pos += 1;
            }
            if c.pos >= c.frames {
                c.pcm = None;
            }
        }
    }

    // ---- the mixer -----------------------------------------------------------------------

    /// gw_ax_run_frame minus the synth callback (the caller runs the game's AX callback first) and
    /// minus the output push: mixes one 5 ms sub-frame from `aram` (the ARAM image the DSP
    /// addresses), writes the game-visible VPB fields back, and returns the 160 stereo samples.
    pub fn run_frame(&mut self, aram: &[u8]) -> &[i16; FRAME_SAMPLES * 2] {
        let mut active = 0;
        self.acc = [[0; FRAME_SAMPLES]; 3];
        self.aux_a.bus.fill(0);
        self.aux_b.bus.fill(0);

        for i in 0..NUM_VOICES {
            let s = &mut self.voices[i];
            if self.in_use[i] && s.dec.active && s.configured {
                mix_voice(s, aram, &mut self.acc, &mut self.aux_a.bus, &mut self.aux_b.bus, &mut self.dpop, &mut self.dpop_step);
                if s.dec.active {
                    active += 1;
                }
            }
        }

        // Ramp out any voice that stopped.
        for i in 0..FRAME_SAMPLES {
            for c in 0..3 {
                if self.dpop[c] == 0 {
                    continue;
                }
                self.acc[c][i] = self.acc[c][i].wrapping_add(self.dpop[c]);
                self.dpop[c] = self.dpop[c].wrapping_sub(self.dpop_step[c]);
                if (self.dpop_step[c] > 0) == (self.dpop[c] < 0) {
                    self.dpop[c] = 0; // crossed zero
                }
            }
        }

        // Effects in place, then fold the buses back in (surround dropped, as the DSP in stereo).
        Self::run_aux(&mut self.fx, &mut self.aux_a, self.nofx);
        Self::run_aux(&mut self.fx, &mut self.aux_b, self.nofx);
        for aux in [&self.aux_a, &self.aux_b] {
            if aux.on {
                for i in 0..FRAME_SAMPLES {
                    self.acc[0][i] = self.acc[0][i].wrapping_add(aux.bus[i]);
                    self.acc[1][i] = self.acc[1][i].wrapping_add(aux.bus[FRAME_SAMPLES + i]);
                }
            }
        }

        self.mix_clips();

        for i in 0..FRAME_SAMPLES {
            self.peak = self.peak.max(self.acc[0][i].wrapping_abs()).max(self.acc[1][i].wrapping_abs());
        }

        // Game-visible writeback (big-endian), read by synth.c:824, 1213, 1289.
        for i in 0..NUM_VOICES {
            if self.in_use[i] {
                let s = &self.voices[i];
                let v = &mut self.vpb[i];
                w16(v, VPB_PB_STATE, s.dec.state);
                w16(v, VPB_VE_VOL, s.volume as u16);
                w32(v, VPB_CURADDR, s.dec.cur_addr);
            }
        }

        if active > 0 && !self.reported_first {
            self.reported_first = true;
            let m = format!("gw: AX: first audible frame, {active} active voice(s), peak={}", self.peak);
            self.log(&m);
        }
        self.frames_run = self.frames_run.wrapping_add(1);
        if self.frames_run % 2400 == 0 {
            let m = format!(
                "gw: AX: {} frames, {active} active voice(s), peak={}, underruns={} overruns={}",
                self.frames_run, self.peak, self.underruns, self.overruns
            );
            self.log(&m);
            self.peak = 0;
        }

        for i in 0..FRAME_SAMPLES {
            self.out[i * 2] = self.acc[0][i].clamp(-32768, 32767) as i16;
            self.out[i * 2 + 1] = self.acc[1][i].clamp(-32768, 32767) as i16;
        }
        &self.out
    }
}

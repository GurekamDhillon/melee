//! AXFX aux-bus processors: shim_ax.c's native reimplementations of the SDK's reverb_std.c
//! (`HandleReverb`) and delay.c (`AXFXDelayCallback`). Melee configures reverb-std on AUX A and
//! delay on AUX B (lbaudio_ax.c); reverb-hi and chorus are unimplemented there and here.
//!
//! Floats: the shim is built `clang --target=i686-pc-windows-msvc -O2` with the default
//! `-ffp-contract=on`; i686 has no FMA, so each `a*b+c` lowers to an f32 multiply then an f32 add
//! (SSE2). Plain Rust f32 arithmetic is exactly that, so no site here is fused.

use crate::FRAME_SAMPLES;

/// `(int32_t)f` as x86 `cvttss2si` computes it: truncation, and 0x80000000 for NaN or out of
/// range (the C cast is undefined there; this is what the shipping build's instruction does).
#[inline]
pub(crate) fn cvtt_i32(f: f32) -> i32 {
    if f.is_nan() || f >= 2147483648.0 || f < -2147483648.0 {
        i32::MIN
    } else {
        f as i32
    }
}

#[derive(Clone, Debug, Default)]
struct DelayLine {
    buf: Vec<f32>,
    in_point: usize,
    out_point: usize,
    last_output: f32,
}

impl DelayLine {
    /// DLcreate zeroes both cursors, then ReverbSTDCreate re-runs DLsetdelay(lag) against
    /// in_point == 0, which lands out_point at (len - lag).
    fn new(max_length: usize, lag: usize) -> Self {
        DelayLine { buf: vec![0.0; max_length], in_point: 0, out_point: max_length - lag, last_output: 0.0 }
    }
    #[inline]
    fn push(&mut self, v: f32) {
        self.buf[self.in_point] = v;
        self.in_point += 1;
        if self.in_point >= self.buf.len() {
            self.in_point = 0;
        }
    }
    #[inline]
    fn pop(&mut self) -> f32 {
        let v = self.buf[self.out_point];
        self.out_point += 1;
        if self.out_point >= self.buf.len() {
            self.out_point = 0;
        }
        v
    }
}

/// shim_ax.c `gw_fx_revstd`.
#[derive(Clone, Debug, Default)]
pub struct ReverbStd {
    ap: [DelayLine; 6],
    c: [DelayLine; 6],
    all_pass_coeff: f32,
    comb_coef: [f32; 6],
    lp_lastout: [f32; 3],
    level: f32,
    damping: f32,
    pre_delay_time: usize,
    pre_delay_line: [Vec<f32>; 3],
    pre_delay_pos: [usize; 3],
}

impl ReverbStd {
    /// reverb_std.c ReverbSTDCreate; `None` where it returns FALSE (parameters out of range).
    pub fn new(coloration: f32, time: f32, mix: f32, damping: f32, predelay: f32) -> Option<Self> {
        const LENS: [i32; 4] = [0x6FD, 0x7CF, 0x1B1, 0x95];
        // Written as the C's comparisons (a NaN parameter passes them, as it does there).
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if coloration < 0.0
            || coloration > 1.0
            || time < 0.01
            || time > 10.0
            || mix < 0.0
            || mix > 1.0
            || damping < 0.0
            || damping > 1.0
            || predelay < 0.0
            || predelay > 0.1
        {
            return None;
        }
        let mut rv = ReverbStd::default();
        for k in 0..3 {
            for i in 0..2 {
                rv.c[i + k * 2] = DelayLine::new(LENS[i] as usize + 2, LENS[i] as usize);
                rv.comb_coef[i + k * 2] = 10.0f32.powf((LENS[i] * -3) as f32 / (32000.0f32 * time));
                rv.ap[i + k * 2] = DelayLine::new(LENS[i + 2] as usize + 2, LENS[i + 2] as usize);
            }
        }
        rv.all_pass_coeff = coloration;
        rv.level = mix;
        let d = if damping < 0.05 { 0.05 } else { damping };
        rv.damping = 1.0 - (0.05 + 0.8 * d);
        if predelay != 0.0 {
            let t = cvtt_i32(32000.0f32 * predelay);
            // below two elements the original's cursor cannot advance
            rv.pre_delay_time = if t < 2 { 0 } else { t as usize };
        }
        if rv.pre_delay_time != 0 {
            for i in 0..3 {
                rv.pre_delay_line[i] = vec![0.0; rv.pre_delay_time];
            }
        }
        Some(rv)
    }

    /// reverb_std.c HandleReverb over the bus's three 160-sample blocks, in place.
    pub fn run(&mut self, bus: &mut [i32; FRAME_SAMPLES * 3]) {
        let apc = self.all_pass_coeff;
        let damp = self.damping;
        let wet = self.level * 0.6;
        let dry = 0.6 - wet;
        let pdt = self.pre_delay_time;
        for k in 0..3 {
            let (c_lo, c_hi) = self.c.split_at_mut(k * 2 + 1);
            let c0 = &mut c_lo[k * 2];
            let c1 = &mut c_hi[0];
            let (a_lo, a_hi) = self.ap.split_at_mut(k * 2 + 1);
            let a0 = &mut a_lo[k * 2];
            let a1 = &mut a_hi[0];
            let cc0 = self.comb_coef[k * 2];
            let cc1 = self.comb_coef[k * 2 + 1];
            let pdl = &mut self.pre_delay_line[k];
            let mut pdp = self.pre_delay_pos[k];
            let mut lp = self.lp_lastout[k];
            let sp = &mut bus[k * FRAME_SAMPLES..(k + 1) * FRAME_SAMPLES];
            for s in sp.iter_mut() {
                let input = *s as f32;
                let x = if pdt != 0 {
                    // The original wraps one element early: usable pre-delay is pdt - 1.
                    let x = pdl[pdp];
                    pdl[pdp] = input;
                    pdp += 1;
                    if pdp >= pdt - 1 {
                        pdp = 0;
                    }
                    x
                } else {
                    input
                };
                c0.push(cc0 * c0.last_output + x);
                c1.push(cc1 * c1.last_output + x);
                let o0 = c0.pop();
                let o1 = c1.pop();
                c0.last_output = o0;
                c1.last_output = o1;
                let mut y = o0 + o1;

                let t = apc * a0.last_output + y;
                a0.push(t);
                y = a0.last_output - apc * t;
                a0.last_output = a0.pop();

                y *= 0.3;
                y = damp * lp + y;
                lp = y;

                let t = apc * a1.last_output + y;
                a1.push(t);
                y = a1.last_output - apc * t;
                a1.last_output = a1.pop();

                *s = cvtt_i32(wet * y + dry * input);
            }
            self.lp_lastout[k] = lp;
            self.pre_delay_pos[k] = pdp;
        }
    }
}

/// shim_ax.c `gw_fx_delay`.
#[derive(Clone, Debug, Default)]
pub struct Delay {
    buf: [Vec<i32>; 3],
    /// in 160-sample blocks
    size: [u32; 3],
    pos: [u32; 3],
    feedback: [u32; 3],
    output: [u32; 3],
}

impl Delay {
    /// delay.c AXFXDelaySettings: per channel delay (ms), feedback (%), output (%).
    pub fn new(ms: [u32; 3], fb: [u32; 3], out: [u32; 3]) -> Self {
        let mut d = Delay::default();
        for i in 0..3 {
            let m = ms[i].max(5);
            d.size[i] = ((m - 5).wrapping_shl(5).wrapping_add(0x9F)) / 160;
            if d.size[i] == 0 {
                d.size[i] = 1;
            }
            d.feedback[i] = fb[i].wrapping_shl(7) / 100;
            d.output[i] = out[i].wrapping_shl(7) / 100;
            d.buf[i] = vec![0; d.size[i] as usize * FRAME_SAMPLES];
        }
        d
    }

    /// delay.c AXFXDelayCallback, in place.
    pub fn run(&mut self, bus: &mut [i32; FRAME_SAMPLES * 3]) {
        for k in 0..3 {
            let base = self.pos[k] as usize * FRAME_SAMPLES;
            let line = &mut self.buf[k][base..base + FRAME_SAMPLES];
            let sp = &mut bus[k * FRAME_SAMPLES..(k + 1) * FRAME_SAMPLES];
            let fb = self.feedback[k] as i32 as i64;
            let og = self.output[k] as i32 as i64;
            for n in 0..FRAME_SAMPLES {
                let v = line[n] as i64;
                line[n] = sp[n].wrapping_add(((v * fb) >> 7) as i32);
                sp[n] = ((v * og) >> 7) as i32;
            }
            self.pos[k] = (self.pos[k] + 1) % self.size[k];
        }
    }
}

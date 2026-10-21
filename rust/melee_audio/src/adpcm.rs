//! Per-voice sample decoder: shim_ax.c `gw_ax_decode_one`, `gw_ax_at_end`, `gw_ax_hit_end`.
//!
//! Nintendo DSP-ADPCM: 8-byte frames, a predictor/scale header byte + 7 bytes (14 nibbles) of
//! 4-bit deltas. ARAM is addressed in nibbles (a DSP address `d` is byte `d/2`, high nibble when
//! `d` is even), so the header is read exactly when the nibble address is 16-aligned. Each nibble
//! decodes as `(n * 2^(scale+11) + c0*yn1 + c1*yn2 + 0x400) >> 11`, clamped to s16.

/// AXPBADDR.format values.
pub const FORMAT_ADPCM: u16 = 0;
pub const FORMAT_PCM16: u16 = 0xA;
pub const FORMAT_PCM8: u16 = 0x19;

/// The decoder half of shim_ax.c's `gw_ax_vstate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decoder {
    pub coef: [[i16; 2]; 8],
    /// Current frame header: predictor << 4 | scale.
    pub pred_scale: u8,
    pub yn1: i16,
    pub yn2: i16,
    /// Initial state from AXPBADPCM (restored by AXSetVoiceCurrentAddr).
    pub init_pred_scale: u8,
    pub init_yn1: i16,
    pub init_yn2: i16,
    /// DSP (nibble) address of the next data nibble.
    pub cur_addr: u32,
    /// Last valid nibble, inclusive.
    pub end_addr: u32,
    pub loop_addr: u32,
    /// 0 = stop at end, else loop.
    pub loop_flag: u16,
    pub format: u16,
    pub loop_pred_scale: u8,
    pub loop_yn1: i16,
    pub loop_yn2: i16,
    /// Voice still playing (shared with the mixer: decoding clears it on end / bad address).
    pub active: bool,
    /// Mirrored to pb.state; cleared with `active`.
    pub state: u16,
}

#[inline]
fn clamp_s16(v: i64) -> i16 {
    v.clamp(-32768, 32767) as i16
}

impl Decoder {
    /// The end test: an (almost) exact match, not `>=`. The .hps stream ring moves endAddress only
    /// after the voice arrives at a block, so for one AX frame the end address is a block behind
    /// the cursor and `>=` would re-loop every sample. 16 nibbles of window so an end address on
    /// a skipped frame-header nibble still terminates; unsigned, so an end behind never matches.
    #[inline]
    fn at_end(&self, next_addr: u32) -> bool {
        next_addr.wrapping_sub(self.end_addr) < 16
    }

    fn hit_end(&mut self) {
        if self.loop_flag != 0 {
            self.cur_addr = self.loop_addr;
            self.pred_scale = self.loop_pred_scale;
            self.yn1 = self.loop_yn1;
            self.yn2 = self.loop_yn2;
        } else {
            self.stop();
        }
    }

    #[inline]
    fn stop(&mut self) {
        self.active = false;
        self.state = 0;
    }

    /// Decode one source sample at `cur_addr` and advance. `aram` is the whole ARAM image.
    ///
    /// Divergence from the C on malformed data only: the C indexes `coef[predictor]` with the raw
    /// header nibble, so a predictor of 8..15 reads past the table (undefined behaviour); here the
    /// predictor is masked to 0..7. Every well-formed DSP-ADPCM header has predictor 0..7.
    pub fn decode_one(&mut self, aram: &[u8]) -> i16 {
        let size = aram.len() as u64;
        match self.format {
            FORMAT_ADPCM => {
                if self.cur_addr & 0xF == 0 {
                    let off = (self.cur_addr >> 1) as u64;
                    if off >= size {
                        self.stop();
                        return 0;
                    }
                    self.pred_scale = aram[off as usize];
                    self.cur_addr = self.cur_addr.wrapping_add(2);
                }
                let off = (self.cur_addr >> 1) as u64;
                if off >= size {
                    self.stop();
                    return 0;
                }
                let b = aram[off as usize];
                let nibble = if self.cur_addr & 1 != 0 { b & 0xF } else { b >> 4 } as i64;
                let predictor = ((self.pred_scale >> 4) & 7) as usize;
                let scale = (self.pred_scale & 0xF) as u32;
                let [c0, c1] = self.coef[predictor];
                let sn = if nibble >= 8 { nibble - 16 } else { nibble };
                let scaled = sn * (1i64 << (scale + 11));
                let pred = c0 as i64 * self.yn1 as i64 + c1 as i64 * self.yn2 as i64;
                let out = clamp_s16((scaled + pred + 0x400) >> 11);
                self.yn2 = self.yn1;
                self.yn1 = out;
                if self.at_end(self.cur_addr) {
                    self.hit_end();
                } else {
                    self.cur_addr = self.cur_addr.wrapping_add(1);
                }
                out
            }
            FORMAT_PCM16 => {
                // One sample per 2 bytes, i.e. 4 nibbles.
                let off = (self.cur_addr >> 1) as u64;
                if off + 1 >= size {
                    self.stop();
                    return 0;
                }
                let out = i16::from_be_bytes([aram[off as usize], aram[off as usize + 1]]);
                if self.at_end(self.cur_addr.wrapping_add(3)) {
                    self.hit_end();
                } else {
                    self.cur_addr = self.cur_addr.wrapping_add(4);
                }
                out
            }
            FORMAT_PCM8 => {
                let off = (self.cur_addr >> 1) as u64;
                if off >= size {
                    self.stop();
                    return 0;
                }
                let out = ((aram[off as usize] as i8 as i16) << 8) as i16;
                if self.at_end(self.cur_addr.wrapping_add(1)) {
                    self.hit_end();
                } else {
                    self.cur_addr = self.cur_addr.wrapping_add(2);
                }
                out
            }
            _ => {
                // Unknown format: silence the voice rather than walk ARAM blindly.
                self.stop();
                0
            }
        }
    }
}

/// Decode a whole DSP-ADPCM buffer from its start (byte 0 = first frame header), the way a voice
/// set up at nibble address 2 with `end_addr = 2*len - 1` and no loop plays it. For tools/tests.
pub fn decode_buffer(coef: [[i16; 2]; 8], data: &[u8], yn1: i16, yn2: i16, out: &mut Vec<i16>) {
    let mut d = Decoder {
        coef,
        yn1,
        yn2,
        cur_addr: 0,
        end_addr: (data.len() as u32 * 2).saturating_sub(1),
        format: FORMAT_ADPCM,
        active: true,
        state: 1,
        ..Default::default()
    };
    while d.active {
        out.push(d.decode_one(data));
        if d.cur_addr as usize >= data.len() * 2 {
            break;
        }
    }
}

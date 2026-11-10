//! The game's sound stack above AX, as one owned value: `synth.c` (HSD_Synth*, in
//! [`crate::synth`]) and `axdriver.c` (HSD_Audio* / AXDriver*: the 96 scripted "virtual voices"
//! that run `smash2.sem` command streams and start synth sounds), plus the `lbaudio_ax.c` entry
//! points the ported game code calls (`lbAudioAx_800237A8`, ...).
//!
//! The handle `HSD_AudioSFXStartParam` returns (`(vidhigh << 7) | slot`) is game state
//! (`Fighter.x2160`, item and stage voice ids). It comes from this file's virtual-voice pool,
//! so it matches the C as long as the sequence of starts and 5 ms AX callbacks between them is
//! the same; the pool frees a voice only in the callback (`AXDriverCallback`), as in the C.
//!
//! Every C struct mirrored here carries its GameCube offsets (`/* 0xNN */`); `tests` checks them
//! against the generated `melee_types` layouts.

use melee_runtime::gekko::sqrtf;
use melee_runtime::lb_00ce::powf;

use crate::disc::StreamSource;
use crate::sem::Sem;
use crate::Ax;

/// hsd_SynthSFXNodes: one per AX voice.
pub const NODES: usize = 0x40;
/// AXDriver_804C45A0: the virtual voice pool.
pub const SMS: usize = 0x60;
/// The game address the synth registers as every voice's AX callback (`dropcallback`); any
/// non-zero value: AX only tests it for NULL and hands the stolen voice back.
pub const DROP_CALLBACK: u32 = 0x8038_91D0;
/// ARAM size (shim_ar.c `gw_aram_size` is 16 MB as on hardware).
pub const ARAM_SIZE: usize = 0x100_0000;

pub const SMSTATE_MASK: u32 = 0xC000_0000;
pub const SMSTATE_ACTIVE: u32 = 0x4000_0000;
pub const SMSTATE_SLEEP: u32 = 0x8000_0000;

/// `struct HSD_SynthSFXNode` (synth.static.h), GC size 0x50. The `user_vol[2]` union fields are
/// split by how synth.c uses them: `user_vol[0].x8` only as a float (the current secondary
/// volume, faded by fader 1), `user_vol[1].x8` only as a u8 (the pan).
#[derive(Clone, Copy, Debug, Default)]
pub struct Node {
    /* 0x00 */ pub x0: i32,
    /* 0x04 */ pub sfx_id: i32,
    /* 0x09 */ pub flags: u8,
    /* 0x0A */ pub voice_count: u8,
    /* 0x0B */ pub xb: u8,
    /* 0x0C */ pub voice: [Option<usize>; 2],
    /* 0x14 */ pub x14: f32,
    /* 0x18 */ pub x18: [f32; 2],
    /* 0x20 */ pub x20: Option<usize>,
    /* 0x24 */ pub x24: u16,
    /* 0x26 */ pub volume_update_pending: u8,
    /* 0x27 */ pub x27: u8,
    /// Current primary volume (faded by fader 0).
    /* 0x28 */ pub unk28: f32,
    /// user_vol[k].volume: fader k's target.
    /* 0x2C, 0x38 */ pub vol_target: [f32; 2],
    /// user_vol[k].x4: fader k's remaining steps.
    /* 0x30, 0x3C */ pub fade: [i32; 2],
    /// user_vol[0].x8_float.
    /* 0x34 */ pub vol2: f32,
    /// user_vol[1].x8.
    /* 0x40 */ pub pan: u8,
    /* 0x44 */ pub x44: f32,
    /* 0x48 */ pub x48: f32,
    /* 0x4C */ pub x4c: f32,
}

/// voicelist_1784: a channel's master volume and its fade.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChanVol {
    /* 0x0 */ pub cur: f32,
    /* 0x4 */ pub target: f32,
    /* 0x8 */ pub count: i32,
}

/// One sound of a loaded SSM (the C's `struct foo` hash entry: next 0x0, id 0x4, voice count
/// 0x8, rate 0xC, then per voice 0x40 bytes: AXPBADDR +0x00, AXPBADPCM +0x10, AXPBADPCMLOOP
/// +0x38, kept as big-endian game bytes with the ARAM addresses relocated).
#[derive(Clone, Debug)]
pub struct SfxEntry {
    /* 0x4 */ pub id: i32,
    /* 0xC */ pub rate: u32,
    /* 0x10 */ pub voices: Vec<[u8; 0x40]>,
}

/// One loaded SSM file (`struct SfxLoadStreamNode`): entrynum 0x4, first id 0x8, count 0xC,
/// ARAM address 0x10, ARAM size 0x14.
#[derive(Clone, Copy, Debug)]
pub struct Group {
    /* 0x04 */ pub entrynum: i32,
    /* 0x08 */ pub base: i32,
    /* 0x0C */ pub count: i32,
    /* 0x10 */ pub ar: i32,
    /* 0x14 */ pub ar_size: i32,
}

/// `HSD_SM` (axdriver.h), GC size 0x34: a scripted virtual voice. `prev`/`next`/`cmd_stream`
/// are indices (pool slot, word of [`Sem::words`]) instead of pointers.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sm {
    /* 0x00 */ pub prev: Option<usize>,
    /* 0x04 */ pub next: Option<usize>,
    /* 0x08 */ pub flags: u32,
    /* 0x0C */ pub unk: i32,
    /* 0x10 */ pub vid: i32,
    /* 0x14 */ pub fid: u16,
    /* 0x16 */ pub x16: u16,
    /* 0x18 */ pub track: u8,
    /* 0x19 */ pub pri: u8,
    /* 0x1A */ pub x1a: u8,
    /* 0x1B */ pub volume: u8,
    /* 0x1C */ pub x1c: u8,
    /* 0x1D */ pub pan: u8,
    /* 0x1E */ pub x1e: u8,
    /* 0x20 */ pub x20: i16,
    /* 0x22 */ pub fadetime: i16,
    /* 0x24 */ pub x24: [u8; 2],
    /* 0x26 */ pub x26: u8,
    /* 0x27 */ pub x27: u8,
    /* 0x29 */ pub itdflag: u8,
    /* 0x2A */ pub x2a: u16,
    /* 0x2C */ pub cmd: usize,
    /* 0x30 */ pub x30: i32,
}

/// What [`Sound`] reads game files from (the C's DVD calls).
pub trait AudioFs: Send {
    /// DVDConvertPathToEntrynum + a whole-file read.
    fn read(&mut self, path: &str) -> Option<(i32, Vec<u8>)>;
    /// DVDConvertPathToEntrynum + an opened file for ranged (stream) reads.
    fn open_stream(&mut self, path: &str) -> Option<(i32, Box<dyn StreamSource>)>;
}

/// The whole sound state: AX, ARAM, synth.c and axdriver.c globals.
pub struct Sound {
    pub ax: Ax,
    pub aram: Vec<u8>,
    pub(crate) ar_top: u32,
    pub fs: Option<Box<dyn AudioFs>>,

    // ---- synth.c ----
    /// hsd_SynthSFXNodes.
    pub nodes: [Node; NODES],
    /// voicelist + HSD_Synth_804D7720: voices to free at the next callback.
    pub(crate) free_list: [Option<usize>; NODES],
    pub(crate) free_count: usize,
    /// hsd_SynthSFXDataHash: 32 buckets, newest first.
    pub(crate) data_hash: Vec<Vec<SfxEntry>>,
    /// HSD_Synth_804C2AE0[0..32]: each bank's loaded files.
    pub(crate) groups: Vec<Vec<Group>>,
    /// HSD_Synth_804C2AE0[bank + 0x20]: the bank's deflag offset.
    pub(crate) bank_deflag: [i32; 32],
    /// hsd_SynthSFXBank: each bank's load cursor (ARAM bytes).
    pub(crate) bank_cur: [i32; 32],
    /// hsd_SynthSFXBankHead.
    pub(crate) bank_head: [i32; 33],
    /// hsd_SynthSFXBankNum.
    pub(crate) bank_num: usize,
    /// hsd_SynthSFXBankAREnd.
    pub(crate) bank_ar_end: u32,
    /// HSD_Synth_804D774C: the envelope's pending list.
    pub(crate) vol_list: Option<usize>,
    /// HSD_Synth_804D7750: the handle sequence.
    pub(crate) handle_seq: i32,
    /// HSD_Synth_804D7754.
    pub sound_mode: u32,
    /// HSD_Synth_804D7758: channels fading.
    pub(crate) chan_fading: u32,
    /// HSD_Synth_804D775C: the callback counter.
    pub(crate) clock: i32,
    /// voicelist_1784.
    pub chan: [ChanVol; 16],
    /// voicelist_1844: the last handle per group.
    pub(crate) group_last: Vec<i32>,
    /// HSD_Synth_804D6030.
    pub stream_vol: f32,
    /// HSD_Synth_804D777C.
    pub(crate) stream_master: i32,
    /// HSD_Synth_804D7760: the stream's node handle.
    pub stream_node: i32,
    /// HSD_Synth_804D7764.
    pub stream_entrynum: i32,
    pub(crate) stream_src: Option<Box<dyn StreamSource>>,
    /// HSD_Synth_804D7768 / 776C / 7770 / 7774: the load, play, loop and voice-position slots.
    pub(crate) stream_load_slot: u32,
    pub(crate) stream_play_slot: u32,
    pub(crate) stream_loop_slot: u32,
    pub(crate) stream_pos: u32,
    /// HSD_Synth_804D7778.
    pub(crate) stream_busy: u8,
    /// The HPS start's DevCom completions (header, first block) wait for `stream_header_done` /
    /// `stream_data_done`, as the C's deferred completions do (set by the trace replay).
    pub defer_stream: bool,
    /// Block refills complete on recorded HHDR / HDONE ops (replay of a trace that has them).
    pub defer_refill: bool,
    pub refill_src: u32,
    /// The refill's data read lands in ARAM chunk by chunk (DCHUNK ops).
    pub refill_chunked: bool,
    pub refill_active: bool,
    pub refill_off: usize,
    pub(crate) stream_hdr_pending: Option<Vec<u8>>,
    pub(crate) stream_data_pending: bool,
    /// HSD_Synth_804D7780: the 3 x 64 KB stream blocks in ARAM.
    pub(crate) stream_base: u32,
    /// HSD_Synth_804D7784: 0x500 bytes of silence (a nibble address).
    pub(crate) silence: u32,
    /// pstHakoHeader[3], big-endian as read from the .hps.
    pub(crate) hako: [[u8; 0x20]; 3],
    /// AXRegisterCallback(HSD_SynthCallback) has run.
    pub ax_callback_registered: bool,
    /// Call trace in the format of the C build's `MELEE_AUDIO_TRACE` (`<sub-frame> OP args`).
    pub trace: Option<Vec<String>>,
    pub trace_sub: u32,
    /// The sound objects are stepped by [`crate::sim::sound_obj_procs`] (the engine's proc phase), not by the per-frame update.
    pub objs_external: bool,
    pub(crate) trace_in_cb: bool,

    // ---- axdriver.c ----
    /// AXDriver_804C45A0.
    pub sm: [Sm; SMS],
    /// AXDriver_804D7790 (free) / 804D7794 (active).
    pub(crate) free_head: Option<usize>,
    pub(crate) active_head: Option<usize>,
    /// vidhigh.
    pub(crate) vidhigh: i32,
    /// AXDriver_804D778C: the driver's clock.
    pub(crate) drv_clock: i32,
    pub sem: Option<Sem>,
    /// AXDriver_804C5920: synth handle & 0x3F -> pool slot.
    pub(crate) by_vid: [Option<usize>; NODES],
    /// AXDriver_804D77C8 / 77D0.
    pub(crate) playing: i32,
    pub(crate) live: i32,
    /// AXDriver_804D77CC: paused channels.
    pub(crate) paused_channels: u32,
    /// AXDriver_804C5A20: aux send per channel.
    pub(crate) aux_send: [[u8; 16]; 2],
    /// AXDriver_804D603C: aux types (bits 8-11, 12-15) and the aux-locked bits 0/1.
    pub(crate) aux_flags: u32,
    /// AXDriver_804D6038: the stream's synth handle.
    pub stream_vid: i32,
    /// lbaudio_ax.c's file statics (banks, deferred sfx, heartbeat loops, current BGM).
    pub lb: crate::lbaudio::LbState,
    /// AXDriver_804D77E0 / 77E4 / 77E8.
    pub(crate) stream_pitch_dirty: i32,
    pub(crate) stream_pitch: i32,
    pub(crate) stream_started_at: i32,
    /// The driver callbacks are installed (HSD_AudioInitMultiPStream ran).
    pub(crate) drv_installed: bool,
}

impl Default for Sound {
    fn default() -> Self {
        Self::new()
    }
}

impl Sound {
    pub fn new() -> Self {
        Sound {
            ax: Ax::new(),
            aram: vec![0; ARAM_SIZE],
            ar_top: 0x4000,
            fs: None,
            nodes: [Node::default(); NODES],
            free_list: [None; NODES],
            free_count: 0,
            data_hash: vec![Vec::new(); 32],
            groups: vec![Vec::new(); 32],
            bank_deflag: [0; 32],
            bank_cur: [0; 32],
            bank_head: [0; 33],
            bank_num: 0,
            bank_ar_end: 0,
            vol_list: None,
            handle_seq: 0,
            sound_mode: 0,
            chan_fading: 0,
            clock: 0,
            chan: [ChanVol::default(); 16],
            group_last: vec![0; 0x100],
            stream_vol: 1.0,
            stream_master: 0,
            stream_node: 0,
            stream_entrynum: 0,
            stream_src: None,
            stream_load_slot: 0,
            stream_play_slot: 0,
            stream_loop_slot: 0,
            stream_pos: 0,
            stream_busy: 0,
            defer_stream: false,
            defer_refill: false,
            refill_src: 0,
            refill_chunked: false,
            refill_active: false,
            refill_off: 0,
            stream_hdr_pending: None,
            stream_data_pending: false,
            stream_base: 0,
            silence: 0,
            hako: [[0; 0x20]; 3],
            ax_callback_registered: false,
            trace: None,
            trace_sub: 0,
            objs_external: false,
            trace_in_cb: false,
            sm: [Sm::default(); SMS],
            free_head: None,
            active_head: None,
            vidhigh: 0,
            drv_clock: 0,
            sem: None,
            by_vid: [None; NODES],
            playing: 0,
            live: 0,
            paused_channels: 0,
            aux_send: [[0; 16]; 2],
            aux_flags: 2,
            stream_vid: -1,
            stream_pitch_dirty: 0,
            stream_pitch: 0,
            stream_started_at: 0,
            drv_installed: false,
            lb: crate::lbaudio::LbState::default(),
        }
    }

    /// One 5 ms DSP sub-frame: the game's AX callback (HSD_SynthCallback), then the mix.
    /// Record one game-facing call (not those made inside the synth callback).
    pub(crate) fn tr(&mut self, op: &str, a: [i32; 5]) {
        if self.trace_in_cb {
            return;
        }
        let sub = self.trace_sub;
        if let Some(t) = self.trace.as_mut() {
            t.push(format!("{sub} {op} {} {} {} {} {}", a[0], a[1], a[2], a[3], a[4]));
        }
    }

    pub fn sub_frame(&mut self) -> [i16; crate::FRAME_SAMPLES * 2] {
        if self.ax_callback_registered {
            self.trace_in_cb = true;
            self.synth_callback();
            self.trace_in_cb = false;
        }
        self.trace_sub += 1;
        let mut out = *self.ax.run_frame(&self.aram);
        crate::brawl::mix_into(&mut out); // adds nothing unless Brawl audio plays
        out
    }

    /// How many SSM sounds are loaded (all banks).
    pub fn loaded_sounds(&self) -> usize {
        self.data_hash.iter().map(|b| b.len()).sum()
    }

    /// Whether any synth node is sounding (diagnostics).
    pub fn active_nodes(&self) -> usize {
        self.nodes.iter().filter(|n| n.x0 > 0).count()
    }

    // ---- boot ------------------------------------------------------------------------------

    /// The audio half of `lbAudioAx_80028690`'s first run and its init (lbaudio_ax.c:2380):
    /// HSD_AudioInitMultiPStream, the aux reverb (std, time 1.88) and delay, the three SFX banks,
    /// `/audio/us/smash2.sem`, and `main.ssm` into bank 0. `bank_sizes` are the three banks'
    /// ARAM sizes (the C sums them from `offsets_arr_803BC4E4`).
    pub fn boot(&mut self, fs: Box<dyn AudioFs>, bank_sizes: [i32; 3]) -> Result<(), String> {
        self.boot_engine(fs, bank_sizes)?;
        self.load_ssm("/audio/us/main.ssm", 0)?;
        Ok(())
    }

    /// `boot` without the first bank load (HSD_SynthSFXLoad of main.ssm is a game call).
    pub fn boot_engine(&mut self, fs: Box<dyn AudioFs>, bank_sizes: [i32; 3]) -> Result<(), String> {
        self.fs = Some(fs);
        let total: i32 = bank_sizes.iter().sum();
        self.lb.bank_size = bank_sizes;
        self.lb.total_size = total;
        self.init_multi_pstream(total);
        let rv = crate::sound::default_reverb_std(1.88);
        self.setup_aux(0, 2, &rv);
        let dl = crate::sound::default_delay();
        self.setup_aux(1, 4, &dl);
        for s in bank_sizes {
            self.sfx_allocate_bank(s);
        }
        // HSD_SynthInit (synth.c): sound mode = OSGetSoundMode(), which the PC shim (shim_os.c) answers 0 (mono); nothing in a match sets it.
        self.load_sem("/audio/us/smash2.sem")?;
        Ok(())
    }

    /// AXDriver_8038DCFC + AXDriver_8038DA70.
    pub fn load_sem(&mut self, path: &str) -> Result<(), String> {
        self.sem = None;
        let fs = self.fs.as_mut().ok_or("no audio filesystem")?;
        let (_, f) = fs.read(path).ok_or_else(|| format!("can not open {path}"))?;
        self.sem = Some(Sem::parse(&f)?);
        Ok(())
    }

    /// HSD_SynthSFXLoad(path, bank) + HSD_SynthSFXWaitForLoadCompletion.
    pub fn load_ssm(&mut self, path: &str, bank: usize) -> Result<i32, String> {
        let fs = self.fs.as_mut().ok_or("no audio filesystem")?;
        let (entry, f) = fs.read(path).ok_or_else(|| format!("{path}: not on the disc"))?;
        self.sfx_load(&f, bank, entry)
    }

    // ---- axdriver.c ------------------------------------------------------------------------

    fn sm_unlink(&mut self, v: usize, active: bool) {
        let (p, n) = (self.sm[v].prev, self.sm[v].next);
        self.sm[v].next = None;
        self.sm[v].prev = None;
        if let Some(p) = p {
            self.sm[p].next = n;
        }
        if let Some(n) = n {
            self.sm[n].prev = p;
        }
        let head = if active { &mut self.active_head } else { &mut self.free_head };
        if *head == Some(v) {
            *head = n;
        }
    }

    fn sm_link(&mut self, v: usize, active: bool) {
        let head = if active { self.active_head } else { self.free_head };
        self.sm[v].prev = None;
        if let Some(h) = head {
            self.sm[h].prev = Some(v);
        }
        self.sm[v].next = head;
        if active {
            self.active_head = Some(v);
        } else {
            self.free_head = Some(v);
        }
    }

    /// HSD_AudioInitMultiPStream.
    pub fn init_multi_pstream(&mut self, aram_size: i32) {
        for i in 0..SMS {
            self.sm[i].flags &= !SMSTATE_MASK;
            self.sm_link(i, false);
        }
        self.synth_init(aram_size);
        self.drv_installed = true;
        self.setup_aux(0, 0, &[]);
        self.setup_aux(1, 0, &[]);
    }

    /// AXDriverSetupAux(channel, type, param bytes): 0 off, 2 reverb std, 4 delay (1 reverb-hi
    /// and 3 chorus register nothing, as their AX shims do not run them).
    pub fn setup_aux(&mut self, channel: usize, ty: u32, param: &[u8]) -> i32 {
        assert!(channel < 2 && ty <= 4);
        let shift = if channel == 0 { 8 } else { 12 };
        let old = (<u32 as melee_core::ppc::Ppc>::pshr(self.aux_flags, (shift) as i64)) & 0xF;
        self.aux_flags &= !(<u32 as melee_core::ppc::Ppc>::pshl(old, (shift) as i64));
        self.aux_flags |= <u32 as melee_core::ppc::Ppc>::pshl(ty, (shift) as i64);
        // The AXFX structs' game addresses: AXDriver_804C5E00 / 804C61E0 [channel].
        let std_owner = 0x804C_5E00 + channel as u32 * 0x154;
        let delay_owner = 0x804C_61E0 + channel as u32 * 0x60;
        if channel == 0 {
            self.ax.register_aux_a(false, 0);
        } else {
            self.ax.register_aux_b(false, 0);
        }
        match old {
            2 => {
                self.ax.fx_reverb_std_shutdown(std_owner);
            }
            4 => {
                self.ax.fx_delay_shutdown(delay_owner);
            }
            _ => {}
        }
        let (ok, owner) = match ty {
            0 => (1, 0),
            2 => (self.ax.fx_reverb_std_init(std_owner, param), std_owner),
            4 => (self.ax.fx_delay_init(delay_owner, param), delay_owner),
            1 => (self.ax.fx_reverb_hi_init(0x804C_5A40), 0),
            _ => (self.ax.fx_chorus_init(0x804C_60A8), 0),
        };
        let on = ok == 1 && owner != 0;
        if channel == 0 {
            self.ax.register_aux_a(on, if on { owner } else { 0 });
        } else {
            self.ax.register_aux_b(on, if on { owner } else { 0 });
        }
        ok
    }

    fn sm_mix(&self, v: usize) -> (f32, f32, f32) {
        let s = &self.sm[v];
        let left_vol = (s.x26 as i32 * s.x24[0] as i32) as f32 / 65535.0f32;
        let right_vol = (s.x27 as i32 * s.x24[1] as i32) as f32 / 65535.0f32;
        let left_sqrt = sqrtf(left_vol);
        let left_inv_sqrt = sqrtf(1.0f32 - left_vol);
        let right_sqrt = sqrtf(right_vol);
        let right_inv_sqrt = sqrtf(1.0f32 - right_vol);
        (left_inv_sqrt * (left_inv_sqrt * right_inv_sqrt), left_sqrt, right_sqrt * left_inv_sqrt)
    }

    /// `powf(2.0F, c / 1200.0F)` with the game's powf (lb_00CE.c), as the C links it.
    fn cents(c: i32) -> f32 {
        powf(2.0f32, c as f32 / 1200.0f32)
    }

    /// AXDriverKeyOff.
    fn drv_key_off(&mut self, v: usize) -> bool {
        let idx = self.sm[v].vid;
        if idx != -1 {
            self.by_vid[(idx & 0x3F) as usize] = None;
            self.sm[v].vid = -1;
            self.sfx_key_off(idx);
            self.playing -= 1;
        }
        self.sm[v].flags &= !0x2000_0000;
        self.sm[v].flags &= !SMSTATE_MASK;
        true
    }

    /// The pool slot a handle names while it is live.
    fn sm_of(&self, vid: i32) -> Option<usize> {
        let idx = (vid & 0x7F) as usize;
        if vid < 0 || idx >= SMS {
            return None;
        }
        let v = &self.sm[idx];
        (v.unk == vid && v.flags & SMSTATE_MASK != 0).then_some(idx)
    }

    /// HSD_AudioSFXKeyOff.
    pub fn audio_sfx_key_off(&mut self, vid: i32) -> bool {
        self.tr("KO", [vid, 0, 0, 0, 0]);
        match self.sm_of(vid) {
            Some(v) => self.drv_key_off(v),
            None => false,
        }
    }

    fn for_active(&mut self, mut f: impl FnMut(&mut Sound, usize)) {
        let mut v = self.active_head;
        while let Some(i) = v {
            let next = self.sm[i].next;
            f(self, i);
            v = next;
        }
    }

    /// HSD_AudioSFXKeyOffAll.
    pub fn audio_sfx_key_off_all(&mut self) {
        self.tr("KOALL", [0; 5]);
        self.for_active(|s, i| {
            if s.sm[i].flags & SMSTATE_MASK != 0 {
                s.drv_key_off(i);
            }
        });
    }

    /// HSD_AudioSFXKeyOffTrack.
    pub fn audio_sfx_key_off_track(&mut self, track: i32) {
        self.tr("KOTRK", [track, 0, 0, 0, 0]);
        self.for_active(|s, i| {
            if s.sm[i].flags & SMSTATE_MASK != 0 && s.sm[i].track as i32 == track {
                s.drv_key_off(i);
            }
        });
    }

    /// AXDriverExec: apply the flags the script set since the last wait.
    fn drv_exec(&mut self, v: usize) {
        for i in 0..=9 {
            let flag = <u32 as melee_core::ppc::Ppc>::pshl(1u32, (i) as i64);
            if self.sm[v].flags & flag != 0 {
                match flag {
                    0x1 => {
                        let (main, a, b) = self.sm_mix(v);
                        let s = self.sm[v];
                        let pitch1 = Self::cents(s.x20 as i32);
                        let pitch2 = Self::cents(s.fadetime as i32);
                        let pan = if s.flags & 0x20000 != 0 { s.pan } else { s.x1c };
                        let vid = self.sfx_play_with_group(
                            s.fid as i32, s.x1a, s.volume, pan, s.pri as i32, s.itdflag as i32, s.track as i32, pitch1,
                            pitch2, main, a, b,
                        );
                        self.sm[v].vid = vid;
                        if vid != -1 {
                            self.by_vid[(vid & 0x3F) as usize] = Some(v);
                            self.sm[v].flags &= 0xFFF4_FF99;
                            if self.sm[v].flags & 0x40000 != 0 {
                                self.sm[v].flags &= !0x40000;
                            }
                            self.playing += 1;
                        } else {
                            debug_assert_eq!(self.sm[v].flags & SMSTATE_MASK, SMSTATE_ACTIVE);
                            self.sm[v].flags &= !SMSTATE_MASK;
                            return;
                        }
                    }
                    0x2 => {
                        let (id, p) = (self.sm[v].vid, self.sm[v].pri as i32);
                        self.sfx_set_priority(id, p);
                    }
                    0x4 => {
                        let (id, x) = (self.sm[v].vid, self.sm[v].x1a);
                        self.sfx_set_volume_fade(id, x, 0);
                    }
                    0x8 => {
                        let (id, x) = (self.sm[v].vid, self.sm[v].x1c);
                        self.sfx_set_pan(id, x);
                    }
                    0x20 => {
                        let (id, c) = (self.sm[v].vid, self.sm[v].x20 as i32);
                        self.sfx_set_pitch_ratio(id, 0, Self::cents(c));
                    }
                    0x40 => {
                        let (id, c) = (self.sm[v].vid, self.sm[v].fadetime as i32);
                        self.sfx_set_pitch_ratio(id, 1, Self::cents(c));
                    }
                    0x80 => {
                        let (main, a, b) = self.sm_mix(v);
                        let id = self.sm[v].vid;
                        self.sfx_set_mix(id, main, a, b);
                    }
                    0x100 => {
                        self.sm[v].flags = (self.sm[v].flags & !SMSTATE_MASK) | SMSTATE_SLEEP;
                        return;
                    }
                    0x200 => {
                        self.drv_key_off(v);
                        return;
                    }
                    _ => {}
                }
            }
            self.sm[v].flags &= !flag;
        }
    }

    /// parseWait.
    fn parse_wait(ty: u32, val: u32) -> u32 {
        match ty {
            0 => val & 0xFF_FFFF,
            6..=11 | 16..=19 => (val >> 8) & 0xFFFF,
            12 | 13 => (val >> 16) & 0xFF,
            _ => 0,
        }
    }

    /// AXDriverInterp: run the voice's script until its next wait.
    fn drv_interp(&mut self, v: usize) {
        let clamp = |lo: i32, x: i32, hi: i32| x.min(hi).max(lo);
        while self.sm[v].x30 == self.drv_clock {
            let w = match self.sem.as_ref().and_then(|s| s.words.get(self.sm[v].cmd)) {
                Some(&w) => w,
                None => {
                    // A script ran off the file: stop the voice (the C would read garbage).
                    self.sm[v].flags |= 0x200;
                    self.drv_exec(v);
                    return;
                }
            };
            let ty = w >> 24;
            let size = Self::parse_wait(ty, w) as i32;
            if size != 0 {
                self.drv_exec(v);
            }
            self.sm[v].x30 = self.sm[v].x30.wrapping_add(size);
            let lock = self.aux_flags;
            let s = &mut self.sm[v];
            match ty {
                2 => {
                    s.x2a = w as u16;
                    if s.x2a == 0 {
                        s.flags |= 0x100000;
                    }
                }
                3 => {
                    if s.flags & 0x100000 != 0 || s.x2a != 0 {
                        s.cmd = s.cmd.wrapping_sub((w & 0xFF_FFFF) as usize);
                        s.x2a = s.x2a.wrapping_sub(1);
                    }
                }
                1 => {
                    s.flags |= 1;
                    s.fid = w as u16;
                }
                4 => {
                    s.flags |= 2;
                    s.pri = w as u8;
                }
                5 => {
                    s.flags |= 2;
                    s.pri = clamp(5, s.pri as i32 + w as u8 as i8 as i32, 0x1C) as u8;
                }
                6 => {
                    s.flags |= 4;
                    s.x1a = w as u8;
                }
                7 => {
                    s.flags |= 4;
                    s.x1a = clamp(0, s.x1a as i32 + w as u8 as i8 as i32, 0xFF) as u8;
                }
                8 => {
                    s.flags |= 8;
                    s.x1c = w as u8;
                }
                9 => {
                    s.flags |= 8;
                    s.x1c = clamp(0, s.x1c as i32 + w as u8 as i8 as i32, 0xFF) as u8;
                }
                10 => {
                    s.flags |= 0x10;
                    s.x1e = w as u8;
                }
                11 => {
                    s.flags |= 0x10;
                    s.x1e = clamp(0, s.x1e as i32 + w as u8 as i8 as i32, 0xFF) as u8;
                }
                12 => {
                    s.flags |= 0x20;
                    s.x20 = w as u16 as i16;
                }
                13 => {
                    s.flags |= 0x20;
                    s.x20 = clamp(-0x2A30, s.x20 as i32 + w as u16 as i16 as i32, 0x960) as i16;
                }
                16 => {
                    if lock & 1 == 0 {
                        s.flags |= 0x80;
                        s.x24[0] = w as u8;
                    }
                }
                20 => {
                    if lock & 1 == 0 {
                        s.x26 = w as u8;
                    }
                }
                21 => {
                    if (lock >> 1) & 1 == 0 {
                        s.x27 = w as u8;
                    }
                }
                17 => {
                    if lock & 1 == 0 {
                        s.flags |= 0x80;
                        s.x24[0] = clamp(0, s.x24[0] as i32 + w as u8 as i8 as i32, 0xFF) as u8;
                    }
                }
                18 => {
                    if (lock >> 1) & 1 == 0 {
                        s.flags |= 0x80;
                        s.x24[1] = w as u8;
                    }
                }
                19 => {
                    if (lock >> 1) & 1 == 0 {
                        s.flags |= 0x80;
                        s.x24[1] = clamp(0, s.x24[1] as i32 + w as u8 as i8 as i32, 0xFF) as u8;
                    }
                }
                15 => {
                    s.flags |= 0x200;
                    self.drv_exec(v);
                    return;
                }
                14 => {
                    s.flags |= 0x100;
                    self.drv_exec(v);
                    return;
                }
                _ => {}
            }
            self.sm[v].cmd = self.sm[v].cmd.wrapping_add(1);
        }
    }

    /// AXDriverCallback: the synth's master clock (once per 5 ms).
    pub(crate) fn drv_callback(&mut self) {
        if self.stream_pitch_dirty != 0 {
            self.stream_pitch_dirty = 0;
            if self.stream_vid != -1 {
                let x = Self::cents(self.stream_pitch.clamp(-0x2A30, 0x960));
                let id = self.stream_vid;
                self.sfx_set_pitch_ratio(id, 1, x);
            }
        }
        self.drv_clock = self.drv_clock.wrapping_add(1);
        let mut v = self.active_head;
        while let Some(i) = v {
            let next = self.sm[i].next;
            if self.sm[i].x30 == -1 {
                self.sm[i].x30 = self.drv_clock;
            } else if self.sm[i].flags & 0x2000_0000 != 0 {
                self.sm[i].x30 = self.sm[i].x30.wrapping_add(1);
            }
            match self.sm[i].flags & SMSTATE_MASK {
                SMSTATE_ACTIVE => {
                    if self.sm[i].x30 == self.drv_clock {
                        self.drv_interp(i);
                    }
                }
                0 => {
                    self.sm_unlink(i, true);
                    self.sm_link(i, false);
                    self.live -= 1;
                }
                SMSTATE_SLEEP => {
                    if self.sm[i].flags & 0x40 != 0 {
                        let (id, c) = (self.sm[i].vid, self.sm[i].fadetime as i32);
                        self.sfx_set_pitch_ratio(id, 1, Self::cents(c));
                        self.sm[i].flags &= 0xFFFF_FFBF;
                    }
                }
                _ => panic!("AXDriverCallback: bad voice state"),
            }
            v = next;
        }
    }

    /// AXDriverKillCallback: the synth ended a node a virtual voice owned.
    pub(crate) fn drv_kill_callback(&mut self, vid: i32) {
        if !self.drv_installed {
            return;
        }
        debug_assert!(vid > 0);
        let idx = (vid & 0x3F) as usize;
        let Some(v) = self.by_vid[idx] else { return };
        if self.sm[v].vid != vid {
            return;
        }
        self.by_vid[idx] = None;
        self.sm[v].flags &= !0x2000_0000;
        self.sm[v].flags &= !SMSTATE_MASK;
        self.playing -= 1;
    }

    /// AXDriverPauseCallback.
    pub(crate) fn drv_pause_callback(&mut self, vid: i32) {
        if !self.drv_installed {
            return;
        }
        if let Some(v) = self.by_vid[(vid & 0x3F) as usize] {
            if self.sm[v].vid == vid {
                self.sm[v].flags |= 0x2000_0000;
            }
        }
    }

    /// HSD_AudioSFXStartParam: start script `sound_id` (bank = id / 10000 of smash2.sem) on a
    /// virtual voice. The handle, or -1.
    pub fn audio_sfx_start_param(&mut self, sound_id: i32, volume: u8, pan: u8, track: i32, channel: i32) -> i32 {
        self.tr("START", [sound_id, volume as i32, pan as i32, track, channel]);
        let h = self.audio_sfx_start_param_impl(sound_id, volume, pan, track, channel);
        self.tr("STARTRET", [h, 0, 0, 0, 0]);
        h
    }

    fn audio_sfx_start_param_impl(&mut self, sound_id: i32, volume: u8, pan: u8, track: i32, channel: i32) -> i32 {
        let Some(sem) = self.sem.as_ref() else { return -1 };
        let bank = sound_id / 10000;
        let mem = sound_id % 10000;
        if bank < 0 || bank as usize >= sem.bank_first.len() {
            return -1;
        }
        let sample = mem.wrapping_add(sem.bank_first[bank as usize] as i32);
        if sample < 0 || sample as usize >= sem.scripts.len() {
            return -1;
        }
        if (bank as usize) < sem.bank_first.len() - 1 && sem.bank_first[bank as usize + 1] as i32 <= sample {
            return -1;
        }
        if !(0..=0xFF).contains(&track) || !(0..0x10).contains(&channel) {
            return -1;
        }
        if self.paused_channels & (<u32 as melee_core::ppc::Ppc>::pshl(1, (channel) as i64)) != 0 {
            return -1;
        }
        let cmd = sem.scripts[sample as usize];
        // AXDriverAssignVVoice.
        let Some(v) = self.free_head else { return -1 };
        self.sm_unlink(v, false);
        let ch = channel as usize;
        let s = &mut self.sm[v];
        s.vid = -1;
        s.x30 = -1;
        s.flags = 0;
        s.x16 = sound_id as u16;
        s.cmd = cmd;
        s.x1a = 0xFF;
        s.volume = volume;
        s.x1c = 0x80;
        s.pan = pan;
        s.x20 = 0;
        s.fadetime = 0;
        s.x24 = [self.aux_send[0][ch], self.aux_send[1][ch]];
        s.x26 = 0xFF;
        s.x27 = 0xFF;
        s.pri = 5;
        s.track = track as u8;
        s.itdflag = channel as u8;
        s.flags |= 0x30000;
        assert!(self.vidhigh as u32 & 0xFE00_0000 == 0, "vidhigh exceeds the max value");
        s.unk = (self.vidhigh << 7) | v as i32;
        self.vidhigh += 1;
        s.flags = (s.flags & !SMSTATE_MASK) | SMSTATE_ACTIVE;
        self.sm_link(v, true);
        self.live += 1;
        self.sm[v].unk
    }

    /// HSD_AudioSFXSetPan.
    pub fn audio_sfx_set_pan(&mut self, vid: i32, pan: u8) -> bool {
        self.tr("PAN", [vid, pan as i32, 0, 0, 0]);
        let Some(v) = self.sm_of(vid) else { return false };
        if self.sm[v].vid != -1 {
            let id = self.sm[v].vid;
            self.sfx_set_pan(id, pan);
        } else {
            self.sm[v].pan = pan;
            self.sm[v].flags |= 0x20000;
        }
        true
    }

    /// HSD_AudioSFXSetVolumeEx.
    pub fn audio_sfx_set_volume_ex(&mut self, vid: i32, volume: u8) -> bool {
        self.tr("VOL", [vid, volume as i32, 0, 0, 0]);
        let Some(v) = self.sm_of(vid) else { return false };
        if self.sm[v].vid != -1 {
            let id = self.sm[v].vid;
            self.sfx_set_volume_fade(id, volume, 1);
        } else {
            self.sm[v].volume = volume;
            self.sm[v].flags |= 0x10000;
        }
        true
    }

    /// HSD_AudioSFXSetPitchFid.
    pub fn audio_sfx_set_pitch_fid(&mut self, vid: i32, pitch: i16) -> bool {
        self.tr("PITCH", [vid, pitch as i32, 0, 0, 0]);
        let Some(v) = self.sm_of(vid) else { return false };
        self.sm[v].fadetime = (pitch as i32).clamp(-0x2A30, 0x960) as i16;
        self.sm[v].flags |= 0x40;
        true
    }

    /// HSD_AudioSFXSetMix.
    pub fn audio_sfx_set_mix(&mut self, vid: i32, aux_bus: i32, send: u8) -> bool {
        self.tr("MIX", [vid, aux_bus, send as i32, 0, 0]);
        let idx = (vid & 0x7F) as usize;
        if vid < 0 || idx >= SMS || !(0..=1).contains(&aux_bus) {
            return false;
        }
        let lock = (<u32 as melee_core::ppc::Ppc>::pshr(self.aux_flags, (aux_bus) as i64)) & 1;
        if lock != 1 {
            return false;
        }
        let Some(v) = self.sm_of(vid) else { return false };
        self.sm[v].x24[aux_bus as usize] = send;
        if self.sm[v].vid != -1 {
            let (main, a, b) = self.sm_mix(v);
            let id = self.sm[v].vid;
            self.sfx_set_mix(id, main, a, b);
        } else {
            self.sm[v].flags |= 0x80000;
        }
        true
    }

    /// HSD_AudioSFXSetMixGroup.
    pub fn audio_sfx_set_mix_group(&mut self, channel: i32, aux_bus: i32, send: i8) -> bool {
        self.tr("MIXGRP", [channel, aux_bus, send as i32, 0, 0]);
        if !(0..0x10).contains(&channel) || !(0..=1).contains(&aux_bus) {
            return false;
        }
        self.for_active(|s, i| {
            if s.sm[i].flags & SMSTATE_MASK != 0 && s.sm[i].itdflag as i32 == channel {
                let unk = s.sm[i].unk;
                s.audio_sfx_set_mix(unk, aux_bus, send as u8);
            }
        });
        self.aux_send[aux_bus as usize][channel as usize] = send as u8;
        true
    }

    /// HSD_AudioSFXCheck: whether the handle's sound is still playing.
    pub fn audio_sfx_check(&self, vid: i32) -> bool {
        match self.sm_of(vid) {
            Some(v) => self.sfx_check(self.sm[v].vid) != -1,
            None => false,
        }
    }

    /// AXDriver_8038E5D4 / 8038E5DC.
    pub fn playing_count(&self) -> i32 {
        self.playing
    }
    pub fn live_count(&self) -> i32 {
        self.live
    }

    fn pause_ch(&mut self, channel: i32, pause: bool) -> bool {
        if !(0..0x10).contains(&channel) {
            return false;
        }
        self.for_active(|s, i| {
            if s.sm[i].flags & SMSTATE_MASK != 0 && s.sm[i].itdflag as i32 == channel {
                let id = s.sm[i].vid;
                if pause {
                    if id != -1 {
                        s.sfx_pause(id);
                    } else {
                        s.sm[i].flags |= 0x2000_0000;
                    }
                } else {
                    if id != -1 {
                        s.sfx_resume(id);
                    }
                    s.sm[i].flags &= 0xDFFF_FFFF;
                }
            }
        });
        if pause {
            self.paused_channels |= <u32 as melee_core::ppc::Ppc>::pshl(1, (channel) as i64);
        } else {
            self.paused_channels &= !(<u32 as melee_core::ppc::Ppc>::pshl(1, (channel) as i64));
        }
        true
    }

    /// HSD_AudioPStreamPauseCh / ResumeCh.
    pub fn pstream_pause_ch(&mut self, channel: i32) -> bool {
        self.tr("PAUSE", [channel, 0, 0, 0, 0]);
        self.pause_ch(channel, true)
    }
    pub fn pstream_resume_ch(&mut self, channel: i32) -> bool {
        self.tr("RESUME", [channel, 0, 0, 0, 0]);
        self.pause_ch(channel, false)
    }

    /// HSD_AudioPStreamStartChParam(path, volume, track): start an .hps on the disc.
    pub fn pstream_start_ch_param(&mut self, path: &str, volume: u8, track: i32) -> bool {
        if !self.trace_in_cb {
            let sub = self.trace_sub;
            if let Some(t) = self.trace.as_mut() {
                t.push(format!("{sub} PSTART {path} {} {}", volume as i32, track));
            }
        }
        if crate::brawl::music_on() {
            // Brawl music holds the music channel (presentation only; never set in replays or the gate)
            self.stream_stop();
            return true;
        }
        let opened = self.fs.as_mut().and_then(|fs| fs.open_stream(path));
        if self.stream_vid != -1 {
            let id = self.stream_vid;
            self.sfx_key_off(id);
        }
        let (entry, src) = match opened {
            Some(x) => x,
            None => (-1, Box::new(Vec::<u8>::new()) as Box<dyn StreamSource>),
        };
        self.stream_vid = self.pstream_start(src, entry, 0xFF, volume, track);
        self.stream_started_at = self.drv_clock;
        true
    }

    /// AXDriverStop / Pause / Resume / Check (the stream).
    pub fn stream_stop(&mut self) -> bool {
        if self.stream_vid == -1 {
            return false;
        }
        let id = self.stream_vid;
        self.sfx_key_off(id);
        self.stream_vid = -1;
        true
    }
    pub fn stream_pause(&mut self) -> bool {
        if self.stream_vid == -1 {
            return false;
        }
        let id = self.stream_vid;
        self.sfx_pause(id);
        true
    }
    pub fn stream_resume(&mut self) -> bool {
        if self.stream_vid == -1 {
            return false;
        }
        let id = self.stream_vid;
        self.sfx_resume(id);
        true
    }
    pub fn stream_check(&self) -> bool {
        self.sfx_check(self.stream_vid) != -1
    }

    // ---- lbaudio_ax.c ----------------------------------------------------------------------

    /// fn_80023750(id, vol, pan, track, channel): the game's 0..127 volume and pan doubled and
    /// clamped to a byte, then HSD_AudioSFXStartParam.
    pub fn lb_start(&mut self, id: i32, vol: i32, pan: i32, track: i32, channel: i32) -> i32 {
        let vol = vol.wrapping_mul(2).clamp(0, 0xFF);
        let pan = pan.wrapping_mul(2).clamp(0, 0xFF);
        self.audio_sfx_start_param(id, vol as u8, pan as u8, track, channel)
    }

    /// lbAudioAx_800237A8(id, vol, pan). (The m-ex id range of TARGET_PC goes through the same
    /// call; LBAX_N is 55 on a vanilla disc, so the m-ex branch is never taken there.)
    pub fn lb_audio_ax_800237a8(&mut self, id: i32, vol: i32, pan: i32) -> i32 {
        if id >= 0x83D61 {
            self.lb_start(0x83D60, 0, 0x40, 0, 7)
        } else {
            self.lb_start(id, vol, pan, 0, 7)
        }
    }

    /// lbAudioAx_80023870(id, vol, pan, track).
    pub fn lb_audio_ax_80023870(&mut self, id: i32, vol: i32, pan: i32, track: i32) -> i32 {
        if track == 0 {
            return self.lb_audio_ax_800237a8(id, vol, pan);
        }
        if id == 0x83D61 {
            self.audio_sfx_key_off_track(track);
            return -1;
        }
        self.lb_start(id, vol, pan, track, 7)
    }

    /// lbAudioAx_80024B94(voice, pitch): pitch clamped to +-0x4B0 cents.
    pub fn lb_audio_ax_80024b94(&mut self, voice: i32, pitch: i32) -> i32 {
        self.audio_sfx_set_pitch_fid(voice, pitch.clamp(-0x4B0, 0x4B0) as i16) as i32
    }

    /// fn_80023ED4(path, vol, track) as lbAudioAx_80023F28 calls it for a BGM: vol doubled and
    /// clamped, track clamped to 0..8, HSD_AudioPStreamStartChParam.
    pub fn lb_play_bgm_file(&mut self, file: &str, vol: i32, track: i32) -> bool {
        let vol = vol.wrapping_mul(2).clamp(0, 0xFF);
        let track = track.clamp(0, 8);
        self.pstream_start_ch_param(&format!("/audio/{file}"), vol as u8, track)
    }
}

/// HSD_AudioSFXGetDefaultAuxParam(REVERB_STD) with lbaudio's `time`, as AXFX_REVERBSTD bytes.
pub fn default_reverb_std(time: f32) -> Vec<u8> {
    use crate::layout::*;
    let mut b = vec![0u8; 0x154];
    b[REVSTD_TEMPDISABLE] = 0;
    for (o, v) in [(REVSTD_TIME, time), (REVSTD_PREDELAY, 0.002), (REVSTD_DAMPING, 0.64), (REVSTD_COLORATION, 0.5), (REVSTD_MIX, 1.0)] {
        b[o..o + 4].copy_from_slice(&f32::to_bits(v).to_be_bytes());
    }
    b
}

/// HSD_AudioSFXGetDefaultAuxParam(DELAY), as AXFX_DELAY bytes.
pub fn default_delay() -> Vec<u8> {
    use crate::layout::*;
    let mut b = vec![0u8; 0x60];
    for (base, vals) in [(DELAY_DELAY, [0x104u32, 0x136, 6]), (DELAY_FEEDBACK, [0x18, 0x18, 0]), (DELAY_OUTPUT, [0x23, 0x23, 0])] {
        for (i, v) in vals.iter().enumerate() {
            b[base + i * 4..base + i * 4 + 4].copy_from_slice(&v.to_be_bytes());
        }
    }
    b
}

/// An [`AudioFs`] over the user's disc image.
pub struct DiscFs {
    pub iso: String,
    pub disc: crate::disc::Disc,
}

impl DiscFs {
    pub fn open(iso: &str) -> Result<Self, String> {
        Ok(DiscFs { iso: iso.to_string(), disc: crate::disc::Disc::open(iso)? })
    }
}

impl AudioFs for DiscFs {
    fn read(&mut self, path: &str) -> Option<(i32, Vec<u8>)> {
        let e = self.disc.find(path)?;
        Some((e.entrynum, self.disc.read(path).ok()?))
    }
    fn open_stream(&mut self, path: &str) -> Option<(i32, Box<dyn StreamSource>)> {
        let e = self.disc.find(path)?;
        let disc = crate::disc::Disc::open(&self.iso).ok()?;
        Some((e.entrynum, Box::new(crate::disc::DiscStream { disc, entry: e })))
    }
}

#[cfg(test)]
mod tests {
    use melee_types::melee_core::gc::GcLayout;
    use melee_types::{HSD_SynthSFXNode, HSD_SM};

    fn off<T: GcLayout>(f: &str) -> u32 {
        T::gc_field(f).unwrap_or_else(|| panic!("{} has no {f}", T::GC_NAME)).offset
    }

    /// The offsets written beside each field above are the GameCube ones.
    #[test]
    fn mirrored_structs_match_gc_layouts() {
        assert_eq!(HSD_SM::GC_SIZE, 0x34);
        for (f, o) in [
            ("prev", 0x0), ("next", 0x4), ("flags", 0x8), ("unk", 0xC), ("vID", 0x10), ("fid", 0x14), ("x16", 0x16),
            ("track", 0x18), ("pri", 0x19), ("x1A", 0x1A), ("volume", 0x1B), ("x1C", 0x1C), ("pan", 0x1D),
            ("x1E", 0x1E), ("x20", 0x20), ("fadetime", 0x22), ("x24", 0x24), ("x26", 0x26), ("x27", 0x27),
            ("itdflag", 0x29), ("x2A", 0x2A), ("cmd_stream", 0x2C), ("x30", 0x30),
        ] {
            assert_eq!(off::<HSD_SM>(f), o, "HSD_SM.{f}");
        }
        assert_eq!(HSD_SynthSFXNode::GC_SIZE, 0x50);
        for (f, o) in [
            ("x0", 0x0), ("sfx_id", 0x4), ("flags", 0x9), ("voice_count", 0xA), ("xB", 0xB), ("voice", 0xC),
            ("x14", 0x14), ("x18", 0x18), ("x20", 0x20), ("x24", 0x24), ("volume_update_pending", 0x26),
            ("x27", 0x27), ("unk28", 0x28), ("user_vol", 0x2C), ("x44", 0x44), ("x48", 0x48), ("x4C", 0x4C),
        ] {
            assert_eq!(off::<HSD_SynthSFXNode>(f), o, "HSD_SynthSFXNode.{f}");
        }
    }
}

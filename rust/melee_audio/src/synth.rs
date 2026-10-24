//! `src/sysdolphin/baselib/synth.c` (HSD_Synth*): SSM sample banks in ARAM, the 64 SFX nodes over
//! AX voices, volume envelopes / fades, pan + ITD, and the HPS music stream ("PStream", three
//! 64 KB ARAM blocks refilled from the disc as the voice plays through them).
//!
//! State lives in [`crate::sound::Sound`]; each C global is named beside its field there. Two
//! deliberate differences from the C, both about *when*, not *what*:
//! - DevCom reads (SSM loads, HPS blocks) complete synchronously: the request's completion
//!   callback runs inside the request, where the C runs it later from the DVD/ARQ queue. The
//!   C's waits (`HSD_SynthSFXWaitForLoadCompletion`, the `do { wait_idle } while (busy)` in
//!   HSD_SynthPStreamStart) therefore never spin.
//! - AX's drop callback (`dropcallback`) runs right after `AXAcquireVoice` returns instead of
//!   inside it. It reads nothing AXAcquireVoice changes (the stolen voice keeps its index), so
//!   the outcome is the same.
//!
//! No float site here contracts (no `a*b+c` with the product feeding the add), so there is no
//! FMA to write out; `sqrtf_accurate` (and axdriver's sqrtf / powf, in [`crate::sound`]) are the
//! game's own MSL / lb ports from melee_runtime, which carry their contracted Newton steps.

use crate::layout::*;
use crate::sound::{Group, SfxEntry, Sound, DROP_CALLBACK, NODES};
/// MSL `sqrtf_accurate` (math_ppc.h), each Newton step's `3.0 - g*g*x` one contracted fma.
use melee_runtime::gekko::sqrtf_accurate;

/// lbl_8040806C: the ITD shift per pan step away from centre.
const ITD_SHIFT: [u8; 128] = [
    0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03, 0x04, 0x04, 0x05, 0x05, 0x06, 0x06, 0x07, 0x07,
    0x08, 0x08, 0x09, 0x09, 0x0A, 0x0A, 0x0B, 0x0B, 0x0C, 0x0C, 0x0D, 0x0D, 0x0E, 0x0E, 0x0F, 0x0F, 0x10, 0x10,
    0x11, 0x11, 0x12, 0x12, 0x13, 0x13, 0x14, 0x14, 0x15, 0x15, 0x16, 0x16, 0x16, 0x17, 0x17, 0x17, 0x17, 0x18,
    0x18, 0x18, 0x18, 0x19, 0x19, 0x19, 0x19, 0x19, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1B, 0x1B, 0x1B, 0x1B,
    0x1B, 0x1B, 0x1B, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D,
    0x1D, 0x1D, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F,
    0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F,
    0x1F, 0x1F,
];


fn ve_block(vol: u16, delta: i16) -> [u8; PBVE_SIZE] {
    let mut b = [0u8; PBVE_SIZE];
    w16(&mut b, 0, vol);
    w16(&mut b, 2, delta as u16);
    b
}

fn src_block(ratio: u32) -> [u8; PBSRC_MIN] {
    ratio.to_be_bytes()
}

#[inline]
fn round_up32(x: u32) -> u32 {
    (x + 31) & !31
}

impl Sound {
    // ---- AX helpers ----------------------------------------------------------------------

    /// AXAcquireVoice(prio, dropcallback, 0), then the drop callback for a stolen voice.
    fn acquire(&mut self, prio: u32) -> Option<usize> {
        let mut stolen = None;
        let v = self.ax.acquire_voice(prio, DROP_CALLBACK, 0, |_, s| stolen = Some(s))?;
        if let Some(s) = stolen {
            self.dropcallback(s.index);
        }
        Some(v)
    }

    fn node_voice(&self, n: usize, i: usize) -> usize {
        self.nodes[n].voice[i].expect("node voice")
    }

    fn pb_state(&self, v: usize) -> u16 {
        r16(self.ax.vpb(v), VPB_PB_STATE)
    }

    /// getNode: the node a handle names, if it is still that handle's.
    pub(crate) fn get_node(&self, id: i32) -> Option<usize> {
        let i = (id & 0x3F) as usize;
        (id > 0 && self.nodes[i].x0 == id).then_some(i)
    }

    // ---- init and banks ------------------------------------------------------------------

    /// ARAlloc (shim_ar.c: a stack from 0x4000, 32-byte rounded).
    pub(crate) fn ar_alloc(&mut self, len: u32) -> u32 {
        let a = round_up32(len);
        if self.ar_top as usize + a as usize > self.aram.len() {
            return 0;
        }
        let o = self.ar_top;
        self.ar_top += a;
        o
    }

    /// HSD_SynthInit(dsp_size, voices, stream_size, bank_size).
    pub fn synth_init(&mut self, bank_size: i32) {
        self.ai_dsp_sample_rate(0);
        self.silence = self.ar_alloc(0x500);
        let s = self.silence as usize;
        self.aram[s..s + 0x500].fill(0); // HSD_DevComRequest(..., 3 = fill)
        self.silence *= 2;
        self.bank_head[0] = self.ar_alloc(bank_size as u32) as i32;
        self.ax_callback_registered = true; // AXRegisterCallback(HSD_SynthCallback)
        self.stream_master = 0xFF;
        self.set_ai_stream_vol();
        for c in self.chan.iter_mut() {
            *c = Default::default();
            c.cur = 1.0;
            c.target = 1.0;
        }
        self.bank_ar_end = (self.bank_head[0] + bank_size) as u32;
        self.stream_base = self.ar_alloc(0x30000);
        self.sound_mode = 0; // OSGetSoundMode: the PC shim (shim_os.c) answers 0
    }

    fn ai_dsp_sample_rate(&mut self, r: u32) {
        self.ax.ai_dsp_sample_rate = r;
    }

    fn set_ai_stream_vol(&mut self) {
        let v = (self.stream_vol * self.stream_master as f32) as u8;
        self.ax.ai_stream_vol_left = v;
        self.ax.ai_stream_vol_right = v;
    }

    /// HSD_SynthSFXAllocateBank.
    pub fn sfx_allocate_bank(&mut self, size: i32) {
        let n = self.bank_num;
        let base = self.bank_head[n];
        self.bank_cur[n] = base;
        self.bank_head[n + 1] = size + base;
        assert!(self.bank_head[n + 1] as u32 <= self.bank_ar_end, "bank overflow");
        self.bank_num += 1;
    }

    /// HSD_SynthSFXLoad + its header and sample callbacks, synchronously. `file` is the whole
    /// .ssm. Returns `entrynum` (the C's return) or an error for a malformed file.
    pub fn sfx_load(&mut self, file: &[u8], bank: usize, entrynum: i32) -> Result<i32, String> {
        assert!(bank < self.bank_num, "invalid bankID = {bank}");
        if file.len() < 0x20 {
            return Err("ssm: shorter than its header".into());
        }
        let lb: Vec<u32> = (0..8).map(|i| r32(file, i * 4)).collect();
        let header_size = lb[0];
        let ar_size = lb[1] as i32;
        // TARGET_PC: reclaim the bank when the load does not fit (HSD_SynthSFXHeaderLoadCallback).
        if self.bank_head[bank + 1] - self.bank_cur[bank] < ar_size {
            self.sfx_unload_bank(bank);
        }
        assert!(
            self.bank_head[bank + 1] - self.bank_cur[bank] >= ar_size,
            "Can't load SFX file; bank(id={bank}) buffer overflow."
        );
        let hdr_end = 0x10 + header_size as usize;
        let data_at = round_up32(header_size + 0x10) as usize;
        if file.len() < hdr_end || file.len() < data_at + ar_size as usize {
            return Err("ssm: truncated".into());
        }
        // The sample data, straight into the bank's ARAM cursor.
        let dst = self.bank_cur[bank] as usize;
        self.aram[dst..dst + ar_size as usize].copy_from_slice(&file[data_at..data_at + ar_size as usize]);
        // The header: file[0x10 .. 0x10 + header_size] (loadbuf[4..8] + the rest, as the C
        // reassembles it).
        let hdr = &file[0x10..hdr_end];
        let count = lb[2] as i32;
        let base = lb[3] as i32;
        let reloc = (self.bank_cur[bank] as u32).wrapping_mul(2);
        let mut p = 0usize;
        for i in 0..count {
            if p + 8 > hdr.len() {
                return Err(format!("ssm: sound {i} header past the end"));
            }
            let n = r32(hdr, p);
            let rate = r32(hdr, p + 4);
            let nbytes = (n as usize) * 0x40 + 8;
            if p + nbytes > hdr.len() {
                return Err(format!("ssm: sound {i} voices past the end"));
            }
            let mut voices = Vec::with_capacity(n as usize);
            for k in 0..n as usize {
                let mut vb = [0u8; 0x40];
                vb.copy_from_slice(&hdr[p + 8 + k * 0x40..p + 8 + (k + 1) * 0x40]);
                // `e + 0x10 != NULL` is always true in the C: the loop address is relocated too.
                for o in [4usize, 8, 0xC] {
                    let a = r32(&vb, o).wrapping_add(reloc);
                    w32(&mut vb, o, a);
                }
                voices.push(vb);
            }
            let id = base + i;
            self.data_hash[(id & 0x1F) as usize].insert(0, SfxEntry { id, rate, voices });
            p += nbytes;
        }
        self.groups[bank].push(Group { entrynum, base, count, ar: self.bank_cur[bank], ar_size });
        self.bank_cur[bank] += ar_size;
        Ok(entrynum)
    }

    /// HSD_SynthSFXDataUnlink.
    pub fn sfx_data_unlink(&mut self, sfx_id: i32) {
        let b = &mut self.data_hash[(sfx_id & 0x1F) as usize];
        if let Some(i) = b.iter().position(|e| e.id == sfx_id) {
            b.remove(i);
        }
    }

    fn group_data_unlink(&mut self, g: Group) {
        for i in 0..g.count {
            self.sfx_data_unlink(g.base + i);
        }
    }

    /// HSD_SynthSFXUnloadBank.
    pub fn sfx_unload_bank(&mut self, bank: usize) {
        self.sfx_stop_range(bank);
        let gs = std::mem::take(&mut self.groups[bank]);
        for g in gs {
            self.group_data_unlink(g);
        }
        self.bank_cur[bank] = self.bank_head[bank];
    }

    /// HSD_SynthSFXGroupDataRemove: drop the group a load's entrynum made.
    pub fn sfx_group_data_remove(&mut self, entrynum: i32) {
        for b in 0..32 {
            if let Some(i) = self.groups[b].iter().position(|g| g.entrynum == entrynum) {
                let g = self.groups[b].remove(i);
                self.group_data_unlink(g);
                return;
            }
        }
    }

    /// HSD_SynthSFXBankDeflag + HSD_SynthSFXGroupDataReaddress: compact a bank's groups to its
    /// base (the ARAM copy is a memmove; the sounds' addresses move with it).
    pub fn sfx_bank_deflag(&mut self, bank: usize) {
        self.sfx_stop_range(bank);
        let mut offset = self.bank_head[bank];
        for gi in 0..self.groups[bank].len() {
            let g = self.groups[bank][gi];
            if g.ar != offset {
                let (from, to, n) = (g.ar as usize, offset as usize, g.ar_size as usize);
                self.aram.copy_within(from..from + n, to);
                let delta = ((offset - g.ar) as u32).wrapping_mul(2);
                for id in g.base..g.base + g.count {
                    for e in self.data_hash[(id & 0x1F) as usize].iter_mut().filter(|e| e.id == id) {
                        for vb in e.voices.iter_mut() {
                            if r16(vb, 0) != 0 {
                                let a = r32(vb, 4).wrapping_add(delta);
                                w32(vb, 4, a);
                            }
                            for o in [8usize, 0xC] {
                                let a = r32(vb, o).wrapping_add(delta);
                                w32(vb, o, a);
                            }
                        }
                    }
                }
                self.groups[bank][gi].ar = offset;
            }
            offset += g.ar_size;
        }
        self.bank_deflag[bank] = offset;
    }

    // ---- voices and nodes ----------------------------------------------------------------

    /// HSD_SynthSFXStopNode.
    pub(crate) fn sfx_stop_node(&mut self, n: usize) {
        let node = self.nodes[n];
        if node.flags & 1 == 0 && node.x27 == 1 {
            self.drv_kill_callback(node.x0);
        }
        for i in 0..node.voice_count as usize {
            let v = self.node_voice(n, i);
            self.ax.free_voice(v);
            self.nodes[v].x0 = 0;
        }
    }

    /// dropcallback: AX stole `voice` for another acquire.
    pub(crate) fn dropcallback(&mut self, voice: usize) {
        for i in 0..self.free_count {
            if self.free_list[i] == Some(voice) {
                self.free_list[i] = None;
                break;
            }
        }
        let mut n = voice;
        if self.nodes[n].x0 == 0 {
            return;
        }
        if self.nodes[n].x0 == -1 {
            n = self.node_voice(n, 0); // a secondary voice: its primary's node
        }
        let node = self.nodes[n];
        if node.flags & 1 == 0 && node.x27 == 1 {
            self.drv_kill_callback(node.x0);
        }
        for i in 0..node.voice_count as usize {
            let v = self.node_voice(n, i);
            if v != voice && self.free_count < self.free_list.len() {
                self.free_list[self.free_count] = Some(v);
                self.free_count += 1;
            }
            self.nodes[v].x0 = 0;
        }
    }

    /// HSD_Synth_80389334: start SSM sound `sfx_id` on one or two voices; the node handle or -1.
    #[allow(clippy::too_many_arguments)]
    /// Whether sound `sfx_id` is in a loaded bank.
    pub fn has_sfx(&self, sfx_id: i32) -> bool {
        self.data_hash[(sfx_id & 0x1F) as usize].iter().any(|e| e.id == sfx_id)
    }

    pub(crate) fn synth_play(
        &mut self,
        sfx_id: i32,
        vol: u8,
        vol2: u8,
        pan: u8,
        priority: i32,
        itd_flag: i32,
        pitch1: f32,
        pitch2: f32,
        mix_main: f32,
        mix_aux_a: f32,
        mix_aux_b: f32,
    ) -> i32 {
        let Some(e) = self.data_hash[(sfx_id & 0x1F) as usize].iter().find(|e| e.id == sfx_id).cloned() else {
            return -1;
        };
        let nv = e.voices.len().min(2);
        let mut voices: [Option<usize>; 2] = [None, None];
        for k in 0..nv {
            voices[k] = self.acquire((priority + 1) as u32);
            if voices[k].is_none() {
                if let Some(v0) = voices[0] {
                    self.ax.free_voice(v0);
                }
                return -1;
            }
        }
        if nv == 2 && voices[0] == voices[1] {
            self.ax.free_voice(voices[0].unwrap());
            return -1;
        }
        if nv == 2 {
            let v1 = voices[1].unwrap();
            self.nodes[v1].x0 = -1;
            self.nodes[v1].voice[0] = voices[0];
        }
        let n = voices[0].unwrap();
        {
            let node = &mut self.nodes[n];
            node.x27 = 1;
            node.sfx_id = sfx_id;
            node.flags = 0;
            node.voice_count = nv as u8;
            node.xb = itd_flag as u8;
            node.voice = voices;
            node.x14 = 0.00003125f32 * e.rate as i32 as f32;
            node.x18 = [pitch1, pitch2];
            let vol_norm = 1.0f32 / 255.0 * vol as f32;
            let vol2_norm = 1.0f32 / 255.0 * vol2 as f32;
            node.unk28 = vol_norm;
            node.vol_target[0] = vol_norm;
            node.fade[0] = 0;
            node.vol2 = vol2_norm;
            node.vol_target[1] = vol2_norm;
            node.fade[1] = 0;
            node.pan = pan;
            node.x44 = mix_main;
            node.x48 = mix_aux_a;
            node.x4c = mix_aux_b;
        }
        self.sfx_update_volume(n);
        self.sfx_update_mix(n, 0);
        let node = self.nodes[n];
        let cur =
            (32767.0f32 * (node.vol2 * (node.unk28 * (self.stream_vol * self.chan[node.xb as usize].cur)))) as u16;
        self.nodes[n].x24 = cur;
        let ve = ve_block(cur, 0);
        let ratio = (65536.0f32 * (node.x18[1] * (node.x14 * node.x18[0]))) as u32;
        for (k, vb) in e.voices.iter().take(nv).enumerate() {
            let v = voices[k].unwrap();
            self.ax.set_voice_priority(v, priority as u32);
            self.ax.set_voice_ve(v, &ve);
            self.ax.set_voice_src(v, &src_block(ratio));
            self.ax.set_voice_addr(v, &vb[0..PBADDR_SIZE]);
            self.ax.set_voice_adpcm(v, &vb[0x10..0x10 + PBADPCM_SIZE]);
            self.ax.set_voice_adpcm_loop(v, &vb[0x38..0x38 + PBADPCMLOOP_SIZE]);
            self.ax.set_voice_state(v, 1);
        }
        self.handle_seq = self.handle_seq.wrapping_add(0x40);
        if self.handle_seq < 0 {
            self.handle_seq = 0x40;
        }
        self.nodes[n].x0 = self.handle_seq + n as i32;
        self.nodes[n].x0
    }

    /// HSD_SynthSFXPlayWithGroup.
    #[allow(clippy::too_many_arguments)]
    pub fn sfx_play_with_group(
        &mut self,
        sfx_id: i32,
        vol: u8,
        vol2: u8,
        pan: u8,
        priority: i32,
        itd_flag: i32,
        group: i32,
        pitch1: f32,
        pitch2: f32,
        mix_main: f32,
        mix_aux_a: f32,
        mix_aux_b: f32,
    ) -> i32 {
        assert!((0..0x100).contains(&group), "sfx group ID {group} out of range.");
        if group != 0 {
            let id = self.group_last[group as usize];
            if id > 0 {
                if let Some(n) = self.get_node(id) {
                    if self.nodes[n].flags != 1 {
                        if self.nodes[n].x27 == 1 {
                            let x0 = self.nodes[n].x0;
                            self.drv_kill_callback(x0);
                        }
                        self.sfx_key_off(id);
                    }
                }
            }
        }
        let r = self.synth_play(sfx_id, vol, vol2, pan, priority, itd_flag, pitch1, pitch2, mix_main, mix_aux_a, mix_aux_b);
        self.group_last[group as usize] = r;
        r
    }

    fn free_node_voices(&mut self, n: usize) {
        for j in 0..self.nodes[n].voice_count as usize {
            let v = self.node_voice(n, j);
            self.ax.free_voice(v);
            self.nodes[v].x0 = 0;
        }
    }

    /// HSD_SynthSFXKeyOff: fade out (flags |= 1), the envelope frees the voices.
    pub fn sfx_key_off(&mut self, id: i32) {
        let Some(n) = self.get_node(id) else { return };
        let node = self.nodes[n];
        if node.flags & 8 != 0 {
            if node.flags & 1 == 0 && node.x27 == 1 {
                self.drv_kill_callback(node.x0);
            }
            self.free_node_voices(n);
        } else if node.flags & 1 == 0 {
            self.nodes[n].flags |= 1;
            self.sfx_update_volume(n);
            for i in 0..node.voice_count as usize {
                let v = self.node_voice(n, i);
                self.ax.set_voice_priority(v, 1);
            }
        }
    }

    /// HSD_SynthSFXStopRange: stop every node playing from this bank's ARAM.
    pub fn sfx_stop_range(&mut self, bank: usize) {
        let lo = (self.bank_head[bank] as u32).wrapping_mul(2);
        let hi = (self.bank_head[bank + 1] as u32).wrapping_mul(2);
        for i in 0..NODES {
            if self.nodes[i].x0 > 0 {
                let v = self.node_voice(i, 0);
                let addr = r32(self.ax.vpb(v), VPB_CURADDR);
                if addr >= lo && addr < hi {
                    self.sfx_stop_node(i);
                }
            }
        }
    }

    /// HSD_SynthSFXPause.
    pub fn sfx_pause(&mut self, id: i32) {
        if let Some(n) = self.get_node(id) {
            self.nodes[n].flags |= 2;
            self.sfx_update_volume(n);
            if self.nodes[n].flags & 8 != 0 {
                self.nodes[n].flags |= 6;
            }
        }
    }

    /// HSD_SynthSFXResume.
    pub fn sfx_resume(&mut self, id: i32) {
        if let Some(n) = self.get_node(id) {
            if self.nodes[n].flags & 2 != 0 {
                self.nodes[n].flags &= !6;
                self.sfx_update_pitch(n);
                self.sfx_update_volume(n);
            }
        }
    }

    /// HSD_SynthSFXCheck: the handle while it is sounding, else -1.
    pub fn sfx_check(&self, id: i32) -> i32 {
        let Some(n) = self.get_node(id) else { return -1 };
        let node = &self.nodes[n];
        if node.flags & 1 != 0 {
            return -1;
        }
        for i in 0..node.voice_count as usize {
            if self.pb_state(self.node_voice(n, i)) == 0 {
                return -1;
            }
        }
        id
    }

    /// HSD_SynthSFXSetVolumeFade(id, vol, flag): `flag` picks the fader; 160-step fades.
    pub fn sfx_set_volume_fade(&mut self, id: i32, vol: u8, flag: i32) {
        if let Some(n) = self.get_node(id) {
            assert!(flag == 0 || flag == 1);
            // `1 / 255.0 * vol` is a double expression in the C, stored as float.
            self.nodes[n].vol_target[flag as usize] = (1.0f64 / 255.0 * vol as f64) as f32;
            self.nodes[n].fade[flag as usize] = 1;
            self.sfx_update_volume(n);
        }
    }

    /// HSD_SynthSFXSetPan.
    pub fn sfx_set_pan(&mut self, id: i32, pan: u8) {
        if let Some(n) = self.get_node(id) {
            self.nodes[n].pan = pan;
            self.sfx_update_mix(n, 1);
        }
    }

    /// HSD_SynthSFXSetMix.
    pub fn sfx_set_mix(&mut self, id: i32, main: f32, aux_a: f32, aux_b: f32) {
        if let Some(n) = self.get_node(id) {
            self.nodes[n].x44 = main;
            self.nodes[n].x48 = aux_a;
            self.nodes[n].x4c = aux_b;
            self.sfx_update_mix(n, 1);
        }
    }

    /// HSD_SynthSFXUpdatePitch.
    pub(crate) fn sfx_update_pitch(&mut self, n: usize) {
        let node = self.nodes[n];
        let ratio = if node.flags & 4 != 0 { 0.0 } else { node.x14 * node.x18[0] * node.x18[1] };
        if node.flags & 8 == 0 {
            for i in 0..node.voice_count as usize {
                let v = self.node_voice(n, i);
                self.ax.set_voice_src_ratio(v, ratio);
            }
        }
    }

    /// HSD_SynthSFXSetPitchRatio(id, which, ratio).
    pub fn sfx_set_pitch_ratio(&mut self, id: i32, flag: i32, ratio: f32) {
        if let Some(n) = self.get_node(id) {
            if self.nodes[n].flags & 1 == 0 {
                assert!(flag == 0 || flag == 1);
                self.nodes[n].x18[flag as usize] = ratio;
                self.sfx_update_pitch(n);
            }
        }
    }

    /// HSD_SynthSFXSetPriority.
    pub fn sfx_set_priority(&mut self, id: i32, prio: i32) {
        if let Some(n) = self.get_node(id) {
            if self.nodes[n].flags & 1 == 0 {
                for i in 0..self.nodes[n].voice_count as usize {
                    let v = self.node_voice(n, i);
                    self.ax.set_voice_priority(v, prio as u32);
                }
            }
        }
    }

    /// HSD_SynthSFXVolumeEnvelope: one 5 ms step of every pending node's fades and its AX
    /// volume ramp; a node whose fade ends is dropped from the list (and freed when keyed off,
    /// or parked when paused).
    pub(crate) fn sfx_volume_envelope(&mut self) {
        if self.chan_fading != 0 {
            for ch in 0..16 {
                if self.chan_fading & (1 << ch) != 0 {
                    let c = &mut self.chan[ch];
                    let cnt = c.count;
                    if cnt != 0 {
                        let cur = c.cur;
                        c.cur = (cur * (cnt as f32 - 1.0)) / cnt as f32 + c.target / cnt as f32;
                        c.count -= 1;
                    }
                    if c.count == 0 {
                        c.cur = c.target;
                        self.chan_fading &= !(1 << ch);
                    }
                }
            }
        }
        let mut prev: Option<usize> = None;
        let mut cur = self.vol_list;
        while let Some(n) = cur {
            let next = self.nodes[n].x20;
            let unlink = |s: &mut Sound| match prev {
                None => s.vol_list = next,
                Some(p) => s.nodes[p].x20 = next,
            };
            if self.nodes[n].x0 <= 0 {
                self.nodes[n].volume_update_pending = 0;
                unlink(self);
                cur = next;
                continue;
            }
            let mut active = false;
            {
                let node = &mut self.nodes[n];
                for k in 0..2 {
                    let c = node.fade[k];
                    if c != 0 {
                        let slot = if k == 0 { &mut node.unk28 } else { &mut node.vol2 };
                        *slot = (*slot * (c as f32 - 1.0)) / c as f32 + node.vol_target[k] / c as f32;
                        node.fade[k] -= 1;
                        if node.fade[k] != 0 {
                            active = true;
                        }
                    }
                }
            }
            let node = self.nodes[n];
            if self.chan[node.xb as usize].count != 0 {
                active = true;
            }
            let vol: i32 = if node.flags & 3 == 0 {
                (32767.0f32 * (node.vol2 * (node.unk28 * (self.stream_vol * self.chan[node.xb as usize].cur)))) as i32
            } else {
                active = false;
                0
            };
            let delta = ((vol - node.x24 as i32) / 160).clamp(-0x14, 0x14);
            for i in 0..node.voice_count as usize {
                let v = self.node_voice(n, i);
                self.ax.set_voice_ve_delta(v, delta as i16);
            }
            self.nodes[n].x24 = node.x24.wrapping_add((delta * 0xA0) as u16);
            if delta == 0 && (!active || vol as f32 == 0.0) {
                self.nodes[n].x24 = vol as u16;
                let ve = ve_block(vol as u16, 0);
                for i in 0..node.voice_count as usize {
                    let v = self.node_voice(n, i);
                    let s = r32(self.ax.vpb(v), VPB_SYNC) & 0xFFFF_FBFF;
                    w32(self.ax.vpb_mut(v), VPB_SYNC, s);
                    self.ax.set_voice_ve(v, &ve);
                }
                if !active {
                    self.nodes[n].volume_update_pending = 0;
                    unlink(self);
                    let flags = node.flags;
                    if flags & 1 != 0 {
                        self.free_node_voices(n);
                    } else if flags & 6 == 2 {
                        self.nodes[n].flags = flags | 6;
                        for _ in 0..node.voice_count {
                            self.sfx_update_pitch(n);
                        }
                        if node.x27 == 1 {
                            self.drv_pause_callback(node.x0);
                        }
                    }
                    cur = next;
                    continue;
                }
            }
            prev = Some(n);
            cur = next;
        }
    }

    /// HSD_SynthSFXUpdateVolume: queue the node for the envelope.
    pub(crate) fn sfx_update_volume(&mut self, n: usize) {
        let node = &mut self.nodes[n];
        if node.volume_update_pending == 0 && node.flags & 8 == 0 {
            node.volume_update_pending = 1;
            node.x20 = self.vol_list;
            self.vol_list = Some(n);
        }
    }

    /// HSD_SynthSFXUpdateMix: pan law, sends, ITD.
    pub(crate) fn sfx_update_mix(&mut self, n: usize, interpolate: i32) {
        let node = self.nodes[n];
        let (mut l, r) = if self.sound_mode != 0 {
            (
                32767.0f32 * sqrtf_accurate(0.003921569f32 * (0xFF - node.pan as i32) as f32),
                32767.0f32 * sqrtf_accurate(0.003921569f32 * node.pan as f32),
            )
        } else {
            (23169.768f32, 23169.768f32)
        };
        let mut mix = [0u8; PBMIX_SIZE];
        let set = |m: &mut [u8; PBMIX_SIZE], t: usize, x: f32| w16(m, PBMIX_OFF[t], x as u16);
        // PBMIX_OFF order: L, R, S, AL, AR, AS, BL, BR, BS.
        if node.voice_count == 1 {
            set(&mut mix, 0, l * node.x44);
            set(&mut mix, 1, r * node.x44);
            set(&mut mix, 3, l * node.x48);
            set(&mut mix, 4, r * node.x48);
            set(&mut mix, 6, l * node.x4c);
            set(&mut mix, 7, r * node.x4c);
            let v = self.node_voice(n, 0);
            self.ax.set_voice_mix(v, &mix);
        } else if self.sound_mode != 0 {
            set(&mut mix, 0, l * node.x44);
            set(&mut mix, 3, l * node.x48);
            set(&mut mix, 6, l * node.x4c);
            let v0 = self.node_voice(n, 0);
            self.ax.set_voice_mix(v0, &mix);
            set(&mut mix, 6, 0.0);
            set(&mut mix, 3, 0.0);
            set(&mut mix, 0, 0.0);
            set(&mut mix, 1, r * node.x44);
            set(&mut mix, 4, r * node.x48);
            set(&mut mix, 7, r * node.x4c);
            let v1 = self.node_voice(n, 1);
            self.ax.set_voice_mix(v1, &mix);
        } else {
            l /= 2.0;
            for (t, g) in [(0, node.x44), (1, node.x44), (3, node.x48), (4, node.x48), (6, node.x4c), (7, node.x4c)] {
                set(&mut mix, t, l * g);
            }
            let v0 = self.node_voice(n, 0);
            let v1 = self.node_voice(n, 1);
            self.ax.set_voice_mix(v0, &mix);
            self.ax.set_voice_mix(v1, &mix);
        }

        let (mut shift_l, mut shift_r) = (0u16, 0u16);
        if self.sound_mode != 0 {
            if node.pan < 0x80 {
                shift_l = ITD_SHIFT[(0x7F - node.pan) as usize] as u16;
            } else {
                shift_r = ITD_SHIFT[(node.pan - 0x80) as usize] as u16;
            }
        }
        let v0 = self.node_voice(n, 0);
        for i in 0..node.voice_count as usize {
            let v0_itd = r16(self.ax.vpb(v0), VPB_ITD_FLAG);
            if shift_l != 0 || shift_r != 0 || v0_itd == 1 {
                let v = self.node_voice(n, i);
                let itd = r16(self.ax.vpb(v), VPB_ITD_FLAG);
                let sync = r32(self.ax.vpb(v), VPB_SYNC);
                if itd == 0 || sync & 0x20 != 0 {
                    self.ax.set_voice_itd_on(v);
                    // Direct pb.itd writes, as the C makes them (the shim's mixer does not read
                    // these fields back; only AXSetVoiceItdTarget moves its shifts).
                    let p = self.ax.vpb_mut(v);
                    if interpolate == 0 {
                        w16(p, VPB_ITD_FLAG + ITD_SHIFT_R, shift_r);
                        w16(p, VPB_ITD_FLAG + ITD_SHIFT_L, shift_l);
                    }
                    w16(p, VPB_ITD_FLAG + ITD_TARGET_R, shift_r);
                    w16(p, VPB_ITD_FLAG + ITD_TARGET_L, shift_l);
                } else {
                    self.ax.set_voice_itd_target(v, shift_l, shift_r);
                }
            }
        }
    }

    /// HSD_SynthSFXUpdateAllVolume(vol, fade_frames, channel): fade a whole channel.
    pub fn sfx_update_all_volume(&mut self, vol: i32, fade_frames: u16, channel: usize) {
        self.chan[channel].target = 1.0f32 / 255.0 * vol as f32;
        if self.chan[channel].cur != self.chan[channel].target {
            self.chan[channel].count = fade_frames as i32;
            self.update_all_volume(1 << channel);
        }
    }

    fn update_all_volume(&mut self, mask: u32) {
        self.chan_fading |= mask;
        for i in 0..NODES {
            if self.nodes[i].x0 > 0 && (1u32 << self.nodes[i].xb) & mask != 0 {
                self.sfx_update_volume(i);
            }
        }
    }

    /// HSD_SynthSetSoundMode (0 mono, 1 stereo).
    pub fn set_sound_mode(&mut self, mode: u32) {
        self.sound_mode = mode;
        for i in 0..NODES {
            if self.nodes[i].x0 > 0 {
                self.sfx_update_mix(i, 1);
            }
        }
    }

    /// HSD_SynthCallback: the AX frame callback, once per 5 ms sub-frame before mixing.
    pub fn synth_callback(&mut self) {
        let base = (self.clock % 8 * 8) as usize;
        while self.free_count > 0 {
            self.free_count -= 1;
            if let Some(v) = self.free_list[self.free_count] {
                self.ax.free_voice(v);
            }
        }
        self.free_count = 0;
        for i in 0..8 {
            let n = base + i;
            let node = self.nodes[n];
            if node.x0 > 0
                && node.flags & 8 == 0
                && self.pb_state(self.node_voice(n, 0)) == 0
                && (node.voice_count == 1 || self.pb_state(self.node_voice(n, 1)) == 0)
            {
                self.sfx_stop_node(n);
            }
        }
        self.sfx_volume_envelope();
        if self.drv_installed {
            self.drv_callback(); // driverMasterClockCallback = AXDriverCallback
        }
        self.pstream_master_clock();
        self.clock = self.clock.wrapping_add(1);
    }

    // ---- the music stream ----------------------------------------------------------------

    fn hako(&self, slot: u32) -> (i32, i32, i32) {
        let h = &self.hako[slot as usize];
        (r32(h, 0) as i32, r32(h, 4) as i32, r32(h, 8) as i32)
    }

    /// A DevCom read of the stream file into ARAM.
    fn stream_read_aram(&mut self, src: u32, dst: u32, len: usize) {
        let data = match self.stream_src.as_mut() {
            Some(s) => s.read(src, len),
            None => vec![0; len],
        };
        let d = dst as usize;
        let n = len.min(self.aram.len().saturating_sub(d));
        self.aram[d..d + n].copy_from_slice(&data[..n]);
    }

    fn stream_read_hako(&mut self, src: u32, slot: u32) {
        let data = match self.stream_src.as_mut() {
            Some(s) => s.read(src, 0x20),
            None => vec![0; 0x20],
        };
        self.hako[slot as usize].copy_from_slice(&data);
    }

    fn slot_aram(&self, slot: u32) -> u32 {
        self.stream_base + (slot << 16)
    }

    /// HSD_SynthPStreamMasterClockCallback: follow the voice through the three blocks and
    /// refill the one it left.
    fn pstream_master_clock(&mut self) {
        let Some(n) = self.get_node(self.stream_node) else { return };
        if self.nodes[n].flags & 8 != 0 {
            return;
        }
        let v0 = self.node_voice(n, 0);
        let pos = r32(self.ax.vpb(v0), VPB_CURADDR).wrapping_sub(self.stream_base.wrapping_mul(2)) >> 0x11;
        if pos >= 3 {
            return; // TARGET_PC guard
        }
        let vc = self.nodes[n].voice_count as usize;
        if pos != self.stream_pos {
            self.stream_pos = pos;
            let (x0, x4, _) = self.hako(pos);
            for i in 0..vc {
                let v = self.node_voice(n, i);
                let a = (self.slot_aram(pos).wrapping_mul(2) as i32 + i as i32 * x0 + x4) as u32;
                self.ax.set_voice_end_addr(v, a);
            }
        }
        if pos == self.stream_loop_slot && pos != self.stream_play_slot {
            let (_, _, x8) = self.hako(self.stream_loop_slot);
            self.stream_loop_slot = (self.stream_loop_slot + 1) % 3;
            if x8 as u32 == u32::MAX {
                for i in 0..vc {
                    let v = self.node_voice(n, i);
                    self.ax.set_voice_loop(v, 0);
                    self.ax.set_voice_loop_addr(v, self.silence);
                }
            } else {
                let s = self.stream_loop_slot;
                let (x0, _, _) = self.hako(s);
                for i in 0..vc {
                    let v = self.node_voice(n, i);
                    let a = (self.slot_aram(s).wrapping_mul(2) as i32 + i as i32 * x0 + 2) as u32;
                    self.ax.set_voice_loop_addr(v, a);
                    let lp = self.hako[s as usize][0xC + i * 8..0xC + i * 8 + PBADPCMLOOP_SIZE].to_vec();
                    self.ax.set_voice_adpcm_loop(v, &lp);
                }
            }
        }
        // HSD_SynthPStreamMasterClockCallback_inline: load the next block's header + data.
        if (self.stream_play_slot + 1) % 3 != pos {
            if self.stream_busy != 0 || self.stream_load_slot != self.stream_play_slot {
                return;
            }
            if self.get_node(self.stream_node).is_some() {
                let (_, _, src) = self.hako(self.stream_play_slot);
                if src == -1 {
                    self.stream_play_slot = (self.stream_play_slot + 1) % 3;
                } else {
                    self.stream_load_slot = (self.stream_play_slot + 1) % 3;
                    self.stream_busy = 1;
                    let src = src as u32;
                    self.stream_read_hako(src, self.stream_load_slot);
                    // HSD_SynthPStreamHakoHeaderCallback, then HSD_SynthResetStreamCounters.
                    let (x0, _, _) = self.hako(self.stream_load_slot);
                    let dst = self.slot_aram(self.stream_load_slot);
                    self.stream_read_aram(src + 0x20, dst, x0.max(0) as usize);
                    self.stream_play_slot = self.stream_load_slot;
                    self.stream_busy = 0;
                }
            }
        }
    }

    /// HSD_SynthPStreamStart(entrynum, vol, vol2, channel): key the old stream off and start
    /// `src` (an .hps file). Returns the stream's node handle.
    pub fn pstream_start(&mut self, src: Box<dyn crate::disc::StreamSource>, entrynum: i32, vol: u8, vol2: u8, channel: i32) -> i32 {
        self.stream_busy = 1;
        if let Some(n) = self.get_node(self.stream_node) {
            if self.nodes[n].flags & 8 != 0 {
                self.sfx_stop_node(n);
            } else if self.nodes[n].flags & 1 == 0 {
                self.nodes[n].flags |= 1;
                self.sfx_update_volume(n);
                for i in 0..self.nodes[n].voice_count as usize {
                    let v = self.node_voice(n, i);
                    self.ax.set_voice_priority(v, 1);
                }
            }
        }
        self.stream_entrynum = entrynum;
        self.stream_src = Some(src);
        let v = self.acquire(0x1D).expect("AX voice for the stream");
        self.handle_seq = self.handle_seq.wrapping_add(0x40);
        if self.handle_seq < 0 {
            self.handle_seq = 0x40;
        }
        self.stream_node = self.handle_seq + v as i32;
        let node = &mut self.nodes[v];
        node.x27 = 2;
        node.x0 = self.stream_node;
        node.sfx_id = 0;
        node.flags = 8;
        node.voice_count = 1;
        node.xb = channel as u8;
        node.voice = [Some(v), None];
        node.x18 = [1.0, 1.0];
        node.unk28 = 0.003921569f32 * vol as f32;
        node.vol_target[0] = 0.003921569f32 * vol as f32;
        node.fade[0] = 0;
        node.vol2 = 0.003921569f32 * vol2 as f32;
        node.vol_target[1] = 0.003921569f32 * vol2 as f32;
        node.fade[1] = 0;
        node.pan = 0x80;
        node.x44 = 1.0;
        node.x48 = 0.0;
        node.x4c = 0.0;
        let header = match self.stream_src.as_mut() {
            Some(s) => s.read(0, 0x80),
            None => vec![0; 0x80],
        };
        self.pstream_header(&header);
        self.stream_node
    }

    /// HSD_SynthPStreamHeaderCallback + FirstHakoHeader + FirstHakoData.
    fn pstream_header(&mut self, entry: &[u8]) {
        let Some(n) = self.get_node(self.stream_node) else {
            self.stream_busy = 0;
            return;
        };
        let channels = r32(entry, 0xC).min(2) as u8;
        self.nodes[n].voice_count = channels;
        if channels == 2 {
            let v1 = self.acquire(0x1D).expect("entry->voice[1]");
            self.nodes[n].voice[1] = Some(v1);
        }
        // The second acquire may have stolen and dropped the stream's own node.
        if self.get_node(self.stream_node).is_none() {
            self.stream_busy = 0;
            return;
        }
        self.nodes[n].x14 = 0.00003125f32 * r32(entry, 8) as f32;
        for i in 0..channels as usize {
            let v = self.node_voice(n, i);
            let o = 0x10 + i * 0x38;
            self.ax.set_voice_addr(v, &entry[o..o + PBADDR_SIZE]);
            self.ax.set_voice_adpcm(v, &entry[o + 0x10..o + 0x10 + PBADPCM_SIZE]);
        }
        self.stream_pos = (self.stream_pos + 2) % 3;
        self.stream_play_slot = self.stream_pos;
        self.stream_loop_slot = self.stream_pos;
        self.stream_load_slot = self.stream_pos;
        let s = self.stream_load_slot;
        self.stream_read_hako(0x80, s);
        let (x0, _, _) = self.hako(s);
        let dst = self.slot_aram(s);
        self.stream_read_aram(0xA0, dst, x0.max(0) as usize);
        self.pstream_first_data();
    }

    /// HSD_SynthPStreamFirstHakoDataCallback: the first block is in, start the voices.
    fn pstream_first_data(&mut self) {
        let Some(n) = self.get_node(self.stream_node) else {
            self.stream_busy = 0;
            return;
        };
        let node = self.nodes[n];
        let cur = if node.flags & 2 == 0 {
            (32767.0f32 * (node.vol2 * (node.unk28 * (self.stream_vol * self.chan[node.xb as usize].cur)))) as u16
        } else {
            0
        };
        self.nodes[n].x24 = cur;
        let s = self.stream_load_slot;
        let (x0, x4, _) = self.hako(s);
        let base = self.slot_aram(s).wrapping_mul(2) as i32;
        for i in 0..node.voice_count as usize {
            let v = self.node_voice(n, i);
            self.ax.set_voice_ve(v, &ve_block(cur, 0));
            let ratio = if node.flags & 4 != 0 { 0 } else { (65536.0f32 * (node.x14 * node.x18[0] * node.x18[1])) as u32 };
            self.ax.set_voice_src(v, &src_block(ratio));
            let start = (base + i as i32 * x0 + 2) as u32;
            self.ax.set_voice_current_addr(v, start);
            self.ax.set_voice_end_addr(v, (base + i as i32 * x0 + x4) as u32);
            self.ax.set_voice_loop_addr(v, start);
            self.ax.set_voice_state(v, 1);
        }
        self.nodes[n].flags &= !8;
        self.sfx_update_volume(n);
        self.sfx_update_mix(n, 0);
        self.stream_busy = 0;
    }

    /// HSD_SynthStreamSetVolume.
    pub fn stream_set_volume(&mut self, v: f32) {
        self.stream_vol = v;
        self.set_ai_stream_vol();
        self.update_all_volume(0xFFFF);
    }
}

//! Core v1 (feature `corev1`): our command stream as the console's, for a host that speaks Hero88go's frozen
//! `mu_core_v1.h` boundary and decodes the GameCube GX FIFO itself.
//!
//! Our FIFO is Aurora's: BP/CP/XF writes are the console's, but memory is named by handles
//! (`GX_AURORA` LOAD_ARRAYBASE / LOAD_TEXOBJ / LOAD_TLUT) and draws may be sized or indexed (DRAW_SIZED,
//! DRAW_INDEXED), none of which a console command processor knows. A core v1 host reads vertex arrays, textures,
//! TLUTs and indexed matrices by physical address from its MEM1 window (`mem1_base()`, the console's 0x80000000).
//! So [`Transcoder`] rewrites each drained buffer:
//!
//! * the data a handle names is copied once into MEM1 (a bump arena, cached by the shared buffer, which is
//!   immutable while we hold it) and the register the console would have held is written with its address:
//!   CP 0xA0+n for an array, BP TEX_IMAGE3 for a texture, BP LOADTLUT0 before the LOADTLUT1 that loads it;
//! * DRAW_SIZED becomes a counted primitive, DRAW_INDEXED a triangle list in index order;
//! * GXCopyTex becomes the console's EFB copy registers, into a MEM1 block kept per destination, and a texture
//!   whose image is a copy destination samples that block;
//! * GXCopyDisp (the frame's end, [`frame_end`]) becomes the copy to the XFB, which is where the host presents;
//! * PC-only commands (render viewport / scissor / projection, polygon offset, the Geno matrix palette, pose tags)
//!   are dropped and counted.
//!
//! The host decodes a whole frame within the one `gx_fifo` call that carries its XFB copy (vertex decode is joined
//! there), so MEM1 bytes written for a frame may be reused once that call returns. Only render state crosses here:
//! nothing in the simulation reads it.
use crate::api::Gx;
use crate::consts::*;
use crate::resources::{GxData, Resources};
use crate::state::GxState;
use std::collections::HashMap;

const TEX_IMAGE3_IDS: [u32; 8] = [0x94, 0x95, 0x96, 0x97, 0xB4, 0xB5, 0xB6, 0xB7];
/// The low MEM1 the console's OS globals occupied: left alone in case a host reads them.
const ARENA_START: usize = 1 << 20;
/// Bytes kept at the top of MEM1 for EFB copy destinations.
const COPY_POOL: usize = 4 << 20;

/// Counters for the log line and the notes (`Transcoder::stats_line`).
#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub frames: u64,
    pub bytes_out: u64,
    pub staged_bytes: u64,
    pub staged_buffers: u64,
    pub arena_resets: u64,
    pub sized_draws: u64,
    pub indexed_draws: u64,
    pub tex_copies: u64,
    pub dropped_pc: u64,
    pub dropped_palette: u64,
    pub unknown: u64,
    /// Time spent transcoding (feed + frame end, the host's gx_fifo excluded) and inside the host's gx_fifo.
    pub transcode_ns: u64,
    pub submit_ns: u64,
}

struct Staged {
    phys: u32,
    /// Holds the bytes alive: their address cannot be reused for other data while the mapping exists.
    _keep: GxData,
}

pub struct Transcoder {
    mem: *mut u8,
    mem_len: usize,
    /// Next free byte of the arena (an offset into MEM1).
    top: usize,
    arena_end: usize,
    staged: HashMap<usize, Staged>,
    /// EFB copy destinations: game address -> (MEM1 offset, bytes).
    copies: HashMap<u64, (u32, u32)>,
    copy_top: usize,
    /// CP state (VCD/VAT) only, for vertex sizes.
    cp: GxState,
    /// What is bound, re-sent after an arena reset: array data per CP slot, texture data per map.
    arrays: [Option<GxData>; 16],
    textures: [Option<(GxData, u64)>; 8],
    pending_tlut1: Option<u32>,
    started: bool,
    tlut_fmt: [u32; 1024],
    tex_tlut: [u32; 8],
    pending_tex_tlut: Vec<u32>,
    out: Vec<u8>,
    submit: Box<dyn FnMut(&[u8])>,
    pub stats: Stats,
}

impl Transcoder {
    /// `mem` is the host's MEM1 (`mem1_base()`, `mem1_size()` bytes, physical address 0 at `mem`); `submit` is the
    /// host's `gx_fifo`.
    ///
    /// # Safety
    /// `mem` must be valid for writes of `mem_len` bytes for the life of the transcoder.
    pub unsafe fn new(mem: *mut u8, mem_len: usize, submit: Box<dyn FnMut(&[u8])>) -> Self {
        let mut cp = GxState::default();
        // GXInit's CP state, as our own decoder starts from (the Gx encoder's init writes it into the first drain)
        let _ = &mut cp;
        Self {
            mem,
            mem_len,
            top: ARENA_START,
            arena_end: mem_len.saturating_sub(COPY_POOL),
            staged: HashMap::new(),
            copies: HashMap::new(),
            copy_top: mem_len.saturating_sub(COPY_POOL),
            cp,
            arrays: Default::default(),
            textures: Default::default(),
            pending_tlut1: None,
            started: false,
            tlut_fmt: [0; 1024],
            tex_tlut: [0; 8],
            pending_tex_tlut: Vec::new(),
            out: Vec::with_capacity(1 << 20),
            submit,
            stats: Stats::default(),
        }
    }

    pub fn stats_line(&self) -> String {
        let s = &self.stats;
        format!(
            "core v1 fifo: {} frames, transcode {:.3} ms/frame, host gx_fifo {:.3} ms/frame, {} MB out, {} buffers / {} MB staged, {} arena resets, {} sized + {} indexed draws, {} tex copies, {} PC-only commands dropped ({} palette loads)",
            s.frames,
            (s.transcode_ns.saturating_sub(s.submit_ns)) as f64 / 1e6 / s.frames.max(1) as f64,
            s.submit_ns as f64 / 1e6 / s.frames.max(1) as f64,
            s.bytes_out >> 20,
            s.staged_buffers,
            s.staged_bytes >> 20,
            s.arena_resets,
            s.sized_draws,
            s.indexed_draws,
            s.tex_copies,
            s.dropped_pc,
            s.dropped_palette
        )
    }

    fn bp(&mut self, v: u32) {
        self.out.push(GX_LOAD_BP_REG);
        self.out.extend_from_slice(&v.to_be_bytes());
    }
    fn cpw(&mut self, addr: u8, v: u32) {
        self.out.push(GX_LOAD_CP_REG);
        self.out.push(addr);
        self.out.extend_from_slice(&v.to_be_bytes());
    }

    /// The MEM1 offset (= physical address) holding `d`'s bytes, copying them on first use.
    fn stage(&mut self, d: &GxData) -> u32 {
        let key = d.as_ptr() as usize;
        if let Some(s) = self.staged.get(&key) {
            return s.phys;
        }
        let len = d.len();
        let at = (self.top + 31) & !31;
        if at + len > self.arena_end {
            self.reset_arena();
            return self.stage(d);
        }
        // SAFETY: at + len <= arena_end <= mem_len (checked above); `mem` is valid for mem_len bytes (new).
        unsafe { std::ptr::copy_nonoverlapping(d.as_ptr(), self.mem.add(at), len) };
        self.top = at + len;
        self.stats.staged_bytes += len as u64;
        self.stats.staged_buffers += 1;
        self.staged.insert(key, Staged { phys: at as u32, _keep: d.clone() });
        at as u32
    }

    /// The arena is full in the middle of a frame: let the host finish reading what was sent (a draw-done makes it
    /// decode every queued vertex), start the arena over and re-send what is bound.
    fn reset_arena(&mut self) {
        self.stats.arena_resets += 1;
        if !self.out.is_empty() {
            self.bp(0x4500_0002);
            self.flush_out();
        }
        self.staged.clear();
        self.top = ARENA_START;
        let len = self.staged_need();
        if len > self.arena_end - ARENA_START {
            // even the bound set alone does not fit: nothing more can be done than to keep going
            eprintln!("core v1 fifo: the bound textures and arrays ({len} bytes) exceed MEM1's arena");
        }
        for i in 0..16 {
            if let Some(d) = self.arrays[i].clone() {
                let p = self.stage(&d);
                self.cpw(0xA0 | i as u8, p);
            }
        }
        for m in 0..8 {
            if let Some((d, guest)) = self.textures[m].clone() {
                self.bind_texture(m, Some(&d), guest);
            }
        }
    }

    fn staged_need(&self) -> usize {
        self.arrays.iter().flatten().map(|d| d.len() + 32).sum::<usize>() + self.textures.iter().flatten().map(|d| d.0.len() + 32).sum::<usize>()
    }

    fn flush_out(&mut self) {
        if self.out.is_empty() {
            return;
        }
        self.stats.bytes_out += self.out.len() as u64;
        let buf = std::mem::take(&mut self.out);
        let t = std::time::Instant::now();
        (self.submit)(&buf);
        self.stats.submit_ns += t.elapsed().as_nanos() as u64;
        self.out = buf;
        self.out.clear();
    }

    fn bind_texture(&mut self, map: usize, data: Option<&GxData>, guest: u64) {
        let phys = match (guest != 0).then(|| self.copies.get(&guest)).flatten() {
            Some(&(off, _)) => Some(off),
            None => data.map(|d| self.stage(d)),
        };
        if let Some(p) = phys {
            self.bp(TEX_IMAGE3_IDS[map] << 24 | ((p >> 5) & 0x00FF_FFFF));
        }
    }

    /// One drained buffer of our FIFO (`Gx::drain`), with the handles it names.
    pub fn feed(&mut self, data: &[u8], res: &Resources) {
        if !self.started {
            // GXInit's GXSetScissorBoxOffset(0, 0): (0 + 342) / 2 on both axes. Our encoder keeps no such register
            // (Aurora's scissor is not biased), but a console decoder takes the box offset off every scissor.
            self.started = true;
            self.bp(0x59 << 24 | 171 << 10 | 171);
        }
        let mut p = 0usize;
        let n = data.len();
        let be32 = |at: usize| u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
        let be16 = |at: usize| u16::from_be_bytes([data[at], data[at + 1]]);
        let be64 = |at: usize| u64::from_be_bytes(data[at..at + 8].try_into().unwrap());
        while p < n {
            let cmd = data[p];
            match cmd {
                GX_NOP => p += 1,
                GX_LOAD_BP_REG => {
                    let v = be32(p + 1);
                    p += 5;
                    match v >> 24 {
                        // LOADTLUT0 carries a handle-era address: the real one is written with the AURORA LOAD_TLUT
                        0x64 => {}
                        0x65 => self.pending_tlut1 = Some(v),
                        // TEX_TLUT: the console's GXLoadTexObj adds the palette format of the region (bits 10-11);
                        // ours writes the slot only, which a console decoder reads as IA8 (grey)
                        r @ (0x98..=0x9B | 0xB8..=0xBB) => {
                            let map = if r < 0xB8 { r - 0x98 } else { r - 0xB8 + 4 } as usize;
                            self.tex_tlut[map] = v;
                            let slot = (v & 0x3FF) as usize;
                            let fmt = self.tlut_fmt.get(slot).copied().unwrap_or(0);
                            self.bp((v & !0xC00) | (fmt & 3) << 10);
                        }
                        _ => self.bp(v),
                    }
                }
                GX_LOAD_CP_REG => {
                    let (addr, v) = (data[p + 1], be32(p + 2));
                    self.cp.process(&data[p..p + 6], res);
                    self.out.extend_from_slice(&data[p..p + 6]);
                    let _ = (addr, v);
                    p += 6;
                }
                GX_LOAD_XF_REG => {
                    let hdr = be32(p + 1);
                    let count = ((hdr >> 16) & 0xFFFF) as usize + 1;
                    let len = 5 + count * 4;
                    let at = self.out.len();
                    self.out.extend_from_slice(&data[p..p + len]);
                    // The viewport origin (XF 0x101D x, 0x101E y): Aurora's encoder biases it by 340, the console's
                    // GXSetViewport by 342, which is what a console decoder takes off again.
                    let first = hdr & 0xFFFF;
                    for reg in [0x101D_u32, 0x101E] {
                        if reg >= first && reg < first + count as u32 {
                            let o = at + 5 + (reg - first) as usize * 4;
                            let v = f32::from_bits(u32::from_be_bytes(self.out[o..o + 4].try_into().unwrap())) + 2.0;
                            self.out[o..o + 4].copy_from_slice(&v.to_bits().to_be_bytes());
                        }
                    }
                    p += len;
                }
                GX_LOAD_INDX_A | GX_LOAD_INDX_B | GX_LOAD_INDX_C | GX_LOAD_INDX_D => {
                    self.out.extend_from_slice(&data[p..p + 5]);
                    p += 5;
                }
                GX_CMD_CALL_DL => {
                    // never written by our encoder (display lists are inlined); a console host would read a handle
                    self.stats.unknown += 1;
                    p += 9;
                }
                GX_CMD_INVL_VC => {
                    self.out.push(cmd);
                    p += 1;
                }
                GX_AURORA => p = self.aurora(data, p + 1, res, &be16, &be32, &be64),
                _ if cmd >= 0x80 => {
                    let fmt = (cmd & GX_VAT_MASK) as u32;
                    let count = be16(p + 1) as usize;
                    let vs = crate::draw::calc_vtx_size(&self.cp, fmt) as usize;
                    let len = 3 + count * vs;
                    self.out.extend_from_slice(&data[p..(p + len).min(n)]);
                    p += len;
                }
                _ => {
                    self.stats.unknown += 1;
                    eprintln!("core v1 fifo: unknown opcode {cmd:#04x} at {p} of {n}; rest of the buffer dropped");
                    return;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn aurora(&mut self, data: &[u8], mut p: usize, res: &Resources, be16: &dyn Fn(usize) -> u16, be32: &dyn Fn(usize) -> u32, be64: &dyn Fn(usize) -> u64) -> usize {
        let sub = be16(p);
        p += 2;
        match sub {
            0x0001 => p + 24,           // LOAD_VIEWPORT_RENDER: the XF viewport is the console's
            0x0002 => p + 16,           // LOAD_SCISSOR_RENDER
            0x0003 => p + 64,           // LOAD_PROJECTION_FULL
            0x1000 => p + 20,           // SET_POLYGON_OFFSET
            0x2000 => p + 5,            // POSE_TAG
            0x2001 => p + 4,            // POSE_CAM
            0x0045 => p,                // END_PALETTE
            s if s & 0xFFF0 == 0x0010 => {
                // LOAD_ARRAYBASE | n: handle u64, size u32, le u8 (our encoder writes big-endian arrays only)
                let idx = (s & 0xF) as usize;
                let d = res.get(be64(p));
                let phys = d.as_ref().map(|d| self.stage(d)).unwrap_or(0);
                self.arrays[idx] = d;
                self.cpw(0xA0 | idx as u8, phys);
                p + 13
            }
            0x0030 => {
                // LOAD_TEXOBJ: id u8, handle u64, w, h, fmt, tlut u32, mips u8, obj id, version u32, guest u64
                let id = data[p] as usize;
                let d = res.get(be64(p + 1));
                let guest = be64(p + 1 + 8 + 16 + 1 + 8);
                self.bind_texture(id, d.as_ref(), guest);
                self.textures[id] = d.map(|d| (d, guest));
                p + 1 + 8 + 16 + 1 + 8 + 8
            }
            0x0031 => {
                // LOAD_TLUT: idx u8, handle u64, fmt u32, n u16, obj id, version u32
                let d = res.get(be64(p + 1));
                if let (Some(d), Some(l1)) = (d, self.pending_tlut1.take()) {
                    let phys = self.stage(&d);
                    self.bp(0x64 << 24 | ((phys >> 5) & 0x001F_FFFF));
                    // LOADTLUT1's size (bits 10-20) counts 16-entry blocks on the console; Aurora's encoder writes
                    // entries - 1 there, which a console decoder reads as 16x the palette and copies over its neighbours
                    let entries = be16(p + 1 + 8 + 4) as u32;
                    let slot = (l1 & 0x3FF) as usize;
                    let fmt = be32(p + 1 + 8);
                    if slot < self.tlut_fmt.len() && self.tlut_fmt[slot] != fmt {
                        self.tlut_fmt[slot] = fmt;
                        // a texture bound before its palette was loaded takes the format now
                        for m in 0..8 {
                            let t = self.tex_tlut[m];
                            if t != 0 && (t & 0x3FF) as usize == slot {
                                self.pending_tex_tlut.push((t & !0xC00) | (fmt & 3) << 10);
                            }
                        }
                    }
                    self.bp((l1 & !0x001F_FC00) | (entries.div_ceil(16) & 0x7FF) << 10);
                    for t in std::mem::take(&mut self.pending_tex_tlut) {
                        self.bp(t);
                    }
                }
                p + 1 + 8 + 4 + 2 + 8
            }
            0x1100 => {
                // COPY_TEX: dest u64, src x,y,w,h u32, dst w,h,fmt u32, half u8, clear u8
                let dest = be64(p);
                let (sx, sy, sw, sh) = (be32(p + 8), be32(p + 12), be32(p + 16), be32(p + 20));
                let (dw, dh, fmt) = (be32(p + 24), be32(p + 28), be32(p + 32));
                let (half, clear) = (data[p + 36] != 0, data[p + 37] != 0);
                self.copy_tex(dest, [sx, sy, sw, sh], dw, dh, fmt, half, clear);
                p + 38
            }
            0x0040 => {
                // DRAW_SIZED: cmd u8, byte length u32, vertices
                let cmd = data[p];
                let bytes = be32(p + 1) as usize;
                let vs = crate::draw::calc_vtx_size(&self.cp, (cmd & GX_VAT_MASK) as u32) as usize;
                let verts = &data[p + 5..p + 5 + bytes];
                if vs != 0 && bytes != 0 {
                    self.stats.sized_draws += 1;
                    // strips and fans cannot be cut, and a sized draw never comes near 65535 vertices
                    let count = (bytes / vs).min(0xFFFF);
                    self.out.push(cmd);
                    self.out.extend_from_slice(&(count as u16).to_be_bytes());
                    self.out.extend_from_slice(&verts[..count * vs]);
                }
                p + 5 + bytes
            }
            0x0041 => {
                // DRAW_INDEXED: cmd u8, vertex count u16, index count u32, LE u16 indices, vertices
                let cmd = data[p];
                let nv = be16(p + 1) as usize;
                let ni = be32(p + 3) as usize;
                let vs = crate::draw::calc_vtx_size(&self.cp, (cmd & GX_VAT_MASK) as u32) as usize;
                let idx_at = p + 7;
                let v_at = idx_at + ni * 2;
                if ni != 0 && vs != 0 {
                    self.stats.indexed_draws += 1;
                    for chunk in (0..ni).collect::<Vec<_>>().chunks(0xFFFF) {
                        self.out.push((cmd & GX_VAT_MASK) | GX_TRIANGLES as u8);
                        self.out.extend_from_slice(&(chunk.len() as u16).to_be_bytes());
                        for &k in chunk {
                            let i = u16::from_le_bytes([data[idx_at + 2 * k], data[idx_at + 2 * k + 1]]) as usize;
                            self.out.extend_from_slice(&data[v_at + i * vs..v_at + (i + 1) * vs]);
                        }
                    }
                }
                v_at + nv * vs
            }
            0x0044 => {
                // LOAD_PALETTE (Geno's PC matrix palette): u16 n, u32 key, n*24 floats. A console has no palette.
                self.stats.dropped_palette += 1;
                let k = be16(p) as usize;
                p + 6 + k * 24 * 4
            }
            other => {
                self.stats.unknown += 1;
                eprintln!("core v1 fifo: unknown aurora subcommand {other:#06x}; rest of the buffer dropped");
                data.len()
            }
        }
        .max({
            if matches!(sub, 0x0001 | 0x0002 | 0x0003 | 0x1000 | 0x2000 | 0x2001 | 0x0045) {
                self.stats.dropped_pc += 1;
            }
            0
        })
    }

    /// GXCopyTex as the console's copy registers (GXSetTexCopySrc / GXSetTexCopyDst / GXCopyTex in libogc's terms).
    #[allow(clippy::too_many_arguments)]
    fn copy_tex(&mut self, dest: u64, src: [u32; 4], dw: u32, dh: u32, fmt: u32, half: bool, clear: bool) {
        self.stats.tex_copies += 1;
        let hw = fmt & 0xF;
        // tile shift by bits per texel: 4 bpp 8x8, 8 bpp 8x4, 16 bpp 4x4, RGBA8 4x4 in two 32-byte lines
        let (tw, th, lines) = match hw {
            0 | 8 => (8u32, 8u32, 1u32), // I4, (C4 / Z4 style)
            1 | 2 | 9 => (8, 4, 1),       // I8, IA4, R8-ish
            6 => (4, 4, 2),               // RGBA8
            _ => (4, 4, 1),               // 16 bpp: IA8, RGB565, RGB5A3
        };
        let stride = dw.div_ceil(tw) * lines;
        let bytes = stride * 32 * dh.div_ceil(th);
        let off = match self.copies.get(&dest) {
            Some(&(o, b)) if b >= bytes => o,
            _ => {
                let at = (self.copy_top + 31) & !31;
                if at + bytes as usize > self.mem_len {
                    // the pool is used up: start it over (an old destination's texture shows what the new copy wrote)
                    self.copies.clear();
                    self.copy_top = self.mem_len.saturating_sub(COPY_POOL);
                }
                let at = (self.copy_top + 31) & !31;
                self.copy_top = at + bytes as usize;
                self.copies.insert(dest, (at as u32, bytes));
                at as u32
            }
        };
        let intensity = fmt < 4 && fmt & 0x20 == 0; // I4, I8, IA4, IA8
        let tpf = ((hw & 7) << 1) | ((hw >> 3) & 1);
        self.bp(0x49 << 24 | (src[1] & 0x3FF) << 10 | (src[0] & 0x3FF));
        self.bp(0x4A << 24 | (src[3].saturating_sub(1) & 0x3FF) << 10 | (src[2].saturating_sub(1) & 0x3FF));
        self.bp(0x4B << 24 | ((off >> 5) & 0x00FF_FFFF));
        self.bp(0x4D << 24 | (stride & 0x3FF));
        let trigger = tpf << 3 | (half as u32) << 9 | (clear as u32) << 11 | (intensity as u32) << 15;
        self.bp(0x52 << 24 | trigger);
    }

    /// GXCopyDisp: the frame's end, as a copy to the XFB the host presents, then the whole frame to the host.
    pub fn frame_end(&mut self, clear: bool) {
        self.stats.frames += 1;
        self.bp(0x49 << 24);
        self.bp(0x4A << 24 | 479 << 10 | 639);
        self.bp(0x4B << 24);
        self.bp(0x4D << 24 | (640 * 2 / 32));
        self.bp(0x4E << 24 | 0x100);
        self.bp(0x52 << 24 | 1 << 14 | (clear as u32) << 11);
        self.flush_out();
    }
}

static mut TAP: Option<Transcoder> = None;

/// Route every drained FIFO buffer through `t` from now on (our own decoder no longer runs).
pub fn install(t: Transcoder) {
    // SAFETY: the game thread is the only one that draws; this runs before its first frame.
    unsafe { TAP = Some(t) };
}

#[allow(static_mut_refs)]
pub(crate) fn tap() -> Option<&'static mut Transcoder> {
    // SAFETY: as install.
    unsafe { TAP.as_mut() }
}

pub fn active() -> bool {
    tap().is_some()
}

/// GXCopyDisp's hook (shim.rs).
pub(crate) fn frame_end(clear: bool) {
    if let Some(t) = tap() {
        let at = std::time::Instant::now();
        t.frame_end(clear);
        t.stats.transcode_ns += at.elapsed().as_nanos() as u64;
    }
}

pub fn stats_line() -> Option<String> {
    tap().map(|t| t.stats_line())
}

/// `Gx::drain`'s hook: true when the buffer went to the transcoder.
pub(crate) fn drain(gx_fifo: &[u8], res: &Resources) -> bool {
    match tap() {
        Some(t) => {
            let at = std::time::Instant::now();
            t.feed(gx_fifo, res);
            t.stats.transcode_ns += at.elapsed().as_nanos() as u64;
            true
        }
        None => false,
    }
}

#[allow(dead_code)]
fn _assert_gx(_: &Gx) {}

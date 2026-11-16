//! Raw video output (`--video-out`): every simulated frame's EFB, scaled onto a `w` x `h` RGBA8 target the way the
//! window shows it (alpha opaque, 4:3 content pillarboxed), read back through two staging buffers and written to a
//! byte sink by a writer thread. Frame N is copied and mapped while frame N-1's readback (submitted a whole frame
//! earlier, normally finished) is collected, so the GPU readback does not stall the loop each frame.
//! Presentation only: nothing here reads or writes simulation state.

use std::io::Write;
use std::sync::mpsc::{Receiver, SyncSender};

use crate::render::Renderer;

struct Slot {
    buf: wgpu::Buffer,
    /// The submission whose copy fills `buf`, while its readback is outstanding.
    inflight: Option<wgpu::SubmissionIndex>,
}

pub struct VideoOut {
    pub w: u32,
    pub h: u32,
    padded: u32,
    tex: Option<wgpu::Texture>,
    slots: Vec<Slot>,
    next: usize,
    tx: Option<SyncSender<Vec<u8>>>,
    free: Receiver<Vec<u8>>,
    writer: Option<std::thread::JoinHandle<std::io::Result<()>>>,
    /// Frames handed to the writer.
    pub frames: u64,
    /// `--video-realtime`: the wall-clock start the frames are paced against (60 per second).
    pace: Option<std::time::Instant>,
    failed: bool,
}

impl VideoOut {
    /// A sink for `w` x `h` frames; `realtime` paces the output to 60 frames per second.
    pub fn new(mut out: Box<dyn Write + Send>, w: u32, h: u32, realtime: bool) -> Self {
        // a few frames of slack between the render loop and the writer (a pipe reader such as ffmpeg drains in bursts)
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(4);
        let (back, free) = std::sync::mpsc::channel::<Vec<u8>>();
        let writer = std::thread::Builder::new()
            .name("video out".into())
            .spawn(move || {
                while let Ok(v) = rx.recv() {
                    out.write_all(&v)?;
                    let _ = back.send(v);
                }
                out.flush()
            })
            .expect("video writer thread");
        VideoOut {
            w,
            h,
            padded: (w * 4).div_ceil(256) * 256,
            tex: None,
            slots: Vec::new(),
            next: 0,
            tx: Some(tx),
            free,
            writer: Some(writer),
            frames: 0,
            pace: realtime.then(std::time::Instant::now),
            failed: false,
        }
    }

    /// The sink stopped taking frames (a closed pipe, a full disk).
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Queue the renderer's current EFB (the frame just drawn) as the next video frame. Returns false once the
    /// sink has failed.
    pub fn capture(&mut self, r: &mut Renderer) -> bool {
        if self.failed {
            return false;
        }
        if self.slots.is_empty() {
            for i in 0..2 {
                let buf = r.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(if i == 0 { "video staging 0" } else { "video staging 1" }),
                    size: (self.padded * self.h) as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                self.slots.push(Slot { buf, inflight: None });
            }
        }
        let tex = self.tex.get_or_insert_with(|| {
            r.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("video out"),
                size: wgpu::Extent3d { width: self.w, height: self.h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        });
        let tex = tex.clone();
        let view = tex.create_view(&Default::default());
        r.blit_to(&view, wgpu::TextureFormat::Rgba8Unorm, self.w, self.h);
        let s = self.next;
        // the slot was collected one frame ago; a slot still out (only after a failed collect) is collected now
        if self.slots[s].inflight.is_some() {
            self.collect(r, s);
        }
        let mut enc = r.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: Default::default() },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.slots[s].buf,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(self.padded), rows_per_image: Some(self.h) },
            },
            wgpu::Extent3d { width: self.w, height: self.h, depth_or_array_layers: 1 },
        );
        let idx = r.queue.submit([enc.finish()]);
        self.slots[s].buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        self.slots[s].inflight = Some(idx);
        self.next = 1 - s;
        // the previous frame's readback: submitted a frame ago, normally complete by now
        if self.slots[1 - s].inflight.is_some() {
            self.collect(r, 1 - s);
        }
        !self.failed
    }

    /// Wait for slot `s`'s copy, hand its rows (tight) to the writer, unmap.
    fn collect(&mut self, r: &Renderer, s: usize) {
        let Some(idx) = self.slots[s].inflight.take() else { return };
        let _ = r.device.poll(wgpu::PollType::Wait { submission_index: Some(idx), timeout: None });
        let row = (self.w * 4) as usize;
        let mut out = self.free.try_recv().unwrap_or_default();
        out.clear();
        out.reserve(row * self.h as usize);
        {
            let data = self.slots[s].buf.slice(..).get_mapped_range();
            for y in 0..self.h as usize {
                let o = y * self.padded as usize;
                out.extend_from_slice(&data[o..o + row]);
            }
        }
        self.slots[s].buf.unmap();
        self.emit(out);
    }

    fn emit(&mut self, v: Vec<u8>) {
        if let Some(t0) = self.pace {
            let due = t0 + std::time::Duration::from_nanos(self.frames * 1_000_000_000 / 60);
            let now = std::time::Instant::now();
            if due > now {
                std::thread::sleep(due - now);
            }
        }
        let ok = self.tx.as_ref().is_some_and(|tx| tx.send(v).is_ok());
        if ok {
            self.frames += 1;
        } else {
            self.failed = true;
        }
    }

    /// Collect the frame still in flight and close the sink. Returns the frames written, or the writer's error.
    pub fn finish(mut self, r: &Renderer) -> std::io::Result<u64> {
        for k in 0..2 {
            let s = (self.next + k) % 2;
            if s < self.slots.len() && self.slots[s].inflight.is_some() {
                self.collect(r, s);
            }
        }
        drop(self.tx.take());
        match self.writer.take().map(|w| w.join()) {
            Some(Ok(Err(e))) => Err(e),
            Some(Err(_)) => Err(std::io::Error::other("video writer thread panicked")),
            _ => Ok(self.frames),
        }
    }
}

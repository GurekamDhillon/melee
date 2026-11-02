//! Swapchain + present (platform step 3, first part): melee_gx's renderer on a winit window.
//!
//! The GX draws into its 640x480 EFB (offscreen, as on the GameCube and in Aurora); a frame the
//! game finished with GXCopyDisp ([`melee_gx::shim::GxShim::take_frame`]) is rendered there and
//! then scaled onto the window's swapchain image ([`melee_gx::render::Renderer::blit_to`]).
//! A frame with no new copy re-presents the last EFB (the VI shows the last XFB).
use melee_gx::draw::DrawCmd;
use melee_gx::render::Renderer;
use melee_gx::state::GxState;
use std::sync::Arc;
use winit::window::Window;

/// EFB size (GXNtsc480*: fbWidth x efbHeight).
pub const EFB_WIDTH: u32 = 640;
pub const EFB_HEIGHT: u32 = 480;

pub struct Presenter {
    pub renderer: Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// Frames presented / swapchain images that could not be acquired (skipped).
    pub presented: u64,
    pub skipped: u64,
    /// The internal scale follows the window (no `MELEE_RENDER_SCALE`).
    auto_scale: bool,
    /// The last two sim frames (render-only interpolation: never fed back into the game).
    prev: Option<Vec<DrawCmd>>,
    cur: Option<Vec<DrawCmd>>,
    /// `cur` is the renderer's prepared frame (geometry uploaded, pipelines looked up).
    prepared: bool,
    /// The EFB holds a blend, not `cur` itself.
    blended: bool,
}

/// `MELEE_VSYNC=off|on` (default on).
pub fn vsync_from_env() -> bool {
    !matches!(std::env::var("MELEE_VSYNC").as_deref().map(str::to_ascii_lowercase).as_deref(), Ok("off" | "0" | "false" | "no"))
}

/// Byte range of one draw's uniform block that holds the projection and the position matrices,
/// and the range of the normal matrices (see `shader::fill_uniform`: header, [line], proj, pos x10, tex x10, nrm x10).
fn mtx_ranges(d: &DrawCmd) -> [(usize, usize); 2] {
    let base = 16 + if d.info.line_mode != 0 { 16 } else { 0 };
    let pos_end = base + 64 + 480;
    let nrm = pos_end + 480;
    [(base, pos_end), (nrm, nrm + 480)]
}

/// Blend `prev` -> `cur` matrices at `t` into `out` (cur's uniform block, already copied). Only structurally
/// identical draws blend; anything else keeps the current frame's values.
fn blend_into(prev: &DrawCmd, cur: &DrawCmd, t: f32, out: &mut [u8]) {
    if prev.prim != cur.prim || prev.fmt != cur.fmt || prev.vtx_count != cur.vtx_count || prev.current_pn_mtx != cur.current_pn_mtx
        || prev.uniform.len() != cur.uniform.len() || prev.info.line_mode != cur.info.line_mode
    {
        return;
    }
    let rd = |b: &[u8], o: usize| f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    let [(a0, a1), (n0, n1)] = mtx_ranges(cur);
    if n1 > cur.uniform.len() {
        return;
    }
    // A projection-type change (ortho <-> perspective: proj[3][3]) or a cut: keep the projection.
    let p33 = a0 + 60;
    let mut start = a0;
    if rd(&prev.uniform, p33) != rd(&cur.uniform, p33) {
        start = a0 + 64;
    }
    // Without a per-vertex matrix index the draw reads only its current matrix slot (GXSetCurrentMtx: id / 3).
    let slot = cur.current_pn_mtx as usize / 3;
    let skinned = cur.config.shader.attrs[0].attr_type != melee_gx::consts::GX_NONE as u8;
    let ranges: [(usize, usize); 3] = if skinned || slot >= 10 {
        [(start, a1), (n0, n1), (0, 0)]
    } else {
        let pos = a0 + 64 + slot * 48;
        [(start, a0 + 64), (pos, pos + 48), (n0 + slot * 48, n0 + slot * 48 + 48)]
    };
    for (lo, hi) in ranges {
        let mut o = lo;
        while o < hi {
            let (p, c) = (rd(&prev.uniform, o), rd(&cur.uniform, o));
            let v = if p != c { p + (c - p) * t } else { c };
            out[o..o + 4].copy_from_slice(&(if v.is_finite() { v } else { c }).to_le_bytes());
            o += 4;
        }
    }
}

impl Presenter {
    /// A surface on `window` and a renderer on an adapter that can present to it.
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = Renderer::instance();
        let size = window.inner_size();
        let surface = instance.create_surface(window).map_err(|e| format!("create_surface: {e}"))?;
        let mut renderer = Renderer::with_instance(&instance, Some(&surface), EFB_WIDTH, EFB_HEIGHT)?;
        // Render the 3D scene at the window's resolution (the EFB's 4:3 coordinates scaled), not at 640x480 upscaled.
        let auto_scale = Renderer::env_scale().is_none();
        if auto_scale {
            renderer.set_scale(Renderer::scale_for_window(size.width, size.height));
        }
        let caps = surface.get_capabilities(&renderer.adapter);
        // The EFB holds display-space values already (no sRGB decode on the GC), so prefer a
        // non-sRGB swapchain format; the blit copies the bytes through.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .ok_or("surface reports no formats")?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: if vsync_from_env() {
                wgpu::PresentMode::AutoVsync
            } else if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
                wgpu::PresentMode::Immediate
            } else if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&renderer.device, &config);
        crate::plog!("gw: video: present mode {:?}", config.present_mode);
        Ok(Self { renderer, surface, config, auto_scale, presented: 0, skipped: 0, prev: None, cur: None, prepared: false, blended: false })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return; // minimised: keep the old configuration until it comes back
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.renderer.device, &self.config);
        if self.auto_scale {
            self.renderer.set_scale(Renderer::scale_for_window(width, height));
            self.blended = true; // the targets were recreated: re-render the current frame
        }
    }

    /// Render `frame` (if the game finished one) into the EFB, then present the EFB.
    pub fn present(&mut self, state: &GxState, frame: Option<&[DrawCmd]>) {
        if let Some(draws) = frame {
            self.renderer.render_frame(state, draws);
            self.prepared = false;
        }
        self.swap_present();
    }

    /// Decoupled presentation: a new sim frame (if any) becomes current, the last becomes previous.
    /// `alpha = Some(t)` renders the previous->current blend of the camera / model matrices at `t`
    /// (one tick of latency); `None` renders the newest frame as is (only when it is new).
    /// Everything here is render-only: the frames are copies the game no longer reads.
    pub fn present_frames(&mut self, state: &GxState, frame: Option<Vec<DrawCmd>>, alpha: Option<f64>) {
        let fresh = frame.is_some();
        if let Some(f) = frame {
            self.prev = self.cur.replace(f);
            self.prepared = false;
        }
        let Some(cur) = self.cur.as_ref() else {
            self.swap_present();
            return;
        };
        let blend = match (alpha, self.prev.as_ref()) {
            (Some(t), Some(prev)) if prev.len() == cur.len() => Some((t.clamp(0.0, 1.0) as f32, prev)),
            _ => None,
        };
        if blend.is_some() || fresh || self.blended {
            if !self.prepared {
                self.renderer.prepare(state, cur);
                self.prepared = true;
            }
            match blend {
                Some((t, prev)) => {
                    let mut f = |i: usize, out: &mut [u8]| blend_into(&prev[i], &cur[i], t, out);
                    self.renderer.draw_prepared(state, cur, Some(&mut f));
                    self.blended = true;
                }
                None => {
                    self.renderer.draw_prepared(state, cur, None);
                    self.blended = false;
                }
            }
        }
        self.swap_present();
    }

    fn swap_present(&mut self) {
        let tex = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.renderer.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(t) => t,
                    Err(_) => {
                        self.skipped += 1;
                        return;
                    }
                }
            }
            Err(_) => {
                self.skipped += 1;
                return;
            }
        };
        let view = tex.texture.create_view(&Default::default());
        self.renderer.blit_to(&view, self.config.format, self.config.width, self.config.height);
        tex.present();
        self.presented += 1;
    }

    /// The EFB as RGBA8 with alpha forced opaque (what the window shows, before scaling).
    pub fn screenshot(&self) -> Vec<u8> {
        let mut px = self.renderer.read_color();
        for p in px.chunks_mut(4) {
            p[3] = 255;
        }
        px
    }
}

/// Write RGBA8 pixels as a PNG.
pub fn write_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let f = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), width, height);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .and_then(|mut w| w.write_image_data(rgba))
        .map_err(|e| format!("{}: {e}", path.display()))
}

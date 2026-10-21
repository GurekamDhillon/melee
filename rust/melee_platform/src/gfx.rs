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
}

impl Presenter {
    /// A surface on `window` and a renderer on an adapter that can present to it.
    pub fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = Renderer::instance();
        let size = window.inner_size();
        let surface = instance.create_surface(window).map_err(|e| format!("create_surface: {e}"))?;
        let renderer = Renderer::with_instance(&instance, Some(&surface), EFB_WIDTH, EFB_HEIGHT)?;
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
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&renderer.device, &config);
        Ok(Self { renderer, surface, config, presented: 0, skipped: 0 })
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
    }

    /// Render `frame` (if the game finished one) into the EFB, then present the EFB.
    pub fn present(&mut self, state: &GxState, frame: Option<&[DrawCmd]>) {
        if let Some(draws) = frame {
            self.renderer.render_frame(state, draws);
        }
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
        self.renderer.blit_to(&view, self.config.format);
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

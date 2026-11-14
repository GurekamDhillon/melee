//! The effect gallery: renders shader effect quads (melee_gx::vfx) over a backdrop, frame by frame, to PNGs. A
//! development aid for tuning the effects without the game:
//!   VFX_SPEC=<spec file> VFX_OUT=<dir> [VFX_BG=<png>] [VFX_FRAMES=n] [VFX_SIZE=WxH] [MELEE_VFX_WGSL=<file>]
//!   cargo test -p melee_gx --release --test vfx_gallery -- --ignored --nocapture
//! Spec: one quad per line, `kind x y w h rot seed t p0 p1 p2 p3 col0 col1 [flags]` on the 640x480 canvas; a number
//! may be `a..b` (runs from a to b over the frames) and `t` may be `f` (frame / 60). `#` starts a comment.
use melee_gx::fx::FxMesh;
use melee_gx::render::Renderer;
use melee_gx::vfx::*;

fn num(tok: &str, u: f32, frame: u32) -> f32 {
    if tok == "f" {
        return frame as f32 / 60.0;
    }
    match tok.split_once("..") {
        Some((a, b)) => {
            let (a, b): (f32, f32) = (a.parse().unwrap(), b.parse().unwrap());
            a + (b - a) * u
        }
        None => tok.parse().unwrap_or_else(|_| panic!("bad number {tok}")),
    }
}

#[test]
#[ignore]
fn vfx_gallery() {
    let Ok(spec) = std::env::var("VFX_SPEC") else { return };
    let out = std::env::var("VFX_OUT").unwrap_or_else(|_| ".".into());
    let frames: u32 = std::env::var("VFX_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let (w, h) = std::env::var("VFX_SIZE").ok().and_then(|v| v.split_once('x').map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))).unwrap_or((640u32, 480u32));
    let Ok(mut r) = Renderer::new(w, h) else {
        eprintln!("SKIPPED: no GPU renderer");
        return;
    };
    let bg = std::env::var("VFX_BG").ok().map(|p| {
        let mut d = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&p).unwrap()));
        d.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
        let mut rd = d.read_info().unwrap();
        let mut buf = vec![0u8; rd.output_buffer_size().unwrap()];
        let info = rd.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (w, h), "the backdrop must be the render size");
        buf.truncate(info.buffer_size());
        buf
    });
    let tri = [[0.0f32; 3]; 3];
    r.fx_arm(&[FxMesh { pos: &tri, nrm: &tri, base: &tri, glow: &tri }]);
    let text = std::fs::read_to_string(&spec).unwrap();
    let black = vec![0u8; (w * h * 4) as usize].chunks(4).flat_map(|_| [10u8, 10, 16, 255]).collect::<Vec<u8>>();
    std::fs::create_dir_all(&out).unwrap();
    let t0 = std::time::Instant::now();
    for frame in 0..frames {
        let u = if frames > 1 { frame as f32 / (frames - 1) as f32 } else { 0.0 };
        let mut list = Vec::new();
        for line in text.lines() {
            let line = line.split('#').next().unwrap().trim();
            if line.is_empty() {
                continue;
            }
            let k: Vec<&str> = line.split_whitespace().collect();
            let n = |i: usize| num(k[i], u, frame);
            let hex = |i: usize| u32::from_str_radix(k[i].trim_start_matches("0x"), 16).unwrap();
            list.push(VfxInst {
                kind: k[0].parse().unwrap(),
                pos: [n(1), n(2), 0.0],
                size: [n(3), n(4)],
                rot: n(5),
                seed: n(6),
                t: n(7),
                p: [n(8), n(9), n(10), n(11)],
                col0: hex(12),
                col1: hex(13),
                flags: F_SCREEN | k.get(14).map_or(0, |f| f.parse().unwrap()),
            });
        }
        r.vfx_preview(Some(bg.as_deref().unwrap_or(&black)), &list, 1.0);
        let px = r.read_composed(&[]);
        let f = std::fs::File::create(format!("{out}/f{frame:03}.png")).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&px).unwrap();
    }
    eprintln!("vfx_gallery: {frames} frames in {:.1} ms each (render + readback + png)", t0.elapsed().as_secs_f64() * 1e3 / frames as f64);
}

/// The cost of the effect quads (ms per frame, GPU waited) with a full load of every kind on screen:
/// `cargo test -p melee_gx --release --test vfx_gallery vfx_cost -- --ignored --nocapture`.
#[test]
#[ignore]
fn vfx_cost() {
    let Ok(mut r) = Renderer::new(640, 480) else {
        eprintln!("SKIPPED: no GPU renderer");
        return;
    };
    let tri = [[0.0f32; 3]; 3];
    // a stacked fight: every item's projectiles and bursts at once (about 8 quads of each kind, fight-sized)
    let mut list = Vec::new();
    for j in 0..8u32 {
        for kind in 1..K_COUNT {
            let big = matches!(kind, K_SHOCK | K_CIRCLE | K_DOME | K_SPARKS);
            let s = if big { 130.0 } else { 60.0 };
            list.push(VfxInst {
                kind,
                pos: [40.0 + (kind * 29 % 560) as f32, 60.0 + (j * 47 + kind * 13) as f32 % 360.0, 0.0],
                size: [s, if kind == K_BOLT || kind == K_TRAIL { 24.0 } else { s }],
                rot: 0.0,
                seed: j as f32,
                t: 0.4 + j as f32 * 0.1,
                p: [0.5, 0.5, 0.5, 1.0],
                col0: 0xFFE1_4AFF,
                col1: 0xFF7A_20FF,
                flags: F_SCREEN | if matches!(kind, K_SHOCK | K_DOME | K_HAZE) { F_DISTORT } else { 0 },
            });
        }
    }
    for (scale, (ow, oh)) in [(2u32, (1280u32, 720u32)), (3, (1920, 1080)), (5, (3840, 2160))] {
        r.fx_disarm();
        r.set_target(scale, 4.0 / 3.0);
        r.fx_arm(&[FxMesh { pos: &tri, nrm: &tri, base: &tri, glow: &tri }]);
        let target = r.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: ow, height: oh, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let mut time = |r: &mut Renderer, l: &[VfxInst]| {
            for _ in 0..10 {
                r.vfx_preview(None, l, 1.0);
                r.blit_to(&view, wgpu::TextureFormat::Rgba8Unorm, ow, oh);
            }
            let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
            let t = std::time::Instant::now();
            for _ in 0..100 {
                r.vfx_preview(None, l, 1.0);
                r.blit_to(&view, wgpu::TextureFormat::Rgba8Unorm, ow, oh);
                let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
            }
            t.elapsed().as_secs_f64() * 1e3 / 100.0
        };
        let none = time(&mut r, &[]);
        let all = time(&mut r, &list);
        eprintln!("vfx cost at internal {}x{} -> output {ow}x{oh}: {} quads {:.3} ms a frame (world pass + bloom + refraction + blit; {none:.3} ms with none: +{:.3})", r.width, r.height, list.len(), all, all - none);
    }
}

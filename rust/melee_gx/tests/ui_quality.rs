//! uiquality: the UI renderer measured on the real GPU pipeline (the shader, the atlas, the blend), offscreen.
//!
//! Known primitives are drawn white on black at the band scales of 720p, 1080p, 1440p and 4K (1.5, 2.25, 3, 4.5 device
//! pixels per canvas unit) and read back, so a pixel's value is its coverage. The tests then check numbers:
//!   * circles are circles: the covered area is pi r^2, width equals height, the centroid sits on the centre and every
//!     edge pixel's coverage agrees with a circle of that radius to a fraction of a pixel;
//!   * a small pill is symmetric left-right and top-bottom (nothing squished or lumpy);
//!   * shapes have no seams: no partly covered pixel between fully covered ones, untilted, rotated and tilted, and a
//!     bar of abutting flat pieces (the only multi-quad shape left) holds together the same way;
//!   * a minified sprite (a 128-texel disc, like a 4x-baked 32-unit sticker, drawn down to 18 px: 7x smaller, the
//!     mip chain covers 8x) is still round.
//! Skipped (passes with a note) when the machine has no GPU adapter.

use melee_gx::ui::*;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    ui: UiGpu,
    atlas: Atlas,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    let ui = UiGpu::new(&device);
    Some(Gpu { device, queue, ui, atlas: Atlas::new() })
}

/// Draw `quads` on a black `w` x `h` target whose band is the whole target scaled `s`; returns the red channel.
fn render(g: &mut Gpu, quads: Vec<UiQuad>, w: u32, h: u32, band: [f32; 4], tilt: [f32; 2]) -> Vec<u8> {
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let tex = g.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: fmt,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&Default::default());
    let frame = UiFrame { quads, hide_gx: false };
    g.ui.upload(&g.device, &g.queue, &frame, &mut g.atlas);
    let mut enc = g.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        g.ui.record(&g.device, &g.queue, &mut pass, fmt, w as f32, h as f32, band, tilt, 0.0);
    }
    let bpr = (w * 4).next_multiple_of(256);
    let buf = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (bpr * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    g.queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            out.push(data[(y * bpr + x * 4) as usize]);
        }
    }
    out
}

fn quad(x: f32, y: f32, w: f32, h: f32) -> UiQuad {
    UiQuad { rect: [x, y, w, h], clip: NO_CLIP, rgba: 0xFFFF_FFFF, ..Default::default() }
}

fn shape(kind: u32, x: f32, y: f32, w: f32, h: f32, radius: f32, border: f32) -> UiQuad {
    let mut q = quad(x, y, w, h);
    q.flags = kind << Q_SHAPE_SHIFT;
    q.shape = [radius, border, 0.0];
    if border > 0.0 {
        q.rgba = 0xFFFF_FF00;
        q.rgba2 = 0xFFFF_FFFF;
    }
    q
}

const SCALES: [f32; 4] = [1.5, 2.25, 3.0, 4.5];

/// Circle measurements inside the window (x0, y0, x1, y1) of `img`: (area, width, height, centroid x, centroid y,
/// worst radial error of an edge pixel against centre (cx, cy) and radius r).
fn measure(img: &[u8], w: u32, win: [u32; 4], cx: f32, cy: f32, r: f32) -> (f32, f32, f32, f32, f32, f32) {
    let (mut area, mut sx, mut sy, mut worst) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let (mut colmax, mut rowmax) = (0.0f32, 0.0f32);
    let mut cols = vec![0.0f32; (win[2] - win[0]) as usize];
    for y in win[1]..win[3] {
        let mut row = 0.0;
        for x in win[0]..win[2] {
            let c = img[(y * w + x) as usize] as f32 / 255.0;
            area += c;
            sx += c * (x as f32 + 0.5);
            sy += c * (y as f32 + 0.5);
            row += c;
            cols[(x - win[0]) as usize] += c;
            if c > 0.1 && c < 0.9 {
                // an edge pixel: a circle's coverage there is about 0.5 + (r - distance)
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                worst = worst.max((d + (c - 0.5) - r).abs());
            }
        }
        rowmax = rowmax.max(row);
    }
    for c in cols {
        colmax = colmax.max(c);
    }
    (area, rowmax, colmax, sx / area, sy / area, worst)
}

#[test]
fn circles_are_circles() {
    let Some(mut g) = gpu() else {
        eprintln!("ui_quality: no GPU adapter, skipped");
        return;
    };
    let mut report = String::new();
    for s in SCALES {
        // diameters in canvas units, at awkward fractional positions; the band is offset by a fractional letterbox
        let (w, h) = ((640.0 * s) as u32, (120.0 * s) as u32);
        let band = [0.0, 0.0, 640.0 * s, 480.0 * s];
        let ds = [2.0f32, 3.0, 4.5, 6.0, 10.0, 16.0, 24.0, 32.0];
        let mut quads = Vec::new();
        let mut at = Vec::new();
        let mut x = 10.3f32;
        for (i, d) in ds.iter().enumerate() {
            let y = 40.0 + 0.37 * i as f32;
            quads.push(shape(SHAPE_BOX, x, y, *d, *d, *d, 0.0));
            // a ring of the same size below it
            let mut ring = shape(SHAPE_BOX, x, y + 40.0, *d, *d, *d, (d * 0.2).max(0.5));
            ring.rgba = 0;
            quads.push(ring);
            at.push((x + d * 0.5, y + d * 0.5, *d));
            x += d + 12.7;
        }
        let img = render(&mut g, quads, w, h, band, [0.0; 2]);
        for (cx, cy, d) in at {
            let (pcx, pcy, r) = (cx * s, cy * s, d * s * 0.5);
            let m = (r + 3.0).ceil();
            let win = [(pcx - m) as u32, (pcy - m) as u32, (pcx + m) as u32 + 1, (pcy + m) as u32 + 1];
            let (area, wd, ht, gx, gy, worst) = measure(&img, w, win, pcx, pcy, r);
            let ideal = std::f32::consts::PI * r * r;
            report += &format!("scale {s} d {d:>4} ({:>5.1} px): area {:+.2}% w/h {:.3} centroid off ({:+.2},{:+.2}) worst edge {:.2} px\n", 2.0 * r, (area / ideal - 1.0) * 100.0, wd / ht, gx - pcx, gy - pcy, worst);
            // tolerances: a dot under 6 px across is a handful of pixels (its measured width depends on where the
            // pixel grid falls), so it is held to 10% area and 5% aspect; anything larger to 2% and 2%
            let (tol, asp) = if r < 3.0 { (0.10, 0.05) } else { (0.02, 0.02) };
            assert!((area / ideal - 1.0).abs() < tol, "area off: scale {s} d {d}: {area} vs {ideal}\n{report}");
            assert!((wd / ht - 1.0).abs() < asp, "not round: scale {s} d {d}: {wd} x {ht}\n{report}");
            assert!((gx - pcx).abs() < 0.08 && (gy - pcy).abs() < 0.08, "off centre: scale {s} d {d}\n{report}");
            assert!(worst < 0.3, "edge off the circle by {worst} px: scale {s} d {d}\n{report}");
        }
    }
    println!("{report}");
}

#[test]
fn small_pills_are_symmetric() {
    let Some(mut g) = gpu() else { return };
    for s in SCALES {
        let (w, h) = ((200.0 * s) as u32, (60.0 * s) as u32);
        // the HUD's small stickers: 12 to 18 units high, radius far above half the height, with a border
        for (i, (pw, ph)) in [(22.0f32, 12.0f32), (57.0, 13.0), (30.0, 18.0), (14.0, 9.0)].into_iter().enumerate() {
            let (x, y) = (7.3 + i as f32 * 0.21, 9.6);
            let mut q = shape(SHAPE_BOX, x, y, pw, ph, 11.5, 3.0);
            q.rgba = 0xFFFF_FFFF;
            q.rgba2 = 0xFFFF_FFFF;
            let img = render(&mut g, vec![q], w, h, [0.0, 0.0, 640.0 * s, 480.0 * s], [0.0; 2]);
            // the snapped box in pixels
            let (bx, by) = ((x * s + 0.5).floor() as i32, (y * s + 0.5).floor() as i32);
            let (bw, bh) = ((pw * s + 0.5).floor() as i32, (ph * s + 0.5).floor() as i32);
            let at = |px: i32, py: i32| img[(py as u32 * w + px as u32) as usize] as i32;
            let mut worst = 0;
            for yy in 0..bh {
                for xx in 0..bw {
                    let a = at(bx + xx, by + yy);
                    worst = worst.max((a - at(bx + bw - 1 - xx, by + yy)).abs()).max((a - at(bx + xx, by + bh - 1 - yy)).abs());
                }
            }
            assert!(worst <= 8, "pill {pw}x{ph} at scale {s} is lopsided by {worst}/255");
            // straight sides are crisp: the middle row and column are fully covered edge to edge, nothing outside
            assert!(at(bx, by + bh / 2) >= 235 && at(bx + bw - 1, by + bh / 2) >= 235);
            assert_eq!(at(bx - 1, by + bh / 2), 0);
            assert_eq!(at(bx + bw, by + bh / 2), 0);
        }
    }
}

/// Seam pixels: partly covered pixels with fully covered pixels on both opposite sides within `reach`.
fn seams(img: &[u8], w: u32, h: u32, reach: i32) -> u32 {
    let at = |x: i32, y: i32| if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { 0 } else { img[(y as u32 * w + x as u32) as usize] };
    let mut n = 0;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if at(x, y) >= 250 {
                continue;
            }
            let full = |dx: i32, dy: i32| (1..=reach).any(|k| at(x + dx * k, y + dy * k) >= 250);
            if (full(-1, 0) && full(1, 0)) || (full(0, -1) && full(0, 1)) {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn no_seams_rotated_or_tilted() {
    let Some(mut g) = gpu() else { return };
    for s in SCALES {
        let (w, h) = ((640.0 * s) as u32, (480.0 * s) as u32);
        let band = [0.0, 0.0, 640.0 * s, 480.0 * s];
        for (rot, tilt) in [(0.0f32, [0.0f32; 2]), (-0.0105, [0.0; 2]), (0.14, [0.0; 2]), (0.0, [0.35, -0.2]), (-0.0105, [-0.3, 0.25])] {
            let mut quads = Vec::new();
            // a panel, a small pill and a sticker (shadow under face: two overlapping shapes)
            for (x, y, pw, ph, r) in [(60.3f32, 50.7f32, 220.0f32, 160.0f32, 14.0f32), (330.2, 60.1, 57.0, 13.0, 11.5), (330.0, 120.0, 150.0, 90.0, 10.0), (333.0, 124.0, 150.0, 90.0, 10.0)] {
                let mut q = shape(SHAPE_BOX, x, y, pw, ph, r, 0.0);
                q.rot = rot;
                q.piv = [x + pw * 0.5, y + ph * 0.5];
                quads.push(q);
            }
            // a bar of three abutting flat pieces (white texel), marked as one shape the way the host marks them
            let uv = g.atlas.white.uv();
            for i in 0..3u32 {
                let mut q = quad(100.0 + 37.3 * i as f32, 300.0, 37.3, 21.7);
                q.uv = uv;
                q.rot = rot;
                q.piv = [155.0, 310.0];
                q.flags = (if i > 0 { Q_IN_L } else { 0 }) | (if i < 2 { Q_IN_R } else { 0 }) | (i << 8);
                quads.push(q);
            }
            let img = render(&mut g, quads, w, h, band, tilt);
            let n = seams(&img, w, h, 3);
            assert_eq!(n, 0, "{n} seam pixels at scale {s}, rot {rot}, tilt {tilt:?}");
            assert!(img.iter().filter(|&&v| v >= 250).count() > 1000, "nothing drawn at scale {s}");
        }
    }
}

#[test]
fn minified_sprite_stays_round() {
    let Some(mut g) = gpu() else { return };
    // a 128-texel disc (a 4x-baked 32-unit sticker) on the atlas
    let n = 128u16;
    let r = g.atlas.alloc(n, n).unwrap();
    let mut px = vec![0u8; n as usize * n as usize * 4];
    for y in 0..n as usize {
        for x in 0..n as usize {
            let d = ((x as f32 + 0.5 - 64.0).powi(2) + (y as f32 + 0.5 - 64.0).powi(2)).sqrt();
            let a = ((60.0 - d + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            px[(y * n as usize + x) * 4..][..4].copy_from_slice(&[255, 255, 255, a]);
        }
    }
    g.atlas.put(r, &px);
    for (s, d) in [(1.5f32, 12.0f32), (2.25, 12.0), (1.5, 19.0), (1.5, 32.0), (2.25, 32.0), (4.5, 12.0), (4.5, 32.0), (3.0, 40.0)] {
        let (w, h) = (320u32, 320u32);
        let mut q = quad(20.3, 20.6, d, d);
        q.uv = r.uv();
        q.flags = Q_FULL;
        let img = render(&mut g, vec![q], w, h, [0.0, 0.0, 640.0 * s, 480.0 * s], [0.0; 2]);
        let (pcx, pcy, rad) = ((20.3 + d * 0.5) * s, (20.6 + d * 0.5) * s, d * s * 0.5 * 120.0 / 128.0);
        let (area, wd, ht, gx, gy, _) = measure(&img, w, [0, 0, w, h], pcx, pcy, rad);
        let ideal = std::f32::consts::PI * rad * rad;
        assert!((area / ideal - 1.0).abs() < 0.05, "sprite disc area off at scale {s} d {d}: {area} vs {ideal}");
        assert!((wd / ht - 1.0).abs() < 0.03, "sprite disc not round at scale {s} d {d}: {wd} x {ht}");
        assert!((gx - pcx).abs() < 0.15 && (gy - pcy).abs() < 0.15, "sprite disc off centre at scale {s} d {d}");
    }
}

/// Small text keeps its stems wherever it sits on the pixel grid: the ink of a thin-stemmed word drawn 9 to 14 px
/// high must not depend on its sub-pixel position (a stem between two pixel centres is two half-covered pixels, not
/// nothing).
#[test]
fn small_text_keeps_its_stems() {
    use melee_gx::ui_font::{emit, fit, TextStyle};
    let Some(mut g) = gpu() else { return };
    let mut report = String::new();
    for (s, size) in [(1.5f32, 7.0f32), (1.5, 9.0), (2.25, 6.0), (1.0, 11.0), (3.0, 5.0)] {
        let st = TextStyle::new(size, false, false);
        let word = "Illtuj1";
        let (n, _) = fit(&g.atlas.faces, st, word, 0.0);
        let mut inks = Vec::new();
        for k in 0..16 {
            // sixteen sub-pixel positions across one pixel
            let x = 8.0 + k as f32 / 16.0 / s;
            let mut quads = Vec::new();
            emit(&g.atlas.faces, st, word, n, x, 6.3, |r, uv| {
                let mut q = quad(r[0], r[1], r[2], r[3]);
                q.uv = uv;
                q.flags = Q_SDF;
                quads.push(q);
            });
            let (w, h) = ((120.0 * s) as u32, (30.0 * s) as u32);
            let img = render(&mut g, quads, w, h, [0.0, 0.0, 640.0 * s, 480.0 * s], [0.0; 2]);
            inks.push(img.iter().map(|&v| v as f32 / 255.0).sum::<f32>());
        }
        let mean = inks.iter().sum::<f32>() / inks.len() as f32;
        let (lo, hi) = inks.iter().fold((f32::MAX, 0.0f32), |a, &v| (a.0.min(v), a.1.max(v)));
        report += &format!("scale {s} size {size} ({:.1} px): ink {mean:.1} px, min {:+.1}% max {:+.1}%\n", size * s, (lo / mean - 1.0) * 100.0, (hi / mean - 1.0) * 100.0);
        assert!(mean > 10.0, "no text drawn at scale {s} size {size}");
        // 4%: the fwidth-normalised edge this replaced swung up to 9.6% here; the range-normalised one stays within 2.3%
        assert!(lo / mean > 0.96 && hi / mean < 1.04, "ink depends on the sub-pixel position:\n{report}");
    }
    println!("{report}");
}

/// A sticker that fades is one translucent piece: face and shadow are composed in the shader before the blend, so
/// at half opacity every pixel inside the face-or-shadow outline has the same value (two overlapping translucent
/// quads would be darker, or lighter, where they overlap).
#[test]
fn fading_sticker_is_one_piece() {
    let Some(mut g) = gpu() else { return };
    for s in SCALES {
        let (w, h) = ((200.0 * s) as u32, (120.0 * s) as u32);
        // white face, white edge and white shadow at half opacity on black: the union must read 127 or 128 all over
        let (x, y, pw, ph) = (20.0f32, 20.0f32, 120.0f32, 60.0f32);
        let mut q = shape(SHAPE_BOX, x, y, pw, ph, 11.5, 3.0);
        q.rgba = 0xFFFF_FF80;
        q.rgba2 = 0xFFFF_FF80;
        q.uv = [3.0, 4.0, 1.5, 0xFFFFFF as f32];
        let img = render(&mut g, vec![q], w, h, [0.0, 0.0, 640.0 * s, 480.0 * s], [0.0; 2]);
        let at = |ux: f32, uy: f32| img[((uy * s) as u32 * w + (ux * s) as u32) as usize];
        // the face alone, the overlap, and the shadow alone (right of and below the face)
        for (ux, uy, what) in [(30.0, 28.0, "face"), (80.0, 50.0, "overlap"), (140.8, 60.0, "shadow right"), (90.0, 81.5, "shadow below")] {
            let v = at(ux, uy);
            assert!((127..=129).contains(&v), "{what} at scale {s}: {v}, expected 128");
        }
        // nothing past the shadow
        assert_eq!(at(150.0, 60.0), 0);
        assert_eq!(at(90.0, 90.0), 0);
    }
}

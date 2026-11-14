//! Shader effects (docs/rust-port/notes/envoy-vfx.md): the one way Envoy draws an effect. Each effect is a quad the
//! host shades with a built-in procedural shader (`melee_gx::vfx`): lightning, magic circles, shockwaves that bend
//! the picture, fire, ice, sparks, trails, orbs. A quad is a kind, a place (the world, or the 640x480 canvas), an
//! age, a seed, up to four numbers and two colours; build one with [`fx`], shape it with the builder calls and
//! [`Fx::emit`] it. Everything queued in a present goes to the host in one call ([`flush`]).
//!
//! Presentation only: nothing here is sim state and nothing is read back, so effects can never change a rollback.
//! Every quad passes through the player's settings (pause overlay, Options): the effect intensity scales its
//! opacity (Off: nothing is queued) and with screen shake off nothing bends the picture.
//!
//! New item? Compose its look from the recipes at the bottom (`burst_ring`, `bolt`, `sparks`, ...) or from raw
//! kinds; keep one distinct silhouette and colour pair per item, and give its hit a `sparks` / `impact`.

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_vfx_batch(ptr: *const Fx, count: u32) -> i32;
}

// ---- kinds (melee_gx::vfx::K_*). The four numbers of each are listed beside it.
/// A soft light. p0 falloff (0: 4), p1 hot core 0..1.
pub const GLOW: u32 = 1;
/// A ring running out. p0 progress 0..1, p1 width (0: 0.08), p2 electric crackle 0..1, p3 refraction (with `.bend()`).
pub const SHOCK: u32 = 2;
/// Lightning along the quad's x axis (use [`Fx::line`]). p0 jaggedness (0: 1), p1 branches 0..1, p2 redraws per
/// second (0: 24), p3 thickness (0: 1).
pub const BOLT: u32 = 3;
/// A magic circle. p0 drawn-in 0..1, p1 cracks 0..1, p2 charge 0..1, p3 spin (0: 0.5).
pub const CIRCLE: u32 = 4;
/// Fire standing on the quad's bottom edge (a wide quad is a wall of fire). p0 height (0: 1), p1 lean, p2 heat 0..1.
pub const FLAME: u32 = 5;
/// Sparks drifting up through the quad. p0 density (0: 6), p1 speed (0: 1).
pub const EMBERS: u32 = 6;
/// Crystal shards growing from the quad's bottom middle. p0 growth 0..1, p1 shatter 0..1.
pub const ICE: u32 = 7;
/// A comet streak, head at +x (use [`Fx::line`]: tail to head). p0 wobble 0..1, p1 head size (0: 1).
pub const TRAIL: u32 = 8;
/// An impact burst of streaks. p0 progress 0..1, p1 count (0: 26), p2 gravity (0 none; -1: 1), p3 spread (0: all
/// round; 0.25: a quarter cone about the quad's +x).
pub const SPARKS: u32 = 9;
/// A four-point sparkle. p0 sharpness (0: 1), p1 spin (radians a second), p2 second cross 0..1.
pub const STAR: u32 = 10;
/// A spectre. p0 facing (-1 / 1), p1 strike flash 0..1.
pub const WISP: u32 = 11;
/// A crescent cut, opening about the quad's +x. p0 progress 0..1, p1 arc span in turns (0: 0.42), p2 thickness.
pub const SLASH: u32 = 12;
/// The ground split along the quad's x axis. p0 spread 0..1, p1 glow 0..1, p2 dust 0..1.
pub const CRACK: u32 = 13;
/// A pillar of light on the quad's bottom edge. p0 core 0..1, p1 motes 0..1.
pub const BEAM: u32 = 14;
/// An energy ball. p0 plasma 0..1, p1 crescent 0..1 (a moon), p2 pulse rate.
pub const ORB: u32 = 15;
/// A hexagonal shield flaring out. p0 progress 0..1, p1 cells across (0: 5), p3 refraction (with `.bend()`).
pub const DOME: u32 = 16;
/// A spiked charge. p0 light 0..1, p1 warning ring 0..1, p2 armed 0..1.
pub const MINE: u32 = 17;
/// A turning coin. p0 turns a second (0: 1.5), p1 sparkle 0..1.
pub const COIN: u32 = 18;
/// Heat shimmer: bends the picture only (always with `.bend()`). p0 strength (0: 1).
pub const HAZE: u32 = 19;
/// Flames round a fighter, hollow in the middle. p0 strength 0..1.
pub const AURA: u32 = 20;
/// A thin standing ring. p0 radius (0: 0.8), p1 line width (0: 0.035), p2 dashes (0: whole), p3 turn rate.
pub const RING: u32 = 21;

const F_SCREEN: u32 = 1;
const F_NO_DEPTH: u32 = 2;
const F_DISTORT: u32 = 4;
const F_LINE: u32 = 8;

/// Depths in the world: behind the fighters (auras, circles, pillars) and in front of them (hits, bolts, sparks).
pub const BACK: f32 = -9.0;
pub const FRONT: f32 = 9.0;

/// One effect quad (the host's `VfxInst`, 64 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Fx {
    pos: [f32; 3],
    rot: f32,
    size: [f32; 2],
    t: f32,
    seed: f32,
    p: [f32; 4],
    col0: u32,
    col1: u32,
    kind: u32,
    flags: u32,
}

const CAP: usize = 192;
const ZERO: Fx = Fx { pos: [0.0; 3], rot: 0.0, size: [0.0; 2], t: 0.0, seed: 0.0, p: [0.0; 4], col0: 0, col1: 0, kind: 0, flags: 0 };
static mut BUF: [Fx; CAP] = [ZERO; CAP];
static mut N: usize = 0;

/// A quad of `kind` centred at (x, y) in the world, `w` wide and `h` tall, in front of the fighters, white.
pub fn fx(kind: u32, x: f32, y: f32, w: f32, h: f32) -> Fx {
    Fx { pos: [x, y, FRONT], size: [w, h], col0: 0xFFFF_FFFF, col1: 0xFFFF_FFFF, kind, ..ZERO }
}

impl Fx {
    /// The main colour (its alpha: the whole effect's opacity) and the outer / second colour.
    pub fn cols(mut self, c0: u32, c1: u32) -> Fx {
        self.col0 = c0;
        self.col1 = c1;
        self
    }
    /// Opacity 0..1 (multiplies the main colour's alpha).
    pub fn a(mut self, a: f32) -> Fx {
        let al = ((self.col0 & 0xFF) as f32 * a.clamp(0.0, 1.0)) as u32;
        self.col0 = self.col0 & !0xFF | al;
        self
    }
    /// The kind's four numbers.
    pub fn p(mut self, p0: f32, p1: f32, p2: f32, p3: f32) -> Fx {
        self.p = [p0, p1, p2, p3];
        self
    }
    /// The age in frames (60 a second); looping effects take any running count.
    pub fn age(mut self, frames: f32) -> Fx {
        self.t = frames / 60.0;
        self
    }
    pub fn seed(mut self, s: f32) -> Fx {
        self.seed = s;
        self
    }
    /// Turned by `r` radians, counter-clockwise.
    pub fn rot(mut self, r: f32) -> Fx {
        self.rot = r;
        self
    }
    /// Behind the fighters.
    pub fn back(mut self) -> Fx {
        self.pos[2] = BACK;
        self
    }
    pub fn z(mut self, z: f32) -> Fx {
        self.pos[2] = z;
        self
    }
    /// Over everything in the scene, whatever its depth.
    pub fn over(mut self) -> Fx {
        self.flags |= F_NO_DEPTH;
        self
    }
    /// On the 640x480 canvas (x right, y down), not in the world.
    pub fn screen(mut self) -> Fx {
        self.flags |= F_SCREEN;
        self
    }
    /// Also bends the picture behind it (SHOCK, DOME, HAZE).
    pub fn bend(mut self) -> Fx {
        self.flags |= F_DISTORT;
        self
    }
    /// A segment from (x, y) to (x1, y1), `width` across: BOLT strikes at (x1, y1); TRAIL's head is at (x1, y1).
    pub fn line(mut self, x1: f32, y1: f32, width: f32) -> Fx {
        self.size = [x1, y1];
        self.rot = width;
        self.flags |= F_LINE;
        self
    }
    /// Queue it for this present, through the player's settings.
    pub fn emit(mut self) {
        let k = crate::pause::fx_scale();
        if k <= 0.0 || self.col0 & 0xFF == 0 {
            return;
        }
        if !crate::pause::shake_on() {
            if self.kind == HAZE {
                return;
            }
            self.flags &= !F_DISTORT;
        }
        self = self.a(k);
        unsafe {
            if N >= CAP {
                flush();
            }
            (*core::ptr::addr_of_mut!(BUF))[N] = self;
            N += 1;
        }
    }
}

/// Hand everything queued to the host (once at the end of the present; `emit` also does it when the buffer fills).
pub fn flush() {
    unsafe {
        if N > 0 {
            gd_vfx_batch(core::ptr::addr_of!(BUF) as *const Fx, N as u32);
            N = 0;
        }
    }
}

fn mixc(c: u32, to: u32, k: f32) -> u32 {
    let k = k.clamp(0.0, 1.0);
    let ch = |s: u32| {
        let (a, b) = ((c >> s & 0xFF) as f32, (to >> s & 0xFF) as f32);
        ((a + (b - a) * k) as u32) << s
    };
    ch(24) | ch(16) | ch(8) | c & 0xFF
}
/// `c` lightened toward white by `k`.
pub fn pale(c: u32, k: f32) -> u32 {
    mixc(c, 0xFFFF_FFFF, k)
}
/// `c` darkened by `k`.
pub fn deep(c: u32, k: f32) -> u32 {
    mixc(c, 0x0000_00FF, k)
}

// ---- recipes: the pieces most effects are made of. `k` is the effect's progress 0..1 and `age` its age in frames.

/// A ring bursting out to radius `r` in colour `c` (crackling with `crackle`), bending the picture as it goes.
pub fn burst_ring(x: f32, y: f32, r: f32, k: f32, c: u32, crackle: f32, age: f32) {
    fx(SHOCK, x, y, r * 2.3, r * 2.3).cols(c, deep(c, 0.35)).p(k, 0.0, crackle, 1.0).age(age).seed(x + y).bend().emit();
}

/// A flash of light at a point, gone in `life` frames (local: this is what replaces a screen flash).
pub fn pop(x: f32, y: f32, r: f32, age: f32, life: f32, c: u32) {
    if age < life {
        let a = 1.0 - age / life;
        fx(GLOW, x, y, r * 2.0, r * 2.0).cols(c, c).p(3.5, 0.5, 0.0, 0.0).a(a * a * 0.8).emit();
    }
}

/// Sparks thrown from a hit: `n` streaks, all round.
pub fn sparks(x: f32, y: f32, r: f32, k: f32, n: f32, c: u32, seed: f32) {
    fx(SPARKS, x, y, r * 2.0, r * 2.0).cols(pale(c, 0.3), c).p(k, n, -1.0, 0.0).seed(seed).emit();
}

/// Lightning from (x0, y0) striking (x1, y1): a thick bolt with branches under a thin white one, and the strike's
/// light. `age` in frames; it flickers for `life` frames.
pub fn bolt(x0: f32, y0: f32, x1: f32, y1: f32, width: f32, age: f32, life: f32, c: u32, seed: f32) {
    if age >= life {
        return;
    }
    let a = 1.0 - (age / life) * (age / life);
    fx(BOLT, x0, y0, 0.0, 0.0).line(x1, y1, width).cols(pale(c, 0.25), deep(c, 0.3)).p(1.0, 1.0, 0.0, 1.0).age(age).seed(seed).a(a).emit();
    fx(BOLT, x0, y0, 0.0, 0.0).line(x1, y1, width * 0.6).cols(0xFFFF_FFFF, c).p(0.6, 0.0, 30.0, 0.7).age(age).seed(seed + 3.0).a(a * 0.8).emit();
    pop(x1, y1, width * 0.9, age, life, pale(c, 0.5));
}

/// A hit's feedback: a white-hot pop, a ring and sparks in `c` (every item's hit gets one).
pub fn impact(x: f32, y: f32, r: f32, age: f32, c: u32, seed: f32) {
    let k = (age / 18.0).min(1.0);
    if age < 18.0 {
        burst_ring(x, y, r, k, c, 0.0, age);
        sparks(x, y, r * 1.3, k, 18.0, c, seed);
    }
    pop(x, y, r * 0.8, age, 6.0, 0xFFFF_FFFF);
}

/// A magic circle lying on the ground (seen at the fight camera's angle): `draw` 0..1 draws it in, `cracks` 0..1
/// breaks it open.
pub fn ground_circle(x: f32, y: f32, r: f32, draw: f32, cracks: f32, charge: f32, age: f32, c: u32, seed: f32) {
    fx(CIRCLE, x, y, r * 2.0, r * 0.62).cols(c, deep(c, 0.4)).p(draw, cracks, charge, 0.5).age(age).seed(seed).back().emit();
}

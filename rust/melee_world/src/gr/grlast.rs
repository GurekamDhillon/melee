//! Final Destination (`src/melee/gr/grlast.c`): the swaying background gobj (map gobj 7,
//! `grLast_8021AC30` on init, `grLast_8021ADD0` each frame). Its RNG draws pick each new sway
//! speed, so they reach the frame hash through `seed`.
//!
//! Native shape: `gp->u.last` is [`LastBg`]. The JObj writes (`HSD_JObjSetScale` on joint 5 at
//! init, `HSD_JObjSetRotationX/Y(xC4 * xDC / xC8 * xDC)` each frame) and `grMaterial_801C94D8`
//! are drawing state: [`LastBg::rotation`] returns the two angles for whoever owns the JObj.
//! The `xE0` generator block (only set by the map state machine's phase 9, which needs 1800+
//! frames) is not ported: TODO(group2 grlast phase 9) panics if reached.

use melee_core::fp::fmadds;

const K_80: f32 = ((std::f64::consts::PI / 180.0) / 250.0) as f32;
const K_7C: f32 = ((std::f64::consts::PI / 180.0) / 1000.0) as f32;
const K_8C: f32 = ((std::f64::consts::PI / 180.0) / 20.0) as f32;
const K_90: f32 = (-(std::f64::consts::PI / 180.0) / 20.0) as f32;
const K_94: f32 = ((std::f64::consts::PI / 180.0) * 15.0) as f32;
const K_98: f32 = (-(std::f64::consts::PI / 180.0) * 15.0) as f32;
const K_9C: f32 = ((std::f64::consts::PI / 180.0) * 10.0) as f32;
const K_A0: f32 = (-(std::f64::consts::PI / 180.0) * 10.0) as f32;

/// `Ground::u.last` (GC: +0xC4..+0xE4 of `Ground`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LastBg {
    pub x_c4: f32,
    pub x_c8: f32,
    pub x_cc: f32,
    pub x_d0: f32,
    pub x_d4: f32,
    pub x_d8: f32,
    /// Sway strength (0 at start; the map's phases 9/10 ramp it).
    pub x_dc: f32,
    /// `xE0`: the phase-9 generator (`grLib_801C96F8`), not ported.
    pub x_e0: Option<u32>,
}

/// The ±1 of `HSD_Randi(2) ? +1.0F : -1.0F`.
fn rand_sign(randi2: i32) -> f32 {
    if randi2 != 0 {
        1.0
    } else {
        -1.0
    }
}

impl LastBg {
    /// `grLast_8021AC30` (the gobj's `on_init`). `randf` / `randi` are `HSD_Randf` /
    /// `HSD_Randi` in C call order.
    pub fn init(randf: &mut dyn FnMut() -> f32, randi: &mut dyn FnMut(i32) -> i32) -> Self {
        let mut s = LastBg::default();
        // grlast.c 455 / 457: HSD_Randf() * K80 + K7C, fused.
        s.x_d4 = fmadds(randf(), K_80, K_7C);
        s.x_d4 *= rand_sign(randi(2));
        s.x_d8 = fmadds(randf(), K_80, K_7C);
        s.x_d8 *= rand_sign(randi(2));
        s.x_dc = 0.0;
        s.x_e0 = None;
        s
    }

    /// `grLast_8021ADD0` (the gobj's proc), the state part.
    pub fn update(&mut self, randf: &mut dyn FnMut() -> f32) {
        self.x_cc += self.x_d4;
        self.x_cc *= 0.99;
        if self.x_cc > K_8C {
            self.x_cc = K_8C;
        } else if self.x_cc < K_90 {
            self.x_cc = K_90;
        }
        self.x_d0 += self.x_d8;
        self.x_d0 *= 0.99;
        if self.x_d0 > K_8C {
            self.x_d0 = K_8C;
        } else if self.x_d0 < K_90 {
            self.x_d0 = K_90;
        }
        self.x_c4 += self.x_cc;
        if self.x_c4 > K_94 {
            self.x_c4 = K_94;
            self.x_cc = -self.x_cc.abs();
            let r = randf();
            // 512: K80 * -randf - K7C, fused.
            self.x_d4 = fmadds(K_80, -r, -K_7C);
        } else if self.x_c4 < K_98 {
            self.x_c4 = K_98;
            self.x_cc = self.x_cc.abs();
            // 517.
            self.x_d4 = fmadds(K_80, randf(), K_7C);
        }
        self.x_c8 += self.x_d0;
        if self.x_c8 > K_9C {
            self.x_c8 = K_9C;
            self.x_d0 = -self.x_d0.abs();
            let r = randf();
            // 526.
            self.x_d8 = fmadds(K_80, -r, -K_7C);
        } else if self.x_c8 < K_A0 {
            self.x_c8 = K_A0;
            self.x_d0 = self.x_d0.abs();
            // 530.
            self.x_d8 = fmadds(K_80, randf(), K_7C);
        }
        if self.x_e0.is_some() {
            panic!("grLast_8021ADD0: xE0 generator AppSRT update (TODO(group2 grlast phase 9))");
        }
    }

    /// The JObj angles `grLast_8021ADD0` writes: (`rotate.x`, `rotate.y`).
    pub fn rotation(&self) -> (f32, f32) {
        (self.x_c4 * self.x_dc, self.x_c8 * self.x_dc)
    }
}

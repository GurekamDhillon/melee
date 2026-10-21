//! Line geometry from src/melee/mp/mplib.c: point remapping, the per-kind "project onto this
//! line chain" queries (mpLib_8004DD90_Floor .. mpLib_8004E684_RightWall), segment
//! intersections, the ledge-extended line ends (mpLib_8004ED5C), and the vertex setters plus
//! mpGetSpeed.
//!
//! FMA: every fused site below is one the shipping build fuses
//! (`python tools/rust-port/fma_sites.py -v --func <name> src/melee/mp/mplib.c`). f64 sites are a
//! true fma (`f64::mul_add`), f32 sites are `fmadds` (f64 multiply-add, one f32 rounding). Clang
//! fuses the LHS multiply of `a*b +/- c*d`, i.e. `fma(a, b, -(c*d))`.

use super::lines::GroundColl;
use super::types::*;
use crate::gekko::frsqrte;
use crate::vec::{ps_vec_normalize, Vec3};
use melee_core::fp::fmadds;

/// `sqrtf` from src/MSL/math_ppc.h (three Newton steps over frsqrte), with the shipping build's
/// contraction: `3.0 - g*g*x` is `fma(-(g*g), x, 3.0)`.
/// TODO(move-to-core): identical to melee_chara::pure::my_sqrtf.
pub fn sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut g = frsqrte(xd);
        for _ in 0..3 {
            g = 0.5 * g * (-(g * g)).mul_add(xd, 3.0);
        }
        (xd * g) as f32
    } else {
        x
    }
}

/// Result of the mpLib_8004xxxx line-chain projections (the C's out-params; the C writes none of
/// them when it returns -1, which is `None` here).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineHit {
    /// The line the point was projected onto (the C return value).
    pub line_id: i32,
    /// `y_out` (floor/ceiling: signed vertical distance) or `x_out` (walls: horizontal).
    pub dist: f32,
    /// `flags_out`: the MapLine's `lo_flags`.
    pub flags: u32,
    /// `normal_out`.
    pub normal: Vec3,
}

/// `mpRemap2d` (static): move point p from segment a to segment b. Returns (x, y).
#[allow(clippy::too_many_arguments)]
pub fn mp_remap2d(
    ax0: f32, ay0: f32, ax1: f32, ay1: f32, bx0: f32, by0: f32, bx1: f32, by1: f32, px: f32,
    py: f32,
) -> (f32, f32) {
    let dx = (ax1 - ax0) as f64;
    let dy = (ay1 - ay0) as f64;
    let f30 = px - ax0;
    let f29 = py - ay0;
    let dist2 = dy.mul_add(dy, dx * dx);
    if dist2.abs() > 0.0001 {
        let mut t = dy.mul_add(f29 as f64, dx * f30 as f64) / dist2;
        if t > 1.0 {
            t = 1.0;
        } else if t < 0.0 {
            t = 0.0;
        }
        let x = t.mul_add((bx1 - ax1) as f64, (1.0 - t).mul_add((bx0 - ax0) as f64, px as f64));
        let y = t.mul_add((by1 - ay1) as f64, (1.0 - t).mul_add((by0 - ay0) as f64, py as f64));
        (x as f32, y as f32)
    } else {
        (px + (bx0 - ax0) + (bx1 - ax0), py + (by0 - ay0) + (by1 - ay0))
    }
}

/// `mpLineIntersection`: direction dependent segment intersection. `Some((x, y))` when the C
/// returns true (and writes `int_x`/`int_y`).
#[allow(clippy::too_many_arguments)]
pub fn mp_line_intersection(
    a0x: f32, a0y: f32, a1x: f32, a1y: f32, b0x: f32, b0y: f32, b1x: f32, b1y: f32,
) -> Option<(f32, f32)> {
    let mut b1_below_a = false;
    let mut b2_above_a = false;
    if a0x <= a1x {
        if (b0x < a0x && b1x < a0x) || (a1x < b0x && a1x < b1x) {
            return None;
        }
    } else if (b0x < a1x && b1x < a1x) || (a0x < b0x && a0x < b1x) {
        return None;
    }
    if a0y <= a1y {
        if (b0y < a0y && b1y < a0y) || (a1y < b0y && a1y < b1y) {
            return None;
        }
    } else if (b0y < a1y && b1y < a1y) || (a0y < b0y && a0y < b1y) {
        return None;
    }
    let ah = (a1y - a0y) as f64;
    let d0x = (b0x - a0x) as f64;
    let aw = (a1x - a0x) as f64;
    let d0y = (b0y - a0y) as f64;
    let hs_b0_a = aw.mul_add(d0y, -(ah * d0x));
    if hs_b0_a < 0.0 {
        if hs_b0_a < -0.1 {
            return None;
        }
        b1_below_a = true;
    }
    let d1x = (b1x - a1x) as f64;
    let d1y = (b1y - a1y) as f64;
    let hs_b1_a = aw.mul_add(d1y, -(ah * d1x));
    if hs_b1_a > 0.0 {
        if hs_b1_a > 0.1 {
            return None;
        }
        b2_above_a = true;
    }
    if hs_b0_a == 0.0 && hs_b1_a == 0.0 {
        return None;
    }
    let det = d0x.mul_add(d1y, -(d0y * d1x));
    if det < hs_b0_a {
        if det < hs_b1_a {
            return None;
        }
    } else if det > hs_b0_a && det > hs_b1_a {
        return None;
    }
    let bw = (b1x - b0x) as f64;
    let bh = (b1y - b0y) as f64;
    if !((bw == 0.0 && bh == 0.0) || (b1_below_a && b2_above_a) || (hs_b0_a >= 0.0 && b2_above_a)) {
        let area = bw.mul_add(ah, -(bh * aw));
        // ABS(area) > 0.0001F: the float constant, compared in double.
        if area.abs() > 0.0001f32 as f64 {
            let t = bw.mul_add(d0y, -(bh * d0x)) / area;
            return Some(if t > 0.0 {
                if t < 1.0 {
                    (aw.mul_add(t, a0x as f64) as f32, ah.mul_add(t, a0y as f64) as f32)
                } else {
                    (a1x, a1y)
                }
            } else {
                (a0x, a0y)
            });
        }
    }
    None
}

/// `mpLineIntersectionH`: a is horizontal at `a0y`, from `a0x` to `a1x`.
#[allow(clippy::too_many_arguments)]
pub fn mp_line_intersection_h(
    a0x: f32, a0y: f32, a1x: f32, b0x: f32, b0y: f32, b1x: f32, b1y: f32,
) -> Option<(f32, f32)> {
    let (min_ax, max_ax);
    if a0x < a1x {
        if (b0x < a0x && b1x < a0x) || (a1x < b0x && a1x < b1x) {
            return None;
        }
        if (((b0y - a0y) as f64) < -0.0001) || ((b1y - a0y) as f64 > 0.0001) {
            return None;
        }
        min_ax = a0x;
        max_ax = a1x;
    } else {
        if (b0x < a1x && b1x < a1x) || (a0x < b0x && a0x < b1x) {
            return None;
        }
        if (((b1y - a0y) as f64) < -0.0001) || ((b0y - a0y) as f64 > 0.0001) {
            return None;
        }
        min_ax = a1x;
        max_ax = a0x;
    }
    let dby = (b1y - b0y) as f64;
    let dbx = (b1x - b0x) as f64;
    if dby.abs() < 0.0001 {
        return None;
    }
    let mut new_x = (dbx / dby).mul_add((a0y - b0y) as f64, b0x as f64);
    let dx = new_x - min_ax as f64;
    if dx < 0.0 {
        if dx < -0.1 {
            return None;
        }
        new_x = min_ax as f64;
    }
    if new_x - max_ax as f64 > 0.0 {
        if new_x - max_ax as f64 > 0.1 {
            return None;
        }
        new_x = max_ax as f64;
    }
    Some((new_x as f32, a0y))
}

/// `mpLineIntersectionV`: a is vertical at `a0x`, from `a0y` to `a1y`.
#[allow(clippy::too_many_arguments)]
pub fn mp_line_intersection_v(
    a0x: f32, a0y: f32, a1y: f32, b0x: f32, b0y: f32, b1x: f32, b1y: f32,
) -> Option<(f32, f32)> {
    let (min_ay, max_ay);
    if a0y < a1y {
        if (b0y < a0y && b1y < a0y) || (a1y < b0y && a1y < b1y) {
            return None;
        }
        if (((b1x - a0x) as f64) < -0.0001) || ((b0x - a0x) as f64 > 0.0001) {
            return None;
        }
        min_ay = a0y;
        max_ay = a1y;
    } else {
        if (b0y < a1y && b1y < a1y) || (a0y < b0y && a0y < b1y) {
            return None;
        }
        if (((b0x - a0x) as f64) < -0.0001) || ((b1x - a0x) as f64 > 0.0001) {
            return None;
        }
        min_ay = a1y;
        max_ay = a0y;
    }
    let dby = (b1y - b0y) as f64;
    let dbx = (b1x - b0x) as f64;
    if dbx.abs() < 0.0001 {
        return None;
    }
    let mut new_y = (dby / dbx).mul_add((a0x - b0x) as f64, b0y as f64);
    let mut dy = new_y - min_ay as f64;
    if dy < 0.0 {
        if dy < -0.1 {
            return None;
        }
        new_y = min_ay as f64;
    }
    dy = new_y - max_ay as f64;
    if dy > 0.0 {
        if dy > 0.1 {
            return None;
        }
        new_y = max_ay as f64;
    }
    Some((a0x, new_y as f32))
}

impl GroundColl {
    fn line_xy(&self, line_id: i32) -> (f32, f32, f32, f32) {
        let l = *self.map_line(line_id);
        let (x0, y0) = self.pos(l.v0_idx);
        let (x1, y1) = self.pos(l.v1_idx);
        (x0, y0, x1, y1)
    }

    fn hit(&self, line_id: i32, dist: f32) -> LineHit {
        let (x0, y0, x1, y1) = self.line_xy(line_id);
        LineHit {
            line_id,
            dist,
            flags: self.map_line(line_id).lo_flags as u32,
            normal: ps_vec_normalize(Vec3 { x: -(y1 - y0), y: x1 - x0, z: 0.0 }),
        }
    }

    /// `mpLib_8004DD90_Floor`: project `vec` onto the floor chain through `line_id`.
    pub fn mp_lib_8004dd90_floor(&self, mut line_id: i32, vec: Vec3) -> Option<LineHit> {
        self.lineid_check(774, line_id);
        let mut dir = 0;
        let (mut x0, mut x1, mut x);
        loop {
            (x0, _, x1, _) = self.line_xy(line_id);
            x = vec.x;
            if x < x0 {
                if dir != 1 {
                    let n = self.mp_line_get_prev(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_FLOOR == 0 {
                        if ((x - x0) as f64) < -0.1 {
                            return None;
                        }
                        x = x0;
                        break;
                    }
                    line_id = n;
                    dir = -1;
                } else {
                    x = x0;
                    break;
                }
            } else if x > x1 {
                if dir != -1 {
                    let n = self.mp_line_get_next(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_FLOOR == 0 {
                        if (x - x1) as f64 > 0.1 {
                            return None;
                        }
                        x = x1;
                        break;
                    }
                    // The C does not set dir here.
                    line_id = n;
                } else {
                    x = x1;
                    break;
                }
            } else {
                break;
            }
        }
        let (_, y0, _, y1) = self.line_xy(line_id);
        let d = ((((y1 - y0) * (x - x0)) / (x1 - x0) + y0 - vec.y) as f64 + 0.0001) as f32;
        Some(self.hit(line_id, d))
    }

    /// `mpLib_8004E090_Ceiling`
    pub fn mp_lib_8004e090_ceiling(&self, mut line_id: i32, vec: Vec3) -> Option<LineHit> {
        self.lineid_check(893, line_id);
        let mut dir = 0;
        let (mut x0, mut x1);
        let mut x = vec.x;
        loop {
            (x0, _, x1, _) = self.line_xy(line_id);
            if vec.x < x1 {
                if dir != 1 {
                    let n = self.mp_line_get_next(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_CEILING == 0 {
                        if ((vec.x - x1) as f64) < -0.1 {
                            return None;
                        }
                        x = x1;
                        break;
                    }
                    line_id = n;
                    dir = -1;
                    continue;
                }
                x = x1;
            } else if vec.x > x0 {
                if dir != -1 {
                    let n = self.mp_line_get_prev(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_CEILING == 0 {
                        if (vec.x - x0) as f64 > 0.1 {
                            return None;
                        }
                        x = x0;
                        break;
                    }
                    line_id = n;
                    dir = 1;
                    continue;
                }
                x = x0;
            }
            break;
        }
        let (_, y0, _, y1) = self.line_xy(line_id);
        let d = ((((y1 - y0) * (x - x0)) / (x1 - x0) + y0 - vec.y) as f64 - 0.0001) as f32;
        Some(self.hit(line_id, d))
    }

    /// `mpLib_8004E398_LeftWall`
    pub fn mp_lib_8004e398_left_wall(&self, mut line_id: i32, vec: Vec3) -> Option<LineHit> {
        self.lineid_check(1014, line_id);
        let mut dir = 0;
        let (mut y0, mut y1);
        let mut y = vec.y;
        loop {
            (_, y0, _, y1) = self.line_xy(line_id);
            if vec.y < y0 {
                if dir != 1 {
                    let n = self.mp_line_get_prev(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_LEFT_WALL == 0 {
                        if ((vec.y - y0) as f64) < -0.1 {
                            return None;
                        }
                        y = y0;
                        break;
                    }
                    line_id = n;
                    dir = -1;
                    continue;
                }
                // The C leaves y = vec->y here (no clamp), unlike the right wall.
            } else if vec.y > y1 && dir != -1 {
                let n = self.mp_line_get_next(line_id);
                if n == -1 || self.flags(n) & COLL_LINE_LEFT_WALL == 0 {
                    if (vec.y - y1) as f64 > 0.1 {
                        return None;
                    }
                    y = y1;
                    break;
                }
                line_id = n;
                dir = 1;
                continue;
            }
            break;
        }
        let (x0, _, x1, _) = self.line_xy(line_id);
        let d = x0 + ((x1 - x0) * (y - y0)) / (y1 - y0) - vec.x;
        Some(self.hit(line_id, d))
    }

    /// `mpLib_8004E684_RightWall`
    pub fn mp_lib_8004e684_right_wall(&self, mut line_id: i32, vec: Vec3) -> Option<LineHit> {
        self.lineid_check(1134, line_id);
        let mut dir = 0;
        let (mut y0, mut y1);
        let mut y = vec.y;
        loop {
            (_, y0, _, y1) = self.line_xy(line_id);
            if vec.y > y0 {
                if dir != -1 {
                    let n = self.mp_line_get_prev(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_RIGHT_WALL == 0 {
                        if (vec.y - y0) as f64 > 0.1 {
                            return None;
                        }
                        y = y0;
                        break;
                    }
                    line_id = n;
                    dir = 1;
                    continue;
                }
                y = y0;
            } else if vec.y < y1 {
                if dir != 1 {
                    let n = self.mp_line_get_next(line_id);
                    if n == -1 || self.flags(n) & COLL_LINE_RIGHT_WALL == 0 {
                        if ((vec.y - y1) as f64) < -0.1 {
                            return None;
                        }
                        y = y1;
                        break;
                    }
                    line_id = n;
                    dir = -1;
                    continue;
                }
                y = y1;
            }
            break;
        }
        let (x0, _, x1, _) = self.line_xy(line_id);
        let d = x0 + ((x1 - x0) * (y - y0)) / (y1 - y0) - vec.x;
        Some(self.hit(line_id, d))
    }

    /// `mpLib_8004ED5C`: the line's endpoints, each pushed one unit outward along the line when
    /// the line has a neighbour on that side. Returns (x0, y0, x1, y1).
    pub fn mp_lib_8004ed5c(&self, line_id: i32) -> (f32, f32, f32, f32) {
        let (mut x0, mut y0, mut x1, mut y1) = self.line_xy(line_id);
        let mut distance = 0.0f32; // C: uninitialised, but always set before it is read.
        let mut calculated = false;
        let dist = |x0: f32, y0: f32, x1: f32, y1: f32| {
            let (dx, dy) = (x0 - x1, y0 - y1);
            sqrtf(fmadds(dx, dx, dy * dy))
        };
        if self.mp_line_get_prev(line_id) != -1 {
            distance = dist(x0, y0, x1, y1);
            if distance > 0.001 {
                x0 += (x0 - x1) / distance;
                y0 += (y0 - y1) / distance;
            }
            calculated = true;
        }
        if self.mp_line_get_next(line_id) != -1 {
            if !calculated {
                distance = dist(x0, y0, x1, y1);
            }
            if distance > 0.001 {
                x1 += (x1 - x0) / distance;
                y1 += (y1 - y0) / distance;
            }
        }
        (x0, y0, x1, y1)
    }

    /// `mpVtxGetPos`
    pub fn mp_vtx_get_pos(&self, vtx_id: i32) -> (f32, f32) {
        let p = self.vtx[vtx_id as usize].pos;
        (p.x, p.y)
    }

    /// `mpVtxSetPos`
    pub fn mp_vtx_set_pos(&mut self, vtx_id: i32, x: f32, y: f32) {
        let v = &mut self.vtx[vtx_id as usize];
        v.pos.x = x;
        v.pos.y = y;
    }

    /// `mpLineSetPos`
    pub fn mp_line_set_pos(&mut self, line_id: i32, x0: f32, y0: f32, x1: f32, y1: f32) {
        let l = *self.map_line(line_id);
        self.mp_vtx_set_pos(l.v0_idx as i32, x0, y0);
        self.mp_vtx_set_pos(l.v1_idx as i32, x1, y1);
    }

    /// `mpLib_80056758`: set the line's endpoints relative to each vertex's base (`x0`, `x4`).
    pub fn mp_lib_80056758(&mut self, line_id: i32, x0: f32, y0: f32, x1: f32, y1: f32) {
        let l = *self.map_line(line_id);
        let v = &mut self.vtx[l.v0_idx as usize];
        v.pos.x = v.x0 + x0;
        v.pos.y = v.x4 + y0;
        let v = &mut self.vtx[l.v1_idx as usize];
        v.pos.x = v.x0 + x1;
        v.pos.y = v.x4 + y1;
    }

    /// `mpGetSpeed`: how far `pos` moved with the line since the last frame (`x10`/`x14` hold
    /// each vertex's previous position). `None` when the C returns false.
    /// The C's `DbLevel >= DbLKind_DebugRom` range assert is debug-ROM only and not ported.
    pub fn mp_get_speed(&self, line_id: i32, pos: Vec3) -> Option<Vec3> {
        if !self.mp_lib_80054ed8(line_id) {
            return None;
        }
        let l = *self.map_line(line_id);
        let v0 = self.vtx[l.v0_idx as usize];
        let v1 = self.vtx[l.v1_idx as usize];
        let (nx, ny) = mp_remap2d(
            v0.x10, v0.x14, v1.x10, v1.x14, v0.pos.x, v0.pos.y, v1.pos.x, v1.pos.y, pos.x, pos.y,
        );
        Some(Vec3 { x: nx - pos.x, y: ny - pos.y, z: 0.0 })
    }
}

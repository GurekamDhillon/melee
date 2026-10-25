//! Fighter colour animations (`ColorOverlay`): the interpreter of `lb_013B.c` (`lb_800144C8` start,
//! `lb_80014258` per-frame step, `lb_80014498` reset) over PlCo.dat's colanim tables
//! (`Fighter_804D653C` = pData[6], the spycloak table `Fighter_804D6538` = pData[7]), and the
//! three lists a fighter keeps (`x408`, `x488`, `x508`; `ftcolanim.c`).
//!
//! Render-only state: nothing here is hashed and nothing draws from the RNG. The effect / sound
//! commands (0x15-0x17) a script can carry are skipped over (their word counts are all that
//! matters to the stream).
use std::sync::Arc;

/// PlCo.dat's data and the offsets of the two colanim tables (8-byte records: script offset, priority, list).
pub struct ColTables {
    data: Vec<u8>,
    main: u32,
    spy: u32,
}

impl ColTables {
    pub fn new(plco: &[u8], root: u32) -> Self {
        let be = |at: u32| u32::from_be_bytes(plco[at as usize..][..4].try_into().unwrap());
        Self { data: plco.to_vec(), main: be(root + 4 * 6), spy: be(root + 4 * 7) }
    }
    fn word(&self, at: u32) -> u32 {
        u32::from_be_bytes(self.data[at as usize..][..4].try_into().unwrap())
    }
    fn rec(&self, spy: bool, i: i32) -> (u32, u8, u8) {
        let base = if spy { self.spy } else { self.main } + 8 * i as u32;
        (self.word(base), self.data[base as usize + 4], self.data[base as usize + 5])
    }
    /// Debug: the script words of colanim `id` up to the first reset/end.
    pub fn dump(&self, id: i32) -> Vec<u32> {
        let mut at = self.rec(false, id).0;
        let mut v = Vec::new();
        for _ in 0..64 {
            let w = self.word(at);
            v.push(w);
            at += 4;
            if w >> 26 == 0 || w >> 26 == 0xA {
                break;
            }
        }
        v
    }
    /// `Fighter_804D653C[id].unk5`: the colanim lives on the x488 list.
    pub fn is_x488(&self, id: i32) -> bool {
        self.rec(false, id).2 != 0
    }
}

/// One `ColorOverlay` (the fields the draw reads and the script state).
#[derive(Clone, Default)]
pub struct Lane {
    timer: i32,
    pri: i32,
    pos: Option<u32>,
    stack: Vec<i64>,
    spy: bool,
    /// `x28_colanim.i` (0 with no animation).
    pub id: i32,
    pub color_enable: bool,
    pub flag2: bool,
    pub light_enable: bool,
    pub hex: [u8; 4],
    fc: [f32; 4],
    dc: [f32; 4],
    pub light: [u8; 4],
    fl: [f32; 4],
    dl: [f32; 4],
}

fn rgba(w: u32) -> [u8; 4] {
    w.to_be_bytes()
}

impl Lane {
    /// `lb_80014498`.
    pub fn reset(&mut self) {
        self.pos = None;
        self.pri = 0;
        self.id = 0;
        self.color_enable = false;
        self.flag2 = false;
    }

    /// `lb_800144C8`: start animation `idx` of table `spy` unless a higher-priority one runs.
    pub fn start(&mut self, t: &ColTables, spy: bool, idx: i32, pri: i32) -> bool {
        let cur = if self.spy == spy { self.id } else { 0 };
        if t.rec(spy, cur).1 <= t.rec(spy, idx).1 {
            self.id = idx;
            self.spy = spy;
            self.pri = pri;
            self.pos = Some(t.rec(spy, idx).0);
            self.timer = 0;
            self.stack.clear();
            self.color_enable = false;
            self.flag2 = false;
            return true;
        }
        false
    }

    /// `lb_80014258`: one frame. `skip`: the ft_800BFF70 callbacks (a x408 animation is running).
    /// True when the animation ended (the caller resets the lane and reapplies the standing overlays).
    pub fn step(&mut self, t: &ColTables) -> bool {
        if self.pos.is_some() && self.timer != 0 {
            self.timer -= 1;
        }
        while let Some(at) = self.pos {
            if self.timer != 0 {
                break;
            }
            let w = t.word(at);
            let op = w >> 26;
            let val = w & 0x03FF_FFFF;
            let next = at + 4;
            match op {
                0 => self.pos = None,
                1 => {
                    self.timer += val as i32;
                    self.pos = Some(next);
                }
                2 => {
                    self.timer = val as i32;
                    self.pos = Some(next);
                }
                3 => {
                    self.stack.push(next as i64);
                    self.stack.push(t.word(at) as i64 & 0x03FF_FFFF);
                    self.pos = Some(next);
                }
                4 => {
                    let n = self.stack.len();
                    self.stack[n - 1] -= 1;
                    if self.stack[n - 1] as i32 != 0 {
                        self.pos = Some(self.stack[n - 2] as u32);
                    } else {
                        self.pos = Some(next);
                        self.stack.truncate(n - 2);
                    }
                }
                5 => {
                    self.stack.push((at + 8) as i64);
                    self.pos = Some(t.word(next));
                }
                6 => {
                    self.pos = self.stack.pop().map(|v| v as u32);
                }
                7 => self.pos = Some(t.word(next)),
                8 => {
                    self.timer = 0x7F7F_FFFFu32 as i32;
                    self.pos = Some(next);
                }
                9 => self.pos = Some(next),
                0xA => return true,
                0xB => {
                    self.timer += val as i32;
                    self.pos = Some(next);
                }
                0xC => {
                    self.color_enable = false;
                    self.flag2 = false;
                    self.pos = Some(next);
                }
                0xD => {
                    self.light_enable = (w >> 25) & 1 != 0;
                    self.set_light(t.word(next));
                    self.flag2 = true;
                    self.pos = Some(next + 4);
                }
                0xE => {
                    self.set_light(t.word(next));
                    self.pos = Some(next + 4);
                }
                0xF => {
                    let f = val as f32;
                    let c = rgba(t.word(next));
                    for i in 0..4 {
                        self.dl[i] = ((0.5f32 + c[i] as f32) - self.light[i] as f32) / f;
                    }
                    self.pos = Some(next + 4);
                }
                0x10 => self.pos = Some(next),
                0x11 => {
                    self.flag2 = false;
                    self.pos = Some(next);
                }
                0x12 => {
                    self.color_enable = true;
                    let c = rgba(t.word(next));
                    self.hex = c;
                    self.fc = c.map(|v| v as f32);
                    self.dc = [0.0; 4];
                    self.pos = Some(next + 4);
                }
                0x13 => {
                    let f = val as f32;
                    let c = rgba(t.word(next));
                    for i in 0..4 {
                        self.dc[i] = ((0.5f32 + c[i] as f32) - self.hex[i] as f32) / f;
                    }
                    self.pos = Some(next + 4);
                }
                0x14 => {
                    self.color_enable = false;
                    self.pos = Some(next);
                }
                // ftAction effect (5 words), sound (3), the x2226_b5 refraction toggle (1).
                0x15 => self.pos = Some(at + 20),
                0x16 => self.pos = Some(at + 12),
                0x17 => self.pos = Some(next),
                _ => self.pos = None,
            }
        }
        if self.color_enable {
            for i in 0..4 {
                self.fc[i] += self.dc[i];
                self.hex[i] = self.fc[i] as u8;
            }
        }
        if self.flag2 {
            for i in 0..4 {
                self.fl[i] += self.dl[i];
                self.light[i] = self.fl[i] as u8;
            }
        }
        if self.pri != 0 {
            self.pri -= 1;
            if self.pri == 0 {
                return true;
            }
        }
        false
    }

    fn set_light(&mut self, w: u32) {
        let c = rgba(w);
        self.light = c;
        self.fl = c.map(|v| v as f32);
        self.dl = [0.0; 4];
    }
}

/// A fighter's three lists and the tables.
#[derive(Clone)]
pub struct ColAnim {
    t: Arc<ColTables>,
    pub x408: Lane,
    pub x488: Lane,
    pub x508: Lane,
    /// `fp->x2224_b2`, `fp->dmg.x18F0 != 0`, `x221D_b6`, `x1994 | x1990 | x2221_b0` and `x2226_b4`: the standing overlays
    /// `reapply_x408` restarts (the sim flags, set by [`ColAnim::standing`]).
    standing: [bool; 4],
    x2226_b4: bool,
    hammer: bool,
    /// The next motion change has `Ft_MF_SkipColAnim`.
    pub skip_next: bool,
    /// `fp->x1994` (the intangibility timer; only its colour animation is modelled).
    x1994: i32,
}

impl ColAnim {
    pub fn new(t: Arc<ColTables>) -> Self {
        Self { t, x408: Lane::default(), x488: Lane::default(), x508: Lane::default(), standing: [false; 4], x2226_b4: false, hammer: false, skip_next: false, x1994: 0 }
    }

    /// `ftCo_800BFFD0`.
    pub fn start(&mut self, id: i32, pri: i32) -> bool {
        if id >= 0x7B {
            return self.x508.start(&self.t, true, id - 0x7B, pri);
        }
        if self.t.is_x488(id) {
            self.x488.start(&self.t, false, id, pri)
        } else {
            self.x408.start(&self.t, false, id, pri)
        }
    }

    /// `ftCo_800C0200`: reset the list `id` belongs to and reapply the standing overlays.
    pub fn reset_id(&mut self, id: i32) {
        if self.t.is_x488(id) {
            self.x488.reset();
            self.reapply_x488();
        } else {
            self.x408.reset();
            self.reapply_x408();
        }
    }

    /// `ftCo_800BFFAC` + the `lb_80014498(&x488)` of `ftCo_800C0134` (a motion change).
    pub fn motion_change(&mut self) {
        if std::mem::take(&mut self.skip_next) {
            return;
        }
        self.x488.reset();
        if self.hammer {
            self.start(0x6A, 0);
        }
    }

    /// The flags the standing overlays read (`x2224_b2`, `dmg.x18F0`, `x221D_b6`, `x1994 | x1990 | x2221_b0`).
    pub fn set_standing(&mut self, f: [bool; 4]) {
        self.standing = f;
    }

    fn reapply_x408(&mut self) {
        let mut s = self.standing;
        s[3] |= self.x1994 != 0;
        if s[0] {
            self.start(0x7A, 0);
        }
        if s[1] {
            self.start(8, 0);
        }
        if s[2] {
            self.start(0x6B, 0);
        }
        if s[3] {
            self.start(9, 0);
        }
    }

    fn reapply_x488(&mut self) {
        if self.hammer {
            self.start(0x6A, 0);
        }
    }

    /// `ftColl_8007B7A4`: an intangibility window of `frames` with its colour animation.
    pub fn intangible(&mut self, frames: i32) {
        if frames > self.x1994 {
            self.x1994 = frames;
        }
        self.start(9, 0);
    }

    /// `ftCo_800C0408`.
    pub fn step(&mut self) {
        let t = self.t.clone();
        // fighter.c: the intangibility timer runs out and its colour animation is reset.
        if self.x1994 != 0 {
            self.x1994 -= 1;
            if self.x1994 == 0 {
                self.standing[3] = false;
                if self.active().id == 9 {
                    self.reset_id(9);
                }
            }
        }
        while self.x508.step(&t) {
            self.x508.reset();
            if self.x2226_b4 {
                self.start(0x80, 0);
            }
        }
        while self.x408.step(&t) {
            self.x408.reset();
            self.reapply_x408();
        }
        while self.x488.step(&t) {
            self.x488.reset();
            self.reapply_x488();
        }
    }

    pub fn dump(&self, id: i32) -> Vec<u32> {
        self.t.dump(id)
    }

    /// `ftCo_800C0658`: the overlay the draw reads.
    pub fn active(&self) -> &Lane {
        if self.x408.id != 0 { &self.x408 } else { &self.x488 }
    }
}

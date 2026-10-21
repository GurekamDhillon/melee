//! HSD random number generator. Port of `src/sysdolphin/baselib/random.c` (TARGET_PC variant:
//! a separate render stream so draws made while drawing never consume the simulation stream).

/// Where `HSD_RandSeedPtr` points. A stage can redirect the logic seed into its own memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedSlot {
    /// The file-static `seed` (default).
    Default,
    /// Redirected to a seed owned elsewhere (raw because the owner is game memory of another
    /// group). TODO: replace with a typed handle once stage state is ported.
    Redirected(*mut u32),
}

/// State of random.c: `seed`, `HSD_RandSeedPtr`, `render_seed`, `render_depth`.
/// All of it is game state for snapshots (render_seed is 0 at every logic boundary).
#[derive(Clone, Debug)]
pub struct HsdRand {
    pub seed: u32,
    pub seed_ptr: SeedSlot,
    pub render_seed: u32,
    pub render_depth: u32,
    /// `Replay_RandTrace` hook (caller id unknown natively; stream 1 = default sim, 0 =
    /// redirected sim, 2 = render).
    pub trace: Option<fn(seed_after: u32, stream: i32)>,
}

impl Default for HsdRand {
    fn default() -> Self {
        Self { seed: 1, seed_ptr: SeedSlot::Default, render_seed: 0, render_depth: 0, trace: None }
    }
}

const MUL: u32 = 214013;
const ADD: u32 = 2531011;

impl HsdRand {
    /// `*HSD_RandSeedPtr`.
    pub fn logic_seed(&self) -> u32 {
        match self.seed_ptr {
            SeedSlot::Default => self.seed,
            // SAFETY: the redirect owner keeps the seed alive until `forget_memory`.
            SeedSlot::Redirected(p) => unsafe { *p },
        }
    }

    pub fn set_logic_seed(&mut self, v: u32) {
        match self.seed_ptr {
            SeedSlot::Default => self.seed = v,
            SeedSlot::Redirected(p) => unsafe { *p = v },
        }
    }

    /// `HSD_RandRenderBegin`.
    pub fn render_begin(&mut self) {
        let d = self.render_depth;
        self.render_depth = d.wrapping_add(1);
        if d == 0 {
            self.render_seed = self.logic_seed();
        }
    }

    /// `HSD_RandRenderEnd`.
    pub fn render_end(&mut self) {
        if self.render_depth != 0 {
            self.render_depth -= 1;
            if self.render_depth == 0 {
                self.render_seed = 0;
            }
        }
    }

    /// One LCG step on the current stream; returns the new seed.
    #[inline]
    fn step(&mut self) -> u32 {
        let v;
        let stream;
        if self.render_depth != 0 {
            self.render_seed = self.render_seed.wrapping_mul(MUL).wrapping_add(ADD);
            v = self.render_seed;
            stream = 2;
        } else {
            v = self.logic_seed().wrapping_mul(MUL).wrapping_add(ADD);
            self.set_logic_seed(v);
            stream = (self.seed_ptr == SeedSlot::Default) as i32;
        }
        if let Some(t) = self.trace {
            t(v, stream);
        }
        v
    }

    /// `HSD_Rand`: 0..=0xFFFF.
    pub fn rand(&mut self) -> i32 {
        (self.step() >> 16) as i32
    }

    /// `HSD_Randf`: [0, 1).
    pub fn randf(&mut self) -> f32 {
        (self.step() >> 16) as f32 / 65536.0
    }

    /// `HSD_Randi`: `max_val * r / 65536` with C's wrapping multiply and truncating divide.
    pub fn randi(&mut self, max_val: i32) -> i32 {
        max_val.wrapping_mul((self.step() >> 16) as i32) / (1 << 16)
    }

    /// random.h's inline wrapper: `HSD_Randi` that accepts an empty range.
    pub fn randi_safe(&mut self, max_val: i32) -> i32 {
        if max_val != 0 {
            self.randi(max_val)
        } else {
            0
        }
    }

    /// `_HSD_RandForgetMemory`: drop a redirect that points into a freed range.
    pub fn forget_memory(&mut self, low: *const u8, high: *const u8) {
        if let SeedSlot::Redirected(p) = self.seed_ptr {
            let p = p as *const u8;
            if low <= p && p < high {
                self.seed_ptr = SeedSlot::Default;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_first_values() {
        let mut r = HsdRand::default();
        // seed 1 -> 1*214013 + 2531011 = 2745024 = 0x29E2C0 -> >>16 = 0x29
        assert_eq!(r.rand(), 0x29);
        assert_eq!(r.logic_seed(), 2745024);
    }

    #[test]
    fn render_stream_is_isolated() {
        let mut r = HsdRand::default();
        r.render_begin();
        r.rand();
        r.render_end();
        assert_eq!(r.logic_seed(), 1);
        assert_eq!(r.render_seed, 0);
    }
}

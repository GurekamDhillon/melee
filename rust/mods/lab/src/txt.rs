//! Fixed-size text for the HUD: formatted in place, never allocated. A readout keeps its `Txt` and the value it was
//! formatted from, and re-formats only when that value changes.

#[derive(Clone, Copy)]
pub struct Txt {
    b: [u8; 48],
    n: u8,
}

impl Txt {
    pub const fn new() -> Txt {
        Txt { b: [0; 48], n: 0 }
    }
    pub fn clear(&mut self) -> &mut Txt {
        self.n = 0;
        self
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    /// Append `s`; what does not fit is cut at a character boundary.
    pub fn s(&mut self, s: &str) -> &mut Txt {
        for (i, c) in s.char_indices() {
            let l = c.len_utf8();
            if self.n as usize + l > self.b.len() {
                break;
            }
            self.b[self.n as usize..self.n as usize + l].copy_from_slice(&s.as_bytes()[i..i + l]);
            self.n += l as u8;
        }
        self
    }
    /// Append an integer (`plus`: a leading + on positive values).
    pub fn int(&mut self, v: i32, plus: bool) -> &mut Txt {
        let mut d = [0u8; 12];
        let mut i = d.len();
        let mut n = v.unsigned_abs();
        loop {
            i -= 1;
            d[i] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        if v < 0 {
            i -= 1;
            d[i] = b'-';
        } else if plus && v > 0 {
            i -= 1;
            d[i] = b'+';
        }
        // digits are ASCII
        self.s(unsafe { core::str::from_utf8_unchecked(&d[i..]) })
    }
    pub fn i(&mut self, v: i32) -> &mut Txt {
        self.int(v, false)
    }
    /// Append a value with `dp` decimals (rounded half away from zero).
    pub fn fix(&mut self, v: f32, dp: u32) -> &mut Txt {
        if v != v {
            return self.s("-");
        }
        let m = 10i64.pow(dp);
        let r = (v.abs() as f64 * m as f64 + 0.5) as i64;
        if v < 0.0 && r != 0 {
            self.s("-");
        }
        self.i((r / m).min(i32::MAX as i64) as i32);
        if dp > 0 {
            self.s(".");
            let mut f = r % m;
            let mut d = [b'0'; 6];
            for k in (0..dp as usize).rev() {
                d[k] = b'0' + (f % 10) as u8;
                f /= 10;
            }
            self.s(unsafe { core::str::from_utf8_unchecked(&d[..dp as usize]) });
        }
        self
    }
    pub fn as_str(&self) -> &str {
        // only whole characters are ever appended
        unsafe { core::str::from_utf8_unchecked(&self.b[..self.n as usize]) }
    }
}

//! DOL referee step 1 (notes/referee.md): the interpreter's Gekko mode (`Machine::new_gekko`, what the referee and
//! the m-ex guest run) against an independent model of Dolphin's interpreter, instruction by instruction, on
//! fuzzed operands with the corner cases weighted in (+-0, denormals, NaN payloads quiet and signalling, infinities,
//! overflow, the single-precision denormal range, fctiw ranges, shift amounts 0..127, INT_MIN / -1, ...).
//!
//! The model is written here from Dolphin's interpreter semantics (Source/Core/Core/PowerPC/Interpreter/*.cpp,
//! Interpreter_FPUtils.h, Common/FloatUtils.cpp; read, not linked) and decodes every word itself: it shares no code
//! with `mex_import::ppc`. The fused multiply-add is an exact software fma (integer mantissas, one rounding), not the
//! host's `mul_add`. Assumed FPSCR: RN = 0 (nearest), NI = 0, all exception enables 0 (what the game runs with; the
//! interpreter has no FPSCR). FPSCR flags and ps1 are not modelled: the interpreter keeps neither.
//!
//! `REFEREE_FUZZ=N` sets the operand sets per instruction form (default 20000; notes/referee.md reports a 2,000,000
//! run). Every disagreement is printed with its operands; the test fails on any outside `KNOWN`.

use mex_import::ppc::{Bridge, Cpu, Fault, FlatMem, Machine, Mem};

const MEM_BASE: u32 = 0x8000_0000;
const MEM_SIZE: usize = 0x1000;

// ---------------------------------------------------------------------------------------------------------------
// random operands

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn coin(&mut self, k: u64) -> bool {
        self.below(k) == 0
    }
    fn int(&mut self) -> u32 {
        match self.below(10) {
            0 => [0, 1, 0xFFFF_FFFF, 0x8000_0000, 0x7FFF_FFFF, 0xFFFF_FFFE, 0x8000_0001, 2][self.below(8) as usize],
            1 => self.below(130) as u32,
            2 => (self.below(130) as u32).wrapping_neg(),
            3 => 1 << self.below(32),
            4 => (self.next() as u32) & 0xFFFF,
            5 => (self.next() as u16 as i16 as i32) as u32,
            _ => self.next() as u32,
        }
    }
    /// A double with the interesting classes weighted in.
    fn fp(&mut self) -> u64 {
        let s = if self.coin(2) { 1u64 << 63 } else { 0 };
        let frac = self.next() & FRAC;
        match self.below(16) {
            0 => s,                                                           // +-0
            1 => s | EXP,                                                     // +-inf
            2 => s | EXP | QBIT | (frac & if self.coin(2) { 0 } else { !0 }), // quiet NaN, payload or not
            3 => s | EXP | ((frac & !QBIT) | 1),                              // signalling NaN
            4 => s | frac.max(1),                                             // double denormal
            5 => s | ((874 + self.below(23)) << 52) | frac,                   // single denormal range
            6 => s | ((1023 + 127 + self.below(3)) << 52) | frac,             // around the single overflow
            7 => {
                // an integer near the fctiw range edges, often with a fraction
                let base = [0.0, 2147483647.0, 2147483648.0, -2147483648.0, -2147483649.0, 4294967296.0][self.below(6) as usize];
                let frac = [0.0, 0.5, 0.25, 0.75, 0.999, -0.5][self.below(6) as usize];
                let v: f64 = base + (self.below(5) as f64 - 2.0) + frac;
                v.to_bits() ^ s
            }
            8 => {
                // small integers and ties
                let v = (self.below(64) as f64 - 32.0) * [1.0, 0.5, 0.25][self.below(3) as usize];
                v.to_bits()
            }
            9 => ((f32::from_bits(self.next() as u32) as f64).to_bits()) | 0, // a single, any class
            10 => {
                // a single with low bits around the Force25Bit rounding bit (bit 27)
                let base = (f32::from_bits(self.next() as u32 & 0x7FFF_FFFF | (s >> 32) as u32) as f64).to_bits();
                base | [0x800_0000, 0x7FF_FFFF, 0x800_0001, self.next() & 0xFFF_FFFF][self.below(4) as usize]
            }
            11 => s | ((1023 - 30 + self.below(60)) << 52) | (frac & 0xFFFF_FFFF_E000_0000), // single-valued, moderate
            12 => [0x3FF0_0000_0000_0000, 0x7FEF_FFFF_FFFF_FFFF, 0x0010_0000_0000_0000, 0x47EF_FFFF_E000_0000, 0x3810_0000_0000_0000, 0x380F_FFFF_FFFF_FFFF][self.below(6) as usize] | s,
            _ => self.next(), // anything
        }
    }
    fn fp_pair(&mut self) -> (u64, u64) {
        let a = self.fp();
        let b = match self.below(8) {
            0 => a,
            1 => a ^ (1 << 63),
            2 => a.wrapping_add(self.below(3)) ^ (1 << 63),
            _ => self.fp(),
        };
        (a, b)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// the model: Dolphin's interpreter semantics

const EXP: u64 = 0x7FF0_0000_0000_0000;
const FRAC: u64 = 0x000F_FFFF_FFFF_FFFF;
const QBIT: u64 = 0x0008_0000_0000_0000;
const SIGN: u64 = 1 << 63;
/// std::numeric_limits<double>::quiet_NaN() on the host Dolphin is built for: positive, payload 0.
const PPC_NAN: u64 = 0x7FF8_0000_0000_0000;

fn is_nan(x: u64) -> bool {
    x & EXP == EXP && x & FRAC != 0
}
fn quiet(x: u64) -> u64 {
    x | QBIT
}
fn d(x: u64) -> f64 {
    f64::from_bits(x)
}

/// Exact fused multiply-add on finite, non-zero-product-and-sum-handled operands: a*c + b with one rounding to
/// nearest even. Built on integer mantissas so it shares nothing with the host fma.
fn soft_fma(a: u64, c: u64, b: u64) -> u64 {
    let (fa, fc, fb) = (d(a), d(c), d(b));
    if is_nan(a) || is_nan(b) || is_nan(c) {
        return PPC_NAN; // callers handle NaN operands
    }
    let ai = a & !SIGN == EXP;
    let ci = c & !SIGN == EXP;
    let bi = b & !SIGN == EXP;
    let psign = (a ^ c) & SIGN;
    let pzero = a & !SIGN == 0 || c & !SIGN == 0;
    if ai || ci {
        if pzero {
            return PPC_NAN; // inf * 0
        }
        if bi && (b & SIGN) != psign {
            return PPC_NAN; // inf - inf
        }
        return psign | EXP;
    }
    if bi {
        return b;
    }
    if pzero {
        // exact zero product: zero + b (the sign of a zero sum is + unless both are -)
        if b & !SIGN == 0 {
            return if psign != 0 && b & SIGN != 0 { SIGN } else { 0 };
        }
        return b;
    }
    let _ = (fa, fc, fb);
    // unpack: value = m * 2^e, m an integer
    fn unpack(x: u64) -> (u64, i32) {
        let e = ((x >> 52) & 0x7FF) as i32;
        let f = x & FRAC;
        if e == 0 { (f, -1074) } else { (f | (1 << 52), e - 1075) }
    }
    let (ma, ea) = unpack(a);
    let (mc, ec) = unpack(c);
    let (mb, eb) = unpack(b);
    let bsign = b & SIGN;
    // product exactly: mp * 2^ep, mp < 2^106
    let mp = ma as u128 * mc as u128;
    let ep = ea + ec;
    // big-ish integer arithmetic in a 4 x u64 little-endian vector, at a common exponent
    // choose the lower exponent as base, but clamp shifts: if one term is far below the other, it only matters as sticky
    type Big = [u64; 6];
    fn shl(x: u128, s: u32) -> Big {
        let mut r = [0u64; 6];
        let (w, b) = ((s / 64) as usize, s % 64);
        let lo = x as u64;
        let hi = (x >> 64) as u64;
        let parts = [lo, hi, 0u64];
        for i in 0..3 {
            if w + i < 6 {
                r[w + i] |= parts[i] << b;
                if b != 0 && w + i + 1 < 6 {
                    r[w + i + 1] |= parts[i] >> (64 - b);
                }
            }
        }
        r
    }
    fn add(a: &Big, b: &Big) -> Big {
        let mut r = [0u64; 6];
        let mut carry = 0u128;
        for i in 0..6 {
            let s = a[i] as u128 + b[i] as u128 + carry;
            r[i] = s as u64;
            carry = s >> 64;
        }
        r
    }
    fn sub(a: &Big, b: &Big) -> Big {
        let mut r = [0u64; 6];
        let mut borrow = 0i128;
        for i in 0..6 {
            let s = a[i] as i128 - b[i] as i128 - borrow;
            if s < 0 {
                r[i] = (s + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                r[i] = s as u64;
                borrow = 0;
            }
        }
        r
    }
    fn cmp(a: &Big, b: &Big) -> std::cmp::Ordering {
        for i in (0..6).rev() {
            if a[i] != b[i] {
                return a[i].cmp(&b[i]);
            }
        }
        std::cmp::Ordering::Equal
    }
    // the base exponent: the smaller of the two, but no more than 200 bits below the larger term's top (a term entirely
    // below that only contributes a sticky bit, represented as one unit at the bottom)
    let top_p = ep + 106;
    let top_b = eb + 53;
    let top = top_p.max(top_b);
    let base = ep.min(eb).max(top - 300);
    let term = |m: u128, e: i32| -> Big {
        if m == 0 {
            return [0; 6];
        }
        if e >= base {
            shl(m, (e - base) as u32)
        } else {
            // below the base: shift right, keeping a sticky unit
            let s = (base - e) as u32;
            let v = if s >= 128 { 0 } else { m >> s };
            let lost = if s >= 128 { true } else { m & ((1u128 << s) - 1) != 0 };
            let mut r = shl(v, 0);
            if lost {
                r[0] |= 1; // sticky (the base is >= 2 bits below the top of anything that survives)
            }
            r
        }
    };
    let tp = term(mp, ep);
    let tb = term(mb as u128, eb);
    let (mag, sign) = if psign == bsign {
        (add(&tp, &tb), psign)
    } else {
        match cmp(&tp, &tb) {
            std::cmp::Ordering::Greater => (sub(&tp, &tb), psign),
            std::cmp::Ordering::Less => (sub(&tb, &tp), bsign),
            std::cmp::Ordering::Equal => return 0, // exact cancellation: +0 in round to nearest
        }
    };
    // round mag * 2^base to a double
    let mut nbits = 0i32;
    for i in (0..6).rev() {
        if mag[i] != 0 {
            nbits = 64 * i as i32 + 64 - mag[i].leading_zeros() as i32;
            break;
        }
    }
    if nbits == 0 {
        return 0;
    }
    let bit = |k: i32| -> u64 { if k < 0 { 0 } else { (mag[(k / 64) as usize] >> (k % 64)) & 1 } };
    let any_below = |k: i32| -> bool {
        // any bit strictly below position k
        for j in 0..k.min(384) {
            if bit(j) != 0 {
                return true;
            }
        }
        false
    };
    // exponent of the leading bit
    let lead = base + nbits - 1; // value in [2^lead, 2^(lead+1))
    // the double's unit in the last place: normal -> lead - 52, but not below -1074 (denormals)
    let ulp = (lead - 52).max(-1074);
    let cut = ulp - base; // bits below position `cut` are rounded off
    let mut m: u64 = 0;
    for k in (cut.max(0)..nbits).rev() {
        m = (m << 1) | bit(k);
    }
    if cut < 0 {
        m <<= (-cut) as u32;
    }
    if cut > 0 {
        let half = bit(cut - 1);
        let rest = any_below(cut - 1);
        if half == 1 && (rest || m & 1 == 1) {
            m += 1;
        }
    }
    // m * 2^ulp
    let mut e = ulp;
    if m >= 1 << 53 {
        m >>= 1;
        e += 1;
    }
    if m == 0 {
        return sign;
    }
    // pack
    if m < 1 << 52 {
        // denormal (e == -1074)
        return sign | m;
    }
    let be = e + 1075;
    if be >= 0x7FF {
        return sign | EXP;
    }
    sign | ((be as u64) << 52) | (m & FRAC)
}

/// Dolphin NI_mul / NI_div / NI_add / NI_sub: the host result, but a NaN result takes frA's NaN (quieted), else
/// frB's, else the default NaN.
fn ni2(a: u64, b: u64, r: f64) -> u64 {
    if r.is_nan() {
        if is_nan(a) {
            return quiet(a);
        }
        if is_nan(b) {
            return quiet(b);
        }
        return PPC_NAN;
    }
    r.to_bits()
}
/// NI_madd / NI_msub: fma(a, c, +-b); NaN operands are checked in the order a, b, c.
fn ni_fma(a: u64, c: u64, b: u64, sub: bool) -> u64 {
    if is_nan(a) {
        return quiet(a);
    }
    if is_nan(b) {
        return quiet(b);
    }
    if is_nan(c) {
        return quiet(c);
    }
    let r = soft_fma(a, c, if sub { b ^ SIGN } else { b });
    if is_nan(r) { PPC_NAN } else { r }
}
/// ForceSingle (NI = 0): the host double -> float conversion. Fill: back to double.
fn single(x: u64) -> u64 {
    ((d(x) as f32) as f64).to_bits()
}
fn force25(x: u64) -> u64 {
    let e = x & EXP;
    let f = x & FRAC;
    if e == 0 && f != 0 {
        let shift = f.leading_zeros() - 11; // 63 - 52 = 11
        let keep = (0xFFFF_FFFF_F800_0000u64 as i64 >> shift) as u64;
        let round = 0x800_0000u64 >> shift;
        (x & keep).wrapping_add(x & round)
    } else {
        (x & 0xFFFF_FFFF_F800_0000).wrapping_add(x & 0x800_0000)
    }
}
/// stfs: the PowerPC store-single conversion (truncating, the denormal range shifted).
fn to_single_bits(x: u64) -> u32 {
    let e = ((x >> 52) & 0x7FF) as u32;
    if e > 896 || x & !SIGN == 0 {
        (((x >> 32) & 0xC000_0000) | ((x >> 29) & 0x3FFF_FFFF)) as u32
    } else if e >= 874 {
        let mut t = 0x8000_0000u32 | ((x & FRAC) >> 21) as u32;
        t >>= 905 - e;
        t | ((x >> 32) & 0x8000_0000) as u32
    } else {
        (((x >> 32) & 0xC000_0000) | ((x >> 29) & 0x3FFF_FFFF)) as u32
    }
}
/// lfs: the PowerPC load-single conversion (exact, a signalling NaN stays signalling).
fn from_single_bits(v: u32) -> u64 {
    let x = v as u64;
    let e = (x >> 23) & 0xFF;
    let f = x & 0x7F_FFFF;
    if e > 0 && e < 255 {
        let y = ((e >> 7) ^ 1) & 1;
        ((x & 0xC000_0000) << 32) | (y << 61) | (y << 60) | (y << 59) | ((x & 0x3FFF_FFFF) << 29)
    } else if e == 0 && f != 0 {
        let mut ex = 1023u64 - 126;
        let mut fr = f;
        loop {
            fr <<= 1;
            ex -= 1;
            if fr & 0x80_0000 != 0 {
                break;
            }
        }
        ((x & 0x8000_0000) << 32) | (ex << 52) | ((fr & 0x7F_FFFF) << 29)
    } else {
        let y = e >> 7;
        ((x & 0xC000_0000) << 32) | (y << 61) | (y << 60) | (y << 59) | ((x & 0x3FFF_FFFF) << 29)
    }
}

// fres / frsqrte: the hardware estimate tables (Common/FloatUtils.cpp, measured on hardware by Dolphin)
const RSQ: [(i32, i32); 32] = [
    (0x1a7e800, -0x568), (0x17cb800, -0x4f3), (0x1552800, -0x48d), (0x130c000, -0x435), (0x10f2000, -0x3e7), (0x0eff000, -0x3a2),
    (0x0d2e000, -0x365), (0x0b7c000, -0x32e), (0x09e5000, -0x2fc), (0x0867000, -0x2d0), (0x06ff000, -0x2a8), (0x05ab800, -0x283),
    (0x046a000, -0x261), (0x0339800, -0x243), (0x0218800, -0x226), (0x0105800, -0x20b), (0x3ffa000, -0x7a4), (0x3c29000, -0x700),
    (0x38aa000, -0x670), (0x3572000, -0x5f2), (0x3279000, -0x584), (0x2fb7000, -0x524), (0x2d26000, -0x4cc), (0x2ac0000, -0x47e),
    (0x2881000, -0x43a), (0x2665000, -0x3fa), (0x2468000, -0x3c2), (0x2287000, -0x38e), (0x20c1000, -0x35e), (0x1f12000, -0x332),
    (0x1d79000, -0x30a), (0x1bf4000, -0x2e6),
];
const RES: [(i32, i32); 32] = [
    (0x7ff800, 0x3e1), (0x783800, 0x3a7), (0x70ea00, 0x371), (0x6a0800, 0x340), (0x638800, 0x313), (0x5d6200, 0x2ea), (0x579000, 0x2c4),
    (0x520800, 0x2a0), (0x4cc800, 0x27f), (0x47ca00, 0x261), (0x430800, 0x245), (0x3e8000, 0x22a), (0x3a2c00, 0x212), (0x360800, 0x1fb),
    (0x321400, 0x1e5), (0x2e4a00, 0x1d1), (0x2aa800, 0x1be), (0x272c00, 0x1ac), (0x23d600, 0x19b), (0x209e00, 0x18b), (0x1d8800, 0x17c),
    (0x1a9000, 0x16e), (0x17ae00, 0x15b), (0x14f800, 0x15b), (0x124400, 0x143), (0x0fbe00, 0x143), (0x0d3800, 0x12d), (0x0ade00, 0x12d),
    (0x088400, 0x11a), (0x065000, 0x11a), (0x041c00, 0x108), (0x020c00, 0x106),
];
fn frsqrte(x: u64) -> u64 {
    let sign = x & SIGN;
    let mut e = (x >> 52) & 0x7FF;
    let mut m = x & FRAC;
    if e == 0 && m == 0 {
        return sign | EXP;
    }
    if e == 0x7FF {
        if m == 0 {
            return if sign != 0 { PPC_NAN } else { 0 };
        }
        return quiet(x);
    }
    if sign != 0 {
        return PPC_NAN;
    }
    if e == 0 {
        // normalise the denormal
        while m & (1 << 52) == 0 {
            m <<= 1;
            e = e.wrapping_sub(1);
        }
        m &= FRAC;
        e = e.wrapping_add(1);
    }
    let elsb = e & 1;
    // the exponent is halved in place (in its bit field), so an odd difference borrows from it
    let es = (e as i64) << 52;
    let out = ((0x3FFi64 << 52) - ((es - (0x3FEi64 << 52)) / 2)) as u64 & EXP;
    let i = (((elsb << 52) | m) >> 37) as i32;
    let (base, dec) = RSQ[(i / 2048) as usize];
    let frac = (base + dec * (i % 2048)) as i64 as u64;
    out | (frac << 26)
}
fn fres(x: u64) -> u64 {
    let sign = x & SIGN;
    let e = (x >> 52) & 0x7FF;
    let m = x & FRAC;
    if e == 0 && m == 0 {
        return sign | EXP;
    }
    if e == 0x7FF {
        return if m == 0 { sign } else { quiet(x) };
    }
    if e < 895 {
        return sign | 0x47EF_FFFF_E000_0000; // FLT_MAX
    }
    if e >= 1149 {
        return sign;
    }
    let oe = 0x7FD - e;
    let i = (m >> 37) as i32;
    let (base, dec) = RES[(i / 1024) as usize];
    let frac = (base - (dec * (i % 1024) + 1) / 2) as i64 as u64;
    sign | (oe << 52) | (frac << 29)
}
fn fctiw(b: u64, toward_zero: bool) -> u64 {
    let v = d(b);
    let r = if toward_zero { v.trunc() } else { v.round_ties_even() };
    let value: u32 = if v.is_nan() {
        0x8000_0000
    } else if r >= 2147483648.0 {
        0x7FFF_FFFF
    } else if r < -2147483648.0 {
        0x8000_0000
    } else {
        r as i32 as u32
    };
    let mut out = 0xFFF8_0000_0000_0000 | value as u64;
    if value == 0 && b & SIGN != 0 {
        out |= 0x1_0000_0000;
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
struct M {
    gpr: [u32; 32],
    fpr: [u64; 32],
    /// ps1 of each FPR (Dolphin's PairedSingle second slot)
    ps1: [u64; 32],
    gqr: [u32; 8],
    cr: u32,
    xer: u32,
    lr: u32,
    ctr: u32,
    pc: u32,
    mem: Vec<u8>,
    fault: bool,
}

impl M {
    fn field(&mut self, f: u32, v: u32) {
        let sh = 28 - 4 * f;
        self.cr = (self.cr & !(0xF << sh)) | ((v & 0xF) << sh);
    }
    fn cr0(&mut self, v: u32) {
        let so = (self.xer >> 31) & 1;
        let c = if (v as i32) < 0 { 8 } else if v > 0 { 4 } else { 2 };
        self.field(0, c | so);
    }
    fn ca(&mut self, c: bool) {
        self.xer = (self.xer & !(1 << 29)) | ((c as u32) << 29);
    }
    fn get_ca(&self) -> u32 {
        (self.xer >> 29) & 1
    }
    fn bit(&self, i: u32) -> u32 {
        (self.cr >> (31 - i)) & 1
    }
    fn mem_ok(&self, ea: u32, n: u32) -> Option<usize> {
        if ea >= MEM_BASE && (ea as u64 + n as u64) <= MEM_BASE as u64 + MEM_SIZE as u64 { Some((ea - MEM_BASE) as usize) } else { None }
    }
    fn ld(&mut self, ea: u32, n: u32) -> Option<u64> {
        let o = self.mem_ok(ea, n)?;
        let mut v = 0u64;
        for k in 0..n as usize {
            v = (v << 8) | self.mem[o + k] as u64;
        }
        Some(v)
    }
    fn st(&mut self, ea: u32, n: u32, v: u64) -> Option<()> {
        let o = self.mem_ok(ea, n)?;
        for k in 0..n as usize {
            self.mem[o + k] = (v >> (8 * (n as usize - 1 - k))) as u8;
        }
        Some(())
    }

    /// Execute one word at `self.pc` (Dolphin's interpreter, the forms the game uses). None: a memory fault or an
    /// instruction the model does not have.
    fn exec(&mut self, w: u32) -> Option<()> {
        let pc = self.pc;
        self.pc = pc.wrapping_add(4);
        let opcd = w >> 26;
        let rd = (w >> 21) & 31;
        let ra = (w >> 16) & 31;
        let rb = (w >> 11) & 31;
        let rc_ = (w >> 6) & 31;
        let simm = w as u16 as i16 as i32 as u32;
        let uimm = w & 0xFFFF;
        let rc = w & 1 != 0;
        let g = |s: &M, r: u32| s.gpr[r as usize];
        let g0 = |s: &M, r: u32| if r == 0 { 0 } else { s.gpr[r as usize] };
        let carry = |v1: u32, v2: u32| v2 > !v1;
        match opcd {
            14 => self.gpr[rd as usize] = g0(self, ra).wrapping_add(simm),
            15 => self.gpr[rd as usize] = g0(self, ra).wrapping_add(simm << 16),
            7 => self.gpr[rd as usize] = (g(self, ra) as i32).wrapping_mul(simm as i32) as u32,
            8 => {
                let a = g(self, ra);
                self.gpr[rd as usize] = simm.wrapping_sub(a);
                let c = a == 0 || carry(0u32.wrapping_sub(a), simm);
                self.ca(c);
            }
            12 | 13 => {
                let a = g(self, ra);
                let r = a.wrapping_add(simm);
                self.gpr[rd as usize] = r;
                self.ca(carry(a, simm));
                if opcd == 13 {
                    self.cr0(r);
                }
            }
            24 => self.gpr[ra as usize] = g(self, rd) | uimm,
            25 => self.gpr[ra as usize] = g(self, rd) | (uimm << 16),
            26 => self.gpr[ra as usize] = g(self, rd) ^ uimm,
            27 => self.gpr[ra as usize] = g(self, rd) ^ (uimm << 16),
            28 | 29 => {
                let v = g(self, rd) & if opcd == 28 { uimm } else { uimm << 16 };
                self.gpr[ra as usize] = v;
                self.cr0(v);
            }
            20 | 21 | 23 => {
                let (mb, me) = ((w >> 6) & 31, (w >> 1) & 31);
                let mut mask = 0u32;
                // bits mb..me inclusive, wrapping, numbered from the MSB
                let mut i = mb;
                loop {
                    mask |= 0x8000_0000 >> i;
                    if i == me {
                        break;
                    }
                    i = (i + 1) & 31;
                }
                let sh = if opcd == 23 { g(self, rb) & 31 } else { rb };
                let rot = g(self, rd).rotate_left(sh);
                let v = if opcd == 20 { (rot & mask) | (g(self, ra) & !mask) } else { rot & mask };
                self.gpr[ra as usize] = v;
                if rc {
                    self.cr0(v);
                }
            }
            10 | 11 => {
                if (w >> 21) & 1 != 0 {
                    return None;
                }
                let crf = (w >> 23) & 7;
                let a = g(self, ra);
                let (lt, gt) = if opcd == 11 { ((a as i32) < simm as i32, (a as i32) > simm as i32) } else { (a < uimm, a > uimm) };
                let eq = if opcd == 11 { a == simm } else { a == uimm };
                let so = self.xer >> 31;
                self.field(crf, (lt as u32) << 3 | (gt as u32) << 2 | (eq as u32) << 1 | so);
            }
            16 => {
                let bo = rd;
                if bo & 4 == 0 {
                    self.ctr = self.ctr.wrapping_sub(1);
                }
                let tf = (bo >> 3) & 1;
                let ctr_ok = bo & 4 != 0 || (((self.ctr != 0) as u32) ^ ((bo >> 1) & 1)) & 1 != 0;
                let cond_ok = bo & 16 != 0 || self.bit(ra) == tf;
                if ctr_ok && cond_ok {
                    if rc {
                        self.lr = pc.wrapping_add(4);
                    }
                    let bd = ((w & 0xFFFC) as u16 as i16) as i32 as u32;
                    self.pc = if w & 2 != 0 { bd } else { pc.wrapping_add(bd) };
                }
            }
            18 => {
                if rc {
                    self.lr = pc.wrapping_add(4);
                }
                let li = (((w & 0x03FF_FFFC) << 6) as i32 >> 6) as u32;
                self.pc = if w & 2 != 0 { li } else { pc.wrapping_add(li) };
            }
            19 => {
                let xo = (w >> 1) & 0x3FF;
                match xo {
                    16 | 528 => {
                        let bo = rd;
                        if xo == 16 && bo & 4 == 0 {
                            self.ctr = self.ctr.wrapping_sub(1);
                        }
                        let ctr_ok = xo == 528 || bo & 4 != 0 || (((self.ctr != 0) as u32) ^ ((bo >> 1) & 1)) & 1 != 0;
                        let cond_ok = bo & 16 != 0 || self.bit(ra) == (bo >> 3) & 1;
                        if ctr_ok && cond_ok {
                            let t = if xo == 16 { self.lr } else { self.ctr } & !3;
                            if rc {
                                self.lr = pc.wrapping_add(4);
                            }
                            self.pc = t;
                        }
                    }
                    0 => {
                        let s = (w >> 18) & 7;
                        let v = (self.cr >> (28 - 4 * s)) & 0xF;
                        self.field((w >> 23) & 7, v);
                    }
                    33 | 129 | 193 | 225 | 257 | 289 | 417 | 449 => {
                        let (x, y) = (self.bit(ra), self.bit(rb));
                        let r = match xo {
                            33 => !(x | y),
                            129 => x & !y,
                            193 => x ^ y,
                            225 => !(x & y),
                            257 => x & y,
                            289 => !(x ^ y),
                            417 => x | !y,
                            _ => x | y,
                        } & 1;
                        let m = 0x8000_0000u32 >> rd;
                        self.cr = (self.cr & !m) | if r != 0 { m } else { 0 };
                    }
                    _ => return None,
                }
            }
            31 => return self.exec31(w, rd, ra, rb, rc),
            32..=55 => {
                let upd = opcd & 1 == 1;
                let ea = if upd { g(self, ra) } else { g0(self, ra) }.wrapping_add(simm);
                match opcd {
                    32 | 33 => self.gpr[rd as usize] = self.ld(ea, 4)? as u32,
                    34 | 35 => self.gpr[rd as usize] = self.ld(ea, 1)? as u32,
                    40 | 41 => self.gpr[rd as usize] = self.ld(ea, 2)? as u32,
                    42 | 43 => self.gpr[rd as usize] = self.ld(ea, 2)? as u16 as i16 as i32 as u32,
                    36 | 37 => self.st(ea, 4, g(self, rd) as u64)?,
                    38 | 39 => self.st(ea, 1, g(self, rd) as u8 as u64)?,
                    44 | 45 => self.st(ea, 2, g(self, rd) as u16 as u64)?,
                    48 | 49 => {
                        let v = self.ld(ea, 4)? as u32;
                        // lfs: PairedSingle::Fill
                        self.fpr[rd as usize] = from_single_bits(v);
                        self.ps1[rd as usize] = from_single_bits(v);
                    }
                    50 | 51 => self.fpr[rd as usize] = self.ld(ea, 8)?,
                    52 | 53 => {
                        let v = to_single_bits(self.fpr[rd as usize]);
                        self.st(ea, 4, v as u64)?
                    }
                    54 | 55 => {
                        let v = self.fpr[rd as usize];
                        self.st(ea, 8, v)?
                    }
                    46 => {
                        let ea = g0(self, ra).wrapping_add(simm);
                        for (k, r) in (rd..32).enumerate() {
                            self.gpr[r as usize] = self.ld(ea.wrapping_add(4 * k as u32), 4)? as u32;
                        }
                        return Some(());
                    }
                    47 => {
                        let ea = g0(self, ra).wrapping_add(simm);
                        for (k, r) in (rd..32).enumerate() {
                            let v = self.gpr[r as usize];
                            self.st(ea.wrapping_add(4 * k as u32), 4, v as u64)?;
                        }
                        return Some(());
                    }
                    _ => return None,
                }
                if upd {
                    self.gpr[ra as usize] = ea;
                }
            }
            59 | 63 => return self.exec_fp(w, opcd == 59, rd, ra, rb, rc_),
            4 => return self.exec_ps(w, rd, ra, rb, rc_),
            56 | 57 | 60 | 61 => {
                // psq_l / psq_lu / psq_st / psq_stu: 12-bit displacement, W at bit 16 (from the MSB), I at bits 17-19
                let disp = ((w & 0xFFF) ^ 0x800).wrapping_sub(0x800);
                let upd = opcd & 1 == 1;
                let ea = (if upd || ra != 0 { g(self, ra) } else { 0 }).wrapping_add(disp);
                let (wb, i) = ((w >> 15) & 1, (w >> 12) & 7);
                if opcd < 60 { self.psq_ld(ea, rd, i, wb)? } else { self.psq_st(ea, rd, i, wb)? }
                if upd {
                    self.gpr[ra as usize] = ea;
                }
            }
            _ => return None,
        }
        Some(())
    }

    fn exec31(&mut self, w: u32, rd: u32, ra: u32, rb: u32, rc: bool) -> Option<()> {
        let xo = (w >> 1) & 0x3FF;
        let (a, b, s) = (self.gpr[ra as usize], self.gpr[rb as usize], self.gpr[rd as usize]);
        let carry = |v1: u32, v2: u32| v2 > !v1;
        let ca = self.get_ca();
        let mut set_d = |m: &mut M, v: u32| {
            m.gpr[rd as usize] = v;
            if rc {
                m.cr0(v);
            }
        };
        let set_a = |m: &mut M, v: u32| {
            m.gpr[ra as usize] = v;
            if rc {
                m.cr0(v);
            }
        };
        // OE forms (bit 10 of the 10-bit XO) are outside the game's set
        match xo {
            0 | 32 => {
                if (w >> 21) & 1 != 0 {
                    return None;
                }
                let (lt, gt) = if xo == 0 { ((a as i32) < (b as i32), (a as i32) > (b as i32)) } else { (a < b, a > b) };
                let so = self.xer >> 31;
                self.field((w >> 23) & 7, (lt as u32) << 3 | (gt as u32) << 2 | ((a == b) as u32) << 1 | so);
            }
            266 => set_d(self, a.wrapping_add(b)),
            40 => set_d(self, b.wrapping_sub(a)),
            104 => set_d(self, a.wrapping_neg()),
            235 => set_d(self, (a as i32).wrapping_mul(b as i32) as u32),
            75 => set_d(self, ((a as i32 as i64 * b as i32 as i64) >> 32) as u32),
            11 => set_d(self, ((a as u64 * b as u64) >> 32) as u32),
            491 => {
                let ov = b == 0 || (a == 0x8000_0000 && b == 0xFFFF_FFFF);
                let v = if ov { if (a as i32) < 0 { 0xFFFF_FFFF } else { 0 } } else { ((a as i32) / (b as i32)) as u32 };
                set_d(self, v)
            }
            459 => set_d(self, if b == 0 { 0 } else { a / b }),
            8 => {
                // subfc: ~a + b + 1 (Dolphin: CA = ~a == -1 || carry(b, ~a + 1))
                let r = b.wrapping_sub(a);
                self.ca(!a == 0xFFFF_FFFF || carry(b, (!a).wrapping_add(1)));
                set_d(self, r)
            }
            136 => {
                let na = !a;
                let r = na.wrapping_add(b).wrapping_add(ca);
                self.ca(carry(na, b) || carry(na.wrapping_add(b), ca));
                set_d(self, r)
            }
            200 => {
                let na = !a;
                self.ca(carry(na, ca));
                set_d(self, na.wrapping_add(ca))
            }
            232 => {
                // Dolphin: Helper_Carry(~a, carry - 1)
                let na = !a;
                self.ca(carry(na, ca.wrapping_sub(1)));
                set_d(self, na.wrapping_add(0xFFFF_FFFF).wrapping_add(ca))
            }
            202 => {
                self.ca(carry(a, ca));
                set_d(self, a.wrapping_add(ca))
            }
            10 => {
                self.ca(carry(a, b));
                set_d(self, a.wrapping_add(b))
            }
            138 => {
                self.ca(carry(a, b) || carry(a.wrapping_add(b), ca));
                set_d(self, a.wrapping_add(b).wrapping_add(ca))
            }
            234 => {
                self.ca(carry(a, ca.wrapping_sub(1)));
                set_d(self, a.wrapping_add(0xFFFF_FFFF).wrapping_add(ca))
            }
            28 => set_a(self, s & b),
            60 => set_a(self, s & !b),
            444 => set_a(self, s | b),
            412 => set_a(self, s | !b),
            316 => set_a(self, s ^ b),
            124 => set_a(self, !(s | b)),
            476 => set_a(self, !(s & b)),
            284 => set_a(self, !(s ^ b)),
            24 => set_a(self, if b & 0x20 != 0 { 0 } else { s << (b & 31) }),
            536 => set_a(self, if b & 0x20 != 0 { 0 } else { s >> (b & 31) }),
            792 => {
                if b & 0x20 != 0 {
                    let neg = s & 0x8000_0000 != 0;
                    self.ca(neg);
                    set_a(self, if neg { 0xFFFF_FFFF } else { 0 })
                } else {
                    let n = b & 31;
                    let r = ((s as i32) >> n) as u32;
                    self.ca((s as i32) < 0 && n > 0 && (s << ((32 - n) & 31)) != 0);
                    set_a(self, r)
                }
            }
            824 => {
                let n = rb;
                let r = ((s as i32) >> n) as u32;
                self.ca((s as i32) < 0 && n > 0 && (s << ((32 - n) & 31)) != 0);
                set_a(self, r)
            }
            26 => set_a(self, s.leading_zeros()),
            954 => set_a(self, s as i8 as i32 as u32),
            922 => set_a(self, s as i16 as i32 as u32),
            19 => self.gpr[rd as usize] = self.cr,
            144 => {
                let crm = (w >> 12) & 0xFF;
                let mut mask = 0u32;
                for i in 0..8 {
                    if crm & (1 << i) != 0 {
                        mask |= 0xF << (4 * i);
                    }
                }
                self.cr = (self.cr & !mask) | (s & mask);
            }
            339 | 467 => {
                let spr = ra | (rb << 5);
                if xo == 339 {
                    self.gpr[rd as usize] = match spr {
                        1 => self.xer,
                        8 => self.lr,
                        9 => self.ctr,
                        _ => return None,
                    };
                } else {
                    match spr {
                        // SetXER keeps CA, SO, OV, the byte count and the compare byte
                        1 => self.xer = s & 0xE000_FF7F,
                        8 => self.lr = s,
                        9 => self.ctr = s,
                        _ => return None,
                    }
                }
            }
            54 | 86 | 150 | 246 | 278 | 470 | 598 | 854 | 982 => {}
            1014 => {
                let ea = (if ra == 0 { 0 } else { a }).wrapping_add(b) & !31;
                for k in 0..8 {
                    self.st(ea + 4 * k, 4, 0)?;
                }
            }
            983 => {
                let ea = (if ra == 0 { 0 } else { a }).wrapping_add(b);
                let v = self.fpr[rd as usize] as u32;
                self.st(ea, 4, v as u64)?;
            }
            _ => {
                // indexed loads and stores
                let (n, kind) = match xo {
                    23 | 55 => (4, 'l'),
                    87 | 119 => (1, 'l'),
                    279 | 311 => (2, 'l'),
                    343 | 375 => (2, 'a'),
                    790 => (2, 'r'),
                    534 => (4, 'r'),
                    151 | 183 => (4, 's'),
                    215 | 247 => (1, 's'),
                    407 | 439 => (2, 's'),
                    662 => (4, 'R'),
                    918 => (2, 'R'),
                    535 | 567 => (4, 'f'),
                    599 | 631 => (8, 'f'),
                    663 | 695 => (4, 'F'),
                    727 | 759 => (8, 'F'),
                    _ => return None,
                };
                let upd = matches!(xo, 55 | 119 | 311 | 375 | 183 | 247 | 439 | 567 | 631 | 695 | 759);
                let ea = (if upd || ra != 0 { a } else { 0 }).wrapping_add(b);
                match kind {
                    'l' => self.gpr[rd as usize] = self.ld(ea, n)? as u32,
                    'a' => self.gpr[rd as usize] = self.ld(ea, n)? as u16 as i16 as i32 as u32,
                    'r' => {
                        let v = self.ld(ea, n)?;
                        self.gpr[rd as usize] = if n == 2 { (v as u16).swap_bytes() as u32 } else { (v as u32).swap_bytes() };
                    }
                    's' => self.st(ea, n, s as u64 & ((1u64 << (8 * n)) - 1))?,
                    'R' => self.st(ea, n, if n == 2 { (s as u16).swap_bytes() as u64 } else { s.swap_bytes() as u64 })?,
                    'f' => {
                        let v = self.ld(ea, n)?;
                        self.fpr[rd as usize] = if n == 4 { from_single_bits(v as u32) } else { v };
                        if n == 4 {
                            self.ps1[rd as usize] = from_single_bits(v as u32);
                        }
                    }
                    _ => {
                        let f = self.fpr[rd as usize];
                        self.st(ea, n, if n == 4 { to_single_bits(f) as u64 } else { f })?
                    }
                }
                if upd {
                    self.gpr[ra as usize] = ea;
                }
            }
        }
        Some(())
    }

    fn exec_fp(&mut self, w: u32, s: bool, fd: u32, fa: u32, fb: u32, fc: u32) -> Option<()> {
        let (a, b, c) = (self.fpr[fa as usize], self.fpr[fb as usize], self.fpr[fc as usize]);
        let xo5 = (w >> 1) & 31;
        let out = |x: u64| if s { single(x) } else { x };
        let neg_unless_nan = |x: u64| if is_nan(x) { x } else { x ^ SIGN };
        let r = match xo5 {
            18 => out(ni2(a, b, d(a) / d(b))),
            20 => out(ni2(a, b, d(a) - d(b))),
            21 => out(ni2(a, b, d(a) + d(b))),
            25 => {
                let c = if s { force25(c) } else { c };
                out(ni2(a, c, d(a) * d(c)))
            }
            24 if s => fres(b),
            26 if !s => frsqrte(b),
            23 if !s => {
                if d(a) >= -0.0 {
                    c
                } else {
                    b
                }
            }
            28..=31 => {
                let c = if s { force25(c) } else { c };
                let sub = xo5 == 28 || xo5 == 30;
                let v = out(ni_fma(a, c, b, sub));
                if xo5 >= 30 { neg_unless_nan(v) } else { v }
            }
            _ if !s => {
                let xo = (w >> 1) & 0x3FF;
                match xo {
                    72 => b,
                    40 => b ^ SIGN,
                    264 => b & !SIGN,
                    136 => b | SIGN,
                    12 => single(b),
                    14 => fctiw(b, false),
                    15 => fctiw(b, true),
                    0 | 32 => {
                        let (x, y) = (d(a), d(b));
                        let v = if x.is_nan() || y.is_nan() {
                            1
                        } else if x < y {
                            8
                        } else if x > y {
                            4
                        } else {
                            2
                        };
                        self.field((w >> 23) & 7, v);
                        return Some(());
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        self.fpr[fd as usize] = r;
        // the `s` forms and frsp: Fill (both slots); the double forms: SetPS0
        if s || (w >> 1) & 0x3FF == 12 {
            self.ps1[fd as usize] = r;
        }
        Some(())
    }

    /// Opcode 4, written from Interpreter_Paired.cpp / Interpreter_LoadStorePaired.cpp (Dolphin's table4).
    fn exec_ps(&mut self, w: u32, fd: u32, fa: u32, fb: u32, fc: u32) -> Option<()> {
        if w & 1 != 0 {
            return None; // Rc forms: not modelled (no FPSCR; the game has none)
        }
        let (a0, a1) = (self.fpr[fa as usize], self.ps1[fa as usize]);
        let (b0, b1) = (self.fpr[fb as usize], self.ps1[fb as usize]);
        let (c0, c1) = (self.fpr[fc as usize], self.ps1[fc as usize]);
        let xo5 = (w >> 1) & 31;
        let xo6 = (w >> 1) & 63;
        let xo10 = (w >> 1) & 0x3FF;
        let neg_unless_nan = |x: u64| if is_nan(x) { x } else { x ^ SIGN };
        let add = |x: u64, y: u64| single(ni2(x, y, d(x) + d(y)));
        let set = match xo5 {
            10 => Some((add(a0, b1), single(c1))),
            11 => Some((single(c0), add(a0, b1))),
            12 | 13 => {
                let k = force25(if xo5 == 12 { c0 } else { c1 });
                Some((single(ni2(a0, k, d(a0) * d(k))), single(ni2(a1, k, d(a1) * d(k)))))
            }
            14 | 15 => {
                let k = force25(if xo5 == 14 { c0 } else { c1 });
                Some((single(ni_fma(a0, k, b0, false)), single(ni_fma(a1, k, b1, false))))
            }
            18 => Some((single(ni2(a0, b0, d(a0) / d(b0))), single(ni2(a1, b1, d(a1) / d(b1))))),
            20 => Some((single(ni2(a0, b0, d(a0) - d(b0))), single(ni2(a1, b1, d(a1) - d(b1))))),
            21 => Some((add(a0, b0), add(a1, b1))),
            23 => Some((if d(a0) >= -0.0 { c0 } else { b0 }, if d(a1) >= -0.0 { c1 } else { b1 })),
            24 => Some((fres(b0), fres(b1))),
            26 => Some((single(frsqrte(b0)), single(frsqrte(b1)))),
            25 => {
                let (k0, k1) = (force25(c0), force25(c1));
                Some((single(ni2(a0, k0, d(a0) * d(k0))), single(ni2(a1, k1, d(a1) * d(k1)))))
            }
            28..=31 => {
                let (k0, k1) = (force25(c0), force25(c1));
                let sub = xo5 == 28 || xo5 == 30;
                let (r0, r1) = (single(ni_fma(a0, k0, b0, sub)), single(ni_fma(a1, k1, b1, sub)));
                Some(if xo5 >= 30 { (neg_unless_nan(r0), neg_unless_nan(r1)) } else { (r0, r1) })
            }
            _ => None,
        };
        if let Some((r0, r1)) = set {
            self.fpr[fd as usize] = r0;
            self.ps1[fd as usize] = r1;
            return Some(());
        }
        if matches!(xo6, 6 | 7 | 38 | 39) {
            let upd = xo6 >= 38;
            let ea = (if upd || fa != 0 { self.gpr[fa as usize] } else { 0 }).wrapping_add(self.gpr[fb as usize]);
            let (wb, i) = ((w >> 10) & 1, (w >> 7) & 7);
            if xo6 & 1 == 0 { self.psq_ld(ea, fd, i, wb)? } else { self.psq_st(ea, fd, i, wb)? }
            if upd {
                self.gpr[fa as usize] = ea;
            }
            return Some(());
        }
        let (r0, r1) = match xo10 {
            0 | 32 | 64 | 96 => {
                let (x, y) = if xo10 >= 64 { (d(a1), d(b1)) } else { (d(a0), d(b0)) };
                let v = if x.is_nan() || y.is_nan() { 1 } else if x < y { 8 } else if x > y { 4 } else { 2 };
                self.field((w >> 23) & 7, v);
                return Some(());
            }
            40 => (b0 ^ SIGN, b1 ^ SIGN),
            72 => (b0, b1),
            136 => (b0 | SIGN, b1 | SIGN),
            264 => (b0 & !SIGN, b1 & !SIGN),
            528 => (a0, b0),
            560 => (a0, b1),
            592 => (a1, b0),
            624 => (a1, b1),
            _ => return None,
        };
        self.fpr[fd as usize] = r0;
        self.ps1[fd as usize] = r1;
        Some(())
    }

    fn psq_ld(&mut self, ea: u32, rd: u32, i: u32, wb: u32) -> Option<()> {
        let q = self.gqr[i as usize];
        let (ty, sc) = ((q >> 16) & 7, (q >> 24) & 63);
        let k: f32 = if sc < 32 { (0.5f64).powi(sc as i32) as f32 } else { 2f64.powi(64 - sc as i32) as f32 };
        let one = 1.0f64.to_bits();
        let (r0, r1) = match ty {
            0 if wb == 1 => (from_single_bits(self.ld(ea, 4)? as u32), one),
            0 => {
                let v = self.ld(ea, 8)?;
                (from_single_bits((v >> 32) as u32), from_single_bits(v as u32))
            }
            4..=7 => {
                let n = if ty == 4 || ty == 6 { 1 } else { 2 };
                let val = |raw: u64| -> u64 {
                    let x = match ty {
                        4 => raw as u8 as f32,
                        6 => raw as u8 as i8 as f32,
                        5 => raw as u16 as f32,
                        _ => raw as u16 as i16 as f32,
                    };
                    ((x * k) as f64).to_bits()
                };
                if wb == 1 {
                    (val(self.ld(ea, n)?), one)
                } else {
                    let v = self.ld(ea, 2 * n)?;
                    (val(v >> (8 * n)), val(v))
                }
            }
            _ => return None,
        };
        self.fpr[rd as usize] = r0;
        self.ps1[rd as usize] = r1;
        Some(())
    }

    fn psq_st(&mut self, ea: u32, rs: u32, i: u32, wb: u32) -> Option<()> {
        let q = self.gqr[i as usize];
        let (ty, sc) = (q & 7, (q >> 8) & 63);
        let k: f32 = if sc < 32 { 2f64.powi(sc as i32) as f32 } else { (0.5f64).powi(64 - sc as i32) as f32 };
        let (p0, p1) = (self.fpr[rs as usize], self.ps1[rs as usize]);
        // ConvertToSingleFTZ: like stfs for normals, but a value below the single normal range (exponent <= 896) is a
        // signed zero
        let ftz = |x: u64| -> u64 {
            let e = (x >> 52) & 0x7FF;
            if e > 896 || x << 1 == 0 { (((x >> 32) & 0xC000_0000) | ((x >> 29) & 0x3FFF_FFFF)) & 0xFFFF_FFFF } else { (x >> 32) & 0x8000_0000 }
        };
        match ty {
            0 if wb == 1 => self.st(ea, 4, ftz(p0)),
            0 => self.st(ea, 8, (ftz(p0) << 32) | ftz(p1)),
            4..=7 => {
                let n = if ty == 4 || ty == 6 { 1 } else { 2 };
                let (lo, hi) = match ty {
                    4 => (0f32, 255f32),
                    5 => (0f32, 65535f32),
                    6 => (-128f32, 127f32),
                    _ => (-32768f32, 32767f32),
                };
                let cv = |x: u64| -> u64 {
                    let v = (d(x) as f32) * k;
                    let v = if v < lo { lo } else if hi < v { hi } else { v };
                    // SType(float): truncation; NaN as x86 cvttss2si (0x80000000, low bits 0)
                    let iv = if v.is_nan() { i32::MIN } else { v as i32 };
                    (iv as u32 as u64) & ((1u64 << (8 * n)) - 1)
                };
                if wb == 1 {
                    self.st(ea, n, cv(p0))
                } else {
                    self.st(ea, 2 * n, (cv(p0) << (8 * n)) | cv(p1))
                }
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// the harness

struct NoCalls;
impl Bridge<FlatMem> for NoCalls {
    fn in_blob(&self, _a: u32) -> bool {
        true
    }
    fn call(&mut self, _c: &mut Cpu, _m: &mut FlatMem, t: u32) -> Result<(), Fault> {
        Err(Fault::Unresolved(t))
    }
}

fn run_interp(cpu: &Cpu, mem: &[u8], w: u32) -> M {
    let mut m = Machine::new_gekko(*cpu);
    let mut fm = FlatMem { base: MEM_BASE, bytes: mem.to_vec() };
    let mut br = NoCalls;
    let ip = cpu.pc;
    m.cpu.pc = ip.wrapping_add(4);
    let r = m.exec(&mut fm, &mut br, mex_import::ppc::ir::decode(w, ip), ip, w);
    let c = m.cpu;
    M { gpr: c.gpr, fpr: c.fpr, ps1: c.ps1, gqr: c.gqr, cr: c.cr, xer: c.xer, lr: c.lr, ctr: c.ctr, pc: c.pc, mem: fm.bytes, fault: r.is_err() }
}

fn run_model(cpu: &Cpu, mem: &[u8], w: u32) -> M {
    let mut m = M { gpr: cpu.gpr, fpr: cpu.fpr, ps1: cpu.ps1, gqr: cpu.gqr, cr: cpu.cr, xer: cpu.xer, lr: cpu.lr, ctr: cpu.ctr, pc: cpu.pc, mem: mem.to_vec(), fault: false };
    let snap = m.clone();
    if m.exec(w).is_none() {
        // a fault leaves the architected state as it was (the interpreter's partial update is compared as a fault)
        let mut f = snap;
        f.fault = true;
        return f;
    }
    m
}

/// One instruction form: a name, a word builder, and an operand setup.
struct Form {
    name: &'static str,
    build: Box<dyn Fn(&mut Rng) -> u32>,
    kind: K,
}
#[derive(Clone, Copy, PartialEq)]
enum K {
    Int,
    Fp,
    Mem,
    Branch,
    /// paired-single arithmetic (opcode 4)
    Ps,
    /// psq_l / psq_st (opcodes 56, 57, 60, 61 and opcode 4's psq_*x)
    PsMem,
}

fn x31(xo: u32, rc: bool) -> impl Fn(&mut Rng) -> u32 {
    move |g: &mut Rng| (31 << 26) | ((g.below(32) as u32) << 21) | ((g.below(32) as u32) << 16) | ((g.below(32) as u32) << 11) | (xo << 1) | rc as u32
}

macro_rules! forms {
    ($( $name:literal $kind:ident |$g:ident| $e:expr ;)*) => {
        vec![ $( Form { name: $name, kind: K::$kind, build: Box::new(|$g: &mut Rng| -> u32 { $e }) } ),* ]
    };
}

fn r(g: &mut Rng) -> u32 {
    g.below(32) as u32
}
fn dform(op: u32, g: &mut Rng) -> u32 {
    (op << 26) | (r(g) << 21) | (r(g) << 16) | (g.next() as u32 & 0xFFFF)
}
fn fa(op: u32, xo5: u32, g: &mut Rng) -> u32 {
    (op << 26) | (r(g) << 21) | (r(g) << 16) | (r(g) << 11) | (r(g) << 6) | (xo5 << 1)
}
fn fx(xo: u32, g: &mut Rng) -> u32 {
    (63 << 26) | (r(g) << 21) | (r(g) << 16) | (r(g) << 11) | (xo << 1)
}
/// A memory form with rA a base register in the test memory (not r0, and not rD for loads with update).
fn mform(op: u32, g: &mut Rng, align: u32) -> u32 {
    let d = r(g);
    let mut a = 1 + g.below(31) as u32;
    if a == d {
        a = if d == 31 { 30 } else { d + 1 };
    }
    let disp = ((g.below(0x200) as u32) * align).wrapping_sub(0x100 * align) & 0xFFFF;
    (op << 26) | (d << 21) | (a << 16) | disp
}
fn xmem(xo: u32, g: &mut Rng) -> u32 {
    let d = r(g);
    let mut a = 1 + g.below(31) as u32;
    if a == d {
        a = if d == 31 { 30 } else { d + 1 };
    }
    let mut b = r(g);
    if b == a || b == d {
        b = (a + 7) % 32;
        if b == 0 || b == d {
            b = (a + 11) % 32;
        }
    }
    (31 << 26) | (d << 21) | (a << 16) | (b << 11) | (xo << 1)
}

fn all_forms() -> Vec<Form> {
    let mut v = forms![
        "addi" Int |g| dform(14, g);
        "addis" Int |g| dform(15, g);
        "mulli" Int |g| dform(7, g);
        "subfic" Int |g| dform(8, g);
        "addic" Int |g| dform(12, g);
        "addic." Int |g| dform(13, g);
        "ori" Int |g| dform(24, g);
        "oris" Int |g| dform(25, g);
        "xori" Int |g| dform(26, g);
        "xoris" Int |g| dform(27, g);
        "andi." Int |g| dform(28, g);
        "andis." Int |g| dform(29, g);
        "rlwimi" Int |g| (20 << 26) | (g.next() as u32 & 0x03FF_FFFF);
        "rlwinm" Int |g| (21 << 26) | (g.next() as u32 & 0x03FF_FFFF);
        "rlwnm" Int |g| (23 << 26) | (g.next() as u32 & 0x03FF_FFFF);
        "cmpi" Int |g| (11 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (g.next() as u32 & 0xFFFF);
        "cmpli" Int |g| (10 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (g.next() as u32 & 0xFFFF);
        "cmp" Int |g| (31 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (r(g) << 11);
        "cmpl" Int |g| (31 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (r(g) << 11) | (32 << 1);
        "srawi" Int |g| (31 << 26) | (r(g) << 21) | (r(g) << 16) | (r(g) << 11) | (824 << 1) | (g.below(2) as u32);
        "mfcr" Int |g| (31 << 26) | (r(g) << 21) | (19 << 1);
        "mtcrf" Int |g| (31 << 26) | (r(g) << 21) | ((g.next() as u32 & 0xFF) << 12) | (144 << 1);
        "mcrf" Branch |g| (19 << 26) | ((g.below(8) as u32) << 23) | ((g.below(8) as u32) << 18);
        "mfspr" Int |g| (31 << 26) | (r(g) << 21) | ([1u32, 8, 9][g.below(3) as usize] << 16) | (339 << 1);
        "mtspr" Int |g| (31 << 26) | (r(g) << 21) | ([1u32, 8, 9][g.below(3) as usize] << 16) | (467 << 1);
        "bc" Branch |g| (16 << 26) | (r(g) << 21) | (r(g) << 16) | (g.next() as u32 & 0xFFFF);
        "b" Branch |g| (18 << 26) | (g.next() as u32 & 0x03FF_FFFF);
        "bclr" Branch |g| (19 << 26) | ((r(g) | 4) << 21) | (r(g) << 16) | (16 << 1) | (g.below(2) as u32);
        "bclr-ctr" Branch |g| (19 << 26) | ((r(g) & !4) << 21) | (r(g) << 16) | (16 << 1) | (g.below(2) as u32);
        "bcctr" Branch |g| (19 << 26) | ((r(g) | 4) << 21) | (r(g) << 16) | (528 << 1) | (g.below(2) as u32);
        "crop" Branch |g| (19 << 26) | (r(g) << 21) | (r(g) << 16) | (r(g) << 11) | ([33u32, 129, 193, 225, 257, 289, 417, 449][g.below(8) as usize] << 1);
        "lwz" Mem |g| mform(32, g, 4);
        "lwzu" Mem |g| mform(33, g, 4);
        "lbz" Mem |g| mform(34, g, 1);
        "lbzu" Mem |g| mform(35, g, 1);
        "lhz" Mem |g| mform(40, g, 2);
        "lhzu" Mem |g| mform(41, g, 2);
        "lha" Mem |g| mform(42, g, 2);
        "lhau" Mem |g| mform(43, g, 2);
        "stw" Mem |g| mform(36, g, 4);
        "stwu" Mem |g| mform(37, g, 4);
        "stb" Mem |g| mform(38, g, 1);
        "stbu" Mem |g| mform(39, g, 1);
        "sth" Mem |g| mform(44, g, 2);
        "sthu" Mem |g| mform(45, g, 2);
        "lfs" Mem |g| mform(48, g, 4);
        "lfsu" Mem |g| mform(49, g, 4);
        "lfd" Mem |g| mform(50, g, 8);
        "lfdu" Mem |g| mform(51, g, 8);
        "stfs" Mem |g| mform(52, g, 4);
        "stfsu" Mem |g| mform(53, g, 4);
        "stfd" Mem |g| mform(54, g, 8);
        "stfdu" Mem |g| mform(55, g, 8);
        "lmw" Mem |g| { let d = 20 + g.below(12) as u32; let a = 1 + g.below(d as u64 - 1) as u32; (46 << 26) | (d << 21) | (a << 16) | (((g.below(0x80) as u32) * 4).wrapping_sub(0x100) & 0xFFFF) };
        "stmw" Mem |g| { let d = 20 + g.below(12) as u32; let a = 1 + g.below(31) as u32; (47 << 26) | (d << 21) | (a << 16) | (((g.below(0x80) as u32) * 4).wrapping_sub(0x100) & 0xFFFF) };
        "lwzx" Mem |g| xmem(23, g);
        "lwzux" Mem |g| xmem(55, g);
        "lbzx" Mem |g| xmem(87, g);
        "lhzx" Mem |g| xmem(279, g);
        "lhax" Mem |g| xmem(343, g);
        "lhaux" Mem |g| xmem(375, g);
        "lhbrx" Mem |g| xmem(790, g);
        "lwbrx" Mem |g| xmem(534, g);
        "stwx" Mem |g| xmem(151, g);
        "stbx" Mem |g| xmem(215, g);
        "sthx" Mem |g| xmem(407, g);
        "sthbrx" Mem |g| xmem(918, g);
        "stwbrx" Mem |g| xmem(662, g);
        "lfsx" Mem |g| xmem(535, g);
        "lfdx" Mem |g| xmem(599, g);
        "stfsx" Mem |g| xmem(663, g);
        "stfdx" Mem |g| xmem(727, g);
        "stfiwx" Mem |g| xmem(983, g);
        "dcbz" Mem |g| xmem(1014, g) & !(31 << 21);
        "fdivs" Fp |g| fa(59, 18, g);
        "fsubs" Fp |g| fa(59, 20, g);
        "fadds" Fp |g| fa(59, 21, g);
        "fres" Fp |g| fa(59, 24, g);
        "fmuls" Fp |g| fa(59, 25, g);
        "fmsubs" Fp |g| fa(59, 28, g);
        "fmadds" Fp |g| fa(59, 29, g);
        "fnmsubs" Fp |g| fa(59, 30, g);
        "fnmadds" Fp |g| fa(59, 31, g);
        "fdiv" Fp |g| fa(63, 18, g);
        "fsub" Fp |g| fa(63, 20, g);
        "fadd" Fp |g| fa(63, 21, g);
        "fmul" Fp |g| fa(63, 25, g);
        "fsel" Fp |g| fa(63, 23, g);
        "frsqrte" Fp |g| fa(63, 26, g);
        "fmsub" Fp |g| fa(63, 28, g);
        "fmadd" Fp |g| fa(63, 29, g);
        "fnmsub" Fp |g| fa(63, 30, g);
        "fnmadd" Fp |g| fa(63, 31, g);
        "fmr" Fp |g| fx(72, g);
        "fneg" Fp |g| fx(40, g);
        "fabs" Fp |g| fx(264, g);
        "fnabs" Fp |g| fx(136, g);
        "frsp" Fp |g| fx(12, g);
        "fctiw" Fp |g| fx(14, g);
        "fctiwz" Fp |g| fx(15, g);
        "fcmpu" Fp |g| (63 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (r(g) << 11);
        "fcmpo" Fp |g| (63 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 16) | (r(g) << 11) | (32 << 1);
    ];
    // paired singles: every A-form and X-form of opcode 4 (Rc = 0) and the quantised loads / stores
    let psa: &[(&'static str, u32)] = &[
        ("ps_sum0", 10), ("ps_sum1", 11), ("ps_muls0", 12), ("ps_muls1", 13), ("ps_madds0", 14), ("ps_madds1", 15),
        ("ps_div", 18), ("ps_sub", 20), ("ps_add", 21), ("ps_sel", 23), ("ps_res", 24), ("ps_mul", 25), ("ps_rsqrte", 26),
        ("ps_msub", 28), ("ps_madd", 29), ("ps_nmsub", 30), ("ps_nmadd", 31),
    ];
    for &(name, xo5) in psa {
        v.push(Form { name, kind: K::Ps, build: Box::new(move |g: &mut Rng| fa(4, xo5, g)) });
    }
    let psx: &[(&'static str, u32)] = &[
        ("ps_cmpu0", 0), ("ps_cmpo0", 32), ("ps_neg", 40), ("ps_cmpu1", 64), ("ps_mr", 72), ("ps_cmpo1", 96), ("ps_nabs", 136),
        ("ps_abs", 264), ("ps_merge00", 528), ("ps_merge01", 560), ("ps_merge10", 592), ("ps_merge11", 624),
    ];
    for &(name, xo) in psx {
        v.push(Form { name, kind: K::Ps, build: Box::new(move |g: &mut Rng| (4 << 26) | ((g.below(8) as u32) << 23) | (r(g) << 21) | (r(g) << 16) | (r(g) << 11) | (xo << 1)) });
    }
    // the D-forms: rA a base register (not r0), a displacement in the 12-bit range, W and I random
    let psd = |op: u32| {
        move |g: &mut Rng| {
            let d = r(g);
            let a = 1 + g.below(31) as u32;
            let disp = ((g.below(0x100) as u32) * 4).wrapping_sub(0x200) & 0xFFF;
            (op << 26) | (d << 21) | (a << 16) | ((g.below(2) as u32) << 15) | ((g.below(8) as u32) << 12) | disp
        }
    };
    v.push(Form { name: "psq_l", kind: K::PsMem, build: Box::new(psd(56)) });
    v.push(Form { name: "psq_lu", kind: K::PsMem, build: Box::new(psd(57)) });
    v.push(Form { name: "psq_st", kind: K::PsMem, build: Box::new(psd(60)) });
    v.push(Form { name: "psq_stu", kind: K::PsMem, build: Box::new(psd(61)) });
    let psxm = |xo6: u32| {
        move |g: &mut Rng| {
            let d = r(g);
            let a = 1 + g.below(31) as u32;
            let mut b = r(g);
            if b == a {
                b = (a + 7) % 32;
            }
            (4 << 26) | (d << 21) | (a << 16) | (b << 11) | ((g.below(2) as u32) << 10) | ((g.below(8) as u32) << 7) | (xo6 << 1)
        }
    };
    v.push(Form { name: "psq_lx", kind: K::PsMem, build: Box::new(psxm(6)) });
    v.push(Form { name: "psq_stx", kind: K::PsMem, build: Box::new(psxm(7)) });
    v.push(Form { name: "psq_lux", kind: K::PsMem, build: Box::new(psxm(38)) });
    v.push(Form { name: "psq_stux", kind: K::PsMem, build: Box::new(psxm(39)) });
    // the X-form integer arithmetic and logic, with and without Rc
    let x: &[(&'static str, &'static str, u32)] = &[
        ("add", "add.", 266), ("subf", "subf.", 40), ("neg", "neg.", 104), ("mullw", "mullw.", 235), ("mulhw", "mulhw.", 75),
        ("mulhwu", "mulhwu.", 11), ("divw", "divw.", 491), ("divwu", "divwu.", 459), ("subfc", "subfc.", 8), ("subfe", "subfe.", 136),
        ("subfze", "subfze.", 200), ("subfme", "subfme.", 232), ("addze", "addze.", 202), ("addc", "addc.", 10), ("adde", "adde.", 138),
        ("addme", "addme.", 234), ("and", "and.", 28), ("andc", "andc.", 60), ("or", "or.", 444), ("orc", "orc.", 412), ("xor", "xor.", 316),
        ("nor", "nor.", 124), ("nand", "nand.", 476), ("eqv", "eqv.", 284), ("slw", "slw.", 24), ("srw", "srw.", 536), ("sraw", "sraw.", 792),
        ("cntlzw", "cntlzw.", 26), ("extsb", "extsb.", 954), ("extsh", "extsh.", 922),
    ];
    for &(n, nr, xo) in x {
        for (name, rc) in [(n, false), (nr, true)] {
            v.push(Form { name, kind: K::Int, build: Box::new(x31(xo, rc)) });
        }
    }
    v
}

/// Disagreements kept on purpose (notes/referee.md section 1): Dolphin's interpreter computes addme / subfme's carry
/// as `Helper_Carry(a, carry - 1)`, which gives CA = 0 when CA was 1; the architecture (and Dolphin's JIT, which uses
/// the host's adc / sbb, and which ran the replays) give CA = 1 there. The interpreter keeps the architected carry.
const KNOWN: &[&str] = &["addme", "addme.", "subfme", "subfme."];

fn setup(g: &mut Rng, kind: K, w: u32) -> (Cpu, Vec<u8>) {
    let mut c = Cpu::default();
    for i in 0..32 {
        c.gpr[i] = g.int();
        c.fpr[i] = g.fp();
        c.ps1[i] = g.fp();
    }
    // GQRs: valid ld/st types (0 float, 4..7 integers) and scales 0..63; GQR0 = 0 half the time (the PSMTX case)
    for i in 0..8 {
        let ty = |g: &mut Rng| [0u32, 4, 5, 6, 7][g.below(5) as usize];
        c.gqr[i] = ty(g) | ((g.below(64) as u32) << 8) | (ty(g) << 16) | ((g.below(64) as u32) << 24);
    }
    if g.coin(2) {
        c.gqr[0] = 0;
    }
    // paired arithmetic: related operands in ps1 too, and ps values that are singles (what psq_l / lfs leave)
    if kind == K::Ps {
        for i in 0..32 {
            if g.coin(2) {
                c.fpr[i] = ((f64::from_bits(c.fpr[i]) as f32) as f64).to_bits();
                c.ps1[i] = ((f64::from_bits(c.ps1[i]) as f32) as f64).to_bits();
            }
        }
        let (a, b) = g.fp_pair();
        c.ps1[((w >> 16) & 31) as usize] = a;
        c.ps1[((w >> 11) & 31) as usize] = b;
    }
    // FP forms: sometimes related operands (cancellation, equal values)
    if kind == K::Fp || kind == K::Ps {
        let (a, b) = g.fp_pair();
        c.fpr[((w >> 16) & 31) as usize] = a;
        c.fpr[((w >> 11) & 31) as usize] = b;
    }
    c.cr = g.next() as u32;
    // XER: only the bits the machine keeps (SO, OV, CA); mtxer of anything is exercised by the mtspr form
    c.xer = g.next() as u32 & 0xE000_0000;
    c.lr = if g.coin(3) { g.next() as u32 } else { 0x8000_4000 + 4 * g.below(64) as u32 };
    c.ctr = match g.below(4) {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => g.next() as u32,
    };
    c.pc = 0x8000_3000 + 4 * g.below(0x100) as u32;
    let mut mem = vec![0u8; if kind == K::Mem || kind == K::PsMem { MEM_SIZE } else { 0 }];
    for k in (0..mem.len()).step_by(8) {
        let v = match g.below(3) {
            0 => g.fp(),
            1 => ((single_word(g) as u64) << 32) | (single_word(g) as u64),
            _ => g.next(),
        };
        mem[k..k + 8].copy_from_slice(&v.to_be_bytes());
    }
    if kind == K::Mem || kind == K::PsMem {
        // base / index registers into the test memory
        let ra = ((w >> 16) & 31) as usize;
        let rb = ((w >> 11) & 31) as usize;
        c.gpr[ra] = MEM_BASE + 0x800;
        if ra != rb && (w >> 26 == 31 || w >> 26 == 4) {
            c.gpr[rb] = (8 * g.below(0x200) as u32).wrapping_sub(0x800);
        }
    }
    (c, mem)
}

fn single_word(g: &mut Rng) -> u32 {
    let s = (g.below(2) as u32) << 31;
    match g.below(6) {
        0 => s,
        1 => s | 0x7F80_0000,
        2 => s | 0x7FC0_0000 | (g.next() as u32 & 0x3F_FFFF),
        3 => s | 0x7F80_0000 | ((g.next() as u32 & 0x3F_FFFF) | 1),
        4 => s | (g.next() as u32 & 0x7F_FFFF).max(1),
        _ => g.next() as u32,
    }
}

fn describe(c: &Cpu, w: u32, a: &M, b: &M) -> String {
    let mut s = format!("word {w:08X} ({:?})", mex_import::ppc::ir::decode(w, c.pc));
    for i in 0..32 {
        if a.gpr[i] != b.gpr[i] {
            s += &format!(" r{i}: in {:08X} interp {:08X} model {:08X};", c.gpr[i], a.gpr[i], b.gpr[i]);
        }
        if a.fpr[i] != b.fpr[i] {
            s += &format!(" f{i}: in {:016X} interp {:016X} model {:016X};", c.fpr[i], a.fpr[i], b.fpr[i]);
        }
        if a.ps1[i] != b.ps1[i] {
            s += &format!(" ps1 f{i}: in {:016X} interp {:016X} model {:016X};", c.ps1[i], a.ps1[i], b.ps1[i]);
        }
    }
    let regs = |w: u32| -> String {
        let (ra, rb, rc) = ((w >> 16) & 31, (w >> 11) & 31, (w >> 6) & 31);
        format!("[fa {:016X} fb {:016X} fc {:016X} ra {:08X} rb {:08X}]", c.fpr[ra as usize], c.fpr[rb as usize], c.fpr[rc as usize], c.gpr[ra as usize], c.gpr[rb as usize])
    };
    if a.cr != b.cr {
        s += &format!(" cr: in {:08X} interp {:08X} model {:08X} {};", c.cr, a.cr, b.cr, regs(w));
    }
    if a.xer != b.xer {
        s += &format!(" xer: in {:08X} interp {:08X} model {:08X} {};", c.xer, a.xer, b.xer, regs(w));
    }
    if a.lr != b.lr || a.ctr != b.ctr || a.pc != b.pc {
        s += &format!(" lr {:08X}/{:08X} ctr {:08X}/{:08X} pc {:08X}/{:08X} (in lr {:08X} ctr {:08X} cr {:08X});", a.lr, b.lr, a.ctr, b.ctr, a.pc, b.pc, c.lr, c.ctr, c.cr);
    }
    if a.fault != b.fault {
        s += &format!(" fault interp {} model {};", a.fault, b.fault);
    }
    if let Some(k) = (0..a.mem.len().min(b.mem.len())).find(|&k| a.mem[k] != b.mem[k]) {
        let k = k & !7;
        s += &format!(" mem {:08X}: interp {:02X?} model {:02X?} {};", MEM_BASE + k as u32, &a.mem[k..k + 8], &b.mem[k..k + 8], regs(w));
    }
    s
}

#[test]
fn soft_fma_matches_host_fma_on_exact_cases() {
    // sanity of the model's fma: cases with a known exact answer
    let one = 1.0f64.to_bits();
    assert_eq!(soft_fma(one, one, one), 2.0f64.to_bits());
    assert_eq!(soft_fma(3.0f64.to_bits(), 0.5f64.to_bits(), (-1.5f64).to_bits()), 0);
    // 1 + 2^-52 squared - (1 + 2^-51) = 2^-104 exactly (the product's low part, lost by an unfused multiply)
    let x = (1.0f64 + f64::EPSILON).to_bits();
    let y = (-(1.0f64 + 2.0 * f64::EPSILON)).to_bits();
    assert_eq!(d(soft_fma(x, x, y)), 2f64.powi(-104));
    // a denormal result
    assert_eq!(soft_fma(f64::MIN_POSITIVE.to_bits(), 0.5f64.to_bits(), 0), (f64::MIN_POSITIVE / 2.0).to_bits());
}

#[test]
fn gekko_vs_dolphin_model() {
    let n: u64 = std::env::var("REFEREE_FUZZ").ok().and_then(|v| v.parse().ok()).unwrap_or(20_000);
    let mut g = Rng(0x5EED_0F_D0_1F_1A);
    let mut bad = vec![];
    let mut total = 0u64;
    let forms = all_forms();
    for f in &forms {
        let mut diffs = 0u64;
        let mut examples = vec![];
        for _ in 0..n {
            let w = (f.build)(&mut g);
            let (cpu, mem) = setup(&mut g, f.kind, w);
            let a = run_interp(&cpu, &mem, w);
            let b = run_model(&cpu, &mem, w);
            // a faulted interpreter step may have changed state before the fault; compare only the fault itself
            let same = if a.fault && b.fault { true } else { a == b };
            if !same {
                diffs += 1;
                if examples.len() < 4 {
                    examples.push(describe(&cpu, w, &a, &b));
                }
            }
        }
        total += n;
        let known = KNOWN.contains(&f.name);
        println!("gekko_trust: {:<9} {n} sets, {diffs} disagreements{}", f.name, if known && diffs > 0 { " (known, kept)" } else { "" });
        for e in &examples {
            println!("gekko_trust:   {e}");
        }
        if diffs > 0 && !known {
            bad.push((f.name, diffs));
        }
    }
    println!("gekko_trust: {} forms, {total} operand sets", forms.len());
    assert!(bad.is_empty(), "disagreements with the Dolphin model: {bad:?}");
}

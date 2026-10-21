//! Bit-for-bit check against the C/C++ source compiled natively (oracle/build.sh ->
//! tests/data/pad_oracle.txt): Aurora's PADClamp family over every input, and
//! gc_adapter_decode.h over a 4000-report stream with periodic recalibration.
use melee_platform::adapter::decode::{gc_decode, Calibration};
use melee_platform::pad::*;

struct Section {
    lines: Vec<String>,
}
impl Section {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }
    fn line(&mut self, s: String) {
        self.lines.push(s);
    }
    fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for l in &self.lines {
            for &c in l.as_bytes().iter().chain(b"\n") {
                h ^= c as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }
}

fn expected() -> std::collections::HashMap<String, (usize, u64, Vec<String>)> {
    let text = include_str!("data/pad_oracle.txt");
    let mut m = std::collections::HashMap::new();
    let mut shown = Vec::new();
    for l in text.lines() {
        if l.starts_with('#') {
            continue;
        }
        if let Some(s) = l.strip_prefix("  ") {
            shown.push(s.to_string());
            continue;
        }
        let f: Vec<&str> = l.split(' ').collect();
        m.insert(
            f[0].to_string(),
            (f[1].parse().unwrap(), u64::from_str_radix(f[2], 16).unwrap(), std::mem::take(&mut shown)),
        );
    }
    m
}

fn check(name: &str, s: &Section) {
    let e = expected();
    let (n, h, shown) = &e[name];
    for (i, l) in shown.iter().enumerate() {
        assert_eq!(&s.lines[i], l, "{name} line {i}");
    }
    assert_eq!(s.lines.len(), *n, "{name} count");
    assert_eq!(s.hash(), *h, "{name} hash");
}

#[test]
fn clamp_stick_all_inputs() {
    use clamp_region::*;
    let mut s = Section::new();
    for r in 0..2 {
        for x in -128i32..128 {
            for y in -128i32..128 {
                let (mut px, mut py) = (x as i8, y as i8);
                if r == 0 {
                    clamp_stick(&mut px, &mut py, MAX_STICK, XY_STICK, MIN_STICK);
                } else {
                    clamp_stick(&mut px, &mut py, MAX_SUBSTICK, XY_SUBSTICK, MIN_SUBSTICK);
                }
                s.line(format!("{r} {x} {y} {px} {py}"));
            }
        }
    }
    check("clamp_stick", &s);
}

#[test]
fn clamp_circle_all_inputs() {
    use clamp_region::*;
    let mut s = Section::new();
    for r in 0..2 {
        for x in -128i32..128 {
            for y in -128i32..128 {
                let (mut px, mut py) = (x as i8, y as i8);
                if r == 0 {
                    clamp_circle(&mut px, &mut py, RAD_STICK, MIN_STICK);
                } else {
                    clamp_circle(&mut px, &mut py, RAD_SUBSTICK, MIN_SUBSTICK);
                }
                s.line(format!("{r} {x} {y} {px} {py}"));
            }
        }
    }
    check("clamp_circle", &s);
}

#[test]
fn clamp_trigger_all_inputs() {
    use clamp_region::*;
    let mut s = Section::new();
    for t in 0..256u32 {
        let mut v = t as u8;
        clamp_trigger(&mut v, MIN_TRIGGER, MAX_TRIGGER);
        s.line(format!("{t} {v}"));
    }
    check("clamp_trigger", &s);
}

fn st_str(s: &PADStatus) -> String {
    format!(
        "{:04x} {} {} {} {} {} {} {} {} {}",
        s.button, s.stickX, s.stickY, s.substickX, s.substickY, s.triggerLeft, s.triggerRight, s.analogA, s.analogB, s.err
    )
}

#[test]
fn adapter_decode_stream() {
    let mut rng: u32 = 12345;
    let mut next = || {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        rng >> 8
    };
    let mut cal = Calibration::default();
    let mut s = Section::new();
    for f in 0..4000 {
        let mut snap = [0u8; 37];
        snap[0] = 0x21;
        for b in snap.iter_mut().skip(1) {
            *b = next() as u8;
        }
        for c in 0..4 {
            let k = next() % 8;
            // C evaluates the conditional's operands left to right; next() inside it runs only for k >= 6.
            let ty = if k < 3 { 1 } else if k < 5 { 2 } else if k == 5 { 0 } else { next() % 16 };
            snap[1 + c * 9] = ((ty << 4) | (next() & 15)) as u8;
            if next() % 4 != 0 {
                snap[2 + c * 9] &= next() as u8;
                snap[3 + c * 9] &= 0x0F;
            }
        }
        let recal = f % 97 == 0;
        let mut st = [pad_disconnected(); 4];
        let any = gc_decode(&mut cal, &snap, &mut st, recal);
        let mut l = format!("{f} {}", any as i32);
        for c in &st {
            l += " | ";
            l += &st_str(c);
        }
        s.line(l);
    }
    check("decode", &s);
}

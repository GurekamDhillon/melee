//! Slippi `.slp` reading (port of pc/platform/gw_replay.c `rp_parse` + the scene derivation of
//! `rp_load`) and the matching writer for the port's own recordings (`MELEE_SLP_RECORD`, the
//! `rec_*` functions). A `.gdlp` is the same event stream in a GD container, see [`crate::gdlp`].
//!
//! What a replay carries and what is done with it is documented at the top of gw_replay.c; this
//! module only turns bytes into the structures playback consumes, with the C's quirks:
//!
//! * the `raw[$U#l` block is found by scanning, a zero length means "still being written";
//! * the first pass sizes the frame range from Pre-Frame events (a first frame of exactly 0 resets
//!   the range on the next event, because the C tests `first == 0 && last == 0`), the second pass
//!   fills inputs;
//! * events with a payload shorter than the version the field needs read as 0 (older Slippi).

pub const GAME_INFO: usize = 0x138;
pub const SLOTS: usize = 8;
pub const FIRST_FRAME: i32 = -123;

/// The size of the standard Slippi 3.19.1 Game Start payload plus the port's two suffixes
/// (`GDT1` match rules, `GDC1` fighter controls).
pub const STANDARD_START: usize = 0x2F8;
pub const RULES_START: usize = STANDARD_START + 8;
pub const CONTROLS_START: usize = RULES_START + 20;
pub const SZ_START: usize = CONTROLS_START;
pub const SZ_PRE: usize = 0x42;
pub const SZ_POST: usize = 0x54;
pub const SZ_END: usize = 6;
pub const SZ_FSTART: usize = 0xC;

fn be32(p: &[u8]) -> u32 {
    u32::from_be_bytes([p[0], p[1], p[2], p[3]])
}
fn bef(p: &[u8]) -> f32 {
    f32::from_bits(be32(p))
}

// ---- match rules (gw_matchrules.h, gw_replay_matchrules.h) ----------------------------------

pub const TURBO_ON: u32 = 1;
pub const TURBO_NO_STEER: u32 = 0x400;
pub const TURBO_NO_MOVE_LOOP: u32 = 0x800;
pub const TURBO_FRAMES_MASK: u32 = 0xFF00_0000;
pub const TURBO_SUPPORTED: u32 = 0x1FF | TURBO_NO_STEER | TURBO_NO_MOVE_LOOP | TURBO_FRAMES_MASK;
pub const TURBO_DEFAULT_FRAMES: u32 = 30;
pub const TURBO_MAX_FRAMES: u32 = 120;

pub fn turbo_frames(w: u32) -> u32 {
    let f = (w & TURBO_FRAMES_MASK) >> 24;
    if f == 0 {
        TURBO_DEFAULT_FRAMES
    } else {
        f
    }
}

/// `gw_turbo_valid`: 0 (off) is valid; otherwise only supported bits, ON set, window <= 120.
pub fn turbo_valid(w: u32) -> bool {
    if w == 0 {
        return true;
    }
    if w & !TURBO_SUPPORTED != 0 {
        return false;
    }
    if w & TURBO_ON == 0 {
        return false;
    }
    turbo_frames(w) <= TURBO_MAX_FRAMES
}

/// `gw_rp_rules_write`: `event` is the command byte plus payload.
pub fn rules_write(event: &mut [u8], rules: u32) {
    let p = 1 + STANDARD_START;
    event[p..p + 4].copy_from_slice(b"GDT1");
    event[p + 4..p + 8].copy_from_slice(&rules.to_be_bytes());
}

/// `gw_rp_rules_read` (`event` = command byte + payload, `size` = payload bytes). `None` = refuse.
pub fn rules_read(event: &[u8], size: usize) -> Option<u32> {
    if size < STANDARD_START + 4 {
        return Some(0);
    }
    let p = &event[1 + STANDARD_START..];
    if p[0] != b'G' || p[1] != b'D' || p[2] != b'T' {
        return Some(0);
    }
    if p[3] != b'1' || size < RULES_START {
        return None;
    }
    let r = be32(&p[4..]);
    if turbo_valid(r) {
        Some(r)
    } else {
        None
    }
}

pub fn controls_word_valid(w: u32) -> bool {
    w & !0xff3d == 0
}

pub fn controls_write(event: &mut [u8], words: &[u32; 4]) {
    let p = 1 + RULES_START;
    event[p..p + 4].copy_from_slice(b"GDC1");
    for (i, w) in words.iter().enumerate() {
        event[p + 4 + i * 4..p + 8 + i * 4].copy_from_slice(&w.to_be_bytes());
    }
}

/// `gw_rp_controls_read`. `None` = refuse.
pub fn controls_read(event: &[u8], size: usize) -> Option<[u32; 4]> {
    if size <= RULES_START {
        return Some([0; 4]);
    }
    let p = &event[1 + RULES_START..];
    if p[0] != b'G' {
        return Some([0; 4]);
    }
    if size < RULES_START + 4 {
        return None;
    }
    if p[1] != b'D' || p[2] != b'C' {
        return Some([0; 4]);
    }
    if p[3] != b'1' || size < CONTROLS_START {
        return None;
    }
    let mut w = [0u32; 4];
    for (i, o) in w.iter_mut().enumerate() {
        *o = be32(&p[4 + i * 4..]);
        if !controls_word_valid(*o) {
            return None;
        }
    }
    Some(w)
}

// ---- the reader ---------------------------------------------------------------------------------

/// `GwRbInput` as the replay fills it (the fields playback reads).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub lx: f32,
    pub ly: f32,
    pub cx: f32,
    pub cy: f32,
    pub trigger: f32,
    pub buttons: u32,
    pub seed: u32,
    pub raw: [i8; 4],
    pub present: bool,
    pub physical_buttons: u16,
    pub physical_l: f32,
    pub physical_r: f32,
    pub physical_complete: bool,
}

/// The console's post-frame state (parity checking, `MELEE_SLP_PARITY=1`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Post {
    pub x: f32,
    pub y: f32,
    pub facing: f32,
    pub percent: f32,
    pub action: u16,
    pub stocks: u8,
    pub has: bool,
}

#[derive(Clone, Debug)]
pub struct Replay {
    pub version: [u8; 4],
    pub game_info: [u8; GAME_INFO],
    pub seed: u32,
    pub turbo: u32,
    pub controls: [u32; 4],
    pub game_start_seen: bool,
    pub truncated: bool,
    pub first: i32,
    pub last: i32,
    /// `(last - first + 1) * SLOTS`, slot = port * 2 + follower.
    pub inputs: Vec<Input>,
    /// Frame Start (0x3A) seed per frame, Slippi 2.2+.
    pub fs_seed: Vec<u32>,
    pub fs_has: Vec<bool>,
    pub ucf_dashback: [u32; 4],
    pub ucf_shield: [u32; 4],
    pub frozen_ps: bool,
    /// Game Start major scene 8: Slippi online, which forces the seed every frame.
    pub online: bool,
    pub post: Option<Vec<Post>>,
    pub parity_expected: i64,
    /// "mode=vs;at=match;stage=ext:N;pP=ck:K/cC/hu|cpu..." (`rp_load`)
    pub scene: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NoRawBlock,
    BadSizeTable,
    /// An unsupported Turbo match rule word.
    UnsupportedRules(u32),
    BadControls,
    BadRange,
}

impl Replay {
    pub fn input(&self, frame: i32, port: usize, follower: bool) -> Option<&Input> {
        if frame < self.first || frame > self.last || port >= 4 {
            return None;
        }
        self.inputs.get((frame - self.first) as usize * SLOTS + port * 2 + follower as usize)
    }
}

/// `rp_parse` + `rp_load`'s scene string. `want_post` = `MELEE_SLP_PARITY=1`.
pub fn parse(d: &[u8], want_post: bool) -> Result<Replay, ParseError> {
    let n = d.len();
    let mut rp = Replay {
        version: [0; 4],
        game_info: [0; GAME_INFO],
        seed: 0,
        turbo: 0,
        controls: [0; 4],
        game_start_seen: false,
        truncated: false,
        first: 0,
        last: 0,
        inputs: Vec::new(),
        fs_seed: Vec::new(),
        fs_has: Vec::new(),
        ucf_dashback: [0; 4],
        ucf_shield: [0; 4],
        frozen_ps: false,
        online: false,
        post: None,
        parity_expected: 0,
        scene: String::new(),
    };
    let mut sizes = [0u16; 256];
    let mut i = 0;
    while i + 12 < n {
        if &d[i..i + 8] == b"raw[$U#l" {
            break;
        }
        i += 1;
    }
    if i + 12 >= n {
        return Err(ParseError::NoRawBlock);
    }
    let pos = i + 12;
    let mut end = pos + be32(&d[i + 8..]) as usize;
    if end > n {
        rp.truncated = true;
        end = n;
    } else if end == pos {
        end = n;
        rp.truncated = true;
    }
    if d[pos] != 0x35 {
        return Err(ParseError::BadSizeTable);
    }
    if pos + 2 > end || d[pos + 1] < 1 || pos + 1 + d[pos + 1] as usize > end || (d[pos + 1] - 1) % 3 != 0 {
        return Err(ParseError::BadSizeTable);
    }
    let tl = d[pos + 1] as usize;
    let mut k = 0;
    while k + 3 <= tl - 1 {
        sizes[d[pos + 2 + k] as usize] = ((d[pos + 3 + k] as u16) << 8) | d[pos + 4 + k] as u16;
        k += 3;
    }
    let mut have_in = false;
    for pass in 0..2 {
        let mut c = pos + 1 + tl;
        while c < end {
            let cmd = d[c];
            let sz = sizes[cmd as usize] as usize;
            if sz == 0 && cmd != 0x35 {
                break;
            }
            if c + 1 + sz > end {
                rp.truncated = true;
                break;
            }
            let b = &d[c..];
            match cmd {
                0x36 if pass == 0 => {
                    if sz < 0x140 {
                        rp.truncated = true;
                        c += 1 + sz;
                        continue;
                    }
                    match rules_read(b, sz) {
                        Some(r) => rp.turbo = r,
                        None => {
                            // the C leaves rp.turbo = 0 or the word it read; only the refusal matters
                            return Err(ParseError::UnsupportedRules(be32(&b[1 + STANDARD_START + 4..])));
                        }
                    }
                    match controls_read(b, sz) {
                        Some(w) => rp.controls = w,
                        None => return Err(ParseError::BadControls),
                    }
                    rp.game_start_seen = true;
                    rp.version.copy_from_slice(&b[1..5]);
                    rp.game_info.copy_from_slice(&b[5..5 + GAME_INFO]);
                    rp.seed = be32(&b[0x13D..]);
                    rp.online = sz >= 0x1A4 && b[0x1A4] == 8;
                    rp.frozen_ps = sz >= 0x1A2 && b[0x1A2] != 0;
                    if sz >= 0x160 {
                        for p in 0..4 {
                            rp.ucf_dashback[p] = be32(&b[0x141 + 8 * p..]);
                            rp.ucf_shield[p] = be32(&b[0x145 + 8 * p..]);
                        }
                    }
                }
                0x38 if pass == 1 && rp.post.is_some() => {
                    let frame = be32(&b[1..]) as i32;
                    let port = b[5] as usize;
                    let fol = (b[6] != 0) as usize;
                    if frame >= rp.first && frame <= rp.last && port < 4 && sz >= 0x21 {
                        let idx = (frame - rp.first) as usize * SLOTS + port * 2 + fol;
                        let q = &mut rp.post.as_mut().unwrap()[idx];
                        q.action = ((b[8] as u16) << 8) | b[9] as u16;
                        q.x = bef(&b[0x0A..]);
                        q.y = bef(&b[0x0E..]);
                        q.facing = bef(&b[0x12..]);
                        q.percent = bef(&b[0x16..]);
                        q.stocks = b[0x21];
                        if !q.has {
                            q.has = true;
                            rp.parity_expected += 1;
                        }
                    }
                }
                0x3A if pass == 1 => {
                    if sz < 8 {
                        rp.truncated = true;
                        c += 1 + sz;
                        continue;
                    }
                    let frame = be32(&b[1..]) as i32;
                    if frame >= rp.first && frame <= rp.last {
                        let k = (frame - rp.first) as usize;
                        rp.fs_seed[k] = be32(&b[5..]);
                        rp.fs_has[k] = true;
                    }
                }
                0x37 => {
                    if sz < 0x30 {
                        rp.truncated = true;
                        c += 1 + sz;
                        continue;
                    }
                    let frame = be32(&b[1..]) as i32;
                    let port = b[5] as usize;
                    let fol = (b[6] != 0) as usize;
                    if pass == 0 {
                        if rp.first == 0 && rp.last == 0 && !have_in {
                            rp.first = frame;
                            rp.last = frame;
                        }
                        if frame < rp.first {
                            rp.first = frame;
                        }
                        if frame > rp.last {
                            rp.last = frame;
                        }
                    } else if port < 4 && frame >= rp.first && frame <= rp.last {
                        // (the C has no range test: a first frame of 0 followed by a later one
                        // shrinks the range under it and the C writes out of bounds)
                        let idx = (frame - rp.first) as usize * SLOTS + port * 2 + fol;
                        let r = &mut rp.inputs[idx];
                        r.seed = be32(&b[0x7..]);
                        r.lx = bef(&b[0x19..]);
                        r.ly = bef(&b[0x1D..]);
                        r.cx = bef(&b[0x21..]);
                        r.cy = bef(&b[0x25..]);
                        r.trigger = bef(&b[0x29..]);
                        r.buttons = be32(&b[0x2D..]);
                        r.raw[0] = if sz >= 0x3B { b[0x3B] as i8 } else { 0 };
                        r.raw[1] = if sz >= 0x40 { b[0x40] as i8 } else { 0 };
                        r.raw[2] = if sz >= 0x41 { b[0x41] as i8 } else { 0 };
                        r.raw[3] = if sz >= 0x42 { b[0x42] as i8 } else { 0 };
                        if sz >= 0x42 {
                            r.physical_buttons = ((b[0x31] as u16) << 8) | b[0x32] as u16;
                            r.physical_l = bef(&b[0x33..]);
                            r.physical_r = bef(&b[0x37..]);
                            r.physical_complete = true;
                        }
                        r.present = true;
                    }
                }
                _ => {}
            }
            c += 1 + sz;
        }
        if pass == 0 {
            if rp.first > rp.last || rp.first < -123 || (rp.last as i64 - rp.first as i64) > 3_600_000 {
                return Err(ParseError::BadRange);
            }
            let frames = (rp.last - rp.first + 1) as usize;
            rp.inputs = vec![Input::default(); frames * SLOTS];
            rp.fs_seed = vec![0; frames];
            rp.fs_has = vec![false; frames];
            if want_post {
                rp.post = Some(vec![Post::default(); frames * SLOTS]);
            }
            have_in = true;
        }
    }
    // rp_load: the scene the launcher takes
    let gi = &rp.game_info;
    let mut s = format!("mode=vs;at=match;stage=ext:{}", ((gi[0x0E] as u32) << 8) | gi[0x0F] as u32);
    for p in 0..4 {
        let pl = &gi[0x60 + 0x24 * p..];
        if pl[1] == 3 {
            continue;
        }
        s += &format!(";p{}=ck:{}/c{}/{}", p + 1, pl[0], pl[3], if pl[1] == 1 { "cpu" } else { "hu" });
    }
    {
        const KEY: &[u8] = b"\x09stageSkinSU";
        let mut i = 0;
        while i + KEY.len() + 1 + 1 < n {
            if &d[i..i + KEY.len()] == KEY {
                let vl = d[i + KEY.len()] as usize;
                if i + KEY.len() + 1 + vl > n {
                    break;
                }
                let mut tok = String::new();
                let mut k = 0;
                while k < 23 && k < vl && d[i + KEY.len() + 1 + k] != b'|' {
                    tok.push(d[i + KEY.len() + 1 + k] as char);
                    k += 1;
                }
                s += &format!(";skin={tok}");
                break;
            }
            i += 1;
        }
    }
    rp.scene = s;
    Ok(rp)
}

/// FNV-1a 64 helpers for summary lines.
fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &c in b {
        h ^= c as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// The oracle's one-line summary of a parsed file (see oracle/replay_oracle.c).
pub fn summary_line(name: &str, r: &Result<Replay, ParseError>) -> String {
    let Ok(rp) = r else { return format!("{name}	active=0") };
    const BASIS: u64 = 0xcbf29ce484222325;
    let u = |h: u64, v: u32| fnv(h, &v.to_le_bytes());
    let f = |h: u64, v: f32| fnv(h, &v.to_bits().to_le_bytes());
    let (mut hi, mut hp) = (BASIS, BASIS);
    let mut present = 0;
    for (i, r) in rp.inputs.iter().enumerate() {
        hi = f(hi, r.lx);
        hi = f(hi, r.ly);
        hi = f(hi, r.cx);
        hi = f(hi, r.cy);
        hi = f(hi, r.trigger);
        hi = u(hi, r.buttons);
        hi = u(hi, r.seed);
        hi = fnv(hi, &[r.raw[0] as u8, r.raw[1] as u8, r.raw[2] as u8, r.raw[3] as u8]);
        hi = u(hi, r.present as u32);
        hi = u(hi, r.physical_buttons as u32);
        hi = f(hi, r.physical_l);
        hi = f(hi, r.physical_r);
        hi = u(hi, r.physical_complete as u32);
        present += r.present as i32;
        if let Some(post) = &rp.post {
            let q = &post[i];
            hp = f(hp, q.x);
            hp = f(hp, q.y);
            hp = f(hp, q.facing);
            hp = f(hp, q.percent);
            hp = u(hp, q.action as u32);
            hp = u(hp, q.stocks as u32);
            hp = u(hp, q.has as u32);
        }
    }
    let mut hf = BASIS;
    for i in 0..rp.fs_seed.len() {
        hf = u(hf, rp.fs_seed[i]);
        hf = u(hf, rp.fs_has[i] as u32);
    }
    let hg = fnv(BASIS, &rp.game_info);
    let v = rp.version;
    format!(
        "{name}	active=1 ver={}.{}.{}.{} first={} last={} seed={:08x} turbo={:08x} ctl={:08x},{:08x},{:08x},{:08x} online={} frozen={} trunc={} gss={} \
         ucfd={},{},{},{} ucfs={},{},{},{} scene=\"{}\" present={} expected={} gi={:016x} in={:016x} fs={:016x} post={:016x}",
        v[0], v[1], v[2], v[3], rp.first, rp.last, rp.seed, rp.turbo,
        rp.controls[0], rp.controls[1], rp.controls[2], rp.controls[3],
        rp.online as i32, rp.frozen_ps as i32, rp.truncated as i32, rp.game_start_seen as i32,
        rp.ucf_dashback[0], rp.ucf_dashback[1], rp.ucf_dashback[2], rp.ucf_dashback[3],
        rp.ucf_shield[0], rp.ucf_shield[1], rp.ucf_shield[2], rp.ucf_shield[3],
        rp.scene, present, rp.parity_expected, hg, hi, hf, hp
    )
}

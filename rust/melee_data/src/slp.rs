//! Slippi `.slp` reader written from the public spec (project-slippi/slippi-wiki, SPEC.md): the UBJSON `raw`
//! element, the Event Payloads table, Game Start (0x36, with the Game Info Block), Post-Frame Update (0x38) and the
//! Gecko List (0x3D, carried by Message Splitter 0x10). Everything is big-endian. A field is read only when the file's
//! own payload size covers it, which is how the spec handles the version differences (new fields are appended).
//!
//! No game logic here: this is the data half of `melee slpcheck` and of the Game Start -> scene conversion.

/// Offsets inside the Game Start event (command byte at 0), spec "Game Start".
mod gs {
    pub const VERSION: usize = 0x1;
    pub const INFO: usize = 0x5;
    pub const INFO_LEN: usize = 312;
    pub const SEED: usize = 0x13D;
    pub const DASHBACK: usize = 0x141; // + 8*i
    pub const SHIELD_DROP: usize = 0x145; // + 8*i
    pub const PAL: usize = 0x1A1;
    pub const FROZEN_PS: usize = 0x1A2;
    pub const MINOR_SCENE: usize = 0x1A3;
    pub const MAJOR_SCENE: usize = 0x1A4;
}

/// What one player slot of the Game Info Block says (spec "Game Info Block", index = port - 1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerInfo {
    /// External character id (CharacterKind order, 0 = Captain Falcon .. 25 = Ganondorf).
    pub ckind: u8,
    /// 0 human, 1 CPU, 2 demo, 3 empty.
    pub ptype: u8,
    pub stocks: u8,
    pub costume: u8,
    /// `PlayerInitData.spawn_pos` (+5): the spawn point the CSS (or a Neutral Spawn code's CSS half) assigned.
    pub spawn_pos: u8,
    pub team_shade: u8,
    pub handicap: u8,
    pub team: u8,
    pub bitfield: u8,
    pub cpu_level: u8,
    pub damage_start: u16,
    pub damage_spawn: u16,
    pub offense: f32,
    pub defense: f32,
    pub scale: f32,
}

/// The Game Info Block (312 bytes, indexed from its start) as typed accessors.
#[derive(Clone, Debug)]
pub struct GameInfo(pub Vec<u8>);

impl GameInfo {
    fn u8(&self, o: usize) -> u8 {
        self.0.get(o).copied().unwrap_or(0)
    }
    fn u16(&self, o: usize) -> u16 {
        be16(&self.0, o).unwrap_or(0)
    }
    fn u32(&self, o: usize) -> u32 {
        be32(&self.0, o).unwrap_or(0)
    }
    fn f32(&self, o: usize) -> f32 {
        f32::from_bits(self.u32(o))
    }
    /// Game Bitfield 1 bits 1-2: 0 none, 2 counting down, 3 counting up.
    pub fn timer_behavior(&self) -> u8 {
        self.u8(0) & 0x03
    }
    /// Game Bitfield 1 bits 6-8: 0 time, 1 stock, 2 coin, 3 bonus.
    pub fn game_mode(&self) -> u8 {
        self.u8(0) >> 5
    }
    pub fn bitfield2(&self) -> u8 {
        self.u8(1)
    }
    pub fn bitfield3(&self) -> u8 {
        self.u8(2)
    }
    pub fn bomb_rain(&self) -> u8 {
        self.u8(0x6)
    }
    pub fn is_teams(&self) -> bool {
        self.u8(0x8) != 0
    }
    /// -1 off, 0 very low .. 4 very high, 5-8 higher.
    pub fn item_spawn(&self) -> i8 {
        self.u8(0xB) as i8
    }
    pub fn sd_score(&self) -> i8 {
        self.u8(0xC) as i8
    }
    /// External stage id.
    pub fn stage(&self) -> u16 {
        self.u16(0xE)
    }
    /// Timer in seconds.
    pub fn timer_secs(&self) -> u32 {
        self.u32(0x10)
    }
    /// Item Spawn Bitfields 1-5 (0x23..0x27).
    pub fn item_bits(&self) -> [u8; 5] {
        [self.u8(0x23), self.u8(0x24), self.u8(0x25), self.u8(0x26), self.u8(0x27)]
    }
    pub fn damage_ratio(&self) -> f32 {
        self.f32(0x30)
    }
    pub fn player(&self, i: usize) -> PlayerInfo {
        let b = 0x60 + 0x24 * i;
        PlayerInfo {
            ckind: self.u8(b),
            ptype: self.u8(b + 1),
            stocks: self.u8(b + 2),
            costume: self.u8(b + 3),
            spawn_pos: self.u8(b + 5),
            team_shade: self.u8(b + 7),
            handicap: self.u8(b + 8),
            team: self.u8(b + 9),
            bitfield: self.u8(b + 0xC),
            cpu_level: self.u8(b + 0xF),
            damage_start: self.u16(b + 0x10),
            damage_spawn: self.u16(b + 0x12),
            offense: self.f32(b + 0x18),
            defense: self.f32(b + 0x1C),
            scale: self.f32(b + 0x20),
        }
    }
}

/// One Post-Frame Update (0x38). `Option` fields are absent in files whose payload is too short for them.
#[derive(Clone, Copy, Debug, Default)]
pub struct Post {
    pub frame: i32,
    pub port: u8,
    pub follower: bool,
    /// Internal character id.
    pub char_id: u8,
    pub action: u16,
    pub x: f32,
    pub y: f32,
    pub facing: f32,
    pub percent: f32,
    pub shield: f32,
    pub last_attack: u8,
    pub combo: u8,
    pub last_hit_by: u8,
    pub stocks: u8,
    /// 0.2.0
    pub state_frame: Option<f32>,
    /// 2.0.0: State Bit Flags 1-5
    pub flags: Option<[u8; 5]>,
    /// 2.0.0: Misc AS (hitstun remaining while in hitstun)
    pub misc_as: Option<f32>,
    /// 2.0.0: 0 grounded, 1 airborne
    pub airborne: Option<u8>,
    /// 2.0.0
    pub jumps: Option<u8>,
    /// 3.8.0
    pub hitlag: Option<f32>,
}

/// A parsed `.slp`.
#[derive(Clone, Debug, Default)]
pub struct Slp {
    pub version: [u8; 4],
    pub info: Option<GameInfo>,
    pub seed: u32,
    pub dashback: [u32; 4],
    pub shield_drop: [u32; 4],
    pub pal: bool,
    pub frozen_ps: bool,
    pub minor_scene: u8,
    pub major_scene: u8,
    /// The reassembled Gecko List (empty before 3.3.0).
    pub gecko: Vec<u8>,
    /// Payload size per command byte from the Event Payloads event (0 = not listed).
    pub sizes: Vec<u16>,
    pub first_frame: i32,
    pub last_frame: i32,
    pub post: Vec<Post>,
    /// `true` when the raw element ran off the end of the file (a recording cut short).
    pub truncated: bool,
}

impl Slp {
    /// `major*10000 + minor*100 + build`.
    pub fn version_num(&self) -> i32 {
        self.version[0] as i32 * 10000 + self.version[1] as i32 * 100 + self.version[2] as i32
    }
    pub fn version_str(&self) -> String {
        format!("{}.{}.{}", self.version[0], self.version[1], self.version[2])
    }
    /// Post-frame rows of one port's leader fighter, indexed by `frame - first_frame` (None where the frame is missing).
    pub fn post_by_port(&self, port: usize) -> Vec<Option<Post>> {
        let n = (self.last_frame - self.first_frame + 1).max(0) as usize;
        let mut v = vec![None; n];
        for p in &self.post {
            if p.port as usize == port && !p.follower {
                let k = p.frame - self.first_frame;
                if k >= 0 && (k as usize) < n {
                    v[k as usize] = Some(*p);
                }
            }
        }
        v
    }
}

pub fn be16(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes(d.get(o..o + 2)?.try_into().ok()?))
}
pub fn be32(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(d.get(o..o + 4)?.try_into().ok()?))
}
fn bef(d: &[u8], o: usize) -> Option<f32> {
    be32(d, o).map(f32::from_bits)
}

/// The key `raw` and its optimised array header: `U 03 r a w [ $ U # l <u32>`.
const RAW_KEY: &[u8] = b"\x55\x03raw[$U#l";

/// Parse the `raw` element. `want_post` keeps every Post-Frame Update (they dominate the memory).
pub fn parse(d: &[u8], want_post: bool) -> Result<Slp, String> {
    let at = d.windows(RAW_KEY.len()).position(|w| w == RAW_KEY).ok_or("no raw element (not a .slp)")?;
    let len_at = at + RAW_KEY.len();
    let declared = be32(d, len_at).ok_or("truncated header")? as usize;
    let start = len_at + 4;
    // while a game is being written the length is 0: read to the end
    let end = if declared == 0 { d.len() } else { (start + declared).min(d.len()) };
    let mut s = Slp { sizes: vec![0; 256], first_frame: i32::MAX, last_frame: i32::MIN, ..Default::default() };
    s.truncated = declared != 0 && start + declared > d.len();
    let raw = &d[start..end];
    // Event Payloads (0x35) comes first: cmd, size byte (counts itself), then 3-byte (cmd, u16 size) entries
    if raw.first() != Some(&0x35) || raw.len() < 2 {
        return Err("the first event is not Event Payloads (0x35)".into());
    }
    let psize = raw[1] as usize;
    for i in 0..psize.saturating_sub(1) / 3 {
        let o = 2 + 3 * i;
        let cmd = *raw.get(o).ok_or("short Event Payloads")? as usize;
        s.sizes[cmd] = be16(raw, o + 1).ok_or("short Event Payloads")?;
    }
    let mut pos = 1 + psize;
    while pos < raw.len() {
        let cmd = raw[pos];
        let sz = s.sizes[cmd as usize] as usize;
        if sz == 0 {
            // an event the table does not list: the spec gives no way to skip it
            s.truncated = true;
            break;
        }
        if pos + 1 + sz > raw.len() {
            s.truncated = true;
            break;
        }
        let ev = &raw[pos..pos + 1 + sz];
        match cmd {
            0x36 => {
                if ev.len() >= gs::VERSION + 4 {
                    s.version.copy_from_slice(&ev[gs::VERSION..gs::VERSION + 4]);
                }
                if ev.len() >= gs::INFO + gs::INFO_LEN {
                    s.info = Some(GameInfo(ev[gs::INFO..gs::INFO + gs::INFO_LEN].to_vec()));
                }
                s.seed = be32(ev, gs::SEED).unwrap_or(0);
                for i in 0..4 {
                    s.dashback[i] = be32(ev, gs::DASHBACK + 8 * i).unwrap_or(0);
                    s.shield_drop[i] = be32(ev, gs::SHIELD_DROP + 8 * i).unwrap_or(0);
                }
                s.pal = ev.get(gs::PAL).copied().unwrap_or(0) != 0;
                s.frozen_ps = ev.get(gs::FROZEN_PS).copied().unwrap_or(0) != 0;
                s.minor_scene = ev.get(gs::MINOR_SCENE).copied().unwrap_or(0);
                s.major_scene = ev.get(gs::MAJOR_SCENE).copied().unwrap_or(0);
            }
            0x10 => {
                // Message Splitter: 0x1 fixed 512-byte block, 0x201 actual size, 0x203 internal command, 0x204 last message
                if ev.len() >= 0x205 && ev[0x203] == 0x3D {
                    let n = (be16(ev, 0x201).unwrap_or(0) as usize).min(512);
                    s.gecko.extend_from_slice(&ev[1..1 + n]);
                }
            }
            0x37 | 0x38 => {
                if let Some(f) = be32(ev, 1) {
                    let f = f as i32;
                    s.first_frame = s.first_frame.min(f);
                    s.last_frame = s.last_frame.max(f);
                    if cmd == 0x38 && want_post {
                        s.post.push(parse_post(ev));
                    }
                }
            }
            _ => {}
        }
        pos += 1 + sz;
    }
    if s.first_frame == i32::MAX {
        s.first_frame = 0;
        s.last_frame = -1;
    }
    Ok(s)
}

fn parse_post(e: &[u8]) -> Post {
    let u8_ = |o: usize| e.get(o).copied();
    Post {
        frame: be32(e, 0x1).unwrap_or(0) as i32,
        port: u8_(0x5).unwrap_or(0),
        follower: u8_(0x6).unwrap_or(0) != 0,
        char_id: u8_(0x7).unwrap_or(0),
        action: be16(e, 0x8).unwrap_or(0),
        x: bef(e, 0xA).unwrap_or(0.0),
        y: bef(e, 0xE).unwrap_or(0.0),
        facing: bef(e, 0x12).unwrap_or(0.0),
        percent: bef(e, 0x16).unwrap_or(0.0),
        shield: bef(e, 0x1A).unwrap_or(0.0),
        last_attack: u8_(0x1E).unwrap_or(0),
        combo: u8_(0x1F).unwrap_or(0),
        last_hit_by: u8_(0x20).unwrap_or(0),
        stocks: u8_(0x21).unwrap_or(0),
        state_frame: bef(e, 0x22),
        flags: if e.len() > 0x2A { Some([e[0x26], e[0x27], e[0x28], e[0x29], e[0x2A]]) } else { None },
        misc_as: bef(e, 0x2B),
        airborne: u8_(0x2F),
        jumps: u8_(0x32),
        hitlag: bef(e, 0x49),
    }
}

/// One Gecko code as the list stores it (8-byte lines).
#[derive(Clone, Debug)]
pub struct GeckoCode {
    /// Code type byte (`C2` insert asm, `04` write word, ...).
    pub kind: u8,
    /// Target address (`0x80000000 | low 25 bits` of the first word).
    pub addr: u32,
    /// Lines of payload after the header line.
    pub lines: u32,
    /// FNV-1a over the whole code, to group identical codes across files.
    pub hash: u64,
}

/// Split a Gecko List into codes. Only the line layouts of the types Slippi ships are known; an unknown type is
/// taken as a single line.
pub fn gecko_codes(g: &[u8]) -> Vec<GeckoCode> {
    let mut out = Vec::new();
    let mut o = 0;
    while o + 8 <= g.len() {
        let w0 = be32(g, o).unwrap();
        let w1 = be32(g, o + 4).unwrap();
        let kind = (w0 >> 24) as u8;
        if kind == 0xF0 || (w0 == 0 && w1 == 0) {
            break;
        }
        let extra = match kind & 0xFE {
            0xC0 | 0xC2 | 0xF2 | 0xF4 => w1 as usize,
            0x06 => (w1 as usize).div_ceil(8),
            0x08 => 1,
            _ => 0,
        };
        let total = 8 * (1 + extra);
        if o + total > g.len() {
            break;
        }
        let mut h = 0xcbf29ce484222325u64;
        for b in &g[o..o + total] {
            h = (h ^ *b as u64).wrapping_mul(0x100000001b3);
        }
        out.push(GeckoCode { kind, addr: 0x8000_0000 | (w0 & 0x01FF_FFFF), lines: extra as u32, hash: h });
        o += total;
    }
    out
}

/// True when the replay's Gecko list carries Slippi's online codeset (its `Online/Core/BrawlOffscreenDamage` insert at
/// 0x8006A880). Offline games played on a netplay build (vs CPU, local multi) record that codeset too, and its
/// always-on codes (offscreen damage, Whispy, the FD RNG save) apply there; the Game Start scene alone says "console".
pub fn ps_camera_independent_monitor_code(g: &[u8]) -> bool {
    gecko_codes(g).iter().any(|c| c.kind == 0xC2 && c.addr == 0x801D_24FC)
}

/// True when the list carries Slippi's "Preload Stadium Transformations" (its `Load Transformation.asm` insert).
pub fn ps_preload_transformations_code(g: &[u8]) -> bool {
    gecko_codes(g).iter().any(|c| c.kind == 0xC2 && c.addr == 0x801D_45EC)
}

/// Whether the replay played with Slippi's Preload Stadium Transformations: its Gecko list has the code (C2 0x801D45EC),
/// or it has no Gecko list (before Slippi 2.0 a replay did not record one) and is Slippi 1.7.0 to 1.7.x, which recorded
/// with the code on (slippi-ssbm-asm 2019-02, `PSPreloadToggle`; Gang-Steals 1.7.1: 404 Stadium files).
pub fn ps_preload_transformations(f: &Slp) -> bool {
    if f.gecko.is_empty() {
        let v = f.version[0] as u32 * 10000 + f.version[1] as u32 * 100 + f.version[2] as u32;
        return (10700..20000).contains(&v);
    }
    ps_preload_transformations_code(&f.gecko)
}

/// True when the list freezes Pokemon Stadium at grStadium_801D4548's entry (IngameCheckIfFrozen, C2 0x801D457C).
pub fn ps_freeze_at_entry_code(g: &[u8]) -> bool {
    gecko_codes(g).iter().any(|c| c.kind == 0xC2 && c.addr == 0x801D_457C)
}

/// True when the list carries the online codeset (its offscreen-damage insert at 0x8006A880).
pub fn online_codeset(g: &[u8]) -> bool {
    gecko_codes(g).iter().any(|c| c.kind == 0xC2 && c.addr == 0x8006_A880)
}

/// True when the replay was recorded on Akaneia (an m-ex disc): its Gecko list carries Akaneia's own `codes.gct`
/// inserts, of which the one at 0x803D7060 is the marker (byte-identical in every file of Hero's Akaneia corpus, absent
/// from the vanilla Slippi sets). On such a replay the external character ids past 25 are the disc's m-ex fighters
/// (MxDt.dat), not the vanilla hidden characters or Sopo.
pub fn akaneia_codeset(g: &[u8]) -> bool {
    gecko_codes(g).iter().any(|c| c.kind == 0xC2 && c.addr == 0x803D_7060)
}

/// A `04 804D4A08 NNNNNNNN` write in a replay's Gecko list: the code sets `DbLevel` (db.c) to N for the whole run. At
/// `DbLKind_DebugRom` (3) and above `ft_80089118` stops staling moves (Hero's 2026-10-23 set has DbLevel 4: every hit
/// lands its whole-number damage). The last such write wins. None: the list does not touch DbLevel.
pub fn db_level_code(g: &[u8]) -> Option<i32> {
    let mut out = None;
    let mut o = 0;
    while o + 8 <= g.len() {
        let (w0, w1) = (be32(g, o)?, be32(g, o + 4)?);
        let kind = (w0 >> 24) as u8;
        if kind == 0xF0 || (w0 == 0 && w1 == 0) {
            break;
        }
        if kind & 0xFE == 0x04 && 0x8000_0000 | (w0 & 0x01FF_FFFF) == 0x804D_4A08 {
            out = Some(w1 as i32);
        }
        let extra = match kind & 0xFE {
            0xC0 | 0xC2 | 0xF2 | 0xF4 => w1 as usize,
            0x06 => (w1 as usize).div_ceil(8),
            0x08 => 1,
            _ => 0,
        };
        o += 8 * (1 + extra);
    }
    out
}

/// The gameplay-visible part of a widescreen code found in a replay's Gecko list (see [`widescreen_code`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Widescreen {
    /// Camera_80030BBC's on-screen x bounds (`px < left` or `px >= right` is off screen), from its two `li r0,N` writes.
    pub left: u16,
    pub right: u16,
    /// CObjLoad's perspective aspect becomes `(aspect * num) / den` (f32 multiply, then f32 divide).
    pub num: f32,
    pub den: f32,
}

/// Find the widescreen camera code in a replay's Gecko list, worked out from the code bytes themselves:
/// - `04 80030C7C 380000LL` and `04 80030C88 3800RRRR`: Camera_80030BBC's scissor loads (`lhz r0,0x20(r1)` and
///   `lhz r0,0x22(r1)`, the left/right scissor) become constants, so the fighters' on-screen test uses LL..RR;
/// - `C2 8036A4A8` (CObjLoad, in place of the perspective `lfs f1,0x34(r31)`): loads the aspect, then
///   `fmuls f1,f1,f2; fdivs f1,f1,f3` with f2/f3 the two floats the code keeps after its `blrl` (bl-over-data).
/// All three must be present; anything else in the list (name tags, dev text, menus, the magnifier's own constants)
/// does not change logic. Returns None when the list has no such code.
pub fn widescreen_code(g: &[u8]) -> Option<Widescreen> {
    let (mut left, mut right, mut ratio) = (None, None, None);
    let mut o = 0;
    while o + 8 <= g.len() {
        let w0 = be32(g, o)?;
        let w1 = be32(g, o + 4)?;
        let kind = (w0 >> 24) as u8;
        if kind == 0xF0 || (w0 == 0 && w1 == 0) {
            break;
        }
        let extra = match kind & 0xFE {
            0xC0 | 0xC2 | 0xF2 | 0xF4 => w1 as usize,
            0x06 => (w1 as usize).div_ceil(8),
            0x08 => 1,
            _ => 0,
        };
        let total = 8 * (1 + extra);
        if o + total > g.len() {
            break;
        }
        let addr = 0x8000_0000 | (w0 & 0x01FF_FFFF);
        match (kind & 0xFE, addr) {
            (0x04, 0x8003_0C7C) if w1 & 0xFFFF_0000 == 0x3800_0000 => left = Some(w1 as u16),
            (0x04, 0x8003_0C88) if w1 & 0xFFFF_0000 == 0x3800_0000 => right = Some(w1 as u16),
            (0xC2, 0x8036_A4A8) => {
                let words: Vec<u32> = (0..extra * 2).filter_map(|k| be32(g, o + 8 + 4 * k)).collect();
                let has = |w: u32| words.contains(&w);
                // lfs f1,0x34(r31) - fmuls f1,f1,f2 - fdivs f1,f1,f3
                if has(0xC03F_0034) && has(0xEC21_00B2) && has(0xEC21_1824) {
                    if let Some(i) = words.iter().position(|&w| w == 0x4E80_0021) {
                        if i + 2 < words.len() {
                            ratio = Some((f32::from_bits(words[i + 1]), f32::from_bits(words[i + 2])));
                        }
                    }
                }
            }
            _ => {}
        }
        o += total;
    }
    let ((num, den), left, right) = (ratio?, left?, right?);
    Some(Widescreen { left, right, num, den })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_file() {
        // Event Payloads: 0x36 -> 0x40 bytes, 0x38 -> 0x5c bytes
        let mut raw = vec![0x35, 7, 0x36, 0x00, 0x40, 0x38, 0x00, 0x5c];
        let mut gs = vec![0u8; 0x41];
        gs[0] = 0x36;
        gs[1] = 3;
        gs[2] = 17;
        let mut post = vec![0u8; 0x5d];
        post[0] = 0x38;
        post[1..5].copy_from_slice(&(-123i32).to_be_bytes());
        post[0xA..0xE].copy_from_slice(&1.5f32.to_be_bytes());
        raw.extend(gs);
        raw.extend(post);
        let mut d = b"{U\x03raw[$U#l".to_vec();
        d.extend((raw.len() as u32).to_be_bytes());
        d.extend(raw);
        let s = parse(&d, true).unwrap();
        assert_eq!(s.version_str(), "3.17.0");
        assert_eq!(s.post.len(), 1);
        assert_eq!(s.post[0].frame, -123);
        assert_eq!(s.post[0].x, 1.5);
        assert_eq!(s.first_frame, -123);
        assert!(!s.truncated);
    }

    #[test]
    fn short_post_has_no_new_fields() {
        let mut raw = vec![0x35, 4, 0x38, 0x00, 0x21];
        let mut post = vec![0u8; 0x22];
        post[0] = 0x38;
        raw.extend(post);
        let mut d = b"{U\x03raw[$U#l".to_vec();
        d.extend((raw.len() as u32).to_be_bytes());
        d.extend(raw);
        let s = parse(&d, true).unwrap();
        assert!(s.post[0].state_frame.is_none() && s.post[0].hitlag.is_none() && s.post[0].airborne.is_none());
    }
}

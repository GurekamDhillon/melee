//! `.gdlp` ("GD live playback"): the LAB / whole-match recording container (port of the container,
//! reader, header writer, identity check and `.slp` export of pc/platform/gw_script_rec.inc).
//!
//! A `.gdlp` is a Slippi-framed event stream (UBJSON `raw` array, the 0x35 Payloads size table, 0x39
//! Game End, metadata) plus GD events 0x90..0x99 that a Slippi reader skips by their sizes: header,
//! fighters, identity hashes, state reference, per-frame raw pads (LAB takes), marks, segment
//! provenance, text (the mods in words), embedded state chunks and the frozen fighter controls. A
//! whole-match recording made by `MELEE_GDLP_RECORD` also carries Game Start / Frame Start /
//! Pre-Frame / Post-Frame / Bookend, so it is a valid `.slp` as well (and the replay reader plays it).
//! The LAB-only half (recording and playback of raw pads) is game-side and not here.

use crate::replay;

pub const GR_PAD: usize = 64;
pub const GR_VP: usize = 72;
pub const GR_VER: u32 = 3;
pub const GR_FRAME_PAYLOAD: usize = 4 + 1 + GR_PAD + GR_VP;
pub const EV_HEAD: u8 = 0x90;
pub const EV_FIGHTER: u8 = 0x91;
pub const EV_IDENT: u8 = 0x92;
pub const EV_STATE: u8 = 0x93;
pub const EV_FRAME: u8 = 0x94;
pub const EV_MARK: u8 = 0x95;
pub const EV_SEGMENT: u8 = 0x96;
pub const EV_TEXT: u8 = 0x97;
pub const EV_STCHUNK: u8 = 0x98;
pub const EV_CONTROLS: u8 = 0x99;
pub const STCHUNK: usize = 60000;
pub const FL_RETAIL: u32 = 1;
pub const FL_STATE: u32 = 2;
pub const FL_WHOLE: u32 = 4;
pub const FL_SLIPPI: u32 = 8;
pub const HEAD_MAX: usize = 700;

/// Payload size of an event (bytes after the command byte), -1 when unknown.
pub fn size_of(cmd: u8) -> i32 {
    match cmd {
        0x39 => 6,
        EV_HEAD => 16,
        EV_FIGHTER => 30,
        EV_IDENT => 40,
        EV_STATE => 44,
        EV_FRAME => GR_FRAME_PAYLOAD as i32,
        EV_MARK => 36,
        EV_SEGMENT => 48,
        EV_TEXT => 256,
        EV_STCHUNK => 12 + 2 + STCHUNK as i32,
        EV_CONTROLS => 36,
        0x36 => replay::SZ_START as i32,
        0x37 => replay::SZ_PRE as i32,
        0x38 => replay::SZ_POST as i32,
        0x3A => replay::SZ_FSTART as i32,
        0x3C => 8,
        _ => -1,
    }
}

/// The first 9 are version 1's table; version 2 adds the standard Slippi events and the state chunk.
pub const CMDS: [u8; 16] = [0x39, EV_HEAD, EV_FIGHTER, EV_IDENT, EV_STATE, EV_FRAME, EV_MARK, EV_SEGMENT, EV_TEXT, 0x36, 0x37, 0x38, 0x3A, 0x3C, EV_STCHUNK, EV_CONTROLS];

fn put32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}
fn get32(p: &[u8]) -> u32 {
    u32::from_be_bytes([p[0], p[1], p[2], p[3]])
}
fn get64(p: &[u8]) -> u64 {
    ((get32(p) as u64) << 32) | get32(&p[4..]) as u64
}

/// `gr_open`: the container and the 0x35 size table (version 1 lists only its own 9 events).
/// Returns the bytes plus (raw length offset, raw start offset).
pub fn open(write_ver: u32) -> (Vec<u8>, usize, usize) {
    let mut f = vec![b'{', b'U', 3, b'r', b'a', b'w', b'[', b'$', b'U', b'#', b'l'];
    let raw_len_at = f.len();
    f.extend_from_slice(&[0; 4]);
    let raw_start = f.len();
    let n = if write_ver >= 3 {
        16
    } else if write_ver == 2 {
        15
    } else {
        9
    };
    f.push(0x35);
    f.push((1 + 3 * n) as u8);
    for &c in CMDS.iter().take(n) {
        let sz = size_of(c);
        f.push(c);
        f.push((sz >> 8) as u8);
        f.push(sz as u8);
    }
    (f, raw_len_at, raw_start)
}

/// What `gr_head_events` reads from the running game.
#[derive(Clone, Debug)]
pub struct HeadInputs {
    pub write_ver: u32,
    /// `gs_identity`: build, disc, mods, Geno data hashes.
    pub identity: [u64; 4],
    pub geno_version: u32,
    /// `gw_Mods_Describe()`
    pub mods: String,
    /// `gw_FighterControls_MatchCK(port)` and `MatchGet(port)`
    pub controls_ck: [i32; 4],
    pub controls: [u32; 4],
    /// `time(NULL)`
    pub time: u32,
}

/// `gr_head_events` for a whole match (`game_info` = the 0x138-byte StartMeleeData): header, six
/// fighters, identity, state reference, mods text and (version 3) the frozen fighter controls.
pub fn match_head(game_info: &[u8], h: &HeadInputs) -> Vec<u8> {
    let mut buf = Vec::new();
    let base = -123i32;
    let extra_flags = FL_WHOLE | FL_SLIPPI;
    let mut retail = true;
    for s in 0..4 {
        let pl = &game_info[0x60 + 0x24 * s..];
        if pl[1] != 3 && pl[0] > 25 {
            retail = false;
        }
    }
    let put = |buf: &mut Vec<u8>, cmd: u8, ev: &[u8; 300]| {
        buf.push(cmd);
        buf.extend_from_slice(&ev[..size_of(cmd) as usize]);
    };
    let mut ev = [0u8; 300];
    ev[0] = (h.write_ver >> 8) as u8;
    ev[1] = h.write_ver as u8;
    ev[2] = ((if retail { FL_RETAIL } else { 0 }) | if h.write_ver >= 2 { extra_flags } else { 0 }) as u8;
    ev[3] = 6;
    ev[4..8].copy_from_slice(&put32(base as u32));
    ev[8] = 0;
    ev[9] = GR_PAD as u8;
    ev[10] = 0;
    ev[11] = GR_VP as u8;
    ev[12..16].copy_from_slice(&put32(h.time));
    put(&mut buf, EV_HEAD, &ev);
    for s in 0..6usize {
        let mut ev = [0u8; 300];
        let pl = &game_info[0x60 + 0x24 * s..];
        let present = s < 4 && pl[1] != 3;
        ev[0] = s as u8;
        ev[1] = present as u8;
        ev[2] = 0;
        ev[3] = if present { pl[0] } else { 0xFF };
        if !present {
            ev[2] = 0xFF;
        }
        ev[4] = if present { pl[3] } else { 0xFF };
        ev[5] = if present { pl[1] } else { 0xFF };
        if present {
            let nm = format!("ck:{}", pl[0]);
            ev[6..6 + nm.len().min(23)].copy_from_slice(&nm.as_bytes()[..nm.len().min(23)]);
        }
        put(&mut buf, EV_FIGHTER, &ev);
    }
    let mut ev = [0u8; 300];
    for i in 0..4 {
        ev[8 * i..8 * i + 8].copy_from_slice(&h.identity[i].to_be_bytes());
    }
    let stage = ((game_info[0x0E] as u32) << 8) | game_info[0x0F] as u32;
    ev[32..36].copy_from_slice(&put32(stage));
    ev[36..40].copy_from_slice(&put32(h.geno_version));
    put(&mut buf, EV_IDENT, &ev);
    let ev = [0u8; 300];
    put(&mut buf, EV_STATE, &ev);
    let mut ev = [0u8; 300];
    ev[0] = 1;
    let m = h.mods.as_bytes();
    let m = &m[..m.iter().position(|&c| c == 0).unwrap_or(m.len()).min(254)];
    ev[1..1 + m.len()].copy_from_slice(m);
    put(&mut buf, EV_TEXT, &ev);
    if h.write_ver >= 3 {
        let mut ev = [0u8; 300];
        ev[0..4].copy_from_slice(b"GDC1");
        for s in 0..4 {
            ev[4 + 8 * s..8 + 8 * s].copy_from_slice(&put32(h.controls_ck[s] as u32));
            ev[8 + 8 * s..12 + 8 * s].copy_from_slice(&put32(h.controls[s]));
        }
        put(&mut buf, EV_CONTROLS, &ev);
    }
    buf
}

/// `gw_Gdlp_OpenMatch`'s file name choice: `spec` is `MELEE_GDLP_RECORD` ("1" = the recordings
/// folder, a folder, or one .gdlp path); `seq` counts opens in the process, `now` is `time(NULL)`,
/// `rec_dir` the recordings folder.
pub fn match_path(spec: &str, seq: u32, now: i64, rec_dir: &str) -> String {
    let l = spec.len();
    if l > 5 && spec[l - 5..].eq_ignore_ascii_case(".gdlp") && seq == 1 {
        spec.to_string()
    } else if l > 1 && spec != "1" {
        format!("{spec}\\match_{now}_{seq}.gdlp")
    } else {
        format!("{rec_dir}\\match_{now}_{seq}.gdlp")
    }
}

#[derive(Clone, Debug, Default)]
pub struct Fighter {
    pub present: i32,
    pub chr: i32,
    pub costume: i32,
    pub cpu: i32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub tag: i32,
    pub have: u8,
    pub pad: [u8; GR_PAD],
    pub vp: [[i32; 9]; 2],
}

#[derive(Clone, Debug, Default)]
pub struct GrFile {
    pub ver: u32,
    pub flags: u32,
    pub base: u32,
    pub unix_time: u32,
    pub fi: [Fighter; 6],
    pub id: [u64; 4],
    pub stage: i32,
    pub geno_version: u32,
    pub controls_ck: [i32; 4],
    pub controls: [u32; 4],
    pub has_controls: bool,
    pub state: String,
    pub mods: String,
    pub frames: Vec<Frame>,
    pub complete: bool,
    pub marks: Vec<(i32, String)>,
    pub seg_src: String,
    pub seg_from: i32,
    pub seg_to: i32,
    pub emb_size: u32,
    pub emb_got: u32,
    pub has_start: bool,
    pub n_pre: i32,
    pub n_post: i32,
    pub first_frame: i32,
    pub last_frame: i32,
}

fn cstr(p: &[u8]) -> String {
    let e = p.iter().position(|&c| c == 0).unwrap_or(p.len());
    p[..e].iter().map(|&c| c as char).collect()
}

/// `gr_read`. `sink` receives the embedded state chunks (`gr_emb_sink`, used by extract_state).
pub fn read(d: &[u8], mut sink: Option<&mut Vec<u8>>) -> Result<GrFile, String> {
    let n = d.len();
    let mut g = GrFile::default();
    if n < 15 + 2 || &d[..11] != b"{U\x03raw[$U#l" {
        return Err("not a .gdlp (or .slp) file".into());
    }
    let mut pos = 15;
    let mut szt = [[0xFFu8; 2]; 256];
    if d[pos] != 0x35 {
        return Err("no Payloads event".into());
    }
    let tabend = pos + 1 + d[pos + 1] as usize;
    let mut i = 0;
    while (i as i32) < (d[pos + 1] as i32 - 1) / 3 && pos + 2 + i * 3 + 2 < n {
        let c = d[pos + 2 + i * 3] as usize;
        szt[c] = [d[pos + 3 + i * 3], d[pos + 4 + i * 3]];
        i += 1;
    }
    pos = tabend;
    loop {
        if pos >= n {
            break;
        }
        let cmd = d[pos];
        if cmd == b'U' {
            break;
        }
        if szt[cmd as usize] == [0xFF, 0xFF] {
            break;
        }
        let sz = ((szt[cmd as usize][0] as usize) << 8) | szt[cmd as usize][1] as usize;
        if pos + 1 + sz > n {
            break;
        }
        let p = &d[pos + 1..]; // the C reads past sz for a short event too
        match cmd {
            EV_CONTROLS => {
                if sz != 36 || &p[..4] != b"GDC1" {
                    return Err("malformed fighter controls record".into());
                }
                for i in 0..4 {
                    g.controls_ck[i] = get32(&p[4 + i * 8..]) as i32;
                    g.controls[i] = get32(&p[8 + i * 8..]);
                    if g.controls_ck[i] < -1 || g.controls_ck[i] > 0x7f || !replay::controls_word_valid(g.controls[i]) {
                        return Err("invalid fighter controls choice".into());
                    }
                }
                g.has_controls = true;
            }
            EV_HEAD => {
                g.ver = ((p[0] as u32) << 8) | p[1] as u32;
                g.flags = p[2] as u32;
                g.base = get32(&p[4..]);
                g.unix_time = get32(&p[12..]);
            }
            EV_FIGHTER => {
                if p[0] < 6 {
                    let x = &mut g.fi[p[0] as usize];
                    x.present = p[1] as i32;
                    x.chr = ((((p[2] as u16) << 8) | p[3] as u16) as i16) as i32;
                    x.costume = if p[4] == 0xFF { -1 } else { p[4] as i32 };
                    x.cpu = if p[5] == 0xFF { -1 } else { p[5] as i32 };
                    x.name = cstr(&p[6..30]);
                }
            }
            EV_IDENT => {
                for i in 0..4 {
                    g.id[i] = get64(&p[8 * i..]);
                }
                g.stage = get32(&p[32..]) as i32;
                g.geno_version = get32(&p[36..]);
            }
            EV_STATE => g.state = cstr(&p[..40]),
            EV_TEXT => {
                if p[0] == 1 {
                    g.mods = cstr(&p[1..256]);
                }
            }
            EV_MARK => {
                if g.marks.len() < 64 {
                    g.marks.push((get32(p) as i32, cstr(&p[4..36])));
                }
            }
            EV_SEGMENT => {
                g.seg_from = get32(p) as i32;
                g.seg_to = get32(&p[4..]) as i32;
                g.seg_src = cstr(&p[8..48]);
            }
            EV_FRAME => {
                let mut fr = Frame { tag: get32(p) as i32, have: p[4], pad: [0; GR_PAD], vp: [[0; 9]; 2] };
                fr.pad.copy_from_slice(&p[5..5 + GR_PAD]);
                for i in 0..2 {
                    for k in 0..9 {
                        fr.vp[i][k] = get32(&p[5 + GR_PAD + (i * 9 + k) * 4..]) as i32;
                    }
                }
                g.frames.push(fr);
            }
            EV_STCHUNK => {
                g.emb_size = get32(&p[8..]);
                let len = (((p[12] as usize) << 8) | p[13] as usize) as u32;
                if let Some(out) = sink.as_deref_mut() {
                    let idx = get32(p) as usize;
                    if len as usize <= STCHUNK {
                        let off = idx * STCHUNK;
                        if out.len() < off + len as usize {
                            out.resize(off + len as usize, 0);
                        }
                        out[off..off + len as usize].copy_from_slice(&p[14..14 + len as usize]);
                        g.emb_got += len;
                    }
                } else {
                    g.emb_got += len;
                }
            }
            0x36 => g.has_start = true,
            0x37 => g.n_pre += 1,
            0x38 => {
                let fno = get32(p) as i32;
                if g.n_post == 0 || fno < g.first_frame {
                    g.first_frame = fno;
                }
                if g.n_post == 0 || fno > g.last_frame {
                    g.last_frame = fno;
                }
                g.n_post += 1;
            }
            0x39 => g.complete = true,
            _ => {}
        }
        pos += 1 + sz;
        if cmd == 0x39 {
            break;
        }
    }
    if g.ver == 0 {
        return Err("no GD header (an .slp is not a LAB recording)".into());
    }
    if g.ver >= 3 && !g.has_controls {
        return Err("missing fighter controls record".into());
    }
    if g.ver > GR_VER {
        return Err(format!("recorded by a newer game (format {}, this reads {})", g.ver, GR_VER));
    }
    Ok(g)
}

/// What the running build holds for `gr_check_ident`.
#[derive(Clone, Copy, Debug)]
pub struct Identity {
    pub id: [u64; 4],
    pub geno_version: u32,
}

/// `gr_check_ident`: refused, with the reason, unless this build / disc / mods / Geno data is the
/// one the recording was made with.
pub fn check_ident(g: &GrFile, me: &Identity) -> Result<(), String> {
    if g.id[0] != me.id[0] {
        return Err("recorded by another build of the game".into());
    }
    if g.id[1] != me.id[1] {
        return Err("recorded with another disc".into());
    }
    if g.id[2] != me.id[2] {
        return Err(format!("recorded with other mods (it used: {})", if g.mods.is_empty() { "none" } else { &g.mods }));
    }
    if g.id[3] != me.id[3] || g.geno_version != me.geno_version {
        return Err("recorded with other Geno fighter data (a define changed)".into());
    }
    Ok(())
}

/// `gw_Gdlp_CheckFile`: may this whole-match recording play here?
pub fn check_file(d: &[u8], me: &Identity) -> Result<(), String> {
    let g = read(d, None)?;
    if g.flags & FL_WHOLE == 0 || g.flags & FL_SLIPPI == 0 {
        return Err("not a whole-match recording (a LAB clip plays with `lab rec play`)".into());
    }
    check_ident(&g, me)
}

/// `gr_extract_state`: the embedded `.gdst` of a version 2 recording.
pub fn extract_state(d: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let g = read(d, Some(&mut out))?;
    if g.emb_size != 0 && g.emb_got == g.emb_size {
        Ok(out)
    } else if g.emb_size != 0 {
        Err(format!("the embedded state is incomplete ({} of {} bytes)", g.emb_got, g.emb_size))
    } else {
        Err("no embedded state".into())
    }
}

/// `gr_export_ok`
fn export_ok(g: &GrFile) -> Result<(), String> {
    if g.flags & FL_STATE != 0 {
        return Err("this clip starts from a saved state (a mid-match LAB take), not a whole match".into());
    }
    if g.flags & FL_WHOLE == 0 || g.flags & FL_SLIPPI == 0 {
        return Err(if g.ver < 2 {
            "a format-1 recording has no Slippi events (only whole matches recorded by format 2 export)".into()
        } else {
            "not a whole-match recording (no Slippi events)".into()
        });
    }
    if g.flags & FL_RETAIL == 0 {
        let mut s = 0;
        while s < 6 {
            if g.fi[s].present != 0 && g.fi[s].chr > 25 {
                break;
            }
            s += 1;
        }
        let name = if s < 6 { g.fi[s].name.as_str() } else { "?" };
        return Err(format!("a Geno / non-retail fighter is in this match (P{} {}): a .slp can only hold the retail roster", s + 1, name));
    }
    if !g.mods.is_empty() && g.mods != "none" {
        return Err(format!("recorded with mods ({}): not a retail match", g.mods));
    }
    if g.seg_to != 0 {
        return Err("a trimmed excerpt cannot be exported as a match".into());
    }
    if !g.has_start || g.n_post == 0 {
        return Err("the recording has no Game Start / frames".into());
    }
    Ok(())
}

/// `gr_export_slp`: a retail-only whole-match `.gdlp` -> a plain `.slp`; returns the bytes and the
/// number of post-frame events copied.
pub fn export_slp(d: &[u8]) -> Result<(Vec<u8>, i32), String> {
    const KEEP: [u8; 6] = [0x36, 0x37, 0x38, 0x39, 0x3A, 0x3C];
    let g = read(d, None)?;
    export_ok(&g)?;
    let n = d.len();
    let mut szt = [[0xFFu8; 2]; 256];
    let tabend = 16 + d[16] as usize;
    for i in 0..((d[16] as i32 - 1) / 3).max(0) as usize {
        szt[d[17 + i * 3] as usize] = [d[18 + i * 3], d[19 + i * 3]];
    }
    let mut o = vec![b'{', b'U', 3, b'r', b'a', b'w', b'[', b'$', b'U', b'#', b'l'];
    let raw_len_at = o.len();
    o.extend_from_slice(&[0; 4]);
    let raw_start = o.len();
    o.push(0x35);
    o.push((1 + 3 * KEEP.len()) as u8);
    for &k in &KEEP {
        let sz = ((szt[k as usize][0] as u16) << 8) | szt[k as usize][1] as u16;
        o.push(k);
        o.push((sz >> 8) as u8);
        o.push(sz as u8);
    }
    let mut pos = tabend;
    let mut ended = false;
    let mut posts = 0;
    while pos < n && !ended {
        let cmd = d[pos];
        if cmd == b'U' || szt[cmd as usize] == [0xFF, 0xFF] {
            break;
        }
        let sz = ((szt[cmd as usize][0] as usize) << 8) | szt[cmd as usize][1] as usize;
        if pos + 1 + sz > n {
            break;
        }
        if KEEP.contains(&cmd) {
            o.extend_from_slice(&d[pos..pos + 1 + sz]);
            if cmd == 0x38 {
                posts += 1;
            }
            if cmd == 0x39 {
                ended = true;
            }
        }
        pos += 1 + sz;
    }
    if !ended {
        o.extend_from_slice(&[0x39, 7, 0, 0, 0, 0, 0]);
    }
    let end = o.len();
    o.extend_from_slice(b"U\x08metadata{U\x08playedOnSU\x08melee-pc}}");
    let len = (end - raw_start) as u32;
    o[raw_len_at..raw_len_at + 4].copy_from_slice(&put32(len));
    Ok((o, posts))
}

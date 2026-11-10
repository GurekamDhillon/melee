//! Readers for Brawl's sound files, written from the public NW4R format descriptions (docs/rust-port/notes/brawl-audio.md):
//!
//! - RSTM (`.brstm`): a block-interleaved DSP-ADPCM music stream with a loop start.
//! - RSAR (`.brsar`): the sound archive. SYMB holds the names, INFO the sound / file / group tables, and the groups
//!   hold the embedded RWSD files plus their wave data.
//! - RWSD: wave sounds (one note → one wave) with a WAVE table describing each wave's channels.
//! - DSP-ADPCM: 8-byte frames, a predictor/scale byte and 14 signed nibbles, two history samples per channel.
//!
//! Everything here is read from the player's own disc at runtime. Nothing is cached to disk.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

fn be16(d: &[u8], o: usize) -> Option<u16> {
    d.get(o..o + 2).map(|b| u16::from_be_bytes([b[0], b[1]]))
}
fn be32(d: &[u8], o: usize) -> Option<u32> {
    d.get(o..o + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}
fn u8at(d: &[u8], o: usize) -> Option<u8> {
    d.get(o).copied()
}

/// The ADPCM parameters of one channel: 8 coefficient pairs and the starting history.
#[derive(Clone, Copy, Debug, Default)]
pub struct AdpcmParams {
    pub coef: [[i16; 2]; 8],
    pub ps: u8,
    pub yn1: i16,
    pub yn2: i16,
}

impl AdpcmParams {
    fn read(d: &[u8], o: usize) -> Option<AdpcmParams> {
        let mut p = AdpcmParams::default();
        for i in 0..8 {
            p.coef[i][0] = be16(d, o + i * 4)? as i16;
            p.coef[i][1] = be16(d, o + i * 4 + 2)? as i16;
        }
        // 0x20 gain, 0x22 predictor/scale, 0x24 yn1, 0x26 yn2
        p.ps = be16(d, o + 0x22)? as u8;
        p.yn1 = be16(d, o + 0x24)? as i16;
        p.yn2 = be16(d, o + 0x26)? as i16;
        Some(p)
    }
}

/// A running DSP-ADPCM decoder for one channel.
#[derive(Clone, Copy, Debug)]
pub struct Dsp {
    coef: [[i16; 2]; 8],
    h1: i32,
    h2: i32,
}

impl Dsp {
    pub fn new(p: &AdpcmParams) -> Dsp {
        Dsp { coef: p.coef, h1: p.yn1 as i32, h2: p.yn2 as i32 }
    }

    /// Decodes `n` samples from `data` (which starts on a frame boundary) onto `out`. Stops early when the data runs
    /// out. Returns the samples written.
    pub fn decode(&mut self, data: &[u8], n: usize, out: &mut Vec<i16>) -> usize {
        let mut done = 0;
        for frame in data.chunks(8) {
            if done >= n || frame.len() < 2 {
                break;
            }
            let header = frame[0];
            let [c1, c2] = self.coef[(header >> 4) as usize & 7];
            let shift = (header & 0xF) as i32;
            for &b in &frame[1..] {
                for nib in [b >> 4, b & 0xF] {
                    if done >= n {
                        break;
                    }
                    let delta = ((nib as i8) << 4 >> 4) as i32; // sign-extend 4 bits
                    let pred = c1 as i32 * self.h1 + c2 as i32 * self.h2;
                    let v = (((delta << shift) << 11) + pred + 1024) >> 11;
                    let v = v.clamp(-32768, 32767);
                    self.h2 = self.h1;
                    self.h1 = v;
                    out.push(v as i16);
                    done += 1;
                }
            }
        }
        done
    }
}

/// Decoded audio: interleaved (`channels` wide) s16 at `rate` Hz, with an optional loop start (in frames).
#[derive(Clone, Debug, Default)]
pub struct Pcm {
    pub rate: u32,
    pub channels: usize,
    pub samples: Vec<i16>,
    pub loop_start: Option<usize>,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        if self.channels == 0 { 0 } else { self.samples.len() / self.channels }
    }
}

fn interleave(chans: Vec<Vec<i16>>, rate: u32, loop_start: Option<usize>) -> Pcm {
    let n = chans.iter().map(|c| c.len()).min().unwrap_or(0);
    let mut samples = Vec::with_capacity(n * chans.len());
    for i in 0..n {
        for c in &chans {
            samples.push(c[i]);
        }
    }
    Pcm { rate, channels: chans.len(), samples, loop_start: loop_start.filter(|&l| l < n) }
}

/// Decodes a whole `.brstm` (ADPCM, PCM16 or PCM8).
pub fn decode_rstm(d: &[u8]) -> Result<Pcm, String> {
    let bad = || "not a readable RSTM".to_string();
    if d.get(0..4) != Some(b"RSTM") {
        return Err(bad());
    }
    let head = be32(d, 0x10).ok_or_else(bad)? as usize;
    if d.get(head..head + 4) != Some(b"HEAD") {
        return Err(bad());
    }
    let base = head + 8;
    let r_info = base + be32(d, base + 4).ok_or_else(bad)? as usize;
    let r_chan = base + be32(d, base + 0x14).ok_or_else(bad)? as usize;
    let format = u8at(d, r_info).ok_or_else(bad)?;
    let looped = u8at(d, r_info + 1).ok_or_else(bad)? != 0;
    let nch = u8at(d, r_info + 2).ok_or_else(bad)? as usize;
    let rate = be16(d, r_info + 4).ok_or_else(bad)? as u32;
    let loop_start = be32(d, r_info + 8).ok_or_else(bad)? as usize;
    let total = be32(d, r_info + 0xC).ok_or_else(bad)? as usize;
    let data_off = be32(d, r_info + 0x10).ok_or_else(bad)? as usize;
    let nblocks = be32(d, r_info + 0x14).ok_or_else(bad)? as usize;
    let block_size = be32(d, r_info + 0x18).ok_or_else(bad)? as usize;
    let block_samples = be32(d, r_info + 0x1C).ok_or_else(bad)? as usize;
    let last_samples = be32(d, r_info + 0x24).ok_or_else(bad)? as usize;
    let last_padded = be32(d, r_info + 0x28).ok_or_else(bad)? as usize;
    if nch == 0 || nch > 8 || rate == 0 || nblocks == 0 {
        return Err(bad());
    }
    let mut params = Vec::with_capacity(nch);
    if format == 2 {
        if u8at(d, r_chan).ok_or_else(bad)? as usize != nch {
            return Err(bad());
        }
        for c in 0..nch {
            // channel table entry -> channel info -> its ADPCM parameter block
            let ci = base + be32(d, r_chan + 4 + c * 8 + 4).ok_or_else(bad)? as usize;
            let ap = base + be32(d, ci + 4).ok_or_else(bad)? as usize;
            params.push(AdpcmParams::read(d, ap).ok_or_else(bad)?);
        }
    }
    let mut dec: Vec<Dsp> = params.iter().map(Dsp::new).collect();
    let mut chans: Vec<Vec<i16>> = (0..nch).map(|_| Vec::with_capacity(total)).collect();
    let mut at = data_off;
    for b in 0..nblocks {
        let last = b + 1 == nblocks;
        let (stride, want) = if last { (last_padded, last_samples) } else { (block_size, block_samples) };
        for c in 0..nch {
            let s = at + c * stride;
            let blk = d.get(s..(s + stride).min(d.len())).unwrap_or(&[]);
            match format {
                2 => {
                    dec[c].decode(blk, want, &mut chans[c]);
                }
                1 => chans[c].extend(blk.chunks_exact(2).take(want).map(|x| i16::from_be_bytes([x[0], x[1]]))),
                0 => chans[c].extend(blk.iter().take(want).map(|&x| (x as i8 as i16) << 8)),
                _ => return Err("unknown RSTM sample format".into()),
            }
        }
        at += stride * nch;
    }
    for c in chans.iter_mut() {
        c.truncate(total);
    }
    Ok(interleave(chans, rate, looped.then_some(loop_start)))
}

/// One sound in the archive's INFO table.
#[derive(Clone, Debug)]
pub struct SoundEntry {
    pub id: u32,
    pub name: String,
    pub file_id: u32,
    /// 1 sequence, 2 stream, 3 wave sound
    pub kind: u8,
    pub volume: u8,
    /// For a wave sound: the index of the sound inside its RWSD.
    pub sub_index: u32,
}

#[derive(Clone, Debug)]
struct FileEntry {
    external: Option<String>,
    /// (group, item) places the file sits in
    places: Vec<(u32, u32)>,
}

#[derive(Clone, Debug)]
struct GroupItem {
    file_id: u32,
    off: u32,
    len: u32,
    wave_off: u32,
    wave_len: u32,
}

#[derive(Clone, Debug)]
struct Group {
    off: u32,
    wave_off: u32,
    items: Vec<GroupItem>,
}

/// The sound archive's tables (the wave data stays on the disc and is read per sound).
pub struct Rsar {
    path: PathBuf,
    pub sounds: Vec<SoundEntry>,
    files: Vec<FileEntry>,
    groups: Vec<Group>,
}

fn read_range(path: &Path, off: u64, len: usize) -> Result<Vec<u8>, String> {
    let mut f = File::open(path).map_err(|e| format!("archive not readable ({})", e.kind()))?;
    f.seek(SeekFrom::Start(off)).map_err(|e| format!("seek ({})", e.kind()))?;
    let mut v = vec![0; len];
    f.read_exact(&mut v).map_err(|e| format!("short read ({})", e.kind()))?;
    Ok(v)
}

/// An INFO table (`u32 count` + count 8-byte references) at `o`: the referenced offsets, relative to `base`.
fn table(d: &[u8], base: usize, o: usize) -> Option<Vec<usize>> {
    let n = be32(d, o)? as usize;
    if n > 100_000 {
        return None;
    }
    (0..n).map(|i| be32(d, o + 4 + i * 8 + 4).map(|v| base + v as usize)).collect()
}

fn cstr(d: &[u8], o: usize) -> String {
    let s = d.get(o..).unwrap_or(&[]);
    let e = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..e]).into_owned()
}

impl Rsar {
    pub fn open(path: &Path) -> Result<Rsar, String> {
        let hdr = read_range(path, 0, 0x40)?;
        if hdr.get(0..4) != Some(b"RSAR") {
            return Err("not an RSAR".into());
        }
        let sec = |i: usize| -> Result<(u64, usize), String> {
            Ok((be32(&hdr, 0x10 + i * 8).ok_or("header")? as u64, be32(&hdr, 0x14 + i * 8).ok_or("header")? as usize))
        };
        let (symb_off, symb_len) = sec(0)?;
        let (info_off, info_len) = sec(1)?;
        let symb = read_range(path, symb_off, symb_len)?;
        let info = read_range(path, info_off, info_len)?;
        let bad = || "unreadable RSAR tables".to_string();
        if symb.get(0..4) != Some(b"SYMB") || info.get(0..4) != Some(b"INFO") {
            return Err(bad());
        }
        // SYMB: the string table (count + offsets relative to SYMB+8)
        let sb = 8;
        let st = sb + be32(&symb, sb).ok_or_else(bad)? as usize;
        let nstr = be32(&symb, st).ok_or_else(bad)? as usize;
        let name = |id: u32| -> String {
            if (id as usize) < nstr {
                be32(&symb, st + 4 + id as usize * 4).map(|o| cstr(&symb, sb + o as usize)).unwrap_or_default()
            } else {
                String::new()
            }
        };
        let ib = 8;
        let r = |i: usize| -> Option<usize> { be32(&info, ib + i * 8 + 4).map(|v| ib + v as usize) };
        let sounds_t = table(&info, ib, r(0).ok_or_else(bad)?).ok_or_else(bad)?;
        let files_t = table(&info, ib, r(3).ok_or_else(bad)?).ok_or_else(bad)?;
        let groups_t = table(&info, ib, r(4).ok_or_else(bad)?).ok_or_else(bad)?;
        let mut sounds = Vec::with_capacity(sounds_t.len());
        for (i, &o) in sounds_t.iter().enumerate() {
            let sid = be32(&info, o).ok_or_else(bad)?;
            let kind = u8at(&info, o + 0x16).ok_or_else(bad)?;
            let detail = ib + be32(&info, o + 0x1C).ok_or_else(bad)? as usize;
            sounds.push(SoundEntry {
                id: i as u32,
                name: name(sid),
                file_id: be32(&info, o + 4).ok_or_else(bad)?,
                kind,
                volume: u8at(&info, o + 0x14).ok_or_else(bad)?,
                sub_index: if kind == 3 { be32(&info, detail).unwrap_or(0) } else { 0 },
            });
        }
        let mut files = Vec::with_capacity(files_t.len());
        for &o in &files_t {
            let ext_is = u8at(&info, o + 0xC).unwrap_or(0) != 0;
            let ext = be32(&info, o + 0x10).unwrap_or(0);
            let external = (ext_is && ext != 0).then(|| cstr(&info, ib + ext as usize)).filter(|s| !s.is_empty());
            let mut places = vec![];
            if let Some(pt) = be32(&info, o + 0x18).map(|v| ib + v as usize) {
                for p in table(&info, ib, pt).unwrap_or_default() {
                    if let (Some(g), Some(ix)) = (be32(&info, p), be32(&info, p + 4)) {
                        places.push((g, ix));
                    }
                }
            }
            files.push(FileEntry { external, places });
        }
        let mut groups = Vec::with_capacity(groups_t.len());
        for &o in &groups_t {
            let items = table(&info, ib, ib + be32(&info, o + 0x24).ok_or_else(bad)? as usize)
                .ok_or_else(bad)?
                .into_iter()
                .filter_map(|it| {
                    Some(GroupItem {
                        file_id: be32(&info, it)?,
                        off: be32(&info, it + 4)?,
                        len: be32(&info, it + 8)?,
                        wave_off: be32(&info, it + 12)?,
                        wave_len: be32(&info, it + 16)?,
                    })
                })
                .collect();
            groups.push(Group { off: be32(&info, o + 0x10).ok_or_else(bad)?, wave_off: be32(&info, o + 0x18).ok_or_else(bad)?, items });
        }
        Ok(Rsar { path: path.to_path_buf(), sounds, files, groups })
    }

    pub fn find(&self, name: &str) -> Option<&SoundEntry> {
        self.sounds.iter().find(|s| s.name == name).or_else(|| self.sounds.iter().find(|s| s.name.eq_ignore_ascii_case(name)))
    }

    pub fn by_id(&self, id: u32) -> Option<&SoundEntry> {
        self.sounds.get(id as usize)
    }

    /// A stream sound's external file, relative to the archive's folder (e.g. `strm/NAME.brstm`).
    pub fn stream_file(&self, s: &SoundEntry) -> Option<String> {
        if s.kind != 2 {
            return None;
        }
        self.files.get(s.file_id as usize)?.external.clone()
    }

    /// Decodes a wave sound (RWSD inside a group) to PCM.
    pub fn decode_wave_sound(&self, s: &SoundEntry) -> Result<Pcm, String> {
        if s.kind != 3 {
            return Err("not a wave sound".into());
        }
        let f = self.files.get(s.file_id as usize).ok_or("no file")?;
        for &(g, ix) in &f.places {
            let Some(grp) = self.groups.get(g as usize) else { continue };
            let Some(it) = grp.items.get(ix as usize) else { continue };
            if it.file_id != s.file_id || it.len == 0 {
                continue;
            }
            let rwsd = read_range(&self.path, grp.off as u64 + it.off as u64, it.len as usize)?;
            let wave = read_range(&self.path, grp.wave_off as u64 + it.wave_off as u64, it.wave_len as usize)?;
            return decode_rwsd(&rwsd, &wave, s.sub_index);
        }
        Err("wave sound has no group data".into())
    }
}

/// The RWSD's `index`-th wave sound: its first note's wave, decoded, at the note's pitch.
pub fn decode_rwsd(rwsd: &[u8], wave: &[u8], index: u32) -> Result<Pcm, String> {
    let bad = || "unreadable RWSD".to_string();
    if rwsd.get(0..4) != Some(b"RWSD") {
        return Err(bad());
    }
    let data = be32(rwsd, 0x10).ok_or_else(bad)? as usize;
    let wblk = be32(rwsd, 0x18).ok_or_else(bad)? as usize;
    if rwsd.get(data..data + 4) != Some(b"DATA") || rwsd.get(wblk..wblk + 4) != Some(b"WAVE") {
        return Err(bad());
    }
    let db = data + 8;
    let entries = table(rwsd, db, db).ok_or_else(bad)?;
    let e = *entries.get(index as usize).ok_or("sound index past the RWSD")?;
    // entry: info ref, track table ref, note table ref
    let notes = db + be32(rwsd, e + 0x14).ok_or_else(bad)? as usize;
    let note = *table(rwsd, db, notes).ok_or_else(bad)?.first().ok_or("no notes")?;
    let wave_index = be32(rwsd, note).ok_or_else(bad)? as usize;
    let pitch = f32::from_bits(be32(rwsd, note + 0x10).ok_or_else(bad)?);
    let pitch = if pitch.is_finite() && pitch > 0.05 && pitch < 20.0 { pitch } else { 1.0 };
    // WAVE block: count + offsets (relative to the block) to wave infos
    let nw = be32(rwsd, wblk + 8).ok_or_else(bad)? as usize;
    if wave_index >= nw {
        return Err("wave index past the table".into());
    }
    let wi = wblk + be32(rwsd, wblk + 12 + wave_index * 4).ok_or_else(bad)? as usize;
    decode_wave_info(rwsd, wi, wave, pitch)
}

/// A wave info block (shared by RWSD and RBNK): format, loop, channels, rate, loop/end in sample-address units,
/// channel table; `wave` is the file's wave data.
fn decode_wave_info(d: &[u8], wi: usize, wave: &[u8], pitch: f32) -> Result<Pcm, String> {
    let bad = || "unreadable wave info".to_string();
    let format = u8at(d, wi).ok_or_else(bad)?;
    let looped = u8at(d, wi + 1).ok_or_else(bad)? != 0;
    let nch = u8at(d, wi + 2).ok_or_else(bad)? as usize;
    let rate = ((u8at(d, wi + 3).ok_or_else(bad)? as u32) << 16) | be16(d, wi + 4).ok_or_else(bad)? as u32;
    let loop_start = be32(d, wi + 8).ok_or_else(bad)? as usize;
    let end = be32(d, wi + 0xC).ok_or_else(bad)? as usize;
    let ctab = wi + be32(d, wi + 0x10).ok_or_else(bad)? as usize;
    let loc = be32(d, wi + 0x14).ok_or_else(bad)? as usize;
    if nch == 0 || nch > 8 || rate == 0 {
        return Err(bad());
    }
    // ADPCM addresses count nibbles including the frame headers: 16 nibbles hold 14 samples
    let to_samples = |a: usize| if format == 2 { a / 16 * 14 + (a % 16).saturating_sub(2) } else { a };
    let n = to_samples(end + 1);
    let mut chans = vec![];
    for c in 0..nch {
        let ci = wi + be32(d, ctab + c * 4).ok_or_else(bad)? as usize;
        let off = loc + be32(d, ci).ok_or_else(bad)? as usize;
        let src = wave.get(off..).ok_or("wave data out of range")?;
        let mut out = Vec::with_capacity(n);
        match format {
            2 => {
                let ap = wi + be32(d, ci + 4).ok_or_else(bad)? as usize;
                Dsp::new(&AdpcmParams::read(d, ap).ok_or_else(bad)?).decode(src, n, &mut out);
            }
            1 => out.extend(src.chunks_exact(2).take(n).map(|x| i16::from_be_bytes([x[0], x[1]]))),
            0 => out.extend(src.iter().take(n).map(|&x| (x as i8 as i16) << 8)),
            _ => return Err("unknown wave format".into()),
        }
        chans.push(out);
    }
    let rate = (rate as f32 * pitch).round() as u32;
    Ok(interleave(chans, rate.max(1), looped.then(|| to_samples(loop_start))))
}

/// The local Brawl folder: `MELEE_BRAWL_LOCAL`, else the first `_build/brawl` found walking up from the working
/// directory or the exe (the same search as the Petey model).
pub fn local_root() -> Option<PathBuf> {
    if std::env::var_os("MELEE_BRAWL_ABSENT").is_some() {
        return None;
    }
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        return Some(PathBuf::from(p));
    }
    let mut starts = vec![];
    if let Ok(c) = std::env::current_dir() {
        starts.push(c);
    }
    if let Ok(e) = std::env::current_exe() {
        if let Some(d) = e.parent() {
            starts.push(d.to_path_buf());
        }
    }
    for s in starts {
        let mut d: Option<&Path> = Some(&s);
        while let Some(p) = d {
            let c = p.join("_build").join("brawl");
            if c.join("disc").join("files").is_dir() {
                return Some(c);
            }
            d = p.parent();
        }
    }
    None
}

/// `disc/files/sound` under the local Brawl folder, if the archive is there.
pub fn sound_dir() -> Option<PathBuf> {
    let d = local_root()?.join("disc").join("files").join("sound");
    d.join(ARCHIVE).is_file().then_some(d)
}

pub const ARCHIVE: &str = "smashbros_sound.brsar";

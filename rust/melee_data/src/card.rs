//! The GameCube memory card as the port provides it: a folder of `.gci` files per slot (port of
//! Aurora's lib/card CardGciFolder + lib/dolphin/card.cpp, and the policy of pc/platform/shim_card.c).
//!
//! The card lives in `<base>/USA/Card A` and `<base>/USA/Card B` (`GALE` -> region `USA`); each file is
//! `<maker>-<game>-<name>.gci`: a 64 byte big-endian directory entry followed by the file's data
//! (`blocks * 0x2000` bytes). A save written by the C build loads here and the other way round: the
//! on-disk layout, the file naming, the index order (directory enumeration order), the free-block
//! arithmetic and every result code are the C's. Raw card images (`MELEE_CARD_RAW=1`) are not ported.
//!
//! Faithful quirks: `CARDGetStatus` copies only the first two `offsetIcon` entries (a `memcpy` of
//! `size()` elements, not bytes); deleting a file moves it to `_deleted/`; renaming changes the
//! directory entry, not the disk name; a read of a closed file is `NOCARD`.

use std::path::{Path, PathBuf};

pub const FILENAME_MAX: usize = 32;
pub const BLOCK_SIZE: u32 = 0x2000;
pub const MAX_FILES: usize = 127;
pub const FST_BLOCKS: u32 = 5;
pub const MBIT_TO_BLOCKS: u32 = 0x10;
pub const CARD_2043_MB: u32 = 0x80;
const HEADER: usize = 64;

pub mod res {
    pub const READY: i32 = 0;
    pub const BUSY: i32 = -1;
    pub const WRONGDEVICE: i32 = -2;
    pub const NOCARD: i32 = -3;
    pub const NOFILE: i32 = -4;
    pub const IOERROR: i32 = -5;
    pub const BROKEN: i32 = -6;
    pub const EXIST: i32 = -7;
    pub const NOENT: i32 = -8;
    pub const INSSPACE: i32 = -9;
    pub const LIMIT: i32 = -11;
    pub const NAMETOOLONG: i32 = -12;
    pub const ENCODING: i32 = -13;
    pub const FATAL_ERROR: i32 = -128;
}

/// `CARDFileInfo` as the game sees it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub chan: i32,
    pub file_no: i32,
    pub offset: i32,
    pub length: i32,
    pub i_block: u16,
}

/// `CARDStat` as the game sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stat {
    pub file_name: [u8; FILENAME_MAX],
    pub length: u32,
    pub time: u32,
    pub game_name: [u8; 4],
    pub company: [u8; 2],
    pub banner_format: u8,
    pub icon_addr: u32,
    pub icon_format: u16,
    pub icon_speed: u16,
    pub comment_addr: u32,
    pub offset_banner: u32,
    pub offset_banner_tlut: u32,
    pub offset_icon: [u32; 8],
    pub offset_icon_tlut: u32,
    pub offset_data: u32,
}

impl Stat {
    pub fn zeroed() -> Stat {
        Stat::filled(0)
    }
    /// A stat whose every byte is `b` (a C `memset`).
    pub fn filled(b: u8) -> Stat {
        let w = u32::from_ne_bytes([b; 4]);
        let h = u16::from_ne_bytes([b; 2]);
        Stat {
            file_name: [b; FILENAME_MAX],
            length: w,
            time: w,
            game_name: [b; 4],
            company: [b; 2],
            banner_format: b,
            icon_addr: w,
            icon_format: h,
            icon_speed: h,
            comment_addr: w,
            offset_banner: w,
            offset_banner_tlut: w,
            offset_icon: [w; 8],
            offset_icon_tlut: w,
            offset_data: w,
        }
    }
}

/// A directory entry (`aurora::card::File`), native byte order in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    game: [u8; 4],
    maker: [u8; 2],
    reserved: u8,
    banner_flags: u8,
    filename: [u8; 32],
    modified_time: u32,
    icon_address: u32,
    icon_fmt: u16,
    anim_speed: u16,
    permissions: u8,
    copy_counter: i8,
    first_block: u16,
    block_count: u16,
    reserved2: u16,
    comment_addr: u32,
}

impl Entry {
    fn from_disk(b: &[u8; HEADER]) -> Entry {
        let u16be = |o: usize| u16::from_be_bytes([b[o], b[o + 1]]);
        let u32be = |o: usize| u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let mut game = [0; 4];
        game.copy_from_slice(&b[0..4]);
        let mut maker = [0; 2];
        maker.copy_from_slice(&b[4..6]);
        let mut filename = [0; 32];
        filename.copy_from_slice(&b[8..40]);
        Entry {
            game,
            maker,
            reserved: b[6],
            banner_flags: b[7],
            filename,
            modified_time: u32be(0x28),
            icon_address: u32be(0x2C),
            icon_fmt: u16be(0x30),
            anim_speed: u16be(0x32),
            permissions: b[0x34],
            copy_counter: b[0x35] as i8,
            first_block: u16be(0x36),
            block_count: u16be(0x38),
            reserved2: u16be(0x3A),
            comment_addr: u32be(0x3C),
        }
    }

    fn to_disk(&self) -> [u8; HEADER] {
        let mut b = [0u8; HEADER];
        b[0..4].copy_from_slice(&self.game);
        b[4..6].copy_from_slice(&self.maker);
        b[6] = self.reserved;
        b[7] = self.banner_flags;
        b[8..40].copy_from_slice(&self.filename);
        b[0x28..0x2C].copy_from_slice(&self.modified_time.to_be_bytes());
        b[0x2C..0x30].copy_from_slice(&self.icon_address.to_be_bytes());
        b[0x30..0x32].copy_from_slice(&self.icon_fmt.to_be_bytes());
        b[0x32..0x34].copy_from_slice(&self.anim_speed.to_be_bytes());
        b[0x34] = self.permissions;
        b[0x35] = self.copy_counter as u8;
        b[0x36..0x38].copy_from_slice(&self.first_block.to_be_bytes());
        b[0x38..0x3A].copy_from_slice(&self.block_count.to_be_bytes());
        b[0x3A..0x3C].copy_from_slice(&self.reserved2.to_be_bytes());
        b[0x3C..0x40].copy_from_slice(&self.comment_addr.to_be_bytes());
        b
    }
}

fn cstr_len(b: &[u8]) -> usize {
    b.iter().position(|&c| c == 0).unwrap_or(b.len())
}

/// `strncmp(a, b, 32) == 0` for a stored 32 byte name and a C string.
fn name_eq(stored: &[u8; 32], name: &[u8]) -> bool {
    for i in 0..32 {
        let a = stored[i];
        let b = name.get(i).copied().unwrap_or(0);
        if a != b {
            return false;
        }
        if a == 0 {
            return true;
        }
    }
    true
}

#[derive(Clone, Debug)]
struct GciFile {
    entry: Entry,
    file_size: usize,
    /// disk name inside the folder
    filename: String,
    opened: bool,
    deleted: bool,
    dirty: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Handle {
    pub idx: u32,
    pub offset: i32,
}

/// One slot's GCI folder (`aurora::card::CardGciFolder`).
pub struct GciFolder {
    files: Vec<GciFile>,
    folder: PathBuf,
    error: i32,
    encoding: u16,
    game: [u8; 5],
    maker: [u8; 3],
    clock: std::sync::Arc<dyn Fn() -> u32 + Send + Sync>,
}

fn image_sizes(fmt: u8) -> (u32, u32, u32) {
    // (banner size, icon size, tlut size) of an EImageFormat (0 none, 1 C8, 2 RGB5A3)
    match fmt {
        1 => (3584, 1024, 512),
        2 => (6144, 2048, 0),
        _ => (0, 0, 0),
    }
}

impl GciFolder {
    pub fn new(clock: std::sync::Arc<dyn Fn() -> u32 + Send + Sync>) -> GciFolder {
        GciFolder { files: Vec::new(), folder: PathBuf::new(), error: res::NOCARD, encoding: 0, game: [0; 5], maker: [0; 3], clock }
    }

    pub fn init_card(&mut self, game: &[u8], maker: &[u8]) {
        self.set_current_game(game);
        self.set_current_maker(maker);
    }

    pub fn set_current_game(&mut self, game: &[u8]) {
        if cstr_len(game) == 4 {
            self.game[..4].copy_from_slice(&game[..4]);
        }
    }

    pub fn set_current_maker(&mut self, maker: &[u8]) {
        if cstr_len(maker) == 2 {
            self.maker[..2].copy_from_slice(&maker[..2]);
        }
    }

    pub fn error(&self) -> i32 {
        self.error
    }

    fn get_file(&self, idx: u32) -> Option<&GciFile> {
        if self.error == res::READY && (idx as usize) < self.files.len() && !self.files[idx as usize].deleted {
            Some(&self.files[idx as usize])
        } else {
            None
        }
    }

    fn get_file_mut(&mut self, idx: u32) -> Option<&mut GciFile> {
        if self.error == res::READY && (idx as usize) < self.files.len() && !self.files[idx as usize].deleted {
            Some(&mut self.files[idx as usize])
        } else {
            None
        }
    }

    fn get_open_file(&self, h: &Handle) -> Option<&GciFile> {
        self.get_file(h.idx).filter(|f| f.opened)
    }

    fn find_file(&self, name: &[u8]) -> i32 {
        let name = &name[..cstr_len(name)];
        if name.len() > FILENAME_MAX {
            return -1;
        }
        for (idx, e) in self.files.iter().enumerate() {
            if !e.deleted && e.entry.game == self.game[..4] && e.entry.maker == self.maker[..2] && name_eq(&e.entry.filename, name) {
                return idx as i32;
            }
        }
        -1
    }

    fn path_of(&self, f: &GciFile) -> PathBuf {
        self.folder.join(&f.filename)
    }

    pub fn open_file_by_name(&mut self, name: &[u8]) -> (i32, FileInfoHandle) {
        if self.error != res::READY {
            return (self.error, FileInfoHandle::default());
        }
        let idx = self.find_file(name);
        if idx < 0 {
            (res::NOFILE, FileInfoHandle::default())
        } else {
            self.open_file_by_no(idx as u32)
        }
    }

    pub fn open_file_by_no(&mut self, no: u32) -> (i32, FileInfoHandle) {
        if self.error != res::READY {
            return (self.error, FileInfoHandle::default());
        }
        if let Some(f) = self.get_file_mut(no) {
            f.opened = true;
            return (res::READY, FileInfoHandle { file_no: no, offset: 0 });
        }
        (res::NOFILE, FileInfoHandle::default())
    }

    pub fn create_file(&mut self, name: &[u8], size: usize) -> (i32, FileInfoHandle) {
        let none = FileInfoHandle::default();
        if self.error != res::READY {
            return (self.error, none);
        }
        let name = &name[..cstr_len(name)];
        if name.len() > FILENAME_MAX {
            return (res::NAMETOOLONG, none);
        }
        if size == 0 || size % BLOCK_SIZE as usize != 0 {
            return (res::FATAL_ERROR, none);
        }
        if self.find_file(name) >= 0 {
            return (res::EXIST, none);
        }
        let (free_bytes, free_files) = self.free_blocks();
        if free_files <= 0 {
            return (res::NOENT, none);
        }
        if size > free_bytes as usize {
            return (res::INSSPACE, none);
        }
        let mut base: Vec<u8> = Vec::new();
        base.extend_from_slice(&self.maker[..2]);
        base.push(b'-');
        base.extend_from_slice(&self.game[..4]);
        base.push(b'-');
        base.extend_from_slice(name);
        for c in base.iter_mut() {
            if *c < 32 || b"<>:\"/\\|?*".contains(c) {
                *c = b'_';
            }
        }
        let base = String::from_utf8_lossy(&base).into_owned();
        let mut disk = format!("{base}.gci");
        let mut suffix = 1;
        while self.folder.join(&disk).exists() {
            disk = format!("{base}-{suffix}.gci");
            suffix += 1;
        }
        let mut e = Entry {
            game: [0; 4],
            maker: [0; 2],
            reserved: 0,
            banner_flags: 0,
            filename: [0; 32],
            modified_time: 0,
            icon_address: 0,
            icon_fmt: 0,
            anim_speed: 0,
            permissions: 0,
            copy_counter: 0,
            first_block: 0,
            block_count: 0,
            reserved2: 0,
            comment_addr: 0,
        };
        e.filename[..name.len()].copy_from_slice(name);
        e.game.copy_from_slice(&self.game[..4]);
        e.maker.copy_from_slice(&self.maker[..2]);
        e.modified_time = (self.clock)();
        e.block_count = (size / BLOCK_SIZE as usize) as u16;
        e.first_block = FST_BLOCKS as u16;
        e.permissions = 4; // Public
        e.icon_address = u32::MAX;
        e.comment_addr = u32::MAX;
        e.anim_speed = 1;
        let mut buf = vec![0u8; HEADER + size];
        buf[..HEADER].copy_from_slice(&e.to_disk());
        if write_atomic(&self.folder.join(&disk), &buf).is_err() {
            return (res::IOERROR, none);
        }
        let entry = GciFile { entry: e, file_size: buf.len(), filename: disk, opened: true, deleted: false, dirty: false };
        let idx = match self.files.iter().position(|f| f.deleted) {
            Some(i) => {
                self.files[i] = entry;
                i
            }
            None => {
                self.files.push(entry);
                self.files.len() - 1
            }
        };
        (res::READY, FileInfoHandle { file_no: idx as u32, offset: 0 })
    }

    pub fn close_file(&mut self, h: FileInfoHandle) -> i32 {
        let hd = Handle { idx: h.file_no, offset: h.offset };
        let ok = self.get_open_file(&hd).is_some();
        if ok {
            self.files[h.file_no as usize].opened = false;
            res::READY
        } else {
            res::NOFILE
        }
    }

    pub fn delete_file_by_name(&mut self, name: &[u8]) -> i32 {
        if self.error != res::READY {
            return self.error;
        }
        let idx = self.find_file(name);
        if idx < 0 {
            res::NOFILE
        } else {
            self.delete_file_by_no(idx as u32)
        }
    }

    pub fn delete_file_by_no(&mut self, no: u32) -> i32 {
        if self.error != res::READY {
            return self.error;
        }
        let Some(f) = self.get_file(no) else { return res::NOFILE };
        let deleted = self.folder.join("_deleted");
        let source = self.folder.join(&f.filename);
        let dest = deleted.join(&f.filename);
        if std::fs::create_dir_all(&deleted).is_err() || std::fs::rename(&source, &dest).is_err() {
            return res::IOERROR;
        }
        let f = &mut self.files[no as usize];
        f.opened = false;
        f.deleted = true;
        f.dirty = false;
        res::READY
    }

    pub fn rename_file(&mut self, old: &[u8], new: &[u8]) -> i32 {
        if self.error != res::READY {
            return self.error;
        }
        let new = &new[..cstr_len(new)];
        if new.len() > FILENAME_MAX {
            return res::NAMETOOLONG;
        }
        let idx = self.find_file(old);
        if idx < 0 {
            return res::NOFILE;
        }
        if self.find_file(new) >= 0 {
            return res::EXIST;
        }
        let f = &mut self.files[idx as usize];
        f.entry.filename = [0; 32];
        f.entry.filename[..new.len()].copy_from_slice(new);
        f.dirty = true;
        res::READY
    }

    pub fn file_write(&mut self, h: &Handle, buf: &[u8], size: usize) -> i32 {
        let Some(f) = self.get_open_file(h) else { return res::NOCARD };
        let data_size = f.file_size.wrapping_sub(HEADER);
        if h.offset < 0 || h.offset as usize > data_size || size > data_size - h.offset as usize {
            return res::LIMIT;
        }
        let path = self.path_of(f);
        if std::fs::OpenOptions::new().read(true).write(true).open(&path).is_err() {
            return res::NOFILE;
        }
        let Ok(mut all) = std::fs::read(&path) else { return res::IOERROR };
        let off = HEADER + h.offset as usize;
        if all.len() < off + size {
            // the C's seek-and-write would extend the file
            all.resize(off + size, 0);
        }
        all[off..off + size].copy_from_slice(&buf[..size]);
        if write_atomic(&path, &all).is_err() {
            return res::IOERROR;
        }
        res::READY
    }

    pub fn file_read(&self, h: &Handle, dst: &mut [u8], size: usize) -> i32 {
        let Some(f) = self.get_open_file(h) else { return res::NOCARD };
        let data_size = f.file_size.wrapping_sub(HEADER);
        if h.offset < 0 || h.offset as usize > data_size || size > data_size - h.offset as usize {
            return res::LIMIT;
        }
        let path = self.path_of(f);
        if std::fs::OpenOptions::new().read(true).write(true).open(&path).is_err() {
            return res::NOFILE;
        }
        let Ok(all) = std::fs::read(&path) else { return res::IOERROR };
        let off = HEADER + h.offset as usize;
        if off + size > all.len() {
            return res::IOERROR;
        }
        dst[..size].copy_from_slice(&all[off..off + size]);
        res::READY
    }

    /// `getStatus(fileNo)`: the full 8 entry offset table, as `aurora::card::CardStat` holds it.
    pub fn get_status(&self, no: u32) -> Result<Stat, i32> {
        let Some(gf) = self.get_file(no) else { return Err(res::NOFILE) };
        let f = &gf.entry;
        let mut s = Stat::zeroed();
        let n = cstr_len(&f.filename).min(32);
        s.file_name = [0; 32];
        s.file_name[..n].copy_from_slice(&f.filename[..n]);
        s.length = f.block_count as u32 * BLOCK_SIZE;
        s.time = f.modified_time;
        s.game_name = f.game;
        s.company = f.maker;
        s.banner_format = f.banner_flags;
        s.icon_addr = f.icon_address;
        s.icon_format = f.icon_fmt;
        s.icon_speed = f.anim_speed;
        s.comment_addr = f.comment_addr;
        if f.icon_address == u32::MAX {
            s.offset_banner = u32::MAX;
            s.offset_banner_tlut = u32::MAX;
            s.offset_icon = [u32::MAX; 8];
            s.offset_icon_tlut = u32::MAX;
            s.offset_data = f.comment_addr.wrapping_add(64);
        } else {
            let mut cur = f.icon_address;
            s.offset_banner = cur;
            let (bs, _, bt) = image_sizes(s.banner_format & 3);
            cur = cur.wrapping_add(bs);
            s.offset_banner_tlut = cur;
            cur = cur.wrapping_add(bt);
            let mut palette = false;
            for i in 0..8 {
                s.offset_icon[i] = cur;
                let fmt = ((s.icon_format >> (i * 2)) & 3) as u8;
                if fmt == 1 {
                    palette = true;
                }
                cur = cur.wrapping_add(image_sizes(fmt).1);
            }
            if palette {
                s.offset_icon_tlut = cur;
                cur = cur.wrapping_add(512);
            } else {
                s.offset_icon_tlut = u32::MAX;
            }
            s.offset_data = cur;
        }
        Ok(s)
    }

    pub fn set_status(&mut self, no: u32, banner: u8, icon_addr: u32, icon_fmt: u16, icon_speed: u16, comment: u32) -> i32 {
        let Some(gf) = self.get_file_mut(no) else { return res::NOFILE };
        gf.entry.banner_flags = banner;
        gf.entry.icon_address = icon_addr;
        gf.entry.icon_fmt = icon_fmt;
        gf.entry.anim_speed = icon_speed;
        gf.entry.comment_addr = comment;
        gf.dirty = true;
        res::READY
    }

    /// (bytes not used, files not used)
    pub fn free_blocks(&self) -> (i32, i32) {
        let mut bytes = ((CARD_2043_MB * MBIT_TO_BLOCKS - FST_BLOCKS) * BLOCK_SIZE) as i32;
        let mut files = MAX_FILES as i32;
        for f in &self.files {
            if !f.deleted {
                bytes = bytes.wrapping_sub((f.entry.block_count as u32 * BLOCK_SIZE) as i32);
                files -= 1;
            }
        }
        (bytes.max(0), files)
    }

    pub fn encoding(&self) -> u16 {
        self.encoding
    }

    pub fn format(&mut self) -> i32 {
        self.encoding = 0;
        if self.folder.as_os_str().is_empty() || std::fs::create_dir_all(&self.folder).is_err() {
            return res::IOERROR;
        }
        self.error = res::READY;
        res::READY
    }

    pub fn commit(&mut self) -> i32 {
        if self.error != res::READY {
            return self.error;
        }
        for i in 0..self.files.len() {
            if self.files[i].deleted || !self.files[i].dirty {
                continue;
            }
            let path = self.folder.join(&self.files[i].filename);
            let hdr = self.files[i].entry.to_disk();
            let ok = std::fs::OpenOptions::new().read(true).write(true).open(&path).is_ok()
                && std::fs::read(&path).ok().map_or(false, |mut all| {
                    if all.len() < HEADER {
                        return false;
                    }
                    all[..HEADER].copy_from_slice(&hdr);
                    write_atomic(&path, &all).is_ok()
                });
            if !ok {
                let mut b = [0u8; HEADER];
                let readable = std::fs::OpenOptions::new().read(true).write(true).open(&path).is_ok()
                    && std::fs::read(&path).ok().map_or(false, |all| {
                        if all.len() < HEADER {
                            return false;
                        }
                        b.copy_from_slice(&all[..HEADER]);
                        true
                    });
                if readable {
                    self.files[i].entry = Entry::from_disk(&b);
                    self.files[i].dirty = false;
                } else {
                    self.error = res::IOERROR;
                }
                return res::IOERROR;
            }
            self.files[i].dirty = false;
        }
        res::READY
    }

    /// `open(folder)`: read every `.gci` header. false leaves the error code the C would hold.
    pub fn open(&mut self, folder: &Path) -> bool {
        self.files.clear();
        self.error = res::NOCARD;
        self.folder = folder.to_path_buf();
        if !folder.is_dir() {
            return false;
        }
        self.error = res::IOERROR;
        let Ok(rd) = std::fs::read_dir(folder) else { return false };
        let mut files: Vec<GciFile> = Vec::new();
        for e in rd {
            let Ok(e) = e else { return false };
            let path = e.path();
            let Ok(md) = std::fs::metadata(&path) else { return false };
            if !md.is_file() {
                continue;
            }
            if path.extension().map_or(true, |x| x != "gci") {
                continue;
            }
            let Ok(data) = std::fs::read(&path) else { return false };
            if data.len() < HEADER {
                return false;
            }
            let mut hb = [0u8; HEADER];
            hb.copy_from_slice(&data[..HEADER]);
            let entry = Entry::from_disk(&hb);
            for ex in &files {
                if ex.entry.game == entry.game && ex.entry.maker == entry.maker && {
                    let a = &ex.entry.filename;
                    let b = &entry.filename;
                    let la = cstr_len(a);
                    let lb = cstr_len(b);
                    a[..la] == b[..lb]
                } {
                    self.error = res::BROKEN;
                    return false;
                }
            }
            if files.len() == MAX_FILES {
                self.error = res::BROKEN;
                return false;
            }
            files.push(GciFile {
                entry,
                file_size: data.len(),
                filename: path.file_name().unwrap().to_string_lossy().into_owned(),
                opened: false,
                deleted: false,
                dirty: false,
            });
        }
        self.files = files;
        self.error = res::READY;
        true
    }

    pub fn close(&mut self) {
        self.files.clear();
        self.folder = PathBuf::new();
        self.error = res::NOCARD;
    }

    /// `probeCardFile`: (result, size in megabits, sector size)
    pub fn probe_card_file(path: &Path) -> (i32, u32, u32) {
        if path.is_dir() {
            (res::READY, CARD_2043_MB, BLOCK_SIZE)
        } else {
            (res::NOCARD, 0, 0)
        }
    }
}

/// `FileHandle` as returned to the API layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct FileInfoHandle {
    pub file_no: u32,
    pub offset: i32,
}

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp-card");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, data)?;
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// The local wall-clock time as seconds since 2000-01-01 (`aurora::card::getGCTime`).
pub fn gc_time_now() -> u32 {
    let unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let local = unix + local_offset_seconds();
    (local - 946_684_800) as u32
}

#[cfg(windows)]
fn local_offset_seconds() -> i64 {
    use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
    let mut tz: TIME_ZONE_INFORMATION = unsafe { std::mem::zeroed() };
    let id = unsafe { GetTimeZoneInformation(&mut tz) };
    let std_off = -((tz.Bias + tz.StandardBias) as i64) * 60;
    std_off + if id == 2 { 3600 } else { 0 }
}

#[cfg(not(windows))]
fn local_offset_seconds() -> i64 {
    0
}

// ---- the CARD API layer (lib/dolphin/card.cpp + shim_card.c) ---------------------------------------

/// `CARDGetStatus`'s copy into the game's struct: `offsetIcon` receives only its first two entries.
fn kabu_stat_to_dolphin(k: &Stat, out: &mut Stat) {
    out.file_name = k.file_name;
    out.length = k.length;
    out.time = k.time;
    out.game_name = k.game_name;
    out.company = k.company;
    out.banner_format = k.banner_format;
    out.icon_addr = k.icon_addr;
    out.icon_format = k.icon_format;
    out.icon_speed = k.icon_speed;
    out.comment_addr = k.comment_addr;
    out.offset_banner = k.offset_banner;
    out.offset_banner_tlut = k.offset_banner_tlut;
    out.offset_icon[0] = k.offset_icon[0];
    out.offset_icon[1] = k.offset_icon[1];
    out.offset_icon_tlut = k.offset_icon_tlut;
    out.offset_data = k.offset_data;
}

/// The two card slots and the CARD entry points the game reaches through shim_card.c.
pub struct CardSystem {
    chans: [Option<GciFolder>; 2],
    paths: [PathBuf; 2],
    base_paths: [PathBuf; 2],
    initialized: bool,
    clock: std::sync::Arc<dyn Fn() -> u32 + Send + Sync>,
    /// `gw_card_ready`: the shim reports NOCARD everywhere until `init` succeeds
    ready: bool,
    pub log: Vec<String>,
}

impl CardSystem {
    pub fn new(clock: std::sync::Arc<dyn Fn() -> u32 + Send + Sync>) -> CardSystem {
        CardSystem { chans: [None, None], paths: [PathBuf::new(), PathBuf::new()], base_paths: [PathBuf::new(), PathBuf::new()], initialized: false, clock, ready: false, log: Vec::new() }
    }

    /// With the real clock.
    pub fn with_system_clock() -> CardSystem {
        Self::new(std::sync::Arc::new(gc_time_now))
    }

    /// `CARDSetBasePath(base, -1)` + `CARDInit("GALE", "01")` (what `gw_CARDInit` and Aurora do).
    pub fn init(&mut self, base: &Path) {
        self.set_base_path(base);
        self.card_init(b"GALE", b"01");
        self.ready = true;
        self.log.push(format!("gw: card: initialised (GCI folder) at {}", base.display()));
    }

    fn set_base_path(&mut self, path: &Path) {
        let mut p = path.to_path_buf();
        if p.file_name().is_some() && !p.is_dir() {
            p.pop();
        }
        self.base_paths = [p.clone(), p];
    }

    fn full_path(base: &Path, slot: usize) -> PathBuf {
        base.join("USA").join(if slot == 0 { "Card A" } else { "Card B" })
    }

    fn card_init(&mut self, game: &[u8], maker: &[u8]) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        for i in 0..2 {
            let mut g = GciFolder::new(self.clock.clone());
            g.init_card(game, maker);
            self.chans[i] = Some(g);
        }
        let mut loaded = false;
        for i in 0..2 {
            if self.paths[i].as_os_str().is_empty() {
                let base = if self.base_paths[i].as_os_str().is_empty() { std::env::current_dir().unwrap_or_default() } else { self.base_paths[i].clone() };
                self.paths[i] = Self::full_path(&base, i);
            }
            let cur = self.paths[i].clone();
            if cur.exists() && self.chans[i].as_mut().unwrap().open(&cur) {
                loaded = true;
            }
        }
        if !loaded && !self.paths[0].exists() {
            let p0 = self.paths[0].clone();
            let c = self.chans[0].as_mut().unwrap();
            c.open(&p0);
            c.format();
            c.close();
            c.open(&p0);
        }
    }

    fn chan_ok(chan: i32) -> bool {
        (0..2).contains(&chan)
    }

    fn card(&mut self, chan: i32) -> &mut GciFolder {
        self.chans[chan as usize].as_mut().expect("CARDInit not called")
    }

    // ---- shim_card.c policy ----

    /// `gw_CARDProbe`
    pub fn probe(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return 0;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let (r, _, _) = GciFolder::probe_card_file(&self.paths[chan as usize]);
        (r != res::NOCARD) as i32
    }

    /// `gw_CARDProbeEx`: (result, memory size in megabits, sector size). Never `BUSY`.
    pub fn probe_ex(&mut self, chan: i32) -> (i32, i32, i32) {
        if !self.ready {
            return (res::NOCARD, 0, 0);
        }
        if !Self::chan_ok(chan) {
            return (res::FATAL_ERROR, 0, 0);
        }
        let (r, m, s) = GciFolder::probe_card_file(&self.paths[chan as usize]);
        let r = if r == res::BUSY { res::NOCARD } else { r };
        (r, m as i32, s as i32)
    }

    /// `gw_CARDMountAsync` (synchronous part): Aurora's `CARDMount` is `READY` for a valid channel.
    pub fn mount(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        res::READY
    }

    /// `gw_CARDUnmount`
    pub fn unmount(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        res::NOCARD
    }

    /// `gw_CARDCheckAsync` (synchronous part)
    pub fn check(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        self.card(chan).error()
    }

    /// `gw_CARDFormatAsync` (synchronous part)
    pub fn format(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        self.card(chan).format()
    }

    /// `CARDGetResultCode`
    pub fn result_code(&mut self, chan: i32) -> i32 {
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        self.card(chan).error()
    }

    /// `CARDSetGameAndMaker`
    pub fn set_game_and_maker(&mut self, chan: i32, game: &[u8], maker: &[u8]) {
        if !Self::chan_ok(chan) {
            return;
        }
        let c = self.card(chan);
        c.set_current_game(game);
        c.set_current_maker(maker);
    }

    /// `gw_CARDOpen`: the info is always written (zeroed when the open fails).
    pub fn open(&mut self, chan: i32, name: &[u8], info: &mut FileInfo) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        *info = FileInfo::default();
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let (r, h) = self.card(chan).open_file_by_name(name);
        if r == res::READY {
            *info = FileInfo { chan, file_no: h.file_no as i32, offset: h.offset, ..Default::default() };
        }
        r
    }

    /// `gw_CARDFastOpen`
    pub fn fast_open(&mut self, chan: i32, file_no: i32, info: &mut FileInfo) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        *info = FileInfo::default();
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let (r, h) = self.card(chan).open_file_by_no(file_no as u32);
        if r == res::READY {
            *info = FileInfo { chan, file_no: h.file_no as i32, offset: h.offset, ..Default::default() };
        }
        r
    }

    /// `gw_CARDClose`
    pub fn close(&mut self, info: &mut FileInfo) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(info.chan) {
            return res::FATAL_ERROR;
        }
        let h = FileInfoHandle { file_no: info.file_no as u32, offset: info.offset };
        self.card(info.chan).close_file(h)
    }

    /// `gw_CARDRead`
    pub fn read(&mut self, info: &FileInfo, buf: &mut [u8], length: i32, offset: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(info.chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(info.chan);
        let mut h = Handle { idx: info.file_no as u32, offset: info.offset };
        if card.get_open_file(&h).is_some() {
            h.offset = offset; // seek(Begin)
        }
        card.file_read(&h, buf, length as usize)
    }

    /// `gw_CARDWrite`: a successful write is committed.
    pub fn write(&mut self, info: &FileInfo, buf: &[u8], length: i32, offset: i32) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(info.chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(info.chan);
        let mut h = Handle { idx: info.file_no as u32, offset: info.offset };
        if card.get_open_file(&h).is_some() {
            h.offset = offset;
        }
        let r = card.file_write(&h, buf, length as usize);
        if r == res::READY {
            card.commit()
        } else {
            r
        }
    }

    /// `gw_CARDCreateAsync` (synchronous part)
    pub fn create(&mut self, chan: i32, name: &[u8], size: u32, info: &mut FileInfo) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        *info = FileInfo::default();
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(chan);
        let (r, h) = card.create_file(name, size as usize);
        if r == res::READY {
            *info = FileInfo { chan, file_no: h.file_no as i32, offset: h.offset, ..Default::default() };
            card.commit()
        } else {
            r
        }
    }

    /// `gw_CARDDeleteAsync`
    pub fn delete(&mut self, chan: i32, name: &[u8]) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(chan);
        let r = card.delete_file_by_name(name);
        if r == res::READY {
            card.commit()
        } else {
            r
        }
    }

    /// `CARDFastDelete`
    pub fn fast_delete(&mut self, chan: i32, file_no: i32) -> i32 {
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(chan);
        let r = card.delete_file_by_no(file_no as u32);
        if r == res::READY {
            card.commit()
        } else {
            r
        }
    }

    /// `gw_CARDRenameAsync`
    pub fn rename(&mut self, chan: i32, old: &[u8], new: &[u8]) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(chan);
        let r = card.rename_file(old, new);
        if r == res::READY {
            card.commit()
        } else {
            r
        }
    }

    /// `gw_CARDGetStatus`: `stat` is only written on success (the shim passes a zeroed struct).
    pub fn get_status(&mut self, chan: i32, file_no: i32, stat: &mut Stat) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        match self.card(chan).get_status(file_no as u32) {
            Ok(k) => {
                kabu_stat_to_dolphin(&k, stat);
                res::READY
            }
            Err(e) => e,
        }
    }

    /// `gw_CARDSetStatusAsync` (synchronous part)
    pub fn set_status(&mut self, chan: i32, file_no: i32, stat: &Stat) -> i32 {
        if !self.ready {
            return res::NOCARD;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        let card = self.card(chan);
        let r = card.set_status(file_no as u32, stat.banner_format, stat.icon_addr, stat.icon_format, stat.icon_speed, stat.comment_addr);
        if r == res::READY {
            card.commit()
        } else {
            r
        }
    }

    /// `gw_CARDFreeBlocks`: (result, bytes not used, files not used)
    pub fn free_blocks(&mut self, chan: i32) -> (i32, i32, i32) {
        if !self.ready {
            return (res::NOCARD, 0, 0);
        }
        if !Self::chan_ok(chan) {
            return (res::FATAL_ERROR, 0, 0);
        }
        let (b, f) = self.card(chan).free_blocks();
        (res::READY, b, f)
    }

    /// `CARDGetEncoding`
    pub fn encoding(&mut self, chan: i32) -> (i32, u16) {
        if !Self::chan_ok(chan) {
            return (res::FATAL_ERROR, 0);
        }
        (res::READY, self.card(chan).encoding())
    }

    /// `CARDGetSectorSize`
    pub fn sector_size(&mut self, chan: i32) -> (i32, u32) {
        if !Self::chan_ok(chan) {
            return (res::FATAL_ERROR, 0);
        }
        (res::READY, 8192)
    }

    /// `CARDGetSerialNo`
    pub fn serial_no(&mut self, chan: i32) -> (i32, u64) {
        if !Self::chan_ok(chan) {
            return (res::FATAL_ERROR, 0);
        }
        (res::READY, 0)
    }

    /// `gw_CARDGetXferredBytes`: every operation completes inline, so nothing is in flight.
    pub fn xferred_bytes(&mut self, chan: i32) -> i32 {
        if !self.ready {
            return 0;
        }
        if !Self::chan_ok(chan) {
            return res::FATAL_ERROR;
        }
        res::READY
    }

    /// `gw___CARDSync`
    pub fn card_sync(&self) -> i32 {
        if self.ready {
            res::READY
        } else {
            res::NOCARD
        }
    }

    /// `aurora_card_remount`: close and reopen a channel so changes on disk are discovered.
    pub fn remount(&mut self, chan: i32) -> bool {
        if !self.initialized || !Self::chan_ok(chan) || self.paths[chan as usize].as_os_str().is_empty() {
            return false;
        }
        let p = self.paths[chan as usize].clone();
        let c = self.card(chan);
        c.close();
        c.open(&p)
    }
}

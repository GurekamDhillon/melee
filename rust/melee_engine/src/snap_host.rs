//! gw_snap.c's snapshot machinery (`sn`) for the engine: the one [`Base`] (snapshot.rs; its copy-on-write and page
//! watch are process-wide, so there is exactly one) and what the C keeps beside it:
//! - frame-keyed sessions (gw_snap_open / gw_snap_save / gw_snap_load: the game half's Geno snapshot tests);
//! - slots by index (gw_snap_reserve / gw_snap_save_index / gw_snap_load_index: the Lua savestates, the Lab history);
//! - the long rewind's keyframe store (gw_rw_*) and its exactness self-test (gw_rw_capture / gw_rw_compare);
//! - the savestate files (gw_snap_file_*);
//! - the rollback visualiser's ring (gw_RbViz_*) and the resimulation flag (gw_Snap_Resimulating / gw_Snap_LabResim).
//!
//! The C's MEM1 + game globals are the base's ranges here (the arena, the OS heap, the statics); a keyframe is a
//! snapshot that shares its unchanged pages with the previous one (`Base::save_after`) where the C stores page deltas.
//! Everything lives on the system heap ([`host_heap`]), never in the arena a load rewinds.
//!
//! Consumers with a base of their own (play_net's ModesGame, netmatch, retail_flow's seek, the Lab's LabRun) move onto
//! [`with_base`] / [`release`] later: the same calls their `with_base` closures make.
use crate::snapshot::{Base, RangeImage, SnapRef};

/// GW_SNAP_MAX_SLOTS
pub const MAX_SLOTS: usize = 48;
/// RW_MAX_KEYS
pub const RW_MAX_KEYS: usize = 512;
/// RW_USER: the script bookkeeping kept beside a keyframe
pub const RW_USER: usize = 128;
/// RBV_N: the rollback visualiser's ring
pub const RBV_N: usize = 600;

/// Runs `f` with the arena off, so what it allocates is on the system heap (a load does not rewind it).
pub fn host_heap<R>(f: impl FnOnce() -> R) -> R {
    let was = crate::arena::active();
    crate::arena::set_active(false);
    let r = f();
    crate::arena::set_active(was);
    r
}

struct Key {
    tag: i32,
    snap: SnapRef,
    user: [u8; RW_USER],
    cursor: Vec<u8>,
    bytes: usize,
}

#[derive(Default)]
struct Rw {
    on: bool,
    base_tag: i32,
    base: Option<SnapRef>,
    base_user: Vec<u8>,
    base_cursor: Vec<u8>,
    keys: Vec<Key>,
    delta_bytes: usize,
    ms_save: f64,
    n_save: u32,
    ms_last_load: f64,
    last_load_pages: u32,
    cap: Option<SnapRef>,
    measure: bool,
}

#[derive(Clone, Copy, Default)]
pub struct RbvRec {
    pub frame: i32,
    pub first: i32,
    pub depth: i32,
    pub cause: i32,
    pub kind: i32,
    pub mismatch: bool,
    pub ms: f32,
}

/// The first mismatch: frame (-1 none), how many compares mismatched, where.
#[derive(Clone, Default)]
pub struct RbvMismatch {
    pub frame: i32,
    pub count: i32,
    pub va: u32,
    pub was: i32,
    pub now: i32,
    pub place: String,
}

struct Sn {
    base: Option<Base>,
    sessions: Vec<(i32, SnapRef)>,
    slots: Vec<Option<(i32, SnapRef)>>,
    resim: bool,
    rw: Rw,
    rbv: Vec<RbvRec>,
    rbv_total: usize,
    rbv_mm: RbvMismatch,
}

static mut SN: *mut Sn = core::ptr::null_mut();

fn sn() -> &'static mut Sn {
    unsafe {
        if SN.is_null() {
            SN = host_heap(|| {
                Box::into_raw(Box::new(Sn {
                    base: None,
                    sessions: Vec::new(),
                    slots: Vec::new(),
                    resim: false,
                    rw: Rw::default(),
                    rbv: vec![RbvRec::default(); RBV_N],
                    rbv_total: 0,
                    rbv_mm: RbvMismatch { frame: -1, ..Default::default() },
                }))
            });
        }
        &mut *SN
    }
}

/// The machinery is open (`sn.enabled`): a base has been taken.
pub fn enabled() -> bool {
    sn().base.is_some()
}

/// The one base, taken on first use (gw_snap_open's capture).
pub fn with_base<R>(f: impl FnOnce(&Base) -> R) -> R {
    let s = sn();
    if s.base.is_none() {
        let b = host_heap(Base::capture);
        s.base = Some(b);
    }
    host_heap(|| f(s.base.as_ref().unwrap()))
}

/// Closes the machinery: every snapshot held (sessions, slots, keyframes, the capture) goes, then the base (its
/// copy-on-write watch ends). The rollback ring survives (native memory, not state).
pub fn release() {
    let s = sn();
    host_heap(|| {
        s.sessions.clear();
        s.slots.clear();
        rw_end();
        s.rw.cap = None;
        s.base = None;
    });
    s.resim = false;
}

/// Loads `snap` and restores the arena flag (a load puts the static the flag lives in back too).
fn load(snap: &SnapRef) {
    let was = crate::arena::active();
    with_base(|b| b.load(snap));
    crate::arena::set_active(was);
}

// ---- frame-keyed sessions (gw_snap_open / save / load)

/// gw_snap_open(k): opens the machinery. 0 = ok.
pub fn open(_k: i32) -> i32 {
    with_base(|_| ());
    0
}
/// gw_snap_save(frame): the live state as session frame `frame` (replacing one of that frame).
pub fn save(frame: i32) {
    let snap = with_base(|b| b.save());
    let s = sn();
    host_heap(|| {
        s.sessions.retain(|x| x.0 != frame);
        s.sessions.push((frame, snap));
    });
}
/// gw_snap_load(frame): 0 = loaded, -1 = no such frame.
pub fn load_frame(frame: i32) -> i32 {
    if !enabled() {
        return -1;
    }
    let Some(snap) = sn().sessions.iter().find(|x| x.0 == frame).map(|x| x.1.clone()) else { return -1 };
    load(&snap);
    0
}

// ---- slots by index (gw_snap_reserve / save_index / load_index / slot_bytes)

/// gw_snap_reserve(n): makes sure n slots exist (at most MAX_SLOTS), opening the machinery; the slots now available.
pub fn reserve(n: i32) -> i32 {
    let n = (n.max(0) as usize).min(MAX_SLOTS);
    open(0);
    let s = sn();
    if s.slots.len() < n {
        host_heap(|| s.slots.resize(n, None));
    }
    s.slots.len() as i32
}
/// gw_snap_slot_count
pub fn slot_count() -> i32 {
    if enabled() { sn().slots.len() as i32 } else { 0 }
}
/// gw_snap_save_index(idx, tag)
pub fn save_index(idx: i32, tag: i32) {
    if !enabled() || idx < 0 || idx as usize >= sn().slots.len() {
        return;
    }
    let snap = with_base(|b| b.save());
    host_heap(|| sn().slots[idx as usize] = Some((tag, snap)));
}
/// gw_snap_load_index(idx): 0 = loaded; -1 = no such slot, or (unlike the C, which would copy the never-synced slot's
/// zero pages in) a slot never saved.
pub fn load_index(idx: i32) -> i32 {
    if !enabled() || idx < 0 {
        return -1;
    }
    let Some(Some((_, snap))) = sn().slots.get(idx as usize).cloned() else { return -1 };
    load(&snap);
    0
}
/// gw_snap_slot_bytes: what one full copy of the state costs (0 while closed)
pub fn slot_bytes() -> u64 {
    if enabled() { with_base(|b| b.state_bytes()) as u64 } else { 0 }
}

// ---- resimulation (gw_Snap_Resimulating, gw_Snap_LabResim, gw_Snap_SessionResim)

/// gw_Snap_Resimulating: `sn.enabled && sn.cur_is_resim`.
pub fn resimulating() -> bool {
    sn().resim
}
/// gw_Snap_LabResim(on): the Lab's silent re-simulation frame (only while the machinery is open).
pub fn lab_resim(on: bool) {
    if enabled() {
        sn().resim = on;
    }
}
/// gw_Snap_SessionResim: the rollback loop marks a resimulated frame (it may hold its own base for now).
pub fn session_resim(on: bool) {
    sn().resim = on;
}

// ---- the long rewind's keyframe store (gw_rw_*)

/// The replay cursor a keyframe keeps (gw_Replay_GetCursor / SetCursor). The replay player lives in the `melee`
/// crate: it installs these; unset, a keyframe keeps no cursor.
pub static mut REPLAY_CURSOR: Option<(fn() -> Vec<u8>, fn(&[u8]))> = None;

fn cursor_get() -> Vec<u8> {
    unsafe { REPLAY_CURSOR.map(|c| (c.0)()).unwrap_or_default() }
}
fn cursor_set(c: &[u8]) {
    unsafe {
        if let Some(h) = REPLAY_CURSOR {
            (h.1)(c)
        }
    }
}
fn user_block(user: &[u8]) -> [u8; RW_USER] {
    let mut u = [0u8; RW_USER];
    let n = user.len().min(RW_USER);
    u[..n].copy_from_slice(&user[..n]);
    u
}
fn ms_since(t: std::time::Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}
fn rw_latest_tag() -> i32 {
    let r = &sn().rw;
    r.keys.last().map_or(r.base_tag, |k| k.tag)
}

/// gw_rw_end
pub fn rw_end() {
    let r = &mut sn().rw;
    host_heap(|| {
        r.keys.clear();
        r.base = None;
    });
    r.on = false;
    r.delta_bytes = 0;
}
/// gw_rw_active
pub fn rw_active() -> bool {
    sn().rw.on
}
/// gw_rw_begin(tag, user): (re)starts the rewind with the live state as the base keyframe. 0 = ok.
pub fn rw_begin(tag: i32, user: &[u8]) -> i32 {
    let snap = with_base(|b| b.save());
    let cursor = cursor_get();
    let r = &mut sn().rw;
    host_heap(|| {
        r.keys.clear();
        r.base = Some(snap);
        r.base_user = user_block(user).to_vec();
        r.base_cursor = cursor;
    });
    r.base_tag = tag;
    r.delta_bytes = 0;
    r.on = true;
    0
}
/// gw_rw_save(tag, user): a keyframe of the live state, `tag` above every stored one. 0 = ok.
pub fn rw_save(tag: i32, user: &[u8]) -> i32 {
    let t0 = std::time::Instant::now();
    if !sn().rw.on || tag <= rw_latest_tag() || sn().rw.keys.len() >= RW_MAX_KEYS {
        return -1;
    }
    let prev = sn().rw.keys.last().map(|k| k.snap.clone()).or_else(|| sn().rw.base.clone());
    let (snap, fresh) = with_base(|b| b.save_after(prev.as_deref()));
    let cursor = cursor_get();
    let r = &mut sn().rw;
    host_heap(|| r.keys.push(Key { tag, snap, user: user_block(user), cursor, bytes: fresh }));
    r.delta_bytes += fresh;
    r.ms_save += ms_since(t0);
    r.n_save += 1;
    0
}
fn rw_index(tag: i32) -> Option<Option<usize>> {
    let r = &sn().rw;
    if !r.on {
        return None;
    }
    if tag == r.base_tag {
        return Some(None);
    }
    r.keys.iter().position(|k| k.tag == tag).map(Some)
}
/// gw_rw_load(tag): the keyframe tagged `tag` (the base included) into the live state; its user block, None when
/// there is no such keyframe.
pub fn rw_load(tag: i32) -> Option<Vec<u8>> {
    let t0 = std::time::Instant::now();
    let idx = rw_index(tag)?;
    let (snap, user, cursor) = {
        let r = &sn().rw;
        match idx {
            None => (r.base.clone()?, r.base_user.clone(), r.base_cursor.clone()),
            Some(i) => (r.keys[i].snap.clone(), r.keys[i].user.to_vec(), r.keys[i].cursor.clone()),
        }
    };
    let before = with_base(|b| b.stats().load_copied);
    load(&snap);
    let copied = with_base(|b| b.stats().load_copied) - before;
    cursor_set(&cursor);
    let r = &mut sn().rw;
    r.last_load_pages = copied as u32;
    r.ms_last_load = ms_since(t0);
    Some(user)
}
/// gw_rw_drop_from(tag): drops every keyframe tagged >= tag; -1 when the base itself would go.
pub fn rw_drop_from(tag: i32) -> i32 {
    let r = &mut sn().rw;
    if !r.on || r.base_tag >= tag {
        return -1;
    }
    host_heap(|| {
        while r.keys.last().is_some_and(|k| k.tag >= tag) {
            let k = r.keys.pop().unwrap();
            r.delta_bytes -= k.bytes;
        }
    });
    0
}
/// gw_rw_trim(limit): folds the keyframes tagged <= limit into the base (the base becomes the newest of them).
pub fn rw_trim(limit: i32) {
    let r = &mut sn().rw;
    host_heap(|| {
        while r.on && r.keys.first().is_some_and(|k| k.tag <= limit) {
            let k = r.keys.remove(0);
            r.delta_bytes -= k.bytes;
            r.base_tag = k.tag;
            r.base_user = k.user.to_vec();
            r.base_cursor = k.cursor;
            r.base = Some(k.snap);
        }
    });
}
/// gw_rw_find_le(tag): the newest keyframe tag <= tag (the base included); None (INT_MIN) when there is none.
pub fn rw_find_le(tag: i32) -> Option<i32> {
    let r = &sn().rw;
    if !r.on || r.base_tag > tag {
        return None;
    }
    Some(r.keys.iter().rev().find(|k| k.tag <= tag).map_or(r.base_tag, |k| k.tag))
}
/// gw_rw_base_tag
pub fn rw_base_tag() -> i32 {
    if sn().rw.on { sn().rw.base_tag } else { 0 }
}
/// gw_rw_latest
pub fn rw_latest() -> i32 {
    if sn().rw.on { rw_latest_tag() } else { 0 }
}
/// gw_rw_count: the keyframes and the base
pub fn rw_count() -> i32 {
    if sn().rw.on { sn().rw.keys.len() as i32 + 1 } else { 0 }
}
/// gw_rw_bytes: bytes held, the base keyframe's pages and every delta (the C: its full base image plus the deltas)
pub fn rw_bytes() -> f64 {
    let r = &sn().rw;
    if !r.on {
        return 0.0;
    }
    (r.base.as_ref().map_or(0, |b| b.bytes()) + r.delta_bytes) as f64
}
/// gw_rw_delta_bytes
pub fn rw_delta_bytes() -> f64 {
    if sn().rw.on { sn().rw.delta_bytes as f64 } else { 0.0 }
}
/// gw_rw_ms_last_load
pub fn rw_ms_last_load() -> f64 {
    sn().rw.ms_last_load
}
/// gw_rw_ms_save_avg
pub fn rw_ms_save_avg() -> f64 {
    let r = &sn().rw;
    if r.n_save > 0 { r.ms_save / r.n_save as f64 } else { 0.0 }
}
/// gw_rw_last_load_pages
pub fn rw_last_load_pages() -> u32 {
    sn().rw.last_load_pages
}
/// gw_rw_capture: a copy of the whole state for the exactness self-test. 0 = ok, -1 while closed.
pub fn rw_capture() -> i32 {
    if !enabled() {
        return -1;
    }
    let snap = with_base(|b| b.save());
    host_heap(|| sn().rw.cap = Some(snap));
    0
}
/// gw_rw_compare: the live state against the capture, which it frees. [every differing byte, the simulation's
/// differing bytes] and a 64-bit hash of each side's simulation bytes (capture, live); None without a capture. The
/// simulation is the C-layout game state (the OS heap); the arena and the statics hold the port's own bookkeeping,
/// the role the C's exemptions (the DVD stream, the light list, render-owned bytes) play.
pub fn rw_compare() -> Option<([f64; 2], [u64; 2])> {
    let cap = host_heap(|| sn().rw.cap.take())?;
    let live = with_base(|b| b.save());
    let (all, sim, hc, hl) = with_base(|b| b.compare(&cap, &live));
    if sim > 0 {
        for l in with_base(|b| b.diff(&cap, &live, 40)) {
            eprintln!("rewind test: {l}");
        }
    }
    host_heap(|| drop((cap, live)));
    Some(([all as f64, sim as f64], [hc, hl]))
}
/// gw_rw_measure(on): the C marks render-owned bytes while the self-test runs; here they are outside the simulation
/// range already, so this only records the request.
pub fn rw_measure(on: bool) {
    sn().rw.measure = on && enabled();
}

// ---- savestate files (gw_snap_file_*). The Rust state is its ranges at their addresses, so a file loads into a
// process whose ranges are where they were (the same run; another one only where the addresses repeat)
//   "GDSTATE1" | u32 hdr_len | hdr | u32 nranges | nranges x { u64 addr, u64 len, u32 npages }
//   | per range: npages x { u32 page index, 4096 bytes } | u64 FNV-1a of everything after the caller header
// Integers little-endian. A load reads and verifies the whole file before it touches the game.

const MAGIC: &[u8; 8] = b"GDSTATE1";

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x100000001b3))
}

/// gw_snap_file_save(path, hdr): the live state after the caller's header, written to `<path>.tmp` then renamed. 0 = ok.
pub fn file_save(path: &str, hdr: &[u8]) -> i32 {
    let snap = with_base(|b| b.save());
    let img = with_base(|b| b.export(&snap));
    host_heap(|| drop(snap));
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(hdr.len() as u32).to_le_bytes());
    out.extend_from_slice(hdr);
    let body = out.len();
    out.extend_from_slice(&(img.len() as u32).to_le_bytes());
    for r in &img {
        out.extend_from_slice(&(r.addr as u64).to_le_bytes());
        out.extend_from_slice(&(r.len as u64).to_le_bytes());
        out.extend_from_slice(&(r.pages.len() as u32).to_le_bytes());
    }
    for r in &img {
        for (pi, p) in &r.pages {
            out.extend_from_slice(&pi.to_le_bytes());
            out.extend_from_slice(p);
        }
    }
    let h = fnv(0xcbf29ce484222325, &out[body..]);
    out.extend_from_slice(&h.to_le_bytes());
    let tmp = format!("{path}.tmp");
    if std::fs::write(&tmp, &out).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return -1;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return -1;
    }
    0
}

fn read_header(f: &[u8], cap: usize) -> Option<Vec<u8>> {
    if f.len() < 12 || &f[..8] != MAGIC {
        return None;
    }
    let len = u32::from_le_bytes(f[8..12].try_into().unwrap()) as usize;
    (len <= cap && f.len() >= 12 + len).then(|| f[12..12 + len].to_vec())
}

/// gw_snap_file_header(path, cap): only the caller's header; None when the file is not a state (or it is longer than cap).
pub fn file_header(path: &str, cap: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut head = [0u8; 12];
    f.read_exact(&mut head).ok()?;
    let len = u32::from_le_bytes(head[8..12].try_into().unwrap()) as usize;
    if &head[..8] != MAGIC || len > cap {
        return None;
    }
    let mut h = vec![0u8; len];
    f.read_exact(&mut h).ok()?;
    let mut all = head.to_vec();
    all.extend_from_slice(&h);
    read_header(&all, cap)
}

/// gw_snap_file_rewrite_header(path, hdr): the header rewritten in place (same length: renaming a state). 0 = ok.
pub fn file_rewrite_header(path: &str, hdr: &[u8]) -> i32 {
    use std::io::{Read, Seek, SeekFrom, Write};
    let Ok(mut f) = std::fs::OpenOptions::new().read(true).write(true).open(path) else { return -1 };
    let mut head = [0u8; 12];
    if f.read_exact(&mut head).is_err() || &head[..8] != MAGIC || u32::from_le_bytes(head[8..12].try_into().unwrap()) as usize != hdr.len() {
        return -1;
    }
    if f.seek(SeekFrom::Start(12)).is_err() || f.write_all(hdr).is_err() || f.flush().is_err() {
        return -1;
    }
    0
}

/// gw_snap_file_load(path): 0 = loaded; -1 unreadable, -2 made for another memory layout, -3 damaged.
pub fn file_load(path: &str) -> i32 {
    let Ok(f) = std::fs::read(path) else { return -1 };
    let Some(hdr) = read_header(&f, usize::MAX) else { return -1 };
    let body = 12 + hdr.len();
    let rd32 = |at: usize| f.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let rd64 = |at: usize| f.get(at..at + 8).map(|b| u64::from_le_bytes(b.try_into().unwrap()));
    let Some(n) = rd32(body) else { return -1 };
    let mut at = body + 4;
    let mut ranges = Vec::new();
    for _ in 0..n {
        let (Some(addr), Some(len), Some(np)) = (rd64(at), rd64(at + 8), rd32(at + 16)) else { return -3 };
        ranges.push((addr as usize, len as usize, np as usize));
        at += 20;
    }
    let layout = with_base(|b| b.layout());
    if ranges.len() != layout.len() || ranges.iter().zip(&layout).enumerate().any(|(ri, (r, l))| r.0 != l.0 || (ri > 0 && r.1 != l.1)) {
        return -2;
    }
    let mut img = Vec::new();
    for &(addr, len, np) in &ranges {
        let mut pages = Vec::with_capacity(np);
        for _ in 0..np {
            let (Some(pi), Some(p)) = (rd32(at), f.get(at + 4..at + 4 + 4096)) else { return -3 };
            pages.push((pi, p.to_vec()));
            at += 4100;
        }
        img.push(RangeImage { addr, len, pages });
    }
    if rd64(at) != Some(fnv(0xcbf29ce484222325, &f[body..at])) || f.len() != at + 8 {
        return -3;
    }
    let Some(snap) = with_base(|b| b.import(&img)) else { return -2 };
    let was = crate::arena::active();
    with_base(|b| b.load_full(&snap));
    crate::arena::set_active(was);
    host_heap(|| drop((snap, img)));
    0
}

// ---- the rollback visualiser (gw_RbViz_*): native memory only, nothing here is game state

/// gw_RbViz_Push
pub fn rbviz_push(frame: i32, first: i32, depth: i32, cause: i32, ms: f64, kind: i32) {
    let s = sn();
    s.rbv[s.rbv_total % RBV_N] = RbvRec { frame, first, depth, cause, kind, mismatch: false, ms: ms as f32 };
    s.rbv_total += 1;
}
/// gw_RbViz_MarkMismatch
pub fn rbviz_mark_mismatch() {
    let s = sn();
    if s.rbv_total > 0 {
        s.rbv[(s.rbv_total - 1) % RBV_N].mismatch = true;
    }
}
/// gw_RbViz_Total
pub fn rbviz_total() -> i32 {
    sn().rbv_total as i32
}
/// gw_RbViz_Get: `back` 0 is the newest; None when there is no such record
pub fn rbviz_get(back: i32) -> Option<RbvRec> {
    let s = sn();
    if back < 0 || back as usize >= s.rbv_total || back as usize >= RBV_N {
        return None;
    }
    Some(s.rbv[(s.rbv_total - 1 - back as usize) % RBV_N])
}
/// gw_RbViz_FirstMismatch
pub fn rbviz_first_mismatch() -> RbvMismatch {
    sn().rbv_mm.clone()
}
/// gw_RbViz_Reset
pub fn rbviz_reset() {
    let s = sn();
    s.rbv_total = 0;
    host_heap(|| s.rbv_mm = RbvMismatch { frame: -1, ..Default::default() });
}
/// gw_RbViz_Desync: a desync the netplay checksum found (no bytes to compare across peers)
pub fn rbviz_desync(frame: i32, local: u32, peer: u32) {
    let s = sn();
    s.rbv_mm.count += 1;
    if s.rbv_mm.frame >= 0 {
        return;
    }
    s.rbv_mm.frame = frame;
    let place = if local == 0 && peer == 0 {
        format!("rollback: frame {frame} needed a correction, its snapshot was gone")
    } else {
        format!("netplay checksum: local {local:08X}, peer {peer:08X}")
    };
    host_heap(|| s.rbv_mm.place = place);
    rbviz_mark_mismatch();
}

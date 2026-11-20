//! The YouGame app's pad mirror: inside the app's Windows sandbox no USB device can be opened, so with
//! `"controllers": "mirror"` in yougame-native.json the app's controller helper keeps the GameCube adapter, writes
//! every raw report it reads into an 80-byte shared section, and hands us that section's handle (already
//! duplicated into this process) as the stdin push `yg {"push":{"type":"pads","handle":N}}` ([`crate::yougame`]).
//! The reports are the adapter's own 37 bytes, so the rest of the adapter path (origins, trigger rest,
//! recalibration) is unchanged: this is just one more [`Device`].
//!
//! Section layout (the helper's; verified against the app's own size check, 80, and the reader in
//! Hero88go's melee-unlocked `gc_adapter.cpp`, whose static_assert pins report at 29 and rumble_seq at 68):
//! ```text
//!  0  [u8;4] "YGPM"        4  u32 version (1)      8  u32 seq (even: stable, odd: being written)
//! 12  u32 reports written  16  f64 helper clock ms 24  f32 helper reads/s   28  u8 connected
//! 29  [u8;37] raw report   66  [u8;2] reserved     68  u32 rumble_seq (ours) 72 [u8;4] rumble per port (ours)
//! ```
//! The helper rewrites the report on every read, so `reports` standing still for 500 ms means it or the adapter
//! stopped: the ports then read as empty (falls back to the other controllers) instead of holding the last state.
use super::{decode::GC_PAYLOAD_SIZE, Device, ReadResult};
use std::sync::atomic::{fence, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const SIZE: usize = 80;
pub const MAGIC: &[u8; 4] = b"YGPM";
pub const VERSION: u32 = 1;
pub const OFF_VERSION: usize = 4;
pub const OFF_SEQ: usize = 8;
pub const OFF_REPORTS: usize = 12;
pub const OFF_CONNECTED: usize = 28;
pub const OFF_REPORT: usize = 29;
pub const OFF_RUMBLE_SEQ: usize = 68;
pub const OFF_RUMBLE: usize = 72;
/// No new report for this long: the helper or the adapter stopped.
pub const STALE: Duration = Duration::from_millis(500);
/// A report with every port empty (what a stale or disconnected mirror reads as).
pub const EMPTY: [u8; GC_PAYLOAD_SIZE] = {
    let mut r = [0u8; GC_PAYLOAD_SIZE];
    r[0] = 0x21;
    r
};

/// One consistent look at the section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub reports: u32,
    pub connected: bool,
    pub report: [u8; GC_PAYLOAD_SIZE],
}

struct Track {
    reports: u32,
    moved: Instant,
    /// What the last `read` handed out: a live report or [`EMPTY`].
    empty_sent: bool,
}

/// A mapped section. `base` points at [`SIZE`] bytes, 8-aligned, valid for the life of this value.
pub struct Mirror {
    base: *mut u8,
    /// The handle it was mapped from (0 in tests); a different handle from the app means a new helper.
    handle: u64,
    unmap: Option<fn(*mut u8)>,
    track: Mutex<Track>,
}

// SAFETY: the section is shared memory read with volatile loads under a seqlock; our writes (rumble) are single
// bytes plus an atomic counter.
unsafe impl Send for Mirror {}
unsafe impl Sync for Mirror {}

impl Mirror {
    /// Checks the header. `None` for a section that is not the helper's (wrong magic or version).
    ///
    /// # Safety
    /// `base` must point at [`SIZE`] readable and writable bytes, 8-aligned, that outlive the result (until
    /// `unmap` runs in drop).
    pub unsafe fn from_raw(base: *mut u8, handle: u64, unmap: Option<fn(*mut u8)>) -> Option<Self> {
        let mut magic = [0u8; 4];
        for (i, m) in magic.iter_mut().enumerate() {
            *m = std::ptr::read_volatile(base.add(i));
        }
        let version = std::ptr::read_volatile(base.add(OFF_VERSION) as *const u32);
        if &magic != MAGIC || version != VERSION {
            if let Some(u) = unmap {
                u(base);
            }
            return None;
        }
        Some(Self { base, handle, unmap, track: Mutex::new(Track { reports: 0, moved: Instant::now(), empty_sent: true }) })
    }

    fn atomic(&self, off: usize) -> &AtomicU32 {
        // SAFETY: in bounds and 4-aligned (base is 8-aligned); AtomicU32 has u32's layout.
        unsafe { &*(self.base.add(off) as *const AtomicU32) }
    }

    /// A torn-free copy (seqlock), `None` while the helper is mid-write.
    pub fn snapshot(&self) -> Option<Snapshot> {
        for _ in 0..8 {
            let before = self.atomic(OFF_SEQ).load(Ordering::Acquire);
            if before & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            let mut report = [0u8; GC_PAYLOAD_SIZE];
            // SAFETY: offsets are inside the SIZE bytes.
            let (reports, connected) = unsafe {
                for (i, b) in report.iter_mut().enumerate() {
                    *b = std::ptr::read_volatile(self.base.add(OFF_REPORT + i));
                }
                (
                    std::ptr::read_volatile(self.base.add(OFF_REPORTS) as *const u32),
                    std::ptr::read_volatile(self.base.add(OFF_CONNECTED)) != 0,
                )
            };
            fence(Ordering::Acquire);
            if self.atomic(OFF_SEQ).load(Ordering::Relaxed) != before {
                continue;
            }
            return Some(Snapshot { reports, connected, report });
        }
        None
    }

    /// One poll: a new live report, [`EMPTY`] once when the mirror goes quiet or the adapter goes away, else
    /// nothing new.
    pub fn poll(&self, now: Instant) -> Option<[u8; GC_PAYLOAD_SIZE]> {
        let snap = self.snapshot()?;
        let mut t = self.track.lock().unwrap();
        let fresh = snap.reports != t.reports;
        if fresh {
            t.reports = snap.reports;
            t.moved = now;
        }
        let live = snap.connected && snap.reports != 0 && snap.report[0] == 0x21 && now.duration_since(t.moved) < STALE;
        if live && fresh {
            t.empty_sent = false;
            return Some(snap.report);
        }
        if !live && !t.empty_sent {
            t.empty_sent = true;
            return Some(EMPTY);
        }
        None
    }

    pub fn handle(&self) -> u64 {
        self.handle
    }
}

impl Device for Mirror {
    fn read(&self) -> ReadResult {
        // The adapter thread expects a bounded blocking read: poll for up to 20 ms (the WinUSB timeout).
        let until = Instant::now() + Duration::from_millis(20);
        loop {
            #[cfg(windows)]
            if crate::yougame::pads_handle() != self.handle {
                return ReadResult::Lost; // a restarted helper sent a new section: reopen on it
            }
            if let Some(r) = self.poll(Instant::now()) {
                return ReadResult::Report(r);
            }
            if Instant::now() >= until {
                return ReadResult::Timeout;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    /// `{0x11, p0..p3}`: the helper drives the motors from our bytes once `rumble_seq` moves.
    fn rumble(&self, cmd: &[u8; 5]) {
        // SAFETY: in bounds.
        unsafe {
            for i in 0..4 {
                std::ptr::write_volatile(self.base.add(OFF_RUMBLE + i), cmd[1 + i]);
            }
        }
        fence(Ordering::Release);
        self.atomic(OFF_RUMBLE_SEQ).fetch_add(1, Ordering::AcqRel);
    }
    fn name(&self) -> &str {
        "YouGame pad mirror"
    }
}

impl Drop for Mirror {
    fn drop(&mut self) {
        if let Some(u) = self.unmap {
            u(self.base);
        }
    }
}

/// Maps the section the app handed over, if it has (Windows; the handle is already ours).
#[cfg(windows)]
pub fn open() -> Option<std::sync::Arc<Mirror>> {
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Memory::{MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE, MEMORY_MAPPED_VIEW_ADDRESS};
    let handle = crate::yougame::pads_handle();
    if handle == 0 {
        return None;
    }
    fn unmap(p: *mut u8) {
        // SAFETY: p came from MapViewOfFile below.
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: p as *mut _ });
        }
    }
    static SAID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let say = SAID.swap(handle, Ordering::Relaxed) != handle;
    // SAFETY: the app duplicated the section handle into this process; we map only SIZE bytes of it.
    let view = unsafe { MapViewOfFile(handle as usize as _, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, SIZE) };
    if view.Value.is_null() {
        if say {
            crate::plog!("gw: gc adapter: the YouGame app's pad mirror could not be mapped (error {})", unsafe { GetLastError() });
        }
        return None;
    }
    // SAFETY: a fresh view of SIZE bytes, page-aligned; unmapped in drop.
    let m = unsafe { Mirror::from_raw(view.Value as *mut u8, handle, Some(unmap)) };
    match &m {
        Some(_) if say => crate::plog!("gw: gc adapter: reading the YouGame app's adapter (pad mirror)"),
        None if say => crate::plog!("gw: gc adapter: the YouGame app's pad mirror has an unknown layout; not used"),
        _ => {}
    }
    m.map(std::sync::Arc::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::Adapter;
    use crate::pad::{PADStatus, PAD_CHANMAX};
    use std::sync::Arc;

    /// A fake section: 80 bytes, 8-aligned.
    struct Fake(Box<[u64; SIZE / 8]>);
    impl Fake {
        fn new() -> Self {
            let mut f = Fake(Box::new([0; SIZE / 8]));
            f.bytes()[..4].copy_from_slice(MAGIC);
            f.bytes()[OFF_VERSION..OFF_VERSION + 4].copy_from_slice(&VERSION.to_le_bytes());
            f
        }
        fn bytes(&mut self) -> &mut [u8] {
            // SAFETY: plain bytes of the u64 array.
            unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u8, SIZE) }
        }
        fn ptr(&mut self) -> *mut u8 {
            self.0.as_mut_ptr() as *mut u8
        }
        fn write(&mut self, reports: u32, connected: bool, report: &[u8; GC_PAYLOAD_SIZE]) {
            let b = self.bytes();
            let seq = u32::from_le_bytes(b[OFF_SEQ..OFF_SEQ + 4].try_into().unwrap());
            b[OFF_SEQ..OFF_SEQ + 4].copy_from_slice(&(seq + 2).to_le_bytes());
            b[OFF_REPORTS..OFF_REPORTS + 4].copy_from_slice(&reports.to_le_bytes());
            b[OFF_CONNECTED] = connected as u8;
            b[OFF_REPORT..OFF_REPORT + GC_PAYLOAD_SIZE].copy_from_slice(report);
        }
    }

    /// Port 1 wired at a 128 rest, stick X and A as given (a button held at plug-in is masked as stuck).
    fn port1_report(stick_x: u8, a: bool) -> [u8; GC_PAYLOAD_SIZE] {
        let mut r = EMPTY;
        r[1..10].copy_from_slice(&[0x14, a as u8, 0x00, stick_x, 128, 128, 128, 30, 30]);
        r
    }

    #[test]
    fn layout_and_seqlock() {
        let mut f = Fake::new();
        let rep = port1_report(128, false);
        f.write(1, true, &rep);
        let m = unsafe { Mirror::from_raw(f.ptr(), 0, None) }.expect("header accepted");
        let t0 = Instant::now();
        assert_eq!(m.poll(t0), Some(rep));
        assert_eq!(m.poll(t0), None, "same report count: nothing new");
        // Mid-write (odd seq): no torn copy.
        f.bytes()[OFF_SEQ] |= 1;
        assert_eq!(m.snapshot(), None);
        f.bytes()[OFF_SEQ] &= !1;
        // Quiet for 500 ms: the ports read empty once.
        assert_eq!(m.poll(t0 + STALE), Some(EMPTY));
        assert_eq!(m.poll(t0 + STALE * 2), None);
        // Reports again.
        f.write(2, true, &rep);
        assert_eq!(m.poll(t0 + STALE * 3), Some(rep));
        // Adapter unplugged from the app's side.
        f.write(3, false, &rep);
        assert_eq!(m.poll(t0 + STALE * 3), Some(EMPTY));
        // Rumble goes into our bytes and bumps rumble_seq.
        m.rumble(&[0x11, 1, 0, 0, 1]);
        assert_eq!(&f.bytes()[OFF_RUMBLE..OFF_RUMBLE + 4], &[1, 0, 0, 1]);
        assert_eq!(f.bytes()[OFF_RUMBLE_SEQ], 1);
    }

    #[test]
    fn wrong_header_is_refused() {
        let mut f = Fake::new();
        f.bytes()[0] = b'X';
        assert!(unsafe { Mirror::from_raw(f.ptr(), 0, None) }.is_none());
        let mut f = Fake::new();
        f.bytes()[OFF_VERSION] = 2;
        assert!(unsafe { Mirror::from_raw(f.ptr(), 0, None) }.is_none());
    }

    /// The mirror through the same adapter path WinUSB feeds: port 1 shows up calibrated from its rest.
    #[test]
    fn feeds_the_adapter() {
        let mut f = Fake::new();
        f.write(1, true, &port1_report(128, false));
        let p = f.ptr() as usize;
        let opener: crate::adapter::Opener =
            Box::new(move || unsafe { Mirror::from_raw(p as *mut u8, 0, None) }.map(|m| Arc::new(m) as Arc<dyn Device>));
        let mut a = Adapter::with_opener(opener, false);
        assert!(a.init());
        let mut st = unsafe { std::mem::zeroed::<[PADStatus; PAD_CHANMAX]>() };
        let t = Instant::now();
        while !a.read(&mut st) {
            assert!(t.elapsed() < Duration::from_secs(2), "no report reached the adapter");
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(st[0].stickX, 0, "origin is the plug-in rest");
        f.write(2, true, &port1_report(200, true));
        let t = Instant::now();
        loop {
            a.read(&mut st);
            if st[0].stickX == 72 {
                break;
            }
            assert!(t.elapsed() < Duration::from_secs(2), "the new report never arrived");
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(st[0].button & 0x0100 != 0, "A held");
        a.shutdown();
        drop(f);
    }
}

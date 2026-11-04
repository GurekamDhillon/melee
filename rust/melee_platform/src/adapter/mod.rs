//! The raw GameCube controller adapter (WUP-028, 057E:0337). Port of `pc/platform/gc_adapter.c`.
//!
//! Same structure as the C: a reader thread parks in a blocking (20 ms bounded) device read and
//! stores the newest 37-byte report under a lock; the game thread only copies that report out and
//! decodes it ([`decode::gc_decode`]), so a frame never waits on USB. A hotplug scanner thread
//! reopens the device (plugged in later, or after a resume). Ownership follows focus: suspend
//! closes the device so another program can claim it, resume reopens it. `MELEE_PAD_IGNORE_ADAPTER=1`
//! or `MELEE_UNATTENDED=1` forbid every open (`gc_adapter_policy.h`).
//!
//! Transports, tried in the C's order: WinUSB ([`win_usb`], Zadig-bound adapters), then the
//! Windows HID class driver ([`win_hid`], the stock no-Zadig path). Non-Windows: none yet
//! (the C's `gc_adapter_linux.inc` is not ported). Tests inject a fake [`Device`] through
//! [`Adapter::with_opener`] (the C's `gw_gc_test_open`).
pub mod decode;
#[cfg(windows)]
pub mod win_hid;
#[cfg(windows)]
pub mod win_usb;

use crate::pad::*;
use crate::plog;
use decode::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub const GC_ADAPTER_VID: u16 = 0x057E;
pub const GC_ADAPTER_PID: u16 = 0x0337;

/// One blocking read's outcome (`gw_gc_read_blocking`: 1 / 0 / -1).
pub enum ReadResult {
    Report([u8; GC_PAYLOAD_SIZE]),
    Timeout,
    Lost,
}

/// An open adapter. `read` is called only from the reader thread; `rumble` and `cancel` from
/// others.
pub trait Device: Send + Sync {
    fn read(&self) -> ReadResult;
    /// Send `{0x11, p0, p1, p2, p3}`.
    fn rumble(&self, cmd: &[u8; 5]);
    /// Abort a pending read (shutdown).
    fn cancel(&self) {}
    fn name(&self) -> &str {
        "device"
    }
}

pub type Opener = Box<dyn Fn() -> Option<Arc<dyn Device>> + Send + Sync>;

/// `gw_gc_env_on`.
pub fn env_on(key: &str) -> bool {
    std::env::var(key).map(|v| v.starts_with('1')).unwrap_or(false)
}
/// `gw_gc_forbidden`.
pub fn forbidden() -> bool {
    env_on("MELEE_PAD_IGNORE_ADAPTER") || env_on("MELEE_UNATTENDED")
}
/// `gw_gc_policy_allows`.
pub fn policy_allows(ignored: bool, unattended: bool, focused: bool) -> bool {
    !ignored && !unattended && focused
}

struct Shared {
    payload: Mutex<Option<[u8; GC_PAYLOAD_SIZE]>>,
    quit: AtomicBool,
    lost: AtomicBool,
    reports: AtomicU32,
}

struct Open {
    dev: Arc<dyn Device>,
    shared: Arc<Shared>,
    reader: Option<JoinHandle<()>>,
}

struct Inner {
    open: Option<Open>,
}

struct Core {
    inner: Mutex<Inner>, // gw_gc_init_srw
    opener: Opener,
    forbidden: bool,
    focused: AtomicBool,
    suspended: AtomicBool,
    recal: AtomicBool,
    scan_quit: AtomicBool,
    scan_wake: (Mutex<bool>, Condvar),
    rumble_state: Mutex<[u8; GC_PORTS]>,
}

impl Core {
    fn allowed(&self) -> bool {
        policy_allows(self.forbidden, false, self.focused.load(Ordering::SeqCst))
    }

    /// `gw_gc_init_locked` + `gw_gc_open_locked`.
    fn open_locked(&self, inner: &mut Inner, quiet: bool) -> bool {
        if inner.open.is_some() {
            return true;
        }
        if !self.allowed() || self.suspended.load(Ordering::SeqCst) {
            return false;
        }
        let Some(dev) = (self.opener)() else {
            if !quiet {
                plog!("gw: gc adapter: no WUP-028 adapter found; using the gamepad path instead.");
            }
            return false;
        };
        let shared = Arc::new(Shared {
            payload: Mutex::new(None),
            quit: AtomicBool::new(false),
            lost: AtomicBool::new(false),
            reports: AtomicU32::new(0),
        });
        let (d2, s2) = (dev.clone(), shared.clone());
        let reader = std::thread::Builder::new()
            .name("gc-adapter-reader".into())
            .spawn(move || {
                while !s2.quit.load(Ordering::SeqCst) {
                    match d2.read() {
                        ReadResult::Report(r) => {
                            *s2.payload.lock().unwrap() = Some(r);
                            s2.reports.fetch_add(1, Ordering::SeqCst);
                        }
                        ReadResult::Timeout => {}
                        ReadResult::Lost => {
                            s2.lost.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            })
            .ok();
        inner.open = Some(Open { dev, shared, reader });
        // An open never survives a suspend that raced it.
        if !self.allowed() || self.suspended.load(Ordering::SeqCst) {
            plog!("gw: gc adapter: opened while suspended - closing it again");
            Self::shutdown_locked(inner);
            return false;
        }
        true
    }

    /// `gw_gc_shutdown_locked`.
    fn shutdown_locked(inner: &mut Inner) {
        if let Some(mut o) = inner.open.take() {
            o.shared.quit.store(true, Ordering::SeqCst);
            o.dev.cancel();
            if let Some(h) = o.reader.take() {
                let _ = h.join();
            }
        }
    }

    fn wake_scanner(&self) {
        let (m, c) = &self.scan_wake;
        *m.lock().unwrap() = true;
        c.notify_all();
    }
}

/// The adapter: owns the reader and scanner threads.
pub struct Adapter {
    core: Arc<Core>,
    cal: Calibration,
    scanner: Option<JoinHandle<()>>,
}

impl Adapter {
    /// The real device ([`win_hid::open`] on Windows; nothing elsewhere). Respects the env policy.
    pub fn new_system() -> Self {
        #[cfg(windows)]
        let opener: Opener = Box::new(|| {
            // WinUSB first (Zadig-bound adapters), then the HID class driver, as in the C.
            if let Some(d) = win_usb::open() {
                return Some(d as Arc<dyn Device>);
            }
            win_hid::open().map(|d| d as Arc<dyn Device>)
        });
        #[cfg(not(windows))]
        let opener: Opener = Box::new(|| None);
        Self::with_opener(opener, forbidden())
    }

    /// Any device source (tests: the C's `gw_gc_test_open`).
    pub fn with_opener(opener: Opener, forbidden: bool) -> Self {
        Self {
            core: Arc::new(Core {
                inner: Mutex::new(Inner { open: None }),
                opener,
                forbidden,
                focused: AtomicBool::new(true),
                suspended: AtomicBool::new(false),
                recal: AtomicBool::new(false),
                scan_quit: AtomicBool::new(false),
                scan_wake: (Mutex::new(false), Condvar::new()),
                rumble_state: Mutex::new([0; GC_PORTS]),
            }),
            cal: Calibration::default(),
            scanner: None,
        }
    }

    /// `gw_gc_adapter_init`: one open attempt now.
    pub fn init(&self) -> bool {
        let mut inner = self.core.inner.lock().unwrap();
        self.core.open_locked(&mut inner, false)
    }

    /// `gw_gc_adapter_start_hotplug`: retry the open once a second (and at once on resume).
    pub fn start_hotplug(&mut self) {
        if self.core.forbidden || self.scanner.is_some() {
            return;
        }
        let core = self.core.clone();
        self.scanner = std::thread::Builder::new()
            .name("gc-adapter-scan".into())
            .spawn(move || loop {
                {
                    let (m, c) = &core.scan_wake;
                    let mut woke = m.lock().unwrap();
                    if !*woke {
                        woke = c.wait_timeout(woke, Duration::from_secs(1)).unwrap().0;
                    }
                    *woke = false;
                }
                if core.scan_quit.load(Ordering::SeqCst) {
                    break;
                }
                if core.suspended.load(Ordering::SeqCst) || !core.allowed() {
                    continue;
                }
                let mut inner = core.inner.lock().unwrap();
                core.open_locked(&mut inner, true);
            })
            .ok();
    }

    /// `gw_gc_adapter_present`.
    pub fn present(&self) -> bool {
        self.core.inner.lock().unwrap().open.is_some()
    }
    /// `gw_gc_adapter_suspended`.
    pub fn suspended(&self) -> bool {
        self.core.suspended.load(Ordering::SeqCst)
    }
    /// Window focus, the C's `gw_gc_allowed` foreground test.
    pub fn set_focused(&self, focused: bool) {
        self.core.focused.store(focused, Ordering::SeqCst);
    }
    /// Reports received since the current open.
    pub fn report_count(&self) -> u32 {
        let inner = self.core.inner.lock().unwrap();
        inner.open.as_ref().map(|o| o.shared.reports.load(Ordering::SeqCst)).unwrap_or(0)
    }

    /// `gw_gc_adapter_suspend`. Returns whether an open adapter was released.
    pub fn suspend(&self) -> bool {
        let mut inner = self.core.inner.lock().unwrap();
        self.core.suspended.store(true, Ordering::SeqCst);
        let was = inner.open.is_some();
        Core::shutdown_locked(&mut inner);
        plog!(
            "gw: gc adapter: suspended (window in the background) - {}",
            if was { "adapter released for other programs" } else { "no adapter was open; scanner paused" }
        );
        was
    }

    /// `gw_gc_adapter_resume`: the scanner reopens off the game thread. Calibration is kept.
    pub fn resume(&self) {
        if !self.core.allowed() {
            return;
        }
        if !self.core.suspended.swap(false, Ordering::SeqCst) {
            return;
        }
        plog!("gw: gc adapter: resumed (window focused) - reclaiming");
        self.core.wake_scanner();
    }

    /// `gw_gc_adapter_recalibrate`.
    pub fn recalibrate(&self) {
        self.core.recal.store(true, Ordering::SeqCst);
        plog!("gw: gc adapter: recalibrating on the next poll -- release everything now");
    }

    /// `gw_gc_adapter_shutdown`.
    pub fn shutdown(&self) {
        let mut inner = self.core.inner.lock().unwrap();
        Core::shutdown_locked(&mut inner);
    }

    /// `gw_gc_adapter_read`: decode the newest report into `st` (only ports with a controller
    /// in the adapter are written). Returns whether any port was written.
    pub fn read(&mut self, st: &mut [PADStatus; PAD_CHANMAX]) -> bool {
        let snap = {
            let mut inner = self.core.inner.lock().unwrap();
            if !self.core.allowed() {
                Core::shutdown_locked(&mut inner);
                return false;
            }
            if inner.open.is_some() && self.core.suspended.load(Ordering::SeqCst) {
                plog!("gw: gc adapter: BUG open while suspended - closing it");
                Core::shutdown_locked(&mut inner);
                return false;
            }
            let Some(o) = inner.open.as_ref() else { return false };
            if o.shared.lost.load(Ordering::SeqCst) {
                plog!("gw: gc adapter: device went away; reverting to the gamepad path");
                Core::shutdown_locked(&mut inner);
                return false;
            }
            let p = *o.shared.payload.lock().unwrap();
            match p {
                Some(p) => p,
                None => return false,
            }
        };
        let recal = self.core.recal.swap(false, Ordering::SeqCst);
        gc_decode(&mut self.cal, &snap, st, recal)
    }

    /// `gw_gc_adapter_rumble`: only state changes are sent.
    pub fn rumble(&self, chan: usize, on: bool) {
        if chan >= GC_PORTS {
            return;
        }
        let inner = self.core.inner.lock().unwrap();
        let Some(o) = inner.open.as_ref() else { return };
        if !self.core.allowed() {
            return;
        }
        let mut state = self.core.rumble_state.lock().unwrap();
        let v = on as u8;
        if state[chan] == v {
            return;
        }
        state[chan] = v;
        let cmd = [0x11, state[0], state[1], state[2], state[3]];
        o.dev.rumble(&cmd);
    }
}

/// A read-only view of the adapter for another thread (early inputs' sampler): the newest report decoded with its own
/// copy of the calibration. It never opens, closes or recalibrates the device.
pub struct AdapterPeek {
    core: Arc<Core>,
    cal: Calibration,
}

impl Adapter {
    pub fn peeker(&self) -> AdapterPeek {
        AdapterPeek { core: self.core.clone(), cal: self.cal.clone() }
    }
}

impl AdapterPeek {
    pub fn read(&mut self, st: &mut [PADStatus; PAD_CHANMAX]) -> bool {
        let snap = {
            let inner = self.core.inner.lock().unwrap();
            if !self.core.allowed() || self.core.suspended.load(Ordering::SeqCst) {
                return false;
            }
            let Some(o) = inner.open.as_ref() else { return false };
            if o.shared.lost.load(Ordering::SeqCst) {
                return false;
            }
            let p = *o.shared.payload.lock().unwrap();
            match p {
                Some(p) => p,
                None => return false,
            }
        };
        gc_decode(&mut self.cal, &snap, st, false)
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.core.scan_quit.store(true, Ordering::SeqCst);
        self.core.wake_scanner();
        if let Some(h) = self.scanner.take() {
            let _ = h.join();
        }
        self.shutdown();
    }
}

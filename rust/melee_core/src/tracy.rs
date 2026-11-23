//! Tracy instrumentation (docs/rust-port/notes/profiling.md). Built only with the cargo feature `profile-tracy`
//! (`cargo build -p melee --release --features profile-tracy`); without it every function here is an empty
//! `#[inline(always)]` body and [`Zone`] is a zero-sized type, so the default build carries no code and no data.
//!
//! With the feature the client starts on the first call and streams to a Tracy GUI (or `tracy-capture`) on this
//! machine. Zones are named with static strings; a zone may carry a number ([`Zone::value`]) and a text
//! ([`Zone::text`]), e.g. an upload's byte size or a mod's name.
//!
//! Simulation safety: Tracy only reads clocks; it never touches game memory, the game arena or any snapshot-covered
//! static. The client's state lives in its own allocations outside the rollback snapshot (net.md's statics rule).

/// An open zone; closed when dropped. Zero-sized without `profile-tracy`.
#[must_use = "a zone ends when dropped; bind it: let _z = tracy::zone(..)"]
pub struct Zone {
    #[cfg(feature = "profile-tracy")]
    z: Option<tracy_client::Span>,
}

#[cfg(feature = "profile-tracy")]
mod imp {
    /// The running client (started once, never stopped: Tracy keeps it for the process).
    #[inline]
    pub fn client() -> tracy_client::Client {
        tracy_client::Client::start()
    }
}

/// Is the Tracy client compiled in (the `profile-tracy` feature)?
#[inline(always)]
pub const fn enabled() -> bool {
    cfg!(feature = "profile-tracy")
}

/// Open a zone named `name` at the caller's source location.
#[inline(always)]
#[track_caller]
pub fn zone(name: &'static str) -> Zone {
    #[cfg(feature = "profile-tracy")]
    {
        let l = core::panic::Location::caller();
        Zone { z: Some(imp::client().span_alloc(Some(name), "", l.file(), l.line(), 0)) }
    }
    #[cfg(not(feature = "profile-tracy"))]
    {
        let _ = name;
        Zone {}
    }
}

/// Open a zone whose name is built at run time (a mod's name, a file's name). `f` only runs with the feature.
#[inline(always)]
#[track_caller]
pub fn zone_dyn(f: impl FnOnce() -> String) -> Zone {
    #[cfg(feature = "profile-tracy")]
    {
        let l = core::panic::Location::caller();
        let n = f();
        Zone { z: Some(imp::client().span_alloc(Some(&n), "", l.file(), l.line(), 0)) }
    }
    #[cfg(not(feature = "profile-tracy"))]
    {
        let _ = f;
        Zone {}
    }
}

impl Zone {
    /// Attach a number (bytes, a count, a frame index).
    #[inline(always)]
    pub fn value(&self, v: u64) {
        #[cfg(feature = "profile-tracy")]
        if let Some(z) = &self.z {
            z.emit_value(v);
        }
        #[cfg(not(feature = "profile-tracy"))]
        let _ = v;
    }
    /// Attach a text; `f` only runs with the feature.
    #[inline(always)]
    pub fn text(&self, f: impl FnOnce() -> String) {
        #[cfg(feature = "profile-tracy")]
        if let Some(z) = &self.z {
            z.emit_text(&f());
        }
        #[cfg(not(feature = "profile-tracy"))]
        let _ = f;
    }
}

/// The end of a presented frame (Tracy's frame bar).
#[inline(always)]
pub fn frame_mark() {
    #[cfg(feature = "profile-tracy")]
    imp::client().frame_mark();
}

/// A secondary frame set (e.g. "tick": one mark per 60 Hz sim tick).
#[inline(always)]
pub fn frame_mark_named(name: &'static str) {
    #[cfg(feature = "profile-tracy")]
    {
        let n = intern(name);
        imp::client().secondary_frame_mark(n);
    }
    #[cfg(not(feature = "profile-tracy"))]
    let _ = name;
}

/// A plot sample (ms, counts, CPU load).
#[inline(always)]
pub fn plot(name: &'static str, v: f64) {
    #[cfg(feature = "profile-tracy")]
    imp::client().plot(intern_plot(name), v);
    #[cfg(not(feature = "profile-tracy"))]
    let _ = (name, v);
}

/// A message on the timeline (scene changes, hitches); `f` only runs with the feature.
#[inline(always)]
pub fn message(f: impl FnOnce() -> String) {
    #[cfg(feature = "profile-tracy")]
    imp::client().message(&f(), 0);
    #[cfg(not(feature = "profile-tracy"))]
    let _ = f;
}

#[cfg(feature = "profile-tracy")]
fn intern(name: &'static str) -> tracy_client::FrameName {
    use std::collections::HashMap;
    // per thread (TLS is not in a rollback snapshot; a static table grown on the system heap was rewound)
    thread_local! {
        static NAMES: core::cell::RefCell<HashMap<&'static str, tracy_client::FrameName>> = core::cell::RefCell::new(HashMap::new());
    }
    NAMES.with(|n| *n.borrow_mut().entry(name).or_insert_with(|| tracy_client::FrameName::new_leak(name.to_string())))
}

#[cfg(feature = "profile-tracy")]
fn intern_plot(name: &'static str) -> tracy_client::PlotName {
    use std::collections::HashMap;
    // per thread (TLS is not in a rollback snapshot)
    thread_local! {
        static NAMES: core::cell::RefCell<HashMap<&'static str, tracy_client::PlotName>> = core::cell::RefCell::new(HashMap::new());
    }
    NAMES.with(|n| *n.borrow_mut().entry(name).or_insert_with(|| tracy_client::PlotName::new_leak(name.to_string())))
}

//! One link from the online select into the match, as Slippi Dolphin and Hero keep one netplay client for both.
//!
//! The select runs in the menu process and the match in the process it relaunches (the window's event loop and the
//! rollback setup are made once per process). Slippi Dolphin does not take a second connection after its first link
//! drops, so the match must not connect again: the menu process keeps polling the select's link after both are
//! locked in (`keep`), and when the match process asks (`take`, over loopback TCP with a one-time token) it stops,
//! hands over the whole ENet + Slippi client state (`Client::export`), and closes its socket. The match process binds
//! the same UDP port and goes on with that state, so the opponent sees one link that never dropped.
//!
//! A Direct set goes back the other way after each game (`hand_back`): the match process keeps the link for the menu
//! process it relaunches into the select, which takes it (`slippi_css_host::start_rematch`) for the next lock-in and
//! hands it on to the next match as above.

use melee_net::slippi::{Client, PacketR, Selections};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The menu process is keeping a link for the match process.
static KEEPING: AtomicBool = AtomicBool::new(false);

/// How long the menu process keeps the link for the match process to take it.
const KEEP_SECONDS: u64 = 120;

/// Keep `cl` alive (polled) on this thread (the session's, which owns the client) until the match process takes it.
/// `ready` gets the `--slippi-link` value for the match once the keeper listens; `to` names the taker in the log.
pub fn keep(mut cl: Client, mine: Selections, theirs: Selections, to: &str, ready: impl FnOnce(String)) -> Result<(), String> {
    // one process (the default, and the only way under YouGame, whose sandbox denies loopback): in memory, no socket
    if inprocess() {
        return keep_mem(cl, mine, theirs, to, ready);
    }
    let l = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("link keeper: {e}"))?;
    l.set_nonblocking(true).map_err(|e| format!("link keeper: {e}"))?;
    let port = l.local_addr().map_err(|e| format!("link keeper: {e}"))?.port();
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    let token = format!("{:016x}", (seed ^ ((std::process::id() as u64) << 32)).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let tok = token.clone();
    KEEPING.store(true, Ordering::SeqCst);
    ready(format!("{port}:{token}"));
    let run = move || {
        let t0 = Instant::now();
        loop {
            cl.poll();
            if let Ok((mut s, _)) = l.accept() {
                let _ = s.set_nonblocking(false);
                let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
                let mut got = [0u8; 16];
                if s.read_exact(&mut got).is_ok() && got == *tok.as_bytes() {
                    cl.poll();
                    cl.host.flush();
                    // the socket closes here, before the match process binds the port; nothing is sent to the opponent
                    let msg = handoff_msg(cl, mine, theirs);
                    let ok = s.write_all(&(msg.len() as u32).to_le_bytes()).and_then(|_| s.write_all(&msg)).is_ok();
                    eprintln!("online css: link: {} to the {to} after {:.2}s", if ok { "handed" } else { "LOST handing" }, t0.elapsed().as_secs_f32());
                    return;
                }
            }
            if t0.elapsed() > Duration::from_secs(KEEP_SECONDS) {
                eprintln!("online css: link: the {to} never took the link; closing it");
                cl.disconnect(0);
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    run();
    KEEPING.store(false, Ordering::SeqCst);
    Ok(())
}

/// The end of a Direct set's game, in the match process: the link (its game ended, `Client::end_game`) is kept for
/// the select the window relaunches into; the value for `MELEE_SLIPPI_REMATCH`'s link field. The client is not `Send`:
/// it is exported here and goes on in the keeper's thread over a socket on the same port, nothing sent meanwhile.
/// `finish` (the end of this process) waits until the select has it.
pub fn hand_back(cl: Client) -> Result<String, String> {
    let port = cl.host.local_port();
    let blob = cl.export();
    drop(cl);
    let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
    std::thread::spawn(move || {
        let t0 = Instant::now();
        let udp = loop {
            match melee_net::transport::UdpTransport::open(0, port) {
                Ok(u) => break u,
                Err(e) if t0.elapsed() > Duration::from_secs(5) => return drop(tx.send(Err(format!("bind: {e}")))),
                Err(_) => std::thread::sleep(Duration::from_millis(5)),
            }
        };
        let Some(cl) = Client::import(Box::new(udp), &blob) else { return drop(tx.send(Err("the link's state is malformed".into()))) };
        let done = tx.clone();
        if let Err(e) = keep(cl, Selections::default(), Selections::default(), "select", move |spec| drop(done.send(Ok(spec)))) {
            let _ = tx.send(Err(e));
        }
    });
    rx.recv_timeout(Duration::from_secs(10)).map_err(|_| "link keeper: no answer".to_string())?
}

/// At the end of the menu process (or of a set's match process): wait until the match process has the link (or the keeper gave up).
pub fn finish() {
    let t0 = Instant::now();
    while KEEPING.load(Ordering::SeqCst) && t0.elapsed() < Duration::from_secs(KEEP_SECONDS + 10) {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The link handed over: the client, this side's selections and the opponent's (both sent on the select).
pub struct Taken {
    pub cl: Client,
    pub mine: Selections,
    pub theirs: Selections,
}

/// In the match process (or the select of a set's next game): take the link `keep` holds (`spec` is the `--slippi-link` value), going on over a socket on
/// `port` (the same local port the select used; `wrap` adds the test hooks' lag and loss).
pub fn take(spec: &str, bind_ip: u32, port: u16, wrap: impl FnOnce(melee_net::transport::UdpTransport) -> Box<dyn melee_net::transport::Transport>) -> Result<Taken, String> {
    let (kp, token) = spec.split_once(':').ok_or("bad --slippi-link")?;
    if kp == MEM {
        let msg = take_mem(token)?;
        return taken_from(&msg, bind_ip, port, wrap);
    }
    let kp: u16 = kp.parse().map_err(|_| "bad --slippi-link")?;
    let mut s = TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], kp)), Duration::from_secs(5)).map_err(|e| format!("the select's link is gone ({e})"))?;
    s.set_read_timeout(Some(Duration::from_secs(10))).map_err(|e| e.to_string())?;
    s.write_all(token.as_bytes()).map_err(|e| format!("link: {e}"))?;
    let mut n = [0u8; 4];
    s.read_exact(&mut n).map_err(|e| format!("the select's link was not handed over ({e})"))?;
    let mut msg = vec![0u8; u32::from_le_bytes(n) as usize];
    s.read_exact(&mut msg).map_err(|e| format!("link: {e}"))?;
    taken_from(&msg, bind_ip, port, wrap)
}

/// The handed-over message: this side's selections, the opponent's, then the client's state (`Client::export`).
fn handoff_msg(cl: Client, mine: Selections, theirs: Selections) -> Vec<u8> {
    let blob = cl.export();
    drop(cl);
    let mut msg = Vec::with_capacity(blob.len() + 40);
    for s in [mine, theirs] {
        let e = s.encode();
        msg.push(e.len() as u8);
        msg.extend_from_slice(&e);
    }
    msg.extend_from_slice(&blob);
    msg
}

fn taken_from(msg: &[u8], bind_ip: u32, port: u16, wrap: impl FnOnce(melee_net::transport::UdpTransport) -> Box<dyn melee_net::transport::Transport>) -> Result<Taken, String> {
    let mut o = 0usize;
    let mut sel = || -> Result<Selections, String> {
        let len = *msg.get(o).ok_or("link: short")? as usize;
        let b = msg.get(o + 1..o + 1 + len).ok_or("link: short")?;
        o += 1 + len;
        let mut r = PacketR::new(b);
        r.o = 1;
        Selections::decode(&mut r).ok_or_else(|| "link: bad selections".to_string())
    };
    let (mine, theirs) = (sel()?, sel()?);
    // the keeper closed its socket just before it wrote: the port is free a moment later
    let t0 = Instant::now();
    let udp = loop {
        match melee_net::transport::UdpTransport::open(bind_ip, port) {
            Ok(u) => break u,
            Err(e) if t0.elapsed() > Duration::from_secs(5) => return Err(format!("bind: {e}")),
            Err(_) => std::thread::sleep(Duration::from_millis(5)),
        }
    };
    let cl = Client::import(wrap(udp), &msg[o..]).ok_or("link: the handed-over state is malformed")?;
    Ok(Taken { cl, mine, theirs })
}

// ------------------------------------------------------------------------------------------------ in memory

/// `--slippi-link mem:<token>`: the keeper is this process's own thread (`inprocess`); no socket.
const MEM: &str = "mem";
/// The taker's token (asks the keeper to hand over) and the keeper's answer.
static MEM_ASK: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
static MEM_SLOT: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

/// `keep` in one process: the same polling, the same message, handed through memory instead of loopback TCP (the
/// YouGame sandbox denies loopback, and nothing outside this process should be able to take the link).
fn keep_mem(mut cl: Client, mine: Selections, theirs: Selections, to: &str, ready: impl FnOnce(String)) -> Result<(), String> {
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    let token = format!("{:016x}", (seed ^ ((std::process::id() as u64) << 32)).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    *MEM_SLOT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    *MEM_ASK.lock().unwrap_or_else(|e| e.into_inner()) = None;
    KEEPING.store(true, Ordering::SeqCst);
    ready(format!("{MEM}:{token}"));
    let t0 = Instant::now();
    loop {
        cl.poll();
        let asked = MEM_ASK.lock().unwrap_or_else(|e| e.into_inner()).take();
        if asked.as_deref() == Some(token.as_str()) {
            cl.poll();
            cl.host.flush();
            let msg = handoff_msg(cl, mine, theirs);
            *MEM_SLOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(msg);
            eprintln!("online css: link: handed to the {to} (in memory) after {:.2}s", t0.elapsed().as_secs_f32());
            break;
        }
        if t0.elapsed() > Duration::from_secs(KEEP_SECONDS) {
            eprintln!("online css: link: the {to} never took the link; closing it");
            cl.disconnect(0);
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    KEEPING.store(false, Ordering::SeqCst);
    Ok(())
}

fn take_mem(token: &str) -> Result<Vec<u8>, String> {
    *MEM_ASK.lock().unwrap_or_else(|e| e.into_inner()) = Some(token.to_string());
    let t0 = Instant::now();
    loop {
        if let Some(m) = MEM_SLOT.lock().unwrap_or_else(|e| e.into_inner()).take() {
            return Ok(m);
        }
        if t0.elapsed() > Duration::from_secs(10) {
            MEM_ASK.lock().unwrap_or_else(|e| e.into_inner()).take();
            return Err("the select's link was not handed over (in memory)".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

// ------------------------------------------------------------------------------------------------ one process

/// The whole online loop (select -> match -> results -> select) in the menu process and its one window: the menu run
/// boots in the rollback run modes (`play_net::prepare_rollback_run`), so the match takes the select's link over
/// in-process (`take` against this process's own keeper) and the window never closes. `MELEE_SLIPPI_INPROCESS=0`
/// keeps the relaunch per match (the older handoff above).
static INPROCESS: AtomicBool = AtomicBool::new(false);

/// At the boot: whether this run plays its online matches in-process (`--slippi-menus on`, windowed, not a match or
/// replay run of its own).
pub fn wants_inprocess(args: &[String]) -> bool {
    let off = std::env::var("MELEE_SLIPPI_INPROCESS").is_ok_and(|v| v == "0");
    // the relaunch hands the link over loopback TCP, which the YouGame sandbox denies: refused there
    if off && melee_platform::yougame::sandboxed() {
        eprintln!("melee: MELEE_SLIPPI_INPROCESS=0 is ignored under YouGame (its relaunch handoff needs loopback, which the sandbox denies)");
        return crate::slippi_menus::requested(args) == Ok(true);
    }
    !off && crate::slippi_menus::requested(args) == Ok(true)
}

pub fn set_inprocess(on: bool) {
    INPROCESS.store(on, Ordering::SeqCst);
}

pub fn inprocess() -> bool {
    INPROCESS.load(Ordering::SeqCst)
}

/// The codes the online match turns on (`slippi_np::online_codes`), as they were before it.
static SAVED: std::sync::Mutex<Option<(u32, u32, bool, bool)>> = std::sync::Mutex::new(None);

pub fn save_offline_codes() {
    if let Ok(mut g) = SAVED.lock() {
        *g = Some(melee_core::codes::save_offline());
    }
}

/// The codes back, and the Slippi code hooks back to their offline values.
pub fn restore_offline_codes() {
    let Some(s) = SAVED.lock().ok().and_then(|mut g| g.take()) else { return };
    melee_core::codes::restore_offline(s);
    unsafe {
        melee_fighter::platform::hook::Slippi_Codes = None;
        // the fighter TUs' and Final Destination's tables: 0 offline (manual.rs)
        melee_core::codes::set_slippi_codeset(0);
        melee_fighter::hooks::SLIPPI_CODES = Some(melee_core::codes::slippi_codeset);
        melee_world::tu::grlast::ext::SLIPPI_CODES = Some(|| 0);
    }
}

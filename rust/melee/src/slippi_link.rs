//! One link from the online select into the match, as Slippi Dolphin and Hero keep one netplay client for both.
//!
//! The select runs in the menu process and the match in the process it relaunches (the window's event loop and the
//! rollback setup are made once per process). Slippi Dolphin does not take a second connection after its first link
//! drops, so the match must not connect again: the menu process keeps polling the select's link after both are
//! locked in (`keep`), and when the match process asks (`take`, over loopback TCP with a one-time token) it stops,
//! hands over the whole ENet + Slippi client state (`Client::export`), and closes its socket. The match process binds
//! the same UDP port and goes on with that state, so the opponent sees one link that never dropped.

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
/// `ready` gets the `--slippi-link` value for the match once the keeper listens.
pub fn keep(mut cl: Client, mine: Selections, theirs: Selections, ready: impl FnOnce(String)) -> Result<(), String> {
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
                    let blob = cl.export();
                    // the socket closes here, before the match process binds the port; nothing is sent to the opponent
                    drop(cl);
                    let mut msg = Vec::with_capacity(blob.len() + 40);
                    for s in [mine, theirs] {
                        let e = s.encode();
                        msg.push(e.len() as u8);
                        msg.extend_from_slice(&e);
                    }
                    msg.extend_from_slice(&blob);
                    let ok = s.write_all(&(msg.len() as u32).to_le_bytes()).and_then(|_| s.write_all(&msg)).is_ok();
                    eprintln!("online css: link: {} to the match process after {:.2}s", if ok { "handed" } else { "LOST handing" }, t0.elapsed().as_secs_f32());
                    return;
                }
            }
            if t0.elapsed() > Duration::from_secs(KEEP_SECONDS) {
                eprintln!("online css: link: the match process never took the link; closing it");
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

/// At the end of the menu process: wait until the match process has the link (or the keeper gave up).
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

/// In the match process: take the link `keep` holds (`spec` is the `--slippi-link` value), going on over a socket on
/// `port` (the same local port the select used; `wrap` adds the test hooks' lag and loss).
pub fn take(spec: &str, bind_ip: u32, port: u16, wrap: impl FnOnce(melee_net::transport::UdpTransport) -> Box<dyn melee_net::transport::Transport>) -> Result<Taken, String> {
    let (kp, token) = spec.split_once(':').ok_or("bad --slippi-link")?;
    let kp: u16 = kp.parse().map_err(|_| "bad --slippi-link")?;
    let mut s = TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], kp)), Duration::from_secs(5)).map_err(|e| format!("the select's link is gone ({e})"))?;
    s.set_read_timeout(Some(Duration::from_secs(10))).map_err(|e| e.to_string())?;
    s.write_all(token.as_bytes()).map_err(|e| format!("link: {e}"))?;
    let mut n = [0u8; 4];
    s.read_exact(&mut n).map_err(|e| format!("the select's link was not handed over ({e})"))?;
    let mut msg = vec![0u8; u32::from_le_bytes(n) as usize];
    s.read_exact(&mut msg).map_err(|e| format!("link: {e}"))?;
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

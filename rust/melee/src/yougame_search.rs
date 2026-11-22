//! YouGame's side of an online search in the game, and Hero's blend of it with Slippi's (his `find_match` with a
//! `claim`, driven by `yougame::blend::Coordinator`): the ticket from the app (`mpTicket`), YouGame's queue, the match
//! room (`melee_net::yougame_room`) and the opponent's address, handed to the online select as a Slippi link
//! ([`crate::slippi_np::Paired`]), the same as a Slippi match.
//!
//! Unranked with a Slippi login is the blend: both pools search, the first opponent wins and the other search is
//! cancelled (a YouGame room that comes too late is declined: opened and left). Without a login it is YouGame's queue
//! alone. Agent and scripted runs reach only loopback servers (`Target::Loopback`): a local fake rooms service, and
//! for the Slippi side the local mock (`slippi_np::pair` refuses any other without `--mm-real`, never passed here).

use crate::slippi_np::Paired;
use melee_net::transport::Addr;
use melee_net::yougame_room as yr;
use melee_platform::yougame_mm as ym;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// What a YouGame-side search found: the link and the room it keeps (begin / finish go through it).
pub struct Found {
    pub paired: Paired,
    pub side: ym::Side,
    pub room: Option<(yr::Joined, yr::WsOpener)>,
    /// The punch failed and both seats agreed: the match's datagrams go through the room (the keeper drives it).
    pub relay: Option<melee_net::yougame_relay::Relay>,
}

pub fn compat() -> &'static str {
    concat!("gd-melee-rust/", env!("CARGO_PKG_VERSION"))
}

fn log(s: &str) {
    eprintln!("{s}");
}

/// The rooms URL is reachable here: always in a normal run; only loopback in an agent or scripted one.
fn target(url: &str, agent: bool) -> Result<melee_net::slippi_mm::Target, String> {
    use melee_net::slippi_mm::Target;
    let loopback = url.strip_prefix("ws://").and_then(|r| r.split([':', '/']).next()).and_then(|h| h.parse::<std::net::Ipv4Addr>().ok()).is_some_and(|ip| ip.is_loopback());
    if loopback {
        Ok(Target::Loopback)
    } else if agent {
        Err("an agent run reaches only a local rooms service".into())
    } else {
        Ok(Target::Real)
    }
}

/// YouGame's half: ticket -> queue -> room -> the opponent's candidate. `claim` is the blend's say before a room is
/// taken; `waiting` hears the queue's count.
pub fn yougame_side(
    bridge: &mut dyn ym::Bridge,
    mode: i32,
    port: u16,
    code: &str,
    agent: bool,
    cancel: &AtomicBool,
    claim: &mut dyn FnMut() -> bool,
    waiting: &mut dyn FnMut(i32),
) -> Result<Found, String> {
    let t = ym::request_ticket(bridge, mode, compat(), None)?;
    if cancel.load(Ordering::SeqCst) {
        return Err("Cancelled".into());
    }
    let tgt = target(&t.url, agent)?;
    let loopback = matches!(tgt, melee_net::slippi_mm::Target::Loopback);
    let mut opener = yr::WsOpener { url: t.url.clone(), target: tgt };
    let mut ticket = t.ticket.clone();
    let cancelled = || cancel.load(Ordering::SeqCst);
    let mut renew = || ym::request_ticket(bridge, mode, compat(), None).ok().map(|t| t.ticket);
    let mut say = |s: &str| log(s);
    let mut h = yr::Hooks { cancelled: &cancelled, claim, searching: waiting, renew: &mut renew, now: &yr::now_ms, log: &mut say };
    let (room, pass) = match t.room.clone() {
        Some(r) => {
            if !(h.claim)() {
                yr::decline(&mut opener, &r, &ticket);
                return Err("Cancelled".into());
            }
            (r, ticket.clone())
        }
        None => match yr::queue(&mut opener, &mut ticket, &mut h) {
            yr::QueueEnd::Matched { room, ticket: fresh } => (room, fresh.unwrap_or(ticket.clone())),
            yr::QueueEnd::Cancelled { declined } => {
                if let Some(r) = declined {
                    log(&format!("yougame: declined room {r}"));
                }
                return Err("Cancelled".into());
            }
            yr::QueueEnd::Failed(e) => return Err(e),
        },
    };
    log(&format!("yougame: matched into room {room}"));
    // where the opponent reaches this seat's netplay port (Hero's own-UDP path, melee_net::yougame_punch): `lan`
    // (loopback for a local service, else this machine's address towards the rooms server) and `ext`, the netplay
    // port's address as STUN sees it; then a punch from that port to both of the opponent's, the first that
    // answers (and acks ours) is the match's peer.
    let ip = if loopback {
        std::net::Ipv4Addr::LOCALHOST
    } else {
        use std::net::ToSocketAddrs;
        let host = t.url.split("://").nth(1).and_then(|r| r.split('/').next()).unwrap_or("");
        let host = if host.contains(':') { host.to_string() } else { format!("{host}:443") };
        host.to_socket_addrs().ok().and_then(|mut a| a.next()).and_then(melee_net::slippi_mm::local_ip_towards).unwrap_or(std::net::Ipv4Addr::UNSPECIFIED)
    };
    let udp = std::net::UdpSocket::bind((if loopback { std::net::Ipv4Addr::LOCALHOST } else { std::net::Ipv4Addr::UNSPECIFIED }, port)).map_err(|e| format!("netplay port {port}: {e}"))?;
    let port = udp.local_addr().map(|a| a.port()).unwrap_or(port);
    let stun_list = stun_servers(agent);
    let ext = melee_net::yougame_punch::stun(&udp, &stun_list, &cancelled);
    log(&format!("yougame: STUN {}", if ext.is_some() { "answered" } else if stun_list.is_empty() { "not asked" } else { "did not answer" }));
    // test hook (agent runs only): publish a dead `lan` so only `ext` can work, as behind a NAT
    // MELEE_YOUGAME_TEST_DEAD_DIRECT=1 (agent runs): both published addresses dead, so only the room relay works
    let dead_direct = std::env::var("MELEE_YOUGAME_TEST_DEAD_DIRECT").is_ok_and(|v| v == "1") && agent;
    let dead_lan = dead_direct || std::env::var("MELEE_YOUGAME_TEST_DEAD_LAN").is_ok_and(|v| v == "1") && agent;
    let lan = if dead_lan { "127.0.0.1:9".to_string() } else { format!("{ip}:{port}") };
    let ext_pub = if dead_direct { "127.0.0.2:9".to_string() } else { ext.map(|a| a.to_string()).unwrap_or_default() };
    let token = yr::now_ms() ^ ((std::process::id() as u64) << 32) | 1;
    let me = yr::Candidate { compat: compat().into(), code: code.into(), lan, ext: ext_pub, token: format!("{token:x}") };
    let mut j = yr::join(&mut opener, &room, pass, &me, &mut h, 15_000)?;
    log(&format!("yougame: room {}: seat {}, opponent seat {}, host {}, revision {}", j.code, j.seat, j.opp_seat, j.room.is_host(), j.room.revision));
    let mut cands = Vec::new();
    if let Some(a) = j.opp_lan {
        cands.push((a, "lan"));
    }
    if let Some(a) = j.opp_ext.filter(|e| Some(*e) != j.opp_lan) {
        cands.push((a, "ext"));
    }
    let theirs = u64::from_str_radix(&j.opp_token, 16).unwrap_or(0);
    let mut peers = [melee_net::yougame_punch::Peer::new(j.opp_seat + 1, token, theirs, cands)];
    let mut say2 = |s: &str| log(s);
    let mut relay = None;
    if !melee_net::yougame_punch::punch(&udp, j.seat + 1, &mut peers, Duration::from_secs(8), &cancelled, &mut say2) {
        // Hero's own_udp_relay: every seat says so and waits for the others; only then the room relay
        log(&format!("yougame: connect failed (own UDP): my STUN {}", if ext.is_some() { "ok" } else { "none" }));
        if cancelled() || !relay_agreed(&mut j, &cancelled) {
            j.socket.send(r#"{"t":"leave"}"#);
            return Err(if cancelled() { "Cancelled".into() } else { "Could not reach the opponent".into() });
        }
        let r = melee_net::yougame_relay::Relay::new(j.seat + 1, &[j.opp_seat + 1], port).map_err(|e| format!("room relay: {e}"))?;
        log(&format!("yougame: own UDP failed: every seat agreed, this match uses the room relay (seat {})", j.seat + 1));
        peers[0].chosen = r.stand_in(j.opp_seat + 1);
        peers[0].via = "relay";
        relay = Some(r);
    }
    drop(udp); // the match binds the same port again: the NAT mapping the punch opened
    let p = peers[0].chosen.unwrap_or(j.peer);
    log(&format!("yougame: the opponent answered via {}", peers[0].via));
    let paired = Paired {
        decider: j.seat == 0,
        local_port: j.seat,
        remote_port: j.opp_seat,
        peer: Addr::new(u32::from(*p.ip()), p.port()),
        fallback: None,
        bind_ip: if loopback { 0x7F00_0001 } else { 0 },
        opp_code: j.opp_code.clone(),
    };
    Ok(Found { paired, side: ym::Side::YouGame, room: Some((j, opener)), relay })
}

/// Hero's relay agreement: send `mu-relay-ok` and wait up to 5 s for the opponent's (its leave ends it).
fn relay_agreed(j: &mut yr::Joined, cancelled: &dyn Fn() -> bool) -> bool {
    use melee_net::yougame_relay as rl;
    j.socket.send(&rl::agree_msg(compat()));
    let until = yr::now_ms() + rl::AGREE_MS;
    while yr::now_ms() < until && !cancelled() {
        match j.socket.next(Duration::from_millis(100)) {
            yr::Next::Closed(_) => return false,
            yr::Next::Idle => {}
            yr::Next::Msg(t) => {
                let Some(m) = melee_net::slippi_mm::Json::parse(&t) else { continue };
                let leave = m.get("t").and_then(|v| v.str()) == Some("leave");
                match j.room.receive(&m, yr::now_ms()) {
                    yr::Event::Message { from, d } if rl::is_agree(&d, compat()) && from != j.room.me => {
                        log("yougame: the opponent agreed to the room relay");
                        return true;
                    }
                    yr::Event::Ended(_) => return false,
                    _ => {}
                }
                if leave && j.room.seated().count() < 2 {
                    return false;
                }
            }
        }
    }
    log("yougame: room relay not agreed (0 of 1 seats)");
    false
}

/// STUN servers: MELEE_YOUGAME_STUN (host:port list; in an agent run loopback ones only), else Hero's in a real run,
/// none in an agent run.
fn stun_servers(agent: bool) -> Vec<std::net::SocketAddr> {
    match std::env::var("MELEE_YOUGAME_STUN").ok().filter(|v| !v.is_empty()) {
        Some(v) => v.split(',').filter_map(|a| a.trim().parse::<std::net::SocketAddr>().ok()).filter(|a| !agent || a.ip().is_loopback()).collect(),
        None if agent => Vec::new(),
        None => melee_net::yougame_punch::resolve(&melee_net::yougame_punch::STUN_SERVERS),
    }
}

/// Pair for the online select by `route`: YouGame's queue alone, Slippi's (`slippi_np::pair` with `sub`), or the
/// blend. `cancel` is the select's cancel (polled); `status` tells the screen which pools search.
pub fn pair(route: ym::Route, mode: i32, sub: &[String], max_s: u64, agent: bool, cancel: &dyn Fn() -> bool, status: &dyn Fn(&str)) -> Result<Found, String> {
    let port: u16 = crate::slippi_np::arg_value(sub, "--port").and_then(|v| v.parse().ok()).unwrap_or(0);
    let code = crate::slippi_np::arg_value(sub, "--code").unwrap_or_default();
    let stop = Arc::new(AtomicBool::new(false));
    let coord = Arc::new(Mutex::new(ym::Coordinator::new(yr::now_ms())));
    let (tx, rx) = mpsc::channel::<(ym::Side, Result<Found, String>)>();
    let blend = route == ym::Route::Blend;
    let flush = |c: &mut ym::Coordinator| c.take_log().iter().for_each(|l| log(l));
    if route == ym::Route::Slippi {
        coord.lock().unwrap().failed(ym::Side::YouGame, yr::now_ms());
    }
    // YouGame's side, on its own thread (the ticket alone can take 30 s)
    if matches!(route, ym::Route::YouGame | ym::Route::Blend) {
        status(if blend { ym::SEARCH_BOTH } else { ym::SEARCH_YOUGAME_ONLY });
        let (tx, stop, coord, code) = (tx.clone(), stop.clone(), coord.clone(), code.clone());
        std::thread::spawn(move || {
            let mut claim = || !blend || coord.lock().unwrap().matched(ym::Side::YouGame, yr::now_ms());
            let mut waiting = |w: i32| {
                if blend {
                    coord.lock().unwrap().searching(w, yr::now_ms());
                }
            };
            let r = yougame_side(&mut ym::StdioBridge, mode, port, &code, agent, &stop, &mut claim, &mut waiting);
            let _ = tx.send((ym::Side::YouGame, r));
        });
    }
    let mut slippi_started = false;
    let start_slippi = |stop: &Arc<AtomicBool>| {
        let (tx, sub, stop) = (tx.clone(), sub.to_vec(), stop.clone());
        std::thread::spawn(move || {
            let r = crate::slippi_np::pair_cancellable(&sub, max_s, &stop).map_err(|(_, m)| m).map(|paired| Found { paired, side: ym::Side::Slippi, room: None, relay: None });
            let _ = tx.send((ym::Side::Slippi, r));
        });
    };
    if route == ym::Route::Slippi {
        coord.lock().unwrap().stop(); // the coordinator is not used: Slippi alone
        start_slippi(&stop);
        slippi_started = true;
    }
    let t0 = std::time::Instant::now();
    let mut pending = 1;
    let mut last_err = String::from("no opponent found");
    loop {
        if cancel() {
            stop.store(true, Ordering::SeqCst);
            coord.lock().unwrap().stop();
            log("online search: cancelled");
            return Err("Cancelled".into());
        }
        if blend {
            let mut c = coord.lock().unwrap();
            c.tick(yr::now_ms());
            if c.take_start_slippi() && !slippi_started {
                slippi_started = true;
                pending += 1;
                start_slippi(&stop);
            }
            flush(&mut c);
        }
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok((side, Ok(found))) => {
                // YouGame's claim already committed it; a Slippi match commits here (or is late and dropped)
                let won = match (side, blend) {
                    (ym::Side::Slippi, true) => coord.lock().unwrap().matched(ym::Side::Slippi, yr::now_ms()),
                    _ => true,
                };
                flush(&mut coord.lock().unwrap());
                if won {
                    stop.store(true, Ordering::SeqCst);
                    log(&format!("online search: matched on {}", side.name()));
                    return Ok(found);
                }
                pending -= 1;
            }
            Ok((side, Err(e))) => {
                pending -= 1;
                if e != "Cancelled" {
                    log(&format!("online search: {} side ended: {e}", side.name()));
                    last_err = e;
                }
                if blend {
                    let mut c = coord.lock().unwrap();
                    c.failed(side, yr::now_ms());
                    if c.take_start_slippi() && !slippi_started {
                        slippi_started = true;
                        pending += 1;
                        start_slippi(&stop);
                    }
                    flush(&mut c);
                }
                if pending <= 0 {
                    return Err(last_err);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err(last_err),
        }
        if t0.elapsed().as_secs() > max_s {
            stop.store(true, Ordering::SeqCst);
            return Err("timed out".into());
        }
    }
}

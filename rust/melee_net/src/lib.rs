//! Rollback netplay transport: a native port of `pc/platform/gw_net.c` (+ `gw_net.h`), wire
//! compatible with the C build (protocol version 9), plus the rollback session interface
//! (`gw_rollback.h`'s network-facing half), the 11-byte raw-pad payload codec of `gw_netplay.c`,
//! and a generic rollback frame/snapshot driver.
//!
//! Layout:
//! - [`wire`]: the little-endian field encoders; packet constants.
//! - [`hash`]: FNV-1a 64 identity hashes (`gw_net_hash64`, `gw_net_hash_file`).
//! - [`transport`]: the pluggable socket ([`Transport`]), real UDP ([`UdpTransport`]), address parsing.
//! - [`net`]: the session object ([`Net`]): handshake, input exchange, time sync, checksums, lobby.
//! - [`sim`]: the deterministic simulated network of `gw_net_tests.c` (loss, dup, jitter, clocks).
//! - [`pad`]: `np_encode` / `np_decode` and [`RbInput`] (GwRbInput).
//! - [`rollback`]: [`RollbackSession`] (what the transport talks to), [`NetAdapter`] (gw_netplay's
//!   glue), and [`RollbackDriver`] over a [`Simulation`] with save/load snapshots.
//!
//! Like the C, nothing here touches game memory and everything runs from one `poll` per logic
//! iteration on the caller's thread: no threads, no blocking.

pub mod app;
pub mod early;
pub mod enet;
pub mod hash;
pub mod net;
pub mod pad;
pub mod rollback;
pub mod session;
pub mod sim;
pub mod slippi;
pub mod slippi_mm;
pub mod transport;
pub mod wire;

pub use hash::{hash64, hash_file};
pub use net::{Callbacks, Config, Event, Net, RemoteConfig, State, Stats};
pub use pad::{np_decode, np_encode, RbInput, NP_PAYLOAD};
pub use rollback::{NetAdapter, RollbackDriver, RollbackSession, Simulation};
pub use transport::{addr_parse, addr_resolve, Addr, Transport, UdpTransport};

/// `GW_NET_PROTOCOL_VERSION` (gw_net.h; see its history comment for what each bump means).
pub const PROTOCOL_VERSION: u32 = 9;
pub const MAX_BLOB: usize = 900;
pub const MAX_SLOTS: usize = 8;
pub const MAX_PAYLOAD: usize = 32;
pub const DEFAULT_PAYLOAD: usize = 9;
pub const RING: u32 = 256;
pub const MAX_INPUTS_PER_PACKET: i32 = 40;
pub const NO_FRAME: i32 = i32::MIN;
pub const MAX_GUEST_INFO: usize = 64;
pub const HOLD_TIMEOUT_MS: u32 = 90000;
pub const LOBBY_MAX: usize = 200;

/// The Envoy mode words of gw_matchbuild.h (protocol 5), and its `gw_envoy_mode_valid`.
pub const ENVOY_MODE_OFF: u32 = 0;
pub const ENVOY_MODE_V1: u32 = 0x4556_0001;
pub const ENVOY_MODE_COOP: u32 = 0x4556_0002;
pub const ENVOY_MODE_RUN: u32 = 0x4556_0003;
pub fn envoy_mode_valid(w: u32) -> bool {
    matches!(w, ENVOY_MODE_OFF | ENVOY_MODE_V1 | ENVOY_MODE_COOP | ENVOY_MODE_RUN)
}

/// One slot's input for one frame (`gw_net_input`): the first `payload_bytes` are meaningful.
pub type Input = [u8; MAX_PAYLOAD];

/// A refusal reason as the guest shows it: the host's "different global game data; host has: X" names only the
/// host's set, so the guest appends its own ("; you have: Y"). The wire text stays the C's (net_oracle); this is
/// display only. Other reasons are returned unchanged.
pub fn refusal_both_sides(why: &str, own_desc: Option<&str>) -> String {
    if !why.starts_with("different global game data") || why.contains("; you have: ") {
        return why.to_string();
    }
    let mine = match own_desc {
        Some(s) if !s.is_empty() => s,
        _ => "none",
    };
    let mine = if mine.len() > 200 {
        let mut n = 197;
        while !mine.is_char_boundary(n) { n -= 1; }
        format!("{}...", &mine[..n])
    } else {
        mine.to_string()
    };
    let host = why.split_once("; host has: ").map(|x| x.1).unwrap_or("");
    match mods_diff(host, own_desc.unwrap_or("")) {
        Some(d) => format!("different global game data: {d}; host has: {host}; you have: {mine}"),
        None => format!("{why}; you have: {mine}"),
    }
}

/// The "name#hash" tokens of a handshake description ("mods a#1234abcd, b#..; code mods w#..; geno ..").
fn desc_tokens(d: &str) -> Vec<(&str, &str)> {
    d.split([',', ';', ' ', '(', ')'])
        .filter_map(|t| t.split_once('#'))
        .filter(|(n, h)| !n.is_empty() && !h.is_empty() && h.bytes().all(|c| c.is_ascii_hexdigit()))
        .collect()
}

/// Which mods differ between the host's and this peer's description: "mod X differs (host 1234abcd, you
/// 9876fedc); only the host has Y; only you have Z". None when neither side lists a "name#hash" token or no named
/// token differs (then only the full lists can tell, e.g. gameplay codes or a cut-off list).
pub fn mods_diff(host: &str, mine: &str) -> Option<String> {
    let cut = host.ends_with("...");
    let host = host.trim_end_matches("...");
    let (mut h, m) = (desc_tokens(host), desc_tokens(mine));
    if cut {
        h.pop(); // the last token may be cut mid-hash
    }
    if h.is_empty() && m.is_empty() {
        return None;
    }
    fn find<'a>(v: &[(&'a str, &'a str)], n: &str) -> Option<&'a str> {
        v.iter().find(|x| x.0.eq_ignore_ascii_case(n)).map(|x| x.1)
    }
    let mut out = Vec::new();
    for (n, hh) in &h {
        if let Some(mh) = find(&m, n) {
            if !mh.eq_ignore_ascii_case(hh) {
                out.push(format!("mod {n} differs (host {hh}, you {mh})"));
            }
        }
    }
    let only_h: Vec<&str> = h.iter().filter(|x| find(&m, x.0).is_none()).map(|x| x.0).collect();
    // a cut-off host list cannot prove the host lacks one of ours
    let only_m: Vec<&str> = if cut { vec![] } else { m.iter().filter(|x| find(&h, x.0).is_none()).map(|x| x.0).collect() };
    if !only_h.is_empty() {
        out.push(format!("only the host has {}", only_h.join(", ")));
    }
    if !only_m.is_empty() {
        out.push(format!("only you have {}", only_m.join(", ")));
    }
    (!out.is_empty()).then(|| out.join("; "))
}

#[cfg(test)]
mod mods_diff_tests {
    use super::*;

    #[test]
    fn names_the_mod_that_differs() {
        let why = "different global game data; host has: mods a#11111111, b#22222222 (abc); code mods w#33333333";
        let r = refusal_both_sides(why, Some("mods a#11111111, b#2222aaaa, c#44444444"));
        assert!(r.starts_with("different global game data: mod b differs (host 22222222, you 2222aaaa); only the host has w; only you have c; host has: "), "{r}");
        assert!(r.ends_with("; you have: mods a#11111111, b#2222aaaa, c#44444444"));
        assert_eq!(mods_diff("no mods", "no mods"), None);
        assert_eq!(mods_diff("none", "mods a#11111111").as_deref(), Some("only you have a"));
        assert_eq!(refusal_both_sides("session full", Some("x")), "session full");
    }
}

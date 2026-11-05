//! One blocking HTTPS GET for the Nucleus browser (port of pc/platform/gw_nucleus_http.inc). Called only from the
//! nucleus worker threads, never the game thread, never during a netplay session (the engine refuses then).
//!
//! Windows: WinHTTP (the system proxy and certificate store apply); redirects are followed and the FINAL address is
//! checked against the allow-list. Elsewhere: the system `curl` program (the game links no TLS library); when it is
//! missing the call fails with a message and the browser says it is offline.
//!
//! Only ssbmnucleus.net and its subdomains are contacted, over https. `MELEE_NUCLEUS_API` may name a loopback fixture
//! server (127.0.0.1 / localhost, plain http allowed) for tests. Every request carries the User-Agent the API's owner
//! asked for, naming this client: `GDMelee/<version> (GD's Melee Rust port; +https://github.com/GurekamDhillon/melee)`.
//! Timeouts (resolve 10 s, connect 10 s, send 20 s, receive 30 s) are the C client's: offline, a request fails in
//! seconds and the worker backs off; the game thread never waits on any of it.

use std::sync::OnceLock;

pub struct Resp {
    /// 0: no answer (network, refused address); else the HTTP status.
    pub status: u32,
    pub body: Vec<u8>,
    pub retry_after: u32,
    pub err: String,
}

impl Resp {
    fn fail(e: impl Into<String>) -> Resp {
        Resp { status: 0, body: Vec::new(), retry_after: 0, err: e.into() }
    }
}

pub fn user_agent() -> &'static str {
    static UA: OnceLock<String> = OnceLock::new();
    UA.get_or_init(|| {
        let v = std::env::var("MELEE_VERSION").ok().filter(|v| !v.is_empty() && v.len() < 24).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
        format!("GDMelee/{v} (GD's Melee Rust port; +https://github.com/GurekamDhillon/melee)")
    })
}

static OVERRIDE_HOST: OnceLock<String> = OnceLock::new();

/// `MELEE_NUCLEUS_API`: a loopback base the allow-list then accepts (anything else is ignored).
pub fn set_override(base: &str) {
    if let Some((host, _)) = split(base) {
        if host == "127.0.0.1" || host == "localhost" {
            let _ = OVERRIDE_HOST.set(host);
        }
    }
}

/// "https://host[:port]/path" -> (host, secure)
pub fn split(url: &str) -> Option<(String, bool)> {
    let (rest, secure) = if let Some(r) = url.strip_prefix("https://") { (r, true) } else { (url.strip_prefix("http://")?, false) };
    let n = rest.find(['/', ':', '?', '#']).unwrap_or(rest.len());
    (n > 0 && n < 96).then(|| (rest[..n].to_string(), secure))
}

pub fn host_ok(host: &str, secure: bool) -> bool {
    let s = "ssbmnucleus.net";
    if secure && (host == s || (host.len() > s.len() && host.ends_with(s) && host.as_bytes()[host.len() - s.len() - 1] == b'.')) {
        return true;
    }
    OVERRIDE_HOST.get().is_some_and(|o| o == host)
}

/// GET `url`, at most `max` bytes of body. `progress(got, expected or 0)` after every read; true abandons it.
pub fn get(url: &str, max: usize, progress: &mut dyn FnMut(usize, usize) -> bool) -> Resp {
    match split(url) {
        Some((h, s)) if host_ok(&h, s) => imp::get(url, max, progress),
        _ => Resp::fail("address not allowed"),
    }
}

#[cfg(windows)]
mod imp {
    use super::Resp;
    use std::ffi::c_void;
    type H = *mut c_void;
    #[link(name = "winhttp")]
    extern "system" {
        fn WinHttpOpen(ua: *const u16, access: u32, proxy: *const u16, bypass: *const u16, flags: u32) -> H;
        fn WinHttpConnect(s: H, host: *const u16, port: u16, r: u32) -> H;
        fn WinHttpOpenRequest(c: H, verb: *const u16, path: *const u16, ver: *const u16, referrer: *const u16, accept: *const *const u16, flags: u32) -> H;
        fn WinHttpSendRequest(r: H, hdr: *const u16, hlen: u32, opt: *const c_void, olen: u32, total: u32, ctx: usize) -> i32;
        fn WinHttpReceiveResponse(r: H, res: *mut c_void) -> i32;
        fn WinHttpQueryHeaders(r: H, level: u32, name: *const u16, buf: *mut c_void, len: *mut u32, index: *mut u32) -> i32;
        fn WinHttpQueryOption(h: H, opt: u32, buf: *mut c_void, len: *mut u32) -> i32;
        fn WinHttpSetOption(h: H, opt: u32, buf: *const c_void, len: u32) -> i32;
        fn WinHttpSetTimeouts(h: H, resolve: i32, connect: i32, send: i32, recv: i32) -> i32;
        fn WinHttpReadData(r: H, buf: *mut c_void, n: u32, read: *mut u32) -> i32;
        fn WinHttpCloseHandle(h: H) -> i32;
    }
    extern "system" {
        fn GetLastError() -> u32;
    }
    const QUERY_STATUS_CODE: u32 = 19;
    const QUERY_CONTENT_LENGTH: u32 = 5;
    const QUERY_RETRY_AFTER: u32 = 36;
    const QUERY_FLAG_NUMBER: u32 = 0x2000_0000;
    const OPTION_URL: u32 = 34;
    const OPTION_DECOMPRESSION: u32 = 118;
    const FLAG_SECURE: u32 = 0x0080_0000;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct Handles([H; 3]);
    impl Drop for Handles {
        fn drop(&mut self) {
            for h in self.0.iter().rev() {
                if !h.is_null() {
                    unsafe { WinHttpCloseHandle(*h) };
                }
            }
        }
    }

    fn num(req: H, q: u32) -> Option<u32> {
        let mut v = 0u32;
        let mut sz = 4u32;
        (unsafe { WinHttpQueryHeaders(req, q | QUERY_FLAG_NUMBER, std::ptr::null(), &mut v as *mut u32 as *mut c_void, &mut sz, std::ptr::null_mut()) } != 0).then_some(v)
    }

    pub fn get(url: &str, max: usize, progress: &mut dyn FnMut(usize, usize) -> bool) -> Resp {
        let secure = url.starts_with("https://");
        let rest = &url[if secure { 8 } else { 7 }..];
        let hend = rest.find(['/', '?']).unwrap_or(rest.len());
        let (hostport, path) = (&rest[..hend], if hend < rest.len() { &rest[hend..] } else { "/" });
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) => match p.parse::<u16>() {
                Ok(p) => (h, p),
                Err(_) => return Resp::fail("bad address"),
            },
            None => (hostport, if secure { 443 } else { 80 }),
        };
        let mut hs = Handles([std::ptr::null_mut(); 3]);
        unsafe {
            let ua = w(super::user_agent());
            hs.0[0] = WinHttpOpen(ua.as_ptr(), 4 /* automatic proxy */, std::ptr::null(), std::ptr::null(), 0);
            if hs.0[0].is_null() {
                hs.0[0] = WinHttpOpen(ua.as_ptr(), 0 /* default proxy */, std::ptr::null(), std::ptr::null(), 0);
            }
            if hs.0[0].is_null() {
                return Resp::fail(format!("cannot start WinHTTP ({})", GetLastError()));
            }
            WinHttpSetTimeouts(hs.0[0], 10000, 10000, 20000, 30000);
            let dec: u32 = 3; // WINHTTP_DECOMPRESSION_FLAG_ALL
            WinHttpSetOption(hs.0[0], OPTION_DECOMPRESSION, &dec as *const u32 as *const c_void, 4);
            hs.0[1] = WinHttpConnect(hs.0[0], w(host).as_ptr(), port, 0);
            if !hs.0[1].is_null() {
                hs.0[2] = WinHttpOpenRequest(hs.0[1], w("GET").as_ptr(), w(path).as_ptr(), std::ptr::null(), std::ptr::null(), std::ptr::null(), if secure { FLAG_SECURE } else { 0 });
            }
            let req = hs.0[2];
            if req.is_null() {
                return Resp::fail(format!("cannot open the request ({})", GetLastError()));
            }
            if WinHttpSendRequest(req, std::ptr::null(), 0, std::ptr::null(), 0, 0, 0) == 0 || WinHttpReceiveResponse(req, std::ptr::null_mut()) == 0 {
                return Resp::fail(format!("no answer ({})", GetLastError()));
            }
            // the final address after redirects must still be one we allow
            let mut fin = [0u16; 1200];
            let mut fl = (fin.len() * 2) as u32;
            if WinHttpQueryOption(req, OPTION_URL, fin.as_mut_ptr() as *mut c_void, &mut fl) != 0 {
                let f = String::from_utf16_lossy(&fin[..(fl as usize / 2).min(fin.len())]);
                let f = f.trim_end_matches('\0');
                if !super::split(f).is_some_and(|(h, s)| super::host_ok(&h, s)) {
                    return Resp::fail("redirected to an address that is not allowed");
                }
            }
            let Some(status) = num(req, QUERY_STATUS_CODE) else { return Resp::fail("no status") };
            let retry_after = num(req, QUERY_RETRY_AFTER).unwrap_or(0);
            let expect = num(req, QUERY_CONTENT_LENGTH).unwrap_or(0) as usize;
            let mut body: Vec<u8> = Vec::new();
            let mut buf = vec![0u8; 65536];
            loop {
                let mut got = 0u32;
                if WinHttpReadData(req, buf.as_mut_ptr() as *mut c_void, buf.len() as u32, &mut got) == 0 {
                    return Resp::fail(format!("connection lost ({})", GetLastError()));
                }
                if got == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..got as usize]);
                if progress(body.len(), expect) {
                    return Resp::fail("stopped");
                }
                if body.len() > max {
                    return Resp::fail("answer is larger than allowed");
                }
            }
            Resp { status, body, retry_after, err: String::new() }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Resp;
    /// The system curl (no progress: curl runs to the end).
    pub fn get(url: &str, max: usize, _progress: &mut dyn FnMut(usize, usize) -> bool) -> Resp {
        if url.contains(['\'', '\n']) {
            return Resp::fail("address not allowed");
        }
        let dir = std::env::temp_dir();
        let id = std::process::id();
        let (body, hdr) = (dir.join(format!("gd-nucleus-{id}.body")), dir.join(format!("gd-nucleus-{id}.hdr")));
        let out = std::process::Command::new("curl")
            .args(["-sS", "-L", "--max-time", "120", "--connect-timeout", "10", "--max-filesize", &max.to_string(), "--proto", "=https,http", "--proto-redir", "=https", "-A", super::user_agent(), "-D"])
            .arg(&hdr)
            .arg("-o")
            .arg(&body)
            .args(["-w", "%{http_code} %{url_effective}", url])
            .output();
        let Ok(out) = out else { return Resp::fail("curl is not available") };
        let w = String::from_utf8_lossy(&out.stdout).to_string();
        let mut it = w.split_whitespace();
        let code: u32 = it.next().and_then(|c| c.parse().ok()).unwrap_or(0);
        let fin = it.next().unwrap_or("");
        let r = if code > 0 && !fin.is_empty() && !super::split(fin).is_some_and(|(h, s)| super::host_ok(&h, s)) {
            Resp::fail("redirected to an address that is not allowed")
        } else if code == 0 {
            Resp::fail("no answer (is curl installed?)")
        } else {
            let ra = std::fs::read_to_string(&hdr).unwrap_or_default().lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("retry-after:").and_then(|v| v.trim().parse().ok())).unwrap_or(0);
            Resp { status: code, body: std::fs::read(&body).unwrap_or_default(), retry_after: ra, err: String::new() }
        };
        let _ = std::fs::remove_file(&body);
        let _ = std::fs::remove_file(&hdr);
        r
    }
}

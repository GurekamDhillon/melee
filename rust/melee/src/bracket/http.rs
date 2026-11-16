//! One blocking HTTPS request to the bracket server, the Nucleus pattern (nucleus/http.rs) widened to a method, a body
//! and extra headers. Called only from the bracket worker thread, never the game thread.
//!
//! Windows: WinHTTP (system proxy and certificate store); redirects are followed and the FINAL address must still be
//! the pinned origin. Elsewhere: the system `curl`, with the body and the headers passed in temp files (never on the
//! command line). One origin only: `https://bracket.gsd.sh` unless `MELEE_BRACKET_SERVER` names another, which must be
//! https, or plain http to a loopback host (the local test server). Same User-Agent as the Nucleus client. Timeouts:
//! resolve 10 s, connect 10 s, send / receive 30 s (an upload: 120 s, the server's own limit).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    pub secure: bool,
    pub host: String,
    pub port: u16,
}

impl Origin {
    /// "https://host[:port]" (a trailing "/" is allowed; no path, query, user info). Plain http only to loopback.
    pub fn parse(base: &str) -> Option<Origin> {
        let base = base.strip_suffix('/').unwrap_or(base);
        let o = parse_url(base)?;
        if o.1 != "/" {
            return None;
        }
        let o = o.0;
        let loopback = o.host == "127.0.0.1" || o.host == "localhost" || o.host == "[::1]";
        (o.secure || loopback).then_some(o)
    }
    pub fn base(&self) -> String {
        let def = if self.secure { 443 } else { 80 };
        let s = if self.secure { "https" } else { "http" };
        if self.port == def { format!("{s}://{}", self.host) } else { format!("{s}://{}:{}", self.host, self.port) }
    }
}

/// An absolute http(s) URL -> (origin, path and query). Host: letters, digits, '.', '-', or a bracketed v6 literal.
pub fn parse_url(url: &str) -> Option<(Origin, String)> {
    let (rest, secure) = if let Some(r) = url.strip_prefix("https://") { (r, true) } else { (url.strip_prefix("http://")?, false) };
    let n = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (hp, path) = (&rest[..n], &rest[n..]);
    let (host, port) = if hp.starts_with('[') {
        let e = hp.find(']')?;
        let p = &hp[e + 1..];
        (&hp[..=e], if p.is_empty() { None } else { Some(p.strip_prefix(':')?.parse::<u16>().ok()?) })
    } else {
        match hp.rsplit_once(':') {
            Some((h, p)) => (h, Some(p.parse::<u16>().ok()?)),
            None => (hp, None),
        }
    };
    if host.is_empty() || host.len() > 96 || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b".-[]:".contains(&b)) {
        return None;
    }
    let path = if path.is_empty() { "/".to_string() } else { path.split('#').next().unwrap_or("/").to_string() };
    Some((Origin { secure, host: host.to_ascii_lowercase(), port: port.unwrap_or(if secure { 443 } else { 80 }) }, path))
}

pub struct Resp {
    /// 0: no answer (network, refused address); else the HTTP status.
    pub status: u32,
    pub body: Vec<u8>,
    pub retry_after: u32,
    /// The ETag header ("" when none): sent back as If-None-Match (the badge's 304 polling).
    pub etag: String,
    pub err: String,
}

impl Resp {
    pub fn fail(e: impl Into<String>) -> Resp {
        Resp { status: 0, body: Vec::new(), retry_after: 0, etag: String::new(), err: e.into() }
    }
}

const MAX_ANSWER: usize = 1 << 20;

/// `method` `path_and_query` on `origin`, with `headers` and `body`; `long` raises the send/receive timeouts to 120 s.
pub fn request(origin: &Origin, method: &str, path_and_query: &str, headers: &[(&str, String)], body: &[u8], long: bool) -> Resp {
    if !path_and_query.starts_with('/') || path_and_query.contains(['\r', '\n', ' ', '#']) || headers.iter().any(|(k, v)| k.contains([':', '\r', '\n']) || v.contains(['\r', '\n'])) {
        return Resp::fail("bad request");
    }
    imp::request(origin, method, path_and_query, headers, body, long)
}

#[cfg(windows)]
mod imp {
    use super::{Origin, Resp, MAX_ANSWER};
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
        fn WinHttpSetTimeouts(h: H, resolve: i32, connect: i32, send: i32, recv: i32) -> i32;
        fn WinHttpReadData(r: H, buf: *mut c_void, n: u32, read: *mut u32) -> i32;
        fn WinHttpCloseHandle(h: H) -> i32;
    }
    extern "system" {
        fn GetLastError() -> u32;
    }
    const QUERY_STATUS_CODE: u32 = 19;
    const QUERY_RETRY_AFTER: u32 = 36;
    const QUERY_CUSTOM: u32 = 65535;
    const QUERY_FLAG_NUMBER: u32 = 0x2000_0000;
    const OPTION_URL: u32 = 34;
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

    pub fn request(o: &Origin, method: &str, path: &str, headers: &[(&str, String)], body: &[u8], long: bool) -> Resp {
        let mut hs = Handles([std::ptr::null_mut(); 3]);
        let host = o.host.trim_start_matches('[').trim_end_matches(']');
        unsafe {
            let ua = w(crate::nucleus::http::user_agent());
            hs.0[0] = WinHttpOpen(ua.as_ptr(), 4 /* automatic proxy */, std::ptr::null(), std::ptr::null(), 0);
            if hs.0[0].is_null() {
                hs.0[0] = WinHttpOpen(ua.as_ptr(), 0, std::ptr::null(), std::ptr::null(), 0);
            }
            if hs.0[0].is_null() {
                return Resp::fail(format!("cannot start WinHTTP ({})", GetLastError()));
            }
            let t = if long { 120_000 } else { 30_000 };
            WinHttpSetTimeouts(hs.0[0], 10_000, 10_000, t, t);
            hs.0[1] = WinHttpConnect(hs.0[0], w(host).as_ptr(), o.port, 0);
            if !hs.0[1].is_null() {
                hs.0[2] = WinHttpOpenRequest(hs.0[1], w(method).as_ptr(), w(path).as_ptr(), std::ptr::null(), std::ptr::null(), std::ptr::null(), if o.secure { FLAG_SECURE } else { 0 });
            }
            let req = hs.0[2];
            if req.is_null() {
                return Resp::fail(format!("cannot open the request ({})", GetLastError()));
            }
            let mut hdr = String::new();
            for (k, v) in headers {
                hdr.push_str(&format!("{k}: {v}\r\n"));
            }
            let hw: Vec<u16> = hdr.encode_utf16().collect();
            let (hp, hl) = if hw.is_empty() { (std::ptr::null(), 0) } else { (hw.as_ptr(), hw.len() as u32) };
            let (bp, bl) = if body.is_empty() { (std::ptr::null(), 0) } else { (body.as_ptr() as *const c_void, body.len() as u32) };
            if WinHttpSendRequest(req, hp, hl, bp, bl, bl, 0) == 0 || WinHttpReceiveResponse(req, std::ptr::null_mut()) == 0 {
                return Resp::fail(format!("no answer ({})", GetLastError()));
            }
            // the final address after redirects must still be the pinned origin
            let mut fin = [0u16; 1200];
            let mut fl = (fin.len() * 2) as u32;
            if WinHttpQueryOption(req, OPTION_URL, fin.as_mut_ptr() as *mut c_void, &mut fl) != 0 {
                let f = String::from_utf16_lossy(&fin[..(fl as usize / 2).min(fin.len())]);
                if !super::parse_url(f.trim_end_matches('\0')).is_some_and(|(fo, _)| &fo == o) {
                    return Resp::fail("redirected to an address that is not allowed");
                }
            }
            let Some(status) = num(req, QUERY_STATUS_CODE) else { return Resp::fail("no status") };
            let retry_after = num(req, QUERY_RETRY_AFTER).unwrap_or(0);
            let etag = {
                let mut b = [0u16; 256];
                let mut l = (b.len() * 2) as u32;
                let name = w("ETag");
                if WinHttpQueryHeaders(req, QUERY_CUSTOM, name.as_ptr(), b.as_mut_ptr() as *mut c_void, &mut l, std::ptr::null_mut()) != 0 {
                    String::from_utf16_lossy(&b[..(l as usize / 2).min(b.len())]).trim_end_matches(' ').to_string()
                } else {
                    String::new()
                }
            };
            let mut out = Vec::new();
            let mut buf = vec![0u8; 16384];
            loop {
                let mut got = 0u32;
                if WinHttpReadData(req, buf.as_mut_ptr() as *mut c_void, buf.len() as u32, &mut got) == 0 {
                    return Resp::fail(format!("connection lost ({})", GetLastError()));
                }
                if got == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..got as usize]);
                if out.len() > MAX_ANSWER {
                    return Resp::fail("answer is larger than allowed");
                }
            }
            Resp { status, body: out, retry_after, etag, err: String::new() }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::{Origin, Resp, MAX_ANSWER};
    /// The system curl. Body and headers go through temp files (user-only), so nothing signed is on a command line.
    pub fn request(o: &Origin, method: &str, path: &str, headers: &[(&str, String)], body: &[u8], long: bool) -> Resp {
        use std::io::Write;
        let url = format!("{}{path}", o.base());
        if url.contains(['\'', '\n']) {
            return Resp::fail("address not allowed");
        }
        let dir = std::env::temp_dir();
        let id = format!("{}-{:?}", std::process::id(), std::thread::current().id()).replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
        let p = |s: &str| dir.join(format!("gd-bracket-{id}.{s}"));
        let (bf, hf, rf, of) = (p("body"), p("hdr"), p("rhdr"), p("out"));
        let write = |path: &std::path::Path, data: &[u8]| -> std::io::Result<()> {
            let mut opt = std::fs::OpenOptions::new();
            opt.write(true).create(true).truncate(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut opt, 0o600);
            opt.open(path)?.write_all(data)
        };
        let hdr: String = headers.iter().map(|(k, v)| format!("{k}: {v}\n")).collect();
        if write(&bf, body).is_err() || write(&hf, hdr.as_bytes()).is_err() {
            return Resp::fail("cannot write a temp file");
        }
        let t = if long { "150" } else { "45" };
        let mut c = std::process::Command::new("curl");
        c.args(["-sS", "-L", "--max-time", t, "--connect-timeout", "10", "--max-filesize", &MAX_ANSWER.to_string(), "--proto", "=https,http", "--proto-redir", if o.secure { "=https" } else { "=http,https" }, "-A", crate::nucleus::http::user_agent(), "-X", method, "-H"])
            .arg(format!("@{}", hf.display()))
            .arg("-D")
            .arg(&rf)
            .arg("-o")
            .arg(&of);
        if !body.is_empty() || method == "POST" {
            c.arg("--data-binary").arg(format!("@{}", bf.display()));
        }
        c.args(["-w", "%{http_code} %{url_effective}", &url]);
        let out = c.output();
        let r = match out {
            Err(_) => Resp::fail("curl is not available"),
            Ok(out) => {
                let w = String::from_utf8_lossy(&out.stdout).to_string();
                let mut it = w.split_whitespace();
                let code: u32 = it.next().and_then(|c| c.parse().ok()).unwrap_or(0);
                let fin = it.next().unwrap_or("");
                if code > 0 && !super::parse_url(fin).is_some_and(|(fo, _)| &fo == o) {
                    Resp::fail("redirected to an address that is not allowed")
                } else if code == 0 {
                    Resp::fail("no answer (is curl installed?)")
                } else {
                    let h = std::fs::read_to_string(&rf).unwrap_or_default();
                    let ra = h.lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("retry-after:").and_then(|v| v.trim().parse().ok())).unwrap_or(0);
                    let etag = h.lines().find_map(|l| l.split_once(':').filter(|(k, _)| k.trim().eq_ignore_ascii_case("etag")).map(|(_, v)| v.trim().to_string())).unwrap_or_default();
                    Resp { status: code, body: std::fs::read(&of).unwrap_or_default(), retry_after: ra, etag, err: String::new() }
                }
            }
        };
        for f in [bf, hf, rf, of] {
            let _ = std::fs::remove_file(f);
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origins() {
        assert_eq!(Origin::parse("https://bracket.gsd.sh").unwrap().base(), "https://bracket.gsd.sh");
        assert_eq!(Origin::parse("https://bracket.gsd.sh/").unwrap().port, 443);
        assert_eq!(Origin::parse("http://127.0.0.1:8123").unwrap().base(), "http://127.0.0.1:8123");
        assert!(Origin::parse("http://bracket.gsd.sh").is_none(), "plain http only to loopback");
        assert!(Origin::parse("https://bracket.gsd.sh/v1").is_none(), "an origin has no path");
        assert!(Origin::parse("https://user@evil.example").is_none());
        let (o, p) = parse_url("https://bracket.gsd.sh/v1/me/next?event=1").unwrap();
        assert_eq!((o, p.as_str()), (Origin::parse("https://bracket.gsd.sh").unwrap(), "/v1/me/next?event=1"));
        assert_ne!(parse_url("https://bracket.gsd.sh.evil.example/").unwrap().0, Origin::parse("https://bracket.gsd.sh").unwrap());
        assert_ne!(parse_url("http://bracket.gsd.sh/").unwrap().0, Origin::parse("https://bracket.gsd.sh").unwrap());
    }
}

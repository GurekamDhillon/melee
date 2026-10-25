//! gw_log.c's logger vs the Rust port: tests/data/log_oracle.txt is what the shipping C printed for
//! tests/data/log_script.txt (oracle/log_oracle.sh): every line written (stamped) and the noted state
//! after each operation.

use melee_data::crashlog::{LogHost, Logger, ReportHost};

fn unhex(h: &str) -> Vec<u8> {
    if h == "-" {
        return Vec::new();
    }
    (0..h.len() / 2).map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap()).collect()
}

fn hex(b: &[u8]) -> String {
    if b.is_empty() {
        "-".into()
    } else {
        b.iter().map(|c| format!("{c:02x}")).collect()
    }
}

fn subst(b: Vec<u8>, user: &str, stamp: &str) -> Vec<u8> {
    let s = String::from_utf8(b).unwrap();
    s.replace("@USER@", user).replace("@PROFILE@", r"C:\Users\Testy").replace("@STAMP@", stamp).into_bytes()
}

fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &c in b {
        h ^= c as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

#[test]
fn logger_matches_the_c() {
    let user = std::env::var("USERNAME").unwrap();
    let stamp = "1badcafe";
    let dir = std::env::temp_dir().join(format!("melee_data_log_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let mut summary: Vec<u8> = Vec::new();
    let mut version: Option<Vec<u8>> = None;
    let mut map: Option<Vec<u8>> = None;
    let script = include_str!("data/log_script.txt");
    let want: Vec<&str> = include_str!("data/log_oracle.txt").lines().collect();
    let mut got: Vec<String> = Vec::new();
    let mut lg = Logger::new(LogHost::default());
    let mut seen = 0usize;
    for line in script.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        match f[0] {
            "T" => {
                lg.host.tick64 = f[1].parse().unwrap();
                lg.host.frame = f[2].parse().unwrap();
            }
            "ENV" => {
                lg.host.env.insert(f[1].into(), String::from_utf8(unhex(f[2])).unwrap());
            }
            "UNENV" => {
                lg.host.env.remove(f[1]);
            }
            "SLOG" => lg.host.settings_log = unhex(f[1]),
            "ISO" => lg.host.iso_path = Some(subst(unhex(f[1]), &user, stamp)),
            "NOISO" => lg.host.iso_path = None,
            "LOG" => lg.log(&subst(unhex(f[1]), &user, stamp)),
            "OSR" => lg.osreport(&subst(unhex(f[1]), &user, stamp)),
            "SUMM" => summary = unhex(f[1]),
            "VERSION" => version = if f[1] == "~" { None } else { Some(subst(unhex(f[1]), &user, stamp)) },
            "MAP" => map = if f[1] == "~" { None } else { Some(subst(unhex(f[1]), &user, stamp)) },
            "DISCFILE" => {
                if f[1] == "~" {
                    let _ = std::fs::remove_file("disc.bin");
                } else {
                    std::fs::write("disc.bin", unhex(f[1])).unwrap();
                }
            }
            "REPORT" => {
                let reason = if f[2] == "~" { None } else { Some(String::from_utf8(subst(unhex(f[2]), &user, stamp)).unwrap()) };
                let tm: Vec<u16> = f[3].split('/').map(|x| x.parse().unwrap()).collect();
                let disc_title = if lg.disc_path.is_empty() {
                    None
                } else {
                    std::fs::read(String::from_utf8_lossy(&lg.disc_path).as_ref()).ok().and_then(|d| d.get(0x20..0x60).map(|x| x.to_vec()))
                };
                let mut env: Vec<(String, String)> = lg.host.env.iter().filter(|(k, _)| k.starts_with("MELEE_")).map(|(k, v)| (k.clone(), v.clone())).collect();
                env.sort();
                let host = ReportHost {
                    time: (tm[0], tm[1], tm[2], tm[3], tm[4], tm[5]),
                    version_line: version.as_ref().map(|v| {
                        let end = v.iter().position(|&c| c == b'\n').map_or(v.len(), |p| p + 1).min(511);
                        v[..end].to_vec()
                    }),
                    link_stamp: 0x1badcafe,
                    disc_title,
                    settings_summary: summary.iter().copied().take(1023).collect(),
                    melee_env: env.iter().map(|(k, v)| format!("{k}={v}").into_bytes()).collect(),
                    os_line: b"X".to_vec(),
                    map: map.clone(),
                    user: user.clone().into_bytes(),
                    profile: br"C:\Users\Testy".to_vec(),
                };
                let kind = String::from_utf8(unhex(f[1])).unwrap();
                let rep = lg.write_report(&kind, reason.as_deref(), &host);
                if let Ok(p) = std::env::var("RUST_DUMP") {
                    use std::io::Write;
                    let mut d = std::fs::OpenOptions::new().create(true).append(true).open(p).unwrap();
                    d.write_all(&rep).unwrap();
                    d.write_all(&[10]).unwrap();
                    d.write_all(b"=====").unwrap();
                    d.write_all(&[10]).unwrap();
                }
                let (mut h, mut n) = (1469598103934665603u64, 0usize);
                for l in rep.split_inclusive(|&c| c == b'\n') {
                    if !l.starts_with(b"os:  ") && !l.starts_with(b"build id:") {
                        h = fnv(h, l);
                        n += l.len();
                    }
                }
                got.push(format!("REPORT {n} {h:016x}"));
            }
            "RAW" => {
                let t = if f[2] == "~" { None } else { Some(unhex(f[2])) };
                lg.raw(&unhex(f[1]), t.as_deref());
            }
            "CRASH" => lg.crash_begin(),
            "GOBJ" => got.push(format!("GOBJ {}", lg.gobj_log_enabled() as i32)),
            "DESC" => {
                lg.gobj_log_enabled();
                got.push(format!("DESC {} {}", hex(&lg.describe()), hex(lg.spec())));
            }
            "RESET" => {
                let host = lg.host.clone();
                lg = Logger::new(host);
                seen = 0;
            }
            "ENVQ" => {
                let v = lg.host.env.get(f[1]);
                got.push(format!("ENVQ {} {}", v.is_some() as i32, hex(v.map(|s| s.as_bytes()).unwrap_or(b""))));
            }
            _ => panic!("bad op {line}"),
        }
        while seen < lg.out.len() {
            got.push(format!("W {}", hex(&lg.out[seen])));
            seen += 1;
        }
        let (nt, ns, np, nw) = lg.counts();
        got.push(format!(
            "ST {} {} {} {} {} {nt} {ns} {np} {nw}",
            hex(&lg.disc_path),
            hex(&lg.disc_id),
            hex(&lg.mods),
            lg.shutting_down as i32,
            lg.crashing() as i32
        ));
    }
    assert_eq!(got.len(), want.len(), "line count");
    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        assert_eq!(g, w, "line {i}");
    }
}

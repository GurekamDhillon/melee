//! gw_log.c's logger vs the Rust port: tests/data/log_oracle.txt is what the shipping C printed for
//! tests/data/log_script.txt (oracle/log_oracle.sh): every line written (stamped) and the noted state
//! after each operation.

use melee_data::crashlog::{LogHost, Logger};

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

#[test]
fn logger_matches_the_c() {
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
            "ISO" => lg.host.iso_path = Some(unhex(f[1])),
            "NOISO" => lg.host.iso_path = None,
            "LOG" => lg.log(&unhex(f[1])),
            "OSR" => lg.osreport(&unhex(f[1])),
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

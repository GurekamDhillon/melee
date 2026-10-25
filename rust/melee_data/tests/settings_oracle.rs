//! settings.cfg vs the shipping C: tests/data/settings_oracle.txt is what gw_settings.c's own table
//! code printed for tests/data/settings_script.txt (oracle/settings_oracle.sh). The Rust port must
//! print the same line for every operation, including the cfg file's bytes after each write.

use melee_data::settings::Settings;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

fn file_line(path: &std::path::Path) -> String {
    format!("FILE {}", hex(&std::fs::read(path).unwrap_or_default()))
}

#[test]
fn settings_match_the_c() {
    let script = include_str!("data/settings_script.txt");
    let want: Vec<&str> = include_str!("data/settings_oracle.txt").lines().collect();
    let dir = std::env::temp_dir().join(format!("melee_data_settings_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let good = dir.join("oracle.cfg");
    let _ = std::fs::remove_file(&good);
    let env: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::default();
    let e2 = env.clone();
    let mut st = Settings::with_env(&good, Box::new(move |n| e2.lock().unwrap().get(n).cloned()));
    let mut got: Vec<String> = Vec::new();
    for line in script.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        match f[0] {
            "S" => {
                st.set_str(f[1], &String::from_utf8(unhex(f[2])).unwrap_or_else(|_| unreachable!()));
                got.push(format!("S ok={}", st.last_save_ok() as i32));
                got.push(file_line(st.path()));
            }
            "I" => {
                st.set_int(f[1], f[2].parse().unwrap());
                got.push(format!("I ok={}", st.last_save_ok() as i32));
                got.push(file_line(st.path()));
            }
            "GS" => {
                let cap: usize = f[2].parse().unwrap();
                let d = if f[3] == "~" { None } else { Some(String::from_utf8(unhex(f[3])).unwrap()) };
                let (found, v) = st.get_str(f[1], cap, d.as_deref());
                got.push(format!("GS {} {}", found as i32, hex(&v)));
            }
            "GI" => got.push(format!("GI {}", st.get_int(f[1], f[2].parse().unwrap()))),
            "R" => {
                let r = st.remove_prefix(f[1]);
                got.push(format!("R {} ok={}", r as i32, st.last_save_ok() as i32));
                got.push(file_line(st.path()));
            }
            "SUM" => {
                let (s, k) = st.summary(f[1].parse().unwrap());
                got.push(format!("SUM {k} {}", hex(&s)));
            }
            "ENV" => {
                env.lock().unwrap().insert(f[1].to_string(), unhex(f[2]));
                got.push("ENV".into());
            }
            "UNENV" => {
                env.lock().unwrap().insert(f[1].to_string(), Vec::new());
                got.push("UNENV".into());
            }
            "RAW" => {
                std::fs::write(&good, unhex(f[1])).unwrap();
                st.set_path(&good);
                st.reload();
                got.push(format!("RAW {}", st.get_int("zzz_probe", -1)));
                got.push(format!("ROWS {}", st.len()));
            }
            "RELOAD" => {
                st.set_path(&good);
                st.reload();
                st.get_int("zzz_probe", -1);
                got.push(format!("RELOAD rows={}", st.len()));
            }
            "BADPATH" => {
                st.set_path("Z:\\no\\such\\dir\\settings.cfg");
                got.push("BADPATH".into());
            }
            "GOODPATH" => {
                st.set_path(&good);
                got.push("GOODPATH".into());
            }
            "FILE" => got.push(file_line(&good)),
            other => panic!("bad op {other}"),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    for (i, (w, g)) in want.iter().zip(got.iter()).enumerate() {
        assert_eq!(w, g, "line {} differs", i + 1);
    }
    assert_eq!(want.len(), got.len());
    assert!(want.len() > 3000);
}

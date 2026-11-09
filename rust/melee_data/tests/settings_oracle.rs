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
#[cfg_attr(not(windows), ignore = "the C oracle ran on Windows: text-mode stdio (CRLF on write; CR before LF dropped and ^Z = EOF on read); see glibc_text_mode")]
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
                // unwritable on every OS: its parent is a regular file (off Windows "Z:\\no\\such\\dir\\settings.cfg" is a
                // valid file name, so the saves succeeded and left that file in the crate directory)
                st.set_path(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml/settings.cfg"));
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

/// Off Windows the store does what the C's stdio does on glibc (settings.rs text_mode_read / text_mode_write): a save
/// writes "\n" line ends, and a ^Z byte is data, not end of file, so the rows after it load.
#[test]
#[cfg(not(windows))]
fn glibc_text_mode() {
    let dir = std::env::temp_dir().join(format!("melee_data_settings_glibc_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("glibc.cfg");
    let _ = std::fs::remove_file(&p);
    let mut st = Settings::new(&p);
    st.set_str("server", "h");
    st.set_int("ucf", 9);
    let raw = std::fs::read(&p).unwrap();
    assert!(st.last_save_ok());
    assert!(!raw.contains(&b'\r') && raw.ends_with(b"ucf=9\n"), "{raw:?}");
    std::fs::write(&p, b"a=1\r\n\x1ab=2\nc=3\n").unwrap();
    st.reload();
    assert_eq!((st.get_int("a", -1), st.get_int("c", -1)), (1, 3));
    assert_eq!(st.len(), 3, "^Z ends nothing: a, \\x1ab and c load");
    let _ = std::fs::remove_dir_all(&dir);
}

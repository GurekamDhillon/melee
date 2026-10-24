//! The memory card (GCI folder) vs the shipping Aurora CARD library: tests/data/card_scenarios.txt are
//! random card folders and CARD call sequences (oracle/card_scenarios.py); tests/data/card_oracle.txt is
//! what aurora_card.lib printed for each (oracle/card_oracle.py). The Rust card must print the same
//! result codes, statuses and read data, and leave byte-identical `.gci` files behind.

use melee_data::card::{res, CardSystem, FileInfo, Stat};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn unhex(h: &str) -> Vec<u8> {
    if h == "-" {
        return Vec::new();
    }
    (0..h.len() / 2).map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap()).collect()
}

fn hx(b: &[u8]) -> String {
    if b.is_empty() {
        "-".into()
    } else {
        b.iter().map(|c| format!("{c:02x}")).collect()
    }
}

fn fnv(b: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &c in b {
        h ^= c as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let r = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if e.path().is_dir() {
                walk(&e.path(), &r, out);
            } else {
                out.push((r, e.path()));
            }
        }
    }
}

fn pr_fi(o: &mut String, f: &FileInfo) {
    write!(o, " fi={},{},{},{},{}", f.chan, f.file_no, f.offset, f.length, f.i_block).unwrap();
}

fn run(script: &str, root: &str) -> String {
    let mut o = String::new();
    let mut card = CardSystem::new(Arc::new(|| 0x1234_5678));
    let mut fis = [FileInfo::default(); 16];
    for line in script.lines() {
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(' ').collect();
        let a = |i: usize| -> &str { f.get(i).copied().unwrap_or("0") };
        let int = |i: usize| -> i32 { a(i).parse::<i64>().unwrap() as i32 };
        let nm = |i: usize| -> &[u8] { f[i].as_bytes() };
        match f[0] {
            "MD" => {
                std::fs::create_dir_all(format!("{root}/{}", f[1].replace('~', " "))).unwrap();
            }
            "MK" => {
                let p = format!("{root}/{}", f[1].replace('~', " "));
                std::fs::create_dir_all(Path::new(&p).parent().unwrap()).unwrap();
                std::fs::write(p, unhex(f[2])).unwrap();
            }
            "INIT" => {
                card.init(Path::new(root));
                writeln!(o, "INIT").unwrap();
            }
            "GAME" => {
                card.set_game_and_maker(int(1), nm(2), nm(3));
                writeln!(o, "GAME").unwrap();
            }
            "PROBE" => writeln!(o, "PROBE {}", card.probe(int(1))).unwrap(),
            "PROBEEX" => {
                let (r, m, s) = card.probe_ex(int(1));
                // (the oracle calls Aurora directly: out-params stay at their -7 marker on a bad channel)
                let (m, s) = if r == res::FATAL_ERROR { (-7, -7) } else { (m, s) };
                writeln!(o, "PROBEEX {r} {m} {s}").unwrap();
            }
            "CHECK" => writeln!(o, "CHECK {}", card.check(int(1))).unwrap(),
            "FORMAT" => writeln!(o, "FORMAT {}", card.format(int(1))).unwrap(),
            "FREE" => {
                let (r, b, n) = card.free_blocks(int(1));
                let (b, n) = if r == res::FATAL_ERROR { (-7, -7) } else { (b, n) };
                writeln!(o, "FREE {r} {b} {n}").unwrap();
            }
            "ENCODING" => {
                let (r, e) = card.encoding(int(1));
                let e = if r == res::FATAL_ERROR { 99 } else { e };
                writeln!(o, "ENCODING {r} {e}").unwrap();
            }
            "SECTOR" => {
                let (r, s) = card.sector_size(int(1));
                let s = if r == res::FATAL_ERROR { 99 } else { s };
                writeln!(o, "SECTOR {r} {s}").unwrap();
            }
            "SERIAL" => {
                let (r, s) = card.serial_no(int(1));
                let s = if r == res::FATAL_ERROR { 99 } else { s };
                writeln!(o, "SERIAL {r} {s}").unwrap();
            }
            "RESULT" => writeln!(o, "RESULT {}", card.result_code(int(1))).unwrap(),
            "XFER" => writeln!(o, "XFER {}", card.xferred_bytes(int(1))).unwrap(),
            "MOUNT" => writeln!(o, "MOUNT {}", card.mount(int(1))).unwrap(),
            "UNMOUNT" => writeln!(o, "UNMOUNT {}", card.unmount(int(1))).unwrap(),
            "CREATE" => {
                let slot = int(4) as usize;
                let mut fi = FileInfo::default();
                let size = a(3).parse::<u64>().unwrap() as u32;
                let r = card.create(int(1), nm(2), size, &mut fi);
                fis[slot] = fi;
                write!(o, "CREATE {r}").unwrap();
                pr_fi(&mut o, &fis[slot]);
                writeln!(o).unwrap();
            }
            "OPEN" => {
                let slot = int(3) as usize;
                let mut fi = FileInfo::default();
                let r = card.open(int(1), nm(2), &mut fi);
                fis[slot] = fi;
                write!(o, "OPEN {r}").unwrap();
                pr_fi(&mut o, &fis[slot]);
                writeln!(o).unwrap();
            }
            "FASTOPEN" => {
                let slot = int(3) as usize;
                let mut fi = FileInfo::default();
                let r = card.fast_open(int(1), int(2), &mut fi);
                fis[slot] = fi;
                write!(o, "FASTOPEN {r}").unwrap();
                pr_fi(&mut o, &fis[slot]);
                writeln!(o).unwrap();
            }
            "CLOSE" => {
                let slot = int(1) as usize;
                let mut fi = fis[slot];
                let r = card.close(&mut fi);
                writeln!(o, "CLOSE {r}").unwrap();
            }
            "READ" => {
                let slot = int(1) as usize;
                let (len, off) = (int(2), int(3));
                let mut buf = vec![0xEEu8; (if len > 0 { len as usize } else { 1 }) + 8];
                let r = card.read(&fis[slot], &mut buf, len, off);
                write!(o, "READ {r} ").unwrap();
                if r == 0 && len > 0 {
                    write!(o, "{:016x} {}", fnv(&buf[..len as usize]), hx(&buf[..(len as usize).min(16)])).unwrap();
                } else {
                    write!(o, "-").unwrap();
                }
                writeln!(o).unwrap();
            }
            "WRITE" => {
                let slot = int(1) as usize;
                let (len, off) = (int(2), int(3));
                let seed = a(4).parse::<u64>().unwrap() as u32;
                let n = if len > 0 { len as usize } else { 1 };
                let buf: Vec<u8> = (0..n).map(|i| (seed.wrapping_add((i as u32).wrapping_mul(7)) & 255) as u8).collect();
                let r = card.write(&fis[slot], &buf, len, off);
                writeln!(o, "WRITE {r}").unwrap();
            }
            "DELETE" => writeln!(o, "DELETE {}", card.delete(int(1), nm(2))).unwrap(),
            "FASTDELETE" => writeln!(o, "FASTDELETE {}", card.fast_delete(int(1), int(2))).unwrap(),
            "RENAME" => writeln!(o, "RENAME {}", card.rename(int(1), nm(2), nm(3))).unwrap(),
            "STATUS" => {
                let mut st = Stat::filled(0xAB);
                let r = card.get_status(int(1), int(2), &mut st);
                write!(o, "STATUS {r}").unwrap();
                if r == 0 {
                    let n = st.file_name.iter().position(|&c| c == 0).unwrap_or(32);
                    write!(
                        o,
                        " name={} len={} t game={} co={} bf={} ia={} if={} is={} ca={} ob={} obt={} oi=",
                        hx(&st.file_name[..n]),
                        st.length,
                        hx(&st.game_name),
                        hx(&st.company),
                        st.banner_format,
                        st.icon_addr,
                        st.icon_format,
                        st.icon_speed,
                        st.comment_addr,
                        st.offset_banner,
                        st.offset_banner_tlut
                    )
                    .unwrap();
                    let oi: Vec<String> = st.offset_icon.iter().map(|v| v.to_string()).collect();
                    write!(o, "{} oit={} od={}", oi.join(","), st.offset_icon_tlut, st.offset_data).unwrap();
                }
                writeln!(o).unwrap();
            }
            "SETSTATUS" => {
                let mut st = Stat::zeroed();
                st.banner_format = a(3).parse::<i64>().unwrap() as u8;
                st.icon_addr = a(4).parse::<u64>().unwrap() as u32;
                st.icon_format = a(5).parse::<i64>().unwrap() as u16;
                st.icon_speed = a(6).parse::<i64>().unwrap() as u16;
                st.comment_addr = a(7).parse::<u64>().unwrap() as u32;
                writeln!(o, "SETSTATUS {}", card.set_status(int(1), int(2), &st)).unwrap();
            }
            "REMOUNT" => writeln!(o, "REMOUNT {}", card.remount(int(1)) as i32).unwrap(),
            "FILES" => {
                let mut v = Vec::new();
                walk(Path::new(root), "", &mut v);
                v.sort_by(|a, b| a.0.cmp(&b.0));
                for (rel, p) in v {
                    let mut d = std::fs::read(&p).unwrap();
                    if rel.ends_with(".gci") && d.len() >= 0x2C {
                        d[0x28..0x2C].fill(0);
                    }
                    writeln!(o, "F {rel} {} {:016x}", d.len(), fnv(&d)).unwrap();
                }
            }
            other => panic!("bad op {other}"),
        }
    }
    let _ = res::READY;
    o
}

#[test]
fn card_scenarios_match_the_c() {
    let scen = include_str!("data/card_scenarios.txt").replace("\r\n", "\n");
    let want_all = include_str!("data/card_oracle.txt").replace("\r\n", "\n");
    let mut want = std::collections::HashMap::new();
    for b in want_all.split("== ").skip(1) {
        let (h, body) = b.split_once('\n').unwrap();
        want.insert(h.trim().to_string(), body.trim_end_matches('\n').to_string());
    }
    let base = std::env::temp_dir().join(format!("melee_data_card_{}", std::process::id()));
    let mut n = 0;
    let only = std::env::var("CARD_ONLY").ok();
    for blk in scen.split("=== ").skip(1) {
        let (name, script) = blk.split_once('\n').unwrap();
        let name = name.trim();
        if only.as_deref().map_or(false, |o| o != name) {
            continue;
        }
        let root_p = base.join(name);
        let _ = std::fs::remove_dir_all(&root_p);
        std::fs::create_dir_all(&root_p).unwrap();
        let root = root_p.to_string_lossy().replace('\\', "/");
        let got = run(script, &root);
        let got = got.trim_end_matches('\n');
        let w = &want[name];
        if got != w {
            if std::env::var("CARD_KEEP").is_ok() {
                let dst = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/card-debug");
                let _ = std::fs::remove_dir_all(&dst);
                let mut v = Vec::new();
                walk(&root_p, "", &mut v);
                for (rel, p) in v {
                    let d = dst.join(&rel);
                    std::fs::create_dir_all(d.parent().unwrap()).unwrap();
                    std::fs::copy(p, d).unwrap();
                }
            }
            let wl: Vec<&str> = w.lines().collect();
            let gl: Vec<&str> = got.lines().collect();
            for i in 0..wl.len().max(gl.len()) {
                if wl.get(i) != gl.get(i) {
                    panic!("{name}: line {} differs\n  C:    {:?}\n  Rust: {:?}", i + 1, wl.get(i), gl.get(i));
                }
            }
        }
        n += 1;
    }
    if std::env::var("CARD_KEEP").is_err() {
        let _ = std::fs::remove_dir_all(&base);
    } else {
        eprintln!("kept {}", base.display());
    }
    assert!(n >= 1);
}

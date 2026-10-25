//! The mods registry vs the shipping gw_mods.c (linked unchanged) and shim_dvd.c's overlay scan:
//! tests/data/mods_scenarios.txt is a set of random mods folders and operation scripts
//! (oracle/mods_scenarios.py); tests/data/mods_oracle.txt is what the C printed for each. The Rust
//! run builds the same folder and must print the same transcript, log lines included.

use melee_data::modreg::{self, ModSet, Overlay};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

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

/// a host path with the scenario root replaced by "<R>"
fn hxp(p: &str, root: &str) -> String {
    match p.strip_prefix(root) {
        Some(rest) => {
            let mut v = b"<R>".to_vec();
            v.extend_from_slice(rest.as_bytes());
            hx(&v)
        }
        None => hx(p.as_bytes()),
    }
}

struct Run {
    out: String,
    logged: usize,
    set: Option<ModSet>,
    overlay: Option<Overlay>,
    overlay_logged: usize,
    mods_env: String,
    root: String,
}

impl Run {
    fn flush_logs(&mut self) {
        if let Some(s) = &self.set {
            while self.logged < s.log.len() {
                writeln!(self.out, "LOG {}", s.log[self.logged]).unwrap();
                self.logged += 1;
            }
        }
        if let Some(o) = &self.overlay {
            while self.overlay_logged < o.log.len() {
                writeln!(self.out, "LOG {}", o.log[self.overlay_logged]).unwrap();
                self.overlay_logged += 1;
            }
        }
    }

    fn boot(&mut self) {
        if self.set.is_none() {
            let dir = format!("{}/mods", self.root);
            let off = self.mods_env.starts_with('0');
            let mut s = ModSet::load(Path::new(&dir), off);
            if !s.mods.is_empty() {
                let l = s.summary_log();
                s.log.extend(l);
            }
            self.set = Some(s);
        }
        self.flush_logs();
    }

    fn mount(&mut self) {
        self.boot();
        if self.overlay.is_none() {
            let set = self.set.as_mut().unwrap();
            self.overlay = Some(Overlay::mount(set));
        }
        self.flush_logs();
    }
}

fn run_script(script: &str, root: &str) -> String {
    let mut r = Run { out: String::new(), logged: 0, set: None, overlay: None, overlay_logged: 0, mods_env: String::new(), root: root.to_string() };
    for line in script.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        match f[0] {
            "MD" => {
                std::fs::create_dir_all(format!("{root}/{}", f[1])).unwrap();
            }
            "MK" => {
                let p = format!("{root}/{}", f[1]);
                if let Some(parent) = Path::new(&p).parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(&p, unhex(f[2])).unwrap();
            }
            "ENV" => {
                if f[1] == "MELEE_MODS" {
                    r.mods_env = String::from_utf8(unhex(f[2])).unwrap();
                }
            }
            "BOOT" => {
                r.boot();
                let s = r.set.as_ref().unwrap();
                writeln!(r.out, "BOOT n={} dir={}", s.mods.len(), hxp(&s.dir.to_string_lossy(), root)).unwrap();
                for (i, m) in s.mods.iter().enumerate() {
                    let con: Vec<u8> = m.con.join(&b","[..]);
                    writeln!(
                        r.out,
                        "M {i} id={} name={} ver={} kind={} pack={} desc={} auto={} req={} con={} src={} nuc={} status={} text={} active={} enabled={} moddir={} find={} payload={}",
                        hx(&m.id),
                        hx(&m.name),
                        hx(&m.version),
                        hx(&m.kind),
                        hx(&m.pack),
                        hx(&m.desc),
                        hx(&m.autostart),
                        hx(&m.requires_text),
                        hx(&con),
                        hx(&m.source),
                        m.nucleus_post,
                        m.status,
                        hx(&m.status_text),
                        (m.status == modreg::MOD_ACTIVE) as i32,
                        m.enabled as i32,
                        m.payload_is_mod_dir as i32,
                        s.find(&m.id).map_or(-1, |i| i as i32),
                        hxp(&m.payload.to_string_lossy(), root)
                    )
                    .unwrap();
                }
                write!(r.out, "ORDER {}:", s.order.len()).unwrap();
                for k in &s.order {
                    write!(r.out, " {k}").unwrap();
                }
                writeln!(r.out).unwrap();
                writeln!(r.out, "FINDMISS {}", s.find(b"no-such-mod").map_or(-1, |i| i as i32)).unwrap();
            }
            "SET" => {
                r.boot();
                let i: i64 = f[1].parse().unwrap();
                let on = f[2].parse::<i32>().unwrap() != 0;
                let s = r.set.as_mut().unwrap();
                let n = if i < 0 || i as usize >= s.mods.len() { 0 } else { s.set_enabled(i as usize, on) };
                let restart = s.restart_needed();
                let flags: String = s.mods.iter().map(|m| if m.enabled { '1' } else { '0' }).collect();
                r.flush_logs();
                writeln!(r.out, "SET {n} restart={}\n{flags}", restart as i32).unwrap();
            }
            "SAVE" => {
                r.boot();
                let rc = r.set.as_mut().unwrap().save();
                let data = std::fs::read(format!("{root}/mods/enabled.txt")).unwrap_or_default();
                r.flush_logs();
                writeln!(r.out, "SAVE {rc} {}", hx(&data)).unwrap();
            }
            "RESTART" => {
                r.boot();
                writeln!(r.out, "RESTART {}", r.set.as_ref().unwrap().restart_needed() as i32).unwrap();
            }
            "FP" => {
                r.mount();
                let s = r.set.as_mut().unwrap();
                let fp = s.fingerprint();
                let d = s.describe();
                writeln!(r.out, "FP {fp:016x}\nDESC {}", hx(d.as_bytes())).unwrap();
                for i in 0..s.mods.len() {
                    writeln!(r.out, "DIG {i} {:016x}", s.digest(i)).unwrap();
                }
                for fl in &r.overlay.as_ref().unwrap().files {
                    write!(
                        r.out,
                        "F {} mod={} idx={} size={} nprov={} host={}",
                        hx(fl.disc.as_bytes()),
                        fl.mod_id,
                        fl.mod_index,
                        fl.size,
                        fl.prov.len(),
                        hxp(&fl.host.to_string_lossy(), root)
                    )
                    .unwrap();
                    for p in &fl.prov {
                        write!(r.out, " {}/{}", p.mod_id, p.size).unwrap();
                    }
                    writeln!(r.out).unwrap();
                }
            }
            "DIFF" => {
                r.mount();
                let peer = String::from_utf8(unhex(f[1])).unwrap();
                let cap: usize = f[2].parse().unwrap();
                let mine = r.set.as_mut().unwrap().describe();
                let (text, n) = modreg::diff_describe(&mine, &peer, cap);
                writeln!(r.out, "DIFF {n} {}", hx(text.as_bytes())).unwrap();
            }
            "WIN" => {
                r.boot();
                let w = r.set.as_ref().unwrap().winner_of(f[1]);
                writeln!(r.out, "WIN {} {}", w.is_some() as i32, hx(w.unwrap_or_default().as_bytes())).unwrap();
            }
            "ENGINE" => {
                let j = unhex(f[1]);
                let m = modreg::json_engine_miss(&j);
                writeln!(r.out, "ENGINE {} {}", m.is_some() as i32, hx(m.unwrap_or_default().as_bytes())).unwrap();
            }
            "CAT" => {
                let data = std::fs::read(format!("{root}/{}", f[1])).unwrap_or_default();
                writeln!(r.out, "CAT {}", hx(&data)).unwrap();
            }
            other => panic!("bad op {other}"),
        }
    }
    r.out.replace(root, "<R>")
}

#[test]
fn mods_scenarios_match_the_c() {
    let scen = include_str!("data/mods_scenarios.txt").replace("\r\n", "\n");
    let want_all = include_str!("data/mods_oracle.txt").replace("\r\n", "\n");
    let mut want = std::collections::HashMap::new();
    for b in want_all.split("== ").skip(1) {
        let (h, body) = b.split_once('\n').unwrap();
        want.insert(h.trim().to_string(), body);
    }
    let base: PathBuf = std::env::temp_dir().join(format!("melee_data_mods_{}", std::process::id()));
    let mut n = 0;
    for blk in scen.split("=== ").skip(1) {
        let (name, script) = blk.split_once('\n').unwrap();
        let root_p = base.join(name.trim());
        let _ = std::fs::remove_dir_all(&root_p);
        std::fs::create_dir_all(&root_p).unwrap();
        let root = root_p.to_string_lossy().replace('\\', "/");
        let got = run_script(script, &root);
        let w = want[name.trim()];
        if got != w {
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
    let _ = std::fs::remove_dir_all(&base);
    assert!(n >= 50);
}

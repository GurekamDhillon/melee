//! Playback, recording, seeds, parity and UCF queries vs the shipping gw_replay.c: every case in
//! tests/data/player_cases.txt was run through oracle/player_oracle.c (the whole of gw_replay.c
//! linked unchanged, oracle/player_oracle.sh) and the transcript stored in tests/data/player_oracle.txt.
//! The Rust program below is the same driver; the two transcripts must be identical, including the
//! log lines, the hashes of the state-trace files and the bytes of the recorded .slp.

use melee_data::gdlp::Identity;
use melee_data::player::{Env, Player, UNARMED};
use melee_data::replay::{FIRST_FRAME, SLOTS};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

struct Fnv(u64);
impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf29ce484222325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for &c in b {
            self.0 ^= c as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

fn flag_val<'a>(flags: &'a str, name: &str) -> Option<&'a str> {
    flags.split(',').find_map(|f| f.strip_prefix(name).and_then(|r| r.strip_prefix('=')))
}
fn has_flag(flags: &str, name: &str) -> bool {
    flags.split(',').any(|f| f == name || f.starts_with(&format!("{name}=")))
}
fn flag_int(flags: &str, name: &str, d: i64) -> i64 {
    flag_val(flags, name).and_then(|v| v.parse().ok()).unwrap_or(d)
}
fn fb(f: f32) -> u32 {
    f.to_bits()
}

/// (plain .slp recording, ./m.gdlp, the recordings-folder .gdlp)
fn split_rec3(p: &Player) -> (Option<&[u8]>, Option<&[u8]>, Option<&[u8]>) {
    let (mut rec, mut gdlp, mut gdlp2) = (None, None, None);
    let mut put = |path: &str, b: &'static [u8]| {
        let _ = (path, b);
    };
    let _ = &mut put;
    for (path, b) in &p.rec.closed_all {
        if path.is_empty() {
            rec = Some(b.as_slice());
        } else if path.ends_with("m.gdlp") {
            gdlp = Some(b.as_slice());
        } else {
            gdlp2 = Some(b.as_slice());
        }
    }
    if let Some(f) = p.rec.file.as_deref() {
        if p.rec.gdlp {
            if p.rec.gdlp_path.ends_with("m.gdlp") {
                gdlp = Some(f);
            } else {
                gdlp2 = Some(f);
            }
        } else {
            rec = Some(f);
        }
    }
    (rec, gdlp, gdlp2)
}

fn split_rec(p: &Player) -> (Option<&[u8]>, Option<&[u8]>) {
    let (r, g, _) = split_rec3(p);
    (r, g)
}

fn fnv_buf(b: Option<&[u8]>) -> (i64, u64) {
    match b {
        None => (-1, 0),
        Some(b) => {
            let mut h = Fnv::new();
            h.bytes(b);
            (b.len() as i64, h.0)
        }
    }
}

struct Out {
    s: String,
    line_hash: Fnv,
    line_count: i64,
    logs_done: usize,
}

impl Out {
    fn flush_logs(&mut self, p: &Player) {
        while self.logs_done < p.log.len() {
            writeln!(self.s, "LOG {}", p.log[self.logs_done]).unwrap();
            self.logs_done += 1;
        }
    }
    fn ln(&mut self, l: String) {
        self.line_hash.bytes(l.as_bytes());
        self.line_hash.bytes(&[10]);
        self.line_count += 1;
    }
    fn say(&mut self, l: String) {
        self.s.push_str(&l);
        self.s.push('\n');
    }
}

fn dump_files(o: &mut Out, p: &Player) {
    // the recording as it stands (open or closed): a plain .slp or the whole-match .gdlp
    let (rec, gdlp, gdlp2) = split_rec3(p);
    let rows = [
        ("trace", p.trace.as_deref()),
        ("vel", p.vel_trace.as_deref()),
        ("seed", p.seed_csv.as_deref()),
        ("rec", rec),
        ("gdlp", gdlp),
        ("gdlp2", gdlp2),
    ];
    for (tag, b) in rows {
        let (len, h) = fnv_buf(b);
        o.say(format!("FILE {tag} {len} {h:016x}"));
    }
}

/// The Rust twin of player_oracle.c's main.
pub fn run_case(slp_path: &str, data: &[u8], flags: &str) -> String {
    let env = Env {
        resync: has_flag(flags, "resync"),
        parity: has_flag(flags, "parity"),
        no_ucf073: has_flag(flags, "noucf073"),
        slippi_codes: flag_val(flags, "codes").map(|v| v[..1].to_string()),
        frozen_ps: has_flag(flags, "frozen"),
        deterministic: flag_val(flags, "det").map(|v| v[..1].to_string()),
        ucf: flag_val(flags, "ucf").map(|v| v.split(',').next().unwrap().to_string()),
        ucf_setting: 84,
        slippi_local_port: flag_int(flags, "local", -1) as i32,
        match_turbo_rules: 0x5FB,
        stage_skin: if has_flag(flags, "skin") { Some(("Frosty Skin".into(), 0x123456789abc)) } else { None },
        gdlp_record: if has_flag(flags, "gdlp") {
            Some("./m.gdlp".to_string())
        } else if has_flag(flags, "gdlp0") {
            Some("0".to_string())
        } else {
            None
        },
        gdlp_record_setting: has_flag(flags, "gset") as i32,
        identity: Some(Identity {
            id: [
                0x1111222233334444,
                0x5555666677778888,
                0x99990000aaaa1111 ^ (flag_int(flags, "idx", 0) == 2) as u64,
                0xdeadbeefcafef00d,
            ],
            geno_version: flag_int(flags, "geno", 9) as u32,
        }),
        mods_describe: flag_val(flags, "mods").map(|v| v.split(',').next().unwrap().to_string()).unwrap_or_default(),
        time: 1700000000,
        rec_dir: ".\\geno-lab_lab\\recordings".into(),
    };
    let mut p = Player::new(env);
    p.resimulating = has_flag(flags, "resim");
    let mut o = Out { s: String::new(), line_hash: Fnv::new(), line_count: 0, logs_done: 0 };
    if has_flag(flags, "play_gdlp") {
        p.load_gdlp(slp_path, data, has_flag(flags, "trace"));
    } else {
        p.load_slp(slp_path, data, has_flag(flags, "trace"));
    }
    if has_flag(flags, "live") {
        p.arm_live(true);
    }
    o.flush_logs(&p);
    o.say(format!("ACTIVE {}", p.active() as i32));
    o.say(format!("SCENE {}", p.scene().unwrap_or("(null)")));
    if has_flag(flags, "rec") {
        p.start_recording("./rec.slp");
        o.flush_logs(&p);
        o.say(format!("RECORDING {}", p.recording() as i32));
    }
    let en = p.enabled();
    o.say(format!("ENABLED {}", en as i32));
    let c_codes = p.slippi_codes();
    let c_ver = p.slippi_version();
    let c_frozen = p.slippi_frozen_stadium() as i32;
    let c_det = p.det_enabled() as i32;
    let c_last = p.last_frame();
    let c_live = p.is_live() as i32;
    let c_raw = p.controls_raw() as i32;
    o.flush_logs(&p);
    o.say(format!(
        "CODES {c_codes} VER {c_ver} FROZEN {c_frozen} DET {c_det} LAST {c_last} LIVE {c_live} CTRLRAW {c_raw} TURBO {:08x}",
        p.turbo_rules()
    ));
    for port in 0..4 {
        o.say(format!("HUMAN {port} {}", p.port_human(port) as i32));
    }
    let mut mbuf = vec![0u8; 0x138 + 16];
    for (i, b) in mbuf.iter_mut().enumerate() {
        *b = (i * 7 + 3) as u8;
    }
    for q in 0..4usize {
        mbuf[0x60 + 0x24 * q + 1] = if q == 2 { 3 } else if q == 3 { 1 } else { 0 };
        mbuf[0x60 + 0x24 * q + 7] = q as u8;
        mbuf[0x60 + 0x24 * q] = (q + 8) as u8;
    }
    let seed = p.apply_match(&mut mbuf, &mut |d| {
        let mut i = 0;
        while i < 0x138 {
            d[i] ^= 0x5A;
            i += 17;
        }
        0xA5A5C3C3
    });
    o.flush_logs(&p);
    let mut h = Fnv::new();
    h.bytes(&mbuf);
    o.say(format!("APPLY seed={seed:08x} buf={:016x} frame={}", h.0, p.frame()));
    p.match_begin(&mbuf, &|port, ck| ((port * 0x11 + ck * 3) & 0xff3d) as u32, &|pl| (pl * 5) as u32);
    o.flush_logs(&p);
    let mc = p.match_controls;
    o.say(format!("MC {:08x} {:08x} {:08x} {:08x}", mc[0], mc[1], mc[2], mc[3]));
    p.record_match(&mbuf, seed);
    o.flush_logs(&p);
    {
        let fi = p.slippi_fixture_info();
        let mut l = format!("FIXTURE {} {}", fi.is_some() as i32, p.fixture_reason());
        if let Some(fi) = fi {
            let v = fi.version;
            write!(l, " {}.{}.{}.{} {}..{} seed={:08x}", v[0], v[1], v[2], v[3], fi.first_frame, fi.last_frame, fi.seed).unwrap();
        }
        o.say(l);
    }
    p.ucf_pad_latch(1, 10, -20, 30, -40);
    p.ucf_pad_latch(1, 11, -21, 31, -41);
    for w in 0..4 {
        let a = p.raw_stick(1, w);
        let b = p.raw_stick_back(1, w, 1);
        o.flush_logs(&p);
        o.say(format!("OFFRAW {w} {a} {b}"));
    }
    let limit = if p.active { (p.r.last as i64 - p.r.first as i64) + 3 } else { 6 };
    let badseed = flag_int(flags, "badseed", -100000);
    let badpost = flag_int(flags, "badpost", -100000);
    let maxframes = flag_int(flags, "frames", 100000);
    let limit = limit.min(maxframes);
    'frames: for _probe in 0..limit {
        let f = p.tick();
        o.flush_logs(&p);
        o.ln(format!("T {f}"));
        if p.exit.is_some() {
            break;
        }
        if f == UNARMED {
            break;
        }
        let rs = p.resync_seed();
        let cs = p.checksum_seed(0x1234, f);
        o.ln(format!("RS {rs:08x} CS {cs:08x}"));
        let mut arrived = if rs != 0 { rs } else { 0x80000000u32.wrapping_add((f as u32).wrapping_mul(0x9E3779B1)) };
        if f as i64 == badseed {
            arrived ^= 1;
        }
        p.note_seed(arrived);
        p.check_seed(arrived);
        o.flush_logs(&p);
        if p.exit.is_some() {
            break 'frames;
        }
        for port in 0..4 {
            for fol in [false, true] {
                if p.has_input(port, fol) {
                    let i = *p.input(port, fol).unwrap();
                    o.ln(format!(
                        "IN {port} {} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                        fol as i32,
                        fb(i.lx),
                        fb(i.ly),
                        fb(i.cx),
                        fb(i.cy),
                        fb(i.trigger),
                        i.buttons
                    ));
                    p.record_input(
                        port,
                        fol,
                        i.lx,
                        i.ly,
                        i.cx,
                        i.cy,
                        i.trigger,
                        i.buttons,
                        port * 1000 + (f & 0xfff),
                        f as f32 * 0.25 + port as f32,
                        port as f32 - f as f32 * 0.125,
                        if fol { -1.0 } else { 1.0 },
                        (f & 0xff) as f32 * 0.5,
                    );
                    o.flush_logs(&p);
                }
            }
        }
        if p.is_live() {
            for port in 0..2 {
                p.record_input(port, false, 0.5 * port as f32, -0.25, 0.125, 1.0, 0.75, 0x100 + port as u32, 0x20 + port, f as f32, (-port) as f32, 1.0, (f & 0x3f) as f32);
                o.flush_logs(&p);
            }
        }
        for port in 0..4 {
            let card = p.ucf_cardinals(port) as i32;
            let ver = p.ucf_version(port);
            let mut ub = format!("U {port} card={card} ver={ver} raw=");
            for w in 0..4 {
                write!(ub, "{}", p.raw_stick(port, w)).unwrap();
                for back in 0..3 {
                    write!(ub, "/{}", p.raw_stick_back(port, w, back)).unwrap();
                }
                ub.push(' ');
            }
            o.flush_logs(&p);
            o.ln(ub);
        }
        for slot in 0..8 {
            match p.peek_input(slot, f) {
                Some(i) => o.ln(format!(
                    "PK {slot} {:08x} {:08x} {:08x} {} {} {} {} conf=1",
                    i.buttons,
                    i.seed,
                    fb(i.lx),
                    i.raw[0],
                    i.raw[1],
                    i.raw[2],
                    i.raw[3]
                )),
                None => o.ln(format!("PK {slot} none")),
            }
        }
        if p.r.post.is_some() && f >= p.r.first && f <= p.r.last {
            for i in 0..SLOTS {
                let q = p.r.post.as_ref().unwrap()[(f - p.r.first) as usize * SLOTS + i];
                if q.has {
                    let mut x = q.x;
                    if f as i64 == badpost && i == 0 {
                        x += 1.0;
                    }
                    p.trace_fighter(
                        (i / 2) as i32,
                        i & 1 != 0,
                        (i / 2 + 1) as i32,
                        q.action as i32,
                        x,
                        q.y,
                        q.facing,
                        q.percent,
                        q.stocks as i32,
                        q.x * 0.5,
                        q.y * 0.25,
                        -q.x,
                        q.y + 1.0,
                        3.0,
                    );
                    o.flush_logs(&p);
                    if p.exit.is_some() {
                        break 'frames;
                    }
                }
            }
        } else if p.active {
            p.trace_fighter(0, false, 1, 0x20, 1.5, -2.25, 1.0, 12.0, 4, 0.1, 0.2, 0.3, 0.4, 5.0);
            o.flush_logs(&p);
            if p.exit.is_some() {
                break 'frames;
            }
        }
        o.ln(format!("M tracing={} procmap={} frame={}", p.tracing() as i32, p.trace_at_proc_map() as i32, p.frame()));
        let c = p.get_cursor();
        o.ln(format!("CUR {} {} {} {:08x}", c[0], c[1], c[2], c[3] as u32));
        p.set_cursor(c);
    }
    o.flush_logs(&p);
    if let Some(_rc) = p.exit {
        // the C leaves through _exit inside gw_hang_final's caller: its stub printed the files
        dump_files(&mut o, &p);
        dump_debug(&p);
        return o.s;
    }
    p.match_record_end();
    o.flush_logs(&p);
    for port in 0..2 {
        let fr = p.r.first + 5;
        match p.slippi_pad(port, fr) {
            Some(pad) => {
                let b = pad.bytes;
                o.say(format!("SPAD {port} {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]));
            }
            None => o.say(format!("SPAD {port} none")),
        }
    }
    let fin = p.finish_recording();
    o.flush_logs(&p);
    o.say(format!("FINISH {}", fin as i32));
    let (n, hh) = (o.line_count, o.line_hash.0);
    o.say(format!("LINES {n} {hh:016x}"));
    dump_files(&mut o, &p);
    dump_debug(&p);
    o.s
}

/// PLAYER_DUMP=<dir> writes the Rust-side files of the last case for diffing against the C's.
fn dump_debug(p: &Player) {
    if let Ok(dir) = std::env::var("PLAYER_DUMP") {
        let (rec, gdlp) = split_rec(p);
        for (n, b) in [("trace.csv", p.trace.as_deref()), ("trace.csv.vel.csv", p.vel_trace.as_deref()), ("trace.csv.seed.csv", p.seed_csv.as_deref()), ("rec.slp", rec), ("m.gdlp", gdlp)] {
            if let Some(b) = b {
                std::fs::write(format!("{dir}/{n}"), b).unwrap();
            }
        }
    }
}

fn read_cases() -> Vec<(char, String, String)> {
    include_str!("data/player_cases.txt")
        .lines()
        .map(|l| {
            let mut it = l.splitn(3, '|');
            let b = it.next().unwrap().chars().next().unwrap();
            (b, it.next().unwrap().to_string(), it.next().unwrap_or("").to_string())
        })
        .collect()
}

fn unix(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn check(real: bool) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let synth = manifest.join("tests/data/slp");
    let realroot = std::env::var("REAL_SLP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest.join("../../../../Gang-Steals"));
    if real && !realroot.is_dir() {
        eprintln!("skipped: {} not found", realroot.display());
        return;
    }
    let realroot = if real { std::fs::canonicalize(&realroot).unwrap() } else { realroot };
    let want_all = include_str!("data/player_oracle.txt");
    let mut want: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    for block in want_all.split("== ").skip(1) {
        let (head, body) = block.split_once('\n').unwrap();
        want.insert(head, body);
    }
    let mut n = 0;
    for (b, name, flags) in read_cases() {
        if (b == 'r') != real {
            continue;
        }
        let base = if b == 'g' {
            unix(&std::fs::canonicalize(manifest.join("tests/data/gdlp")).unwrap()).trim_start_matches("//?/").to_string()
        } else if b == 'r' { unix(&realroot).trim_start_matches("//?/").to_string() } else { unix(&std::fs::canonicalize(&synth).unwrap()).trim_start_matches("//?/").to_string() };
        let path = format!("{base}/{name}");
        let data = std::fs::read(&path).unwrap();
        let got = run_case(&path, &data, &flags).replace(&base, "<S>");
        let key = format!("{b}|{name}|{flags}");
        if let Ok(only) = std::env::var("PLAYER_ONLY") {
            if !key.contains(&only) {
                continue;
            }
        }
        let w = want.get(key.as_str()).unwrap_or_else(|| panic!("no oracle block for {key}"));
        if *w != got {
            let wl: Vec<&str> = w.lines().collect();
            let gl: Vec<&str> = got.lines().collect();
            for i in 0..wl.len().max(gl.len()) {
                if wl.get(i) != gl.get(i) {
                    panic!("{key}: line {} differs\n  C:    {:?}\n  Rust: {:?}", i + 1, wl.get(i), gl.get(i));
                }
            }
        }
        n += 1;
    }
    assert!(n > 0);
    eprintln!("{n} cases identical");
}

#[test]
fn synthetic_cases_match_the_c() {
    check(false);
}

#[test]
fn real_cases_match_the_c() {
    check(true);
}

#[test]
fn first_frame() {
    assert_eq!(FIRST_FRAME, -123);
}

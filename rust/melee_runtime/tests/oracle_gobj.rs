//! The GObj system (`melee_runtime::gobj`) against the C (gobj.c, gobjplink.c, gobjgxlink.c,
//! gobjproc.c, gobjobject.c, gobjuserdata.c, gobjinit.c). `oracle/build_gobj.sh` runs
//! `oracle/driver_gobj.c`, a seeded random operation generator, against the C TUs compiled with
//! the shipping flags and writes `tests/data/oracle_gobj.txt`: every callback the C made (proc
//! invokes, renders, removal functions, render-random begin/end) and, after every operation, the
//! FNV-1a 64 hash of a full state dump (every p_link / gx_link list both directions, every proc
//! queue with prev links and flags, the ProcList table, every live gobj's fields and proc chain,
//! the phase and delayed-request block). This test runs the same generator (a line-for-line
//! mirror of the driver) against the Rust and requires the identical transcript.
use std::collections::HashMap;
use std::fmt::Write;

use melee_runtime::gobj::*;

const DATA: &str = include_str!("data/oracle_gobj.txt");

type Sys = GObjSystem<u32, u32>;

struct Drv {
    st: u64,
    pm: u32,
    gm: u32,
    sm: u32,
    g: Vec<GObjId>,
    gser: HashMap<GObjId, u32>,
    pser: HashMap<ProcId, u32>,
    ngs: u32,
    nps: u32,
    out: Vec<String>,
}

impl Drv {
    fn nx(&mut self) -> u64 {
        self.st ^= self.st << 13;
        self.st ^= self.st >> 7;
        self.st ^= self.st << 17;
        self.st
    }
    fn rn(&mut self, n: u32) -> u32 {
        ((self.nx() >> 33) as u32) % n
    }
    fn gs(&self, g: Option<GObjId>) -> u32 {
        g.map_or(0, |g| self.gser[&g])
    }
    fn ps(&self, p: Option<ProcId>) -> u32 {
        p.map_or(0, |p| self.pser[&p])
    }
    fn pick(&mut self) -> Option<GObjId> {
        if self.g.is_empty() {
            None
        } else {
            let n = self.g.len() as u32;
            let k = self.rn(n) as usize;
            Some(self.g[k])
        }
    }
    fn pos_for(&mut self, sys: &Sys, where_: &mut u32, pl: u8, ex: Option<GObjId>) -> Option<GObjId> {
        if *where_ < 2 {
            return None;
        }
        let c: Vec<GObjId> =
            self.g.iter().copied().filter(|&x| sys.gobj(x).p_link == pl && Some(x) != ex).collect();
        let n = c.len() as u32;
        if *where_ == 2 {
            let k = self.rn(n + 1);
            return if k < n { Some(c[k as usize]) } else { None };
        }
        if n == 0 {
            *where_ = 0;
            return None;
        }
        Some(c[self.rn(n) as usize])
    }
    fn create(&mut self, sys: &mut Sys, incb: bool) -> Option<GObjId> {
        let mut where_ = if incb { self.rn(2) } else { self.rn(4) };
        let cls = self.rn(8) as u16;
        let pl = self.rn(self.pm + 1) as u8;
        let pr = self.rn(6) as u8;
        let pos = self.pos_for(sys, &mut where_, pl, None);
        let g = sys.create_gobj(self, where_ as i32, cls, pl, pr, pos);
        if let Some(g) = g {
            self.ngs += 1;
            self.gser.insert(g, self.ngs);
            self.g.push(g);
        }
        let s = self.gs(g);
        self.out.push(format!("c {s}"));
        g
    }
    fn setup_proc(&mut self, sys: &mut Sys, g: GObjId) {
        let t = self.rn(4);
        let pri = self.rn(self.sm + 1) as u8;
        let p = sys.setup_proc(g, t, pri);
        self.nps += 1;
        self.pser.insert(p, self.nps);
        self.out.push(format!("s {}", self.nps));
    }
    fn change_pri(&mut self, sys: &mut Sys, g: GObjId) {
        let mut t = self.rn(4);
        let pl = self.rn(self.pm + 1) as u8;
        let pr = self.rn(6) as u8;
        let pos = self.pos_for(sys, &mut t, pl, Some(g));
        sys.change_gobj_pri(t, g, pl, pr, pos);
    }
    fn gx_op(&mut self, sys: &mut Sys, g: GObjId) {
        let v = self.rn(3);
        if sys.gobj(g).gx_link == HSD_GOBJ_GXLINK_NONE {
            let r = self.rn(2);
            if v == 0 {
                let l = self.rn(self.gm + 1) as u8;
                let p = self.rn(6);
                sys.setup_gx_link(g, Some(r), l, p);
            } else if v == 1 {
                let p = self.rn(6);
                sys.setup_gx_link_max(g, Some(r), p);
            } else {
                let p = self.rn(6);
                sys.setup_gx_link_max_sorted(g, Some(r), p);
            }
        } else if v == 0 {
            sys.gxlink_8039084c(g);
        } else if v == 1 {
            let l = self.rn(self.gm + 1) as u8;
            let p = self.rn(6) as u8;
            sys.gxlink_80390908(g, l, p);
        } else {
            let c: Vec<GObjId> = self
                .g
                .iter()
                .copied()
                .filter(|&x| x != g && sys.gobj(x).gx_link != HSD_GOBJ_GXLINK_NONE)
                .collect();
            if !c.is_empty() {
                let o = c[self.rn(c.len() as u32) as usize];
                sys.gxlink_803909d8(g, o);
            } else {
                sys.gxlink_8039084c(g);
            }
        }
    }
    fn flag_op(&mut self, sys: &mut Sys, g: GObjId) {
        match self.rn(4) {
            0 => sys.hsd_gobj_80390c5c(g),
            1 => sys.hsd_gobj_80390c84(g),
            2 => sys.hsd_gobj_80390cac(g),
            _ => sys.hsd_gobj_80390cd4(g),
        }
    }
    fn act(&mut self, sys: &mut Sys, g: GObjId, depth: u32) {
        match self.rn(12) {
            0 => sys.free(self, g),
            1 => {
                let p = sys.current_invoked_proc.unwrap();
                sys.remove_proc(p)
            }
            2 => self.change_pri(sys, g),
            3 => self.setup_proc(sys, g),
            4 => {
                let n = self.create(sys, true);
                if let Some(n) = n {
                    if self.rn(2) != 0 {
                        self.setup_proc(sys, n);
                    }
                }
            }
            5 => self.gx_op(sys, g),
            6 => self.flag_op(sys, g),
            _ => {}
        }
        if depth == 0 && self.rn(4) == 0 {
            self.act(sys, g, 1);
        }
    }
    fn obj_op(&mut self, sys: &mut Sys, g: GObjId) {
        match self.rn(6) {
            0 => {
                if sys.gobj(g).obj_kind == HSD_GOBJ_OBJ_NONE {
                    let k = self.rn(2) as u8;
                    let t = 1 + self.rn(1000);
                    sys.gobjobject_80390a70(g, k, Some(t));
                }
            }
            1 => {
                let t = sys.gobjobject_80390adc(g).unwrap_or(0);
                self.out.push(format!("t {t}"));
            }
            2 => sys.gobjobject_80390b0c(self, g),
            3 => {
                if sys.gobj(g).user_data_kind == HSD_GOBJ_USER_DATA_NONE {
                    let k = self.rn(3) as u8;
                    let f = self.rn(2);
                    let t = 1 + self.rn(1000);
                    sys.init_user_data(g, k, Some(f), Some(t));
                }
            }
            4 => sys.remove_user_data(self, g),
            _ => {
                let c = self.rn(8) as u16;
                let l = self.rn(self.pm + 1) as u8;
                let f = sys.gobjobject_80390a3c(c, l);
                let s = self.gs(f);
                self.out.push(format!("f {s}"));
            }
        }
    }
    fn dump(&self, sys: &Sys) -> String {
        let mut d = String::new();
        for l in 0..=self.pm as usize {
            if sys.plink_head[l].is_none() && sys.plink_low[l].is_none() {
                continue;
            }
            write!(d, "p{l}:").unwrap();
            let mut g = sys.plink_head[l];
            while let Some(x) = g {
                write!(d, " {}", self.gser[&x]).unwrap();
                g = sys.gobj(x).next;
            }
            d.push_str(" |");
            let mut g = sys.plink_low[l];
            while let Some(x) = g {
                write!(d, " {}", self.gser[&x]).unwrap();
                g = sys.gobj(x).prev;
            }
            d.push('\n');
        }
        for l in 0..=self.gm as usize + 1 {
            if sys.gxlink_head[l].is_none() && sys.gxlink_low[l].is_none() {
                continue;
            }
            write!(d, "x{l}:").unwrap();
            let mut g = sys.gxlink_head[l];
            while let Some(x) = g {
                write!(d, " {}", self.gser[&x]).unwrap();
                g = sys.gobj(x).next_gx;
            }
            d.push_str(" |");
            let mut g = sys.gxlink_low[l];
            while let Some(x) = g {
                write!(d, " {}", self.gser[&x]).unwrap();
                g = sys.gobj(x).prev_gx;
            }
            d.push('\n');
        }
        for s in 0..=self.sm as usize {
            write!(d, "q{s}:").unwrap();
            let mut p = sys.proc_head[s];
            while let Some(x) = p {
                let pr = sys.proc_(x);
                write!(
                    d,
                    " {}<{}:{}{}{}{}/{}/{}",
                    self.pser[&x],
                    self.ps(pr.prev),
                    pr.flags_1,
                    pr.flags_2,
                    pr.flags_3,
                    pr.flags_4,
                    self.gser[&pr.gobj],
                    pr.on_invoke
                )
                .unwrap();
                p = pr.next;
            }
            d.push('\n');
        }
        d.push_str("L:");
        for e in &sys.proc_list {
            write!(d, " {}", self.ps(*e)).unwrap();
        }
        d.push('\n');
        for &x in &self.g {
            let g = sys.gobj(x);
            write!(
                d,
                "g {} {} {} {} {} {} {} {} {:x} {} {} {} {}",
                self.gser[&x],
                g.classifier,
                g.p_link,
                g.gx_link,
                g.p_priority,
                g.render_priority,
                g.obj_kind,
                g.user_data_kind,
                g.gxlink_prios,
                g.hsd_obj.unwrap_or(0),
                g.user_data.unwrap_or(0),
                g.render_cb.map_or(8, |f| f as i32),
                g.user_data_remove_func.map_or(8, |f| f as i32)
            )
            .unwrap();
            let mut p = g.proc_;
            while let Some(c) = p {
                write!(d, " {}", self.pser[&c]).unwrap();
                p = sys.proc_(c).child;
            }
            d.push('\n');
        }
        let dl = &sys.delayed;
        writeln!(
            d,
            "d {} {:x} {} {} {} {} {} {} {}",
            sys.phase,
            dl_flags_native(dl),
            dl.type_,
            dl.p_link,
            dl.p_prio,
            self.gs(sys.current_invoked_proc_gobj),
            self.ps(sys.current_invoked_proc),
            self.ps(sys.next_invoked_proc),
            sys.current_invoked_s_link
        )
        .unwrap();
        d
    }
}

/// The union word as the x86 oracle prints it (LSB-first bitfields).
fn dl_flags_native(d: &DelayedProcInfo) -> u32 {
    d.in_delayed_proc as u32
        | (d.delay_remove_gobj as u32) << 1
        | (d.delay_remove_proc as u32) << 2
        | (d.delay_change_gobj_pri as u32) << 3
}

fn fnv(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl GObjHost<u32, u32> for Drv {
    fn invoke(&mut self, sys: &mut Sys, gobj: GObjId, func: u32) {
        let line = format!("i {} {} {}", func, self.gser[&gobj], self.ps(sys.current_invoked_proc));
        self.out.push(line);
        self.act(sys, gobj, 0);
    }
    fn render(&mut self, sys: &mut Sys, gobj: GObjId, func: u32, pass: i32) {
        let line = format!(
            "r {} {} {} {} {}",
            func,
            self.gser[&gobj],
            pass,
            self.gs(sys.gobj_804d7814),
            self.gs(sys.gobj_804d7818)
        );
        self.out.push(line);
    }
    fn remove_user_data(&mut self, _sys: &mut Sys, func: u32, data: Option<u32>) {
        self.out.push(format!("u {} {}", func, data.unwrap_or(0)));
    }
    fn remove_obj(&mut self, _sys: &mut Sys, func: KindFunc<u32>, obj: Option<u32>) {
        match func {
            KindFunc::User(f) => self.out.push(format!("o {} {}", f, obj.unwrap_or(0))),
            other => panic!("built-in kind func {other:?} reached"),
        }
    }
    fn stage_slot_destroyed(&mut self, _sys: &mut Sys, gobj: GObjId) {
        self.g.retain(|&x| x != gobj);
    }
    fn rand_render_begin(&mut self) {
        self.out.push("rb".into());
    }
    fn rand_render_end(&mut self) {
        self.out.push("re".into());
    }
}

fn run() -> Vec<String> {
    let mut d = Drv { st: 0, pm: 0, gm: 0, sm: 0, g: vec![], gser: HashMap::new(), pser: HashMap::new(), ngs: 0, nps: 0, out: vec![] };
    let mut sys = Sys::new();
    for seed in 0..40u64 {
        d.st = 0x9E3779B97F4A7C15u64 ^ (seed + 1).wrapping_mul(0x2545F4914F6CDD1D);
        d.nx();
        let mut data: HsdGObjLibInitData<u32> = if seed % 8 == 0 {
            hsd_gobj_set_init_defaults()
        } else {
            let p = 1 + d.rn(5) as u8;
            let g = d.rn(4) as u8;
            let s = d.rn(3) as u8;
            HsdGObjLibInitData { p_link_max: p, gx_link_max: g, gproc_pri_max: s, funcs: vec![] }
        };
        data.funcs.push(vec![KindFunc::User(0), KindFunc::User(1)]);
        d.pm = data.p_link_max as u32;
        d.gm = data.gx_link_max as u32;
        d.sm = data.gproc_pri_max as u32;
        d.g.clear();
        d.gser.clear();
        d.pser.clear();
        d.ngs = 0;
        d.nps = 0;
        // one system for all seeds: like the C globals, HSD_GObjInit leaves some state alone
        sys.init(&mut data);
        d.out.push(format!(
            "S {} {} {} {} k {} {} {} {} e {} {} {} {}",
            seed,
            d.pm,
            d.gm,
            d.sm,
            sys.camera_kind,
            sys.light_kind,
            sys.jobj_kind,
            sys.fog_kind,
            hsd_gobj_80390eb8(0),
            hsd_gobj_80390eb8(1),
            hsd_gobj_80390eb8(2),
            hsd_gobj_80390eb8(3)
        ));
        for _ in 0..400 {
            let r = d.rn(20);
            d.out.push(format!("O {r}"));
            match r {
                0..=4 => {
                    d.create(&mut sys, false);
                }
                5 | 6 => {
                    if let Some(g) = d.pick() {
                        sys.free(&mut d, g)
                    }
                }
                7 => {
                    if let Some(g) = d.pick() {
                        d.change_pri(&mut sys, g)
                    }
                }
                8 | 9 => {
                    if let Some(g) = d.pick() {
                        d.setup_proc(&mut sys, g)
                    }
                }
                10 => {
                    if let Some(g) = d.pick() {
                        let mut c = vec![];
                        let mut p = sys.gobj(g).proc_;
                        while let Some(x) = p {
                            if c.len() >= 256 {
                                break;
                            }
                            c.push(x);
                            p = sys.proc_(x).child;
                        }
                        if !c.is_empty() {
                            let k = d.rn(c.len() as u32) as usize;
                            sys.remove_proc(c[k]);
                        }
                    }
                }
                11 => {
                    if let Some(g) = d.pick() {
                        sys.remove_all_procs(g)
                    }
                }
                12 | 13 => {
                    if let Some(g) = d.pick() {
                        d.gx_op(&mut sys, g)
                    }
                }
                14 => {
                    if let Some(g) = d.pick() {
                        d.flag_op(&mut sys, g)
                    }
                }
                15 | 16 => {
                    let m = d.rn(4);
                    sys.pause_mask = if m == 1 {
                        d.nx() & 2u64.wrapping_shl(d.pm).wrapping_sub(1)
                    } else {
                        0
                    };
                    sys.run_procs(&mut d);
                }
                17 => {
                    if let Some(g) = d.pick() {
                        let mask = d.rn(8);
                        let v = d.nx() & 2u64.wrapping_shl(d.gm + 1).wrapping_sub(1);
                        sys.gobj_mut(g).gxlink_prios = v;
                        sys.hsd_gobj_80390ed0(&mut d, g, mask);
                    }
                }
                18 => {
                    if let Some(g) = d.pick() {
                        d.obj_op(&mut sys, g)
                    }
                }
                _ => sys.hsd_gobj_80390fc0(&mut d),
            }
            let h = fnv(&d.dump(&sys));
            d.out.push(format!("h {h:016x}"));
            if std::env::var_os("GOBJ_DUMP").is_some() {
                eprint!("{}", d.dump(&sys));
            }
        }
    }
    d.out
}

#[test]
fn gobj_system_matches_c_build() {
    let got = run();
    let want: Vec<&str> = DATA.lines().collect();
    let mut hashes = 0;
    for (i, w) in want.iter().enumerate() {
        let g = got.get(i).map(String::as_str).unwrap_or("<end>");
        if g != *w {
            let from = i.saturating_sub(12);
            panic!(
                "transcript differs at line {}: got `{g}`, want `{w}`\ncontext (C):\n{}",
                i + 1,
                want[from..=i].join("\n")
            );
        }
        if w.starts_with('h') {
            hashes += 1;
        }
    }
    assert_eq!(got.len(), want.len(), "transcript length");
    assert_eq!(hashes, 16000);
    eprintln!("{} transcript lines equal ({} state hashes)", want.len(), hashes);
}

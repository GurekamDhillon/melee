//! TU state-oracle runtime, Rust side. Mirrors oracle/tuo/tuo_rt.c line for line (generator,
//! registry, log format); see there and oracle/tuo/gen_tuo.py. `run` replays every case of the
//! cfg file against the Rust port and compares the per-case log hash with the one the C
//! produced (tests/data/tuo_<tu>.txt). Set TUO_FULL=<fn> or <fn>:<idx> to print the full logs.
#![allow(dead_code, static_mut_refs)]

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::fmt::Write as _;

pub type VisFn = fn(&mut V, &str, *mut u8);

pub struct TypeInfo {
    pub name: &'static str,
    pub size: usize,
    pub align: usize,
    pub vis: VisFn,
}
pub struct Global {
    pub name: &'static str,
    pub p: *mut u8,
    pub size: usize,
    pub vis: Option<VisFn>,
    pub fill: bool,
}
pub struct Glue {
    pub types: Vec<TypeInfo>,
    pub globals: Vec<Global>,
    pub fns: Vec<(&'static str, usize)>,
    pub size_ok: &'static [&'static str],
    pub install: fn(),
    pub call: fn(&str, &[u64]) -> u64,
    pub expr: fn(&str) -> u64,
}

struct Dom {
    key: String,
    v: Vec<u64>,
}
struct View {
    key: String,
    member: String,
    fns: Vec<String>,
}
struct Link {
    key: String,
    tgt: String,
    gate: Vec<String>,
    force: Vec<String>,
}
struct FnCfg {
    name: String,
    n: usize,
    ret: String,
    args: Vec<String>,
}
struct Reg {
    name: String,
    base: usize,
    size: usize,
}
struct PoolObj {
    name: String,
    p: *mut u8,
    ty: Option<usize>,
    size: usize,
    cnt: usize,
    ptrs: usize,
    layout: Layout,
}

pub struct V {
    mode: u8,
    rs: u64,
    lh: u64,
    full: bool,
    out: Vec<String>,
    doms: Vec<Dom>,
    links: Vec<Link>,
    regs: Vec<Reg>,
    nperm: usize,
    fps: Vec<(String, usize)>,
    types: Vec<(String, usize, usize, VisFn)>,
    globals: Vec<(String, *mut u8, usize, Option<VisFn>, bool, Vec<u8>)>,
    pool: Vec<PoolObj>,
    npool: usize,
    cl: String,
    curfn: String,
    args: [u64; 12],
    argptr: [bool; 12],
    pending_arg: Option<usize>,
    views: Vec<View>,
    shallow: u32,
}

static mut VV: Option<V> = None;
pub fn vv() -> &'static mut V {
    unsafe { VV.as_mut().unwrap() }
}

fn fnv(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}

fn parse_u(s: &str) -> u64 {
    if let Some(h) = s.strip_prefix("f:") {
        return u64::from_str_radix(h, 16).unwrap();
    }
    if s.starts_with('-') {
        return s.parse::<i64>().unwrap() as u64;
    }
    if let Some(h) = s.strip_prefix("0x") {
        return u64::from_str_radix(h, 16).unwrap();
    }
    s.parse::<u64>().unwrap()
}

impl V {
    pub fn set(&self) -> bool {
        self.mode == 1
    }
    /// Is the running function one of "a|b|c" (FIELD_FNS).
    pub fn fn_in(&self, list: &str) -> bool {
        list.split('|').any(|f| f == self.curfn)
    }
    /// A union is visited through a member only for the functions its `view` line names (none named = all).
    pub fn view_on(&self, key: &str, member: &str) -> bool {
        match self.views.iter().find(|w| w.key == key && w.member == member) {
            None => true,
            Some(w) => {
                // "~fn": fill the member for fn but do not dump it (its bytes are rewritten through an aliasing view)
                w.fns.is_empty()
                    || w.fns.iter().any(|f| match f.strip_prefix('~') {
                        Some(g) => g == self.curfn && self.mode != 2,
                        None => *f == self.curfn,
                    })
            }
        }
    }
    pub fn draw(&mut self) -> u64 {
        self.rs = self.rs.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.rs;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn emit(&mut self, line: &str) {
        for b in line.bytes() {
            self.lh ^= b as u64;
            self.lh = self.lh.wrapping_mul(0x100000001B3);
        }
        self.lh ^= b'\n' as u64;
        self.lh = self.lh.wrapping_mul(0x100000001B3);
        if self.full {
            self.out.push(line.to_string());
        }
    }
    fn dom(&self, path: &str) -> Option<usize> {
        self.doms.iter().position(|d| path.ends_with(&d.key))
    }
    pub fn leaf_int(&mut self, path: &str, bits: u32, sgn: u32) -> u64 {
        let r = self.draw();
        if let Some(i) = self.dom(path) {
            let d = &self.doms[i];
            return d.v[(r % d.v.len() as u64) as usize];
        }
        let mut r = r;
        if bits < 64 {
            let m = (1u64 << bits) - 1;
            r &= m;
            if sgn != 0 && (r >> (bits - 1)) != 0 {
                r |= !m;
            }
        }
        r
    }
    pub fn leaf_f32(&mut self, path: &str) -> u32 {
        let r = self.draw();
        if let Some(i) = self.dom(path) {
            let d = &self.doms[i];
            return d.v[(r % d.v.len() as u64) as usize] as u32;
        }
        match r & 7 {
            0 => 0.0f32.to_bits(),
            1 => 1.0f32.to_bits(),
            2 => (-1.0f32).to_bits(),
            3 => 0.5f32.to_bits(),
            4 => (-0.5f32).to_bits(),
            _ => ((((r >> 8) % 2000001) as f64 / 1000.0 - 1000.0) as f32).to_bits(),
        }
    }
    pub fn leaf_f64(&mut self, path: &str) -> u64 {
        let r = self.draw();
        if let Some(i) = self.dom(path) {
            let d = &self.doms[i];
            return d.v[(r % d.v.len() as u64) as usize];
        }
        match r & 7 {
            0 => 0.0f64.to_bits(),
            1 => 1.0f64.to_bits(),
            2 => (-1.0f64).to_bits(),
            3 => 0.5f64.to_bits(),
            4 => (-0.5f64).to_bits(),
            _ => (((r >> 8) % 2000001) as f64 / 1000.0 - 1000.0).to_bits(),
        }
    }
    fn mk_target(&mut self, path: &str, tgt: &str) -> usize {
        if tgt == "null" {
            return 0;
        }
        if let Some(n) = tgt.strip_prefix("fn:") {
            return self.fps.iter().find(|f| f.0 == n).unwrap_or_else(|| panic!("no function {n}")).1;
        }
        if let Some(n) = tgt.strip_prefix("maybebuf:") {
            return if self.draw() & 1 != 0 { 0 } else { self.new_buf(None, n.parse().unwrap()) as usize };
        }
        if let Some(n) = tgt.strip_prefix("mayarg:") {
            // mayarg:N:Type = the case's pointer argument N, else (N is not a pointer argument) a new Type
            let (n, ty) = n.split_once(':').unwrap();
            let n: usize = n.parse().unwrap();
            if self.argptr[n] {
                return if self.draw() & 1 != 0 { self.args[n] as usize } else { 0 };
            }
            if ty == "null" {
                return 0;
            }
            return if self.draw() & 1 != 0 { 0 } else { self.mk_target(path, &format!("new:{ty}")) };
        }
        if let Some(g) = tgt.strip_prefix('&') {
            for gl in &self.globals {
                if gl.0 == g {
                    return gl.1 as usize;
                }
            }
            panic!("no global {g}");
        }
        if let Some(n) = tgt.strip_prefix("buf:") {
            return self.new_buf(None, n.parse().unwrap()) as usize;
        }
        if let Some(n) = tgt.strip_prefix("cell:") {
            let bits: usize = n.parse().unwrap();
            let cp = self.new_buf(None, 8);
            let cv = self.leaf_int(path, bits as u32, 1);
            unsafe { std::ptr::copy_nonoverlapping(cv.to_le_bytes().as_ptr(), cp, bits / 8) };
            return cp as usize;
        }
        if let Some(t) = tgt.strip_prefix("ptrs:") {
            let (ty, n) = t.split_once(':').unwrap();
            let n: usize = n.parse().unwrap();
            let layout = Layout::from_size_align(n * 8 + 16, 16).unwrap();
            let arr = unsafe { alloc_zeroed(layout) } as *mut usize;
            let nm = format!("p{}", self.npool);
            self.npool += 1;
            self.reg(&nm, arr as usize, n * 8);
            self.pool.push(PoolObj { name: nm, p: arr as *mut u8, ty: None, size: n * 8, cnt: 0, ptrs: n, layout });
            for i in 0..n {
                let e = self.new_typed(None, ty, 0) as usize;
                unsafe { *arr.add(i) = e };
            }
            return arr as usize;
        }
        if let Some(t) = tgt.strip_prefix("shallow:") {
            self.shallow += 1;
            let r = self.new_typed(None, t, 0) as usize;
            self.shallow -= 1;
            return r;
        }
        if let Some(t) = tgt.strip_prefix("new:") {
            let (ty, cnt) = match t.split_once(':') {
                Some((a, b)) => (a, b.parse::<usize>().unwrap()),
                None => (t, 0),
            };
            return self.new_typed(None, ty, cnt) as usize;
        }
        panic!("bad link target {tgt}");
    }
    pub fn leaf_ptr(&mut self, path: &str) -> usize {
        if self.shallow > 0 && path.ends_with(".user_data") {
            return 0;
        }
        let i = match self.links.iter().position(|l| path.ends_with(&l.key)) {
            Some(i) => i,
            None => return 0,
        };
        let tgt = self.links[i].tgt.clone();
        if !self.links[i].gate.is_empty() && !self.links[i].gate.iter().any(|g| *g == self.curfn) {
            return 0;
        }
        if let Some(t) = tgt.strip_prefix("maybe:") {
            let forced = self.links[i].force.iter().any(|g| *g == self.curfn);
            if !forced && self.draw() & 1 != 0 {
                return 0;
            }
            return self.mk_target(path, &format!("new:{t}"));
        }
        self.mk_target(path, &tgt)
    }
    pub fn dump_i(&mut self, path: &str, v: i64) {
        self.emit(&format!("{path}={v}"));
    }
    pub fn dump_u(&mut self, path: &str, v: u64) {
        self.emit(&format!("{path}={v}"));
    }
    pub fn dump_f32(&mut self, path: &str, v: u32) {
        self.emit(&format!("{path}=f{v:08x}"));
    }
    pub fn dump_f64(&mut self, path: &str, v: u64) {
        self.emit(&format!("{path}=f{v:016x}"));
    }
    fn pstr(&self, a: usize) -> String {
        if a == 0 {
            return "null".into();
        }
        for r in self.regs.iter().rev() {
            if a >= r.base && a < r.base + r.size {
                return if a == r.base { r.name.clone() } else { format!("{}+{:x}", r.name, a - r.base) };
            }
        }
        for (n, f) in &self.fps {
            if *f == a {
                return format!("fn:{n}");
            }
        }
        "X".into()
    }
    pub fn dump_p(&mut self, path: &str, p: usize) {
        let s = self.pstr(p);
        self.emit(&format!("{path}=@{s}"));
    }
    fn dump_bytes(&mut self, path: &str, p: *const u8, n: usize) {
        let mut s = String::new();
        let mut k = 0usize;
        let mut i = 0;
        while i < n && k + 3 < 65536 {
            let b = unsafe { *p.add(i) };
            write!(s, "{b:02x}").unwrap();
            k += 2;
            i += 1;
        }
        self.emit(&format!("{path}=x{s}"));
    }
    pub fn bytes(&mut self, path: &str, p: *mut u8, n: usize) {
        if self.mode == 1 {
            for i in 0..n {
                let d = self.draw();
                unsafe { *p.add(i) = d as u8 };
            }
        } else {
            self.dump_bytes(path, p, n);
        }
    }

    // ---- stub call logging ----
    pub fn call_begin(&mut self, name: &str) {
        self.cl = format!("call {name}(");
    }
    fn sep(&mut self) {
        if !self.cl.ends_with('(') {
            self.cl.push(',');
        }
    }
    pub fn arg_i(&mut self, v: i64) {
        self.sep();
        write!(self.cl, "{v}").unwrap();
    }
    pub fn arg_u(&mut self, v: u64) {
        self.sep();
        write!(self.cl, "{v}").unwrap();
    }
    pub fn arg_f32(&mut self, b: u32) {
        self.sep();
        write!(self.cl, "f{b:08x}").unwrap();
    }
    pub fn arg_f64(&mut self, b: u64) {
        self.sep();
        write!(self.cl, "f{b:016x}").unwrap();
    }
    pub fn arg_p(&mut self, p: usize) {
        self.sep();
        let s = self.pstr(p);
        write!(self.cl, "@{s}").unwrap();
    }
    pub fn arg_s(&mut self, s: *const u8) {
        self.sep();
        if s.is_null() {
            self.cl.push_str("\"\"null");
            return;
        }
        let mut t = String::new();
        let mut i = 0;
        unsafe {
            while i < 200 && *s.add(i) != 0 {
                t.push(*s.add(i) as char);
                i += 1;
            }
        }
        write!(self.cl, "\"{t}\"").unwrap();
    }
    pub fn arg_bytes(&mut self, p: *const u8, n: usize) {
        self.sep();
        self.cl.push('x');
        for i in 0..n {
            write!(self.cl, "{:02x}", unsafe { *p.add(i) }).unwrap();
        }
    }
    pub fn call_end(&mut self) {
        self.cl.push(')');
        let l = std::mem::take(&mut self.cl);
        self.emit(&l);
    }
    pub fn ret_int(&mut self, name: &str, bits: u32, sgn: u32) -> u64 {
        let r = self.leaf_int(&format!("ret:{name}"), bits, sgn);
        if std::env::var("TUO_TRACE").is_ok() {
            eprintln!("ret {name} = {}", r as i64);
        }
        r
    }
    pub fn ret_f32(&mut self, name: &str) -> u32 {
        self.leaf_f32(&format!("ret:{name}"))
    }
    pub fn ret_f64(&mut self, name: &str) -> u64 {
        self.leaf_f64(&format!("ret:{name}"))
    }
    pub fn ret_ptr(&mut self, name: &str) -> usize {
        self.leaf_ptr(&format!("ret:{name}"))
    }

    // ---- pool ----
    fn reg(&mut self, name: &str, p: usize, n: usize) {
        self.regs.push(Reg { name: name.to_string(), base: p, size: n.max(1) });
    }
    pub fn new_typed(&mut self, label: Option<&str>, ty: &str, count: usize) -> *mut u8 {
        let ti = self.types.iter().position(|t| t.0 == ty).unwrap_or_else(|| panic!("no type {ty}"));
        let (size, align, vis) = (self.types[ti].1, self.types[ti].2, self.types[ti].3);
        let sz = size * count.max(1);
        let layout = Layout::from_size_align(sz + 16, align.max(16)).unwrap();
        let p = unsafe { alloc_zeroed(layout) };
        let nm = match label {
            Some(l) => l.to_string(),
            None => {
                let n = format!("p{}", self.npool);
                self.npool += 1;
                n
            }
        };
        self.reg(&nm, p as usize, sz);
        if let Some(i) = self.pending_arg.take() {
            self.args[i] = p as usize as u64;
            self.argptr[i] = true;
        }
        self.pool.push(PoolObj { name: nm.clone(), p, ty: Some(ti), size: sz, cnt: count, ptrs: 0, layout });
        let m = self.mode;
        self.mode = 1;
        for i in 0..count.max(1) {
            let path = if count != 0 { format!("{nm}[{i}]") } else { nm.clone() };
            vis(self, &path, unsafe { p.add(i * size) });
        }
        self.mode = m;
        p
    }
    pub fn new_buf(&mut self, label: Option<&str>, n: usize) -> *mut u8 {
        let layout = Layout::from_size_align(n + 16, 16).unwrap();
        let p = unsafe { alloc_zeroed(layout) };
        let nm = match label {
            Some(l) => l.to_string(),
            None => {
                let n = format!("p{}", self.npool);
                self.npool += 1;
                n
            }
        };
        self.reg(&nm, p as usize, n);
        self.pool.push(PoolObj { name: nm, p, ty: None, size: n, cnt: 0, ptrs: 0, layout });
        for i in 0..n {
            let d = self.draw();
            unsafe { *p.add(i) = d as u8 };
        }
        p
    }
    fn dump_all(&mut self) {
        let v = self;
    for i in 0..v.globals.len() {
        let path = format!("g:{}", v.globals[i].0);
        let (p, size, vis) = (v.globals[i].1, v.globals[i].2, v.globals[i].3);
        match vis {
            Some(vis) => vis(v, &path, p),
            None => v.dump_bytes(&path, p, size),
        }
    }
    for i in 0..v.pool.len() {
        let (name, p, ty, size, cnt) = (v.pool[i].name.clone(), v.pool[i].p, v.pool[i].ty, v.pool[i].size, v.pool[i].cnt);
        match ty {
            Some(ti) => {
                let (tsz, vis) = (v.types[ti].1, v.types[ti].3);
                for k in 0..cnt.max(1) {
                    let path = if cnt != 0 { format!("{name}[{k}]") } else { name.clone() };
                    vis(v, &path, unsafe { p.add(k * tsz) });
                }
            }
            None if v.pool[i].ptrs != 0 => {
                for k in 0..v.pool[i].ptrs {
                    let e = unsafe { *(p as *const usize).add(k) };
                    let s = v.pstr(e);
                    v.emit(&format!("{name}[{k}]=@{s}"));
                }
            }
            None => v.dump_bytes(&name, p, size),
        }
    }

    }
    fn pool_reset(&mut self) {
        for o in self.pool.drain(..) {
            unsafe { dealloc(o.p, o.layout) };
        }
        self.npool = 0;
        self.regs.truncate(self.nperm);
    }
}

pub fn run(cfg: &str, expected: &str, glue: Glue) {
    let mut v = V {
        mode: 0,
        rs: 0,
        lh: 0,
        full: false,
        out: vec![],
        doms: vec![],
        links: vec![],
        regs: vec![],
        nperm: 0,
        fps: vec![],
        types: vec![],
        globals: vec![],
        pool: vec![],
        npool: 0,
        cl: String::new(),
        curfn: String::new(),
        args: [0; 12],
        argptr: [false; 12],
        pending_arg: None,
        views: vec![],
        shallow: 0,
    };
    let mut fns: Vec<FnCfg> = vec![];
    let mut tysizes: Vec<(String, usize)> = vec![];
    for line in cfg.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        match t[0] {
            "dom" => v.doms.push(Dom { key: t[1].to_string(), v: t[2..].iter().map(|x| parse_u(x)).collect() }),
            "link" => v.links.push(Link { key: t[1].to_string(), tgt: t[2].to_string(), gate: vec![], force: vec![] }),
            "gate" => {
                for l in v.links.iter_mut().filter(|l| l.key == t[1]) {
                    l.gate.extend(t[2..].iter().map(|s| s.to_string()));
                }
            }
            "fn" => fns.push(FnCfg {
                name: t[1].to_string(),
                n: t[2].parse().unwrap(),
                ret: t[3].to_string(),
                args: t[4..].iter().map(|s| s.to_string()).collect(),
            }),
            "view" => v.views.push(View { key: t[1].to_string(), member: t[2].to_string(), fns: t[3..].iter().map(|s| s.to_string()).collect() }),
            "force" => {
                for l in v.links.iter_mut().filter(|l| l.key == t[1]) {
                    l.force.extend(t[2..].iter().map(|s| s.to_string()));
                }
            }
            "tysize" => tysizes.push((t[1].to_string(), t[2].parse().unwrap())),
            _ => panic!("bad cfg line {line}"),
        }
    }
    for (n, a) in &glue.fns {
        v.fps.push((n.to_string(), *a));
    }
    for t in &glue.types {
        v.types.push((t.name.to_string(), t.size, t.align, t.vis));
    }
    for (n, csz) in &tysizes {
        if let Some(t) = glue.types.iter().find(|t| t.name == n) {
            assert!(t.size == *csz || glue.size_ok.contains(&t.name), "type {n}: Rust size {} != C size {csz}", t.size);
        }
    }
    (glue.install)();
    for g in &glue.globals {
        let img = unsafe { std::slice::from_raw_parts(g.p, g.size) }.to_vec();
        v.globals.push((g.name.to_string(), g.p, g.size, g.vis, g.fill, img));
        v.regs.push(Reg { name: format!("g:{}", g.name), base: g.p as usize, size: g.size.max(1) });
        v.nperm = v.regs.len();
    }
    unsafe { VV = Some(v) };
    let full_sel = std::env::var("TUO_FULL").ok();
    let only = std::env::var("TUO_ONLY").ok(); // "<fn>:<idx>": run just that case (debugging)
    let mut got = vec![];
    for f in &fns {
        for idx in 0..f.n {
            if let Some(o) = &only {
                if *o != format!("{}:{}", f.name, idx) {
                    continue;
                }
            }
            let v = vv();
            v.full = false;
            if let Some(sel) = &full_sel {
                v.full = sel == &f.name || *sel == format!("{}:{}", f.name, idx);
            }
            v.out.clear();
            v.curfn = f.name.clone();
            v.rs = fnv(&f.name) ^ ((idx as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15));
            v.draw();
            v.lh = 0xcbf29ce484222325;
            v.pool_reset();
            for i in 0..v.globals.len() {
                let (p, size) = (v.globals[i].1, v.globals[i].2);
                let img = v.globals[i].5.clone();
                unsafe { std::ptr::copy_nonoverlapping(img.as_ptr(), p, size) };
            }
            v.mode = 1;
            v.args = [0; 12];
            v.argptr = [false; 12];
            let argfirst = v.links.iter().any(|l| l.tgt.starts_with("mayarg:"));
            let mut a = [0u64; 12];
            if argfirst {
                // pointer arguments exist before the globals are filled, so links can name them
                for (i, t) in f.args.iter().enumerate() {
                    if let Some(ty) = t.strip_prefix("p:") {
                        let lab = format!("a{i}");
                        v.pending_arg = Some(i);
                        a[i] = v.new_typed(Some(&lab), ty, 0) as usize as u64;
                        v.args[i] = a[i];
                        v.argptr[i] = true;
                    }
                }
            }
            for i in 0..v.globals.len() {
                if !v.globals[i].4 {
                    continue;
                }
                let path = format!("g:{}", v.globals[i].0);
                let (p, size, vis) = (v.globals[i].1, v.globals[i].2, v.globals[i].3);
                match vis {
                    Some(vis) => vis(v, &path, p),
                    None => v.bytes(&path, p, size),
                }
            }
            for (i, t) in f.args.iter().enumerate() {
                let key = format!("arg:{}#{}", f.name, i);
                let lab = format!("a{i}");
                if argfirst && t.starts_with("p:") {
                    continue;
                }
                a[i] = if t.starts_with('i') {
                    let bits: u32 = t[1..t.len() - 1].parse().unwrap();
                    v.leaf_int(&key, bits, (t.ends_with('s')) as u32)
                } else if t == "f32" {
                    v.leaf_f32(&key) as u64
                } else if t == "f64" {
                    v.leaf_f64(&key)
                } else if t == "null" {
                    0
                } else if let Some(n) = t.strip_prefix("e:") {
                    (glue.expr)(n)
                } else if t.starts_with("fn:") {
                    v.mk_target(&key, t) as u64
                } else if let Some(ty) = t.strip_prefix("pn:") {
                    if v.draw() & 3 == 0 {
                        0
                    } else {
                        v.new_typed(Some(&lab), ty, 0) as usize as u64
                    }
                } else if let Some(ty) = t.strip_prefix("p:") {
                    v.new_typed(Some(&lab), ty, 0) as usize as u64
                } else if let Some(n) = t.strip_prefix("cell:") {
                    let bits: usize = n.parse().unwrap();
                    let cp = v.new_buf(Some(&lab), 8);
                    let cv = v.leaf_int(&key, bits as u32, 1);
                    unsafe { std::ptr::copy_nonoverlapping(cv.to_le_bytes().as_ptr(), cp, bits / 8) };
                    cp as usize as u64
                } else if let Some(n) = t.strip_prefix("buf:") {
                    v.new_buf(Some(&lab), n.parse().unwrap()) as usize as u64
                } else {
                    panic!("bad arg {t}")
                };
            }
            v.emit(&format!("case {} {}", f.name, idx));
            if std::env::var("TUO_PRE").is_ok() {
                v.mode = 2;
                v.dump_all();
                v.mode = 1;
            }
            let r = (glue.call)(&f.name, &a);
            let v = vv();
            v.mode = 2;
            match f.ret.as_str() {
                "v" => v.emit("ret=void"),
                "i" => v.emit(&format!("ret={}", r as i64)),
                "u" => v.emit(&format!("ret={r}")),
                "f32" => v.emit(&format!("ret=f{:08x}", r as u32)),
                "f64" => v.emit(&format!("ret=f{r:016x}")),
                _ => {
                    let s = v.pstr(r as usize);
                    v.emit(&format!("ret=@{s}"))
                }
            }
            v.dump_all();
            if v.full {
                for l in &v.out {
                    println!("{l}");
                }
            }
            got.push(format!("{} {} {:016x}", f.name, idx, v.lh));
        }
    }
    if only.is_some() {
        return;
    }
    let exp: Vec<&str> = expected.lines().collect();
    let mut bad = vec![];
    let mut fns_bad: std::collections::BTreeMap<String, usize> = Default::default();
    for (i, g) in got.iter().enumerate() {
        if exp.get(i).map(|e| *e != g.as_str()).unwrap_or(true) {
            bad.push(i);
            *fns_bad.entry(g.split(' ').next().unwrap().to_string()).or_default() += 1;
        }
    }
    if got.len() != exp.len() || !bad.is_empty() {
        let first: Vec<String> = bad.iter().take(10).map(|&i| format!("{} (C: {})", got[i], exp.get(i).unwrap_or(&"-"))).collect();
        panic!("{} of {} cases differ (expected {} rows); per fn {:?}; first: {:#?}", bad.len(), got.len(), exp.len(), fns_bad, first);
    }
}

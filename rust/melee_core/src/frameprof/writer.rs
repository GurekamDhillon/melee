//! The profiler's writer thread: drains the game thread's records, finds the hitches and their causes, samples the
//! machine, and (re)writes the three files. Everything it allocates is its own (system heap, never the game arena:
//! the arena serves the game thread only).
use super::sys::{Sample, Sampler};
use super::{Event, Frame, Kind, State, C, MAX_MODS, NC};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

const TICK_US: f64 = 16_666.7;

/// One hitch: a presented frame or a tick over its threshold.
#[derive(Clone, Debug)]
pub(super) struct Hitch {
    pub t_us: u64,
    pub interval_us: u64,
    pub lost_us: u64,
    pub tick_max_us: u64,
    pub scene: String,
    pub tick: u64,
    pub cause: String,
    pub detail: String,
    pub parts: String,
    pub events: Vec<String>,
    pub busy: Option<String>,
}

/// The per-frame parts the cause is chosen from (µs).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Parts {
    pub interval: f64,
    pub expected: f64,
    pub tick: f64,
    pub tick_max: f64,
    pub ticks: u64,
    pub files: f64,
    pub parse: f64,
    pub mods: f64,
    pub prepare: f64,
    pub pipes: f64,
    pub tex: f64,
    pub draw: f64,
    pub acquire: f64,
    pub present: f64,
    pub ui: f64,
    pub sleep: f64,
    pub engine: f64,
    pub capture: f64,
}

impl Parts {
    fn of(f: &Frame) -> Parts {
        let us = |c: C| f.c[c as usize] as f64 / 1e3;
        Parts {
            interval: f.interval_us as f64,
            expected: f.expected_us as f64,
            tick: us(C::TickNs),
            tick_max: us(C::TickMaxNs),
            ticks: f.c[C::Ticks as usize],
            files: us(C::FileNs),
            parse: us(C::ParseNs),
            mods: us(C::ModNs),
            prepare: us(C::PrepareNs),
            pipes: us(C::PipeNs),
            tex: us(C::TexNs),
            draw: us(C::DrawNs),
            acquire: us(C::AcquireNs),
            present: us(C::PresentNs),
            ui: us(C::UiNs),
            sleep: us(C::SleepNs),
            engine: us(C::EngineNs),
            capture: us(C::CaptureNs),
        }
    }
    /// What the timed parts do not cover (the event loop, the OS, other processes): never negative.
    pub fn other(&self) -> f64 {
        (self.interval - self.tick - self.prepare - self.draw - self.acquire - self.present - self.ui - self.sleep).max(0.0)
    }
    fn text(&self) -> String {
        let ms = |v: f64| v / 1e3;
        format!(
            "sim {:.1} ms ({} tick{}: engine step {:.1}, of it mods {:.1}; frame capture {:.1}; harness {:.1}), prepare {:.1}, draw {:.1}, display wait {:.1}, present {:.1}, ui {:.1}, pacing sleep {:.1}, unaccounted {:.1}",
            ms(self.tick), self.ticks, if self.ticks == 1 { "" } else { "s" }, ms(self.engine), ms(self.mods), ms(self.capture), ms((self.tick - self.engine - self.capture).max(0.0)), ms(self.prepare), ms(self.draw), ms(self.acquire), ms(self.present), ms(self.ui), ms(self.sleep), ms(self.other())
        )
    }
}

/// The cause of a hitch from its parts, its events and the machine: (category, detail). The category is what the
/// "top causes" table sums; the detail names files, mods, counts.
pub(super) fn classify(p: &Parts, ev: &[(Kind, String, u32, u64)], f: &Frame, mod_names: &[String], busy: Option<&Sample>) -> (String, String) {
    let has = |k: Kind| ev.iter().any(|e| e.0 == k);
    let names = |k: Kind| ev.iter().filter(|e| e.0 == k).map(|e| e.1.clone()).collect::<Vec<_>>();
    let short = |v: Vec<String>| -> String {
        let n = v.len();
        let mut s = v.into_iter().take(4).collect::<Vec<_>>().join("; ");
        if n > 4 {
            let _ = write!(s, "; +{} more", n - 4);
        }
        s
    };
    let context = || -> String {
        let mut c = Vec::new();
        for k in [Kind::Stage, Kind::Fighter, Kind::Envoy, Kind::Scene, Kind::Test] {
            c.extend(names(k));
        }
        if c.is_empty() { String::new() } else { format!(" during: {}", short(c)) }
    };
    // the largest timed part
    let parts = [("sim", p.tick), ("prepare", p.prepare), ("draw", p.draw), ("display wait", p.acquire), ("present", p.present), ("ui", p.ui), ("other", p.other()), ("sleep", p.sleep)];
    let (big, big_v) = parts.iter().copied().fold(("", -1.0), |a, b| if b.1 > a.1 { b } else { a });
    if has(Kind::Test) {
        return ("test (induced)".into(), short(names(Kind::Test)));
    }
    let busy_text = busy.filter(|b| b.cpu.is_some_and(|c| c >= 85.0) || b.others.iter().any(|o| !o.0.contains("[below normal]") && !o.0.contains("[idle]"))).map(|b| {
        let mut s = String::new();
        if let Some(c) = b.cpu {
            let _ = write!(s, "CPU {c:.0}% busy");
        }
        if b.other_count() > 0 {
            let _ = write!(s, "{}other heavy processes: {}", if s.is_empty() { "" } else { ", " }, b.others_text());
        }
        s
    });
    match big {
        "sim" => {
            let load = p.files + p.parse;
            if has(Kind::Stage) {
                let mut d = short(names(Kind::Stage));
                if load > 0.0 {
                    let _ = write!(d, "; file loads {:.0} ms: {}", load / 1e3, short(names(Kind::File)));
                }
                return ("stage switch".into(), d);
            }
            if load >= 0.4 * p.tick {
                return ("file load".into(), format!("{} ({:.0} ms read + parse){}", short(names(Kind::File)), load / 1e3, context()));
            }
            if has(Kind::Fighter) && !has(Kind::Scene) {
                return ("fighter create / wake".into(), format!("{}{}", short(names(Kind::Fighter)), if has(Kind::Envoy) { format!("; {}", short(names(Kind::Envoy))) } else { String::new() }));
            }
            if p.mods >= 0.4 * p.tick {
                let (i, v) = f.mods.iter().enumerate().fold((0, 0u64), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
                let n = mod_names.get(i).cloned().unwrap_or_else(|| format!("mod {i}"));
                return (format!("mod {n}"), format!("{n} ran {:.1} ms{}", v as f64 / 1e6, context()));
            }
            if has(Kind::Alloc) {
                return ("big allocation".into(), format!("{}{}", short(names(Kind::Alloc)), context()));
            }
            if has(Kind::Scene) {
                return ("scene change".into(), format!("{}{}", short(names(Kind::Scene)), if load > 0.0 { format!("; file loads {:.0} ms: {}", load / 1e3, short(names(Kind::File))) } else { String::new() }));
            }
            if p.capture >= 0.4 * p.tick {
                return ("frame capture".into(), format!("handing the frame to the presenter took {:.1} ms{}", p.capture / 1e3, context()));
            }
            if p.engine > 0.0 && p.tick - p.engine - p.capture >= 0.5 * p.tick && p.mods < 0.4 * p.tick {
                return ("harness around the tick".into(), format!("{:.1} ms outside the engine step (UI mods, audio, scene bookkeeping){}", (p.tick - p.engine - p.capture) / 1e3, context()));
            }
            if p.ticks >= 2 && p.tick_max < 0.6 * p.tick {
                return ("catch-up ticks".into(), format!("{} ticks ran back to back (the frame before was late){}", p.ticks, context()));
            }
            let c = context();
            if let (Some(b), true) = (busy_text.as_ref(), c.is_empty()) {
                return ("sim tick slow (machine busy)".into(), format!("longest tick {:.1} ms; {b}", p.tick_max / 1e3));
            }
            ("sim tick slow".into(), format!("longest tick {:.1} ms{}", p.tick_max / 1e3, c))
        }
        "prepare" => {
            if p.pipes >= 0.3 * p.prepare {
                ("new shaders".into(), format!("{} GPU pipeline(s) created, {:.1} ms", f.c[C::Pipes as usize], p.pipes / 1e3))
            } else if p.tex >= 0.3 * p.prepare {
                ("texture uploads".into(), format!("{} texture(s), {} KB, {:.1} ms", f.c[C::Textures as usize], f.c[C::TexBytes as usize] / 1024, p.tex / 1e3))
            } else {
                ("frame prepare".into(), format!("prepare {:.1} ms ({} pipelines, {} textures)", p.prepare / 1e3, f.c[C::Pipes as usize], f.c[C::Textures as usize]))
            }
        }
        "draw" => ("draw / interpolation".into(), format!("draw {:.1} ms (interpolation {:.1} ms)", p.draw / 1e3, f.c[C::InterpNs as usize] as f64 / 1e6)),
        "display wait" => {
            if p.pipes > 0.0 || f.c[C::Textures as usize] > 0 {
                ("GPU busy (new shaders / textures)".into(), format!("waited {:.1} ms for a swapchain image after {} new pipelines, {} textures", p.acquire / 1e3, f.c[C::Pipes as usize], f.c[C::Textures as usize]))
            } else if let Some(b) = busy_text.as_ref() {
                ("display wait (machine busy)".into(), format!("waited {:.1} ms for a swapchain image; {b}", p.acquire / 1e3))
            } else {
                ("display / GPU wait".into(), format!("waited {:.1} ms for a swapchain image (vsync, the GPU, or the compositor)", p.acquire / 1e3))
            }
        }
        "sleep" => ("pacing sleep overshoot".into(), format!("the main loop slept {:.1} ms waiting for its next frame slot (the frame-cap / VRR pacer woke late or the slot was already late)", p.sleep / 1e3)),
        "present" => ("present call".into(), format!("blit + present {:.1} ms", p.present / 1e3)),
        "ui" => ("ui mods".into(), format!("UI frame {:.1} ms", p.ui / 1e3)),
        _ => {
            let _ = big_v;
            if has(Kind::Window) {
                ("window event".into(), short(names(Kind::Window)))
            } else if let Some(b) = busy_text {
                ("machine busy".into(), format!("{:.1} ms outside the game's own work; {b}", p.other() / 1e3))
            } else {
                ("outside the game (OS / driver)".into(), format!("{:.1} ms outside the game's own work (the OS did not run the game: scheduler, driver, compositor)", p.other() / 1e3))
            }
        }
    }
}

struct Agg {
    t0_wall: String,
    stem: String,
    frames_n: u64,
    intervals: Vec<u32>,
    ticks: Vec<u32>,
    engine: Vec<u32>,
    capture: Vec<u32>,
    parts: [Vec<u32>; 8],
    total_ticks: u64,
    total_mod_ns: [u64; MAX_MODS],
    max_mod_ns: [u64; MAX_MODS],
    totals: [u64; NC],
    hitches: Vec<Hitch>,
    pending: Vec<Event>,
    events_n: HashMap<&'static str, u64>,
    scenes: HashMap<u16, String>,
    scene_now: String,
    samples: Vec<Sample>,
    last_t: u64,
    dropped: u64,
    interp_frames: u64,
    timeline_scenes: Vec<(u64, String)>,
    spans: Vec<(u64, u64, &'static str, String)>,
}

impl Agg {
    fn scene_name(&self, code: u16) -> String {
        self.scenes.get(&code).cloned().unwrap_or_else(|| if code == 0 { "boot".into() } else { format!("mode {} scene {}", code >> 8, code & 0xFF) })
    }
}

fn mod_names(s: &State) -> Vec<String> {
    s.mod_names.lock().unwrap().iter().map(|m| String::from_utf8_lossy(&m.name[..m.n as usize]).into_owned()).collect()
}

pub(super) fn run(s: &'static State, stem: String) {
    super::WRITER.with(|w| w.set(true));
    let dir = s.cfg.dir.clone();
    let open = |ext: &str| std::fs::File::create(dir.join(format!("{stem}.{ext}"))).ok().map(|f| std::io::BufWriter::with_capacity(1 << 16, f));
    let mut csv = open("csv");
    let mut jsonl = open("jsonl");
    if let Some(c) = csv.as_mut() {
        let _ = writeln!(c, "t_ms,interval_ms,expected_ms,hitch,scene,tick,ticks,tick_ms,tick_max_ms,file_ms,parse_ms,files,file_kb,mods_ms,prepare_ms,pipelines,pipeline_ms,textures,texture_kb,texture_ms,draw_ms,interp_ms,display_wait_ms,present_ms,ui_ms,pacing_sleep_ms,unaccounted_ms,big_allocs,arena_allocs,window_events,interp{}", (0..MAX_MODS).map(|i| format!(",mod{i}_ms")).collect::<String>());
    }
    let mut agg = Agg {
        t0_wall: super::local_stamp(),
        stem: stem.clone(),
        frames_n: 0,
        intervals: Vec::with_capacity(1 << 16),
        ticks: Vec::with_capacity(1 << 16),
        engine: Vec::new(),
        capture: Vec::new(),
        parts: Default::default(),
        total_ticks: 0,
        total_mod_ns: [0; MAX_MODS],
        max_mod_ns: [0; MAX_MODS],
        totals: [0; NC],
        hitches: Vec::new(),
        pending: Vec::new(),
        events_n: HashMap::new(),
        scenes: HashMap::new(),
        scene_now: "boot".into(),
        samples: Vec::new(),
        last_t: 0,
        dropped: 0,
        interp_frames: 0,
        timeline_scenes: Vec::new(),
        spans: Vec::new(),
    };
    let mut spare_f: Vec<Frame> = Vec::with_capacity(super::FRAME_CAP);
    let mut spare_e: Vec<Event> = Vec::with_capacity(super::EVENT_CAP);
    let mut sampler = Sampler::default();
    sampler.cpu();
    let (mut t_cpu, mut t_proc, mut t_report) = (Instant::now(), Instant::now() - Duration::from_secs(10), Instant::now());
    let mut cur = Sample::default();
    loop {
        let stopping = s.stop.load(Relaxed);
        if !stopping {
            std::thread::sleep(Duration::from_millis(200));
        }
        // machine: CPU every second, the process list every 3 s
        if t_cpu.elapsed() >= Duration::from_secs(1) || stopping {
            t_cpu = Instant::now();
            let (cpu, own) = sampler.cpu();
            if t_proc.elapsed() >= Duration::from_secs(3) {
                t_proc = Instant::now();
                cur.others = sampler.others();
            }
            cur.t_us = super::now_us(s);
            cur.cpu = cpu;
            cur.own = own;
            if let Some(j) = jsonl.as_mut() {
                let _ = writeln!(j, "{{\"type\":\"machine\",\"t_ms\":{:.0},\"cpu_pct\":{},\"game_cpu_pct\":{},\"others\":\"{}\"}}", cur.t_us as f64 / 1e3, cpu.map_or("null".into(), |c| format!("{c:.1}")), own.map_or("null".into(), |c| format!("{c:.1}")), cur.others_text());
            }
            agg.samples.push(cur.clone());
        }
        // drain (swap with the reserved spares: the game thread never allocates)
        std::mem::swap(&mut *s.frames.lock().unwrap(), &mut spare_f);
        std::mem::swap(&mut *s.events.lock().unwrap(), &mut spare_e);
        let names = mod_names(s);
        // new events: counted and written once, then kept a while for matching to the frames around them
        for e in spare_e.drain(..) {
            *agg.events_n.entry(Kind::from_u8(e.kind).name()).or_default() += 1;
            if e.kind == Kind::Scene as u8 {
                agg.scenes.insert(e.val as u16, e.text.as_str().trim_start_matches("scene ").to_string());
            }
            if let Some(j) = jsonl.as_mut() {
                let _ = writeln!(j, "{{\"type\":\"event\",\"t_ms\":{:.1},\"tick\":{},\"kind\":\"{}\",\"ms\":{:.2},\"value\":{},\"text\":\"{}\"}}", e.t_us as f64 / 1e3, e.tick, Kind::from_u8(e.kind).name(), e.dur_us as f64 / 1e3, e.val, e.text.as_str());
            }
            if matches!(Kind::from_u8(e.kind), Kind::Stage | Kind::Fighter | Kind::Envoy | Kind::Test) && agg.spans.len() < 5000 {
                agg.spans.push((e.t_us, e.tick, Kind::from_u8(e.kind).name(), e.text.as_str().to_string()));
            }
            agg.pending.push(e);
        }
        let frames: Vec<Frame> = spare_f.drain(..).collect();
        for f in &frames {
            write_frame(s, &mut agg, f, &names, csv.as_mut(), jsonl.as_mut());
        }
        let keep_from = agg.last_t.saturating_sub(5_000_000);
        agg.pending.retain(|e| e.t_us >= keep_from);
        if spare_f.capacity() < super::FRAME_CAP {
            spare_f.reserve(super::FRAME_CAP);
        }
        if spare_e.capacity() < super::EVENT_CAP {
            spare_e.reserve(super::EVENT_CAP);
        }
        agg.dropped = s.dropped.load(Relaxed);
        if t_report.elapsed() >= Duration::from_secs(5) || stopping {
            t_report = Instant::now();
            if let Some(c) = csv.as_mut() {
                let _ = c.flush();
            }
            if let Some(j) = jsonl.as_mut() {
                let _ = j.flush();
            }
            let txt = report(s, &agg, &names, stopping);
            let tmp = dir.join(format!("{stem}.txt.tmp"));
            if std::fs::write(&tmp, &txt).is_ok() {
                let _ = std::fs::rename(&tmp, dir.join(format!("{stem}.txt")));
            }
        }
        if stopping {
            if let Some(j) = jsonl.as_mut() {
                let _ = writeln!(j, "{{\"type\":\"summary\",\"frames\":{},\"hitches\":{},\"lost_ms\":{:.0},\"dropped_records\":{}}}", agg.frames_n, agg.hitches.len(), agg.hitches.iter().map(|h| h.lost_us).sum::<u64>() as f64 / 1e3, agg.dropped);
                let _ = j.flush();
            }
            break;
        }
    }
}

/// The events (kind, text, ms, value) of the window `(a, b]`.
fn events_in(agg: &Agg, a: u64, b: u64) -> Vec<(Kind, String, u32, u64)> {
    agg.pending.iter().filter(|e| e.t_us > a && e.t_us <= b).map(|e| (Kind::from_u8(e.kind), e.text.as_str().to_string(), e.dur_us, e.val)).collect()
}

fn write_frame(s: &State, agg: &mut Agg, f: &Frame, names: &[String], csv: Option<&mut std::io::BufWriter<std::fs::File>>, jsonl: Option<&mut std::io::BufWriter<std::fs::File>>) {
    let prev_t = agg.last_t;
    agg.last_t = f.t_us;
    let scene = agg.scene_name(f.scene);
    if scene != agg.scene_now {
        agg.scene_now = scene.clone();
        agg.timeline_scenes.push((f.t_us, scene.clone()));
    }
    for i in 0..NC {
        agg.totals[i] += f.c[i];
    }
    let ticks = f.c[C::Ticks as usize];
    agg.total_ticks += ticks;
    for i in 0..MAX_MODS {
        agg.total_mod_ns[i] += f.mods[i];
        agg.max_mod_ns[i] = agg.max_mod_ns[i].max(f.mods[i]);
    }
    if f.interval_us == 0 {
        return;
    }
    agg.frames_n += 1;
    if f.interp {
        agg.interp_frames += 1;
    }
    let p = Parts::of(f);
    agg.intervals.push(f.interval_us);
    if ticks > 0 {
        agg.engine.push(p.engine as u32);
        agg.capture.push(p.capture as u32);
        let max = (p.tick_max as u32).max(1);
        agg.ticks.push(max);
        let rest = (p.tick - p.tick_max).max(0.0);
        for _ in 1..ticks {
            agg.ticks.push((rest / (ticks - 1) as f64) as u32);
        }
    }
    for (i, v) in [p.prepare, p.draw, p.acquire, p.present, p.ui, p.other(), p.mods, p.files + p.parse].into_iter().enumerate() {
        agg.parts[i].push(v as u32);
    }
    let hitch = p.interval > s.cfg.frame_mult * p.expected || p.tick_max > s.cfg.tick_mult * TICK_US;
    if let Some(c) = csv {
        let ms = |v: f64| v / 1e3;
        let mut row = format!(
            "{:.2},{:.3},{:.3},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{},{},{:.3},{:.3},{},{:.3},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{}",
            f.t_us as f64 / 1e3, ms(p.interval), ms(p.expected), hitch as u8, scene.replace(',', ";"), f.tick, ticks, ms(p.tick), ms(p.tick_max), ms(p.files), ms(p.parse), f.c[C::Files as usize], f.c[C::FileBytes as usize] / 1024, ms(p.mods),
            ms(p.prepare), f.c[C::Pipes as usize], ms(p.pipes), f.c[C::Textures as usize], f.c[C::TexBytes as usize] / 1024, ms(p.tex), ms(p.draw), f.c[C::InterpNs as usize] as f64 / 1e6, ms(p.acquire), ms(p.present), ms(p.ui), ms(p.sleep), ms(p.other()),
            f.c[C::BigAllocs as usize], f.c[C::ArenaAllocs as usize], f.c[C::WindowEvents as usize], f.interp as u8
        );
        for m in f.mods {
            let _ = write!(row, ",{:.3}", m as f64 / 1e6);
        }
        let _ = writeln!(c, "{row}");
    }
    if !hitch {
        return;
    }
    let a = prev_t.min(f.t_us.saturating_sub(f.interval_us as u64));
    let ev = events_in(agg, a, f.t_us);
    // the machine sample nearest before the hitch (within 3 s)
    let busy = agg.samples.iter().rev().find(|x| x.t_us <= f.t_us + 1_000_000 && f.t_us.saturating_sub(x.t_us) <= 3_000_000).cloned();
    let (cause, detail) = classify(&p, &ev, f, names, busy.as_ref());
    let lost = if p.interval > s.cfg.frame_mult * p.expected { (p.interval - p.expected).max(0.0) } else { (p.tick_max - TICK_US).max(0.0) };
    let h = Hitch {
        t_us: f.t_us.saturating_sub(f.interval_us as u64),
        interval_us: f.interval_us as u64,
        lost_us: lost as u64,
        tick_max_us: p.tick_max as u64,
        scene,
        tick: f.tick,
        cause,
        detail,
        parts: p.text(),
        events: ev.iter().map(|e| format!("{}: {}", e.0.name(), e.1)).collect(),
        busy: busy.map(|b| format!("{}{}", b.cpu.map_or("CPU ?".into(), |c| format!("CPU {c:.0}%")), if b.other_count() > 0 { format!(", others: {}", b.others_text()) } else { String::new() })),
    };
    if let Some(j) = jsonl {
        let esc = |t: &str| t.replace('\\', "/").replace('"', "'");
        let _ = writeln!(
            j,
            "{{\"type\":\"hitch\",\"t_ms\":{:.1},\"length_ms\":{:.2},\"lost_ms\":{:.2},\"longest_tick_ms\":{:.2},\"scene\":\"{}\",\"tick\":{},\"cause\":\"{}\",\"detail\":\"{}\",\"parts\":\"{}\",\"events\":[{}],\"machine\":\"{}\"}}",
            h.t_us as f64 / 1e3, h.interval_us as f64 / 1e3, h.lost_us as f64 / 1e3, h.tick_max_us as f64 / 1e3, esc(&h.scene), h.tick, esc(&h.cause), esc(&h.detail), esc(&h.parts),
            h.events.iter().map(|e| format!("\"{}\"", esc(e))).collect::<Vec<_>>().join(","), esc(h.busy.as_deref().unwrap_or(""))
        );
    }
    agg.hitches.push(h);
}

fn q(v: &[u32], f: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mut w = v.to_vec();
    let k = ((w.len() - 1) as f64 * f).round() as usize;
    let (_, x, _) = w.select_nth_unstable(k);
    *x as f64 / 1e3
}

fn clock(us: u64) -> String {
    let s = us as f64 / 1e6;
    format!("{}:{:04.1}", (s / 60.0) as u64, s % 60.0)
}

fn dur(us: u64) -> String {
    let s = us / 1_000_000;
    if s >= 60 { format!("{} min {} s", s / 60, s % 60) } else { format!("{:.1} s", us as f64 / 1e6) }
}

fn report(s: &State, agg: &Agg, names: &[String], done: bool) -> String {
    let info = s.info.lock().unwrap().clone();
    let get = |k: &str| info.iter().find(|e| e.0 == k).map(|e| e.1.clone());
    let span = agg.last_t.max(1);
    let mins = span as f64 / 6e7;
    let fps = agg.frames_n as f64 / (span as f64 / 1e6);
    let tps = agg.total_ticks as f64 / (span as f64 / 1e6);
    let lost: u64 = agg.hitches.iter().map(|h| h.lost_us).sum();
    let visible = agg.hitches.iter().filter(|h| h.interval_us >= 25_000 || h.tick_max_us >= 25_000).count();
    let big = agg.hitches.iter().filter(|h| h.interval_us >= 50_000 || h.tick_max_us >= 50_000).count();
    let on_time = 100.0 * (1.0 - agg.hitches.len() as f64 / agg.frames_n.max(1) as f64);
    let mut causes: HashMap<&str, (u64, u64, u64)> = HashMap::new();
    for h in &agg.hitches {
        let c = causes.entry(h.cause.as_str()).or_default();
        c.0 += 1;
        c.1 += h.lost_us;
        c.2 = c.2.max(h.interval_us.max(h.tick_max_us));
    }
    let mut causes: Vec<_> = causes.into_iter().collect();
    causes.sort_by(|a, b| b.1 .1.cmp(&a.1 .1).then(b.1 .0.cmp(&a.1 .0)));
    let mut o = String::new();
    let _ = writeln!(o, "MELEE FRAME PROFILE  {}  ({})", agg.stem, if done { "complete" } else { "in progress: rewritten every 5 s" });
    let _ = writeln!(o, "{}", "=".repeat(78));
    let _ = writeln!(o);
    let _ = writeln!(o, "IN PLAIN WORDS");
    {
        let busy_lost: u64 = agg.hitches.iter().filter(|h| h.cause.contains("busy")).map(|h| h.lost_us).sum();
        if !agg.hitches.is_empty() && busy_lost * 2 >= lost.max(1) {
            let _ = writeln!(o, "- MAIN FINDING: most of the time lost to hitches ({:.0}%) was while OTHER heavy programs were running on this machine (other melee builds, cargo / rustc / link); the game itself was not the cause. See \"Other programs\" below.", 100.0 * busy_lost as f64 / lost.max(1) as f64);
        } else if !agg.hitches.is_empty() {
            let _ = writeln!(o, "- MAIN FINDING: the hitches came mostly from the game's own work (see \"What cost the most\"), not from other programs{}.", if agg.samples.iter().any(|x| x.other_count() > 0) { " (other heavy processes did run, see below, but the CPU had room and they ran at low priority when these hitches happened)" } else { "" });
        }
    }
    let mode = if names.is_empty() { "no mods (vanilla)".to_string() } else { format!("mods {}", names.join(", ")) };
    let _ = writeln!(o, "- The session ran {} ({mode}). The game showed {fps:.0} frames a second on average and ran {tps:.1} ticks a second (60 is full speed).", dur(span));
    let _ = writeln!(o, "  Display: {}. Interpolation was on for {:.0}% of frames.", get("pacing").unwrap_or_else(|| "unknown".into()), 100.0 * agg.interp_frames as f64 / agg.frames_n.max(1) as f64);
    let _ = writeln!(
        o,
        "- Smoothness: {on_time:.2}% of frames arrived on time. {} hitches ({:.1} a minute); together they cost {:.2} s. {visible} of them were long enough to see (25 ms or more), {big} were big (50 ms or more).",
        agg.hitches.len(), agg.hitches.len() as f64 / mins.max(1e-9), lost as f64 / 1e6
    );
    if let Some(w) = agg.hitches.iter().max_by_key(|h| h.interval_us.max(h.tick_max_us)) {
        let _ = writeln!(o, "- The worst: {:.0} ms at {} in {} (tick {}): {} - {}.", w.interval_us.max(w.tick_max_us) as f64 / 1e3, clock(w.t_us), w.scene, w.tick, w.cause, w.detail);
    }
    if !causes.is_empty() {
        let _ = writeln!(o, "- What cost the most:");
        for (i, (c, (n, l, worst))) in causes.iter().take(5).enumerate() {
            let _ = writeln!(o, "  {}. {c}: {n} hitch{}, {:.0} ms lost in all, worst {:.0} ms.", i + 1, if *n == 1 { "" } else { "es" }, *l as f64 / 1e3, *worst as f64 / 1e3);
        }
    }
    // the machine
    let with_others = agg.samples.iter().filter(|x| x.other_count() > 0).count();
    let hot = agg.samples.iter().filter(|x| x.cpu.is_some_and(|c| c >= 85.0)).count();
    let busy_hitches = agg.hitches.iter().filter(|h| h.cause.contains("busy")).count();
    if !agg.samples.is_empty() {
        let mut all: HashMap<String, u32> = HashMap::new();
        for x in &agg.samples {
            for (n, c) in &x.others {
                let e = all.entry(n.clone()).or_default();
                *e = (*e).max(*c);
            }
        }
        let mut all: Vec<_> = all.into_iter().collect();
        all.sort();
        let avg_cpu = agg.samples.iter().filter_map(|x| x.cpu).sum::<f32>() / agg.samples.iter().filter(|x| x.cpu.is_some()).count().max(1) as f32;
        let own = agg.samples.iter().filter_map(|x| x.own).sum::<f32>() / agg.samples.iter().filter(|x| x.own.is_some()).count().max(1) as f32;
        if with_others > 0 {
            let _ = writeln!(
                o,
                "- Other programs: heavy processes ran beside the game for {:.0}% of the session ({}). The machine's CPU averaged {avg_cpu:.0}% (the game itself {own:.0}% of one core); it was over 85% busy {:.0}% of the time. {busy_hitches} of the {} hitches were put down to the machine being busy (CPU over 85%, or competing processes at normal priority or higher).",
                100.0 * with_others as f64 / agg.samples.len() as f64, all.iter().map(|(n, c)| if *c > 1 { format!("{n} up to x{c}") } else { n.clone() }).collect::<Vec<_>>().join(", "), 100.0 * hot as f64 / agg.samples.len() as f64, agg.hitches.len()
            );
        } else {
            let _ = writeln!(o, "- Other programs: no other melee / cargo / rustc / Dolphin processes ran. The machine's CPU averaged {avg_cpu:.0}% (the game itself {own:.0}% of one core); over 85% busy {:.0}% of the time.", 100.0 * hot as f64 / agg.samples.len() as f64);
        }
    }
    if !names.is_empty() {
        let per: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n} {:.2} ms per tick on average (worst frame {:.1} ms)", agg.total_mod_ns[i] as f64 / 1e6 / agg.total_ticks.max(1) as f64, agg.max_mod_ns[i] as f64 / 1e6))
            .collect();
        let _ = writeln!(o, "- Mods: {}.", per.join("; "));
    }
    let t = &agg.totals;
    let _ = writeln!(
        o,
        "- During the session: {} file loads ({} KB, {:.0} ms), {} archive parses ({:.0} ms), {} new GPU pipelines ({:.0} ms, most built on the GX decode thread, not on the frame's path), {} textures uploaded ({} KB, {:.0} ms), {} big allocations, {} window events.",
        t[C::Files as usize], t[C::FileBytes as usize] / 1024, t[C::FileNs as usize] as f64 / 1e6, t[C::Parses as usize], t[C::ParseNs as usize] as f64 / 1e6, t[C::Pipes as usize], t[C::PipeNs as usize] as f64 / 1e6,
        t[C::Textures as usize], t[C::TexBytes as usize] / 1024, t[C::TexNs as usize] as f64 / 1e6, t[C::BigAllocs as usize], t[C::WindowEvents as usize]
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "SETTINGS");
    for (k, v) in &info {
        let _ = writeln!(o, "  {k:<14} {v}");
    }
    let _ = writeln!(o, "  {:<14} present over {:.1}x the display's frame interval, or a tick over {:.1}x 16.7 ms", "hitch rule", s.cfg.frame_mult, s.cfg.tick_mult);
    let _ = writeln!(o);
    let _ = writeln!(o, "FRAME TIMES (ms)                   p50      p95      p99      max");
    let row = |o: &mut String, name: &str, v: &[u32]| {
        let _ = writeln!(o, "  {name:<30} {:>7.2}  {:>7.2}  {:>7.2}  {:>7.2}", q(v, 0.5), q(v, 0.95), q(v, 0.99), q(v, 1.0));
    };
    row(&mut o, "presented frame interval", &agg.intervals);
    row(&mut o, "tick (sim)", &agg.ticks);
    row(&mut o, "  engine step (per frame)", &agg.engine);
    row(&mut o, "  frame capture (per frame)", &agg.capture);
    for (i, n) in ["  fresh-frame prepare", "  draw + interpolation", "  display wait (acquire)", "  present call", "  ui frame", "  unaccounted (OS, loop)", "  wasm mods (per frame)", "  file read + parse"].iter().enumerate() {
        row(&mut o, n, &agg.parts[i]);
    }
    let _ = writeln!(o, "  ({} presented frames, {} ticks)", agg.frames_n, agg.total_ticks);
    let _ = writeln!(o);
    let _ = writeln!(o, "TOP CAUSES (by time lost)");
    let _ = writeln!(o, "  {:<36} {:>7} {:>9} {:>9}", "cause", "hitches", "lost ms", "worst ms");
    for (c, (n, l, w)) in &causes {
        let _ = writeln!(o, "  {c:<36} {n:>7} {:>9.0} {:>9.0}", *l as f64 / 1e3, *w as f64 / 1e3);
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "TRANSITIONS AND SPAWNS (timed, whether or not they caused a hitch)");
    for (t, tick, k, txt) in agg.spans.iter().take(300) {
        let _ = writeln!(o, "  {:>8}  tick {:>6}  {k}: {txt}", clock(*t), tick);
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "SCENES");
    for (t, n) in &agg.timeline_scenes {
        let _ = writeln!(o, "  {:>8}  {n}", clock(*t));
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "TIMELINE OF HITCHES (time, length, time lost, scene, tick, cause - detail)");
    let max_lines = 400;
    for h in agg.hitches.iter().take(max_lines) {
        let _ = writeln!(o, "  {:>8}  {:>6.1} ms  lost {:>6.1}  {:<16} {:>6}  {} - {}", clock(h.t_us), h.interval_us.max(h.tick_max_us) as f64 / 1e3, h.lost_us as f64 / 1e3, h.scene, h.tick, h.cause, h.detail);
    }
    if agg.hitches.len() > max_lines {
        let _ = writeln!(o, "  ... {} more in the .jsonl", agg.hitches.len() - max_lines);
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "THE 15 WORST HITCHES IN DETAIL");
    let mut worst: Vec<&Hitch> = agg.hitches.iter().collect();
    worst.sort_by_key(|h| std::cmp::Reverse(h.interval_us.max(h.tick_max_us)));
    for h in worst.iter().take(15) {
        let _ = writeln!(o, "  {} {:.1} ms in {} (tick {}): {} - {}", clock(h.t_us), h.interval_us.max(h.tick_max_us) as f64 / 1e3, h.scene, h.tick, h.cause, h.detail);
        let _ = writeln!(o, "      split: {}", h.parts);
        if !h.events.is_empty() {
            let _ = writeln!(o, "      events: {}", h.events.iter().take(12).cloned().collect::<Vec<_>>().join(" | "));
        }
        if let Some(b) = &h.busy {
            let _ = writeln!(o, "      machine: {b}");
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "WHAT THIS CANNOT SEE");
    let _ = writeln!(o, "  - Inside the GPU: a frame whose GPU work is slow shows only as a longer display wait on a later frame.");
    let _ = writeln!(o, "  - What the display did with a frame (a frame presented on time can still be shown late or twice by the compositor);");
    let _ = writeln!(o, "    smears between frames are an interpolation / image question this tool does not measure.");
    let _ = writeln!(o, "  - Which other program used the CPU at the instant of a hitch: the machine is sampled once a second, the process list every 3 s.");
    let _ = writeln!(o, "  - Time inside one mod's own code paths (only the time of each call into a mod), and the audio thread beyond underrun counts.");
    if agg.dropped > 0 {
        let _ = writeln!(o, "  - {} records were dropped (the buffers were full).", agg.dropped);
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "Files beside this one: {0}.csv (one row per presented frame), {0}.jsonl (machine samples, events, hitches). Started {1}.", agg.stem, agg.t0_wall);
    o
}

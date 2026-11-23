//! Per-frame timing for `slpcheck --time` / `--render` (and anything else that wants a frame timer).
//!
//! Two clocks per span: wall time (`Instant`) and, on Windows, the calling thread's own CPU cycles
//! (`QueryThreadCycleTime`, converted with a start-up calibration against the wall clock). The cycle count is
//! what Hero's Source Port reports as `cpu_ms` (game-thread cycles of one sim frame: waits and preemption by other
//! processes excluded), so it is the figure to compare. Elsewhere the CPU figure falls back to wall.
//!
//! Zero cost when off: the caller holds an `Option<FrameTimes>`; nothing here runs without it.
use std::io::Write;
use std::time::Instant;

#[cfg(windows)]
mod sys {
    #[link(name = "kernel32")]
    extern "system" {
        fn QueryThreadCycleTime(h: isize, cycles: *mut u64) -> i32;
    }
    pub fn thread_cycles() -> u64 {
        let mut c = 0u64;
        // SAFETY: -2 is GetCurrentThread()'s pseudo handle (the calling thread) and a valid out pointer.
        unsafe { QueryThreadCycleTime(-2, &mut c) };
        c
    }
}
#[cfg(not(windows))]
mod sys {
    pub fn thread_cycles() -> u64 {
        0
    }
}

/// A point in time on both clocks.
#[derive(Clone, Copy)]
pub struct Mark {
    wall: Instant,
    cyc: u64,
}

pub fn mark() -> Mark {
    Mark { wall: Instant::now(), cyc: sys::thread_cycles() }
}

struct Row {
    fr: i32,
    sim_wall: f32,
    sim_cpu: f32,
    rnd_wall: f32,
    rnd_cpu: f32,
}

pub struct FrameTimes {
    rows: Vec<Row>,
    csv: Option<std::io::BufWriter<std::fs::File>>,
    /// thread cycles per second (0: no cycle counter, CPU figures are wall)
    cps: f64,
    pub render: bool,
    start: Instant,
}

impl FrameTimes {
    /// `csv`: a per-frame CSV (Hero's `MELEE_SIM_TIMES` columns first, so scratch/perfcmp/analyze.py reads it).
    pub fn new(csv: Option<&str>, render: bool) -> Self {
        let cps = {
            // calibrate: spin 40 ms on this thread, cycles / seconds
            let a = mark();
            while a.wall.elapsed().as_millis() < 40 {
                std::hint::spin_loop();
            }
            let b = mark();
            if b.cyc > a.cyc { (b.cyc - a.cyc) as f64 / b.wall.duration_since(a.wall).as_secs_f64() } else { 0.0 }
        };
        let mut csv = csv.and_then(|p| std::fs::File::create(p).ok()).map(std::io::BufWriter::new);
        if let Some(f) = csv.as_mut() {
            let _ = f.write_all(b"retrace,scene_major,scene_minor,match_frame,sim_ms,cpu_ms,sim_only_ms,sim_only_cpu_ms,render_ms,render_cpu_ms\n");
        }
        FrameTimes { rows: Vec::new(), csv, cps, render, start: Instant::now() }
    }

    fn ms(&self, a: Mark, b: Mark) -> (f32, f32) {
        let w = b.wall.duration_since(a.wall).as_secs_f64() * 1e3;
        let c = if self.cps > 0.0 { b.cyc.saturating_sub(a.cyc) as f64 / self.cps * 1e3 } else { w };
        (w as f32, c as f32)
    }

    /// One replay frame: the sim span `a..b`, then the render span `b..c` (== `b` when not rendering).
    pub fn push(&mut self, fr: i32, a: Mark, b: Mark, c: Mark) {
        let (sw, sc) = self.ms(a, b);
        let (rw, rc) = self.ms(b, c);
        if let Some(f) = self.csv.as_mut() {
            // match_frame: the replay frame + 1 (0 for the pre-game frames); sim_ms/cpu_ms are the whole frame
            let _ = writeln!(f, "{},0,0,{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}", self.rows.len(), (fr + 1).max(0), sw + rw, sc + rc, sw, sc, rw, rc);
        }
        self.rows.push(Row { fr, sim_wall: sw, sim_cpu: sc, rnd_wall: rw, rnd_cpu: rc });
    }

    /// The summary block `slpcheck` prints. Frames before replay frame 0 (the pre-game countdown) are left out.
    pub fn summary(&mut self) -> String {
        if let Some(f) = self.csv.as_mut() {
            let _ = f.flush();
        }
        let wall_total = self.start.elapsed().as_secs_f64();
        let sel = |g: &dyn Fn(&Row) -> f32| -> Vec<f32> { self.rows.iter().filter(|r| r.fr >= 0).map(g).collect() };
        let mut out = format!("TIMING frames={} total_wall_s={:.2} clock={}\n", self.rows.iter().filter(|r| r.fr >= 0).count(), wall_total, if self.cps > 0.0 { "thread-cycles+wall" } else { "wall" });
        let mut line = |name: &str, v: Vec<f32>| {
            if v.is_empty() {
                return;
            }
            let mean = v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64;
            let mut s = v.clone();
            s.sort_by(|a, b| a.total_cmp(b));
            let q = |p: f64| s[((s.len() - 1) as f64 * p).round() as usize];
            // histogram: <0.25 <0.5 <1 <2 <4 <8 <16 >=16 ms
            let edges = [0.25f32, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];
            let mut h = [0usize; 8];
            for &x in &v {
                h[edges.iter().position(|&e| x < e).unwrap_or(7)] += 1;
            }
            out.push_str(&format!(
                "TIMING {name:<14} mean {mean:.3} p50 {:.3} p99 {:.3} max {:.3} ms | hist <.25:{} <.5:{} <1:{} <2:{} <4:{} <8:{} <16:{} >=16:{}\n",
                q(0.5), q(0.99), s[s.len() - 1], h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
            ));
        };
        line("sim_cpu_ms", sel(&|r| r.sim_cpu));
        line("sim_wall_ms", sel(&|r| r.sim_wall));
        if self.render {
            line("render_cpu_ms", sel(&|r| r.rnd_cpu));
            line("render_wall_ms", sel(&|r| r.rnd_wall));
            line("frame_cpu_ms", sel(&|r| r.sim_cpu + r.rnd_cpu));
            line("frame_wall_ms", sel(&|r| r.sim_wall + r.rnd_wall));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_frames_and_skips_pregame() {
        let mut t = FrameTimes::new(None, true);
        for fr in -3..10 {
            let a = mark();
            std::thread::sleep(std::time::Duration::from_micros(200));
            let b = mark();
            t.push(fr, a, b, b);
        }
        let s = t.summary();
        assert!(s.contains("TIMING frames=10 "), "{s}");
        assert!(s.contains("sim_cpu_ms") && s.contains("render_cpu_ms") && s.contains("hist"), "{s}");
    }
}

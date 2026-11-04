//! gd-ui-v1 micro-benchmark (wasm-ui-lab-api.md "Budgets"): host CPU per present for a UI mod, steady-state
//! allocations (a counting global allocator) and the quads / draw calls of one frame. No GPU, no game: the present
//! path is everything `PresHost::present` does (scene switch, navigation, events, the mod's gd_ui_frame with its one
//! gd_ui_submit, node layout and drawing, the sort).
//!   cargo run -p melee_modhost --example ui_bench --profile fast -- build/mods/ui_demo.wasm [frames]
use melee_modhost::pres::{PresFrame, Session};
use melee_modhost::ui::PresHost;
use melee_types::PADStatus;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        System.realloc(p, l, n)
    }
}
#[global_allocator]
static A: Counting = Counting;

fn run(h: &mut PresHost, frames: usize, menu: bool) -> (f64, f64, f64, u64, usize) {
    let f = PresFrame { session: Session::Offline, scene: (2, 2), match_frame: 0 };
    let mut ns = Vec::with_capacity(frames);
    let mut allocs = 0;
    for i in 0..frames {
        let mut pads = [PADStatus::zeroed(); 4];
        // D-pad left once opens the demo's menu; then hold down for the host's key repeat (focus moves every 4)
        if menu && i == 1 {
            pads[0].button = 0x1;
        }
        if menu && i > 10 {
            pads[0].button = 0x4;
        }
        let a0 = ALLOCS.load(Relaxed);
        let t = std::time::Instant::now();
        h.input(&mut pads, &f);
        h.present(&f);
        let dt = t.elapsed().as_nanos() as f64 / 1e3;
        let da = ALLOCS.load(Relaxed) - a0;
        if i >= 120 {
            ns.push(dt);
            allocs += da;
        }
    }
    ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| ns[((ns.len() - 1) as f64 * p) as usize];
    (q(0.5), q(0.99), ns[ns.len() - 1], allocs, h.last_quads)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(String::as_str).unwrap_or("../build/mods/ui_demo.wasm");
    let frames: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(3600);
    let bytes = std::fs::read(path).expect("wasm");
    for (name, menu) in [("HUD (Frame data preset)", false), ("HUD + full menu", true)] {
        let mut h = PresHost::new();
        h.set_window(1280, 720);
        h.load(&bytes, std::env::temp_dir()).expect("load");
        let (p50, p99, max, allocs, quads) = run(&mut h, frames, menu);
        let nodes = h.shared.borrow().tree.nodes.len();
        println!("{name}: present p50 {p50:.1} us, p99 {p99:.1} us, max {max:.1} us over {} frames; {allocs} allocations after warm-up; {quads} quads, 1 atlas page -> 1 draw call; {nodes} nodes", frames - 120);
    }
}

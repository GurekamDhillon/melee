//! PADRead pipeline behaviour (shim_pad.c order), the adapter lifecycle with a fake device
//! (the C's gw_gc_test_open tests), Aurora's gamepad arithmetic, the txt pad script, the
//! 60 Hz clock and the headless runner.
use melee_platform::adapter::decode::GC_PAYLOAD_SIZE;
use melee_platform::adapter::{Adapter, Device, ReadResult};
use melee_platform::aurora::{aurora_port, DeadZones, KeyboardBindings, NativeButton, RawGamepad};
use melee_platform::pad::*;
use melee_platform::pad_read::{GamepadSource, InputMode, PadSystem, Source};
use melee_platform::script::PadScript;
use melee_platform::{run_headless, FixedStep, Game};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn quiet() {
    std::env::set_var("MELEE_PLATFORM_QUIET", "1");
}

/// A fake adapter: returns `report` every read (2 ms apart), counts rumble writes.
struct Fake {
    report: Mutex<[u8; GC_PAYLOAD_SIZE]>,
    rumbles: AtomicU32,
    lost: std::sync::atomic::AtomicBool,
}
impl Device for Fake {
    fn read(&self) -> ReadResult {
        std::thread::sleep(Duration::from_millis(2));
        if self.lost.load(Ordering::SeqCst) {
            return ReadResult::Lost;
        }
        ReadResult::Report(*self.report.lock().unwrap())
    }
    fn rumble(&self, _cmd: &[u8; 5]) {
        self.rumbles.fetch_add(1, Ordering::SeqCst);
    }
}

/// Port 1 (index 1) wired, sticks at 128 rest, triggers rest 20; A held when `a`.
fn report(a: bool, sx: u8) -> [u8; GC_PAYLOAD_SIZE] {
    let mut r = [0u8; GC_PAYLOAD_SIZE];
    r[0] = 0x21;
    let p = 1 + 9;
    r[p] = 0x10;
    r[p + 1] = if a { 0x01 } else { 0 };
    r[p + 3] = sx;
    r[p + 4] = 128;
    r[p + 5] = 128;
    r[p + 6] = 128;
    r[p + 7] = 20;
    r[p + 8] = 20;
    r
}

fn fake_adapter() -> (Adapter, Arc<Fake>, Arc<AtomicU32>) {
    let dev = Arc::new(Fake { report: Mutex::new(report(false, 128)), rumbles: AtomicU32::new(0), lost: false.into() });
    let opens = Arc::new(AtomicU32::new(0));
    let (d, o) = (dev.clone(), opens.clone());
    let a = Adapter::with_opener(
        Box::new(move || {
            o.fetch_add(1, Ordering::SeqCst);
            Some(d.clone() as Arc<dyn Device>)
        }),
        false,
    );
    (a, dev, opens)
}

fn wait_report(a: &Adapter) {
    let t = Instant::now();
    while a.report_count() == 0 && t.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn adapter_overrides_gamepad_and_calibrates_on_first_report() {
    quiet();
    let (a, dev, _) = fake_adapter();
    assert!(a.init());
    wait_report(&a);
    let mut gp = RawGamepad::default();
    gp.buttons.insert(NativeButton::East); // B on the gamepad in port 1 and 2
    let mut ps = PadSystem::build(
        InputMode::Devices,
        Some(a),
        GamepadSource::Fixed([None, Some(gp.clone()), Some(gp), None]),
        None,
    );
    ps.set_ignore_adapter(false);
    let mut st = [PADStatus::zeroed(); 4];
    ps.read(&mut st);
    assert_eq!(st[0].err, PAD_ERR_NO_CONTROLLER);
    assert_eq!(ps.sources[1], Source::Adapter);
    assert_eq!(st[1].button, 0, "adapter replaced the gamepad's B");
    assert_eq!(st[1].stickX, 0, "first report is the origin");
    assert_eq!(ps.sources[2], Source::Gamepad);
    assert_eq!(st[2].button, PAD_BUTTON_B);
    *dev.report.lock().unwrap() = report(true, 200);
    std::thread::sleep(Duration::from_millis(20));
    ps.read(&mut st);
    assert_eq!(st[1].button, PAD_BUTTON_A);
    assert_eq!(st[1].stickX, 72);
    assert_eq!(st[1].triggerLeft, 0);
}

#[test]
fn adapter_suspend_resume_and_lost() {
    quiet();
    let (mut a, dev, opens) = fake_adapter();
    assert!(a.init());
    a.start_hotplug();
    assert!(a.suspend());
    assert!(!a.present());
    assert!(!a.init(), "no open while suspended");
    a.resume();
    let t = Instant::now();
    while !a.present() && t.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(a.present(), "scanner reopened after resume");
    assert!(opens.load(Ordering::SeqCst) >= 2);
    // focus loss: reads close the device (gw_gc_allowed)
    a.set_focused(false);
    let mut st = [pad_disconnected(); 4];
    assert!(!a.read(&mut st));
    assert!(!a.present());
    a.set_focused(true);
    assert!(a.init());
    // rumble: only changes are sent
    a.rumble(1, true);
    a.rumble(1, true);
    a.rumble(1, false);
    assert_eq!(dev.rumbles.load(Ordering::SeqCst), 2);
    // device lost -> next read shuts down
    wait_report(&a);
    dev.lost.store(true, Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(20));
    a.read(&mut st);
    assert!(!a.present());
}

#[test]
fn forbidden_adapter_never_opens() {
    quiet();
    let a = Adapter::with_opener(Box::new(|| panic!("opened a forbidden adapter")), true);
    assert!(!a.init());
}

#[test]
fn mode_none_port0_idle_and_script_overrides() {
    quiet();
    let script = PadScript::parse("# test\n2 100 10 -20\n1 0x1000\n", PadScript::parse_channels("02"));
    let mut ps = PadSystem::build(InputMode::None, None, GamepadSource::Fixed(Default::default()), Some(script));
    let mut st = [PADStatus::zeroed(); 4];
    let mut seen = Vec::new();
    for _ in 0..4 {
        ps.read(&mut st);
        seen.push((st[0].button, st[0].stickX, st[0].stickY, st[1].err, st[2].button, st[3].err));
    }
    assert_eq!(seen[0], (0x100, 10, -20, -1, 0x100, -1));
    assert_eq!(seen[1], (0x100, 10, -20, -1, 0x100, -1));
    assert_eq!(seen[2], (0x1000, 0, 0, -1, 0x1000, -1));
    assert_eq!(seen[3], (0x1000, 0, 0, -1, 0x1000, -1), "last segment holds");
    assert_eq!(ps.sources[0], Source::Script);
    assert_eq!(ps.sources[1], Source::None);
}

#[test]
fn mode_none_without_script_is_port0_at_rest() {
    let mut ps = PadSystem::build(InputMode::None, None, GamepadSource::Fixed(Default::default()), None);
    let mut st = [PADStatus::zeroed(); 4];
    ps.read(&mut st);
    assert!(pad_eq(&st[0], &pad_zero()));
    assert_eq!(ps.sources[0], Source::Idle);
    for s in &st[1..] {
        assert!(pad_eq(s, &pad_disconnected()));
    }
}

#[test]
fn reset_combo_fires_on_release() {
    quiet();
    let combo = format!("3 {:x}\n1 0\n", melee_platform::pad_read::GW_PAD_RESET_COMBO);
    let mut ps = PadSystem::build(InputMode::None, None, GamepadSource::Fixed(Default::default()), Some(PadScript::parse(&combo, 1)));
    let mut st = [PADStatus::zeroed(); 4];
    for _ in 0..3 {
        ps.read(&mut st);
    }
    assert_eq!(ps.reset_combo_fired, 0);
    ps.read(&mut st);
    assert_eq!(ps.reset_combo_fired, 1);
}

#[test]
fn script_parse_matches_sscanf_rules() {
    let s = PadScript::parse("  5 1ff 1 2 3 4\n0 100\n-3 100\n7\nx 1\n# c\n2 0x20 5 x 9\n", 1);
    assert_eq!(s.segments.len(), 2);
    assert_eq!((s.segments[0].frames, s.segments[0].buttons, s.segments[0].tr), (5, 0x1ff, 4));
    assert_eq!((s.segments[1].buttons, s.segments[1].sx, s.segments[1].sy, s.segments[1].tl), (0x20, 5, 0, 0));
    assert_eq!(PadScript::parse_channels("xyz"), 1);
    assert_eq!(PadScript::parse_channels("13"), 0b1010);
}

#[test]
fn aurora_gamepad_arithmetic() {
    let dz = DeadZones::default();
    let none = HashSet::new();
    let kb = KeyboardBindings::default();
    // no device, no bindings -> disconnected
    assert_eq!(aurora_port(&none, &kb, None, &dz).0.err, PAD_ERR_NO_CONTROLLER);
    let mut p = RawGamepad { left_x: 32767, left_y: -32768, ..Default::default() };
    let s = aurora_port(&none, &kb, Some(&p), &dz).0;
    assert_eq!((s.stickX, s.stickY), (127, 127), "full right / full up");
    p.left_y = 32767;
    p.left_x = -32768;
    let s = aurora_port(&none, &kb, Some(&p), &dz).0;
    assert_eq!((s.stickX, s.stickY), (-127, -128), "full left / full down (Aurora's unsigned y math)");
    p.left_x = 8000;
    p.left_y = -8000;
    let s = aurora_port(&none, &kb, Some(&p), &dz).0;
    assert_eq!((s.stickX, s.stickY), (0, 0), "inside the 8000 dead zone");
    p.trigger_l = 32767;
    p.trigger_r = 20000;
    let s = aurora_port(&none, &kb, Some(&p), &dz).0;
    assert_eq!(s.button & (PAD_TRIGGER_L | PAD_TRIGGER_R), PAD_TRIGGER_L);
    assert_eq!((s.triggerLeft, s.triggerRight), (180, 156));
    // keyboard: bound keys drive a port with no gamepad
    let kb = KeyboardBindings::preset_default();
    let keys: HashSet<_> = [winit::keyboard::KeyCode::KeyD, winit::keyboard::KeyCode::Space].into();
    let s = aurora_port(&keys, &kb, None, &dz).0;
    assert_eq!((s.err, s.button, s.stickX), (0, PAD_BUTTON_A, 127));
}

#[test]
fn fixed_step_is_exact_60hz() {
    let mut c = FixedStep::default();
    let mut n = 0;
    for _ in 0..1000 {
        n += c.advance(Duration::from_millis(1));
    }
    assert_eq!(n, 60);
    assert_eq!(c.advance(Duration::from_secs(1)), melee_platform::main_loop::MAX_CATCH_UP);
    assert_eq!(c.dropped, 55);
    let mut c = FixedStep::default();
    let mut total = 0u64;
    for _ in 0..600 {
        total += c.advance(Duration::from_nanos(16_666_667)) as u64; // slightly over a tick
    }
    assert_eq!(total, 600);
    assert!(c.until_next() <= Duration::from_nanos(16_666_667));
}

struct Counter {
    frames: u64,
    a_frames: u64,
}
impl Game for Counter {
    fn tick(&mut self, pads: &mut PadSystem) {
        let mut st = [PADStatus::zeroed(); 4];
        pads.read(&mut st);
        if st[0].button & PAD_BUTTON_A != 0 {
            self.a_frames += 1;
        }
        self.frames += 1;
    }
}

#[test]
fn headless_runs_ticks_with_script() {
    let mut ps = PadSystem::build(InputMode::None, None, GamepadSource::Fixed(Default::default()), Some(PadScript::parse("10 100\n1 0\n", 1)));
    let mut g = Counter { frames: 0, a_frames: 0 };
    assert_eq!(run_headless(&mut g, &mut ps, 120), 120);
    assert_eq!((g.frames, g.a_frames), (120, 10));
}

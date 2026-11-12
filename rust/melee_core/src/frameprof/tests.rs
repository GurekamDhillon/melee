use super::writer::{classify, Parts};
use super::*;

fn frame() -> Frame {
    Frame { t_us: 1_000_000, interval_us: 120_000, scene: 0x0203, tick: 100, c: [0; NC], mods: [0; MAX_MODS], interp: true, expected_us: 4167 }
}

#[test]
fn text_builds_without_allocating_and_sanitizes() {
    let t = Text::new().s("file ").b(b"GrNBa.dat\0").s(" ").n(-42).s(" \"x\\");
    assert_eq!(t.as_str(), "file GrNBa.dat -42 ?x?");
    let long = Text::new().s(&"a".repeat(200));
    assert_eq!(long.as_str().len(), TEXT);
}

#[test]
fn classify_picks_the_largest_part_and_its_reason() {
    let f = frame();
    // a long tick that was mostly file loading
    let p = Parts { interval: 120_000.0, expected: 4167.0, tick: 110_000.0, tick_max: 110_000.0, ticks: 1, files: 60_000.0, parse: 20_000.0, ..Default::default() };
    let ev = vec![(Kind::File, "file GrNBa.dat 1200 KB".to_string(), 60_000, 1)];
    assert_eq!(classify(&p, &ev, &f, &[], None).0, "file load");
    // the same with a stage switch event: the stage switch is the cause
    let ev2 = vec![ev[0].clone(), (Kind::Stage, "stage switch to kind 3".to_string(), 0, 3)];
    assert_eq!(classify(&p, &ev2, &f, &[], None).0, "stage switch");
    // new pipelines in a slow prepare
    let mut f2 = frame();
    f2.c[C::Pipes as usize] = 7;
    let p2 = Parts { interval: 40_000.0, expected: 4167.0, prepare: 30_000.0, pipes: 25_000.0, ..Default::default() };
    assert_eq!(classify(&p2, &[], &f2, &[], None).0, "new shaders");
    // nothing timed: outside the game; with other heavy processes running, the machine
    let p3 = Parts { interval: 40_000.0, expected: 4167.0, tick: 1000.0, ticks: 1, tick_max: 1000.0, ..Default::default() };
    assert_eq!(classify(&p3, &[], &f, &[], None).0, "outside the game (OS / driver)");
    let busy = sys::Sample { t_us: 0, cpu: Some(97.0), own: Some(30.0), others: vec![("rustc.exe".into(), 3)] };
    let (c, d) = classify(&p3, &[], &f, &[], Some(&busy));
    assert_eq!(c, "machine busy");
    assert!(d.contains("rustc.exe x3"), "{d}");
    // a mod that took most of the tick
    let mut f4 = frame();
    f4.mods[1] = 50_000_000;
    let p4 = Parts { interval: 70_000.0, expected: 4167.0, tick: 60_000.0, tick_max: 60_000.0, ticks: 1, mods: 50_000.0, ..Default::default() };
    assert_eq!(classify(&p4, &[], &f4, &["envoy_ui".into(), "envoy_sim".into()], None).0, "mod envoy_sim");
    // the induced test hitch wins over everything
    let ev5 = vec![(Kind::Test, "test sleep 100 ms".to_string(), 100_000, 100)];
    assert_eq!(classify(&p4, &ev5, &f4, &[], None).0, "test (induced)");
}

#[test]
fn heavy_process_names() {
    for n in ["melee-v3v.exe", "MELEE.EXE", "cargo.exe", "rustc.exe", "Dolphin.exe"] {
        assert!(sys::heavy(n), "{n}");
    }
    for n in ["explorer.exe", "chrome.exe", "melody.exe"] {
        assert!(!sys::heavy(n), "{n}");
    }
}

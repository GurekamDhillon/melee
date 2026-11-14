use super::cmd::*;
use super::nodes::*;
use super::*;
use melee_gx::ui::{Atlas, Q_IN_B, Q_IN_L, Q_IN_R, Q_IN_T, Q_WINDOW};

/// A tiny encoder for the tests (the mods use gd_ui_enc).
#[derive(Default)]
struct Enc(Vec<u8>);
impl Enc {
    fn rec(&mut self, op: u16, words: &[u32]) -> &mut Self {
        self.0.extend_from_slice(&((op as u32) | ((words.len() as u32) << 16)).to_le_bytes());
        for w in words {
            self.0.extend_from_slice(&w.to_le_bytes());
        }
        self
    }
    fn layer(&mut self, l: i32, z: i32) -> &mut Self {
        self.rec(OP_LAYER, &[l as u32, z as u32])
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: u32) -> &mut Self {
        self.rec(OP_RECT, &[x.to_bits(), y.to_bits(), w.to_bits(), h.to_bits(), c, 0])
    }
    fn text(&mut self, x: f32, y: f32, s: &str) -> &mut Self {
        let mut w = vec![x.to_bits(), y.to_bits(), 2, 0xFFFF_FFFF, 0, 0, s.len() as u32];
        let mut b = s.as_bytes().to_vec();
        while b.len() % 4 != 0 {
            b.push(0);
        }
        w.extend(b.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())));
        self.rec(OP_TEXT, &w)
    }
}

fn atlas() -> Atlas {
    Atlas::new()
}

#[test]
fn rect_and_space() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut e = Enc::default();
    e.rect(1.0, 2.0, 3.0, 4.0, 0xFF0000FF).rec(OP_SPACE, &[SPACE_WINDOW as u32, 2.0f32.to_bits()]).rect(10.0, 10.0, 5.0, 5.0, 0x00FF00FF);
    b.submit(&e.0, 0, &a, &[], false).unwrap();
    b.finish(false);
    let q = &b.frame.quads;
    assert_eq!(q.len(), 2);
    assert_eq!(q[0].rect, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(q[0].flags & Q_WINDOW, 0);
    assert_eq!(q[1].rect, [20.0, 20.0, 10.0, 10.0]);
    assert_eq!(q[1].flags & Q_WINDOW, Q_WINDOW);
    assert_eq!(q[0].rgba, 0xFF0000FF);
}

#[test]
fn first_error_refuses_whole_buffer() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut ok = Enc::default();
    ok.rect(0.0, 0.0, 1.0, 1.0, 0xFF);
    b.submit(&ok.0, 0, &a, &[], false).unwrap();
    let mut bad = Enc::default();
    bad.rect(0.0, 0.0, 1.0, 1.0, 0xFF).rect(0.0, 0.0, 1.0, 1.0, 0xFF).rec(99, &[]);
    let e = b.submit(&bad.0, 1, &a, &[], false).unwrap_err();
    assert_eq!(e.what, "unknown op");
    assert_eq!(e.at, 2 * 28);
    assert_eq!(b.len(), 1, "the refused buffer left nothing behind");
    // truncated, misaligned, bad payload size, bad layer, NaN, bad handle, empty clip stack
    for (buf, what) in [
        (vec![2u8, 0, 6, 0, 0, 0, 0, 0], "record runs past the buffer"),
        (vec![1u8, 2, 3], "length not a multiple of 4"),
        (Enc::default().rec(OP_RECT, &[0; 5]).0.clone(), "payload size"),
        (Enc::default().layer(7, 0).0.clone(), "layer id"),
        (Enc::default().rect(f32::NAN, 0.0, 1.0, 1.0, 0xFF).0.clone(), "rect: non-finite"),
        (Enc::default().rec(OP_TEX, &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]).0.clone(), "tex: bad handle"),
        (Enc::default().rec(OP_UNCLIP, &[]).0.clone(), "unclip: stack empty"),
        (Enc::default().rec(OP_KIT, &[99, 0, 0, 0, 0]).0.clone(), "kit: bad id"),
    ] {
        assert_eq!(b.submit(&buf, 0, &a, &[], false).unwrap_err().what, what);
    }
    let v = Enc::default().rec(OP_VECTOR, &[0; 8]).0.clone();
    assert_eq!(b.submit(&v, 0, &a, &[], false).unwrap_err().code, crate::pres::GD_ECAP);
    assert!(b.submit(&v, 0, &a, &[], true).is_ok());
    assert_eq!(b.len(), 1);
}

#[test]
fn text_records() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut e = Enc::default();
    e.text(0.0, 0.0, "AB C");
    b.submit(&e.0, 0, &a, &[], false).unwrap();
    assert_eq!(b.len(), 3, "spaces draw nothing");
    // the padding must match the length exactly
    let mut bad = e.0.clone();
    bad.extend_from_slice(&[0; 4]);
    bad[2] += 1;
    assert_eq!(b.submit(&bad, 0, &a, &[], false).unwrap_err().what, "payload size");
    let mut e = Enc::default();
    e.text(0.0, 0.0, "\u{0}\u{0}");
    let mut u = e.0.clone();
    u[32] = 0xFF;
    assert_eq!(b.submit(&u, 0, &a, &[], false).unwrap_err().what, "text: not UTF-8");
}

#[test]
fn sort_is_layer_z_mod_submission() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    // mod 1 submits first, on HUD z 0 then MENU; mod 0 later on HUD z 0 and HUD z -1
    let mut m1 = Enc::default();
    m1.layer(LAYER_HUD, 0).rect(0.0, 0.0, 1.0, 1.0, 0x1).layer(LAYER_MENU, 0).rect(0.0, 0.0, 1.0, 1.0, 0x2).layer(LAYER_HUD, 0).rect(0.0, 0.0, 1.0, 1.0, 0x3);
    let mut m0 = Enc::default();
    m0.layer(LAYER_HUD, 0).rect(0.0, 0.0, 1.0, 1.0, 0x4).layer(LAYER_HUD, -1).rect(0.0, 0.0, 1.0, 1.0, 0x5);
    b.submit(&m1.0, 1, &a, &[], false).unwrap();
    b.submit(&m0.0, 0, &a, &[], false).unwrap();
    b.finish(false);
    let order: Vec<u32> = b.frame.quads.iter().map(|q| q.rgba).collect();
    // HUD z-1 (mod 0), HUD z0 mod 0, HUD z0 mod 1 in submission order, then MENU
    assert_eq!(order, vec![0x5, 0x4, 0x1, 0x3, 0x2]);
}

#[test]
fn clip_stack() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut e = Enc::default();
    let c = |x: f32, y: f32, w: f32, h: f32| [x.to_bits(), y.to_bits(), w.to_bits(), h.to_bits()];
    e.rec(OP_CLIP, &c(0.0, 0.0, 100.0, 100.0)).rec(OP_CLIP, &c(50.0, 50.0, 100.0, 100.0)).rect(0.0, 0.0, 1.0, 1.0, 0xFF).rec(OP_UNCLIP, &[]).rect(0.0, 0.0, 1.0, 1.0, 0xFF);
    b.submit(&e.0, 0, &a, &[], false).unwrap();
    b.finish(false);
    assert_eq!(b.frame.quads[0].clip, [50.0, 50.0, 100.0, 100.0]);
    assert_eq!(b.frame.quads[1].clip, [0.0, 0.0, 100.0, 100.0]);
    let mut deep = Enc::default();
    for _ in 0..9 {
        deep.rec(OP_CLIP, &c(0.0, 0.0, 1.0, 1.0));
    }
    assert_eq!(b.submit(&deep.0, 0, &a, &[], false).unwrap_err().what, "clip: stack depth 8");
    // a space change inside a clip is refused (the clip is in the old space)
    let mut sp = Enc::default();
    sp.rec(OP_CLIP, &c(0.0, 0.0, 1.0, 1.0)).rec(OP_SPACE, &[1, 1.0f32.to_bits()]);
    assert!(b.submit(&sp.0, 0, &a, &[], false).is_err());
}

#[test]
fn nine_slice_is_nine_quads_and_one_page() {
    let a = atlas();
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut e = Enc::default();
    e.rec(OP_SLICE, &[1, 10.0f32.to_bits(), 10.0f32.to_bits(), 100.0f32.to_bits(), 40.0f32.to_bits(), 0xFFFFFFFF]);
    b.submit(&e.0, 0, &a, &[], false).unwrap();
    b.finish(false);
    assert_eq!(b.frame.quads.len(), 9);
    let x1: f32 = b.frame.quads.iter().map(|q| q.rect[0] + q.rect[2]).fold(0.0, f32::max);
    assert_eq!(x1, 110.0);
    // every uv is inside the single atlas page
    assert!(b.frame.quads.iter().all(|q| q.uv.iter().all(|&u| (0.0..=1.0).contains(&u))));
}

/// uiseams: a rotated 9-slice's pieces mark their shared sides inner (no edge AA there), the outer sides stay AA'd;
/// unrotated the same (harmless: no AA there); all nine share the first piece's tilt depth.
#[test]
fn rotated_nine_slice_marks_inner_sides() {
    let a = atlas();
    let ins = Q_IN_L | Q_IN_T | Q_IN_R | Q_IN_B;
    for rot in [0.0f32, 0.05] {
        let mut b = Batch::with_capacity(64);
        b.begin();
        let mut e = Enc::default();
        e.rec(OP_ROT, &[rot.to_bits(), 60.0f32.to_bits(), 30.0f32.to_bits()]);
        e.rec(OP_SLICE, &[2, 10.0f32.to_bits(), 10.0f32.to_bits(), 100.0f32.to_bits(), 40.0f32.to_bits(), 0xFFFFFFFF]);
        b.submit(&e.0, 0, &a, &[], false).unwrap();
        b.finish(false);
        let f: Vec<u32> = b.frame.quads.iter().map(|q| q.flags & ins).collect();
        // every piece points back to the first (the tilt's shared parallax depth)
        let back: Vec<u32> = b.frame.quads.iter().map(|q| (q.flags >> 8) & 0xFF).collect();
        assert_eq!(back, (0..9).collect::<Vec<u32>>());
        // row-major 3x3: corners have two inner sides, edges three, the centre four
        assert_eq!(f[0], Q_IN_R | Q_IN_B);
        assert_eq!(f[1], Q_IN_L | Q_IN_R | Q_IN_B);
        assert_eq!(f[4], ins);
        assert_eq!(f[8], Q_IN_L | Q_IN_T);
    }
}

#[test]
fn steady_state_reuses_buffers() {
    let a = atlas();
    let mut b = Batch::with_capacity(16);
    let mut e = Enc::default();
    for i in 0..200 {
        e.layer(LAYER_HUD, i % 3).rect(0.0, 0.0, 1.0, 1.0, 0xFF);
    }
    let mut ptrs = None;
    for f in 0..10 {
        b.begin();
        b.submit(&e.0, 0, &a, &[], false).unwrap();
        b.finish(false);
        let p = (b.frame.quads.as_ptr(), b.frame.quads.capacity());
        if f >= 1 {
            assert_eq!(Some(p), ptrs, "no reallocation after the high-water frame");
        }
        ptrs = Some(p);
    }
}

fn shared() -> Shared {
    let mut s = Shared::new();
    s.features.push(FEATURE_UI_INPUT);
    s
}

#[test]
fn capture_rules() {
    let mut s = shared();
    assert!(s.capture(0, 1, CAP_EXCLUSIVE).is_ok());
    assert_eq!(s.masked_ports(), 1);
    assert_eq!(s.capture(0, 1, CAP_INJECT), Err(GD_ECAP), "INJECT needs UI_SIM_INPUT");
    assert_eq!(s.capture(0, 1, CAP_EXCLUSIVE | CAP_SHARED), Err(GD_EINVAL));
    assert!(s.capture(0, 0, CAP_KEYBOARD).is_ok());
    s.caps.clear();
    // normal netplay: KEYBOARD refused, EXCLUSIVE refused in a match, allowed out of it
    s.session = Session::Netplay;
    assert_eq!(s.capture(0, 0, CAP_KEYBOARD), Err(GD_EOFFLINE));
    s.in_match = true;
    assert_eq!(s.capture(0, 1, CAP_EXCLUSIVE), Err(GD_EOFFLINE));
    s.in_match = false;
    assert!(s.capture(0, 1, CAP_EXCLUSIVE).is_ok());
    s.in_match = true;
    assert_eq!(s.masked_ports(), 0, "never masks a netplay match");
    s.caps.clear();
    // GAP-14: an online Lab session takes the keyboard (local only)
    s.session = Session::LabNetplay;
    assert!(s.capture(0, 0, CAP_KEYBOARD).is_ok());
    assert!(s.keyboard_wanted());
    s.session = Session::Synctest;
    assert_eq!(s.capture(0, 0, CAP_KEYBOARD), Err(GD_EOFFLINE));
}

#[test]
fn node_navigation() {
    let mut t = Tree::default();
    let root = t.create(0, 0, K_PANEL, 1);
    let tabs = t.create(0, root, K_TABS, 2);
    t.get_mut(tabs, 0).unwrap().choices = vec!["A".into(), "B".into(), "C".into()];
    let list = t.create(0, root, K_LIST, 3);
    let b1 = t.create(0, list, K_BUTTON, 10);
    let tg = t.create(0, list, K_TOGGLE, 11);
    let sl = t.create(0, list, K_SLIDER, 12);
    t.get_mut(sl, 0).unwrap().range = (0, 10, 2);
    assert_eq!(t.create(0, 999, K_BUTTON, 0), 0, "bad parent");
    assert_eq!(t.create(1, root, K_BUTTON, 0), 0, "another mod's node");
    assert!(t.show(0, root, LAYER_MENU, 0));
    assert_eq!(t.nav[0].focus, b1);
    // down: a fresh press moves, holding repeats only after the delay
    t.navigate(0, PAD_DOWN, PAD_DOWN, 0, 0);
    assert_eq!(t.nav[0].focus, tg);
    for _ in 0..REPEAT_DELAY - 1 {
        t.navigate(0, PAD_DOWN, 0, 0, 0);
    }
    assert_eq!(t.nav[0].focus, tg);
    t.navigate(0, PAD_DOWN, 0, 0, 0);
    assert_eq!(t.nav[0].focus, sl);
    t.navigate(0, 0, 0, 0, 0);
    // wrap from the last row to the first
    t.navigate(0, PAD_DOWN, PAD_DOWN, 0, 0);
    assert_eq!(t.nav[0].focus, b1);
    t.navigate(0, 0, 0, 0, 0);
    t.navigate(0, PAD_UP, PAD_UP, 0, 0);
    assert_eq!(t.nav[0].focus, sl);
    t.navigate(0, 0, 0, 0, 0);
    // slider: right steps by 2, clamped
    t.navigate(0, PAD_RIGHT, PAD_RIGHT, 0, 0);
    assert_eq!(t.get(sl, 0).unwrap().value, 2);
    t.events.clear();
    t.navigate(0, PAD_A | PAD_R, PAD_A | PAD_R, 0, 0);
    assert!(t.events.contains(&(0, sl, EV_ACTIVATE, 2)));
    assert!(t.events.contains(&(0, tabs, EV_CHANGE, 1)));
    t.navigate(0, PAD_B, PAD_B, 0, 0);
    assert!(t.events.iter().any(|e| e.2 == EV_BACK));
    // NO_WRAP stops at the ends
    t.get_mut(list, 0).unwrap().flags = F_NO_WRAP;
    t.navigate(0, 0, 0, 0, 0);
    t.navigate(0, PAD_DOWN, PAD_DOWN, 0, 0);
    assert_eq!(t.nav[0].focus, sl);
    // layout and drawing
    t.layout();
    let a = atlas();
    let mut b = Batch::with_capacity(512);
    b.begin();
    t.draw(&mut b, &a, theme::THEME_A);
    assert!(b.len() > 20);
    t.free_subtree(root, 0);
    assert!(t.shown.is_empty() && t.nav[0].focus == 0);
}

const WAT: &str = r#"
(module
  (import "gd_ui" "gd_ui_submit" (func $submit (param i32 i32) (result i32)))
  (import "gd_ui" "gd_ui_capture" (func $cap (param i32 i32) (result i32)))
  (memory (export "memory") 1 1)
  (global (export "gd_mod_manifest") i32 (i32.const 256))
  (global (export "gdmod_ui_abi") i32 (i32.const 0x10000))
  (global $n (mut i32) (i32.const 0))
  ;; one RECT record at 0: op 2, 6 words
  (data (i32.const 0) "\02\00\06\00\00\00\20\41\00\00\20\41\00\00\a0\41\00\00\a0\41\ff\00\00\ff\00\00\00\00")
  (data (i32.const 256) "GDMD\7c\00\00\00\00\00\01\00\00\00\01\00\01\00\00\00\00\00\01\00\50\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00ui.test")
  (func (export "gd_ui_init") (result i32) (drop (call $cap (i32.const 1) (i32.const 1))) (i32.const 0))
  (func (export "gd_ui_frame") (param i32)
    (global.set $n (i32.add (global.get $n) (i32.const 1)))
    (drop (call $submit (i32.const 0) (i32.const 28))))
)"#;

#[test]
fn end_to_end_wat_mod() {
    let wasm = wat::parse_str(WAT).unwrap();
    assert!(is_presentation(&wasm));
    let mut h = PresHost::new();
    h.load(&wasm, std::env::temp_dir()).unwrap();
    assert_eq!(h.mods[0].id, "ui.test");
    let f = PresFrame { session: Session::Offline, scene: (2, 2), match_frame: 0 };
    let mut pads = [PADStatus::zeroed(); 4];
    pads[0].button = PAD_A;
    pads[1].button = PAD_B;
    h.input(&mut pads, &f);
    assert_eq!(pads[0].button, 0, "port 1 is captured exclusively");
    assert_eq!(pads[1].button, PAD_B);
    h.present(&f);
    h.with_frame(|fr, _| {
        assert_eq!(fr.quads.len(), 1);
        assert_eq!(fr.quads[0].rect, [10.0, 10.0, 20.0, 20.0]);
        assert_eq!(fr.quads[0].rgba, 0xFF0000FF);
    });
    // the sim host refuses gd_ui imports, so install_dir skips such a module
    assert!(crate::Host::new().load(&wasm).is_err());
}

#[test]
fn text_styles_v1_1() {
    let a = atlas();
    // v1.0 roles resolve exactly as before
    assert_eq!(style(2), melee_gx::ui_font::TextStyle::new(13.0, false, false));
    assert!(style(0).upper && style(3).face == melee_gx::ui_font::FACE_SANS_BOLD);
    assert_eq!(style(99).size, ROLES[5].0, "out-of-range roles clamp");
    let s = style(2 | TEXT_MONO | TEXT_BOLD | 20 << 16);
    assert_eq!((s.size, s.face), (20.0, melee_gx::ui_font::FACE_MONO_BOLD));
    // mono: every char the same advance; sans is proportional and kerned
    let m = |t: &str, r: i32| a.measure(t, style(r));
    assert!((m("iiii", 2 | TEXT_MONO) - m("WWWW", 2 | TEXT_MONO)).abs() < 1e-3);
    assert!(m("iiii", 2) < m("WWWW", 2));
    assert!(m("AV", 2) < m("A", 2) + m("V", 2), "AV is kerned");
    // max_w cuts at a glyph boundary
    let mut b = Batch::with_capacity(64);
    b.begin();
    let mut e = Enc::default();
    e.text(0.0, 0.0, "AB C");
    b.submit(&e.0, 0, &a, &[], false).unwrap();
    b.finish(false);
    assert!(b.frame.quads.iter().all(|q| q.flags & melee_gx::ui::Q_SDF != 0));
}

/// Cost of ~200 glyphs per frame through the command path (validate + layout + quads), and the atlas build.
#[test]
fn text_perf_200_glyphs() {
    let t0 = std::time::Instant::now();
    let a = atlas();
    let build = t0.elapsed();
    let mut e = Enc::default();
    for i in 0..12 {
        e.text(10.0, 20.0 * i as f32, "Frame 123 / Hitstun 45 ");
    }
    let mut b = Batch::with_capacity(4096);
    let n = 2000;
    let t = std::time::Instant::now();
    for _ in 0..n {
        b.begin();
        b.submit(&e.0, 0, &a, &[], false).unwrap();
        b.finish(false);
    }
    let per = t.elapsed().as_secs_f64() * 1000.0 / n as f64;
    eprintln!("UI_TEXT_PERF atlas_build_ms={:.1} glyph_quads={} ms_per_frame={:.4}", build.as_secs_f64() * 1000.0, b.frame.quads.len(), per);
    assert!(b.frame.quads.len() >= 190);
}

const WAT_HOST: &str = r#"
(module
  (import "gd_ui" "gd_ui_list_info" (func $info (param i32) (result i32)))
  (import "gd_ui" "gd_ui_list_get" (func $get (param i32 i32 i32 i32) (result i32)))
  (import "gd_ui" "gd_ui_request" (func $req (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1 1)
  (global (export "gd_mod_manifest") i32 (i32.const 256))
  (global (export "gdmod_ui_abi") i32 (i32.const 0x10002))
  (global $info_v (export "info_v") (mut i32) (i32.const 0))
  (global $got (export "got") (mut i32) (i32.const 0))
  (data (i32.const 0) "lab")
  (data (i32.const 256) "GDMD\7c\00\00\00\00\00\01\00\00\00\01\00\01\00\00\00\00\00\01\00\00\00\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00ui.host")
  (func (export "gd_ui_frame") (param i32)
    (global.set $info_v (call $info (i32.const 0)))
    (global.set $got (call $get (i32.const 0) (i32.const 1) (i32.const 512) (i32.const 3)))
    (drop (call $req (i32.const 1) (i32.const 0) (i32.const 3))))
)"#;

#[test]
fn host_catalogs_and_requests_v1_2() {
    let wasm = wat::parse_str(WAT_HOST).unwrap();
    let mut h = PresHost::new();
    h.load(&wasm, std::env::temp_dir()).unwrap();
    assert!(h.wants_host());
    h.set_list(LIST_MODES, vec![b"a\0A".to_vec(), b"lab\0Geno\0LAB".to_vec()]);
    let f = PresFrame { session: Session::Offline, scene: (1, 1), match_frame: 0 };
    h.present(&f);
    let reqs = h.take_requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!((reqs[0].op, reqs[0].arg.as_str()), (REQ_LAUNCH, "lab"));
    // the record was copied (truncated to cap 3) into the mod's memory at 512
    let m = &h.mods[0];
    let mem = m.store.data().mem.unwrap();
    assert_eq!(&mem.data(&m.store)[512..515], b"lab");
    assert!(h.take_requests().is_empty());
    // a mod without UI_HOST gets GD_ECAP; a bad op is refused
    let s = h.shared.borrow();
    assert_eq!(s.list_gen[LIST_MODES], 1);
    assert_eq!(s.lists[LIST_MODES].len(), 2);
}

//! Ports of the controls tests: pc/tests/fighter_controls_test.c (schema_parse and the pure model of
//! preference_and_freeze) and pc/tests/fighter_controls_registry_tests.inc (test_geno_controls_schema /
//! test_geno_controls_lookup). Same inputs, same expectations.
use melee_script::geno::controls::{self as fc, Schema};
use melee_script::geno::json::Doc;
use melee_script::geno::{NoEnv, Registry};

fn parse(text: &str) -> Option<Schema> {
    let (d, root) = Doc::parse(text.as_bytes());
    if root < 0 {
        return None; // a document the reader refuses (a repeated key) refuses the block too
    }
    fc::parse(&d, root)
}

const SWAP: &str = "{\"layouts\":[{\"name\":\"Swap attacks\",\"buttons\":{\"A\":\"B\",\"B\":\"A\",\"Y\":\"B\"}},{\"name\":\"Standard\",\"buttons\":{\"A\":\"A\",\"B\":\"B\"}}],\"extras\":[{\"name\":\"Y boost\",\"buttons\":{\"Y\":[\"A\",\"B\"]}}]}";

#[test]
fn schema_parse() {
    let s = parse(SWAP).expect("valid");
    assert_eq!((s.layout.len(), s.extra.len()), (2, 1));
    assert_eq!((s.layout[0].name.as_str(), s.layout[1].name.as_str()), ("Swap attacks", "Standard"));
    for bad in [
        "[]",
        "{\"layouts\":true}",
        "{\"layouts\":[{\"name\":\"\",\"buttons\":{\"X\":\"A\"}}]}",
        "{\"layouts\":[{\"name\":\"Bad\",\"buttons\":{\"W\":\"A\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":\"boost\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":[]}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":[\"A\",\"A\"]}}]}",
        "{\"extras\":[{\"name\":\"Same\",\"buttons\":{\"Y\":\"A\"}},{\"name\":\"Same\",\"buttons\":{\"X\":\"B\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\\nName\",\"buttons\":{\"Y\":\"A\"}}]}",
        "{\"unknown\":1}",
        "{\"extras\":[],\"extras\":[]}",
    ] {
        assert_eq!(parse(bad), None, "{bad}");
    }
    let s = parse("{\"extras\":[{\"name\":\"Consume Y\",\"buttons\":{\"Y\":\"none\"}}]}").unwrap();
    assert_eq!((s.layout.len(), s.extra.len(), s.extra[0].dest[3]), (0, 1, 0));
    // the checked-in fixture parses and its layout 0 is not the identity
    let t = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../pc/tests/fixtures/controls-fighter/geno.json")).unwrap();
    let (d, root) = Doc::parse(&t);
    let entry = d.n[d.get(root, "fighters") as usize].first;
    let s = fc::parse(&d, d.get(entry, "controls")).unwrap();
    assert_eq!((s.layout.len(), s.extra.len(), s.layout[0].dest[0]), (2, 1, 0x200));
    // the packed word holds 16 layouts and 8 extras; one more refuses the block (never a silent truncation)
    for (n, ok) in [(16, true), (17, false)] {
        let rows: Vec<String> = (0..n).map(|i| format!("{{\"name\":\"L{i}\",\"buttons\":{{\"A\":\"B\"}}}}")).collect();
        let s = parse(&format!("{{\"layouts\":[{}]}}", rows.join(",")));
        assert_eq!(s.map(|s| s.layout.len()), ok.then_some(16));
    }
    for (n, ok) in [(8, true), (9, false)] {
        let rows: Vec<String> = (0..n).map(|i| format!("{{\"name\":\"E{i}\",\"buttons\":{{\"Y\":\"A\"}}}}")).collect();
        let s = parse(&format!("{{\"extras\":[{}]}}", rows.join(",")));
        assert_eq!(s.map(|s| s.extra.len()), ok.then_some(8));
    }
}

#[test]
fn build_and_apply() {
    let s0 = parse(SWAP).unwrap();
    // the first named layout is the fighter's default; Tap jump off changes no button
    let m = fc::build(Some(&s0), 0);
    assert_eq!(fc::apply(&m, 0x100), 0x200);
    let m = fc::build(Some(&s0), 1);
    assert_eq!((fc::apply(&m, 0x800), fc::apply(&m, 0x100), fc::apply(&m, 0x200), fc::apply(&m, 0x400)), (0x200, 0x200, 0x100, 0x400));
    // Standard layout: Y stays Y; the Y boost extra
    assert_eq!(fc::apply(&fc::build(Some(&s0), 1 | 4), 0x800), 0x800);
    assert_eq!(fc::apply(&fc::build(Some(&s0), 1 | 0x100), 0x800), 0x300);
    // maps read the original input once: X->Y, Y->A never cascades
    let mut s1 = s0.clone();
    s1.layout[0].dest[2] = 0x800;
    s1.layout[0].dest[3] = 0x100;
    s1.layout[0].specified |= (1 << 2) | (1 << 3);
    let m = fc::build(Some(&s1), 0);
    assert_eq!((fc::apply(&m, 0x400), fc::apply(&m, 0xc00)), (0x800, 0x900));
    s1.layout[0].dest[0] = 0x400; // explicit fighter A -> jump
    assert_eq!(fc::apply(&fc::build(Some(&s1), 0), 0x100), 0x400);
    assert_eq!(fc::normalize(Some(&s0), 0xffff_ffff), 0x101);
    // the last enabled extra wins
    let mut e = s1.extra[0].clone();
    e.dest[3] = 0x10;
    s1.extra.push(e);
    assert_eq!(fc::apply(&fc::build(Some(&s1), 0x300), 0x800), 0x10);
    // no schema: the identity, any word normalizes to Tap jump alone; bits outside the map pass through
    let m = fc::build(None, 0xffff);
    for bits in [0x100, 0x1f7f, 0x8000_0000u32 as i32 | 0x100, 0x80] {
        assert_eq!(fc::apply(&m, bits), bits);
    }
    assert_eq!(fc::normalize(None, 0xffff), 1);
}

const DEFINE: &str = "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"controls-fixture\",\"name\":\"Controls Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"controls\":";

#[test]
fn registry_schema_and_identity() {
    let mut r = Registry::default();
    let good = format!("{DEFINE}{{\"layouts\":[{{\"name\":\"Swap attacks\",\"buttons\":{{\"A\":\"B\",\"B\":\"A\",\"Y\":\"B\"}}}}],\"extras\":[{{\"name\":\"Y boost\",\"buttons\":{{\"Y\":[\"A\",\"B\"]}}}}]}}}}]}}");
    assert_eq!(r.parse_text(good.as_bytes(), "controls-test", &NoEnv), 1, "{:?}", r.log);
    let p = &r.p[0];
    assert_eq!((p.controls.layout.len(), p.controls.extra.len()), (1, 1));
    let good_id = p.id;
    let m = fc::build(Some(&p.controls), 0);
    assert_eq!((fc::apply(&m, 0x100), fc::apply(&m, 0x800)), (0x200, 0x200));
    assert_eq!(fc::apply(&fc::build(Some(&p.controls), 1 | 0x100), 0x800), 0x300);
    // a bad block falls back atomically (the fighter still loads) and the controls salt the identity
    let mut r = Registry::default();
    let bad = format!("{DEFINE}{{\"layouts\":[{{\"name\":\"Broken\",\"buttons\":{{\"Y\":\"boost\"}}}}]}}}}]}}");
    assert_eq!(r.parse_text(bad.as_bytes(), "controls-test", &NoEnv), 1);
    assert!(!r.p[0].controls.any() && r.p[0].id != good_id);
    assert!(r.log.iter().any(|l| l.contains("malformed controls - using Both/Default/no extras")), "{:?}", r.log);
    // an attach entry's controls are ignored with a log
    let mut r = Registry::default();
    r.parse_text(b"{\"geno\":6,\"fighters\":[{\"attach\":\"kirby\",\"controls\":{\"extras\":[]}}]}", "controls-test", &NoEnv);
    assert!(!r.p[0].controls.any() && r.log.iter().any(|l| l.contains("controls requires a define - ignored")));
}

#[test]
fn registry_lookup() {
    let mut r = Registry::default();
    let t = format!("{DEFINE}{{\"extras\":[{{\"name\":\"Y boost\",\"buttons\":{{\"Y\":[\"A\",\"B\"]}}}}]}}}}]}}");
    assert_eq!(r.parse_text(t.as_bytes(), "controls-test", &NoEnv), 1);
    r.build_kinds(&NoEnv);
    let ck = r.define_resolve("controls-fixture");
    let s = r.controls_schema(ck).expect("the define's schema");
    assert!(ck >= 34 && s.layout.is_empty() && s.extra.len() == 1, "ck {ck}");
    assert!(r.controls_schema(8).is_none() && r.controls_schema(-1).is_none() && r.controls_schema(255).is_none());
}

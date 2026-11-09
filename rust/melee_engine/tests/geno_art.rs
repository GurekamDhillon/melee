//! A define's menu / results / HUD art and the skins installed for it, through the engine's Geno host: ports of
//! geno_define_tests.inc test_geno_define_presentation_ck and test_geno_gxtex_art (the handle half),
//! pc/tests/geno_css_art_registry.inc (the CK half), and the skin / results paths of gw_Geno_DefineArtOpen,
//! gw_Geno_DefineHasArt, gw_Geno_DefineCostumeCount(CK), gw_Geno_DefineCostumeName and gw_Geno_DefineString.
//! Its own test binary: the registry is process-wide, and the lib's tests swap it too. No disc.
#![allow(non_snake_case)]

/// The registry, the skin registry and the handle table are process-wide: one test at a time.
static REG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
use melee_engine::geno_host::*;
use melee_engine::skins_host as sk;

fn none_t9(cos: &str, entry: &str) -> std::ffi::CString {
    std::ffi::CString::new(format!(
        "{{\"geno\":9,\"fighters\":[{{\"define\":{{\"key\":\"t9\",\"name\":\"T9\",\"base\":\"none\",\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"}},\"fighter\":{{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[{cos}]}}{entry}}}]}}"
    ))
    .unwrap()
}
fn cos(file: &str, extra: &str) -> String {
    format!("{{\"file\":\"{file}\",\"joint\":\"j\"{extra}}}")
}
fn name(ck: i32, c: i32) -> Option<String> {
    let mut b = [0i8; 32];
    (DefineCostumeName(ck, c, b.as_mut_ptr(), 32) != 0)
        .then(|| unsafe { core::ffi::CStr::from_ptr(b.as_ptr()) }.to_string_lossy().into_owned())
}
/// A v1 .gxtex: `fmt`, 4x4, a 32-byte image at 64 (a TLUT of 32 bytes at 96 when `tlut`).
fn gxtex(fmt: u8, tlut: bool) -> Vec<u8> {
    let mut b = vec![0u8; if tlut { 128 } else { 96 }];
    b[0..4].copy_from_slice(b"GXTX");
    b[7] = 1;
    b[11] = fmt;
    b[15] = 4;
    b[19] = 4;
    b[20..24].fill(0xFF);
    b[31] = 32;
    b[39] = 64;
    if tlut {
        b[20..24].copy_from_slice(&[0, 0, 0, 2]);
        b[27] = 16;
        b[35] = 32;
        b[43] = 96;
    }
    b[64] = 0xFC;
    b
}

#[test]
fn presentation_ck_like_c() {
    let _g = REG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    sk::Suspend(true);
    let t = none_t9(
        &[cos("a.dat", ",\"name\":\"Plain\""), cos("b.dat", ",\"name\":\"Ember\",\"team\":\"red\""), cos("c.dat", ",\"team\":\"blue\""), cos("d.dat", ",\"team\":\"green\"")].join(","),
        ",\"presentation\":{\"icon\":\"gn_absent_icon.gxtex\",\"stock\":[\"gn_absent_s0.gxtex\"]}",
    );
    assert_eq!(TestInstall(t.as_ptr()), 1);
    let ck = DefineResolve("t9");
    assert!(ck >= 34, "ck {ck}");
    assert_eq!((DefineCostumeCountCK(ck), DefineTeamCostume(ck, 0), DefineTeamCostume(ck, 1), DefineTeamCostume(ck, 2)), (4, 1, 2, 3));
    assert_eq!((name(ck, 1).as_deref(), name(ck, 2), name(ck, 4)), (Some("Ember"), None, None));
    assert_eq!((DefineHasArt(ck, 0, 0), DefineHasArt(ck, 1, 0), DefineHasArt(ck, 2, 3)), (1, 0, 1));
    assert_eq!(DefineArtOpen(ck, 2, 0), -1, "a missing art file answers -1");
    assert_eq!((DefineCostumeCountCK(8), DefineCostumeCountCK(0), DefineCostumeCountCK(255), DefineTeamCostume(8, 0), DefineHasArt(8, 0, 0)), (0, 0, 0, -1, 0));
    TestRestore();
    sk::Suspend(false);
}

/// geno_css_art_registry.inc: the art donor of each template, nothing fabricated for a costume past the lists,
/// no donor for a non-define.
#[test]
fn css_art_registry_like_c() {
    let _g = REG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    sk::Suspend(true);
    let head = |v: i32, extra: &str| format!("{{\"geno\":{v},\"fighters\":[{{\"define\":{{\"key\":\"t7\",\"name\":\"T7\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}{extra}}}]}}");
    let tmpl = |t: &str| format!("{{\"geno\":10,\"fighters\":[{{\"define\":{{\"key\":\"tc\",\"name\":\"TC\",\"base\":\"none\",{t}\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"}},\"fighter\":{{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[{}]}}}}]}}", cos("a.dat", ""));
    let cases = [(head(9, ""), 8), (tmpl("\"template\":\"captain\","), 0), (tmpl("\"template\":\"mario\","), 8), (tmpl(""), -1)];
    for (i, (text, base)) in cases.iter().enumerate() {
        let t = std::ffi::CString::new(text.as_str()).unwrap();
        assert_eq!(TestInstall(t.as_ptr()), 1, "fixture {i}");
        let ck = DefineCKAt(0);
        assert!(ck >= 34 && DefineArtBaseCK(ck) == *base, "fixture {i}: ck {ck} base {}", DefineArtBaseCK(ck));
        for what in 0..5 {
            assert_eq!((DefineHasArt(ck, what, 254), DefineArtOpen(ck, what, 254)), (0, -1), "fixture {i}: art {what} fabricated");
        }
        TestRestore();
    }
    // results: its own list per costume, a costume past it the first; the portrait supplies results without one
    let t = std::ffi::CString::new(head(9, ",\"presentation\":{\"icon\":\"i.gxtex\",\"portrait\":\"p.gxtex\",\"stock\":\"s.gxtex\",\"results\":[\"r0.gxtex\",\"r1.gxtex\"]}")).unwrap();
    assert_eq!(TestInstall(t.as_ptr()), 1);
    let ck = DefineCKAt(0);
    let f = |w, c| DefinePresFile(ck, w, c);
    assert_eq!((f(4, 1).as_deref(), f(4, 254).as_deref(), f(1, 1).as_deref()), (Some("r1.gxtex"), Some("r0.gxtex"), Some("p.gxtex")));
    TestRestore();
    let t = std::ffi::CString::new(head(9, ",\"presentation\":{\"portrait\":\"p.gxtex\"}")).unwrap();
    assert_eq!(TestInstall(t.as_ptr()), 1);
    assert_eq!(DefinePresFile(DefineCKAt(0), 4, 0).as_deref(), Some("p.gxtex"));
    TestRestore();
    for ck in [0, 8, -1, 255] {
        assert_eq!(DefineArtBaseCK(ck), -1);
    }
    sk::Suspend(false);
}

/// test_geno_gxtex_art's handle half: gw_GxTex_OpenBlob and the queries, the refusals.
#[test]
fn gxtex_handles_like_c() {
    let _g = REG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut b = gxtex(5, false);
    let g = GxTex_OpenBlob(b.clone(), "gn:test");
    assert!(g >= 0);
    assert_eq!((GxTex_Width(g), GxTex_Height(g), GxTex_Format(g), GxTex_ImageSize(g), GxTex_TlutFormat(g)), (4, 4, 5, 32, -1));
    let mut copy = [0u8; 32];
    GxTex_CopyImage(g, copy.as_mut_ptr() as _);
    assert_eq!((copy[0], copy[1]), (0xFC, 0));
    GxTex_Close(g);
    assert_eq!(GxTex_Width(g), 0, "a closed handle still answers");
    assert_eq!(GxTex_OpenBlob(b[..20].to_vec(), "gn:short"), -1);
    b[0] = b'X';
    assert_eq!(GxTex_OpenBlob(b, "gn:magic"), -1);
    let p = GxTex_OpenBlob(gxtex(9, true), "gn:ci8");
    assert_eq!((GxTex_TlutFormat(p), GxTex_TlutEntries(p), GxTex_TlutSize(p)), (2, 16, 32));
    GxTex_Close(p);
}

/// gw_Geno_DefineArtOpen's order: a paletted results override falls through to the portrait (skin CSP, then the
/// package's); stock refuses a palette; a skin costume's own CSP, name, file and count come first.
#[test]
fn results_fallback_and_skins_like_c() {
    let _g = REG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let d = std::env::temp_dir().join(format!("genoart-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("test/files")).unwrap();
    std::fs::create_dir_all(d.join("skin/files")).unwrap();
    std::fs::write(d.join("test/files/r.gxtex"), gxtex(9, true)).unwrap();
    std::fs::write(d.join("test/files/p.gxtex"), gxtex(9, true)).unwrap();
    std::fs::write(d.join("test/files/s.gxtex"), gxtex(8, true)).unwrap();
    std::fs::write(d.join("test/files/p5.gxtex"), gxtex(5, false)).unwrap();
    std::fs::write(d.join("skin/files/x.dat"), "x").unwrap();
    std::fs::write(d.join("skin/files/csp.gxtex"), gxtex(6, false)).unwrap();
    set_boot_mods(&d, &[]);
    let t = none_t9(
        &[cos("a.dat", ""), cos("b.dat", "")].join(","),
        ",\"presentation\":{\"portrait\":[\"p.gxtex\",\"p5.gxtex\"],\"stock\":\"s.gxtex\",\"results\":\"r.gxtex\"}",
    );
    assert_eq!(TestInstall(t.as_ptr()), 1);
    let ck = DefineResolve("t9");
    assert!(ck >= 34, "ck {ck}");
    let h = DefineArtOpen(ck, 4, 1);
    assert_eq!(GxTex_Format(h), 5, "a paletted results override falls through to the (unpaletted) portrait");
    GxTex_Close(h);
    assert_eq!(DefineArtOpen(ck, 4, 0), -1, "results: a paletted portrait is refused too (the caller tries the donor / letters)");
    assert_eq!(DefineArtOpen(ck, 2, 0), -1, "a paletted stock icon is refused");
    let h = DefineArtOpen(ck, 1, 0);
    assert_eq!(GxTex_Format(h), 9, "the portrait (CSS) keeps its palette");
    GxTex_Close(h);
    assert_eq!((DefineCostumeCount(ck - 1), DefineCostumeCountCK(ck)), (2, 2));

    // a skin pack targeting the define: costume 2 after its own two
    let mut r = sk::Reg::default();
    let took = sk::read_text(
        &mut r,
        "skin",
        &d.join("skin/files"),
        br#"{"skin":{"format":1,"target":{"geno":"t9"},"costumes":[{"name":"Gold","file":"x.dat","joint":"JG","csp":"csp.gxtex"}]}}"#,
        &Tg,
    );
    assert_eq!(took, 1, "{:?}", r.log);
    let kind = ck - 1;
    r.finalize(&|fk| if fk == kind { 2 } else { 0 }, &|_| None);
    sk::install_for_test(Some(r));
    assert_eq!((DefineCostumeCount(kind), DefineCostumeCountCK(ck), sk::CostumeCountCK(ck)), (3, 3, 3));
    assert_eq!((name(ck, 2).as_deref(), name(ck, 0)), (Some("Gold"), None));
    let s = |w, c| {
        let p = DefineString(kind, w, c);
        (!p.is_null()).then(|| unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned())
    };
    assert_eq!((s(2, 2).as_deref(), s(3, 2).as_deref(), s(4, 2), s(2, 1).as_deref()), (Some("x.dat"), Some("JG"), None, Some("b.dat")));
    assert_eq!((DefineHasArt(ck, 1, 2), DefineHasArt(ck, 4, 2)), (1, 1), "the skin CSP answers portrait and results");
    let h = DefineArtOpen(ck, 1, 2);
    assert_eq!(GxTex_Format(h), 6, "the skin CSP before the package portrait");
    GxTex_Close(h);
    let h = DefineArtOpen(ck, 4, 2);
    assert_eq!(GxTex_Format(h), 6, "results: the paletted override falls to the skin CSP");
    GxTex_Close(h);
    sk::Suspend(true);
    assert_eq!((DefineCostumeCountCK(ck), name(ck, 2)), (2, None), "suspended: as if no skins");
    sk::Suspend(false);
    sk::install_for_test(None);
    TestRestore();
    set_boot_mods(std::path::Path::new(""), &[]);
    let _ = std::fs::remove_dir_all(&d);
}

struct Tg;
impl sk::Targets for Tg {
    fn define_resolve(&self, key: &str) -> i32 {
        DefineResolve(key)
    }
    fn define_base_kind(&self, kind: i32) -> i32 {
        DefineBaseKind(kind)
    }
    fn define_costume_count_native(&self, kind: i32) -> i32 {
        DefineCostumeCountNative(kind)
    }
    fn kind_for_file(&self, _: &str) -> i32 {
        -1
    }
}

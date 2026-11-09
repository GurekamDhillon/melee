//! Ports of geno_registry.c's tests (test_geno_registry_*): same inputs, same expectations, including the pinned
//! stable id of the canonical form (a change there would change every installed overlay's netplay identity).
use melee_script::geno::consts::*;
use melee_script::geno::registry::{attr_find, attr_is_int, callback_find, target};
use melee_script::geno::{NoEnv, Registry};

const GN_NONE: u32 = 0xFFFF_FFFF;
const GN_KINDS_TEST: i32 = 0x21 + 94;
const TEST_JSON: &str = "{ \"geno\": 1,\n  \"fighters\": [\n    { \"attach\": \"kirby\", \"name\": \"Test Kirby\",\n      \"attributes\": { \"gravity\": 0.08, \"max_jumps\": 9, \"no_such_attr\": 1 },\n      \"jumps\": { \"max\": 9, \"air_vy\": [1.5, 1.25, 1.0] },\n      \"hooks\": { \"on_frame\": [\"geno.count_frames:3\", \"nope\"], \"on_init\": [\"geno.log\"] },\n      \"future_key\": { \"x\": [1, 2] } },\n    { \"define\": { \"name\": \"Meta Knight\" } },\n    { \"attach\": \"PlZz.dat\" }\n  ] }\n";

fn reg(text: &str, m: &str) -> (Registry, i32) {
    let mut r = Registry::default();
    let n = r.parse_text(text.as_bytes(), m, &NoEnv);
    (r, n)
}
fn f(bits: u32) -> f32 {
    f32::from_bits(bits)
}

#[test]
fn parse() {
    let (mut r, n) = reg(TEST_JSON, "test-mod");
    assert_eq!(n, 2, "{:?}", r.log);
    let p = &r.p[0];
    assert_eq!((p.plfile.as_str(), p.name.as_str()), ("PlKb.dat", "Test Kirby"));
    assert_eq!((p.nattr, p.max_jumps, p.njvy), (2, 9, 3));
    assert_eq!(f(p.attr_bits[0]), 0.08f32);
    assert!(attr_is_int(p.attr_index[1]) && p.attr_bits[1] == 9);
    assert_eq!(f(p.jvy[1]), 1.25);
    assert_eq!(p.nhook[GENO_EV_FRAME as usize], 1);
    assert_eq!(p.hook[GENO_EV_FRAME as usize][0], GENO_HOOK_COUNT_FRAMES as i32);
    assert_eq!(p.hook_arg[GENO_EV_FRAME as usize][0], 3);
    assert_eq!(p.hook[GENO_EV_INIT as usize][0], GENO_HOOK_LOG as i32);
    r.build_kinds(&NoEnv);
    assert_eq!(r.kind_profile[4], 0);
    assert_eq!(r.p[1].kind, -1);
}

#[test]
fn stable_ids() {
    let t1 = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":9}}]}";
    let t2 = "{ \"geno\": 1, \"fighters\": [ {\n \"attach\" : \"kirby\" , \"jumps\" : { \"max\" : 9.0 } } ] }";
    let t3 = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":8}}]}";
    let (a, na) = reg(t1, "one");
    let (b, nb) = reg(t2, "two");
    let (c, nc) = reg(t3, "one");
    assert_eq!((na, nb, nc), (1, 1, 1));
    assert_eq!(a.p[0].id, b.p[0].id);
    assert_ne!(a.p[0].id, c.p[0].id);
    // pinned by the C build (gn_test_pinned_hex)
    assert_eq!(a.p[0].hex, "39ebc1bd1de2fc58");
}

#[test]
fn bad_json() {
    let mut r = Registry::default();
    assert_eq!(r.parse_text(b"{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\"", "bad", &NoEnv), -1);
    assert_eq!(r.p.len(), 0);
    assert_eq!(r.parse_text(b"{\"fighters\":[]}", "nover", &NoEnv), -1);
    assert_eq!(r.parse_text(b"{\"geno\":1, // v1\n\"fighters\":[{\"attach\":\"fox\",},],}", "lax", &NoEnv), 1);
}

#[test]
fn empty_is_inert() {
    let mut r = Registry::default();
    for k in 0..GN_KINDS_TEST {
        assert_eq!(r.profile_for_kind(k, &NoEnv), -1);
    }
    assert_eq!(r.salt_for_pl_file("PlKb.dat"), 0);
    assert_eq!(r.salt_for_pl_file("PlSh.dat"), 0);
}

#[test]
fn v1() {
    let t = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\
        \"special_attributes\":[{\"index\":13,\"float\":16.0},{\"offset\":\"0x2C\",\"int\":10},\
        {\"index\":400,\"int\":1},{\"index\":2}],\
        \"on_land\":[{\"from\":\"special:4\",\"to\":\"special:6\"},{\"from\":66,\"to\":43,\"keep_frame\":true},\
        {\"from\":\"nope:1\",\"to\":1}],\
        \"subactions\":[{\"index\":87,\"words\":[\"0xEC220001\",5,4294967295]},{\"index\":2000,\"words\":[1]},\
        {\"index\":88,\"words\":[]}],\
        \"hooks\":{\"on_land\":[\"geno.count_frames:5\"]}}]}";
    let (r, n) = reg(t, "v1");
    assert_eq!(n, 1);
    let p = &r.p[0];
    assert_eq!(p.nspec, 2);
    assert_eq!((p.spec_index[0], f(p.spec_bits[0])), (13, 16.0));
    assert_eq!((p.spec_index[1], p.spec_bits[1]), (11, 10));
    assert_eq!(p.nland, 2);
    assert_eq!(p.land_from[0], target(GENO_TGT_SPECIAL, 4));
    assert_eq!(p.land_to[0], target(GENO_TGT_SPECIAL, 6));
    assert_eq!(p.land_from[1], 66);
    assert_eq!(p.land_to[1], 43 | GENO_TGT_KEEP_FRAME);
    assert_eq!((p.nov, p.ov_anim[0], r.slot_off.len(), r.slot_len[0]), (1, 87, 1, 4));
    let o = r.slot_off[0];
    assert_eq!(&r.pool[o..o + 4], &[0xEC220001, 5, 0xFFFFFFFF, 0]);
    assert_eq!(p.hook[GENO_EV_LAND as usize][0], GENO_HOOK_COUNT_FRAMES as i32);
    assert_eq!(p.hook_arg[GENO_EV_LAND as usize][0], 5);
}

#[test]
fn v2() {
    let t = "{\"geno\":2,\"fighters\":[{\"attach\":\"kirby\",\
        \"states\":[{\"name\":\"GlideStart\",\"behavior\":\"geno.glide.start\",\"subaction\":57},\
        {\"name\":\"Glide\",\"behavior\":\"geno.glide\",\"subaction\":\"motion:65\",\"land\":\"geno:GlideLanding\"},\
        {\"name\":\"GlideLanding\",\"behavior\":\"geno.glide.landing\",\"phys\":\"none\",\"coll\":\"like\",\
        \"anim\":\"nope\",\"like\":\"motion:43\",\"landing_lag\":12,\"move_id\":7,\"flags\":\"0x10\"},\
        {\"name\":\"X\",\"behavior\":\"geno.nope\",\"next\":\"geno:Glide\"}],\
        \"glide\":{\"entry_hold\":20,\"s_start\":2.5,\"bogus\":1},\
        \"tornado\":{\"mash_add\":17},\
        \"specials\":{\"n\":\"geno:Glide\",\"hi\":\"special:3\",\"air_hi\":\"geno:X\",\"lw\":\"geno:Nope\"},\
        \"on_land\":[{\"from\":\"geno:Glide\",\"to\":\"geno:GlideLanding\"}]}]}";
    let (r, n) = reg(t, "v2");
    assert_eq!(n, 1);
    let p = &r.p[0];
    assert_eq!(p.nstate, 4);
    assert_eq!(p.st_name[1], "Glide");
    assert_eq!(p.st_bhv[0], GENO_BHV_GLIDE_START as i32);
    assert_eq!(p.st_bhv[1], GENO_BHV_GLIDE as i32);
    assert_eq!(p.st_bhv[3], GENO_BHV_NONE as i32);
    assert_eq!((p.st_anim[0], p.st_anim[1]), (57, -1));
    assert_eq!(p.st_anim_from[1], target(GENO_TGT_MOTION, 65));
    assert_eq!(p.st_land[1], target(GENO_TGT_GENO, 2));
    assert_eq!(p.st_next[3], target(GENO_TGT_GENO, 1));
    assert_eq!(p.st_like[2], target(GENO_TGT_MOTION, 43));
    assert_eq!(p.st_move_id[2], 7);
    assert!(p.st_flags_set[2] && p.st_flags[2] == 0x10);
    assert_eq!(f(p.st_lag[2]), 12.0);
    // the C test asserts `< 0` here, but GenoGame_CallbackFind(PHYS, "none") is GENO_CBID_NONE (4): the C test is stale
    assert_eq!(p.st_cb[2][GENO_CB_PHYS as usize], 4);
    assert_eq!(p.st_cb[2][GENO_CB_COLL as usize], GENO_CB_LIKE as i32);
    assert_eq!(p.st_cb[2][GENO_CB_ANIM as usize], -1);
    assert_eq!(p.st_cb[2][GENO_CB_IASA as usize], -1);
    assert_eq!(p.nparam, 3);
    assert_eq!(p.param_id[0], GENO_BP_GLIDE_entry_hold as i32);
    assert_eq!(p.param_id[1], GENO_BP_GLIDE_s_start as i32);
    assert_eq!(p.param_id[2], GENO_BP_TORNADO_mash_add as i32);
    assert_eq!(f(p.param_bits[1]), 2.5);
    assert_eq!(p.special[GENO_SP_N as usize], target(GENO_TGT_GENO, 1));
    assert_eq!(p.special[GENO_SP_AIR_N as usize], target(GENO_TGT_GENO, 1));
    assert_eq!(p.special[GENO_SP_HI as usize], target(GENO_TGT_SPECIAL, 3));
    assert_eq!(p.special[GENO_SP_AIR_HI as usize], target(GENO_TGT_GENO, 3));
    assert_eq!(p.special[GENO_SP_LW as usize], GN_NONE);
    assert_eq!(p.special[GENO_SP_S as usize], GN_NONE);
    assert_eq!((p.nland, p.land_from[0], p.land_to[0]), (1, target(GENO_TGT_GENO, 1), target(GENO_TGT_GENO, 2)));
}

#[test]
fn reload_layout() {
    let json = "{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\
        \"articles\":[{\"name\":\"Fire\"},{\"name\":\"Ice\"}],\
        \"subactions\":[{\"index\":44,\"words\":[\"0x04000001\"]},\
        {\"index\":45,\"words\":[\"0x04000002\"]}]}]}";
    // A profile holds its Lua state (define::Def) and is not Clone: each copy is the same registry built again.
    let build = || {
        let (mut a, n) = reg(json, "reload-test");
        assert_eq!(n, 1);
        a.build_kinds(&NoEnv);
        a.p[0].nstate = 1;
        a.p[0].st_bhv[0] = GENO_BHV_GROUND as i32;
        a.p[0].st_cb[0] = [-1; 4];
        a
    };
    let a = build();
    assert!(Registry::reload_layout_changed(&a, &build()).is_none(), "a rebuild has the same layout");
    let clone = |_: &Registry| build();
    for slot in 0..GENO_CB_SLOTS {
        let mut b = clone(&a);
        b.p[0].st_cb[0][slot as usize] = callback_find(slot, if slot == GENO_CB_ANIM { "hold" } else { "none" });
        assert!(Registry::reload_layout_changed(&a, &b).is_some(), "slot {slot}");
    }
    let mut b = clone(&a);
    b.p[0].st_bhv[0] = GENO_BHV_AIR as i32;
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
    let mut b = clone(&a);
    b.p[0].st_anim[0] += 1;
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
    let mut b = clone(&a);
    b.p[0].art_param[0][GENO_AP_LIFETIME as usize] = 20.0f32.to_bits();
    b.pool[b.slot_off[0]] = 0x04000003;
    assert!(Registry::reload_layout_changed(&a, &b).is_none(), "article params and same-layout overlay words reload in place");
    let mut b = clone(&a);
    b.p[0].nart -= 1;
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
    let mut b = clone(&a);
    b.p[0].art_name.swap(0, 1);
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
    let mut b = clone(&a);
    b.slot_off[1] += 1;
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
    let mut b = clone(&a);
    b.slot_len[1] += 1;
    assert!(Registry::reload_layout_changed(&a, &b).is_some());
}

#[test]
fn name_tables() {
    assert!(attr_find("max_jumps") >= 0 && attr_is_int(attr_find("max_jumps")));
    assert_eq!(attr_find("nope"), -1);
    assert_eq!(callback_find(GENO_CB_ANIM, "like"), GENO_CB_LIKE as i32);
}

/// Every shipped geno.json parses (attach entries fully; define entries are counted as skipped until the define
/// parser is ported).
#[test]
fn shipped_mods_parse() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pc/geno/mods");
    let mut seen = 0;
    for ent in std::fs::read_dir(&root).unwrap().flatten() {
        let f = ent.path().join("geno.json");
        let Ok(text) = std::fs::read(&f) else { continue };
        let mut r = Registry::default();
        let id = ent.file_name().to_string_lossy().into_owned();
        let n = r.parse_text(&text, &id, &NoEnv);
        assert!(n >= 0, "{id}: {:?}", r.log);
        assert!(!r.log.iter().any(|l| l.contains("not valid JSON")), "{id}");
        eprintln!("{id}: {n} attach entries, log {} lines", r.log.len());
        seen += 1;
    }
    assert!(seen > 0);
}

//! The identity gate on a whole-match `.gdlp` (gr_check_ident): the C-recorded golden recording
//! plays only where build / disc / mods / Geno data all match, and each mismatch is refused with
//! the C's reason. Uses docs/rust-port/golden/slp/pad_falcon_fox_fd.gdlp (no disc data in it).

use melee_data::gdlp::{self, Identity};
use melee_data::player::{Env, Player};

fn golden() -> Option<Vec<u8>> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/rust-port/golden/slp/pad_falcon_fox_fd.gdlp")).ok()
}

#[test]
fn gate_matches_and_refuses_each_part() {
    let Some(d) = golden() else { return };
    let g = gdlp::read(&d, None).unwrap();
    let me = Identity { id: g.id, geno_version: g.geno_version };
    assert!(gdlp::check_file(&d, &me).is_ok());
    let why = |f: &dyn Fn(&mut Identity)| {
        let mut m = me;
        f(&mut m);
        gdlp::check_file(&d, &m).unwrap_err()
    };
    assert_eq!(why(&|m| m.id[0] ^= 1), "recorded by another build of the game");
    assert_eq!(why(&|m| m.id[1] ^= 1), "recorded with another disc");
    assert!(why(&|m| m.id[2] ^= 1).starts_with("recorded with other mods (it used: "));
    assert_eq!(why(&|m| m.id[3] ^= 1), "recorded with other Geno fighter data (a define changed)");
    assert_eq!(why(&|m| m.geno_version += 1), "recorded with other Geno fighter data (a define changed)");
}

#[test]
fn player_plays_only_on_matching_identity() {
    let Some(d) = golden() else { return };
    let g = gdlp::read(&d, None).unwrap();
    let me = Identity { id: g.id, geno_version: g.geno_version };
    let mut ok = Player::new(Env { resync: true, identity: Some(me), ..Default::default() });
    ok.load_gdlp("g.gdlp", &d, false);
    assert!(ok.active);
    let mut bad = Player::new(Env { resync: true, identity: Some(Identity { id: [1, 2, 3, 4], geno_version: 11 }), ..Default::default() });
    bad.load_gdlp("g.gdlp", &d, false);
    assert!(!bad.active);
    assert!(bad.log.iter().any(|l| l.contains("refused: recorded by another build")));
}

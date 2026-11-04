//! Slippi Ledge Grab Limit (melee_core::codes::LGL) in the time-out ranking `fn_80165AC0`
//! (notes/gameplay-codes.md): off = the game's own ranking; on = a player over the limit loses a tie on time.

use melee_core::codes;
use melee_modes::tu::gm_1601::fn_80165AC0;

fn tie(outcome: u8) -> Box<melee_types::MatchEnd> {
    let mut m: Box<melee_types::MatchEnd> = Box::new(unsafe { core::mem::zeroed() });
    m.outcome = outcome;
    for i in 0..6 {
        m.player_standings[i].pkind = if i < 2 { 0 } else { 3 };
        m.player_standings[i].score = 0;
    }
    m
}

fn winners(m: &melee_types::MatchEnd) -> Vec<u8> {
    m.winners[..m.n_winners as usize].to_vec()
}

#[test]
fn lgl_off_on() {
    // vanilla: a tie on time has two winners (sudden death)
    codes::reset();
    unsafe { codes::LEDGE_GRABS = [99, 0, 0, 0, 0, 0] };
    let mut m = tie(1);
    unsafe { fn_80165AC0(&mut *m) };
    assert_eq!(winners(&m), vec![0, 1], "off: unchanged");

    // on, P1 over the limit on a time-out: P2 wins alone
    codes::set(codes::LGL, 45);
    unsafe { codes::LEDGE_GRABS = [46, 3, 0, 0, 0, 0] };
    let mut m = tie(1);
    unsafe { fn_80165AC0(&mut *m) };
    assert_eq!(winners(&m), vec![1], "on: the player over the limit loses");

    // on, at the limit (not over): unchanged
    unsafe { codes::LEDGE_GRABS = [45, 3, 0, 0, 0, 0] };
    let mut m = tie(1);
    unsafe { fn_80165AC0(&mut *m) };
    assert_eq!(winners(&m), vec![0, 1]);

    // on, but the match did not end on time (elimination): unchanged
    unsafe { codes::LEDGE_GRABS = [99, 0, 0, 0, 0, 0] };
    let mut m = tie(2);
    unsafe { fn_80165AC0(&mut *m) };
    assert_eq!(winners(&m), vec![0, 1]);

    // the counter only runs with the code on
    codes::reset();
    unsafe { codes::LEDGE_GRABS = [0; 6] };
    codes::note_ledge_grab(0);
    assert_eq!(unsafe { codes::LEDGE_GRABS[0] }, 0);
    codes::set(codes::LGL, 45);
    codes::note_ledge_grab(0);
    assert_eq!(unsafe { codes::LEDGE_GRABS[0] }, 1);
    codes::reset();
}

use super::*;

#[test]
fn cmd_round_trip() {
    let c = LabCmd::new(OP_PLAY_SET, 1, 0x0102, 0xDEAD_BEEF, 7, 0x9876_5432);
    let b = c.to_bytes();
    assert_eq!(b.len(), LAB_CMD_BYTES);
    assert_eq!(LabCmd::from_bytes(&b), Some(c));
    assert_eq!(LabCmd::from_bytes(&[0u8; 16]), None, "op 0 is not a command");
    assert_eq!(LabCmd::from_bytes(&b[..15]), None);
}

#[test]
fn push_is_session_and_state_gated() {
    let mut s = LabShared::default();
    let c = LabCmd::new(OP_SAVE, 0, 3, 0, 0, 0);
    assert_eq!(s.push(c), GD_ESTATE, "no lab match running");
    s.active = true;
    s.apply_frame = 40;
    assert_eq!(s.push(c), 40);
    for _ in 0..LAB_CMDS_PER_FRAME - 1 {
        assert_eq!(s.push(c), 40);
    }
    assert_eq!(s.push(c), 41, "a ninth command waits for the next frame");
    s.set_session(Session::Netplay);
    assert!(s.outbox.is_empty(), "a session change drops queued commands");
    assert_eq!(s.push(c), GD_EOFFLINE);
    s.set_session(Session::LabNetplay);
    assert_eq!(s.push(c), 40);
    s.set_session(Session::Synctest);
    assert_eq!(s.push(c), GD_EOFFLINE);
}

#[test]
fn rec_handles_carry_the_issuer() {
    let mut s = LabShared::default();
    let a = s.new_rec();
    s.issuer = 1;
    let b = s.new_rec();
    assert_ne!(a, b);
    assert_eq!(a >> 6, 0);
    assert_eq!(b >> 6, 1);
    assert_ne!(a & 63, 0);
}

#[test]
fn presentation_rng_is_a_function_of_seed_and_frame() {
    let mut a = LabShared::default();
    let mut b = LabShared::default();
    a.match_frame = 100;
    b.match_frame = 100;
    let va: Vec<u32> = (0..4).map(|_| a.rng()).collect();
    let vb: Vec<u32> = (0..4).map(|_| b.rng()).collect();
    assert_eq!(va, vb);
    assert_ne!(va[0], va[1]);
    a.match_frame = 101;
    assert_ne!(a.rng(), va[0]);
}

#[test]
fn react_param_packs() {
    let p = ReactParam { delay: 3, strength: 80, chance: 25, rec: 0x41 };
    assert_eq!(ReactParam::from_u32(p.to_u32()), p);
}

#[test]
fn event_history_is_bounded() {
    let mut s = LabShared::default();
    for f in 0..2000 {
        s.match_frame = f;
        s.publish_events(&[LabEvent { kind: EV_HIT, port: 0, frame: f, a: 1, b: 2 }]);
    }
    assert_eq!(s.events.len(), 1);
    assert!(s.history.len() as i32 <= EVENT_HISTORY + 1);
    assert_eq!(s.history.front().unwrap().frame, 1999 - EVENT_HISTORY + 0);
}

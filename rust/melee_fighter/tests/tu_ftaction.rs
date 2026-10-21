//! Smoke tests for tu/ftaction.rs: handlers that only decode a command word into the fighter,
//! the TARGET_PC hitbox-slot remap, and the word-count table.
use melee_fighter::tu::ftaction as m;
use melee_fighter::types::*;

fn setup() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut fi: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *fi as *mut Fighter as *mut _;
    (fi, g)
}

#[test]
fn cmd_var_and_flags() {
    let (fi, mut g) = setup();
    let mut words: [CmdUnion; 4] = unsafe { core::mem::zeroed() };
    unsafe {
        words[0].set_cmd_var.set_idx(2);
        words[0].set_cmd_var.set_value(0x1234);
        words[1].set_jab_rapid.set_state(1);
        words[2].set_fighter_vis.set_value(1);
    }
    let mut cmd: CommandInfo = unsafe { core::mem::zeroed() };
    cmd.anon_8.u = words.as_mut_ptr();
    m::ftAction_80071820(&mut *g, &mut cmd);
    m::ftAction_80071B28(&mut *g, &mut cmd);
    m::ftAction_80071FA0(&mut *g, &mut cmd);
    assert_eq!(fi.cmd_vars[2], 0x1234);
    assert_eq!(fi.x2218_b2(), 1);
    assert_eq!(fi.x221E_b5(), 1);
    assert_eq!(unsafe { cmd.anon_8.u.offset_from(words.as_ptr()) }, 3);
}

#[test]
fn hitbox_slot_remap() {
    let (mut fi, _g) = setup();
    unsafe {
        let p0: *mut HitCapsule = &mut fi.x914[1];
        assert_eq!(m::ftAction_HitboxSlot(&mut fi, 1, true), p0);
        fi.x914[0].state = HitCapsule_Enabled;
        // id 4: lowest disabled slot (1) gets the mark
        let p = m::ftAction_HitboxSlot(&mut fi, 4, true);
        assert_eq!(p, &mut fi.x914[1] as *mut _);
        assert_eq!(fi.x914[1].x43_b7(), 1);
        fi.x914[1].state = HitCapsule_Enabled;
        assert_eq!(m::ftAction_HitboxSlot(&mut fi, 5, false), &mut fi.x914[1] as *mut _);
    }
    assert_eq!(m::ftAction_803C0870.iter().map(|&n| n as u32).sum::<u32>(), 78);
}

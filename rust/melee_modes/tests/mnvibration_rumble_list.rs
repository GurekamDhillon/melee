//! Regression: Options > Rumble switching rumble on queued `mnVibration_804D4FF0`, stored host-endian, so the
//! interpreter read op 0 (loop to head) forever and the game hung (tick 1173 of the Envoy camp_pad scenario).
use melee_modes::tu::mnvibration::mnVibration_804D4FF0;
use melee_runtime::tu::rumble::HSD_PadRumbleInterpret1;
use melee_types::*;

#[test]
fn vibration_menu_rumble_list_is_one_frame_then_end() {
    unsafe {
        let head = (&raw mut mnVibration_804D4FF0).cast::<HSD_Rumble>();
        let cmd0 = (*head).command;
        let cmd1 = (*head.add(1)).command;
        assert_eq!((cmd0.op(), cmd0.frame()), (1, 1), "first command: rumble on for 1 frame");
        assert_eq!(cmd1.op(), 0, "second command: end of list");

        // Interpret it the way HSD_PadRumbleAdd(i, 0, 14, 0, list) queues it: it must return, not spin.
        let mut node = HSD_PadRumbleListData::zeroed();
        node.frame = 14;
        node.headp = head;
        node.listp = head;
        let mut status = 0u8;
        assert_eq!(HSD_PadRumbleInterpret1(&mut node, &mut status), 0);
        assert_eq!(status, 2, "op 1 sets status 3 - 1");
        // The next call reaches op 0 with frame != -2 and loops to the head: still terminates (wait reloads).
        assert_eq!(HSD_PadRumbleInterpret1(&mut node, &mut status), 0);
    }
}

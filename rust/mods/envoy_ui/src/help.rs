//! The camp's "How to play" card (envoy-readiness.md, first-run gaps): one screen of plain lines, the goal, the pair
//! and the tag button, Heat, drives, Volley, coins, the pause. Opened from the camp's row; A, B or Start closes it.
//! Presentation only: it reads the tag button the camp shows and draws; nothing here reaches the sim.

use crate::camp::Pad;
use crate::kit::*;
use gd_ui_enc::*;

static mut OPEN: bool = false;
static mut AGE: i32 = 0;

pub unsafe fn is_open() -> bool {
    OPEN
}

pub unsafe fn open() {
    OPEN = true;
    AGE = 0;
}

/// The lines: a short heading and one sentence each. `tag`: the camp's tag button (0 Y, 1 X, 2 the classic D-pad).
fn lines(tag: i32) -> [(&'static str, &'static str); 7] {
    [
        ("Goal", "Clear three rooms and the boss. Then the next loop begins, harder. The run ends when your stocks run out."),
        (
            "Your pair",
            match tag {
                0 => "Two fighters, one stock pool. Hold Y to swap them. Tap Y: your partner strikes (30 Heat).",
                1 => "Two fighters, one stock pool. Hold X to swap them. Tap X: your partner strikes (30 Heat).",
                _ => "Two fighters, one stock pool. D-pad down swaps them.",
            },
        ),
        ("Heat", "Damage you deal builds Heat, damage you take cools it. Hotter: every drive hits harder. A KO cashes Heat out as coins."),
        ("Drives", "Drives fire on their own while you fight. Pick new ones after rooms. The same drive again levels it up."),
        ("Volley", "Cash-outs fill three pips. You start full. With all three, D-pad right fires your whole row at once."),
        ("Coins", "The shop opens at each stop (after room 2 and after the boss): drives, level-ups, a stock back."),
        ("Pause", "Start pauses. L + R + A + Start leaves the fight: the run is kept, the fight starts over."),
    ]
}

/// One frame of the card over the camp's backdrop. Returns false once it has closed.
pub unsafe fn frame(e: &mut E, pad: &Pad, tag: i32) -> bool {
    AGE += 1;
    if AGE > 6 && (pad.a || pad.b || pad.start) {
        OPEN = false;
        return false;
    }
    e.layer(LAYER_MENU, 0);
    spr(e, S_BG_CAMP, -107.0, 0.0, 854.0, 480.0, WHITE);
    e.rect(-107.0, 0.0, 854.0, 480.0, 0x1A10_4A90);
    let (x, y, w) = (40.0, 26.0, 560.0);
    sticker(e, S_FACE_WHITE, x, y, w, 404.0, -0.3, WHITE, |e| {
        e.rect(x + 1.5, y + 1.5, w - 3.0, 44.0, SUNNY);
        e.text_ex(x + 18.0, y + 9.0, d(24), INK, ALIGN_LEFT, 0.0, "How to play");
        let mut ly = y + 58.0;
        for (head, body) in lines(tag) {
            e.text_ex(x + 18.0, ly, d(15), INK, ALIGN_LEFT, 0.0, head);
            crate::pick::wrap(e, x + 112.0, ly + 1.0, w - 132.0, bs(13), 17.0, LILAC, ALIGN_LEFT, body, 2);
            ly += 48.0;
        }
    });
    hints(e, &[("B", "Back to the camp")]);
    true
}

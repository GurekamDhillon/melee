//! pc/platform/gw_slippi_pad.c: the eight-byte Slippi PAD packet and the online frame numbering.

/// Eight bytes sent in a Slippi PAD packet: buttons (BE16), main X/Y, C X/Y (signed), L/R bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlippiPad {
    pub bytes: [u8; 8],
}

/// Melee clamps raw trigger bytes to 140 and divides by 140; this is the inverse (nearest byte).
pub fn trigger_byte(value: f32) -> u8 {
    if !(value >= 0.0) {
        return 0;
    }
    if value >= 1.0 {
        return 140;
    }
    (value * 140.0 + 0.5).floor() as u8
}

/// `gw_SlippiPad_FromPhysical`: None when a trigger is not a finite value in 0..=1.
pub fn from_physical(buttons: u16, sx: i8, sy: i8, cx: i8, cy: i8, l: f32, r: f32) -> Option<SlippiPad> {
    if !l.is_finite() || !r.is_finite() || l < 0.0 || l > 1.0 || r < 0.0 || r > 1.0 {
        return None;
    }
    Some(SlippiPad {
        bytes: [(buttons >> 8) as u8, buttons as u8, sx as u8, sy as u8, cx as u8, cy as u8, trigger_byte(l), trigger_byte(r)],
    })
}

/// The replay frame is an APPLIED input; online PAD frame 1 is replay frame -123.
pub fn online_frame(slp_frame: i32) -> i32 {
    if slp_frame >= -123 {
        slp_frame + 124
    } else {
        -1
    }
}

pub fn slp_frame(online_frame: i32) -> i32 {
    if online_frame >= 1 {
        online_frame - 124
    } else {
        -1
    }
}

/// A sample at online frame F is transmitted for applied PAD frame F + delay.
pub fn sample_frame(slp_frame: i32, delay: i32) -> i32 {
    let applied = online_frame(slp_frame);
    if delay >= 0 && applied > delay {
        applied - delay
    } else {
        -1
    }
}

//! Port of `pc/platform/gc_adapter_decode.h`: the WUP-028 report -> `PADStatus`, with the
//! per-port calibration (stick origin, trigger rest, stuck-button mask) captured on the first
//! report of a port or on a recalibrate request.
//!
//! Report: 37 bytes, tag 0x21, then 4 ports x 9 bytes:
//! `[0] status (high nibble 1 wired / 2 wireless / 0 empty), [1] buttons lo, [2] buttons hi,
//!  [3] stickX, [4] stickY, [5] substickX, [6] substickY, [7] trigL, [8] trigR`.
use crate::pad::*;
use crate::plog;

pub const GC_PAYLOAD_SIZE: usize = 37;
pub const GC_PORTS: usize = 4;

pub const GC_BTN_A: u8 = 0x01;
pub const GC_BTN_B: u8 = 0x02;
pub const GC_BTN_X: u8 = 0x04;
pub const GC_BTN_Y: u8 = 0x08;
pub const GC_BTN_DLEFT: u8 = 0x10;
pub const GC_BTN_DRIGHT: u8 = 0x20;
pub const GC_BTN_DDOWN: u8 = 0x40;
pub const GC_BTN_DUP: u8 = 0x80;
pub const GC_BTN2_START: u8 = 0x01;
pub const GC_BTN2_Z: u8 = 0x02;
pub const GC_BTN2_R: u8 = 0x04;
pub const GC_BTN2_L: u8 = 0x08;

/// `GW_GC_TRIG_DEAD`: resting value above which a trigger has no usable travel left.
pub const GW_GC_TRIG_DEAD: u8 = 200;

/// `gw_gc_origin` / `gw_gc_have_origin` / `gw_gc_trig_rest` / `gw_gc_btn_stuck`.
#[derive(Clone, Debug, Default)]
pub struct Calibration {
    pub origin: [[u8; 4]; GC_PORTS],
    pub have_origin: [bool; GC_PORTS],
    pub trig_rest: [[u8; 2]; GC_PORTS],
    pub btn_stuck: [[u8; 2]; GC_PORTS],
}

/// `gw_gc_trigger`.
pub fn gc_trigger(raw: u8, rest: u8) -> u8 {
    if rest >= GW_GC_TRIG_DEAD {
        return 0;
    }
    let span = 255 - rest as i32;
    let v = (raw as i32 - rest as i32) * 255 / span;
    v.clamp(0, 255) as u8
}

/// `gw_gc_axis`.
pub fn gc_axis(raw: u8, origin: u8) -> i8 {
    (raw as i32 - origin as i32).clamp(-128, 127) as i8
}

/// `gw_gc_decode`. Ports with nothing plugged in are left untouched (and lose their origin).
/// Returns whether any port was written.
pub fn gc_decode(
    cal: &mut Calibration,
    snap: &[u8; GC_PAYLOAD_SIZE],
    st: &mut [PADStatus; PAD_CHANMAX],
    recal: bool,
) -> bool {
    let mut any = false;
    for chan in 0..GC_PORTS {
        let p = &snap[1 + chan * 9..1 + chan * 9 + 9];
        let ty = p[0] >> 4;
        if ty != 1 && ty != 2 {
            cal.have_origin[chan] = false;
            continue;
        }
        if !cal.have_origin[chan] || recal {
            cal.origin[chan] = [p[3], p[4], p[5], p[6]];
            cal.trig_rest[chan] = [p[7], p[8]];
            cal.btn_stuck[chan] = [p[1], p[2]];
            cal.have_origin[chan] = true;
            plog!(
                "gw: gc adapter: ch{} calibrated -- stick ({},{}) c ({},{}) triggers ({},{}){}{}",
                chan, p[3], p[4], p[5], p[6], p[7], p[8],
                if p[7] >= GW_GC_TRIG_DEAD { " [L has no travel, disabled]" } else { "" },
                if p[8] >= GW_GC_TRIG_DEAD { " [R has no travel, disabled]" } else { "" }
            );
            if p[1] != 0 || p[2] != 0 {
                plog!(
                    "gw: gc adapter: ch{} has buttons held at rest (b1={:02X} b2={:02X}); masking them",
                    chan, p[1], p[2]
                );
            }
        }
        let b1 = p[1] & !cal.btn_stuck[chan][0];
        let b2 = p[2] & !cal.btn_stuck[chan][1];
        let mut btn: u16 = 0;
        const MAP1: [(u8, u16); 8] = [
            (GC_BTN_A, PAD_BUTTON_A),
            (GC_BTN_B, PAD_BUTTON_B),
            (GC_BTN_X, PAD_BUTTON_X),
            (GC_BTN_Y, PAD_BUTTON_Y),
            (GC_BTN_DLEFT, PAD_BUTTON_LEFT),
            (GC_BTN_DRIGHT, PAD_BUTTON_RIGHT),
            (GC_BTN_DDOWN, PAD_BUTTON_DOWN),
            (GC_BTN_DUP, PAD_BUTTON_UP),
        ];
        const MAP2: [(u8, u16); 4] = [
            (GC_BTN2_START, PAD_BUTTON_START),
            (GC_BTN2_Z, PAD_TRIGGER_Z),
            (GC_BTN2_R, PAD_TRIGGER_R),
            (GC_BTN2_L, PAD_TRIGGER_L),
        ];
        for (m, b) in MAP1 {
            if b1 & m != 0 {
                btn |= b;
            }
        }
        for (m, b) in MAP2 {
            if b2 & m != 0 {
                btn |= b;
            }
        }
        let o = cal.origin[chan];
        let s = &mut st[chan];
        s.button = btn;
        s.stickX = gc_axis(p[3], o[0]);
        s.stickY = gc_axis(p[4], o[1]);
        s.substickX = gc_axis(p[5], o[2]);
        s.substickY = gc_axis(p[6], o[3]);
        s.triggerLeft = gc_trigger(p[7], cal.trig_rest[chan][0]);
        s.triggerRight = gc_trigger(p[8], cal.trig_rest[chan][1]);
        s.analogA = 0;
        s.analogB = 0;
        s.err = 0;
        // extButton is not written by the C decode either (it stays whatever the caller had).
        any = true;
    }
    any
}

/// `gw_gc_accept_report`: find the 0x21 tag at offset 0 or later (Windows HID prefixes a
/// report-ID byte) and return the 37-byte payload.
pub fn accept_report(buf: &[u8]) -> Option<[u8; GC_PAYLOAD_SIZE]> {
    let mut off = 0;
    while off + GC_PAYLOAD_SIZE <= buf.len() {
        if buf[off] == 0x21 {
            let mut out = [0u8; GC_PAYLOAD_SIZE];
            out.copy_from_slice(&buf[off..off + GC_PAYLOAD_SIZE]);
            return Some(out);
        }
        off += 1;
    }
    None
}

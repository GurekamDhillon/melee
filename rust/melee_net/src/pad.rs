//! One slot's input as the session holds it (`GwRbInput`, gw_rollback.h) and its 11-byte wire
//! form (`np_encode` / `np_decode`, gw_netplay.c: `NP_PAYLOAD`, the payload size the game uses).
//!
//! GwRbInput is a native-only struct (it lives in the exe's memory, never in GameCube MEM1), so it
//! has no GameCube offset table; the wire form is the compatibility contract.

/// `NP_PAYLOAD`: buttons u16 (big-endian on the wire), stick x/y, c-stick x/y, L, R, analog A,
/// analog B, err.
pub const NP_PAYLOAD: usize = 11;

/// `GwRbInput`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RbInput {
    pub lx: f32,
    pub ly: f32,
    pub cx: f32,
    pub cy: f32,
    pub trigger: f32,
    pub buttons: u32,
    pub seed: u32,
    pub raw: [i8; 4],
    pub present: u8,
    pub confirmed: u8,
    pub is_raw: u8,
    pub pad_l: u8,
    pub pad_r: u8,
    pub pad_a: u8,
    pub pad_b: u8,
    pub pad_err: i8,
    pub physical_buttons: u16,
    pub physical_l: f32,
    pub physical_r: f32,
    pub physical_complete: u8,
}

impl RbInput {
    /// The raw-controller fields compared for a misprediction (what the wire carries).
    pub fn same_pad(&self, o: &RbInput) -> bool {
        let mut a = [0u8; NP_PAYLOAD];
        let mut b = [0u8; NP_PAYLOAD];
        np_encode(self, &mut a);
        np_encode(o, &mut b);
        a == b && self.present == o.present
    }
}

/// `np_encode`.
pub fn np_encode(i: &RbInput, o: &mut [u8]) {
    o[0] = (i.buttons >> 8) as u8;
    o[1] = i.buttons as u8;
    o[2] = i.raw[0] as u8;
    o[3] = i.raw[1] as u8;
    o[4] = i.raw[2] as u8;
    o[5] = i.raw[3] as u8;
    o[6] = i.pad_l;
    o[7] = i.pad_r;
    o[8] = i.pad_a;
    o[9] = i.pad_b;
    o[10] = i.pad_err as u8;
}

/// `np_decode`: a present, confirmed, raw-controller entry.
pub fn np_decode(b: &[u8]) -> RbInput {
    RbInput {
        present: 1,
        is_raw: 1,
        confirmed: 1,
        buttons: ((b[0] as u32) << 8) | b[1] as u32,
        raw: [b[2] as i8, b[3] as i8, b[4] as i8, b[5] as i8],
        pad_l: b[6],
        pad_r: b[7],
        pad_a: b[8],
        pad_b: b[9],
        pad_err: b[10] as i8,
        ..RbInput::default()
    }
}

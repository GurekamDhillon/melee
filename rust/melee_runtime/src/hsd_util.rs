//! `src/sysdolphin/baselib/util.c` (3/3) and `crypt.c` (5/5: memory-card save checksum and
//! byte scrambler). No float code, no fused sites.

/// `GXColor` (GameCube record: 4 x u8, size 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GxColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
melee_core::gc_layout!(GxColor, "GXColor", size 0x4, align 1, [
    r @ 0x0: 1 "u8",
    g @ 0x1: 1 "u8",
    b @ 0x2: 1 "u8",
    a @ 0x3: 1 "u8",
]);

/// `HSD_MulColor`.
pub fn mul_color(a: &GxColor, b: &GxColor) -> GxColor {
    let m = |x: u8, y: u8| ((x as u32 * y as u32) / 255) as u8;
    GxColor { r: m(a.r, b.r), g: m(a.g, b.g), b: m(a.b, b.b), a: m(a.a, b.a) }
}

/// `HSD_GetNbBits`.
pub fn get_nb_bits(c: u32) -> u32 {
    c.count_ones()
}

/// `HSD_Index2PosNrmMtx` (C asserts on > 9 and then returns 0).
pub fn index_to_pos_nrm_mtx(i: u32) -> i32 {
    if i <= 9 { (i * 3) as i32 } else { 0 }
}

/// `HSD_identityMtx`.
pub const HSD_IDENTITY_MTX: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];

const KEYS: [u32; 13] = [0x26, 0xFF, 0xE8, 0xEF, 0x42, 0xD6, 0x01, 0x54, 0x14, 0xA3, 0x80, 0xFD, 0x6E];

/// `HSD_Checksum`: 16 bytes into `dest`.
pub fn checksum(src: &[u8]) -> [u8; 16] {
    let mut m: [u8; 16] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10,
    ];
    for (i, &b) in src.iter().enumerate() {
        m[i % 16] = m[i % 16].wrapping_add(b);
    }
    for i in 1..16 {
        if m[i - 1] == m[i] {
            m[i] ^= 0xFF;
        }
    }
    m
}

/// `encryptByte` (static).
fn encrypt_byte(prev: u8, cur: u8) -> u8 {
    let key = KEYS[(prev % 13) as usize];
    let v = (prev as u32 ^ cur as u32) ^ key;
    (match prev % 7 {
        0 => (v & 1) | ((v << 3) & 0x10) | ((v >> 1) & 2) | ((v << 2) & 0x20) | ((v >> 2) & 4)
            | ((v << 1) & 0x40) | ((v >> 3) & 8) | (v & 0x80),
        1 => ((v << 3) & 8) | ((v >> 1) & 1) | (v & 4) | ((v << 3) & 0x40) | ((v << 1) & 0x20)
            | ((v >> 1) & 0x10) | ((v << 1) & 0x80) | ((v >> 6) & 2),
        2 => ((v << 6) & 0x40) | ((v << 4) & 0x20) | ((v >> 2) & 1) | ((v >> 2) & 2) | ((v >> 1) & 8)
            | ((v << 2) & 0x80) | ((v >> 4) & 4) | ((v >> 3) & 0x10),
        3 => ((v << 1) & 2) | ((v << 2) & 8) | ((v << 5) & 0x80) | ((v << 1) & 0x10) | ((v >> 4) & 1)
            | ((v >> 3) & 4) | ((v >> 1) & 0x20) | ((v >> 1) & 0x40),
        4 => ((v << 7) & 0x80) | ((v << 1) & 4) | ((v << 3) & 0x20) | ((v >> 3) & 1) | ((v << 2) & 0x40)
            | ((v >> 4) & 2) | ((v >> 2) & 0x10) | ((v >> 4) & 8),
        5 => ((v & 1) << 5) | ((v << 5) & 0x40) | ((v << 2) & 0x10) | (v & 8) | ((v << 3) & 0x80)
            | ((v >> 5) & 1) | ((v >> 5) & 2) | ((v >> 5) & 4),
        _ => ((v << 2) & 4) | (v & 2) | ((v & 4) << 4) | ((v << 4) & 0x80) | (v & 0x10) | ((v >> 2) & 8)
            | ((v >> 6) & 1) | ((v >> 2) & 0x20),
    }) as u8
}

/// `decryptByte` (static).
fn decrypt_byte(prev: u8, c: u32) -> u8 {
    let cur: u32 = match prev % 7 {
        0 => (c & 1) | ((c << 1) & 4) | ((c << 2) & 0x10) | ((c << 3) & 0x40) | ((c >> 3) & 2)
            | ((c >> 2) & 8) | ((c >> 1) & 0x20) | (c & 0x80),
        1 => ((c << 1) & 2) | ((c << 6) & 0x80) | (c & 4) | ((c >> 3) & 1) | ((c << 1) & 0x20)
            | ((c >> 1) & 0x10) | ((c >> 3) & 8) | ((c >> 1) & 0x40),
        2 => ((c & 1) << 2) | ((c << 2) & 8) | ((c << 4) & 0x40) | ((c << 1) & 0x10) | ((c << 3) & 0x80)
            | ((c >> 4) & 2) | ((c >> 6) & 1) | ((c >> 2) & 0x20),
        3 => ((c << 4) & 0x10) | ((c >> 1) & 1) | ((c << 3) & 0x20) | ((c >> 2) & 2) | ((c >> 1) & 8)
            | ((c << 1) & 0x40) | ((c << 1) & 0x80) | ((c >> 5) & 4),
        4 => ((c << 3) & 8) | ((c << 4) & 0x20) | ((c >> 1) & 2) | ((c << 4) & 0x80) | ((c << 2) & 0x40)
            | ((c >> 3) & 4) | ((c >> 2) & 0x10) | ((c >> 7) & 1),
        5 => ((c & 1) << 5) | ((c << 5) & 0x40) | ((c & 4) << 5) | (c & 8) | ((c >> 2) & 4) | ((c >> 5) & 1)
            | ((c >> 5) & 2) | ((c >> 3) & 0x10),
        _ => ((c << 6) & 0x40) | (c & 2) | ((c >> 2) & 1) | ((c << 2) & 0x20) | (c & 0x10) | ((c << 2) & 0x80)
            | ((c >> 4) & 4) | ((c >> 4) & 8),
    } & 0xFF;
    ((cur ^ KEYS[(prev % 13) as usize]) ^ prev as u32) as u8
}

/// `HSD_Encrypt`: checksum of `data[16..]` into `data[0..16]`, then scramble `data[16..]`.
/// `len` < 16 behaves like the C only for the checksum of an empty range (C would read before
/// the buffer for negative lengths; returns -1 here).
pub fn encrypt(data: &mut [u8]) -> i32 {
    if data.len() < 16 {
        return -1;
    }
    let sum = checksum(&data[16..]);
    data[..16].copy_from_slice(&sum);
    for i in 16..data.len() {
        data[i] = encrypt_byte(data[i - 1], data[i]);
    }
    0
}

/// `HSD_Decrypt`: 0 if the checksum matches, else -1 (the data is decrypted either way).
pub fn decrypt(data: &mut [u8]) -> i32 {
    if data.len() < 16 {
        return -1;
    }
    let mut prev = data[15];
    for i in 16..data.len() {
        let cur = data[i];
        data[i] = decrypt_byte(prev, cur as u32);
        prev = cur;
    }
    let check = checksum(&data[16..]);
    if check[..] == data[..16] { 0 } else { -1 }
}

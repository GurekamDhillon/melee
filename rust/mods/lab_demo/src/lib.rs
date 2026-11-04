//! lab_demo: a minimal gd-lab-v1 mod (docs/rust-port/notes/wasm-ui-lab-api.md part B). It exercises savestates, the
//! rewind ring, record and playback and the frame meter feed, and draws a small HUD with gd-ui-v1 (theme A):
//! a status chip, the two-row frame meter from `gd_lab_meter` with the advantage from `gd_lab_frame_adv`, toasts for
//! the lab events and for each action, and the hit/hurt/ECB/ledge overlays (B4).
//!
//! Controls (port 1, shared capture: the game still sees the pad): D-down save slot 0, D-up load it, D-left rewind 60
//! frames, D-right go live; Z + D-left starts / stops recording P1, Z + D-right plays the last recording on P2
//! (mirrored, looped) or stops it.
//!
//! Autopilot: with no D-pad press it runs one scripted pass (the headless checks use it) at fixed lab frames: save,
//! record P1, rewind, go live, load, play the recording on P2. Online (LabNetplay) the host's copy drives it and the
//! guest's copy saves and loads slot 1, so both peers' commands cross the lockstep stream.
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_lab")]
extern "C" {
    fn gd_lab_save(slot: u32) -> i32;
    fn gd_lab_load(slot: u32) -> i32;
    fn gd_lab_ring(frames: u32) -> i32;
    fn gd_lab_rewind(n: u32) -> i32;
    fn gd_lab_go_live() -> i32;
    fn gd_lab_rec_start(mask: u32) -> u32;
    fn gd_lab_rec_stop(rec: u32) -> i32;
    fn gd_lab_play(rec: u32, mask: u32, flags: u32) -> i32;
    fn gd_lab_play_stop() -> i32;
    fn gd_lab_overlay(mask: u32, bits: u32) -> i32;
    fn gd_lab_meter(port: u32, out: *mut u8, n: u32) -> i32;
    fn gd_lab_frame_adv(a: u32, b: u32) -> i32;
    fn gd_lab_events(out: *mut u8, max: u32) -> i32;
    fn gd_lab_status(out: *mut i32) -> i32;
}

const FEATURE_LAB: u32 = 1 << 10;
const FEATURE_LAB_STATE: u32 = 1 << 11;
const FEATURE_LAB_INPUT: u32 = 1 << 12;
const FEATURE_LAB_OVERLAY: u32 = 1 << 14;
const PLAY_LOOP: u32 = 1;
const PLAY_MIRROR: u32 = 2;
const OV: u32 = 1 | 2 | 4 | 8 | (1 << 7); // hitbox, hurtbox, ECB, ledges, labels

static mut ENC: Enc<4096> = Enc::new();
static mut TOK: [u32; 26] = [0; 26];
static mut CAP: u32 = 0;
static mut SETUP: bool = false;
static mut REC: u32 = 0;
static mut RECORDING: bool = false;
static mut PLAYING: bool = false;
static mut MANUAL: bool = false;
static mut STEP: usize = 0;
static mut TOASTS: [([u8; 32], usize, u32); 3] = [([0; 32], 0, 0); 3];
static mut STATUS: [i32; 10] = [0; 10];
static mut METER: [[u8; 50]; 2] = [[0; 50]; 2];
static mut EVENTS: [u8; 16 * 16] = [0; 256];

const T_PANEL_HUD: usize = 1;
const T_BORDER: usize = 2;
const T_TEXT: usize = 3;
const T_MUTED: usize = 4;
const T_ACCENT: usize = 5;
const T_ACCENT_INK: usize = 6;
const T_P1: usize = 10;
const T_P2: usize = 11;
const T_CELL: usize = 12;

fn toast(msg: &str) {
    unsafe {
        TOASTS.rotate_right(1);
        let n = msg.len().min(32);
        TOASTS[0].0[..n].copy_from_slice(&msg.as_bytes()[..n]);
        TOASTS[0].1 = n;
        TOASTS[0].2 = 120;
    }
}

/// The autopilot: (lab frame, action) for the host / offline copy, and for an online guest.
const HOST_PLAN: [(i32, u8); 8] = [(60, b's'), (90, b'r'), (210, b'R'), (300, b'w'), (330, b'g'), (420, b'l'), (450, b'p'), (900, b'P')];
const GUEST_PLAN: [(i32, u8); 2] = [(500, b'S'), (560, b'L')];

fn act(a: u8) {
    unsafe {
        match a {
            b's' => {
                gd_lab_save(0);
                toast("Saved slot 1");
            }
            b'l' => {
                gd_lab_load(0);
                toast("Loaded slot 1");
            }
            b'S' => {
                gd_lab_save(1);
                toast("P2 saved slot 2");
            }
            b'L' => {
                gd_lab_load(1);
                toast("P2 loaded slot 2");
            }
            b'w' => {
                gd_lab_rewind(60);
                toast("Rewind 60f");
            }
            b'g' => {
                gd_lab_go_live();
                toast("Live");
            }
            b'r' => {
                REC = gd_lab_rec_start(0b01);
                RECORDING = REC != 0;
                toast("REC P1");
            }
            b'R' => {
                gd_lab_rec_stop(REC);
                RECORDING = false;
                toast("REC stopped");
            }
            b'p' => {
                if REC != 0 {
                    gd_lab_play(REC, 0b10, PLAY_LOOP | PLAY_MIRROR);
                    PLAYING = true;
                    toast("Play on P2 (mirror)");
                }
            }
            b'P' => {
                gd_lab_play_stop();
                PLAYING = false;
                toast("Playback stopped");
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe {
        gd_ui_theme(0);
        for i in 0..26 {
            TOK[i] = gd_ui_token(i as i32);
        }
        CAP = gd_ui_capture(0x1, CAP_SHARED);
    }
    0
}

fn num(v: i32, out: &mut [u8; 12]) -> &str {
    let mut n = v.unsigned_abs();
    let mut i = out.len();
    loop {
        i -= 1;
        out[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if v < 0 {
        i -= 1;
        out[i] = b'-';
    } else if v > 0 {
        i -= 1;
        out[i] = b'+';
    }
    core::str::from_utf8(&out[i..]).unwrap_or("?")
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe {
        if gd_lab_status(STATUS.as_mut_ptr()) != 0 || STATUS[0] == 0 {
            // no lab match (menus, or a session the Lab is not allowed in): nothing to drive
            SETUP = false;
            STEP = 0;
            return;
        }
        let frame = STATUS[1];
        let guest = STATUS[8] == 1;
        if !SETUP {
            gd_lab_ring(600);
            gd_lab_overlay(0b11, OV);
            SETUP = true;
            toast("LAB ready");
        }
        // manual controls on port 1
        let mut pad = [0u8; 16];
        if gd_ui_pad(0, pad.as_mut_ptr()) == 0 {
            let held = u16::from_le_bytes([pad[0], pad[1]]);
            let edge = u16::from_le_bytes([pad[12], pad[13]]);
            let z = held & 0x10 != 0;
            let a = match (edge & 0xF, z) {
                (0x4, false) => b's',
                (0x8, false) => b'l',
                (0x1, false) => b'w',
                (0x2, false) => b'g',
                (0x1, true) => if RECORDING { b'R' } else { b'r' },
                (0x2, true) => if PLAYING { b'P' } else { b'p' },
                _ => 0,
            };
            if a != 0 {
                MANUAL = true;
                act(a);
            }
        }
        if !MANUAL {
            let plan: &[(i32, u8)] = if guest { &GUEST_PLAN } else { &HOST_PLAN };
            while STEP < plan.len() && frame >= plan[STEP].0 {
                act(plan[STEP].1);
                STEP += 1;
            }
        }
        // event toasts (L-cancel, tech, powershield, wavedash, ...)
        let n = gd_lab_events(EVENTS.as_mut_ptr(), 16);
        for k in 0..n.max(0) as usize {
            let e = &EVENTS[k * 16..k * 16 + 16];
            let a = i32::from_le_bytes([e[8], e[9], e[10], e[11]]);
            match e[0] {
                1 => toast(if a != 0 { "L-cancel OK" } else { "L-cancel miss" }),
                2 => toast(if a != 0 { "Tech" } else { "Missed tech" }),
                3 => toast("Out of shield"),
                7 => toast("Powershield"),
                8 => toast("Wavedash"),
                9 => toast("Ledgedash"),
                _ => {}
            }
        }
        // the frame meter feed
        for p in 0..2 {
            let got = gd_lab_meter(p as u32, METER[p].as_mut_ptr(), 50);
            if got > 0 && (got as usize) < 50 {
                METER[p].copy_within(0..got as usize, 50 - got as usize);
                for c in METER[p][..50 - got as usize].iter_mut() {
                    *c = 0;
                }
            }
        }
        let adv = gd_lab_frame_adv(0, 1);
        // draw: band space (640x480), overlay layer
        let t = &TOK;
        let e = &mut *core::ptr::addr_of_mut!(ENC);
        e.clear();
        e.layer(LAYER_OVERLAY, 0).space(SPACE_BAND, 1.0);
        let mut b = [0u8; 12];
        e.slice(SLICE_FILL_R4, 8.0, 8.0, 150.0, 22.0, t[T_PANEL_HUD]).slice(SLICE_LINE_R4, 8.0, 8.0, 150.0, 22.0, t[T_BORDER]);
        let chip = if STATUS[3] != 0 { "LAB  replay" } else if RECORDING { "LAB  REC" } else if PLAYING { "LAB  play" } else { "LAB" };
        e.text(16.0, 13.0, ROLE_VALUE, t[T_ACCENT], chip);
        e.text_ex(150.0, 13.0, ROLE_VALUE, t[T_TEXT], ALIGN_RIGHT, 0.0, num(frame, &mut b));
        // frame meter: 2 rows x 50 cells of 8x10, gap 2
        let (mx, my) = (170.0f32, 8.0f32);
        e.slice(SLICE_FILL_R4, mx, my, 560.0 - mx + 50.0, 36.0, t[T_PANEL_HUD]);
        for (row, col) in [t[T_P1], t[T_P2]].iter().enumerate() {
            let y = my + 6.0 + row as f32 * 13.0;
            e.rect(mx + 4.0, y, 3.0, 10.0, *col);
            for (i, &k) in METER[row].iter().enumerate() {
                e.rect(mx + 10.0 + i as f32 * 8.0, y, 6.0, 10.0, t[T_CELL + (k as usize).min(5)]);
            }
        }
        e.text_ex(600.0, my + 12.0, ROLE_BIG, t[T_ACCENT], ALIGN_RIGHT, 0.0, num(adv, &mut b));
        // toasts
        for (i, (msg, n, life)) in TOASTS.iter_mut().enumerate() {
            if *life == 0 {
                continue;
            }
            *life -= 1;
            let y = 60.0 + i as f32 * 24.0;
            let a = if *life > 30 { 0xFF } else { *life * 8 };
            let fade = |c: u32| (c & 0xFFFF_FF00) | (((c & 0xFF) * a) / 255);
            e.slice(SLICE_FILL_R4, 220.0, y, 200.0, 20.0, fade(t[T_ACCENT]));
            e.text_ex(320.0, y + 4.0, ROLE_TOAST, fade(t[T_ACCENT_INK]), ALIGN_CENTER, 0.0, core::str::from_utf8(&msg[..*n]).unwrap_or(""));
        }
        let _ = T_MUTED;
        e.submit();
    }
}

#[repr(C)]
pub struct Manifest {
    magic: u32, struct_size: u32, abi_min: u32, abi_max: u32, hook_table_version: u32, mod_version: u32,
    required_features: u32, optional_features: u32, state_bytes: u32, hook_count: u32, hooks: *const u8,
    mod_id: [u8; 32], mod_name: [u8; 48],
}
unsafe impl Sync for Manifest {}

const fn text<const N: usize>(s: &[u8]) -> [u8; N] {
    let mut o = [0u8; N];
    let mut i = 0;
    while i < s.len() {
        o[i] = s[i];
        i += 1;
    }
    o
}

#[no_mangle]
pub static gdmod_abi_version: u32 = 1 << 16;
#[no_mangle]
pub static gdmod_ui_abi: u32 = UI_ABI;
#[no_mangle]
pub static gdmod_lab_abi: u32 = 1 << 16;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_INPUT | FEATURE_LAB | FEATURE_LAB_STATE | FEATURE_LAB_INPUT | FEATURE_LAB_OVERLAY,
    optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.lab_demo"), mod_name: text(b"LAB demo (savestates, rewind, record)"),
};

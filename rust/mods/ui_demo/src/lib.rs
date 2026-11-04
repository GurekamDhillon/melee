//! ui_demo: a gd-ui-v1 presentation mod (demo only; the real LAB comes later). It draws the panels of the LAB's
//! "Frame data" HUD preset (lab-ui-spec.md 5.1: session chip, toasts, frame meter, move readout) in theme A with
//! static data, in window space (GAP-0) on the 1280x720 layout scaled to the window. D-pad left opens a host-node menu
//! (tabs, rows, a help panel: about 60 nodes, the "full menu" of the budgets); B closes it.
//!
//! One `gd_ui_submit` per present, from a fixed buffer; no allocation (no allocator at all).
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

static mut ENC: Enc<8192> = Enc::new();
static mut TOK: [u32; 26] = [0; 26];
static mut MENU: u32 = 0;
static mut MENU_OPEN: bool = false;
static mut SHARED_CAP: u32 = 0;
static mut EXCL_CAP: u32 = 0;
static mut HATCH: u32 = 0;

const T_PANEL_HUD: usize = 1;
const T_BORDER: usize = 2;
const T_TEXT: usize = 3;
const T_MUTED: usize = 4;
const T_ACCENT: usize = 5;
const T_ACCENT_INK: usize = 6;
const T_DIVIDER: usize = 7;
const T_P1: usize = 10;
const T_P2: usize = 11;
const T_CELL: usize = 12; // idle, startup, active, recovery, stun, can act

fn s(x: &[u8]) -> (*const u8, u32) {
    (x.as_ptr(), x.len() as u32)
}

/// Frame meter cells (static demo data): 0 idle 1 startup 2 active 3 recovery 4 stun 5 can act.
const P1: [u8; 50] = [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const P2: [u8; 50] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const MOVE: [(&str, &str); 6] = [("Startup", "5f"), ("Active", "5-7"), ("Landing lag", "9f / L 4f"), ("Damage", "14%"), ("IASA", "33"), ("On shield", "-6")];

const TABS: &[u8] = b"Dummy\0Record\0Display\0HUD\0Drills\0Session\0";
const ROWS: [(&[u8], i32); 18] = [
    (b"Control", NODE_CHOICE), (b"Block", NODE_CHOICE), (b"Shield hold", NODE_SLIDER), (b"After block", NODE_CHOICE),
    (b"After hit", NODE_CHOICE), (b"On wakeup", NODE_CHOICE), (b"DI", NODE_CHOICE), (b"DI strength", NODE_SLIDER),
    (b"SDI", NODE_CHOICE), (b"Tech", NODE_CHOICE), (b"Getup", NODE_CHOICE), (b"Ledge", NODE_CHOICE),
    (b"Counter delay", NODE_SLIDER), (b"Start percent", NODE_SLIDER), (b"Lock percent", NODE_TOGGLE), (b"Reset on load", NODE_TOGGLE),
    (b"Intangible", NODE_TOGGLE), (b"Reset position", NODE_BUTTON),
];
const CHOICES: &[u8] = b"None\0In\0Out\0Away\0Toward\0Random\0";
const HELP: [&[u8]; 20] = [
    b"DI", b"How the dummy", b"influences the", b"knockback of the", b"next hit.", b"", b"None: no DI.", b"In: toward the",
    b"stage centre.", b"Out: away from it.", b"Random: a draw", b"from the game RNG", b"(replayable).", b"",
    b"Strength 0-100%", b"scales the stick.", b"", b"L / R: tabs", b"A: change", b"B: close",
];

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe {
        gd_ui_theme(0);
        for i in 0..26 {
            TOK[i] = gd_ui_token(i as i32);
        }
        let (p, l) = s(b"ui.hatch");
        HATCH = gd_ui_kit_id(p, l);
        // the menu: host nodes (focus, wrap, repeat and look are the host's)
        let root = gd_ui_node(0, NODE_PANEL, 1);
        let (p, l) = s(b"LAB MENU");
        gd_ui_node_label(root, p, l);
        gd_ui_node_rect(root, 16.0, 16.0, 330.0, 0.0);
        let tabs = gd_ui_node(root, NODE_TABS, 2);
        let (p, l) = s(TABS);
        gd_ui_node_choices(tabs, p, l);
        let list = gd_ui_node(root, NODE_LIST, 3);
        for (i, &(name, kind)) in ROWS.iter().enumerate() {
            let n = gd_ui_node(list, kind, 100 + i as u32);
            let (p, l) = s(name);
            gd_ui_node_label(n, p, l);
            match kind {
                NODE_CHOICE => {
                    let (p, l) = s(CHOICES);
                    gd_ui_node_choices(n, p, l);
                    gd_ui_node_value(n, (i % 6) as i32);
                }
                NODE_SLIDER => {
                    gd_ui_node_range(n, 0, 100, 5);
                    gd_ui_node_value(n, 40 + i as i32);
                }
                _ => {}
            }
        }
        // the help pane (part 11): a second panel of labels
        let help = gd_ui_node(0, NODE_PANEL, 4);
        let (p, l) = s(b"HELP");
        gd_ui_node_label(help, p, l);
        gd_ui_node_rect(help, 356.0, 16.0, 268.0, 0.0);
        for (i, line) in HELP.iter().enumerate() {
            let n = gd_ui_node(help, NODE_LABEL, 200 + i as u32);
            let (p, l) = s(line);
            gd_ui_node_label(n, p, l);
        }
        MENU = root;
        // help is a sibling root shown with the menu
        MENU_HELP = help;
        SHARED_CAP = gd_ui_capture(0xF, CAP_SHARED);
    }
    0
}

static mut MENU_HELP: u32 = 0;

#[no_mangle]
pub extern "C" fn gd_ui_event(_node: u32, ev: i32, _value: i32) {
    if ev == EV_BACK {
        close_menu();
    }
}

fn close_menu() {
    unsafe {
        if MENU_OPEN {
            gd_ui_hide(MENU);
            gd_ui_hide(MENU_HELP);
            gd_ui_release(EXCL_CAP);
            MENU_OPEN = false;
        }
    }
}

fn panel(e: &mut Enc<8192>, x: f32, y: f32, w: f32, h: f32) {
    let t = unsafe { &TOK };
    e.slice(SLICE_FILL_R4, x, y, w, h, t[T_PANEL_HUD]).slice(SLICE_LINE_R4, x, y, w, h, t[T_BORDER]);
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe {
        // D-pad left (a new press on any port) opens the menu
        let mut pad = [0u8; 16];
        let mut pressed = 0u16;
        for port in 0..4 {
            if gd_ui_pad(port, pad.as_mut_ptr()) == 0 {
                pressed |= u16::from_le_bytes([pad[12], pad[13]]);
            }
        }
        if pressed & 0x1 != 0 && !MENU_OPEN {
            gd_ui_show(MENU, LAYER_MENU, 0);
            gd_ui_show(MENU_HELP, LAYER_MENU, 0);
            EXCL_CAP = gd_ui_capture(0x1, CAP_EXCLUSIVE);
            MENU_OPEN = true;
        }
        let t = &TOK;
        let mut b = [0f32; 6];
        gd_ui_band_rect(b.as_mut_ptr());
        let sc = (b[5] / 720.0).max(0.25);
        let vw = b[4] / sc; // virtual width of the 720-high layout
        let e = &mut *core::ptr::addr_of_mut!(ENC);
        e.clear();
        e.layer(LAYER_OVERLAY, 0).space(SPACE_WINDOW, sc);
        // session chip (part 2): 20, 20
        let chip = "LAB · Frame data";
        let cw = measure(chip, ROLE_VALUE) + 30.0;
        e.slice(SLICE_FILL_R4, 20.0, 20.0, cw, 24.0, t[T_PANEL_HUD]).slice(SLICE_LINE_R4, 20.0, 20.0, cw, 24.0, t[T_BORDER]);
        e.kit(1, 28.0, 27.0, 1.0, t[T_ACCENT]);
        e.text(44.0, 26.0, ROLE_VALUE, t[T_TEXT], chip);
        // frame meter (part 4): centred, y 20, 700 wide; two rows of 50 cells 9x14, gap 2
        let (mx, my, mw) = ((vw - 700.0) * 0.5, 20.0, 700.0);
        panel(e, mx, my, mw, 82.0);
        e.text(mx + 12.0, my + 10.0, ROLE_LABEL, t[T_MUTED], "Frame meter");
        e.text_ex(mx + mw - 12.0, my + 10.0, ROLE_LABEL, t[T_MUTED], ALIGN_RIGHT, 0.0, "advantage");
        for (row, (cells, col, name)) in [(&P1, t[T_P1], "P1"), (&P2, t[T_P2], "P2")].iter().enumerate() {
            let y = my + 28.0 + row as f32 * 22.0;
            e.text(mx + 12.0, y + 1.0, ROLE_VALUE, *col, name);
            for (i, &k) in cells.iter().enumerate() {
                let x = mx + 44.0 + i as f32 * 11.0;
                e.slice(SLICE_FILL_R1, x, y, 9.0, 14.0, t[T_CELL + k as usize]);
                if k == 4 {
                    // stun: the colour-blind hatch on top
                    e.kit(HATCH, x, y, 1.0, 0x0000_0040);
                }
            }
        }
        e.text_ex(mx + mw - 14.0, my + 34.0, ROLE_BIG, t[T_ACCENT], ALIGN_RIGHT, 0.0, "+3");
        // move readout (part 5): right 20, y 20, 220 wide, 6 rows
        let (rx, ry, rw) = (vw - 20.0 - 220.0, 20.0, 220.0);
        panel(e, rx, ry, rw, 30.0 + 6.0 * 22.0 + 8.0);
        e.text(rx + 12.0, ry + 10.0, ROLE_LABEL, t[T_MUTED], "Move · Fox fair");
        for (i, (k, v)) in MOVE.iter().enumerate() {
            let y = ry + 30.0 + i as f32 * 22.0;
            if i > 0 {
                e.rect(rx + 10.0, y, rw - 20.0, 1.0, t[T_DIVIDER]);
            }
            e.text(rx + 12.0, y + 5.0, ROLE_VALUE, t[T_MUTED], k);
            e.text_ex(rx + rw - 12.0, y + 5.0, ROLE_VALUE, t[T_TEXT], ALIGN_RIGHT, 0.0, v);
        }
        // toasts (part 3): centred at y 120, 300 wide, stacked with fading opacity
        for (i, (msg, a)) in [("L-cancel  5/7f", 0xFFu32), ("Tech  early 2f", 0xBF)].iter().enumerate() {
            let (tx, ty) = ((vw - 300.0) * 0.5, 120.0 + i as f32 * 34.0);
            let fade = |c: u32| (c & 0xFFFF_FF00) | (((c & 0xFF) * *a) / 255);
            e.slice(SLICE_FILL_R4, tx, ty, 300.0, 28.0, fade(t[T_ACCENT]));
            e.text_ex(tx + 150.0, ty + 7.0, ROLE_TOAST, fade(t[T_ACCENT_INK]), ALIGN_CENTER, 0.0, msg);
        }
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
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_MENU | FEATURE_UI_INPUT, optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.ui_demo"), mod_name: text(b"UI demo (Frame data HUD)"),
};

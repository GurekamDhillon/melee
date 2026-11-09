//! tag_fighter_ui: the tag cinematic, presentation only (gd-ui-v1). It reads the engine's transition record
//! (`gd_ui_transition`, read-only) and draws from the sim clock in it; it never writes the sim, so a rollback or a
//! skipped present cannot change the match. Timeline (the freeze is 66 logic frames, rust/mods/tag_fighter):
//! * freeze 0-10: a white flash on the outgoing fighter's frame; letterbox bars close in (0-18);
//! * freeze 8-56: the name card: "P<n>" and the outgoing name, an arrow, the incoming name sliding in;
//! * freeze 46-66: a slanted wipe crosses the screen and covers it at the swap;
//! * after the release 0-18: the wipe leaves to the right, revealing the incoming fighter; the bars open.
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_transition(out: *mut i32) -> i32;
}

static mut ENC: Enc<4096> = Enc::new();

const LEN: f32 = 66.0;
const REVEAL: i32 = 18;
/// Fighter kinds (ft/forward.h FighterKind order).
const NAMES: [&str; 27] = [
    "MARIO", "FOX", "CAPTAIN FALCON", "DONKEY KONG", "KIRBY", "BOWSER", "LINK", "SHEIK", "NESS", "PEACH", "POPO",
    "NANA", "PIKACHU", "SAMUS", "YOSHI", "JIGGLYPUFF", "MEWTWO", "LUIGI", "MARTH", "ZELDA", "YOUNG LINK", "DR. MARIO",
    "FALCO", "PICHU", "MR. GAME & WATCH", "GANONDORF", "ROY",
];
const PORT: [u32; 4] = [0xE8_3A3A_FF, 0x3A6E_E8FF, 0xF0C0_30FF, 0x3AB8_5AFF];

fn name(k: i32) -> &'static str {
    if (0..27).contains(&k) { NAMES[k as usize] } else { "FIGHTER" }
}

fn alpha(c: u32, a: f32) -> u32 {
    let a = if a < 0.0 { 0.0 } else if a > 1.0 { 1.0 } else { a };
    (c & 0xFFFF_FF00) | ((((c & 0xFF) as f32) * a) as u32)
}

fn ease(t: f32) -> f32 {
    let t = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    t * t * (3.0 - 2.0 * t)
}

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe { gd_ui_theme(0) };
    0
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe {
        let mut t = [0i32; 10];
        if gd_ui_transition(t.as_mut_ptr()) != 0 {
            return;
        }
        let (phase, slot, frame, since, from, to, skipped) = (t[0], t[2], t[3] as f32, t[5], t[7], t[8], t[9] != 0);
        let frozen = phase == 1;
        if !frozen && since >= REVEAL {
            return;
        }
        let mut b = [0f32; 6];
        gd_ui_band_rect(b.as_mut_ptr());
        let sc = (b[5] / 720.0).max(0.25);
        let vw = b[4] / sc;
        let vh = 720.0;
        let e = &mut *core::ptr::addr_of_mut!(ENC);
        e.clear();
        e.layer(LAYER_OVERLAY, 10).space(SPACE_WINDOW, sc);
        let port = PORT[(slot.clamp(0, 3)) as usize];
        if frozen {
            // letterbox (the "push in")
            let bar = 84.0 * ease(frame / 18.0);
            e.rect(0.0, 0.0, vw, bar, 0x0000_00E6).rect(0.0, vh - bar, vw, bar, 0x0000_00E6);
            // flash
            if frame < 10.0 {
                e.rect(0.0, 0.0, vw, vh, alpha(0xFFFF_FFFF, 0.85 * (1.0 - frame / 10.0)));
            }
            // name card
            if (8.0..56.0).contains(&frame) {
                let a = ease((frame - 8.0) / 8.0) * (1.0 - ease((frame - 48.0) / 8.0));
                let y = vh * 0.5 - 40.0;
                let cx = vw * 0.5;
                e.rect_shear(cx - 380.0, y - 8.0, 760.0, 96.0, alpha(0x0A0A_14FF, 0.82 * a), -0.25);
                e.rect(cx - 380.0, y - 8.0, 8.0, 96.0, alpha(port, a));
                let mut pn = [b'P', b'1'];
                pn[1] = b'1' + slot.clamp(0, 3) as u8;
                e.text(cx - 360.0, y + 2.0, ROLE_LABEL, alpha(port, a), core::str::from_utf8_unchecked(&pn));
                let out_x = cx - 200.0 - 30.0 * ease((frame - 30.0) / 20.0);
                e.text_ex(out_x, y + 24.0, ROLE_TITLE, alpha(0xB8B8_C8FF, a), ALIGN_CENTER, 0.0, name(from));
                e.text_ex(cx, y + 24.0, ROLE_TITLE, alpha(port, a * ease((frame - 18.0) / 6.0)), ALIGN_CENTER, 0.0, ">>");
                let in_t = ease((frame - 22.0) / 12.0);
                let in_x = cx + 200.0 + (1.0 - in_t) * 260.0;
                e.text_ex(in_x, y + 24.0, ROLE_TITLE, alpha(0xFFFF_FFFF, a * in_t), ALIGN_CENTER, 0.0, name(to));
                e.text_ex(cx, y + 70.0, ROLE_LABEL, alpha(0x9090_A0FF, a), ALIGN_CENTER, 0.0, "TAG IN  ·  D-pad down to skip");
            }
            // wipe in
            if frame >= 46.0 {
                let p = ease((frame - 46.0) / (LEN - 1.0 - 46.0));
                let w = (vw + 400.0) * p;
                e.rect_shear(-200.0, 0.0, w, vh, port, -0.35);
                e.rect_shear(-200.0, 0.0, w - 40.0, vh, 0x0A0A_14FF, -0.35);
            }
        } else if !skipped || since < REVEAL {
            // reveal: the wipe leaves to the right, the bars open
            let p = ease(since as f32 / REVEAL as f32);
            let x = -200.0 + (vw + 400.0) * p;
            e.rect_shear(x, 0.0, vw + 400.0, vh, port, -0.35);
            e.rect_shear(x + 40.0, 0.0, vw + 400.0, vh, 0x0A0A_14FF, -0.35);
            let bar = 84.0 * (1.0 - p);
            e.rect(0.0, 0.0, vw, bar, 0x0000_00E6).rect(0.0, vh - bar, vw, bar, 0x0000_00E6);
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
    required_features: FEATURE_UI_DRAW, optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.tag_fighter_ui"), mod_name: text(b"Tag fighter cinematic"),
};

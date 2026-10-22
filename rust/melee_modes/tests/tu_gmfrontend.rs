//! State oracle for tu/gmfrontend.rs (src/oracle.rs, gen/tu/run.py; the C side is the patched copy
//! gen/tu/patches/gmfrontend.py, which replaces the fifteen `gmfrontend_*.inc` includes by recorded
//! stand-ins). Every function of src/melee/gm/gmfrontend.c runs on a synthetic frontend scene in the C
//! compiled natively and in the Rust port, and the resulting state is compared.
use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmfrontend as tu;
use melee_modes::tu::gmfrontend::ext;
use melee_modes::GXColor;
use melee_types::*;
use std::vec::Vec;

fn pick<T: Copy>(c: &mut Case, xs: &[T]) -> T {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

fn chance(c: &mut Case, n: u64, d: u64) -> bool {
    c.rng.chance(n, d)
}

fn rgba(c: &mut Case) -> GXColor {
    GXColor { r: c.rng.range(0, 255) as u8, g: c.rng.range(0, 255) as u8, b: c.rng.range(0, 255) as u8, a: c.rng.range(0, 255) as u8 }
}

/// A C string in the arena (`n` letters from a small set, a NUL after).
fn cstr(c: &mut Case, n: usize) -> *mut i8 {
    let p = c.zbuf(n + 8);
    let set = b"abcdeXYZ 019-";
    unsafe {
        for i in 0..n {
            *p.add(i) = set[c.rng.range(0, set.len() as i64 - 1) as usize];
        }
    }
    p.cast()
}

// ---- the callees' return values -----------------------------------------------------------------------

fn rets(c: &mut Case) {
    let rules = c.obj::<GameRules>();
    unsafe { (*rules).mode = pick(c, &[0u8, 1, 1, 2, 3]) };
    c.ret_ptr("gmMainLib_GetGameRules", rules);
    let prefs = c.obj::<GamePrefs>();
    c.ret_ptr("gmMainLib_GetGamePrefs", prefs);
    for n in [
        "Frontend_NativeSelect", "Frontend_TrainingSelect", "Gfx_LoadScreenEnabled", "Netplay_EnvoyAvailable", "Netplay_FighterAvailable",
        "Netplay_LobbyActive", "Netplay_MenuHasServer", "Ui_CssKnown", "Ui_Ready", "Ui_SelCanOpen", "fa_room_on", "fas_active", "fcr_frame",
        "fe_is_controls_help", "fe_is_settings", "fm_replaced", "fms_in_quad", "fss_frame", "fss_intercepts", "fte_frame",
    ] {
        c.ret_int(n, 0, 1);
    }
    for n in ["Netplay_MenuBegin", "Netplay_RandomBegin", "Netplay_FighterWhy"] {
        c.ret_int(n, 0, 1);
    }
    c.ret_int("PcFrontendEnabled", 0, 1);
    c.ret_int("PcFrontendMenusEnabled", 0, 1);
    c.ret_int("Settings_Int", 0, 1);
    c.ret_int("Netplay_StageMode", 0, 1);
    c.ret_int("Netplay_TurboPref", 0, 1);
    c.ret_int("Netplay_EnvoyPref", 0, 1);
    c.ret_int("Gfx_PipelinesCreated", 0, 400);
    c.ret_int("Gfx_PipelinesPending", 0, 30);
    c.ret_int("Gfx_PipelinesUrgent", 0, 30);
    c.ret_int("Gfx_SeedCoreCount", 0, 300);
    c.ret_int("Gfx_SeedCorePending", 0, 300);
    c.ret_int("Gfx_SeedPipelinesBuilt", 0, 300);
    c.ret_int("mn_80229624", 0, 0x3F);
    c.ret_int("fms_menu_bits", 0, 0x3F);
    c.ret_pool("HSD_SisLib_803A6754", core::mem::size_of::<HSD_Text>());
    c.ret_int("HSD_SisLib_803A6B98", 0, 5);
    c.ret_int("HSD_SisLib_803A611C", 0, 20);
    c.ret_pool("GObj_Create", 8);
    c.ret_pool("HSD_MemAlloc", 4096);
    c.ret_int("GxTex_OpenUI", -1, 3);
    c.ret_int("GxTex_ImageSize", 1, 600);
    c.ret_int("GxTex_TlutFormat", -1, 2);
    c.ret_int("GxTex_TlutSize", 1, 600);
    c.ret_int("GxTex_Width", 1, 256);
    c.ret_int("GxTex_Height", 1, 256);
    c.ret_int("GxTex_Format", 0, 14);
    c.ret_int("GxTex_TlutEntries", 1, 256);
    c.out("fe_panel_rect", 0, 16);
    c.mem_arg("fe_panel_art_frame", 0, 16);
    c.mem_arg("fe_panel_art_image", 1, 16);
    c.mem_arg("HSD_SisLib_803A74F0", 2, 4);
    c.str_arg("HSD_SisLib_803A6B98", 3);
    c.str_arg("SceneLaunch_SetText", 0);
    c.str_arg("Settings_Int", 0);
    c.str_arg("Settings_SetInt", 0);
    c.str_arg("GxTex_OpenUI", 0);
    c.str_arg("fl_lobby_notice", 0);
    c.out("Netplay_MenuStatus", 0, 8);
    c.out("Netplay_FighterWhy", 1, 8);
    c.out("fm_position_for", 1, 1);
    c.out("fm_position_for", 2, 1);
}

// ---- the scene -----------------------------------------------------------------------------------------

fn screens(c: &mut Case) -> *const FrontendScreen {
    unsafe {
        let s: [*const FrontendScreen; 11] = [
            addr_of!(tu::fe_screen_vs_setup),
            addr_of!(tu::fe_screen_online),
            addr_of!(tu::fe_screen_code),
            addr_of!(tu::fe_screen_wait),
            addr_of!(tu::fe_screen_lobby),
            addr_of!(tu::fe_screen_loading),
            addr_of!(tu::fe_screen_css),
            addr_of!(tu::fe_screen_sss),
            addr_of!(tu::fe_screen_remap),
            addr_of!(tu::fe_screen_settings) as *const FrontendScreen,
            addr_of!(tu::fe_screen_vs_setup),
        ];
        s[c.rng.range(0, 10) as usize]
    }
}

fn mk_text(c: &mut Case) -> *mut HSD_Text {
    let t = c.zobj::<HSD_Text>();
    unsafe {
        (*t).pos_x = c.rng.f32_in(-10.0, 700.0);
        (*t).pos_y = c.rng.f32_in(-10.0, 500.0);
        (*t).font_size.x = pick(c, &[0.5f32, 0.56, 0.9]);
        (*t).font_size.y = (*t).font_size.x;
        (*t).default_alignment = pick(c, &[0u8, 1, 2]);
        (*t).hidden = pick(c, &[0u8, 1]);
    }
    t
}

fn put_str(dst: *mut i8, s: *const i8, cap: usize) {
    unsafe {
        let mut i = 0;
        while *s.add(i) != 0 && i < cap - 1 {
            *dst.add(i) = *s.add(i);
            i += 1;
        }
        *dst.add(i) = 0;
    }
}

/// Set by the builders of the functions that tolerate `fe.screen == NULL`.
static ALLOW_NULL: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// The inc-owned views, the item tables' state and `fe` around a screen (never NULL).
fn mk_scene(c: &mut Case) {
    mk_scene_x(c, false);
}

/// As [`mk_scene`]; the screen is sometimes NULL (the functions that test for it).
fn mk_scene_n(c: &mut Case) {
    mk_scene_x(c, true);
}

/// 1 = bias the scene towards the main path (a rows screen, nothing else active).
static MODE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// Force a callee's return value (call before `mk_scene`: the first spec of a name wins).
fn pre(c: &mut Case, xs: &[(&str, i64)]) {
    for (n, v) in xs {
        c.ret_int(n, *v, *v);
    }
}

fn mk_scene_main(c: &mut Case) {
    MODE.store(1, core::sync::atomic::Ordering::Relaxed);
    mk_scene_x(c, false);
    MODE.store(0, core::sync::atomic::Ordering::Relaxed);
}

fn mk_scene_x(c: &mut Case, allow_null: bool) {
    let _ = &ALLOW_NULL;
    let main = MODE.load(core::sync::atomic::Ordering::Relaxed) == 1 && c.rng.chance(4, 5);
    unsafe {
        let fe = &mut *addr_of_mut!(tu::fe);
        fe.screen = if main {
            if chance(c, 1, 2) { addr_of!(tu::fe_screen_vs_setup) } else { addr_of!(tu::fe_screen_online) }
        } else if allow_null && chance(c, 1, 14) {
            core::ptr::null()
        } else {
            screens(c)
        };
        fe.continue_to = pick(c, &[GM_VS as u8, GM_MENU as u8, 3]);
        fe.back_to = pick(c, &[GM_MENU as u8, GM_VS as u8]);
        fe.reported = pick(c, &[GM_VS as u8, GM_MENU as u8, tu::GM_FRONTEND, 5]);
        fe.canvas = c.rng.range(0, 9) as i32;
        fe.frames = pick(c, &[0, 1, 3, 4, 5, 20, 100, 1000]);
        fe.fade = pick(c, &[0, 1, 5, 9, 10, 11]);
        fe.leaving = pick(c, &[0, 0, 0, 1, 2]);
        fe.hl_y = if chance(c, 1, 2) { c.rng.f32_in(0.0, 7.0) } else { 0.0 } + pick(c, &[0.0f32, 0.35, 1.0, 2.5, 3.0, 6.9, 0.123, 4.71, 1.37]);
        fe.loading = pick(c, &[0, 0, 0, 1]);
        fe.warmed = pick(c, &[0, 1]);
        fe.next_menus = pick(c, &[0, 0, 1]);
        fe.load_base = c.rng.range(0, 100) as i32;
        fe.load_core = pick(c, &[0, 10, 100]);
        fe.load_settled = pick(c, &[0, 0, 5, 19, 20]);
        fe.load_frames = pick(c, &[0, 10, 44, 45, 479, 480]);
        fe.progress = if chance(c, 1, 2) { c.rng.f32_in(0.0, 1.0) } else { 0.0 } + pick(c, &[0.0f32, 0.5, 0.9, 0.99, 0.991, 0.994, 0.996, 1.0, 0.37, 0.123]);
        fe.percent = pick(c, &[-1, 0, 50, 100]);
        fe.help_item = pick(c, &[-1, 0, 3]);
        for k in 0..6 {
            let t = if chance(c, 1, 3) { null_mut() } else { mk_text(c) };
            match k {
                0 => fe.title = t,
                1 => fe.subtitle = t,
                2 => fe.help = t,
                3 => fe.hints = t,
                4 => fe.hint_change = t,
                _ => fe.hint_back = t,
            }
        }
        let n_items = if fe.screen.is_null() { 0 } else { (*fe.screen).n_items };
        let n_list = if n_items > 0 { c.rng.range(0, n_items as i64) as i32 } else { 0 };
        let mut used = [false; 32];
        for i in 0..n_list as usize {
            let mut k = c.rng.range(0, n_items as i64 - 1) as usize;
            while used[k] {
                k = (k + 1) % n_items as usize;
            }
            used[k] = true;
            fe.vis[i] = k as i32;
        }
        fe.n_list = n_list;
        fe.has_button = if n_items > 0 && chance(c, 1, 3) { 1 } else { 0 };
        fe.n_vis = n_list;
        if fe.has_button != 0 {
            let mut k = c.rng.range(0, n_items as i64 - 1) as usize;
            while used[k] {
                k = (k + 1) % n_items as usize;
                if k == 0 && used.iter().take(n_items as usize).all(|u| *u) {
                    break;
                }
            }
            fe.vis[fe.n_vis as usize] = k as i32;
            fe.n_vis += 1;
        }
        fe.cursor = if fe.n_vis > 0 { c.rng.range(0, fe.n_vis as i64 - 1) as i32 } else { pick(c, &[0, 1]) };
        fe.scroll = if n_list > 0 { c.rng.range(0, (n_list - n_list.min(7)) as i64) as i32 } else { 0 };
        for slot in 0..7usize {
            fe.label[slot] = if chance(c, 1, 5) { null_mut() } else { mk_text(c) };
            fe.value[slot] = if chance(c, 1, 5) { null_mut() } else { mk_text(c) };
            // what each text currently shows: sometimes the item's own label
            let mut lbl = { let n_ = c.rng.range(0, 8) as usize; cstr(c, n_) };
            if n_items > 0 && chance(c, 1, 2) {
                let idx = c.rng.range(0, n_items as i64 - 1) as isize;
                lbl = (*(*fe.screen).items.offset(idx)).label as *mut i8;
            }
            put_str(fe.label_str[slot].as_mut_ptr(), lbl, 512);
            let v = { let n_ = c.rng.range(0, 5) as usize; cstr(c, n_) };
            put_str(fe.value_str[slot].as_mut_ptr(), v, 80);
        }
        let hs = { let n_ = c.rng.range(0, 6) as usize; cstr(c, n_) };
        put_str(fe.help_str.as_mut_ptr(), hs, 512);
        if chance(c, 1, 4) {
            let n = { let n_ = c.rng.range(1, 12) as usize; cstr(c, n_) };
            put_str(addr_of_mut!(tu::fe_ol_note) as *mut i8, n, 512);
        }
        tu::fe_ol_note_frames = pick(c, &[0, 100, 299, 300, 301]);
        tu::fe_kit = pick(c, &[0, 0, 1]);
        tu::fe_np_pick = pick(c, &[0, 0, 1]);
        tu::fe_np_phase = pick(c, &[0, 1, 2]);
        tu::fe_np_ck = c.rng.range(0, 40) as i32;
        tu::fe_np_color = c.rng.range(0, 5) as i32;
        tu::fe_np_stocks = c.rng.range(1, 9) as i32;
        tu::fe_np_minutes = c.rng.range(1, 20) as i32;
        tu::fe_np_delay = c.rng.range(0, 8) as i32;
        tu::fe_enabled_cache = pick(c, &[-1, -1, 0, 1]);
        tu::fe_menus_on_cache = pick(c, &[-1, -1, 0, 1]);
        tu::fm_back_to_online_item = pick(c, &[0, 1]);
        tu::fm_back_kind = pick(c, &[-1, 0, 4]);
        tu::fm_back_sel = c.rng.range(0, 70) as i32;
        // views
        let menu = c.zobj::<tu::FeMenu>();
        (*menu).style = pick(c, &[0u8, 1]);
        tu::fm.menu = if chance(c, 1, 8) { null_mut() } else { menu };
        tu::fm.active = pick(c, &[0, 0, 0, 1]);
        if tu::fm.active != 0 && tu::fm.menu.is_null() {
            tu::fm.menu = menu;
        }
        tu::fm.start_kind = c.rng.range(0, 9) as u8;
        tu::fm.start_sel = c.rng.range(0, 9) as u8;
        tu::fm.native_req = pick(c, &[0, 1]);
        tu::fm.native_kind = c.rng.range(0, 9) as u8;
        tu::fm.native_sel = c.rng.range(0, 9) as u8;
        tu::fm.nvis = c.rng.range(0, 12) as i32;
        tu::fm.fade = pick(c, &[0, 1, 4, 8]);
        tu::fk.on = pick(c, &[0, 0, 0, 1]);
        tu::fk.nrows = c.rng.range(0, 12) as i32;
        for i in 0..12 {
            tu::fk.tile[i] = c.rng.range(0, 15) as i32;
        }
        tu::fk.scroll = c.rng.range(0, 5) as i32;
        tu::fl.on = pick(c, &[0, 0, 0, 1]);
        tu::fp.ok = pick(c, &[0, 1]);
        tu::fp.frame = c.rng.range(0, 100) as i32;
        for i in 0..16 {
            tu::fp.tile[i] = tu::FpTile { q_plate: c.rng.range(-1, 20) as i32, q_face: c.rng.range(-1, 20) as i32 };
        }
        tu::fms.moved = pick(c, &[0, 1]);
        tu::fms.click = pick(c, &[0, 1]);
        tu::fms.wheel = pick(c, &[0, 0, 1, -1, 2, -3]);
        tu::fte.on = pick(c, &[0, 0, 0, 1]);
        tu::ff.state = pick(c, &[0, 1, 1, -1]);
        tu::kit.state = pick(c, &[0, 1, 1, -1]);
        tu::fa_mods_door = pick(c, &[0, 1]);
        tu::fa_page_from_main = pick(c, &[0, 1]);
        tu::fm_scroll = c.rng.range(0, 5) as i32;
        tu::fcr_port = c.rng.range(0, 3) as i32;
        tu::fss.want_label = null_mut();
        let sm = cstr(c, 4) as *const i8;
        tu::fe_sel_mode_pending = pick(c, &[core::ptr::null::<i8>(), sm]);
        tu::fe_sel_mode = core::ptr::null();
        let b_ = c.buf(16).cast::<c_void>();
        tu::fe_ats_css = pick(c, &[null_mut(), b_]);
        let b_ = c.buf(16).cast::<c_void>();
        tu::fe_ats_sss = pick(c, &[null_mut(), b_]);
        let b_ = c.buf(16).cast::<c_void>();
        tu::fe_train_css = pick(c, &[null_mut(), b_]);
        let b_ = c.buf(16).cast::<c_void>();
        tu::fe_train_sss = pick(c, &[null_mut(), b_]);
        for i in 0..20usize {
            let t = &mut tu::fe_tex[i];
            t.ok = pick(c, &[0, 1]);
            t.lut = if chance(c, 1, 2) { null_mut() } else { c.buf(16).cast() };
        }
    }
    if main {
        unsafe {
            tu::fm.active = 0;
            tu::fk.on = if chance(c, 1, 3) { 1 } else { 0 };
            tu::fl.on = 0;
            tu::fte.on = 0;
            let fe = &mut *addr_of_mut!(tu::fe);
            fe.loading = 0;
            fe.leaving = if chance(c, 1, 8) { 1 } else { 0 };
            fe.frames = pick(c, &[4, 5, 30]);
            fe.fade = pick(c, &[0, 3]);
            tu::fe_kit = if tu::fk.on != 0 { 1 } else { pick(c, &[0, 0, 1]) };
            tu::fms.wheel = pick(c, &[0, 0, 1, -1, 2]);
            if chance(c, 2, 3) {
                let n = { let n_ = c.rng.range(1, 12) as usize; cstr(c, n_) };
                put_str(addr_of_mut!(tu::fe_ol_note) as *mut i8, n, 512);
                tu::fe_ol_note_frames = pick(c, &[299, 300, 301, 0]);
            }
            if fe.has_button != 0 && chance(c, 1, 2) {
                fe.cursor = fe.n_vis - 1;
            }
            tu::fm.fade = pick(c, &[0, 4, 8, 8]);
        }
    }
    rets(c);
}

fn b_scene(c: &mut Case) {
    mk_scene(c);
}

fn b_scene_n(c: &mut Case) {
    mk_scene_n(c);
}

fn gobj_pass(c: &mut Case) {
    if chance(c, 3, 4) {
        pre(c, &[("fas_active", 0)]);
        mk_scene_main(c);
    } else {
        mk_scene_n(c);
    }
    let g = c.gobj_any();
    c.arg(g);
    let pass = pick(c, &[0i32, 0, 0, 1, 2, 2]);
    c.arg(pass);
}

fn b_fade(c: &mut Case) {
    mk_scene_n(c);
    unsafe {
        if chance(c, 1, 2) {
            tu::fm.active = 1;
            tu::fm.fade = pick(c, &[0, 1, 4, 8, 8]);
        } else {
            tu::fm.active = 0;
            tu::fe.fade = pick(c, &[-1, 0, 3, 10, 9]);
        }
    }
    let g = c.gobj_any();
    c.arg(g);
    let pass = pick(c, &[2i32, 2, 2, 0, 1]);
    c.arg(pass);
}

/// The screens the inc files own (remap, settings) have no title here, and fe_text reads one.
fn titled(c: &mut Case) {
    unsafe {
        let fe = &mut *addr_of_mut!(tu::fe);
        let s = fe.screen;
        if s.is_null() || (*s).title.is_null() {
            fe.screen = screens_titled(c);
        }
    }
}

fn screens_titled(c: &mut Case) -> *const FrontendScreen {
    unsafe {
        let s: [*const FrontendScreen; 8] = [
            addr_of!(tu::fe_screen_vs_setup),
            addr_of!(tu::fe_screen_online),
            addr_of!(tu::fe_screen_code),
            addr_of!(tu::fe_screen_wait),
            addr_of!(tu::fe_screen_lobby),
            addr_of!(tu::fe_screen_loading),
            addr_of!(tu::fe_screen_css),
            addr_of!(tu::fe_screen_sss),
        ];
        s[c.rng.range(0, 7) as usize]
    }
}

/// The enter data / state arguments of the scene's entry points.
fn b_enter(c: &mut Case) {
    mk_scene_n(c);
    titled(c);
    let css = unsafe { addr_of_mut!(tu::gmVsMelee_CssData) as *mut c_void };
    let sss = unsafe { addr_of_mut!(tu::gmVsMelee_SssData) as *mut c_void };
    let d = unsafe {
        match c.rng.range(0, 9) {
            0 => null_mut(),
            1 | 2 => css,
            3 | 4 => sss,
            5 => tu::fe_ats_css,
            6 => tu::fe_ats_sss,
            7 => tu::fe_train_css,
            8 => tu::fe_train_sss,
            _ => c.buf(8).cast(),
        }
    };
    c.arg(d);
    c.ret_pool("GObj_Create", 8);
    c.ret_pool("HSD_MemAlloc", 4096);
}

fn mk_state(c: &mut Case) -> *mut GameModeState {
    let s = c.zobj::<GameModeState>();
    unsafe { (*s).info.scene_kind = pick(c, &[0u8, 1, 8, 9]) };
    s
}

fn b_state_sss(c: &mut Case) {
    if chance(c, 3, 4) {
        pre(c, &[("Ui_Ready", 1), ("Ui_SelCanOpen", 1), ("Frontend_NativeSelect", 0), ("Ui_CssKnown", 1), ("Settings_Int", 0)]);
    }
    mk_scene(c);
    let s = if chance(c, 1, 8) { null_mut() } else { mk_state(c) };
    c.arg(s);
    let sss = pick(c, &[0i32, 0, 0, 1]);
    c.arg(sss);
    // the enter data a CSS state hands over (match_type read for the Atlas select)
    let css = c.zobj::<CSSData>();
    unsafe { (*css).match_type = pick(c, &[0u8, 1, 5, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x16, 0x17]) };
    let ptrs = [css as u64, 0, css as u64];
    c.ret_seq_u64("gm_GetGameModeStateEnterData", &ptrs);
}

fn b_state_name(c: &mut Case) {
    b_state_sss(c);
    let n = if chance(c, 1, 3) { null_mut() } else { cstr(c, 5) };
    c.arg(n);
}

fn b_state_only(c: &mut Case) {
    mk_scene(c);
    let s = if chance(c, 1, 8) { null_mut() } else { mk_state(c) };
    c.arg(s);
}

fn b_route(c: &mut Case) {
    if chance(c, 2, 3) {
        pre(c, &[("PcFrontendEnabled", 1), ("PcFrontendMenusEnabled", 1), ("fm_replaced", 1)]);
    }
    mk_scene(c);
    if chance(c, 1, 2) {
        unsafe {
            tu::fe_enabled_cache = -1;
            tu::fe_menus_on_cache = -1;
            tu::fm.native_req = 0;
        }
    }
    let from = pick(c, &[GM_MENU as u8, GM_VS as u8, tu::GM_FRONTEND, 7]);
    let to = pick(c, &[GM_MENU as u8, GM_VS as u8, tu::GM_FRONTEND, 7]);
    c.arg(from as i32);
    c.arg(to as i32);
}

fn b_aspect(c: &mut Case) {
    mk_scene(c);
    let f = [1.0f32, 4.0 / 3.0, 16.0 / 9.0, 2.3333333, 0.5, 0.0, -1.0, 100.0];
    let bits: Vec<u64> = (0..4).map(|_| f[c.rng.range(0, 7) as usize].to_bits() as u64).collect();
    c.ret_seq_u64("View_AspectBits", &bits);
}

fn b_rect(c: &mut Case) {
    mk_scene(c);
    for _ in 0..4 {
        let v = c.rng.f32_in(-50.0, 700.0);
        c.arg(v);
    }
    let a = rgba(c);
    let b = rgba(c);
    c.arg(a);
    c.arg(b);
}

fn b_solid(c: &mut Case) {
    mk_scene(c);
    for _ in 0..4 {
        let v = c.rng.f32_in(-50.0, 700.0);
        c.arg(v);
    }
    let a = rgba(c);
    c.arg(a);
}

fn b_arrow(c: &mut Case) {
    mk_scene(c);
    for _ in 0..3 {
        let v = c.rng.f32_in(-50.0, 700.0);
        c.arg(v);
    }
    let dir = pick(c, &[-1i32, 1, 0, -5]);
    c.arg(dir);
    let a = rgba(c);
    c.arg(a);
}

fn b_tex_load(c: &mut Case) {
    mk_scene(c);
    let t = c.zobj::<tu::FeTex>();
    c.arg(t);
    let n = cstr(c, 6);
    c.arg(n);
    c.ret_pool("HSD_MemAlloc", 700);
}

fn b_tex_quad_uv(c: &mut Case) {
    mk_scene(c);
    let t = c.zobj::<tu::FeTex>();
    unsafe {
        (*t).ok = pick(c, &[0, 1, 1]);
        (*t).lut = if chance(c, 1, 2) { null_mut() } else { c.buf(8).cast() };
    }
    c.arg(t);
    for _ in 0..8 {
        let v = c.rng.f32_in(-5.0, 700.0);
        c.arg(v);
    }
    let col = rgba(c);
    c.arg(col);
}

fn b_tex_quad(c: &mut Case) {
    mk_scene(c);
    let t = c.zobj::<tu::FeTex>();
    unsafe {
        (*t).ok = pick(c, &[0, 1, 1]);
        (*t).lut = if chance(c, 1, 2) { null_mut() } else { c.buf(8).cast() };
    }
    c.arg(t);
    for _ in 0..4 {
        let v = c.rng.f32_in(-5.0, 700.0);
        c.arg(v);
    }
    let col = rgba(c);
    c.arg(col);
}

fn b_tex_or_solid(c: &mut Case) {
    mk_scene(c);
    let which = pick(c, &[0i32, 6, 7, 8, 9, 10, 11]);
    c.arg(which);
    for _ in 0..4 {
        let v = c.rng.f32_in(-5.0, 700.0);
        c.arg(v);
    }
    let col = rgba(c);
    c.arg(col);
}

fn b_route2(c: &mut Case) {
    mk_scene(c);
    let k = c.rng.range(0, 9) as i32;
    c.arg(k);
    let s = c.rng.range(0, 9) as i32;
    c.arg(s);
}

fn b_route_pos(c: &mut Case) {
    mk_scene(c);
    let p = c.rng.range(0, 8) as i32;
    c.arg(p);
    let k = c.zbuf(8);
    c.arg(k);
    let s = c.zbuf(8);
    c.arg(s);
}

fn b_native_req(c: &mut Case) {
    mk_scene(c);
    let k = c.zbuf(8);
    c.arg(k);
    let s = c.zbuf(8);
    c.arg(s);
}

fn b_slot(c: &mut Case) {
    mk_scene(c);
    let s = c.rng.range(-2, 12) as i32;
    c.arg(s);
}

fn b_text(c: &mut Case) {
    mk_scene(c);
    for _ in 0..3 {
        let v = c.rng.f32_in(0.0, 640.0);
        c.arg(v);
    }
    let al = pick(c, &[0i32, 1, 2]);
    c.arg(al);
    let col = rgba(c);
    c.arg(col);
    let n = c.rng.range(0, 10) as usize;
    let s = cstr(c, n);
    c.arg(s);
    if chance(c, 1, 5) {
        c.ret_ptr("HSD_SisLib_803A6754", null_mut::<HSD_Text>());
    }
}

fn b_set_text(c: &mut Case) {
    mk_scene(c);
    let pt = c.zbuf(8) as *mut *mut HSD_Text;
    unsafe { *pt = if chance(c, 1, 6) { null_mut() } else { mk_text(c) } };
    c.arg(pt);
    let cur = { let n_ = c.rng.range(0, 9) as usize; cstr(c, n_) };
    let cap = pick(c, &[80i32, 512, 4, 2]);
    let n = c.rng.range(0, 9) as usize;
    let mut s = cstr(c, n);
    if chance(c, 1, 2) {
        // sometimes the very same text as `cur`
        unsafe {
            let mut i = 0;
            while *cur.add(i) != 0 {
                *s.add(i) = *cur.add(i);
                i += 1;
            }
            *s.add(i) = 0;
        }
    }
    if chance(c, 1, 8) {
        s = c.zbuf(8).cast();
    }
    c.arg(cur);
    c.arg(cap);
    c.arg(s);
    let col = rgba(c);
    c.arg(col);
    if chance(c, 1, 6) {
        c.ret_ptr("HSD_SisLib_803A6754", null_mut::<HSD_Text>());
    }
}

/// A synthetic FrontendItem with callbacks from the port's own set.
fn mk_item(c: &mut Case) -> *mut FrontendItem {
    let it = c.zobj::<FrontendItem>();
    unsafe {
        (*it).kind = pick(c, &[0u8, 1, 2, 3, 1, 2, 3]);
        (*it).action = pick(c, &[0u8, 1, 2]);
        (*it).label = cstr(c, 5);
        (*it).help = if chance(c, 1, 4) { core::ptr::null() } else { cstr(c, 6) };
        (*it).get = pick(c, &[None, Some(tu::fe_get_mode as unsafe extern "C" fn() -> i32), Some(tu::fe_get_time), Some(tu::fe_get_ratio), Some(tu::fe_get_items), Some(tu::fe_np_get_stocks), Some(tu::fe_get_pause)]);
        (*it).set = pick(c, &[None, Some(tu::fe_set_mode as unsafe extern "C" fn(i32)), Some(tu::fe_set_time), Some(tu::fe_set_items), Some(tu::fe_np_set_delay)]);
        (*it).min = pick(c, &[0, 1, -1, 5]);
        (*it).max = (*it).min + pick(c, &[0, 1, 3, 20, 99]);
        (*it).step = pick(c, &[0, 1, 5, -2]);
        let n = ((*it).max - (*it).min + 1).clamp(1, 10) as usize;
        let opts = c.zbuf(8 * 10) as *mut *const i8;
        for i in 0..n {
            *opts.add(i) = cstr(c, 4);
        }
        (*it).options = if chance(c, 1, 5) { core::ptr::null() } else { opts };
        (*it).format = pick(c, &[None, Some(tu::fe_fmt_time as unsafe extern "C" fn(i32, *mut i8)), Some(tu::fe_fmt_ratio), Some(tu::fe_np_fmt_minutes)]);
        (*it).visible = pick(c, &[None, Some(tu::fe_is_stock as unsafe extern "C" fn() -> i32)]);
        (*it).call = pick(c, &[None, Some(tu::fe_ol_join as unsafe extern "C" fn()), Some(tu::fe_ol_random)]);
        (*it).enabled = pick(c, &[None, Some(tu::fe_is_stock as unsafe extern "C" fn() -> i32), Some(tu::fe_get_pause)]);
    }
    it
}

fn b_value_string(c: &mut Case) {
    mk_scene(c);
    let it = mk_item(c);
    c.arg(it);
    let out = c.zbuf(FE_OUT);
    c.arg(out);
}
const FE_OUT: usize = 600;

fn b_change(c: &mut Case) {
    mk_scene(c);
    let it: *mut FrontendItem = if chance(c, 1, 2) { mk_item(c) } else { unsafe { addr_of_mut!(tu::fe_items_vs_setup[c.rng.range(0, 9) as usize]) } };
    c.arg(it);
    let dir = pick(c, &[-1i32, 1, 1, -1, 2]);
    c.arg(dir);
}

fn b_switch(c: &mut Case) {
    mk_scene(c);
    titled(c);
    let s = screens_titled(c);
    c.arg(s as *mut FrontendScreen);
}

fn b_int_arg(c: &mut Case) {
    mk_scene(c);
    let v = pick(c, &[-3i32, -1, 0, 1, 2, 5, 9, 10, 25, 99, 100]);
    c.arg(v);
}

fn b_fmt(c: &mut Case) {
    let v = pick(c, &[-3i32, -1, 0, 1, 2, 5, 9, 10, 25, 99, 100, 1234]);
    c.arg(v);
    let out = c.zbuf(64);
    c.arg(out);
}

fn b_settings_page(c: &mut Case) {
    mk_scene(c);
    let v = pick(c, &[-1i32, 0, 1, 2, 3, 4, 5, 6, 7]);
    c.arg(v);
}

fn b_native_return(c: &mut Case) {
    mk_scene(c);
    let k = pick(c, &[0i32, 2, 4, 5]);
    c.arg(k);
    let s = pick(c, &[0i32, 3, 4, 7]);
    c.arg(s);
}

fn b_notice(c: &mut Case) {
    mk_scene(c);
    let s = if chance(c, 1, 6) {
        core::ptr::null_mut()
    } else if chance(c, 1, 6) {
        // longer than the note buffer
        let p = c.zbuf(600);
        unsafe { core::ptr::write_bytes(p, b'n', 560) };
        p.cast()
    } else {
        { let n_ = c.rng.range(0, 20) as usize; cstr(c, n_) }
    };
    c.arg(s);
    let bad = pick(c, &[0i32, 1]);
    c.arg(bad);
}

fn b_picked(c: &mut Case) {
    mk_scene(c);
    let which = pick(c, &[0i32, 1, 2]);
    c.arg(which);
    let a = pick(c, &[-1i32, 0, 5, 33]);
    c.arg(a);
    let b = c.rng.range(0, 5) as i32;
    c.arg(b);
}

fn b_open(c: &mut Case) {
    mk_scene(c);
    let w = pick(c, &[0i32, 1, 2]);
    c.arg(w);
    let s = cstr(c, 6);
    c.arg(s);
}

// ---- icon capture ---------------------------------------------------------------------------------------

/// A patterned (mostly zero) source buffer of `n` bytes, with marks at the start, the middle and the end.
fn big_buf(c: &mut Case, n: usize) -> *mut u8 {
    let p = c.zbuf(n);
    unsafe {
        for i in [0, 1, 7, n / 3, n / 2, n - 9, n - 1] {
            *p.add(i) = c.rng.range(1, 255) as u8;
        }
    }
    p
}

fn mk_tobj_chain(c: &mut Case, big: *mut u8) -> *mut HSD_TObj {
    let mut head: *mut HSD_TObj = null_mut();
    for _ in 0..c.rng.range(0, 3) {
        let t = c.zobj::<HSD_TObj>();
        unsafe {
            if !chance(c, 1, 5) {
                let id = c.zobj::<HSD_ImageDesc>();
                (*id).image_ptr = if chance(c, 1, 6) { null_mut() } else { big.cast() };
                (*id).width = pick(c, &[4u16, 8, 16, 32, 64, 128]);
                (*id).height = pick(c, &[4u16, 8, 16, 32, 64, 128]);
                (*id).format = pick(c, &[0u32, 1, 3, 4, 5, 9, 14]) as _;
                (*t).imagedesc = id;
            }
            (*t).tlut_no = pick(c, &[-1i32, -1, 0, 1, 3]);
            (*t).tlut = if chance(c, 1, 5) { null_mut() } else { mk_tlut(c) };
            if chance(c, 1, 2) {
                let tbl = c.zbuf(8 * 4) as *mut *mut HSD_Tlut;
                for i in 0..4 {
                    *tbl.add(i) = if chance(c, 1, 6) { null_mut() } else { mk_tlut(c) };
                }
                (*t).tluttbl = tbl;
            }
            (*t).next = head;
        }
        head = t;
    }
    head
}

fn mk_tlut(c: &mut Case) -> *mut HSD_Tlut {
    let t = c.zobj::<HSD_Tlut>();
    unsafe {
        (*t).lut = if chance(c, 1, 6) { null_mut() } else { big_buf(c, 1100).cast() };
        (*t).fmt = c.rng.range(0, 2) as _;
        (*t).n_entries = pick(c, &[0u16, 16, 255, 256, 257, 300]);
    }
    t
}

fn mk_dobj_chain(c: &mut Case, big: *mut u8) -> *mut HSD_DObj {
    let mut head: *mut HSD_DObj = null_mut();
    for _ in 0..c.rng.range(0, 2) {
        let d = c.zobj::<HSD_DObj>();
        unsafe {
            if !chance(c, 1, 4) {
                let m = c.zobj::<HSD_MObj>();
                (*m).tobj = mk_tobj_chain(c, big);
                (*d).mobj = m;
            }
            (*d).next = head;
        }
        head = d;
    }
    head
}

/// A joint tree (`child` / `next` links) whose `HSD_JObjGetDObj` answers come from a sequence.
fn mk_icon_tree(c: &mut Case) -> *mut HSD_JObj {
    let big = big_buf(c, 70002);
    let root = c.jobj();
    unsafe {
        if chance(c, 1, 2) {
            (*root).next = c.jobj();
        }
        let mut cur = root;
        for _ in 0..c.rng.range(0, 3) {
            let ch = c.jobj();
            (*cur).child = ch;
            if chance(c, 1, 2) {
                let sib = c.jobj();
                (*ch).next = sib;
            }
            cur = ch;
        }
    }
    let seq: Vec<u64> = (0..10).map(|_| mk_dobj_chain(c, big) as usize as u64).collect();
    c.ret_seq_u64("HSD_JObjGetDObj", &seq);
    let sizes = [0u64, 100, 4096, 4097, 20000, 65536, 65537, 70000];
    let pickv: Vec<u64> = (0..6).map(|_| sizes[c.rng.range(0, 7) as usize]).collect();
    c.ret_seq_u64("GXGetTexBufferSize", &pickv);
    root
}

fn b_find_tex(c: &mut Case) {
    mk_scene(c);
    let root = mk_icon_tree(c);
    c.arg(root);
    let best = c.zbuf(8);
    c.arg(best);
    let area = c.zbuf(8);
    unsafe { *(area as *mut i32) = pick(c, &[0i32, 0, 100, 5000]) };
    c.arg(area);
    let depth = pick(c, &[0i32, 0, 1, 11, 12, 13]);
    c.arg(depth);
}

fn b_store(c: &mut Case) {
    mk_scene(c);
    let _ = mk_icon_tree(c);
    let big = big_buf(c, 70002);
    let which = pick(c, &[-1i32, 0, 1, 5, 127, 128, 200]);
    c.arg(which);
    let id = c.zobj::<HSD_ImageDesc>();
    unsafe {
        (*id).image_ptr = if chance(c, 1, 8) { null_mut() } else { big.cast() };
        (*id).width = pick(c, &[8u16, 64]);
        (*id).height = pick(c, &[8u16, 64]);
        (*id).format = pick(c, &[0u32, 4, 5, 9]) as _;
    }
    let im = if chance(c, 1, 8) { null_mut() } else { id };
    c.arg(im);
    let tl = if chance(c, 1, 5) { null_mut() } else { mk_tlut(c) };
    c.arg(tl);
}

fn b_capture(c: &mut Case) {
    mk_scene(c);
    let root = if chance(c, 1, 8) { null_mut() } else { mk_icon_tree(c) };
    let k = pick(c, &[-1i32, 0, 1, 2, 77, 127, 128]);
    c.arg(k);
    c.arg(root);
}

// ---- the router / misc ----------------------------------------------------------------------------------

fn b_noargs(c: &mut Case) {
    mk_scene(c);
}

fn b_load_step(c: &mut Case) {
    if chance(c, 2, 3) {
        pre(c, &[("Gfx_SeedCorePending", 0), ("Gfx_PipelinesUrgent", 0), ("Gfx_LoadScreenEnabled", 1)]);
    }
    mk_scene(c);
    unsafe {
        let fe = &mut *addr_of_mut!(tu::fe);
        if chance(c, 2, 3) {
            fe.load_frames = pick(c, &[43, 44, 45, 478, 479]);
            fe.load_settled = pick(c, &[18, 19, 20, 0]);
        }
    }
}

fn b_frame_main(c: &mut Case) {
    if chance(c, 3, 4) {
        pre(c, &[("fss_frame", 0), ("fcr_frame", 0), ("fe_is_settings", 0), ("fms_in_quad", 1)]);
        mk_scene_main(c);
        let m = pick(c, &[0u64, 0, 0x10, 0x20, 0x1, 0x2, 0x4, 0x8]);
        let _ = m;
    } else {
        mk_scene_n(c);
    }
}

fn b_two_ptr_f(c: &mut Case) {
    mk_scene(c);
}

#[test]
fn gmfrontend_vs_c() {
    let funcs = [
        fd!("fe_get_mode", "", "i32", 6, b_noargs, |a| { r(tu::fe_get_mode()) }),
        fd!("fe_set_mode", "i", "v", 6, b_int_arg, |a| { tu::fe_set_mode(ai32(a, 0)); 0 }),
        fd!("fe_is_stock", "", "i32", 6, b_noargs, |a| { r(tu::fe_is_stock()) }),
        fd!("fe_get_stocks", "", "i32", 4, b_noargs, |a| { r(tu::fe_get_stocks()) }),
        fd!("fe_set_stocks", "i", "v", 6, b_int_arg, |a| { tu::fe_set_stocks(ai32(a, 0)); 0 }),
        fd!("fe_get_time", "", "i32", 10, b_noargs, |a| { r(tu::fe_get_time()) }),
        fd!("fe_set_time", "i", "v", 10, b_int_arg, |a| { tu::fe_set_time(ai32(a, 0)); 0 }),
        fd!("fe_fmt_time", "ip", "v", 12, b_fmt, |a| { tu::fe_fmt_time(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fe_get_handicap", "", "i32", 4, b_noargs, |a| { r(tu::fe_get_handicap()) }),
        fd!("fe_set_handicap", "i", "v", 6, b_int_arg, |a| { tu::fe_set_handicap(ai32(a, 0)); 0 }),
        fd!("fe_get_ratio", "", "i32", 4, b_noargs, |a| { r(tu::fe_get_ratio()) }),
        fd!("fe_set_ratio", "i", "v", 6, b_int_arg, |a| { tu::fe_set_ratio(ai32(a, 0)); 0 }),
        fd!("fe_fmt_ratio", "ip", "v", 12, b_fmt, |a| { tu::fe_fmt_ratio(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fe_get_stage_sel", "", "i32", 4, b_noargs, |a| { r(tu::fe_get_stage_sel()) }),
        fd!("fe_set_stage_sel", "i", "v", 6, b_int_arg, |a| { tu::fe_set_stage_sel(ai32(a, 0)); 0 }),
        fd!("fe_get_items", "", "i32", 6, b_noargs, |a| { r(tu::fe_get_items()) }),
        fd!("fe_set_items", "i", "v", 8, b_int_arg, |a| { tu::fe_set_items(ai32(a, 0)); 0 }),
        fd!("fe_get_ff", "", "i32", 6, b_noargs, |a| { r(tu::fe_get_ff()) }),
        fd!("fe_set_ff", "i", "v", 8, b_int_arg, |a| { tu::fe_set_ff(ai32(a, 0)); 0 }),
        fd!("fe_get_pause", "", "i32", 6, b_noargs, |a| { r(tu::fe_get_pause()) }),
        fd!("fe_set_pause", "i", "v", 6, b_int_arg, |a| { tu::fe_set_pause(ai32(a, 0)); 0 }),
        fd!("fe_np_get_stocks", "", "i32", 4, b_noargs, |a| { r(tu::fe_np_get_stocks()) }),
        fd!("fe_np_set_stocks", "i", "v", 4, b_int_arg, |a| { tu::fe_np_set_stocks(ai32(a, 0)); 0 }),
        fd!("fe_np_get_minutes", "", "i32", 4, b_noargs, |a| { r(tu::fe_np_get_minutes()) }),
        fd!("fe_np_set_minutes", "i", "v", 4, b_int_arg, |a| { tu::fe_np_set_minutes(ai32(a, 0)); 0 }),
        fd!("fe_np_fmt_minutes", "ip", "v", 6, b_fmt, |a| { tu::fe_np_fmt_minutes(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fe_np_get_delay", "", "i32", 4, b_noargs, |a| { r(tu::fe_np_get_delay()) }),
        fd!("fe_np_set_delay", "i", "v", 4, b_int_arg, |a| { tu::fe_np_set_delay(ai32(a, 0)); 0 }),
        fd!("fe_np_fmt_delay", "ip", "v", 6, b_fmt, |a| { tu::fe_np_fmt_delay(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fe_np_zero", "", "i32", 2, b_noargs, |a| { r(melee_modes::gm::fe_np_zero()) }),
        fd!("Frontend_OnlinePick", "", "i32", 4, b_noargs, |a| { r(tu::Frontend_OnlinePick()) }),
        fd!("fe_np_find_tex", "pppi", "v", 80, b_find_tex, |a| { tu::fe_np_find_tex(p(a, 0), p(a, 1), p(a, 2), ai32(a, 3)); 0 }),
        fd!("fe_np_store", "ipp", "v", 100, b_store, |a| { tu::fe_np_store(ai32(a, 0), p(a, 1), p(a, 2)); 0 }),
        fd!("Frontend_CaptureCharIcon", "ip", "v", 80, b_capture, |a| { tu::Frontend_CaptureCharIcon(ai32(a, 0), p(a, 1)); 0 }),
        fd!("Frontend_CaptureIcon", "ip", "v", 60, b_capture, |a| { tu::Frontend_CaptureIcon(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fe_np_pick_char", "", "v", 8, b_scene, |a| { tu::fe_np_pick_char(); 0 }),
        fd!("fe_ol_notice", "pi", "v", 20, b_notice, |a| { tu::fe_ol_notice(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fe_ol_host", "", "v", 24, b_scene, |a| { tu::fe_ol_host(); 0 }),
        fd!("fe_np_get_stage_mode", "", "i32", 4, b_scene, |a| { r(tu::fe_np_get_stage_mode()) }),
        fd!("fe_np_set_stage_mode", "i", "v", 4, b_int_arg, |a| { tu::fe_np_set_stage_mode(ai32(a, 0)); 0 }),
        fd!("fe_np_get_turbo", "", "i32", 4, b_scene, |a| { r(tu::fe_np_get_turbo()) }),
        fd!("fe_np_set_turbo", "i", "v", 6, b_int_arg, |a| { tu::fe_np_set_turbo(ai32(a, 0)); 0 }),
        fd!("fe_np_get_envoy", "", "i32", 4, b_scene, |a| { r(tu::fe_np_get_envoy()) }),
        fd!("fe_np_set_envoy", "i", "v", 12, b_int_arg, |a| { tu::fe_np_set_envoy(ai32(a, 0)); 0 }),
        fd!("fe_ol_random", "", "v", 24, b_scene, |a| { tu::fe_ol_random(); 0 }),
        fd!("fe_ol_join", "", "v", 24, b_scene, |a| { tu::fe_ol_join(); 0 }),
        fd!("Frontend_ScreenTitle", "", "s", 12, b_scene_n, |a| { cstr_hash(tu::Frontend_ScreenTitle()) }),
        fd!("Frontend_ScreenSubtitle", "", "s", 12, b_scene_n, |a| { cstr_hash(tu::Frontend_ScreenSubtitle()) }),
        fd!("Frontend_Cursor", "", "i32", 12, b_scene_n, |a| { r(tu::Frontend_Cursor()) }),
        fd!("Frontend_CursorLabel", "", "s", 40, b_scene_n, |a| { cstr_hash(tu::Frontend_CursorLabel()) }),
        fd!("Frontend_OnlinePicked", "iii", "v", 40, b_picked, |a| { tu::Frontend_OnlinePicked(ai32(a, 0), ai32(a, 1), ai32(a, 2)); 0 }),
        fd!("Frontend_BackToOnline", "", "v", 6, b_scene, |a| { tu::Frontend_BackToOnline(); 0 }),
        fd!("fe_np_open", "ip", "v", 8, b_open, |a| { tu::fe_np_open(ai32(a, 0), p(a, 1)); 0 }),
        fd!("gmFrontend_SelectScene", "pi", "v", 24, b_state_sss, |a| { tu::gmFrontend_SelectScene(p(a, 0), ai32(a, 1)); 0 }),
        fd!("gmFrontend_TrainingSelect", "pi", "v", 24, b_state_sss, |a| { tu::gmFrontend_TrainingSelect(p(a, 0), ai32(a, 1)); 0 }),
        fd!("gmFrontend_ModeSelect", "pip", "v", 12, b_state_name, |a| { tu::gmFrontend_ModeSelect(p(a, 0), ai32(a, 1), p(a, 2)); 0 }),
        fd!("gmFrontend_AtlasSelect", "pip", "v", 80, b_state_name, |a| { tu::gmFrontend_AtlasSelect(p(a, 0), ai32(a, 1), p(a, 2)); 0 }),
        fd!("gmFrontend_BeginLoading", "", "v", 4, b_scene, |a| { tu::gmFrontend_BeginLoading(); 0 }),
        fd!("gmFrontend_TakeWarmed", "", "b", 6, b_scene, |a| { r(tu::gmFrontend_TakeWarmed() != 0) }),
        fd!("fe_enabled", "", "i32", 8, b_scene, |a| { r(tu::fe_enabled()) }),
        fd!("fe_menus_on", "", "i32", 12, b_scene, |a| { r(tu::fe_menus_on()) }),
        fd!("fe_rules_enabled", "", "i32", 6, b_scene, |a| { r(tu::fe_rules_enabled()) }),
        fd!("gmFrontend_Route", "ii", "i8", 120, b_route, |a| { r(tu::gmFrontend_Route(ai32(a, 0) as u8, ai32(a, 1) as u8)) }),
        fd!("gmFrontend_ReportedMode", "", "i8", 4, b_scene, |a| { r(tu::gmFrontend_ReportedMode()) }),
        fd!("fe_canvas_width", "", "f", 8, b_aspect, |a| { r(tu::fe_canvas_width()) }),
        fd!("fe_canvas_left", "", "f", 8, b_aspect, |a| { r(tu::fe_canvas_left()) }),
        fd!("fe_canvas_extra", "", "f", 8, b_aspect, |a| { r(tu::fe_canvas_extra()) }),
        fd!("fe_rect", "ffffcc", "v", 12, b_rect, |a| {
            tu::fe_rect(af32(a, 0), af32(a, 1), af32(a, 2), af32(a, 3), <GXColor as OVal>::from_bits(a[4]), <GXColor as OVal>::from_bits(a[5]));
            0
        }),
        fd!("fe_solid", "ffffc", "v", 12, b_solid, |a| {
            tu::fe_solid(af32(a, 0), af32(a, 1), af32(a, 2), af32(a, 3), <GXColor as OVal>::from_bits(a[4]));
            0
        }),
        fd!("fe_arrow", "fffic", "v", 24, b_arrow, |a| {
            tu::fe_arrow(af32(a, 0), af32(a, 1), af32(a, 2), ai32(a, 3), <GXColor as OVal>::from_bits(a[4]));
            0
        }),
        fd!("fe_tex_load", "pp", "v", 40, b_tex_load, |a| { tu::fe_tex_load(p(a, 0), p(a, 1)); 0 }),
        fd!("fe_tex_quad_uv", "pffffffffc", "v", 24, b_tex_quad_uv, |a| {
            tu::fe_tex_quad_uv(p(a, 0), af32(a, 1), af32(a, 2), af32(a, 3), af32(a, 4), af32(a, 5), af32(a, 6), af32(a, 7), af32(a, 8), <GXColor as OVal>::from_bits(a[9]));
            0
        }),
        fd!("fe_tex_quad", "pffffc", "v", 12, b_tex_quad, |a| {
            tu::fe_tex_quad(p(a, 0), af32(a, 1), af32(a, 2), af32(a, 3), af32(a, 4), <GXColor as OVal>::from_bits(a[5]));
            0
        }),
        fd!("fe_tex_or_solid", "iffffc", "v", 30, b_tex_or_solid, |a| {
            tu::fe_tex_or_solid(ai32(a, 0), af32(a, 1), af32(a, 2), af32(a, 3), af32(a, 4), <GXColor as OVal>::from_bits(a[5]));
            0
        }),
        fd!("fe_kit_available", "", "b", 12, b_scene, |a| { r(tu::fe_kit_available() != 0) }),
        fd!("fm_route_to_menus", "ii", "v", 8, b_route2, |a| { tu::fm_route_to_menus(ai32(a, 0) as u8, ai32(a, 1) as u8); 0 }),
        fd!("fm_route_native", "", "b", 6, b_scene, |a| { r(tu::fm_route_native() != 0) }),
        fd!("fm_route_position", "ipp", "b", 24, b_route_pos, |a| { r(tu::fm_route_position(ai32(a, 0) as u8, p(a, 1), p(a, 2)) != 0) }),
        fd!("fe_match_setup_from_menus", "", "v", 4, b_scene, |a| { tu::fe_match_setup_from_menus(); 0 }),
        fd!("gmFrontend_NativeRequest", "pp", "b", 12, b_native_req, |a| { r(tu::gmFrontend_NativeRequest(p(a, 0), p(a, 1)) != 0) }),
        fd!("gmFrontend_TakeNativeRequest", "pp", "b", 12, b_native_req, |a| { r(tu::gmFrontend_TakeNativeRequest(p(a, 0), p(a, 1)) != 0) }),
        fd!("gmFrontend_NativeReturn", "ii", "b", 60, b_native_return, |a| { r(tu::gmFrontend_NativeReturn(ai32(a, 0), ai32(a, 1)) != 0) }),
        fd!("fe_state_enter", "p", "v", 12, b_state_only, |a| { tu::fe_state_enter(p(a, 0)); 0 }),
        fd!("fe_row_offset", "i", "f", 24, b_slot, |a| { r(tu::fe_row_offset(ai32(a, 0))) }),
        fd!("fe_rows_shown", "", "i32", 12, b_scene, |a| { r(tu::fe_rows_shown()) }),
        fd!("fe_draw_frame", "", "v", 6, b_aspect, |a| { tu::fe_draw_frame(); 0 }),
        fd!("fe_draw_panels", "pi", "v", 400, gobj_pass, |a| { tu::fe_draw_panels(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fe_draw_fade", "pi", "v", 80, b_fade, |a| { tu::fe_draw_fade(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fe_text", "ffficp", "p", 40, b_text, |a| {
            r(tu::fe_text(af32(a, 0), af32(a, 1), af32(a, 2), ai32(a, 3) as u8, <GXColor as OVal>::from_bits(a[4]), p(a, 5)))
        }),
        fd!("fe_set_text", "ppipc", "v", 120, b_set_text, |a| {
            tu::fe_set_text(p(a, 0), p(a, 1), ai32(a, 2), p(a, 3), <GXColor as OVal>::from_bits(a[4]));
            0
        }),
        fd!("fe_value_string", "pp", "v", 80, b_value_string, |a| { tu::fe_value_string(p(a, 0), p(a, 1)); 0 }),
        fd!("fe_refresh_rows", "", "v", 120, b_scene, |a| { tu::fe_refresh_rows(); 0 }),
        fd!("fe_rebuild_visible", "", "v", 80, b_scene, |a| { tu::fe_rebuild_visible(); 0 }),
        fd!("gm_Scene_Frontend_OnEnter", "p", "v", 200, b_enter, |a| { tu::gm_Scene_Frontend_OnEnter(p(a, 0)); 0 }),
        fd!("fe_switch_screen", "p", "v", 120, b_switch, |a| { tu::fe_switch_screen(p(a, 0)); 0 }),
        fd!("fe_online_from_menus", "", "v", 4, b_scene, |a| { tu::fe_online_from_menus(); 0 }),
        fd!("fe_settings_from_menus", "i", "v", 40, b_settings_page, |a| { tu::fe_settings_from_menus(ai32(a, 0)); 0 }),
        fd!("fe_settings_resume_game", "", "v", 4, b_scene, |a| { tu::fe_settings_resume_game(); 0 }),
        fd!("fe_change", "pi", "v", 120, b_change, |a| { tu::fe_change(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fe_load_begin", "", "v", 6, b_scene, |a| { tu::fe_load_begin(); 0 }),
        fd!("fe_load_step", "", "b", 500, b_load_step, |a| { r(tu::fe_load_step() != 0) }),
        fd!("fe_loading_frame", "", "v", 60, b_scene, |a| { tu::fe_loading_frame(); 0 }),
        fd!("fe_mouse_row", "", "i32", 40, b_scene, |a| { r(tu::fe_mouse_row()) }),
        fd!("gm_Scene_Frontend_OnFrame", "", "v", 800, b_frame_main, |a| { tu::gm_Scene_Frontend_OnFrame(); 0 }),
        fd!("gm_Scene_Frontend_OnExit", "p", "v", 24, b_scene_n, |a| { tu::gm_Scene_Frontend_OnExit(null_mut()); 0 }),
    ];
    let _ = (offset_of!(HSD_Text, pos_x), addr_of!(tu::fe_enter_data));
    let globals = tu::oracle_globals();
    let mut fn_table: Vec<(&str, usize)> = Vec::new();
    macro_rules! reg {
        ($($n:ident),*) => { $( fn_table.push((stringify!($n), tu::$n as usize)); )* };
    }
    reg!(
        fe_get_mode, fe_set_mode, fe_is_stock, fe_get_stocks, fe_set_stocks, fe_get_time, fe_set_time, fe_fmt_time, fe_get_handicap,
        fe_set_handicap, fe_get_ratio, fe_set_ratio, fe_fmt_ratio, fe_get_stage_sel, fe_set_stage_sel, fe_get_items, fe_set_items, fe_get_ff,
        fe_set_ff, fe_get_pause, fe_set_pause, fe_np_get_stocks, fe_np_set_stocks, fe_np_get_minutes, fe_np_set_minutes, fe_np_fmt_minutes,
        fe_np_get_delay, fe_np_set_delay, fe_np_fmt_delay, fe_ol_host, fe_ol_join, fe_ol_random, fe_np_get_stage_mode, fe_np_set_stage_mode,
        fe_np_get_turbo, fe_np_set_turbo, fe_np_get_envoy, fe_np_set_envoy, fe_state_enter, fe_draw_panels, fe_draw_fade
    );
    run_tu(&TuOracle { tu: "gmfrontend", funcs: &funcs, globals, fn_table });
}

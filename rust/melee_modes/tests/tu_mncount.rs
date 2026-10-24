//! State oracle for tu/mncount.rs (src/oracle.rs, gen/tu/run.py; patch gen/tu/patches/mncount.py).
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::mncount as tu;
use melee_modes::tu::mncount::ext;
use melee_types::*;
#[path = "mn_common/mod.rs"]
mod mn;
use mn::*;

type GetVal = Option<unsafe extern "C" fn(u32) -> u32>;

unsafe fn flow(c: &mut Case) {
    tu::mn_804A04F0.cur_menu = pick(c, &[MENU_KIND_RECORDS_MISC as u8, MENU_KIND_RECORDS_MISC as u8, 3, 0x20]);
    tu::mn_804A04F0.prev_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.entering_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.buttons = c.rng.next();
    tu::mn_804D6BC8.cooldown = pick(c, &[0, 0, 0, 1, 5]);
    tu::mn_804D6BC8.x2 = c.rng.range(0, 99) as u16;
    tu::mn_804D6BC8.x4 = c.rng.range(0, 99) as i32;
    tu::HSD_GObj_804D783C = c.rng.range(0, 7) as i32;
    tu::HSD_GObj_JObjKind = c.rng.range(0, 255) as u8;
    tu::mn_804D4B64 = GXColor { r: c.rng.range(0, 255) as u8, g: c.rng.range(0, 255) as u8, b: c.rng.range(0, 255) as u8, a: 255 };
}

/// 25 persistent fighter records, handed out one per call (the callers walk 0..24 in order).
/// `kind` biases the statistics so that ties and an all-zero history both occur.
fn fighters(c: &mut Case) {
    let zero_all = c.rng.chance(1, 8);
    let few = c.rng.chance(1, 2);
    let mut ps = std::vec::Vec::new();
    for _ in 0..25 {
        let f = c.obj::<FighterData>();
        unsafe {
            let m = if few { 3 } else { 9 };
            let mut v = || if zero_all { 0 } else { c.rng.range(0, m) as u32 };
            (*f).stats.play_time = v();
            (*f).stats.victories = v() as u16;
            (*f).stats.losses = v() as u16;
            (*f).stats.damage_dealt = v() as i32;
            (*f).stats.damage_taken = v() as i32;
            (*f).stats.sd_count = v() as u16;
        }
        ps.push(f);
    }
    c.ret_seq("GetPersistentFighterData", &ps);
    c.ret_int("mnDiagram_GetFighterTotalKOs", 0, if few { 2 } else { 6 });
    c.ret_int("mnDiagram_GetFighterTotalFalls", 0, if few { 2 } else { 6 });
    let un: std::vec::Vec<u64> = (0..64).map(|_| if c.rng.chance(3, 4) { 1 } else { 0 }).collect();
    c.ret_seq_u64("gm_IsCKindUnlocked", &un);
    let st: std::vec::Vec<u64> = (0..29).map(|_| if c.rng.chance(1, 2) { 1 } else { 0 }).collect();
    c.ret_seq_u64("gm_80164430", &st);
    c.ret_int("gm_SelKindToCKind", 0, 24);
    c.ret_int("gm_801641CC", 0, 100);
}

fn counters(c: &mut Case) {
    for n in [
        "gmMainLib_GetBonusMatchTotal", "gmMainLib_GetCoinMatchTotal", "gmMainLib_GetCombinedVSPlayTime", "gmMainLib_GetKOTotal",
        "gmMainLib_GetMatchResetCounter", "gmMainLib_GetPowerCount", "gmMainLib_GetSelfDestructTotal",
        "gmMainLib_GetSingleplayerTime", "gmMainLib_GetStockMatchTotal", "gmMainLib_GetTimeMatchTotal",
        "gmMainLib_GetTotalDamage", "gmMainLib_GetVsPlayContestants", "gmMainLib_GetVsPlayTime", "gm_GetPowerTime",
    ] {
        let p = c.zbuf(8).cast::<u32>();
        let big = c.rng.chance(1, 3);
        unsafe {
            *p = match c.rng.range(0, 3) {
                0 => c.rng.u32() % 100000,
                1 => 3_599_999_940u32.wrapping_add(c.rng.range(-2, 100) as u32),
                2 => 999_999_999u32.wrapping_add(c.rng.range(-2, 100) as u32),
                _ => if big { c.rng.u32() } else { c.rng.range(0, 10) as u32 },
            };
        }
        c.ret_ptr(n, p);
    }
    c.ret_int("gm_GetPlayTime", -5, 4_000_000_000i64.min(i32::MAX as i64));
    c.ret_int("gm_GetVsPlayMatchTotal", 0, 4_000_000_000);
    c.ret_int("Toy_GetTrophyTotal", 0, 300);
    c.ret_int("GetNameCount", 0, 120);
}

fn texts(c: &mut Case) {
    c.ret_pool("HSD_SisLib_803A5ACC", core::mem::size_of::<HSD_Text>());
    c.ret_pool("HSD_SisLib_803A6754", core::mem::size_of::<HSD_Text>());
}

fn rets(c: &mut Case) {
    texts(c);
    fighters(c);
    counters(c);
    c.ret_int("mn_80229624", 0, 0xFFF);
    c.ret_flt("mn_8022ED6C", 0.0, 10.0);
    lb_children(c, 4);
    let ps = procs(c, 3);
    unsafe { tu::HSD_GObj_CurrentInvokedProc = ps[0] };
}

/// MnCountData with random scroll position and texts in some slots.
unsafe fn userdata(c: &mut Case) -> *mut MnCountData {
    let u = c.zobj::<MnCountData>();
    (*u).scroll_pos = pick(c, &[0, 0, 1, 5, 19, 20]);
    (*u).expand_anim_duration = pick(c, &[0, 0, 1, 10, -1]);
    for i in 0..10 {
        (*u).labels[i] = if c.rng.chance(1, 2) { text(c) } else { null_mut() };
        (*u).values[i] = if c.rng.chance(1, 2) { text(c) } else { null_mut() };
    }
    (*u).title = if c.rng.chance(2, 3) { text(c) } else { null_mut() };
    u
}

unsafe fn gobj_ud(c: &mut Case) -> *mut HSD_GObj {
    let u = userdata(c);
    let j = jobj(c);
    gobj(c, u.cast(), j.cast())
}

fn b_gobj(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { gobj_ud(c) };
    c.arg(g);
}

fn b_gobj_menu(c: &mut Case) {
    // the input proc reads the file's menu gobj
    unsafe {
        flow(c);
        tu::menu_gobj = gobj_ud(c);
    }
    rets(c);
    let btn = pick(c, &[0u64, 0x20, 0x1, 0x2, 0x3, 0x21]);
    c.ret_seq_u64("mn_80229624", &[btn]);
    let g = c.zobj::<HSD_GObj>();
    c.arg(g);
}

fn b_ud_gobj(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { gobj_ud(c) };
    let u = unsafe { (*g).user_data };
    c.arg(u);
    c.arg(g);
}

fn b_ud_gobj_i(c: &mut Case) {
    b_ud_gobj(c);
    let i = pick(c, &[0, 0, 3, 9]);
    c.arg(i);
}

fn b_ud(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let u = unsafe { userdata(c) };
    c.arg(u);
}

fn b_create(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mn_804D6BB8 = c.buf(16).cast();
        let u = userdata(c);
        c.ret_ptr("HSD_MemAlloc", u);
        let j = jobj(c);
        let g = gobj(c, u.cast(), j.cast());
        c.ret_ptr("GObj_Create", g);
    }
    rets(c);
}

fn b_i(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let v = c.rng.range(0, 35) as i32;
    c.arg(v);
}

fn b_sel(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let v = c.rng.range(0, 24) as u32;
    c.arg(v);
}

fn cbs() -> [GetVal; 7] {
    [
        Some(tu::mnCount_GetMatchTime),
        Some(tu::mnCount_GetKOKingpin),
        Some(tu::mnCount_GetNoDefenseNelly),
        Some(tu::mnCount_GetDisasterMaster),
        Some(tu::mnCount_GetSmashChamp),
        Some(tu::mnCount_GetSmashSap),
        Some(tu::mnCount_GetSlugMeister),
    ]
}

fn b_rank(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let skip = pick(c, &[0, 0, 1, 2, 5, 24, 30]);
    c.arg(skip);
    let f = pick(c, &cbs());
    c.arg(f);
    let mode = c.rng.range(0, 1) as i32;
    c.arg(mode);
}

fn b_rank2(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let f = pick(c, &cbs());
    let _ = f;
    let skip = pick(c, &[0, 0, 1, 2, 5, 24, 30]);
    c.arg(skip);
    let f = pick(c, &cbs());
    c.arg(f);
}

fn b_ties(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let e = c.zbuf(core::mem::size_of::<CountEntry>() * 25) as *mut CountEntry;
    unsafe {
        for i in 0..25 {
            (*e.add(i)).selkind = i as u8;
            (*e.add(i)).stat_value = c.rng.range(0, 2) as u32;
        }
    }
    c.arg(e);
    let s = pick(c, &[0, 0, 1, 3, 10, 23]);
    c.arg(s);
    let m = c.rng.range(0, 1) as i32;
    c.arg(m);
}

#[test]
fn mncount_vs_c() {
    let funcs = [
        fd!("inline_is_row_time", "i", "i32", 36, b_i, |a| { r(tu::inline_is_row_time(ai32(a, 0)) as i32) }),
        fd!("inline_is_row_char", "i", "i32", 36, b_i, |a| { r(tu::inline_is_row_char(ai32(a, 0)) as i32) }),
        fd!("inline_update_entries", "p", "v", 30, b_gobj, |a| { tu::inline_update_entries(p(a, 0)); 0 }),
        fd!("mnCount_8025035C_inline", "", "i32", 30, b_i, |a| { r(tu::mnCount_8025035C_inline() as i32) }),
        fd!("mnCount_GetMatchTime", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetMatchTime(au32(a, 0))) }),
        fd!("mnCount_GetKOKingpin", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetKOKingpin(au32(a, 0))) }),
        fd!("mnCount_GetNoDefenseNelly", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetNoDefenseNelly(au32(a, 0))) }),
        fd!("mnCount_GetDisasterMaster", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetDisasterMaster(au32(a, 0))) }),
        fd!("mnCount_8025035C", "ip", "i32", 400, b_rank2, |a| { r(tu::mnCount_8025035C(ai32(a, 0), GetVal::from_bits(a[1]))) }),
        fd!("mnCount_GetSmashChamp", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetSmashChamp(au32(a, 0))) }),
        fd!("mnCount_GetSmashSap", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetSmashSap(au32(a, 0))) }),
        fd!("mnCount_GetSlugMeister", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetSlugMeister(au32(a, 0))) }),
        fd!("mnCount_GetPunchingBag", "i", "i32", 30, b_sel, |a| { r(tu::mnCount_GetPunchingBag(au32(a, 0))) }),
        fd!("mnCount_8025072C", "pii", "i32", 300, b_ties, |a| { r(tu::mnCount_8025072C(p(a, 0), ai32(a, 1), ai32(a, 2))) }),
        fd!("mnCount_8025092C_inline", "i", "i32", 30, b_i, |a| { r(tu::mnCount_8025092C_inline(ai32(a, 0)) as i32) }),
        fd!("mnCount_8025092C", "ipi", "i32", 400, b_rank, |a| { r(tu::mnCount_8025092C(ai32(a, 0), GetVal::from_bits(a[1]), ai32(a, 2))) }),
        fd!("mnCount_CountUnlockedChars", "", "i32", 30, b_i, |a| { r(tu::mnCount_CountUnlockedChars()) }),
        fd!("mnCount_CountUnlockedMaps", "", "i32", 30, b_i, |a| { r(tu::mnCount_CountUnlockedMaps()) }),
        fd!("mnCount_GetRowValue_Character", "i", "i32", 300, b_i, |a| { r(tu::mnCount_GetRowValue_Character(au32(a, 0))) }),
        fd!("mnCount_GetCombinedVSPlayTimeValue", "", "i32", 20, b_i, |a| { r(tu::mnCount_GetCombinedVSPlayTimeValue()) }),
        fd!("mnCount_GetRowValue_Number", "i", "i32", 400, b_i, |a| { r(tu::mnCount_GetRowValue_Number(ai32(a, 0))) }),
        fd!("mnCount_CreateRow", "pii", "v", 600, b_ud_gobj_row, |a| { tu::mnCount_CreateRow(p(a, 0), ai32(a, 1), au32(a, 2)); 0 }),
        fd!("mnCount_HandleUserInput", "p", "v", 300, b_gobj_menu, |a| { tu::mnCount_HandleUserInput(p(a, 0)); 0 }),
        fd!("mnCount_UpdateArrowIndicators", "p", "v", 40, b_gobj, |a| { tu::mnCount_UpdateArrowIndicators(p(a, 0)); 0 }),
        fd!("mnCount_UpdateArrowIndicators_noinline", "p", "v", 20, b_gobj, |a| { tu::mnCount_UpdateArrowIndicators_noinline(p(a, 0)); 0 }),
        fd!("fn_802514B8", "p", "v", 4, b_gobj, |a| { tu::fn_802514B8(p(a, 0)); 0 }),
        fd!("fn_802514D8_inline", "pp", "v", 60, b_ud_gobj, |a| { tu::fn_802514D8_inline(p(a, 0), p(a, 1)); 0 }),
        fd!("fn_802514D8", "p", "v", 60, b_gobj, |a| { tu::fn_802514D8(p(a, 0)); 0 }),
        fd!("fn_80251640_FreeText", "ppi", "v", 40, b_ud_gobj_i2, |a| { tu::fn_80251640_FreeText(p(a, 0), p(a, 1), ai32(a, 2)); 0 }),
        fd!("fn_80251640_InitModel", "ppi", "v", 40, b_model2, |a| { tu::fn_80251640_InitModel(p(a, 0), p(a, 1), ai32(a, 2)); 0 }),
        fd!("fn_80251640", "p", "v", 150, b_model_gobj, |a| { tu::fn_80251640(p(a, 0)); 0 }),
        fd!("mnCount_InitUserData", "p", "v", 8, b_ud, |a| { tu::mnCount_InitUserData(p(a, 0)); 0 }),
        fd!("mnCount_InitUserData_noinline", "p", "v", 8, b_ud, |a| { tu::mnCount_InitUserData_noinline(p(a, 0)); 0 }),
        fd!("mnCount_AllocUserData", "", "p", 4, b_alloc, |a| { r(tu::mnCount_AllocUserData()) }),
        fd!("mnCount_Create", "", "v", 40, b_create, |a| { tu::mnCount_Create(); 0 }),
    ];
    let mut fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("mnCount_GetMatchTime", tu::mnCount_GetMatchTime as usize),
        ("mnCount_GetKOKingpin", tu::mnCount_GetKOKingpin as usize),
        ("mnCount_GetNoDefenseNelly", tu::mnCount_GetNoDefenseNelly as usize),
        ("mnCount_GetDisasterMaster", tu::mnCount_GetDisasterMaster as usize),
        ("mnCount_GetSmashChamp", tu::mnCount_GetSmashChamp as usize),
        ("mnCount_GetSmashSap", tu::mnCount_GetSmashSap as usize),
        ("mnCount_GetSlugMeister", tu::mnCount_GetSlugMeister as usize),
        ("mnCount_GetPunchingBag", tu::mnCount_GetPunchingBag as usize),
        ("mnCount_HandleUserInput", tu::mnCount_HandleUserInput as usize),
        ("fn_802514B8", tu::fn_802514B8 as usize),
        ("fn_802514D8", tu::fn_802514D8 as usize),
        ("fn_80251640", tu::fn_80251640 as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_Free", ext::HSD_Free as usize),
    ];
    let _ = (&mut fn_table, null_mut::<c_void>());
    run_tu(&TuOracle { tu: "mncount", funcs: &funcs, globals: tu::oracle_globals(), fn_table });
}

fn b_ud_gobj_row(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { gobj_ud(c) };
    c.arg(g);
    let r = c.rng.range(0, 9) as i32;
    c.arg(r);
    let d = c.rng.range(0, 29) as u32;
    c.arg(d);
}

fn b_gobj_ud(c: &mut Case) {
    unsafe { flow(c) };
    rets(c);
    let g = unsafe { gobj_ud(c) };
    let u = unsafe { (*g).user_data };
    c.arg(g);
    c.arg(u);
}

fn b_ud_gobj_i2(c: &mut Case) {
    b_gobj_ud(c);
    let i = pick(c, &[0, 0, 3, 9]);
    c.arg(i);
}

fn b_model2(c: &mut Case) {
    b_model_pre(c);
    b_gobj_ud(c);
    let i = pick(c, &[0, 0, 3, 9]);
    c.arg(i);
}

fn b_model_pre(c: &mut Case) {
    unsafe {
        tu::model_desc.joint = c.buf(16).cast();
        tu::model_desc.animjoint = c.buf(16).cast();
        tu::model_desc.matanim_joint = c.buf(16).cast();
        tu::model_desc.shapeanim_joint = c.buf(16).cast();
    }
    let j = jobj(c);
    c.ret_ptr("HSD_JObjLoadJoint", j);
}

fn b_model_gobj(c: &mut Case) {
    b_model_pre(c);
    b_gobj(c);
}

fn b_alloc(c: &mut Case) {
    let u = c.zobj::<MnCountData>();
    if c.rng.chance(3, 4) {
        c.ret_ptr("HSD_MemAlloc", u);
    } else {
        c.ret_ptr("HSD_MemAlloc", u);
    }
}

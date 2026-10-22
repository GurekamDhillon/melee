//! State oracle for tu/gmresult.rs (src/oracle.rs, gen/tu/run.py; the C side is the patched copy,
//! gen/tu/patches/gmresult.py: the SJIS words are byte-swapped so x86 holds the big-endian bytes).
use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmresult as tu;
use melee_modes::tu::gmresult::ext;
use melee_types::*;

fn rd() -> &'static mut ResultsData {
    unsafe { &mut *addr_of_mut!(tu::lbl_8046DBE8) }
}

fn pick<T: Copy>(c: &mut Case, xs: &[T]) -> T {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

fn rvec(c: &mut Case) -> Vec3 {
    Vec3 { x: c.rng.f32(), y: c.rng.f32(), z: c.rng.f32() }
}

fn cstr(c: &mut Case) -> *mut u8 {
    let p = c.zbuf(8);
    unsafe {
        *p = b'a' + c.rng.range(0, 20) as u8;
        *p.add(1) = b'a' + c.rng.range(0, 20) as u8;
    }
    p
}

/// A MatchEnd with plausible results (bitfields registered).
fn mk_me(c: &mut Case) -> *mut MatchEnd {
    let me = c.obj::<MatchEnd>();
    unsafe {
        (*me).match_kind = pick(c, &[0u8, 1, 2, 3]);
        (*me).is_teams = pick(c, &[0u8, 1, 1]);
        (*me).outcome = pick(c, &[0u8, 1, 2, 7, 8, 5]);
        (*me).frame_count = pick(c, &[0u32, 3600, 7200, 999999]);
        for i in 0..6usize {
            let ps = addr_of_mut!((*me).player_standings[i]);
            (*ps).pkind = pick(c, &[0u8, 1, 2, 3, 3]);
            (*ps).ckind = c.rng.range(0, 25) as i8;
            (*ps).team = c.rng.range(0, 3) as u8;
            (*ps).stocks = c.rng.range(-1, 4) as i8;
            (*ps).score = c.rng.range(-5, 40) as i32;
            (*ps).x28 = pick(c, &[0u32, 3600, 3601, (*me).frame_count, 400000]);
            (*ps).is_big_loser = c.rng.range(0, 4) as u8;
            (*ps).is_small_loser = c.rng.range(0, 4) as u8;
            c.bitfields::<MatchPlayerData>(ps, offset_of!(MatchPlayerData, anon_3), 3, 1);
        }
        for t in 0..5usize {
            (*me).team_standings[t].is_big_loser = c.rng.range(0, 3) as u8;
        }
        for p in 0..6usize {
            let u = addr_of_mut!((*me).x44C[p]);
            for i in 0..257usize {
                (*u).x0[i] = if c.rng.chance(1, 8) { c.rng.range(1, 3) as u8 } else { 0 };
                (*u).x104[i] = c.rng.range(-30, 300) as i32;
            }
        }
    }
    me
}

/// The scene state (`lbl_8046DBE8`) around a MatchEnd.
fn mk_rd(c: &mut Case, me: *mut MatchEnd) {
    unsafe {
        let d = rd();
        *d = core::mem::zeroed();
        c.bitfields::<ResultsData>(d, offset_of!(ResultsData, bf_0), 0, 1);
        d.bf_0[0] = c.rng.range(0, 255) as u8;
        d.x94 = me;
        d.x4 = c.rng.range(0, 3) as u8;
        d.x5 = c.rng.range(0, 3) as u8;
        d.x6 = c.rng.range(0, 3) as u8;
        d.num_pages = pick(c, &[2u8, 3]);
        d.cobj = if c.rng.chance(1, 5) { null_mut() } else { c.obj::<HSD_CObj>() };
        d.x18 = c.gobj_any();
        d.x20 = emblem_child(c);
        d.x24 = c.jobj();
        d.x28 = c.jobj();
        d.x2C = if c.rng.chance(1, 2) { null_mut() } else { c.zobj::<HSD_Text>() };
        d.x30 = c.jobj();
        for i in 0..6usize {
            d.x34[i] = c.jobj();
            d.x4C[i] = rvec(c);
        }
        d.xC = c.rng.f32();
        d.pnlsce = c.scene_desc();
        d.flmsce = c.scene_desc();
        for p in 0..6usize {
            let pd = addr_of_mut!(d.player_data[p]);
            c.bitfields::<ResultsPlayerData>(pd, offset_of!(ResultsPlayerData, bf_0), 0, 1);
            (*pd).bf_0[0] = c.rng.range(0, 255) as u8;
            (*pd).page = pick(c, &[0u8, 1, 2]);
            (*pd).scroll_offset = pick(c, &[0.0f32, 0.5, 1.0, 3.7, 12.0, 25.0]);
            for row in 0..3usize {
                for i in 0..10usize {
                    (*pd).stats_text[row][i] = if c.rng.chance(1, 2) { null_mut() } else { c.zobj::<HSD_Text>() };
                }
            }
            (*pd).ko_count = c.zobj::<HSD_Text>();
            (*pd).ko_time = c.zobj::<HSD_Text>();
            for j in 0..15usize {
                (*pd).jobjs[j] = c.jobj();
            }
            (*pd).stats_position = rvec(c);
        }
        for i in 0..4usize {
            tu::lbl_8046E190[i] = tu::ResultsStatsInfo { x0: 2, x1: c.rng.range(2, 9) as u8, pad_2: [0; 2], x4: 0 };
        }
    }
}

/// A JObj with `u.dobj -> mobj -> (tobj -> aobj, aobj)`.
fn emblem_child(c: &mut Case) -> *mut HSD_JObj {
    let j = c.jobj();
    unsafe {
        let a = c.aobj();
        let t = c.zobj::<HSD_TObj>();
        (*t).aobj = a;
        let m = c.zobj::<HSD_MObj>();
        (*m).tobj = t;
        (*m).aobj = c.aobj();
        let dobj = c.zobj::<HSD_DObj>();
        (*dobj).mobj = m;
        (*j).u.dobj = dobj;
    }
    j
}

/// An emblem root: `n` children, each with the dobj chain.
fn emblem_root(c: &mut Case, n: usize) -> *mut HSD_JObj {
    let root = c.jobj();
    let mut prev: *mut HSD_JObj = null_mut();
    for _ in 0..n {
        let ch = emblem_child(c);
        unsafe {
            if prev.is_null() {
                (*root).child = ch;
            } else {
                (*prev).next = ch;
            }
        }
        prev = ch;
    }
    root
}

fn text_pools(c: &mut Case) {
    let n = core::mem::size_of::<HSD_Text>();
    c.ret_pool("HSD_SisLib_803A6754", n);
    c.ret_pool("HSD_SisLib_803A5ACC", n);
}

/// Return values for the stats-table callbacks.
fn table_rets(c: &mut Case) {
    let s = cstr(c);
    let s2 = cstr(c);
    for n in ["fn_8017BB94", "fn_8017BC50", "fn_8017BD0C", "fn_8017BDC8"] {
        c.ret_seq(n, &[s, null_mut(), s2, s]);
    }
    for n in [
        "fn_8017AE70", "fn_8017AED8", "fn_8017AF40", "fn_8017AFA8", "fn_8017B07C", "fn_8017B0E4", "fn_8017B14C", "fn_8017B1B4", "fn_8017B21C",
        "fn_8017AE0C", "fn_8017B010", "fn_8017B280", "fn_8017B2E4", "fn_8017B348", "fn_8017B3AC", "fn_8017B410", "fn_8017B4D0", "fn_8017B534",
        "fn_8017B598", "fn_8017B5FC", "fn_8017B660", "fn_8017B6C4", "fn_8017B728", "fn_8017B78C", "fn_8017B7F0", "fn_8017B854", "fn_8017B8B8",
        "fn_8017B91C", "fn_8017B9F4", "fn_8017BACC", "fn_8017BB30",
    ] {
        c.ret_int(n, -3, 1500);
    }
}

fn b_noop(c: &mut Case) {
    let _ = c;
}

fn b_me_rd(c: &mut Case) {
    let me = mk_me(c);
    mk_rd(c, me);
}

fn b_slot(c: &mut Case) {
    b_me_rd(c);
    text_pools(c);
    table_rets(c);
    c.ret_int("gm_80160968", 0, 0xFFFF_FFFF);
    c.ret_int("fn_8017AD28", -30, 30);
    c.ret_int("fn_8017AD48", -30, 30);
    c.ret_int("fn_8017AD78", -30, 30);
    c.ret_int("fn_8017ADA8", -30, 30);
    c.mem_arg("HSD_SisLib_803A74F0", 2, 4);
    c.out("lb_8000B1CC", 2, 12);
    let s = c.rng.range(0, 3) as i32;
    c.arg(s);
}

fn b_slot_u8(c: &mut Case) {
    b_me_rd(c);
    let s = c.rng.range(0, 5) as i32;
    c.arg(s);
}

fn b_outcome(c: &mut Case) {
    let v = pick(c, &[0u8, 1, 2, 5, 6, 7, 8, 9, 255]) as i32;
    c.arg(v);
}

fn stats_entry(c: &mut Case) -> *mut tu::StatsEntry {
    let e = c.zobj::<tu::StatsEntry>();
    unsafe {
        (*e).value = pick(c, &[-1i16, -1, 0, 5, 0x20]);
        (*e).check = pick(c, &[None, Some(ext::fn_8017AE70 as unsafe extern "C" fn(i32) -> i32)]);
        (*e).get = pick(c, &[None, Some(ext::fn_8017BB94 as unsafe extern "C" fn(i32) -> *const i8)]);
    }
    e
}

fn b_entry(c: &mut Case) {
    table_rets(c);
    let e = stats_entry(c);
    let s = c.rng.range(0, 3) as i32;
    c.arg(s);
    c.arg(e);
}

fn b_find_nth(c: &mut Case) {
    let p = c.zbuf(260);
    unsafe {
        for i in 0..256usize {
            if c.rng.chance(1, 6) {
                *p.add(i) = 1;
            }
        }
    }
    c.arg(p);
    let n = c.rng.range(0, 12) as i32;
    c.arg(n);
}

fn b_format_label(c: &mut Case) {
    table_rets(c);
    let e = c.zobj::<tu::StatsEntry>();
    unsafe { (*e).get = Some(ext::fn_8017BB94) };
    let t = c.zobj::<HSD_Text>();
    let s = c.rng.range(0, 3) as i32;
    c.arg(s);
    c.arg(e);
    c.arg(0.0f32);
    c.arg(-30.0f32);
    c.arg(t);
}

fn b_row(c: &mut Case) {
    b_me_rd(c);
    table_rets(c);
    c.ret_int("fn_8016F1F0", 0, 0x200);
    c.ret_int("fn_8016F280", 0, 0x30000);
    let slot = c.rng.range(0, 3) as i32;
    let t1 = c.zobj::<HSD_Text>();
    let t2 = c.zobj::<HSD_Text>();
    let t3 = c.zobj::<HSD_Text>();
    let mode = c.rng.range(0, 2);
    let list: *mut tu::StatsList;
    let idx;
    unsafe {
        if mode == 2 {
            list = addr_of_mut!(tu::lbl_8046E190[slot as usize]).cast();
            idx = c.rng.range(0, 8) as i32;
        } else {
            list = addr_of_mut!(tu::lbl_803D6878[mode as usize]);
            idx = c.rng.range(0, (*list).count as i64 - 1) as i32;
        }
    }
    c.arg(slot);
    c.arg(t1);
    c.arg(t2);
    c.arg(t3);
    c.arg(list);
    c.arg(idx);
}

fn b_find_list(c: &mut Case) {
    let arr = c.zobj::<[tu::ResultsStatsInfo; 4]>();
    unsafe {
        for (i, m) in [0u8, 1, 3, 2].iter().enumerate() {
            (*arr)[i].x0 = *m;
        }
    }
    let mode = pick(c, &[0i32, 1, 2, 3]);
    let idx = c.rng.range(0, 3) as i32;
    c.arg(arr);
    c.arg(mode);
    c.arg(idx);
}

fn b_pdata(c: &mut Case) {
    b_me_rd(c);
    let s = c.rng.range(0, 5) as usize;
    let p = addr_of_mut!(rd().player_data[s]);
    c.arg(p);
}

fn b_scissor(c: &mut Case) {
    b_me_rd(c);
    c.out("lbVector_WorldToScreen", 2, 12);
    c.out("lb_8000B1CC", 2, 12);
    let g = c.gobj_any();
    c.arg(g);
}

fn b_list_slot(c: &mut Case) {
    b_me_rd(c);
    table_rets(c);
    let mode = c.rng.range(0, 2) as usize;
    let slot = c.rng.range(0, 3) as i32;
    let list: *mut tu::StatsList = unsafe { addr_of_mut!(tu::lbl_803D6878[mode]) };
    c.arg(list);
    c.arg(slot);
}

fn b_blk(c: &mut Case) {
    b_me_rd(c);
    text_pools(c);
    let slot = c.rng.range(0, 5) as i32;
    c.arg(addr_of_mut!(tu::lbl_8046DBE8));
    c.arg(slot);
    c.arg(0u64);
    c.arg(0u64);
}

fn b_rows(c: &mut Case) {
    b_me_rd(c);
    text_pools(c);
    table_rets(c);
    c.ret_int("fn_8016F1F0", 0, 0x200);
    c.ret_int("fn_8016F280", 0, 0x30000);
    let slot = c.rng.range(0, 3) as i32;
    c.arg(addr_of_mut!(tu::lbl_8046DBE8));
    c.arg(slot);
}

fn b_dobj(c: &mut Case) {
    let j = emblem_child(c);
    c.arg(j);
    let v = c.rng.range(-3, 40) as i32;
    c.arg(v);
}

fn b_disp(c: &mut Case) {
    let j = c.jobj();
    let g = c.gobj(j.cast());
    c.ret_int("HSD_GObj_80390EB8", 0, 0xFFFF);
    c.arg(g);
    let f = c.rng.range(0, 8) as i32;
    c.arg(f);
}

fn b_pos(c: &mut Case) {
    b_slot(c);
    let v = c.obj::<Vec3>();
    let _ = v;
    unsafe {
        // (re-pushed args: slot is arg 0)
        let slot = c.args[0] as i32;
        c.args.clear();
        c.arg(slot);
        c.arg(v);
    }
}

fn b_dc8(c: &mut Case) {
    b_me_rd(c);
    text_pools(c);
    c.out("lb_80011E24", 1, 8);
    c.out("lb_8000B1CC", 2, 12);
    c.mem_arg("lb_8000B1CC", 1, 12);
    let j = c.jobj_chain(2);
    c.ret_ptr("HSD_JObjLoadJoint", j);
    let g = c.gobj_any();
    c.arg(g);
}

fn b_cam(c: &mut Case) {
    b_me_rd(c);
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.str_arg("HSD_SisLib_803A62A0", 1);
    c.str_arg("HSD_SisLib_803A62A0", 2);
    if c.rng.chance(1, 4) {
        c.ret_ptr("GObj_Create", null_mut::<c_void>());
    } else {
        c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    }
    if c.rng.chance(1, 4) {
        c.ret_ptr("HSD_CObjLoadDesc", null_mut::<c_void>());
    } else {
        c.ret_pool("HSD_CObjLoadDesc", core::mem::size_of::<HSD_CObj>());
    }
}

fn b_gobj_jobj(c: &mut Case) {
    let j = c.jobj();
    let g = c.gobj(j.cast());
    c.arg(g);
}

fn b_ckind(c: &mut Case) {
    c.ret_int("Mex_InsigniaForExt", -2, 20);
    c.ret_int("Mex_PortCKindToExt", 0, 60);
    let v = if c.idx < 40 { c.idx as i32 } else { c.rng.range(0, 130) as i32 };
    c.arg(v);
}

fn b_emblem(c: &mut Case) {
    c.ret_int("Mex_InsigniaForExt", -2, 20);
    c.ret_int("Mex_PortCKindToExt", 0, 60);
    let n = pick(c, &[0usize, 5, 12, 14, 20]);
    let root = emblem_root(c, n);
    c.out_ptrs("lb_80011E24", 1, &[root]);
    let j = c.jobj();
    c.arg(j);
    let ck = if c.rng.chance(1, 4) { c.rng.range(34, 127) } else { c.rng.range(0, 30) } as u8;
    c.arg(ck);
    let a2 = pick(c, &[0i32, 1]);
    c.arg(a2);
}

fn b_podium(c: &mut Case) {
    b_me_rd(c);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("HSD_JObjLoadJoint", core::mem::size_of::<HSD_JObj>());
    let pos = c.buf(4 * 12);
    c.arg(pos);
}

fn b_f60(c: &mut Case) {
    b_me_rd(c);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    // the model tree: its first four children are the podium spots
    let root = c.jobj();
    let mut prev: *mut HSD_JObj = null_mut();
    for _ in 0..pick(c, &[2usize, 4, 5]) {
        let ch = c.jobj();
        unsafe {
            if prev.is_null() {
                (*root).child = ch;
            } else {
                (*prev).next = ch;
            }
        }
        prev = ch;
    }
    c.ret_ptr("HSD_JObjLoadJoint", root);
    let er = emblem_root(c, 14);
    c.out_ptrs("lb_80011E24", 1, &[er]);
    c.ret_int("Mex_InsigniaForExt", -2, 20);
    c.ret_int("Mex_PortCKindToExt", 0, 60);
    c.ret_int("Player_GetTeam", 0, 3);
    c.ret_int("gm_80160854", 0, 20);
}

fn b_771c0(c: &mut Case) {
    b_me_rd(c);
    c.ret_int("fn_80165418", 0, 3);
    c.ret_int("fn_801654A0", 0, 3);
    c.arg(addr_of_mut!(tu::lbl_8046DBE8));
}

fn b_archive(c: &mut Case) {
    b_me_rd(c);
    if c.rng.chance(1, 3) {
        rd().pnlsce = null_mut();
    }
    if c.rng.chance(1, 3) {
        rd().flmsce = null_mut();
    }
    c.arg(addr_of_mut!(tu::lbl_8046DBE8));
}

fn b_report(c: &mut Case) {
    let _ = c;
}

fn b_enter(c: &mut Case) {
    b_f60(c);
    text_pools(c);
    let info = c.obj::<ResultsMatchInfo>();
    unsafe {
        c.bitfields::<ResultsMatchInfo>(info, offset_of!(ResultsMatchInfo, bf_0), 0, 1);
        let me = mk_me(c);
        (*info).match_end = *me;
        for i in 0..6usize {
            let ps = addr_of_mut!((*info).match_end.player_standings[i]);
            c.bitfields::<MatchPlayerData>(ps, offset_of!(MatchPlayerData, anon_3), 3, 1);
        }
    }
    // the scene state the stubs would leave (memzero / archive loads do nothing in the oracle)
    let scratch_me = mk_me(c);
    mk_rd(c, scratch_me);
    c.out("lb_8000B1CC", 2, 12);
    c.ret_pool("HSD_CObjLoadDesc", core::mem::size_of::<HSD_CObj>());
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.ret_int("fn_801701B8", 0, 1);
    c.ret_int("fn_80165418", 0, 3);
    c.ret_int("fn_801654A0", 0, 3);
    c.ret_int("fn_80160400", 0, 100);
    c.ret_pool("fn_8017A67C", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("fn_8017A318", core::mem::size_of::<HSD_GObj>());
    c.ret_int("fn_8016F140", 0, 1);
    c.arg(info);
}

#[test]
fn gmresult_vs_c() {
    let funcs = [
        fd!("fn_80174274", "", "p", 4, b_me_rd, |a| { r(tu::fn_80174274()) }),
        fd!("fn_80174284", "i", "i32", 24, b_slot_u8, |a| { r(tu::fn_80174284(ai32(a, 0) as u8)) }),
        fd!("fn_80174284_noinline", "i", "i32", 8, b_slot_u8, |a| { r(tu::fn_80174284_noinline(ai32(a, 0) as u8)) }),
        fd!("fn_80174338", "", "v", 2, b_noop, |a| { tu::fn_80174338(); 0 }),
        fd!("fn_8017435C", "", "v", 2, b_noop, |a| { tu::fn_8017435C(); 0 }),
        fd!("fn_80174380", "", "v", 2, b_noop, |a| { tu::fn_80174380(); 0 }),
        fd!("gm_WasMatchCanceled", "i", "i32", 12, b_outcome, |a| { r(tu::gm_WasMatchCanceled(au8(a, 0)) as i32) }),
        fd!("fn_801743C4", "ip", "i32", 40, b_entry, |a| { r(tu::fn_801743C4(ai32(a, 0), p(a, 1))) }),
        fd!("gmResultFindNth", "pi", "i32", 30, b_find_nth, |a| { r(tu::gmResultFindNth(p::<u8>(a, 0), ai32(a, 1))) }),
        fd!("gmResultFormatLabel", "ipffp", "i32", 12, b_format_label, |a| { r(tu::gmResultFormatLabel(ai32(a, 0), p(a, 1), af32(a, 2), af32(a, 3), p(a, 4))) }),
        fd!("fn_80174468", "ippppi", "v", 120, b_row, |a| { tu::fn_80174468(ai32(a, 0), p(a, 1), p(a, 2), p(a, 3), p(a, 4), ai32(a, 5)); 0 }),
        fd!("fn_801748EC", "pii", "p", 16, b_find_list, |a| { r(tu::fn_801748EC(p(a, 0), ai32(a, 1), ai32(a, 2))) }),
        fd!("fn_80174920", "p", "v", 8, b_pdata, |a| { tu::fn_80174920(p(a, 0)); 0 }),
        fd!("fn_801749B8", "p", "v", 20, b_scissor, |a| { tu::fn_801749B8(p(a, 0)); 0 }),
        fd!("fn_80174A60", "pi", "i32", 30, b_list_slot, |a| { r(tu::fn_80174A60(p(a, 0), ai32(a, 1))) }),
        fd!("fn_80174B4C_blk14829", "pioo", "v", 12, b_blk, |a| {
            let mut pd: *mut ResultsPlayerData = null_mut();
            let mut l: *mut tu::StatsList = null_mut();
            tu::fn_80174B4C_blk14829(p(a, 0), ai32(a, 1), &mut pd, &mut l);
            0
        }),
        fd!("fn_80174B4C", "pi", "v", 120, b_rows, |a| { tu::fn_80174B4C(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_80174FD0", "pi", "v", 8, b_dobj, |a| { tu::fn_80174FD0(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_80175038", "pi", "v", 8, b_disp, |a| { tu::fn_80175038(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_8017507C", "i", "g4", 40, b_slot, |a| { r(tu::fn_8017507C(ai32(a, 0))) }),
        fd!("fn_80175240", "i", "v", 40, b_slot, |a| { tu::fn_80175240(ai32(a, 0)); 0 }),
        fd!("fn_8017556C", "i", "v", 40, b_slot, |a| { tu::fn_8017556C(ai32(a, 0)); 0 }),
        fd!("fn_801756E0", "i", "v", 40, b_slot, |a| { tu::fn_801756E0(ai32(a, 0)); 0 }),
        fd!("fn_80175880", "i", "v", 60, b_slot, |a| { tu::fn_80175880(ai32(a, 0)); 0 }),
        fd!("fn_80175A94_get_match_end", "", "p", 2, b_me_rd, |a| { r(tu::fn_80175A94_get_match_end()) }),
        fd!("fn_80175A94", "ip", "v", 60, b_pos, |a| { tu::fn_80175A94(ai32(a, 0), p(a, 1)); 0 }),
        fd!("fn_80175C5C", "", "v", 12, b_slot, |a| { tu::fn_80175C5C(); 0 }),
        fd!("fn_80175D34", "", "v", 12, b_slot, |a| { tu::fn_80175D34(); 0 }),
        fd!("fn_80175DC8", "p", "v", 24, b_dc8, |a| { tu::fn_80175DC8(p(a, 0)); 0 }),
        fd!("fn_80176A6C", "", "v", 16, b_cam, |a| { tu::fn_80176A6C(); 0 }),
        fd!("fn_80176BCC", "p", "v", 4, b_gobj_jobj, |a| { tu::fn_80176BCC(p(a, 0)); 0 }),
        fd!("fn_80176BF0_inline", "i", "i32", 48, b_ckind, |a| { r(tu::fn_80176BF0_inline(au8(a, 0))) }),
        fd!("fn_80176BF0", "pii", "p", 60, b_emblem, |a| { r(tu::fn_80176BF0(p(a, 0), au8(a, 1), ai32(a, 2))) }),
        fd!("fn_80176D18", "p", "v", 4, b_gobj_jobj, |a| { tu::fn_80176D18(p(a, 0)); 0 }),
        fd!("fn_80176D3C", "p", "v", 40, b_podium, |a| { tu::fn_80176D3C(p(a, 0)); 0 }),
        fd!("fn_80176F60", "", "v", 40, b_f60, |a| { tu::fn_80176F60(); 0 }),
        fd!("fn_801771C0", "p", "v", 40, b_771c0, |a| { tu::fn_801771C0(p(a, 0)); 0 }),
        fd!("gmResultLoadArchive", "p", "v", 12, b_archive, |a| { tu::gmResultLoadArchive(p(a, 0)); 0 }),
        fd!("gmResultReportLightGObj", "", "v", 1, b_report, |a| { tu::gmResultReportLightGObj(); 0 }),
        fd!("gmResultReportLightLObj", "", "v", 1, b_report, |a| { tu::gmResultReportLightLObj(); 0 }),
        fd!("gmResultReportModelGObj", "", "v", 1, b_report, |a| { tu::gmResultReportModelGObj(); 0 }),
        fd!("gm_Scene_Results_OnEnter", "p", "v", 60, b_enter, |a| { tu::gm_Scene_Results_OnEnter(p(a, 0)); 0 }),
        fd!("gm_Scene_Results_OnExit", "p", "v", 2, b_noop, |a| { tu::gm_Scene_Results_OnExit(null_mut()); 0 }),
    ];
    let mut globals = tu::oracle_globals();
    globals.extend(tu::oracle_globals_ext());
    let mut fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("fn_80175038", tu::fn_80175038 as usize),
        ("fn_801749B8", tu::fn_801749B8 as usize),
        ("fn_80175DC8", tu::fn_80175DC8 as usize),
        ("fn_80176BCC", tu::fn_80176BCC as usize),
        ("fn_80176D18", tu::fn_80176D18 as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize),
        ("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize),
        ("fn_80179350", ext::fn_80179350 as usize),
    ];
    macro_rules! reg {
        ($($n:ident),*) => { $( fn_table.push((stringify!($n), ext::$n as usize)); )* };
    }
    reg!(
        fn_8017AE70, fn_8017AED8, fn_8017AF40, fn_8017AFA8, fn_8017B07C, fn_8017B0E4, fn_8017B14C, fn_8017B1B4, fn_8017B21C, fn_8017AE0C,
        fn_8017B010, fn_8017B280, fn_8017B2E4, fn_8017B348, fn_8017B3AC, fn_8017B410, fn_8017B4D0, fn_8017B534, fn_8017B598, fn_8017B5FC,
        fn_8017B660, fn_8017B6C4, fn_8017B728, fn_8017B78C, fn_8017B7F0, fn_8017B854, fn_8017B8B8, fn_8017B91C, fn_8017B9F4, fn_8017BACC,
        fn_8017BB30, fn_8017BB94, fn_8017BC50, fn_8017BD0C, fn_8017BDC8
    );
    run_tu(&TuOracle { tu: "gmresult", funcs: &funcs, globals, fn_table });
}

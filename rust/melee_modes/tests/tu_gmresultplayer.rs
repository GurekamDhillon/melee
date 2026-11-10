//! State oracle for tu/gmresultplayer.rs (src/oracle.rs, gen/tu/run.py): every function of
//! src/melee/gm/gmresultplayer.c (TARGET_PC, with its two .inc files) runs on a synthetic results
//! scene, in the C compiled natively and in the Rust port, and the resulting state is compared.
use core::mem::offset_of;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmresultplayer as tu;
use melee_modes::tu::gmresultplayer::ext;
use melee_types::*;

/// Every builder first clears the stats table (gmresult.c's; here only its address matters, and its
/// pointers into Rust statics would not hash the same in two processes).
macro_rules! z {
    ($b:ident) => {{
        fn f(c: &mut Case) {
            unsafe { core::ptr::write_bytes(addr_of_mut!(melee_modes::tu::gmresult::lbl_803D6878), 0, 1) };
            let e = pick(c, &[-1i32, 0, 5, 20, 24, 25, 30, 38]);
            KEY_FRAME.store(e, core::sync::atomic::Ordering::Relaxed);
            $b(c)
        }
        f as fn(&mut Case)
    }};
}

fn rd() -> &'static mut ResultsData {
    unsafe { &mut *addr_of_mut!(melee_modes::tu::gmresult::lbl_8046DBE8) }
}

fn pick<T: Copy>(c: &mut Case, xs: &[T]) -> T {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

// ---- animation streams -------------------------------------------------------------------

fn val_bytes(c: &mut Case, frac: u8, out: &mut std::vec::Vec<u8>) {
    match frac & 0xE0 {
        0x00 => {
            let f = c.rng.range(0, 40) as f32 + pick(c, &[0.0f32, 0.25, 0.5]);
            out.extend_from_slice(&f.to_bits().to_be_bytes());
        }
        0x20 | 0x40 => {
            // (image indices stay below ~500: the port allocates a table of them from an 8 KiB pool)
            out.push(c.rng.range(0, 1) as u8);
            out.push(c.rng.range(0, 255) as u8);
        }
        _ => out.push(c.rng.range(0, 255) as u8),
    }
}

fn packed(c: &mut Case, out: &mut std::vec::Vec<u8>) {
    let w = pick(c, &[0u32, 1, 2, 3, 5, 9, 20, 127, 128, 300, 5000]);
    let mut v = w;
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            out.push(b | 0x80);
        } else {
            out.push(b);
            break;
        }
    }
}

/// A texture-image track: `(frac_value, frac_slope, bytes)`.
fn stream(c: &mut Case) -> (u8, u8, std::vec::Vec<u8>) {
    let fv = pick(c, &[0x00u8, 0x20, 0x40, 0x60, 0x80, 0x01, 0x22, 0x43, 0xA0, 0xE1]);
    let fs = pick(c, &[0x00u8, 0x20, 0x60, 0x80, 0x42]);
    let mut v: std::vec::Vec<u8> = std::vec![];
    for _ in 0..c.rng.range(0, 4) {
        let op = pick(c, &[1u8, 2, 3, 4, 5, 6, 1, 2, 6, 3, 4, 5, 0, 9]);
        let n = pick(c, &[0u32, 0, 1, 2, 3, 7, 8, 12]);
        let mut b = op | (((n & 7) as u8) << 4);
        if n > 7 {
            b |= 0x80;
            v.push(b);
            v.push(((n >> 3) & 0x7F) as u8);
        } else {
            v.push(b);
        }
        if !(1..=6).contains(&op) {
            break;
        }
        for _ in 0..=n {
            match op {
                1 | 2 | 3 => {
                    val_bytes(c, fv, &mut v);
                    packed(c, &mut v);
                }
                4 => {
                    val_bytes(c, fv, &mut v);
                    val_bytes(c, fs, &mut v);
                    packed(c, &mut v);
                }
                5 => val_bytes(c, fs, &mut v),
                _ => val_bytes(c, fv, &mut v),
            }
        }
    }
    (fv, fs, v)
}

/// A frame the scene builder wants a texture-image key on (the m-ex name frame), or -1.
static KEY_FRAME: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);

/// An AObj whose FObj list holds 0..2 tracks (usually texture-image ones).
fn aobj_tracks(c: &mut Case) -> *mut HSD_AObj {
    let a = c.aobj();
    let mut head: *mut HSD_FObj = null_mut();
    let kf = KEY_FRAME.load(core::sync::atomic::Ordering::Relaxed);
    if kf >= 0 && c.rng.chance(3, 4) {
        // one CON key at frame `kf`: float value, wait
        let mut bytes: std::vec::Vec<u8> = std::vec![1u8];
        bytes.extend_from_slice(&(c.rng.range(0, 30) as f32).to_bits().to_be_bytes());
        bytes.push(3);
        let buf = c.zbuf(bytes.len() + 8);
        let fo = c.zobj::<HSD_FObj>();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
            (*fo).ad_head = buf;
            (*fo).ad = buf;
            (*fo).length = bytes.len() as u32;
            (*fo).obj_type = 1;
            (*fo).frac_value = 0;
            (*fo).startframe = kf as i16;
            (*a).fobj = fo;
        }
        return a;
    }
    for _ in 0..c.rng.range(0, 2) {
        let (fv, fs, bytes) = stream(c);
        let buf = c.zbuf(bytes.len() + 8);
        let fo = c.zobj::<HSD_FObj>();
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
            (*fo).ad_head = buf;
            (*fo).ad = buf;
            (*fo).length = (bytes.len() as i64 - if c.rng.chance(1, 8) { 1 } else { 0 }).max(0) as u32;
            (*fo).obj_type = pick(c, &[1u8, 1, 1, 2, 5]);
            (*fo).frac_value = fv;
            (*fo).frac_slope = fs;
            (*fo).startframe = c.rng.range(-3, 20) as i16;
            (*fo).next = head;
        }
        head = fo;
    }
    unsafe { (*a).fobj = head };
    a
}

/// A TObj with an image (`w` x `h`, `format`), a track-carrying AObj, a TLUT and optionally a TLUT table.
fn tobj_img(c: &mut Case, w: u16, h: u16, fmt: u32) -> *mut HSD_TObj {
    let t = c.zobj::<HSD_TObj>();
    let img = c.zobj::<HSD_ImageDesc>();
    unsafe {
        (*img).image_ptr = c.zbuf(64).cast();
        (*img).width = w;
        (*img).height = h;
        (*img).format = fmt as _;
        (*t).imagedesc = img;
        (*t).aobj = if c.rng.chance(1, 6) { null_mut() } else { aobj_tracks(c) };
        (*t).tlut = if c.rng.chance(1, 4) { null_mut() } else { c.zobj::<HSD_Tlut>() };
        if c.rng.chance(1, 2) {
            let tbl = c.zbuf(8 * 4) as *mut *mut HSD_Tlut;
            for i in 0..c.rng.range(0, 2) as usize {
                *tbl.add(i) = c.zobj::<HSD_Tlut>();
            }
            (*t).tluttbl = tbl;
        }
        (*t).tlut_no = c.rng.range(-1, 3) as i32;
    }
    t
}

fn tobj_any(c: &mut Case) -> *mut HSD_TObj {
    let (w, h) = if c.rng.chance(1, 12) {
        pick(c, &[(1024u16, 8u16), (1025, 8), (8, 1024), (0, 8), (8, 0), (1, 1)])
    } else {
        (pick(c, &[8u16, 16, 32, 64]), pick(c, &[8u16, 16, 20, 24, 64]))
    };
    let f = pick(c, &[0u32, 0, 0, 1, 5, 8, 9, 10]);
    tobj_img(c, w, h, f)
}

/// A JObj whose `u.dobj` is a chain of `n` DObjs, each with a MObj carrying a TObj and an AObj.
fn jobj_dobjs(c: &mut Case, n: usize) -> *mut HSD_JObj {
    let j = c.jobj();
    unsafe {
        (*j).aobj = c.aobj();
        let mut head: *mut HSD_DObj = null_mut();
        for _ in 0..n {
            let d = c.zobj::<HSD_DObj>();
            let m = c.zobj::<HSD_MObj>();
            (*m).tobj = tobj_any(c);
            (*m).aobj = c.aobj();
            (*d).mobj = m;
            (*d).next = head;
            head = d;
        }
        (*j).u.dobj = head;
    }
    j
}

// ---- the results scene ---------------------------------------------------------------------

fn mk_me(c: &mut Case) -> *mut MatchEnd {
    let me = c.obj::<MatchEnd>();
    unsafe { fill_me(c, me) };
    me
}

unsafe fn fill_me(c: &mut Case, me: *mut MatchEnd) {
    (*me).match_kind = pick(c, &[0u8, 1, 2, 3]);
    (*me).is_teams = pick(c, &[0u8, 1]);
    (*me).outcome = pick(c, &[0u8, 1, 2, 7, 8, 5]);
    (*me).sd_penalty = pick(c, &[0i8, 1, -1, 5, -7, -128, 127]);
    for i in 0..6usize {
        let ps = addr_of_mut!((*me).player_standings[i]);
        (*ps).pkind = pick(c, &[0u8, 1, 2, 3, 3, 0, 1]);
        (*ps).ckind = pick(c, &[0i8, 3, 10, 25, 26, 33, 34, 40, 100, 127, 0, 5, 36]);
        (*ps).ftkind = c.rng.range(0, 5) as i8;
        (*ps).team = c.rng.range(0, 3) as u8;
        (*ps).stocks = c.rng.range(-1, 4) as i8;
        (*ps).xE = c.rng.range(0, 9) as u16;
        (*ps).is_big_loser = pick(c, &[0u8, 0, 1, 2, 3]);
        (*ps).is_small_loser = pick(c, &[0u8, 0, 1, 2]);
        c.bitfields::<MatchPlayerData>(ps, offset_of!(MatchPlayerData, x3_u), 3, 1);
    }
    for t in 0..5usize {
        (*me).team_standings[t].is_big_loser = pick(c, &[0u8, 0, 1, 2]);
    }
}

/// The scene state (`lbl_8046DBE8`) around a MatchEnd, and the pads.
fn mk_rd(c: &mut Case, me: *mut MatchEnd) {
    unsafe {
        let d = rd();
        *d = core::mem::zeroed();
        c.bitfields::<ResultsData>(d, offset_of!(ResultsData, bf_0), 0, 1);
        d.bf_0[0] = c.rng.range(0, 255) as u8;
        d.x94 = me;
        d.x1 = pick(c, &[0u8, 1, 2, 3, 4, 5]);
        d.x3 = pick(c, &[0u8, 1, 5, 20]);
        d.x4 = c.rng.range(0, 3) as u8;
        d.x5 = c.rng.range(0, 3) as u8;
        d.x6 = c.rng.range(0, 3) as u8;
        d.x8 = pick(c, &[0i32, 1, 2, 0x9A, 0xA0, 0xA2, 0xA1, 7000, -1, -2]);
        d.num_pages = pick(c, &[2u8, 3]);
        d.x30 = { let n = pick(c, &[2usize, 3]); jobj_dobjs(c, n) };
        for p in 0..6usize {
            let pd = addr_of_mut!(d.player_data[p]);
            c.bitfields::<ResultsPlayerData>(pd, offset_of!(ResultsPlayerData, bf_0), 0, 1);
            (*pd).bf_0[0] = c.rng.range(0, 255) as u8;
            (*pd).page = pick(c, &[0u8, 1, 2]);
            (*pd).x2 = pick(c, &[0u16, 5, 180, 181, 999, 1000, 1001, 65535]);
            (*pd).scroll_offset = pick(c, &[0.0f32, 0.05, 0.5, 1.0, 3.7, 12.0, 25.0, 25.95]);
            for j in 0..15usize {
                (*pd).jobjs[j] = match j {
                    0 | 7 => { let n = pick(c, &[1usize, 2, 3]); jobj_dobjs(c, n) },
                    2 | 5 => jobj_dobjs(c, 1),
                    _ => c.jobj(),
                };
            }
            (*pd).stats_position = Vec3 { x: c.rng.f32(), y: c.rng.f32(), z: c.rng.f32() };
        }
        let pads = &mut *addr_of_mut!(melee_runtime::tu::controller::HSD_PadCopyStatus);
        let master = &mut *addr_of_mut!(melee_modes::tu::gmregclear::HSD_PadMasterStatus);
        for i in 0..4usize {
            pads[i].err = pick(c, &[0i8, 0, 0, -1, 5]);
            pads[i].trigger = pick(c, &[0u32, 0, 0x100, 0x200, 0x1000, 0x40000, 0x80000, 0x1100, 0x201, 0x40000 | 0x1000]);
            pads[i].nml_stickY = pick(c, &[0.0f32, -0.0, 0.2, 0.29, 0.3, 0.31, 0.8, -0.2, -0.3, -0.5, -1.0, 1.0]);
            master[i].err = pick(c, &[0i8, 0, 0, -1]);
            master[i].trigger = pick(c, &[0u32, 0x1000, 0x100]);
        }
    }
}

fn b_me_rd(c: &mut Case) {
    let me = mk_me(c);
    mk_rd(c, me);
    c.ret_ptr("fn_80174274", me);
    c.ret_int("gm_WasMatchCanceled", 0, 1);
}

fn scene_rets(c: &mut Case) {
    let kids: std::vec::Vec<*mut HSD_JObj> = (0..4).map(|_| c.jobj()).collect();
    c.out_ptrs("lb_80011E24", 1, &kids);
    c.ret_flt("gm_80168B34", -1.0, 300.0);
    c.ret_flt("gm_MexStockFrame", -1.0, 50.0);
    c.ret_flt("lbGetJObjCurrFrame", 0.0, 60.0);
    c.ret_int("gm_80160854", 0, 255);
    let e = KEY_FRAME.load(core::sync::atomic::Ordering::Relaxed);
    if e < 0 || c.rng.chance(1, 4) {
        c.ret_int("Mex_PortCKindToExt", -2, 40);
    } else {
        c.ret_seq_u64("Mex_PortCKindToExt", &[e as u64; 8]);
    }
    c.ret_int("Mex_InsigniaForExt", -2, 30);
    c.ret_int("Mex_ResultArtIsOwn", 0, 1);
    c.ret_int("Geno_DefineBaseCK", -1, 5);
    c.ret_int("Geno_DefineArtBaseCK", -1, 5);
    c.ret_int("Geno_DefineArtOpen", -1, 4);
    c.ret_int("GxTex_Width", 0, 64);
    c.ret_int("GxTex_Height", 0, 64);
    c.ret_int("GxTex_Format", 0, 10);
    c.ret_int("GxTex_ImageSize", 0, 600);
    c.ret_int("Geno_DefineName", 0, 1);
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.ret_int("mnCharSel_PcArtPortrait", 0, 1);
    c.ret_pool("HSD_MemAlloc", 8192);
    c.ret_pool("HSD_JObjGetDObj", 16);
    c.str_arg("UI_TextI4", 0);
    c.str_arg("lbDvd_8001819C", 0);
    c.str_arg("lbArchive_LoadArchive", 0);
    c.str_arg("HSD_ArchiveGetPublicAddress", 1);
    c.mem_arg("HSD_TlutLoadDesc", 0, 18);
    let maj = c.zobj::<HSD_MatAnimJoint>();
    unsafe { (*maj).matanim = if c.rng.chance(1, 4) { null_mut() } else { c.buf(0x20).cast() } };
    { let z = c.rng.chance(1, 5); c.ret_ptr("gm_MexStockMatAnim", if z { null_mut() } else { maj }); }
    let maj2 = c.zobj::<HSD_MatAnimJoint>();
    unsafe { (*maj2).matanim = if c.rng.chance(1, 4) { null_mut() } else { c.buf(0x20).cast() } };
    { let z = c.rng.chance(1, 5); c.ret_ptr("HSD_ArchiveGetPublicAddress", if z { null_mut() } else { maj2 }); }
    let arch = c.buf(0x20);
    { let z = c.rng.chance(1, 4); c.ret_ptr("lbDvd_8001819C", if z { null_mut() } else { arch }); }
    let a2 = c.buf(0x20);
    { let z = c.rng.chance(1, 4); c.ret_ptr("lbArchive_LoadArchive", if z { null_mut() } else { a2 }); }
    mex_names(c);
    geno_names(c);
    portrait_outs(c);
}

/// `Mex_FighterName` results (a NULL, or short strings) for the name-drawing code.
fn mex_names(c: &mut Case) {
    let mut v = std::vec![0u64];
    for s in ["Meta Knight", "falcon", "A", ""] {
        let p = c.zbuf(64);
        unsafe { core::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len()) };
        v.push(p as usize as u64);
    }
    // a name that is long enough to be cut at 47 characters
    let p = c.zbuf(80);
    unsafe { core::ptr::write_bytes(p, b'q', 60) };
    v.push(p as usize as u64);
    let k = c.rng.range(0, 5) as usize;
    v.rotate_left(k);
    c.ret_seq_u64("Mex_FighterName", &v);
}

/// `Geno_DefineName`'s out buffer: 7 chars and a NUL, whatever the other bytes hold.
fn geno_names(c: &mut Case) {
    let mut words = std::vec![];
    for s in [&b"Geno"[..], b"mr game", b"a b c", b"ZXCV_12", b"", b"7up cola", b"-x-yz"] {
        let mut b = [0u8; 8];
        for (i, ch) in s.iter().take(7).enumerate() {
            b[i] = *ch;
        }
        words.push(u64::from_le_bytes(b));
    }
    c.specs.outv.push(("Geno_DefineName".into(), 1, words));
}

/// `mnCharSel_PcArtPortrait`'s two out pointers: an image and a palette.
fn portrait_outs(c: &mut Case) {
    let mut imgs = std::vec![];
    let mut tluts = std::vec![];
    for _ in 0..4 {
        let w = pick(c, &[8u16, 16, 32]);
        let fmt = pick(c, &[0u32, 5, 8, 9, 10]);
        let img = c.zobj::<HSD_ImageDesc>();
        let tl = c.zobj::<HSD_Tlut>();
        unsafe {
            (*img).image_ptr = if c.rng.chance(1, 6) { null_mut() } else { c.zbuf(64).cast() };
            (*img).width = w;
            (*img).height = w;
            (*img).format = fmt as _;
            (*tl).lut = if c.rng.chance(1, 4) { null_mut() } else { c.zbuf(64).cast() };
            (*tl).fmt = c.rng.range(0, 2) as _;
            (*tl).tlut_name = c.rng.u32();
            (*tl).n_entries = c.rng.range(1, 256) as u16;
        }
        imgs.push(img as usize as u64);
        tluts.push(if c.rng.chance(1, 5) { 0 } else { tl as usize as u64 });
    }
    c.specs.outv.push(("mnCharSel_PcArtPortrait".into(), 2, imgs));
    c.specs.outv.push(("mnCharSel_PcArtPortrait".into(), 3, tluts));
}

// ---- per-function builders ----------------------------------------------------------------

fn b_inline0(c: &mut Case) {
    let j = c.jobj();
    c.arg(j);
    let f = pick(c, &[0.0f32, 1.0, 12.5, 0.1, 100.0]);
    c.arg(f);
}

fn b_fobj_value(c: &mut Case) {
    let buf = c.buf(16);
    let slot = c.zbuf(8) as *mut *mut u8;
    unsafe { *slot = buf };
    c.arg(slot);
    let frac = pick(c, &[0x00u8, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0, 0x01, 0x25, 0x4F, 0x7F, 0x9F, 0x10, 0x3F, 0x1F]);
    c.arg(frac);
}

fn b_packed(c: &mut Case) {
    let buf = c.zbuf(16);
    let n = c.rng.range(1, 6) as usize;
    unsafe {
        for i in 0..n {
            *buf.add(i) = (c.rng.range(0, 255) as u8) | 0x80;
        }
        *buf.add(n) = c.rng.range(0, 0x7F) as u8;
        if c.rng.chance(1, 3) {
            *buf = c.rng.range(0, 0x7F) as u8;
        }
    }
    let slot = c.zbuf(8) as *mut *mut u8;
    unsafe { *slot = buf };
    c.arg(slot);
}

fn b_image_key(c: &mut Case) {
    let t = if c.rng.chance(1, 8) { null_mut() } else { tobj_any(c) };
    c.arg(t);
    let frame = c.rng.range(-1, 25) as i32;
    c.arg(frame);
    let out = c.zbuf(8);
    c.arg(out);
}

fn b_initials(c: &mut Case) {
    let name = c.zbuf(40);
    let n = c.rng.range(0, 14) as usize;
    let set = b"abcXYZ019 -_.  QrstU";
    unsafe {
        for i in 0..n {
            *name.add(i) = set[c.rng.range(0, set.len() as i64 - 1) as usize];
        }
    }
    c.arg(name);
    let out = c.zbuf(8);
    c.arg(out);
}

fn ck_mix(c: &mut Case) -> i32 {
    pick(c, &[0i32, 5, 25, 26, 33, 34, 35, 60, 127, 3])
}

fn b_name_frame(c: &mut Case) {
    scene_rets(c);
    let t = tobj_any(c);
    c.arg(t);
    let ck = ck_mix(c);
    c.arg(ck);
    let fk = c.rng.range(0, 5) as i32;
    c.arg(fk);
}

fn b_draw_name(c: &mut Case) {
    scene_rets(c);
    let t = if c.rng.chance(1, 10) { null_mut() } else { tobj_any(c) };
    if !t.is_null() && c.rng.chance(1, 8) {
        unsafe { (*t).imagedesc = null_mut() };
    }
    c.arg(t);
    let ck = ck_mix(c);
    c.arg(ck);
    let style = c.rng.range(0, 1) as i32;
    c.arg(style);
}

fn b_mex_attach(c: &mut Case) {
    scene_rets(c);
    let j = if c.rng.chance(1, 8) {
        null_mut()
    } else if c.rng.chance(1, 8) {
        let j = c.jobj();
        unsafe { (*j).u.dobj = null_mut() };
        j
    } else {
        { let n = c.rng.range(1, 3) as usize; jobj_dobjs(c, n) }
    };
    if !j.is_null() && c.rng.chance(1, 8) {
        unsafe { (*j).u.dobj = null_mut() };
    }
    c.arg(j);
    let idx = c.rng.range(0, 4) as i32;
    c.arg(idx);
    let stock = c.rng.range(0, 1) as i32;
    c.arg(stock);
}

fn b_tobj_at(c: &mut Case) {
    let j = if c.rng.chance(1, 8) { null_mut() } else { { let n = c.rng.range(0, 3) as usize; jobj_dobjs(c, n) } };
    c.arg(j);
    let idx = c.rng.range(0, 4) as i32;
    c.arg(idx);
}

fn b_set_image(c: &mut Case) {
    scene_rets(c);
    let t = if c.rng.chance(1, 8) { null_mut() } else { tobj_any(c) };
    c.arg(t);
    let d = if c.rng.chance(1, 8) { null_mut() } else { c.zobj::<HSD_ImageDesc>() };
    c.arg(d);
}

fn b_apply_art(c: &mut Case) {
    scene_rets(c);
    let t = if c.rng.chance(1, 8) { null_mut() } else { tobj_any(c) };
    c.arg(t);
    let ck = c.rng.range(30, 90) as i32;
    c.arg(ck);
    let what = pick(c, &[1i32, 2, 3, 4]);
    c.arg(what);
    let costume = c.rng.range(0, 5) as i32;
    c.arg(costume);
}

fn b_letters(c: &mut Case) {
    scene_rets(c);
    let t = if c.rng.chance(1, 8) { null_mut() } else { tobj_any(c) };
    c.arg(t);
    let ck = c.rng.range(30, 90) as i32;
    c.arg(ck);
}

fn b_art_state(c: &mut Case) {
    scene_rets(c);
    unsafe {
        tu::gmRst_GenoChrArt = if c.rng.chance(1, 2) { null_mut() } else { c.buf(16).cast() };
        tu::gmRst_GenoChrTried = pick(c, &[0, 1]);
    }
}

fn b_portrait(c: &mut Case) {
    b_art_state(c);
    let t = if c.rng.chance(1, 8) { null_mut() } else { tobj_any(c) };
    c.arg(t);
    let base = c.rng.range(-1, 25) as i32;
    c.arg(base);
    let costume = c.rng.range(0, 5) as i32;
    c.arg(costume);
}

fn b_geno_card(c: &mut Case) {
    scene_rets(c);
    unsafe {
        tu::gmRst_GenoChrArt = if c.rng.chance(1, 2) { null_mut() } else { c.buf(16).cast() };
        tu::gmRst_GenoChrTried = pick(c, &[0, 1]);
    }
    let j = { let n = c.rng.range(1, 3) as usize; jobj_dobjs(c, n) };
    c.arg(j);
    let ck = c.rng.range(30, 90) as i32;
    c.arg(ck);
    let costume = c.rng.range(0, 5) as i32;
    c.arg(costume);
}

fn b_scene(c: &mut Case) {
    b_me_rd(c);
    scene_rets(c);
    c.ret_pool("fn_801748EC", 16);
    c.ret_int("fn_80174A60", 0, 30);
    c.ret_pool("fn_80174920", 8);
}

fn b_gobj_jobj(c: &mut Case) {
    b_scene(c);
    let j = jobj_dobjs(c, 2);
    let g = c.gobj(j.cast());
    c.arg(g);
}

fn b_slot(c: &mut Case) {
    b_scene(c);
    let s = c.rng.range(0, 3) as i32;
    c.arg(s);
}

fn b_slot_amount(c: &mut Case) {
    b_slot(c);
    let a = pick(c, &[0.0f32, 0.1, 0.5, 1.0, 2.5, 40.0]);
    c.arg(a);
}

fn b_noargs(c: &mut Case) {
    b_scene(c);
}

fn b_arg0(c: &mut Case) {
    b_scene(c);
    let a = c.rng.range(0, 5) as i32;
    c.arg(a);
}

fn b_init_players(c: &mut Case) {
    b_scene(c);
    let me = rd().x94;
    c.arg(rd() as *mut ResultsData);
    c.arg(me);
    let i = c.zbuf(8);
    c.arg(i);
    let k = c.zbuf(8);
    c.arg(k);
}

fn b_update(c: &mut Case) {
    b_scene(c);
    let me = rd().x94;
    let j = jobj_dobjs(c, 2);
    let g = c.gobj(j.cast());
    c.arg(rd() as *mut ResultsData);
    c.arg(me);
    c.arg(g);
}

fn b_frame(c: &mut Case) {
    b_gobj_jobj(c);
    c.out("lb_8000B1CC", 2, 12);
}

/// `lbl_8046E3AC.match_end` for the two placing helpers.
fn b_state_placing(c: &mut Case) {
    unsafe {
        let st = addr_of_mut!(tu::lbl_8046E3AC);
        core::ptr::write_bytes(st, 0, 1);
        c.bitfields::<lbl_8046E3AC_t>(st, offset_of!(lbl_8046E3AC_t, bf_0), 0, 1);
        (*st).bf_0[0] = c.rng.range(0, 255) as u8;
        fill_me(c, addr_of_mut!((*st).match_end));
    }
}

fn b_placing_arg(c: &mut Case) {
    b_state_placing(c);
    let a = c.rng.range(-1, 4) as i32;
    c.arg(a);
}

#[test]
fn gmresultplayer_vs_c() {
    let funcs = [
        fd!("inline0", "pf", "v", 8, z!(b_inline0), |a| { tu::inline0(p(a, 0), af32(a, 1)); 0 }),
        fd!("genoArt_FObjValue", "pi", "f", 60, z!(b_fobj_value), |a| {
            r(tu::genoArt_FObjValue(p(a, 0), au8(a, 1)))
        }),
        fd!("genoArt_FObjPacked", "p", "i32", 24, z!(b_packed), |a| { r(tu::genoArt_FObjPacked(p(a, 0))) }),
        fd!("genoArt_ImageKeyAt", "pip", "b", 200, z!(b_image_key), |a| {
            r(tu::genoArt_ImageKeyAt(p(a, 0), ai32(a, 1), p(a, 2)))
        }),
        fd!("genoArt_Initials", "pp", "v", 60, z!(b_initials), |a| { tu::genoArt_Initials(p(a, 0), p(a, 1)); 0 }),
        fd!("gmRst_NameFrame", "pii", "i32", 80, z!(b_name_frame), |a| { r(tu::gmRst_NameFrame(p(a, 0), ai32(a, 1), ai32(a, 2))) }),
        fd!("gmRst_DrawName", "pii", "v", 120, z!(b_draw_name), |a| { tu::gmRst_DrawName(p(a, 0), ai32(a, 1), ai32(a, 2)); 0 }),
        fd!("gmRst_MexAttach", "pii", "b", 60, z!(b_mex_attach), |a| { r(tu::gmRst_MexAttach(p(a, 0), ai32(a, 1), ab(a, 2))) }),
        fd!("gmRst_GenoTObjAt", "pi", "p", 24, z!(b_tobj_at), |a| { r(tu::gmRst_GenoTObjAt(p(a, 0), ai32(a, 1))) }),
        fd!("gmRst_GenoSetImage", "pp", "b", 40, z!(b_set_image), |a| { r(tu::gmRst_GenoSetImage(p(a, 0), p(a, 1))) }),
        fd!("gmRst_GenoApplyArt", "piii", "b", 120, z!(b_apply_art), |a| {
            r(tu::gmRst_GenoApplyArt(p(a, 0), ai32(a, 1), ai32(a, 2), ai32(a, 3)))
        }),
        fd!("gmRst_GenoLetters", "pi", "b", 80, z!(b_letters), |a| { r(tu::gmRst_GenoLetters(p(a, 0), ai32(a, 1))) }),
        fd!("gmRst_GenoArtReset", "", "v", 4, z!(b_art_state), |a| { tu::gmRst_GenoArtReset(); 0 }),
        fd!("gmRst_GenoArtClose", "", "v", 8, z!(b_art_state), |a| { tu::gmRst_GenoArtClose(); 0 }),
        fd!("gmRst_GenoRetailPortrait", "pii", "b", 120, z!(b_portrait), |a| {
            r(tu::gmRst_GenoRetailPortrait(p(a, 0), ai32(a, 1), ai32(a, 2)))
        }),
        fd!("gmRst_GenoCard", "pii", "b", 120, z!(b_geno_card), |a| { r(tu::gmRst_GenoCard(p(a, 0), ai32(a, 1), ai32(a, 2))) }),
        fd!("fn_80177748", "", "v", 120, z!(b_noargs), |a| { tu::fn_80177748(); 0 }),
        fd!("fn_80177920", "p", "v", 60, z!(b_gobj_jobj), |a| { tu::fn_80177920(p(a, 0)); 0 }),
        fd!("pagePrev", "i", "b", 24, z!(b_slot), |a| { r(tu::pagePrev(ai32(a, 0))) }),
        fd!("pageNext", "i", "b", 24, z!(b_slot), |a| { r(tu::pageNext(ai32(a, 0))) }),
        fd!("scrollDown", "if", "b", 60, z!(b_slot_amount), |a| { r(tu::scrollDown(ai32(a, 0), af32(a, 1))) }),
        fd!("scrollUp", "if", "b", 40, z!(b_slot_amount), |a| { r(tu::scrollUp(ai32(a, 0), af32(a, 1))) }),
        fd!("fn_80177B7C", "i", "b", 120, z!(b_slot), |a| { r(tu::fn_80177B7C(ai32(a, 0))) }),
        fd!("fn_80177DD0", "i", "b", 160, z!(b_slot), |a| { r(tu::fn_80177DD0(ai32(a, 0))) }),
        fd!("fn_80178050", "p", "v", 200, z!(b_gobj_jobj), |a| { tu::fn_80178050(p(a, 0)); 0 }),
        fd!("fn_801785B0", "p", "v", 24, z!(b_gobj_jobj), |a| { tu::fn_801785B0(p(a, 0)); 0 }),
        fd!("fn_80178BB4_init_players", "pppp", "v", 160, z!(b_init_players), |a| {
            tu::fn_80178BB4_init_players(p(a, 0), p(a, 1), p(a, 2), p(a, 3));
            0
        }),
        fd!("fn_80178BB4", "p", "v", 160, z!(b_gobj_jobj), |a| { tu::fn_80178BB4(p(a, 0)); 0 }),
        fd!("fn_801791E4", "", "b", 60, z!(b_noargs), |a| { r(tu::fn_801791E4()) }),
        fd!("fn_80179350_inline", "", "i32", 12, z!(b_noargs), |a| { r(tu::fn_80179350_inline()) }),
        fd!("fn_80179350_update", "ppp", "v", 200, z!(b_update), |a| { tu::fn_80179350_update(p(a, 0), p(a, 1), p(a, 2)); 0 }),
        fd!("fn_80179350", "p", "v", 160, z!(b_frame), |a| { tu::fn_80179350(p(a, 0)); 0 }),
        fd!("fn_801795D4", "", "i32", 60, z!(b_state_placing), |a| { r(tu::fn_801795D4()) }),
        fd!("fn_801796F0", "i", "i32", 60, z!(b_placing_arg), |a| { r(tu::fn_801796F0(ai32(a, 0))) }),
    ];
    let mut globals = tu::oracle_globals();
    globals.extend(tu::oracle_globals_ext());
    // HSD_PadCopyStatus sat after gmRst_GenoChrTried in this TU's table when the C data was
    // generated (now melee_runtime's single global, listed in the ext table): keep that layout.
    if let Some(i) = globals.iter().position(|g| g.name == "HSD_PadCopyStatus") {
        let g = globals.remove(i);
        let at = globals.iter().position(|g| g.name == "gmRst_GenoChrTried").unwrap() + 1;
        globals.insert(at, g);
    }
    let mut fn_table: std::vec::Vec<(&str, usize)> = std::vec![];
    macro_rules! reg {
        ($($n:ident),*) => { $( fn_table.push((stringify!($n), ext::$n as usize)); )* };
    }
    reg!(
        HSD_AObjSetRate, HSD_AObjSetCurrentFrame, fn_80179D3C, fn_80179D60, fn_80179D84, fn_80179DA8, fn_80179DCC, fn_80179E34,
        fn_80179E9C, fn_80179F04
    );
    run_tu(&TuOracle { tu: "gmresultplayer", funcs: &funcs, globals, fn_table });
}

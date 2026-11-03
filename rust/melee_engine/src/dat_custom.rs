//! Archive roots that are tables of differently typed pointers (`void**` in the C, split by index at the call
//! site). The generated `dat_types.rs` hands a `custom:<name>` symbol here. Seams only: this decodes the file, the
//! game code that reads the result is the tu port.
//!
//! An object whose size the bytes do not state (a float table, a record array) is decoded up to the next
//! offset that something points at (or the next public, or the end of the data block): `Bounds::extent`.

use core::ffi::c_void;
use melee_core::gcdecode::{GcDecode, GcDecoder, GcPointee};
use melee_types::{
    FighterPartsTable, Fighter_804D64FC_t, Fighter_804D651C_t, Fighter_804D6518_t, Fighter_804D6520_t, Fighter_804D6524_t,
    Fighter_804D653C_t, Fighter_804D6540_t, Fighter_ShakeTable_t, HSD_Joint, Vec2, ftCommonData, Article, ItemCommonData,
    it_804D6D20_t, it_804D6D40_t,
};

/// Sorted data offsets where something starts: every relocation target, every public, the end of the data.
pub struct Bounds(pub Vec<u32>, pub usize);

impl Bounds {
    /// Native address of data offset `off` in the RAW (big-endian, relocated) image of the archive.
    pub fn raw_addr(&self, off: u32) -> *mut c_void {
        (self.1 + off as usize) as *mut c_void
    }
    /// Bytes from `off` to the next boundary above it (at least 1).
    pub fn extent(&self, off: u32) -> u32 {
        match self.0.binary_search(&off) {
            Ok(i) => self.0.get(i + 1).map_or(1, |n| n - off),
            Err(i) => self.0.get(i).map_or(1, |n| n - off),
        }
        .max(1)
    }
}

/// `T[]` at `off`, sized by the next boundary.
fn arr<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut T {
    let n = (b.extent(off) / T::GC_BYTES).max(1) as usize;
    d.array::<T>(off, n)
}

/// The `T*` stored in the pointer word at `at`, as a `T[]` sized by the next boundary (null if not a pointer).
fn parr<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut c_void {
    match d.target(at) {
        Some(t) => arr::<T>(d, b, t) as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

/// The table of pointers at `at` (`T**`): a leaked `[*mut T]`, each entry a `T[]` sized by the next boundary.
fn ptable<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut c_void {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut c_void> = (0..n).map(|i| parr::<T>(d, b, t + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// Pointer words that are themselves pointees (no array): `T*` where `T: GcPointee`.
fn one<T: GcPointee>(d: &mut GcDecoder<'_>, at: u32) -> *mut c_void {
    d.ptr::<T>(at) as *mut c_void
}

/// Decode custom root `name` at data offset `off`; the result is the native address of the root.
pub fn decode(name: &str, d: &mut GcDecoder<'_>, off: u32, _extent: u32, b: &Bounds, sym: &str) -> *mut c_void {
    match name {
        "plco" => plco(d, b, off),
        "itco" => itco(d, b, off),
        "ftdata" => ftdata(d, b, off, sym),
        "kirbyhat" => kirbyhat(d, b, off, sym),
        "ptrtable_itemdata" => {
            // the stage's own items (`GroundItemData { kind, Article* }`): their state arrays can be longer than 8 as well
            let n = (b.extent(off) / 4).max(1);
            let mut arts: Vec<*mut melee_types::Article> = Vec::new();
            for i in 0..n {
                if let Some(rec) = d.target(off + 4 * i) {
                    if let Some(ta) = d.target(rec + 4) {
                        note_article_states(d, b, ta);
                        let art = d.object::<melee_types::Article>(ta);
                        if d.u32_at(rec) == melee_types::It_Kind_Heiho as u32 {
                            // Shy Guy: `attr[0]` is a relocated pointer to its health `s32` (`it_802D8EC8` reads `**attr`)
                            if let Some(sa) = d.target(ta + 4) {
                                if let Some(pt) = d.target(sa) {
                                    unsafe { (*core::ptr::addr_of_mut!(HEIHO_HEALTH)).push((art, b.1, pt)) };
                                }
                            }
                        }
                        // the Adventure stages' enemies (see zako_attrs)
                        let kind = d.u32_at(rec);
                        zako_attrs(d, b, kind, ta);
                        if d.u32_at(rec) == melee_types::It_Kind_Kyasarin as u32 {
                            // Kyasarin: `itKyasarinAttributes` starts with a pointer (`x0`, unused), so the native record is 4 bytes wider than the
                            // word-swapped block and every `attr->xN` read natively would land one word late (the walk speed read the next field)
                            if let Some(sa) = d.target(ta + 4) {
                                let typed = d.object::<melee_types::itKyasarinAttributes>(sa) as *mut c_void;
                                unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
                            }
                        }
                        if d.u32_at(rec) == melee_types::It_Kind_Kyasarin_Egg as u32 {
                            // Birdo's egg: `itKyasarinEggAttributes { s32* x0; f32 x4, x8; s32 xC, x10; }` has the same leading pointer, so
                            // word-swapped `ap->x4` (the roll speed, it_802EFB0C) read the bounce `x8` natively (stage_20_mushroomkingdom2 1430)
                            if let Some(sa) = d.target(ta + 4) {
                                let typed = d.object::<melee_types::itKyasarinEggAttributes>(sa) as *mut c_void;
                                unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
                            }
                        }
                        if d.u32_at(rec) == melee_types::It_Kind_Arwing_Laser as u32 {
                            // Arwing laser (Corneria, Venom): `ArwingLaserAttr { ItemAttr* x0; f32 x4, x8; }` starts with a relocated pointer
                            // (`itArwinglaser_UnkMotion2_Phys` reads `attrs->x0->x4_throw_speed_mul`); the word swap leaves it a big-endian
                            // image offset and puts x4 / x8 one word early in the 8-byte-pointer native record
                            if let Some(sa) = d.target(ta + 4) {
                                let rec = melee_items::tu::itarwinglaser::ArwingLaserAttr {
                                    x0: d.ptr::<melee_types::ItemAttr>(sa),
                                    x4: f32::from_bits(d.u32_at(sa + 4)),
                                    x8: f32::from_bits(d.u32_at(sa + 8)),
                                };
                                let typed = Box::leak(Box::new(rec)) as *mut _ as *mut c_void;
                                unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
                            }
                        }
                        arts.push(art);
                    }
                }
            }
            // the articles' special attributes are word-swapped like the common items' (`fix_articles`)
            if !arts.is_empty() {
                let cnt = arts.len();
                let tbl = Box::leak(arts.into_boxed_slice()).as_mut_ptr();
                unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TABLES)).push((tbl as usize, cnt, b.1, b.0.clone())) };
            }
            ptrtable::<melee_types::GroundItemData>(d, b, off)
        }
        "ptrtable_dynmodel" => ptrtable::<melee_types::DynamicModelDesc>(d, b, off),
        "ptrtable_lightlist" => ptrtable::<melee_types::LightList>(d, b, off),
        "rawptrs" => {
            // a null terminated table of pointers to raw (big-endian) data, kept as native pointers into the raw image
            let n = (b.extent(off) / 4).max(1);
            let mut v: Vec<*mut c_void> = Vec::new();
            for i in 0..n {
                match d.target(off + 4 * i) {
                    Some(t) => v.push(b.raw_addr(t)),
                    None => {
                        v.push(core::ptr::null_mut());
                        // slot 0 is unused (`ALDYakuAll`: the C walks from 1 to the first null)
                        if i > 0 {
                            break;
                        }
                    }
                }
            }
            Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
        }
        "rawhalves" => {
            // a block of s16 the game code reads natively (toy.c `s16* _Toy_sbss_804D6EB4`, tyExpDifferentTbl): halfword-swapped copy
            let n = (b.extent(off) / 2).max(1) as usize;
            let buf = native_buf(n * 2 + 2);
            for k in 0..n {
                let w = d.u32_at(off + (2 * k as u32 & !3));
                let h = if k % 2 == 0 { (w >> 16) as u16 } else { w as u16 };
                unsafe { (buf as *mut u16).add(k).write(h) };
            }
            buf as *mut c_void
        }
        "rawwords" => {
            // a block of 32-bit ints / floats the game code reads as a native struct (`yakumono_param`): word-swapped copy
            let n = (b.extent(off) / 4).max(1) as usize;
            if n == 2 && d.target(off).is_some() && d.target(off + 4).is_some() {
                // `grBattle_YakumonoParam { void* bg_curr_color_overlay; void* bg_prev_color_overlay; }`: two relocated pointers to
                // colour-overlay command streams (big-endian bytes the lb colour-animation code reads), widened to raw-image addresses
                let (a, c) = (d.target(off).unwrap(), d.target(off + 4).unwrap());
                let buf = native_buf(16) as *mut usize;
                unsafe {
                    buf.write(b.raw_addr(a) as usize);
                    buf.add(1).write(b.raw_addr(c) as usize);
                }
                return buf as *mut c_void;
            }
            let buf = native_buf(n * 4);
            for k in 0..n {
                let w = d.u32_at(off + 4 * k as u32);
                unsafe { (buf as *mut u32).add(k).write(w) };
            }
            // a stage whose record packs s16 / u8 fields (`crate::yakumono_layouts`, generated from the stage TUs' structs): swap
            // each field by its own width instead of the whole word (a word swap exchanges the two halves of an s16 pair)
            let kind = unsafe { (*core::ptr::addr_of!(melee_world::tu::ground::stage_info)).grkind } as i32;
            if let Some((_, _, csize, rsize, fields)) = crate::yakumono_layouts::YAKUMONO_LAYOUTS.iter().find(|l| l.0 == kind) {
                if (*csize as usize) <= n * 4 {
                    // the native struct differs from the GameCube record when it holds pointers (8 bytes against 4): build it
                    // field by field at its own offsets
                    let out = if rsize != csize { native_buf((*rsize as usize).max(n * 4)) } else { buf };
                    for &(o, r, w, k) in fields.iter() {
                        if k == 0 {
                            for i in 0..w as u32 {
                                let byte = (d.u32_at(off + (o & !3)) >> (8 * (3 - ((o & 3) + i)))) as u8;
                                unsafe { *out.add((r + (w as u32 - 1 - i)) as usize) = byte };
                            }
                        } else {
                            let p: *mut c_void = match d.target(off + o) {
                                None => core::ptr::null_mut(),
                                Some(t) => match k {
                                    // the stages' `DynamicsDesc*` (Mute City, Poke Floats, the Mewtwo / Jigglypuff targets) are the
                                    // `on_touch_line` hit descriptions every consumer reads as `lbColl_80008D30_arg1` (ftCo_800C0B20's cast,
                                    // `->count` is its word 1): the native `DynamicsDesc` starts with an 8-byte pointer, so it is decoded as that
                                    // all-word record, each field at its GameCube offset
                                    2 => d.object::<melee_types::lbColl_80008D30_arg1>(t) as *mut c_void,
                                    4 => d.object::<melee_types::lbColl_80008D30_arg1>(t) as *mut c_void,
                                    3 => d.array::<i16>(t, (b.extent(t) / 2).max(1) as usize) as *mut c_void,
                                    _ => b.raw_addr(t),
                                },
                            };
                            unsafe { (out.add(r as usize) as *mut usize).write(p as usize) };
                        }
                    }
                    return out as *mut c_void;
                }
            }
            buf as *mut c_void
        }
        "sis" => sis_table(d, b, off),
        "efdat" => efdat(d, b, off),
        "ptclcmd" => ptcl_cmd(d, b, off),
        "ptcltex" => ptcl_tex(d, b, off),
        "lbrefdata" => {
            // lbrefract.c `static struct { u8 x0; f32* x4; }* refract_data`
            let r = melee_runtime::tu::lbrefract::RefractData { x0: d.u8_at(off), x4: parr::<f32>(d, b, off + 4) as *mut f32 };
            Box::leak(Box::new(r)) as *mut _ as *mut c_void
        }
        _ => panic!("dat_custom: no decoder for `{name}`"),
    }
}

/// `ftLoadCommonData` (PlCo.dat): the 23 pointers `Fighter_LoadCommonData` distributes to its globals.
fn plco(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let at = |i: u32| off + 4 * i;
    let mut out: Vec<*mut c_void> = vec![core::ptr::null_mut(); 23];
    out[0] = parr::<ftCommonData>(d, b, at(0));
    // Fighter_804D6550: declared `int**`, read as a flat table of 12-byte float rows (ftCo_ItemThrow, base - 0x468): native words, not a pointer table
    out[1] = swapped_words(d, b, at(1), 1);
    out[2] = parr::<f32>(d, b, at(2)); // float (*)[5]
    out[3] = parr::<f32>(d, b, at(3)); // float*
    out[4] = ptable::<FighterPartsTable>(d, b, at(4));
    out[5] = ptable::<Fighter_804D6540_t>(d, b, at(5));
    out[6] = parr::<Fighter_804D653C_t>(d, b, at(6));
    out[7] = parr::<Fighter_804D653C_t>(d, b, at(7));
    // [17] UNK_T: no type in the C either; read as null
    out[9] = shake_vec2_table(d, b, at(9)); // Vec2**: { Vec2[], frame count } x 3 (Fighter_804D6530)
    out[10] = parr::<Fighter_ShakeTable_t>(d, b, at(10));
    out[11] = parr::<Fighter_ShakeTable_t>(d, b, at(11));
    out[12] = parr::<Fighter_804D6524_t>(d, b, at(12));
    out[13] = parr::<Fighter_804D6520_t>(d, b, at(13));
    out[14] = parr::<Fighter_804D651C_t>(d, b, at(14));
    out[15] = parr::<Fighter_804D6518_t>(d, b, at(15));
    out[16] = one::<HSD_Joint>(d, at(16));
    out[18] = parr::<u8>(d, b, at(18));
    out[19] = parr::<u8>(d, b, at(19));
    out[20] = one::<HSD_Joint>(d, at(20));
    out[21] = one::<melee_types::CrowdConfig>(d, at(21)); // gCrowdConfig
    {
        // Fighter_804D6534: { respawn platform HSD_Joint*, its HSD_AnimJoint* } (ftCo_800D4FF4 reads tbl[0], tbl[1])
        if let Some(t) = d.target(at(8)) {
            let j = d.ptr::<HSD_Joint>(t) as *mut c_void;
            let a = d.ptr::<melee_types::HSD_AnimJoint>(t + 4) as *mut c_void;
            out[8] = Box::leak(vec![j, a].into_boxed_slice()).as_mut_ptr() as *mut c_void;
        }
    }
    out[22] = {
        // x4..x1C are kind-indexed `T**` tables (Fighter_LoadCommonData indexes them by kind): the generated decode of a
        // `T**` field would give one entry, and ftCommonData_ExtendCpuTables reads up to Ft_Kind_Mex0 of them.
        let p = parr::<Fighter_804D64FC_t>(d, b, at(22)) as *mut Fighter_804D64FC_t;
        if let Some(t) = d.target(at(22)) {
            // the lists are `ftCo_AttackEntry` arrays (nine 32-bit words each, 0-terminated by `cmd`): word-swapped native copies
            let tables = [4u32, 8, 0xC, 0x10, 0x14, 0x18, 0x1C].map(|f| attack_lists(d, b, t + f, melee_types::Ft_Kind_Mex0 as u32));
            unsafe {
                // x20: `f32[kind]` (a hurt-distance threshold per fighter); x24: the six weapon reaches
                // cmdscripts: `u8*` per script id, byte streams read bytewise (kept in the raw image)
                (*p).cmdscripts = raw_table(d, b, t) as *mut *mut u8;
                (*p).x20 = swapped_words(d, b, t + 0x20, melee_types::Ft_Kind_Mex0 as usize) as *mut f32;
                (*p).x24 = swapped_words(d, b, t + 0x24, 8) as *mut c_void;
                (*p).x4 = tables[0];
                (*p).x8 = tables[1];
                (*p).xC = tables[2];
                (*p).x10 = tables[3];
                (*p).x14 = tables[4];
                (*p).x18 = tables[5];
                (*p).x1C = tables[6];
            }
        }
        p as *mut c_void
    };
    // the root is a pointer to the table (`void** pData` after `lbArchive_LoadSymbols`)
    Box::leak(out.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// `Fighter_804D6530` (`Vec2**` in the C): pairs of { `Vec2[]` shake offsets, the frame count as a plain integer in the pointer slot }.
fn shake_vec2_table(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut c_void {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = ((b.extent(t) / 4).max(1)) as u32;
    let v: Vec<*mut c_void> = (0..n)
        .map(|i| {
            if i % 2 == 0 {
                parr::<Vec2>(d, b, t + 4 * i)
            } else {
                d.u32_at(t + 4 * i) as usize as *mut c_void
            }
        })
        .collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// The block the pointer word at `at` targets as native 32-bit words (`min` words at least, to the next boundary), shared per target.
fn swapped_words(d: &mut GcDecoder<'_>, b: &Bounds, at: u32, min: usize) -> *mut c_void {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    memo((b.1, t | 0x8000_0000), || {
        let n = ((b.extent(t) / 4) as usize).max(min);
        let buf = native_buf(n * 4);
        for k in 0..n {
            let w = d.u32_at(t + 4 * k as u32);
            unsafe { (buf as *mut u32).add(k).write(w) };
        }
        buf as *mut c_void
    })
}

/// The table of pointers at `at`, each a pointer into the raw (big-endian) image (null where the word is not a pointer).
fn raw_table(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut c_void {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut c_void> = (0..n).map(|i| d.target(t + 4 * i).map_or(core::ptr::null_mut(), |q| b.raw_addr(q))).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// A kind-indexed table (`void**`) of 0-terminated `ftCo_AttackEntry` lists, as word-swapped native copies. `min` entries at least.
fn attack_lists(d: &mut GcDecoder<'_>, b: &Bounds, at: u32, min: u32) -> *mut *mut c_void {
    const WORDS: u32 = 9;
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut c_void> = (0..n.max(min))
        .map(|i| {
            if i >= n {
                return core::ptr::null_mut();
            }
            let Some(te) = d.target(t + 4 * i) else { return core::ptr::null_mut() };
            memo((b.1, te | 0x4000_0000), || {
                let mut words: Vec<u32> = Vec::new();
                for e in 0..256u32 {
                    let at_e = te + e * WORDS * 4;
                    let ent: Vec<u32> = (0..WORDS).map(|k| d.u32_at(at_e + 4 * k)).collect();
                    let end = ent[0] == 0;
                    words.extend(ent);
                    if end {
                        break;
                    }
                }
                let buf = native_buf(words.len() * 4);
                for (k, w) in words.iter().enumerate() {
                    unsafe { (buf as *mut u32).add(k).write(*w) };
                }
                buf as *mut c_void
            })
        })
        .collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// A table of `T*` (`T**`), one record per entry (null entry = null pointer), sized by the next boundary.
fn rtable<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut T {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut T> = (0..n).map(|i| d.ptr::<T>(t + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}


/// Article tables whose `x4_specialAttributes` blocks still point into the big-endian image: (table, entries, image
/// base, boundaries). Filled while decoding (an object's fields exist only after the decoder ran), applied by
/// [`fix_articles`] right after `finish()`.
static mut ARTICLE_TABLES: Vec<(usize, usize, usize, Vec<u32>)> = Vec::new();

/// Articles whose `xC_itemStates` runs past the 8 entries of `ItemStateArray`: (table, entry, state array of the full
/// extent). The C indexes the array by item state (`Ground_801C0800` sets the Random item's `ald_yaku_all` scripts
/// for every state), so the native copy must hold them all.
static mut ARTICLE_STATES: Vec<(*mut melee_types::Article, *mut melee_types::ItemStateDesc, usize)> = Vec::new();

/// Articles with an `x14_dynamics` block: (article, image base, data offset of the block). The header types it `ItemDynamics
/// {count, descs}` but the C (`it_8027163C`) reads `ItCollDynamics {pad[8], count, descs}`: rebuilt from the image by [`fix_articles`].
static mut ARTICLE_DYNAMICS: Vec<(*mut melee_types::Article, usize, u32)> = Vec::new();

/// Articles with an `x8_hurtbones` list `{count, descs}`: the decoder gives `descs` room for one `ItHurtBoneDesc`, the C
/// indexes it up to `count` (Klap Klap has 2). Rebuilt from the image by [`fix_articles`]: (article, image base, data offset).
static mut ARTICLE_HURTBONES: Vec<(*mut melee_types::Article, usize, u32)> = Vec::new();

/// Queue the full-extent state array of the article at data offset `ta` when its states run past 8 entries.
fn note_article_states(d: &mut GcDecoder<'_>, b: &Bounds, ta: u32) {
    if let Some(hb) = d.target(ta + 0x8) {
        let art = d.object::<melee_types::Article>(ta);
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_HURTBONES)).push((art, b.1, hb)) };
    }
    if let Some(dy) = d.target(ta + 0x14) {
        let art = d.object::<melee_types::Article>(ta);
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_DYNAMICS)).push((art, b.1, dy)) };
    }
    let Some(ts) = d.target(ta + 0xC) else { return };
    let cnt = (b.extent(ts) / 0x10) as usize;
    if cnt >= 8 && cnt <= 32 {
        let art = d.object::<melee_types::Article>(ta);
        let arr = d.ptr_array::<melee_types::ItemStateDesc>(ta + 0xC, cnt);
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_STATES)).push((art, arr, cnt)) };
    }
}

/// Common-item articles whose special attributes hold `HSD_Joint*` fields the word swap cannot relocate (the Foods rows
/// `{s32, HSD_Joint*, s32, s32}`): (raw block address, the decoded joint of every row). Patched by [`fix_articles`].
static mut ARTICLE_JOINT_ROWS: Vec<(usize, Vec<melee_types::itFoodsAttributes>)> = Vec::new();

/// `ItemKind` of Foods (the index of its article in the common-item table).
const KIND_FOODS: u32 = 18;
/// `ItemKind` of the Super Mushroom: its attribute block is `KinokoAnim { HSD_AnimJoint* joint }` entries (`it_80293660` reads `[idx + 2]`).
const KIND_KINOKO: u32 = 26;
/// `ItemKind` of the Warp Star (`It_Kind_WStar`): its attribute block ends in pointer-bearing `itWstarAttrEntry` rows.
const KIND_WSTAR: u32 = melee_types::It_Kind_WStar as u32;

/// `rtable::<Article>`; see [`fix_articles`].
fn rtable_articles(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut melee_types::Article {
    let tbl = rtable::<melee_types::Article>(d, b, at);
    if let Some(t) = d.target(at) {
        let n = (b.extent(t) / 4).max(1) as usize;
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TABLES)).push((tbl as usize, n, b.1, b.0.clone())) };
        for i in 0..n as u32 {
            if let Some(ta) = d.target(t + 4 * i) {
                note_article_states(d, b, ta);
                if i == KIND_KINOKO && n > KIND_KINOKO as usize {
                    // the animation roots are 32-bit relocated pointers in the block; the native `KinokoAnim` is 8 bytes wide, so the decoded
                    // roots live in a side table (itkinoko.rs `KINOKO_JOINTS`) and the C's `attrs[idx + 2].joint` reads it
                    if let Some(sa) = d.target(ta + 4) {
                        let words = (b.extent(sa) / 4) as u32;
                        for k in 0..words.min(32) {
                            if d.target(sa + 4 * k).is_some() {
                                unsafe { melee_items::tu::itkinoko::KINOKO_JOINTS[k as usize] = d.ptr::<melee_types::HSD_AnimJoint>(sa + 4 * k) };
                            }
                        }
                    }
                }
                // Unown (`itUnknownAttributes`: nine scalar words, then 26 `HSD_Joint*` letter models): the pointer array does not fit the word-swapped
                // 4-byte slots (`it_802CE710` passes `attr->x24[HSD_Randi(26)]` to `HSD_JObjLoadJoint`), so the block is decoded as its record
                if let Some(sa) = d.target(ta + 4) {
                    if b.extent(sa) >= 0x8C && (9..35u32).all(|w| d.is_reloc(sa + 4 * w)) {
                        let art = d.object::<melee_types::Article>(ta);
                        let typed = d.object::<melee_types::itUnknownAttributes>(sa) as *mut c_void;
                        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
                    }
                }
                if i == KIND_WSTAR && n > KIND_WSTAR as usize {
                    // Warp Star (`itWstarAttributes`: nine f32, `s32 x24_count`, then `itWstarAttrEntry { HSD_AnimJoint* x0_anim_joint;
                    // enum_t x4_sfx; }[x24_count]` at GC +0x28): the entries are 8 bytes on the GameCube and 16 natively, and the word swap
                    // leaves each anim joint a big-endian image word (`ftCo_800C4724` hands `it_80294364`'s pick to `HSD_JObjAddAnim`).
                    // The block is rebuilt as the native record: the header words swapped at their GC offsets, the entries decoded.
                    if let Some(sa) = d.target(ta + 4) {
                        let count = d.u32_at(sa + 0x24);
                        if (1..=7).contains(&count) && (0..count).all(|k| d.is_reloc(sa + 0x28 + 8 * k)) {
                            use melee_types::{itWstarAttrEntry, itWstarAttributes};
                            let words = core::mem::size_of::<itWstarAttributes>() / 8 + (count as usize - 1) * core::mem::size_of::<itWstarAttrEntry>() / 8;
                            let buf: &mut [u64] = Box::leak(std::vec![0u64; words].into_boxed_slice());
                            let rec = buf.as_mut_ptr() as *mut itWstarAttributes;
                            unsafe {
                                let w = rec as *mut u32;
                                for k in 0..10u32 {
                                    *w.add(k as usize) = d.u32_at(sa + 4 * k);
                                }
                                let es = &raw mut (*rec).x28_entries as *mut itWstarAttrEntry;
                                for k in 0..count {
                                    *es.add(k as usize) = <itWstarAttrEntry as GcDecode>::gc_decode(d, sa + 0x28 + 8 * k);
                                }
                            }
                            let art = d.object::<melee_types::Article>(ta);
                            unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, rec as *mut c_void)) };
                        }
                    }
                }
                if i == KIND_FOODS && n > KIND_FOODS as usize {
                    if let Some(sa) = d.target(ta + 4) {
                        let rows = (b.extent(sa) / 0x10) as u32;
                        let js: Vec<melee_types::itFoodsAttributes> = (0..rows)
                            .map(|r| {
                                let mut a = melee_types::itFoodsAttributes::zeroed();
                                a.x0 = d.u32_at(sa + 0x10 * r) as i32;
                                a.x4 = d.ptr::<HSD_Joint>(sa + 0x10 * r + 4);
                                a.x8 = d.u32_at(sa + 0x10 * r + 8) as i32;
                                a.xC = d.u32_at(sa + 0x10 * r + 12) as i32;
                                a
                            })
                            .collect();
                        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_JOINT_ROWS)).push((b.1 + sa as usize, js)) };
                    }
                }
            }
        }
    }
    tbl
}

/// The C reads an article's special attributes as `s32` / `f32` fields of the kind's attribute struct, but the pointer
/// is a `void*` into the big-endian image. Each block (up to the next boundary) is copied with its 32-bit words
/// swapped; articles sharing a block share the copy.
pub fn fix_articles() {
    use std::collections::HashMap;
    let typed = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_TYPED)) };
    for (a, t) in typed {
        unsafe { (*a).x4_specialAttributes = t };
    }
    let dyns = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_DYNAMICS)) };
    for (a, base, off) in dyns {
        unsafe {
            let be = |at: usize| u32::from_be_bytes(*((base + at) as *const [u8; 4]));
            let off = off as usize;
            let count = be(off + 8) as usize;
            let descs_off = be(off + 0xC) as usize;
            let mut v: Vec<melee_items::tu::itcoll::ItCollDynamicsDesc> = Vec::new();
            if count <= 8 && descs_off != 0 {
                for k in 0..count {
                    let d0 = descs_off + 20 * k;
                    v.push(melee_items::tu::itcoll::ItCollDynamicsDesc {
                        bone_id: be(d0) as i32,
                        offset: melee_types::Vec3 { x: f32::from_bits(be(d0 + 4)), y: f32::from_bits(be(d0 + 8)), z: f32::from_bits(be(d0 + 12)) },
                        size: f32::from_bits(be(d0 + 16)),
                    });
                }
            }
            let dd = Box::leak(v.into_boxed_slice());
            let nd = Box::leak(Box::new(melee_items::tu::itcoll::ItCollDynamics { _pad: [0; 8], count: count.min(8) as i32, descs: dd.as_mut_ptr() }));
            (*a).x14_dynamics = nd as *mut _ as *mut melee_types::ItemDynamics;
        }
    }
    let hurts = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_HURTBONES)) };
    for (a, base, off) in hurts {
        unsafe {
            let be = |at: usize| u32::from_be_bytes(*((base + at) as *const [u8; 4]));
            let off = off as usize;
            let count = be(off) as usize;
            let descs_off = be(off + 4) as usize;
            let mut v: Vec<melee_types::ItHurtBoneDesc> = Vec::new();
            if count <= 8 && descs_off != 0 {
                let f = |at: usize| f32::from_bits(be(at));
                for k in 0..count {
                    let d0 = descs_off + 0x20 * k;
                    let mut e = melee_types::ItHurtBoneDesc::zeroed();
                    e.bone_id = be(d0) as _;
                    e.a_offset = melee_types::Vec3 { x: f(d0 + 4), y: f(d0 + 8), z: f(d0 + 12) };
                    e.b_offset = melee_types::Vec3 { x: f(d0 + 16), y: f(d0 + 20), z: f(d0 + 24) };
                    e.scale = f(d0 + 28);
                    v.push(e);
                }
            }
            let dd = Box::leak(v.into_boxed_slice());
            let nl = Box::leak(Box::new(melee_types::ItHurtBoneList { count: count.min(8) as i32, descs: dd.as_mut_ptr() }));
            (*a).x8_hurtbones = nl as *mut _;
        }
    }
    let states = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_STATES)) };
    for (a, arr, cnt) in states {
        // the C writes one or two entries past the extent of some arrays (`Ground_801C0800`: the Random item's
        // `ald_yaku_all` scripts, into the bytes that follow in the file); the native copy keeps room for that
        let room = (cnt + 8) * core::mem::size_of::<melee_types::ItemStateDesc>();
        let buf = native_buf(room) as *mut melee_types::ItemStateDesc;
        unsafe {
            core::ptr::copy_nonoverlapping(arr, buf, cnt);
            (*a).xC_itemStates = buf as *mut melee_types::ItemStateArray;
        }
    }
    let joint_rows = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_JOINT_ROWS)) };
    let tables = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_TABLES)) };
    for (tbl, n, base, bounds) in tables {
        let b = Bounds(bounds, base);
        let end = base + b.0.last().copied().unwrap_or(0) as usize;
        let tbl = tbl as *mut *mut melee_types::Article;
        let mut done: HashMap<usize, usize> = HashMap::new();
        for i in 0..n {
            unsafe {
                let a = *tbl.add(i);
                if a.is_null() {
                    continue;
                }
                if (*a).x4_specialAttributes.is_null() {
                    continue;
                }
                let raw = (*a).x4_specialAttributes as usize;
                if raw < base || raw >= end {
                    continue; // already rebuilt (a shared article)
                }
                let copy = *done.entry(raw).or_insert_with(|| {
                    let off = (raw - base) as u32;
                    let len = (b.extent(off) as usize + 3) / 4 * 4;
                    let src = core::slice::from_raw_parts(raw as *const u8, len);
                    let mut v: Vec<u8> = Vec::with_capacity(len);
                    for w in src.chunks(4) {
                        let mut x = [0u8; 4];
                        x[..w.len()].copy_from_slice(w);
                        v.extend_from_slice(&u32::from_be_bytes(x).to_ne_bytes());
                    }
                    let out = Box::leak(v.into_boxed_slice()).as_mut_ptr();
                    // Foods rows: the joint pointer of variant r lives in `FOODS_JOINTS[r]` (see itfoods.rs); the row keeps its index
                    if let Some((_, rows)) = joint_rows.iter().find(|(a, _)| *a == raw) {
                        for (r, row) in rows.iter().enumerate().take(32) {
                            melee_items::tu::itfoods::FOODS_JOINTS[r] = row.x4;
                            if r * 0x10 + 8 <= len {
                                core::ptr::write_unaligned(out.add(r * 0x10 + 4) as *mut u32, r as u32);
                            }
                        }
                    }
                    out as usize
                });
                (*a).x4_specialAttributes = copy as *mut c_void;
            }
        }
    }
    let heiho = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(HEIHO_HEALTH)) };
    for (a, base, pt) in heiho {
        unsafe {
            let health = u32::from_be_bytes(*((base + pt as usize) as *const [u8; 4]));
            let sa = (*a).x4_specialAttributes as *mut u32;
            if !sa.is_null() {
                *sa = health;
            }
        }
    }
}

/// `rtable` with at least `min` entries (missing ones are null), for tables the C indexes by a count the bytes do not state.
fn rtable_min<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32, min: u32) -> *mut *mut T {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut T> = (0..n.max(min))
        .map(|i| if i < n { d.ptr::<T>(t + 4 * i) } else { core::ptr::null_mut() })
        .collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// The coin (`It_Kind_Unk4`, coin mode: `ftCommon_8007EA90` -> `it_802E5F8C`), the character-item table's entry
/// `It_Kind_Unk4 - It_Kind_Kuriboh` (it/item.c `Item_80267978`: `it_804D6D38 = it_804D6D20->x8`). Its special attributes are
/// an `it_2E5A_Attrs`: scalars, then three `it_2E5A_TierEntry { HSD_Joint* joint; ...anim joints; s32 value, threshold; ... }`
/// rows at GC +0x3C that `it_802E6380` reads (`it_80273318(gobj, attr->tiers[off].joint)`). The word swap keeps the GC
/// offsets and leaves the joints as image words, so the block is decoded as its record (as Unown's letters are).
fn coin_attrs(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) {
    let idx = (melee_types::It_Kind_Unk4 - melee_types::It_Kind_Kuriboh) as u32;
    let Some(t) = d.target(at) else { return };
    if b.extent(t) / 4 <= idx {
        return;
    }
    let Some(ta) = d.target(t + 4 * idx) else { return };
    let Some(sa) = d.target(ta + 4) else { return };
    if b.extent(sa) < <melee_types::it_2E5A_Attrs as GcDecode>::GC_BYTES || !d.is_reloc(sa + 0x3C) {
        return;
    }
    let art = d.object::<melee_types::Article>(ta);
    let typed = d.object::<melee_types::it_2E5A_Attrs>(sa) as *mut c_void;
    unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
}

/// The enemies' ("zako") articles, of the Adventure stages (stage `itemdata`) and of the character-item table (ItCo.dat:
/// Kuriboh, Leadead, Octarock, Ottosea): their attribute records start with a relocated pointer (`attr->x0->x4`, the walk
/// speed), which the 4-byte slot of the word-swapped block cannot hold, and every later field would read one word early
/// natively. Decoded as their records (the pointee as its record). The Goombas' `s32* x0` is read as `((f32*) x0)[1]`, so
/// their pointee is the block's words, and their record (all 4-byte scalars) is `[native ptr][the remaining words]`.
fn zako_attrs(d: &mut GcDecoder<'_>, b: &Bounds, kind: u32, ta: u32) {
    use melee_types as t;
    let Some(sa) = d.target(ta + 4) else { return };
    let k = kind;
    let typed: *mut c_void = if k == t::It_Kind_Patapata as u32 {
        d.object::<t::itPatapataAttributes>(sa) as *mut c_void
    } else if k == t::It_Kind_Old_Lead as u32 || k == t::It_Kind_Leadead as u32 {
        d.object::<t::itLeadeadAttributes>(sa) as *mut c_void
    } else if k == t::It_Kind_Old_Octa as u32 || k == t::It_Kind_Octarock as u32 {
        d.object::<t::itOctarockAttributes>(sa) as *mut c_void
    } else if k == t::It_Kind_Old_Otto as u32 || k == t::It_Kind_Ottosea as u32 {
        d.object::<t::itOldottoseaAttributes>(sa) as *mut c_void
    } else if k == t::It_Kind_Whitebea as u32 {
        d.object::<t::itWhiteBeaAttributes>(sa) as *mut c_void
    } else if (k == t::It_Kind_Old_Kuri as u32 || k == t::It_Kind_Kuriboh as u32) && d.is_reloc(sa) {
        let words = (b.extent(sa) / 4).max(4);
        let pt = d.target(sa);
        let pw = pt.map_or(0, |pt| (b.extent(pt) / 4).max(2));
        let pointee = native_buf(4 * pw as usize + 8);
        let out = native_buf(8 + 4 * words as usize + 16);
        unsafe {
            if let Some(pt) = pt {
                for w in 0..pw {
                    (pointee.add(4 * w as usize) as *mut u32).write_unaligned(d.u32_at(pt + 4 * w));
                }
            }
            (out as *mut *mut u8).write(pointee);
            for w in 1..words {
                (out.add(4 + 4 * w as usize) as *mut u32).write_unaligned(d.u32_at(sa + 4 * w));
            }
        }
        out as *mut c_void
    } else {
        return;
    };
    let art = d.object::<melee_types::Article>(ta);
    unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
}

/// [`zako_attrs`] for the character-item table's enemies (`It_Kind_Kuriboh` .. `It_Kind_Ottosea`, entries 0..3).
fn zako_char_items(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) {
    let Some(t) = d.target(at) else { return };
    let n = b.extent(t) / 4;
    for idx in 0..4u32.min(n) {
        if let Some(ta) = d.target(t + 4 * idx) {
            zako_attrs(d, b, melee_types::It_Kind_Kuriboh as u32 + idx, ta);
        }
    }
}

/// `itPublicData` (ItCo.dat / ItCoUS.dat): `it_804D6D20_t`, whose `Article**` tables run to the next boundary.
fn itco(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let at = |i: u32| off + 4 * i;
    let mut r = it_804D6D20_t::zeroed();
    r.x0 = d.ptr::<ItemCommonData>(at(0));
    r.x4 = rtable_articles(d, b, at(1));
    r.x8 = rtable_articles(d, b, at(2));
    coin_attrs(d, b, at(2));
    zako_char_items(d, b, at(2));
    r.xC = rtable_articles(d, b, at(3));
    r.x10 = d.ptr::<it_804D6D40_t>(at(4));
    r.x14 = parr::<Fighter_804D653C_t>(d, b, at(5)) as *mut Fighter_804D653C_t;
    Box::leak(Box::new(r)) as *mut _ as *mut c_void
}

/// A root that is a table of `T*` (C `T**`), one record per entry, running to the next boundary. The root is the
/// table itself (`HSD_ArchiveGetPublicAddress` returns the address of the first slot).
fn ptrtable<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let n = (b.extent(off) / 4).max(1);
    let v: Vec<*mut T> = (0..n).map(|i| d.ptr::<T>(off + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// A fighter's public data (`ftDataCaptain`, ...): the record plus the per-animation arrays whose length the C takes
/// from the retail tables (`ftData_Table_Unk0` rows for `xC` / `x10`, `ftData_UnkIntPairs` rows for `x14` / `x18`).
fn ftdata(d: &mut GcDecoder<'_>, b: &Bounds, off: u32, sym: &str) -> *mut c_void {
    use melee_core::gc::GcLayout;
    use melee_fighter::tu::ftdata as fd;
    use melee_types::{ftData, Fighter_WaitAnimData};
    let kind = unsafe {
        (0..fd::ftData_803C1F40.len())
            .find(|&k| {
                let b = fd::ftData_803C1F40[k].b;
                !b.is_null() && core::ffi::CStr::from_ptr(b).to_bytes() == sym.as_bytes()
            })
            .unwrap_or_else(|| panic!("dat_custom: `{sym}` is not a fighter data symbol of ftData_803C1F40"))
    };
    let (n, m) = unsafe { (fd::ftData_Table_Unk0[kind].count as usize, fd::ftData_UnkIntPairs[kind].count as usize) };
    let at = |f: &str| off + ftData::gc_field(f).expect("ftData field").offset;
    let mut v = <ftData as GcDecode>::gc_decode(d, off);
    v.xC = d.ptr_array::<Fighter_WaitAnimData>(at("xC"), n);
    v.x10 = d.ptr_array::<[u8; 2]>(at("x10"), n);
    v.x14 = d.ptr_array::<Fighter_WaitAnimData>(at("x14"), m);
    v.x18 = d.ptr_array::<[u8; 2]>(at("x18"), m);
    // `int* x54`: the random-bone table `ftCo_8009F834` indexes with the 0..4 counter `x2220_b0` (the plain pointer decode would read one `int`)
    v.x54 = d.ptr_array::<i32>(at("x54"), 5);
    v.x48_items = rtable_articles(d, b, at("x48_items")) as *mut *mut c_void;
    typed_article_attrs(d, b, sym.trim_start_matches("ftData").to_lowercase().as_str(), at("x48_items"));
    if sym.trim_start_matches("ftData").eq_ignore_ascii_case("gamewatch") {
        gw_article_attrs(d, b, at("x48_items"));
    }
    if sym.trim_start_matches("ftData").eq_ignore_ascii_case("samus") {
        // `ftSs_Init_CreateThrowGrappleBeam`: `x48_items[4]` is a `UNK_SAMUS_S1 { joint, anim_joints, anim_joint, matanim_joint }`, not an article
        if let Some(t) = d.target(at("x48_items")) {
            let n = (b.extent(t) / 4).max(1) as usize;
            if n > 4 {
                let mut tbl: std::vec::Vec<*mut c_void> = (0..n).map(|i| unsafe { *(v.x48_items as *mut *mut c_void).add(i) }).collect();
                tbl[4] = d.ptr::<melee_types::UNK_SAMUS_S1>(t + 16) as *mut c_void;
                v.x48_items = Box::leak(tbl.into_boxed_slice()).as_mut_ptr() as *mut *mut c_void;
            }
        }
    }
    // `x48_items` slots the C reads as something other than an `Article*` (the table is `void**`): decoded as their own type
    if let Some(t) = d.target(at("x48_items")) {
        let n = (b.extent(t) / 4).max(1) as usize;
        let w = |i: usize| t + 4 * i as u32;
        let over: std::vec::Vec<(usize, *mut c_void)> = match sym.trim_start_matches("ftData").to_lowercase().as_str() {
            // ftLk_Init_OnLoad / ftCl_Init_OnLoad: `ftParts_800753D4(fp, .., item_list[6])`, the hylian shield / master sword joint
            "link" | "clink" if n > 6 => std::vec![(6, d.ptr::<HSD_Joint>(w(6)) as *mut c_void)],
            // ftKb_SpecialN_800F5898: `return ca->x48_items[4]` (HSD_Joint*)
            "kirby" if n > 4 => std::vec![(4, d.ptr::<HSD_Joint>(w(4)) as *mut c_void)],
            // ftYs_SpecialN_8012CDD4: `return ca->x48_items[3]` (HSD_Joint*)
            "yoshi" if n > 3 => std::vec![(3, d.ptr::<HSD_Joint>(w(3)) as *mut c_void)],
            // ftGw_Init_OnLoad: `fp->x5AC.xC[4] = items[10]`, an FtPartsVisLookup array of `model_num` entries (ftParts_80074D7C)
            "gamewatch" if n > 10 && d.target(at("x8")).is_some() => {
                // the `FtPartsDesc.model_num` word of `x8` (the decoded record is filled later, so read the bytes)
                let t8 = d.target(at("x8")).unwrap();
                let models = d.u32_at(t8) as usize;
                std::vec![(10, d.ptr_array::<melee_types::FtPartsVisLookup>(w(10), models) as *mut c_void)]
            }
            // ftSk_SpecialS_80110610: `item = items[4 | 5]` is an `HSD_Joint**` (the aim animation joints, read as `item[2]`)
            "seak" if n > 5 => std::vec![
                (4, rtable::<HSD_Joint>(d, b, w(4)) as *mut c_void),
                (5, rtable::<HSD_Joint>(d, b, w(5)) as *mut c_void),
            ],
            _ => std::vec::Vec::new(),
        };
        if !over.is_empty() {
            let mut tbl: std::vec::Vec<*mut c_void> = (0..n).map(|i| unsafe { *(v.x48_items as *mut *mut c_void).add(i) }).collect();
            for (i, p) in over {
                tbl[i] = p;
            }
            v.x48_items = Box::leak(tbl.into_boxed_slice()).as_mut_ptr() as *mut *mut c_void;
        }
    }
    note_dynamics(d, &v, at("x2C"));
    let lower = sym.trim_start_matches("ftData").to_lowercase();
    if let Some(p) = crate::extattr::decode(&lower, d, at("ext_attr")) {
        v.ext_attr = p;
    }
    Box::leak(Box::new(v)) as *mut ftData as *mut c_void
}

/// `ftDataKirbyCopy*` (PlKbCpXx.dat): a `KirbyHatStruct` whose five-slot `hat_dynamics` is really a table of differently typed
/// pointers (`void*` casts at the C call sites: articles registered with `it_8026B3F8` by `ftKb_SpecialN_800F16D0`, accessory joints),
/// with up to seven slots (Game & Watch registers slots 5 and 6, Yoshi slot 5): the record is rebuilt in a buffer of eight slots.
fn kirbyhat(d: &mut GcDecoder<'_>, b: &Bounds, off: u32, sym: &str) -> *mut c_void {
    use melee_types::KirbyHatStruct;
    // slots 0..4 are the record's, 5/6 the extra articles, 7 the slot-2 joint tree, 8 the raw address of slot 4 (Game & Watch colours)
    const SLOTS: usize = 9;
    // by value: `d.object` hands back a record that is filled in later, and this copy is taken now
    let v = <KirbyHatStruct as GcDecode>::gc_decode(d, off);
    let h = &v as *const KirbyHatStruct;
    let base = core::mem::offset_of!(KirbyHatStruct, hat_dynamics);
    let size = base + SLOTS * core::mem::size_of::<usize>();
    let buf = native_buf(size.max(core::mem::size_of::<KirbyHatStruct>()));
    unsafe { core::ptr::copy_nonoverlapping(h as *const u8, buf, core::mem::size_of::<KirbyHatStruct>()) };
    let slots = unsafe { buf.add(base) as *mut *mut c_void };
    let word = |k: usize| off + 0xC + 4 * k as u32;
    // slot 7: slot 2 read as an `HSD_Joint` tree (`ftKb_SpecialN_800EF438`: `root = hat->hat_dynamics[2]`; slot 2 is a dynamics block in some hats)
    unsafe { *slots.add(7) = d.ptr::<HSD_Joint>(word(2)) as *mut c_void };
    // a slot word that is not relocated is a plain value the C casts back (`u32 mask = (u32) hat->hat_dynamics[1]`, the part
    // mask of every LOAD_HAT hat, ftKb_SpecialN_800EF040): keep the word instead of the decoder's null
    for k in 0..5 {
        if !d.is_reloc(word(k)) {
            unsafe { *slots.add(k) = d.u32_at(word(k)) as usize as *mut c_void };
        }
    }
    let lower = sym.trim_start_matches("ftDataKirbyCopy").to_lowercase();
    // the slots holding an `Article*` (ftKb_SpecialN_800F16D0; Mario's hat is `Kirby_Unk`, its `xC` is slot 0)
    let arts: &[usize] = match lower.as_str() {
        "mario" | "luigi" | "popo" | "samus" | "koopa" | "drmario" => &[0],
        "peach" | "fox" | "link" | "clink" | "ness" | "pikachu" | "pichu" | "seak" => &[0, 1],
        "falco" => &[3, 4],
        "mewtwo" => &[3],
        "gamewatch" => &[5, 6],
        "yoshi" => &[5],
        _ => &[],
    };
    let mut tas: Vec<u32> = Vec::new();
    let mut tbl: Vec<*mut Article> = Vec::new();
    for &k in arts {
        if let Some(ta) = d.target(word(k)) {
            note_article_states(d, b, ta);
            let art = d.object::<Article>(ta);
            unsafe { *slots.add(k) = art as *mut c_void };
            tas.push(ta);
            tbl.push(art);
        }
    }
    if !tbl.is_empty() {
        let cnt = tbl.len();
        let t = Box::leak(tbl.into_boxed_slice()).as_mut_ptr();
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TABLES)).push((t as usize, cnt, b.1, b.0.clone())) };
    }
    typed_article_attrs_of(d, b, &lower, &tas);
    if lower == "gamewatch" {
        for &ta in &tas {
            gw_article_attr(d, b, ta);
        }
    }
    // the slot holding the hat's dynamic bones (`ftCo_8009D074` .. `ftCo_8009DB50`, `ftKb_Init_800EEB00`; the C indexes `hats[]` by kind - 1),
    // words swapped like a fighter's `x2C`
    let dynamics: Option<usize> = match lower.as_str() {
        "link" | "clink" | "pikachu" | "pichu" | "seak" => Some(2),
        "koopa" | "mars" | "emblem" => Some(1),
        "zelda" => Some(0),
        "purin" => Some(3),
        "mewtwo" | "gamewatch" => Some(4),
        _ => None,
    };
    if let Some(k) = dynamics {
        if d.is_reloc(word(k)) {
            note_dynamics_at(d, unsafe { *slots.add(k) } as usize, word(k));
        }
    }
    // Game & Watch (`ftKb_SpecialN_800F14B4`, the C's `hats[Ft_Kind_Pichu]`): slot 3 is the `FtPartsVisLookup` array of the
    // hat's models (`fp->x5AC.xC[4]`, walked by ftParts_80074D7C), and the colour words are read as raw bytes
    // at slot 4 + 4 / + 8 (slot 4 itself stays the dynamics block): slot 8 holds slot 4's raw address
    if lower == "gamewatch" {
        if d.is_reloc(word(3)) {
            // `model_num` is the record's first word (LOAD_HAT passes the hat itself as the `FtPartsDesc`). The lookup is walked
            // for the WEARER's model count (Kirby's `fp->x5AC.model_num`, 2) while the hat has 1 entry: the C reads on into the
            // next words (a relocated pointer as the count, negative, so no iterations). The native array is padded with empty
            // rows up to the parts-model maximum (11) so the extra rows are the same no-op instead of a read past the decode.
            use melee_types::FtPartsVisLookup;
            const MAX_MODELS: usize = 11;
            let models = (d.u32_at(off) as usize).min(MAX_MODELS);
            let src = d.ptr_array::<FtPartsVisLookup>(word(3), models);
            let dst = native_buf(MAX_MODELS * core::mem::size_of::<FtPartsVisLookup>()) as *mut FtPartsVisLookup;
            if !src.is_null() {
                unsafe { core::ptr::copy_nonoverlapping(src, dst, models) };
            }
            unsafe { *slots.add(3) = dst as *mut c_void };
        }
        if let Some(t) = d.target(word(4)) {
            unsafe { *slots.add(8) = b.raw_addr(t) };
        }
    }
    // Yoshi (the C's `hats[Ft_Kind_Samus]` / `hats[0xD]`, ftkirbyspecialyoshi.c): slots 1..4 are `HSD_AnimJoint*` trees for
    // `HSD_JObjAddAnimAll`; slot 0 is the egg's `HSD_Joint` tree (ftKb_SpecialNYs_801093A0 returns it typed `ftDynamics*`, and
    // ftKb_SpecialNYs_8010AC78 passes it to `ftCommon_SetAccessory` as `(HSD_Joint*)`)
    if lower == "yoshi" {
        if d.is_reloc(word(0)) {
            unsafe { *slots.add(0) = d.ptr::<HSD_Joint>(word(0)) as *mut c_void };
        }
        for k in 1..5 {
            if d.is_reloc(word(k)) {
                unsafe { *slots.add(k) = d.ptr::<melee_types::HSD_AnimJoint>(word(k)) as *mut c_void };
            }
        }
    }
    // accessory models: Mars / Marth's emblem slot 0, the Ice Climbers' slot 1 (`ftCommon_SetAccessory(fp, (HSD_Joint*) hat_dynamics[k])`)
    let joint: Option<usize> = match lower.as_str() {
        "mars" | "emblem" => Some(0),
        "popo" => Some(1),
        _ => None,
    };
    if let Some(k) = joint {
        if d.is_reloc(word(k)) {
            unsafe { *slots.add(k) = d.ptr::<HSD_Joint>(word(k)) as *mut c_void };
        }
    }
    buf as *mut c_void
}

/// Fighter-owned articles whose special attributes hold `HSD_Joint*` fields (the grapple beam, the hookshot, the boomerang,
/// the arrow, the chain, the yo-yo, the Ice Climbers' string): the 32-bit pointer slots of the raw block do not fit the native
/// struct, so the block is decoded as its record and swapped in by [`fix_articles`]: (article, decoded attributes).
static mut ARTICLE_TYPED: Vec<(*mut melee_types::Article, *mut c_void)> = Vec::new();

/// Decode the fighter's `x48_items` articles whose attribute block is one of the pointer-bearing records of that fighter.
fn typed_article_attrs(d: &mut GcDecoder<'_>, b: &Bounds, sym: &str, at_x48: u32) {
    let Some(t) = d.target(at_x48) else { return };
    let n = (b.extent(t) / 4).max(1);
    let tas: Vec<u32> = (0..n).filter_map(|i| d.target(t + 4 * i)).collect();
    typed_article_attrs_of(d, b, sym, &tas);
}

/// [`typed_article_attrs`] over explicit article offsets.
fn typed_article_attrs_of(d: &mut GcDecoder<'_>, b: &Bounds, sym: &str, tas: &[u32]) {
    use melee_types::{Article, itClimbersStringAttributes, itLinkArrowAttributes, itLinkBoomerangAttributes, itLinkHookshotAttributes, itSamusGrappleAttributes, itSeakChain_Attrs, itYoyoAttributes};
    type Dec = fn(&mut GcDecoder<'_>, u32) -> *mut c_void;
    fn dec<T: GcPointee + GcDecode>(d: &mut GcDecoder<'_>, sa: u32) -> *mut c_void {
        d.object::<T>(sa) as *mut c_void
    }
    fn cand<T: GcPointee + GcDecode + melee_core::gc::GcLayout>() -> (u32, &'static [melee_core::gc::GcField], Dec) {
        (<T as GcDecode>::GC_BYTES, <T as melee_core::gc::GcLayout>::GC_FIELDS, dec::<T> as Dec)
    }
    let cands: std::vec::Vec<(u32, &'static [melee_core::gc::GcField], Dec)> = match sym {
        "samus" => std::vec![cand::<itSamusGrappleAttributes>()],
        "link" | "clink" => std::vec![cand::<itLinkHookshotAttributes>(), cand::<itLinkBoomerangAttributes>(), cand::<itLinkArrowAttributes>()],
        "seak" => std::vec![cand::<itSeakChain_Attrs>()],
        "ness" => std::vec![cand::<itYoyoAttributes>()],
        "popo" | "nana" => std::vec![cand::<itClimbersStringAttributes>()],
        _ => return,
    };
    for &ta in tas {
        let Some(sa) = d.target(ta + 4) else { continue };
        let ext = b.extent(sa);
        // The block's extent runs to the next object (it can fall short of the record by a word, or include the next block): the record
        // is the candidate whose pointer-typed fields hold the relocated words of the block (null pointers are not relocated), with the
        // fewest relocated words in its other fields
        let score = |sz: u32, fields: &[melee_core::gc::GcField]| -> (u32, u32) {
            let (mut hit, mut extra) = (0, 0);
            for w in 0..sz / 4 {
                if d.is_reloc(sa + 4 * w) {
                    if fields.iter().any(|f| f.offset == 4 * w && f.c_type.contains('*')) {
                        hit += 1;
                    } else {
                        extra += 1;
                    }
                }
            }
            (hit, extra)
        };
        let pick = cands
            .iter()
            .filter(|(sz, fields, _)| *sz <= ext + 8 && score(*sz, fields).0 > 0)
            .min_by_key(|(sz, fields, _)| {
                let (hit, extra) = score(*sz, fields);
                (extra, core::cmp::Reverse(hit))
            });
        if let Some((_, _, f)) = pick {
            let art = d.object::<Article>(ta);
            let typed = f(d, sa);
            unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, typed)) };
        }
    }
}

/// Game & Watch articles: word 0 of the special attributes is a relocated pointer to the `it_266F_ItemVars` record that `it_8027CE64`
/// stores in `xDD4` and `it_8026EECC` reads (`void* x0` of the C attribute structs, e.g. `itGamewatchchefAttributes`). The 4-byte slot of
/// the word-swapped block cannot hold the native pointer, so the block is rebuilt as `[native ptr][the remaining words, swapped]`
/// (the layout the native structs have: a pointer, then 4-byte fields), and swapped in by [`fix_articles`].
fn gw_article_attrs(d: &mut GcDecoder<'_>, b: &Bounds, at_x48: u32) {
    let Some(t) = d.target(at_x48) else { return };
    let n = (b.extent(t) / 4).max(1);
    for i in 0..n {
        if let Some(ta) = d.target(t + 4 * i) {
            gw_article_attr(d, b, ta);
        }
    }
}

/// [`gw_article_attrs`] for the one article at data offset `ta` (also Kirby's Game & Watch hat articles, `kirbyhat` slots 5 / 6:
/// the chef's pan reads `xDD4.gamewatch.attr` = special attribute word 0 as its `it_266F_ItemVars*` in `it_8026EECC`).
fn gw_article_attr(d: &mut GcDecoder<'_>, b: &Bounds, ta: u32) {
    use melee_types::{Article, it_266F_ItemVars};
    let Some(sa) = d.target(ta + 4) else { return };
    if !d.is_reloc(sa) {
        return;
    }
    let words = (b.extent(sa) / 4).max(1);
    let rec = d.ptr::<it_266F_ItemVars>(sa);
    let out = native_buf(8 + 4 * words as usize + 16);
    unsafe {
        (out as *mut *mut it_266F_ItemVars).write(rec);
        for k in 1..words {
            (out.add(4 + 4 * k as usize) as *mut u32).write_unaligned(d.u32_at(sa + 4 * k));
        }
        let art = d.object::<Article>(ta);
        (*core::ptr::addr_of_mut!(ARTICLE_TYPED)).push((art, out as *mut c_void));
    }
}

/// Shy Guy articles: (article, image base, data offset of the health `s32` that `attr[0]` points at). [`fix_articles`] stores the value
/// itself in the native word 0 of the swapped attribute copy (the native pointer would not fit the 4-byte slot).
static mut HEIHO_HEALTH: Vec<(*mut melee_types::Article, usize, u32)> = Vec::new();

/// Dynamics blocks to rebuild after the decoder ran: (the `ftDynamics` object, per bone the native word block).
static mut DYNAMICS_FIX: Vec<(usize, Vec<(usize, Vec<u32>)>)> = Vec::new();

/// `ftData.x2C` (`ftDynamics`): each bone's `dyn_desc.data` points at `count` records of `lb_00F9_UnkDesc1Inner` (15 `f32`,
/// 0x3C bytes each) that `lb_80011710` reads as an array. The generic decode kept the first `PolymorphicDesc` worth of
/// raw big-endian bytes (two records, unswapped); [`fix_dynamics`] rebuilds the block native, `count` records long.
fn note_dynamics(d: &mut GcDecoder<'_>, v: &melee_types::ftData, at_x2c: u32) {
    note_dynamics_at(d, v.x2C as usize, at_x2c);
}

/// [`note_dynamics`] for the `ftDynamics` object `obj` decoded from the pointer word at `at_x2c` (a Kirby hat's `hat_dynamics[4]` too).
fn note_dynamics_at(d: &mut GcDecoder<'_>, obj: usize, at_x2c: u32) {
    const REC: usize = 15;
    let Some(t) = d.target(at_x2c) else { return };
    let n = d.u32_at(t) as usize;
    let Some(tb) = d.target(t + 4) else { return };
    if obj == 0 {
        return;
    }
    let mut bones = Vec::new();
    for i in 0..n.min(10) {
        let e = tb + 0x18 * i as u32;
        let (Some(td), count) = (d.target(e + 4), d.u32_at(e + 8) as usize) else { continue };
        if count == 0 || count > 16 {
            continue;
        }
        bones.push((i, (0..count * REC).map(|k| d.u32_at(td + 4 * k as u32)).collect::<Vec<_>>()));
    }
    unsafe { (*core::ptr::addr_of_mut!(DYNAMICS_FIX)).push((obj, bones)) };
}

/// Applies [`note_dynamics`] (run by `archive.rs` right after `GcDecoder::finish`).
pub fn fix_dynamics() {
    use melee_types::{DynamicsData, ftDynamics};
    let todo = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(DYNAMICS_FIX)) };
    for (obj, bones) in todo {
        let tbl = unsafe { (*(obj as *mut ftDynamics)).anon_0.ftDynamicBones };
        if tbl.is_null() {
            eprintln!("engine: an ftDynamics table decoded to null");
            continue;
        }
        for (i, words) in bones {
            let buf = native_buf((words.len() * 4).max(core::mem::size_of::<DynamicsData>()));
            for (k, w) in words.iter().enumerate() {
                unsafe { (buf as *mut u32).add(k).write(*w) };
            }
            unsafe { (*tbl).array[i].dyn_desc.data = buf as *mut DynamicsData };
        }
    }
}

// ---- particle banks (baselib/particle.c: psInitDataBankLocate / psInitDataBankLoad) ----------------------------
//
// The cmd and tex banks are blobs of GameCube-layout records addressed by 32-bit offsets relative to the bank start
// (NOT archive relocations: `psInitDataBankLocate` adds the bank address to them). The native `tu::particle` reads
// pointer-sized slots and native records, so each bank is rebuilt into one native image whose slots hold byte
// offsets relative to the image start, which `psInitDataBankLocate` then relocates exactly as the C does. What
// stays raw: the command STREAM of each record (opcodes and big-endian operands, read byte-wise;
// `particle::CMD_STREAM_BIG_ENDIAN`) and the texture/palette image bytes (pointers into the raw archive image).

use std::collections::HashMap;

static mut BANKS: Option<HashMap<(usize, u32), usize>> = None;

#[allow(static_mut_refs)]
fn memo(key: (usize, u32), make: impl FnOnce() -> *mut c_void) -> *mut c_void {
    let m = unsafe { BANKS.get_or_insert_with(HashMap::new) };
    if let Some(&p) = m.get(&key) {
        return p as *mut c_void;
    }
    let p = make();
    unsafe { BANKS.as_mut().unwrap().insert(key, p as usize) };
    p
}

/// A new file image parsed at raw data address `base` (`host_parse`): the banks built from an earlier file at that address
/// are not this file's (a later scene loads another file, or the same one again, into the same heap block). Their memo
/// entries go, so the next lookup builds and `psInitDataBankLocate` relocates a fresh bank, as the C relocates the
/// freshly read file.
#[allow(static_mut_refs)]
pub fn forget_banks(base: usize) {
    if let Some(m) = unsafe { BANKS.as_mut() } {
        m.retain(|k, _| k.0 != base);
    }
}

/// A zeroed 8-aligned native buffer of `n` bytes that lives for the rest of the process.
/// A `SIS_*Data` root (`HSD_SisLib_804D1124[font]`): word 0 / 1 are the kerning and glyph-texture tables (`SIS { kerning, textures }`, typed
/// arrays), the following words are the pointers to the text strings, which the text code (`HSD_SisLib_803A6368`, `803A660C`, ...) reads as
/// byte streams: native pointer slots, the strings kept in the raw image.
fn sis_table(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let n = ((b.extent(off) / 4) as usize).max(2);
    let buf = native_buf(n * 8) as *mut usize;
    unsafe {
        buf.write(d.ptr_array_extent::<melee_types::TextKerning>(off) as usize);
        buf.add(1).write(d.ptr_array_extent::<melee_types::TextGlyphTexture>(off + 4) as usize);
        for i in 2..n {
            if let Some(t) = d.target(off + 4 * i as u32) {
                buf.add(i).write(b.raw_addr(t) as usize);
            }
        }
    }
    buf as *mut c_void
}

fn native_buf(n: usize) -> *mut u8 {
    Box::leak(vec![0u64; (n + 7) / 8].into_boxed_slice()).as_mut_ptr() as *mut u8
}

/// The particle command bank starting at data offset `off`: `{ version, count.., offsets }` + `HSD_PSCmdList` records.
pub fn ptcl_cmd(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    use melee_types::HSD_PSCmdList;
    memo((b.1, off), || {
        let version = d.u16_at(off);
        // (slot count of the header, index of the first record slot, record count)
        let (hdr, first, count) = match version {
            0 => (2usize, 2usize, d.u32_at(off + 4) as usize),
            // versions 0x40..0x43: `num` is the index base (generator ids run from `num`; the C points the list `3 - num` words
            // before the table), `count` the number of records
            0x40..=0x43 => (3, 3, d.u32_at(off + 8) as usize),
            v => panic!("particle cmd bank {off:#x}: unknown version {v:#x}"),
        };
        let slots = hdr + count;
        let rel: Vec<u32> = (0..count).map(|i| d.u32_at(off + 4 * (first + i) as u32)).collect();
        let mut starts: Vec<u32> = rel.iter().copied().filter(|&r| r != 0).collect();
        starts.sort_unstable();
        starts.dedup();
        let end_of_bank = off + b.extent(off);
        const HDR: usize = 0x3C; // HSD_PSCmdList up to `cmdList`
        let rec_len = |r: u32| -> usize {
            let at = off + r;
            let stop = match starts.binary_search(&r) {
                Ok(i) => starts.get(i + 1).map_or(end_of_bank, |n| off + n),
                Err(_) => end_of_bank,
            };
            (stop.max(at + HDR as u32) - at) as usize
        };
        let total = slots * 8 + starts.iter().map(|&r| (rec_len(r) + 7) & !7).sum::<usize>();
        let img = native_buf(total);
        let w = |at: usize, v: isize| unsafe { (img.add(at) as *mut isize).write(v) };
        w(0, version as isize);
        if version == 0 {
            w(8, count as isize);
        } else {
            w(8, d.u32_at(off + 4) as isize);
            w(16, count as isize);
        }
        let mut cur = slots * 8;
        let mut placed: HashMap<u32, usize> = HashMap::new();
        for &r in &starts {
            let len = rec_len(r);
            let rec = <HSD_PSCmdList as GcDecode>::gc_decode(d, off + r);
            unsafe {
                core::ptr::copy_nonoverlapping(&rec as *const HSD_PSCmdList as *const u8, img.add(cur), HDR);
                for k in HDR..len {
                    *img.add(cur + k) = d.u8_at(off + r + k as u32);
                }
            }
            placed.insert(r, cur);
            cur += (len + 7) & !7;
        }
        for (i, &r) in rel.iter().enumerate() {
            w(8 * (first + i), if r == 0 { 0 } else { placed[&r] as isize });
        }
        img as *mut c_void
    })
}

/// The particle texture bank at data offset `off`: `{ group count, group offsets }` + `HSD_PSTexGroup` records whose
/// texture / palette slots become offsets from the image start to the (raw, big-endian) image bytes.
pub fn ptcl_tex(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    memo((b.1, off), || {
        let n = d.u32_at(off) as usize;
        let rel: Vec<u32> = (0..n).map(|i| d.u32_at(off + 4 + 4 * i as u32)).collect();
        // slot count of group record `g`: `num`, plus the palette entries `psInitDataBankLocate` relocates
        let slots_of = |d: &mut GcDecoder<'_>, g: u32| -> usize {
            let num = d.u32_at(off + g);
            let fmt = d.u32_at(off + g + 4);
            let palnum = d.u16_at(off + g + 0x14) as u32;
            let palflag = d.u16_at(off + g + 0x16) as u32;
            let end = if fmt != 8 && fmt.wrapping_sub(9) > 1 {
                num
            } else if palflag & 1 != 0 {
                num + 1
            } else if palnum != 0 {
                num + palnum
            } else {
                num.wrapping_mul(2)
            };
            end as usize
        };
        let sizes: Vec<usize> = rel.iter().map(|&g| if g == 0 { 0 } else { 24 + 8 * slots_of(d, g).max(1) }).collect();
        let total = 8 + 8 * n + sizes.iter().sum::<usize>();
        let img = native_buf(total);
        let base = img as usize;
        unsafe { (img as *mut isize).write(n as isize) };
        let mut cur = 8 + 8 * n;
        for (i, &g) in rel.iter().enumerate() {
            if g == 0 {
                continue;
            }
            unsafe { (img.add(8 + 8 * i) as *mut isize).write(cur as isize) };
            let slots = slots_of(d, g);
            let at = off + g;
            unsafe {
                for (o, src) in [(0usize, 0u32), (4, 4), (8, 8), (12, 12), (16, 16)] {
                    (img.add(cur + o) as *mut u32).write(d.u32_at(at + src));
                }
                (img.add(cur + 20) as *mut u16).write(d.u16_at(at + 0x14));
                (img.add(cur + 22) as *mut u16).write(d.u16_at(at + 0x16));
                for k in 0..slots {
                    let t = d.u32_at(at + 0x18 + 4 * k as u32);
                    // `psInitDataBankLocate` adds the image base: store (raw image address - base)
                    let v: isize = if t == 0 { 0 } else { (b.raw_addr(off + t) as usize).wrapping_sub(base) as isize };
                    (img.add(cur + 24 + 8 * k) as *mut isize).write(v);
                }
            }
            cur += sizes[i];
        }
        img as *mut c_void
    })
}

/// An effect bank root (`effCommonDataTable`, `effFoxDataTable`, ...): C `struct { cmd bank*, tex bank*, EF_EffectDesc descs[] }`
/// (`EF_DAT_Entry` for the first two words; `efAsync_LoadSync` keeps `&entry->data` as the `EF_EffectDesc` table). Native:
/// two converted bank pointers, then the native `EF_EffectDesc` array, contiguous from byte 16 (= `offsetof(EF_DAT_Entry, data)`).
fn efdat(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    use melee_types::EF_EffectDesc;
    let cmd = d.target(off).map_or(core::ptr::null_mut(), |t| ptcl_cmd(d, b, t));
    let tex = d.target(off + 4).map_or(core::ptr::null_mut(), |t| ptcl_tex(d, b, t));
    let n = (b.extent(off + 8) / <EF_EffectDesc as GcDecode>::GC_BYTES) as usize;
    let sz = core::mem::size_of::<EF_EffectDesc>();
    let img = native_buf(16 + n.max(1) * sz);
    unsafe {
        (img as *mut *mut c_void).write(cmd);
        (img.add(8) as *mut *mut c_void).write(tex);
        if n > 0 {
            let descs = d.array::<EF_EffectDesc>(off + 8, n);
            core::ptr::copy_nonoverlapping(descs as *const u8, img.add(16), n * sz);
        }
    }
    img as *mut c_void
}

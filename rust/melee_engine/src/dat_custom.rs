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
pub fn decode(name: &str, d: &mut GcDecoder<'_>, off: u32, _extent: u32, b: &Bounds) -> *mut c_void {
    match name {
        "plco" => plco(d, b, off),
        "itco" => itco(d, b, off),
        "ptrtable_itemdata" => ptrtable::<melee_types::GroundItemData>(d, b, off),
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
                        break;
                    }
                }
            }
            Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
        }
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
    out[1] = ptable::<i32>(d, b, at(1)); // int**
    out[2] = parr::<f32>(d, b, at(2)); // float (*)[5]
    out[3] = parr::<f32>(d, b, at(3)); // float*
    out[4] = ptable::<FighterPartsTable>(d, b, at(4));
    out[5] = ptable::<Fighter_804D6540_t>(d, b, at(5));
    out[6] = parr::<Fighter_804D653C_t>(d, b, at(6));
    out[7] = parr::<Fighter_804D653C_t>(d, b, at(7));
    // [8] UNK_T, [17] UNK_T, [21] crowd config: no type in the C either; read as null
    out[9] = ptable::<Vec2>(d, b, at(9)); // Vec2**
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
    out[22] = parr::<Fighter_804D64FC_t>(d, b, at(22));
    // the root is a pointer to the table (`void** pData` after `lbArchive_LoadSymbols`)
    Box::leak(out.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// A table of `T*` (`T**`), one record per entry (null entry = null pointer), sized by the next boundary.
fn rtable<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut T {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut T> = (0..n).map(|i| d.ptr::<T>(t + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// `itPublicData` (ItCo.dat / ItCoUS.dat): `it_804D6D20_t`, whose `Article**` tables run to the next boundary.
fn itco(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let at = |i: u32| off + 4 * i;
    let mut r = it_804D6D20_t::zeroed();
    r.x0 = d.ptr::<ItemCommonData>(at(0));
    r.x4 = rtable::<Article>(d, b, at(1));
    r.x8 = rtable::<Article>(d, b, at(2));
    r.xC = rtable::<Article>(d, b, at(3));
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

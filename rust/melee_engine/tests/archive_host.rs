//! Loads real archives through the game's own `lbFile` and `lbArchive` TUs on the engine host, and checks the
//! native image against the reference decode (`melee_runtime::dat`, which `fighter_data` / `stage` use) and the
//! raw image against the file. Needs the disc (`GW_ISO_VANILLA`); without it the tests pass vacuously.
#[path = "../../melee_core/test_support/disc.rs"]
mod disc;
use melee_engine::{gcm::Gcm, Host};
use melee_runtime::dat::DatArchive;
use melee_runtime::tu::{lbarchive, lbfile};
use melee_types::{ftCo_DatAttrs, ftData, it_804D6D20_t, EF_DAT_Entry, UnkStageDat, HSD_Archive};

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The host is a global: tests take the lock and boot a fresh one.
fn boot() -> Option<std::sync::MutexGuard<'static, ()>> {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let iso = disc::iso("archive_host")?;
    let disc = Gcm::open(&iso).ok()?;
    Host::boot(disc, 256 << 20).unwrap();
    melee_engine::install();
    // the retail data tables (ftData_803C1F40's Pl*.dat symbol names, ...), as Engine's boot installs them (fe6a566a0)
    melee_link::install_retail();
    Some(g)
}

/// lbFile_LoadInto -> lbArchive_InitializeDAT, as `lbArchive_LoadSymbols` does. Returns (archive, file bytes, size).
unsafe fn load(name: &str) -> (*mut HSD_Archive, Vec<u8>, usize) {
    let h = Host::get();
    let c = std::ffi::CString::new(name).unwrap();
    let want = Gcm::entry_len(&h.disc, h.disc.entry_for_path(name)).expect("file on disc") as usize;
    let cap = (want + 31) & !31;
    // HSD_OSInit's sequence: OSInitAlloc, then one heap over the rest of the arena
    let lo = h.os.init_alloc(h.os.arena_lo(), h.os.arena_hi(), 4);
    let heap = h.os.create_heap(lo, h.os.arena_hi());
    let buf = h.os.alloc_from_heap(heap, cap as u32);
    let mut size = 0usize;
    assert_eq!(lbfile::lbFile_LoadInto(c.as_ptr(), buf, cap, &mut size), 1);
    assert_eq!(size, want);
    let orig = std::slice::from_raw_parts(buf as *const u8, size).to_vec();
    let rec = h.os.alloc_from_heap(heap, std::mem::size_of::<HSD_Archive>() as u32) as *mut HSD_Archive;
    lbarchive::lbArchive_InitializeDAT(rec, buf, size);
    h.os.destroy_heap(heap); // the blocks stay mapped; the host owns the arena
    (rec, orig, size)
}

fn raw_matches(rec: *mut HSD_Archive, orig: &[u8]) {
    let img = &Host::get().archives[&(rec as usize)];
    let raw = unsafe { std::slice::from_raw_parts(img.raw as *const u8, orig.len()) };
    let data_base = (img.raw + 0x20) as u32;
    let mut expect = orig.to_vec();
    for &r in &img.dat.relocs {
        let at = 0x20 + r as usize;
        let v = u32::from_be_bytes(expect[at..at + 4].try_into().unwrap()).wrapping_add(data_base);
        expect[at..at + 4].copy_from_slice(&v.to_be_bytes());
    }
    // lbArchive_InitializeDAT then locates every extern to NULL: each chain slot reads 0
    for e in &img.dat.externs {
        let mut off = e.offset;
        while off != u32::MAX && (off as usize) < img.dat.data.len() {
            let at = 0x20 + off as usize;
            let next = u32::from_be_bytes(orig[at..at + 4].try_into().unwrap());
            expect[at..at + 4].fill(0);
            off = next;
        }
    }
    assert!(raw == &expect[..], "raw image differs from file + Locate");
    assert!(img.raw + orig.len() <= 1 << 32, "raw image above 4 GiB");
}

fn bytes_of<T>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(p as *const T as *const u8, std::mem::size_of::<T>()) }
}

#[test]
fn fighter_files() {
    let Some(_g) = boot() else { return };
    for (file, sym) in [("PlCa.dat", "ftDataCaptain"), ("PlFx.dat", "ftDataFox")] {
        let (rec, orig, _) = unsafe { load(file) };
        raw_matches(rec, &orig);
        let name = std::ffi::CString::new(sym).unwrap();
        let ft = unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, name.as_ptr()) } as *const ftData;
        assert!(!ft.is_null());
        let reference = DatArchive::parse(&orig).unwrap().load::<ftData>(sym).unwrap();
        let (a, b) = unsafe { (&*(*ft).x0, &*reference.get().x0) };
        assert_eq!(bytes_of::<ftCo_DatAttrs>(a), bytes_of::<ftCo_DatAttrs>(b), "{file}: ftData->x0 differs");
        // the 0x30 hurtbox table: same count
        unsafe { assert_eq!((*(*ft).x30).count, (*(*reference.get()).x30).count) };
    }
}

#[test]
fn stage_item_effect_files() {
    let Some(_g) = boot() else { return };
    let (rec, orig, _) = unsafe { load("GrNLa.dat") };
    raw_matches(rec, &orig);
    let img = &Host::get().archives[&(rec as usize)];
    let c = |s: &str| std::ffi::CString::new(s).unwrap();
    let head = unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, c("map_head").as_ptr()) } as *const UnkStageDat;
    assert!(!head.is_null());
    let reference = DatArchive::parse(&orig).unwrap().load::<UnkStageDat>("map_head").unwrap();
    assert_eq!(bytes_of::<UnkStageDat>(unsafe { &*head }).len(), bytes_of::<UnkStageDat>(reference.get()).len());
    for sym in ["coll_data", "grGroundParam"] {
        assert!(unsafe { !melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, c(sym).as_ptr()).is_null() }, "{sym}");
    }
    // the stage's particle command bank: a native decoded copy (dat_custom "ptclcmd", 12878a285; it was a `raw`
    // symbol answering inside the raw image before)
    let ptcl = unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, c("map_ptcl").as_ptr()) } as usize;
    assert!(ptcl != 0 && !(ptcl > img.raw && ptcl < img.raw + img.raw_len));
    // a public symbol outside the type table is a hard stop that names it (the table grows: ALDYakuAll, once the
    // example, is decoded since 65d986b52, so take whichever public of the file the table does not know)
    let unknown = DatArchive::parse(&orig).unwrap().publics.iter().map(|p| p.name.clone())
        .find(|n| melee_engine::dat_types::type_of(n).is_none());
    if let Some(sym) = unknown {
        let r = std::panic::catch_unwind(|| unsafe {
            // the engine function directly: the TU's `extern "C"` entry cannot unwind
            melee_engine::archive::host_public_address(rec, c(&sym).as_ptr())
        });
        let msg = *r.expect_err("must stop").downcast::<String>().unwrap();
        assert!(msg.contains(sym.as_str()), "{msg}");
    } else {
        eprintln!("GrNLa.dat: every public symbol is in the type table; the hard stop is not exercised");
    }

    let (rec, orig, _) = unsafe { load("ItCo.dat") };
    raw_matches(rec, &orig);
    let it = unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, c("itPublicData").as_ptr()) } as *const it_804D6D20_t;
    assert!(!it.is_null());
    assert!(unsafe { !(*it).x0.is_null() });

    let (rec, orig, _) = unsafe { load("EfCoData.dat") };
    raw_matches(rec, &orig);
    let ef = unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(rec, c("effCommonDataTable").as_ptr()) } as *const EF_DAT_Entry;
    assert!(!ef.is_null());
}

//! melee_runtime::dat on synthetic archives (built here, no disc data), plus an opt-in check
//! against the user's own disc (`GW_ISO_VANILLA`, read locally, never written anywhere).

use melee_core::gc::GcLayout;
use melee_runtime::dat::{DatArchive, DatBuilder, DatError};
use melee_types::{HSD_DObjDesc, HSD_Joint};

fn off<T: GcLayout>(field: &str) -> u32 {
    T::gc_field(field).unwrap_or_else(|| panic!("{}.{field}", T::GC_NAME)).offset
}

/// root joint "root" -> child "kid" (with a DObjDesc) -> next "sib" -> child = root (cycle);
/// the matrix pointer goes to a 3x4 f32 block.
fn joint_tree() -> Vec<u8> {
    let mut b = DatBuilder::new();
    let jsz = HSD_Joint::GC_SIZE as usize;
    let (root, kid, sib) = (b.alloc(jsz, 4), b.alloc(jsz, 4), b.alloc(jsz, 4));
    let dobj = b.alloc(HSD_DObjDesc::GC_SIZE as usize, 4);
    let mtx = b.alloc(48, 4);
    for i in 0..12 {
        b.put_f32(mtx + 4 * i, i as f32 * 0.5 - 1.0);
    }
    for (j, name, flags) in [(root, "root", 0x1234_5678u32), (kid, "kid", 8), (sib, "sib", 0x8000_0000)] {
        let s = b.string(name);
        b.put_ptr(j + off::<HSD_Joint>("class_name"), s);
        b.put_u32(j + off::<HSD_Joint>("flags"), flags);
        let r = off::<HSD_Joint>("rotation");
        b.put_f32(j + r, 0.25);
        b.put_f32(j + r + 4, -0.0);
        b.put_f32(j + r + 8, f32::from_bits(0x7FC0_0001));
        let sc = off::<HSD_Joint>("scale");
        for k in 0..3 {
            b.put_f32(j + sc + 4 * k, 1.0 + k as f32);
        }
        b.put_f32(j + off::<HSD_Joint>("position") + 4, 12.5);
    }
    b.put_ptr(root + off::<HSD_Joint>("child"), kid);
    b.put_ptr(kid + off::<HSD_Joint>("next"), sib);
    b.put_ptr(sib + off::<HSD_Joint>("child"), root);
    b.put_ptr(kid + off::<HSD_Joint>("u"), dobj);
    b.put_ptr(root + off::<HSD_Joint>("mtx"), mtx);
    let dn = b.string("dobj");
    b.put_ptr(dobj + off::<HSD_DObjDesc>("class_name"), dn);
    b.public("root_joint", root);
    b.public("kid_joint", kid);
    b.extern_ref("ext_sym", sib + off::<HSD_Joint>("robjdesc"));
    b.finish()
}

fn cstr(p: *const i8) -> &'static str {
    unsafe { core::ffi::CStr::from_ptr(p) }.to_str().unwrap()
}

#[test]
fn header_and_tables() {
    let f = joint_tree();
    let a = DatArchive::parse(&f).unwrap();
    assert_eq!(a.header.file_size as usize, f.len());
    assert_eq!(a.header.nb_public, 2);
    assert_eq!(a.header.nb_reloc, 9); // 3 names + child, next, child, u, mtx + dobj name
    assert_eq!(a.public_offset("kid_joint"), Some(HSD_Joint::GC_SIZE));
    assert_eq!(a.extern_name(0), Some("ext_sym"));
    assert_eq!(a.extern_name(1), None);
    assert_eq!(a.extern_name(-1), None);
    assert!(a.is_reloc(off::<HSD_Joint>("child")));
}

#[test]
fn joint_graph_decodes_to_native() {
    let f = joint_tree();
    let a = DatArchive::parse(&f).unwrap();
    let l = a.load::<HSD_Joint>("root_joint").unwrap();
    assert!(l.issues.is_empty(), "{:?}", l.issues);
    let root = l.get();
    unsafe {
        assert_eq!(cstr(root.class_name), "root");
        assert_eq!(root.flags, 0x1234_5678);
        assert_eq!(root.rotation.x.to_bits(), 0.25f32.to_bits());
        assert_eq!(root.rotation.y.to_bits(), 0x8000_0000);
        assert_eq!(root.rotation.z.to_bits(), 0x7FC0_0001, "NaN payload kept");
        assert_eq!((root.scale.x, root.scale.y, root.scale.z), (1.0, 2.0, 3.0));
        assert_eq!(root.position.y, 12.5);
        assert!(root.next.is_null());
        // MtxPtr is C `f32 (*)[4]`: a pointer to the first row, so one row decodes (see
        // gcdecode: pointer-to-first-element). Whole matrices need GcDecoder::array(target, 3).
        let m = &*root.mtx;
        assert_eq!(*m, [-1.0, -0.5, 0.0, 0.5]);
        let kid = &*root.child;
        assert_eq!(cstr(kid.class_name), "kid");
        let d = &*kid.u.dobjdesc;
        assert_eq!(cstr(d.class_name), "dobj");
        assert!(d.next.is_null() && d.mobjdesc.is_null());
        let sib = &*kid.next;
        assert_eq!(cstr(sib.class_name), "sib");
        assert_eq!(sib.flags, 0x8000_0000);
        assert_eq!(sib.child as *const HSD_Joint, l.root as *const _, "cycle shares the root object");
    }
    // 3 joints, 1 dobj, 4 strings, and 2 matrix allocations: the field decoder's one row, then
    // HSD_Joint's decode replaces `mtx` with the whole 3-row envelope matrix jobj.c copies.
    assert_eq!(l.arena.len(), 10);
}

#[test]
fn shared_decoder_merges_roots() {
    let f = joint_tree();
    let a = DatArchive::parse(&f).unwrap();
    let mut d = a.decoder();
    let r = d.object::<HSD_Joint>(a.public_offset("root_joint").unwrap());
    let k = d.object::<HSD_Joint>(a.public_offset("kid_joint").unwrap());
    let (_arena, issues) = d.finish();
    assert!(issues.is_empty());
    assert_eq!(unsafe { (*r).child }, k);
}

#[test]
fn rejects_bad_files() {
    let mut f = joint_tree();
    assert!(matches!(DatArchive::parse(&f[..0x10]), Err(DatError::Truncated { .. })));
    // A little-endian header (or a wrong size) is HSD_ArchiveParse's byte-order mismatch.
    let n = f.len() as u32;
    f[0..4].copy_from_slice(&n.to_le_bytes());
    assert!(matches!(DatArchive::parse(&f), Err(DatError::SizeMismatch { .. })));
    f[0..4].copy_from_slice(&n.to_be_bytes());
    // A relocation outside the data block.
    let data_size = u32::from_be_bytes(f[4..8].try_into().unwrap());
    let r0 = 0x20 + data_size as usize;
    f[r0..r0 + 4].copy_from_slice(&(data_size + 8).to_be_bytes());
    assert!(matches!(DatArchive::parse(&f), Err(DatError::BadReloc { .. })));
    assert!(matches!(DatArchive::parse(&joint_tree()).unwrap().load::<HSD_Joint>("nope"),
                     Err(DatError::NoSuchSymbol(_))));
}

#[test]
fn unrelocated_pointer_is_null_and_reported() {
    let mut b = DatBuilder::new();
    let j = b.alloc(HSD_Joint::GC_SIZE as usize, 4);
    b.put_u32(j + off::<HSD_Joint>("child"), 0x40); // not in the relocation table
    b.public("j", j);
    let f = b.finish();
    let l = DatArchive::parse(&f).unwrap().load::<HSD_Joint>("j").unwrap();
    assert!(l.get().child.is_null());
    assert_eq!(l.issues.len(), 1, "{:?}", l.issues);
}

// ---- opt-in: the user's own disc (never committed; nothing is written) ----

/// Read one file out of a GameCube disc image through its FST.
fn iso_file(iso: &str, want: &str) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(iso).ok()?;
    let mut rd = |off: u64, n: usize| -> Vec<u8> {
        let mut v = vec![0; n];
        f.seek(SeekFrom::Start(off)).unwrap();
        f.read_exact(&mut v).unwrap();
        v
    };
    let h = rd(0x424, 8);
    let (fst_off, fst_len) = (u32::from_be_bytes(h[0..4].try_into().unwrap()) as u64,
                              u32::from_be_bytes(h[4..8].try_into().unwrap()) as usize);
    let fst = rd(fst_off, fst_len);
    let n = u32::from_be_bytes(fst[8..12].try_into().unwrap()) as usize;
    let names = &fst[n * 12..];
    for i in 1..n {
        let e = &fst[i * 12..i * 12 + 12];
        if e[0] != 0 {
            continue; // directory
        }
        let no = (u32::from_be_bytes(e[0..4].try_into().unwrap()) & 0xFF_FFFF) as usize;
        let end = names[no..].iter().position(|&c| c == 0)?;
        if names[no..no + end].eq_ignore_ascii_case(want.as_bytes()) {
            let (o, l) = (u32::from_be_bytes(e[4..8].try_into().unwrap()), u32::from_be_bytes(e[8..12].try_into().unwrap()));
            return Some(rd(o as u64, l as usize));
        }
    }
    None
}

fn count_joints(j: *const HSD_Joint, seen: &mut std::collections::HashSet<usize>) -> usize {
    if j.is_null() || !seen.insert(j as usize) {
        return 0;
    }
    let jj = unsafe { &*j };
    1 + count_joints(jj.child, seen) + count_joints(jj.next, seen)
}

/// `GW_ISO_VANILLA=<iso> cargo test -p melee_runtime --test dat -- --ignored`
#[test]
#[ignore = "needs the user's disc image (GW_ISO_VANILLA)"]
fn disc_joint_roots_decode() {
    let iso = std::env::var("GW_ISO_VANILLA").expect("GW_ISO_VANILLA");
    for file in ["PlFxNr.dat", "PlCaNr.dat", "GrNBa.dat", "GrNLa.dat"] {
        let Some(bytes) = iso_file(&iso, file) else { panic!("{file} not on disc") };
        let a = DatArchive::parse(&bytes).unwrap_or_else(|e| panic!("{file}: {e}"));
        let mut roots = 0;
        for s in a.publics.iter().filter(|s| s.name.ends_with("_joint")) {
            let l = a.load_at::<HSD_Joint>(s.offset);
            let n = count_joints(l.root, &mut Default::default());
            let bad: Vec<_> = l.issues.iter().filter(|i| i.contains("past the data")).collect();
            assert!(bad.is_empty(), "{file} {}: {bad:?}", s.name);
            eprintln!("{file} {}: {n} joints, {} objects, {} issues", s.name, l.arena.len(), l.issues.len());
            let mut kinds = std::collections::BTreeMap::new();
            for i in &l.issues {
                let k: String = i.split(" (in ").nth(1).unwrap_or(i).split(" @").next().unwrap().to_string() + if i.starts_with("void") { " void*" } else { " word" };
                *kinds.entry(k).or_insert(0) += 1;
            }
            eprintln!("    issue kinds: {kinds:?}");
            for i in l.issues.iter().filter(|i| std::env::var("DAT_ISSUES").is_ok() && !i.starts_with("void")).take(12) { eprintln!("      {i}"); }
            roots += 1;
        }
        eprintln!("{file}: {} bytes, {} relocs, {} publics, {roots} joint roots",
                  bytes.len(), a.relocs.len(), a.publics.len());
    }
}

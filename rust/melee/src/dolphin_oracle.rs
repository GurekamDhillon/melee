//! The Dolphin oracle's Rust side (docs/rust-port/notes/dolphin-oracle.md).
//!
//! `melee dolphin-wide --raw FILE.nhraw --out FILE.txt`: decode the GameCube memory records the
//! patched Slippi Dolphin writes (`MELEE_DOLPHIN_NHV1`) into named field lines.
//! `slpcheck --wide-out FILE.txt` writes the same lines from the engine's native state, so the two
//! files compare line by line (`tools/rust-port/dolphin/wide_diff.py`).
//!
//! Line: `<slippi frame> <instance> <field> <value>`; instance `s<slot>` (slot = player*2 + follower),
//! `it<k>` (k-th item of the item p-link, list order), `cam` (game_camera), `stage` (stage_info),
//! `rng` (`*HSD_RandSeedPtr`). Field names and GameCube offsets come from diff_layout.rs
//! (tools/rust-port/diff/gen_layout.py, from the decomp headers through melee_types' generator).
//! Pointers and unions are left out (not comparable across address spaces).

use crate::diff_layout::{T_Camera, T_Fighter, T_Item, T_StageInfo};
use melee_core::gcdecode::{GcDecode, GcDecoder, RelocSet};
use std::fmt::Write as _;
use std::process::ExitCode;

fn raw<T>(v: &T) -> &[u8] {
    // SAFETY: the bytes of a live plain-data value.
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}

/// GameCube bytes -> the native record, every pointer null.
fn decode<T: GcDecode>(gc: &[u8]) -> T {
    let relocs = RelocSet::default();
    // the generator's GC size can exceed the retail record (port-only fields appended at the end): zero-pad
    let mut buf = gc.to_vec();
    buf.resize(buf.len().max(T::GC_BYTES as usize), 0);
    let mut d = GcDecoder::new(&buf, &relocs);
    T::gc_decode(&mut d, 0)
}

/// The named lines of one native record.
pub fn lines(out: &mut String, frame: i32, inst: &str, ty: &'static str, tab: &'static [crate::diff::F], native: &[u8]) {
    crate::diff::named(native, ty, tab, &mut |f, v| {
        let _ = writeln!(out, "{frame} {inst} {f} {v}");
    });
}

/// One frame of the engine (slpcheck): fighters by slot, items in list order, camera, stage, seed.
pub fn engine_frame(e: &melee_engine::Engine, frame: i32, wide: bool, out: &mut String) {
    let _ = writeln!(out, "{frame} rng seed {:08x}", unsafe { *melee_runtime::tu::random::HSD_RandCurrentSeed(melee_runtime::tu::random::global::state()) });
    let _ = e;
    for slot in 0..6 {
        for j in 0..2 {
            // nhv1.md 5.2: Player_GetEntityAtIndex(i, j), the slot = i*2 + j
            let g = melee_fighter::player::get_entity_at_index(slot, j);
            // SAFETY: a live fighter GObj of the sim, between ticks.
            unsafe {
                if g.is_null() || (*g).user_data.is_null() {
                    continue;
                }
                let b = std::slice::from_raw_parts((*g).user_data as *const u8, std::mem::size_of::<melee_types::Fighter>());
                lines(out, frame, &format!("s{}", slot * 2 + j), "Fighter", T_Fighter, b);
                if wide && std::env::var_os("MELEE_WIDE_BONES").is_some() {
                    bones(out, frame, slot * 2 + j, &*((*g).user_data as *const melee_types::Fighter));
                }
            }
        }
    }
    for (k, b) in e.item_bytes().iter().enumerate() {
        lines(out, frame, &format!("it{k}"), "Item", T_Item, b);
    }
    if wide {
        // SAFETY: the sim is between ticks (single-threaded).
        unsafe {
            lines(out, frame, "cam", "Camera", T_Camera, raw(&*std::ptr::addr_of!(melee_world::tu::camera::game_camera)));
            lines(out, frame, "stage", "StageInfo", T_StageInfo, raw(&*std::ptr::addr_of!(melee_world::tu::ground::stage_info)));
        }
    }
}

/// `MELEE_WIDE_BONES` (with `--wide`): each part's jobj world matrix, `<frame> s<k> bone[<i>].mtx[<n>] <bits>`, the form
/// the patched Dolphin writes with `MELEE_DOLPHIN_NHV1_BONES` (notes/dolphin-oracle.md "Bone dump").
unsafe fn bones(out: &mut String, frame: i32, slot: i32, fp: &melee_types::Fighter) {
    let tbl = melee_fighter::hooks::ft_parts_table();
    let e = *tbl.add(fp.kind as usize);
    if e.is_null() || fp.parts.is_null() {
        return;
    }
    for i in 0..(*e).parts_num as usize {
        let j = (*fp.parts.add(i)).joint;
        if j.is_null() {
            continue;
        }
        for (n, v) in (*j).mtx.iter().flatten().enumerate() {
            let _ = writeln!(out, "{frame} s{slot} bone[{i}].mtx[{n}] {:08x}", v.to_bits());
        }
    }
}

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.p..self.p + n)?;
        self.p += n;
        Some(s)
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

const FIGHTER_GC: usize = 0x23EC;
const ITEM_GC: usize = 0xFCC;
const CAMERA_GC: usize = 0x39C;
const STAGE_GC: usize = 0x748;

/// `melee dolphin-wide --raw FILE --out FILE [--from F] [--to F]`
pub fn dolphin_wide(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let (Some(inp), Some(outp)) = (get("--raw"), get("--out")) else {
        eprintln!("usage: melee dolphin-wide --raw FILE.nhraw --out FILE.txt [--from F] [--to F]");
        return ExitCode::from(2);
    };
    let from: i32 = get("--from").and_then(|v| v.parse().ok()).unwrap_or(i32::MIN);
    let to: i32 = get("--to").and_then(|v| v.parse().ok()).unwrap_or(i32::MAX);
    let data = match std::fs::read(&inp) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("melee dolphin-wide: {inp}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if data.get(..8) != Some(b"NHRAW001") {
        eprintln!("melee dolphin-wide: {inp}: not an NHRAW001 file");
        return ExitCode::from(2);
    }
    let mut r = Rd { b: &data, p: 8 };
    let wide = r.u32().unwrap_or(0) & 1 != 0;
    let mut out = String::new();
    let mut file = match std::fs::File::create(&outp) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("melee dolphin-wide: {outp}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut frames = 0;
    while r.p < data.len() {
        let rec = (|| {
            if r.u32()? != 0x4D52_4652 {
                return None;
            }
            let frame = r.u32()? as i32;
            let _unk0 = r.u32()?;
            let seed = r.u32()?;
            let (nf, ni) = (r.u32()?, r.u32()?);
            let show = frame >= from && frame <= to;
            if show {
                let _ = writeln!(out, "{frame} rng seed {seed:08x}");
            }
            for _ in 0..nf {
                let h = r.take(8)?;
                let b = r.take(FIGHTER_GC)?;
                if show {
                    let fp: melee_types::Fighter = decode(b);
                    lines(&mut out, frame, &format!("s{}", h[0]), "Fighter", T_Fighter, raw(&fp));
                }
            }
            for k in 0..ni {
                r.take(4)?;
                let b = r.take(ITEM_GC)?;
                if show {
                    let ip: melee_types::Item = decode(b);
                    lines(&mut out, frame, &format!("it{k}"), "Item", T_Item, raw(&ip));
                }
            }
            if wide {
                let c = r.take(CAMERA_GC)?;
                let s = r.take(STAGE_GC)?;
                if show {
                    let cam: melee_types::Camera = decode(c);
                    lines(&mut out, frame, "cam", "Camera", T_Camera, raw(&cam));
                    let st: melee_types::StageInfo = decode(s);
                    lines(&mut out, frame, "stage", "StageInfo", T_StageInfo, raw(&st));
                }
            }
            Some(())
        })();
        if rec.is_none() {
            eprintln!("melee dolphin-wide: truncated or bad record at byte {}", r.p);
            break;
        }
        frames += 1;
        if out.len() > 1 << 24 {
            use std::io::Write;
            let _ = file.write_all(out.as_bytes());
            out.clear();
        }
    }
    use std::io::Write;
    let _ = file.write_all(out.as_bytes());
    eprintln!("melee dolphin-wide: {frames} frames");
    ExitCode::SUCCESS
}

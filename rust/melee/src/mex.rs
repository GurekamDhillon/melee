//! An m-ex fighter's `ftFunction` code (from its gdmod's PlXx.dat) run in the native match.
//!
//! The C build runs these blobs through pc/platform/gw_ppc.c over big-endian MEM1
//! (gw_mex_ftfunction_runtime.c: gw_mex_interp_run, r3 = gobj, r2 = mexData, r1 = guest stack).
//! Here the same PPC32 runs through mex_import's IR interpreter (`ppc::Machine`, checked op for
//! op against gw_ppc.c) over `remap::GuestSpace`, which gives the native little-endian `Fighter`
//! and its `HSD_GObj` guest addresses and rewrites every field access through the GameCube offset
//! tables. So the mod writes the very `Fighter` the native match simulates.
//!
//! Wired slots (gw_mex_ftfunction_runtime.c GW_MEX_SLOT_*): 0 onLoad (Fighter_Create, after the
//! fighter is set up), 23 onFrame (once per fighter per frame, Fighter_8006A360's
//! Mex_OnFrameDispatch). Out-of-blob calls go through [`MexBridge`]: memset/memcpy are bound;
//! any other vanilla (or m-ex DOL-side) function ends that callback with `Unresolved` (its stores
//! made before the call stay, as they would have in the C), and is counted by name.
//! r2 (m-ex's mexData holder in the C build) and r13 (vanilla small data) have no backing here:
//! an access through them faults the callback and is counted.

use mex_import::mexfn::{Blob, Owner};
use mex_import::ppc::{Bridge, Cpu, Fault, Machine};
use mex_import::remap::GuestSpace;
use melee_types::{Fighter, HSD_GObj};
use std::collections::BTreeMap;

/// The m-ex internal fighter id of `name` (MxDt.dat, from the mounted base gdmod), or None for a
/// vanilla fighter / no MxDt.dat. Matched by the fighter's PlXx.dat (fighter_data::files_for).
pub fn internal_id(disc: &mut crate::disc::Disc, name: &str) -> Result<Option<u32>, String> {
    let Some((file, _, _)) = crate::fighter_data::files_for(name) else { return Ok(None) };
    if disc.mods.as_ref().and_then(|m| m.lookup("MxDt.dat")).is_none() {
        return Ok(None);
    }
    let t = mex_import::hsd::Tables::parse(&disc.read("MxDt.dat")?).map_err(|e| format!("MxDt.dat: {e}"))?;
    let ext = t.ext_of_internal();
    for k in mex_import::hsd::FIRST_NEW_FIGHTER..t.meta["internal_id_count"] - 6 {
        let (_, files) = t.fighter_claims(k, ext.get(&(k as u8)).copied());
        if files.first().is_some_and(|f| f.eq_ignore_ascii_case(file)) {
            return Ok(Some(k as u32));
        }
    }
    Ok(None)
}

pub const SLOT_ON_LOAD: u32 = 0;
pub const SLOT_ON_FRAME: u32 = 23;
pub const SLOT_NAMES: [&str; 46] = [
    "onLoad", "OnRespawn", "onDestroy", "MoveLogic", "SpecialN", "SpecialNAir", "SpecialS", "SpecialSAir", "SpecialHi",
    "SpecialHiAir", "SpecialLw", "SpecialLwAir", "onAbsorb", "OnItemPickup", "onMakeItemInvisible", "onMakeItemVisible",
    "OnItemRelease", "OnItemPickup2", "onUnknownItemRelated", "onApplyHeadItem", "onRemoveHeadItem", "EyeTextureDamaged",
    "EyeTextureNormal", "onFrame", "onActionStateChange", "onReapplyAttr", "onModelRender", "onShadowRender",
    "onUnknownMultijump", "onActionStateChangeWhileEyeTextureIsChanged", "onTwoEntryTable", "onFloat", "onDoubleJump",
    "onZair", "onLanding", "onFSmash", "onUSmash", "onDSmash", "onGetExtResultAnim", "onIndexExtResultAnim",
    "MoveLogicDemo", "onIntroL", "onIntroR", "onTaunt", "onCatch", "GetTrailData",
];

/// Guest layout. The blob is relocated to EXEC_BASE, inside rel24 reach of the game's text (as the
/// C build loads it into MEM1), so its `bl` to a vanilla function decodes to that function.
pub const EXEC_BASE: u32 = 0x8180_0000;
const STACK_BASE: u32 = 0x8170_0000;
const STACK_SIZE: u32 = 0x10000;
const R2_MEXDATA: u32 = 0x81F0_0000; // unbacked: see the module comment
const FIGHTER_BASE: u32 = 0x8200_0000; // + slot * 0x4000
const GOBJ_BASE: u32 = 0x8210_0000; // + slot * 0x100
const ATTRS_BASE: u32 = 0x8220_0000; // + slot * 0x1000: fp->dat_attrs_backup
/// fighter.c: HSD_ObjAllocInit(&fighter_dat_attrs_alloc_data, 0x424, 4).
const ATTRS_SIZE: usize = 0x424;
const BUDGET: u32 = 2_000_000;

const DAT_BASE: u32 = 0x8300_0000; // the fighter's PlXx.dat data section, relocated
/// Fighter.ft_data's GC offset (fp->ft_data, the `ftData *` the C reads from the PlXx.dat).
const FIGHTER_FT_DATA: u32 = 0x10C;

const MEX_INDEX_FIGHTER_ITEM: u32 = 0x803D_7058; // gw_mex_ftfunction_runtime.c GW_MEX_GUEST_INDEX_ITEM
const MEMSET: u32 = 0x8000_3100;
const MEMCPY: u32 = 0x8000_31F4;

/// Out-of-blob calls. Bound: memset / memcpy (pure guest memory, MSL semantics, r3 returned) and
/// m-ex's MEX_IndexFighterItem (gw_mex_shim_index_item: registers an article's ItemDesc for item
/// creation; here into [`MexBridge::articles`], since the native match has no m-ex item table and
/// spawns no articles). Anything else is counted by target and ends the callback.
pub struct MexBridge {
    lo: u32,
    hi: u32,
    pub bound: BTreeMap<u32, u64>,
    /// MEX_IndexFighterItem calls: (fighter kind, item id) -> ItemDesc guest address.
    pub articles: BTreeMap<(u32, u32), u32>,
    pub unresolved: BTreeMap<u32, u64>,
}

impl Bridge<GuestSpace> for MexBridge {
    fn in_blob(&self, a: u32) -> bool {
        (self.lo..self.hi).contains(&a)
    }
    fn call(&mut self, cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<(), Fault> {
        use mex_import::ppc::Mem;
        let (d, a, n) = (cpu.gpr[3], cpu.gpr[4], cpu.gpr[5]);
        match target {
            MEMCPY => {
                for k in 0..n {
                    let b = mem.ld8(a.wrapping_add(k))?;
                    mem.st8(d.wrapping_add(k), b)?;
                }
            }
            MEMSET => {
                for k in 0..n {
                    mem.st8(d.wrapping_add(k), a as u8)?;
                }
            }
            MEX_INDEX_FIGHTER_ITEM => {
                self.articles.insert((d, n), a);
                cpu.gpr[3] = 0;
            }
            _ => {
                let v = mex_import::mexfn::vanilla_target(target);
                *self.unresolved.entry(v).or_default() += 1;
                return Err(Fault::Unresolved(v));
            }
        }
        *self.bound.entry(target).or_default() += 1;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct SlotStats {
    pub calls: u64,
    pub returned: u64,
    pub insns: u64,
    /// Fault kind -> count ("unresolved 8008A2BC Fighter_...", "mem 81800010", ...).
    pub faults: BTreeMap<String, u64>,
}

pub struct MexFighter {
    pub file: String,
    pub blob: Blob,
    space: GuestSpace,
    bridge: MexBridge,
    /// The fighter's `HSD_GObj` (the C's gobj; user_data -> the native Fighter). Boxed: stable.
    pub gobj: Box<HSD_GObj>,
    /// Guest address of the fighter's ftData (in the DAT region).
    pub ft_data: u32,
    pub stats: BTreeMap<u32, SlotStats>,
    /// Every Fighter field access the mod made (GC offset, store) -> count.
    pub fighter_access: BTreeMap<(u32, bool), u64>,
}

impl MexFighter {
    /// The `ftFunction` blob of `file` (an m-ex fighter's PlXx.dat), None when it has none.
    /// `data_name` is the fighter's ftData public (`ftDataWolf`).
    pub fn load(file: &str, data_name: &str, dat: &[u8]) -> Result<Option<Self>, String> {
        let ar = mex_import::hsd::Archive::parse(dat).map_err(|e| format!("{file}: {e}"))?;
        let Some(off) = ar.public("ftFunction") else { return Ok(None) };
        let blob = mex_import::mexfn::parse_at_base(&ar, off, Owner::Fighter, format!("{file}:ftFunction"), EXEC_BASE)
            .map_err(|e| format!("{file}: ftFunction: {e}"))?;
        let mut space = GuestSpace::new();
        space.add_bytes(EXEC_BASE, blob.code.clone());
        space.add_bytes(STACK_BASE, vec![0; STACK_SIZE as usize]);
        // The DAT as HSD_ArchiveParse leaves it in memory: big-endian data, every relocated word
        // made absolute. The mod reads its ftData (and attribute / article tables) from here.
        let mut data = ar.data.clone();
        for &o in &ar.relocs {
            let o = o as usize;
            if o + 4 <= data.len() {
                let w = u32::from_be_bytes(data[o..o + 4].try_into().unwrap());
                data[o..o + 4].copy_from_slice(&w.wrapping_add(DAT_BASE).to_be_bytes());
            }
        }
        space.add_bytes(DAT_BASE, data);
        let ft_data = DAT_BASE + ar.public(data_name).ok_or(format!("{file}: no {data_name}"))?;
        space.trace = Some(vec![]);
        let bridge = MexBridge { lo: EXEC_BASE, hi: EXEC_BASE + blob.code.len() as u32, bound: BTreeMap::new(), articles: BTreeMap::new(), unresolved: BTreeMap::new() };
        Ok(Some(Self { file: file.into(), blob, space, bridge, ft_data, gobj: Box::new(HSD_GObj::zeroed()), stats: BTreeMap::new(), fighter_access: BTreeMap::new() }))
    }

    /// The code entry of override slot `slot` (absolute-address hooks are not slots).
    pub fn slot_target(&self, slot: u32) -> Option<u32> {
        self.blob.overrides.iter().find(|o| o.0 == slot).map(|o| EXEC_BASE.wrapping_add(o.1))
    }

    pub fn overrides(&self) -> Vec<String> {
        self.blob
            .overrides
            .iter()
            .map(|&(s, off)| {
                let name = if s & 0x8000_0000 == 0 { SLOT_NAMES.get(s as usize).copied().unwrap_or("?").to_string() } else { format!("hook {s:08X}") };
                let sym = self.blob.symbols.iter().find(|x| x.0 == off).map(|x| x.2.as_str()).unwrap_or("");
                format!("{name}={sym}@{off:#x}")
            })
            .collect()
    }

    /// Run override `slot` with r3 = the fighter's gobj (gw_mex_interp_run). The native Fighter
    /// is mapped at a fixed guest address per slot, so a gobj/fighter pointer the mod keeps in its
    /// own data stays valid across frames.
    pub fn run(&mut self, slot: u32, player: i32, fp: &mut Fighter) -> Option<u32> {
        let target = self.slot_target(slot)?;
        // The C's gobj <-> user_data link, on the native objects.
        self.gobj.user_data = fp as *mut Fighter as *mut core::ffi::c_void;
        self.gobj.classifier = 4; // HSD_GOBJ_CLASS_FIGHTER (gobj.h); p_link 8 = HSD_GOBJ_PLINK_FIGHTER
        self.gobj.p_link = 8;
        fp.gobj = &mut *self.gobj;
        let fb = FIGHTER_BASE + player as u32 * 0x4000;
        let gb = GOBJ_BASE + player as u32 * 0x100;
        // Registered once: the boxes do not move, and the regions keep their padding shadows (GC
        // bytes the mod may park data in) across calls, as MEM1 would.
        if !self.space.regions.iter().any(|r| r.base == fb) {
            // Fighter_Create: fp->dat_attrs_backup = HSD_ObjAlloc(0x424-byte pool), mod-visible
            // as plain guest bytes (the native match keeps no attribute buffer of its own).
            let ab = ATTRS_BASE + player as u32 * 0x1000;
            self.space.add_bytes(ab, vec![0; ATTRS_SIZE]);
            fp.dat_attrs_backup = self.space.bytes(ab).unwrap().as_ptr() as *mut core::ffi::c_void;
            // SAFETY: both objects outlive the space (the Player owns the Fighter box, this the
            // gobj box); every pointer field the guest can follow is null or one of the two (the
            // native match keeps fp's other pointers null).
            unsafe {
                self.space.add_native(Some(fb), "Fighter", fp as *mut Fighter as *mut u8).ok()?;
                self.space.add_native(Some(gb), "HSD_GObj", &mut *self.gobj as *mut HSD_GObj as *mut u8).ok()?;
            }
            // fp->ft_data: the native match keeps it null (it reads the DAT through FighterFiles);
            // the guest sees the DAT region's ftData.
            self.space.ptr_override.insert((fb, FIGHTER_FT_DATA), self.ft_data);
        }
        let mut cpu = Cpu::default();
        cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100;
        cpu.gpr[2] = R2_MEXDATA;
        cpu.gpr[3] = gb;
        cpu.lr = 0; // return to the native caller
        cpu.pc = target;
        let mut m = Machine::new(cpu);
        let mut n = 0u64;
        let trace = std::env::var_os("MELEE_MEX_TRACE").is_some() && self.stats.get(&slot).map_or(0, |s| s.calls) < 2;
        let mut res = None;
        let mut fault = None;
        loop {
            if n >= BUDGET as u64 {
                fault = Some(Fault::Budget);
                break;
            }
            n += 1;
            if trace {
                eprintln!("mex trace slot {slot}: pc {:08X} r3 {:08X} lr {:08X}", m.cpu.pc, m.cpu.gpr[3], m.cpu.lr);
            }
            match m.step(&mut self.space, &mut self.bridge) {
                Ok(mex_import::ppc::Step::Continue) => {}
                Ok(mex_import::ppc::Step::Return) => {
                    res = Some(m.cpu.gpr[3]);
                    break;
                }
                Err(f) => {
                    fault = Some(f);
                    break;
                }
            }
        }
        let fk = fault.map(|f| match f {
            Fault::Unresolved(t) => format!("unresolved {t:08X} {}", mex_import::layout::vanilla_name(t).unwrap_or("?")),
            Fault::Mem(a) => format!("mem {a:08X} at {}", self.where_(m.cpu.pc.wrapping_sub(4))),
            other => format!("{other:?} at {}", self.where_(m.cpu.pc.wrapping_sub(4))),
        });
        let st = self.stats.entry(slot).or_default();
        st.calls += 1;
        st.insns += n;
        match fk {
            None => st.returned += 1,
            Some(k) => {
                *st.faults.entry(k).or_default() += 1;
            }
        }
        if let Some(t) = &mut self.space.trace {
            for r in t.drain(..) {
                if r.rec == "Fighter" {
                    *self.fighter_access.entry((r.gc_off, r.store)).or_default() += 1;
                }
            }
        }
        fp.gobj = core::ptr::null_mut();
        res
    }

    /// `function+0xoff` for a guest pc inside the blob.
    fn where_(&self, pc: u32) -> String {
        let off = pc.wrapping_sub(EXEC_BASE);
        match self.blob.symbols.iter().rev().find(|s| s.0 <= off) {
            Some(s) => format!("{}+{:#x}", s.2, off - s.0),
            None => format!("{pc:08X}"),
        }
    }

    pub fn report(&self) -> String {
        let mut s = format!("mex: {} ftFunction: {} bytes, overrides {}\n", self.file, self.blob.code.len(), self.overrides().join(" "));
        for (slot, st) in &self.stats {
            s += &format!(
                "mex:   slot {slot} {}: {} calls, {} returned, {} insns",
                SLOT_NAMES.get(*slot as usize).unwrap_or(&"?"),
                st.calls,
                st.returned,
                st.insns
            );
            for (k, v) in &st.faults {
                s += &format!("; {k} x{v}");
            }
            s += "\n";
        }
        for ((k, id), d) in &self.bridge.articles {
            s += &format!("mex:   MEX_IndexFighterItem(kind {k}, item {id}) -> ItemDesc {d:08X}
");
        }
        for (t, n) in &self.bridge.bound {
            let name = if *t == MEX_INDEX_FIGHTER_ITEM { "MEX_IndexFighterItem" } else { mex_import::layout::vanilla_name(*t).unwrap_or("?") };
            s += &format!("mex:   bound call {t:08X} {name} x{n}
");
        }
        let loads: u64 = self.fighter_access.iter().filter(|e| !e.0 .1).map(|e| e.1).sum();
        let stores: u64 = self.fighter_access.iter().filter(|e| e.0 .1).map(|e| e.1).sum();
        s += &format!("mex:   native Fighter accesses through the rewrite: {loads} loads, {stores} stores ({} distinct)\n", self.fighter_access.len());
        s
    }
}

//! The HSD particle system: `src/sysdolphin/baselib/particle.c` (the bytecode interpreter
//! `hsd_8039930C`, particle creation and the per-frame walk), `generator.c` (generators: the
//! list, the per-frame update `hsd_8039EE24` and the emitter `hsd_8039DAD4`) and `psappsrt.c`.
//!
//! Every draw these make comes from the simulation RNG, so particle state that the frame hash
//! never reads still reaches it through `seed` (a stage's background emitter draws every frame).
//!
//! Native shape:
//! - The C keeps raw pointers into the effect DAT (`HSD_PSCmdList*`, the bytecode `u8*`). Here a
//!   bank is parsed once from the DAT bytes ([`PsBankData::locate`] = `psInitDataBankLocate`,
//!   including its kind-bit rewrite) and [`PsSystem::load_bank`] = `psInitDataBankLoad`. A
//!   command list's bytecode is a [`CmdRef`] (shared DAT bytes + offset); a float operand is read
//!   big-endian byte by byte as `psReadFloat` does.
//! - Generators, particles and AppSRTs live in arenas; `next` links are arena ids. Freed slots
//!   are reused last-in first-out, as `HSD_ObjAlloc`'s free list does, so id identity follows the
//!   C's pointer identity (the particle walk compares links).
//! - Calls that leave the particle system go through [`PsHost`]: the RNG, the JObj world matrix
//!   (`HSD_JObjSetupMatrix` + `jobj->mtx`), JObj refcounts, the generator-created hook
//!   (`hsd_804D7900`, efLib's `efLib_Cb_PtclAppSRTHook`) and the hooks no ported caller sets yet
//!   (user funcs, particle callbacks, custom generator shapes, the billboard camera, the particle
//!   JObj slots), whose default host methods panic with a TODO.
//!
//! FMA: the shipping build fuses exactly the sites `tools/rust-port/fma_sites.py` lists for
//! these files (by source line: generator.c 292-306 D71C, 395-396 481 527 550 559 619 649 665
//! 821 825 843-850 860-861 941-943 DAD4, 1011 EE24, 1194-1197 F05C; particle.c 556 573 591-595
//! 80398F8C, 613 624-626 803991D8, 1300 1306 1312 1428 1485-1494 1602 1620-1621 1978 2015-2016
//! 2779 2895 2905-2909 8039930C, plus the f64 Newton steps of the inline sqrtf). Each is written
//! as [`fmadds`] below with the line number; nothing else is fused.

use std::sync::Arc;

use melee_core::fp::fmadds;

use crate::gekko::{cosf, sinf, sqrtf, tanf};
use crate::hsd_mtx::{self, fabsf_bitwise};
use crate::lbtrigf::atan2f;
use crate::mtx::{self, Mtx, Quaternion, Vec3};

pub const NUM_BANKS: usize = 65;
pub const NUM_LINKS: usize = 16;

/// `HSD_ParticleKind` bits.
pub const KIND_TORNADO: u32 = 1 << 2;
pub const KIND_TEX_EDGE: u32 = 1 << 3;
pub const KIND_COM_TLUT: u32 = 1 << 4;
pub const KIND_MIRROR_S: u32 = 1 << 5;
pub const KIND_MIRROR_T: u32 = 1 << 6;
pub const KIND_PRIM_ENV: u32 = 1 << 7;
pub const KIND_TEX_INTERP_NEAR: u32 = 1 << 9;
pub const KIND_DISP_TEXTURE: u32 = 1 << 10;
pub const KIND_TEX_FLIP_S: u32 = 1 << 18;
pub const KIND_TEX_FLIP_T: u32 = 1 << 19;
pub const KIND_TRAIL: u32 = 1 << 20;
pub const KIND_DIR_VEC: u32 = 1 << 21;

const M_PI: f64 = std::f64::consts::PI;
const M_PI_2: f64 = std::f64::consts::FRAC_PI_2;

/// A JObj handle owned by the host (the JObj system is not this module's).
pub type JObjRef = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SrtId(pub u32);

/// `GXColor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

fn be16(d: &[u8], at: u32) -> u16 {
    let at = at as usize;
    match d.get(at..at + 2) {
        Some(b) => u16::from_be_bytes([b[0], b[1]]),
        None => 0,
    }
}
fn be32(d: &[u8], at: u32) -> u32 {
    let at = at as usize;
    match d.get(at..at + 4) {
        Some(b) => u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
        None => 0,
    }
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}

/// A command list's bytecode: the DAT bytes and the offset of `HSD_PSCmdList::cmdList`.
/// Equality is the C's pointer equality (same DAT, same offset).
#[derive(Clone, Debug)]
pub struct CmdRef {
    pub data: Arc<[u8]>,
    pub off: u32,
}

impl PartialEq for CmdRef {
    fn eq(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.data, &o.data) && self.off == o.off
    }
}

impl CmdRef {
    #[inline]
    fn byte(&self, pc: u32) -> u8 {
        self.data.get((self.off + pc) as usize).copied().unwrap_or(0)
    }
}

/// `HSD_PSCmdList` (GC 0x3C + bytecode).
#[derive(Clone, Debug)]
pub struct CmdList {
    pub type_: u16,
    pub tex_group: u16,
    pub gen_life: u16,
    pub life: u16,
    pub kind: u32,
    pub grav: f32,
    pub fric: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub radius: f32,
    pub angle: f32,
    pub random: f32,
    pub size: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    pub cmd_list: CmdRef,
}

impl melee_core::gc::GcLayout for CmdList {
    const GC_NAME: &'static str = <melee_types::_HSD_PSCmdList as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::_HSD_PSCmdList as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::_HSD_PSCmdList as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::_HSD_PSCmdList as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl CmdList {
    fn parse(data: &Arc<[u8]>, p: u32, kind: u32) -> Self {
        let d = &data[..];
        CmdList {
            type_: be16(d, p),
            tex_group: be16(d, p + 2),
            gen_life: be16(d, p + 4),
            life: be16(d, p + 6),
            kind,
            grav: bef(d, p + 0xC),
            fric: bef(d, p + 0x10),
            vx: bef(d, p + 0x14),
            vy: bef(d, p + 0x18),
            vz: bef(d, p + 0x1C),
            radius: bef(d, p + 0x20),
            angle: bef(d, p + 0x24),
            random: bef(d, p + 0x28),
            size: bef(d, p + 0x2C),
            param1: bef(d, p + 0x30),
            param2: bef(d, p + 0x34),
            param3: bef(d, p + 0x38),
            cmd_list: CmdRef { data: data.clone(), off: p + 0x3C },
        }
    }
}

/// The texture bank (`texBank`): `[num_groups, group offsets...]`, each group an
/// `HSD_PSTexGroup` (GC 0x18 + `texTable`). Only what the simulation reads is exposed:
/// `palflag` and whether `texTable[i]` is non-null.
#[derive(Clone, Debug)]
pub struct TexBank {
    data: Arc<[u8]>,
    /// Absolute offset of each group (`None` = NULL entry).
    groups: Vec<Option<u32>>,
}

impl TexBank {
    pub fn num_groups(&self) -> i32 {
        self.groups.len() as i32
    }
    /// `psTexGroupArray[bank][i]` (the C does not bound-check `i`; past the table it reads the
    /// following words, here `None`).
    fn group(&self, i: u32) -> Option<u32> {
        self.groups.get(i as usize).copied().flatten()
    }
    /// `tg->palflag`.
    pub fn palflag(&self, i: u32) -> Option<u16> {
        self.group(i).map(|g| be16(&self.data, g + 0x16))
    }
    /// `tg->texTable[n] != NULL` (relocation keeps NULL entries NULL).
    fn tex_present(&self, g: u32, n: u32) -> bool {
        be32(&self.data, g + 0x18 + 4 * n) != 0
    }
}

/// One located bank: `psInitDataBankLocate` applied to a `cmdBank` / `texBank` pair.
#[derive(Clone, Debug)]
pub struct PsBankData {
    /// First valid index (`count` of a version 0x40-0x43 bank, 0 for version 0).
    pub base: i32,
    /// `psCmdListArray[bank]` once loaded (one past the last index).
    pub end: i32,
    pub cmds: Vec<Option<CmdList>>,
    pub tex: TexBank,
}

impl PsBankData {
    /// `psInitDataBankLocate(cmdBank, texBank, NULL)`: relocate (natively, resolve) the command
    /// list table and the texture groups, and rewrite each command list's kind bits 25-27 to 4
    /// (`kind = (kind & 0xF1FFFFFF) | 0x08000000`). `data` is the DAT's data section; `cmd_off`
    /// / `tex_off` the two public symbols (a stage's `map_ptcl` / `map_texg`). The form bank is
    /// not ported (no caller passes one; TODO(group1 psform)).
    pub fn locate(data: &Arc<[u8]>, cmd_off: u32, tex_off: u32) -> Result<Self, String> {
        let d = &data[..];
        let version = be16(d, cmd_off);
        let (base, num, table) = match version {
            0 => (0i32, be32(d, cmd_off + 4) as i32, cmd_off + 8),
            0x40..=0x43 => {
                let count = be32(d, cmd_off + 4) as i32;
                (count, be32(d, cmd_off + 8) as i32, cmd_off + 12)
            }
            _ => return Err(format!("psInitDataBanks: unknown version {version:#x}")),
        };
        let mut cmds = Vec::with_capacity(num.max(0) as usize);
        for i in 0..num.max(0) as u32 {
            let rel = be32(d, table + 4 * i);
            // Version 0 relocates every entry unconditionally (a 0 becomes cmdBank itself);
            // versions 0x40+ keep 0 as NULL.
            if rel == 0 && version != 0 {
                cmds.push(None);
                continue;
            }
            let p = cmd_off + rel;
            let kind = (be32(d, p + 8) & 0xF1FF_FFFF) | 0x0800_0000;
            cmds.push(Some(CmdList::parse(data, p, kind)));
        }
        let ngroups = be32(d, tex_off);
        let groups = (0..ngroups)
            .map(|k| {
                let rel = be32(d, tex_off + 4 + 4 * k);
                (rel != 0).then_some(tex_off + rel)
            })
            .collect();
        Ok(PsBankData { base, end: base + num, cmds, tex: TexBank { data: data.clone(), groups } })
    }

    /// `ptclref_804D0E5C[bank][idx]` for `idx < psCmdListArray[bank]`.
    pub fn cmd(&self, idx: i32) -> Option<&CmdList> {
        if idx < self.base {
            // The C indexes the table from `cmdBank + 3 - count` and reads the header words
            // (or memory before the bank) as command-list pointers.
            panic!("ptclref[bank][{idx}] below the bank's first index {} (C reads the bank header as a pointer)", self.base);
        }
        self.cmds.get((idx - self.base) as usize).and_then(|c| c.as_ref())
    }
}

/// A loaded bank slot (`psInitDataBankLoad`): the located data, and `hsd_804D0948[bank]` (the
/// optional index remap table).
#[derive(Clone, Debug)]
pub struct Bank {
    pub data: Arc<PsBankData>,
    pub refs: Option<Arc<[u32]>>,
}

/// `HSD_psAppSRT` (GC 0xA4).
#[derive(Clone, Debug, Default)]
pub struct AppSrt {
    pub next: Option<SrtId>,
    pub gp: Option<GenId>,
    pub translate: Vec3,
    pub rot: Quaternion,
    pub scale: Vec3,
    pub status: u8,
    pub frame_num: u8,
    pub used_count: u16,
    pub mmtx: Mtx,
    pub ssx: f32,
    pub ssy: f32,
    pub x6c: [f32; 12],
    /// `freefunc` (a host callback id; none of the ported callers sets one).
    pub freefunc: Option<u32>,
    pub idnum: u16,
    pub x_a2: u8,
}

impl melee_core::gc::GcLayout for AppSrt {
    const GC_NAME: &'static str = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// `HSD_Generator` (GC 0x94). `aux` is the C union as 13 words (indices: disc min/max 0/1;
/// line x2/y2/z2 0-2; tornado vel 0; rect x y z 0-2, xx..zz 3-11; cone min/max/height 0-2;
/// sphere speed latMid latRange lonMid lonRange 0-4); `aux_flag` is `aux.rect.flag` (+0x90).
#[derive(Clone, Debug)]
pub struct Generator {
    pub next: Option<GenId>,
    pub kind: u32,
    pub random: f32,
    pub count: f32,
    pub jobj: Option<JObjRef>,
    pub gen_life: u16,
    pub type_: u16,
    pub bank: u8,
    pub link_no: u8,
    pub tex_group: u8,
    pub idnum: u16,
    pub life: u16,
    pub cmd_list: Option<CmdRef>,
    pub pos: Vec3,
    pub vel: Vec3,
    pub grav: f32,
    pub fric: f32,
    pub size: f32,
    pub radius: f32,
    pub angle: f32,
    pub num_child: u32,
    pub appsrt: Option<SrtId>,
    /// `userfunc` (`HSD_PSUserFunc*`; a host id).
    pub userfunc: Option<u32>,
    pub callback: Option<u32>,
    pub aux: [f32; 13],
    pub aux_flag: u16,
}

impl melee_core::gc::GcLayout for Generator {
    const GC_NAME: &'static str = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl Generator {
    /// `memset(gen, 0, sizeof *gen)`.
    fn zeroed() -> Self {
        Generator {
            next: None,
            kind: 0,
            random: 0.0,
            count: 0.0,
            jobj: None,
            gen_life: 0,
            type_: 0,
            bank: 0,
            link_no: 0,
            tex_group: 0,
            idnum: 0,
            life: 0,
            cmd_list: None,
            pos: Vec3::default(),
            vel: Vec3::default(),
            grav: 0.0,
            fric: 0.0,
            size: 0.0,
            radius: 0.0,
            angle: 0.0,
            num_child: 0,
            appsrt: None,
            userfunc: None,
            callback: None,
            aux: [0.0; 13],
            aux_flag: 0,
        }
    }
}

/// `HSD_Particle` (GC 0x98).
#[derive(Clone, Debug, Default)]
pub struct Particle {
    pub next: Option<PId>,
    pub kind: u32,
    pub bank: u8,
    pub tex_group: u8,
    pub pose_num: u8,
    pub pal_num: u8,
    pub size_count: u16,
    pub prim_col_count: u16,
    pub env_col_count: u16,
    pub prim_col: Rgba,
    pub env_col: Rgba,
    pub cmd_wait: u16,
    pub loop_count: u8,
    pub link_no: u8,
    pub idnum: u16,
    pub cmd_list: Option<CmdRef>,
    pub cmd_ptr: u16,
    pub cmd_mark_ptr: u16,
    pub cmd_loop_ptr: u16,
    pub life: u16,
    pub vel: Vec3,
    pub grav: f32,
    pub fric: f32,
    pub pos: Vec3,
    pub size: f32,
    pub rotate: f32,
    pub a_cmp_count: u16,
    pub a_cmp_mode: u8,
    pub a_cmp_param1: u8,
    pub a_cmp_param2: u8,
    pub p_jobj_ofs: u8,
    pub mat_col_count: u16,
    pub amb_col_count: u16,
    pub rotate_count: u16,
    pub size_target: f32,
    pub rotate_target: f32,
    pub prim_col_remain: u16,
    pub env_col_remain: u16,
    pub prim_col_target: Rgba,
    pub env_col_target: Rgba,
    pub mat_col_remain: u16,
    pub amb_col_remain: u16,
    pub a_cmp_remain: u16,
    pub a_cmp_param1_target: u8,
    pub a_cmp_param2_target: u8,
    pub mat_rgb: u8,
    pub mat_a: u8,
    pub amb_rgb: u8,
    pub amb_a: u8,
    pub mat_rgb_target: u8,
    pub mat_a_target: u8,
    pub amb_rgb_target: u8,
    pub amb_a_target: u8,
    pub trail: f32,
    pub gen: Option<GenId>,
    pub appsrt: Option<SrtId>,
    /// `userdata` (`float*` owned by a user func; a host id).
    pub userdata: Option<u32>,
    /// `callback` = `psCallback[i - 1]`: the 1-based index into the host's table.
    pub callback: Option<u8>,
}

impl melee_core::gc::GcLayout for Particle {
    const GC_NAME: &'static str = <melee_types::HSD_Particle as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_Particle as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_Particle as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_Particle as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// What the particle system calls outside itself.
pub trait PsHost {
    /// `HSD_Randf` on the simulation stream.
    fn randf(&mut self) -> f32;
    /// `HSD_JObjSetupMatrix(jobj)` then `jobj->mtx`.
    fn jobj_setup_matrix(&mut self, j: JObjRef) -> Mtx;
    /// `jobj->mtx` as it stands (no setup; `hsd_8039DAD4` copies it after `hsd_8039D214` set
    /// it up).
    fn jobj_matrix(&mut self, j: JObjRef) -> Mtx {
        self.jobj_setup_matrix(j)
    }
    /// `ref_INC(jobj)`.
    fn jobj_ref(&mut self, _j: JObjRef) {}
    /// `HSD_JObjUnref(jobj)`.
    fn jobj_unref(&mut self, _j: JObjRef) {}
    /// `hsd_804D7900(gen)`, called at the end of `hsd_8039F05C`. efLib installs
    /// `efLib_Cb_PtclAppSRTHook`, which only acts on four bank-0 command lists.
    fn generator_created(&mut self, _ps: &mut PsSystem, _g: GenId) {}
    /// `gp->userfunc->hookCreate(pp)` / `hookDelete(pp)` / `setUserData(pp, idx, v)`.
    fn userfunc_hook_create(&mut self, _ps: &mut PsSystem, _userfunc: u32, _p: PId) {
        panic!("HSD_PSUserFunc hookCreate (TODO(group5 ef user funcs))")
    }
    fn userfunc_hook_delete(&mut self, _ps: &mut PsSystem, _userfunc: u32, _p: PId) {
        panic!("HSD_PSUserFunc hookDelete (TODO(group5 ef user funcs))")
    }
    /// Returns false when the user func has no `setUserData` (the C then writes `userdata[idx]`).
    fn userfunc_set_user_data(&mut self, _ps: &mut PsSystem, _userfunc: u32, _p: PId, _idx: u8, _v: f32) -> bool {
        panic!("HSD_PSUserFunc setUserData (TODO(group5 ef user funcs))")
    }
    /// `userdata[idx] = v` for a particle with `userdata` but no `setUserData`.
    fn particle_userdata_write(&mut self, _userdata: u32, _idx: u8, _v: f32) {
        panic!("particle userdata write (TODO(group5 ef user funcs))")
    }
    /// `psCallback[index - 1](pp)` (opcode 0xE1 sets it; `-1` deletes the particle).
    fn particle_callback(&mut self, _ps: &mut PsSystem, _index: u8, _p: PId) -> i32 {
        panic!("particle callback psCallback (TODO(group5 ef psCallback))")
    }
    /// `srt->freefunc(srt)`.
    fn appsrt_free(&mut self, _ps: &mut PsSystem, _freefunc: u32, _s: SrtId) {
        panic!("HSD_psAppSRT freefunc (TODO(group5 ef appsrt))")
    }
    /// `hsd_804D78EC(gen)` (generator shapes > 8 at creation) / `hsd_804D78E8(gen, mtx)` (their
    /// emitter). No ported caller installs them; the C skips a NULL one.
    fn custom_gen_init(&mut self, _ps: &mut PsSystem, _g: GenId) -> bool {
        false
    }
    fn custom_gen_emit(&mut self, _ps: &mut PsSystem, _g: GenId, _rot: &Mtx) -> bool {
        false
    }
    /// `psCamera->eyepos->pos` and `HSD_CObjGetUpVector(psCamera)` (billboarded generators).
    fn billboard_camera(&mut self) -> (Vec3, Vec3) {
        panic!("generator kind 0x10000 needs psCamera (TODO(group1 cobj psCamera))")
    }
    /// `HSD_JObjAlloc()` for a particle JObj slot (kind 0x8000), and
    /// `HSD_JObjAddTranslation{X,Y,Z}`.
    fn jobj_alloc(&mut self) -> Option<JObjRef> {
        panic!("particle JObj slot needs HSD_JObjAlloc (TODO(group1 jobj))")
    }
    fn jobj_add_translation(&mut self, _j: JObjRef, _d: Vec3) {
        panic!("particle JObj slot translation (TODO(group1 jobj))")
    }
}

/// Where `psGenerateParticle0` links a new particle: the head of list `linkNo` (`head ==
/// NULL`) or right after a particle (`head = &pp->next`).
#[derive(Clone, Copy, Debug)]
pub enum Head {
    List,
    After(PId),
}

/// The particle system's globals (particle.c / generator.c / psappsrt.c file statics).
#[derive(Clone, Debug)]
pub struct PsSystem {
    /// `ptclref_804D0E5C`, `psCmdListArray`, `psTexGroupArray`, `psFormGroupArray`,
    /// `hsd_804D0948` per bank.
    pub banks: Vec<Option<Bank>>,
    gens: Vec<Option<Generator>>,
    gen_free: Vec<u32>,
    /// `hsd_804D78FC`: the generator list.
    pub gen_head: Option<GenId>,
    /// `hsd_804D78F8`: the walk's "previous" generator (also read by `hsd_8039D9C8`).
    pub gen_cursor: Option<GenId>,
    /// `hsd_804D78F4`: generators queued by `hsd_8039F6CC` for a one-shot `hsd_8039D71C`.
    pub gen_queue: Vec<GenId>,
    /// `hsd_804D78E0` / `hsd_804D78DA`: live / peak generator count.
    pub num_gens: u16,
    pub peak_gens: u16,
    parts: Vec<Option<Particle>>,
    part_free: Vec<u32>,
    /// `hsd_804D0908`: the particle lists.
    pub heads: [Option<PId>; NUM_LINKS],
    /// `hsd_804D78E2` / `numPeakParticles`.
    pub num_parts: u16,
    pub peak_parts: u16,
    /// `hsd_804D08E8`: particle JObj slots.
    pub jobj_slots: [Option<JObjRef>; 8],
    srts: Vec<Option<AppSrt>>,
    srt_free: Vec<u32>,
    /// `hsd_804D78DE` / `hsd_804D78D8`.
    pub num_srts: u16,
    pub peak_srts: u16,
    /// `lbl_804D6368`: the id counter (starts at 0x100).
    pub idnum_counter: u16,
}

impl Default for PsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PsSystem {
    /// `hsd_80398A08` + `hsd_8039D354` + `psInitAppSRT`: empty system.
    pub fn new() -> Self {
        PsSystem {
            banks: vec![None; NUM_BANKS],
            gens: Vec::new(),
            gen_free: Vec::new(),
            gen_head: None,
            gen_cursor: None,
            gen_queue: Vec::new(),
            num_gens: 0,
            peak_gens: 0,
            parts: Vec::new(),
            part_free: Vec::new(),
            heads: [None; NUM_LINKS],
            num_parts: 0,
            peak_parts: 0,
            jobj_slots: [None; 8],
            srts: Vec::new(),
            srt_free: Vec::new(),
            num_srts: 0,
            peak_srts: 0,
            idnum_counter: 0x100,
        }
    }

    // ---- arenas ------------------------------------------------------------------------------

    pub fn gen(&self, g: GenId) -> &Generator {
        self.gens[g.0 as usize].as_ref().expect("freed generator")
    }
    pub fn gen_mut(&mut self, g: GenId) -> &mut Generator {
        self.gens[g.0 as usize].as_mut().expect("freed generator")
    }
    pub fn part(&self, p: PId) -> &Particle {
        self.parts[p.0 as usize].as_ref().expect("freed particle")
    }
    pub fn part_mut(&mut self, p: PId) -> &mut Particle {
        self.parts[p.0 as usize].as_mut().expect("freed particle")
    }
    pub fn srt(&self, s: SrtId) -> &AppSrt {
        self.srts[s.0 as usize].as_ref().expect("freed appsrt")
    }
    pub fn srt_mut(&mut self, s: SrtId) -> &mut AppSrt {
        self.srts[s.0 as usize].as_mut().expect("freed appsrt")
    }
    fn alloc_gen(&mut self, g: Generator) -> GenId {
        match self.gen_free.pop() {
            Some(i) => {
                self.gens[i as usize] = Some(g);
                GenId(i)
            }
            None => {
                self.gens.push(Some(g));
                GenId(self.gens.len() as u32 - 1)
            }
        }
    }
    fn free_gen(&mut self, g: GenId) {
        self.gens[g.0 as usize] = None;
        self.gen_free.push(g.0);
    }
    fn alloc_part(&mut self, p: Particle) -> PId {
        match self.part_free.pop() {
            Some(i) => {
                self.parts[i as usize] = Some(p);
                PId(i)
            }
            None => {
                self.parts.push(Some(p));
                PId(self.parts.len() as u32 - 1)
            }
        }
    }
    fn free_part(&mut self, p: PId) {
        self.parts[p.0 as usize] = None;
        self.part_free.push(p.0);
    }

    /// Live generators in list order.
    pub fn generators(&self) -> Vec<GenId> {
        let mut v = Vec::new();
        let mut g = self.gen_head;
        while let Some(id) = g {
            v.push(id);
            g = self.gen(id).next;
        }
        v
    }
    /// Live particles of list `link` in list order.
    pub fn particles(&self, link: usize) -> Vec<PId> {
        let mut v = Vec::new();
        let mut p = self.heads[link];
        while let Some(id) = p {
            v.push(id);
            p = self.part(id).next;
        }
        v
    }

    // ---- banks -------------------------------------------------------------------------------

    /// `psInitDataBankLoad(bank, cmdBank, texBank, ref, NULL)` on an already located bank.
    pub fn load_bank(&mut self, bank: usize, data: Arc<PsBankData>, refs: Option<Arc<[u32]>>) {
        if bank < NUM_BANKS {
            self.banks[bank] = Some(Bank { data, refs });
        }
    }

    /// `psCmdListArray[bank]` (0 for an empty slot).
    fn cmd_list_end(&self, bank: i32) -> i32 {
        self.banks.get(bank as usize).and_then(|b| b.as_ref()).map_or(0, |b| b.data.end)
    }
    fn cmd_list(&self, bank: i32, idx: i32) -> Option<CmdList> {
        self.banks[bank as usize].as_ref().and_then(|b| b.data.cmd(idx).cloned())
    }
    /// `psTexGroupArray[bank][tg]` (NULL when the bank has none).
    fn tex_palflag(&self, bank: u8, tg: u32) -> Option<u16> {
        self.banks.get(bank as usize).and_then(|b| b.as_ref()).and_then(|b| b.data.tex.palflag(tg))
    }
    /// `hsd_804D0948[bank][idx]` when the bank has a remap table.
    fn remap(&self, bank: u8, idx: i32) -> i32 {
        match self.banks.get(bank as usize).and_then(|b| b.as_ref()).and_then(|b| b.refs.clone()) {
            Some(r) => r[idx as usize] as i32,
            None => idx,
        }
    }

    // ---- psappsrt.c --------------------------------------------------------------------------

    /// `psAddGeneratorAppSRT(status, idnum)`.
    pub fn add_appsrt(&mut self, status: u8, idnum: u16) -> SrtId {
        let s = AppSrt {
            used_count: 1,
            status,
            scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            idnum,
            ..Default::default()
        };
        let id = match self.srt_free.pop() {
            Some(i) => {
                self.srts[i as usize] = Some(s);
                SrtId(i)
            }
            None => {
                self.srts.push(Some(s));
                SrtId(self.srts.len() as u32 - 1)
            }
        };
        self.num_srts += 1;
        if self.num_srts > self.peak_srts {
            self.peak_srts = self.num_srts;
        }
        id
    }
    /// `psAddParticleAppSRT_begin`.
    pub fn add_particle_appsrt_begin(&mut self, p: PId, status: u8) -> SrtId {
        let idnum = self.part(p).idnum;
        let s = self.add_appsrt(status, idnum);
        self.part_mut(p).appsrt = Some(s);
        s
    }
    /// `psAddGeneratorAppSRT_begin`.
    pub fn add_generator_appsrt_begin(&mut self, g: GenId, status: u8) -> SrtId {
        let idnum = self.gen(g).idnum;
        let s = self.add_appsrt(status, idnum);
        self.gen_mut(g).appsrt = Some(s);
        s
    }
    /// `psAttachParticleAppSRT`.
    pub fn attach_particle_appsrt(&mut self, p: PId, srt: Option<SrtId>) -> i32 {
        let Some(s) = srt else { return -1 };
        if self.part(p).appsrt.is_some() {
            return -1;
        }
        self.part_mut(p).appsrt = Some(s);
        let st = self.srt_mut(s);
        st.used_count = st.used_count.wrapping_add(1);
        st.used_count as i32
    }
    /// `psAttachGeneratorAppSRT`.
    pub fn attach_generator_appsrt(&mut self, g: GenId, srt: Option<SrtId>) -> i32 {
        let Some(s) = srt else { return -1 };
        if self.gen(g).appsrt.is_some() {
            return -1;
        }
        self.gen_mut(g).appsrt = Some(s);
        let st = self.srt_mut(s);
        st.used_count = st.used_count.wrapping_add(1);
        st.used_count as i32
    }
    fn srt_release(&mut self, host: &mut dyn PsHost, s: SrtId) -> i32 {
        let st = self.srt_mut(s);
        st.used_count = st.used_count.wrapping_sub(1);
        let count = st.used_count as i32;
        if count == 0 {
            if let Some(f) = self.srt(s).freefunc {
                host.appsrt_free(self, f, s);
            }
            self.srts[s.0 as usize] = None;
            self.srt_free.push(s.0);
            self.num_srts = self.num_srts.wrapping_sub(1);
        }
        count
    }
    /// `psRemoveParticleAppSRT`.
    pub fn remove_particle_appsrt(&mut self, host: &mut dyn PsHost, p: PId) -> i32 {
        let Some(s) = self.part(p).appsrt else { return -1 };
        let count = self.srt_release(host, s);
        self.part_mut(p).appsrt = None;
        count
    }
    /// `psRemoveGeneratorSRT`.
    pub fn remove_generator_srt(&mut self, host: &mut dyn PsHost, g: GenId) -> i32 {
        let Some(s) = self.gen(g).appsrt else { return -1 };
        if self.srt(s).gp == Some(g) {
            self.srt_mut(s).gp = None;
        }
        let count = self.srt_release(host, s);
        self.gen_mut(g).appsrt = None;
        count
    }

    // ---- generator.c -------------------------------------------------------------------------

    /// `hsd_8039D1E4`: set the user func.
    pub fn set_userfunc(&mut self, g: GenId, userfunc: Option<u32>) {
        self.gen_mut(g).userfunc = userfunc;
    }

    /// `hsd_8039D1EC`: next id.
    pub fn next_idnum(&mut self) -> u16 {
        self.idnum_counter = self.idnum_counter.wrapping_add(1);
        if self.idnum_counter < 256 {
            self.idnum_counter = 256;
        }
        self.idnum_counter
    }

    /// The common head of `hsd_8039D214` / `hsd_8039D71C` / `hsd_803983A4`: refresh the
    /// position / AppSRT translate / scale from the JObj. Returns the JObj matrix when used.
    fn gen_follow_jobj(&mut self, host: &mut dyn PsHost, g: GenId) -> Option<Mtx> {
        let (ty, jobj) = (self.gen(g).type_, self.gen(g).jobj);
        if ty & 0x100 == 0 {
            return None;
        }
        let j = jobj?;
        let m = host.jobj_setup_matrix(j);
        if ty & 0x200 != 0 {
            self.gen_mut(g).pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
        }
        if ty & 0x800 != 0 {
            if let Some(s) = self.gen(g).appsrt {
                if self.srt(s).gp == Some(g) {
                    self.srt_mut(s).translate = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
                }
            }
        }
        if ty & 0x1000 != 0 {
            if let Some(s) = self.gen(g).appsrt {
                if self.srt(s).gp == Some(g) {
                    self.srt_mut(s).scale = hsd_mtx::get_scale(&m);
                }
            }
        }
        Some(m)
    }

    /// `hsd_8039D214`.
    pub fn gen_update_from_jobj(&mut self, host: &mut dyn PsHost, g: GenId) {
        self.gen_follow_jobj(host, g);
    }

    /// `hsd_803983A4` (particle.c): the same, skipped for a paused (kind 0x800) generator.
    pub fn gen_update_from_jobj_unpaused(&mut self, host: &mut dyn PsHost, g: Option<GenId>) {
        let Some(g) = g else { return };
        if self.gen(g).kind & 0x800 != 0 {
            return;
        }
        self.gen_follow_jobj(host, g);
    }

    /// `hsd_8039D3AC(gen, prev)`: remove a finished generator (or keep it while particles or a
    /// shared AppSRT still need it). Returns the new "previous".
    pub fn gen_remove(&mut self, host: &mut dyn PsHost, g: GenId, prev: Option<GenId>) -> Option<GenId> {
        if self.gen(g).type_ & 0x80 != 0 {
            self.kill_generator_children(host, g);
        }
        if self.gen(g).num_child != 0 {
            let gp = self.gen_mut(g);
            gp.random = 0.0;
            gp.gen_life = 1;
            return Some(g);
        }
        let gp = self.gen(g);
        if gp.type_ & 0x100 != 0 && gp.jobj.is_some() && gp.type_ & 0x1800 != 0 {
            if let Some(s) = gp.appsrt {
                if self.srt(s).gp == Some(g) && self.srt(s).used_count != 1 {
                    let gp = self.gen_mut(g);
                    gp.random = 0.0;
                    gp.gen_life = 1;
                    return Some(g);
                }
            }
        }
        let next = self.gen(g).next;
        match prev {
            None => self.gen_head = next,
            Some(p) => self.gen_mut(p).next = next,
        }
        if self.gen(g).appsrt.is_some() {
            self.remove_generator_srt(host, g);
        }
        if let Some(j) = self.gen(g).jobj {
            host.jobj_unref(j);
            self.gen_mut(g).jobj = None;
        }
        self.free_gen(g);
        self.num_gens = self.num_gens.wrapping_sub(1);
        prev
    }

    /// `hsd_8039D4DC`: remove one generator, wherever it is in the list.
    pub fn gen_kill(&mut self, host: &mut dyn PsHost, g: GenId) {
        let mut cur = self.gen_head;
        self.gen_cursor = None;
        while let Some(c) = cur {
            if c == g {
                self.gen_cursor = self.gen_remove(host, g, self.gen_cursor);
                if let Some(mut k) = self.gen_cursor {
                    while let Some(n) = self.gen(k).next {
                        k = n;
                    }
                    self.gen_cursor = Some(k);
                } else if let Some(mut k) = self.gen_head {
                    while let Some(n) = self.gen(k).next {
                        k = n;
                    }
                    self.gen_cursor = Some(k);
                }
                return;
            }
            self.gen_cursor = Some(c);
            cur = self.gen(c).next;
        }
    }

    /// `hsd_8039D580`: remove every generator attached to `jobj`.
    pub fn gen_kill_jobj(&mut self, host: &mut dyn PsHost, jobj: Option<JObjRef>) {
        let Some(j) = jobj else { return };
        let mut cur = self.gen_head;
        while let Some(c) = cur {
            let next = self.gen(c).next;
            if self.gen(c).jobj == Some(j) {
                self.gen_kill(host, c);
            }
            cur = next;
        }
    }

    /// `hsd_8039D688(jobj, ...)`: release the generators on `jobj`, marking AppSRT-driven ones
    /// (type 0x100) to kill their particles.
    pub fn gen_release_jobj(&mut self, host: &mut dyn PsHost, jobj: Option<JObjRef>) {
        let mut cur = self.gen_head;
        self.gen_cursor = None;
        while let Some(c) = cur {
            let next = self.gen(c).next;
            if self.gen(c).jobj == jobj {
                if self.gen(c).appsrt.is_some() && self.gen(c).type_ & 0x100 != 0 {
                    self.gen_mut(c).type_ |= 0x80;
                }
                self.gen_cursor = self.gen_remove(host, c, self.gen_cursor);
            } else {
                self.gen_cursor = Some(c);
            }
            cur = next;
        }
    }

    /// `hsd_8039D71C`: one-shot JObj alignment of a queued generator (velocity and line end
    /// rotated into the JObj's frame).
    pub fn gen_align_to_jobj(&mut self, host: &mut dyn PsHost, g: GenId) {
        let Some(m) = self.gen_follow_jobj(host, g) else { return };
        let c0 = mtx::vec_normalize(&Vec3 { x: m[0][0], y: m[1][0], z: m[2][0] });
        let c1 = mtx::vec_normalize(&Vec3 { x: m[0][1], y: m[1][1], z: m[2][1] });
        let c2 = mtx::vec_normalize(&Vec3 { x: m[0][2], y: m[1][2], z: m[2][2] });
        let gp = self.gen_mut(g);
        let (vx, vy, vz) = (gp.vel.x, gp.vel.y, gp.vel.z);
        // generator.c 292-294: ((A + B) + C), both adds fused.
        gp.vel.x = fmadds(c2.x, vz, fmadds(c0.x, vx, c1.x * vy));
        gp.vel.y = fmadds(c2.y, vz, fmadds(c0.y, vx, c1.y * vy));
        gp.vel.z = fmadds(c2.z, vz, fmadds(c0.z, vx, c1.z * vy));
        if gp.type_ & 0xF == 1 {
            let (ax, ay, az) = (gp.aux[0], gp.aux[1], gp.aux[2]);
            // 302-306.
            gp.aux[0] = fmadds(m[0][2], az, fmadds(m[0][0], ax, m[0][1] * ay));
            gp.aux[1] = fmadds(m[1][2], az, fmadds(m[1][0], ax, m[1][1] * ay));
            gp.aux[2] = fmadds(m[2][2], az, fmadds(m[2][0], ax, m[2][1] * ay));
        }
    }

    /// `hsd_8039D9C8`: allocate a zeroed generator and link it after the walk cursor.
    fn gen_alloc(&mut self) -> GenId {
        let g = self.alloc_gen(Generator::zeroed());
        self.num_gens = self.num_gens.wrapping_add(1);
        if self.num_gens > self.peak_gens {
            self.peak_gens = self.num_gens;
        }
        let cursor_next = self.gen_cursor.and_then(|c| self.gen(c).next);
        match (self.gen_cursor, cursor_next) {
            (Some(_), Some(cn)) => {
                // `cursor->next->next = gen` after `gen->next = cursor->next->next`.
                let nn = self.gen(cn).next;
                self.gen_mut(g).next = nn;
                self.gen_mut(cn).next = Some(g);
            }
            _ => match self.gen_head {
                None => {
                    self.gen_mut(g).next = None;
                    self.gen_head = Some(g);
                }
                Some(h) => {
                    let hn = self.gen(h).next;
                    self.gen_mut(g).next = hn;
                    self.gen_mut(h).next = Some(g);
                }
            },
        }
        self.idnum_counter = self.idnum_counter.wrapping_add(1);
        if self.idnum_counter < 0x100 {
            self.idnum_counter = 0x100;
        }
        let id = self.idnum_counter;
        let gp = self.gen_mut(g);
        gp.idnum = id;
        gp.appsrt = None;
        gp.userfunc = None;
        g
    }

    /// `hsd_8039DAD4`: emit `floor(count)` particles; returns the remaining count.
    pub fn gen_emit(&mut self, host: &mut dyn PsHost, g: GenId) -> f32 {
        let mut angle3 = 0.0f32;
        let mut angle1 = 0.0f32;
        if self.gen(g).count < 1.0 {
            return self.gen(g).count;
        }
        let vel_copy = self.gen(g).vel;
        // generator.c 395-396.
        let vel_mag_sq =
            sqrtf(fmadds(vel_copy.z, vel_copy.z, fmadds(vel_copy.x, vel_copy.x, vel_copy.y * vel_copy.y)));
        let mut rot = mtx::identity();
        {
            let gp = self.gen(g);
            if gp.type_ & 0x100 != 0 && gp.jobj.is_some() && gp.type_ & 0x400 != 0 && gp.kind & 0x30000 == 0 {
                let jm = host.jobj_matrix(gp.jobj.unwrap());
                for col in 0..3 {
                    let v = mtx::vec_normalize(&Vec3 { x: jm[0][col], y: jm[1][col], z: jm[2][col] });
                    rot[0][col] = v.x;
                    rot[1][col] = v.y;
                    rot[2][col] = v.z;
                }
                rot[2][3] = 0.0;
                rot[1][3] = 0.0;
                rot[0][3] = 0.0;
            }
        }
        if self.gen(g).kind & 0x10000 != 0 {
            let (eye, up) = host.billboard_camera();
            let pos = self.gen(g).pos;
            let look = mtx::vec_normalize(&Vec3 { x: eye.x - pos.x, y: eye.y - pos.y, z: eye.z - pos.z });
            let mut cam_up = mtx::vec_normalize(&up);
            let cross1 = mtx::vec_cross(&cam_up, &look);
            cam_up = mtx::vec_cross(&look, &cross1);
            rot[0][0] = cross1.x;
            rot[1][0] = cross1.y;
            rot[2][0] = cross1.z;
            rot[0][1] = cam_up.x;
            rot[1][1] = cam_up.y;
            rot[2][1] = cam_up.z;
            rot[0][2] = look.x;
            rot[1][2] = look.y;
            rot[2][2] = look.z;
        }
        let shape = self.gen(g).type_ & 0xF;
        if shape != 1 && vel_mag_sq > 1.192_092_9e-7 {
            let vn = mtx::vec_normalize(&self.gen(g).vel);
            let mut sin_az = if fabsf_bitwise(vn.z) < 1.175_494_4e-38 {
                if vn.y >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
            } else {
                atan2f(vn.y, vn.z)
            };
            let cos_az = sinf(sin_az);
            sin_az = cosf(sin_az);
            // 481: vel_norm.y * cos_az + vel_norm.z * sin_az.
            let projected_z = fmadds(vn.y, cos_az, vn.z * sin_az);
            let elevation = if fabsf_bitwise(projected_z) < 1.175_494_4e-38 {
                if vn.x >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
            } else {
                atan2f(vn.x, projected_z)
            };
            let sin_el = sinf(elevation);
            let cos_el = cosf(elevation);
            let neg_cos_az = -cos_az;
            let trig = [
                [cos_el, 0.0, sin_el, 0.0],
                [neg_cos_az * sin_el, sin_az, cos_az * cos_el, 0.0],
                [-sin_az * sin_el, neg_cos_az, sin_az * cos_el, 0.0],
            ];
            rot = mtx::concat(&rot, &trig);
        }
        let elevation = vel_mag_sq;
        if shape == 2 {
            angle1 = if fabsf_bitwise(rot[2][2]) < 1.175_494_4e-38 {
                if rot[1][2] >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
            } else {
                atan2f(rot[1][2], rot[2][2])
            };
            let first = rot[2][2] * cosf(angle1);
            // 527: first + rot[1][2] * sinf(angle1) (the right product is fused).
            let comb = fmadds(rot[1][2], sinf(angle1), first);
            angle3 = if fabsf_bitwise(comb) < 1.175_494_4e-38 {
                if rot[0][2] >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
            } else {
                atan2f(rot[0][2], comb)
            };
        }

        let mut angle_step = 0.0f32;
        let mut cur_angle = 0.0f32;
        if self.gen(g).angle < 0.0 {
            match shape {
                0 | 3 | 4 | 6 | 7 => {
                    let (min_a, max_a, count) = (self.gen(g).aux[0], self.gen(g).aux[1], self.gen(g).count);
                    let rnd = host.randf();
                    angle_step = (max_a - min_a) / (count as i32) as f32;
                    // 550 / 559.
                    cur_angle = fmadds(angle_step, rnd, min_a);
                }
                _ => {
                    let rnd = host.randf();
                    cur_angle = ((M_PI * rnd as f64) * 2.0) as f32;
                    angle_step = ((2.0 * M_PI) / (self.gen(g).count as i32) as f64) as f32;
                }
            }
        }

        let eps = 0.001f32 as f64;
        let mut cone_angle;
        let mut sin_az;
        let mut radius;
        let mut emit_pos;
        let mut vec;
        while self.gen(g).count >= 1.0 {
            let gp = self.gen(g).clone();
            match shape {
                0 | 3 | 4 | 6 | 7 => {
                    if gp.radius < 0.0 {
                        sin_az = -gp.radius;
                        radius = 1.0;
                    } else {
                        radius = host.randf();
                        if shape == 3 || shape == 4 {
                            radius = sqrtf(radius);
                        }
                        sin_az = radius * gp.radius;
                    }
                    match shape {
                        6 => {
                            if gp.angle < 0.0 {
                                cur_angle += angle_step;
                                cone_angle = if fabsf_bitwise(sin_az) < 1.175_494_4e-38 {
                                    if gp.aux[2] >= 0.0 { -gp.angle } else { (M_PI - gp.angle as f64) as f32 }
                                } else {
                                    ((M_PI_2 - atan2f(gp.aux[2], sin_az) as f64) - gp.angle as f64) as f32
                                };
                            } else {
                                cur_angle = gp.aux[0];
                                let rnd = host.randf();
                                cone_angle = gp.aux[1] - cur_angle;
                                // 619.
                                cur_angle = fmadds(cone_angle, rnd, cur_angle);
                                let _ = radius * gp.angle;
                                cone_angle = if fabsf_bitwise(sin_az) < 1.175_494_4e-38 {
                                    if gp.aux[2] >= 0.0 { gp.angle } else { (M_PI + gp.angle as f64) as f32 }
                                } else {
                                    (gp.angle as f64 + (M_PI_2 - atan2f(gp.aux[2], sin_az) as f64)) as f32
                                };
                            }
                        }
                        7 => {
                            if gp.angle < 0.0 {
                                cone_angle = (M_PI_2 - gp.angle as f64) as f32;
                                cur_angle += angle_step;
                            } else {
                                cur_angle = gp.aux[0];
                                let rnd = host.randf();
                                cone_angle = gp.aux[1] - cur_angle;
                                // 649.
                                cur_angle = fmadds(cone_angle, rnd, cur_angle);
                                cone_angle = (M_PI_2 + gp.angle as f64) as f32;
                            }
                        }
                        _ => {
                            if gp.angle < 0.0 {
                                cone_angle = -gp.angle;
                                cur_angle += angle_step;
                                cone_angle *= radius;
                            } else {
                                cur_angle = gp.aux[0];
                                let rnd = host.randf();
                                cone_angle = radius * gp.angle;
                                let sin_el = gp.aux[1] - cur_angle;
                                // 665.
                                cur_angle = fmadds(sin_el, rnd, cur_angle);
                            }
                        }
                    }
                    let cos_az = cur_angle;
                    emit_pos = Vec3 { x: sin_az * cosf(cos_az), y: sin_az * sinf(cos_az), z: 0.0 };
                    if shape == 6 || shape == 7 {
                        emit_pos.z = host.randf();
                        if shape == 6 {
                            emit_pos.x *= 1.0 - emit_pos.z;
                            emit_pos.y *= 1.0 - emit_pos.z;
                        }
                        emit_pos.z *= gp.aux[2];
                    }
                    let sin_ca = elevation * sinf(cone_angle);
                    vec = Vec3 { x: sin_ca * cosf(cos_az), y: sin_ca * sinf(cos_az), z: elevation * cosf(cone_angle) };
                    if shape == 3 {
                        vec.x *= radius;
                        vec.y *= radius;
                        vec.z *= radius;
                    }
                    emit_pos = mtx::mult_vec(&rot, &emit_pos);
                    emit_pos.x += gp.pos.x;
                    emit_pos.y += gp.pos.y;
                    emit_pos.z += gp.pos.z;
                    vec = mtx::mult_vec(&rot, &vec);
                    cur_angle = cos_az;
                    self.spawn_from_gen(host, g, gp.kind, emit_pos, vec, gp.size, gp.grav, gp.fric);
                }
                1 => {
                    let rnd = host.randf();
                    emit_pos = Vec3 { x: rnd * gp.aux[0], y: rnd * gp.aux[1], z: rnd * gp.aux[2] };
                    emit_pos = mtx::mult_vec(&rot, &emit_pos);
                    emit_pos.x += gp.pos.x;
                    emit_pos.y += gp.pos.y;
                    emit_pos.z += gp.pos.z;
                    vec = mtx::mult_vec(&rot, &vel_copy);
                    self.spawn_from_gen(host, g, gp.kind, emit_pos, vec, gp.size, gp.grav, gp.fric);
                }
                2 => {
                    sin_az = if gp.radius < 0.0 { 1.0 } else { host.randf() };
                    if gp.angle < 0.0 {
                        cur_angle += angle_step;
                    } else {
                        let rnd = host.randf();
                        cur_angle = (2.0 * (M_PI * rnd as f64)) as f32;
                    }
                    self.gen_mut(g).aux[0] = elevation;
                    let p = Vec3::default();
                    let v = Vec3 { x: cur_angle, y: sin_az, z: 0.0 };
                    self.spawn_from_gen(host, g, gp.kind | 4, p, v, gp.size, angle1, angle3);
                }
                5 => {
                    emit_pos = Vec3 { x: host.randf(), y: host.randf(), z: host.randf() };
                    let half = |v: f32| if v > 0.5 { 1.0 } else { 0.0 };
                    match gp.aux_flag {
                        1 => emit_pos.x = half(emit_pos.x),
                        2 => emit_pos.y = half(emit_pos.y),
                        3 => {
                            let rnd = host.randf();
                            let a2 = gp.aux[0];
                            if rnd > a2 / (a2 + gp.aux[1]) {
                                emit_pos.y = half(emit_pos.y);
                            } else {
                                emit_pos.x = half(emit_pos.x);
                            }
                        }
                        4 => emit_pos.z = half(emit_pos.z),
                        5 => {
                            let rnd = host.randf();
                            let a2 = gp.aux[0];
                            if rnd > a2 / (a2 + gp.aux[2]) {
                                emit_pos.z = half(emit_pos.z);
                            } else {
                                emit_pos.x = half(emit_pos.x);
                            }
                        }
                        6 => {
                            let rnd = host.randf();
                            let a2 = gp.aux[1];
                            if rnd > a2 / (a2 + gp.aux[2]) {
                                emit_pos.z = half(emit_pos.z);
                            } else {
                                emit_pos.y = half(emit_pos.y);
                            }
                        }
                        7 => {
                            let rnd = host.randf();
                            let (a2, b2, c2) = (gp.aux[2], gp.aux[1], gp.aux[0]);
                            // 821: c2 * (b2 + a2) + b2 * a2.
                            let r0 = 1.0 / fmadds(c2, b2 + a2, b2 * a2);
                            if rnd < r0 * (c2 * b2) {
                                emit_pos.z = half(emit_pos.z);
                            } else if rnd > fmadds(-r0, c2 * a2, 1.0) {
                                // 825: 1.0F - r0 * (c2 * a2).
                                emit_pos.y = half(emit_pos.y);
                            } else {
                                emit_pos.x = half(emit_pos.x);
                            }
                        }
                        _ => {}
                    }
                    emit_pos.x -= 0.5;
                    emit_pos.y -= 0.5;
                    emit_pos.z -= 0.5;
                    let a = &gp.aux;
                    // 843-851: rect.xx/yx/zx ... (indices 3..11).
                    let tmp = Vec3 {
                        x: fmadds(a[9], emit_pos.z, fmadds(a[3], emit_pos.x, a[6] * emit_pos.y)),
                        y: fmadds(a[10], emit_pos.z, fmadds(a[4], emit_pos.x, a[7] * emit_pos.y)),
                        z: fmadds(a[11], emit_pos.z, fmadds(a[5], emit_pos.x, a[8] * emit_pos.y)),
                    };
                    emit_pos = mtx::mult_vec(&rot, &tmp);
                    emit_pos.x += gp.pos.x;
                    emit_pos.y += gp.pos.y;
                    emit_pos.z += gp.pos.z;
                    // 860-863.
                    let mag = sqrtf(fmadds(a[11], a[11], fmadds(a[9], a[9], a[10] * a[10])));
                    let scale = elevation / mag;
                    vec = Vec3 { x: a[9] * scale, y: a[10] * scale, z: a[11] * scale };
                    vec = mtx::mult_vec(&rot, &vec);
                    self.spawn_from_gen(host, g, gp.kind, emit_pos, vec, gp.size, gp.grav, gp.fric);
                }
                8 => {
                    let r0 = gp.aux[2];
                    if r0 == 0.0 || (r0 as f64 - M_PI).abs() < eps {
                        radius = host.randf();
                        radius = sqrtf(radius);
                        radius = (M_PI_2 * radius as f64) as f32;
                        let rnd2 = host.randf();
                        if rnd2 < 0.5 {
                            radius = (M_PI - radius as f64) as f32;
                        }
                    } else {
                        radius = host.randf();
                        radius = sqrtf(radius);
                        radius *= gp.aux[2];
                    }
                    cone_angle = host.randf();
                    cone_angle = (2.0 * (M_PI * cone_angle as f64)) as f32;
                    let r2 = gp.radius;
                    if r2 < 0.0 {
                        sin_az = -r2;
                    } else {
                        sin_az = host.randf();
                        sin_az = sqrtf(sin_az);
                        sin_az *= r2;
                    }
                    let cos_r = cosf(cone_angle);
                    let cos_az = sinf(radius);
                    let mut v = Vec3 { x: cos_az * cos_r, y: 0.0, z: 0.0 };
                    let sin_el = sinf(cone_angle);
                    let sin_r = sinf(radius);
                    v.y = sin_r * sin_el;
                    v.z = cosf(radius);
                    emit_pos = mtx::mult_vec(&rot, &v);
                    vec = Vec3 { x: emit_pos.x * gp.aux[0], y: emit_pos.y * gp.aux[0], z: emit_pos.z * gp.aux[0] };
                    if gp.radius >= 0.0 && gp.aux[0] < 0.0 {
                        let scale = sin_az / gp.radius;
                        vec.x *= scale;
                        vec.y *= scale;
                        vec.z *= scale;
                    }
                    // 941-943.
                    emit_pos.x = fmadds(sin_az, emit_pos.x, gp.pos.x);
                    emit_pos.y = fmadds(sin_az, emit_pos.y, gp.pos.y);
                    emit_pos.z = fmadds(sin_az, emit_pos.z, gp.pos.z);
                    self.spawn_from_gen(host, g, gp.kind, emit_pos, vec, gp.size, gp.grav, gp.fric);
                }
                _ => {
                    host.custom_gen_emit(self, g, &rot);
                }
            }
            self.gen_mut(g).count -= 1.0;
        }
        self.gen(g).count
    }

    /// The `hsd_80398F0C(gen->linkNo, gen->bank, kind, gen->texGroup, gen->cmdList, gen->life,
    /// 0, gen, pos, vel, size, grav, fric)` call of every emitter shape.
    #[allow(clippy::too_many_arguments)]
    fn spawn_from_gen(&mut self, host: &mut dyn PsHost, g: GenId, kind: u32, pos: Vec3, vel: Vec3, size: f32, grav: f32, fric: f32) {
        let gp = self.gen(g);
        let (link, bank, tg, list, life) = (gp.link_no, gp.bank, gp.tex_group as u16, gp.cmd_list.clone(), gp.life);
        self.generate_particle0(host, Head::List, link as i32, bank as i32, kind, tg, list, life as i32, 0, pos, vel, size, grav, fric, Some(g), true);
    }

    /// `hsd_8039EE24(mask)`: the per-frame generator update.
    pub fn gen_update_all(&mut self, host: &mut dyn PsHost, mask: u32) {
        while !self.gen_queue.is_empty() {
            let g = self.gen_queue.remove(0);
            self.gen_align_to_jobj(host, g);
            if let Some(j) = self.gen(g).jobj {
                host.jobj_unref(j);
                self.gen_mut(g).jobj = None;
            }
        }
        let mut cur = self.gen_head;
        self.gen_cursor = None;
        while let Some(g) = cur {
            let (link, kind) = (self.gen(g).link_no, self.gen(g).kind);
            if mask & (1 << (link as u32 + 16)) != 0 || kind & 0x800 != 0 {
                self.gen_cursor = Some(g);
                cur = self.gen(g).next;
                continue;
            }
            self.gen_update_from_jobj(host, g);
            if self.gen(g).random < 0.0 {
                let gp = self.gen_mut(g);
                gp.count -= gp.random;
            } else {
                let r = host.randf();
                let gp = self.gen_mut(g);
                // generator.c 1011: count += random * HSD_Randf().
                gp.count = fmadds(gp.random, r, gp.count);
            }
            if self.gen(g).count >= 1.0 {
                let c = self.gen_emit(host, g);
                self.gen_mut(g).count = c;
            }
            if self.gen(g).gen_life != 0 {
                let life = self.gen(g).gen_life - 1;
                self.gen_mut(g).gen_life = life;
                if life == 0 {
                    self.gen_cursor = self.gen_remove(host, g, self.gen_cursor);
                    cur = match self.gen_cursor {
                        Some(c) => self.gen(c).next,
                        None => self.gen_head,
                    };
                    continue;
                }
            }
            self.gen_cursor = Some(g);
            cur = self.gen(g).next;
        }
    }

    /// `hsd_8039EFAC(linkNo, bank, gfx_id, jobj)`: a generator that follows `jobj`.
    pub fn gen_create_on_jobj(&mut self, host: &mut dyn PsHost, link_no: i32, bank: i32, idx: i32, jobj: Option<JObjRef>) -> Option<GenId> {
        let g = self.gen_create(host, link_no, bank, idx)?;
        self.gen_mut(g).jobj = jobj;
        if let Some(j) = jobj {
            host.jobj_ref(j);
        }
        let gp = self.gen_mut(g);
        gp.type_ |= if gp.kind & 0x20000 != 0 { 0x500 } else { 0x700 };
        Some(g)
    }

    /// `hsd_8039F6CC`: as [`Self::gen_create_on_jobj`], queued for a one-shot alignment.
    pub fn gen_create_on_jobj_once(&mut self, host: &mut dyn PsHost, link_no: i32, bank: i32, idx: i32, jobj: Option<JObjRef>) -> Option<GenId> {
        let g = self.gen_create_on_jobj(host, link_no, bank, idx, jobj)?;
        self.gen_queue.push(g);
        Some(g)
    }

    /// `hsd_8039F05C(linkNo, bank, idx)`: a generator from a command list. (The TARGET_PC
    /// stage-slot override of `linkNo` is the port's mod system; not ported.)
    pub fn gen_create(&mut self, host: &mut dyn PsHost, link_no: i32, bank: i32, idx: i32) -> Option<GenId> {
        if bank >= 0x41 || link_no >= 8 || idx >= self.cmd_list_end(bank) {
            return None;
        }
        let c = self.cmd_list(bank, idx)?;
        let g = self.gen_alloc();
        {
            let gp = self.gen_mut(g);
            gp.type_ = c.type_;
            gp.bank = bank as u8;
            gp.link_no = link_no as u8;
            gp.kind = c.kind;
            gp.tex_group = c.tex_group as u8;
            gp.life = c.life;
            gp.gen_life = c.gen_life;
            gp.pos = Vec3::default();
            gp.vel = Vec3 { x: c.vx, y: c.vy, z: c.vz };
            gp.grav = c.grav;
            gp.fric = c.fric;
            gp.size = c.size;
            gp.cmd_list = Some(c.cmd_list.clone());
            gp.radius = c.radius;
            gp.angle = c.angle;
            gp.random = c.random;
        }
        let (kind, random) = (c.kind, c.random);
        let count = if kind & 0x100 != 0 {
            if random < 0.0 {
                if 1.0 + random > 1.192_092_9e-7 { 1.0 } else { 0.0 }
            } else {
                0.999_999_9
            }
        } else if random < 0.0 {
            0.0
        } else {
            host.randf()
        };
        self.gen_mut(g).count = count;
        let tg = self.gen(g).tex_group as u32;
        if let Some(pf) = self.tex_palflag(bank as u8, tg) {
            if pf != 0 {
                self.gen_mut(g).kind |= 0x10;
            }
        }
        {
            let gp = self.gen_mut(g);
            gp.jobj = None;
            gp.num_child = 0;
        }
        match c.type_ & 0xF {
            0 | 3 | 4 | 6 | 7 => {
                let gp = self.gen_mut(g);
                if c.param1 == 0.0 && c.param2 == 0.0 {
                    gp.aux[0] = 0.0;
                    gp.aux[1] = 6.283_185_5;
                } else {
                    gp.aux[0] = c.param1;
                    gp.aux[1] = c.param2;
                }
                if matches!(c.type_ & 0xF, 6 | 7) {
                    gp.aux[2] = c.param3;
                }
            }
            1 => {
                let gp = self.gen_mut(g);
                gp.aux[0] = c.param1;
                gp.aux[1] = c.param2;
                gp.aux[2] = c.param3;
            }
            2 => {}
            5 => {
                let gp = self.gen_mut(g);
                gp.aux[0] = c.param1;
                gp.aux[3] = c.param1;
                gp.aux[1] = c.param2;
                gp.aux[7] = c.param2;
                gp.aux[2] = c.param3;
                gp.aux[11] = c.param3;
                for k in [10, 9, 8, 6, 5, 4] {
                    gp.aux[k] = 0.0;
                }
                gp.aux_flag = 0;
                if c.param1 < 0.0 {
                    gp.aux_flag |= 1;
                }
                if c.param2 < 0.0 {
                    gp.aux_flag |= 2;
                }
                if c.param3 < 0.0 {
                    gp.aux_flag |= 4;
                }
            }
            8 => {
                let gp = self.gen_mut(g);
                let v = gp.vel;
                // 1194-1195.
                gp.aux[0] = sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)));
                // 1197.
                let mag = sqrtf(fmadds(v.x, v.x, v.z * v.z));
                gp.aux[1] = if mag < 1.175_494_4e-38 {
                    if v.y >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
                } else {
                    atan2f(v.y, mag)
                };
                gp.aux[3] = if fabsf_bitwise(v.x) < 1.175_494_4e-38 {
                    if v.z >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
                } else {
                    atan2f(v.z, v.x)
                };
                gp.aux[2] = c.param1;
                if gp.aux[2] < 0.0 {
                    gp.aux[2] = -gp.aux[2];
                    gp.aux[0] = -gp.aux[0];
                }
                gp.aux[4] = c.param2;
            }
            _ => {
                host.custom_gen_init(self, g);
            }
        }
        if self.gen(g).kind & 0x20000 != 0 {
            self.gen_mut(g).type_ |= 0x800;
            let s = self.add_generator_appsrt_begin(g, 0);
            let st = self.srt_mut(s);
            st.x_a2 = 1;
            st.gp = Some(g);
        }
        host.generator_created(self, g);
        Some(g)
    }

    // ---- particle.c --------------------------------------------------------------------------

    /// `psGenerateParticle0`.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_particle0(
        &mut self,
        host: &mut dyn PsHost,
        head: Head,
        link_no: i32,
        bank: i32,
        kind: u32,
        tex_group: u16,
        list: Option<CmdRef>,
        life: i32,
        palflag: i32,
        pos: Vec3,
        vel: Vec3,
        size: f32,
        grav: f32,
        fric: f32,
        gp: Option<GenId>,
        interpret: bool,
    ) -> Option<PId> {
        let p = self.alloc_part(Particle::default());
        self.num_parts = self.num_parts.wrapping_add(1);
        if self.num_parts > self.peak_parts {
            self.peak_parts = self.num_parts;
        }
        let idnum = match gp {
            Some(g) => self.gen(g).idnum,
            None => self.next_idnum(),
        };
        self.part_mut(p).idnum = idnum;
        self.part_mut(p).appsrt = None;
        if let Some(g) = gp {
            if let Some(s) = self.gen(g).appsrt {
                self.attach_particle_appsrt(p, Some(s));
            }
        }
        match head {
            Head::List => {
                let h = self.heads[link_no as usize];
                self.part_mut(p).next = h;
                self.heads[link_no as usize] = Some(p);
            }
            Head::After(prev) => {
                let n = self.part(prev).next;
                self.part_mut(p).next = n;
                self.part_mut(prev).next = Some(p);
            }
        }
        {
            let pp = self.part_mut(p);
            pp.bank = bank as u8;
            pp.link_no = link_no as u8;
            pp.kind = kind;
            pp.tex_group = tex_group as u8;
            pp.pos = pos;
            pp.vel = vel;
            pp.size = size;
            pp.grav = grav;
            pp.fric = fric;
            pp.life = (life + 1) as u16;
            pp.cmd_wait = if list.is_some() { 1 } else { 0 };
            pp.cmd_list = list;
            pp.cmd_mark_ptr = 0;
            pp.cmd_ptr = 0;
            if palflag != 0 {
                pp.kind |= 0x10;
            }
            pp.pose_num = 0;
            pp.pal_num = 0xFF;
            pp.prim_col = Rgba { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
            pp.env_col = Rgba::default();
            pp.env_col_count = 0;
            pp.prim_col_count = 0;
            pp.size_count = 0;
            pp.env_col_remain = 0;
            pp.prim_col_remain = 0;
            pp.a_cmp_mode = 0x33;
            pp.a_cmp_param1 = 1;
            pp.a_cmp_param2 = 0xFF;
            pp.a_cmp_remain = 0;
            pp.a_cmp_count = 0;
            pp.rotate_count = 0;
            pp.rotate_target = 0.0;
            pp.rotate = 0.0;
            pp.gen = gp;
        }
        if let Some(g) = gp {
            let gg = self.gen_mut(g);
            gg.num_child = gg.num_child.wrapping_add(1);
        }
        {
            let pp = self.part_mut(p);
            pp.p_jobj_ofs = 0;
            pp.mat_col_remain = 0;
            pp.mat_col_count = 0;
            pp.mat_rgb = 0xFF;
            pp.mat_a = 0xFF;
            pp.amb_col_remain = 0;
            pp.amb_col_count = 0;
            pp.amb_rgb = 0xFF;
            pp.amb_a = 0xFF;
            pp.trail = 1.0;
            pp.userdata = None;
        }
        if let Some(g) = gp {
            if let Some(uf) = self.gen(g).userfunc {
                host.userfunc_hook_create(self, uf, p);
            }
        }
        self.part_mut(p).callback = None;
        if interpret {
            self.interpret(host, p, None);
        }
        Some(p)
    }

    /// `hsd_80398F8C(pp, angle)`: scatter the velocity in a cone of half-angle `angle` around
    /// its current direction (opcode 0xA9).
    pub fn scatter_velocity(&mut self, host: &mut dyn PsHost, p: PId, angle: f32) {
        let v = self.part(p).vel;
        let (vx, vy, vz) = (v.x, v.y, v.z);
        let azimuth = if fabsf_bitwise(vz) < 1.175_494_4e-38 {
            if vy >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
        } else {
            atan2f(vy, vz)
        };
        let sin_a = sinf(azimuth);
        let cos_a = cosf(azimuth);
        // 556.
        let temp = fmadds(vy, sin_a, vz * cos_a);
        let az2 = if fabsf_bitwise(temp) < 1.175_494_4e-38 {
            if vx >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
        } else {
            atan2f(vx, temp)
        };
        let sin_e = sinf(az2);
        let cos_e = cosf(az2);
        // 573: (vx*vx + vy*vy) + vz*vz, both fused; then the inline sqrt (576-579).
        let mut mag = fmadds(vz, vz, fmadds(vx, vx, vy * vy));
        if mag > 0.0 {
            mag = sqrtf(mag);
        }
        let rand_angle = ((M_PI * host.randf() as f64) * 2.0) as f32;
        let sin_angle = mag * sinf(angle);
        let cos_rand = sin_angle * cosf(rand_angle);
        let sin_rand = sin_angle * sinf(rand_angle);
        let cos_angle = mag * cosf(angle);
        let pp = self.part_mut(p);
        // 591-595.
        pp.vel.x = fmadds(cos_rand, cos_e, cos_angle * sin_e);
        pp.vel.y = fmadds(cos_e, cos_angle * sin_a, fmadds(sin_e, -cos_rand * sin_a, sin_rand * cos_a));
        pp.vel.z = fmadds(cos_e, cos_angle * cos_a, fmadds(sin_e, -cos_rand * cos_a, -(sin_rand * sin_a)));
    }

    /// `hsd_803991D8((HSD_Generator*) pp, jobj, force, range)` as opcode 0xB8 calls it: the
    /// generator fields it names alias the particle's (`fric`/`size`/`radius` = `pos.x/y/z`,
    /// `pos.z`/`vel.x`/`vel.y` = `vel.x/y/z`). Returns true when within `range`.
    fn attract_to_jobj(&mut self, host: &mut dyn PsHost, p: PId, jobj: Option<JObjRef>, force: f32, range: f32) -> bool {
        let Some(j) = jobj else { return false };
        if range < 0.0 {
            return false;
        }
        let m = host.jobj_setup_matrix(j);
        let pp = self.part_mut(p);
        let dx = m[0][3] - pp.pos.x;
        let dy = m[1][3] - pp.pos.y;
        let dz = m[2][3] - pp.pos.z;
        // 613.
        let dist_sq = fmadds(dz, dz, fmadds(dx, dx, dy * dy));
        if dist_sq <= range * range {
            return true;
        }
        if dist_sq == 0.0 {
            return false;
        }
        let scale = force / dist_sq;
        // 624-626.
        pp.vel.x = fmadds(scale, dx, pp.vel.x);
        pp.vel.y = fmadds(scale, dy, pp.vel.y);
        pp.vel.z = fmadds(scale, dz, pp.vel.z);
        false
    }

    /// `psEnableTexture` on the particle's texture group.
    fn enable_texture(&mut self, p: PId) {
        let (bank, tg, pose) = (self.part(p).bank, self.part(p).tex_group, self.part(p).pose_num);
        let Some(b) = self.banks.get(bank as usize).and_then(|b| b.as_ref()) else { return };
        let tex = &b.data.tex;
        if let Some(grp) = tex.group(tg as u32) {
            if tex.tex_present(grp, pose as u32) {
                self.part_mut(p).kind |= KIND_DISP_TEXTURE;
            }
        }
    }

    /// The child-particle spawn shared by opcodes 0xA4/0xF1/0xAA/0xB9/0xF2 (`psSpawnChild` and
    /// its inlined copies): a particle from command list `idx`, linked after `pp`.
    fn spawn_child(&mut self, host: &mut dyn PsHost, p: PId, idx: i32) -> Option<PId> {
        let (link, bank) = (self.part(p).link_no as i32, self.part(p).bank as i32);
        if link >= 8 || bank >= 65 || idx >= self.cmd_list_end(bank) {
            return None;
        }
        let cl = self.cmd_list(bank, idx)?;
        let palflag = self.tex_palflag(bank as u8, cl.tex_group as u32).unwrap_or(0) as i32;
        let v = Vec3 { x: cl.vx, y: cl.vy, z: cl.vz };
        self.generate_particle0(host, Head::After(p), link, bank, cl.kind, cl.tex_group, Some(cl.cmd_list.clone()), cl.life as i32, palflag, Vec3::default(), v, cl.size, cl.grav, cl.fric, None, false)
    }

    /// Opcodes 0xA5 / 0xEF / 0xF0: a child generator at the particle.
    fn spawn_child_generator(&mut self, host: &mut dyn PsHost, p: PId, idx: i32, flags: Option<u8>) {
        let (link, bank) = (self.part(p).link_no as i32, self.part(p).bank as i32);
        let Some(gc) = self.gen_create(host, link, bank, idx) else { return };
        let idnum = self.part(p).idnum;
        self.gen_mut(gc).idnum = idnum;
        if let Some(pg) = self.part(p).gen {
            let j = self.gen(pg).jobj;
            self.gen_mut(gc).jobj = j;
            if let Some(j) = j {
                host.jobj_ref(j);
            }
        }
        self.gen_mut(gc).type_ |= 0x100;
        if let Some(pg) = self.part(p).gen {
            let t = self.gen(pg).type_ & 0x1E00;
            let gcp = self.gen_mut(gc);
            gcp.type_ |= t;
            if gcp.kind & (1 << 17) != 0 {
                gcp.type_ &= !(1 << 9);
            }
        }
        if let Some(s) = self.part(p).appsrt {
            self.attach_generator_appsrt(gc, Some(s));
        }
        if let Some(fl) = flags {
            let gcp = self.gen_mut(gc);
            gcp.kind &= 0xF1FF_FFFF;
            gcp.kind |= ((fl & 7) as u32) << 25;
        }
        let ppos = self.part(p).pos;
        match (self.part(p).appsrt, self.gen(gc).appsrt) {
            (Some(_), None) => {}
            (Some(ps), Some(gs)) => {
                if gs == ps {
                    self.gen_mut(gc).pos = ppos;
                } else {
                    let t = self.srt(ps).translate;
                    self.srt_mut(gs).translate = t;
                }
            }
            (None, Some(gs)) => self.srt_mut(gs).translate = ppos,
            (None, None) => self.gen_mut(gc).pos = ppos,
        }
    }

    /// The fixed-point colour step every colour opcode runs first:
    /// `cur = (target<<16 + step*(cur - target)) >> 16`, `step = (remain<<16) / count`.
    fn col_step(cur: u8, target: u8, step: i32) -> u8 {
        (((target as i32) << 16).wrapping_add(step.wrapping_mul(cur as i32 - target as i32)) >> 16) as u8
    }
    fn col_interp(cur: &mut Rgba, target: Rgba, remain: u16, count: u16) {
        if count != 0 {
            let step = ((remain as i32) << 16) / count as i32;
            cur.r = Self::col_step(cur.r, target.r, step);
            cur.g = Self::col_step(cur.g, target.g, step);
            cur.b = Self::col_step(cur.b, target.b, step);
            cur.a = Self::col_step(cur.a, target.a, step);
        }
    }
    /// `val = (f32) target + delta; clamp [0, 255]; (u8)(s32) val`.
    fn col_add(target: u8, delta: f32) -> u8 {
        let mut val = target as f32 + delta;
        if val < 0.0 {
            val = 0.0;
        }
        if val > 255.0 {
            val = 255.0;
        }
        val as i32 as u8
    }

    /// `hsd_8039930C(pp, prev)`: one particle's frame: interpolators, the bytecode (when its
    /// wait expires), life, physics and the JObj slot. Returns the next particle to visit
    /// (`pp->next`, or after a deletion the C's recomputed successor).
    pub fn interpret(&mut self, host: &mut dyn PsHost, p: PId, prev: Option<PId>) -> Option<PId> {
        if self.part(p).kind & 0x800 != 0 {
            return self.part(p).next;
        }
        {
            let pp = self.part_mut(p);
            if pp.size_count != 0 {
                pp.size += (pp.size_target - pp.size) / pp.size_count as f32;
                pp.size_count -= 1;
            }
            if pp.prim_col_count != 0 {
                pp.prim_col_remain = pp.prim_col_remain.wrapping_sub(1);
                if pp.prim_col_remain == 0 {
                    pp.prim_col_count = 0;
                    pp.prim_col = pp.prim_col_target;
                }
            }
            if pp.env_col_count != 0 {
                pp.env_col_remain = pp.env_col_remain.wrapping_sub(1);
                if pp.env_col_remain == 0 {
                    pp.env_col_count = 0;
                    pp.env_col = pp.env_col_target;
                }
            }
            if pp.mat_col_count != 0 {
                pp.mat_col_remain = pp.mat_col_remain.wrapping_sub(1);
                if pp.mat_col_remain == 0 {
                    pp.mat_col_count = 0;
                    pp.mat_rgb = pp.mat_rgb_target;
                    pp.mat_a = pp.mat_a_target;
                }
            }
            if pp.amb_col_count != 0 {
                pp.amb_col_remain = pp.amb_col_remain.wrapping_sub(1);
                if pp.amb_col_remain == 0 {
                    pp.amb_col_count = 0;
                    pp.amb_rgb = pp.amb_rgb_target;
                    pp.amb_a = pp.amb_a_target;
                }
            }
            if pp.a_cmp_count != 0 {
                pp.a_cmp_remain = pp.a_cmp_remain.wrapping_sub(1);
                if pp.a_cmp_remain == 0 {
                    pp.a_cmp_count = 0;
                    pp.a_cmp_param1 = pp.a_cmp_param1_target;
                    pp.a_cmp_param2 = pp.a_cmp_param2_target;
                }
            }
            if pp.rotate_count != 0 {
                pp.rotate += (pp.rotate_target - pp.rotate) / pp.rotate_count as f32;
                pp.rotate_count -= 1;
            }
        }

        let mut run = false;
        {
            let pp = self.part_mut(p);
            if pp.cmd_wait != 0 {
                pp.cmd_wait -= 1;
                run = pp.cmd_wait == 0;
            }
        }
        if run {
            self.run_bytecode(host, p);
        }

        // do_life
        {
            let pp = self.part_mut(p);
            pp.life = pp.life.wrapping_sub(1);
        }
        if self.part(p).life == 0 {
            return self.delete_particle(host, p, prev);
        }

        // Physics.
        if self.part(p).kind & KIND_TORNADO != 0 {
            let g = self.part(p).gen.expect("tornado particle without a generator");
            let gp = self.gen(g).clone();
            let pp = self.part_mut(p);
            let sin_a = sinf(pp.grav);
            let sin_b = sinf(pp.fric);
            let cos_a = cosf(pp.grav);
            let cos_b = cosf(pp.fric);
            pp.vel.z += gp.aux[0];
            let mut r = if gp.radius < 0.0 { -gp.radius } else { gp.radius };
            let ang = if gp.angle < 0.0 { -gp.angle } else { gp.angle };
            // 2895: R += vel.z * tanf(ang).
            r = fmadds(pp.vel.z, tanf(ang), r);
            r *= pp.vel.y;
            pp.vel.x += gp.grav;
            let d = r * cosf(pp.vel.x);
            let e = r * sinf(pp.vel.x);
            let nd = -d;
            let vz = pp.vel.z;
            // 2905-2909.
            pp.pos.x = gp.pos.x + fmadds(d, cos_b, vz * sin_b);
            pp.pos.y = gp.pos.y + fmadds(cos_b, vz * sin_a, fmadds(sin_b, nd * sin_a, e * cos_a));
            pp.pos.z = gp.pos.z + fmadds(cos_b, vz * cos_a, fmadds(sin_b, nd * cos_a, -(e * sin_a)));
        } else {
            let pp = self.part_mut(p);
            if pp.kind & 1 != 0 {
                pp.vel.y -= pp.grav;
            }
            if pp.kind & 2 != 0 {
                pp.vel.x *= pp.fric;
                pp.vel.y *= pp.fric;
                pp.vel.z *= pp.fric;
            }
            pp.pos.x += pp.vel.x;
            pp.pos.y += pp.vel.y;
            pp.pos.z += pp.vel.z;
        }

        if self.part(p).kind & 0x8000 != 0 {
            let slot = ((self.part(p).kind & 0x7000) >> 12) as usize;
            if self.jobj_slots[slot].is_none() {
                if let Some(j) = host.jobj_alloc() {
                    self.set_point_jobj(host, slot as i32 + 1, j);
                    host.jobj_unref(j);
                }
            }
            if let Some(j) = self.jobj_slots[slot] {
                let m = host.jobj_setup_matrix(j);
                let pos = self.part(p).pos;
                host.jobj_add_translation(j, Vec3 { x: pos.x - m[0][3], y: pos.y - m[1][3], z: pos.z - m[2][3] });
            }
        }

        if let Some(cb) = self.part(p).callback {
            if host.particle_callback(self, cb, p) == -1 {
                return self.delete_particle(host, p, prev);
            }
        }
        self.part(p).next
    }

    /// The bytecode loop of `hsd_8039930C` (from `pc = cmdList + cmdPtr` to `exit_loop`).
    fn run_bytecode(&mut self, host: &mut dyn PsHost, p: PId) {
        let list = self.part(p).cmd_list.clone().expect("cmdWait set without a command list");
        let mut pc: u32 = self.part(p).cmd_ptr as u32;
        let mut operand: u16;
        macro_rules! rd {
            () => {{
                let b = list.byte(pc);
                pc += 1;
                b
            }};
        }
        // psReadFloat: four bytes in stream order into a float (big-endian on the console).
        macro_rules! rdf {
            () => {{
                let b = [list.byte(pc), list.byte(pc + 1), list.byte(pc + 2), list.byte(pc + 3)];
                pc += 4;
                f32::from_be_bytes(b)
            }};
        }
        // A count byte with the 0x80 two-byte extension.
        macro_rules! rdcount {
            () => {{
                let mut cnt = rd!() as u16;
                if cnt & 0x80 != 0 {
                    cnt = ((cnt & 0x7F) << 8) + rd!() as u16;
                }
                cnt
            }};
        }
        macro_rules! pp {
            () => {
                self.part_mut(p)
            };
        }
        'outer: loop {
            let opcode = rd!();
            if opcode < 0x80 {
                operand = (opcode & 0x1F) as u16;
                if opcode & 0x20 != 0 {
                    operand = (operand << 8) + rd!() as u16;
                }
                if opcode & 0xC0 == 0x40 {
                    let pn = rd!();
                    pp!().pose_num = pn;
                    self.enable_texture(p);
                }
            } else {
                operand = 0;
                let mut cls = opcode & 0xF8;
                if cls > 0x98 {
                    cls = opcode & 0xF0;
                    if cls != 0xC0 && cls != 0xD0 {
                        cls = opcode;
                    }
                }
                match cls {
                    0x80 | 0x88 | 0x90 | 0x98 => {
                        for (bit, axis) in [(1u8, 0), (2, 1), (4, 2)] {
                            if opcode & bit != 0 {
                                let f = rdf!();
                                let pp = pp!();
                                let v = match cls {
                                    0x80 | 0x88 => &mut pp.pos,
                                    _ => &mut pp.vel,
                                };
                                let c = match axis {
                                    0 => &mut v.x,
                                    1 => &mut v.y,
                                    _ => &mut v.z,
                                };
                                if cls == 0x80 || cls == 0x90 {
                                    *c = f;
                                } else {
                                    *c += f;
                                }
                            }
                        }
                    }
                    0xA0 => {
                        let cnt = rdcount!();
                        let f = rdf!();
                        let pp = pp!();
                        pp.size_count = cnt;
                        pp.size_target = f;
                        if pp.size_count == 0 {
                            pp.size = pp.size_target;
                        }
                    }
                    0xA1 => pp!().kind &= !KIND_DISP_TEXTURE,
                    0xA2 => {
                        let f = rdf!();
                        let pp = pp!();
                        pp.grav = f;
                        if pp.grav == 0.0 {
                            pp.kind &= !1;
                        } else {
                            pp.kind |= 1;
                        }
                    }
                    0xA3 => {
                        let f = rdf!();
                        let pp = pp!();
                        pp.fric = f;
                        if pp.fric == 1.0 {
                            pp.kind &= !2;
                        } else {
                            pp.kind |= 2;
                        }
                    }
                    0xA4 | 0xF1 | 0xB9 | 0xF2 => {
                        let mut idx = ((list.byte(pc) as i32) << 8) + list.byte(pc + 1) as i32;
                        pc += 2;
                        if cls == 0xF1 || cls == 0xF2 {
                            idx = self.remap(self.part(p).bank, idx);
                        }
                        if let Some(c) = self.spawn_child(host, p, idx) {
                            let src = self.part(p).clone();
                            if cls == 0xB9 || cls == 0xF2 {
                                let cp = self.part_mut(c);
                                cp.pos = src.pos;
                                cp.vel = src.vel;
                                cp.idnum = src.idnum;
                                cp.gen = src.gen;
                            } else {
                                let cp = self.part_mut(c);
                                cp.idnum = src.idnum;
                                cp.gen = src.gen;
                            }
                            if let Some(g) = src.gen {
                                let gg = self.gen_mut(g);
                                gg.num_child = gg.num_child.wrapping_add(1);
                            }
                            self.attach_particle_appsrt(c, src.appsrt);
                            if cls == 0xA4 || cls == 0xF1 {
                                self.part_mut(c).pos = src.pos;
                            }
                            self.interpret(host, c, Some(p));
                        }
                    }
                    0xA5 => {
                        let idx = ((list.byte(pc) as i32) << 8) + list.byte(pc + 1) as i32;
                        pc += 2;
                        self.spawn_child_generator(host, p, idx, None);
                    }
                    0xEF | 0xF0 => {
                        let mut idx = ((list.byte(pc) as i32) << 8) + list.byte(pc + 1) as i32;
                        let flags = list.byte(pc + 2);
                        pc += 3;
                        if cls == 0xF0 {
                            idx = self.remap(self.part(p).bank, idx);
                        }
                        self.spawn_child_generator(host, p, idx, Some(flags));
                    }
                    0xA6 => {
                        let base = ((rd!() as i32) << 8) + rd!() as i32;
                        let range = ((rd!() as i32) << 8) + rd!() as i32;
                        let r = host.randf();
                        pp!().life = base.wrapping_add((range as f32 * r) as i32) as u16;
                    }
                    0xA7 => {
                        let threshold = rd!() as i32;
                        let r = host.randf();
                        if threshold >= (100.0f32 * r) as i32 {
                            pp!().life = 1;
                            break 'outer;
                        }
                    }
                    0xA8 => {
                        for axis in 0..3 {
                            let f = rdf!();
                            let r = host.randf();
                            // 1300/1306/1312: (2.0F * fval) * Randf() - fval, fused.
                            let d = fmadds(2.0 * f, r, -f);
                            let pp = pp!();
                            match axis {
                                0 => pp.pos.x += d,
                                1 => pp.pos.y += d,
                                _ => pp.pos.z += d,
                            }
                        }
                    }
                    0xA9 => {
                        let f = rdf!();
                        self.scatter_velocity(host, p, f);
                    }
                    0xAA => {
                        let mut idx = ((rd!() as i32) << 8) + rd!() as i32;
                        let range = ((rd!() as i32) << 8) + rd!() as i32;
                        let r = host.randf();
                        idx += (range as f32 * r) as i32;
                        idx = self.remap(self.part(p).bank, idx);
                        if let Some(c) = self.spawn_child(host, p, idx) {
                            let src = self.part(p).clone();
                            let cp = self.part_mut(c);
                            cp.pos = src.pos;
                            cp.idnum = src.idnum;
                            cp.gen = src.gen;
                            if let Some(g) = src.gen {
                                let gg = self.gen_mut(g);
                                gg.num_child = gg.num_child.wrapping_add(1);
                            }
                            if src.appsrt.is_some() {
                                self.attach_particle_appsrt(c, src.appsrt);
                            }
                            self.interpret(host, c, Some(p));
                        }
                    }
                    0xAB => {
                        let s = rdf!();
                        let pp = pp!();
                        pp.vel.x *= s;
                        pp.vel.y *= s;
                        pp.vel.z *= s;
                    }
                    0xAC => {
                        let cnt = rdcount!();
                        let target = rdf!();
                        let range = rdf!();
                        let r = host.randf();
                        let pp = pp!();
                        pp.size_count = cnt;
                        // 1428: sizeTarget += range * Randf().
                        pp.size_target = fmadds(range, r, target);
                        if pp.size_count == 0 {
                            pp.size = pp.size_target;
                        }
                    }
                    0xAD => pp!().kind |= KIND_PRIM_ENV,
                    0xAE => pp!().kind &= !(KIND_MIRROR_S | KIND_MIRROR_T),
                    0xAF => {
                        let pp = pp!();
                        pp.kind &= !KIND_MIRROR_T;
                        pp.kind |= KIND_MIRROR_S;
                    }
                    0xB0 => {
                        let pp = pp!();
                        pp.kind &= !KIND_MIRROR_S;
                        pp.kind |= KIND_MIRROR_T;
                    }
                    0xB1 => pp!().kind |= KIND_MIRROR_S | KIND_MIRROR_T,
                    0xB2 => {
                        if let Some(s) = self.part(p).appsrt {
                            if self.srt(s).x_a2 == 0 {
                                let gp = self.srt(s).gp;
                                self.gen_update_from_jobj_unpaused(host, gp);
                                let st = self.srt(s);
                                let rot = Vec3 { x: st.rot.x, y: st.rot.y, z: st.rot.z };
                                let m = hsd_mtx::srt(&st.scale, &rot, &st.translate, None);
                                self.srt_mut(s).mmtx = m;
                                let pp = self.part_mut(p);
                                // 1485-1496: each row two fused adds, then + m[i][3]; the
                                // rows read the already-updated components, as the C does.
                                pp.pos.x = fmadds(m[0][2], pp.pos.z, fmadds(m[0][0], pp.pos.x, m[0][1] * pp.pos.y)) + m[0][3];
                                pp.pos.y = fmadds(m[1][2], pp.pos.z, fmadds(m[1][0], pp.pos.x, m[1][1] * pp.pos.y)) + m[1][3];
                                pp.pos.z = fmadds(m[2][2], pp.pos.z, fmadds(m[2][0], pp.pos.x, m[2][1] * pp.pos.y)) + m[2][3];
                                self.remove_particle_appsrt(host, p);
                            }
                        }
                    }
                    0xB3 => {
                        {
                            let pp = pp!();
                            if pp.a_cmp_count != 0 {
                                let step = ((pp.a_cmp_remain as i32) << 16) / pp.a_cmp_count as i32;
                                pp.a_cmp_param1 = Self::col_step(pp.a_cmp_param1, pp.a_cmp_param1_target, step);
                                pp.a_cmp_param2 = Self::col_step(pp.a_cmp_param2, pp.a_cmp_param2_target, step);
                            }
                        }
                        let cnt = rdcount!();
                        let mode = rd!();
                        let t1 = rd!();
                        let t2 = rd!();
                        let pp = pp!();
                        pp.a_cmp_count = cnt;
                        pp.a_cmp_mode = mode;
                        pp.a_cmp_param1_target = t1;
                        pp.a_cmp_param2_target = t2;
                        if pp.a_cmp_count == 0 {
                            pp.a_cmp_param1 = pp.a_cmp_param1_target;
                            pp.a_cmp_param2 = pp.a_cmp_param2_target;
                            pp.a_cmp_remain = 0;
                            pp.a_cmp_count = 0;
                        } else {
                            pp.a_cmp_remain = pp.a_cmp_count;
                        }
                    }
                    0xB4 => pp!().kind |= KIND_TEX_INTERP_NEAR,
                    0xB5 => pp!().kind &= !KIND_TEX_INTERP_NEAR,
                    0xB6 => {
                        let cnt = rdcount!();
                        let f = rdf!();
                        let pp = pp!();
                        pp.rotate_count = cnt;
                        pp.rotate_target += f;
                        if pp.rotate_count == 0 {
                            pp.rotate = pp.rotate_target;
                        }
                    }
                    0xB7 => {
                        // TARGET_PC masks the slot index to 3 bits.
                        let slot = ((rd!() as u32 + self.part(p).p_jobj_ofs as u32) & 7) as usize;
                        if let Some(j) = self.jobj_slots[slot] {
                            let m = host.jobj_setup_matrix(j);
                            let pp = pp!();
                            // 1602: vx*vx + vy*vy fused.
                            let val = fmadds(pp.vel.x, pp.vel.x, pp.vel.y * pp.vel.y);
                            let mut vel_mag = pp.vel.z * pp.vel.z;
                            vel_mag += val;
                            let dx = m[0][3] - pp.pos.x;
                            let mut dy = m[1][3];
                            dy -= pp.pos.y;
                            let dz = m[2][3] - pp.pos.z;
                            if vel_mag > 0.0 {
                                vel_mag = sqrtf(vel_mag);
                            }
                            // 1620-1621.
                            let val = fmadds(dx, dx, dy * dy);
                            let mut dist = fmadds(dz, dz, val);
                            if dist != 0.0 {
                                if dist > 0.0 {
                                    dist = sqrtf(dist);
                                }
                                let scale = vel_mag / dist;
                                pp.vel = Vec3 { x: dx * scale, y: dy * scale, z: dz * scale };
                            }
                        }
                    }
                    0xB8 => {
                        let idx = rd!() as u32 + self.part(p).p_jobj_ofs as u32;
                        let force = rdf!();
                        let range = rdf!();
                        let j = self.jobj_slots[(idx & 7) as usize];
                        if self.attract_to_jobj(host, p, j, force, range) {
                            pp!().life = 1;
                            break 'outer;
                        }
                    }
                    0xBA | 0xBB => {
                        let prim = cls == 0xBA;
                        {
                            let pp = pp!();
                            if prim {
                                let t = pp.prim_col_target;
                                Self::col_interp(&mut pp.prim_col, t, pp.prim_col_remain, pp.prim_col_count);
                            } else {
                                let t = pp.env_col_target;
                                Self::col_interp(&mut pp.env_col, t, pp.env_col_remain, pp.env_col_count);
                            }
                        }
                        for ch in 0..4 {
                            let r = host.randf();
                            let delta = rd!() as i8;
                            let rv = ((delta as i32) << 1) as f32 * r;
                            let pp = pp!();
                            let t = if prim { &mut pp.prim_col_target } else { &mut pp.env_col_target };
                            let c = match ch {
                                0 => &mut t.r,
                                1 => &mut t.g,
                                2 => &mut t.b,
                                _ => &mut t.a,
                            };
                            *c = Self::col_add(*c, rv);
                        }
                        let pp = pp!();
                        if prim {
                            if pp.prim_col_count == 0 {
                                pp.prim_col = pp.prim_col_target;
                            } else {
                                pp.prim_col_remain = pp.prim_col_count;
                            }
                        } else if pp.env_col_count == 0 {
                            pp.env_col = pp.env_col_target;
                        } else {
                            pp.env_col_remain = pp.env_col_count;
                        }
                    }
                    0xBC => {
                        let pn = rd!();
                        let range = rd!() as f32;
                        let r = host.randf();
                        // 1978: randRange * Randf() + (f32) poseNum, fused.
                        pp!().pose_num = fmadds(range, r, pn as f32) as i32 as u8;
                        self.enable_texture(p);
                    }
                    0xBD => {
                        let base = rdf!();
                        let range = rdf!();
                        let r = host.randf();
                        // 2015.
                        let mut speed = fmadds(range, r, base);
                        let pp = pp!();
                        // 2016: vx*vx + vy*vy + vz*vz, two fused adds.
                        let mut mag = fmadds(pp.vel.z, pp.vel.z, fmadds(pp.vel.x, pp.vel.x, pp.vel.y * pp.vel.y));
                        if mag > 0.0 {
                            mag = sqrtf(mag);
                        }
                        if mag > 1e-10 {
                            speed /= mag;
                            pp.vel.x *= speed;
                            pp.vel.y *= speed;
                            pp.vel.z *= speed;
                        }
                    }
                    0xBE => {
                        let fx = rdf!();
                        pp!().vel.x *= fx;
                        let fy = rdf!();
                        pp!().vel.y *= fy;
                        let fz = rdf!();
                        pp!().vel.z *= fz;
                    }
                    0xBF => {
                        let idx = rd!();
                        let pp = pp!();
                        pp.kind |= ((((idx as u32) + pp.p_jobj_ofs as u32) & 7) << 12) | 0x8000;
                    }
                    0xC0 | 0xD0 => {
                        let prim = cls == 0xC0;
                        {
                            let pp = pp!();
                            if prim {
                                let t = pp.prim_col_target;
                                Self::col_interp(&mut pp.prim_col, t, pp.prim_col_remain, pp.prim_col_count);
                            } else {
                                let t = pp.env_col_target;
                                Self::col_interp(&mut pp.env_col, t, pp.env_col_remain, pp.env_col_count);
                            }
                        }
                        let cnt = rdcount!();
                        let mut t = if prim { self.part(p).prim_col } else { self.part(p).env_col };
                        if opcode & 1 != 0 {
                            t.r = rd!();
                        }
                        if opcode & 2 != 0 {
                            t.g = rd!();
                        }
                        if opcode & 4 != 0 {
                            t.b = rd!();
                        }
                        if opcode & 8 != 0 {
                            t.a = rd!();
                        }
                        let pp = pp!();
                        if prim {
                            pp.prim_col_count = cnt;
                            pp.prim_col_target = t;
                            if cnt == 0 {
                                pp.prim_col = t;
                                pp.prim_col_remain = 0;
                            } else {
                                pp.prim_col_remain = cnt;
                            }
                        } else {
                            pp.env_col_count = cnt;
                            pp.env_col_target = t;
                            if cnt == 0 {
                                pp.env_col = t;
                                pp.env_col_remain = 0;
                            } else {
                                pp.env_col_remain = cnt;
                            }
                        }
                    }
                    0xE0 => {
                        {
                            let pp = pp!();
                            let t = pp.prim_col_target;
                            Self::col_interp(&mut pp.prim_col, t, pp.prim_col_remain, pp.prim_col_count);
                            let t = pp.env_col_target;
                            Self::col_interp(&mut pp.env_col, t, pp.env_col_remain, pp.env_col_count);
                        }
                        for ch in 0..4 {
                            let r = host.randf();
                            let delta = rd!() as i8;
                            let rv = ((delta as i32) << 1) as f32 * r;
                            let pp = pp!();
                            let (pc_, ec_) = match ch {
                                0 => (&mut pp.prim_col_target.r, &mut pp.env_col_target.r),
                                1 => (&mut pp.prim_col_target.g, &mut pp.env_col_target.g),
                                2 => (&mut pp.prim_col_target.b, &mut pp.env_col_target.b),
                                _ => (&mut pp.prim_col_target.a, &mut pp.env_col_target.a),
                            };
                            *pc_ = Self::col_add(*pc_, rv);
                            *ec_ = Self::col_add(*ec_, rv);
                        }
                        let pp = pp!();
                        if pp.prim_col_count == 0 {
                            pp.prim_col = pp.prim_col_target;
                        }
                        pp.prim_col_remain = pp.prim_col_count;
                        if pp.env_col_count == 0 {
                            pp.env_col = pp.env_col_target;
                        }
                        pp.env_col_remain = pp.env_col_count;
                    }
                    0xE9 => {
                        {
                            let pp = pp!();
                            let t = pp.prim_col_target;
                            Self::col_interp(&mut pp.prim_col, t, pp.prim_col_remain, pp.prim_col_count);
                            let t = pp.env_col_target;
                            Self::col_interp(&mut pp.env_col, t, pp.env_col_remain, pp.env_col_count);
                        }
                        let flags = rd!();
                        let timing = rd!() as i32;
                        let scale = if timing != 0 {
                            let r = host.randf();
                            ((timing + 1) as f32 * r) as i32 as f32 / timing as f32
                        } else {
                            host.randf()
                        };
                        for (bit, ch) in [(1u8, 0), (2, 1), (4, 2)] {
                            if flags & bit != 0 {
                                let delta = rd!() as i8;
                                let df = scale * ((delta as i32) << 1) as f32;
                                let pp = pp!();
                                let (pc_, ec_) = match ch {
                                    0 => (&mut pp.prim_col_target.r, &mut pp.env_col_target.r),
                                    1 => (&mut pp.prim_col_target.g, &mut pp.env_col_target.g),
                                    _ => (&mut pp.prim_col_target.b, &mut pp.env_col_target.b),
                                };
                                if flags & 0x10 != 0 {
                                    *pc_ = Self::col_add(*pc_, df);
                                }
                                if flags & 0x20 != 0 {
                                    *ec_ = Self::col_add(*ec_, df);
                                }
                            }
                        }
                        if flags & 0x08 != 0 {
                            let a_rand = host.randf();
                            let delta = rd!() as i8;
                            let a_rand = ((timing + 1) as f32 * a_rand) as i32 as f32;
                            let df = (((delta as i32) << 1) as f32 * a_rand) / timing as f32;
                            let pp = pp!();
                            if flags & 0x10 != 0 {
                                pp.prim_col_target.a = Self::col_add(pp.prim_col_target.a, df);
                            }
                            if flags & 0x20 != 0 {
                                pp.env_col_target.a = Self::col_add(pp.env_col_target.a, df);
                            }
                        }
                        let pp = pp!();
                        if pp.prim_col_count == 0 {
                            pp.prim_col = pp.prim_col_target;
                        }
                        pp.prim_col_remain = pp.prim_col_count;
                        if pp.env_col_count == 0 {
                            pp.env_col = pp.env_col_target;
                        }
                        pp.env_col_remain = pp.env_col_count;
                    }
                    0xE1 => {
                        let idx = rd!();
                        pp!().callback = if idx == 0 { None } else { Some(idx) };
                    }
                    0xEC => {
                        let idx = rd!();
                        let v = rdf!();
                        let g = self.part(p).gen.expect("opcode 0xEC on a particle without a generator (C dereferences NULL)");
                        let handled = match self.gen(g).userfunc {
                            Some(uf) => host.userfunc_set_user_data(self, uf, p, idx, v),
                            None => false,
                        };
                        if !handled {
                            if let Some(ud) = self.part(p).userdata {
                                host.particle_userdata_write(ud, idx, v);
                            }
                        }
                    }
                    0xE2 => pp!().kind |= KIND_TEX_EDGE,
                    0xE3 => {
                        let n = rd!();
                        pp!().pal_num = n;
                    }
                    0xE4 | 0xE5 => {
                        let bit = if cls == 0xE4 { KIND_TEX_FLIP_S } else { KIND_TEX_FLIP_T };
                        let mode = rd!() & 3;
                        match mode {
                            0 => pp!().kind &= !bit,
                            1 => pp!().kind |= bit,
                            2 => pp!().kind ^= bit,
                            _ => {
                                if host.randf() < 0.5 {
                                    pp!().kind &= !bit;
                                } else {
                                    pp!().kind |= bit;
                                }
                            }
                        }
                    }
                    0xE6 => pp!().kind |= KIND_DIR_VEC,
                    0xE7 => pp!().kind &= !KIND_DIR_VEC,
                    0xE8 => {
                        let trail = rdf!();
                        let pp = pp!();
                        if trail < 0.0 {
                            pp.kind &= !KIND_TRAIL;
                        } else {
                            pp.kind |= KIND_TRAIL;
                            pp.trail = trail;
                        }
                    }
                    0xEA | 0xEB => {
                        let mat = cls == 0xEA;
                        {
                            let pp = pp!();
                            let (count, remain) = if mat { (pp.mat_col_count, pp.mat_col_remain) } else { (pp.amb_col_count, pp.amb_col_remain) };
                            if count != 0 {
                                let step = ((remain as i32) << 16) / count as i32;
                                if mat {
                                    pp.mat_rgb = Self::col_step(pp.mat_rgb, pp.mat_rgb_target, step);
                                    pp.mat_a = Self::col_step(pp.mat_a, pp.mat_a_target, step);
                                } else {
                                    pp.amb_rgb = Self::col_step(pp.amb_rgb, pp.amb_rgb_target, step);
                                    pp.amb_a = Self::col_step(pp.amb_a, pp.amb_a_target, step);
                                }
                            }
                        }
                        let cnt = rdcount!();
                        let flags = rd!();
                        let rgb = if flags & 1 != 0 { Some(rd!()) } else { None };
                        let a = if flags & 8 != 0 { Some(rd!()) } else { None };
                        let pp = pp!();
                        if mat {
                            pp.mat_col_count = cnt;
                            pp.mat_rgb_target = rgb.unwrap_or(pp.mat_rgb);
                            if let Some(a) = a {
                                pp.mat_a_target = a;
                            }
                            if cnt == 0 {
                                pp.mat_rgb = pp.mat_rgb_target;
                                pp.mat_col_remain = 0;
                            } else {
                                pp.mat_col_remain = cnt;
                            }
                        } else {
                            pp.amb_col_count = cnt;
                            pp.amb_rgb_target = rgb.unwrap_or(pp.amb_rgb);
                            if let Some(a) = a {
                                pp.amb_a_target = a;
                            }
                            if cnt == 0 {
                                pp.amb_rgb = pp.amb_rgb_target;
                                pp.amb_col_remain = 0;
                            } else {
                                pp.amb_col_remain = cnt;
                            }
                        }
                    }
                    0xED => {
                        let mut base = rdf!();
                        let range = rdf!();
                        let timing = rd!() as i32;
                        if timing != 0 {
                            let r = host.randf();
                            let randi = ((timing + 1) as f32 * r) as i32;
                            base += range * randi as f32 / timing as f32;
                        } else {
                            let r = host.randf();
                            // 2779.
                            base = fmadds(range, r, base);
                        }
                        let pp = pp!();
                        pp.rotate_target += base;
                        pp.rotate += base;
                    }
                    0xFA => {
                        let n = rd!();
                        let pp = pp!();
                        pp.loop_count = n;
                        pp.cmd_loop_ptr = pc as u16;
                    }
                    0xFB => {
                        let pp = pp!();
                        pp.loop_count = pp.loop_count.wrapping_sub(1);
                        if pp.loop_count != 0 {
                            pc = pp.cmd_loop_ptr as u32;
                        }
                    }
                    0xFC => pp!().cmd_mark_ptr = pc as u16,
                    0xFD => pc = self.part(p).cmd_mark_ptr as u32,
                    0xFE | 0xFF => {
                        pp!().life = 1;
                        break 'outer;
                    }
                    _ => {}
                }
            }
            if operand != 0 {
                break;
            }
        }
        let pp = pp!();
        pp.cmd_ptr = pc as u16;
        pp.cmd_wait = operand;
    }

    /// `delete_particle` of `hsd_8039930C`. Returns the successor the walk continues with.
    fn delete_particle(&mut self, host: &mut dyn PsHost, p: PId, prev: Option<PId>) -> Option<PId> {
        if let Some(g) = self.part(p).gen {
            if let Some(uf) = self.gen(g).userfunc {
                host.userfunc_hook_delete(self, uf, p);
            }
        }
        let link = self.part(p).link_no as usize;
        let next = self.part(p).next;
        match prev {
            None => self.heads[link] = next,
            Some(pr) => self.part_mut(pr).next = next,
        }
        let mut next_pp = next;
        if let Some(g) = self.part(p).gen {
            let gg = self.gen_mut(g);
            gg.num_child = gg.num_child.wrapping_sub(1);
        }
        if self.part(p).appsrt.is_some() && self.remove_particle_appsrt(host, p) == 0 && prev.is_none() {
            let head = self.heads[link];
            if head != next_pp {
                next_pp = head;
            }
        }
        self.release_point_jobj(host, p);
        self.free_part(p);
        self.num_parts = self.num_parts.wrapping_sub(1);
        next_pp
    }

    /// `hsd_8039CEAC(mask)`: run every particle list not masked off (bit 16 + link).
    pub fn particles_update_all(&mut self, host: &mut dyn PsHost, mask: u32) {
        let mut bits = mask;
        for i in 0..NUM_LINKS {
            if bits & 0x10000 == 0 {
                let mut cur = self.heads[i];
                let mut prev: Option<PId> = None;
                while let Some(c) = cur {
                    let next = self.interpret(host, c, prev);
                    match prev {
                        None => {
                            if self.heads[i] != next {
                                prev = Some(c);
                            }
                        }
                        Some(pr) => {
                            if next != self.part(pr).next {
                                prev = Some(c);
                            }
                        }
                    }
                    cur = next;
                }
            }
            bits >>= 1;
        }
    }

    /// `hsd_8039CF4C(index, jobj)`: set particle JObj slot `index - 1`, or with 0 clear every
    /// slot holding `jobj`.
    pub fn set_point_jobj(&mut self, host: &mut dyn PsHost, index: i32, jobj: JObjRef) {
        if !(0..=8).contains(&index) {
            return;
        }
        if index != 0 {
            let slot = (index - 1) as usize;
            let old = self.jobj_slots[slot];
            if old != Some(jobj) {
                if old.is_some() {
                    // The C unrefs the NEW jobj here (`HSD_JObjUnref(jobj)`), not the old one.
                    host.jobj_unref(jobj);
                }
                self.jobj_slots[slot] = Some(jobj);
                host.jobj_ref(jobj);
            }
        } else {
            for i in 0..8 {
                if self.jobj_slots[i] == Some(jobj) {
                    host.jobj_unref(jobj);
                    self.jobj_slots[i] = None;
                }
            }
        }
    }

    /// `hsd_8039D048(pp)`: drop the particle's JObj slot.
    fn release_point_jobj(&mut self, host: &mut dyn PsHost, p: PId) {
        let flags = self.part(p).kind;
        if flags & 0x8000 != 0 {
            let slot = ((flags >> 12) & 7) as usize;
            if let Some(j) = self.jobj_slots[slot].take() {
                host.jobj_unref(j);
            }
        }
    }

    /// `hsd_8039D0A0(gen)`: delete the generator's own particles (TARGET_PC list head).
    pub fn kill_generator_children(&mut self, host: &mut dyn PsHost, g: GenId) {
        let idnum = self.gen(g).idnum;
        let link = self.gen(g).link_no as usize;
        let mut prev: Option<PId> = None;
        let mut prt = self.heads[link];
        while let Some(c) = prt {
            let next = self.part(c).next;
            if self.part(c).idnum == idnum && self.part(c).gen == Some(g) {
                if let Some(uf) = self.gen(g).userfunc {
                    host.userfunc_hook_delete(self, uf, c);
                }
                match prev {
                    None => self.heads[link] = next,
                    Some(pr) => self.part_mut(pr).next = next,
                }
                let gg = self.gen_mut(g);
                gg.num_child = gg.num_child.wrapping_sub(1);
                if self.part(c).appsrt.is_some() {
                    self.remove_particle_appsrt(host, c);
                }
                self.release_point_jobj(host, c);
                self.free_part(c);
                self.num_parts = self.num_parts.wrapping_sub(1);
            } else {
                prev = Some(c);
            }
            prt = next;
        }
    }
}

#[cfg(test)]
mod tests;

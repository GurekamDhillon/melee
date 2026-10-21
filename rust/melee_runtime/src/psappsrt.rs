//! Port of `src/sysdolphin/baselib/psappsrt.c`: the particle "app SRT" records a generator (and
//! the particles it spawns) share, with a use count.
//!
//! Native shape: the records live in an [`AppSrtPool`] arena (the C's `HSD_ObjAlloc` pool
//! `HSD_PSAppSrt_804D10B0`) and are named by [`SrtId`]; a free slot is reused last-freed first,
//! as the HSD object allocator does. `gp` is the owning generator's [`GenId`]. The counters
//! `hsd_804D78DE` (live) / `hsd_804D78D8` (peak) are particle.c globals that only this file
//! writes; they live in the pool. `freefunc` is an opaque host tag ([`AppSrtHost::free_srt`]).
//! The particle-side functions take the particle's `appsrt` slot and `idnum` (HSD_Particle is
//! not ported yet). GcLayout: `melee_types::HSD_psAppSRT` (0xA4).
use crate::generator::GenId;
use crate::mtx::{Mtx, Quaternion, Vec3};

/// Index of an app SRT in its [`AppSrtPool`].
pub type SrtId = u32;

/// `HSD_psAppSRT`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdPsAppSrt {
    /// 0x00 `next` (never linked by psappsrt.c; kept for the particle code).
    pub next: Option<SrtId>,
    /// 0x04 `gp`.
    pub gp: Option<GenId>,
    /// 0x08
    pub translate: Vec3,
    /// 0x14
    pub rot: Quaternion,
    /// 0x24
    pub scale: Vec3,
    /// 0x30
    pub status: u8,
    /// 0x31
    pub frame_num: u8,
    /// 0x32
    pub used_count: u16,
    /// 0x34
    pub mmtx: Mtx,
    /// 0x64
    pub ssx: f32,
    /// 0x68
    pub ssy: f32,
    /// 0x6C..0x98 (`x6C` .. `x98`).
    pub x6c: [f32; 12],
    /// 0x9C `freefunc` (opaque host tag; None = NULL).
    pub freefunc: Option<u32>,
    /// 0xA0
    pub idnum: u16,
    /// 0xA2
    pub xa2: u8,
}

impl melee_core::gc::GcLayout for HsdPsAppSrt {
    const GC_NAME: &'static str = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_psAppSRT as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// What psappsrt.c calls outside itself.
pub trait AppSrtHost {
    /// `srt->freefunc(srt)` (only called when `freefunc` is set).
    fn free_srt(&mut self, tag: u32, id: SrtId, srt: &HsdPsAppSrt);
}

/// The app SRT allocator and its counters.
#[derive(Clone, Debug, Default)]
pub struct AppSrtPool {
    pub slots: Vec<Option<HsdPsAppSrt>>,
    free: Vec<SrtId>,
    /// `HSD_PSAppSrt_804D7958[0]`: the record size psInitAppSRT was given (the C memsets that many
    /// bytes; natively a new record is always fully reset).
    pub size: u16,
    /// `hsd_804D78DE`: live records.
    pub live: u16,
    /// `hsd_804D78D8`: peak of `live`.
    pub peak: u16,
}

impl AppSrtPool {
    pub fn get(&self, id: SrtId) -> &HsdPsAppSrt {
        self.slots[id as usize].as_ref().expect("freed app SRT")
    }
    pub fn get_mut(&mut self, id: SrtId) -> &mut HsdPsAppSrt {
        self.slots[id as usize].as_mut().expect("freed app SRT")
    }
    fn alloc(&mut self, v: HsdPsAppSrt) -> SrtId {
        if let Some(id) = self.free.pop() {
            self.slots[id as usize] = Some(v);
            id
        } else {
            self.slots.push(Some(v));
            (self.slots.len() - 1) as u32
        }
    }
    fn release(&mut self, id: SrtId) {
        self.slots[id as usize] = None;
        self.free.push(id);
    }

    /// `psInitAppSRT(num, size)`: always 0. (The C re-inits the allocator; records still
    /// allocated are leaked there, dropped here.)
    pub fn init(&mut self, _num: i32, size: i32) -> i32 {
        self.live = 0;
        self.peak = 0;
        self.size = size as u16;
        self.slots.clear();
        self.free.clear();
        0
    }

    /// `psAddGeneratorAppSRT(status, idnum)` (the allocator never fails natively).
    pub fn add(&mut self, status: i32, idnum: u16) -> Option<SrtId> {
        let srt = HsdPsAppSrt {
            used_count: 1,
            status: status as u8,
            scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            idnum,
            ..Default::default()
        };
        let id = self.alloc(srt);
        self.live = self.live.wrapping_add(1);
        if self.live > self.peak {
            self.peak = self.live;
        }
        Some(id)
    }

    /// `psAddParticleAppSRT_begin` / `psAddGeneratorAppSRT_begin`: `*slot = add(status, idnum)`.
    pub fn add_begin(&mut self, slot: &mut Option<SrtId>, status: i32, idnum: u16) -> Option<SrtId> {
        *slot = self.add(status, idnum);
        *slot
    }

    /// `psAttachParticleAppSRT` / `psAttachGeneratorAppSRT` (`slot` None = a NULL object).
    pub fn attach(&mut self, slot: Option<&mut Option<SrtId>>, srt: Option<SrtId>) -> i32 {
        let (Some(srt), Some(slot)) = (srt, slot) else { return -1 };
        if slot.is_some() {
            return -1;
        }
        *slot = Some(srt);
        let s = self.get_mut(srt);
        s.used_count = s.used_count.wrapping_add(1);
        s.used_count as i32
    }

    fn drop_ref<H: AppSrtHost>(&mut self, id: SrtId, host: &mut H) -> i32 {
        let s = self.get_mut(id);
        s.used_count = s.used_count.wrapping_sub(1);
        let count = s.used_count as i32;
        if count == 0 {
            if let Some(tag) = s.freefunc {
                let snapshot = s.clone();
                host.free_srt(tag, id, &snapshot);
            }
            self.release(id);
            self.live = self.live.wrapping_sub(1);
        }
        count
    }

    /// `psRemoveParticleAppSRT`: -1 when the particle has none.
    pub fn remove_particle<H: AppSrtHost>(&mut self, slot: &mut Option<SrtId>, host: &mut H) -> i32 {
        let Some(id) = *slot else { return -1 };
        let count = self.drop_ref(id, host);
        *slot = None;
        count
    }

    /// `psRemoveGeneratorSRT`: also clears `gp` when it is this generator.
    pub fn remove_generator<H: AppSrtHost>(&mut self, gen: GenId, slot: &mut Option<SrtId>, host: &mut H) -> i32 {
        let Some(id) = *slot else { return -1 };
        let s = self.get_mut(id);
        if s.gp == Some(gen) {
            s.gp = None;
        }
        let count = self.drop_ref(id, host);
        *slot = None;
        count
    }
}

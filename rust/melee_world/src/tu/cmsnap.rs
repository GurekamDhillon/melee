//! Port of src/melee/cm/cmsnap.c (the screen snapshot state machine: request, capture via
//! lb_800122C8 on the next GX pass, three frames of latency, then hand the image out).
#![allow(non_snake_case)]

/// `CmSnapStatus` (cm/forward.h).
pub mod status {
    pub const SLEEP: i32 = 0;
    pub const UNK1: i32 = 1;
    pub const UNK2: i32 = 2;
    pub const UNK3: i32 = 3;
    pub const UNK4: i32 = 4;
    pub const UNK5: i32 = 5;
}

/// Opaque handle for the image buffer (`HSD_ImageDesc.image_ptr`); `None` is NULL.
pub type ImagePtr = Option<u32>;

pub trait CmSnapHost {
    /// `HSD_ASSERT(line, cond)`
    fn HSD_ASSERT(&mut self, line: i32, cond: bool);
    /// `lb_800122C8(&unk1, 0, 0, 0)`: capture the EFB into the image descriptor.
    fn lb_800122C8(&mut self, img: ImagePtr);
    /// `lb_800121FC(&unk1, 0x280, 0x1E0, 4, 0x7D6)`: allocate the image; returns image_ptr.
    fn lb_800121FC(&mut self, w: i32, h: i32, fmt: i32, id: i32) -> ImagePtr;
    /// `GObj_SetupGXLinkMax(GObj_Create(0x12, 0x12, 0), cmSnap_80031640, 4)`.
    fn create_snap_gobj(&mut self);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire to baselib debug/gobj and lb/lbspdisplay ports).
    pub struct Ext;
    impl CmSnapHost for Ext {
        fn HSD_ASSERT(&mut self, _: i32, _: bool) { todo!("HSD_ASSERT") }
        fn lb_800122C8(&mut self, _: ImagePtr) { todo!("lb_800122C8") }
        fn lb_800121FC(&mut self, _: i32, _: i32, _: i32, _: i32) -> ImagePtr { todo!("lb_800121FC") }
        fn create_snap_gobj(&mut self) { todo!("GObj_Create / GObj_SetupGXLinkMax") }
    }
}

/// `cmsnap_data` ({CmSnapStatus status; HSD_ImageDesc unk1; int unk2}).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CmSnap {
    pub status: i32,
    pub image_ptr: ImagePtr,
    pub unk2: i32,
}

impl CmSnap {
    /// `cmSnap_800315C8`: request a snapshot.
    pub fn cmSnap_800315C8<H: CmSnapHost + ?Sized>(&mut self, h: &mut H) {
        h.HSD_ASSERT(0x55, self.status == status::SLEEP);
        self.status = status::UNK1;
    }

    /// `cmSnap_80031618`: the finished image, once (status 5 -> Sleep), else NULL.
    pub fn cmSnap_80031618(&mut self) -> ImagePtr {
        if self.status != status::UNK5 {
            return None;
        }
        self.status = status::SLEEP;
        self.image_ptr
    }

    /// `cmSnap_80031640`: the GX-link callback.
    pub fn cmSnap_80031640<H: CmSnapHost + ?Sized>(&mut self, h: &mut H, _code: i32) {
        match self.status {
            status::UNK1 => {
                h.lb_800122C8(self.image_ptr);
                self.status = status::UNK2;
            }
            status::UNK2 | status::UNK3 | status::UNK4 => self.status += 1,
            _ => {}
        }
    }

    /// `cmSnap_800316B4`: init.
    pub fn cmSnap_800316B4<H: CmSnapHost + ?Sized>(&mut self, h: &mut H) {
        self.status = status::SLEEP;
        self.image_ptr = None;
        h.create_snap_gobj();
        self.image_ptr = h.lb_800121FC(0x280, 0x1E0, 4, 0x7D6);
    }
}

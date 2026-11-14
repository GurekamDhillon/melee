//! The one host instance: OS heaps, disc and the loaded archive images. The game is single threaded and the C
//! keeps all of this in globals, so the host is a global too (`static mut`, set up once by `Host::boot`).

use crate::archive::ArchiveImage;
use crate::gcm::Gcm;
use crate::os::Os;
use std::collections::HashMap;

pub struct Host {
    pub os: Os,
    pub disc: Gcm,
    /// ARAM, grown on demand (a `HSD_DevComRequest` type 0x23 target below 0x8000_0000).
    pub aram: Vec<u8>,
    /// Loaded archives by the address of their native `HSD_Archive` record.
    pub archives: HashMap<usize, ArchiveImage>,
    /// Images whose archive record was reused by a later parse: never freed, native pointers into them may live on.
    pub retired: Vec<ArchiveImage>,
    /// Archive file names loaded, in order (run log).
    pub loaded: Vec<String>,
}

static mut HOST: Option<Host> = None;

impl Host {
    /// Install a fresh host (replacing any earlier one) and return it.
    pub fn boot(disc: Gcm, mem_size: usize) -> Result<&'static mut Host, String> {
        let os = Os::new(mem_size)?;
        crate::asset::init(disc.path.clone());
        unsafe {
            // ARAM at its full size (ARGetSize, 32 MB), zeroed and touched at boot: growing it on demand reallocated,
            // copied and zero-filled up to 28 MB of fresh pages inside the first fight's tick (an AJ animation file
            // landing high in ARAM, ~3 ms). Bytes past what was written read 0 either way.
            HOST = Some(Host { os, disc, aram: vec![0u8; 32 << 20], archives: HashMap::new(), retired: Vec::new(), loaded: Vec::new() });
            // touch every page now (a zeroed allocation can be lazily mapped)
            for i in (0..HOST.as_ref().unwrap().aram.len()).step_by(4096) {
                core::ptr::write_volatile(HOST.as_mut().unwrap().aram.as_mut_ptr().add(i), 0);
            }
            crate::asset::boot_hints();
            Ok(HOST.as_mut().unwrap())
        }
    }
    pub fn get() -> &'static mut Host {
        unsafe { HOST.as_mut().expect("melee_engine::Host::boot has not run") }
    }
    pub fn try_get() -> Option<&'static mut Host> {
        unsafe { HOST.as_mut() }
    }
    pub fn is_booted() -> bool {
        unsafe { HOST.is_some() }
    }
}

//! Memory the FIFO refers to. Aurora writes raw host pointers into its command stream
//! (`GX_AURORA_LOAD_ARRAYBASE`, `LOAD_TEXOBJ`, `LOAD_TLUT`); a Rust command stream carries a
//! handle instead, resolved here. The bytes are the game's own (big-endian GC data), shared, never
//! copied per draw.
use std::collections::HashMap;
use std::sync::Arc;

/// Game memory a vertex array, texture or TLUT points at.
pub type GxData = Arc<[u8]>;

/// Handle table. Handle 0 is null. A buffer registered twice keeps one handle (by address).
#[derive(Default, Debug)]
pub struct Resources {
    by_handle: HashMap<u64, GxData>,
    by_ptr: HashMap<(usize, usize), u64>,
    next: u64,
}

impl Resources {
    pub fn handle(&mut self, data: Option<&GxData>) -> u64 {
        let Some(d) = data else { return 0 };
        let key = (d.as_ptr() as usize, d.len());
        if let Some(&h) = self.by_ptr.get(&key) {
            return h;
        }
        self.next += 1;
        self.by_handle.insert(self.next, d.clone());
        self.by_ptr.insert(key, self.next);
        self.next
    }
    pub fn get(&self, handle: u64) -> Option<GxData> {
        if handle == 0 {
            None
        } else {
            self.by_handle.get(&handle).cloned()
        }
    }
}

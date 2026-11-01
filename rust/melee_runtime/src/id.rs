
//! Port of `src/sysdolphin/baselib/id.c`: the id -> object hash table (101 buckets, chained).
//!
//! Native shape: `IdTable<T>` owns its buckets; each bucket is a `Vec` whose **last** element
//! is the C chain's head (an insert of a new key pushes, which is the C's prepend), so lookups
//! walk it back to front in the C's order. The C's `default_table` (used when `table == NULL`)
//! is just another `IdTable` the caller owns. Keys are `HSD_IDKey` (`uintptr_t` on PC).
//! GameCube records: `melee_types::HSD_IDTable` (0x194), `melee_types::IDEntry` (0xC).

pub type IdKey = u64;

pub const ID_BUCKETS: usize = 101;

#[derive(Clone, Debug)]
pub struct IdTable<T> {
    buckets: Vec<Vec<(IdKey, T)>>,
}

impl<T> Default for IdTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> melee_core::gc::GcLayout for IdTable<T> {
    const GC_NAME: &'static str = <melee_types::HSD_IDTable as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_IDTable as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_IDTable as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_IDTable as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl<T> IdTable<T> {
    /// `HSD_IDSetup` / `_HSD_IDForgetMemory` on a fresh table (both memset it to zero).
    pub fn new() -> Self {
        IdTable { buckets: (0..ID_BUCKETS).map(|_| Vec::new()).collect() }
    }
    /// `HSD_IDSetup` and `_HSD_IDForgetMemory` (the C forgets the entries without freeing them).
    pub fn clear(&mut self) {
        self.buckets.iter_mut().for_each(Vec::clear);
    }

    /// `HSD_IDInsertToTable`: replaces the data of an existing key, else adds at the chain head.
    pub fn insert(&mut self, id: IdKey, data: T) {
        let b = &mut self.buckets[hash(id)];
        if let Some(e) = b.iter_mut().rev().find(|e| e.0 == id) {
            e.1 = data;
        } else {
            b.push((id, data));
        }
    }

    /// `HSD_IDRemoveByIDFromTable`: unlinks the first entry with `id` (from the head).
    pub fn remove(&mut self, id: IdKey) {
        let b = &mut self.buckets[hash(id)];
        if let Some(i) = b.iter().rposition(|e| e.0 == id) {
            b.remove(i);
        }
    }

    /// `HSD_IDGetDataFromTable`: `None` is the C's NULL return with `*success = 0`.
    pub fn get(&self, id: IdKey) -> Option<&T> {
        self.buckets[hash(id)].iter().rev().find(|e| e.0 == id).map(|e| &e.1)
    }
}

pub use crate::tu::id::hash_fp as hash;

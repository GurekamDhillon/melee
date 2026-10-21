//! Port of `src/sysdolphin/baselib/list.c`: the HSD singly linked list.
//!
//! Native shape: `SList<T>` is the list from its head (`HSD_SList*`, `None` = NULL) stored as a
//! `Vec` in list order. Note the C's naming: `HSD_SListAppendList(list, next)` links `next`
//! **after the head** (second position), not at the tail; `PrependList` makes it the new head;
//! `Remove` frees the head and returns the rest. GameCube records: `melee_types::HSD_SList`
//! (0x8), `melee_types::HSD_DList` (0xC; list.c has no DList operations besides its alloc data).
//! The alloc-data functions are native allocation and are not ported.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SList<T> {
    pub items: Vec<T>,
}

impl<T> melee_core::gc::GcLayout for SList<T> {
    const GC_NAME: &'static str = <melee_types::HSD_SList as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_SList as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_SList as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_SList as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl<T> SList<T> {
    pub fn new() -> Self {
        SList { items: Vec::new() }
    }
    /// `HSD_SListAllocAndAppend` (= `HSD_SListAppendList` on a new node).
    pub fn alloc_and_append(&mut self, data: T) {
        if self.items.is_empty() {
            self.items.push(data);
        } else {
            self.items.insert(1, data);
        }
    }
    /// `HSD_SListAllocAndPrepend` (= `HSD_SListPrependList` on a new node).
    pub fn alloc_and_prepend(&mut self, data: T) {
        self.items.insert(0, data);
    }
    /// `HSD_SListRemove`: drops the head (NULL list: no-op).
    pub fn remove(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

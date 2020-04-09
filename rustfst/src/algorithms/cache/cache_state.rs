use crate::algorithms::cache::CacheFlags;
use std::slice::Iter as IterSlice;
use std::slice::IterMut as IterSliceMut;

use crate::Arc;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct CacheState<W> {
    arcs: Vec<Arc<W>>,
    final_weight: Option<W>,
    flags: CacheFlags,
    ref_count: usize,
}

impl<W> CacheState<W> {
    pub fn new() -> Self {
        Self {
            arcs: Vec::new(),
            final_weight: None,
            flags: CacheFlags::empty(),
            ref_count: 0,
        }
    }

    pub fn has_final(&self) -> bool {
        self.flags.contains(CacheFlags::CACHE_FINAL)
    }

    pub fn expanded(&self) -> bool {
        self.flags.contains(CacheFlags::CACHE_ARCS)
    }

    pub fn mark_expanded(&mut self) {
        self.flags |= CacheFlags::CACHE_ARCS;
    }

    pub fn set_final_weight(&mut self, final_weight: Option<W>) {
        self.final_weight = final_weight;
        self.flags |= CacheFlags::CACHE_FINAL;
    }

    pub fn final_weight(&self) -> Option<&W> {
        self.final_weight.as_ref()
    }

    pub fn push_arc(&mut self, arc: Arc<W>) {
        self.arcs.push(arc);
    }

    pub fn reserve_arcs(&mut self, n: usize) {
        self.arcs.reserve(n);
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn get_arc_unchecked(&self, n: usize) -> &Arc<W> {
        unsafe { self.arcs.get_unchecked(n) }
    }

    pub fn get_arc_unchecked_mut(&mut self, n: usize) -> &mut Arc<W> {
        unsafe { self.arcs.get_unchecked_mut(n) }
    }

    pub fn arcs_iter(&self) -> IterSlice<Arc<W>> {
        self.arcs.iter()
    }

    pub fn arcs_iter_mut(&mut self) -> IterSliceMut<Arc<W>> {
        self.arcs.iter_mut()
    }

    pub fn delete_arcs(&mut self) {
        self.arcs.clear();
    }

    pub fn flags(&self) -> CacheFlags {
        self.flags
    }

    pub fn set_flags(&mut self, flags: CacheFlags, mask: CacheFlags) {
        self.flags &= !mask;
        self.flags |= flags;
    }

    pub fn incr_ref_count(&mut self) {
        self.ref_count += 1;
    }

    pub fn decr_ref_count(&mut self) {
        self.ref_count -= 1;
    }

    pub fn ref_count(&self) -> usize {
        self.ref_count
    }
}

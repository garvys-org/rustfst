use std::slice::Iter as IterSlice;
use std::slice::IterMut as IterSliceMut;

use crate::Arc;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct CacheState<W> {
    arcs: Vec<Arc<W>>,
    final_weight: Option<W>,
    expanded: bool,
    has_final: bool,
}

impl<W> CacheState<W> {
    pub fn new() -> Self {
        Self {
            arcs: Vec::new(),
            final_weight: None,
            expanded: false,
            has_final: false,
        }
    }

    pub fn has_final(&self) -> bool {
        self.has_final
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub fn mark_expanded(&mut self) {
        self.expanded = true;
    }

    pub fn set_final_weight(&mut self, final_weight: Option<W>) {
        self.final_weight = final_weight;
        self.has_final = true;
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
}

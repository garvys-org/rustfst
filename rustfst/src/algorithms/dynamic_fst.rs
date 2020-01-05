#[derive(Clone)]
pub struct StatesIteratorDynamicFst<'a, T> {
    pub(crate) fst: &'a T,
    pub(crate) s: usize,
}

macro_rules! dynamic_fst {
    ($name: expr, $dyn_fst: ty) => {
        use crate::algorithms::dynamic_fst::StatesIteratorDynamicFst;

        impl<F: ExpandedFst> ReplaceFst<F>
        where
            F::W: 'static,
        {
            fn num_known_states(&self) -> usize {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };
                fst_impl.num_known_states()
            }
        }

        impl<F: ExpandedFst> PartialEq for $dyn_fst
        where
            F::W: 'static,
        {
            fn eq(&self, other: &Self) -> bool {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };

                let ptr_other = other.fst_impl.get();
                let fst_impl_other = unsafe { ptr_other.as_ref().unwrap() };

                fst_impl.eq(fst_impl_other)
            }
        }

        impl<F: ExpandedFst> fmt::Debug for $dyn_fst
        where
            F::W: 'static,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };
                write!(f, "{:?} {{ {:?} }}", $name, &fst_impl)
            }
        }

        impl<F: ExpandedFst + 'static> Clone for $dyn_fst
        where
            F::W: 'static,
        {
            fn clone(&self) -> Self {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };
                Self {
                    fst_impl: UnsafeCell::new(fst_impl.clone()),
                    isymt: self.input_symbols(),
                    osymt: self.output_symbols(),
                }
            }
        }

        impl<F: ExpandedFst> CoreFst for $dyn_fst
        where
            F::W: 'static,
        {
            type W = F::W;

            fn start(&self) -> Option<usize> {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_mut().unwrap() };
                fst_impl.start().unwrap()
            }

            fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_mut().unwrap() };
                fst_impl.final_weight(state_id)
            }

            unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
                self.final_weight(state_id).unwrap()
            }

            fn num_arcs(&self, s: usize) -> Fallible<usize> {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_mut().unwrap() };
                fst_impl.num_arcs(s)
            }

            unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
                self.num_arcs(s).unwrap()
            }
        }

        impl<'a, F: ExpandedFst> ArcIterator<'a> for $dyn_fst
        where
            F::W: 'static,
        {
            type Iter = IterSlice<'a, Arc<F::W>>;

            fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_mut().unwrap() };
                fst_impl.arcs_iter(state_id)
            }

            unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
                self.arcs_iter(state_id).unwrap()
            }
        }

        impl<'a, F: ExpandedFst> Iterator for StatesIteratorDynamicFst<'a, $dyn_fst>
        where
            F::W: 'static,
        {
            type Item = StateId;

            fn next(&mut self) -> Option<Self::Item> {
                if self.s < self.fst.num_known_states() {
                    let s_cur = self.s;
                    // Force expansion of the state
                    self.fst.arcs_iter(s_cur).unwrap();
                    self.s += 1;
                    Some(s_cur)
                } else {
                    None
                }
            }
        }

        impl<'a, F: ExpandedFst + 'static> StateIterator<'a> for $dyn_fst
        where
            F::W: 'static,
        {
            type Iter = StatesIteratorDynamicFst<'a, $dyn_fst>;

            fn states_iter(&'a self) -> Self::Iter {
                self.start();
                StatesIteratorDynamicFst { fst: &self, s: 0 }
            }
        }

        impl<'a, F: ExpandedFst + 'static> Fst for $dyn_fst
        where
            F::W: 'static,
        {
            fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
                self.isymt.clone()
            }

            fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
                self.osymt.clone()
            }
        }
    };
}
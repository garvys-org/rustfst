#[derive(Clone)]
pub struct StatesIteratorDynamicFst<'a, T> {
    pub(crate) fst: &'a T,
    pub(crate) s: usize,
}

macro_rules! dynamic_fst {
    ($name: expr, $dyn_fst: ty, $([$a:tt => $b:tt $( < $c:ty > )? ])* $( where $d:ty => $e: tt )?) => {
        use crate::algorithms::dynamic_fst::StatesIteratorDynamicFst;

        impl<$($a: $b $( < $c >)? ),*> $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
        {
            fn num_known_states(&self) -> usize {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };
                fst_impl.num_known_states()
            }
        }

        impl<$($a: $b $( < $c >)? ),*> std::fmt::Debug for $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ptr = self.fst_impl.get();
                let fst_impl = unsafe { ptr.as_ref().unwrap() };
                write!(f, "{:?} {{ {:?} }}", $name, &fst_impl)
            }
        }

        impl<$($a: $b $( < $c >)? ),*> CoreFst for $dyn_fst
        where
            $($d: $e,)?
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

        impl<'a, $($a: $b $( < $c >)? ),*> ArcIterator<'a> for $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
            $($d: $e)?
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

        impl<'a, $($a: $b $( < $c >)? ),*> Iterator
            for StatesIteratorDynamicFst<'a, $dyn_fst>
        where
            $($d: $e,)?
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

        impl<'a, $($a: $b $( < $c >)? ),*> StateIterator<'a>
            for $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
            $($a : 'static),*
        {
            type Iter = StatesIteratorDynamicFst<'a, $dyn_fst>;

            fn states_iter(&'a self) -> Self::Iter {
                self.start();
                StatesIteratorDynamicFst { fst: &self, s: 0 }
            }
        }

        impl<$($a: $b $( < $c >)? ),*> Fst for $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
            $($a : 'static),*
        {
            fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
                self.isymt.clone()
            }

            fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
                self.osymt.clone()
            }
        }

        impl<'a, $($a: $b $( < $c >)? ),*> FstIterator<'a> for $dyn_fst
        where
            $($d: $e,)?
            F::W: 'static,
            $($a : 'static),*
        {
            type ArcsIter = <$dyn_fst as ArcIterator<'a>>::Iter;
            type FstIter = Map<
                Zip<<$dyn_fst as StateIterator<'a>>::Iter, Repeat<&'a Self>>,
                Box<dyn FnMut((StateId, &'a Self)) -> (StateId, Self::ArcsIter, Option<&'a F::W>)>,
            >;

            fn fst_iter(&'a self) -> Self::FstIter {
                let it = repeat(self);
                izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| {
                    (
                        state_id,
                        p.arcs_iter(state_id).unwrap(),
                        p.final_weight(state_id).unwrap(),
                    )
                }))
            }
        }
    };
}

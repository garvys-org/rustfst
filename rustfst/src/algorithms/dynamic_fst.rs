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

            fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
                self.isymt = Some(Rc::clone(&symt))
            }

            fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
                self.osymt = Some(Rc::clone(&symt));
            }

            fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
                self.isymt.take()
            }

            fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
                self.osymt.take()
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
                Box<dyn FnMut((StateId, &'a Self)) -> FstIterData<&'a F::W, Self::ArcsIter>>,
            >;

            fn fst_iter(&'a self) -> Self::FstIter {
                let it = repeat(self);
                izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| FstIterData{

                        state_id,
                        arcs: p.arcs_iter(state_id).unwrap(),
                        final_weight: p.final_weight(state_id).unwrap(),
                        num_arcs: p.num_arcs(state_id).unwrap()
                }))
            }
        }
    };
}

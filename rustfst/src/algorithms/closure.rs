use crate::algorithms::ReplaceFst;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{SymbolTable, TrsVec, EPS_LABEL};
use anyhow::Result;
use std::sync::Arc;
use unsafe_unwrap::UnsafeUnwrap;

/// Defines the different types of closure : Star or Plus.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ClosureType {
    ClosureStar,
    ClosurePlus,
}

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
///  If closure_star then the empty string is transduced to itself with weight `1` as well.
///
/// # Example
///
/// ## Input
/// ![closure_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_in.svg?sanitize=true)
///
/// ## Closure Plus
/// ![closure_out_closure_plus](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_plus.svg?sanitize=true)
///
/// ## Closure Star
/// ![closure_out_closure_star](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_star.svg?sanitize=true)
pub fn closure<W, F>(fst: &mut F, closure_type: ClosureType)
where
    W: Semiring,
    F: MutableFst<W>,
{
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst
            .final_states_iter()
            .map(|s| (s, unsafe { fst.final_weight_unchecked(s).unsafe_unwrap() }))
            .collect();
        for (final_state_id, final_weight) in final_states_id {
            unsafe {
                fst.add_tr_unchecked(
                    final_state_id,
                    Tr::new(EPS_LABEL, EPS_LABEL, final_weight, start_state),
                )
            };
        }
    }

    if closure_type == ClosureType::ClosureStar {
        let nstart = fst.add_state();

        // Add a new start state to allow empty path
        if let Some(start_state_id) = fst.start() {
            unsafe {
                fst.add_tr_unchecked(
                    nstart,
                    Tr::new(EPS_LABEL, EPS_LABEL, W::one(), start_state_id),
                );
            }
        }

        unsafe {
            fst.set_start_unchecked(nstart);
            fst.set_final_unchecked(nstart, W::one());
        }
    }
}

/// Computes the concatenative closure. This version is a delayed FST. If an FST
/// transduces string x to y with weight a, then its closure transduces x to y
/// with weight a, xx to yy with weight Times(a, a), xxx to yyy with weight
/// Times(Times(a, a), a), etc. If closure_type == CLOSURE_STAR, then the empty
/// string is transduced to itself with weight Weight::One() as well.
#[derive(Debug)]
pub struct ClosureFst<W: Semiring, F: Fst<W> + 'static>(ReplaceFst<W, F, F>);

impl<W, F> ClosureFst<W, F>
where
    W: Semiring,
    F: MutableFst<W> + AllocableFst<W>,
{
    //TODO: Use a borrow and not a move
    //TODO: Allow fsts of different types
    pub fn new(fst: F, closure_type: ClosureType) -> Result<Self> {
        let mut rfst = F::new();
        if let Some(isymt) = fst.input_symbols() {
            rfst.set_input_symbols(Arc::clone(isymt));
        }
        if let Some(osymt) = fst.output_symbols() {
            rfst.set_output_symbols(Arc::clone(osymt));
        }
        match closure_type {
            ClosureType::ClosureStar => {
                rfst.add_state();
                unsafe {
                    rfst.set_start_unchecked(0);
                    rfst.set_final_unchecked(0, W::one());
                    rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX, W::one(), 0));
                }
            }
            ClosureType::ClosurePlus => {
                rfst.add_states(2);
                unsafe {
                    rfst.set_start_unchecked(0);
                    rfst.set_final_unchecked(1, W::one());
                    rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX, W::one(), 1));
                    rfst.add_tr_unchecked(1, Tr::new(EPS_LABEL, EPS_LABEL, W::one(), 0));
                }
            }
        };
        let mut fst_tuples = Vec::with_capacity(3);
        fst_tuples.push((0, rfst));
        fst_tuples.push((std::usize::MAX, fst));

        Ok(ClosureFst(ReplaceFst::new(fst_tuples, 0, false)?))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}
impl<W, F> CoreFst<W> for ClosureFst<W, F>
where
    W: Semiring,
    F: Fst<W>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.0.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }
}

impl<'a, W, F> StateIterator<'a> for ClosureFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'a,
{
    type Iter = <ReplaceFst<W, F, F> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<W, F> Fst<W> for ClosureFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_output_symbols()
    }
}

impl<'a, W, F> FstIterator<'a, W> for ClosureFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'a,
{
    type FstIter = <ReplaceFst<W, F, F> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

use anyhow::Result;

use crate::fst_properties::compute_fst_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{Fst, FstIntoIterator};
use crate::semirings::{Semiring, WeightQuantize};
use crate::{Trs, KDELTA};

/// Trait defining the necessary methods that should implement an ExpandedFST e.g
/// a FST where all the states are already computed and not computed on the fly.
pub trait ExpandedFst<W: Semiring>: Fst<W> + Clone + PartialEq + FstIntoIterator<W> {
    /// Returns the number of states that contains the FST. They are all counted even if some states
    /// are not on a successful path (doesn't perform triming).
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.num_states(), 0);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 1);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 2);
    ///
    /// ```
    fn num_states(&self) -> usize;

    /// Compute the properties verified by the Fst.
    fn properties(&self) -> Result<FstProperties> {
        compute_fst_properties(self)
    }

    fn equal_quantized<F2: ExpandedFst<W>>(&self, fst2: &F2) -> bool
    where
        W: WeightQuantize,
    {
        let n = self.num_states();
        if fst2.num_states() != n {
            println!("Not the same number of states");
            return false;
        }
        if self.start() != fst2.start() {
            println!("Not the same start state");
            return false;
        }
        for state in 0..n {
            let trs1 = unsafe { self.get_trs_unchecked(state) };
            let trs2 = unsafe { fst2.get_trs_unchecked(state) };

            if trs1.trs().len() != trs2.trs().len() {
                println!("Not the same number of trs for state {:?}", state);
                return false;
            }

            for (tr1, tr2) in trs1.trs().iter().zip(trs2.trs().iter()) {
                if tr1.ilabel != tr2.ilabel
                    || tr1.olabel != tr2.olabel
                    || tr1.nextstate != tr2.nextstate
                {
                    println!("A tr1 : {:?} tr2 : {:?} for state {:?}", &tr1, &tr2, state);
                    return false;
                }
                let w1 = tr1.weight.quantize(KDELTA).unwrap();
                let w2 = tr2.weight.quantize(KDELTA).unwrap();
                if w1 != w2 {
                    println!("B");
                    return false;
                }
            }

            let fw1 =
                unsafe { self.final_weight_unchecked(state) }.map(|w| w.quantize(KDELTA).unwrap());
            let fw2 =
                unsafe { fst2.final_weight_unchecked(state) }.map(|w| w.quantize(KDELTA).unwrap());

            if fw1 != fw2 {
                return false;
            }
        }

        true
    }
}

use anyhow::Result;
use std::ops::Range;

use crate::algorithms::fst_convert_from_ref;
use crate::algorithms::tr_mappers::QuantizeMapper;
use crate::fst_traits::{AllocableFst, Fst, FstIntoIterator, MutableFst};
use crate::semirings::{Semiring, WeightQuantize};
use crate::{StateId, Trs};

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

    fn states_range(&self) -> Range<StateId> {
        0..(self.num_states() as StateId)
    }

    fn approx_equal<F2: ExpandedFst<W>>(&self, fst2: &F2, delta: f32) -> bool {
        let n = self.num_states();
        if fst2.num_states() != n {
            println!("Not the same number of states");
            return false;
        }
        if self.start() != fst2.start() {
            println!("Not the same start state");
            return false;
        }
        for state in 0..(n as StateId) {
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
                    return false;
                }

                if !tr1.weight.approx_equal(&tr2.weight, delta) {
                    return false;
                }
            }

            let fw1 = unsafe { self.final_weight_unchecked(state) };
            let fw2 = unsafe { fst2.final_weight_unchecked(state) };

            let fw_equal = match (fw1, fw2) {
                (Some(w1), Some(w2)) => w1.approx_equal(w2, delta),
                (Some(_), None) => false,
                (None, Some(_)) => false,
                (None, None) => true,
            };

            if !fw_equal {
                return false;
            }
        }

        true
    }

    fn quantize<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2>
    where
        W: WeightQuantize,
    {
        let mut fst_tr_map: F2 = fst_convert_from_ref(self);
        let mut mapper = QuantizeMapper::default();
        fst_tr_map.tr_map(&mut mapper)?;
        Ok(fst_tr_map)
    }
}

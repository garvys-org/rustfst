use std::cmp::Ordering;
use std::collections::VecDeque;

use anyhow::Result;

use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs, KDELTA};
use std::marker::PhantomData;

struct Isomorphism<'a, W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>> {
    fst_1: &'a F1,
    fst_2: &'a F2,
    state_pairs: Vec<Option<StateId>>,
    queue: VecDeque<(StateId, StateId)>,
    w: PhantomData<W>,
    delta: f32,
    non_det: bool,
}

/// Compare trs in the order input label, output label, weight and nextstate.
pub fn tr_compare<W: Semiring>(tr_1: &Tr<W>, tr_2: &Tr<W>) -> Ordering {
    if tr_1.ilabel < tr_2.ilabel {
        return Ordering::Less;
    }
    if tr_1.ilabel > tr_2.ilabel {
        return Ordering::Greater;
    }
    if tr_1.olabel < tr_2.olabel {
        return Ordering::Less;
    }
    if tr_1.olabel > tr_2.olabel {
        return Ordering::Greater;
    }
    if tr_1.weight < tr_2.weight {
        return Ordering::Less;
    }
    if tr_1.weight > tr_2.weight {
        return Ordering::Greater;
    }
    if tr_1.nextstate < tr_2.nextstate {
        return Ordering::Less;
    }
    if tr_1.nextstate > tr_2.nextstate {
        return Ordering::Greater;
    }
    Ordering::Equal
}

impl<'a, W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>> Isomorphism<'a, W, F1, F2> {
    fn new(fst_1: &'a F1, fst_2: &'a F2, delta: f32) -> Self {
        Self {
            fst_1,
            fst_2,
            state_pairs: vec![None; fst_1.num_states()],
            queue: VecDeque::new(),
            w: PhantomData,
            delta,
            non_det: false,
        }
    }

    // Maintains state correspondences and queue.
    fn pair_state(&mut self, s1: StateId, s2: StateId) -> bool {
        if self.state_pairs[s1 as usize] == Some(s2) {
            return true; // already seen this pair
        } else if self.state_pairs[s1 as usize].is_some() {
            return false; // s1 already paired with another s2
        }
        self.state_pairs[s1 as usize] = Some(s2);
        self.queue.push_back((s1, s2));
        true
    }

    fn ismorphic_state(&mut self, s1: StateId, s2: StateId) -> Result<bool> {
        let fw1 = self.fst_1.final_weight(s1)?;
        let fw2 = self.fst_2.final_weight(s2)?;
        let fw_equal = match (fw1, fw2) {
            (Some(w1), Some(w2)) => w1.approx_equal(w2, self.delta),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        };
        if !fw_equal {
            return Ok(false);
        }

        let ntrs1 = self.fst_1.num_trs(s1)?;
        let ntrs2 = self.fst_2.num_trs(s2)?;

        if ntrs1 != ntrs2 {
            return Ok(false);
        }

        let trs1_owner = self.fst_1.get_trs(s1)?;
        let mut trs1: Vec<_> = trs1_owner.trs().iter().collect();
        let trs2_owner = self.fst_2.get_trs(s2)?;
        let mut trs2: Vec<_> = trs2_owner.trs().iter().collect();

        trs1.sort_by(|a, b| tr_compare(a, b));
        trs2.sort_by(|a, b| tr_compare(a, b));

        for i in 0..trs1.len() {
            let arc1 = trs1[i];
            let arc2 = trs2[i];
            if arc1.ilabel != arc2.ilabel {
                return Ok(false);
            }
            if arc1.olabel != arc2.olabel {
                return Ok(false);
            }
            if !(arc1.weight.approx_equal(&arc2.weight, self.delta)) {
                return Ok(false);
            }
            if !(self.pair_state(arc1.nextstate, arc2.nextstate)) {
                return Ok(false);
            }
            if i > 0 {
                let arc0 = trs1[i - 1];
                if arc1.ilabel == arc0.ilabel
                    && arc1.olabel == arc0.olabel
                    && arc1.weight.approx_equal(&arc0.weight, self.delta)
                {
                    // Any subsequent matching failure maybe a false negative
                    // since we only consider one permutation when pairing destination
                    // states of nondeterministic transitions.
                    self.non_det = true;
                }
            }
        }
        Ok(true)
    }

    fn isomorphic(&mut self) -> Result<bool> {
        // Both FSTs don't have a start state => both don't recognize anything
        if self.fst_1.start().is_none() && self.fst_2.start().is_none() {
            return Ok(true);
        }

        // Only one FST has a start state => false
        if self.fst_1.start().is_none() || self.fst_2.start().is_none() {
            return Ok(false);
        }

        self.pair_state(self.fst_1.start().unwrap(), self.fst_2.start().unwrap());

        while !self.queue.is_empty() {
            let (s1, s2) = self.queue.pop_front().unwrap();
            if !self.ismorphic_state(s1, s2)? {
                if self.non_det {
                    bail!("Isomorphic: Non-determinism as an unweighted automaton. state1 = {} state2 = {}", s1, s2)
                }
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Configuration for isomorphic comparison.
pub struct IsomorphicConfig {
    delta: f32,
}

impl Default for IsomorphicConfig {
    fn default() -> Self {
        Self { delta: KDELTA }
    }
}

impl IsomorphicConfig {
    pub fn new(delta: f32) -> Self {
        Self { delta }
    }
}

/// Determine if two transducers with a certain required determinism
/// have the same states, irrespective of numbering, and the same transitions with
/// the same labels and weights, irrespective of ordering.
///
/// In other words, Isomorphic(A, B) is true if and only if the states of A can
/// be renumbered and the transitions leaving each state reordered so that Equal(A, B) is true.
pub fn isomorphic<W, F1, F2>(fst_1: &F1, fst_2: &F2) -> Result<bool>
where
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
{
    isomorphic_with_config(fst_1, fst_2, IsomorphicConfig::default())
}

/// Determine, with configurable comparison delta, if two transducers with a
/// certain required determinism have the same states, irrespective of
/// numbering, and the same transitions with the same labels and
/// weights, irrespective of ordering.
///
/// In other words, Isomorphic(A, B) is true if and only if the states of A can
/// be renumbered and the transitions leaving each state reordered so that Equal(A, B) is true.
pub fn isomorphic_with_config<W, F1, F2>(
    fst_1: &F1,
    fst_2: &F2,
    config: IsomorphicConfig,
) -> Result<bool>
where
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
{
    let mut iso = Isomorphism::new(fst_1, fst_2, config.delta);
    iso.isomorphic()
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::fst_impls::VectorFst;
    use crate::fst_traits::{MutableFst, SerializableFst};
    use crate::semirings::{LogWeight, Semiring};
    use crate::Tr;

    #[test]
    fn test_isomorphic_1() -> Result<()> {
        let fst_1: VectorFst<LogWeight> = SerializableFst::from_text_string(
            "0\t1\t12\t25\n\
             1\n",
        )?;

        let mut fst_2 = fst_1.clone();
        assert!(isomorphic(&fst_1, &fst_2)?);

        fst_2.add_tr(0, Tr::new(33, 45, LogWeight::new(0.3), 1))?;
        assert!(!isomorphic(&fst_1, &fst_2)?);

        Ok(())
    }

    #[test]
    fn test_isomorphic_2() -> Result<()> {
        let fst_1: VectorFst<LogWeight> = SerializableFst::from_text_string(
            "0\t1\t12\t25\n\
             1\n",
        )?;

        let fst_2: VectorFst<LogWeight> = SerializableFst::from_text_string(
            "1\t0\t12\t25\n\
             0\n",
        )?;

        assert!(isomorphic(&fst_1, &fst_2)?);

        Ok(())
    }
}

use std::cmp::Ordering;
use std::collections::VecDeque;

use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Arc, Result, StateId};

struct Isomorphism<'a, W: Semiring, F1: ExpandedFst<W = W>, F2: ExpandedFst<W = W>> {
    fst_1: &'a F1,
    fst_2: &'a F2,
    state_pairs: Vec<Option<StateId>>,
    queue: VecDeque<(StateId, StateId)>,
}

fn arc_compare<W: Semiring>(arc_1: &Arc<W>, arc_2: &Arc<W>) -> Ordering {
    if arc_1.ilabel < arc_2.ilabel {
        return Ordering::Less;
    }
    if arc_1.ilabel > arc_2.ilabel {
        return Ordering::Greater;
    }
    if arc_1.olabel < arc_2.olabel {
        return Ordering::Less;
    }
    if arc_1.olabel > arc_2.olabel {
        return Ordering::Greater;
    }
    if arc_1.weight < arc_2.weight {
        return Ordering::Less;
    }
    if arc_1.weight > arc_2.weight {
        return Ordering::Greater;
    }
    if arc_1.nextstate < arc_2.nextstate {
        return Ordering::Less;
    }
    if arc_1.nextstate > arc_2.nextstate {
        return Ordering::Greater;
    }
    Ordering::Equal
}

impl<'a, W: Semiring, F1: ExpandedFst<W = W>, F2: ExpandedFst<W = W>> Isomorphism<'a, W, F1, F2> {
    fn new(fst_1: &'a F1, fst_2: &'a F2) -> Self {
        Self {
            fst_1,
            fst_2,
            state_pairs: vec![None; fst_1.num_states()],
            queue: VecDeque::new(),
        }
    }

    // Maintains state correspondences and queue.
    fn pair_state(&mut self, s1: StateId, s2: StateId) -> bool {
        if self.state_pairs[s1] == Some(s2) {
            return true; // already seen this pair
        } else if self.state_pairs[s1].is_some() {
            return false; // s1 already paired with another s2
        }
        self.state_pairs[s1] = Some(s2);
        self.queue.push_back((s1, s2));
        true
    }

    fn ismorphic_state(&mut self, s1: StateId, s2: StateId) -> Result<bool> {
        if !(self.fst_1.final_weight(s1) == self.fst_2.final_weight(s2)) {
            return Ok(false);
        }

        let narcs1 = self.fst_1.num_arcs(s1).unwrap();
        let narcs2 = self.fst_2.num_arcs(s2).unwrap();

        if narcs1 != narcs2 {
            return Ok(false);
        }

        let mut arcs1: Vec<_> = self.fst_1.arcs_iter(s1).unwrap().collect();
        let mut arcs2: Vec<_> = self.fst_2.arcs_iter(s2).unwrap().collect();

        arcs1.sort_by(|a, b| arc_compare(a, b));
        arcs2.sort_by(|a, b| arc_compare(a, b));

        for i in 0..arcs1.len() {
            let arc1 = arcs1[i];
            let arc2 = arcs2[i];
            if arc1.ilabel != arc2.ilabel {
                return Ok(false);
            }
            if arc1.olabel != arc2.olabel {
                return Ok(false);
            }
            if !(arc1.weight == arc2.weight) {
                return Ok(false);
            }
            if !(self.pair_state(arc1.nextstate, arc2.nextstate)) {
                return Ok(false);
            }
            if i > 0 {
                let arc0 = arcs1[i - 1];
                if arc1 == arc0 {
                    bail!("Isomorphic: Non-determinism as an unweighted automaton")
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
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// This operation determines if two transducers with a certain required determinism
/// have the same states, irrespective of numbering, and the same transitions with
/// the same labels and weights, irrespective of ordering.
///
/// In other words, Isomorphic(A, B) is true if and only if the states of A can
/// be renumbered and the transitions leaving each state reordered so that Equal(A, B) is true.
pub fn isomorphic<W, F1, F2>(fst_1: &F1, fst_2: &F2) -> Result<bool>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
{
    let mut iso = Isomorphism::new(fst_1, fst_2);
    iso.isomorphic()
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::fst_impls::VectorFst;
    use crate::fst_traits::{MutableFst, TextParser};
    use crate::semirings::{LogWeight, Semiring};
    use crate::Arc;

    #[test]
    fn test_isomorphic_1() -> Result<()> {
        let fst_1: VectorFst<LogWeight> = TextParser::from_text_string(
            "0\t1\t12\t25\n\
             1\n",
        )?;

        let mut fst_2 = fst_1.clone();
        assert!(isomorphic(&fst_1, &fst_2)?);

        fst_2.add_arc(0, Arc::new(33, 45, LogWeight::new(0.3), 1))?;
        assert!(!isomorphic(&fst_1, &fst_2)?);

        Ok(())
    }

    #[test]
    fn test_isomorphic_2() -> Result<()> {
        let fst_1: VectorFst<LogWeight> = TextParser::from_text_string(
            "0\t1\t12\t25\n\
             1\n",
        )?;

        let fst_2: VectorFst<LogWeight> = TextParser::from_text_string(
            "1\t0\t12\t25\n\
             0\n",
        )?;

        assert!(isomorphic(&fst_1, &fst_2)?);

        Ok(())
    }
}

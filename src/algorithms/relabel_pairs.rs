use std::collections::hash_map::Entry;
use std::collections::HashMap;

use failure::{bail, format_err, ResultExt};

use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::{Result, StateId};

fn iterator_to_hashmap<I>(pairs: I) -> Result<HashMap<StateId, StateId>>
where
    I: IntoIterator<Item = (StateId, StateId)>,
{
    let mut map_labels = HashMap::new();
    for (s1, s2) in pairs {
        match map_labels.entry(s1) {
            Entry::Occupied(_) => bail!("State {:?} is present twice in the relabeling pairs", s1),
            Entry::Vacant(v) => {
                v.insert(s2);
            }
        }
    }
    Ok(map_labels)
}

/// Replaces input and/or output labels using pairs of labels.
///
/// This operation destructively relabels the input and/or output labels of the
/// FST using pairs of the form (old_ID, new_ID); omitted indices are
/// identity-mapped.
///
/// # Example
/// ```
/// use rustfst::utils::transducer;
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::relabel_pairs;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// relabel_pairs(&mut fst, vec![(2,5)], vec![(3,4)]).unwrap();
///
/// assert_eq!(fst, transducer(vec![5].into_iter(), vec![4].into_iter()).unwrap());
/// ```
pub fn relabel_pairs<F, I, J>(fst: &mut F, ipairs: I, opairs: J) -> Result<()>
where
    F: ExpandedFst + MutableFst,
    I: IntoIterator<Item = (StateId, StateId)>,
    J: IntoIterator<Item = (StateId, StateId)>,
{
    let map_ilabels = iterator_to_hashmap(ipairs)
        .with_context(|_| format_err!("Error while creating the HashMap for ipairs"))?;

    let map_olabels = iterator_to_hashmap(opairs)
        .with_context(|_| format_err!("Error while creating the HashMap for opairs"))?;

    let states: Vec<_> = fst.states_iter().collect();
    for state_id in states {
        for arc in fst.arcs_iter_mut(state_id)? {
            if let Some(v) = map_ilabels.get(&arc.ilabel) {
                arc.ilabel = *v;
            }

            if let Some(v) = map_olabels.get(&arc.olabel) {
                arc.olabel = *v;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::Arc;
    use crate::fst_impls::VectorFst;
    use crate::semirings::{IntegerWeight, Semiring};

    #[test]
    fn test_projection_input_generic() -> Result<()> {
        // Initial FST
        let mut fst = VectorFst::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(s0)?;

        fst.add_arc(s0, Arc::new(3, 18, IntegerWeight::new(10), s1))?;
        fst.add_arc(s0, Arc::new(2, 5, IntegerWeight::new(10), s1))?;
        fst.add_arc(s0, Arc::new(5, 9, IntegerWeight::new(18), s2))?;
        fst.add_arc(s0, Arc::new(5, 7, IntegerWeight::new(18), s2))?;
        fst.set_final(s1, IntegerWeight::new(31))?;
        fst.set_final(s2, IntegerWeight::new(45))?;

        // Expected FST
        // Initial FST
        let mut expected_fst = VectorFst::new();
        let s0 = expected_fst.add_state();
        let s1 = expected_fst.add_state();
        let s2 = expected_fst.add_state();
        expected_fst.set_start(s0)?;

        expected_fst.add_arc(s0, Arc::new(45, 51, IntegerWeight::new(10), s1))?;
        expected_fst.add_arc(s0, Arc::new(2, 75, IntegerWeight::new(10), s1))?;
        expected_fst.add_arc(s0, Arc::new(75, 9, IntegerWeight::new(18), s2))?;
        expected_fst.add_arc(s0, Arc::new(75, 85, IntegerWeight::new(18), s2))?;
        expected_fst.set_final(s1, IntegerWeight::new(31))?;
        expected_fst.set_final(s2, IntegerWeight::new(45))?;

        let ipairs = vec![(3, 45), (5, 75)];
        let opairs = vec![(18, 51), (5, 75), (7, 85)];

        relabel_pairs(&mut fst, ipairs, opairs)?;
        assert_eq!(fst, expected_fst);

        Ok(())
    }
}

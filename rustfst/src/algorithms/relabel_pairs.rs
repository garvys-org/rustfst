use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::{bail, format_err, Context, Result};

use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::StateId;

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

/// Replace input and/or output labels using pairs of labels.
///
/// This operation destructively relabels the input and/or output labels of the
/// FST using pairs of the form (old_ID, new_ID); omitted indices are
/// identity-mapped.
///
/// # Example
/// ```
/// #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::relabel_pairs;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// relabel_pairs(&mut fst, vec![(2,5)], vec![(3,4)])?;
///
/// assert_eq!(fst, fst![5 => 4]);
/// # Ok(())
/// # }
/// ```
pub fn relabel_pairs<W, F, I, J>(fst: &mut F, ipairs: I, opairs: J) -> Result<()>
where
    W: Semiring,
    F: MutableFst<W>,
    I: IntoIterator<Item = (StateId, StateId)>,
    J: IntoIterator<Item = (StateId, StateId)>,
{
    let map_ilabels = iterator_to_hashmap(ipairs)
        .with_context(|| format_err!("Error while creating the HashMap for ipairs"))?;

    let map_olabels = iterator_to_hashmap(opairs)
        .with_context(|| format_err!("Error while creating the HashMap for opairs"))?;

    for state_id in fst.states_range() {
        unsafe {
            let mut it_tr = fst.tr_iter_unchecked_mut(state_id);
            for idx_tr in 0..it_tr.len() {
                let tr = it_tr.get_unchecked(idx_tr);

                match (map_ilabels.get(&tr.ilabel), map_olabels.get(&tr.olabel)) {
                    (Some(ilabel), Some(olabel)) => {
                        it_tr.set_labels_unchecked(idx_tr, *ilabel, *olabel)
                    }
                    (Some(ilabel), None) => it_tr.set_ilabel_unchecked(idx_tr, *ilabel),
                    (None, Some(olabel)) => it_tr.set_olabel_unchecked(idx_tr, *olabel),
                    (None, None) => {}
                };
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fst_impls::VectorFst;
    use crate::semirings::{IntegerWeight, Semiring};
    use crate::tr::Tr;

    use super::*;

    #[test]
    fn test_projection_input_generic() -> Result<()> {
        // Initial FST
        let mut fst = VectorFst::<IntegerWeight>::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(s0)?;

        fst.add_tr(s0, Tr::new(3, 18, 10, s1))?;
        fst.add_tr(s0, Tr::new(2, 5, 10, s1))?;
        fst.add_tr(s0, Tr::new(5, 9, 18, s2))?;
        fst.add_tr(s0, Tr::new(5, 7, 18, s2))?;
        fst.set_final(s1, 31)?;
        fst.set_final(s2, 45)?;

        // Expected FST
        // Initial FST
        let mut expected_fst = VectorFst::new();
        let s0 = expected_fst.add_state();
        let s1 = expected_fst.add_state();
        let s2 = expected_fst.add_state();
        expected_fst.set_start(s0)?;

        expected_fst.add_tr(s0, Tr::new(45, 51, IntegerWeight::new(10), s1))?;
        expected_fst.add_tr(s0, Tr::new(2, 75, IntegerWeight::new(10), s1))?;
        expected_fst.add_tr(s0, Tr::new(75, 9, IntegerWeight::new(18), s2))?;
        expected_fst.add_tr(s0, Tr::new(75, 85, IntegerWeight::new(18), s2))?;
        expected_fst.set_final(s1, IntegerWeight::new(31))?;
        expected_fst.set_final(s2, IntegerWeight::new(45))?;

        let ipairs = vec![(3, 45), (5, 75)];
        let opairs = vec![(18, 51), (5, 75), (7, 85)];

        relabel_pairs(&mut fst, ipairs, opairs)?;
        assert_eq!(fst, expected_fst);

        Ok(())
    }
}

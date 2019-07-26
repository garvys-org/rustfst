use std::collections::VecDeque;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::reverse as reverse_f;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::{Semiring, SemiringProperties};
use crate::StateId;

/// This operation computes the shortest distance from the state `state_id` to every state.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::single_source_shortest_distance;
/// # use rustfst::Arc;
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(18), s1));
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(21), s2));
/// fst.add_arc(s1, Arc::new(32, 23, IntegerWeight::new(55), s2));
///
/// let dists = single_source_shortest_distance(&fst, s1).unwrap();
///
/// assert_eq!(dists, vec![
///     IntegerWeight::zero(),
///     IntegerWeight::one(),
///     IntegerWeight::new(55),
/// ]);
///
/// ```
pub fn single_source_shortest_distance<F: ExpandedFst>(
    fst: &F,
    state_id: StateId,
) -> Fallible<Vec<<F as CoreFst>::W>> {
    let mut d = vec![];
    let mut r = vec![];

    // Check whether the wFST contains the state
    if state_id < fst.num_states() {
        while d.len() <= state_id {
            d.push(<F as CoreFst>::W::zero());
            r.push(<F as CoreFst>::W::zero());
        }
        d[state_id] = <F as CoreFst>::W::one();
        r[state_id] = <F as CoreFst>::W::one();

        let mut queue = VecDeque::new();
        queue.push_back(state_id);

        while !queue.is_empty() {
            let state_cour = unsafe { queue.pop_front().unsafe_unwrap() };
            while d.len() <= state_cour {
                d.push(<F as CoreFst>::W::zero());
                r.push(<F as CoreFst>::W::zero());
            }
            let r2 = &r[state_cour].clone();
            r[state_cour] = <F as CoreFst>::W::zero();

            for arc in unsafe { fst.arcs_iter_unchecked(state_cour) } {
                let nextstate = arc.nextstate;
                while d.len() <= nextstate {
                    d.push(<F as CoreFst>::W::zero());
                    r.push(<F as CoreFst>::W::zero());
                }
                if d[nextstate] != d[nextstate].plus(&r2.times(&arc.weight)?)? {
                    d[nextstate] = d[nextstate].plus(&r2.times(&arc.weight)?)?;
                    r[nextstate] = r[nextstate].plus(&r2.times(&arc.weight)?)?;
                    if !queue.contains(&nextstate) {
                        queue.push_back(nextstate);
                    }
                }
            }
        }
    }

    Ok(d)
}

pub fn _shortest_distance<F: ExpandedFst>(fst: &F) -> Fallible<Vec<<F as CoreFst>::W>> {
    if !F::W::properties().contains(SemiringProperties::RIGHT_SEMIRING) {
        bail!("ShortestDistance: Weight needs to be right distributive");
    }
    if let Some(start_state) = fst.start() {
        return single_source_shortest_distance(fst, start_state);
    }
    Ok(vec![])
}

/// This operation computes the shortest distance from the initial state to every state.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::shortest_distance;
/// # use rustfst::Arc;
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(18), s1));
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(21), s2));
/// fst.add_arc(s1, Arc::new(32, 23, IntegerWeight::new(55), s2));
///
/// let dists = shortest_distance(&fst, false).unwrap();
///
/// assert_eq!(dists, vec![
///     IntegerWeight::one(),
///     IntegerWeight::new(18),
///     IntegerWeight::new(21 + 18*55),
/// ]);
///
/// ```
pub fn shortest_distance<F: ExpandedFst>(fst: &F, reverse: bool) -> Fallible<Vec<<F as CoreFst>::W>>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    if !reverse {
        _shortest_distance(fst)
    } else {
        let rfst: VectorFst<_> = reverse_f(fst)?;
        let rdistance = _shortest_distance(&rfst)?;
        let mut distance = vec![];
        while distance.len() < (rdistance.len() - 1) {
            // TODO: Need to find a better to say that W::ReverseWeight::ReverseWeight == W
            let rw = rdistance[distance.len() + 1].reverse()?;
            distance.push(
                unsafe {
                    std::mem::transmute::<
                    &<<<F as CoreFst>::W as Semiring>::ReverseWeight as Semiring>::ReverseWeight,
                    &<F as CoreFst>::W,
                >(&rw)
                }
                .clone(),
            );
        }
        Ok(distance)
    }
}

#[cfg(test)]
mod tests {
    //    use super::*;
    //    use crate::fst_traits::StateIterator;
    //    use crate::semirings::{IntegerWeight, Semiring};
    //    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    //    #[test]
    //    fn test_single_source_shortest_distance_generic() -> Fallible<()> {
    //        for data in get_vector_fsts_for_tests() {
    //            let fst = data.fst;
    //            let d_ref = data.all_distances;
    //
    //            for state in fst.states_iter() {
    //                let d = single_source_shortest_distance(&fst, state)?;
    //                assert_eq!(
    //                    d, d_ref[state],
    //                    "Test failing for single source shortest distance on wFST {:?} at state {:?}",
    //                    data.name, state
    //                );
    //            }
    //
    //            let d = single_source_shortest_distance(&fst, fst.num_states())?;
    //            assert_eq!(
    //                d,
    //                vec![IntegerWeight::zero(); fst.num_states()],
    //                "Test failing for single source shortest distance on wFST {:?} at state {:?}",
    //                data.name,
    //                fst.num_states()
    //            );
    //        }
    //        Ok(())
    //    }
    //
    //    #[test]
    //    fn test_shortest_distance_generic() -> Fallible<()> {
    //        for data in get_vector_fsts_for_tests() {
    //            let fst = data.fst;
    //            let d_ref = data.all_distances;
    //            let d = shortest_distance(&fst, false)?;
    //
    //            if let Some(start_state) = fst.start() {
    //                assert_eq!(
    //                    d, d_ref[start_state],
    //                    "Test failing for all shortest distance on wFST : {:?}",
    //                    data.name
    //                );
    //            } else {
    //                assert_eq!(
    //                    d,
    //                    vec![IntegerWeight::zero(); fst.num_states()],
    //                    "Test failing for all shortest distance on wFST : {:?}",
    //                    data.name
    //                );
    //            }
    //        }
    //        Ok(())
    //    }
}

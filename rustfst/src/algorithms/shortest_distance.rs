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
/// let mut fst = VectorFst::<IntegerWeight>::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_arc(s0, Arc::new(32, 23, 18, s1));
/// fst.add_arc(s0, Arc::new(32, 23, 21, s2));
/// fst.add_arc(s1, Arc::new(32, 23, 55, s2));
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
            d.push(F::W::zero());
            r.push(F::W::zero());
        }
        d[state_id] = F::W::one();
        r[state_id] = F::W::one();

        let mut queue = VecDeque::new();
        queue.push_back(state_id);

        while !queue.is_empty() {
            let state_cour = unsafe { queue.pop_front().unsafe_unwrap() };
            while d.len() <= state_cour {
                d.push(F::W::zero());
                r.push(F::W::zero());
            }
            let r2 = &r[state_cour].clone();
            r[state_cour] = F::W::zero();

            for arc in unsafe { fst.arcs_iter_unchecked(state_cour) } {
                let nextstate = arc.nextstate;
                while d.len() <= nextstate {
                    d.push(F::W::zero());
                    r.push(F::W::zero());
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
/// # use failure::Fallible;
/// fn main() -> Fallible<()> {
/// let mut fst = VectorFst::<IntegerWeight>::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_arc(s0, Arc::new(32, 23, 18, s1));
/// fst.add_arc(s0, Arc::new(32, 23, 21, s2));
/// fst.add_arc(s1, Arc::new(32, 23, 55, s2));
///
/// let dists = shortest_distance(&fst, false)?;
///
/// assert_eq!(dists, vec![
///     IntegerWeight::one(),
///     IntegerWeight::new(18),
///     IntegerWeight::new(21 + 18*55),
/// ]);
/// # Ok(())
/// # }
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


pub mod revamp {
    use failure::Fallible;

    use crate::fst_traits::Fst;
    use crate::algorithms::arc_filters::AnyArcFilter;
    use crate::algorithms::queues::AutoQueue;

    pub fn shortest_distance<F: Fst>(fst: &F, reverse: bool) -> Fallible<Vec<F::W>>{

        if !reverse {
            let arc_filer = AnyArcFilter{};
//            let queue = AutoQueue::
            unimplemented!()
        } else {
            unimplemented!()
        }

        unimplemented!()
    }
}
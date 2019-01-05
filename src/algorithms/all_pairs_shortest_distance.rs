use crate::fst_traits::CoreFst;
use crate::fst_traits::ExpandedFst;
use crate::fst_traits::Fst;
use crate::semirings::{Semiring, StarSemiring};
use crate::Result;

/// This operation computes the shortest distance from each state to every other states.
/// The shortest distance from `p` to `q `is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::all_pairs_shortest_distance;
/// # use rustfst::Arc;
/// # use rustfst::Result;
/// # fn main() -> Result<()> {
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(18), s1));
/// fst.add_arc(s0, Arc::new(32, 23, IntegerWeight::new(21), s2));
/// fst.add_arc(s1, Arc::new(32, 23, IntegerWeight::new(55), s2));
///
/// let dists = all_pairs_shortest_distance(&fst)?;
///
/// assert_eq!(dists, vec![
///     vec![IntegerWeight::ONE, IntegerWeight::new(18), IntegerWeight::new(18*55 + 21)],
///     vec![IntegerWeight::ZERO, IntegerWeight::ONE, IntegerWeight::new(55)],
///     vec![IntegerWeight::ZERO, IntegerWeight::ZERO, IntegerWeight::ONE],
/// ]);
/// # Ok(())
/// # }
/// ```
pub fn all_pairs_shortest_distance<F>(fst: &F) -> Result<(Vec<Vec<F::W>>)>
where
    F: Fst + ExpandedFst,
    F::W: StarSemiring,
{
    let num_states = fst.num_states();

    // Distance between all states are initialized to zero
    let mut d = vec![vec![<F as CoreFst>::W::ZERO; num_states]; num_states];

    // Iterator over the wFST to add the weight of the arcs
    for state_id in fst.states_iter() {
        for arc in fst.arcs_iter(state_id)? {
            let nextstate = arc.nextstate;
            let weight = &arc.weight;

            d[state_id][nextstate] += *weight;
        }
    }

    for k in fst.states_iter() {
        let closure_d_k_k = d[k][k].closure();
        for i in fst.states_iter().filter(|s| *s != k) {
            for j in fst.states_iter().filter(|s| *s != k) {
                let a = (d[i][k].times(&closure_d_k_k)).times(&d[k][j]);
                d[i][j] += a;
            }
        }
        for i in fst.states_iter().filter(|s| *s != k) {
            d[k][i] = closure_d_k_k.times(&d[k][i]);
            d[i][k] = d[i][k].times(&closure_d_k_k);
        }
        d[k][k] = closure_d_k_k;
    }

    Ok(d)
}

#[cfg(test)]
mod test {
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    use super::*;

    #[test]
    fn test_all_pairs_distance_generic() -> Result<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = data.fst;
            let d_ref = data.all_distances;

            let d = all_pairs_shortest_distance(&fst)?;

            assert_eq!(
                d, d_ref,
                "Test failing for all shortest distance on wFST : {:?}",
                data.name
            );
        }
        Ok(())
    }
}

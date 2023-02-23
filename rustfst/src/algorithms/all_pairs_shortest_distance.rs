use anyhow::Result;

use crate::fst_traits::ExpandedFst;
use crate::fst_traits::Fst;
use crate::semirings::StarSemiring;
use crate::Trs;

/// Compute the shortest distance from each state to every other states.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::all_pairs_shortest_distance;
/// # use rustfst::Tr;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let mut fst = VectorFst::<IntegerWeight>::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.add_tr(s0, Tr::new(32, 23, 18, s1));
/// fst.add_tr(s0, Tr::new(32, 23, 21, s2));
/// fst.add_tr(s1, Tr::new(32, 23, 55, s2));
///
/// let dists = all_pairs_shortest_distance(&fst)?;
///
/// assert_eq!(dists, vec![
///     vec![IntegerWeight::one(), IntegerWeight::new(18), IntegerWeight::new(18*55 + 21)],
///     vec![IntegerWeight::zero(), IntegerWeight::one(), IntegerWeight::new(55)],
///     vec![IntegerWeight::zero(), IntegerWeight::zero(), IntegerWeight::one()],
/// ]);
/// # Ok(())
/// # }
/// ```
pub fn all_pairs_shortest_distance<W, F>(fst: &F) -> Result<Vec<Vec<W>>>
where
    F: Fst<W> + ExpandedFst<W>,
    W: StarSemiring,
{
    let num_states = fst.num_states();

    // Distance between all states are initialized to zero
    let mut d = vec![vec![W::zero(); num_states]; num_states];

    // Iterator over the wFST to add the weight of the trs
    for state_id in fst.states_iter() {
        for tr in fst.get_trs(state_id)?.trs() {
            let nextstate = tr.nextstate;
            let weight = &tr.weight;

            d[state_id as usize][nextstate as usize].plus_assign(weight)?;
        }
    }

    for k in fst.states_iter() {
        let u_k = k as usize;
        let closure_d_k_k = d[u_k][u_k].closure();
        for i in fst.states_iter().filter(|s| *s != k) {
            let i = i as usize;
            for j in fst.states_iter().filter(|s| *s != k) {
                let j = j as usize;
                let a = (d[i][u_k].times(&closure_d_k_k)?).times(&d[u_k][j])?;
                d[i][j].plus_assign(a)?;
            }
        }
        for i in fst.states_iter().filter(|s| *s != k) {
            let i = i as usize;
            d[u_k][i] = closure_d_k_k.times(&d[u_k][i])?;
            d[i][u_k] = d[i][u_k].times(&closure_d_k_k)?;
        }
        d[u_k][u_k] = closure_d_k_k;
    }

    Ok(d)
}

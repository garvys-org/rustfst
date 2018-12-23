use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::itertools::iproduct;
use crate::semirings::Semiring;
use crate::Result;

/// This operation computes the composition of two transducers.
/// If `A` transduces string `x` to `y` with weight `a` and `B` transduces `y` to `z`
/// with weight `b`, then their composition transduces string `x` to `z` with weight `a ⊗ b`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::Result;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::compose;
/// # fn main() -> Result<()> {
/// let fst_1 : VectorFst<IntegerWeight> = transducer![1,2 => 2,3];
///
/// let fst_2 : VectorFst<IntegerWeight> = transducer![2,3 => 3,4];
///
/// let fst_ref : VectorFst<IntegerWeight> = transducer![1,2 => 3,4];
///
/// let composed_fst : VectorFst<_> = compose(&fst_1, &fst_2)?;
/// assert_eq!(composed_fst, fst_ref);
/// # Ok(())
/// # }
/// ```
pub fn compose<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Result<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut composed_fst = F3::new();
    let mut queue = VecDeque::new();

    let mut mapping_states = HashMap::new();

    if let (Some(state_state_1), Some(start_state_2)) = (fst_1.start(), fst_2.start()) {
        let start_state = composed_fst.add_state();
        mapping_states.insert((state_state_1, start_state_2), start_state);
        composed_fst.set_start(start_state)?;
        queue.push_back((state_state_1, start_state_2, start_state));
    }

    while !queue.is_empty() {
        let (q1, q2, q) = queue.pop_front().unwrap();

        if let (Some(rho_1), Some(rho_2)) = (fst_1.final_weight(q1), fst_2.final_weight(q2)) {
            composed_fst.set_final(q, rho_1.times(&rho_2))?;
        }

        let arcs_it1 = fst_1.arcs_iter(q1)?;
        let arcs_it2 = fst_2.arcs_iter(q2)?;

        for (arc_1, arc_2) in iproduct!(arcs_it1, arcs_it2) {
            if arc_1.olabel == arc_2.ilabel {
                let n1 = arc_1.nextstate;
                let n2 = arc_2.nextstate;

                let q_prime = match mapping_states.entry((n1, n2)) {
                    Entry::Vacant(v) => {
                        let q_prime = composed_fst.add_state();
                        v.insert(q_prime);
                        queue.push_back((n1, n2, q_prime));
                        q_prime
                    }
                    Entry::Occupied(o) => *o.get(),
                };

                composed_fst.add_arc(
                    q,
                    Arc::new(
                        arc_1.ilabel,
                        arc_2.olabel,
                        arc_1.weight.times(&arc_2.weight),
                        q_prime,
                    ),
                )?;
            }
        }
    }

    Ok(composed_fst)
}

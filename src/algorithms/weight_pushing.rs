use algorithms::shortest_distance;
use failure::format_err;
use fst_traits::{ExpandedFst, FinalStatesIterator, Fst, MutableFst};
use semirings::{Semiring, WeaklyDivisibleSemiring};
use Result;

macro_rules! state_to_dist {
    ($state: expr, $dist: expr) => {
        $dist
            .get($state)
            .ok_or_else(|| format_err!("State {} not in dists array", $state))?;
    };
}

pub fn push_weights<F>(fst: &mut F) -> Result<()>
where
    F: Fst + ExpandedFst + MutableFst,
    F::W: WeaklyDivisibleSemiring,
{
    let dist = shortest_distance(fst)?;
    println!("{:?}", dist);

    let num_states = dist.len();

    for state in 0..num_states {
        let d_s = state_to_dist!(state, dist);

        if d_s.is_zero() {
            continue;
        }
        for arc in fst.arcs_iter_mut(&state)? {
            let d_ns = state_to_dist!(arc.nextstate, dist);
            arc.weight = d_s.inverse().times(&arc.weight.times(d_ns));
        }
    }

    let final_states: Vec<_> = fst.final_states_iter().collect();

    for final_state in final_states {
        let d_s = state_to_dist!(final_state.state_id, dist);
        if d_s.is_zero() {
            continue
        }
        let new_weight = d_s.inverse().times(&final_state.final_weight);
        fst.set_final(&final_state.state_id, new_weight)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_data::text_fst::get_test_data_for_text_parser;

    #[test]
    fn test_push_weights() {
        for data in get_test_data_for_text_parser() {
            let mut fst = data.vector_fst;
            println!("AAA");
            println!("{}", fst);
            push_weights(&mut fst).unwrap();
            println!("{}", fst);
        }
    }
}
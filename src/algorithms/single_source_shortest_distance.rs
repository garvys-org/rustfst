use fst_traits::{CoreFst, ExpandedFst};
use semirings::Semiring;
use std::collections::VecDeque;
use Result;
use StateId;

pub fn single_source_shortest_distance<F: ExpandedFst>(
    fst: &F,
    state: StateId,
) -> Result<Vec<<F as CoreFst>::W>> {
    let num_states = fst.num_states();

    let mut d = vec![];
    let mut r = vec![];

    d.resize(num_states, <F as CoreFst>::W::zero());
    r.resize(num_states, <F as CoreFst>::W::zero());

    // Check whether the wFST contains the state
    if state < fst.num_states() {
        d[state] = <F as CoreFst>::W::one();
        r[state] = <F as CoreFst>::W::one();

        let mut queue = VecDeque::new();
        queue.push_back(state);

        while !queue.is_empty() {
            let state_cour = queue.pop_front().unwrap();
            let r2 = r[state_cour].clone();
            r[state_cour] = <F as CoreFst>::W::zero();

            for arc in fst.arcs_iter(&state_cour)? {
                let nextstate = arc.nextstate;
                if d[nextstate] != d[nextstate].plus(&r2.times(&arc.weight)) {
                    d[nextstate] = d[nextstate].plus(&r2.times(&arc.weight));
                    r[nextstate] = r[nextstate].plus(&r2.times(&arc.weight));
                    if !queue.contains(&nextstate) {
                        queue.push_back(nextstate);
                    }
                }
            }
        }
    }

    Ok(d)
}

pub fn shortest_distance<F: ExpandedFst>(fst: &F) -> Result<Vec<<F as CoreFst>::W>> {
    if let Some(start_state) = fst.start() {
        return single_source_shortest_distance(fst, start_state);
    }
    return Ok(vec![<F as CoreFst>::W::zero(); fst.num_states()]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fst_traits::StateIterator;
    use semirings::{IntegerWeight, Semiring};
    use test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_single_source_shortest_distance_generic() {
        for data in get_vector_fsts_for_tests() {
            let fst = data.fst;
            let d_ref = data.all_distances;

            for state in fst.states_iter() {
                let d = single_source_shortest_distance(&fst, state).unwrap();
                assert_eq!(
                    d, d_ref[state],
                    "Test failing for single source shortest distance on wFST {:?} at state {:?}",
                    data.name, state
                );
            }

            let d = single_source_shortest_distance(&fst, fst.num_states()).unwrap();
            assert_eq!(
                d,
                vec![IntegerWeight::zero(); fst.num_states()],
                "Test failing for single source shortest distance on wFST {:?} at state {:?}",
                data.name,
                fst.num_states()
            );
        }
    }

    #[test]
    fn test_shortest_distance_generic() {
        for data in get_vector_fsts_for_tests() {
            let fst = data.fst;
            let d_ref = data.all_distances;
            let d = shortest_distance(&fst).unwrap();

            if let Some(start_state) = fst.start() {
                assert_eq!(
                    d, d_ref[start_state],
                    "Test failing for all shortest distance on wFST : {:?}",
                    data.name
                );
            } else {
                assert_eq!(
                    d,
                    vec![IntegerWeight::zero(); fst.num_states()],
                    "Test failing for all shortest distance on wFST : {:?}",
                    data.name
                );
            }
        }
    }
}

use fst_traits::{CoreFst, ExpandedFst};
use semirings::Semiring;
use std::collections::VecDeque;

pub fn shortest_distance<F: ExpandedFst>(fst: &F) -> Vec<<F as CoreFst>::W> {
    let num_states = fst.num_states();

    let mut d = vec![];
    let mut r = vec![];

    d.resize(num_states, <F as CoreFst>::W::zero());
    r.resize(num_states, <F as CoreFst>::W::zero());

    let start_state = fst.start().unwrap();

    d[start_state] = <F as CoreFst>::W::one();
    r[start_state] = <F as CoreFst>::W::one();

    // TODO : Move to LinkedHashSet
    let mut queue = VecDeque::new();
    queue.push_back(start_state);

    while !queue.is_empty() {
        let state_cour = queue.pop_front().unwrap();
        let r2 = r[state_cour].clone();
        r[state_cour] = <F as CoreFst>::W::zero();

        for arc in fst.arcs_iter(&state_cour) {
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

    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use fst_traits::MutableFst;
    use semirings::ProbabilityWeight;
    use vector_fst::VectorFst;

    #[test]
    fn test_shortest_distance() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, ProbabilityWeight::new(10.0));
        fst.add_arc(&s1, &s2, 5, 7, ProbabilityWeight::new(18.0));
        fst.add_arc(&s2, &s3, 3, 5, ProbabilityWeight::new(3.0));
        fst.add_arc(&s2, &s3, 5, 7, ProbabilityWeight::new(5.0));
        fst.set_final(&s3, ProbabilityWeight::new(31.0));

        println!("{:?}", shortest_distance(&fst));
    }
}

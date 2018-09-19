use fst::ExpandedFst;
use semirings::Semiring;
use std::collections::VecDeque;

pub fn shortest_distance<W: Semiring, F: ExpandedFst<W>>(fst: &F) -> Vec<W> {
    let num_states = fst.num_states();

    let mut d = vec![];
    let mut r = vec![];

    d.resize(num_states, W::zero());
    r.resize(num_states, W::zero());

    let start_state = fst.start().unwrap();

    d[start_state] = W::one();
    r[start_state] = W::one();

    // TODO : Move to LinkedHashSet
    let mut queue = VecDeque::new();
    queue.push_back(start_state);

    while !queue.is_empty() {
        let state_cour = queue.pop_front().unwrap();
        let r2 = r[state_cour].clone();
        r[state_cour] = W::zero();

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
    use semirings::integer_weight::IntegerWeight;
    use vector_fst::VectorFst;
    use fst::MutableFst;

    #[test]
    fn test_shortest_distance() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, IntegerWeight::new(10));
        fst.add_arc(&s1, &s2, 5, 7, IntegerWeight::new(18));
        fst.add_arc(&s2, &s3, 3, 5, IntegerWeight::new(3));
        fst.add_arc(&s2, &s3, 5, 7, IntegerWeight::new(5));
        fst.set_final(&s3, IntegerWeight::new(31));

        println!("{:?}", shortest_distance(&fst));
    }
}
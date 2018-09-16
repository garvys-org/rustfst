use fst::ExpandedFst;
use semirings::Semiring;
use std::collections::VecDeque;

pub fn shortest_distance<W: Semiring, F: ExpandedFst<W>>(fst: &mut F) -> Vec<W> {
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

        for arc in fst.arc_iter(&state_cour) {
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

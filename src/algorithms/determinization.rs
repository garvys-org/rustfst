use fst_traits::{CoreFst, ExpandedFst, MutableFst};
use semirings::{Semiring, WeaklyDivisibleSemiring};
use std::collections::BTreeMap;
use std::collections::{HashSet, VecDeque};
use Label;
use StateId;

#[derive(PartialEq, Eq, Clone, Ord, PartialOrd)]
struct PairStateWeight<W: Semiring> {
    state: StateId,
    weight: W,
}

impl<W: Semiring> PairStateWeight<W> {
    pub fn new(state: StateId, weight: W) -> Self {
        PairStateWeight { state, weight }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Ord, PartialOrd)]
struct WeightedSubset<W: Semiring> {
    pairs: Vec<PairStateWeight<W>>,
}

impl<W: Semiring> WeightedSubset<W> {
    pub fn from_vec(vec: Vec<PairStateWeight<W>>) -> Self {
        WeightedSubset { pairs: vec }
    }

    pub fn add(&mut self, state: StateId, weight: W) {
        self.pairs.push(PairStateWeight::new(state, weight));
    }

    pub fn input_labels<F: ExpandedFst>(&self, fst: &F) -> HashSet<Label> {
        let mut set = HashSet::new();
        for pair in &self.pairs {
            let state = pair.state;
            for arc in fst.arcs_iter(&state) {
                set.insert(arc.ilabel);
            }
        }
        set
    }

    pub fn nextstates<F: ExpandedFst>(&self, x: Label, fst: &F) -> HashSet<StateId> {
        let mut set = HashSet::new();
        for pair in &self.pairs {
            let state = pair.state;
            for arc in fst.arcs_iter(&state) {
                if arc.ilabel == x {
                    set.insert(arc.nextstate);
                }
            }
        }
        set
    }
}

fn compute_weight<F: ExpandedFst>(
    x: Label,
    weighted_subset: &WeightedSubset<<F as CoreFst>::W>,
    fst: &F,
) -> <F as CoreFst>::W {
    let mut w_prime = None;

    for pair in &weighted_subset.pairs {
        let p = &pair.state;
        let v = &pair.weight;

        for arc in fst.arcs_iter(&p) {
            let w = &arc.weight;

            if arc.ilabel == x {
                let temp = v.times(&w);
                w_prime = w_prime
                    .map(|value: <F as CoreFst>::W| value.plus(&temp))
                    .or_else(|| Some(temp));
            }
        }
    }

    w_prime.unwrap()
}

fn compute_new_weighted_subset<W, F>(
    x: Label,
    w_prime: &W,
    weighted_subset: &WeightedSubset<W>,
    fst: &F,
) -> WeightedSubset<W>
where
    W: WeaklyDivisibleSemiring,
    F: ExpandedFst<W = W>,
{
    let mut new_weighted_subset = WeightedSubset::default();

    for q in weighted_subset.nextstates(x, fst) {
        let mut new_weight = None;
        for pair in &weighted_subset.pairs {
            let p = &pair.state;
            let v = &pair.weight;

            for arc in fst.arcs_iter(&p) {
                if arc.ilabel == x && arc.nextstate == q {
                    let w = &arc.weight;
                    let temp = w_prime.inverse().times(&v.times(&w));
                    new_weight = new_weight
                        .map(|value: W| value.plus(&temp))
                        .or_else(|| Some(temp));
                }
            }
        }
        new_weighted_subset.add(q, new_weight.unwrap());
    }

    new_weighted_subset
}

use std::collections::btree_map::Entry;
pub fn determinize<W, F1, F2>(fst_in: &F1) -> F2
where
    W: WeaklyDivisibleSemiring + Ord + Eq,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W>,
{
    let mut deminized_fst = F2::new();

    let mut mapping_states = BTreeMap::new();

    let mut queue = VecDeque::new();

    let initial_state = deminized_fst.add_state();
    deminized_fst.set_start(&initial_state);

    let initial_subset = WeightedSubset::from_vec(vec![PairStateWeight::new(
        fst_in.start().unwrap(),
        W::one(),
    )]);
    mapping_states.insert(initial_subset.clone(), initial_state);

    queue.push_back(initial_subset);

    while !queue.is_empty() {
        let weighted_subset = queue.pop_front().unwrap();

        for x in weighted_subset.input_labels(fst_in) {
            let w_prime = compute_weight(x, &weighted_subset, fst_in);
            let new_weighted_subset =
                compute_new_weighted_subset(x, &w_prime, &weighted_subset, fst_in);

            if let Entry::Vacant(lol) = mapping_states.entry(new_weighted_subset.clone()) {
                let state_id = deminized_fst.add_state();

                let mut final_weight = None;
                for pair in &new_weighted_subset.pairs {
                    let q = &pair.state;
                    let v = &pair.weight;
                    if let Some(rho_q) = fst_in.final_weight(q) {
                        let temp = v.times(&rho_q);
                        final_weight = final_weight
                            .map(|value: W| value.plus(&temp))
                            .or_else(|| Some(temp));
                    }
                }

                if let Some(pouet) = final_weight {
                    deminized_fst.set_final(&state_id, pouet);
                }

                // Enqueue
                lol.insert(state_id);
            }

            deminized_fst.add_arc(
                &mapping_states[&weighted_subset],
                &mapping_states[&new_weighted_subset],
                x,
                x,
                w_prime,
            );
        }
    }

    deminized_fst
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use semirings::TropicalWeight;
//     use vector_fst::VectorFst;

//     #[test]
//     fn test_determinize() {
//         let mut fst = VectorFst::new();
//         let s0 = fst.add_state();
//         let s1 = fst.add_state();
//         let s2 = fst.add_state();
//         let s3 = fst.add_state();
//         fst.set_start(&s0);
//         fst.set_final(&s3, TropicalWeight::new(0.0));

//         fst.add_arc(&s0, &s1, 1, 1, TropicalWeight::new(1.0));
//         fst.add_arc(&s0, &s2, 1, 1, TropicalWeight::new(2.0));

//         fst.add_arc(&s1, &s1, 2, 2, TropicalWeight::new(3.0));
//         fst.add_arc(&s2, &s2, 2, 2, TropicalWeight::new(3.0));

//         fst.add_arc(&s1, &s3, 3, 3, TropicalWeight::new(5.0));
//         fst.add_arc(&s2, &s3, 4, 4, TropicalWeight::new(6.0));

//         let determinized_fst : VectorFst<TropicalWeight> = determinize(&fst);
//         println!("{:?}", determinized_fst);
//     }
// }

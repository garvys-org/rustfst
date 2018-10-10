use fst_traits::{Fst, StateIterator, ArcIterator};
use semirings::Semiring;
use StateId;
use path::Path;
use std::collections::VecDeque;

/// Trait to iterate over the paths accepted by an FST
pub trait PathsIterator<'a> {
    type W: Semiring;
    type Iter: Iterator<Item = Path<Self::W>>;
    fn paths_iter(&'a self) -> Self::Iter;
}

impl<'a, F> PathsIterator<'a> for F
    where
        F: 'a + Fst,
{
    type W = F::W;
    type Iter = StructPathsIterator<'a, F>;
    fn paths_iter(&'a self) -> Self::Iter {
        StructPathsIterator::new(&self)
    }
}

pub struct StructPathsIterator<'a, F>
    where
        F: 'a + Fst,
{
    fst: &'a F,
    queue: VecDeque<(StateId, Path<F::W>)>,
}

impl<'a, F> StructPathsIterator<'a, F>
where
    F : 'a + Fst
{
    pub fn new(fst: &'a F) -> Self {
        let mut queue = VecDeque::new();

        if let Some(state_start) = fst.start() {
            queue.push_back((state_start, Path::default()));
        }

        StructPathsIterator{
            fst, queue
        }
    }
}

impl<'a, F> Iterator for StructPathsIterator<'a, F>
    where
        F: 'a + Fst,
{
    type Item = Path<F::W>;

    fn next(&mut self) -> Option<Self::Item> {
        while ! self.queue.is_empty() {

            let (state_id, path) = self.queue.pop_front().unwrap();

            for arc in self.fst.arcs_iter(&state_id).unwrap() {
                let mut new_path = path.clone();
                new_path.add_to_path(arc.ilabel, arc.olabel, arc.weight.clone());
                self.queue.push_back((arc.nextstate, new_path));
            }

            if self.fst.is_final(&state_id) {
                return Some(path)
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc::Arc;
    use fst_impls::VectorFst;
    use fst_traits::{
        ArcIterator, CoreFst, ExpandedFst, FinalStatesIterator, MutableArcIterator, MutableFst,
        StateIterator,
    };
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use semirings::{ProbabilityWeight, Semiring};
    use utils::acceptor;
    use std::collections::BTreeSet;

    #[test]
    fn test_paths_iterator_linear_fst() {
        let labels = vec![153, 45, 96];

        let fst: VectorFst<ProbabilityWeight> = acceptor(labels.clone().into_iter()).unwrap();

        assert_eq!(fst.paths_iter().count(), 1);

        for path in fst.paths_iter() {
            assert_eq!(path, Path::new(labels.clone(), labels.clone(), ProbabilityWeight::one()));
        }
    }

    #[test]
    fn test_paths_iterator_small_fst() {
        let mut fst: VectorFst<ProbabilityWeight> = VectorFst::new();

        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();
        let s4 = fst.add_state();

        fst.set_start(&s1).unwrap();
        fst.set_final(&s4, ProbabilityWeight::one()).unwrap();

        fst.add_arc(&s1, Arc::new(1, 1, ProbabilityWeight::new(1.0), s2)).unwrap();
        fst.add_arc(&s1, Arc::new(2, 2, ProbabilityWeight::new(2.0), s3)).unwrap();
        fst.add_arc(&s1, Arc::new(3, 3, ProbabilityWeight::new(3.0), s4)).unwrap();
        fst.add_arc(&s2, Arc::new(4, 4, ProbabilityWeight::new(4.0), s4)).unwrap();
        fst.add_arc(&s3, Arc::new(5, 5, ProbabilityWeight::new(5.0), s4)).unwrap();

        assert_eq!(fst.paths_iter().count(), 3);

        // TODO: ADD ASSERT

//        let mut paths_ref = BTreeSet::new();
//        paths_ref.insert(Path::new(vec![1, 4], vec![1, 4], ProbabilityWeight::new(4.0)));
//        paths_ref.insert(Path::new(vec![2, 5], vec![2, 5], ProbabilityWeight::new(10.0)));
//        paths_ref.insert(Path::new(vec![3], vec![3], ProbabilityWeight::new(3.0)));
//
//        let paths : BTreeSet<_> = fst.paths_iter().collect();


//        for path in fst.paths_iter() {
//            println!("{:?}", path);
//        }
    }
}
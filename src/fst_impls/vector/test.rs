#[cfg(test)]
mod tests {
    use arc::Arc;
    use fst_impls::VectorFst;
    use fst_traits::{
        ArcIterator, CoreFst, ExpandedFst, FinalStatesIterator, MutableArcIterator, MutableFst,
        StateIterator,
    };
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use semirings::{ProbabilityWeight, Semiring};

    #[test]
    fn test_small_fst() {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(&s1).unwrap();

        // Arcs
        let arc_1 = Arc::new(3, 5, ProbabilityWeight::new(10.0), s2);
        fst.add_arc(&s1, arc_1.clone()).unwrap();

        assert_eq!(fst.num_arcs(), 1);

        let arc_2 = Arc::new(5, 7, ProbabilityWeight::new(18.0), s2);
        fst.add_arc(&s1, arc_2.clone()).unwrap();
        assert_eq!(fst.num_arcs(), 2);
        assert_eq!(fst.arcs_iter(&s1).unwrap().count(), 2);

        // Iterates on arcs leaving s1
        let mut it_s1 = fst.arcs_iter(&s1).unwrap();

        let arc = it_s1.next().unwrap();
        assert_eq!(arc_1, *arc);

        let arc = it_s1.next().unwrap();
        assert_eq!(arc_2, *arc);

        let arc = it_s1.next();
        assert!(arc.is_none());

        // Iterates on arcs leaving s2
        let mut it_s2 = fst.arcs_iter(&s2).unwrap();

        let d = it_s2.next();
        assert!(d.is_none());
    }

    #[test]
    fn test_mutable_iter_arcs_small() {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(&s1).unwrap();

        // Arcs
        let arc_1 = Arc::new(3, 5, ProbabilityWeight::new(10.0), s2);
        fst.add_arc(&s1, arc_1.clone()).unwrap();
        let arc_2 = Arc::new(5, 7, ProbabilityWeight::new(18.0), s2);
        fst.add_arc(&s1, arc_2.clone()).unwrap();

        let new_arc_1 = Arc::new(15, 29, ProbabilityWeight::new(33.0), s2 + 55);

        // Modify first arc leaving s1
        fst.arcs_iter_mut(&s1)
            .unwrap()
            .next()
            .unwrap()
            .set_value(&new_arc_1);

        let mut it_s1 = fst.arcs_iter(&s1).unwrap();

        let arc = it_s1.next().unwrap();
        assert_eq!(new_arc_1, *arc);

        let arc = it_s1.next().unwrap();
        assert_eq!(arc_2, *arc);

        assert!(it_s1.next().is_none());
    }

    #[test]
    fn test_start_states() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Should be no start state
        assert_eq!(fst.start(), None);

        // New start state
        fst.set_start(&states[18]).unwrap();
        assert_eq!(fst.start(), Some(states[18]));

        // New start state
        fst.set_start(&states[32]).unwrap();
        assert_eq!(fst.start(), Some(states[32]));
    }

    #[test]
    fn test_only_final_states() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Number of final states should be zero
        assert_eq!(fst.final_states_iter().count(), 0);

        // Set all states as final
        states
            .iter()
            .for_each(|v| fst.set_final(&v, ProbabilityWeight::one()).unwrap());

        // Number of final states should be n_states
        assert_eq!(fst.final_states_iter().count(), n_states);
    }

    #[test]
    fn test_final_weight() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;
        let n_final_states = 300;

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // None of the states are final => None final weight
        assert!(fst
            .states_iter()
            .map(|state_id| fst.final_weight(&state_id))
            .all(|v| v.is_none()));

        // Select randomly n_final_states
        let mut rg = StdRng::from_seed([53; 32]);
        rg.shuffle(&mut states);
        let final_states: Vec<_> = states.into_iter().take(n_final_states).collect();

        // Set those as final with a weight equals to its position in the vector
        final_states.iter().enumerate().for_each(|(idx, state_id)| {
            fst.set_final(state_id, ProbabilityWeight::new(idx as f32))
                .unwrap()
        });

        // Check they are final with the correct weight
        assert!(final_states.iter().all(|state_id| fst.is_final(state_id)));
        assert!(final_states
            .iter()
            .enumerate()
            .all(|(idx, state_id)| fst.final_weight(state_id)
                == Some(ProbabilityWeight::new(idx as f32))));
    }

    #[test]
    fn test_del_state_arcs() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.add_arc(&s1, Arc::new(0, 0, ProbabilityWeight::one(), s2))
            .unwrap();
        fst.add_arc(&s2, Arc::new(0, 0, ProbabilityWeight::one(), s1))
            .unwrap();
        fst.add_arc(&s2, Arc::new(0, 0, ProbabilityWeight::one(), s2))
            .unwrap();

        assert_eq!(fst.num_arcs(), 3);
        assert_eq!(fst.arcs_iter(&s1).unwrap().count(), 1);
        assert_eq!(fst.arcs_iter(&s2).unwrap().count(), 2);

        fst.del_state(&s1).unwrap();

        assert_eq!(fst.num_arcs(), 1);

        let only_state = fst.states_iter().next().unwrap();
        assert_eq!(fst.arcs_iter(&only_state).unwrap().count(), 1);
    }

    #[test]
    fn test_deleting_twice_same_state() {
        let mut fst1 = VectorFst::<ProbabilityWeight>::new();

        let s = fst1.add_state();

        let mut fst2 = fst1.clone();

        // Perform test with del_state
        assert!(fst1.del_state(&s).is_ok());
        assert!(fst1.del_state(&s).is_err());

        // Perform test with del_states
        let states_to_remove = vec![s, s];
        assert!(fst2.del_states(states_to_remove.into_iter()).is_err());
    }

    #[test]
    fn test_del_multiple_states() {
        // Test to check that
        let mut fst1 = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst1.add_state();
        let s2 = fst1.add_state();

        let mut fst2 = fst1.clone();

        // Pass because s2 state id is modified by the first call
        assert!(fst1.del_state(&s1).is_ok());
        assert!(fst1.del_state(&s2).is_err());

        // Test that the above issue doesn't arrive when calling del_states
        let states_to_remove = vec![s1, s2];
        assert!(fst2.del_states(states_to_remove.into_iter()).is_ok());
    }

    #[test]
    fn test_del_states_big() {
        let n_states = 1000;
        let n_states_to_delete = 300;

        let mut fst = VectorFst::<ProbabilityWeight>::new();

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Check those N states do exist
        assert_eq!(fst.num_states(), n_states);

        // Sample n_states_to_delete to remove from the FST
        let mut rg = StdRng::from_seed([53; 32]);
        rg.shuffle(&mut states);
        let states_to_delete: Vec<_> = states.into_iter().take(n_states_to_delete).collect();

        fst.del_states(states_to_delete).unwrap();

        // Check they are correctly removed
        assert_eq!(fst.num_states(), n_states - n_states_to_delete);
    }
}

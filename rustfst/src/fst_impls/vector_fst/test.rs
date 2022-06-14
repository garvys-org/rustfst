#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use anyhow::Result;

    use crate::fst_impls::VectorFst;
    use crate::fst_traits::{
        CoreFst, ExpandedFst, Fst, MutableFst, SerializableFst, StateIterator,
    };
    use crate::semirings::{ProbabilityWeight, Semiring, TropicalWeight};
    use crate::tr::Tr;
    use crate::{SymbolTable, Trs};
    use rand::seq::SliceRandom;
    use std::sync::Arc;

    #[test]
    fn test_small_fst() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(s1)?;

        // Trs
        let tr_1 = Tr::new(3, 5, 10.0, s2);
        fst.add_tr(s1, tr_1.clone())?;

        assert_eq!(fst.num_trs(s1).unwrap(), 1);

        let tr_2 = Tr::new(5, 7, 18.0, s2);
        fst.add_tr(s1, tr_2.clone())?;
        assert_eq!(fst.num_trs(s1).unwrap(), 2);
        assert_eq!(fst.get_trs(s1)?.trs().iter().count(), 2);

        // Iterates on trs leaving s1
        let it_s1 = fst.get_trs(s1)?;
        assert_eq!(it_s1.len(), 2);
        assert_eq!(tr_1, it_s1.trs()[0]);
        assert_eq!(tr_2, it_s1.trs()[1]);

        // Iterates on trs leaving s2
        let it_s2 = fst.get_trs(s2)?;

        assert_eq!(it_s2.len(), 0);
        Ok(())
    }

    #[test]
    fn test_mutable_iter_trs_small() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(s1)?;

        // Trs
        let tr_1 = Tr::new(3, 5, 10.0, s2);
        fst.add_tr(s1, tr_1.clone())?;
        let tr_2 = Tr::new(5, 7, 18.0, s2);
        fst.add_tr(s1, tr_2.clone())?;

        let new_tr_1 = Tr::new(15, 29, 33.0, s2 + 55);

        // Modify first transition leaving s1
        let mut tr_it = fst.tr_iter_mut(s1)?;
        tr_it.set_tr(0, new_tr_1.clone())?;

        let it_s1 = fst.get_trs(s1)?;
        assert_eq!(new_tr_1, it_s1[0]);
        assert_eq!(tr_2, it_s1[1]);
        assert_eq!(it_s1.len(), 2);

        Ok(())
    }

    #[test]
    fn test_start_states() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Should be no start state
        assert_eq!(fst.start(), None);

        // New start state
        fst.set_start(states[18])?;
        assert_eq!(fst.start(), Some(states[18]));

        // New start state
        fst.set_start(states[32])?;
        assert_eq!(fst.start(), Some(states[32]));

        Ok(())
    }

    #[test]
    fn test_only_final_states() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Number of final states should be zero
        assert_eq!(fst.final_states_iter().count(), 0);

        // Set all states as final
        states
            .iter()
            .for_each(|v| fst.set_final(*v, ProbabilityWeight::one()).unwrap());

        // Number of final states should be n_states
        assert_eq!(fst.final_states_iter().count(), n_states);

        Ok(())
    }

    #[test]
    fn test_final_weight() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;
        let n_final_states = 300;

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // None of the states are final => None final weight
        assert!(fst
            .states_iter()
            .map(|state_id| fst.final_weight(state_id).unwrap())
            .all(|v| v.is_none()));

        // Select randomly n_final_states
        let mut rg = StdRng::from_seed([53; 32]);
        states.shuffle(&mut rg);
        let final_states: Vec<_> = states.into_iter().take(n_final_states).collect();

        // Set those as final with a weight equals to its position in the vector
        final_states.iter().enumerate().for_each(|(idx, state_id)| {
            fst.set_final(*state_id, ProbabilityWeight::new(idx as f32 + 1_f32))
                .unwrap()
        });

        // Check they are final with the correct weight
        assert!(final_states
            .iter()
            .all(|state_id| fst.is_final(*state_id).unwrap()));
        assert!(final_states
            .iter()
            .enumerate()
            .all(|(idx, state_id)| fst.final_weight(*state_id).unwrap()
                == Some(ProbabilityWeight::new(idx as f32 + 1_f32))));
        Ok(())
    }

    #[test]
    fn test_del_state_trs() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::one(), s2))?;
        fst.add_tr(s2, Tr::new(0, 0, ProbabilityWeight::one(), s1))?;
        fst.add_tr(s2, Tr::new(0, 0, ProbabilityWeight::one(), s2))?;

        assert_eq!(fst.num_trs(s1)?, 1);
        assert_eq!(fst.num_trs(s2)?, 2);
        assert_eq!(fst.get_trs(s1)?.len(), 1);
        assert_eq!(fst.get_trs(s2)?.len(), 2);

        fst.del_state(s1)?;

        assert_eq!(fst.num_trs(0)?, 1);

        let only_state = fst.states_iter().next().unwrap();
        assert_eq!(fst.get_trs(only_state)?.len(), 1);
        Ok(())
    }

    #[test]
    fn test_deleting_twice_same_state() -> Result<()> {
        let mut fst1 = VectorFst::<ProbabilityWeight>::new();

        let s = fst1.add_state();

        //        let mut fst2 = fst1.clone();

        // Perform test with del_state
        assert!(fst1.del_state(s).is_ok());
        assert!(fst1.del_state(s).is_err());

        // Perform test with del_states
        //        let states_to_remove = vec![s, s];
        //        assert!(fst2.del_states(states_to_remove.into_iter()).is_err());
        Ok(())
    }

    #[test]
    fn test_del_multiple_states() {
        // Test to check that
        let mut fst1 = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst1.add_state();
        let s2 = fst1.add_state();

        let mut fst2 = fst1.clone();

        // Pass because s2 state id is modified by the first call
        assert!(fst1.del_state(s1).is_ok());
        assert!(fst1.del_state(s2).is_err());

        // Test that the above issue doesn't arrive when calling del_states
        let states_to_remove = vec![s1, s2];
        assert!(fst2.del_states(states_to_remove.into_iter()).is_ok());
    }

    #[test]
    fn test_del_states_big() -> Result<()> {
        let n_states = 1000;
        let n_states_to_delete = 300;

        let mut fst = VectorFst::<ProbabilityWeight>::new();

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Check those N states do exist
        assert_eq!(fst.num_states(), n_states);

        // Sample n_states_to_delete to remove from the FST
        let mut rg = StdRng::from_seed([53; 32]);
        states.shuffle(&mut rg);
        let states_to_delete: Vec<_> = states.into_iter().take(n_states_to_delete).collect();

        fst.del_states(states_to_delete)?;

        // Check they are correctly removed
        assert_eq!(fst.num_states(), n_states - n_states_to_delete);
        Ok(())
    }

    #[test]
    fn test_parse_single_final_state() -> Result<()> {
        let parsed_fst = VectorFst::<TropicalWeight>::from_text_string("0\tInfinity\n")?;

        let mut fst_ref: VectorFst<TropicalWeight> = VectorFst::new();

        fst_ref.add_state();
        fst_ref.set_start(0)?;

        assert_eq!(fst_ref, parsed_fst);

        Ok(())
    }

    #[test]
    fn test_del_all_states() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::one(), s2))?;
        fst.add_tr(s2, Tr::new(0, 0, ProbabilityWeight::one(), s1))?;
        fst.add_tr(s2, Tr::new(0, 0, ProbabilityWeight::one(), s2))?;

        fst.set_start(s1)?;
        fst.set_final(s2, ProbabilityWeight::one())?;

        assert_eq!(fst.num_states(), 2);
        fst.del_all_states();
        assert_eq!(fst.num_states(), 0);

        Ok(())
    }

    #[test]
    fn test_attach_symt() -> Result<()> {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.add_tr(s1, Tr::new(1, 0, ProbabilityWeight::one(), s2))?;
        fst.add_tr(s2, Tr::new(2, 0, ProbabilityWeight::one(), s1))?;
        fst.add_tr(s2, Tr::new(3, 0, ProbabilityWeight::one(), s2))?;

        fst.set_start(s1)?;
        fst.set_final(s2, ProbabilityWeight::one())?;

        // Test input symbol table
        {
            let mut symt = SymbolTable::new();
            symt.add_symbol("a"); // 1
            symt.add_symbol("b"); // 2
            symt.add_symbol("c"); // 3

            fst.set_input_symbols(Arc::new(symt));
        }
        {
            let symt = fst.input_symbols();
            assert!(symt.is_some());
            let symt = symt.unwrap();
            assert_eq!(symt.len(), 4);
        }

        // Test output symbol table
        {
            let symt = SymbolTable::new();
            fst.set_output_symbols(Arc::new(symt));
        }
        {
            let symt = fst.output_symbols();
            assert!(symt.is_some());
            let symt = symt.unwrap();
            assert_eq!(symt.len(), 1);
        }

        Ok(())
    }
}

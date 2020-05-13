use crate::algorithms::factor_weight::Element;
use crate::algorithms::lazy_fst_revamp::StateTable;
use crate::semirings::Semiring;
use crate::StateId;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct FactorWeightStateTable<W: Semiring> {
    state_table: StateTable<Element<W>>,
    unfactored: Mutex<HashMap<StateId, StateId>>,
    factor_tr_weights: bool,
}

impl<W: Semiring> FactorWeightStateTable<W> {
    pub fn new(factor_tr_weights: bool) -> Self {
        Self {
            factor_tr_weights,
            state_table: StateTable::new(),
            unfactored: Mutex::new(HashMap::new()),
        }
    }

    pub fn find_tuple(&self, tuple_id: StateId) -> Element<W> {
        self.state_table.find_tuple(tuple_id)
    }

    pub fn find_state(&self, elt: &Element<W>) -> StateId {
        if !self.factor_tr_weights && elt.weight.is_one() && elt.state.is_some() {
            let mut unfactored = self.unfactored.lock().unwrap();
            let old_state = elt.state.unwrap();
            if !unfactored.contains_key(&elt.state.unwrap()) {
                let new_state = self.state_table.insert(elt.clone());
                unfactored.insert(old_state, new_state);
            }
            unfactored[&old_state]
        } else {
            self.state_table.find_id_from_ref(&elt)
        }
    }
}

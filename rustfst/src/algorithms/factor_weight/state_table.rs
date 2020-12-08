use std::collections::HashMap;
use std::sync::Mutex;

use bimap::BiHashMap;

use crate::algorithms::factor_weight::Element;
use crate::semirings::Semiring;
use crate::StateId;

#[derive(Debug)]
struct InnerStateTable<W: Semiring> {
    bimap: BiHashMap<StateId, Element<W>>,
    unfactored: HashMap<StateId, StateId>,
}

impl<W: Semiring> InnerStateTable<W> {
    fn new() -> Self {
        Self {
            bimap: BiHashMap::new(),
            unfactored: HashMap::new(),
        }
    }

    fn find_tuple_bimap(&self, tuple_id: StateId) -> Element<W> {
        self.bimap.get_by_left(&tuple_id).unwrap().clone()
    }

    fn find_id_or_insert_bimap(&mut self, elt: &Element<W>) -> StateId {
        if !self.bimap.contains_right(elt) {
            let n = self.bimap.len() as StateId;
            self.bimap.insert(n, elt.clone());
            return n;
        }
        *self.bimap.get_by_right(elt).unwrap()
    }

    pub fn insert_bimap(&mut self, tuple: Element<W>) -> StateId {
        let n = self.bimap.len() as StateId;
        self.bimap.insert(n, tuple);
        n
    }
}

#[derive(Debug)]
pub struct FactorWeightStateTable<W: Semiring> {
    inner_state_table: Mutex<InnerStateTable<W>>,
    factor_tr_weights: bool,
}

impl<W: Semiring> FactorWeightStateTable<W> {
    pub fn new(factor_tr_weights: bool) -> Self {
        Self {
            factor_tr_weights,
            inner_state_table: Mutex::new(InnerStateTable::new()),
        }
    }

    pub fn find_tuple(&self, tuple_id: StateId) -> Element<W> {
        let inner_state_table = self.inner_state_table.lock().unwrap();
        inner_state_table.find_tuple_bimap(tuple_id)
    }

    pub fn find_state(&self, elt: &Element<W>) -> StateId {
        let mut inner_state_table = self.inner_state_table.lock().unwrap();
        if !self.factor_tr_weights && elt.weight.is_one() && elt.state.is_some() {
            let old_state = elt.state.unwrap();
            if !inner_state_table
                .unfactored
                .contains_key(&elt.state.unwrap())
            {
                let new_state = inner_state_table.insert_bimap(elt.clone());
                inner_state_table.unfactored.insert(old_state, new_state);
            }
            inner_state_table.unfactored[&old_state]
        } else {
            inner_state_table.find_id_or_insert_bimap(elt)
        }
    }
}

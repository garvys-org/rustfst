use std::cell::RefCell;
use std::collections::HashMap;

use crate::algorithms::encode::EncodeType;
use crate::algorithms::FinalTr;
use crate::{Label, Semiring, Tr, EPS_LABEL};
use std::collections::hash_map::Entry;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EncodeTuple<W: Semiring> {
    pub ilabel: Label,
    pub olabel: Label,
    pub weight: W,
}

pub struct EncodeTableMut<W: Semiring> {
    pub encode_type: EncodeType,
    // FIXME : Store references ?
    id_to_tuple: Vec<EncodeTuple<W>>,
    tuple_to_id: HashMap<EncodeTuple<W>, usize>,
}

pub struct EncodeTable<W: Semiring>(pub RefCell<EncodeTableMut<W>>);

impl<W: Semiring> EncodeTableMut<W> {
    pub fn new(encode_type: EncodeType) -> Self {
        EncodeTableMut {
            encode_type,
            id_to_tuple: vec![],
            tuple_to_id: HashMap::new(),
        }
    }

    pub fn tr_to_tuple(&self, tr: &Tr<W>) -> EncodeTuple<W> {
        EncodeTuple {
            ilabel: tr.ilabel,
            olabel: if self.encode_type.encode_labels() {
                tr.olabel
            } else {
                EPS_LABEL
            },
            weight: if self.encode_type.encode_weights() {
                tr.weight.clone()
            } else {
                W::one()
            },
        }
    }

    pub fn final_tr_to_tuple(&self, tr: &FinalTr<W>) -> EncodeTuple<W> {
        EncodeTuple {
            ilabel: tr.ilabel,
            olabel: if self.encode_type.encode_labels() {
                tr.olabel
            } else {
                EPS_LABEL
            },
            weight: if self.encode_type.encode_weights() {
                tr.weight.clone()
            } else {
                W::one()
            },
        }
    }

    pub fn encode(&mut self, tuple: EncodeTuple<W>) -> usize {
        let a = match self.tuple_to_id.entry(tuple.clone()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let new_id = self.id_to_tuple.len();
                self.id_to_tuple.push(tuple);
                *e.insert(new_id)
            }
        };
        a + 1
    }

    pub fn decode(&mut self, tuple_id: usize) -> Option<&EncodeTuple<W>> {
        self.id_to_tuple.get(tuple_id - 1)
    }
}

impl<W: Semiring> Default for EncodeTableMut<W> {
    fn default() -> Self {
        Self::new(EncodeType::EncodeWeightsAndLabels)
    }
}

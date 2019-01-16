use std::collections::hash_map::{Entry, HashMap};

use crate::algorithms::ArcMapper;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Arc;
use crate::Label;
use crate::EPS_LABEL;

#[derive(PartialEq, Eq, Hash, Clone)]
struct EncodeTuple<W: Semiring> {
    ilabel: Label,
    olabel: Label,
    weight: W,
}

struct EncodeTable<W: Semiring> {
    encode_labels: bool,
    encode_weights: bool,
    // FIXME : Store references ?
    id_to_tuple: Vec<EncodeTuple<W>>,
    tuple_to_id: HashMap<EncodeTuple<W>, usize>,
}

impl<W: Semiring> EncodeTable<W> {
    pub fn new(encode_labels: bool, encode_weights: bool) -> Self {
        EncodeTable {
            encode_labels,
            encode_weights,
            id_to_tuple: vec![],
            tuple_to_id: HashMap::new(),
        }
    }

    fn arc_to_tuple(&self, arc: &Arc<W>) -> EncodeTuple<W> {
        EncodeTuple {
            ilabel: arc.ilabel,
            olabel: if self.encode_labels {
                arc.olabel
            } else {
                EPS_LABEL
            },
            weight: if self.encode_weights {
                arc.weight.clone()
            } else {
                W::one()
            },
        }
    }

    pub fn encode(&mut self, arc: &Arc<W>) -> usize {
        let tuple = self.arc_to_tuple(arc);

        match self.tuple_to_id.entry(tuple.clone()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let new_id = self.id_to_tuple.len();
                self.id_to_tuple.push(tuple);
                *e.insert(new_id)
            }
        }
    }

    pub fn decode(&mut self, tuple_id: usize) -> Option<&EncodeTuple<W>> {
        self.id_to_tuple.get(tuple_id)
    }
}

impl<W: Semiring> Default for EncodeTable<W> {
    fn default() -> Self {
        Self::new(true, true)
    }
}

struct EncodeMapper<W: Semiring> {
    encode_table: EncodeTable<W>,
}

impl<W: Semiring> EncodeMapper<W> {
    pub fn new(encode_labels: bool, encode_weights: bool) -> Self {
        EncodeMapper {
            encode_table: EncodeTable::new(encode_labels, encode_weights),
        }
    }
}

impl<W: Semiring> ArcMapper<W> for EncodeMapper<W> {
    fn arc_map(&mut self, arc: &mut Arc<W>) {
        let label = self.encode_table.encode(arc);
        arc.ilabel = label;
        if self.encode_table.encode_labels {
            arc.olabel = label;
        }
        if self.encode_table.encode_weights {
            arc.weight.set_value(W::one().value());
        }
    }

    fn final_weight_map(&mut self, weight: &mut W) {
        if self.encode_table.encode_weights {
            weight.set_value(W::one().value());
        }
    }
}

pub fn encode<F>(fst: &mut F, encode_labels: bool, encode_weights: bool)
where
    F: MutableFst,
{
    let mut encode_mapper = EncodeMapper::new(encode_labels, encode_weights);
    fst.arc_map(&mut encode_mapper);
}

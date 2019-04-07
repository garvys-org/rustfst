use std::collections::hash_map::{Entry, HashMap};

use failure::{Fallible, ResultExt};

use crate::algorithms::{rm_final_epsilon, ArcMapper, FinalArc, MapFinalAction};
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;
use crate::Label;
use crate::EPS_LABEL;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EncodeTuple<W: Semiring> {
    ilabel: Label,
    olabel: Label,
    weight: W,
}

pub struct EncodeTable<W: Semiring> {
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

    pub fn arc_to_tuple(&self, arc: &Arc<W>) -> EncodeTuple<W> {
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

    pub fn final_arc_to_tuple(&self, arc: &FinalArc<W>) -> EncodeTuple<W> {
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
    fn arc_map(&mut self, arc: &mut Arc<W>) -> Fallible<()> {
        let tuple = self.encode_table.arc_to_tuple(arc);
        let label = self.encode_table.encode(tuple);
        arc.ilabel = label;
        if self.encode_table.encode_labels {
            arc.olabel = label;
        }
        if self.encode_table.encode_weights {
            arc.weight.set_value(W::one().value());
        }
        Ok(())
    }

    fn final_arc_map(&mut self, final_arc: &mut FinalArc<W>) -> Fallible<()> {
        if self.encode_table.encode_weights {
            let tuple = self.encode_table.final_arc_to_tuple(final_arc);
            let label = self.encode_table.encode(tuple);
            final_arc.ilabel = label;
            if self.encode_table.encode_labels {
                final_arc.olabel = label;
            }
            if self.encode_table.encode_weights {
                final_arc.weight.set_value(W::one().value());
            }
        }
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        if self.encode_table.encode_weights {
            MapFinalAction::MapRequireSuperfinal
        } else {
            MapFinalAction::MapNoSuperfinal
        }
    }
}

struct DecodeMapper<W: Semiring> {
    encode_table: EncodeTable<W>,
}

impl<W: Semiring> DecodeMapper<W> {
    pub fn new(encode_table: EncodeTable<W>) -> Self {
        DecodeMapper { encode_table }
    }
}

impl<W: Semiring> ArcMapper<W> for DecodeMapper<W> {
    fn arc_map(&mut self, arc: &mut Arc<W>) -> Fallible<()> {
        let tuple = self.encode_table.decode(arc.ilabel).unwrap().clone();
        arc.ilabel = tuple.ilabel;
        if self.encode_table.encode_labels {
            arc.olabel = tuple.olabel;
        }
        if self.encode_table.encode_weights {
            arc.weight = tuple.weight;
        }
        Ok(())
    }

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<W>) -> Fallible<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

/// The `encode` operation allows the representation of a weighted transducer as a weighted automaton,
/// an unweighted transducer or an unweighted automaton by considering the pair
/// (input label, output), the pair (input label, weight) or the triple (input label,
/// output label, weight) as a single label depending on the value
/// of the encode flags: `encode_labels` and `encode_weights`.
///
/// The encoding of each pair or triple of labels and/or weights as a unique key is stored
/// in an `EncodeTable` object.
pub fn encode<F>(
    fst: &mut F,
    encode_labels: bool,
    encode_weights: bool,
) -> Fallible<EncodeTable<F::W>>
where
    F: MutableFst,
{
    let mut encode_mapper = EncodeMapper::new(encode_labels, encode_weights);
    fst.arc_map(&mut encode_mapper)
        .with_context(|_| format_err!("Error calling ArcMap with EncodeMapper."))?;
    Ok(encode_mapper.encode_table)
}

/// The `decode` operation takes as input an encoded FST and the corresponding `EncodeTable` object
/// and reverts the encoding.
pub fn decode<F>(fst: &mut F, encode_table: EncodeTable<F::W>) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
{
    let mut decode_mapper = DecodeMapper::new(encode_table);
    fst.arc_map(&mut decode_mapper)
        .with_context(|_| format_err!("Error calling ArcMap with EncodeMapper."))?;
    rm_final_epsilon(fst)?;
    Ok(())
}

use anyhow::{Context, Result};

use crate::algorithms::{encode::EncodeTable, rm_final_epsilon};
use crate::algorithms::{FinalTr, MapFinalAction, TrMapper};
use crate::fst_traits::MutableFst;
use crate::{Semiring, Tr};

struct DecodeMapper<W: Semiring> {
    encode_table: EncodeTable<W>,
}

impl<W: Semiring> DecodeMapper<W> {
    pub fn new(encode_table: EncodeTable<W>) -> Self {
        DecodeMapper { encode_table }
    }
}

impl<W: Semiring> TrMapper<W> for DecodeMapper<W> {
    fn tr_map(&self, tr: &mut Tr<W>) -> Result<()> {
        let tuple = self
            .encode_table
            .0
            .borrow_mut()
            .decode(tr.ilabel)
            .unwrap()
            .clone();
        tr.ilabel = tuple.ilabel;
        if self.encode_table.0.borrow().encode_labels {
            tr.olabel = tuple.olabel;
        }
        if self.encode_table.0.borrow().encode_weights {
            tr.weight = tuple.weight;
        }
        Ok(())
    }

    fn final_tr_map(&self, _final_tr: &mut FinalTr<W>) -> Result<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

/// The `decode` operation takes as input an encoded FST and the corresponding `EncodeTable` object
/// and reverts the encoding.
pub fn decode<W, F>(fst: &mut F, encode_table: EncodeTable<W>) -> Result<()>
where
    W: Semiring,
    F: MutableFst<W>,
{
    let mut decode_mapper = DecodeMapper::new(encode_table);
    fst.tr_map(&mut decode_mapper)
        .with_context(|| format_err!("Error calling TrMap with EncodeMapper."))?;
    rm_final_epsilon(fst)?;
    Ok(())
}

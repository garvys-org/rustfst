use anyhow::{Context, Result};

use crate::algorithms::{encode::EncodeTable, rm_final_epsilon};
use crate::algorithms::{FinalTr, MapFinalAction, TrMapper};
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::{Semiring, Tr};

struct DecodeMapper<W: Semiring> {
    encode_table: EncodeTable<W>,
}

impl<W: Semiring> DecodeMapper<W> {
    pub fn new(encode_table: EncodeTable<W>) -> Self {
        DecodeMapper { encode_table }
    }

    pub fn encode_weights(&self) -> bool {
        self.encode_table.0.borrow().encode_type.encode_weights()
    }

    pub fn encode_labels(&self) -> bool {
        self.encode_table.0.borrow().encode_type.encode_labels()
    }
}

impl<W: Semiring> TrMapper<W> for DecodeMapper<W> {
    fn tr_map(&self, tr: &mut Tr<W>) -> Result<()> {
        let tuple = self
            .encode_table
            .0
            .borrow_mut()
            .decode(tr.ilabel as usize)
            .ok_or_else(|| format_err!("Can't decode ilabel : {:?}", tr.ilabel))?
            .clone();
        tr.ilabel = tuple.ilabel;
        if self.encode_labels() {
            tr.olabel = tuple.olabel;
        }
        if self.encode_weights() {
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

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        let outprops = inprops;
        let mut mask = FstProperties::all_properties();
        if self.encode_labels() {
            mask &= FstProperties::i_label_invariant_properties()
                & FstProperties::o_label_invariant_properties();
        }
        if self.encode_weights() {
            mask &= FstProperties::i_label_invariant_properties()
                & FstProperties::weight_invariant_properties()
                & FstProperties::rm_super_final_properties()
        }
        outprops & mask
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

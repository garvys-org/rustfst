use std::sync::Arc;

use crate::fst_traits::Fst;
use crate::symbol_table::SymbolTable;
use crate::trs::Trs;
use crate::Semiring;

pub mod const_fst_bin_deserializer;
pub mod const_fst_bin_serializer;
pub mod const_fst_text_deserialization;
pub mod const_fst_text_serialization;
pub mod vector_fst_bin_deserializer;
pub mod vector_fst_bin_serializer;
pub mod vector_fst_text_deserialization;
pub mod vector_fst_text_serialization;

fn generate_symbol_table<W: Semiring, F: Fst<W>>(
    prefix: &str,
    fst: &F,
) -> (Arc<SymbolTable>, Arc<SymbolTable>) {
    let mut input_symt = SymbolTable::new();
    let mut output_symt = SymbolTable::new();
    let mut highest_ilabel = 0;
    let mut highest_olabel = 0;
    for state in fst.fst_iter() {
        for tr_out in state.trs.trs() {
            highest_ilabel = highest_ilabel.max(tr_out.ilabel);
            highest_olabel = highest_olabel.max(tr_out.olabel);
        }
    }

    let input_symbols =
        (0..(highest_ilabel + 1)).map(|it| format!("{}_input_symbol_{}", prefix, it));
    input_symt.add_symbols(input_symbols);

    let output_symbols =
        (0..(highest_olabel + 1)).map(|it| format!("{}_input_symbol_{}", prefix, it));
    output_symt.add_symbols(output_symbols);
    (Arc::new(input_symt), Arc::new(output_symt))
}

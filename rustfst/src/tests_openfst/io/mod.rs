pub mod const_fst_bin_deserializer;
pub mod const_fst_bin_serializer;
pub mod const_fst_text_serialization;
pub mod vector_fst_bin_deserializer;
pub mod vector_fst_bin_serializer;
pub mod vector_fst_text_serialization;


use crate::symbol_table::SymbolTable;
use std::rc::Rc;
use crate::fst_traits::Fst;


fn generate_symbol_table<F: Fst>(prefix: &str, fst: &F) -> (Rc<SymbolTable>, Rc<SymbolTable>) {
    let mut input_symt = SymbolTable::new();
    let mut output_symt = SymbolTable::new();
    let mut highest_ilabel = 0;
    let mut highest_olabel = 0;
    for state in fst.fst_iter() {
        for arc_out in state.arcs {
            highest_ilabel = highest_ilabel.max(arc_out.ilabel);
            highest_olabel = highest_olabel.max(arc_out.olabel);
        }
    }

    let input_symbols = (0..(highest_ilabel + 1))
            .map(|it| format!("{}_input_symbol_{}", prefix, it));
    input_symt.add_symbols(input_symbols);

     let output_symbols = (0..(highest_olabel + 1))
            .map(|it| format!("{}_input_symbol_{}", prefix, it));
    output_symt.add_symbols(output_symbols);
    (Rc::new(input_symt), Rc::new(output_symt))
}
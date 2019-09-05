#ifndef SYMT_002
#define SYMT_002

fst::SymbolTable compute_symt_002() {
    fst::SymbolTable symt;
    symt.AddSymbol("<eps>");
    symt.AddSymbol("a");
    symt.AddSymbol("b");
    return symt;
}

#endif
#ifndef SYMT_001
#define SYMT_001

fst::SymbolTable compute_symt_001() {
    fst::SymbolTable symt;
    symt.AddSymbol("<eps>");
    symt.AddSymbol("a");
    symt.AddSymbol("b");
    return symt;
}

#endif
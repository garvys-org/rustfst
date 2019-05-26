#ifndef FST_001
#define FST_001

fst::VectorFst<fst::StdArc> compute_fst_001() {
    fst::VectorFst<fst::StdArc> f;
    auto s0 = f.AddState();
    auto s1 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s1, fst::TropicalWeight::One());

    f.AddArc(s0, fst::StdArc(12,25,fst::TropicalWeight::One(),s1));

    return f;
}

#endif
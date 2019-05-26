#ifndef FST_003
#define FST_003

fst::VectorFst<fst::StdArc> compute_fst_003() {
    fst::VectorFst<fst::StdArc> f;
    auto s0 = f.AddState();
    auto s1 = f.AddState();
    auto s2 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s2, 0.7);

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s1));
    f.AddArc(s0, fst::StdArc(14, 26, 0.2, s1));
    f.AddArc(s1, fst::StdArc(5, 3, 0.1, s2));
    f.AddArc(s2, fst::StdArc(6, 7, 0.4, s2));

    return f;
}

#endif
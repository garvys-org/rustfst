#ifndef FST_002
#define FST_002

fst::VectorFst<fst::StdArc> compute_fst_002() {
    fst::VectorFst<fst::StdArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();
    auto s2 = f.AddState();
    auto s3 = f.AddState();
    auto s4 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s3, fst::TropicalWeight(0.7));

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s1));
    f.AddArc(s1, fst::StdArc(112, 75, 0.1, s2));
    f.AddArc(s2, fst::StdArc(124, 75, 0.5, s3));
    f.AddArc(s3, fst::StdArc(152, 55, 0.6, s4));

    auto s5 = f.AddState();
    auto s6 = f.AddState();

    f.AddArc(s5, fst::StdArc(12, 25, 0.4, s4));
    f.AddArc(s5, fst::StdArc(12, 25, 0.1, s2));

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s6));
    f.AddArc(s1, fst::StdArc(12, 25, 0.2, s6));

    return f;
}

#endif
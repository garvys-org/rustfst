#ifndef FST_010
#define FST_010

fst::VectorFst<fst::LogArc> compute_fst_010() {
    fst::VectorFst<fst::LogArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();
    auto s2 = f.AddState();
    auto s3 = f.AddState();
    auto s4 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s3, 0.7);
    f.SetFinal(s4, 0.8);

    f.AddArc(s0, fst::LogArc(12, 12, 0.3, s1));
    f.AddArc(s1, fst::LogArc(13, 13, 0.4, s3));

    f.AddArc(s0, fst::LogArc(14, 14, 0.5, s2));
    f.AddArc(s2, fst::LogArc(15, 15, 0.6, s4));

    return f;
}

#endif
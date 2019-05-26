#ifndef FST_005
#define FST_005

fst::VectorFst<fst::LogArc> compute_fst_005() {
    fst::VectorFst<fst::LogArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s1, 0.7);

    f.AddArc(s0, fst::LogArc(12, 25, 0.3, s1));
    f.AddArc(s0, fst::LogArc(12, 25, 0.4, s1));
    f.AddArc(s0, fst::LogArc(12, 25, 0.1, s1));
    f.AddArc(s0, fst::LogArc(12, 26, 0.7, s1));
    f.AddArc(s0, fst::LogArc(12, 25, 0.5, s1));
    f.AddArc(s0, fst::LogArc(12, 26, 0.2, s1));

    return f;
}

#endif
#ifndef FST_006
#define FST_006

fst::VectorFst<fst::StdArc> compute_fst_006() {
    fst::VectorFst<fst::StdArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s1, 0.7);

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s1));
    f.AddArc(s0, fst::StdArc(12, 25, 0.4, s1));
    f.AddArc(s0, fst::StdArc(12, 25, 0.1, s1));
    f.AddArc(s0, fst::StdArc(12, 26, 0.7, s1));
    f.AddArc(s0, fst::StdArc(12, 25, 0.5, s1));
    f.AddArc(s0, fst::StdArc(12, 26, 0.2, s1));

    return f;
}

#endif
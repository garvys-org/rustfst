#ifndef FST_004
#define FST_004

fst::VectorFst<fst::StdArc> compute_fst_004() {
    fst::VectorFst<fst::StdArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();
    f.AddState();
    auto s3 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s1, 0.7);

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s1));
    f.AddArc(s0, fst::StdArc(10, 26, 0.4, s1));
    f.AddArc(s1, fst::StdArc(4, 5, 0.1, s3));

    return f;
}

#endif
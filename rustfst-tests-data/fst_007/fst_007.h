#ifndef FST_007
#define FST_007

fst::VectorFst<fst::StdArc> compute_fst_007() {
    fst::VectorFst<fst::StdArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();
    auto s2 = f.AddState();
    auto s3 = f.AddState();
    auto s4 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s4, 0.7);

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s1));
    f.AddArc(s1, fst::StdArc(13, 26, 0.4, s3));

    f.AddArc(s0, fst::StdArc(12, 25, 0.3, s2));
    f.AddArc(s2, fst::StdArc(13, 26, 0.4, s3));

    f.AddArc(s3, fst::StdArc(14, 27, 0.6, s4));

    return f;
}

#endif
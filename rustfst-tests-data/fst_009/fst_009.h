#ifndef FST_009
#define FST_009

fst::VectorFst<fst::StdArc> compute_fst_009() {
    fst::VectorFst<fst::StdArc> f;

    auto s0 = f.AddState();
    auto s1 = f.AddState();
    auto s2 = f.AddState();
    auto s3 = f.AddState();
    auto s4 = f.AddState();

    f.SetStart(s0);
    f.SetFinal(s4, 0.7);

    f.AddArc(s0, fst::StdArc(12, 12, 0.3, s1));
    f.AddArc(s1, fst::StdArc(13, 13, 0.4, s3));

    f.AddArc(s0, fst::StdArc(12, 12, 0.3, s2));
    f.AddArc(s2, fst::StdArc(13, 13, 0.4, s3));
    f.AddArc(s2, fst::StdArc(15, 15, 0.1, s4));
    f.AddArc(s2, fst::StdArc(16, 16, 0.1, s2));
    f.AddArc(s2, fst::StdArc(17, 17, 0.15, s3));

    f.AddArc(s3, fst::StdArc(14, 14, 0.6, s4));

    return f;
}

#endif
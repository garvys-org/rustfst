#ifndef FST_009
#define FST_009

class FstTestData009 {
public:
    using MyArc = fst::StdArc;
    using MyWeight = MyArc::Weight;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData009() {}

    MyFst get_fst() const {
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

    fst::VectorFst<MyArc> get_fst_compose() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(1, MyWeight(1.2));
        fst_2.AddArc(0, MyArc(12, 2, MyWeight(1.2), 1));
        fst_2.AddArc(0, MyArc(13, 3, MyWeight(1.3), 1));
        fst_2.AddArc(0, MyArc(14, 4, MyWeight(1.4), 1));
        fst_2.AddArc(0, MyArc(15, 5, MyWeight(1.5), 1));
        fst_2.AddArc(0, MyArc(16, 6, MyWeight(1.6), 1));
        fst_2.AddArc(0, MyArc(17, 7, MyWeight(1.7), 1));
        fst_2.AddArc(1, MyArc(0, 18, MyWeight(1.8), 0));
        return fst_2;
    }

    MyWeight get_weight_plus_mapper() const {
        return MyWeight(1.5);
    }

    MyWeight get_weight_times_mapper() const {
        return MyWeight(1.5);
    }

    fst::VectorFst<MyArc> get_fst_concat() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.AddState();
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(2, MyWeight(0.3));
        fst_2.AddArc(0, MyArc(2, 12, MyWeight(1.2), 1));
        fst_2.AddArc(0, MyArc(3, 1, MyWeight(2.2), 1));
        fst_2.AddArc(1, MyArc(6, 3, MyWeight(2.3), 2));
        fst_2.AddArc(1, MyArc(4, 2, MyWeight(1.7), 2));
        return fst_2;
    }

    fst::VectorFst<MyArc> get_fst_union() const {
        return get_fst_concat();
    }

    MyWeight random_weight() const {
        return MyWeight(custom_random_float());
    }
};

#endif
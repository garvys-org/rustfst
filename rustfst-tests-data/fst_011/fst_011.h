#ifndef FST_011
#define FST_011

class FstTestData011 {
public:
    using MyWeight = fst::ProductWeight<fst::TropicalWeight, fst::LogWeight>;
    using MyArc = fst::ArcTpl<MyWeight>;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData011() {}

    MyFst get_fst() const {
        MyFst f;

        auto s0 = f.AddState();
        auto s1 = f.AddState();
        auto s2 = f.AddState();
        auto s3 = f.AddState();
        auto s4 = f.AddState();

        f.SetStart(s0);
        f.SetFinal(s3, MyWeight(0.7, 0.7));
        f.SetFinal(s4, MyWeight(0.8, 0.8));

        f.AddArc(s0, MyArc(12, 12, MyWeight(0.2, 0.3), s1));
        f.AddArc(s1, MyArc(13, 13, MyWeight(0.2, 0.3), s3));

        f.AddArc(s0, MyArc(14, 14, MyWeight(0.2, 0.3), s2));
        f.AddArc(s2, MyArc(15, 15, MyWeight(0.2, 0.3), s4));

        return f;
    }

    fst::VectorFst<MyArc> get_fst_compose() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(0, MyWeight(1.2, 0.1));
        fst_2.AddArc(0, MyArc(12, 2, MyWeight(1.7, 0.3), 0));
        fst_2.AddArc(0, MyArc(13, 3, MyWeight(1.7, 1.8), 0));
        fst_2.AddArc(0, MyArc(14, 4, MyWeight(1.7, 0.2), 0));
        fst_2.AddArc(0, MyArc(15, 5, MyWeight(1.7, 1.8), 0));
        return fst_2;
    }

    MyWeight get_weight_plus_mapper() const {
        return MyWeight(1.5, 2.3);
    }

    MyWeight get_weight_times_mapper() const {
        return MyWeight(1.5, 2.3);
    }

    fst::VectorFst<MyArc> get_fst_concat() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.AddState();
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(2, MyWeight(0.3, 1.3));
        fst_2.AddArc(0, MyArc(2, 12, MyWeight(1.2, 1.6), 1));
        fst_2.AddArc(0, MyArc(3, 1, MyWeight(2.2, 1.3), 1));
        fst_2.AddArc(1, MyArc(6, 3, MyWeight(2.3, 2.4), 2));
        fst_2.AddArc(1, MyArc(4, 2, MyWeight(1.7, 0.2), 2));
        return fst_2;
    }

    fst::VectorFst<MyArc> get_fst_union() const {
        return get_fst_concat();
    }

    MyWeight random_weight() const {
        return MyWeight(custom_random_float(), custom_random_float());
    }
};


#endif
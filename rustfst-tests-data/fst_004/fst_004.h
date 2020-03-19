#ifndef FST_004
#define FST_004

class FstTestData004 {
public:
    using MyArc = fst::StdArc;
    using MyWeight = MyArc::Weight;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData004() {}

    MyFst get_fst() const {
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

    fst::VectorFst<MyArc> get_fst_compose() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(0, MyWeight(1.2));
        fst_2.AddArc(0, MyArc(25, 2, MyWeight(1.7), 0));
        fst_2.AddArc(0, MyArc(26, 4, MyWeight(2.7), 0));
        fst_2.AddArc(0, MyArc(5, 3, MyWeight(3.7), 0));
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
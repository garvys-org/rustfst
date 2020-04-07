#ifndef FST_006
#define FST_006

class FstTestData006 {
public:
    using MyArc = fst::StdArc;
    using MyWeight = MyArc::Weight;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData006() {}

    MyFst get_fst() const {
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

    fst::VectorFst<MyArc> get_fst_compose() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(1, MyWeight(1.2));
        fst_2.AddArc(0, MyArc(26, 2, MyWeight(1.7), 1));
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
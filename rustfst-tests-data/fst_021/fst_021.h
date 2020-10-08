#ifndef FST_021
#define FST_021

class FstTestData021 {
public:
    using MyWeight = fst::TropicalWeight;
    using MyArc = fst::ArcTpl<MyWeight>;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData021() {}

    MyFst get_fst() const {
        fst::VectorFst<fst::StdArc> f;

        for (int i = 0; i < 15; i++) {
            f.AddState();
        }

        f.SetStart(4);

        f.AddArc(0, fst::StdArc(0, 0, 0.8, 1));

        f.AddArc(1, fst::StdArc(0, 0, 0.8, 4));
        f.SetFinal(1, 0.9);

        f.AddArc(4, fst::StdArc(0, 0, 0.3, 9));
        f.SetFinal(4, 0.2);

        f.AddArc(5, fst::StdArc(0, 0, 0.8, 0));
        f.SetFinal(5, 0.8);

        f.AddArc(9, fst::StdArc(0, 0, 0.8, 5));

        return f;
    }

    fst::VectorFst<MyArc> get_fst_compose() const {
        return fst::VectorFst<MyArc>();
    }

    MyWeight get_weight_plus_mapper() const {
        return MyWeight(1.5);
    }

    MyWeight get_weight_times_mapper() const {
        return MyWeight(1.5);
    }

    fst::VectorFst<MyArc> get_fst_concat() const {
        return get_fst_compose();
    }

    fst::VectorFst<MyArc> get_fst_union() const {
        return get_fst_concat();
    }

    MyWeight random_weight() const {
        return MyWeight(custom_random_float());
    }
};


#endif
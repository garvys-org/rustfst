#ifndef FST_012
#define FST_012

class FstTestData012 {
public:
    using MyWeight = fst::StringWeight<int, fst::STRING_RESTRICT>;
    using MyArc = fst::ArcTpl<MyWeight>;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData012() {}

    MyFst get_fst() const {
        MyFst f;

        auto s0 = f.AddState();
        auto s1 = f.AddState();
        auto s2 = f.AddState();
        auto s3 = f.AddState();
        auto s4 = f.AddState();

        f.SetStart(s0);
        f.SetFinal(s3, MyWeight(1));
        f.SetFinal(s4, MyWeight(2));

        f.AddArc(s0, MyArc(12, 12, MyWeight(3), s1));
        f.AddArc(s1, MyArc(13, 13, MyWeight(4), s3));

        f.AddArc(s0, MyArc(14, 14, MyWeight(5), s2));
        f.AddArc(s2, MyArc(15, 15, MyWeight(3), s4));

        return f;
    }

    MyWeight get_weight_plus_mapper() const {
        return MyWeight(3);
    }

    MyWeight get_weight_times_mapper() const {
        return MyWeight(3);
    }

    fst::VectorFst<MyArc> get_fst_concat() const {
        fst::VectorFst<MyArc> fst_2;
        fst_2.AddState();
        fst_2.AddState();
        fst_2.AddState();
        fst_2.SetStart(0);
        fst_2.SetFinal(2, MyWeight(3));
        fst_2.AddArc(0, MyArc(2, 12, MyWeight(3), 1));
        fst_2.AddArc(0, MyArc(3, 1, MyWeight(3), 1));
        fst_2.AddArc(1, MyArc(6, 3, MyWeight(3), 2));
        fst_2.AddArc(1, MyArc(4, 2, MyWeight(3), 2));
        return fst_2;
    }

    fst::VectorFst<MyArc> get_fst_union() const {
        return get_fst_concat();
    }

    MyWeight random_weight() const {
        return MyWeight(1);
    }
};


#endif
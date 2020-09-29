#ifndef FST_016
#define FST_016

class FstTestData016 {
public:
    using MyWeight = fst::TropicalWeight;
    using MyArc = fst::ArcTpl<MyWeight>;
    using MyFst = fst::VectorFst<MyArc>;

    FstTestData016() {}

    MyFst get_fst() const {
        auto parsed_fst = fst::VectorFst<MyArc>::Read(std::string("fst_016/simple.fst.in"));
        return *parsed_fst;
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
macro_rules! arc_mapper_to_weight_convert_mapper_methods {
    ($semiring: ty) => {
        fn arc_map(&mut self, arc: &Arc<$semiring>) -> Arc<$semiring> {
            let mut mapped_arc = arc.clone();
            (self as &mut ArcMapper<$semiring>).arc_map(&mut mapped_arc);
            mapped_arc
        }

        fn final_arc_map(&mut self, final_arc: &FinalArc<$semiring>) -> FinalArc<$semiring> {
            let mut mapped_final_arc = final_arc.clone();
            (self as &mut ArcMapper<$semiring>).final_arc_map(&mut mapped_final_arc);
            mapped_final_arc
        }

        fn final_action(&self) -> MapFinalAction {
            (self as &ArcMapper<$semiring>).final_action()
        }
    }
}

macro_rules! arc_mapper_to_weight_convert_mapper {
    ($mapper: ty) => {
        impl<S> WeightConverter<S, S> for $mapper
        where
            S: Semiring,
        {
            arc_mapper_to_weight_convert_mapper_methods!(S);
        }
    };
}

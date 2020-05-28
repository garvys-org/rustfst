macro_rules! tr_mapper_to_weight_convert_mapper_methods {
    ($semiring: ty) => {
        fn tr_map(&mut self, tr: &Tr<$semiring>) -> Result<Tr<$semiring>> {
            let mut mapped_tr = tr.clone();
            (self as &mut dyn TrMapper<$semiring>).tr_map(&mut mapped_tr)?;
            Ok(mapped_tr)
        }

        fn final_tr_map(&mut self, final_tr: &FinalTr<$semiring>) -> Result<FinalTr<$semiring>> {
            let mut mapped_final_tr = final_tr.clone();
            (self as &mut dyn TrMapper<$semiring>).final_tr_map(&mut mapped_final_tr)?;
            Ok(mapped_final_tr)
        }

        fn final_action(&self) -> MapFinalAction {
            (self as &dyn TrMapper<$semiring>).final_action()
        }

        fn properties(&self, iprops: FstProperties) -> FstProperties {
            (self as &dyn TrMapper<$semiring>).properties(iprops)
        }
    };
}

macro_rules! tr_mapper_to_weight_convert_mapper {
    ($mapper: ty) => {
        impl<S> WeightConverter<S, S> for $mapper
        where
            S: Semiring,
        {
            tr_mapper_to_weight_convert_mapper_methods!(S);
        }
    };
}

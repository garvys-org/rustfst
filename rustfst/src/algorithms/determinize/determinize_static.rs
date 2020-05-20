use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::determinize::determinize_fsa::DeterminizeFsa;
use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::{DefaultCommonDivisor, DeterminizeType, GallicCommonDivisor};
use crate::algorithms::factor_weight::factor_iterators::{
    GallicFactor, GallicFactorMin, GallicFactorRestrict,
};
use crate::algorithms::factor_weight::{factor_weight, FactorWeightOptions, FactorWeightType};
use crate::algorithms::weight_convert;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::SemiringProperties;
use crate::semirings::{
    GallicWeight, GallicWeightMin, GallicWeightRestrict, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::{EPS_LABEL, KDELTA};

pub fn determinize_with_distance<W, F1, F2>(
    ifst: Arc<F1>,
    in_dist: Arc<Vec<W>>,
) -> Result<(F2, Vec<W>)>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W>,
    F2: MutableFst<W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let fst = DeterminizeFsa::<_, _, DefaultCommonDivisor>::new(ifst, Some(in_dist))?;
    fst.compute_with_distance()
}

pub fn determinize_fsa<W, F1, F2, CD>(fst_in: Arc<F1>) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: Fst<W>,
    F2: MutableFst<W>,
    CD: CommonDivisor<W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let det_fsa: DeterminizeFsa<_, _, CD> = DeterminizeFsa::new(fst_in, None)?;
    det_fsa.compute()
}

pub fn determinize_fst<W, F1, F2>(fst_in: Arc<F1>, det_type: DeterminizeType) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut to_gallic = ToGallicConverter {};
    let mut from_gallic = FromGallicConverter {
        superfinal_label: EPS_LABEL,
    };

    let factor_opts = FactorWeightOptions {
        delta: KDELTA,
        mode: FactorWeightType::FACTOR_FINAL_WEIGHTS,
        final_ilabel: EPS_LABEL,
        final_olabel: EPS_LABEL,
        increment_final_ilabel: false,
        increment_final_olabel: false,
    };

    match det_type {
        DeterminizeType::DeterminizeDisambiguate => {
            if !W::properties().contains(SemiringProperties::PATH) {
                bail!("determinize : weight needs to have the path property to disambiguate output")
            }
            let fsa: VectorFst<GallicWeightMin<W>> = weight_convert(&fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightMin<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(Arc::new(fsa))?;
            let factored_determinized_fsa: VectorFst<GallicWeightMin<W>> =
                factor_weight::<_, VectorFst<GallicWeightMin<W>>, _, _, GallicFactorMin<W>>(
                    &determinized_fsa,
                    factor_opts,
                )?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeFunctional => {
            let fsa: VectorFst<GallicWeightRestrict<W>> = weight_convert(&fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(Arc::new(fsa))?;
            let factored_determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                factor_weight::<
                    _,
                    VectorFst<GallicWeightRestrict<W>>,
                    _,
                    _,
                    GallicFactorRestrict<W>,
                >(&determinized_fsa, factor_opts)?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeNonFunctional => {
            let fsa: VectorFst<GallicWeight<W>> = weight_convert(&fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeight<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(Arc::new(fsa))?;
            let factored_determinized_fsa: VectorFst<GallicWeight<W>> =
                factor_weight::<_, VectorFst<GallicWeight<W>>, _, _, GallicFactor<W>>(
                    &determinized_fsa,
                    factor_opts,
                )?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
    }
}

/// This operations creates an equivalent FST that has the property that no
/// state has two transitions with the same input label. For this algorithm,
/// epsilon transitions are treated as regular symbols.
///
/// # Example
///
/// ## Input
///
/// ![determinize_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/determinize_in.svg?sanitize=true)
///
/// ## Determinize
///
/// ![determinize_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/determinize_out.svg?sanitize=true)
///
pub fn determinize<W, F1, F2>(fst_in: Arc<F1>, det_type: DeterminizeType) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut fst_res: F2 = if fst_in.is_acceptor() {
        determinize_fsa::<_, _, _, DefaultCommonDivisor>(Arc::clone(&fst_in))?
    } else {
        determinize_fst(Arc::clone(&fst_in), det_type)?
    };

    fst_res.set_symts_from_fst(&fst_in);
    Ok(fst_res)
}

#[cfg(test)]
mod tests {
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;
    use crate::tr::Tr;
    use crate::Semiring;

    use super::*;

    #[test]
    fn test_determinize() -> Result<()> {
        let mut input_fst = VectorFst::<TropicalWeight>::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s1, TropicalWeight::one())?;

        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s1, TropicalWeight::one())?;

        ref_fst.add_tr(s0, Tr::new(1, 1, TropicalWeight::new(2.0), s1))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(Arc::new(input_fst), DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }

    #[test]
    fn test_determinize_2() -> Result<()> {
        let mut input_fst = VectorFst::<TropicalWeight>::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();
        let s2 = input_fst.add_state();
        let s3 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s3, TropicalWeight::one())?;

        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 3.0, s2))?;

        input_fst.add_tr(s1, Tr::new(2, 2, 4.0, s3))?;
        input_fst.add_tr(s2, Tr::new(2, 2, 3.0, s3))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();
        let s2 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s2, TropicalWeight::one())?;

        ref_fst.add_tr(s0, Tr::new(1, 1, TropicalWeight::new(2.0), s1))?;
        ref_fst.add_tr(s1, Tr::new(2, 2, TropicalWeight::new(4.0), s2))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(Arc::new(input_fst), DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }
}

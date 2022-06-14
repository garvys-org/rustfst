use anyhow::Result;

use crate::algorithms::tr_filters::AnyTrFilter;
use crate::fst_impls::VectorFst;
pub use randgen_config::RandGenConfig;
pub use randgen_fst::RandGenFst;
use tr_sampler::TrSampler;
pub use tr_selector::{TrSelector, UniformTrSelector};

use crate::fst_traits::Fst;
use crate::prelude::dfs_visit::dfs_visit;
use crate::prelude::randgen::randgen_visitor::RandGenVisitor;
use crate::prelude::MutableFst;
use crate::Semiring;

mod rand_state;
mod randgen_config;
mod randgen_fst;
mod randgen_fst_op;
mod randgen_visitor;
mod tr_sampler;
mod tr_selector;

/// Randomly generate paths through an Fst; execution controlled by
/// RandGenConfig.
pub fn randgen_with_config<
    W: Semiring<Type = f32>,
    FI: Fst<W>,
    FO: MutableFst<W>,
    S: TrSelector,
>(
    ifst: &FI,
    config: RandGenConfig<S>,
) -> Result<FO> {
    let sampler = TrSampler::<_, FI, _, _>::new(ifst, config.selector, config.max_length);
    let randgen_fst = RandGenFst::new(
        ifst,
        sampler,
        config.npath,
        config.weighted,
        config.remove_total_weight,
    );
    if config.weighted {
        randgen_fst.compute()
    } else {
        // TODO: No need to do that if dfs_visit supports NOT expanded FST.
        let randgen_fst_static: VectorFst<_> = randgen_fst.compute()?;
        let mut visitor = RandGenVisitor::new();
        dfs_visit(&randgen_fst_static, &mut visitor, &AnyTrFilter {}, false);
        Ok(visitor.into_output_fst())
    }
}

/// Randomly generate a path through an Fst with the uniform distribution
/// over the transitions.
pub fn randgen<W: Semiring<Type = f32>, FI: Fst<W>, FO: MutableFst<W>>(ifst: &FI) -> Result<FO> {
    let selector = UniformTrSelector::new();
    let config = RandGenConfig::new(selector);
    randgen_with_config(ifst, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::union::union;
    use crate::semirings::TropicalWeight;
    use crate::utils::acceptor;

    #[test]
    fn test_randgen_weighted() -> Result<()> {
        let mut fst: VectorFst<TropicalWeight> = acceptor(&[1, 2, 3], TropicalWeight::one());
        union(
            &mut fst,
            &acceptor::<_, VectorFst<_>>(&[4, 5], TropicalWeight::one()),
        )?;

        let config = RandGenConfig::new(UniformTrSelector::from_seed(2022))
            .with_npath(10)
            .with_weighted(true);
        let res: VectorFst<_> = randgen_with_config(&fst, config)?;

        let paths = res.paths_iter().collect::<Vec<_>>();
        assert_eq!(paths.len(), 2);

        for path in paths {
            assert!(path.ilabels == vec![1, 2, 3] || path.ilabels == vec![4, 5]);
            assert!(path.olabels == vec![1, 2, 3] || path.olabels == vec![4, 5]);
        }

        Ok(())
    }

    #[test]
    fn test_randgen_unweighted() -> Result<()> {
        let mut fst: VectorFst<TropicalWeight> = acceptor(&[1, 2, 3], TropicalWeight::one());
        union(
            &mut fst,
            &acceptor::<_, VectorFst<_>>(&[4, 5], TropicalWeight::one()),
        )?;

        let config = RandGenConfig::new(UniformTrSelector::from_seed(2022))
            .with_npath(10)
            .with_weighted(false);
        let res: VectorFst<_> = randgen_with_config(&fst, config)?;

        let paths = res.paths_iter().collect::<Vec<_>>();
        assert_eq!(paths.len(), 10);

        for path in paths {
            assert!(path.ilabels == vec![1, 2, 3] || path.ilabels == vec![4, 5]);
            assert!(path.olabels == vec![1, 2, 3] || path.olabels == vec![4, 5]);
        }

        Ok(())
    }
}

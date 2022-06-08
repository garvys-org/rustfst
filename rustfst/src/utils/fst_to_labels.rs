use anyhow::{bail, Result};

use crate::fst_path::FstPath;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;

/// Decode a linear FST to retrieves the only path recognized by it. A path is composed of the
/// input symbols, the output symbols and the weight (multiplication of the weights of the trs
/// of the path).
///
/// # Example
///
/// ```
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{BooleanWeight, Semiring};
/// # use rustfst::utils::{transducer, decode_linear_fst};
/// # use rustfst::Tr;
/// # use rustfst::FstPath;
/// let labels_input = vec![32, 43, 21];
/// let labels_output = vec![53, 18, 89];
///
/// let fst : VectorFst<BooleanWeight> = transducer(&labels_input, &labels_output, BooleanWeight::one());
///
/// let path = decode_linear_fst(&fst).unwrap();
///
/// assert_eq!(path, FstPath::new(labels_input, labels_output, BooleanWeight::one()));
/// ```
pub fn decode_linear_fst<W: Semiring, F: Fst<W>>(fst: &F) -> Result<FstPath<W>> {
    let mut it_path = fst.paths_iter();
    let path = it_path.next().unwrap_or_default();
    if it_path.next().is_some() {
        bail!("The FST is not linear !")
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::{BooleanWeight, Semiring};
    use crate::tr::Tr;
    use crate::utils::{acceptor, transducer};

    #[test]
    fn test_decode_linear_fst_acceptor() -> Result<()> {
        let labels = vec![1, 2, 3];
        let fst: VectorFst<BooleanWeight> = acceptor(&labels, BooleanWeight::one());

        let path = decode_linear_fst(&fst)?;
        let path_ref = FstPath::new(labels.clone(), labels, BooleanWeight::one());
        assert_eq!(path, path_ref);
        Ok(())
    }

    #[test]
    fn test_decode_linear_fst_transducer() -> Result<()> {
        let labels_input = vec![1, 2, 3];
        let labels_output = vec![43, 22, 18];
        let fst: VectorFst<BooleanWeight> =
            transducer(&labels_input, &labels_output, BooleanWeight::one());

        let path = decode_linear_fst(&fst)?;
        let path_ref = FstPath::new(labels_input, labels_output, BooleanWeight::one());

        assert_eq!(path, path_ref);
        Ok(())
    }

    #[test]
    fn test_decode_linear_fst_empty_fst() -> Result<()> {
        let fst = VectorFst::<BooleanWeight>::new();
        let path = decode_linear_fst(&fst)?;

        assert_eq!(path, FstPath::default());

        Ok(())
    }

    #[test]
    fn test_decode_linear_fst_state_start_and_final() -> Result<()> {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s = fst.add_state();
        fst.set_start(s)?;
        fst.set_final(s, BooleanWeight::one())?;

        let path = decode_linear_fst(&fst)?;

        assert_eq!(path, FstPath::default());

        Ok(())
    }

    #[test]
    fn test_decode_linear_fst_fst_not_linear() -> Result<()> {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(s1)?;
        fst.set_final(s2, BooleanWeight::one())?;
        fst.add_tr(s1, Tr::new(10, 10, BooleanWeight::one(), s2))?;
        fst.add_tr(s1, Tr::new(10, 10, BooleanWeight::one(), s2))?;

        assert!(decode_linear_fst(&fst).is_err());
        Ok(())
    }
}

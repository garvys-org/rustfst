use fst_traits::Fst;
use path::Path;
use Result;

/// Decode a linear FST to retrieves the only path recognized by it. A path is composed of the
/// input symbols, the output symbols and the weight (multiplication of the weights of the arcs
/// of the path).
///
/// # Example
///
/// ```
/// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::semirings::{BooleanWeight, Semiring};
/// use rustfst::utils::{transducer, decode_linear_fst};
/// use rustfst::arc::Arc;
/// use rustfst::path::Path;
///
/// let labels_input = vec![32, 43, 21];
/// let labels_output = vec![53, 18, 89];
///
/// let fst : VectorFst<BooleanWeight> = transducer(labels_input.clone().into_iter(), labels_output.clone().into_iter()).unwrap();
///
/// let path = decode_linear_fst(&fst).unwrap();
///
/// assert_eq!(path, Path::new(labels_input, labels_output, BooleanWeight::one()));
/// ```
pub fn decode_linear_fst<F: Fst>(fst: &F) -> Result<Path<F::W>> {
    let mut path = Path::default();

    let mut state_cour = match fst.start() {
        None => return Ok(path),
        Some(x) => x,
    };

    loop {
        if fst.is_final(&state_cour) {
            return Ok(path);
        }

        let mut arcs_it = fst.arcs_iter(&state_cour)?;

        let arc = arcs_it.next();

        // FST is not linear => Error
        if arcs_it.next().is_some() {
            bail!("The state {:?} has more than one outgoing arcs. The FST must be linear")
        }

        match arc {
            None => return Ok(Path::default()),
            Some(ref x) => {
                path.add_to_path(x.ilabel, x.olabel, x.weight.clone());

                if fst.is_final(&state_cour) {
                    return Ok(path);
                }

                state_cour = x.nextstate;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc::Arc;
    use fst_impls::VectorFst;
    use fst_traits::MutableFst;
    use semirings::{BooleanWeight, Semiring};
    use utils::{acceptor, transducer};

    #[test]
    fn test_decode_linear_fst_acceptor() {
        let labels = vec![1, 2, 3];
        let fst: VectorFst<BooleanWeight> = acceptor(labels.clone().into_iter()).unwrap();

        let path = decode_linear_fst(&fst).unwrap();
        let path_ref = Path::new(labels.clone(), labels, BooleanWeight::one());
        assert_eq!(path, path_ref);
    }

    #[test]
    fn test_decode_linear_fst_transducer() {
        let labels_input = vec![1, 2, 3];
        let labels_output = vec![43, 22, 18];
        let fst: VectorFst<BooleanWeight> = transducer(
            labels_input.clone().into_iter(),
            labels_output.clone().into_iter(),
        ).unwrap();

        let path = decode_linear_fst(&fst).unwrap();
        let path_ref = Path::new(labels_input, labels_output, BooleanWeight::one());

        assert_eq!(path, path_ref);
    }

    #[test]
    fn test_decode_linear_fst_empty_fst() {
        let fst = VectorFst::<BooleanWeight>::new();
        let path = decode_linear_fst(&fst).unwrap();

        assert_eq!(path, Path::default());
    }

    #[test]
    fn test_decode_linear_fst_state_start_and_final() {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s = fst.add_state();
        fst.set_start(&s).unwrap();
        fst.set_final(&s, BooleanWeight::one()).unwrap();

        let path = decode_linear_fst(&fst).unwrap();

        assert_eq!(path, Path::default());
    }

    #[test]
    fn test_decode_linear_fst_fst_not_linear() {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1).unwrap();
        fst.set_final(&s2, BooleanWeight::one()).unwrap();
        fst.add_arc(&s1, Arc::new(10, 10, BooleanWeight::one(), s2))
            .unwrap();
        fst.add_arc(&s1, Arc::new(10, 10, BooleanWeight::one(), s2))
            .unwrap();

        assert!(decode_linear_fst(&fst).is_err())
    }

}

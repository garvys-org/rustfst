use fst_traits::Fst;
use Result;
use Label;


pub fn decode_linear_fst<F: Fst>(fst: &F) -> Result<(Vec<Label>, Vec<Label>)> {
    let mut res_input = vec![];
    let mut res_output = vec![];

    let mut state_cour = match fst.start() {
        None => return Ok((vec![], vec![])),
        Some(x) => x
    };

    loop {
        if fst.is_final(&state_cour) {
            return Ok((res_input, res_output));
        }

        let mut arcs_it = fst.arcs_iter(&state_cour)?;

        let arc = arcs_it.next();

        // FST is not linear => Error
        if arcs_it.next().is_some() {
            bail!("The state {:?} has more than one outgoing arcs. The FST must be linear")
        }

        match arc {
            None => {
                return Ok((vec![], vec![]))
            },
            Some(x) => {
                res_input.push(x.ilabel);
                res_output.push(x.olabel);

                if fst.is_final(&state_cour) {
                    return Ok((res_input, res_output))
                }

                state_cour = x.nextstate;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::{acceptor, transducer};
    use fst_impls::VectorFst;
    use semirings::{BooleanWeight, Semiring};
    use fst_traits::MutableFst;
    use arc::Arc;

    #[test]
    fn test_decode_linear_fst_acceptor() {
        let labels = vec![1, 2, 3];
        let fst : VectorFst<BooleanWeight> = acceptor(labels.clone().into_iter()).unwrap();

        let (res_input, res_output) = decode_linear_fst(&fst).unwrap();
        assert_eq!(labels, res_input);
        assert_eq!(labels, res_output);
    }

    #[test]
    fn test_decode_linear_fst_transducer() {
        let labels_input = vec![1, 2, 3];
        let labels_output = vec![43, 22 ,18];
        let fst : VectorFst<BooleanWeight> = transducer(labels_input.clone().into_iter(), labels_output.clone().into_iter()).unwrap();

        let (res_input, res_output) = decode_linear_fst(&fst).unwrap();
        assert_eq!(labels_input, res_input);
        assert_eq!(labels_output, res_output);
    }

    #[test]
    fn test_decode_linear_fst_empty_fst() {
        let fst = VectorFst::<BooleanWeight>::new();
        let (res_input, res_output) = decode_linear_fst(&fst).unwrap();

        assert_eq!(res_input, vec![]);
        assert_eq!(res_output, vec![]);
    }

    #[test]
    fn test_decode_linear_fst_state_start_and_final() {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s = fst.add_state();
        fst.set_start(&s).unwrap();
        fst.set_final(&s, BooleanWeight::one()).unwrap();

        let (res_input, res_output) = decode_linear_fst(&fst).unwrap();

        assert_eq!(res_input, vec![]);
        assert_eq!(res_output, vec![]);
    }

    #[test]
    fn test_decode_linear_fst_fst_not_linear() {
        let mut fst = VectorFst::<BooleanWeight>::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1).unwrap();
        fst.set_final(&s2, BooleanWeight::one()).unwrap();
        fst.add_arc(&s1, Arc::new(10, 10, BooleanWeight::one(), s2)).unwrap();
        fst.add_arc(&s1, Arc::new(10, 10, BooleanWeight::one(), s2)).unwrap();

        assert!(decode_linear_fst(&fst).is_err())
    }

}



use failure::Fallible;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{MutableFst, TextParser};
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL};

impl<W: 'static + Semiring<Type = f32>> TextParser for VectorFst<W> {
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst) -> Fallible<Self> {
        let start_state = parsed_fst_text.start();
        let num_states = parsed_fst_text.num_states();

        let states = vec![VectorFstState::<W>::default(); num_states];

        let mut fst = VectorFst {
            states,
            start_state,
        };

        for transition in parsed_fst_text.transitions.into_iter() {
            let weight = transition.weight.map(W::new).unwrap_or_else(W::one);
            let arc = Arc::new(
                transition.ilabel,
                transition.olabel,
                weight,
                transition.nextstate,
            );
            fst.add_arc(transition.state, arc)?;
        }

        for final_state in parsed_fst_text.final_states.into_iter() {
            let weight = final_state.weight.map(W::new).unwrap_or_else(W::one);
            fst.set_final(final_state.state, weight)?;
        }

        Ok(fst)
    }
}

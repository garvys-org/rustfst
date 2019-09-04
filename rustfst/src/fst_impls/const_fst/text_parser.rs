use failure::Fallible;
use itertools::Itertools;

use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::ConstFst;
use crate::fst_traits::TextParser;
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL};

impl<W: 'static + Semiring<Type = f32>> TextParser for ConstFst<W> {
    fn from_parsed_fst_text(mut parsed_fst_text: ParsedTextFst) -> Fallible<Self> {
        let start_state = parsed_fst_text.start();
        let num_states = parsed_fst_text.num_states();
        let num_arcs = parsed_fst_text.transitions.len();

        let mut const_states = Vec::with_capacity(num_states);
        let mut const_arcs = Vec::with_capacity(num_arcs);

        parsed_fst_text.transitions.sort_by_key(|v| v.state);
        for (_state, arcs_iterator) in parsed_fst_text
            .transitions
            .into_iter()
            .group_by(|v| v.state)
            .into_iter()
        {
            let pos = const_arcs.len();
            // Some states might not have outgoing arcs.
            const_states.resize_with(_state, || ConstState {
                final_weight: None,
                pos,
                narcs: 0,
                niepsilons: 0,
                noepsilons: 0,
            });
            let mut niepsilons = 0;
            let mut noepsilons = 0;
            const_arcs.extend(arcs_iterator.map(|v| {
                debug_assert_eq!(_state, v.state);
                let arc = Arc {
                    ilabel: v.ilabel,
                    olabel: v.olabel,
                    weight: v.weight.map(W::new).unwrap_or_else(W::one),
                    nextstate: v.nextstate,
                };
                if arc.ilabel == EPS_LABEL {
                    niepsilons += 1;
                }
                if arc.olabel == EPS_LABEL {
                    noepsilons += 1;
                }
                arc
            }));
            let num_arcs_this_state = const_arcs.len() - pos;
            const_states.push(ConstState::<W> {
                final_weight: None,
                pos,
                narcs: num_arcs_this_state,
                niepsilons,
                noepsilons,
            })
        }
        const_states.resize_with(num_states, || ConstState {
            final_weight: None,
            pos: const_arcs.len(),
            narcs: 0,
            niepsilons: 0,
            noepsilons: 0,
        });
        debug_assert_eq!(num_states, const_states.len());
        for final_state in parsed_fst_text.final_states.into_iter() {
            let weight = final_state.weight.map(W::new).unwrap_or_else(W::one);
            unsafe {
                const_states
                    .get_unchecked_mut(final_state.state)
                    .final_weight = Some(weight)
            };
        }

        Ok(ConstFst {
            states: const_states,
            arcs: const_arcs,
            start: start_state,
        })
    }
}

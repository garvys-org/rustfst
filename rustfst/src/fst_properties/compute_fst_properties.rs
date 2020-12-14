use std::collections::HashSet;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_properties::{known_properties, FstProperties};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs};

/// Computes all the FstProperties of the FST bit don't attach them to the FST.
pub fn compute_fst_properties<W: Semiring, F: ExpandedFst<W>>(
    fst: &F,
    mask: FstProperties,
    known: &mut FstProperties,
    use_stored: bool,
) -> Result<FstProperties> {
    let fst_props = fst.properties();

    if use_stored {
        let known_props = known_properties(fst_props);
        if (known_props & mask) == mask {
            *known = known_props;
            return Ok(fst_props);
        }
    }

    let mut comp_props = fst_props & FstProperties::binary_properties();

    let dfs_props = FstProperties::ACYCLIC
        | FstProperties::CYCLIC
        | FstProperties::INITIAL_ACYCLIC
        | FstProperties::INITIAL_CYCLIC
        | FstProperties::ACCESSIBLE
        | FstProperties::NOT_ACCESSIBLE
        | FstProperties::COACCESSIBLE
        | FstProperties::NOT_COACCESSIBLE;

    let sccs = if mask
        .intersects(dfs_props | FstProperties::WEIGHTED_CYCLES | FstProperties::UNWEIGHTED_CYCLES)
    {
        let mut visitor = SccVisitor::new(fst, true, true);
        dfs_visit(fst, &mut visitor, &AnyTrFilter {}, false);

        // Retrieves props computed in the DFS.
        comp_props |= dfs_props & visitor.props;

        unsafe { visitor.scc.unsafe_unwrap() }
    } else {
        vec![]
    };

    if mask.intersects(!(FstProperties::binary_properties() | dfs_props)) {
        comp_props |= FstProperties::ACCEPTOR
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::UNWEIGHTED
            | FstProperties::TOP_SORTED
            | FstProperties::STRING;

        if mask.intersects(FstProperties::I_DETERMINISTIC | FstProperties::NOT_I_DETERMINISTIC) {
            comp_props |= FstProperties::I_DETERMINISTIC;
        }

        if mask.intersects(FstProperties::O_DETERMINISTIC | FstProperties::NOT_O_DETERMINISTIC) {
            comp_props |= FstProperties::O_DETERMINISTIC;
        }

        if mask.intersects(
            FstProperties::WEIGHTED_CYCLES | FstProperties::UNWEIGHTED_CYCLES | dfs_props,
        ) {
            comp_props |= FstProperties::UNWEIGHTED_CYCLES;
        }

        let mut nfinal = 0;
        for state in 0..fst.num_states() {
            let state = state as StateId;
            let mut ilabels = if mask
                .intersects(FstProperties::I_DETERMINISTIC | FstProperties::NOT_I_DETERMINISTIC)
            {
                Some(HashSet::new())
            } else {
                None
            };
            let mut olabels = if mask
                .intersects(FstProperties::O_DETERMINISTIC | FstProperties::NOT_O_DETERMINISTIC)
            {
                Some(HashSet::new())
            } else {
                None
            };

            let mut prev_tr: Option<&Tr<W>> = None;
            for tr in fst.get_trs(state)?.trs() {
                // There is already an outgoing transition with this ilabel
                if let Some(ilabels_in) = &ilabels {
                    if ilabels_in.contains(&tr.ilabel) {
                        comp_props |= FstProperties::NOT_I_DETERMINISTIC;
                        comp_props &= !FstProperties::I_DETERMINISTIC;
                    }
                }

                // There is already an outgoing transition with this olabel
                if let Some(olabels_in) = &olabels {
                    if olabels_in.contains(&tr.olabel) {
                        comp_props |= FstProperties::NOT_O_DETERMINISTIC;
                        comp_props &= !FstProperties::O_DETERMINISTIC;
                    }
                }

                if tr.ilabel != tr.olabel {
                    comp_props |= FstProperties::NOT_ACCEPTOR;
                    comp_props &= !FstProperties::ACCEPTOR;
                }

                if tr.ilabel == 0 && tr.olabel == 0 {
                    comp_props |= FstProperties::EPSILONS;
                    comp_props &= !FstProperties::NO_EPSILONS;
                }

                if tr.ilabel == 0 {
                    comp_props |= FstProperties::I_EPSILONS;
                    comp_props &= !FstProperties::NO_I_EPSILONS;
                }

                if tr.olabel == 0 {
                    comp_props |= FstProperties::O_EPSILONS;
                    comp_props &= !FstProperties::NO_O_EPSILONS;
                }

                // Not first transition
                if let Some(_prev_tr) = prev_tr {
                    if tr.ilabel < _prev_tr.ilabel {
                        comp_props |= FstProperties::NOT_I_LABEL_SORTED;
                        comp_props &= !FstProperties::I_LABEL_SORTED;
                    }

                    if tr.olabel < _prev_tr.olabel {
                        comp_props |= FstProperties::NOT_O_LABEL_SORTED;
                        comp_props &= !FstProperties::O_LABEL_SORTED;
                    }
                }

                if !tr.weight.is_one() && !tr.weight.is_zero() {
                    comp_props |= FstProperties::WEIGHTED;
                    comp_props &= !FstProperties::UNWEIGHTED;

                    if comp_props.contains(FstProperties::UNWEIGHTED_CYCLES)
                        && sccs[state as usize] == sccs[tr.nextstate as usize]
                    {
                        comp_props |= FstProperties::WEIGHTED_CYCLES;
                        comp_props &= !FstProperties::UNWEIGHTED_CYCLES;
                    }
                }

                if tr.nextstate <= state {
                    comp_props |= FstProperties::NOT_TOP_SORTED;
                    comp_props &= !FstProperties::TOP_SORTED;
                }

                if tr.nextstate != state + 1 {
                    comp_props |= FstProperties::NOT_STRING;
                    comp_props &= !FstProperties::STRING;
                }

                prev_tr = Some(tr);

                if let Some(ilabels_in) = &mut ilabels {
                    ilabels_in.insert(tr.ilabel);
                }
                if let Some(olabels_in) = &mut olabels {
                    olabels_in.insert(tr.olabel);
                }
            }

            if nfinal > 0 {
                comp_props |= FstProperties::NOT_STRING;
                comp_props &= !FstProperties::STRING;
            }
            if fst.is_final(state)? {
                let final_weight = unsafe { fst.final_weight_unchecked(state).unsafe_unwrap() };
                if !final_weight.is_one() {
                    comp_props |= FstProperties::WEIGHTED;
                    comp_props &= !FstProperties::UNWEIGHTED;
                }
                nfinal += 1;
            } else if fst.num_trs(state)? != 1 {
                comp_props |= FstProperties::NOT_STRING;
                comp_props &= !FstProperties::STRING;
            }
        }

        if let Some(start) = fst.start() {
            if start != 0 {
                comp_props |= FstProperties::NOT_STRING;
                comp_props &= !FstProperties::STRING;
            }
        }
    }

    *known = known_properties(comp_props);
    Ok(comp_props)
}

use std::collections::HashSet;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::connect;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{Trs, EPS_LABEL};

/// Removes final states that have epsilon-only input trs.
pub fn rm_final_epsilon<W, F>(ifst: &mut F) -> Result<()>
where
    W: Semiring,
    F: MutableFst<W>,
{
    let mut visitors = SccVisitor::new(ifst, false, true);
    dfs_visit(ifst, &mut visitors, &AnyTrFilter {}, false);

    let mut finals = HashSet::new();

    for s in ifst.states_range() {
        if unsafe { ifst.is_final_unchecked(s) } {
            let mut future_coaccess = false;

            for tr in unsafe { ifst.get_trs_unchecked(s).trs() } {
                if visitors.coaccess[tr.nextstate as usize] {
                    future_coaccess = true;
                    break;
                }
            }

            if !future_coaccess {
                finals.insert(s);
            }
        }
    }

    let mut trs_to_del = vec![];
    for state in ifst.states_range() {
        let mut weight = None;
        trs_to_del.clear();

        for (idx, tr) in unsafe { ifst.get_trs_unchecked(state).trs().iter().enumerate() } {
            if finals.contains(&tr.nextstate) && tr.ilabel == EPS_LABEL && tr.olabel == EPS_LABEL {
                unsafe {
                    weight
                        .get_or_insert_with(|| {
                            ifst.final_weight_unchecked(state).unwrap_or_else(W::zero)
                        })
                        .plus_assign(
                            ifst.final_weight_unchecked(tr.nextstate)
                                .unsafe_unwrap()
                                .times(&tr.weight)?,
                        )?
                };
                trs_to_del.push(idx);
            }
        }

        if !trs_to_del.is_empty() {
            let w = unsafe { weight.unsafe_unwrap() };
            if !w.is_zero() {
                unsafe { ifst.set_final_unchecked(state, w) };
            }
            unsafe { ifst.del_trs_id_sorted_unchecked(state, &trs_to_del) };
        }
    }

    connect(ifst)?;

    Ok(())
}

use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use std::collections::HashMap;
use Result;
use StateId;

fn add_epsilon_arc_to_initial_state<F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Result<()>
where
    F1: ExpandedFst,
    F2: MutableFst,
{
    let start_state = fst_out.start().unwrap();
    if let Some(old_start_state_fst) = fst.start() {
        fst_out.add_arc(
            &start_state,
            Arc::new(
                0,
                0,
                <F2 as CoreFst>::W::one(),
                *mapping.get(&old_start_state_fst).unwrap(),
            ),
        )?;
    }
    Ok(())
}

fn set_new_final_states<W, F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Result<()>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W>,
{
    for old_final_state in fst.final_states_iter() {
        let final_state = mapping
            .get(&old_final_state.state_id)
            .ok_or_else(|| format_err!("Key {:?} doesn't exist in mapping", old_final_state.state_id))?;
        fst_out.set_final(
            final_state,
            fst.final_weight(&old_final_state.state_id)
                .ok_or_else(|| format_err!("State {:?} is not final", old_final_state.state_id))?,
        )?;
    }

    Ok(())
}

pub fn union<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Result<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    let start_state = fst_out.add_state();
    fst_out.set_start(&start_state)?;

    let mapping_states_fst_1 = fst_out.add_fst(fst_1)?;
    let mapping_states_fst_2 = fst_out.add_fst(fst_2)?;

    add_epsilon_arc_to_initial_state(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    add_epsilon_arc_to_initial_state(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    set_new_final_states(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    set_new_final_states(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    Ok(fst_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fst_impls::VectorFst;
    use semirings::BooleanWeight;
    use utils::transducer;

    #[test]
    fn test_union() {
        let v_a_1 = vec![1, 2, 3];
        let v_a_2 = vec![4, 5, 6];

        let v_b_1 = vec![10, 20, 30];
        let v_b_2 = vec![40, 50, 60];

        let t_a: VectorFst<BooleanWeight> =
            transducer(v_a_1.clone().into_iter(), v_a_2.clone().into_iter()).unwrap();
        let t_b: VectorFst<BooleanWeight> =
            transducer(v_b_1.clone().into_iter(), v_b_2.into_iter()).unwrap();

        let new_fst: VectorFst<_> = union(&t_a, &t_b).unwrap();

        println!("{:?}", new_fst);
    }

}

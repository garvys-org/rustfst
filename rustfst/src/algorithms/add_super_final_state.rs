use unsafe_unwrap::UnsafeUnwrap;

use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL};

/// Add, if needed, a super final state to the given FST. The super final state
/// is returned if it is possible.
///
/// # Definition
/// A super final state is a state that is the only final state in the FST with
/// a weight of `W::One()`.
///
/// # Behaviour
/// If the input FST has no final states, this algorithm will add super final state
/// that is connected to no other state.
///
/// If the input FST has only one final state with a weight of `W::One()`, this
/// algorithm will have no effect and this final state will be returned as the super
/// final state.
///
/// Otherwise, a final super state will be added to the input FST. Any final state will
/// point to this final super state where the transition weight will be their final weight.
///
pub fn add_super_final_state<W: Semiring, F: MutableFst<W>>(ifst: &mut F) -> StateId {
    let final_states = ifst.final_states_iter().collect::<Vec<_>>();
    if final_states.len() == 1
        && unsafe { ifst.final_weight_unchecked(final_states[0]) } == Some(W::one())
    {
        return final_states[0];
    }

    let super_final_state = ifst.add_state();
    unsafe {
        ifst.set_final_unchecked(super_final_state, W::one());
    }

    for final_state in final_states {
        let weight = unsafe {
            ifst.take_final_weight_unchecked(final_state)
                .unsafe_unwrap()
        };
        unsafe {
            ifst.add_tr_unchecked(
                final_state,
                Tr {
                    ilabel: EPS_LABEL,
                    olabel: EPS_LABEL,
                    weight,
                    nextstate: super_final_state,
                },
            )
        }
    }

    super_final_state
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::fst_impls::VectorFst;
    use crate::fst_traits::{CoreFst, ExpandedFst};
    use crate::semirings::TropicalWeight;

    use super::*;

    #[test]
    fn test_add_super_final_states() -> Result<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s0)?;
        fst.emplace_tr(s0, 1, 0, 1.0, s1)?;
        fst.emplace_tr(s1, 1, 0, 1.0, s2)?;
        fst.emplace_tr(s1, 1, 0, 1.0, s3)?;

        fst.set_final(s2, 1.0)?;
        fst.set_final(s3, 1.0)?;

        let num_states = fst.num_states();

        let super_final_state = add_super_final_state(&mut fst);
        assert_eq!(num_states, super_final_state as usize);
        assert!(!fst.is_final(s2)?);
        assert_eq!(1, fst.num_trs(s2)?);
        assert!(!fst.is_final(s3)?);
        assert_eq!(1, fst.num_trs(s3)?);
        assert_eq!(
            Some(TropicalWeight::one()),
            fst.final_weight(super_final_state)?
        );
        Ok(())
    }

    #[test]
    fn test_add_super_final_states_1() -> Result<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s0)?;
        fst.emplace_tr(s0, 1, 0, 1.0, s1)?;
        fst.emplace_tr(s1, 1, 0, 1.0, s2)?;
        fst.emplace_tr(s2, 1, 0, 1.0, s3)?;

        fst.set_final(s3, TropicalWeight::one())?;

        let super_final_state = add_super_final_state(&mut fst);
        assert_eq!(s3, super_final_state);
        assert_eq!(
            Some(TropicalWeight::one()),
            fst.final_weight(super_final_state)?
        );
        Ok(())
    }

    #[test]
    fn test_add_super_final_states_2() -> Result<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s0)?;
        fst.emplace_tr(s0, 1, 0, 1.0, s1)?;
        fst.emplace_tr(s1, 1, 0, 1.0, s2)?;
        fst.emplace_tr(s2, 1, 0, 1.0, s3)?;

        fst.set_final(s3, 2.0)?;

        let num_states = fst.num_states();

        let super_final_state = add_super_final_state(&mut fst);
        assert_eq!(num_states, super_final_state as usize);
        assert!(!fst.is_final(s3)?);
        assert_eq!(1, fst.num_trs(s3)?);
        assert_eq!(
            Some(TropicalWeight::one()),
            fst.final_weight(super_final_state)?
        );
        Ok(())
    }

    #[test]
    fn test_add_super_final_states_3() -> Result<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s0)?;
        fst.emplace_tr(s0, 1, 0, 1.0, s1)?;
        fst.emplace_tr(s1, 1, 0, 1.0, s2)?;
        fst.emplace_tr(s2, 1, 0, 1.0, s3)?;

        let num_states = fst.num_states();

        let super_final_state = add_super_final_state(&mut fst);
        assert_eq!(num_states, super_final_state as usize);
        assert_eq!(
            Some(TropicalWeight::one()),
            fst.final_weight(super_final_state)?
        );
        Ok(())
    }
}

use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::closure::ClosureType;
use crate::fst_properties::mutable_properties::closure_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::EPS_LABEL;

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
///  If closure_star then the empty string is transduced to itself with weight `1` as well.
///
/// # Example
///
/// ## Input
/// ![closure_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_in.svg?sanitize=true)
///
/// ## Closure Plus
/// ![closure_out_closure_plus](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_plus.svg?sanitize=true)
///
/// ## Closure Star
/// ![closure_out_closure_star](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/closure_out_closure_star.svg?sanitize=true)
pub fn closure<W, F>(fst: &mut F, closure_type: ClosureType)
where
    W: Semiring,
    F: MutableFst<W>,
{
    let props = fst.properties();
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst
            .final_states_iter()
            .map(|s| (s, unsafe { fst.final_weight_unchecked(s).unsafe_unwrap() }))
            .collect();
        for (final_state_id, final_weight) in final_states_id {
            unsafe {
                fst.add_tr_unchecked(
                    final_state_id,
                    Tr::new(EPS_LABEL, EPS_LABEL, final_weight, start_state),
                )
            };
        }
    }

    if closure_type == ClosureType::ClosureStar {
        let nstart = fst.add_state();

        // Add a new start state to allow empty path
        if let Some(start_state_id) = fst.start() {
            unsafe {
                fst.add_tr_unchecked(
                    nstart,
                    Tr::new(EPS_LABEL, EPS_LABEL, W::one(), start_state_id),
                );
            }
        }

        unsafe {
            fst.set_start_unchecked(nstart);
            fst.set_final_unchecked(nstart, W::one());
        }
    }

    fst.set_properties_with_mask(
        closure_properties(props, false),
        FstProperties::all_properties(),
    );
}

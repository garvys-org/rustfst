use crate::algorithms::fst_convert_from_ref;
use crate::algorithms::lookahead_matchers::add_on::FstAddOn;
use crate::algorithms::lookahead_matchers::label_reachable::{LabelReachable, LabelReachableData};
use crate::algorithms::matchers::MatchType;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{Fst, MutableFst};

use failure::Fallible;

pub struct LabelLookAheadRelabeler {}

impl LabelLookAheadRelabeler {
    pub fn init<F: MutableFst>(
        fst_addon: &mut FstAddOn<F, (Option<LabelReachableData>, Option<LabelReachableData>)>,
    ) -> Fallible<()> {
        let fst = &mut fst_addon.fst;
        let data = &fst_addon.add_on;

        let mfst = fst;

        if data.0.is_some() {
            let reachable = LabelReachable::new_from_data(data.0.as_ref().unwrap().clone());
            reachable.relabel_fst(mfst, true)?;
        } else {
            let reachable = LabelReachable::new_from_data(data.1.as_ref().unwrap().clone());
            reachable.relabel_fst(mfst, false)?;
        }

        Ok(())
    }

    pub fn relabel<F: MutableFst>(
        fst: &mut F,
        addon: &(Option<LabelReachableData>, Option<LabelReachableData>),
        relabel_input: bool,
    ) -> Fallible<()> {
        let reachable_data = if addon.0.as_ref().is_some() {
            addon.0.as_ref().unwrap().clone()
        } else {
            addon.1.as_ref().unwrap().clone()
        };
        let reachable = LabelReachable::new_from_data(reachable_data);
        reachable.relabel_fst(fst, relabel_input)
    }
}

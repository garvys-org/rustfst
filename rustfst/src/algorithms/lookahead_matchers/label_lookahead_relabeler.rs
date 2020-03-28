use failure::Fallible;

use crate::algorithms::lookahead_matchers::add_on::FstAddOn;
use crate::algorithms::lookahead_matchers::label_reachable::{LabelReachable, LabelReachableData};
use crate::fst_traits::MutableFst;
use std::rc::Rc;

pub struct LabelLookAheadRelabeler {}

impl LabelLookAheadRelabeler {
    pub fn init<F: MutableFst>(
        fst_addon: &mut FstAddOn<
            F,
            (
                Option<Rc<LabelReachableData>>,
                Option<Rc<LabelReachableData>>,
            ),
        >,
    ) -> Fallible<()> {
        let fst = &mut fst_addon.fst;
        let data = &fst_addon.add_on;

        let mfst = fst;

        if data.0.is_some() {
            let reachable = LabelReachable::new_from_data(Rc::clone(data.0.as_ref().unwrap()));
            reachable.relabel_fst(mfst, true)?;
        } else {
            let reachable = LabelReachable::new_from_data(Rc::clone(data.1.as_ref().unwrap()));
            reachable.relabel_fst(mfst, false)?;
        }

        Ok(())
    }

    pub fn relabel<F: MutableFst>(
        fst: &mut F,
        addon: &(
            Option<Rc<LabelReachableData>>,
            Option<Rc<LabelReachableData>>,
        ),
        relabel_input: bool,
    ) -> Fallible<()> {
        let reachable_data = if addon.0.as_ref().is_some() {
            Rc::clone(addon.0.as_ref().unwrap())
        } else {
            Rc::clone(addon.1.as_ref().unwrap())
        };
        let reachable = LabelReachable::new_from_data(reachable_data);
        reachable.relabel_fst(fst, relabel_input)
    }
}

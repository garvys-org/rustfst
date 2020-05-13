use std::cell::RefCell;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::FstAddOn;
use crate::algorithms::compose::{LabelReachable, LabelReachableData};
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

pub struct LabelLookAheadRelabeler {}

impl LabelLookAheadRelabeler {
    pub fn init<W: Semiring, F: MutableFst<W>>(
        fst_addon: &mut FstAddOn<
            F,
            (
                Option<Arc<RefCell<LabelReachableData>>>,
                Option<Arc<RefCell<LabelReachableData>>>,
            ),
        >,
    ) -> Result<()> {
        let fst = &mut fst_addon.fst;
        let data = &fst_addon.add_on;

        let mfst = fst;

        if data.0.is_some() {
            let reachable = LabelReachable::new_from_data(Arc::clone(data.0.as_ref().unwrap()));
            reachable.relabel_fst(mfst, true)?;
        } else {
            let reachable = LabelReachable::new_from_data(Arc::clone(data.1.as_ref().unwrap()));
            reachable.relabel_fst(mfst, false)?;
        }

        Ok(())
    }

    pub fn relabel<W: Semiring, F: MutableFst<W>>(
        fst: &mut F,
        addon: &(
            Option<Arc<RefCell<LabelReachableData>>>,
            Option<Arc<RefCell<LabelReachableData>>>,
        ),
        relabel_input: bool,
    ) -> Result<()> {
        let reachable_data = if addon.0.as_ref().is_some() {
            Arc::clone(addon.0.as_ref().unwrap())
        } else {
            Arc::clone(addon.1.as_ref().unwrap())
        };
        let reachable = LabelReachable::new_from_data(reachable_data);
        reachable.relabel_fst(fst, relabel_input)
    }
}

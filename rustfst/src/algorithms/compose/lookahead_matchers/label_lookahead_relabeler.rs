use anyhow::Result;

use crate::algorithms::compose::LabelReachableData;
use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::Semiring;
use std::sync::Arc;

pub struct LabelLookAheadRelabeler {}

impl LabelLookAheadRelabeler {
    pub fn init<W: Semiring, F: MutableFst<W>>(
        mfst: &mut F,
        addon: &mut (Option<LabelReachableData>, Option<LabelReachableData>),
    ) -> Result<()> {
        if let Some(reachable_data) = &mut addon.0 {
            reachable_data.relabel_fst(mfst, true)?;
            return Ok(());
        }

        if let Some(reachable_data) = &mut addon.1 {
            reachable_data.relabel_fst(mfst, false)?;
            return Ok(());
        }

        bail!("Addon contains only None elements")
    }

    pub fn relabel<W: Semiring, F: MutableFst<W>>(
        fst: &mut F,
        addon: &mut (Option<LabelReachableData>, Option<LabelReachableData>),
        relabel_input: bool,
    ) -> Result<()> {
        if let Some(reachable_data) = &mut addon.0 {
            reachable_data.relabel_fst(fst, relabel_input)?;
            return Ok(());
        }
        if let Some(reachable_data) = &mut addon.1 {
            reachable_data.relabel_fst(fst, relabel_input)?;
            return Ok(());
        }
        bail!("Addon contains only None elements")
    }

    pub fn relabel_lazy<W: Semiring, F: Fst<W> + 'static>(
        fst: Arc<F>,
        addon: &mut (Option<LabelReachableData>, Option<LabelReachableData>),
        relabel_input: bool,
    ) -> Result<impl Fst<W>> {
        if let Some(reachable_data) = &mut addon.0 {
            let lazy_fst = reachable_data.relabel_fst_lazy(fst, relabel_input)?;
            return Ok(lazy_fst);
        }

        if let Some(reachable_data) = &mut addon.1 {
            let lazy_fst = reachable_data.relabel_fst_lazy(fst, relabel_input)?;
            return Ok(lazy_fst);
        }

        bail!("Addon contains only None elements")
    }
}

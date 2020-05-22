use anyhow::Result;

use crate::algorithms::compose::LabelReachableData;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

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
}

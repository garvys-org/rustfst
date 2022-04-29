use anyhow::Result;

use crate::fst_traits::MutableFst;
use crate::Semiring;

pub fn epsilon_machine<W: Semiring, F: MutableFst<W>>() -> Result<F> {
    let mut fst = F::new();
    let s = fst.add_state();
    fst.set_start(s)?;
    fst.set_final(s, W::one())?;
    Ok(fst)
}

use std::path::Path;

use failure::Fallible;

use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

pub trait BinarySerializer: ExpandedFst
where
    Self::W: Semiring<Type = f32>,
{
    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Fallible<()>;
}

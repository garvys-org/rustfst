use std::path::Path;

use failure::Fallible;

use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

pub trait BinaryDeserializer: ExpandedFst
where
    Self::W: Semiring<Type = f32>,
{
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self>;
}

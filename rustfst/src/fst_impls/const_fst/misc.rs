use std::fmt;

use crate::fst_impls::ConstFst;
use crate::fst_traits::{ArcIterator, CoreFst, FinalStatesIterator, StateIterator};
use crate::semirings::Semiring;

display_fst_trait!(W, ConstFst<W>);

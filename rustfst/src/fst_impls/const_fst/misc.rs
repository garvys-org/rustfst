use std::fmt;

use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, FinalStatesIterator, StateIterator};
use crate::semirings::SerializableSemiring;
use crate::Trs;

display_fst_trait!(W, ConstFst<W>);

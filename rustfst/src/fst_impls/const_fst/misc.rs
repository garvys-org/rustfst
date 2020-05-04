use std::fmt;

use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, FinalStatesIterator, StateIterator, TrIterator};
use crate::semirings::SerializableSemiring;

display_fst_trait!(W, ConstFst<W>);

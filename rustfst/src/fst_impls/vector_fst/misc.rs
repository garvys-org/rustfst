use std::fmt;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{TrIterator, CoreFst, FinalStatesIterator, StateIterator};
use crate::semirings::SerializableSemiring;

display_fst_trait!(W, VectorFst<W>);

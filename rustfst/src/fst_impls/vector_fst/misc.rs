use std::fmt;

use unsafe_unwrap::UnsafeUnwrap;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, Fst, StateIterator};
use crate::semirings::SerializableSemiring;
use crate::Trs;

display_fst_trait!(W, VectorFst<W>);

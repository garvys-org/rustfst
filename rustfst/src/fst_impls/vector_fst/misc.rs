use std::fmt;
use std::ops::{Add, BitOr};

use failure::Fallible;

use crate::algorithms::{concat, union};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, CoreFst, FinalStatesIterator, StateIterator};
use crate::semirings::Semiring;

add_or_fst!(W, VectorFst<W>);
display_fst_trait!(W, VectorFst<W>);

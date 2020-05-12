use std::collections::HashMap;
use std::iter::{Map, Repeat, repeat, Zip};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use itertools::izip;
use unsafe_unwrap::UnsafeUnwrap;

use crate::{StateId, Trs, SymbolTable, TrsVec};
use crate::fst_traits::{CoreFst, Fst, FstIterator, FstIterData, StateIterator};
use crate::semirings::Semiring;
use std::fmt::Debug;




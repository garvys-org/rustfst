use crate::Label;

mod simple_transducer;
mod tropical_weight;
mod vector_fst;

static MAX_NUM_OPERATIONS: usize = 30;
static MAX_ILABEL: Label = 10;
static MAX_OLABEL: Label = 10;
static MAX_WEIGHT_VALUE: usize = 10;

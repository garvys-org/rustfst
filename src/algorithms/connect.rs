use semirings::Semiring;
use fst::{ExpandedFst, MutableFst};

pub fn connect<W: Semiring, F: ExpandedFst<W> + MutableFst<W>> (fst: &mut F) {

}
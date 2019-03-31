use failure::Fallible;

use crate::semirings::{Semiring, StringWeightLeft};

trait CommonDivisor<W: Semiring> {
    fn common_divisor(w1: &W, w2: &W) -> Fallible<W>;
}

struct DefaultCommonDivisor {}

impl<W: Semiring> CommonDivisor<W> for DefaultCommonDivisor {
    fn common_divisor(w1: &W, w2: &W) -> Fallible<W> {
        w1.plus(w2)
    }
}

struct LabelCommonDivisor {}

impl CommonDivisor<StringWeightLeft> for LabelCommonDivisor {
    fn common_divisor(w1: &StringWeightLeft, w2: &StringWeightLeft) -> Fallible<StringWeightLeft> {
        if w1.value.is_empty_list() || w2.value.is_empty_list() {
            Ok(StringWeightLeft::one())
        } else {
            unimplemented!()
        }
    }
}

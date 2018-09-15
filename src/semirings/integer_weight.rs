use semirings::Semiring;

#[derive(Clone, Debug, PartialEq)]
pub struct IntegerWeight {
    value: i32
}

impl IntegerWeight {
    pub fn new(value: i32) -> Self {
        IntegerWeight {value}
    }
}

impl Semiring for IntegerWeight {
    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value * rhs.value)
    }

    fn zero() -> Self {
        Self::new(0)
    }

    fn one() -> Self {
        Self::new(1)
    }
}
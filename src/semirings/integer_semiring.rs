use semirings::Semiring;

#[derive(Clone)]
pub struct IntegerSemiring {
    value: i32
}

impl IntegerSemiring {
    pub fn new(value: i32) -> Self {
        IntegerSemiring {value}
    }
}

impl Semiring for IntegerSemiring {
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
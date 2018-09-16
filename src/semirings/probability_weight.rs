use semirings::Semiring;

#[derive(Clone, Debug, PartialEq)]
pub struct ProbabilityWeight {
    value: f32,
}

impl ProbabilityWeight {
    pub fn new(value: f32) -> Self {
        ProbabilityWeight { value }
    }
}

impl Semiring for ProbabilityWeight {
    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value * rhs.value)
    }

    fn zero() -> Self {
        Self::new(0.0)
    }

    fn one() -> Self {
        Self::new(1.0)
    }
}

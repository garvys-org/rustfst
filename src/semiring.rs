pub trait Semiring: Clone {
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

#[derive(Clone)]
pub struct ProbabilitySemiring {
    value: f32
}

impl ProbabilitySemiring {
    pub fn new(value: f32) -> Self {
        ProbabilitySemiring {value}
    }
}

impl Semiring for ProbabilitySemiring {
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
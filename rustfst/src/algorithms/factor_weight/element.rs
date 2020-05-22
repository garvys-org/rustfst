use crate::semirings::Semiring;
use crate::StateId;

#[derive(PartialOrd, PartialEq, Hash, Clone, Debug, Eq)]
pub struct Element<W: Semiring> {
    pub state: Option<StateId>,
    pub weight: W,
}

impl<W: Semiring> Element<W> {
    pub fn new(state: Option<StateId>, weight: W) -> Self {
        Self { state, weight }
    }
}

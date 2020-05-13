use crate::algorithms::shortest_distance::ShortestDistanceConfig;
use crate::algorithms::tr_filters::EpsilonTrFilter;
use crate::algorithms::Queue;
use crate::semirings::Semiring;
use crate::StateId;

pub struct RmEpsilonConfig<W: Semiring, Q: Queue> {
    pub sd_opts: ShortestDistanceConfig<W, Q, EpsilonTrFilter>,
    pub connect: bool,
    pub weight_threshold: W,
    pub state_threshold: Option<StateId>,
}

impl<W: Semiring, Q: Queue> RmEpsilonConfig<W, Q> {
    pub fn new(
        queue: Q,
        connect: bool,
        weight_threshold: W,
        state_threshold: Option<StateId>,
    ) -> Self {
        Self {
            sd_opts: ShortestDistanceConfig::new_with_default(EpsilonTrFilter {}, queue),
            connect,
            weight_threshold,
            state_threshold,
        }
    }

    pub fn new_with_default(queue: Q) -> Self {
        Self::new(queue, true, W::zero(), None)
    }
}

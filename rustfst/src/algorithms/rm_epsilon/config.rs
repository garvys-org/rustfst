use crate::algorithms::shortest_distance::ShortestDistanceInternalConfig;
use crate::algorithms::tr_filters::EpsilonTrFilter;
use crate::algorithms::Queue;
use crate::semirings::Semiring;
use crate::{StateId, KSHORTESTDELTA};

pub(crate) struct RmEpsilonInternalConfig<W: Semiring, Q: Queue> {
    pub(crate) sd_opts: ShortestDistanceInternalConfig<W, Q, EpsilonTrFilter>,
    pub connect: bool,
    pub weight_threshold: W,
    pub state_threshold: Option<StateId>,
}

impl<W: Semiring, Q: Queue> RmEpsilonInternalConfig<W, Q> {
    pub fn new(
        queue: Q,
        connect: bool,
        weight_threshold: W,
        state_threshold: Option<StateId>,
        delta: f32,
    ) -> Self {
        Self {
            sd_opts: ShortestDistanceInternalConfig::new_with_default(
                EpsilonTrFilter {},
                queue,
                delta,
            ),
            connect,
            weight_threshold,
            state_threshold,
        }
    }

    pub fn new_with_default(queue: Q) -> Self {
        Self::new(queue, true, W::zero(), None, KSHORTESTDELTA)
    }
}

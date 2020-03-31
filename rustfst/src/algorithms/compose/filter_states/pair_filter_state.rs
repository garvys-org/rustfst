use self::super::FilterState;

/// Filter state that is the combination of two filter states.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct PairFilterState<FS1, FS2> {
    state: (FS1, FS2),
}

impl<FS1: FilterState, FS2: FilterState> FilterState for PairFilterState<FS1, FS2> {
    type Type = (FS1, FS2);

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self {
            state: (FS1::new_no_state(), FS2::new_no_state()),
        }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

impl<FS1, FS2> PairFilterState<FS1, FS2> {
    pub fn state1(&self) -> &FS1 {
        &self.state.0
    }

    pub fn state2(&self) -> &FS2 {
        &self.state.1
    }
}

use self::super::FilterState;

/// Filter state that is the combination of two filter states.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct PairFilterState<FS1: FilterState, FS2: FilterState> {
    state: Option<(FS1, FS2)>,
}

impl<FS1: FilterState, FS2: FilterState> FilterState for PairFilterState<FS1, FS2> {
    type Type = (FS1, FS2);

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn state(&self) -> Option<&Self::Type> {
        self.state.as_ref()
    }
}

impl<FS1: FilterState, FS2: FilterState> Default for PairFilterState<FS1, FS2> {
    fn default() -> Self {
        Self { state: None }
    }
}

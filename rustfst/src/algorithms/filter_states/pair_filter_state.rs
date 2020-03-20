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

    fn new_no_state() -> Self {
        unimplemented!()
    }

    fn state(&self) -> &Self::Type {
        unimplemented!()
    }
}
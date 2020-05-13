use std::cell::RefCell;
use std::sync::Arc;

pub struct ComposeFstOpOptions<M1, M2, CF, ST> {
    pub matcher1: Option<Arc<RefCell<M1>>>,
    pub matcher2: Option<Arc<RefCell<M2>>>,
    pub filter: Option<CF>,
    pub state_table: Option<ST>,
}

impl<M1, M2, CF, ST> Default for ComposeFstOpOptions<M1, M2, CF, ST> {
    fn default() -> Self {
        Self {
            matcher1: None,
            matcher2: None,
            filter: None,
            state_table: None,
        }
    }
}

impl<M1, M2, CF, ST> ComposeFstOpOptions<M1, M2, CF, ST> {
    pub fn new<
        IM1: Into<Option<Arc<RefCell<M1>>>>,
        IM2: Into<Option<Arc<RefCell<M2>>>>,
        ICF: Into<Option<CF>>,
        IST: Into<Option<ST>>,
    >(
        matcher1: IM1,
        matcher2: IM2,
        filter: ICF,
        state_table: IST,
    ) -> Self {
        Self {
            matcher1: matcher1.into(),
            matcher2: matcher2.into(),
            filter: filter.into(),
            state_table: state_table.into(),
        }
    }
}

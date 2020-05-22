pub struct ComposeFstOpOptions<M1, M2, CFB, ST> {
    pub matcher1: Option<M1>,
    pub matcher2: Option<M2>,
    pub filter_builder: Option<CFB>,
    pub state_table: Option<ST>,
}

impl<M1, M2, CFB, ST> Default for ComposeFstOpOptions<M1, M2, CFB, ST> {
    fn default() -> Self {
        Self {
            matcher1: None,
            matcher2: None,
            filter_builder: None,
            state_table: None,
        }
    }
}

impl<M1, M2, CFB, ST> ComposeFstOpOptions<M1, M2, CFB, ST> {
    pub fn new<
        IM1: Into<Option<M1>>,
        IM2: Into<Option<M2>>,
        ICFB: Into<Option<CFB>>,
        IST: Into<Option<ST>>,
    >(
        matcher1: IM1,
        matcher2: IM2,
        filter: ICFB,
        state_table: IST,
    ) -> Self {
        Self {
            matcher1: matcher1.into(),
            matcher2: matcher2.into(),
            filter_builder: filter.into(),
            state_table: state_table.into(),
        }
    }
}

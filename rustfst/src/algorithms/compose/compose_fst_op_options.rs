pub struct ComposeFstOpOptions<M1, M2, CFB, OS> {
    pub matcher1: Option<M1>,
    pub matcher2: Option<M2>,
    pub filter_builder: Option<CFB>,
    pub op_state: Option<OS>,
}

impl<M1, M2, CFB, OS> Default for ComposeFstOpOptions<M1, M2, CFB, OS> {
    fn default() -> Self {
        Self {
            matcher1: None,
            matcher2: None,
            filter_builder: None,
            op_state: None,
        }
    }
}

impl<M1, M2, CFB, OS> ComposeFstOpOptions<M1, M2, CFB, OS> {
    pub fn new<
        IM1: Into<Option<M1>>,
        IM2: Into<Option<M2>>,
        ICFB: Into<Option<CFB>>,
        IST: Into<Option<OS>>,
    >(
        matcher1: IM1,
        matcher2: IM2,
        filter: ICFB,
        op_state: IST,
    ) -> Self {
        Self {
            matcher1: matcher1.into(),
            matcher2: matcher2.into(),
            filter_builder: filter.into(),
            op_state: op_state.into(),
        }
    }
}

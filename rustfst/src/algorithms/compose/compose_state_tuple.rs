use crate::StateId;

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct ComposeStateTuple<FS> {
    pub fs: FS,
    pub s1: StateId,
    pub s2: StateId,
}

use crate::algorithms::matchers::SortedMatcher;

// TODO: Change this to use InitMatcher once supported.
pub type GenericMatcher<'fst, F> = SortedMatcher<'fst, F>;

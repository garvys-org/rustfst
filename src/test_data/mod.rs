#[macro_use]
pub(crate) mod test_fst_trait;
pub(crate) mod vector_fst;

#[cfg(test)]
pub(crate) use self::test_fst_trait::{TestFst, TestFstData};

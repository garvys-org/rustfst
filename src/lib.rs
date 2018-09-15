
pub type Label = usize;
pub type StateId = usize;

pub mod semirings;
pub mod arc;
pub mod fst;
pub mod vector_fst;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

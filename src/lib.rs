
pub type Label = usize;
pub type StateId = usize;

pub mod semiring;
pub mod arc;
// pub mod fst;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod compose;
pub mod concat;
pub mod connect;
pub mod determinize;
pub mod isomorphic;
mod minimize;
pub mod optimize;
pub mod project;
pub mod randgen;
pub mod replace;
pub mod reverse;
pub mod rm_epsilon;
pub mod shortest_path;
pub mod top_sort;
pub mod tr_sort;
pub mod tr_unique;
pub mod union;

#[derive(Debug)]
pub struct EnumConversionError {}

impl std::error::Error for EnumConversionError {}

impl std::fmt::Display for EnumConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unexpected enum variant")
    }
}

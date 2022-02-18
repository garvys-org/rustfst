pub mod compose;
pub mod concat;
pub mod determinize;
pub mod optimize;
pub mod project;
pub mod replace;
pub mod reverse;
pub mod shortest_path;
pub mod tr_sort;
pub mod union;

#[derive(Debug)]
pub struct EnumConversionError {}

impl std::error::Error for EnumConversionError {}

impl std::fmt::Display for EnumConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unexpected enum variant")
    }
}

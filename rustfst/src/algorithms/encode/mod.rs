pub use decode_static::decode;
pub use encode_static::encode;
pub(self) use table::EncodeTableMut;
pub use table::EncodeTable;

mod table;
mod encode_static;
mod decode_static;


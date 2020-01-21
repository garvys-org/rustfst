use failure::Fallible;
use std::io::Write;

#[inline]
pub(crate) fn write_bin_i32<W: Write>(file: &mut W, i: i32) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_u32<W: Write>(file: &mut W, i: u32) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_u64<W: Write>(file: &mut W, i: u64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_i64<W: Write>(file: &mut W, i: i64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_f32<W: Write>(file: &mut W, i: f32) -> Fallible<()> {
    file.write_all(&i.to_bits().to_le_bytes())
        .map_err(|e| e.into())
}

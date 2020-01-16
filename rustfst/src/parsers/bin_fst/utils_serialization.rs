use failure::Fallible;
use std::io::Write;

#[inline]
pub(crate) fn write_bin_i32<F: Write>(file: &mut F, i: i32) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_u64<F: Write>(file: &mut F, i: u64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_i64<F: Write>(file: &mut F, i: i64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_f32<F: Write>(file: &mut F, i: f32) -> Fallible<()> {
    file.write_all(&i.to_bits().to_le_bytes())
        .map_err(|e| e.into())
}

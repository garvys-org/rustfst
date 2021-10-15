use anyhow::Result;
use std::io::Write;

#[inline]
pub fn write_bin_i32<F: Write>(file: &mut F, i: i32) -> Result<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub fn write_bin_u32<W: Write>(file: &mut W, i: u32) -> Result<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub fn write_bin_u64<W: Write>(file: &mut W, i: u64) -> Result<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub fn write_bin_i64<F: Write>(file: &mut F, i: i64) -> Result<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
pub fn write_bin_f32<F: Write>(file: &mut F, i: f32) -> Result<()> {
    file.write_all(&i.to_bits().to_le_bytes())
        .map_err(|e| e.into())
}

#[inline]
pub(crate) fn write_bin_u8<F: Write>(file: &mut F, i: u8) -> Result<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

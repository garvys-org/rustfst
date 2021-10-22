use crate::parsers::{write_bin_i32, write_bin_u8};
use crate::semirings::SerializableSemiring;
use crate::Tr;
use anyhow::Result;
use std::io::Write;

#[inline]
pub(crate) fn write_bin_fst_tr<F: Write, W: SerializableSemiring>(
    file: &mut F,
    tr: &Tr<W>,
) -> Result<()> {
    write_bin_i32(file, tr.ilabel as i32)?;
    write_bin_i32(file, tr.olabel as i32)?;
    tr.weight.write_binary(file)?;
    write_bin_i32(file, tr.nextstate as i32)?;
    Ok(())
}

pub fn write_final_weight<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    final_weight: &Option<W>,
) -> Result<()> {
    if let Some(final_weight) = final_weight {
        // Write as Some
        write_bin_u8(writter, 1)?;
        final_weight.write_binary(writter)?;
    } else {
        // Write as None
        write_bin_u8(writter, 0)?;
    };
    Ok(())
}

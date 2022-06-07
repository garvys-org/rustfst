use crate::{FstPath, Label, Semiring, SymbolTable};
use anyhow::{format_err, Result};
use std::sync::Arc;

/// Wrapper around `FstPath` to nicely handle `SymbolTable`s.
#[derive(Debug, Clone)]
pub struct StringPath<W: Semiring> {
    fst_path: FstPath<W>,
    isymt: Arc<SymbolTable>,
    osymt: Arc<SymbolTable>,
}

impl<W: Semiring> StringPath<W> {
    pub fn new(fst_path: FstPath<W>, isymt: Arc<SymbolTable>, osymt: Arc<SymbolTable>) -> Self {
        Self {
            fst_path,
            isymt,
            osymt,
        }
    }

    pub fn weight(&self) -> &W {
        &self.fst_path.weight
    }

    pub fn ilabels(&self) -> &[Label] {
        self.fst_path.ilabels.as_slice()
    }

    pub fn olabels(&self) -> &[Label] {
        self.fst_path.olabels.as_slice()
    }

    pub fn istring(&self) -> Result<String> {
        let res: Result<Vec<_>> = self
            .fst_path
            .ilabels
            .iter()
            .map(|e| {
                self.isymt
                    .get_symbol(*e)
                    .ok_or_else(|| format_err!("Missing {} in symbol table", e))
                // .map_err()
            })
            .collect();
        let res = res?.join(" ");
        Ok(res)
    }

    pub fn ostring(&self) -> Result<String> {
        let res: Result<Vec<_>> = self
            .fst_path
            .olabels
            .iter()
            .map(|e| {
                self.osymt
                    .get_symbol(*e)
                    .ok_or_else(|| format_err!("Missing {} in symbol table", e))
                // .map_err()
            })
            .collect();
        let res = res?.join(" ");
        Ok(res)
    }
}

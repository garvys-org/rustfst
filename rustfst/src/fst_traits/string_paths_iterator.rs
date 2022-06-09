use crate::fst_traits::paths_iterator::PathsIterator;
use crate::fst_traits::Fst;
use crate::{Semiring, StringPath, SymbolTable};
use anyhow::{format_err, Result};
use std::sync::Arc;

/// Iterator on the paths recognized by an Fst. Plus handles the SymbolTable
/// to be able to retrieve the strings.
pub struct StringPathsIterator<'a, W, F>
where
    W: Semiring,
    F: 'a + Fst<W>,
{
    isymt: Arc<SymbolTable>,
    osymt: Arc<SymbolTable>,
    paths_iter: PathsIterator<'a, W, F>,
}

impl<'a, W, F> StringPathsIterator<'a, W, F>
where
    W: Semiring,
    F: 'a + Fst<W>,
{
    pub fn new(fst: &'a F) -> Result<Self> {
        let paths_iter = PathsIterator::new(fst);
        let isymt = fst
            .input_symbols()
            .ok_or_else(|| format_err!("Missing input symbol table"))?;
        let osymt = fst
            .output_symbols()
            .ok_or_else(|| format_err!("Missing output symbol table"))?;
        Ok(Self {
            paths_iter,
            isymt: Arc::clone(isymt),
            osymt: Arc::clone(osymt),
        })
    }
}

impl<'a, W, F> Iterator for StringPathsIterator<'a, W, F>
where
    W: Semiring,
    F: 'a + Fst<W>,
{
    type Item = StringPath<W>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths_iter.next().map(|fst_path| {
            StringPath::new(fst_path, Arc::clone(&self.isymt), Arc::clone(&self.osymt))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::prelude::TropicalWeight;
    use crate::symt;
    use crate::utils::transducer;

    #[test]
    fn test_string_paths_iterator() -> Result<()> {
        let mut fst: VectorFst<_> = transducer(&[1, 2, 3], &[4, 5], TropicalWeight::one());
        let symt = symt!["a", "b", "c", "d", "e"];
        let symt = Arc::new(symt);
        fst.set_input_symbols(Arc::clone(&symt));
        fst.set_output_symbols(Arc::clone(&symt));

        let paths: Vec<_> = fst.string_paths_iter()?.collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].ilabels(), &[1, 2, 3]);
        assert_eq!(paths[0].olabels(), &[4, 5]);
        assert_eq!(paths[0].weight(), &TropicalWeight::one());
        assert_eq!(paths[0].istring()?, "a b c".to_string());
        assert_eq!(paths[0].ostring()?, "d e".to_string());

        Ok(())
    }
}

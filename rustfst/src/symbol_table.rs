use std::fmt;
use std::fs::{read, File};
use std::io::{BufWriter, LineWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::parsers::bin_symt::nom_parser::{parse_symbol_table_bin, write_bin_symt};
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::text_symt::parsed_text_symt::ParsedTextSymt;
use crate::{Label, EPS_SYMBOL};
use std::collections::hash_map::{Entry, RandomState};
use std::collections::HashMap;
use std::hash::BuildHasher;

/// A symbol table stores a bidirectional mapping between transition labels and "symbols" (strings).
#[derive(Debug, Clone)]
pub struct SymbolTable<H: BuildHasher = RandomState> {
    bimap: BiHashMapString<H>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Creates a `SymbolTable` with a single element in it: the pair (`EPS_LABEL`, `EPS_SYMBOL`).
    ///
    /// # Examples
    /// ```rust
    /// # use rustfst::SymbolTable;
    /// let mut symt = SymbolTable::new();
    /// ```
    pub fn new() -> Self {
        let mut symt = SymbolTable::empty();

        symt.add_symbol(EPS_SYMBOL.to_string());

        symt
    }

    pub fn empty() -> Self {
        SymbolTable {
            bimap: BiHashMapString::new(),
        }
    }

    fn from_parsed_symt_text(parsed_symt_text: ParsedTextSymt) -> Result<Self> {
        let mut bimap = BiHashMapString::new();
        for (symbol, label) in parsed_symt_text.pairs.into_iter() {
            let inserted_label = bimap.get_id_or_insert(symbol);
            if inserted_label as Label != label {
                bail!("The SymbolTable should contain labels with increasing ids and no hole. Expected {} and got {}", inserted_label, label)
            }
        }

        Ok(SymbolTable { bimap })
    }

    pub fn from_text_string(symt_string: &str) -> Result<Self> {
        let parsed_symt = ParsedTextSymt::from_string(symt_string)?;
        Self::from_parsed_symt_text(parsed_symt)
    }

    pub fn read_text<P: AsRef<Path>>(path_text_symt: P) -> Result<Self> {
        let parsed_symt = ParsedTextSymt::from_path(path_text_symt)?;
        Self::from_parsed_symt_text(parsed_symt)
    }

    pub fn read<P: AsRef<Path>>(path_bin_symt: P) -> Result<Self> {
        let data = read(path_bin_symt.as_ref()).with_context(|| {
            format!(
                "Can't open SymbolTable binary file : {:?}",
                path_bin_symt.as_ref()
            )
        })?;

        let (_, symt) = parse_symbol_table_bin(&data).map_err(|e| {
            e.map(|e_inner| match e_inner {
                NomCustomError::Nom(_, k) => {
                    format_err!("Error while parsing binary SymbolTable. Error kind {:?}", k)
                }
                NomCustomError::SymbolTableError(e) => {
                    format_err!("Error while parsing symbolTable from binary : {}", e)
                }
            })
        })?;

        Ok(symt)
    }
}

impl<H: BuildHasher> SymbolTable<H> {
    pub fn with_hasher(hasher_builder: H) -> Self {
        let mut bimap = BiHashMapString::with_hasher(hasher_builder);
        bimap.get_id_or_insert(EPS_SYMBOL);
        Self { bimap }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds a symbol to the symbol table. The corresponding label is returned.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let mut symt = symt!["a", "b"];
    ///
    /// // Elements in the table : `<eps>`, `a`, `b`
    /// assert_eq!(symt.len(), 3);
    ///
    /// // Add a single symbol
    /// symt.add_symbol("c");
    ///
    /// // Elements in the table : `<eps>`, `a`, `b`, `c`
    /// assert_eq!(symt.len(), 4);
    /// # }
    /// ```
    pub fn add_symbol(&mut self, sym: impl Into<String>) -> Label {
        self.bimap.get_id_or_insert(sym.into()) as Label
    }

    pub fn add_symbols<S: Into<String>, P: IntoIterator<Item = S>>(&mut self, symbols: P) {
        for symbol in symbols.into_iter() {
            self.add_symbol(symbol.into());
        }
    }

    /// Returns the number of symbols stored in the symbol table.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let mut symt = symt!["a", "b"];
    /// assert_eq!(symt.len(), 3);
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.bimap.len()
    }

    /// Given a symbol, returns the label corresponding.
    /// If the symbol is not stored in the table then `None` is returned.
    ///
    /// # Examples
    /// ```
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let mut symt = symt!["a", "b"];
    /// let label = symt.add_symbol("c");
    /// assert_eq!(symt.get_label("c"), Some(label));
    /// assert_eq!(symt.get_label("d"), None);
    /// # }
    /// ```
    pub fn get_label(&self, sym: impl AsRef<str>) -> Option<Label> {
        self.bimap.get_id(sym).map(|it| it as Label)
    }

    /// Given a label, returns the symbol corresponding.
    /// If no there is no symbol with this label in the table then `None` is returned.
    ///
    /// # Examples
    /// ```
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let mut symt = symt!["a", "b"];
    /// let label = symt.add_symbol("c");
    /// assert_eq!(symt.get_symbol(label), Some("c"));
    /// assert_eq!(symt.get_symbol(label + 1), None);
    /// # }
    /// ```
    pub fn get_symbol(&self, label: Label) -> Option<&str> {
        self.bimap.get_string(label as usize)
    }

    /// Given a symbol, returns whether it is present in the table.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let symt = symt!["a", "b"];
    /// assert!(symt.contains_symbol("a"));
    /// # }
    /// ```
    pub fn contains_symbol(&self, sym: impl AsRef<str>) -> bool {
        self.get_label(sym).is_some()
    }

    /// Given a label, returns whether it is present in the table.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let mut symt = symt!["a", "b"];
    /// let label = symt.add_symbol("c");
    /// assert!(symt.contains_label(label));
    /// assert!(!symt.contains_label(label+1));
    /// # }
    pub fn contains_label(&self, label: Label) -> bool {
        self.get_symbol(label).is_some()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the `SymbolTable`.
    /// The collection may reserve more space to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.bimap.reserve(additional)
    }

    /// An iterator on all the labels stored in the `SymbolTable`.
    /// The iterator element is `&'a Label`.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let symt = symt!["a", "b"];
    /// let mut iterator = symt.labels();
    ///
    /// # }
    /// ```
    pub fn labels(&self) -> impl Iterator<Item = Label> {
        self.bimap.iter_ids().map(|it| it as Label)
    }

    /// An iterator on all the symbols stored in the `SymbolTable`.
    /// The iterator element is `&'a Symbol`.
    ///
    /// # Examples
    /// ```rust
    /// # #[macro_use] extern crate rustfst; fn main() {
    /// # use rustfst::SymbolTable;
    /// let symt = symt!["a", "b"];
    /// let mut iterator = symt.symbols();
    ///
    /// for symbol in symt.symbols() {
    ///     println!("Symbol : {:?}", symbol);
    /// }
    /// # }
    /// ```
    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.bimap.iter_strings()
    }

    /// An iterator on all the labels stored in the `SymbolTable`.
    /// The iterator element is `(&'a Label, &'a Symbol)`.
    pub fn iter(&self) -> impl Iterator<Item = (Label, &str)> {
        self.bimap.iter().map(|(label, sym)| (label as Label, sym))
    }

    /// Adds another SymbolTable to this table.
    pub fn add_table(&mut self, other: &SymbolTable) {
        for symbol in other.symbols() {
            self.add_symbol(symbol);
        }
    }

    pub fn write_text<P: AsRef<Path>>(&self, path_output: P) -> Result<()> {
        let buffer = File::create(path_output.as_ref())?;
        let mut writer = BufWriter::new(LineWriter::new(buffer));

        write!(writer, "{}", self)?;

        Ok(())
    }

    pub fn write<P: AsRef<Path>>(&self, path_bin_symt: P) -> Result<()> {
        let buffer = File::create(path_bin_symt.as_ref())?;
        let mut writer = BufWriter::new(LineWriter::new(buffer));

        write_bin_symt(&mut writer, self)?;

        Ok(())
    }

    /// Writes the text_fst representation of the symbol table into a String.
    pub fn text(&self) -> Result<String> {
        let buffer = Vec::<u8>::new();
        let mut writer = BufWriter::new(LineWriter::new(buffer));
        write!(writer, "{}", self)?;
        Ok(String::from_utf8(writer.into_inner()?.into_inner()?)?)
    }
}

impl<H: BuildHasher> fmt::Display for SymbolTable<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (label, symbol) in self.iter().sorted_by_key(|k| k.0) {
            writeln!(f, "{}\t{}", symbol, label)?;
        }
        Ok(())
    }
}

impl<H: BuildHasher> PartialEq for SymbolTable<H> {
    fn eq(&self, other: &Self) -> bool {
        self.bimap.eq(&other.bimap)
    }
}

/// Creates a `SymbolTable` containing the arguments.
/// ```
/// # #[macro_use] extern crate rustfst; fn main() {
/// # use rustfst::{SymbolTable, EPS_SYMBOL};
/// let symt = symt!["a", "b"];
/// assert_eq!(symt.len(), 3);
/// assert_eq!(symt.get_symbol(0).unwrap(), EPS_SYMBOL);
/// assert_eq!(symt.get_symbol(1).unwrap(), "a");
/// assert_eq!(symt.get_symbol(2).unwrap(), "b");
/// # }
/// ```
#[macro_export]
macro_rules! symt {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = SymbolTable::new();
            $(
                temp_vec.add_symbol($x.to_string());
            )*
            temp_vec
        }
    };
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BiHashMapString<H: BuildHasher = RandomState> {
    string_to_id: HashMap<String, usize, H>,
    id_to_string: Vec<String>,
}

impl<H: BuildHasher> PartialEq for BiHashMapString<H> {
    fn eq(&self, other: &Self) -> bool {
        self.string_to_id.eq(&other.string_to_id) && self.id_to_string.eq(&other.id_to_string)
    }
}

impl BiHashMapString {
    pub fn new() -> Self {
        Self {
            string_to_id: HashMap::new(),
            id_to_string: Vec::new(),
        }
    }
}

impl<H: BuildHasher> BiHashMapString<H> {
    pub fn with_hasher(hash_builder: H) -> Self {
        Self {
            string_to_id: HashMap::with_hasher(hash_builder),
            id_to_string: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.id_to_string.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.string_to_id.reserve(additional);
        self.id_to_string.reserve(additional);
    }

    pub fn get_id_or_insert(&mut self, v: impl Into<String>) -> usize {
        match self.string_to_id.entry(v.into()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let n = self.id_to_string.len();
                self.id_to_string.push(e.key().clone());
                e.insert(n);
                n
            }
        }
    }

    pub fn get_id(&self, v: impl AsRef<str>) -> Option<usize> {
        self.string_to_id.get(v.as_ref()).cloned()
    }

    pub fn get_string(&self, id: usize) -> Option<&str> {
        self.id_to_string.get(id).map(|s| s.as_str())
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = usize> {
        0..self.id_to_string.len()
    }

    pub fn iter_strings(&self) -> impl Iterator<Item = &str> {
        self.id_to_string.iter().map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.id_to_string.iter().map(|s| s.as_str()).enumerate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symt() {
        let mut symt = SymbolTable::new();
        symt.add_symbol("a");
        symt.add_symbol("b");

        assert_eq!(symt.len(), 3);

        assert!(!symt.is_empty());

        assert_eq!(symt.get_label(EPS_SYMBOL), Some(0));
        assert_eq!(symt.get_label("a"), Some(1));
        assert_eq!(symt.get_label("b"), Some(2));

        assert!(symt.contains_symbol(EPS_SYMBOL));
        assert!(symt.contains_symbol("a"));
        assert!(symt.contains_symbol("b"));
        assert!(!symt.contains_symbol("c"));

        assert_eq!(symt.get_symbol(0), Some(EPS_SYMBOL));
        assert_eq!(symt.get_symbol(1), Some("a"));
        assert_eq!(symt.get_symbol(2), Some("b"));

        assert!(symt.contains_label(0));
        assert!(symt.contains_label(1));
        assert!(symt.contains_label(2));
        assert!(!symt.contains_label(3));
    }

    #[test]
    fn test_symt_add_twice_symbol() {
        let mut symt = SymbolTable::new();
        symt.add_symbol("a");
        symt.add_symbol("a");

        assert_eq!(symt.len(), 2);
        assert_eq!(symt.get_label("a"), Some(1));
    }

    #[test]
    fn test_add_table() {
        let mut symt1 = SymbolTable::new();
        symt1.add_symbol("a");
        symt1.add_symbol("b");

        let mut symt2 = SymbolTable::new();
        symt2.add_symbol("c");
        symt2.add_symbol("b");

        symt1.add_table(&symt2);

        assert_eq!(symt1.len(), 4);
        assert_eq!(symt1.get_label(EPS_SYMBOL), Some(0));
        assert_eq!(symt1.get_label("a"), Some(1));
        assert_eq!(symt1.get_label("b"), Some(2));
        assert_eq!(symt1.get_label("c"), Some(3));
    }
}

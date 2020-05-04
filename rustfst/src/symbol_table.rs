use std::collections::hash_map::{Entry, Iter, Keys};
use std::collections::HashMap;
use std::fmt;
use std::fs::{read, File};
use std::io::{BufWriter, LineWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::parsers::bin_symt::nom_parser::{parse_symbol_table_bin, write_bin_symt};
use crate::parsers::text_symt::parsed_text_symt::ParsedTextSymt;
use crate::{Label, Symbol, EPS_SYMBOL};

/// A symbol table stores a bidirectional mapping between transition labels and "symbols" (strings).
#[derive(PartialEq, Debug, Clone, Default)]
pub struct SymbolTable {
    label_to_symbol: HashMap<Label, Symbol>,
    symbol_to_label: HashMap<Symbol, Label>,
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
            label_to_symbol: HashMap::new(),
            symbol_to_label: HashMap::new(),
        }
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
    pub fn add_symbol<S: Into<String>>(&mut self, sym: S) -> Label {
        let label = self.len();
        let sym = sym.into();

        // Only insert if
        match self.symbol_to_label.entry(sym.clone()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                e.insert(label);
                self.label_to_symbol.entry(label).or_insert(sym);
                label
            }
        }
    }

    pub fn add_symbols<S: Into<String>, P: IntoIterator<Item = S>>(&mut self, symbols: P) {
        for symbol in symbols.into_iter() {
            self.add_symbol(symbol.into());
        }
    }

    pub(crate) fn add_symbol_key<S: Into<String>>(&mut self, sym: S, key: usize) {
        let sym = sym.into();
        self.symbol_to_label.insert(sym.clone(), key);
        self.label_to_symbol.insert(key, sym);
        // TODO: Add some checks here
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
        self.label_to_symbol.len()
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
    pub fn get_label<S: Into<String>>(&self, sym: S) -> Option<Label> {
        self.symbol_to_label.get(&sym.into()).cloned()
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
        self.label_to_symbol.get(&label).map(|v| v.as_str())
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
    pub fn contains_symbol<S: Into<String>>(&self, sym: S) -> bool {
        self.get_label(sym.into()).is_some()
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
        self.label_to_symbol.reserve(additional);
        self.symbol_to_label.reserve(additional);
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
    pub fn labels(&self) -> Keys<Label, Symbol> {
        self.label_to_symbol.keys()
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
    pub fn symbols(&self) -> Keys<Symbol, Label> {
        self.symbol_to_label.keys()
    }

    /// An iterator on all the labels stored in the `SymbolTable`.
    /// The iterator element is `(&'a Label, &'a Symbol)`.
    pub fn iter(&self) -> Iter<Label, Symbol> {
        self.label_to_symbol.iter()
    }

    /// Adds another SymbolTable to this table.
    pub fn add_table(&mut self, other: &SymbolTable) {
        for symbol in other.symbols() {
            self.add_symbol(symbol.as_str());
        }
    }

    fn from_parsed_symt_text(parsed_symt_text: ParsedTextSymt) -> Result<Self> {
        let mut label_to_symbol: HashMap<Label, Symbol> = HashMap::new();
        let mut symbol_to_label: HashMap<Symbol, Label> = HashMap::new();
        for (symbol, label) in parsed_symt_text.pairs.into_iter() {
            label_to_symbol.insert(label, symbol.clone());
            symbol_to_label.insert(symbol, label);
        }

        Ok(SymbolTable {
            symbol_to_label,
            label_to_symbol,
        })
    }

    pub fn from_text_string(symt_string: &str) -> Result<Self> {
        let parsed_symt = ParsedTextSymt::from_string(symt_string)?;
        Self::from_parsed_symt_text(parsed_symt)
    }

    pub fn read_text<P: AsRef<Path>>(path_text_symt: P) -> Result<Self> {
        let parsed_symt = ParsedTextSymt::from_path(path_text_symt)?;
        Self::from_parsed_symt_text(parsed_symt)
    }

    pub fn write_text<P: AsRef<Path>>(&self, path_output: P) -> Result<()> {
        let buffer = File::create(path_output.as_ref())?;
        let mut writer = BufWriter::new(LineWriter::new(buffer));

        write!(writer, "{}", self)?;

        Ok(())
    }

    pub fn read<P: AsRef<Path>>(path_bin_symt: P) -> Result<Self> {
        let data = read(path_bin_symt.as_ref()).with_context(|| {
            format!(
                "Can't open SymbolTable binary file : {:?}",
                path_bin_symt.as_ref()
            )
        })?;

        let (_, symt) = parse_symbol_table_bin(&data)
            .map_err(|e| format_err!("Error while parsing binary SymbolTable : {:?}", e))?;

        Ok(symt)
    }

    pub fn write<P: AsRef<Path>>(&self, path_bin_symt: P) -> Result<()> {
        let buffer = File::create(path_bin_symt.as_ref())?;
        let mut writer = BufWriter::new(LineWriter::new(buffer));

        write_bin_symt(&mut writer, &self)?;

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

impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (label, symbol) in self.iter().sorted_by_key(|k| k.0) {
            writeln!(f, "{}\t{}", symbol, label)?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symt() {
        let mut symt = SymbolTable::new();
        symt.add_symbol("a");
        symt.add_symbol("b");

        assert_eq!(symt.len(), 3);

        assert_eq!(symt.is_empty(), false);

        assert_eq!(symt.get_label(EPS_SYMBOL), Some(0));
        assert_eq!(symt.get_label("a"), Some(1));
        assert_eq!(symt.get_label("b"), Some(2));

        assert_eq!(symt.contains_symbol(EPS_SYMBOL), true);
        assert_eq!(symt.contains_symbol("a"), true);
        assert_eq!(symt.contains_symbol("b"), true);
        assert_eq!(symt.contains_symbol("c"), false);

        assert_eq!(symt.get_symbol(0), Some(EPS_SYMBOL));
        assert_eq!(symt.get_symbol(1), Some("a"));
        assert_eq!(symt.get_symbol(2), Some("b"));

        assert_eq!(symt.contains_label(0), true);
        assert_eq!(symt.contains_label(1), true);
        assert_eq!(symt.contains_label(2), true);
        assert_eq!(symt.contains_label(3), false);
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

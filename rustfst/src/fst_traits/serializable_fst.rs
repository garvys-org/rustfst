use std::path::Path;

use failure::Fallible;

use crate::fst_traits::{ExpandedFst, FinalStatesIterator};
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::DrawingConfig;
use std::fs::File;
use std::io::{LineWriter, Write};

pub trait SerializableFst: ExpandedFst
where
    Self::W: SerializableSemiring,
{
    fn fst_type() -> String;

    // BINARY

    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self>;
    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Fallible<()>;

    // TEXT

    /// Turns a generic wFST format into the one of the wFST.
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst<Self::W>) -> Fallible<Self>;

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn from_text_string(fst_string: &str) -> Fallible<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Fallible<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Serializes the FST as a text file in a format compatible with OpenFST.
    fn write_text<P: AsRef<Path>>(&self, path_output: P) -> Fallible<()> {
        let buffer = File::create(path_output.as_ref())?;
        let mut line_writer = LineWriter::new(buffer);
        write_fst!(self, line_writer, true);
        Ok(())
    }

    /// Writes the text representation of the FST into a String.
    fn text(&self) -> Fallible<String> {
        let buffer = Vec::<u8>::new();
        let mut line_writer = LineWriter::new(buffer);
        write_fst!(self, line_writer, true);
        Ok(String::from_utf8(line_writer.into_inner()?)?)
    }

    /// Serializes the FST as a DOT file compatible with GraphViz binaries.
    fn draw<P: AsRef<Path>>(&self, path_output: P, config: &DrawingConfig) -> Fallible<()> {
        let buffer = File::create(path_output.as_ref())?;
        let mut f = LineWriter::new(buffer);

        if let Some(start_state) = self.start() {
            writeln!(f, "digraph FST {{")?;

            if config.vertical {
                writeln!(f, "rankdir = BT;")?;
            } else {
                writeln!(f, "rankdir = LR;")?;
            }

            writeln!(f, "size = \"{},{}\";", config.width, config.height)?;
            writeln!(f, "label = \"{}\";", config.title)?;
            writeln!(f, "center = 1;")?;

            if config.portrait {
                writeln!(f, "orientation = Portrait;")?;
            } else {
                writeln!(f, "orientation = Landscape;")?;
            }

            writeln!(f, "ranksep = {}", config.ranksep)?;
            writeln!(f, "nodesep = {}", config.nodesep)?;

            // Start state first
            draw_single_state!(self, start_state, f, config);

            for state in self.states_iter() {
                if state != start_state {
                    draw_single_state!(self, state, f, config);
                }
            }

            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

use std::path::Path;

use failure::Fallible;

use crate::fst_traits::{ExpandedFst, FinalStatesIterator};
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::{DrawingConfig, StateId};
use std::fs::File;
use std::io::{BufWriter, LineWriter, Write};

/// Trait definining the methods an Fst must implement to be serialized and deserialized.
pub trait SerializableFst: ExpandedFst
where
    Self::W: SerializableSemiring,
{
    /// String identifying the type of the FST. Will be used when serialiing and
    /// deserializing an FST in binary format.
    fn fst_type() -> String;

    // BINARY

    /// Loads an FST from a file in binary format.
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self>;
    /// Writes the FST to a file in binary format.
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
        let mut f = BufWriter::new(LineWriter::new(buffer));

        if let Some(start_state) = self.start() {
            writeln!(f, "digraph FST {{")?;

            if config.vertical {
                writeln!(f, "rankdir = BT;")?;
            } else {
                writeln!(f, "rankdir = LR;")?;
            }

            if let Some((width, height)) = config.size {
                writeln!(f, "size = \"{},{}\";", width, height)?;
            }

            writeln!(f, "label = \"{}\";", config.title)?;
            writeln!(f, "center = 1;")?;

            if config.portrait {
                writeln!(f, "orientation = Portrait;")?;
            } else {
                writeln!(f, "orientation = Landscape;")?;
            }

            if let Some(ranksep) = config.ranksep {
                writeln!(f, "ranksep = {}", ranksep)?;
            }

            if let Some(nodesep) = config.nodesep {
                writeln!(f, "nodesep = {}", nodesep)?;
            }

            // Start state first
            draw_single_fst_state(self, &mut f, start_state, config)?;

            for state in self.states_iter() {
                if state != start_state {
                    draw_single_fst_state(self, &mut f, state, config)?;
                }
            }

            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

fn draw_single_fst_state<F: SerializableFst, W: Write>(
    fst: &F,
    writer: &mut W,
    state_id: StateId,
    config: &DrawingConfig,
) -> Fallible<()>
where
    F::W: SerializableSemiring,
{
    let opt_isymt = fst.input_symbols();
    let opt_osymt = fst.output_symbols();

    write!(writer, "{}", state_id)?;
    write!(writer, " [label = \"{}", state_id)?;
    if let Some(final_weight) = fst.final_weight(state_id)? {
        if config.print_weight && (config.show_weight_one || !final_weight.is_one()) {
            write!(writer, "/{}", final_weight)?;
        }
        write!(writer, "\", shape = doublecircle,")?;
    } else {
        write!(writer, "\", shape = circle,")?;
    }

    if fst.is_start(state_id) {
        write!(writer, " style = bold,")?;
    } else {
        write!(writer, " style = solid,")?;
    }

    writeln!(writer, " fontsize = {}]", config.fontsize)?;

    for arc in fst.arcs_iter(state_id).unwrap() {
        write!(writer, "\t{} -> {}", state_id, arc.nextstate)?;

        let ilabel = opt_isymt.clone().map_or_else(
            || Ok(format!("{}", arc.ilabel)),
            |symt| {
                symt.get_symbol(arc.ilabel)
                    .map(|v| v.to_string())
                    .ok_or_else(|| format_err!("Missing {} in input SymbolTable", arc.ilabel))
            },
        )?;

        let olabel = opt_osymt.clone().map_or_else(
            || Ok(format!("{}", arc.ilabel)),
            |symt| {
                symt.get_symbol(arc.olabel)
                    .map(|v| v.to_string())
                    .ok_or_else(|| format_err!("Missing {} in output SymbolTable", arc.olabel))
            },
        )?;

        write!(writer, " [label = \"{}", ilabel)?;
        if !config.acceptor {
            write!(writer, ":{}", olabel)?;
        }

        if config.print_weight && (config.show_weight_one || !arc.weight.is_one()) {
            write!(writer, "/{}", arc.weight)?;
        }
        writeln!(writer, "\", fontsize = {}];", config.fontsize)?;
    }

    Ok(())
}

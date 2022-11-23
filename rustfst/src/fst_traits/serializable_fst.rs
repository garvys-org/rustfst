use std::fs::File;
use std::io::{BufWriter, LineWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use unsafe_unwrap::UnsafeUnwrap;

use crate::fst_traits::ExpandedFst;
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::SerializableSemiring;
use crate::Trs;
use crate::{DrawingConfig, StateId};

/// Trait definining the methods an Fst must implement to be serialized and deserialized.
pub trait SerializableFst<W: SerializableSemiring>: ExpandedFst<W> {
    /// String identifying the type of the FST. Will be used when serialiing and
    /// deserializing an FST in binary format.
    fn fst_type() -> String;

    // BINARY

    /// Loads an FST from the binary format data in a `Read`.
    fn load(input: &[u8]) -> Result<Self>;

    /// Store the FST in binary format to a `Write`.
    fn store<O: Write>(&self, output: O) -> Result<()>;

    /// Loads an FST from a file in binary format.
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Result<Self> {
        let data: Vec<u8> = std::fs::read(path_bin_fst.as_ref()).with_context(|| {
            format!(
                "Can't open {}Fst binary file : {:?}",
                Self::fst_type(),
                path_bin_fst.as_ref()
            )
        })?;
        Self::load(&data)
    }
    /// Writes the FST to a file in binary format.
    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Result<()> {
        let output = std::fs::File::create(path_bin_fst.as_ref()).with_context(|| {
            format!(
                "Cannot create {}Fst binary file : {:?}",
                Self::fst_type(),
                path_bin_fst.as_ref(),
            )
        })?;
        self.store(BufWriter::new(output))
    }

    // TEXT

    /// Turns a generic wFST format into the one of the wFST.
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst<W>) -> Result<Self>;

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn from_text_string(fst_string: &str) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Serializes the FST as a text file in a format compatible with OpenFST.
    fn write_text<P: AsRef<Path>>(&self, path_output: P) -> Result<()> {
        let buffer = File::create(path_output.as_ref())?;
        let mut line_writer = LineWriter::new(buffer);
        write_fst!(self, line_writer, true, false);
        Ok(())
    }

    /// Writes the text representation of the FST into a String.
    fn text(&self) -> Result<String> {
        let buffer = Vec::<u8>::new();
        let mut line_writer = LineWriter::new(buffer);
        write_fst!(self, line_writer, true, false);
        Ok(String::from_utf8(line_writer.into_inner()?)?)
    }

    /// Serializes the FST as a DOT file compatible with GraphViz binaries.
    fn draw<P: AsRef<Path>>(&self, path_output: P, config: &DrawingConfig) -> Result<()> {
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

fn draw_single_fst_state<S: SerializableSemiring, F: SerializableFst<S>, W: Write>(
    fst: &F,
    writer: &mut W,
    state_id: StateId,
    config: &DrawingConfig,
) -> Result<()> {
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

    for tr in fst.get_trs(state_id).unwrap().trs() {
        write!(writer, "\t{} -> {}", state_id, tr.nextstate)?;

        let ilabel = opt_isymt.map_or_else(
            || Ok(format!("{}", tr.ilabel)),
            |symt| {
                symt.get_symbol(tr.ilabel)
                    .map(|v| v.to_string())
                    .ok_or_else(|| format_err!("Missing {} in input SymbolTable", tr.ilabel))
            },
        )?;

        let olabel = opt_osymt.map_or_else(
            || Ok(format!("{}", tr.olabel)),
            |symt| {
                symt.get_symbol(tr.olabel)
                    .map(|v| v.to_string())
                    .ok_or_else(|| format_err!("Missing {} in output SymbolTable", tr.olabel))
            },
        )?;

        write!(writer, " [label = \"{}", ilabel)?;
        if !config.acceptor {
            write!(writer, ":{}", olabel)?;
        }

        if config.print_weight && (config.show_weight_one || !tr.weight.is_one()) {
            write!(writer, "/{}", tr.weight)?;
        }
        writeln!(writer, "\", fontsize = {}];", config.fontsize)?;
    }

    Ok(())
}

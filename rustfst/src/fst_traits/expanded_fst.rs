use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::Path;

use failure::Fallible;

use crate::fst_properties::compute_fst_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::final_states_iterator::FinalStatesIterator;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::DrawingConfig;

/// Trait defining the necessary methods that should implement an ExpandedFST e.g
/// a FST where all the states are already computed and not computed on the fly.
pub trait ExpandedFst: Fst {
    /// Returns the number of states that contains the FST. They are all counted even if some states
    /// are not on a successful path (doesn't perform triming).
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.num_states(), 0);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 1);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 2);
    ///
    /// ```
    fn num_states(&self) -> usize;

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

    /// Compute the properties verified by the Fst.
    fn properties(&self) -> Fallible<FstProperties> {
        compute_fst_properties(self)
    }
}

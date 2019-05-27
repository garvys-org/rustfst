use std::fs::read;
use std::path::Path;

use failure::Fallible;
use nom::{le_f32, le_i32, le_i64, le_u64};

use crate::fst_impls::vector::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{BinaryParser, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

// Identifies stream data as an FST (and its endianity).
static FST_MAGIC_NUMBER: i32 = 2125659606;
static MIN_FILE_VERSION: i32 = 2;
static NO_STATE_ID: i32 = -1;

#[derive(Debug)]
struct FstHeader {
    magic_number: i32,
    fst_type: OpenFstString,
    arc_type: OpenFstString,
    version: i32,
    flags: i32,
    properties: u64,
    start: i64,
    num_states: i64,
    num_arcs: i64,
}

#[derive(Debug)]
struct ParsedFst {
    header: FstHeader,
    states: Vec<FstState>,
}

#[derive(Debug)]
struct OpenFstString {
    n: i32,
    s: String,
}

#[derive(Debug)]
struct FstState {
    final_weight: f32,
    num_arcs: i64,
    arcs: Vec<Transition>,
}

#[derive(Debug, PartialEq)]
struct Transition {
    ilabel: i32,
    olabel: i32,
    weight: f32,
    nextstate: i32,
}

named!(parse_kaldi_string <&[u8], OpenFstString>, do_parse!(
    n: le_i32 >>
    s: take!(n as usize) >>
    (OpenFstString{n, s: String::from_utf8(s.to_vec()).unwrap()}))
);

named!(parse_fst_header <&[u8], FstHeader>, do_parse!(
    magic_number: verify!(le_i32, |v: i32| v == FST_MAGIC_NUMBER) >>
    fst_type: parse_kaldi_string >>
    arc_type: parse_kaldi_string >>
    version: verify!(le_i32, |v: i32| v >= MIN_FILE_VERSION) >>
    flags: le_i32 >>
    properties: le_u64 >>
    start: le_i64 >>
    num_states: le_i64 >>
    num_arcs: le_i64 >>
    (FstHeader {magic_number, fst_type, arc_type, version, flags, properties, start, num_states, num_arcs}))
);

named!(parse_fst_arc <&[u8], Transition>, do_parse!(
    ilabel: le_i32 >>
    olabel: le_i32 >>
    weight: le_f32 >>
    nextstate: le_i32 >>
    (Transition{ilabel, olabel, weight, nextstate})
));

named!(parse_fst_state <&[u8], FstState>, do_parse!(
    final_weight: le_f32 >>
    num_arcs: le_i64 >>
    arcs: count!(parse_fst_arc, num_arcs as usize) >>
    (FstState{final_weight, num_arcs, arcs})
));

named!(parse_fst <&[u8], ParsedFst>, do_parse!(
    header: parse_fst_header >>
    states: count!(parse_fst_state, header.num_states as usize) >>
    (ParsedFst {header, states}))
);

named!(complete_parse_fst <&[u8], ParsedFst>, complete!(parse_fst));

impl<W: 'static + Semiring<Type = f32>> BinaryParser for VectorFst<W> {
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self> {
        let data = read(path_bin_fst.as_ref())?;
        let (_, parsed_fst) = complete_parse_fst(&data)
            .map_err(|_| format_err!("Error while parsing binary VectorFst"))?;

        let start_state = if parsed_fst.header.start == NO_STATE_ID as i64 {
            None
        } else {
            Some(parsed_fst.header.start as StateId)
        };

        let num_states = if parsed_fst.header.num_states == NO_STATE_ID as i64 {
            0
        } else {
            parsed_fst.header.num_states as usize
        };

        let states = vec![VectorFstState::<W>::default(); num_states];

        let mut fst = VectorFst {
            states,
            start_state,
        };

        let zero_weight = W::zero().value();

        for state in 0..num_states {
            if parsed_fst.states[state].final_weight != zero_weight {
                let final_weight = W::new(parsed_fst.states[state].final_weight);
                fst.set_final(state, final_weight)?;
            };

            for transition in parsed_fst.states[state].arcs.iter() {
                let weight = W::new(transition.weight);
                let arc = Arc::new(
                    transition.ilabel as usize,
                    transition.olabel as usize,
                    weight,
                    transition.nextstate as StateId,
                );
                fst.add_arc(state, arc)?;
            }
        }

        Ok(fst)
    }
}

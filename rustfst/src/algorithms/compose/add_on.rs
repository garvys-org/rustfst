use std::fmt::Debug;
use std::io::Write;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use nom::combinator::verify;
use nom::IResult;

use crate::{NomCustomError, StateId, SymbolTable, Tr};
use crate::fst_properties::FstProperties;
use crate::fst_properties::properties::EXPANDED;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, SerializableFst, StateIterator};
use crate::parsers::{parse_bin_bool, parse_bin_i32, write_bin_bool, write_bin_i32};
use crate::parsers::bin_fst::fst_header::{FST_MAGIC_NUMBER, FstFlags, FstHeader, OpenFstString};
use crate::prelude::{SerializableSemiring, SerializeBinary};
use crate::semirings::Semiring;

/// Adds an object of type T to an FST.
/// The resulting type is a new FST implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct FstAddOn<W, F, T>
    where
        W: Semiring,
        F: Fst<W>
{
    pub(crate) fst: F,
    pub(crate) add_on: T,
    w: PhantomData<W>,
    fst_type: String
}

impl<W: Semiring, F: Fst<W>, T> FstAddOn<W, F, T> {
    pub fn new(fst: F, add_on: T, fst_type: String) -> Self {
        Self { fst, add_on, w: PhantomData, fst_type }
    }

    pub fn fst(&self) -> &F {
        &self.fst
    }

    pub fn fst_mut(&mut self) -> &mut F {
        &mut self.fst
    }

    pub fn add_on(&self) -> &T {
        &self.add_on
    }
}

impl<W: Semiring, F: Fst<W>, T> CoreFst<W> for FstAddOn<W, F, T> {
    type TRS = F::TRS;

    fn start(&self) -> Option<StateId> {
        self.fst.start()
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        self.fst.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W> {
        self.fst.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        self.fst.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.fst.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS> {
        self.fst.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS {
        self.fst.get_trs_unchecked(state_id)
    }

    fn properties(&self) -> FstProperties {
        self.fst.properties()
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        self.fst.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        self.fst.num_output_epsilons(state)
    }
}

impl<'a, W: Semiring, F: Fst<W>, T> StateIterator<'a> for FstAddOn<W, F, T> {
    type Iter = <F as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.fst.states_iter()
    }
}

impl<'a, W, F, T> FstIterator<'a, W> for FstAddOn<W, F, T>
where
    W: Semiring + 'a,
    F: Fst<W>,
{
    type FstIter = <F as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.fst.fst_iter()
    }
}

impl<W, F, T: Debug> Fst<W> for FstAddOn<W, F, T>
where
    W: Semiring,
    F: Fst<W>,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst.take_output_symbols()
    }
}

impl<W, F, T> ExpandedFst<W> for FstAddOn<W, F, T>
where
    W: Semiring,
    F: ExpandedFst<W>,
    T: Debug + Clone + PartialEq,
{
    fn num_states(&self) -> usize {
        self.fst.num_states()
    }
}

impl<W, F, T> FstIntoIterator<W> for FstAddOn<W, F, T>
where
    W: Semiring,
    F: FstIntoIterator<W> + Fst<W> ,
    T: Debug,
{
    type TrsIter = F::TrsIter;
    type FstIter = <F as FstIntoIterator<W>>::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        self.fst.fst_into_iter()
    }
}

static ADD_ON_MAGIC_NUMBER: i32 = 446681434;
static ADD_ON_MIN_FILE_VERSION: i32 = 1;
static ADD_ON_FILE_VERSION: i32 = 1;

impl<W, F, AO1, AO2> SerializeBinary for FstAddOn<W, F, (Option<Arc<AO1>>, Option<Arc<AO2>>)>
where
    W: SerializableSemiring,
    F: SerializableFst<W>,
    AO1: SerializeBinary + Debug + Clone + PartialEq,
    AO2: SerializeBinary + Debug + Clone + PartialEq,
{
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {

        let (i, hdr) = FstHeader::parse(
            i,
            ADD_ON_MIN_FILE_VERSION,
            Option::<&str>::None,
            Tr::<W>::tr_type(),
        )?;

        let (i, _) = verify(parse_bin_i32, |v: &i32| *v == ADD_ON_MAGIC_NUMBER)(i)?;
        let (i, fst) = F::parse_binary(i)?;

        let (i, _have_addon) = verify(parse_bin_bool, |v| *v)(i)?;

        let (i, have_addon1) = parse_bin_bool(i)?;
        let (i, add_on_1) = if have_addon1 {
            let (s, a) = AO1::parse_binary(i)?;
            (s, Some(a))
        } else {
            (i, None)
        };
        let (i, have_addon2) = parse_bin_bool(i)?;
        let (i, add_on_2) = if have_addon2 {
            let (s, a) = AO2::parse_binary(i)?;
            (s, Some(a))
        } else {
            (i, None)
        };

        let add_on = (add_on_1.map(Arc::new), add_on_2.map(Arc::new));
        let fst_add_on = FstAddOn::new(fst, add_on, hdr.fst_type.s().clone());
        Ok((i, fst_add_on))
    }

    fn write_binary<WB: Write>(&self, writer: &mut WB) -> Result<()> {
        let hdr = FstHeader {
            magic_number: FST_MAGIC_NUMBER,
            fst_type: OpenFstString::new(&self.fst_type),
            tr_type: OpenFstString::new(Tr::<W>::tr_type()),
            version: ADD_ON_FILE_VERSION,
            flags: FstFlags::empty(),
            properties: self.properties().bits() | EXPANDED,
            start: -1,
            num_states: 0,
            num_trs: 0,
            isymt: None,
            osymt: None,
        };
        hdr.write(writer)?;
        write_bin_i32(writer, ADD_ON_MAGIC_NUMBER)?;
        self.fst.write_binary(writer)?;
        write_bin_bool(writer, true)?;
        if let Some(add_on) = self.add_on.0.as_ref() {
            write_bin_bool(writer, true)?;
            add_on.write_binary(writer)?;
        } else {
            write_bin_bool(writer, false)?;
        }
        if let Some(add_on) = self.add_on.1.as_ref() {
            write_bin_bool(writer, true)?;
            add_on.write_binary(writer)?;
        } else {
            write_bin_bool(writer, false)?;
        }
        Ok(())
    }
}

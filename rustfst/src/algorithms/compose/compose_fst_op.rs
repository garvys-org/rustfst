use std::borrow::Borrow;
use std::fmt::Debug;
use std::fs::{read, File};
use std::hash::Hash;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::matchers::{IterItemMatcher, MatcherFlags};
use crate::algorithms::compose::matchers::{MatchType, Matcher, REQUIRE_PRIORITY};
use crate::algorithms::compose::{ComposeFstOpOptions, ComposeStateTuple};
use crate::algorithms::lazy::{AccessibleOpState, FstOp, SerializableOpState, StateTable};
use crate::fst_properties::mutable_properties::compose_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::parsers::SerializeBinary;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs, TrsVec, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone)]
pub struct ComposeFstOpState<T: Hash + Eq + Clone> {
    state_table: StateTable<T>,
}

impl<T: Hash + Eq + Clone> Default for ComposeFstOpState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq + Clone> ComposeFstOpState<T> {
    pub fn new() -> Self {
        ComposeFstOpState {
            state_table: StateTable::<T>::new(),
        }
    }
}

impl<T: Hash + Eq + Clone + SerializeBinary> SerializableOpState for ComposeFstOpState<T> {
    /// Loads a ComposeFstOpState from a file in binary format.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = read(path.as_ref())
            .with_context(|| format!("Can't open file : {:?}", path.as_ref()))?;

        // Parse StateTable
        let (_, state_table) = StateTable::<T>::parse_binary(&data)
            .map_err(|e| format_err!("Error while parsing binary StateTable : {:?}", e))?;

        Ok(Self { state_table })
    }

    /// Writes a ComposeFstOpState to a file in binary format.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = BufWriter::new(File::create(path)?);

        // Write StateTable
        self.state_table.write_binary(&mut file)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
{
    compose_filter_builder: CFB,
    compose_state: ComposeFstOpState<
        ComposeStateTuple<<CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS>,
    >,
    match_type: MatchType,
    properties: FstProperties,
    fst1: B1,
    fst2: B2,
}

impl<W, F1, F2, B1, B2, M1, M2, CFB> Clone for ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
{
    fn clone(&self) -> Self {
        Self {
            compose_filter_builder: self.compose_filter_builder.clone(),
            compose_state: self.compose_state.clone(),
            match_type: self.match_type,
            properties: self.properties,
            fst1: self.fst1.clone(),
            fst2: self.fst2.clone(),
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB> ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
{
    // Compose specifying two matcher types Matcher1 and Matcher2. Requires input
    // FST (of the same Tr type, but o.w. arbitrary) match the corresponding
    // matcher FST types). Recommended only for advanced use in demanding or
    // specialized applications due to potential code bloat and matcher
    // incompatibilities.
    // fn new2(fst1: &'fst F1, fst2: &'fst F2) -> Result<Self> {
    //     unimplemented!()
    // }

    pub fn new(
        fst1: B1,
        fst2: B2,
        opts: ComposeFstOpOptions<
            M1,
            M2,
            CFB,
            ComposeFstOpState<
                ComposeStateTuple<
                    <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS,
                >,
            >,
        >,
    ) -> Result<Self> {
        let matcher1 = opts.matcher1;
        let matcher2 = opts.matcher2;
        let compose_filter_builder = opts.filter_builder.unwrap_or_else(|| {
            ComposeFilterBuilder::new(fst1.clone(), fst2.clone(), matcher1, matcher2).unwrap()
        });
        let compose_filter = compose_filter_builder.build()?;
        let match_type = Self::match_type(compose_filter.matcher1(), compose_filter.matcher2())?;

        let fprops1 = fst1.borrow().properties();
        let fprops2 = fst2.borrow().properties();
        let cprops = compose_properties(fprops1, fprops2);
        let properties = compose_filter.properties(cprops);

        Ok(Self {
            compose_filter_builder,
            compose_state: opts.op_state.unwrap_or_default(),
            match_type,
            properties,
            fst1,
            fst2,
        })
    }

    fn match_type(matcher1: &CFB::IM1, matcher2: &CFB::IM2) -> Result<MatchType> {
        if matcher1.flags().contains(MatcherFlags::REQUIRE_MATCH)
            && matcher1.match_type(true)? != MatchType::MatchOutput
        {
            bail!("ComposeFst: 1st argument cannot perform required matching (sort?)")
        }
        if matcher2.flags().contains(MatcherFlags::REQUIRE_MATCH)
            && matcher2.match_type(true)? != MatchType::MatchInput
        {
            bail!("ComposeFst: 2nd argument cannot perform required matching (sort?)")
        }

        let type1 = matcher1.match_type(false)?;
        let type2 = matcher2.match_type(false)?;
        let mt = if type1 == MatchType::MatchOutput && type2 == MatchType::MatchInput {
            MatchType::MatchBoth
        } else if type1 == MatchType::MatchOutput {
            MatchType::MatchOutput
        } else if type2 == MatchType::MatchInput {
            MatchType::MatchInput
        } else if matcher1.match_type(true)? == MatchType::MatchOutput {
            MatchType::MatchOutput
        } else if matcher2.match_type(true)? == MatchType::MatchInput {
            MatchType::MatchInput
        } else {
            bail!("ComposeFst: 1st argument cannot match on output labels and 2nd argument cannot match on input labels (sort?).")
        };
        Ok(mt)
    }

    fn match_input(&self, s1: StateId, s2: StateId, compose_filter: &CFB::CF) -> Result<bool> {
        match self.match_type {
            MatchType::MatchInput => Ok(true),
            MatchType::MatchOutput => Ok(false),
            _ => {
                // Match both
                let priority1 = compose_filter.matcher1().priority(s1)?;
                let priority2 = compose_filter.matcher2().priority(s2)?;
                if priority1 == REQUIRE_PRIORITY && priority2 == REQUIRE_PRIORITY {
                    bail!("Both sides can't require match")
                }
                if priority1 == REQUIRE_PRIORITY {
                    return Ok(false);
                }
                if priority2 == REQUIRE_PRIORITY {
                    return Ok(true);
                }
                Ok(priority1 <= priority2)
            }
        }
    }

    fn ordered_expand(
        &self,
        sa: StateId,
        sb: StateId,
        match_input: bool,
        mut compose_filter: CFB::CF,
        selector: Selector,
    ) -> Result<TrsVec<W>> {
        let tr_loop = if match_input {
            Tr::new(EPS_LABEL, NO_LABEL, W::one(), sb)
        } else {
            Tr::new(NO_LABEL, EPS_LABEL, W::one(), sb)
        };
        let mut trs = vec![];

        match selector {
            Selector::Fst1Matcher2 => {
                self.match_tr(
                    sa,
                    &tr_loop,
                    match_input,
                    &mut compose_filter,
                    selector,
                    &mut trs,
                )?;
                for tr in self.fst1.borrow().get_trs(sb)?.trs() {
                    self.match_tr(sa, tr, match_input, &mut compose_filter, selector, &mut trs)?;
                }
            }
            Selector::Fst2Matcher1 => {
                self.match_tr(
                    sa,
                    &tr_loop,
                    match_input,
                    &mut compose_filter,
                    selector,
                    &mut trs,
                )?;
                for tr in self.fst2.borrow().get_trs(sb)?.trs() {
                    self.match_tr(sa, tr, match_input, &mut compose_filter, selector, &mut trs)?;
                }
            }
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn add_tr(
        &self,
        mut arc1: Tr<W>,
        arc2: Tr<W>,
        fs: <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS,
    ) -> Result<Tr<W>> {
        let tuple = ComposeStateTuple {
            fs,
            s1: arc1.nextstate,
            s2: arc2.nextstate,
        };
        arc1.weight.times_assign(arc2.weight)?;
        Ok(Tr::new(
            arc1.ilabel,
            arc2.olabel,
            arc1.weight,
            self.compose_state.state_table.find_id(tuple),
        ))
    }

    fn match_tr_selected(
        &self,
        sa: StateId,
        tr: &Tr<W>,
        match_input: bool,
        compose_filter: &mut CFB::CF,
        it: impl Iterator<Item = IterItemMatcher<W>>,
        trs: &mut Vec<Tr<W>>,
    ) -> Result<()> {
        let match_type = if match_input {
            MatchType::MatchInput
        } else {
            MatchType::MatchOutput
        };
        for arca in it {
            let mut arca = arca.into_tr(sa, match_type)?;
            let mut arcb = tr.clone();
            if match_input {
                let fs = compose_filter.filter_tr(&mut arcb, &mut arca)?;
                if fs
                    != <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS::new_no_state()
                {
                    trs.push(self.add_tr(arcb, arca, fs)?);
                }
            } else {
                let fs = compose_filter.filter_tr(&mut arca, &mut arcb)?;

                if fs
                    != <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS::new_no_state()
                {
                    trs.push(self.add_tr(arca, arcb, fs)?);
                }
            }
        }
        Ok(())
    }

    fn match_tr(
        &self,
        sa: StateId,
        tr: &Tr<W>,
        match_input: bool,
        compose_filter: &mut CFB::CF,
        selector: Selector,
        trs: &mut Vec<Tr<W>>,
    ) -> Result<()> {
        let label = if match_input { tr.olabel } else { tr.ilabel };

        match selector {
            Selector::Fst2Matcher1 => self.match_tr_selected(
                sa,
                tr,
                match_input,
                compose_filter,
                compose_filter.matcher1().iter(sa, label)?,
                trs,
            ),
            Selector::Fst1Matcher2 => self.match_tr_selected(
                sa,
                tr,
                match_input,
                compose_filter,
                compose_filter.matcher2().iter(sa, label)?,
                trs,
            ),
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB> AccessibleOpState
    for ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS: SerializeBinary,
{
    type FstOpState = ComposeFstOpState<
        ComposeStateTuple<<CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS>,
    >;

    fn get_op_state(&self) -> &Self::FstOpState {
        &self.compose_state
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB> FstOp<W> for ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
{
    fn compute_start(&self) -> Result<Option<StateId>> {
        let compose_filter = self.compose_filter_builder.build()?;
        let s1 = self.fst1.borrow().start();
        if s1.is_none() {
            return Ok(None);
        }
        let s1 = s1.unwrap();
        let s2 = self.fst2.borrow().start();
        if s2.is_none() {
            return Ok(None);
        }
        let s2 = s2.unwrap();
        let fs = compose_filter.start();
        let tuple = ComposeStateTuple { fs, s1, s2 };
        Ok(Some(self.compose_state.state_table.find_id(tuple)))
    }

    fn compute_trs(&self, state: StateId) -> Result<TrsVec<W>> {
        let tuple = self.compose_state.state_table.find_tuple(state);
        let s1 = tuple.s1;
        let s2 = tuple.s2;

        let mut compose_filter = self.compose_filter_builder.build()?;
        compose_filter.set_state(s1, s2, &tuple.fs)?;
        if self.match_input(s1, s2, &compose_filter)? {
            self.ordered_expand(s2, s1, true, compose_filter, Selector::Fst1Matcher2)
        } else {
            self.ordered_expand(s1, s2, false, compose_filter, Selector::Fst2Matcher1)
        }
    }

    fn compute_final_weight(&self, state: StateId) -> Result<Option<W>> {
        let tuple = self.compose_state.state_table.find_tuple(state);

        // Construct a new ComposeFilter each time to avoid mutating the internal state.
        let mut compose_filter = self.compose_filter_builder.build()?;

        let s1 = tuple.s1;
        let final1 = compose_filter.matcher1().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let mut final1 = final1.unwrap();

        let s2 = tuple.s2;
        let final2 = compose_filter.matcher2().final_weight(s2)?;
        if final2.is_none() {
            return Ok(None);
        }
        let mut final2 = final2.unwrap();

        compose_filter.set_state(s1, s2, &tuple.fs)?;
        compose_filter.filter_final(&mut final1, &mut final2)?;

        final1.times_assign(&final2)?;
        if final1.is_zero() {
            Ok(None)
        } else {
            Ok(Some(final1))
        }
    }

    fn properties(&self) -> FstProperties {
        self.properties
    }
}

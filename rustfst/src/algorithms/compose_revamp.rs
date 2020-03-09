use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use failure::Fallible;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::compose_filters::{
    AltSequenceComposeFilter, ComposeFilter, MatchComposeFilter, NoMatchComposeFilter,
    NullComposeFilter, SequenceComposeFilter, TrivialComposeFilter,
};
use crate::algorithms::dynamic_fst::DynamicFst;
use crate::algorithms::matchers::{MatchType, SortedMatcher};
use crate::algorithms::matchers::{Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::{Arc, StateId, EPS_LABEL, NO_LABEL};

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct ComposeStateTuple<FS> {
    fs: FS,
    s1: StateId,
    s2: StateId,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComposeFstImpl<
    'fst,
    F1: Fst + 'fst,
    F2: Fst<W = F1::W> + 'fst,
    CF: ComposeFilter<'fst, F1, F2>,
> {
    fst1: &'fst F1,
    fst2: &'fst F2,
    matcher1: Rc<RefCell<CF::M1>>,
    matcher2: Rc<RefCell<CF::M2>>,
    compose_filter: CF,
    cache_impl: CacheImpl<F1::W>,
    state_table: StateTable<ComposeStateTuple<CF::FS>>,
    match_type: MatchType,
}

impl<'fst, F1: Fst + 'fst, F2: Fst<W = F1::W> + 'fst, CF: ComposeFilter<'fst, F1, F2>>
    ComposeFstImpl<'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self> {
        let compose_filter = CF::new(fst1, fst2)?;
        let matcher1 = compose_filter.matcher1();
        let matcher2 = compose_filter.matcher2();
        Ok(Self {
            fst1,
            fst2,
            compose_filter,
            cache_impl: CacheImpl::new(),
            state_table: StateTable::new(),
            match_type: Self::match_type(&matcher1, &matcher2)?,
            matcher1,
            matcher2,
        })
    }

    fn match_type(
        matcher1: &Rc<RefCell<CF::M1>>,
        matcher2: &Rc<RefCell<CF::M2>>,
    ) -> Fallible<MatchType> {
        if matcher1
            .borrow()
            .flags()
            .contains(MatcherFlags::REQUIRE_MATCH)
            && matcher1.borrow().match_type() == MatchType::MatchOutput
        {
            bail!("ComposeFst: 1st argument cannot perform required matching (sort?)")
        }
        if matcher2
            .borrow()
            .flags()
            .contains(MatcherFlags::REQUIRE_MATCH)
            && matcher2.borrow().match_type() == MatchType::MatchInput
        {
            bail!("ComposeFst: 2nd argument cannot perform required matching (sort?)")
        }

        let type1 = matcher1.borrow().match_type();
        let type2 = matcher2.borrow().match_type();
        let mt = if type1 == MatchType::MatchOutput && type2 == MatchType::MatchInput {
            MatchType::MatchBoth
        } else if type1 == MatchType::MatchOutput {
            MatchType::MatchOutput
        } else if type2 == MatchType::MatchInput {
            MatchType::MatchInput
        } else {
            bail!("ComposeFst: 1st argument cannot match on output labels and 2nd argument cannot match on input labels (sort?).")
        };
        Ok(mt)
    }

    fn match_input(&self, _s1: StateId, _s2: StateId) -> bool {
        match self.match_type {
            MatchType::MatchInput => true,
            MatchType::MatchOutput => false,
            _ => unimplemented!(),
        }
    }

    fn ordered_expand<'b, FA: Fst<W = F1::W> + 'b, FB: Fst<W = FA::W> + 'b, M: Matcher<'b, FA>>(
        &mut self,
        s: StateId,
        sa: StateId,
        fstb: &FB,
        sb: StateId,
        matchera: Rc<RefCell<M>>,
        match_input: bool,
    ) -> Fallible<()> {
        let arc_loop = if match_input {
            Arc::new(EPS_LABEL, NO_LABEL, FA::W::one(), sb)
        } else {
            Arc::new(NO_LABEL, EPS_LABEL, FA::W::one(), sb)
        };
        self.match_arc(s, sa, Rc::clone(&matchera), &arc_loop, match_input)?;
        for arc in fstb.arcs_iter(sb)? {
            self.match_arc(s, sa, Rc::clone(&matchera), arc, match_input)?;
        }
        Ok(())
    }

    fn add_arc(
        &mut self,
        s: StateId,
        mut arc1: Arc<F1::W>,
        arc2: Arc<F1::W>,
        fs: CF::FS,
    ) -> Fallible<()> {
        let tuple = ComposeStateTuple {
            fs,
            s1: arc1.nextstate,
            s2: arc2.nextstate,
        };
        arc1.weight.times_assign(arc2.weight)?;
        self.cache_impl.push_arc(
            s,
            Arc::new(
                arc1.ilabel,
                arc2.olabel,
                arc1.weight,
                self.state_table.find_id(tuple),
            ),
        )?;

        Ok(())
    }

    fn match_arc<'b, F: Fst<W = F1::W> + 'b, M: Matcher<'b, F>>(
        &mut self,
        s: StateId,
        sa: StateId,
        matchera: Rc<RefCell<M>>,
        arc: &Arc<F::W>,
        match_input: bool,
    ) -> Fallible<()> {
        let label = if match_input { arc.olabel } else { arc.ilabel };

        for arca in matchera.borrow().iter(sa, label)? {
            let mut arca = arca.into_arc(
                sa,
                if match_input {
                    MatchType::MatchInput
                } else {
                    MatchType::MatchOutput
                },
            )?;
            let mut arcb = arc.clone();
            if match_input {
                let opt_fs = self.compose_filter.filter_arc(&mut arcb, &mut arca);
                if let Some(fs) = opt_fs {
                    self.add_arc(s, arcb, arca, fs)?;
                }
            } else {
                let opt_fs = self.compose_filter.filter_arc(&mut arca, &mut arcb);
                if let Some(fs) = opt_fs {
                    self.add_arc(s, arca, arcb, fs)?;
                }
            }
        }

        Ok(())
    }
}

impl<'fst, F1: Fst + 'fst, F2: Fst<W = F1::W> + 'fst, CF: ComposeFilter<'fst, F1, F2>> FstImpl
    for ComposeFstImpl<'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    type W = F1::W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        let tuple = self.state_table.find_tuple(state);
        let s1 = tuple.s1;
        let s2 = tuple.s2;
        self.compose_filter.set_state(s1, s2, &tuple.fs);
        drop(tuple);
        if self.match_input(s1, s2) {
            self.ordered_expand(state, s2, self.fst1, s1, Rc::clone(&self.matcher2), true)?;
        } else {
            self.ordered_expand(state, s1, self.fst2, s2, Rc::clone(&self.matcher1), false)?;
        }
        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<StateId>> {
        let s1 = self.fst1.start();
        if s1.is_none() {
            return Ok(None);
        }
        let s1 = s1.unwrap();
        let s2 = self.fst2.start();
        if s2.is_none() {
            return Ok(None);
        }
        let s2 = s2.unwrap();
        let fs = self.compose_filter.start();
        let tuple = ComposeStateTuple { s1, s2, fs };
        Ok(Some(self.state_table.find_id(tuple)))
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<Self::W>> {
        let tuple = self.state_table.find_tuple(state);

        let s1 = tuple.s1;
        let final1 = self.compose_filter.matcher1().borrow().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let mut final1 = final1.unwrap().clone();

        let s2 = tuple.s2;
        let final2 = self.compose_filter.matcher2().borrow().final_weight(s2)?;
        if final2.is_none() {
            return Ok(None);
        }
        let mut final2 = final2.unwrap().clone();

        self.compose_filter.set_state(s1, s2, &tuple.fs);
        self.compose_filter.filter_final(&mut final1, &mut final2);

        final1.times_assign(&final2)?;
        Ok(Some(final1))
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum ComposeFilterEnum {
    AutoFilter,
    NullFilter,
    TrivialFilter,
    SequenceFilter,
    AltSequenceFilter,
    MatchFilter,
    NoMatchFilter,
}

pub type ComposeFst<'fst, F1, F2, CF> = DynamicFst<ComposeFstImpl<'fst, F1, F2, CF>>;

impl<'fst, F1: Fst + 'fst, F2: Fst<W = F1::W> + 'fst, CF: ComposeFilter<'fst, F1, F2>>
    ComposeFst<'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    pub fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self> {
        let isymt = fst1.input_symbols();
        let osymt = fst2.output_symbols();
        let compose_impl = ComposeFstImpl::new(fst1, fst2)?;
        Ok(Self::from_impl(compose_impl, isymt, osymt))
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub struct ComposeConfig {
    compose_filter: ComposeFilterEnum,
    connect: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_filter: ComposeFilterEnum::AutoFilter,
            connect: true,
        }
    }
}

pub fn compose_with_config<F1: Fst, F2: Fst<W = F1::W>, F3: MutableFst<W = F1::W>>(
    fst1: &F1,
    fst2: &F2,
    config: ComposeConfig,
) -> Fallible<F3>
where
    F1::W: 'static,
{
    let mut ofst: F3 = match config.compose_filter {
        ComposeFilterEnum::AutoFilter => unimplemented!(),
        ComposeFilterEnum::NullFilter => {
            ComposeFst::<_, _, NullComposeFilter<SortedMatcher<_>, SortedMatcher<_>>>::new(
                fst1, fst2,
            )?
            .compute()?
        }
        ComposeFilterEnum::SequenceFilter => {
            ComposeFst::<_, _, SequenceComposeFilter<_, SortedMatcher<_>, SortedMatcher<_>>>::new(
                fst1, fst2,
            )?
            .compute()?
        }
        ComposeFilterEnum::AltSequenceFilter => ComposeFst::<
            _,
            _,
            AltSequenceComposeFilter<_, SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::MatchFilter => {
            ComposeFst::<_, _, MatchComposeFilter<_, _, SortedMatcher<_>, SortedMatcher<_>>>::new(
                fst1, fst2,
            )?
            .compute()?
        }
        ComposeFilterEnum::NoMatchFilter => {
            ComposeFst::<_, _, NoMatchComposeFilter<SortedMatcher<_>, SortedMatcher<_>>>::new(
                fst1, fst2,
            )?
            .compute()?
        }
        ComposeFilterEnum::TrivialFilter => {
            ComposeFst::<_, _, TrivialComposeFilter<SortedMatcher<_>, SortedMatcher<_>>>::new(
                fst1, fst2,
            )?
            .compute()?
        }
    };

    if config.connect {
        crate::algorithms::connect(&mut ofst)?;
    }

    Ok(ofst)
}

pub fn compose<F1: Fst, F2: Fst<W = F1::W>, F3: MutableFst<W = F1::W>>(
    fst1: &F1,
    fst2: &F2,
) -> Fallible<F3>
where
    F1::W: 'static,
{
    let config = ComposeConfig::default();
    compose_with_config(fst1, fst2, config)
}

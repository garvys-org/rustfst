use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use anyhow::Result;
use itertools::Itertools;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::compose::compose_filters::{
    AltSequenceComposeFilter, ComposeFilter, MatchComposeFilter, NoMatchComposeFilter,
    NullComposeFilter, SequenceComposeFilter, TrivialComposeFilter,
};
use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::matchers::{
    GenericMatcher, MatchType, SortedMatcher, REQUIRE_PRIORITY,
};
use crate::algorithms::compose::matchers::{Matcher, MatcherFlags};
use crate::algorithms::lazy_fst::LazyFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL, NO_LABEL};

pub struct ComposeFstImplOptions<M1, M2, CF, ST> {
    matcher1: Option<Rc<RefCell<M1>>>,
    matcher2: Option<Rc<RefCell<M2>>>,
    filter: Option<CF>,
    state_table: Option<ST>,
}

impl<M1, M2, CF, ST> Default for ComposeFstImplOptions<M1, M2, CF, ST> {
    fn default() -> Self {
        Self {
            matcher1: None,
            matcher2: None,
            filter: None,
            state_table: None,
        }
    }
}

impl<M1, M2, CF, ST> ComposeFstImplOptions<M1, M2, CF, ST> {
    pub fn new<
        IM1: Into<Option<Rc<RefCell<M1>>>>,
        IM2: Into<Option<Rc<RefCell<M2>>>>,
        ICF: Into<Option<CF>>,
        IST: Into<Option<ST>>,
    >(
        matcher1: IM1,
        matcher2: IM2,
        filter: ICF,
        state_table: IST,
    ) -> Self {
        Self {
            matcher1: matcher1.into(),
            matcher2: matcher2.into(),
            filter: filter.into(),
            state_table: state_table.into(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct ComposeStateTuple<FS> {
    fs: FS,
    s1: StateId,
    s2: StateId,
}

#[derive(Clone, Debug)]
pub struct ComposeFstImpl<W: Semiring, CF: ComposeFilter<W>> {
    fst1: Rc<<CF::M1 as Matcher<W>>::F>,
    fst2: Rc<<CF::M2 as Matcher<W>>::F>,
    matcher1: Rc<RefCell<CF::M1>>,
    matcher2: Rc<RefCell<CF::M2>>,
    compose_filter: CF,
    cache_impl: CacheImpl<W>,
    state_table: StateTable<ComposeStateTuple<CF::FS>>,
    match_type: MatchType,
}

impl<W: Semiring, CF: ComposeFilter<W>> ComposeFstImpl<W, CF> {
    // Compose specifying two matcher types Matcher1 and Matcher2. Requires input
    // FST (of the same Tr type, but o.w. arbitrary) match the corresponding
    // matcher FST types). Recommended only for advanced use in demanding or
    // specialized applications due to potential code bloat and matcher
    // incompatibilities.
    // fn new2(fst1: &'fst F1, fst2: &'fst F2) -> Result<Self> {
    //     unimplemented!()
    // }

    fn new(
        fst1: Rc<<CF::M1 as Matcher<W>>::F>,
        fst2: Rc<<CF::M2 as Matcher<W>>::F>,
        opts: ComposeFstImplOptions<CF::M1, CF::M2, CF, StateTable<ComposeStateTuple<CF::FS>>>,
    ) -> Result<Self> {
        let opts_matcher1 = opts.matcher1;
        let opts_matcher2 = opts.matcher2;
        let compose_filter = opts.filter.unwrap_or_else(|| {
            CF::new(
                Rc::clone(&fst1),
                Rc::clone(&fst2),
                opts_matcher1,
                opts_matcher2,
            )
            .unwrap()
        });
        let matcher1 = compose_filter.matcher1();
        let matcher2 = compose_filter.matcher2();
        Ok(Self {
            fst1,
            fst2,
            compose_filter,
            cache_impl: CacheImpl::new(),
            state_table: opts.state_table.unwrap_or_else(StateTable::new),
            match_type: Self::match_type(&matcher1, &matcher2)?,
            matcher1,
            matcher2,
        })
    }

    fn match_type(
        matcher1: &Rc<RefCell<CF::M1>>,
        matcher2: &Rc<RefCell<CF::M2>>,
    ) -> Result<MatchType> {
        if matcher1
            .borrow()
            .flags()
            .contains(MatcherFlags::REQUIRE_MATCH)
            && matcher1.borrow().match_type() != MatchType::MatchOutput
        {
            bail!("ComposeFst: 1st argument cannot perform required matching (sort?)")
        }
        if matcher2
            .borrow()
            .flags()
            .contains(MatcherFlags::REQUIRE_MATCH)
            && matcher2.borrow().match_type() != MatchType::MatchInput
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

    fn match_input(&self, s1: StateId, s2: StateId) -> Result<bool> {
        match self.match_type {
            MatchType::MatchInput => Ok(true),
            MatchType::MatchOutput => Ok(false),
            _ => {
                // Match both
                let priority1 = self.matcher1.borrow().priority(s1)?;
                let priority2 = self.matcher2.borrow().priority(s2)?;
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

    fn ordered_expand<FA: ExpandedFst<W = W>, FB: ExpandedFst<W = W>, M: Matcher<W, F = FA>>(
        &mut self,
        s: StateId,
        sa: StateId,
        fstb: Rc<FB>,
        sb: StateId,
        matchera: Rc<RefCell<M>>,
        match_input: bool,
    ) -> Result<()> {
        let tr_loop = if match_input {
            Tr::new(EPS_LABEL, NO_LABEL, W::one(), sb)
        } else {
            Tr::new(NO_LABEL, EPS_LABEL, W::one(), sb)
        };
        self.match_tr(s, sa, Rc::clone(&matchera), &tr_loop, match_input)?;
        for tr in fstb.tr_iter(sb)? {
            self.match_tr(s, sa, Rc::clone(&matchera), tr, match_input)?;
        }
        Ok(())
    }

    fn add_tr(&mut self, s: StateId, mut arc1: Tr<W>, arc2: Tr<W>, fs: CF::FS) -> Result<()> {
        let tuple = ComposeStateTuple {
            fs,
            s1: arc1.nextstate,
            s2: arc2.nextstate,
        };
        arc1.weight.times_assign(arc2.weight)?;
        self.cache_impl.push_tr(
            s,
            Tr::new(
                arc1.ilabel,
                arc2.olabel,
                arc1.weight,
                self.state_table.find_id(tuple),
            ),
        )?;

        Ok(())
    }

    fn match_tr<M: Matcher<W>>(
        &mut self,
        s: StateId,
        sa: StateId,
        matchera: Rc<RefCell<M>>,
        tr: &Tr<W>,
        match_input: bool,
    ) -> Result<()> {
        let label = if match_input { tr.olabel } else { tr.ilabel };

        // Collect necessary here because need to borrow_mut a matcher later. To investigate.
        let temp = matchera.borrow().iter(sa, label)?.collect_vec();
        for arca in temp {
            let mut arca = arca.into_tr(
                sa,
                if match_input {
                    MatchType::MatchInput
                } else {
                    MatchType::MatchOutput
                },
            )?;
            let mut arcb = tr.clone();
            if match_input {
                let fs = self.compose_filter.filter_tr(&mut arcb, &mut arca)?;
                if fs != CF::FS::new_no_state() {
                    self.add_tr(s, arcb, arca, fs)?;
                }
            } else {
                let fs = self.compose_filter.filter_tr(&mut arca, &mut arcb)?;

                if fs != CF::FS::new_no_state() {
                    self.add_tr(s, arca, arcb, fs)?;
                }
            }
        }

        Ok(())
    }
}

impl<W: Semiring + 'static, CF: ComposeFilter<W>> FstImpl for ComposeFstImpl<W, CF> {
    type W = W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        let tuple = self.state_table.find_tuple(state);
        let s1 = tuple.s1;
        let s2 = tuple.s2;
        self.compose_filter.set_state(s1, s2, &tuple.fs)?;
        drop(tuple);
        if self.match_input(s1, s2)? {
            self.ordered_expand(
                state,
                s2,
                Rc::clone(&self.fst1),
                s1,
                Rc::clone(&self.matcher2),
                true,
            )?;
        } else {
            self.ordered_expand(
                state,
                s1,
                Rc::clone(&self.fst2),
                s2,
                Rc::clone(&self.matcher1),
                false,
            )?;
        }
        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<StateId>> {
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

    fn compute_final(&mut self, state: usize) -> Result<Option<Self::W>> {
        let tuple = self.state_table.find_tuple(state);

        let s1 = tuple.s1;
        let final1 = self.compose_filter.matcher1().borrow().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let final1 = final1.unwrap();
        let mut final1 = unsafe { &*final1 }.clone();

        let s2 = tuple.s2;
        let final2 = self.compose_filter.matcher2().borrow().final_weight(s2)?;
        if final2.is_none() {
            return Ok(None);
        }
        let final2 = final2.unwrap();
        let mut final2 = unsafe { &*final2 }.clone();

        self.compose_filter.set_state(s1, s2, &tuple.fs)?;

        self.compose_filter.filter_final(&mut final1, &mut final2)?;

        final1.times_assign(&final2)?;
        if final1.is_zero() {
            Ok(None)
        } else {
            Ok(Some(final1))
        }
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

pub type ComposeFst<W, CF> = LazyFst<ComposeFstImpl<W, CF>>;

fn create_base<W: Semiring + 'static, F1: ExpandedFst<W = W>, F2: ExpandedFst<W = W>>(
    fst1: Rc<F1>,
    fst2: Rc<F2>,
) -> Result<ComposeFstImpl<W, SequenceComposeFilter<W, GenericMatcher<F1>, GenericMatcher<F2>>>> {
    // TODO: change this once Lookahead matchers are supported.
    let opts = ComposeFstImplOptions::<
        GenericMatcher<_>,
        GenericMatcher<_>,
        SequenceComposeFilter<_, _, _>,
        _,
    >::default();
    let compose_impl = ComposeFstImpl::new(fst1, fst2, opts)?;
    Ok(compose_impl)
}

impl<W: Semiring, CF: ComposeFilter<W>> ComposeFst<W, CF> {
    pub fn new_with_options(
        fst1: Rc<<CF::M1 as Matcher<W>>::F>,
        fst2: Rc<<CF::M2 as Matcher<W>>::F>,
        opts: ComposeFstImplOptions<CF::M1, CF::M2, CF, StateTable<ComposeStateTuple<CF::FS>>>,
    ) -> Result<Self>
    where
        W: 'static,
    {
        let isymt = fst1.input_symbols();
        let osymt = fst2.output_symbols();
        let compose_impl = ComposeFstImpl::new(fst1, fst2, opts)?;
        Ok(Self::from_impl(compose_impl, isymt, osymt))
    }

    // TODO: Change API, no really user friendly
    pub fn new(
        fst1: Rc<<CF::M1 as Matcher<W>>::F>,
        fst2: Rc<<CF::M2 as Matcher<W>>::F>,
    ) -> Result<Self>
    where
        W: 'static,
    {
        Self::new_with_options(fst1, fst2, ComposeFstImplOptions::default())
    }
}

impl<W: Semiring + 'static, F1: ExpandedFst<W = W>, F2: ExpandedFst<W = W>>
    ComposeFst<W, SequenceComposeFilter<W, GenericMatcher<F1>, GenericMatcher<F2>>>
{
    pub fn new_auto(fst1: Rc<F1>, fst2: Rc<F2>) -> Result<Self> {
        let isymt = fst1.input_symbols();
        let osymt = fst2.output_symbols();
        let compose_impl = create_base(fst1, fst2)?;
        Ok(Self::from_impl(compose_impl, isymt, osymt))
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub struct ComposeConfig {
    pub compose_filter: ComposeFilterEnum,
    pub connect: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_filter: ComposeFilterEnum::AutoFilter,
            connect: true,
        }
    }
}

pub fn compose_with_config<F1: ExpandedFst, F2: ExpandedFst<W = F1::W>, F3: MutableFst<W = F1::W>>(
    fst1: Rc<F1>,
    fst2: Rc<F2>,
    config: ComposeConfig,
) -> Result<F3>
where
    F1::W: 'static,
{
    let mut ofst: F3 = match config.compose_filter {
        ComposeFilterEnum::AutoFilter => ComposeFst::new_auto(fst1, fst2)?.compute()?,
        ComposeFilterEnum::NullFilter => {
            ComposeFst::<_, NullComposeFilter<SortedMatcher<_>, SortedMatcher<_>>>::new(fst1, fst2)?
                .compute()?
        }
        ComposeFilterEnum::SequenceFilter => ComposeFst::<
            _,
            SequenceComposeFilter<_, SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::AltSequenceFilter => ComposeFst::<
            _,
            AltSequenceComposeFilter<_, SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::MatchFilter => ComposeFst::<
            _,
            MatchComposeFilter<_, _, SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::NoMatchFilter => ComposeFst::<
            _,
            NoMatchComposeFilter<SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::TrivialFilter => ComposeFst::<
            _,
            TrivialComposeFilter<SortedMatcher<_>, SortedMatcher<_>>,
        >::new(fst1, fst2)?
        .compute()?,
    };

    if config.connect {
        crate::algorithms::connect(&mut ofst)?;
    }

    Ok(ofst)
}

/// This operation computes the composition of two transducers.
/// If `A` transduces string `x` to `y` with weight `a` and `B` transduces `y` to `z`
/// with weight `b`, then their composition transduces string `x` to `z` with weight `a ⊗ b`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use anyhow::Result;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::compose;
/// # use std::rc::Rc;
/// # fn main() -> Result<()> {
/// let fst_1 : VectorFst<IntegerWeight> = fst![1,2 => 2,3];
///
/// let fst_2 : VectorFst<IntegerWeight> = fst![2,3 => 3,4];
///
/// let fst_ref : VectorFst<IntegerWeight> = fst![1,2 => 3,4];
///
/// let composed_fst : VectorFst<_> = compose(Rc::new(fst_1), Rc::new(fst_2))?;
/// assert_eq!(composed_fst, fst_ref);
/// # Ok(())
/// # }
/// ```
pub fn compose<F1: ExpandedFst, F2: ExpandedFst<W = F1::W>, F3: MutableFst<W = F1::W>>(
    fst1: Rc<F1>,
    fst2: Rc<F2>,
) -> Result<F3>
where
    F1::W: 'static,
{
    let config = ComposeConfig::default();
    compose_with_config(fst1, fst2, config)
}

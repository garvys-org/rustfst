use std::cell::RefCell;
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;

use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher, REQUIRE_PRIORITY};
use crate::algorithms::compose::{ComposeFstOpOptions, ComposeStateTuple};
use crate::algorithms::lazy_fst_revamp::{FstOp, StateTable};
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs, TrsVec, EPS_LABEL, NO_LABEL};

#[derive(Debug)]
pub struct ComposeFstOp<W: Semiring, CF: ComposeFilter<W>> {
    fst1: Arc<<CF::M1 as Matcher<W>>::F>,
    fst2: Arc<<CF::M2 as Matcher<W>>::F>,
    compose_filter_builder: ComposeFilterBuilder<W, CF>,
    state_table: StateTable<ComposeStateTuple<CF::FS>>,
    match_type: MatchType,
}

impl<W: Semiring, CF: ComposeFilter<W>> ComposeFstOp<W, CF> {
    // Compose specifying two matcher types Matcher1 and Matcher2. Requires input
    // FST (of the same Tr type, but o.w. arbitrary) match the corresponding
    // matcher FST types). Recommended only for advanced use in demanding or
    // specialized applications due to potential code bloat and matcher
    // incompatibilities.
    // fn new2(fst1: &'fst F1, fst2: &'fst F2) -> Result<Self> {
    //     unimplemented!()
    // }

    pub fn new(
        fst1: Arc<<CF::M1 as Matcher<W>>::F>,
        fst2: Arc<<CF::M2 as Matcher<W>>::F>,
        opts: ComposeFstOpOptions<CF::M1, CF::M2, ComposeFilterBuilder<W, CF>, StateTable<ComposeStateTuple<CF::FS>>>,
    ) -> Result<Self> {
        let matcher1 = opts.matcher1;
        let matcher2 = opts.matcher2;
        let compose_filter_builder = opts.filter.unwrap_or_else(|| {
            ComposeFilterBuilder::new(
                Arc::clone(&fst1),
                Arc::clone(&fst2),
                matcher1, matcher2
            )
        });
        let compose_filter = compose_filter_builder.build()?;
        let match_type = Self::match_type(&compose_filter.matcher1(), &compose_filter.matcher2())?;
        Ok(Self {
            fst1,
            fst2,
            compose_filter_builder,
            state_table: opts.state_table.unwrap_or_else(StateTable::new),
            match_type,
        })
    }

    fn match_type(
        matcher1: &Arc<RefCell<CF::M1>>,
        matcher2: &Arc<RefCell<CF::M2>>,
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

    fn match_input(&self, s1: StateId, s2: StateId, compose_filter: &CF) -> Result<bool> {
        match self.match_type {
            MatchType::MatchInput => Ok(true),
            MatchType::MatchOutput => Ok(false),
            _ => {
                // Match both
                let priority1 = compose_filter.matcher1().borrow().priority(s1)?;
                let priority2 = compose_filter.matcher2().borrow().priority(s2)?;
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

    fn ordered_expand<FA: ExpandedFst<W>, FB: ExpandedFst<W>, M: Matcher<W, F = FA>>(
        &self,
        s: StateId,
        sa: StateId,
        fstb: Arc<FB>,
        sb: StateId,
        matchera: Arc<RefCell<M>>,
        match_input: bool,
        mut compose_filter: CF,
    ) -> Result<TrsVec<W>> {
        let tr_loop = if match_input {
            Tr::new(EPS_LABEL, NO_LABEL, W::one(), sb)
        } else {
            Tr::new(NO_LABEL, EPS_LABEL, W::one(), sb)
        };
        let mut trs = vec![];
        trs.extend(self.match_tr(s, sa, Arc::clone(&matchera), &tr_loop, match_input, &mut compose_filter)?);
        for tr in fstb.get_trs(sb)?.trs() {
            trs.extend(self.match_tr(s, sa, Arc::clone(&matchera), tr, match_input, &mut compose_filter)?);
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn add_tr(&self, s: StateId, mut arc1: Tr<W>, arc2: Tr<W>, fs: CF::FS) -> Result<Tr<W>> {
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
                self.state_table.find_id(tuple)))

    }

    fn match_tr<M: Matcher<W>>(
        &self,
        s: StateId,
        sa: StateId,
        matchera: Arc<RefCell<M>>,
        tr: &Tr<W>,
        match_input: bool,
        compose_filter: &mut CF,
    ) -> Result<Vec<Tr<W>>> {
        let label = if match_input { tr.olabel } else { tr.ilabel };
        let mut trs = vec![];

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
                let fs = compose_filter.filter_tr(&mut arcb, &mut arca)?;
                if fs != CF::FS::new_no_state() {
                    trs.push(self.add_tr(s, arcb, arca, fs)?);
                }
            } else {
                let fs = compose_filter.filter_tr(&mut arca, &mut arcb)?;

                if fs != CF::FS::new_no_state() {
                    trs.push(self.add_tr(s, arca, arcb, fs)?);
                }
            }
        }

        Ok(trs)
    }
}

impl<W: Semiring, CF: ComposeFilter<W>> FstOp<W> for ComposeFstOp<W, CF> {
    fn compute_start(&self) -> Result<Option<usize>> {
        let compose_filter = self.compose_filter_builder.build()?;
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
        let fs = compose_filter.start();
        let tuple = ComposeStateTuple { s1, s2, fs };
        Ok(Some(self.state_table.find_id(tuple)))
    }

    fn compute_trs(&self, state: usize) -> Result<TrsVec<W>> {
        let tuple = self.state_table.find_tuple(state);
        let s1 = tuple.s1;
        let s2 = tuple.s2;

        let mut compose_filter = self.compose_filter_builder.build()?;
        compose_filter.set_state(s1, s2, &tuple.fs)?;
        drop(tuple);
        if self.match_input(s1, s2, &compose_filter)? {
            self.ordered_expand(
                state,
                s2,
                Arc::clone(&self.fst1),
                s1,
                compose_filter.matcher2(),
                true,
                compose_filter
            )
        } else {
            self.ordered_expand(
                state,
                s1,
                Arc::clone(&self.fst2),
                s2,
                compose_filter.matcher1(),
                false,
                compose_filter
            )
        }
    }

    fn compute_final_weight(&self, state: usize) -> Result<Option<W>> {
        let tuple = self.state_table.find_tuple(state);

        // Construct a new ComposeFilter each time to avoid mutating the internal state.
        let mut compose_filter = self.compose_filter_builder.build()?;

        let s1 = tuple.s1;
        let final1 = compose_filter.matcher1().borrow().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let mut final1 = final1.unwrap();

        let s2 = tuple.s2;
        let final2 = compose_filter.matcher2().borrow().final_weight(s2)?;
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
}

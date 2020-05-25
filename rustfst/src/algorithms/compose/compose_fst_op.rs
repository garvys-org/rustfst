use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher, REQUIRE_PRIORITY};
use crate::algorithms::compose::{ComposeFstOpOptions, ComposeStateTuple};
use crate::algorithms::lazy_fst_revamp::{FstOp, StateTable};
use crate::fst_traits::CoreFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs, TrsVec, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone)]
pub struct ComposeFstOp<W: Semiring, CFB: ComposeFilterBuilder<W>> {
    compose_filter_builder: CFB,
    state_table: StateTable<ComposeStateTuple<<CFB::CF as ComposeFilter<W>>::FS>>,
    match_type: MatchType,
}

impl<W: Semiring, CFB: ComposeFilterBuilder<W>> ComposeFstOp<W, CFB> {
    // Compose specifying two matcher types Matcher1 and Matcher2. Requires input
    // FST (of the same Tr type, but o.w. arbitrary) match the corresponding
    // matcher FST types). Recommended only for advanced use in demanding or
    // specialized applications due to potential code bloat and matcher
    // incompatibilities.
    // fn new2(fst1: &'fst F1, fst2: &'fst F2) -> Result<Self> {
    //     unimplemented!()
    // }

    pub fn new(
        fst1: Arc<<<CFB::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<CFB::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        opts: ComposeFstOpOptions<
            CFB::M1,
            CFB::M2,
            CFB,
            StateTable<ComposeStateTuple<<CFB::CF as ComposeFilter<W>>::FS>>,
        >,
    ) -> Result<Self> {
        let matcher1 = opts.matcher1;
        let matcher2 = opts.matcher2;
        let compose_filter_builder = opts.filter_builder.unwrap_or_else(|| {
            ComposeFilterBuilder::new(Arc::clone(&fst1), Arc::clone(&fst2), matcher1, matcher2)
                .unwrap()
        });
        let compose_filter = compose_filter_builder.build()?;
        let match_type = Self::match_type(compose_filter.matcher1(), compose_filter.matcher2())?;
        Ok(Self {
            compose_filter_builder,
            state_table: opts.state_table.unwrap_or_else(StateTable::new),
            match_type,
        })
    }

    fn match_type(
        matcher1: &<CFB::CF as ComposeFilter<W>>::M1,
        matcher2: &<CFB::CF as ComposeFilter<W>>::M2,
    ) -> Result<MatchType> {
        if matcher1.flags().contains(MatcherFlags::REQUIRE_MATCH)
            && matcher1.match_type() != MatchType::MatchOutput
        {
            bail!("ComposeFst: 1st argument cannot perform required matching (sort?)")
        }
        if matcher2.flags().contains(MatcherFlags::REQUIRE_MATCH)
            && matcher2.match_type() != MatchType::MatchInput
        {
            bail!("ComposeFst: 2nd argument cannot perform required matching (sort?)")
        }

        let type1 = matcher1.match_type();
        let type2 = matcher2.match_type();
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
                let fst = Arc::clone(compose_filter.fst1());
                trs.extend(self.match_tr(
                    sa,
                    &tr_loop,
                    match_input,
                    &mut compose_filter,
                    selector,
                )?);
                for tr in fst.get_trs(sb)?.trs() {
                    trs.extend(self.match_tr(
                        sa,
                        tr,
                        match_input,
                        &mut compose_filter,
                        selector,
                    )?);
                }
            }
            Selector::Fst2Matcher1 => {
                let fst = Arc::clone(compose_filter.fst2());
                trs.extend(self.match_tr(
                    sa,
                    &tr_loop,
                    match_input,
                    &mut compose_filter,
                    selector,
                )?);
                for tr in fst.get_trs(sb)?.trs() {
                    trs.extend(self.match_tr(
                        sa,
                        tr,
                        match_input,
                        &mut compose_filter,
                        selector,
                    )?);
                }
            }
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn add_tr(
        &self,
        mut arc1: Tr<W>,
        arc2: Tr<W>,
        fs: <CFB::CF as ComposeFilter<W>>::FS,
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
            self.state_table.find_id(tuple),
        ))
    }

    fn match_tr(
        &self,
        sa: StateId,
        tr: &Tr<W>,
        match_input: bool,
        compose_filter: &mut CFB::CF,
        selector: Selector,
    ) -> Result<Vec<Tr<W>>> {
        let label = if match_input { tr.olabel } else { tr.ilabel };
        let mut trs = vec![];

        // Collect necessary here because need to borrow_mut a matcher later. To investigate.
        let temp = match selector {
            Selector::Fst2Matcher1 => compose_filter.matcher1().iter(sa, label)?.collect_vec(),
            Selector::Fst1Matcher2 => compose_filter.matcher2().iter(sa, label)?.collect_vec(),
        };
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
                if fs != <CFB::CF as ComposeFilter<W>>::FS::new_no_state() {
                    trs.push(self.add_tr(arcb, arca, fs)?);
                }
            } else {
                let fs = compose_filter.filter_tr(&mut arca, &mut arcb)?;

                if fs != <CFB::CF as ComposeFilter<W>>::FS::new_no_state() {
                    trs.push(self.add_tr(arca, arcb, fs)?);
                }
            }
        }

        Ok(trs)
    }
}

impl<W: Semiring, CFB: ComposeFilterBuilder<W>> FstOp<W> for ComposeFstOp<W, CFB> {
    fn compute_start(&self) -> Result<Option<usize>> {
        let compose_filter = self.compose_filter_builder.build()?;
        let s1 = compose_filter.fst1().start();
        if s1.is_none() {
            return Ok(None);
        }
        let s1 = s1.unwrap();
        let s2 = compose_filter.fst2().start();
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
        let res = if self.match_input(s1, s2, &compose_filter)? {
            self.ordered_expand(s2, s1, true, compose_filter, Selector::Fst1Matcher2)
        } else {
            self.ordered_expand(s1, s2, false, compose_filter, Selector::Fst2Matcher1)
        };
        res
    }

    fn compute_final_weight(&self, state: usize) -> Result<Option<W>> {
        let tuple = self.state_table.find_tuple(state);

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
}

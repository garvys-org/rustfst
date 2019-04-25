use std::cmp::max;

use failure::Fallible;

use crate::algorithms::arc_compares::ilabel_compare;
use crate::algorithms::arc_mappers::QuantizeMapper;
use crate::algorithms::factor_iterators::GallicFactorLeft;
use crate::algorithms::partition::Partition;
use crate::algorithms::state_mappers::ArcUniqueMapper;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::{
    arc_map, arc_sort, connect, decode, encode, factor_weight, push_weights, state_map,
    weight_convert, FactorWeightOptions, FactorWeightType, ReweightType,
};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{
    GallicWeightLeft, Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::StateId;
use crate::EPS_LABEL;
use crate::KDELTA;
use std::collections::HashSet;

pub fn minimize<F>(ifst: &mut F, allow_nondet: bool) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
    F::W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    let props = ifst.properties()?;
    let mut allow_acyclic_minimization;

    if props.contains(FstProperties::I_DETERMINISTIC) {
        allow_acyclic_minimization = true;
    } else {
        if !F::W::properties().contains(SemiringProperties::IDEMPOTENT) {
            bail!("Cannot minimize a non-deterministic FST over a non-idempotent semiring")
        } else if !allow_nondet {
            bail!("Refusing to minimize a non-deterministic FST with allow_nondet = false")
        }

        allow_acyclic_minimization = false;
    }

    if !props.contains(FstProperties::ACCEPTOR) {
        // Weighted transducer
        let mut to_gallic = ToGallicConverter {};
        let mut gfst: VectorFst<GallicWeightLeft<F::W>> = weight_convert(ifst, &mut to_gallic)?;
        push_weights(&mut gfst, ReweightType::ReweightToInitial)?;
        let mut quantize_mapper = QuantizeMapper {};
        arc_map(&mut gfst, &mut quantize_mapper)?;
        let encode_table = encode(&mut gfst, true, true)?;
        acceptor_minimize(&mut gfst, allow_acyclic_minimization)?;
        decode(&mut gfst, encode_table)?;
        let factor_opts: FactorWeightOptions = FactorWeightOptions {
            delta: KDELTA,
            mode: FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS,
            final_ilabel: 0,
            final_olabel: 0,
            increment_final_ilabel: false,
            increment_final_olabel: false,
        };
        let fwfst: VectorFst<_> =
            factor_weight::<_, _, GallicFactorLeft<F::W>>(&gfst, factor_opts)?;
        let mut from_gallic = FromGallicConverter {
            superfinal_label: EPS_LABEL,
        };
        *ifst = weight_convert(&fwfst, &mut from_gallic)?;
        Ok(())
    } else if props.contains(FstProperties::WEIGHTED) {
        // Weighted acceptor
        push_weights(ifst, ReweightType::ReweightToInitial)?;
        let mut quantize_mapper = QuantizeMapper {};
        arc_map(ifst, &mut quantize_mapper)?;
        let encode_table = encode(ifst, true, true)?;
        acceptor_minimize(ifst, allow_acyclic_minimization)?;
        decode(ifst, encode_table)
    } else {
        // Unweighted acceptor
        acceptor_minimize(ifst, allow_acyclic_minimization)
    }
}

fn acceptor_minimize<F: MutableFst + ExpandedFst>(
    ifst: &mut F,
    allow_acyclic_minimization: bool,
) -> Fallible<()> {
    let props = ifst.properties()?;
    if !props.contains(FstProperties::ACCEPTOR | FstProperties::UNWEIGHTED) {
        bail!("FST is not an unweighted acceptor");
    }

    connect(ifst)?;

    if ifst.num_states() == 0 {
        return Ok(());
    }

    if allow_acyclic_minimization && props.contains(FstProperties::ACYCLIC) {
        // Acyclic minimization
        arc_sort(ifst, ilabel_compare)?;
        let minimizer = AcyclicMinimizer::new(ifst)?;
        merge_states(minimizer.get_partition(), ifst)?;
    } else {
        // Cyclic minimization
        let minimizer = CyclicMinimizer::new(ifst);
        merge_states(minimizer.get_partition(), ifst)?;
    }

    let mut mapper = ArcUniqueMapper {};
    state_map(ifst, &mut mapper)?;

    Ok(())
}

fn merge_states<F: MutableFst + ExpandedFst>(partition: Partition, fst: &mut F) -> Fallible<()> {
    let mut state_map = vec![None; partition.num_classes()];
    for i in 0..partition.num_classes() {
        state_map[i] = partition.iter(i).next();
    }

    for c in 0..partition.num_classes() {
        for s in partition.iter(c) {
            if s == state_map[c].unwrap() {
                for arc in fst.arcs_iter_mut(s)? {
                    arc.nextstate = state_map[partition.get_class_id(arc.nextstate)].unwrap();
                }
            } else {
                let arcs: Vec<_> = fst
                    .arcs_iter(s)?
                    .cloned()
                    .map(|mut arc| {
                        arc.nextstate = state_map[partition.get_class_id(arc.nextstate)].unwrap();
                        arc
                    })
                    .collect();
                for arc in arcs.into_iter() {
                    fst.add_arc(state_map[c].unwrap(), arc)?;
                }
            }
        }
    }

    fst.set_start(state_map[partition.get_class_id(fst.start().unwrap())].unwrap())?;
    connect(fst)?;
    Ok(())
}

// Compute the height (distance) to final state
pub fn fst_depth<F: Fst>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    fully_examined_states: &mut HashSet<StateId>,
    heights: &mut Vec<i32>,
) -> Fallible<()> {
    accessible_states.insert(state_id_cour);

    for i in heights.len()..=state_id_cour {
        heights.push(-1);
    }

    let mut height_cur_state = 0;
    for arc in fst.arcs_iter(state_id_cour)? {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            fst_depth(
                fst,
                nextstate,
                accessible_states,
                fully_examined_states,
                heights,
            )?;
        }

        height_cur_state = max(height_cur_state, 1 + heights[nextstate]);
    }
    fully_examined_states.insert(state_id_cour);

    heights[state_id_cour] = height_cur_state;

    Ok(())
}

struct AcyclicMinimizer {
    partition: Partition
}

impl AcyclicMinimizer {
    pub fn new<F: MutableFst + ExpandedFst>(fst: &mut F) -> Fallible<Self> {
        let mut c = Self{partition: Partition::empty_new()};
        c.initialize(fst)?;
        c.refine(fst);
        Ok(c)
    }

    fn initialize<F: MutableFst + ExpandedFst>(&mut self, fst: &mut F) -> Fallible<()> {
        let mut accessible_state = HashSet::new();
        let mut fully_examined_states = HashSet::new();
        let mut heights = Vec::new();
        fst_depth(
            fst,
            fst.start().unwrap(),
            &mut accessible_state,
            &mut fully_examined_states,
            &mut heights,
        )?;
        self.partition.initialize(heights.len());
        self.partition.allocate_classes((heights.iter().max().unwrap() + 1) as usize);
        for s in 0..heights.len() {
            self.partition.add(s, heights[s] as usize);
        }
        Ok(())
    }


    fn refine<F: MutableFst + ExpandedFst>(&mut self, fst: &mut F) {
        unimplemented!()
    }

    pub fn get_partition(self) -> Partition {
        self.partition
    }
}

struct CyclicMinimizer<'a, F: MutableFst + ExpandedFst> {
    fst: &'a mut F,
}

impl<'a, F: MutableFst + ExpandedFst> CyclicMinimizer<'a, F> {
    pub fn new(fst: &'a mut F) -> Self {
        Self { fst }
    }

    pub fn get_partition(&self) -> Partition {
        unimplemented!()
    }
}

struct StateComparator<'a, 'b, F: MutableFst + ExpandedFst> {
    fst: &'a F,
    partition: &'b Partition
}

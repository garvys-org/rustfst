use std::cell::RefCell;
use std::cmp::Ordering;

use binary_heap_plus::BinaryHeap;

use failure::Fallible;

use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::{connect, determinize_with_distance, reverse, shortest_distance, Queue};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, CoreFst, ExpandedFst, MutableFst};
use crate::semirings::{Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize};
use crate::Arc;
use crate::StateId;
use crate::algorithms::arc_filters::AnyArcFilter;

/// Creates an FST containing the n-shortest paths in the input FST. The n-shortest paths are the
/// n-lowest weight paths w.r.t. the natural semiring order.
///
/// # Example
///
/// ## Input
///
/// ![shortestpath_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/shortestpath_in.svg?sanitize=true)
///
/// ## Output with n=1
///
/// ![shortestpath_out_n_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/shortestpath_out_n_1.svg?sanitize=true)
///
/// ## Output with n=2
///
/// ![shortestpath_out_n_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/shortestpath_out_n_2.svg?sanitize=true)
///
pub fn shortest_path<FI, FO>(ifst: &FI, nshortest: usize, unique: bool) -> Fallible<FO>
where
    FI: MutableFst + ExpandedFst,
    FO: MutableFst<W = FI::W>,
    FI::W: 'static
        + Into<<<FI as CoreFst>::W as Semiring>::ReverseWeight>
        + From<<<FI as CoreFst>::W as Semiring>::ReverseWeight>,
    <<FI as CoreFst>::W as Semiring>::ReverseWeight: WeightQuantize + WeaklyDivisibleSemiring,
{
    if nshortest == 0 {
        return Ok(FO::new());
    }

    if nshortest == 1 {
        let mut parent = vec![];
        let mut f_parent = None;
        let mut distance = vec![];

        single_shortest_path(ifst, &mut distance, &mut f_parent, &mut parent)?;
        let mut fst_res: FO = single_shortest_path_backtrace(ifst, &f_parent, &parent)?;
        fst_res.set_symts_from_fst(ifst);
        return Ok(fst_res);
    }

    if !FI::W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
        bail!("ShortestPath : Weight need to have the Path property and be distributive")
    }

    let mut distance = shortest_distance(ifst, false)?;

    let rfst: VectorFst<_> = reverse(ifst)?;
    let mut d = FI::W::zero();
    for rarc in rfst.arcs_iter(0)? {
        let state = rarc.nextstate - 1;
        if state < distance.len() {
            let rweight: FI::W = hack_convert_reverse_reverse(rarc.weight.reverse()?);
            d.plus_assign(rweight.times(&distance[state])?)?;
        }
    }

    let mut distance_2 = vec![d];
    distance_2.append(&mut distance);
    let mut fst_res: FO = if !unique {
        n_shortest_path(&rfst, &distance_2, nshortest)?
    } else {
        let distance_2_reversed: Vec<<<FI as CoreFst>::W as Semiring>::ReverseWeight> =
            distance_2.into_iter().map(|v| v.into()).collect();
        let (dfst, distance_3_reversed): (VectorFst<_>, _) =
            determinize_with_distance(&rfst, &distance_2_reversed)?;
        let distance_3: Vec<_> = distance_3_reversed.into_iter().map(|v| v.into()).collect();
        n_shortest_path(&dfst, &distance_3, nshortest)?
    };

    fst_res.set_symts_from_fst(ifst);

    Ok(fst_res)
}

pub fn hack_convert_reverse_reverse<W: Semiring>(
    p: <<W as Semiring>::ReverseWeight as Semiring>::ReverseWeight,
) -> W {
    unsafe {
        std::mem::transmute::<&<<W as Semiring>::ReverseWeight as Semiring>::ReverseWeight, &W>(&p)
    }
    .clone()
}

fn single_shortest_path<F>(
    ifst: &F,
    distance: &mut Vec<F::W>,
    f_parent: &mut Option<StateId>,
    parent: &mut Vec<Option<(StateId, usize)>>,
) -> Fallible<()>
where
    F: ExpandedFst + MutableFst,
    <F as CoreFst>::W: 'static,
{
    parent.clear();
    *f_parent = None;
    let start = ifst.start();
    if start.is_none() {
        return Ok(());
    }
    let mut enqueued = vec![];
    let mut queue = AutoQueue::new(ifst, None, &AnyArcFilter{})?;
    let source = unsafe { start.unsafe_unwrap() };
    let mut f_distance = F::W::zero();
    distance.clear();
    queue.clear();
    if !F::W::properties().contains(SemiringProperties::PATH | SemiringProperties::RIGHT_SEMIRING) {
        bail!(
            "SingleShortestPath: Weight needs to have the path property and be right distributive"
        )
    }
    while distance.len() < source {
        distance.push(F::W::zero());
        enqueued.push(false);
        parent.push(None)
    }
    distance.push(F::W::one());
    parent.push(None);
    queue.enqueue(source);
    enqueued.push(true);

    while !queue.is_empty() {
        // Safe because non empty
        let s = unsafe { queue.head().unsafe_unwrap() };
        queue.dequeue();
        enqueued[s] = false;
        let sd = distance[s].clone();

        if let Some(final_weight) = ifst.final_weight(s)? {
            let plus = f_distance.plus(&sd.times(final_weight)?)?;
            if f_distance != plus {
                f_distance = plus;
                *f_parent = Some(s);
            }
        }

        for (pos, arc) in unsafe { ifst.arcs_iter_unchecked(s).enumerate() } {
            while distance.len() <= arc.nextstate {
                distance.push(F::W::zero());
                enqueued.push(false);
                parent.push(None)
            }
            let nd = &mut distance[arc.nextstate];
            let weight = sd.times(&arc.weight)?;
            if *nd != nd.plus(&weight)? {
                *nd = nd.plus(&weight)?;
                parent[arc.nextstate] = Some((s, pos));
                if !enqueued[arc.nextstate] {
                    queue.enqueue(arc.nextstate);
                    enqueued[arc.nextstate] = true;
                } else {
                    queue.update(arc.nextstate);
                }
            }
        }
    }
    Ok(())
}

fn single_shortest_path_backtrace<FI, FO>(
    ifst: &FI,
    f_parent: &Option<StateId>,
    parent: &[Option<(StateId, usize)>],
) -> Fallible<FO>
where
    FI: ExpandedFst + MutableFst,
    FO: MutableFst<W = FI::W>,
{
    let mut ofst = FO::new();
    let mut s_p = None;
    let mut d_p;

    let mut d: Option<StateId> = None;
    let mut nextstate = *f_parent;
    while let Some(state) = nextstate {
        d_p = s_p;
        s_p = Some(ofst.add_state());
        if d.is_none() {
            if let Some(final_weight) = ifst.final_weight(f_parent.unwrap())? {
                ofst.set_final(s_p.unwrap(), final_weight.clone())?;
            }
        } else {
            let pos = parent[d.unwrap()].unwrap().1;
            let mut arc = ifst.arcs_iter(state)?.nth(pos).unwrap().clone();
            arc.nextstate = d_p.unwrap();
            ofst.add_arc(s_p.unwrap(), arc)?;
        }

        // Next iteration
        d = Some(state);
        nextstate = parent[state].map(|v| v.0);
    }

    if let Some(_s_p) = s_p {
        ofst.set_start(_s_p)?;
    }
    Ok(ofst)
}

pub fn natural_less<W: Semiring>(w1: &W, w2: &W) -> Fallible<bool> {
    Ok((&w1.plus(w2)? == w1) && (w1 != w2))
}

struct ShortestPathCompare<'a, 'b, W: Semiring> {
    pairs: &'a RefCell<Vec<(Option<StateId>, W)>>,
    distance: &'b [W],
    weight_zero: W,
    weight_one: W,
}

impl<'a, 'b, W: Semiring> ShortestPathCompare<'a, 'b, W> {
    pub fn new(pairs: &'a RefCell<Vec<(Option<StateId>, W)>>, distance: &'b [W]) -> Self {
        Self {
            pairs,
            distance,
            weight_zero: W::zero(),
            weight_one: W::one(),
        }
    }

    fn pweight(&self, state: &Option<StateId>) -> &W {
        if let Some(_state) = state {
            if *_state < self.distance.len() {
                &self.distance[*_state]
            } else {
                &self.weight_zero
            }
        } else {
            &self.weight_one
        }
    }

    fn compare(&self, x: StateId, y: StateId) -> bool {
        let b = self.pairs.borrow();
        let px = &b[x];
        let py = &b[y];
        let wx = self.pweight(&px.0).times(&px.1).unwrap();
        let wy = self.pweight(&py.0).times(&py.1).unwrap();
        if px.0.is_none() && py.0.is_some() {
            natural_less(&wy, &wx).unwrap() || (wy == wx)
        } else if px.0.is_some() && py.0.is_none() {
            natural_less(&wy, &wx).unwrap() && !(wy == wx)
        } else {
            natural_less(&wy, &wx).unwrap()
        }
    }
}

fn n_shortest_path<W, FI, FO>(ifst: &FI, distance: &[W], nshortest: usize) -> Fallible<FO>
where
    W: Semiring + 'static,
    FI: ExpandedFst<W = W::ReverseWeight> + MutableFst<W = W::ReverseWeight>,
    FO: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let mut ofst = FO::new();
    if nshortest == 0 {
        return Ok(ofst);
    }

    if !FI::W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
        bail!("NShortestPath: Weight needs to have the path property and be distributive");
    }

    let istart = ifst.start();
    if istart.is_none() || distance.len() <= istart.unwrap() || distance[istart.unwrap()].is_zero()
    {
        // No start state or start state is unreachable
        return Ok(ofst);
    }
    let istart = istart.unwrap();
    let ostart = ofst.add_state();
    ofst.set_start(ostart)?;
    let final_state = ofst.add_state();
    ofst.set_final(final_state, W::one())?;
    let pairs = RefCell::new(vec![(None, W::zero()); final_state + 1]);
    pairs.borrow_mut()[final_state] = (Some(istart), W::one());

    let shortest_path_compare = ShortestPathCompare::new(&pairs, distance);
    let mut heap = BinaryHeap::new_by(|v1, v2| {
        if shortest_path_compare.compare(*v1, *v2) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    heap.push(final_state);

    let mut r = vec![];
    while !heap.is_empty() {
        let state = heap.pop().unwrap();
        let p = pairs.borrow()[state].clone();
        let p_first_real = p.0.map(|v| v as i32).unwrap_or(-1) + 1;
        while r.len() as i32 <= p_first_real {
            r.push(0);
        }
        r[p_first_real as usize] += 1;
        if p.0.is_none() {
            ofst.add_arc(ofst.start().unwrap(), Arc::new(0, 0, W::one(), state))?;
        }
        if p.0.is_none() && r[p_first_real as usize] == nshortest {
            break;
        }
        if r[p_first_real as usize] > nshortest {
            continue;
        }
        if p.0.is_none() {
            continue;
        }
        for rarc in ifst.arcs_iter(p.0.unwrap())? {
            let mut arc: Arc<W> = Arc::new(
                rarc.ilabel,
                rarc.olabel,
                hack_convert_reverse_reverse::<W>(rarc.weight.reverse()?),
                rarc.nextstate,
            );
            let weight = p.1.times(&arc.weight)?;
            let next = ofst.add_state();
            pairs.borrow_mut().push((Some(arc.nextstate), weight));
            arc.nextstate = state;
            ofst.add_arc(next, arc)?;
            heap.push(next);
        }
        let final_weight = ifst.final_weight(p.0.unwrap())?;
        if let Some(_final_weight) = final_weight {
            let r_final_weight: W = hack_convert_reverse_reverse(_final_weight.reverse()?);
            if !r_final_weight.is_zero() {
                let weight = p.1.times(&r_final_weight)?;
                let next = ofst.add_state();
                pairs.borrow_mut().push((None, weight));
                ofst.add_arc(next, Arc::new(0, 0, r_final_weight, state))?;
                heap.push(next);
            }
        }
    }

    connect(&mut ofst)?;

    Ok(ofst)
}

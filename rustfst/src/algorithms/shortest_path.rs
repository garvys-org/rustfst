use std::cell::RefCell;
use std::cmp::Ordering;

use binary_heap_plus::BinaryHeap;

use anyhow::Result;

use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::algorithms::{connect, determinize_with_distance, reverse, shortest_distance, Queue};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, MutableFst};
use crate::semirings::{
    ReverseBack, Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::Tr;
use crate::{StateId, Trs};

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
pub fn shortest_path<W, FI, FO>(ifst: &FI, nshortest: usize, unique: bool) -> Result<FO>
where
    FI: ExpandedFst<W>,
    FO: MutableFst<W>,
    W: Semiring + Into<<W as Semiring>::ReverseWeight> + From<<W as Semiring>::ReverseWeight>,
    <W as Semiring>::ReverseWeight: WeightQuantize + WeaklyDivisibleSemiring,
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

    if !W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
        bail!("ShortestPath : Weight need to have the Path property and be distributive")
    }

    let mut distance = shortest_distance(ifst, false)?;

    let rfst: VectorFst<_> = reverse(ifst)?;
    let mut d = W::zero();
    for rarc in rfst.get_trs(0)?.trs() {
        let state = rarc.nextstate - 1;
        if state < distance.len() {
            let rweight: W = rarc.weight.reverse_back()?;
            d.plus_assign(rweight.times(&distance[state])?)?;
        }
    }

    let mut distance_2 = vec![d];
    distance_2.append(&mut distance);
    let mut fst_res: FO = if !unique {
        n_shortest_path(&rfst, &distance_2, nshortest)?
    } else {
        let distance_2_reversed: Vec<<W as Semiring>::ReverseWeight> =
            distance_2.into_iter().map(|v| v.into()).collect();
        let (dfst, distance_3_reversed): (VectorFst<_>, _) =
            determinize_with_distance(&rfst, &distance_2_reversed)?;
        let distance_3: Vec<_> = distance_3_reversed.into_iter().map(|v| v.into()).collect();
        n_shortest_path(&dfst, &distance_3, nshortest)?
    };

    fst_res.set_symts_from_fst(ifst);

    Ok(fst_res)
}

fn single_shortest_path<W, F>(
    ifst: &F,
    distance: &mut Vec<W>,
    f_parent: &mut Option<StateId>,
    parent: &mut Vec<Option<(StateId, usize)>>,
) -> Result<()>
where
    W: Semiring,
    F: ExpandedFst<W>,
{
    parent.clear();
    *f_parent = None;
    let start = ifst.start();
    if start.is_none() {
        return Ok(());
    }
    let mut enqueued = vec![];
    let mut queue = AutoQueue::new(ifst, None, &AnyTrFilter {})?;
    let source = unsafe { start.unsafe_unwrap() };
    let mut f_distance = W::zero();
    distance.clear();
    queue.clear();
    if !W::properties().contains(SemiringProperties::PATH | SemiringProperties::RIGHT_SEMIRING) {
        bail!(
            "SingleShortestPath: Weight needs to have the path property and be right distributive"
        )
    }
    while distance.len() < source {
        distance.push(W::zero());
        enqueued.push(false);
        parent.push(None)
    }
    distance.push(W::one());
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

        for (pos, tr) in unsafe { ifst.get_trs_unchecked(s).trs().iter().enumerate() } {
            while distance.len() <= tr.nextstate {
                distance.push(W::zero());
                enqueued.push(false);
                parent.push(None)
            }
            let nd = &mut distance[tr.nextstate];
            let weight = sd.times(&tr.weight)?;
            if *nd != nd.plus(&weight)? {
                *nd = nd.plus(&weight)?;
                parent[tr.nextstate] = Some((s, pos));
                if !enqueued[tr.nextstate] {
                    queue.enqueue(tr.nextstate);
                    enqueued[tr.nextstate] = true;
                } else {
                    queue.update(tr.nextstate);
                }
            }
        }
    }
    Ok(())
}

fn single_shortest_path_backtrace<W, FI, FO>(
    ifst: &FI,
    f_parent: &Option<StateId>,
    parent: &[Option<(StateId, usize)>],
) -> Result<FO>
where
    W: Semiring,
    FI: ExpandedFst<W>,
    FO: MutableFst<W>,
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
            let mut tr = ifst.get_trs(state)?.trs().iter().nth(pos).unwrap().clone();
            tr.nextstate = d_p.unwrap();
            ofst.add_tr(s_p.unwrap(), tr)?;
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

pub fn natural_less<W: Semiring>(w1: &W, w2: &W) -> Result<bool> {
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

fn n_shortest_path<W, FI, FO>(ifst: &FI, distance: &[W], nshortest: usize) -> Result<FO>
where
    W: Semiring,
    FI: MutableFst<W::ReverseWeight>,
    FO: MutableFst<W>,
{
    let mut ofst = FO::new();
    if nshortest == 0 {
        return Ok(ofst);
    }

    if !W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
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
            ofst.add_tr(ofst.start().unwrap(), Tr::new(0, 0, W::one(), state))?;
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
        for rarc in ifst.get_trs(p.0.unwrap())?.trs() {
            let mut tr: Tr<W> = Tr::new(
                rarc.ilabel,
                rarc.olabel,
                rarc.weight.reverse_back()?,
                rarc.nextstate,
            );
            let weight = p.1.times(&tr.weight)?;
            let next = ofst.add_state();
            pairs.borrow_mut().push((Some(tr.nextstate), weight));
            tr.nextstate = state;
            ofst.add_tr(next, tr)?;
            heap.push(next);
        }
        let final_weight = ifst.final_weight(p.0.unwrap())?;
        if let Some(_final_weight) = final_weight {
            let r_final_weight: W = _final_weight.reverse_back()?;
            if !r_final_weight.is_zero() {
                let weight = p.1.times(&r_final_weight)?;
                let next = ofst.add_state();
                pairs.borrow_mut().push((None, weight));
                ofst.add_tr(next, Tr::new(0, 0, r_final_weight, state))?;
                heap.push(next);
            }
        }
    }

    connect(&mut ofst)?;

    Ok(ofst)
}

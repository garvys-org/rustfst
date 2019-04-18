use failure::Fallible;

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::{reverse, shortest_distance, Queue};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{Semiring, SemiringProperties};
use crate::StateId;

pub fn shortest_path<FI, FO>(ifst: &FI, nshortest: usize, unique: bool) -> Fallible<FO>
where
    FI: ExpandedFst + MutableFst,
    FO: MutableFst<W = FI::W>,
    FI::W: 'static,
{
    let queue = AutoQueue::new(ifst, None)?;

    if nshortest == 0 {
        return Ok(FO::new());
    }

    if nshortest == 1 {
        let mut parent = vec![];
        let mut f_parent = None;
        let mut distance = vec![];

        single_shortest_path(ifst, &mut distance, &mut f_parent, &mut parent)?;
        return single_shortest_path_backtrace(ifst, &f_parent, &parent);
    }

    if !FI::W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
        bail!("ShortestPath : Weight need to have the Path property and be distributive")
    }

    let distance = shortest_distance(ifst, false)?;

    let rfst: VectorFst<_> = reverse(ifst)?;
    let mut d = FI::W::zero();
    for rarc in rfst.arcs_iter(0)? {
        let state = rarc.nextstate - 1;
        if state < distance.len() {
            let rweight: FI::W = hack_convert_reverse_reverse(rarc.weight.reverse()?);
            d.plus_assign(rweight.times(&distance[state])?)?;
        }
    }
    unimplemented!()
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
    if ifst.start().is_none() {
        return Ok(());
    }
    let mut enqueued = vec![];
    let mut queue = AutoQueue::new(ifst, None)?;
    let source = ifst.start().unwrap();
    let mut final_seen = false;
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
        let s = queue.head().unwrap();
        queue.dequeue();
        enqueued[s] = false;
        let sd = distance[s].clone();

        if let Some(final_weight) = ifst.final_weight(s) {
            let plus = f_distance.plus(&sd.times(&final_weight)?)?;
            if f_distance != plus {
                f_distance = plus;
                *f_parent = Some(s);
            }
            final_seen = true;
        }

        for (pos, arc) in ifst.arcs_iter(s)?.enumerate() {
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
    parent: &Vec<Option<(StateId, usize)>>,
) -> Fallible<FO>
where
    FI: ExpandedFst + MutableFst,
    FO: MutableFst<W = FI::W>,
{
    let mut ofst = FO::new();
    let mut s_p = None;
    let mut d_p = None;

    let mut d : Option<StateId> = None;
    let mut nextstate = f_parent.clone();
    while let Some(state) = nextstate {
        d_p = s_p;
        s_p = Some(ofst.add_state());
        if d.is_none() {
            if let Some(final_weight) =ifst.final_weight(f_parent.unwrap()) {
                ofst.set_final(s_p.unwrap(), final_weight)?;
            }
        } else {
            let pos = parent[d.unwrap()].unwrap().1;
            let mut arc = ifst.arcs_iter(state)?.skip(pos).next().unwrap().clone();
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

mod add_super_final_state;
mod all_pairs_shortest_distance;
mod tr_map;
mod tr_sort;
mod tr_sum;
pub(crate) mod tr_unique;
mod closure;
pub mod compose;
mod concat;
mod condense;
mod connect;
mod determinize;
pub(crate) mod dfs_visit;
pub(crate) mod lazy_fst;
mod encode;
mod factor_weight;
mod fst_convert;
mod inversion;
mod isomorphic;
mod minimize;
mod partition;
mod projection;
mod push;
mod queue;
mod relabel_pairs;
mod replace;
mod reverse;
mod reweight;
mod rm_epsilon;
mod rm_final_epsilon;
mod shortest_distance;
mod shortest_path;
mod state_sort;
mod top_sort;
mod union;
mod weight_convert;

/// Module that provides different structures implementing the `Queue` trait.
pub mod queues;

/// Function objects to restrict which arcs are traversed in an FST.
pub mod tr_filters;

/// Module that provides structures implementing the `TrMapper` trait.
pub mod tr_mappers;

pub(crate) mod visitors;

#[allow(unused)]
pub(crate) mod cache;

#[allow(unused)]
pub(crate) mod factor_iterators;

/// Module that provides structures implementing the `WeightConverter` trait.
pub mod weight_converters;

/// Functions to compare / sort the Trs of an FST.
pub mod tr_compares {
    pub use super::tr_sort::{ilabel_compare, olabel_compare};
    pub use super::isomorphic::tr_compare;
}

pub use self::{
    add_super_final_state::add_super_final_state,
    all_pairs_shortest_distance::all_pairs_shortest_distance,
    tr_map::{tr_map, TrMapper, FinalTr, MapFinalAction},
    tr_sort::tr_sort,
    tr_sum::tr_sum,
    tr_unique::tr_unique,
    closure::{closure, ClosureFst, ClosureType},
    compose::compose,
    concat::{concat, ConcatFst},
    condense::condense,
    connect::connect,
    determinize::{determinize, determinize_with_distance, DeterminizeType},
    encode::{decode, encode},
    fst_convert::{fst_convert, fst_convert_from_ref},
    inversion::invert,
    isomorphic::isomorphic,
    minimize::minimize,
    projection::{project, ProjectType},
    push::{push, push_weights, PushType},
    queue::{Queue, QueueType},
    relabel_pairs::relabel_pairs,
    replace::{replace, ReplaceFst},
    reverse::reverse,
    reweight::{reweight, ReweightType},
    rm_epsilon::{rm_epsilon, RmEpsilonFst},
    rm_final_epsilon::rm_final_epsilon,
    shortest_distance::shortest_distance,
    shortest_path::shortest_path,
    state_sort::state_sort,
    top_sort::top_sort,
    union::{union, UnionFst},
    weight_convert::{weight_convert, WeightConverter},
};

pub use self::factor_weight::{
    factor_weight, FactorIterator, FactorWeightFst, FactorWeightOptions, FactorWeightType,
};

mod all_pairs_shortest_distance;
mod arc_map;
mod arc_sort;
mod arc_sum;
pub(crate) mod arc_unique;
mod closure;
mod composition;
mod concat;
mod connect;
mod determinize;
mod dfs;
mod encode;
mod factor_weight;
mod inversion;
mod isomorphic;
mod minimize;
mod partition;
mod projection;
mod queue;
mod relabel_pairs;
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
mod weight_pushing;

pub mod queues;

/// Function objects to restrict which arcs are traversed in an FST.
pub mod arc_filters;

/// Module that provide structures implementing the `ArcMapper` trait.
pub mod arc_mappers;

#[allow(unused)]
pub(crate) mod cache;

#[allow(unused)]
pub(crate) mod factor_iterators;

/// Module that provide structures implementing the `WeightConverter` trait.
pub mod weight_converters;

/// Functions to compare / sort the Arcs of an FST.
pub mod arc_compares {
    pub use super::arc_sort::{ilabel_compare, olabel_compare};
    pub use super::isomorphic::arc_compare;
}

pub use self::{
    all_pairs_shortest_distance::all_pairs_shortest_distance,
    arc_map::{arc_map, ArcMapper, FinalArc, MapFinalAction},
    arc_sort::arc_sort,
    arc_sum::arc_sum,
    arc_unique::arc_unique,
    closure::{closure_plus, closure_star},
    composition::compose,
    concat::concat,
    connect::connect,
    determinize::{determinize, determinize_with_distance, DeterminizeType},
    dfs::{dfs, find_strongly_connected_components},
    encode::{decode, encode},
    inversion::invert,
    isomorphic::isomorphic,
    minimize::minimize,
    projection::{project, ProjectType},
    queue::{Queue, QueueType},
    relabel_pairs::relabel_pairs,
    reverse::reverse,
    reweight::{reweight, ReweightType},
    rm_epsilon::rm_epsilon,
    rm_final_epsilon::rm_final_epsilon,
    shortest_distance::{shortest_distance, single_source_shortest_distance},
    shortest_path::shortest_path,
    state_sort::state_sort,
    top_sort::top_sort,
    union::union,
    weight_convert::{weight_convert, WeightConverter},
    weight_pushing::push_weights,
};

#[allow(unused)]
pub(crate) use self::factor_weight::{
    factor_weight, FactorIterator, FactorWeightOptions, FactorWeightType,
};

#[allow(unused)]
pub(crate) use self::partition::Partition;

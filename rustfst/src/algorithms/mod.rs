pub use self::{
    add_super_final_state::add_super_final_state,
    all_pairs_shortest_distance::all_pairs_shortest_distance,
    condense::condense,
    connect::connect,
    fst_convert::{fst_convert, fst_convert_from_ref},
    inversion::invert,
    isomorphic::{isomorphic, isomorphic_with_config, IsomorphicConfig},
    minimize::{acceptor_minimize, minimize, minimize_with_config, MinimizeConfig},
    optimize::optimize,
    projection::{project, ProjectType},
    push::{
        push, push_weights, push_weights_with_config, push_with_config, PushConfig, PushType,
        PushWeightsConfig,
    },
    queue::{Queue, QueueType},
    relabel_pairs::relabel_pairs,
    reverse::reverse,
    reweight::{reweight, ReweightType},
    rm_final_epsilon::rm_final_epsilon,
    shortest_distance::{shortest_distance, shortest_distance_with_config, ShortestDistanceConfig},
    shortest_path::{shortest_path, shortest_path_with_config, ShortestPathConfig},
    state_sort::state_sort,
    top_sort::top_sort,
    tr_map::{tr_map, FinalTr, MapFinalAction, TrMapper},
    tr_sort::tr_sort,
    tr_sum::tr_sum,
    tr_unique::tr_unique,
    weight_convert::{weight_convert, WeightConverter},
};

mod add_super_final_state;
mod all_pairs_shortest_distance;
/// Functions to compute Kleene closure (star or plus) of an FST.
pub mod closure;
#[allow(clippy::type_complexity)]
/// Functions to compose FSTs.
pub mod compose;
/// Functions to concatenate FSTs.
pub mod concat;
mod condense;
mod connect;
/// Functions to determinize FSTs.
pub mod determinize;
pub(crate) mod dfs_visit;
/// Functions to encode FSTs as FSAs and vice versa.
pub mod encode;
/// Functions to factor various weight types.
pub mod factor_weight;
mod fst_convert;
mod inversion;
mod isomorphic;
mod minimize;
mod optimize;
mod partition;
mod projection;
mod push;
mod queue;

/// Functions to randomly generate paths through an Fst. A static and a delayed version are available.
pub mod randgen;
mod relabel_pairs;
/// Functions for lazy replacing transitions in an FST.
pub mod replace;
mod reverse;
mod reweight;

/// Functions to remove epsilon transitions from an Fst. A static and a delayed version are available.
pub mod rm_epsilon;
mod rm_final_epsilon;
mod shortest_distance;
mod shortest_path;
mod state_sort;
mod top_sort;
mod tr_map;
mod tr_sort;
mod tr_sum;
pub(crate) mod tr_unique;
/// Functions to compute the union of FSTs.
pub mod union;
mod weight_convert;

/// Module providing different structures implementing the `Queue` trait.
pub mod queues;

/// Function objects to restrict which trs are traversed in an FST.
pub mod tr_filters;

/// Module that provides structures implementing the `TrMapper` trait.
pub mod tr_mappers;

pub(crate) mod visitors;

/// Module providing structures implementing the `WeightConverter` trait.
pub mod weight_converters;

/// Functions to compare / sort the Trs of an FST.
pub mod tr_compares {
    pub use super::isomorphic::tr_compare;
    pub use super::tr_sort::{ILabelCompare, OLabelCompare, TrCompare};
}

/// Module providing the necessary functions to implement a new Delayed Fst.
pub mod lazy;

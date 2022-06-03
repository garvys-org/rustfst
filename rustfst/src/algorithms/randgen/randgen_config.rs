use crate::algorithms::randgen::TrSelector;

/// Configuration struct for random path generation.
pub struct RandGenConfig<S: TrSelector> {
    /// How an arc is selected at a state.
    pub selector: S,
    /// Maximum path length.
    pub max_length: usize,
    /// Number of paths to generate.
    pub npath: usize,
    ///Is the output tree weighted by path count, or is it just an unweighted DAG?
    pub weighted: bool,
    /// Remove total weight when output is weighted?
    pub remove_total_weight: bool,
}

impl<S: TrSelector> RandGenConfig<S> {
    pub fn new(selector: S) -> Self {
        Self {
            selector,
            max_length: usize::MAX,
            npath: 1,
            weighted: false,
            remove_total_weight: false,
        }
    }

    pub fn with_max_length(self, max_length: usize) -> Self {
        Self { max_length, ..self }
    }

    pub fn with_npath(self, npath: usize) -> Self {
        Self { npath, ..self }
    }

    pub fn with_weighted(self, weighted: bool) -> Self {
        Self { weighted, ..self }
    }

    pub fn with_remove_total_weight(self, remove_total_weight: bool) -> Self {
        Self {
            remove_total_weight,
            ..self
        }
    }
}

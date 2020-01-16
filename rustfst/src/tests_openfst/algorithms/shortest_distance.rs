use std::f32;

use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::shortest_distance;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::semirings::{Semiring, SerializableSemiring};

use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShorestDistanceOperationResult {
    reverse: bool,
    result: Vec<String>,
}

#[derive(Debug)]
pub struct ShortestDistanceTestData<W> {
    reverse: bool,
    result: Vec<W>,
}

impl ShorestDistanceOperationResult {
    pub fn parse<W: SerializableSemiring + Semiring<Type = f32>>(
        &self,
    ) -> ShortestDistanceTestData<W> {
        let inf = "Infinity".to_string();
        let r = self
            .result
            .iter()
            .map(|v| {
                if v == &inf {
                    f32::INFINITY
                } else {
                    v.parse().unwrap()
                }
            })
            .map(|v| W::new(v))
            .collect();
        ShortestDistanceTestData {
            result: r,
            reverse: self.reverse,
        }
    }
}

pub fn test_shortest_distance<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst,
    F::W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    for data in &test_data.shortest_distance {
        let distance = shortest_distance(&test_data.raw, data.reverse)?;
        assert_eq!(
            data.result, distance,
            "Test failing for ShortestDistance with reverse={}",
            data.reverse
        );
    }
    Ok(())
}

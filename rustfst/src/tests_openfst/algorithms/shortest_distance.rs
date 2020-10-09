use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::shortest_distance;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
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
    pub fn parse<W: SerializableSemiring>(&self) -> ShortestDistanceTestData<W> {
        let r = self
            .result
            .iter()
            .map(|v| {
                let (_, w) = W::parse_text(v.as_str()).unwrap();
                w
            })
            .collect();
        ShortestDistanceTestData {
            result: r,
            reverse: self.reverse,
        }
    }
}

pub fn test_shortest_distance<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W>,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    for data in &test_data.shortest_distance {
        let distance = shortest_distance(&test_data.raw, data.reverse)?;
        assert_eq!(
            data.result,
            // .iter()
            // .map(|w| w.quantize(KDELTA).unwrap())
            // .collect_vec(),
            distance,
            // .iter()
            // .map(|w| w.quantize(KDELTA).unwrap())
            // .collect_vec(),
            "Test failing for ShortestDistance with reverse={}",
            data.reverse
        );
    }
    Ok(())
}

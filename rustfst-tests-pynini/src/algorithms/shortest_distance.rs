use std::f32;

use failure::{Fallible, bail};
use serde_derive::{Deserialize, Serialize};

use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;
use rustfst::semirings::WeightQuantize;
use rustfst::algorithms::shortest_distance;

use crate::TestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShorestDistanceOperationResult {
    reverse: bool,
    result: Vec<String>,
}

#[derive(Debug)]
pub struct ShortestDistanceTestData<W: Semiring<Type=f32>> {
    reverse: bool,
    result: Vec<W>
}

impl ShorestDistanceOperationResult {
    pub fn parse<W: Semiring<Type=f32>>(&self) -> ShortestDistanceTestData<W> {
        let inf = "Infinity".to_string();
        let r = self.result.iter().map(|v| {
            if v == &inf {
                f32::INFINITY
            } else {
                v.parse().unwrap()
            }
        }).map(|v| W::new(v)).collect();
        ShortestDistanceTestData {result: r, reverse: self.reverse}
    }
}

pub fn test_shortest_distance<F>(test_data: &TestData<F>) -> Fallible<()>
    where
        F: TextParser + MutableFst,
        F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    for data in &test_data.shortest_distance {
        println!("reverse = {:?}", data.reverse);
        let distance = shortest_distance(&test_data.raw, data.reverse)?;
        assert_eq!(data.result, distance, "Test failing for ShortestDistance with reverse={}", data.reverse);
    }
    Ok(())
}
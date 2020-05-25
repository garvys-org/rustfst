use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::algorithms::Queue;
use crate::fst_impls::VectorFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;
use crate::StateId;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QueueOperation {
    op_type: String,
    state: StateId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueOperationResult {
    result: Vec<QueueOperation>,
}

pub fn test_queue<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let mut queue = AutoQueue::new(&test_data.raw, None, &AnyTrFilter {})?;

    for op in &test_data.queue.result {
        match op.op_type.as_str() {
            "enqueue" => {
                queue.enqueue(op.state);
            }
            "dequeue" => {
                assert!(!queue.is_empty());
                assert_eq!(op.state, queue.head().unwrap());
                queue.dequeue();
            }
            _ => panic!("Unknown op_type : {:?}", op.op_type.as_str()),
        };
    }

    assert!(queue.is_empty());
    Ok(())
}

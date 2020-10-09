use std::fmt::Display;
use std::marker::PhantomData;
use std::path::Path;

use anyhow::{format_err, Context, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::encode::{decode, encode, EncodeType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodeOperationResult {
    encode_labels: bool,
    encode_weights: bool,
    result_path: String,
}

pub struct EncodeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub encode_type: EncodeType,
    pub result: F,
    w: PhantomData<W>,
}

impl EncodeOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> EncodeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        EncodeTestData {
            encode_type: EncodeType::from_bools(self.encode_weights, self.encode_labels).unwrap(),
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_encode_decode<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    for encode_test_data in &test_data.encode_decode {
        let mut fst_encoded = test_data.raw.clone();
        let encode_table =
            encode(&mut fst_encoded, encode_test_data.encode_type).with_context(|| {
                format_err!(
                    "Error when running test_encode_decode with parameters encode_type={:?}.",
                    encode_test_data.encode_type
                )
            })?;
        decode(&mut fst_encoded, encode_table)?;
        test_eq_fst(
            &encode_test_data.result,
            &fst_encoded,
            format!(
                "Encode/Decode with encode_type={:?}",
                encode_test_data.encode_type
            ),
        );
    }
    Ok(())
}

pub fn test_encode<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    for encode_test_data in &test_data.encode {
        // println!("Encode labels = {:?} Encode weights = {:?}", encode_test_data.encode_labels, encode_test_data.encode_weights);
        let mut fst_encoded = test_data.raw.clone();
        encode(&mut fst_encoded, encode_test_data.encode_type).with_context(|| {
            format_err!(
                "Error when running test_encode with parameters encode_type={:?}.",
                encode_test_data.encode_type
            )
        })?;
        test_eq_fst(
            &encode_test_data.result,
            &fst_encoded,
            format!("Encode encode_type = {:?}", encode_test_data.encode_type),
        );

        fst_encoded.compute_and_update_properties_all()?;
        if encode_test_data.encode_type.encode_labels() {
            assert!(fst_encoded.properties().contains(FstProperties::ACCEPTOR));
        }
        if encode_test_data.encode_type.encode_weights() {
            assert!(fst_encoded.properties().contains(FstProperties::UNWEIGHTED));
        }
    }
    Ok(())
}

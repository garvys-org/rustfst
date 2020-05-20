use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::{format_err, Context, Result};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use crate::algorithms::encode::{decode, encode};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodeOperationResult {
    encode_labels: bool,
    encode_weights: bool,
    result: String,
}

pub struct EncodeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub encode_labels: bool,
    pub encode_weights: bool,
    pub result: F,
    w: PhantomData<W>,
}

impl EncodeOperationResult {
    pub fn parse<W, F>(&self) -> EncodeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        EncodeTestData {
            encode_weights: self.encode_weights,
            encode_labels: self.encode_labels,
            result: F::from_text_string(self.result.as_str()).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_encode_decode<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring,
{
    for encode_test_data in &test_data.encode_decode {
        let mut fst_encoded = test_data.raw.clone();
        let encode_table = encode(&mut fst_encoded, encode_test_data.encode_labels, encode_test_data.encode_weights)
            .with_context(|| format_err!(
            "Error when running test_encode_decode with parameters encode_labels={:?} and encode_weights={:?}.",
            encode_test_data.encode_labels, encode_test_data.encode_weights))?;
        decode(&mut fst_encoded, encode_table)?;
        assert_eq!(
            encode_test_data.result,
            fst_encoded,
            "{}",
            error_message_fst!(
                encode_test_data.result,
                fst_encoded,
                format!(
                    "Encode/Decode with encode_labels={:?} and encode_weights={:?}",
                    encode_test_data.encode_labels, encode_test_data.encode_weights
                )
            )
        );
    }
    Ok(())
}

pub fn test_encode<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring,
{
    for encode_test_data in &test_data.encode {
        // println!("Encode labels = {:?} Encode weights = {:?}", encode_test_data.encode_labels, encode_test_data.encode_weights);
        let mut fst_encoded = test_data.raw.clone();
        encode(&mut fst_encoded, encode_test_data.encode_labels, encode_test_data.encode_weights)
            .with_context(|| format_err!(
            "Error when running test_encode with parameters encode_labels={:?} and encode_weights={:?}.",
            encode_test_data.encode_labels, encode_test_data.encode_weights))?;
        if encode_test_data.encode_labels {
            assert!(fst_encoded.properties()?.contains(FstProperties::ACCEPTOR));
        }
        if encode_test_data.encode_weights {
            assert!(fst_encoded
                .properties()?
                .contains(FstProperties::UNWEIGHTED));
        }
        assert_eq!(
            encode_test_data.result,
            fst_encoded,
            "{}",
            error_message_fst!(
                encode_test_data.result,
                fst_encoded,
                format!(
                    "Encode encode_labels = {} encode_weights = {}",
                    encode_test_data.encode_labels, encode_test_data.encode_weights
                )
            )
        );
    }
    Ok(())
}

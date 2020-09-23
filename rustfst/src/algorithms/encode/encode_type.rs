use anyhow::Result;

#[derive(PartialEq, Ord, PartialOrd, Eq, Clone, Copy, Debug)]
pub enum EncodeType {
    EncodeWeights,
    EncodeLabels,
    EncodeWeightsAndLabels,
}

impl EncodeType {
    pub fn from_bools(encode_weights: bool, encode_labels: bool) -> Result<Self> {
        match (encode_weights, encode_labels) {
            (true, true) => Ok(EncodeType::EncodeWeightsAndLabels),
            (true, false) => Ok(EncodeType::EncodeWeights),
            (false, true) => Ok(EncodeType::EncodeLabels),
            (false, false) => bail!(
                "Encode type with encode_weights=false and encode_labels=false is not supported"
            ),
        }
    }

    pub fn encode_weights(&self) -> bool {
        *self == EncodeType::EncodeWeights || *self == EncodeType::EncodeWeightsAndLabels
    }

    pub fn encode_labels(self) -> bool {
        self == EncodeType::EncodeLabels || self == EncodeType::EncodeWeightsAndLabels
    }
}

use crate::fst_properties::FstProperties;

/// Both bits are set iff one bit is set.
pub fn known_properties(props: FstProperties) -> FstProperties {
    FstProperties::binary_properties()
        | (props & FstProperties::trinary_properties())
        | ((props & FstProperties::pos_trinary_properties()) << 1)
        | ((props & FstProperties::neg_trinary_properties()) >> 1)
}

impl FstProperties {
    /// Check that all the `props` passed as parameter are known.
    pub fn knows(self, props: FstProperties) -> bool {
        let known = known_properties(self);
        known.contains(props)
    }
}

/// Tests compatibility between two sets of properties.
pub fn compat_properties(props1: FstProperties, props2: FstProperties) -> bool {
    let known_props1 = known_properties(props1);
    let known_props2 = known_properties(props2);
    let known_props = known_props1 & known_props2;
    let incompat_props = (props1 & known_props) ^ (props2 & known_props);
    incompat_props.is_empty()
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_known_properties() -> Result<()> {
        let props = FstProperties::ACCEPTOR | FstProperties::ACCESSIBLE;
        let ref_known_props = FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE;

        let known_props = known_properties(props);

        assert_eq!(ref_known_props, known_props);

        Ok(())
    }
}

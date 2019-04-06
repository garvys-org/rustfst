use crate::properties::FstProperties;

pub fn known_properties(props: FstProperties) -> FstProperties {
    props | ((props & FstProperties::POS_PROPERTIES) << 1) | ((props & FstProperties::NEG_PROPERTIES) >> 1)
}

pub fn compat_properties(props1: FstProperties,props2: FstProperties) -> bool {
    let known_props1 = known_properties(props1);
    let known_props2 = known_properties(props2);
    let known_props = known_props1 & known_props2;
    let incompat_props = (props1 & known_props) ^ (props2 & known_props);
    incompat_props.is_empty()
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Fallible;

    #[test]
    fn test_known_properties() -> Fallible<()> {

        let props = FstProperties::ACCEPTOR | FstProperties::ACCESSIBLE;
        let ref_known_props = FstProperties::ACCEPTOR |FstProperties::NOT_ACCEPTOR | FstProperties::ACCESSIBLE | FstProperties::NOT_ACCESSIBLE;

        let known_props = known_properties(props);

        assert_eq!(ref_known_props, known_props);

        Ok(())
    }
}
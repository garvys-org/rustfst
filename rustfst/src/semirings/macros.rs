macro_rules! test_semiring_serializable {
    ($sem_name:  tt, $semiring: ty, $( $weight: expr )* ) => {

        #[cfg(test)]
        mod $sem_name {
            use super::*;

            #[test]
            fn test_serializable_binary() -> Result<()> {
                for weight in &[ $( $weight ),* ] {
                    let weight = weight.clone();
                    let mut serialization = vec![];

                    weight.write_binary(&mut serialization)?;

                    let (_, weight_deserialized) = <$semiring>::parse_binary(serialization.as_slice())
                        .map_err(|e| format_err!("Can't parse weight : {:?}", e))?;

                    assert_eq!(weight_deserialized, weight);
                }

                Ok(())
            }

            #[test]
            fn test_serializable_text() -> Result<()> {
                for weight in &[ $( $weight ),* ] {
                    let weight = weight.clone();
                    let serialization = format!("{}", weight);

                    let (_, weight_deserialized) = <$semiring>::parse_text(serialization.as_str())
                        .map_err(|e| format_err!("Can't parse weight : {:?}", e))?;

                    assert_eq!(weight_deserialized, weight);
                }

                Ok(())
            }
        }

    };
}

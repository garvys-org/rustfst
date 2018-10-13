#[cfg(test)]
use fst_traits::Fst;

#[cfg(test)]
pub(crate) trait TestFst {
    type F: Fst;
    fn get_fst() -> Self::F;
    fn get_name() -> String;
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct TestFstData<F: Fst> {
    pub(crate) fst: F,
    pub(crate) name: String,
}

macro_rules! gen_test_fst {
    ($struct_name: ty, $fst_code:block, $name: expr) => {
        #[cfg(test)]
        impl $struct_name {
            pub(crate) fn new() -> Self {
                Self {}
            }
        }

        #[cfg(test)]
        impl TestFst for $struct_name {
            type F = VectorFst<IntegerWeight>;

            fn get_fst() -> <Self as TestFst>::F {
                $fst_code
            }

            fn get_name() -> String {
                String::from($name)
            }
        }

        #[cfg(test)]
        impl Into<TestFstData<<Self as TestFst>::F>> for $struct_name {
            fn into(self) -> TestFstData<<Self as TestFst>::F> {
                TestFstData {
                    fst: Self::get_fst(),
                    name: Self::get_name(),
                }
            }
        }
    };
}

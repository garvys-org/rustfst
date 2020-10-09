#[macro_use]
mod semiring;

#[macro_use]
mod macros;

mod boolean_weight;
mod gallic_weight;
mod integer_weight;
mod log_weight;
mod power_weight;
mod probability_weight;
mod product_weight;
mod string_variant;
mod string_weight;
mod tropical_weight;
mod union_weight;
pub(crate) mod utils_float;

pub use self::boolean_weight::BooleanWeight;
pub use self::gallic_weight::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
};
pub use self::integer_weight::IntegerWeight;
pub use self::log_weight::LogWeight;
pub use self::probability_weight::ProbabilityWeight;
pub use self::product_weight::ProductWeight;
pub use self::semiring::{
    CompleteSemiring, DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
pub(crate) use self::string_variant::StringWeightVariant;
pub use self::string_weight::{
    StringType, StringWeightLeft, StringWeightRestrict, StringWeightRight,
};
pub use self::tropical_weight::TropicalWeight;
pub use self::union_weight::{UnionWeight, UnionWeightOption};

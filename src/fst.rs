use arc::Arc;
use StateId;
use semirings::Semiring;

pub trait Fst<W: Semiring> {
    type Arc: Arc<W>;
    // type Iter: Iterator<Item=Self::Arc>;
    //type Symtab: IntoIterator<Item=String>;
    fn start(&self) -> Option<StateId>;
    fn final_weight(&self, &StateId) -> Option<W>;
    // fn arc_iter(&self, StateId) -> Self::Iter;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
    fn is_final(&self, &StateId) -> bool;
}
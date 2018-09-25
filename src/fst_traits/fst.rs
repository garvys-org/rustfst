use arc::Arc;
use semirings::Semiring;
use StateId;

pub trait CoreFst {
    type W: Semiring;
    //type Symtab: IntoIterator<Item=String>;
    fn start(&self) -> Option<StateId>;
    fn final_weight(&self, &StateId) -> Option<<Self as CoreFst>::W>;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
    fn is_final(&self, &StateId) -> bool;
    fn num_arcs(&self) -> usize;
}

pub trait Fst: CoreFst + PartialEq + for<'a> ArcIterator<'a> + for<'b> StateIterator<'b> {}

pub trait StateIterator<'a> {
    type Iter: Iterator<Item = StateId>;
    fn states_iter(&'a self) -> Self::Iter;
}

pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    type Iter: Iterator<Item = &'a Arc<Self::W>>;
    fn arcs_iter(&'a self, &StateId) -> Self::Iter;
}

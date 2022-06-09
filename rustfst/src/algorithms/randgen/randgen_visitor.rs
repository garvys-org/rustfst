use anyhow::Result;

use crate::algorithms::dfs_visit::Visitor;
use crate::fst_traits::{Fst, MutableFst};
use crate::{Semiring, StateId, Tr};

pub struct RandGenVisitor<'a, W: Semiring, FI: Fst<W>, FO: MutableFst<W>> {
    ofst: FO,
    ifst: Option<&'a FI>,
    path: Vec<Tr<W>>,
}

impl<'a, W: Semiring, FI: Fst<W>, FO: MutableFst<W>> RandGenVisitor<'a, W, FI, FO> {
    pub fn into_output_fst(self) -> FO {
        self.ofst
    }
}

impl<'a, W: Semiring, FI: Fst<W>, FO: MutableFst<W>> RandGenVisitor<'a, W, FI, FO> {
    pub fn new() -> Self {
        Self {
            ifst: None,
            ofst: FO::new(),
            path: vec![],
        }
    }

    fn output_path(&mut self) -> Result<()> {
        if self.ofst.start().is_none() {
            let start = self.ofst.add_state();
            self.ofst.set_start(start)?;
        }
        let mut src = self.ofst.start().unwrap();
        for i in 0..self.path.len() {
            let dest = self.ofst.add_state();
            let tr = Tr::new(self.path[i].ilabel, self.path[i].olabel, W::one(), dest);
            self.ofst.add_tr(src, tr)?;
            src = dest;
        }
        self.ofst.set_final(src, W::one())?;
        Ok(())
    }
}

impl<'a, W: Semiring, FI: Fst<W>, FO: MutableFst<W>> Visitor<'a, W, FI>
    for RandGenVisitor<'a, W, FI, FO>
{
    fn init_visit(&mut self, fst: &'a FI) {
        self.ifst = Some(fst);
        self.ofst.del_all_states();
        self.ofst.set_symts_from_fst(fst);
        self.path.clear();
    }

    fn init_state(&mut self, _s: StateId, _root: StateId) -> bool {
        true
    }

    fn tree_tr(&mut self, _s: StateId, tr: &Tr<W>) -> bool {
        if !self.ifst.unwrap().is_final(tr.nextstate).unwrap() {
            self.path.push(tr.clone());
        } else {
            self.output_path().unwrap();
        }
        true
    }

    fn back_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        panic!("RandGenVisitor: cyclic input");
    }

    fn forward_or_cross_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        self.output_path().unwrap();
        true
    }

    fn finish_state(&mut self, s: StateId, parent: Option<StateId>, _tr: Option<&Tr<W>>) {
        if parent.is_some() && !self.ifst.unwrap().is_final(s).unwrap() {
            self.path.pop();
        }
    }

    fn finish_visit(&mut self) {}
}

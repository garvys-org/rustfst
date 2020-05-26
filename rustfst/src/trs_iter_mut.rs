use std::ops::Index;

use anyhow::Result;

use crate::{EPS_LABEL, Label, Semiring, StateId, Tr};
use crate::fst_properties::FstProperties;

pub struct TrsIterMut<'a, W: Semiring> {
    trs: &'a mut Vec<Tr<W>>,
    properties: &'a mut FstProperties,
    idx: usize,
}

impl<'a, W: Semiring> Index<usize> for TrsIterMut<'a, W> {
    type Output = Tr<W>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trs[index]
    }
}

impl<'a, W: Semiring> TrsIterMut<'a, W> {
    pub(crate) fn new(trs: &'a mut Vec<Tr<W>>, properties: &'a mut FstProperties) -> Self {
        Self {
            trs,
            properties,
            idx: 0,
        }
    }

    pub fn get(&self, idx: StateId) -> Option<&Tr<W>> {
        self.trs.get(idx)
    }

    pub unsafe fn get_unchecked(&self, idx: StateId) -> &Tr<W> {
        self.trs.get_unchecked(idx)
    }

    pub fn len(&self) -> usize {
        self.trs.len()
    }

    pub fn set_ilabel(&mut self, ilabel: Label) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(self.idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            ilabel,
            old_tr.olabel,
        );
        old_tr.ilabel = ilabel;
        Ok(())
    }

    pub fn set_olabel(&mut self, olabel: Label) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(self.idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            old_tr.ilabel,
            olabel,
        );
        old_tr.olabel = olabel;
        Ok(())
    }

    pub fn set_weight(&mut self, weight: W) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(self.idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties =
            compute_new_properties_weights(*self.properties, &old_tr.weight, &weight);
        old_tr.weight = weight;
        Ok(())
    }

    pub fn set_nextstate(&mut self, nextstate: StateId) -> Result<()> {
        keep_only_relevant_properties(self.properties);
        self.trs
            .get_mut(self.idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?
            .nextstate = nextstate;
        Ok(())
    }

    pub fn set_tr(&mut self, tr: Tr<W>) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(self.idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_all(*self.properties, old_tr, &tr);
        *old_tr = tr;
        Ok(())
    }

    pub unsafe fn set_ilabel_unchecked(&mut self, ilabel: Label) {
        let old_tr = self.trs.get_unchecked_mut(self.idx);
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            ilabel,
            old_tr.olabel,
        );
        old_tr.ilabel = ilabel;
    }

    pub unsafe fn set_olabel_unchecked(&mut self, olabel: Label) {
        let old_tr = self.trs.get_unchecked_mut(self.idx);
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            old_tr.ilabel,
            olabel,
        );
        old_tr.olabel = olabel;
    }

    pub unsafe fn set_weight_unchecked(&mut self, weight: W) {
        let old_tr = self.trs.get_unchecked_mut(self.idx);
        *self.properties =
            compute_new_properties_weights(*self.properties, &old_tr.weight, &weight);
        old_tr.weight = weight;
    }

    pub unsafe fn set_nextstate_unchecked(&mut self, nextstate: StateId) {
        keep_only_relevant_properties(self.properties);
        self.trs.get_unchecked_mut(self.idx).nextstate = nextstate;
    }

    pub unsafe fn set_tr_unchecked(&mut self, tr: Tr<W>) {
        let old_tr = self.trs.get_unchecked_mut(self.idx);
        *self.properties = compute_new_properties_all(*self.properties, old_tr, &tr);
        *old_tr = tr;
    }
}

fn update_properties_labels(
    properties: &mut FstProperties,
    old_ilabel: Label,
    old_olabel: Label,
    new_ilabel: Label,
    new_olabel: Label,
) {
    if old_ilabel != old_olabel {
        *properties &= !FstProperties::NOT_ACCEPTOR;
    }
    if old_ilabel == EPS_LABEL {
        *properties &= !FstProperties::I_EPSILONS;
        if old_olabel == EPS_LABEL {
            *properties &= !FstProperties::EPSILONS;
        }
    }
    if old_olabel == EPS_LABEL {
        *properties &= !FstProperties::O_EPSILONS;
    }

    if new_ilabel != new_olabel {
        *properties |= FstProperties::NOT_ACCEPTOR;
        *properties &= !FstProperties::ACCEPTOR;
    }
    if new_ilabel == EPS_LABEL {
        *properties |= FstProperties::I_EPSILONS;
        *properties &= !FstProperties::NO_I_EPSILONS;
        if new_olabel == EPS_LABEL {
            *properties |= FstProperties::EPSILONS;
            *properties &= !FstProperties::NO_EPSILONS;
        }
    }
    if new_olabel == EPS_LABEL {
        *properties |= FstProperties::O_EPSILONS;
        *properties &= !FstProperties::NO_O_EPSILONS;
    }
}

fn update_properties_weights<W: Semiring>(
    properties: &mut FstProperties,
    old_weight: &W,
    new_weight: &W,
) {
    if !old_weight.is_zero() && !old_weight.is_one() {
        *properties &= !FstProperties::WEIGHTED;
    }
    if !new_weight.is_zero() && !new_weight.is_one() {
        *properties |= FstProperties::WEIGHTED;
        *properties &= !FstProperties::UNWEIGHTED;
    }
}

fn keep_only_relevant_properties(properties: &mut FstProperties) {
    *properties &= FstProperties::set_arc_properties()
        | FstProperties::ACCEPTOR
        | FstProperties::NOT_ACCEPTOR
        | FstProperties::EPSILONS
        | FstProperties::NO_EPSILONS
        | FstProperties::I_EPSILONS
        | FstProperties::NO_I_EPSILONS
        | FstProperties::O_EPSILONS
        | FstProperties::NO_O_EPSILONS
        | FstProperties::WEIGHTED
        | FstProperties::UNWEIGHTED;
}

fn compute_new_properties_all<W: Semiring>(
    mut properties: FstProperties,
    old_tr: &Tr<W>,
    new_tr: &Tr<W>,
) -> FstProperties {
    update_properties_labels(
        &mut properties,
        old_tr.ilabel,
        old_tr.olabel,
        new_tr.ilabel,
        new_tr.olabel,
    );
    update_properties_weights(&mut properties, &old_tr.weight, &new_tr.weight);
    keep_only_relevant_properties(&mut properties);
    properties
}

fn compute_new_properties_labels(
    mut properties: FstProperties,
    old_ilabel: Label,
    old_olabel: Label,
    new_ilabel: Label,
    new_olabel: Label,
) -> FstProperties {
    update_properties_labels(
        &mut properties,
        old_ilabel,
        old_olabel,
        new_ilabel,
        new_olabel,
    );
    keep_only_relevant_properties(&mut properties);
    properties
}

fn compute_new_properties_weights<W: Semiring>(
    mut properties: FstProperties,
    old_weight: &W,
    new_weight: &W,
) -> FstProperties {
    update_properties_weights(&mut properties, old_weight, new_weight);
    keep_only_relevant_properties(&mut properties);
    properties
}

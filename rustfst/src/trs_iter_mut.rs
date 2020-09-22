use std::ops::Index;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::{Label, Semiring, StateId, Tr, EPS_LABEL};

pub struct TrsIterMut<'a, W: Semiring> {
    trs: &'a mut Vec<Tr<W>>,
    properties: &'a mut FstProperties,
    niepsilons: &'a mut usize,
    noepsilons: &'a mut usize,
}

impl<'a, W: Semiring> Index<usize> for TrsIterMut<'a, W> {
    type Output = Tr<W>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trs[index]
    }
}

// Use macro to avoid issue with the boroow checker. Indeed, If a function is used
// instead then the whole struct is mutably borrowed.
macro_rules! updt_nieps {
    ($s: expr, $old_ilabel: expr, $new_ilabel: expr) => {
        if $old_ilabel == EPS_LABEL {
            *$s.niepsilons -= 1;
        }
        if $new_ilabel == EPS_LABEL {
            *$s.niepsilons += 1;
        }
    };
}

macro_rules! updt_noeps {
    ($s: expr, $old_olabel: expr, $new_olabel: expr) => {
        if $old_olabel == EPS_LABEL {
            *$s.noepsilons -= 1;
        }
        if $new_olabel == EPS_LABEL {
            *$s.noepsilons += 1;
        }
    };
}

impl<'a, W: Semiring> TrsIterMut<'a, W> {
    pub(crate) fn new(
        trs: &'a mut Vec<Tr<W>>,
        properties: &'a mut FstProperties,
        niepsilons: &'a mut usize,
        noepsilons: &'a mut usize,
    ) -> Self {
        Self {
            trs,
            properties,
            niepsilons,
            noepsilons,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Tr<W>> {
        self.trs.get(idx)
    }

    /// Get a reference to the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn get_unchecked(&self, idx: usize) -> &Tr<W> {
        self.trs.get_unchecked(idx)
    }

    pub fn len(&self) -> usize {
        self.trs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trs.is_empty()
    }

    pub fn set_ilabel(&mut self, idx: usize, ilabel: Label) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            ilabel,
            old_tr.olabel,
        );
        updt_nieps!(self, old_tr.ilabel, ilabel);
        old_tr.ilabel = ilabel;
        Ok(())
    }

    pub fn set_olabel(&mut self, idx: usize, olabel: Label) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            old_tr.ilabel,
            olabel,
        );
        updt_noeps!(self, old_tr.olabel, olabel);
        old_tr.olabel = olabel;
        Ok(())
    }

    pub fn set_weight(&mut self, idx: usize, weight: W) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties =
            compute_new_properties_weights(*self.properties, &old_tr.weight, &weight);
        old_tr.weight = weight;
        Ok(())
    }

    pub fn set_nextstate(&mut self, idx: usize, nextstate: StateId) -> Result<()> {
        keep_only_relevant_properties(self.properties);
        self.trs
            .get_mut(idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?
            .nextstate = nextstate;
        Ok(())
    }

    pub fn set_tr(&mut self, idx: usize, tr: Tr<W>) -> Result<()> {
        let old_tr = self
            .trs
            .get_mut(idx)
            .ok_or_else(|| format_err!("set_tr shouldn't be called when the iteration is over"))?;
        *self.properties = compute_new_properties_all(*self.properties, old_tr, &tr);
        updt_nieps!(self, old_tr.ilabel, tr.ilabel);
        updt_noeps!(self, old_tr.olabel, tr.olabel);
        *old_tr = tr;
        Ok(())
    }

    /// Modify the ilabel of the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_ilabel_unchecked(&mut self, idx: usize, ilabel: Label) {
        let old_tr = self.trs.get_unchecked_mut(idx);
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            ilabel,
            old_tr.olabel,
        );
        updt_nieps!(self, old_tr.ilabel, ilabel);
        old_tr.ilabel = ilabel;
    }

    /// Modify the olabel of the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_olabel_unchecked(&mut self, idx: usize, olabel: Label) {
        let old_tr = self.trs.get_unchecked_mut(idx);
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            old_tr.ilabel,
            olabel,
        );
        updt_noeps!(self, old_tr.olabel, olabel);
        old_tr.olabel = olabel;
    }

    /// Modify the labels of the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_labels_unchecked(&mut self, idx: usize, ilabel: Label, olabel: Label) {
        let old_tr = self.trs.get_unchecked_mut(idx);
        *self.properties = compute_new_properties_labels(
            *self.properties,
            old_tr.ilabel,
            old_tr.olabel,
            ilabel,
            olabel,
        );
        updt_nieps!(self, old_tr.ilabel, ilabel);
        updt_noeps!(self, old_tr.olabel, olabel);
        old_tr.ilabel = ilabel;
        old_tr.olabel = olabel;
    }

    /// Modify the weight of the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_weight_unchecked(&mut self, idx: usize, weight: W) {
        let old_tr = self.trs.get_unchecked_mut(idx);
        *self.properties =
            compute_new_properties_weights(*self.properties, &old_tr.weight, &weight);
        old_tr.weight = weight;
    }

    /// Modify the nextstate of the  number `idx` `Tr` of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_nextstate_unchecked(&mut self, idx: usize, nextstate: StateId) {
        keep_only_relevant_properties(self.properties);
        self.trs.get_unchecked_mut(idx).nextstate = nextstate;
    }

    /// Modify the number `idx` element of the object.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `self.len() >= idx`
    pub unsafe fn set_tr_unchecked(&mut self, idx: usize, tr: Tr<W>) {
        let old_tr = self.trs.get_unchecked_mut(idx);
        *self.properties = compute_new_properties_all(*self.properties, old_tr, &tr);
        updt_nieps!(self, old_tr.ilabel, tr.ilabel);
        updt_noeps!(self, old_tr.olabel, tr.olabel);
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

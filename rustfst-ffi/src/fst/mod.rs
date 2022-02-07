pub mod vector_fst;

use crate::symbol_table::CSymbolTable;
use crate::tr::CTr;
use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{Fst, MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;
use rustfst::DrawingConfig;
use rustfst::Semiring;
use std::ffi::CStr;

type CVecFst = CFst;

/// Struct wrapping a boxable FST
pub struct CFst<F: Fst<TropicalWeight> = VectorFst<TropicalWeight>>(pub(crate) F);

impl<F: Fst<TropicalWeight>> RawPointerConverter<CFst<F>> for CFst<F> {
    fn into_raw_pointer(self) -> *const CFst<F> {
        Box::into_raw(Box::new(self)) as _
    }

    fn into_raw_pointer_mut(self) -> *mut CFst<F> {
        Box::into_raw(Box::new(self))
    }

    unsafe fn from_raw_pointer(input: *const CFst<F>) -> Result<Self, UnexpectedNullPointerError> {
        Self::from_raw_pointer_mut(input as *mut CFst<F>)
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CFst<F>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input))
        }
    }
}

/// Core FST methods
/// As defined in fst_traits

/// Returns the ID of the start state of the wFST if it exists else none
pub fn fst_start<F>(fst: *const CFst<F>, mut state: *mut CStateId) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        fst.start()
            .map(|it| unsafe { *state = it })
            .unwrap_or_else(|| state = std::ptr::null_mut());
        Ok(())
    })
}

/// Retrieves the final weight of a state (if the state is a final one).
pub fn fst_final_weight<F>(
    fst: *const CFst<F>,
    state_id: CStateId,
    mut final_weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        fst.final_weight(state_id)?
            .map(|it| unsafe { *final_weight = *it.value() })
            .unwrap_or_else(|| final_weight = std::ptr::null_mut());
        Ok(())
    })
}

/// Number of trs leaving a specific state in the wFST.
pub fn fst_num_trs<F>(
    fst: *const CFst<F>,
    state: CStateId,
    num_trs: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let res = fst.num_trs(state)?;
        unsafe { *num_trs = res };
        Ok(())
    })
}

/// Returns whether or not the state with identifier passed as parameters is a final state.
pub fn fst_is_final<F>(
    fst: *const CFst<F>,
    state: CStateId,
    is_final: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let res = fst.is_final(state)?;
        unsafe { *is_final = res as usize }
        Ok(())
    })
}

/// Returns whether or not the state with identifier passed as parameters is a final state.
pub fn fst_is_start<F>(
    fst: *const CFst<F>,
    state: CStateId,
    is_start: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let res = fst.is_start(state);
        unsafe { *is_start = res as usize }
        Ok(())
    })
}

// Missing methods for the CoreFst trait:
//- fn properties(&self) -> FstProperties;
//- fn get_trs(&self, state_id: StateId) -> Result<Self::TRS>;
//- fn num_input_epsilons(&self, state: StateId) -> Result<usize>;
//- fn num_output_epsilons(&self, state: StateId) -> Result<usize>;

/// Fst methods
/// As described in fst_traits

/// Retrieves the input `SymbolTable` associated to the Fst.
/// If no SymbolTable has been previously attached then a null pointer is returned.
pub fn fst_input_symbols<F>(
    fst: *const CFst<F>,
    mut input_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        fst.input_symbols()
            .map(|it| {
                let symt = CSymbolTable(it.clone());
                unsafe { *input_symt = symt }
            })
            .unwrap_or_else(|| input_symt = std::ptr::null_mut());
        Ok(())
    })
}

/// Retrieves the output `SymbolTable` associated to the Fst.
/// If no SymbolTable has been previously attached then a null pointer is returned.
pub fn fst_output_symbols<F>(
    fst: *const CFst<F>,
    mut output_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        fst.output_symbols()
            .map(|it| {
                let symt = CSymbolTable(it.clone());
                unsafe { *output_symt = symt }
            })
            .unwrap_or_else(|| output_symt = std::ptr::null_mut());
        Ok(())
    })
}

/// Attaches an input `SymbolTable` to the Fst.
/// The `SymbolTable` is not duplicated with the use of Arc.
pub fn fst_set_input_symbols<F>(fst: *mut CFst<F>, symt: *const CSymbolTable) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        let symt = get!(CSymbolTable, symt);
        fst.set_input_symbols(symt.clone());
        Ok(())
    })
}

/// Attaches an output `SymbolTable` to the Fst.
/// The `SymbolTable` is not duplicated with the use of Arc.
pub fn fst_set_output_symbols<F>(fst: *mut CFst<F>, symt: *const CSymbolTable) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        let symt = get!(CSymbolTable, symt);
        fst.set_output_symbols(symt.clone());
        Ok(())
    })
}

/// Removes the input symbol table from the Fst and retrieves it.
pub fn fst_unset_input_symbols<F>(fst: *mut CFst<F>) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        fst.take_input_symbols();
        Ok(())
    })
}

/// Removes the output symbol table from the Fst and retrieves it.
pub fn fst_unset_output_symbols<F>(fst: *mut CFst<F>) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        fst.take_output_symbols();
        Ok(())
    })
}

// Missing methods for Fst trait:
//- fn set_symts_from_fst<W2: Semiring, OF: Fst<W2>>(&mut self, other_fst: &OF);
//- fn final_states_iter(&self) -> FinalStatesIterator<W, Self>;

/// MutableFst methods
/// As described in fst_traits

/// Creates an empty wFST.
pub fn fst_new<F>(ptr: *mut *const CFst<F>) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = F::new();
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// The state with identifier `state_id` is now the start state.
/// If the `state_id` doesn't exist an error is raised.
pub fn fst_set_start<F>(fst: *mut CFst<F>, state: CStateId) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        fst.set_start(state)?;
        Ok(())
    })
}

/// The state with identifier `state_id` is now a final state with a weight `final_weight`.
/// If the `state_id` doesn't exist an error is raised.
pub fn fst_set_final<F>(
    fst: *mut CFst<F>,
    state: CStateId,
    weight: libc::c_float,
) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        fst.set_final(state, TropicalWeight::new(weight as f32))?;
        Ok(())
    })
}

/// Adds a new state to the current FST. The identifier of the new state is returned
pub fn fst_add_state<F>(fst: *mut CFst<F>, state: *mut CStateId) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        let res = fst.add_state();
        unsafe { *state = res }
        Ok(())
    })
}

/// Remove all the states in the FST. As a result, all the trs are also removed,
/// as well as the start state and all the fina states.
pub fn fst_delete_states<F>(fst: *mut CFst<F>) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        fst.del_all_states();
        Ok(())
    })
}

/// Adds a transition to the FST. The transition will start in the state `source`.
pub fn fst_add_tr<F>(fst: *mut CFst<F>, state: CStateId, tr: *const CTr) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? }.as_rust()?;
        fst.add_tr(state, tr)?;
        Ok(())
    })
}

// Missing methods for MutableFst trait:
//- fn add_states(&mut self, n: usize);
//- fn tr_iter_mut(&mut self, state: StateId) -> Result<TrsIterMut<W>>;
//- fn del_state(&mut self, state_id: StateId) -> Result<()>;
//- fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T) -> Result<()>;
//- fn emplace_tr<S: Into<W>>(&mut self,source: StateId,ilabel: Label,olabel: Label,weight: S,nextstate: StateId,) -> Result<()>;
//- fn delete_final_weight(&mut self, source: StateId) -> Result<()>;
//- fn delete_trs(&mut self, source: StateId) -> Result<()>;
//- fn pop_trs(&mut self, source: StateId) -> Result<Vec<Tr<W>>>;
//- fn take_final_weight(&mut self, state_id: StateId) -> Result<Option<W>>;
//- fn sort_trs_unchecked<F: Fn(&Tr<W>, &Tr<W>) -> Ordering>(&mut self, state: StateId, f: F);
//- fn closure(&mut self, closure_type: ClosureType);
//- fn tr_map<M: TrMapper<W>>(&mut self, mapper: &mut M) -> Result<()>;
//- fn set_properties(&mut self, props: FstProperties);
//- fn set_properties_with_mask(&mut self, props: FstProperties, mask: FstProperties);
//- fn compute_and_update_properties(&mut self, mask: FstProperties) -> Result<FstProperties>;
//- fn compute_and_update_properties_all(&mut self) -> Result<FstProperties>;

/// SerializableFst methods
/// As described in fst_traits

/// Loads an FST from a file in binary format.
pub fn fst_from_path<F>(ptr: *mut *const CFst<F>, path: *const libc::c_char) -> RUSTFST_FFI_RESULT
where
    F: SerializableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let fst = F::read(&path)?;
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// Writes the FST to a file in binary format.
pub fn fst_write_file<F>(fst: *const CFst<F>, path: *const libc::c_char) -> RUSTFST_FFI_RESULT
where
    F: SerializableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        fst.write(&path)?;
        Ok(())
    })
}

/// Serializes the FST as a DOT file compatible with GraphViz binaries.
pub fn fst_draw<F>(
    fst_ptr: *mut CFst<F>,
    isyms: *const CSymbolTable,
    osyms: *const CSymbolTable,
    fname: *const libc::c_char,
    title: *const libc::c_char,
    acceptor: libc::size_t,
    width: libc::c_float,
    height: libc::c_float,
    portrait: libc::size_t,
    vertical: libc::size_t,
    ranksep: libc::c_float,
    nodesep: libc::c_float,
    fontsize: libc::size_t,
    show_weight_one: libc::size_t,
    print_weight: libc::size_t,
) -> RUSTFST_FFI_RESULT
where
    F: SerializableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get_mut!(CFst<F>, fst_ptr);
        if !isyms.is_null() {
            let isymt = get!(CSymbolTable, isyms);
            fst.set_input_symbols(isymt.clone());
        }

        if !osyms.is_null() {
            let osymt = get!(CSymbolTable, osyms);
            fst.set_output_symbols(osymt.clone());
        }

        let drawing_config = DrawingConfig {
            vertical: if vertical > 0 { true } else { false },
            size: if width >= 0.0 && height >= 0.0 {
                Some((width, height))
            } else {
                None
            },
            title: unsafe { CStr::from_ptr(title).as_rust()? },
            portrait: if portrait > 0 { true } else { false },
            ranksep: if ranksep >= 0.0 { Some(ranksep) } else { None },
            nodesep: if nodesep >= 0.0 { Some(nodesep) } else { None },
            fontsize: fontsize as u32,
            acceptor: if acceptor > 0 { true } else { false },
            show_weight_one: if show_weight_one > 0 { true } else { false },
            print_weight: if print_weight > 0 { true } else { false },
        };

        fst.draw(unsafe { CStr::from_ptr(fname).as_rust()? }, &drawing_config)?;

        Ok(())
    })
}

// Missing methods for MutableFst trait:
//- fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst<W>) -> Result<Self>;
//- fn from_text_string(fst_string: &str) -> Result<Self>;
//- fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Result<Self>;
//- fn write_text<P: AsRef<Path>>(&self, path_output: P) -> Result<()>;
//- fn text(&self) -> Result<String>;

/// ExpandedFst methods
/// As described in fst_traits

/// Returns the number of states that contains the FST. They are all counted even if some states
/// are not on a successful path (doesn't perform triming).
pub fn fst_num_states<F>(fst: *const CFst<F>, num_states: *mut libc::size_t) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let res = fst.num_states();
        unsafe { *num_states = res };
        Ok(())
    })
}

pub fn fst_equals<F>(
    fst: *const CFst<F>,
    other_fst: *const CFst<F>,
    is_equal: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT
where
    F: MutableFst<TropicalWeight> + Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        let fst = get!(CFst<F>, fst);
        let other_fst = get!(CFst<F>, other_fst);
        let res = fst.eq(other_fst);
        unsafe { *is_equal = res as usize }
        Ok(())
    })
}

// Missing methods for MutableFst trait:
//- fn states_range(&self) -> Range<StateId>;
//- fn quantize<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2>;

#[no_mangle]
pub extern "C" fn fst_weight_one(weight_one: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::one();
        unsafe { *weight_one = *weight.value() };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_weight_zero(weight_zero: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::zero();
        unsafe { *weight_zero = *weight.value() };
        Ok(())
    })
}

/// drop impl
pub fn fst_destroy<F>(fst_ptr: *mut CFst<F>) -> RUSTFST_FFI_RESULT
where
    F: Fst<TropicalWeight> + 'static,
{
    wrap(|| {
        if fst_ptr.is_null() {
            return Ok(());
        }

        unsafe {
            Box::from_raw(fst_ptr);
        }
        Ok(())
    })
}

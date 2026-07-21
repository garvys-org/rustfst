use proc_macro::TokenStream;
use quote::quote;

pub fn impl_rawpointerconverter_macro(input: &syn::DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

    quote!(
        impl RawPointerConverter<# struct_name> for # struct_name {
            fn into_raw_pointer(self) -> *const # struct_name {
                ffi_convert::convert_into_raw_pointer(self)
            }

            fn into_raw_pointer_mut(self) -> *mut # struct_name {
                ffi_convert::convert_into_raw_pointer_mut(self)
            }

            unsafe fn from_raw_pointer_mut(input: *mut # struct_name) -> Result<# struct_name, ffi_convert::UnexpectedNullPointerError> {
                ffi_convert::take_back_from_raw_pointer_mut(input)
            }

            unsafe fn from_raw_pointer(input: *const # struct_name) -> Result<# struct_name, ffi_convert::UnexpectedNullPointerError> {
                ffi_convert::take_back_from_raw_pointer(input)
            }

        }
    ).into()
}

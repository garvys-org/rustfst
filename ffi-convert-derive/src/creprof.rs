use proc_macro::TokenStream;

use quote::quote;

use crate::utils::{
    parse_enum_variants, parse_struct_fields, parse_target_type, Field, TypeArrayOrTypePath,
};

pub fn impl_creprof_macro(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let target_type = parse_target_type(&input.attrs);

    match &input.data {
        syn::Data::Struct(data_struct) => impl_creprof_struct(name, &target_type, data_struct),
        syn::Data::Enum(data_enum) => impl_creprof_enum(name, &target_type, data_enum),
        _ => panic!("CReprOf can only be derived for structs and unit enums"),
    }
}

fn impl_creprof_struct(
    struct_name: &syn::Ident,
    target_type: &syn::Path,
    data: &syn::DataStruct,
) -> TokenStream {
    let fields = parse_struct_fields(data);
    let c_repr_of_fields = fields
        .iter()
        .map(|field| {
            let Field {
                name: field_name,
                target_name: target_field_name,
                ref field_type,
                ..
            } = field;

            let mut conversion = if field.is_string {
                quote!(std::ffi::CString::c_repr_of(field)?)
            } else {
                match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote!(<#type_array>::c_repr_of(field)?)
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote!(#type_path::c_repr_of(field)?)
                    }
                }
            };

            if field.is_pointer {
                for _ in 0..field.levels_of_indirection {
                    conversion = quote!(#conversion.into_raw_pointer())
                }
            }

            conversion = if field.is_nullable {
                quote!(
                    #field_name: if let Some(field) = input.#target_field_name {
                        #conversion
                    } else {
                        std::ptr::null() as _
                    }
                )
            } else {
                quote!(#field_name: { let field = input.#target_field_name ; #conversion })
            };
            if let Some(convert) = &field.c_repr_of_convert {
                quote!(#field_name: #convert)
            } else {
                conversion
            }
        })
        .collect::<Vec<_>>();

    quote!(
        impl CReprOf<#target_type> for #struct_name {
            fn c_repr_of(input: #target_type) -> Result<Self, ffi_convert::CReprOfError> {
                use ffi_convert::RawPointerConverter;
                Ok(Self {
                    #(#c_repr_of_fields,)*
                })
            }
        }
    )
    .into()
}

fn impl_creprof_enum(
    enum_name: &syn::Ident,
    target_type: &syn::Path,
    data: &syn::DataEnum,
) -> TokenStream {
    let variants = parse_enum_variants(data);

    let match_arms = variants
        .iter()
        .map(|variant| quote!(#target_type::#variant => Ok(#enum_name::#variant)));

    quote!(
        impl CReprOf<#target_type> for #enum_name {
            fn c_repr_of(input: #target_type) -> Result<Self, ffi_convert::CReprOfError> {
                match input {
                    #(#match_arms,)*
                }
            }
        }
    )
    .into()
}

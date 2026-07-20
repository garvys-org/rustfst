use proc_macro::TokenStream;

use quote::quote;
use syn::parse::{Parse, ParseBuffer};

use crate::utils::{
    parse_enum_variants, parse_struct_fields, parse_target_type, Field, TypeArrayOrTypePath,
};

pub fn impl_asrust_macro(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let target_type = parse_target_type(&input.attrs);

    match &input.data {
        syn::Data::Struct(data_struct) => {
            impl_asrust_struct(name, &target_type, data_struct, input)
        }
        syn::Data::Enum(data_enum) => impl_asrust_enum(name, &target_type, data_enum),
        _ => panic!("AsRust can only be derived for structs and unit enums"),
    }
}

fn impl_asrust_struct(
    struct_name: &syn::Ident,
    target_type: &syn::Path,
    data: &syn::DataStruct,
    input: &syn::DeriveInput,
) -> TokenStream {
    let fields = parse_struct_fields(data)
        .iter()
        .filter_map(|field| {
            let Field {
                name: field_name,
                target_name: target_field_name,
                ref field_type,
                ..
            } = field;

            if field.levels_of_indirection > 1 && !field.is_nullable {
                panic!(
                    "The CReprOf, AsRust, and CDrop traits cannot be derived automatically: \
                    The field {} is a pointer field has too many levels of indirection \
                    ({} in this case). Please implements those traits manually.",
                    field_name, field.levels_of_indirection
                )
            }

            let mut conversion = if field.is_string {
                quote!( {
                    use ffi_convert::RawBorrow;
                    unsafe { std::ffi::CStr::raw_borrow(self.#field_name) }?.as_rust()?
                })
            } else if field.is_pointer {
                match field_type {
                    TypeArrayOrTypePath::TypeArray(type_array) => {
                        quote!( {
                        let ref_to_array = unsafe { <#type_array>::raw_borrow(self.#field_name)? };
                        let converted_array = ref_to_struct.as_rust()?;
                        converted_array
                    })
                    }
                    TypeArrayOrTypePath::TypePath(type_path) => {
                        quote!( {
                        let ref_to_struct = unsafe { #type_path::raw_borrow(self.#field_name)? };
                        let converted_struct = ref_to_struct.as_rust()?;
                        converted_struct
                    })
                    }
                }

            } else {
                quote!(self.#field_name.as_rust()?)
            };

            conversion = if field.is_nullable {
                quote!(
                    #target_field_name: if !self.#field_name.is_null() {
                        Some(#conversion)
                    } else {
                        None
                    }
                )
            } else {
                quote!(
                    #target_field_name: #conversion
                )
            };
            if field.c_repr_of_convert.is_some() {
                // ignore field for as_rust if it has a special c_repr_of handling
                None
            } else {
                Some(conversion)
            }
        })
        .collect::<Vec<_>>();

    let extra_fields = &input
        .attrs
        .iter()
        .filter(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string())
                == Some("as_rust_extra_field".into())
        })
        .map(|it| {
            let ExtraFieldsArgs { field_name, init } = it
                .parse_args()
                .expect("Could not parse args for as_rust_extra_field");
            quote! {#field_name: #init}
        })
        .collect::<Vec<_>>();

    quote!(
        impl AsRust<#target_type> for #struct_name {
            fn as_rust(&self) -> Result<#target_type, ffi_convert::AsRustError> {
                Ok(#target_type {
                    #(#fields, )*
                    #(#extra_fields, )*
                })
            }
        }
    )
    .into()
}

fn impl_asrust_enum(
    enum_name: &syn::Ident,
    target_type: &syn::Path,
    data: &syn::DataEnum,
) -> TokenStream {
    let variants = parse_enum_variants(data);

    let match_arms = variants
        .iter()
        .map(|variant| quote!(#enum_name::#variant => Ok(#target_type::#variant)));

    quote!(
        impl AsRust<#target_type> for #enum_name {
            fn as_rust(&self) -> Result<#target_type, ffi_convert::AsRustError> {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    )
    .into()
}

struct ExtraFieldsArgs {
    field_name: syn::Ident,
    init: syn::Expr,
}

impl Parse for ExtraFieldsArgs {
    fn parse(input: &ParseBuffer) -> Result<Self, syn::parse::Error> {
        let field_name = input.parse()?;

        input.parse::<syn::Token![=]>()?;

        let init = input.parse()?;

        Ok(ExtraFieldsArgs { field_name, init })
    }
}

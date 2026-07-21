pub fn parse_target_type(attrs: &[syn::Attribute]) -> syn::Path {
    let target_type_attribute = attrs
        .iter()
        .find(|attribute| {
            attribute.path.get_ident().map(|it| it.to_string()) == Some("target_type".into())
        })
        .expect("Can't derive CReprOf without target_type helper attribute.");

    target_type_attribute.parse_args().unwrap()
}

pub fn parse_no_drop_impl_flag(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        attribute.path.get_ident().map(|it| it.to_string()) == Some("no_drop_impl".to_string())
    })
}

pub fn parse_struct_fields(data: &syn::DataStruct) -> Vec<Field<'_>> {
    data.fields.iter().map(parse_field).collect::<Vec<Field>>()
}

pub fn parse_enum_variants(data: &syn::DataEnum) -> Vec<&syn::Ident> {
    data.variants
        .iter()
        .map(|variant| {
            if !variant.fields.is_empty() {
                panic!(
                    "CReprOf, AsRust, and CDrop derive for enums only supports unit variants \
                     (no fields). Variant `{}` has fields.",
                    variant.ident
                );
            }
            &variant.ident
        })
        .collect()
}

#[derive(PartialEq, Eq, Debug)]
pub enum TypeArrayOrTypePath {
    TypeArray(syn::TypeArray),
    TypePath(syn::TypePath),
}

pub struct Field<'a> {
    pub name: &'a syn::Ident,
    pub target_name: syn::Ident,
    pub field_type: TypeArrayOrTypePath,
    #[allow(unused)]
    pub type_params: Option<syn::AngleBracketedGenericArguments>,
    pub is_nullable: bool,
    pub is_string: bool,
    pub is_pointer: bool,
    pub c_repr_of_convert: Option<syn::Expr>,
    pub levels_of_indirection: u32,
}

pub fn parse_field(field: &syn::Field) -> Field<'_> {
    let name = field.ident.as_ref().expect("Field should have an ident");

    let target_name = field
        .attrs
        .iter()
        .find(|attr| attr.path.get_ident().map(|it| it.to_string()) == Some("target_name".into()))
        .map(|attr| {
            attr.parse_args()
                .expect("Could not parse attributes of c_repr_of_convert")
        })
        .unwrap_or_else(|| name.clone());

    let mut inner_field_type: syn::Type = field.ty.clone();
    let mut levels_of_indirection: u32 = 0;

    while let syn::Type::Ptr(ptr_t) = inner_field_type {
        inner_field_type = *ptr_t.elem;
        levels_of_indirection += 1;
    }

    let (field_type, type_params) = match inner_field_type {
        syn::Type::Path(type_path) => generic_path_to_concrete_type_path(type_path),
        syn::Type::Array(type_array) => (TypeArrayOrTypePath::TypeArray(type_array), None),
        _ => panic!("Field type used in this struct is not supported by the proc macro"),
    };

    let is_nullable = field
        .attrs
        .iter()
        .any(|attr| attr.path.get_ident().map(|it| it.to_string()) == Some("nullable".into()));

    let c_repr_of_convert = field
        .attrs
        .iter()
        .find(|attr| {
            attr.path.get_ident().map(|it| it.to_string()) == Some("c_repr_of_convert".into())
        })
        .map(|attr| {
            attr.parse_args()
                .expect("Could not parse attributes of c_repr_of_convert")
        });

    let is_string = match &field.ty {
        syn::Type::Ptr(ptr_t) => {
            match &*ptr_t.elem {
                syn::Type::Path(path_t) => {
                    // We are trying to detect the c_char identifier in the last segment
                    if let Some(segment) = path_t.path.segments.last() {
                        &segment.ident.to_string() == "c_char"
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        _ => false,
    };

    let is_pointer = matches!(&field.ty, syn::Type::Ptr(_));

    Field {
        name,
        target_name,
        field_type,
        is_nullable,
        is_string,
        is_pointer,
        c_repr_of_convert,
        levels_of_indirection,
        type_params,
    }
}

/// A helper function that extracts type parameters from type definitions of fields.  
///
/// Some procedural macros need to extract type parameters from the definitions of a struct's fields.
/// For instance, if a struct has a field, with the following type :
///  `std::module1::module2::Vec<Hello>`, the goal of this function is to transform this in :
/// `(std::module1::module2::Vec`, `Hello`)`
///
pub fn generic_path_to_concrete_type_path(
    mut path: syn::TypePath,
) -> (
    TypeArrayOrTypePath,
    Option<syn::AngleBracketedGenericArguments>,
) {
    let last_seg: Option<&mut syn::PathSegment> = path.path.segments.last_mut();

    if let Some(last_segment) = last_seg {
        if let syn::PathArguments::AngleBracketed(ref bracketed_type_params) =
            last_segment.arguments
        {
            let extracted_type_params = (*bracketed_type_params).clone();
            last_segment.arguments = syn::PathArguments::None;
            (
                TypeArrayOrTypePath::TypePath(path),
                Some(extracted_type_params),
            )
        } else {
            (TypeArrayOrTypePath::TypePath(path), None)
        }
    } else {
        panic!("Invalid type path: no segments on the TypePath")
    }
}

#[cfg(test)]
mod tests {
    use syn::TypePath;

    use super::*;

    #[test]
    fn test_type_parameter_extraction() {
        let type_path = syn::parse_str::<TypePath>("std::mod1::mod2::Foo<Bar>").unwrap();

        let (transformed_type_path, extracted_type_param) =
            generic_path_to_concrete_type_path(type_path);

        assert_eq!(extracted_type_param.unwrap().args.len(), 1);
        assert_eq!(
            transformed_type_path,
            TypeArrayOrTypePath::TypePath(
                syn::parse_str::<TypePath>("std::mod1::mod2::Foo").unwrap()
            )
        );
    }

    #[test]
    fn test_type_parameters_extraction() {
        let type_path = syn::parse_str::<TypePath>("std::mod1::mod2::Foo<Bar, Baz>").unwrap();

        let (transformed_type_path, extracted_type_param) =
            generic_path_to_concrete_type_path(type_path);

        assert_eq!(
            transformed_type_path,
            TypeArrayOrTypePath::TypePath(
                syn::parse_str::<TypePath>("std::mod1::mod2::Foo").unwrap()
            )
        );
        assert_eq!(extracted_type_param.unwrap().args.len(), 2)
    }

    #[test]
    fn test_type_parameter_extraction_works_without_params() {
        let original_path = syn::parse_str::<TypePath>("std::module1::module2::Hello")
            .expect("Could not parse str into syn::Path");
        let (transformed_path, extracted_type_params) =
            generic_path_to_concrete_type_path(original_path);

        assert!(extracted_type_params.is_none());
        assert_eq!(
            transformed_path,
            TypeArrayOrTypePath::TypePath(
                syn::parse_str::<TypePath>("std::module1::module2::Hello").unwrap()
            )
        )
    }

    #[test]
    fn test_field_parsing_1() {
        let fields = syn::parse_str::<syn::FieldsNamed>("{ field : *const mod1::CDummy }").unwrap();

        let parsed_fields = fields.named.iter().map(parse_field).collect::<Vec<Field>>();

        assert_eq!(parsed_fields[0].is_string, false);
        assert_eq!(parsed_fields[0].is_pointer, true);
        assert_eq!(parsed_fields[0].is_nullable, false);

        if let TypeArrayOrTypePath::TypePath(type_path) = &parsed_fields[0].field_type {
            assert_eq!(type_path.path.segments.len(), 2);
        } else {
            panic!("Unexpected type")
        }
    }

    #[test]
    fn test_field_parsing_2() {
        let fields = syn::parse_str::<syn::FieldsNamed>(
            "{\
                field1: *const mod1::CDummy, \
                field2: *const CDummy\
            }",
        )
        .unwrap();

        let parsed_fields = fields
            .named
            .iter()
            .map(|f| {
                println!("f : {:?}", f);
                f
            })
            .map(parse_field)
            .collect::<Vec<Field>>();

        assert_eq!(parsed_fields[0].is_pointer, true);
        assert_eq!(parsed_fields[1].is_pointer, true);
        assert_eq!(parsed_fields[0].is_string, false);
        assert_eq!(parsed_fields[1].is_string, false);

        let field_type0 =
            if let TypeArrayOrTypePath::TypePath(type_path) = &parsed_fields[0].field_type {
                type_path
            } else {
                panic!("unexpected type")
            };
        let field_type1 =
            if let TypeArrayOrTypePath::TypePath(type_path) = &parsed_fields[1].field_type {
                type_path
            } else {
                panic!("unexpected type")
            };

        let parsed_path_0 = field_type0.path.clone();
        let parsed_path_1 = field_type1.path.clone();

        assert_eq!(parsed_path_0.segments.len(), 2);
        assert_eq!(parsed_path_1.segments.len(), 1);
    }

    #[test]
    fn test_field_parsing_3() {
        let fields = syn::parse_str::<syn::FieldsNamed>(
            "{\
                field1: *const mod1::CFoo<CBar>, \
                field2: *const CFoo<CBar>\
            }",
        )
        .unwrap();

        let parsed_fields = fields
            .named
            .iter()
            .map(|f| {
                println!("f : {:?}", f);
                f
            })
            .map(parse_field)
            .collect::<Vec<Field>>();

        assert_eq!(parsed_fields[0].is_pointer, true);
        assert_eq!(parsed_fields[1].is_pointer, true);
        assert_eq!(parsed_fields[0].is_string, false);
        assert_eq!(parsed_fields[1].is_string, false);

        let field_type0 =
            if let TypeArrayOrTypePath::TypePath(type_path) = &parsed_fields[0].field_type {
                type_path
            } else {
                panic!("unexpected type")
            };
        let field_type1 =
            if let TypeArrayOrTypePath::TypePath(type_path) = &parsed_fields[1].field_type {
                type_path
            } else {
                panic!("unexpected type")
            };

        let parsed_path_0 = field_type0.path.clone();
        let parsed_path_1 = field_type1.path.clone();

        assert_eq!(parsed_path_0.segments.len(), 2);
        assert_eq!(parsed_path_1.segments.len(), 1);
    }
}

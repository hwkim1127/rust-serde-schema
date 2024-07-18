extern crate proc_macro2;

use quote;
use serde_derive_internals::{ast, attr};
use proc_macro2::TokenStream;

use super::{derive_element, derive_field, derive_register_field_types, variant_field_type_variable};

pub fn derive_enum<'a>(
    variants: &'a [ast::Variant<'a>],
    attr_container: &'a attr::Container,
) -> TokenStream {
    let name = attr_container.name().serialize_name();
    let len = variants.len();

    let mut expanded_type_ids = TokenStream::new();
    for (variant_idx, variant) in variants.iter().enumerate() {
        expanded_type_ids.extend(derive_register_field_types(
            variant_idx,
            &variant.fields,
        ));
    }

    let mut expanded_build_type = quote! {
        ::serde_schema::types::Type::build()
            .enum_type(#name, #len)
    };

    for (variant_idx, variant) in variants.iter().enumerate() {
        let variant_name = variant.attrs.name().serialize_name();
        let expanded_build_variant = match variant.style {
            ast::Style::Struct => {
                derive_struct_variant(&variant_name, variant_idx, &variant.fields)
            }
            ast::Style::Newtype => derive_newtype_variant(&variant_name, variant_idx),
            ast::Style::Tuple => derive_tuple_variant(&variant_name, variant_idx, &variant.fields),
            ast::Style::Unit => derive_unit_variant(&variant_name),
        };
        expanded_build_type.extend(expanded_build_variant);
    }

    expanded_build_type.extend(quote! {
        .end()
    });

    quote! {
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    }
}

fn derive_unit_variant(variant_name: &str) -> TokenStream {
    quote! {
        .unit_variant(#variant_name)
    }
}

fn derive_newtype_variant(variant_name: &str, variant_idx: usize) -> TokenStream {
    let field_type = variant_field_type_variable(variant_idx, 0);
    quote! {
        .newtype_variant(#variant_name, #field_type)
    }
}

fn derive_struct_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &[ast::Field<'a>],
) -> TokenStream {
    let fields_len = fields.len();
    let mut expanded = quote! {
        .struct_variant(#variant_name, #fields_len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded.extend(derive_field(variant_idx, field_idx, field));
    }
    expanded.extend(quote! {
        .end()
    });
    expanded
}

fn derive_tuple_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &[ast::Field<'a>],
) -> TokenStream {
    let fields_len = fields.len();
    let mut expanded = quote! {
        .tuple_variant(#variant_name, #fields_len)
    };
    for (field_idx, _) in fields.iter().enumerate() {
        expanded.extend(derive_element(variant_idx, field_idx));
    }
    expanded.extend(quote! {
        .end()
    });
    expanded
}
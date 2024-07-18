extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;

use proc_macro2::{Span, TokenStream as Proc2TokenStream};
use quote::quote;
use serde_derive_internals::{ast, Ctxt, Derive};
use syn::{parse_macro_input, DeriveInput};

use proc_macro::TokenStream as ProcTokenStream;

mod derive_enum;
mod derive_struct;


#[proc_macro_derive(SchemaSerialize)]
pub fn derive_schema_serialize(input: ProcTokenStream) -> ProcTokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Create a new context
    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input, Derive::Serialize);

    // Generate the implementation
    let expanded: Proc2TokenStream = if let Some(container) = container {
        let inner_impl = match &container.data {
            ast::Data::Enum(variants) => derive_enum::derive_enum(variants, &container.attrs),
            ast::Data::Struct(style, fields) => {
                derive_struct::derive_struct(style, fields, &container.attrs)
            }
        };

        let ident = &container.ident;
        let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

        quote! {
            impl #impl_generics ::serde_schema::SchemaSerialize for #ident #ty_generics #where_clause {
                fn schema_register<S>(schema: &mut S) -> Result<S::TypeId, S::Error>
                    where S: ::serde_schema::Schema
                {
                    #inner_impl
                }
            }
        }
    } else {
        quote! { compile_error!("Failed to parse derive input") }
    };

    cx.check().unwrap();

    // Convert the proc_macro2::TokenStream to proc_macro::TokenStream
    expanded.into()
}

fn variant_field_type_variable(variant_idx: usize, field_idx: usize) -> syn::Ident {
    syn::Ident::new(&format!("type_id_{}_{}", variant_idx, field_idx), Span::call_site())
}

fn derive_register_field_types<'a>(
    variant_idx: usize,
    fields: &'a [ast::Field<'a>]
) -> Proc2TokenStream {
    let mut expanded = Proc2TokenStream::new();
    for (field_idx, field) in fields.iter().enumerate() {
        let field_type = &field.ty;
        let type_id_ident = variant_field_type_variable(variant_idx, field_idx);
        let tokens = quote! {
            let #type_id_ident =
                <#field_type as ::serde_schema::SchemaSerialize>::schema_register(schema)?;
        };
        expanded.extend(tokens);
    }
    expanded
}

fn derive_field<'a>(variant_idx: usize, field_idx: usize, field: &ast::Field<'a>) -> Proc2TokenStream {
    let type_id_ident = variant_field_type_variable(variant_idx, field_idx);
    let field_name = field.attrs.name().serialize_name();
    quote!{
        .field(#field_name, #type_id_ident)
    }
}

fn derive_element<'a>(variant_idx: usize, element_idx: usize) -> Proc2TokenStream {
    let type_id_ident = variant_field_type_variable(variant_idx, element_idx);
    quote!{
        .element(#type_id_ident)
    }
}
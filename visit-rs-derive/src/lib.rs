use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_quote, Attribute, DataStruct, DeriveInput, Fields, Ident, MetaList, Path, Type,
    WhereClause, WherePredicate,
};

fn make_impl(
    input: &DeriveInput,
    fields: &Fields,
    trait_path_fields: &Path,
    trait_path: &Path,
    named: Option<&Path>,
    sync: bool,
) -> TokenStream {
    let ident = &input.ident;

    let (_, ty_generics, _) = &input.generics.split_for_impl();

    let mut generics = input.generics.clone();

    generics.params.push(syn::parse_quote! { __visit_rs__V });

    let predicates = &mut generics
        .where_clause
        .get_or_insert(WhereClause {
            predicates: Default::default(),
            where_token: Default::default(),
        })
        .predicates;

    predicates.push(syn::parse_quote! { __visit_rs__V: visit_rs::Visitor });
    if sync {
        predicates.extend(fields.iter().map(|f| &f.ty).map(|t| -> WherePredicate {
            parse_quote! { #t: Sync }
        }));
    }

    let mut ty_set = HashSet::new();
    for (_, field) in field_iter(fields) {
        let ty = &field.ty;
        if !ty_set.insert(ty) {
            continue;
        }
        if let Some(named) = named {
            predicates.push(syn::parse_quote! { for<'__visit_rs__named> #named <'__visit_rs__named, #ty>: #trait_path<__visit_rs__V> });
        } else {
            predicates.push(syn::parse_quote! { #ty: #trait_path<__visit_rs__V> });
        }
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #trait_path_fields<__visit_rs__V> for #ident #ty_generics #where_clause
    }
}

fn field_iter(fields: &Fields) -> impl Iterator<Item = (usize, &syn::Field)> {
    fields.iter().enumerate().filter(|(_, field)| {
        !field.attrs.iter().any(|attr| {
            attr.path().is_ident("visit")
                && attr.parse_args::<Ident>().map_or(false, |id| id == "skip")
        })
    })
}

fn field_idx_iter(fields: &Fields) -> impl Iterator<Item = TokenStream> {
    field_iter(fields).map(|(index, field)| {
        let field_name = &field.ident;
        if let Some(name) = field_name {
            quote! { #name }
        } else {
            let index = syn::Index::from(index);
            quote! { #index }
        }
    })
}

fn field_name_idx_iter(fields: &syn::Fields) -> impl Iterator<Item = (TokenStream, TokenStream)> {
    field_iter(fields).map(|(index, field)| {
        let field_name = &field.ident;
        let idx = if let Some(name) = field_name {
            quote! { #name }
        } else {
            let index = syn::Index::from(index);
            quote! { #index }
        };
        let name = if let Some(name) = field_name {
            quote! { Some(stringify!(#name)) }
        } else {
            quote! { None }
        };
        (name, idx)
    })
}

#[proc_macro_derive(VisitFields, attributes(visit))]
pub fn derive_visit_fields_(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let syn::Data::Struct(data) = &ast.data else {
        let span = match &ast.data {
            syn::Data::Enum(data) => data.enum_token.span,
            syn::Data::Union(data) => data.union_token.span,
            _ => Span::call_site(),
        };
        return syn::Error::new(span, "VisitFields can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let all_impls = match (|| {
        Ok::<_, syn::Error>([
            derive_visit_fields(&ast, data)?,
            derive_visit_fields_async(&ast, data)?,
            derive_visit_fields_named(&ast, data)?,
            derive_visit_fields_named_async(&ast, data)?,
        ])
    })() {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    // panic!(
    //     "{}",
    proc_macro::TokenStream::from(quote! {
        #(#all_impls)*
    })
    // )
}

fn derive_visit_fields(ast: &DeriveInput, data: &DataStruct) -> Result<TokenStream, syn::Error> {
    let impl_t = make_impl(
        &ast,
        &data.fields,
        &syn::parse_quote! { visit_rs::VisitFields },
        &syn::parse_quote! { visit_rs::Visit },
        None,
        false,
    );

    let visit_fields_impl = field_idx_iter(&data.fields).enumerate().map(|(num, idx)| {
        quote! {
            #num => {
                pos += 1;
                Some(visit_rs::Visit::visit(&self.#idx, visitor))
            }
        }
    });

    Ok(quote! {
        #impl_t {
            fn visit_fields<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &'__visit_rs__a mut __visit_rs__V
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> {
                std::iter::from_fn({
                    let mut pos = 0;
                    move || match pos {
                        #(#visit_fields_impl)*
                        _ => None,
                    }
                })
            }
        }
    })
}

fn derive_visit_fields_async(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let impl_t = make_impl(
        &ast,
        &data.fields,
        &syn::parse_quote! { visit_rs::VisitFieldsAsync },
        &syn::parse_quote! { visit_rs::VisitAsync },
        None,
        true,
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            yield visit_rs::VisitAsync::visit_async(&self.#idx, visitor).await;
        }
    });

    Ok(quote! {
        #impl_t {
            fn visit_fields_async<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &'__visit_rs__a mut __visit_rs__V,
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + Send + '__visit_rs__a
            where
                __visit_rs__V: Send,
                <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            {
                visit_rs::lib::async_stream::stream! {
                    #(#visit_fields_impl)*
                }
            }
        }
    })
}

fn derive_visit_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let impl_t = make_impl(
        &ast,
        &data.fields,
        &syn::parse_quote! { visit_rs::VisitFieldsNamed },
        &syn::parse_quote! { visit_rs::Visit },
        Some(&syn::parse_quote! { visit_rs::Named }),
        false,
    );

    let visit_fields_named_impl =
        field_name_idx_iter(&data.fields)
            .enumerate()
            .map(|(num, (name, idx))| {
                quote! {
                    #num => {
                        pos += 1;
                        Some(visit_rs::Visit::visit(&visit_rs::Named {
                            name: #name,
                            value: &self.#idx,
                        }, visitor))
                    }
                }
            });

    Ok(quote! {
        #impl_t {
            fn visit_fields_named<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &'__visit_rs__a mut __visit_rs__V
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + '__visit_rs__a {
                std::iter::from_fn({
                    let mut pos = 0;
                    move || match pos {
                        #(#visit_fields_named_impl)*
                        _ => None,
                    }
                })
            }
        }
    })
}

fn derive_visit_fields_named_async(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let impl_t = make_impl(
        &ast,
        &data.fields,
        &syn::parse_quote! { visit_rs::VisitFieldsNamedAsync },
        &syn::parse_quote! { visit_rs::VisitAsync },
        Some(&syn::parse_quote! { visit_rs::Named }),
        true,
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            yield visit_rs::VisitAsync::visit_async(&visit_rs::Named {
                name: #name,
                value: &self.#idx,
            }, visitor).await;
        }
    });

    Ok(quote! {
        #impl_t {
            fn visit_fields_named_async<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &'__visit_rs__a mut __visit_rs__V,
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + Send + '__visit_rs__a
            where
                __visit_rs__V: Send,
                <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            {
                visit_rs::lib::async_stream::stream! {
                    #(#visit_fields_named_impl)*
                }
            }
        }
    })
}

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Lit, Meta, Path, WhereClause, WherePredicate};
use std::collections::HashSet;

pub fn get_rename_attribute(ast: &DeriveInput) -> Option<String> {
    for attr in &ast.attrs {
        // Check for #[visit(rename = "...")]
        if attr.path().is_ident("visit") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                return Some(s.value());
                            }
                        }
                    }
                }
            }
        }
        // Check for #[serde(rename = "...")]
        if attr.path().is_ident("serde") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                return Some(s.value());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn make_impl(
    input: &DeriveInput,
    fields: &Fields,
    trait_path_fields: &Path,
    trait_path: &Path,
    named: Option<&Path>,
    sync: bool,
    is_static: bool,
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
            syn::parse_quote! { #t: Sync }
        }));
    }

    let mut ty_set = HashSet::new();
    for (_, field) in field_iter(fields) {
        let ty = &field.ty;
        if !ty_set.insert(ty) {
            continue;
        }
        if let Some(named) = named {
            if is_static {
                predicates.push(
                    syn::parse_quote! { for<'__visit_rs__named> #named <'__visit_rs__named, visit_rs::Static<#ty>>: #trait_path<__visit_rs__V> },
                );
            } else {
                predicates.push(syn::parse_quote! { for<'__visit_rs__named> #named <'__visit_rs__named, #ty>: #trait_path<__visit_rs__V> });
            }
        } else {
            if is_static {
                predicates
                    .push(syn::parse_quote! { visit_rs::Static<#ty>: #trait_path<__visit_rs__V> });
            } else {
                predicates.push(syn::parse_quote! { #ty: #trait_path<__visit_rs__V> });
            }
        }
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #trait_path_fields<__visit_rs__V> for #ident #ty_generics #where_clause
    }
}

pub fn field_iter(fields: &Fields) -> impl Iterator<Item = (usize, &syn::Field)> {
    fields.iter().enumerate().filter(|(_, field)| {
        !field.attrs.iter().any(|attr| {
            attr.path().is_ident("visit")
                && attr.parse_args::<Ident>().map_or(false, |id| id == "skip")
        })
    })
}

pub fn field_idx_iter(fields: &Fields) -> impl Iterator<Item = TokenStream> {
    field_iter(fields).map(|(idx, field)| {
        if let Some(ident) = &field.ident {
            quote! { #ident }
        } else {
            let idx = syn::Index::from(idx);
            quote! { #idx }
        }
    })
}

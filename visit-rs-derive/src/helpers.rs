use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Lit, Meta, Path, WhereClause, WherePredicate, Variant};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum RenameRule {
    None,
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(RenameRule::LowerCase),
            "UPPERCASE" => Some(RenameRule::UpperCase),
            "PascalCase" => Some(RenameRule::PascalCase),
            "camelCase" => Some(RenameRule::CamelCase),
            "snake_case" => Some(RenameRule::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Some(RenameRule::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameRule::ScreamingKebabCase),
            _ => None,
        }
    }

    pub fn apply(&self, s: &str) -> String {
        match self {
            RenameRule::None => s.to_string(),
            RenameRule::LowerCase => s.to_lowercase(),
            RenameRule::UpperCase => s.to_uppercase(),
            RenameRule::PascalCase => to_pascal_case(s),
            RenameRule::CamelCase => to_camel_case(s),
            RenameRule::SnakeCase => to_snake_case(s),
            RenameRule::ScreamingSnakeCase => to_snake_case(s).to_uppercase(),
            RenameRule::KebabCase => to_kebab_case(s),
            RenameRule::ScreamingKebabCase => to_kebab_case(s).to_uppercase(),
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lowercase = false;
    for (i, ch) in s.chars().enumerate() {
        if ch == '-' {
            result.push('_');
            prev_is_lowercase = false;
        } else if ch.is_uppercase() {
            if i > 0 && prev_is_lowercase {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_lowercase = false;
        } else {
            result.push(ch);
            prev_is_lowercase = ch.is_lowercase();
        }
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

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

pub fn get_rename_all_attribute(ast: &DeriveInput) -> RenameRule {
    for attr in &ast.attrs {
        // Check for #[visit(rename_all = "...")]
        if attr.path().is_ident("visit") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename_all") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                if let Some(rule) = RenameRule::from_str(&s.value()) {
                                    return rule;
                                }
                            }
                        }
                    }
                }
            }
        }
        // Check for #[serde(rename_all = "...")]
        if attr.path().is_ident("serde") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename_all") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                if let Some(rule) = RenameRule::from_str(&s.value()) {
                                    return rule;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    RenameRule::None
}

pub fn get_variant_rename(variant: &Variant, default_rule: RenameRule) -> String {
    // First check for explicit rename attribute
    for attr in &variant.attrs {
        if attr.path().is_ident("visit") || attr.path().is_ident("serde") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                return s.value();
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply rename_all rule
    default_rule.apply(&variant.ident.to_string())
}

pub fn get_field_rename(field: &syn::Field, default_rule: RenameRule) -> Option<String> {
    let field_name = field.ident.as_ref()?.to_string();

    // First check for explicit rename attribute
    for attr in &field.attrs {
        if attr.path().is_ident("visit") || attr.path().is_ident("serde") {
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

    // Apply rename_all rule
    Some(default_rule.apply(&field_name))
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

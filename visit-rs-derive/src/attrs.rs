use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Expr, Lit, Meta};

/// Parse syn::Meta into our AttributeMeta representation
fn parse_meta_to_attribute_meta(meta: &Meta) -> TokenStream {
    match meta {
        Meta::Path(path) => {
            let path_str = path.to_token_stream().to_string();
            quote! {
                visit_rs::metadata::AttributeMeta::Path {
                    path: #path_str,
                }
            }
        }
        Meta::List(list) => {
            let path_str = list.path.to_token_stream().to_string();
            let tokens_str = list.tokens.to_string();

            // Try to parse the list contents
            if let Ok(nested_meta) = syn::parse2::<Meta>(list.tokens.clone()) {
                let nested = parse_meta_to_attribute_meta(&nested_meta);
                quote! {
                    visit_rs::metadata::AttributeMeta::List {
                        path: #path_str,
                        items: &[#nested],
                    }
                }
            } else {
                // Fallback to unparsed
                quote! {
                    visit_rs::metadata::AttributeMeta::Unparsed {
                        path: #path_str,
                        tokens: #tokens_str,
                    }
                }
            }
        }
        Meta::NameValue(nv) => {
            let path_str = nv.path.to_token_stream().to_string();
            let name_str = nv
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            let value = match &nv.value {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Str(s) => {
                        let val = s.value();
                        quote! { visit_rs::metadata::MetaValue::Str(#val) }
                    }
                    Lit::Bool(b) => {
                        let val = b.value;
                        quote! { visit_rs::metadata::MetaValue::Bool(#val) }
                    }
                    Lit::Int(i) => {
                        if let Ok(val) = i.base10_parse::<i64>() {
                            quote! { visit_rs::metadata::MetaValue::Int(#val) }
                        } else {
                            let s = i.to_string();
                            quote! { visit_rs::metadata::MetaValue::Unparsed(#s) }
                        }
                    }
                    Lit::Float(f) => {
                        let s = f.to_string();
                        quote! { visit_rs::metadata::MetaValue::Float(#s) }
                    }
                    _ => {
                        let s = expr_lit.to_token_stream().to_string();
                        quote! { visit_rs::metadata::MetaValue::Unparsed(#s) }
                    }
                },
                Expr::Path(path) => {
                    let s = path.to_token_stream().to_string();
                    quote! { visit_rs::metadata::MetaValue::Path(#s) }
                }
                _ => {
                    let s = nv.value.to_token_stream().to_string();
                    quote! { visit_rs::metadata::MetaValue::Unparsed(#s) }
                }
            };

            quote! {
                visit_rs::metadata::AttributeMeta::NameValue {
                    path: #path_str,
                    name: #name_str,
                    value: #value,
                }
            }
        }
    }
}

/// Extract all visit and serde attributes and convert to AttributeMeta
pub fn extract_all_meta(attrs: &[Attribute]) -> Vec<TokenStream> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("visit") || attr.path().is_ident("serde") {
                Some(parse_meta_to_attribute_meta(&attr.meta))
            } else {
                None
            }
        })
        .collect()
}


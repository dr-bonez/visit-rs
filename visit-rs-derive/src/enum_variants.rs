use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Ident};
use std::collections::HashSet;

use crate::helpers::get_rename_attribute;

pub fn derive_all_variant_traits(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let enum_info = derive_enum_info(ast, data)?;
    let visit_variant = derive_visit_variant(ast, data)?;
    let visit_variants_static = derive_visit_variants_static(ast, data)?;
    let visit_variant_fields = derive_visit_variant_fields(ast, data)?;
    let visit_variant_fields_covered = derive_visit_variant_fields_covered(ast, data)?;
    let visit_variant_fields_static = derive_visit_variant_fields_static(ast, data)?;
    let visit_variant_fields_named = derive_visit_variant_fields_named(ast, data)?;
    let visit_variant_fields_static_named = derive_visit_variant_fields_static_named(ast, data)?;
    let visit_variant_fields_async = derive_visit_variant_fields_async(ast, data)?;
    let visit_variant_fields_covered_async = derive_visit_variant_fields_covered_async(ast, data)?;
    let visit_variant_fields_static_async = derive_visit_variant_fields_static_async(ast, data)?;
    let visit_variant_fields_named_async = derive_visit_variant_fields_named_async(ast, data)?;
    let visit_variant_fields_static_named_async = derive_visit_variant_fields_static_named_async(ast, data)?;

    Ok(quote! {
        #enum_info
        #visit_variant
        #visit_variants_static
        #visit_variant_fields
        #visit_variant_fields_covered
        #visit_variant_fields_static
        #visit_variant_fields_named
        #visit_variant_fields_static_named
        #visit_variant_fields_async
        #visit_variant_fields_covered_async
        #visit_variant_fields_static_async
        #visit_variant_fields_named_async
        #visit_variant_fields_static_named_async
    })
}

fn derive_enum_info(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let name = get_rename_attribute(ast).unwrap_or_else(|| ident.to_string());
    let variant_count = data.variants.len();

    // Generate StructInfoData for each variant
    let variant_infos = data.variants.iter().map(|variant| {
        let variant_name = variant.ident.to_string();
        let named_fields = matches!(variant.fields, Fields::Named(_));
        let field_count = variant.fields.iter().count();

        quote! {
            visit_rs::StructInfoData {
                name: #variant_name,
                named_fields: #named_fields,
                field_count: #field_count,
            }
        }
    });

    // Generate variant_info match arms
    let variant_info_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        let named_fields = matches!(variant.fields, Fields::Named(_));
        let field_count = variant.fields.iter().count();

        let pattern = match &variant.fields {
            Fields::Named(_) => quote! { Self::#variant_name { .. } },
            Fields::Unnamed(_) => {
                let placeholders = (0..field_count).map(|_| quote! { _ });
                quote! { Self::#variant_name(#(#placeholders),*) }
            }
            Fields::Unit => quote! { Self::#variant_name },
        };

        quote! {
            #pattern => visit_rs::StructInfoData {
                name: #variant_name_str,
                named_fields: #named_fields,
                field_count: #field_count,
            }
        }
    });

    // Generate variant_info_by_name match arms
    let variant_by_name_arms = data.variants.iter().map(|variant| {
        let variant_name_str = variant.ident.to_string();
        let named_fields = matches!(variant.fields, Fields::Named(_));
        let field_count = variant.fields.iter().count();

        quote! {
            #variant_name_str => Some(visit_rs::StructInfoData {
                name: #variant_name_str,
                named_fields: #named_fields,
                field_count: #field_count,
            })
        }
    });

    Ok(quote! {
        impl #impl_generics visit_rs::EnumInfo for #ident #ty_generics #where_clause {
            const DATA: visit_rs::EnumInfoData = visit_rs::EnumInfoData {
                name: #name,
                variant_count: #variant_count,
            };

            fn variants() -> impl IntoIterator<Item = visit_rs::StructInfoData> + Send + Sync + 'static {
                [#(#variant_infos),*]
            }

            fn variant_info(&self) -> visit_rs::StructInfoData {
                match self {
                    #(#variant_info_arms),*
                }
            }

            fn variant_info_by_name(name: &str) -> Option<visit_rs::StructInfoData> {
                match name {
                    #(#variant_by_name_arms,)*
                    _ => None
                }
            }
        }
    })
}

fn derive_visit_variant(ast: &DeriveInput, _data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariant<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            for<'a> visit_rs::Variant<'a, Self>: visit_rs::Visit<__visit_rs__V>,
        {
            fn visit_variant(&self, visitor: &mut __visit_rs__V) -> <__visit_rs__V as visit_rs::Visitor>::Result {
                visit_rs::Variant {
                    info: self.variant_info(),
                    value: self,
                }
                .visit(visitor)
            }
        }
    })
}

fn derive_visit_variants_static(ast: &DeriveInput, _data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantsStatic<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            for<'a> visit_rs::Variant<'a, visit_rs::Static<Self>>: visit_rs::Visit<__visit_rs__V>,
        {
            fn visit_variants_static<'a>(visitor: &'a mut __visit_rs__V) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                Self::variants().into_iter().map(|info| {
                    visit_rs::Variant {
                        info,
                        value: visit_rs::Static::new_ref(),
                    }
                    .visit(visitor)
                })
            }
        }
    })
}

fn derive_visit_variant_fields(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Collect all unique field types for trait bounds
    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { #ty: visit_rs::Visit<__visit_rs__V> });
            }
        }
    }

    // Generate match arms for each variant
    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_matches = (0..field_names.len()).map(|idx| {
                    let field_name = &field_names[idx];
                    quote! {
                        #idx => #field_name.visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_matches = (0..field_idents.len()).map(|idx| {
                    let field_ident = &field_idents[idx];
                    quote! {
                        #idx => #field_ident.visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => match i {
                        _ => return None,
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFields<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            #(#field_predicates),*
        {
            fn visit_variant_fields<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V,
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                let mut i = 0;
                std::iter::from_fn(move || {
                    let res = match self {
                        #(#variant_arms),*
                    };
                    i += 1;
                    Some(res)
                })
            }
        }
    })
}

fn derive_visit_variant_fields_covered(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Collect all unique field types for trait bounds with Covered wrapper
    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__covered> visit_rs::Covered<'__visit_rs__covered, #ty>: visit_rs::Visit<__visit_rs__V> });
            }
        }
    }

    // Generate match arms for each variant
    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_matches = (0..field_names.len()).map(|idx| {
                    let field_name = &field_names[idx];
                    quote! {
                        #idx => visit_rs::Covered(#field_name).visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_matches = (0..field_idents.len()).map(|idx| {
                    let field_ident = &field_idents[idx];
                    quote! {
                        #idx => visit_rs::Covered(#field_ident).visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => match i {
                        _ => return None,
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsCovered<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            #(#field_predicates),*
        {
            fn visit_variant_fields_covered<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                let mut i = 0;
                std::iter::from_fn(move || {
                    let res = match self {
                        #(#variant_arms),*
                    };
                    i += 1;
                    Some(res)
                })
            }
        }
    })
}

fn derive_visit_variant_fields_static(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Collect all unique field types for trait bounds
    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { visit_rs::Static<#ty>: visit_rs::Visit<__visit_rs__V> });
            }
        }
    }

    // Build a list of all field types per variant for static iteration
    let variant_field_types: Vec<_> = data.variants.iter().map(|variant| {
        let variant_name_str = variant.ident.to_string();
        let field_types: Vec<_> = variant.fields.iter().map(|f| &f.ty).collect();
        (variant_name_str, field_types)
    }).collect();

    // Generate match arms for each variant name
    let variant_arms = variant_field_types.iter().map(|(variant_name, field_types)| {
        let field_matches = (0..field_types.len()).map(|idx| {
            let ty = &field_types[idx];
            quote! {
                #idx => visit_rs::Static::<#ty>::new().visit(visitor)
            }
        });

        quote! {
            #variant_name => match i {
                #(#field_matches,)*
                _ => return None,
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsStatic<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            #(#field_predicates),*
        {
            fn visit_variant_fields_static<'a>(
                info: &'a visit_rs::StructInfoData,
                visitor: &'a mut __visit_rs__V,
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                let mut i = 0;
                std::iter::from_fn(move || {
                    let res = match info.name {
                        #(#variant_arms,)*
                        x => {
                            debug_assert!(false, "UNREACHABLE: unknown variant {}", x);
                            return None;
                        }
                    };
                    i += 1;
                    Some(res)
                })
            }
        }
    })
}

fn derive_visit_variant_fields_named(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, #ty>: visit_rs::Visit<__visit_rs__V> });
            }
        }
    }

    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_matches = (0..field_names.len()).map(|idx| {
                    let field_name = &field_names[idx];
                    quote! {
                        #idx => visit_rs::Named {
                            name: Some(stringify!(#field_name)),
                            value: #field_name,
                        }.visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_matches = (0..field_idents.len()).map(|idx| {
                    let field_ident = &field_idents[idx];
                    quote! {
                        #idx => visit_rs::Named {
                            name: None,
                            value: #field_ident,
                        }.visit(visitor)
                    }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => match i {
                        #(#field_matches,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => match i {
                        _ => return None,
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsNamed<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            #(#field_predicates),*
        {
            fn visit_variant_fields_named<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                let mut i = 0;
                std::iter::from_fn(move || {
                    let res = match self {
                        #(#variant_arms),*
                    };
                    i += 1;
                    Some(res)
                })
            }
        }
    })
}

fn derive_visit_variant_fields_static_named(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, visit_rs::Static<#ty>>: visit_rs::Visit<__visit_rs__V> });
            }
        }
    }

    let variant_match_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        match &variant.fields {
            Fields::Named(fields) => {
                let field_visits = fields.named.iter().enumerate().map(|(idx, field)| {
                    let ty = &field.ty;
                    let field_name = field.ident.as_ref().unwrap();
                    quote! {
                        #idx => visit_rs::Named {
                            name: Some(stringify!(#field_name)),
                            value: &visit_rs::Static::<#ty>::new(),
                        }.visit(visitor)
                    }
                });

                quote! {
                    #variant_name_str => match i {
                        #(#field_visits,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_visits = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                    let ty = &field.ty;
                    quote! {
                        #idx => visit_rs::Named {
                            name: None,
                            value: &visit_rs::Static::<#ty>::new(),
                        }.visit(visitor)
                    }
                });

                quote! {
                    #variant_name_str => match i {
                        #(#field_visits,)*
                        _ => return None,
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #variant_name_str => match i {
                        _ => return None,
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsStaticNamed<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor,
            #(#field_predicates),*
        {
            fn visit_variant_fields_static_named<'a>(
                info: &'a visit_rs::StructInfoData,
                visitor: &'a mut __visit_rs__V
            ) -> impl Iterator<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                let mut i = 0;
                std::iter::from_fn(move || {
                    let res = match info.name {
                        #(#variant_match_arms,)*
                        x => {
                            debug_assert!(false, "UNREACHABLE: unknown variant {}", x);
                            return None;
                        }
                    };
                    i += 1;
                    Some(res)
                })
            }
        }
    })
}

fn derive_visit_variant_fields_async(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { #ty: visit_rs::VisitAsync<__visit_rs__V> });
            }
        }
    }

    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_visits = field_names.iter().map(|field_name| {
                    quote! { yield visit_rs::VisitAsync::visit_async(#field_name, visitor).await; }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_visits = field_idents.iter().map(|field_ident| {
                    quote! { yield visit_rs::VisitAsync::visit_async(#field_ident, visitor).await; }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => {}
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsAsync<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor + Send,
            <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            #(#field_predicates),*
        {
            fn visit_variant_fields_async<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + Send + 'a {
                visit_rs::lib::async_stream::stream! {
                    match self {
                        #(#variant_arms)*
                    }
                    #[allow(unreachable_code)]
                    if false {
                        yield unreachable!() as <__visit_rs__V as visit_rs::Visitor>::Result
                    }
                }
            }
        }
    })
}

fn derive_visit_variant_fields_covered_async(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__covered> visit_rs::Covered<'__visit_rs__covered, #ty>: visit_rs::VisitAsync<__visit_rs__V> });
            }
        }
    }

    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_visits = field_names.iter().map(|field_name| {
                    quote! { yield visit_rs::VisitAsync::visit_async(&visit_rs::Covered(#field_name), visitor).await; }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_visits = field_idents.iter().map(|field_ident| {
                    quote! { yield visit_rs::VisitAsync::visit_async(&visit_rs::Covered(#field_ident), visitor).await; }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => {}
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsCoveredAsync<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor + Send,
            <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            #(#field_predicates),*
        {
            fn visit_variant_fields_covered_async<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + Send + 'a {
                visit_rs::lib::async_stream::stream! {
                    match self {
                        #(#variant_arms)*
                    }
                    #[allow(unreachable_code)]
                    if false {
                        yield unreachable!() as <__visit_rs__V as visit_rs::Visitor>::Result
                    }
                }
            }
        }
    })
}

fn derive_visit_variant_fields_static_async(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { visit_rs::Static<#ty>: visit_rs::VisitAsync<__visit_rs__V> });
            }
        }
    }

    let variant_match_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        match &variant.fields {
            Fields::Named(fields) => {
                let field_visits = fields.named.iter().map(|field| {
                    let ty = &field.ty;
                    quote! { yield visit_rs::VisitAsync::visit_async(&visit_rs::Static::<#ty>::new(), visitor).await; }
                });

                quote! {
                    #variant_name_str => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_visits = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! { yield visit_rs::VisitAsync::visit_async(&visit_rs::Static::<#ty>::new(), visitor).await; }
                });

                quote! {
                    #variant_name_str => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #variant_name_str => {}
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsStaticAsync<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor + Send,
            <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            #(#field_predicates),*
        {
            fn visit_variant_fields_static_async<'a>(
                info: &'a visit_rs::StructInfoData,
                visitor: &'a mut __visit_rs__V
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                visit_rs::lib::async_stream::stream! {
                    match info.name {
                        #(#variant_match_arms,)*
                        x => {
                            debug_assert!(false, "UNREACHABLE: unknown variant {}", x);
                        }
                    }
                    #[allow(unreachable_code)]
                    if false {
                        yield unreachable!() as <__visit_rs__V as visit_rs::Visitor>::Result
                    }
                }
            }
        }
    })
}

fn derive_visit_variant_fields_named_async(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, #ty>: visit_rs::VisitAsync<__visit_rs__V> });
            }
        }
    }

    let variant_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_visits = field_names.iter().map(|field_name| {
                    quote! {
                        yield visit_rs::VisitAsync::visit_async(&visit_rs::Named {
                            name: Some(stringify!(#field_name)),
                            value: #field_name,
                        }, visitor).await;
                    }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| Ident::new(&format!("tup_{}", i), Span::call_site()))
                    .collect();
                let field_visits = field_idents.iter().map(|field_ident| {
                    quote! {
                        yield visit_rs::VisitAsync::visit_async(&visit_rs::Named {
                            name: None,
                            value: #field_ident,
                        }, visitor).await;
                    }
                });

                quote! {
                    Self::#variant_name(#(#field_idents),*) => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => {}
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsNamedAsync<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor + Send,
            <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            #(#field_predicates),*
        {
            fn visit_variant_fields_named_async<'a>(
                &'a self,
                visitor: &'a mut __visit_rs__V
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + Send + 'a {
                visit_rs::lib::async_stream::stream! {
                    match self {
                        #(#variant_arms)*
                    }
                    #[allow(unreachable_code)]
                    if false {
                        yield unreachable!() as <__visit_rs__V as visit_rs::Visitor>::Result
                    }
                }
            }
        }
    })
}

fn derive_visit_variant_fields_static_named_async(ast: &DeriveInput, data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut ty_set = HashSet::new();
    let mut field_predicates = Vec::new();
    for variant in &data.variants {
        for field in &variant.fields {
            let ty = &field.ty;
            if ty_set.insert(ty) {
                field_predicates.push(quote! { for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, visit_rs::Static<#ty>>: visit_rs::VisitAsync<__visit_rs__V> });
            }
        }
    }

    let variant_match_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        match &variant.fields {
            Fields::Named(fields) => {
                let field_visits = fields.named.iter().map(|field| {
                    let ty = &field.ty;
                    let field_name = field.ident.as_ref().unwrap();
                    quote! {
                        yield visit_rs::VisitAsync::visit_async(&visit_rs::Named {
                            name: Some(stringify!(#field_name)),
                            value: &visit_rs::Static::<#ty>::new(),
                        }, visitor).await;
                    }
                });

                quote! {
                    #variant_name_str => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_visits = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        yield visit_rs::VisitAsync::visit_async(&visit_rs::Named {
                            name: None,
                            value: &visit_rs::Static::<#ty>::new(),
                        }, visitor).await;
                    }
                });

                quote! {
                    #variant_name_str => {
                        #(#field_visits)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #variant_name_str => {}
                }
            }
        }
    });

    Ok(quote! {
        impl<__visit_rs__V, #impl_generics> visit_rs::VisitVariantFieldsStaticNamedAsync<__visit_rs__V> for #ident #ty_generics
        #where_clause
        where
            __visit_rs__V: visit_rs::Visitor + Send,
            <__visit_rs__V as visit_rs::Visitor>::Result: Send,
            #(#field_predicates),*
        {
            fn visit_variant_fields_static_named_async<'a>(
                info: &'a visit_rs::StructInfoData,
                visitor: &'a mut __visit_rs__V
            ) -> impl visit_rs::lib::futures::Stream<Item = <__visit_rs__V as visit_rs::Visitor>::Result> + 'a {
                visit_rs::lib::async_stream::stream! {
                    match info.name {
                        #(#variant_match_arms,)*
                        x => {
                            debug_assert!(false, "UNREACHABLE: unknown variant {}", x);
                        }
                    }
                    #[allow(unreachable_code)]
                    if false {
                        yield unreachable!() as <__visit_rs__V as visit_rs::Visitor>::Result
                    }
                }
            }
        }
    })
}

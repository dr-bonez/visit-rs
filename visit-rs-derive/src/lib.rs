use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Attribute, DataStruct, DeriveInput, Fields, Ident, MetaList, Path, Type, WhereClause,
    WherePredicate,
};

fn make_impl(
    input: &DeriveInput,
    fields: &Fields,
    res_ty: &Type,
    trait_path_fields: &Path,
    trait_path: &Path,
    named: Option<&Path>,
    extra_predicates: impl IntoIterator<Item = WherePredicate>,
) -> TokenStream {
    let ident = &input.ident;

    let (_, ty_generics, _) = &input.generics.split_for_impl();

    let mut generics = input.generics.clone();

    generics.params.push(syn::parse_quote! { Visitor });

    let predicates = &mut generics
        .where_clause
        .get_or_insert(WhereClause {
            predicates: Default::default(),
            where_token: Default::default(),
        })
        .predicates;

    predicates.extend(extra_predicates.into_iter());

    let mut ty_set = HashSet::new();
    for (_, field) in field_iter(fields) {
        let ty = &field.ty;
        if !ty_set.insert(ty) {
            continue;
        }
        if let Some(named) = named {
            predicates.push(syn::parse_quote! { for<'__visit_rs__named> #named <'__visit_rs__named, #ty>: #trait_path<Visitor, Result = #res_ty> });
        } else {
            predicates.push(syn::parse_quote! { #ty: #trait_path<Visitor, Result = #res_ty> });
        }
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #trait_path_fields<Visitor> for #ident #ty_generics #where_clause
    }
}

struct StructAttrs {
    res_ty: Type,
    res: TokenStream,
    try_token: TokenStream,
}

fn parse_attrs(attrs: &[Attribute]) -> Result<StructAttrs, syn::Error> {
    if let Some(ty) = attrs.iter().find_map(|attr| {
        attr.path()
            .is_ident("visit")
            .then(|| {
                attr.parse_args::<MetaList>().ok().and_then(|meta| {
                    meta.path
                        .is_ident("try")
                        .then(|| meta.parse_args::<Type>().ok())
                        .flatten()
                })
            })
            .flatten()
    }) {
        // (
        //     syn::parse_quote_spanned! { ty.span() => Result<(), #ty> },
        //     quote_spanned! { ty.span() => Ok(()) },
        //     quote_spanned! { ty.span() => ? },
        // )
        todo!()
    } else {
        // (syn::parse_quote! { () }, quote! { () }, quote! {})
        todo!()
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
            derive_visit_fields_named(&ast, data)?,
            derive_visit_mut_fields(&ast, data)?,
            derive_visit_mut_fields_named(&ast, data)?,
            derive_async_visit_fields(&ast, data)?,
            derive_async_visit_fields_named(&ast, data)?,
            derive_parallel_async_visit_fields(&ast, data)?,
            derive_parallel_async_visit_fields_named(&ast, data)?,
            derive_async_visit_mut_fields(&ast, data)?,
            derive_async_visit_mut_fields_named(&ast, data)?,
        ])
    })() {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    panic!(
        "{}",
        proc_macro::TokenStream::from(quote! {
            #(#all_impls)*
        })
    )
}

fn derive_visit_fields(ast: &DeriveInput, data: &DataStruct) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::VisitFields },
        &syn::parse_quote! { visit_rs::Visit },
        None,
        [],
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            visit_rs::Visit::<Visitor>::visit(&self.#idx, visitor) #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;
            fn visit_fields(&self, visitor: &mut Visitor) -> Self::Result {
                #(#visit_fields_impl)*

                #res
            }
        }
    })
}

fn derive_visit_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::VisitFieldsNamed },
        &syn::parse_quote! { visit_rs::Visit },
        Some(&syn::parse_quote! { visit_rs::Named }),
        [],
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            visit_rs::Visit::<Visitor>::visit(&visit_rs::Named {
                name: #name,
                value: &self.#idx,
            }, visitor) #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;
            fn visit_fields_named(&self, visitor: &mut Visitor) -> Self::Result {
                #(#visit_fields_named_impl)*

                #res
            }
        }
    })
}

fn derive_visit_mut_fields(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::VisitMutFields },
        &syn::parse_quote! { visit_rs::VisitMut },
        None,
        [],
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            visit_rs::VisitMut::<Visitor>::visit_mut(&mut self.#idx, visitor) #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;
            fn visit_mut_fields(&mut self, visitor: &mut Visitor) -> Self::Result {
                #(#visit_fields_impl)*

                #res
            }
        }
    })
}

fn derive_visit_mut_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::VisitMutFieldsNamed },
        &syn::parse_quote! { visit_rs::VisitMut },
        Some(&syn::parse_quote! { visit_rs::NamedMut }),
        [],
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            visit_rs::VisitMut::<Visitor>::visit_mut(&mut visit_rs::NamedMut {
                name: #name,
                value: &mut self.#idx,
            }, visitor) #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;
            fn visit_mut_fields_named(&mut self, visitor: &mut Visitor) -> Self::Result {
                #(#visit_fields_named_impl)*

                #res
            }
        }
    })
}

fn derive_async_visit_fields(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::AsyncVisitFields },
        &syn::parse_quote! { visit_rs::AsyncVisit },
        None,
        [syn::parse_quote! { Visitor: Send }],
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            visit_rs::AsyncVisit::visit_async(&self.#idx, visitor).await #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            async fn visit_fields_async(
                &self,
                visitor: &mut Visitor,
            ) -> Self::Result {
                #(#visit_fields_impl)*

                #res
            }
        }
    })
}

fn derive_async_visit_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::AsyncVisitFieldsNamed },
        &syn::parse_quote! { visit_rs::AsyncVisit },
        Some(&syn::parse_quote! { visit_rs::Named }),
        [syn::parse_quote! { Visitor: Send }],
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            visit_rs::AsyncVisit::visit_async(&visit_rs::Named {
                name: #name,
                value: &self.#idx,
            }, visitor).await #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            async fn visit_fields_named_async(
                &self,
                visitor: &mut Visitor,
            ) -> Self::Result {
                #(#visit_fields_named_impl)*

                #res
            }
        }
    })
}

fn derive_parallel_async_visit_fields(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::ParallelAsyncVisitFields },
        &syn::parse_quote! { visit_rs::AsyncVisit },
        None,
        [syn::parse_quote! { Visitor: Send + Clone }],
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            futures.push_back(
                visit_rs::lib::futures::FutureExt::boxed({
                    let mut visitor = visitor.clone();
                    async move { visit_rs::AsyncVisit::visit_async(&self.#idx, &mut visitor).await }
                })
            );
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            fn parallel_visit_fields_async<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &Visitor,
            ) -> impl std::future::Future<Output = Self::Result> + Send + '__visit_rs__a
            where
                Visitor: '__visit_rs__a,
            {
                let mut futures = visit_rs::lib::futures::stream::FuturesOrdered::new();

                #(#visit_fields_impl)*

                async move {
                    while let Some(res) = visit_rs::lib::futures::stream::StreamExt::next(&mut futures).await {
                        res #try_token;
                    }
                    #res
                }
            }
        }
    })
}

fn derive_parallel_async_visit_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::ParallelAsyncVisitFieldsNamed },
        &syn::parse_quote! { visit_rs::AsyncVisit },
        Some(&syn::parse_quote! { visit_rs::Named }),
        [syn::parse_quote! { Visitor: Send + Clone }],
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            futures.push_back(
                visit_rs::lib::futures::FutureExt::boxed({
                    let mut visitor = visitor.clone();
                    async move {
                        visit_rs::AsyncVisit::visit_async(
                            &visit_rs::Named {
                                name: #name,
                                value: &self.#idx,
                            },
                            &mut visitor,
                        )
                        .await
                    }
                })
            );
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            fn parallel_visit_fields_named_async<'__visit_rs__a>(
                &'__visit_rs__a self,
                visitor: &Visitor,
            ) -> impl std::future::Future<Output = Self::Result> + Send + '__visit_rs__a
            where
                Visitor: '__visit_rs__a,
            {
                let mut futures = visit_rs::lib::futures::stream::FuturesOrdered::new();

                #(#visit_fields_named_impl)*

                async move {
                    while let Some(res) = visit_rs::lib::futures::stream::StreamExt::next(&mut futures).await {
                        res #try_token;
                    }
                    #res
                }
            }
        }
    })
}

fn derive_async_visit_mut_fields(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::AsyncVisitMutFields },
        &syn::parse_quote! { visit_rs::AsyncVisitMut },
        None,
        [syn::parse_quote! { Visitor: Send }],
    );

    let visit_fields_impl = field_idx_iter(&data.fields).map(|idx| {
        quote! {
            visit_rs::AsyncVisitMut::visit_mut_async(&mut self.#idx, visitor).await #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            async fn visit_mut_fields_async(
                &mut self,
                visitor: &mut Visitor,
            ) -> Self::Result {
                #(#visit_fields_impl)*

                #res
            }
        }
    })
}

fn derive_async_visit_mut_fields_named(
    ast: &DeriveInput,
    data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let StructAttrs {
        res_ty,
        res,
        try_token,
    } = parse_attrs(&ast.attrs)?;

    let impl_t = make_impl(
        &ast,
        &data.fields,
        &res_ty,
        &syn::parse_quote! { visit_rs::AsyncVisitMutFieldsNamed },
        &syn::parse_quote! { visit_rs::AsyncVisitMut },
        Some(&syn::parse_quote! { visit_rs::NamedMut }),
        [syn::parse_quote! { Visitor: Send }],
    );

    let visit_fields_named_impl = field_name_idx_iter(&data.fields).map(|(name, idx)| {
        quote! {
            visit_rs::AsyncVisitMut::visit_mut_async(&mut visit_rs::NamedMut {
                name: #name,
                value: &mut self.#idx,
            }, visitor).await #try_token;
        }
    });

    Ok(quote! {
        #impl_t {
            type Result = #res_ty;

            async fn visit_mut_fields_named_async(
                &mut self,
                visitor: &mut Visitor,
            ) -> Self::Result {
                #(#visit_fields_named_impl)*

                #res
            }
        }
    })
}

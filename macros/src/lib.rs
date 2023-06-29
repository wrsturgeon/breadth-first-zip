/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automatically implement `BreadthFirstIterator` for tuples up to a finite but huge size.

use core::ops::RangeInclusive;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned};

const START_CHAR: u8 = b'A';
const END_CHAR: u8 = b'Q';
const TO_LOWERCASE: u8 = b'a' - b'A';

#[proc_macro]
pub fn implement(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match fallible(ts.into()) {
        Ok(ok) => ok,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn fallible(ts: TokenStream) -> syn::Result<TokenStream> {
    if !ts.is_empty() {
        return Err(syn::Error::new(ts.span(), "This macro takes no arguments"));
    }
    let mut out = TokenStream::new();
    for endc in START_CHAR..=END_CHAR {
        let chars = START_CHAR..=endc;

        // `BreadthFirstZip` implementation
        let mut a_good_start: syn::ItemImpl = syn::parse2(quote! {
            impl<TODO> BreadthFirstZip for TODO {}
        })?;
        a_good_start.generics.params = impl_generics(chars.clone())?;
        a_good_start.generics.where_clause = Some(where_clause(chars.clone())?);
        a_good_start.self_ty = Box::new(flat_tuple_type(chars.clone())?);
        a_good_start.items = vec![type_nested_equals(chars.clone())?, fn_breadth_first_zip()?];
        a_good_start.to_tokens(&mut out);

        // `Flatten` implementation
        let mut a_good_start: syn::ItemImpl = syn::parse2(quote! {
            impl<TODO> Flatten for TODO {}
        })?;
        a_good_start.generics.params = chars
            .clone()
            .map(|ref c| {
                syn::GenericParam::Type(syn::TypeParam {
                    attrs: vec![],
                    ident: cr2i(c),
                    colon_token: None,
                    bounds: Punctuated::new(),
                    eq_token: None,
                    default: None,
                })
            })
            .collect();
        a_good_start.self_ty = Box::new(huge_nested_tuple(chars.clone())?);
        a_good_start.items = vec![
            type_flattened_equals(chars.clone())?,
            fn_flatten(chars.clone())?,
        ];
        a_good_start.to_tokens(&mut out);

        // `Unflatten` implementation
        let mut a_good_start: syn::ItemImpl = syn::parse2(quote! {
            impl<TODO> Unflatten for TODO {}
        })?;
        a_good_start.generics.params = impl_generics(chars.clone())?;
        a_good_start.generics.where_clause = Some(where_clause(chars.clone())?);
        a_good_start.self_ty = Box::new(flat_tuple_type(chars.clone())?);
        a_good_start.items = vec![
            type_unflattened_equals(chars.clone())?,
            fn_unflatten(chars)?,
        ];
        a_good_start.to_tokens(&mut out);
    }
    Ok(out)
}

#[inline]
fn cr2s(c: &u8) -> &str {
    core::str::from_utf8(core::slice::from_ref(c)).unwrap()
}
#[inline]
fn cr2i(c: &u8) -> syn::Ident {
    syn::Ident::new(cr2s(c), Span::call_site())
}

#[inline]
fn impl_generics(
    chars: RangeInclusive<u8>,
) -> syn::Result<Punctuated<syn::GenericParam, syn::token::Comma>> {
    Ok(chars
        .map(|ref c| {
            syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: cr2i(c),
                colon_token: Some(syn::token::Colon {
                    spans: [Span::call_site()],
                }),
                bounds: [
                    syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: Some(syn::token::PathSep {
                                spans: [Span::call_site(), Span::call_site()],
                            }),
                            segments: [
                                syn::PathSegment {
                                    ident: syn::Ident::new("core", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                                syn::PathSegment {
                                    ident: syn::Ident::new("iter", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                                syn::PathSegment {
                                    ident: syn::Ident::new("Iterator", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                    }),
                    syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: [syn::PathSegment {
                                ident: syn::Ident::new("Clone", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    }),
                ]
                .into_iter()
                .collect(),
                eq_token: None,
                default: None,
            })
        })
        .collect())
}

#[inline]
fn where_clause(chars: RangeInclusive<u8>) -> syn::Result<syn::WhereClause> {
    Ok(syn::WhereClause {
        where_token: syn::parse2(quote!(where))?,
        predicates: chars
            .map(|ref c| {
                syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: [
                                syn::PathSegment {
                                    ident: cr2i(c),
                                    arguments: syn::PathArguments::None,
                                },
                                syn::PathSegment {
                                    ident: syn::Ident::new("Item", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                    }),
                    colon_token: syn::token::Colon {
                        spans: [Span::call_site()],
                    },
                    bounds: [syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: [syn::PathSegment {
                                ident: syn::Ident::new("Clone", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    })]
                    .into_iter()
                    .collect(),
                })
            })
            .collect(),
    })
}

#[inline]
fn flat_tuple_type(chars: RangeInclusive<u8>) -> syn::Result<syn::Type> {
    Ok(syn::Type::Tuple(syn::TypeTuple {
        paren_token: paren_token(),
        elems: chars
            .map(|ref c| {
                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: [syn::PathSegment {
                            ident: cr2i(c),
                            arguments: syn::PathArguments::None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                })
            })
            .collect(),
    }))
}

#[inline]
fn paren_token() -> syn::token::Paren {
    syn::token::Paren {
        span: proc_macro2::Group::new(proc_macro2::Delimiter::Parenthesis, TokenStream::new())
            .delim_span(),
    }
}

#[inline]
fn type_nested_equals(chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    Ok(syn::ImplItem::Type(syn::ImplItemType {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        type_token: syn::parse2(quote!(type))?,
        ident: syn::Ident::new("Nested", Span::call_site()),
        generics: syn::Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        },
        eq_token: syn::parse2(quote!(=))?,
        ty: huge_nested_type(chars)?,
        semi_token: syn::parse2(quote!(;))?,
    }))
}

#[inline]
fn huge_nested_type(chars: RangeInclusive<u8>) -> syn::Result<syn::Type> {
    Ok(chars
        .rev()
        .fold(syn::parse2(quote!(BaseCase))?, |acc, ref c| {
            syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: [syn::PathSegment {
                        ident: syn::Ident::new("BreadthFirstZipped", Span::call_site()),
                        arguments: syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: syn::token::Lt {
                                    spans: [Span::call_site()],
                                },
                                args: [
                                    syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: [syn::PathSegment {
                                                ident: cr2i(c),
                                                arguments: syn::PathArguments::None,
                                            }]
                                            .into_iter()
                                            .collect(),
                                        },
                                    })),
                                    syn::GenericArgument::Type(acc),
                                ]
                                .into_iter()
                                .collect(),
                                gt_token: syn::token::Gt {
                                    spans: [Span::call_site()],
                                },
                            },
                        ),
                    }]
                    .into_iter()
                    .collect(),
                },
            })
        }))
}

#[inline]
fn huge_nested_tuple(chars: RangeInclusive<u8>) -> syn::Result<syn::Type> {
    Ok(chars.rev().fold(syn::parse2(quote!(()))?, |acc, ref c| {
        syn::Type::Tuple(syn::TypeTuple {
            paren_token: paren_token(),
            elems: [
                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: [syn::PathSegment {
                            ident: cr2i(c),
                            arguments: syn::PathArguments::None,
                        }]
                        .into_iter()
                        .collect(),
                    },
                }),
                acc,
            ]
            .into_iter()
            .collect(),
        })
    }))
}

#[inline]
fn fn_breadth_first_zip() -> syn::Result<syn::ImplItem> {
    syn::parse2(quote! {
        #[inline(always)]
        #[must_use]
        fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str> {
            self.unflatten().map(BreadthFirstManager::new)
        }
    })
}

#[inline]
fn type_flattened_equals(chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    Ok(syn::ImplItem::Type(syn::ImplItemType {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        type_token: syn::parse2(quote!(type))?,
        ident: syn::Ident::new("Flattened", Span::call_site()),
        generics: syn::Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        },
        eq_token: syn::parse2(quote!(=))?,
        ty: flat_tuple_type(chars)?,
        semi_token: syn::parse2(quote!(;))?,
    }))
}

#[inline]
fn fn_flatten(mut chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    let mut a_good_start: syn::ImplItemFn = syn::parse2(quote! {
        #[inline(always)]
        #[must_use]
        fn flatten(self) -> Self::Flattened {}
    })?;
    chars.next(); // discard the head
    a_good_start.block.stmts = vec![
        syn::Stmt::Local(syn::Local {
            attrs: vec![],
            let_token: syn::parse2(quote!(let))?,
            pat: syn::Pat::Tuple(syn::PatTuple {
                attrs: vec![],
                paren_token: paren_token(),
                elems: chars
                    .clone()
                    .map(|ref c| {
                        syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: cr2i(&(c + TO_LOWERCASE)),
                            subpat: None,
                        })
                    })
                    .collect(),
            }),
            init: Some(syn::LocalInit {
                eq_token: syn::parse2(quote!(=))?,
                expr: Box::new(syn::parse2(if chars.len() != 1 {
                    quote!(self.1.flatten())
                } else {
                    // FIXME: The `syn` bug again
                    quote!(self.1.flatten().0)
                })?),
                diverge: None,
            }),
            semi_token: syn::parse2(quote!(;))?,
        }),
        syn::Stmt::Expr(
            syn::Expr::Tuple(syn::ExprTuple {
                attrs: vec![],
                paren_token: paren_token(),
                elems: [syn::parse2(quote!(self.0))?]
                    .into_iter()
                    .chain(chars.map(|c| {
                        syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: [syn::PathSegment {
                                    ident: cr2i(&(c + TO_LOWERCASE)),
                                    arguments: syn::PathArguments::None,
                                }]
                                .into_iter()
                                .collect(),
                            },
                        })
                    }))
                    .collect(),
            }),
            None,
        ),
    ];
    Ok(syn::ImplItem::Fn(a_good_start))
}

#[inline]
fn type_unflattened_equals(chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    Ok(syn::ImplItem::Type(syn::ImplItemType {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        type_token: syn::parse2(quote!(type))?,
        ident: syn::Ident::new("Unflattened", Span::call_site()),
        generics: syn::Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        },
        eq_token: syn::parse2(quote!(=))?,
        ty: huge_nested_type(chars)?,
        semi_token: syn::parse2(quote!(;))?,
    }))
}

#[inline]
fn fn_unflatten(mut chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    let mut a_good_start: syn::ImplItemFn = syn::parse2(quote! {
        #[inline(always)]
        #[must_use]
        fn unflatten(self) -> Result<Self::Unflattened, &'static str> {}
    })?;
    a_good_start.block.stmts = vec![
        syn::Stmt::Local(syn::Local {
            attrs: vec![],
            let_token: syn::parse2(quote!(let))?,
            pat: syn::Pat::Tuple(syn::PatTuple {
                attrs: vec![],
                paren_token: paren_token(),
                elems: chars
                    .clone()
                    .map(|c| {
                        syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: cr2i(&(c + TO_LOWERCASE)),
                            subpat: None,
                        })
                    })
                    .collect(),
            }),
            init: Some(syn::LocalInit {
                eq_token: syn::parse2(quote!(=))?,
                expr: Box::new(syn::parse2(if chars.len() != 1 {
                    quote!(self)
                } else {
                    // FIXME: The `syn` bug again
                    quote!(self.0)
                })?),
                diverge: None,
            }),
            semi_token: syn::parse2(quote!(;))?,
        }),
        syn::Stmt::Expr(
            syn::Expr::Call(syn::ExprCall {
                attrs: vec![],
                func: Box::new(syn::parse2(quote!(BreadthFirstZipped::new))?),
                paren_token: paren_token(),
                args: [
                    syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: [syn::PathSegment {
                                ident: cr2i(
                                    &(chars.next().ok_or_else(|| {
                                        syn::Error::new(
                                            Span::call_site(),
                                            "Internal error: empty char list",
                                        )
                                    })? + TO_LOWERCASE),
                                ),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    }),
                    syn::Expr::Try(syn::ExprTry {
                        attrs: vec![],
                        expr: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(syn::Expr::Tuple(syn::ExprTuple {
                                attrs: vec![],
                                paren_token: paren_token(),
                                elems: chars
                                    .map(|ref c| {
                                        syn::Expr::Path(syn::ExprPath {
                                            attrs: vec![],
                                            qself: None,
                                            path: syn::Path {
                                                leading_colon: None,
                                                segments: [syn::PathSegment {
                                                    ident: cr2i(&(c + TO_LOWERCASE)),
                                                    arguments: syn::PathArguments::None,
                                                }]
                                                .into_iter()
                                                .collect(),
                                            },
                                        })
                                    })
                                    .collect(),
                            })),
                            dot_token: syn::parse2(quote!(.))?,
                            method: syn::parse2(quote!(unflatten))?,
                            turbofish: None,
                            paren_token: paren_token(),
                            args: Punctuated::new(),
                        })),
                        question_token: syn::parse2(quote!(?))?,
                    }),
                ]
                .into_iter()
                .collect(),
            }),
            None,
        ),
    ];
    Ok(syn::ImplItem::Fn(a_good_start))
}

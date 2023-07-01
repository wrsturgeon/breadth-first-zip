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
const END_CHAR: u8 = b'A' + 15; // b'Z'; // Inclusive
const TO_LOWERCASE: u8 = b'a' - b'A';

#[proc_macro]
pub fn implement_flatten(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    flatten_fallible(ts.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn implement(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fallible(ts.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn write_tests(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tests_fallible(ts.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[inline]
fn flatten_fallible(ts: TokenStream) -> syn::Result<TokenStream> {
    if !ts.is_empty() {
        return Err(syn::Error::new(ts.span(), "This macro takes no arguments"));
    }
    let mut out = TokenStream::new();
    for endc in START_CHAR..=END_CHAR {
        let chars = START_CHAR..=endc;
        let mut a_good_start: syn::ItemImpl = syn::parse2(quote! {
            impl<TODO> crate::Flatten for TODO {}
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
    }
    Ok(out)
}

#[inline]
fn fallible(ts: TokenStream) -> syn::Result<TokenStream> {
    let is_pure = {
        let span = ts.span();
        let mut iter = ts.into_iter();
        let parsed_bool = syn::parse2::<syn::LitBool>(
            iter.next()
                .ok_or_else(|| syn::Error::new(span, "This macro takes one argument"))?
                .into_token_stream(),
        )?
        .value;
        if iter.next().is_some() {
            return Err(syn::Error::new(span, "This macro takes one argument"));
        }
        parsed_bool
    };
    let mut out = TokenStream::new();
    for endc in START_CHAR..=END_CHAR {
        let chars = START_CHAR..=endc;
        let mut a_good_start: syn::ItemImpl = syn::parse2(quote! {
            impl<TODO> BreadthFirstZip for TODO {}
        })?;
        a_good_start.generics.params = impl_generics(chars.clone(), is_pure)?;
        a_good_start.generics.where_clause = Some(where_clause(chars.clone())?);
        a_good_start.self_ty = Box::new(flat_tuple_type(chars.clone())?);
        a_good_start.items = vec![
            type_nested_equals(chars.clone())?,
            fn_breadth_first_zip()?,
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
    is_pure: bool,
) -> syn::Result<Punctuated<syn::GenericParam, syn::token::Comma>> {
    Ok(chars
        .map(|ref c| {
            syn::GenericParam::Type(syn::TypeParam {
                attrs: vec![],
                ident: cr2i(c),
                colon_token: Some(syn::token::Colon {
                    spans: [Span::call_site()],
                }),
                bounds: {
                    let iterator = syn::TypeParamBound::Trait(syn::TraitBound {
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
                    });
                    if is_pure {
                        [iterator].into_iter().collect()
                    } else {
                        [
                            iterator,
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
                        .collect()
                    }
                },
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
        .fold(syn::parse2(quote!(crate::BaseCase))?, |acc, ref c| {
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
fn fn_unflatten(mut chars: RangeInclusive<u8>) -> syn::Result<syn::ImplItem> {
    let mut a_good_start: syn::ImplItemFn = syn::parse2(quote! {
        #[inline(always)]
        #[must_use]
        fn unflatten(self) -> Result<Self::Nested, &'static str> {}
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
                        question_token: syn::token::Question {
                            spans: [Span::call_site()],
                        },
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

#[inline]
fn tests_fallible(ts: TokenStream) -> syn::Result<TokenStream> {
    if !ts.is_empty() {
        return Err(syn::Error::new(ts.span(), "This macro takes no arguments"));
    }
    Ok(quote! { #[cfg(test)] mod tests {
        use super::*;

        #[test]
        fn triples() {
            let indices = 0..3_u8;
            let mut iter = (indices.clone(), indices.clone(), indices)
                .breadth_first_zip()
                .unwrap();
            // index sum = 0
            assert_eq!(iter.next(), Some((0, 0, 0))); /* 1 item */
            // index sum = 1
            assert_eq!(iter.next(), Some((0, 0, 1)));
            assert_eq!(iter.next(), Some((0, 1, 0)));
            assert_eq!(iter.next(), Some((1, 0, 0))); /* 3 items */
            // index sum = 2
            assert_eq!(iter.next(), Some((0, 0, 2)));
            assert_eq!(iter.next(), Some((0, 1, 1)));
            assert_eq!(iter.next(), Some((0, 2, 0)));
            assert_eq!(iter.next(), Some((1, 0, 1)));
            assert_eq!(iter.next(), Some((1, 1, 0)));
            assert_eq!(iter.next(), Some((2, 0, 0))); /* 6 items */
            // index sum = 3
            assert_eq!(iter.next(), Some((0, 1, 2)));
            assert_eq!(iter.next(), Some((0, 2, 1)));
            assert_eq!(iter.next(), Some((1, 0, 2)));
            assert_eq!(iter.next(), Some((1, 1, 1)));
            assert_eq!(iter.next(), Some((1, 2, 0)));
            assert_eq!(iter.next(), Some((2, 0, 1)));
            assert_eq!(iter.next(), Some((2, 1, 0))); /* 7 items */
            // index sum = 4
            assert_eq!(iter.next(), Some((0, 2, 2)));
            assert_eq!(iter.next(), Some((1, 1, 2)));
            assert_eq!(iter.next(), Some((1, 2, 1)));
            assert_eq!(iter.next(), Some((2, 0, 2)));
            assert_eq!(iter.next(), Some((2, 1, 1)));
            assert_eq!(iter.next(), Some((2, 2, 0))); /* 6 items */
            // index sum = 5
            assert_eq!(iter.next(), Some((1, 2, 2)));
            assert_eq!(iter.next(), Some((2, 1, 2)));
            assert_eq!(iter.next(), Some((2, 2, 1))); /* 3 items */
            // index sum = 6
            assert_eq!(iter.next(), Some((2, 2, 2))); /* 1 item */
            // index sum too high
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn reduced_qc_example() { //                                               vvvvvvv For some reason, this is necessary!
            let (va, vb, vc, a0, b0, c0) = (vec![], vec![], vec![((0,),), ((1,),), ((2,),)], 0_u8, (0_u8,), ((3_u8,),));
            let va = { let mut va = va; va.push(a0); va.sort_unstable(); va.dedup(); va };
            let vb = { let mut vb = vb; vb.push(b0); vb.sort_unstable(); vb.dedup(); vb };
            let vc = { let mut vc = vc; vc.push(c0); vc.sort_unstable(); vc.dedup(); vc };
            let total_elements = va.len() * vb.len() * vc.len();
            let mut seen = ::std::collections::HashSet::new();
            let mut iter = (va.iter(), vb.iter(), vc.iter()).breadth_first_zip().unwrap();
            for _ in 0..total_elements {
                let Some((a, b, c)) = iter.next() else { panic!("Returned `None` prematurely"); };
                if seen.contains(&(a, b, c)) { panic!("Returned an element already seen"); }
                seen.insert((a, b, c));
                if !va.contains(&a) { panic!("`a` not in `A`"); }
                if !vb.contains(&b) { panic!("`b` not in `B`"); }
                if !vc.contains(&c) { panic!("`c` not in `C`"); }
            }
            if iter.next().is_some() { panic!("Kept returning after should have returned `None`"); }
        }

        mod qc {
            #![allow(warnings)]
            use super::*;

            type A = usize;
            type B = (usize,);
            type C = ((usize,),);

            quickcheck::quickcheck! {
                fn prop_everything(va: Vec<A>, vb: Vec<B>, vc: Vec<C>, a0: A, b0: B, c0: C) -> bool {
                    let va = { let mut va = va; va.push(a0); va.sort_unstable(); va.dedup(); va };
                    let vb = { let mut vb = vb; vb.push(b0); vb.sort_unstable(); vb.dedup(); vb };
                    let vc = { let mut vc = vc; vc.push(c0); vc.sort_unstable(); vc.dedup(); vc };
                    let total_elements = va.len() * vb.len() * vc.len();
                    let mut seen = ::std::collections::HashSet::new();
                    let mut iter = (va.iter(), vb.iter(), vc.iter()).breadth_first_zip().unwrap();
                    for _ in 0..total_elements {
                        let Some((a, b, c)) = iter.next() else { panic!("Returned `None` prematurely"); return false; };
                        if seen.contains(&(a, b, c)) { panic!("Returned an element already seen"); return false; }
                        seen.insert((a, b, c));
                        if !va.contains(&a) { panic!("`a` not in `A`"); return false; }
                        if !vb.contains(&b) { panic!("`b` not in `B`"); return false; }
                        if !vc.contains(&c) { panic!("`c` not in `C`"); return false; }
                    }
                    if iter.next().is_some() { panic!("Kept returning after should have returned `None`"); return false; }
                    true
                }
            }
        }
    }})
}

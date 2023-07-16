use crate::{parse_function, Ident, InputEnum, Span};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{self},
    Arm, Expr, FieldValue, Fields, ItemFn, Lifetime, Type,
};

pub(crate) struct EnumFunctionsBuilder<'a> {
    input: &'a InputEnum,
    functions: Vec<ItemFn>,
}

impl<'a> EnumFunctionsBuilder<'a> {
    pub(crate) fn new(input: &'a InputEnum) -> Self {
        Self {
            input,
            functions: vec![],
        }
    }

    pub(crate) fn is_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("is_{nm}").as_str(), Span::call_site());
            let pat = self.input.match_variant(i, None);
            let ts = quote! {
                #vs fn #sp (&self) -> bool {
                    matches!(self, #pat)
                }
            };

            let mut ifn = None;
            parse_function(ts, &mut ifn);

            if let Some(ifn) = ifn {
                self.functions.push(ifn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn unwrap_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("unwrap_{nm}").as_str(), Span::call_site());
            let arm = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);
            let ts = quote! {
                #vs fn #sp (self) -> #return_type {
                    match self {
                        #arm
                        _ => panic!()
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn unwrap_ref_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("unwrap_ref_{nm}").as_str(), Span::call_site());
            let arm = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);

            let rt = match return_type {
                ty @ Type::Path(_) => Type::Reference(syn::TypeReference {
                    and_token: token::And {
                        spans: [Span::call_site(); 1],
                    },
                    lifetime: None,
                    mutability: None,
                    elem: Box::new(ty),
                }),
                Type::Tuple(tuple) => Type::Tuple(syn::TypeTuple {
                    paren_token: tuple.paren_token,
                    elems: tuple
                        .elems
                        .into_iter()
                        .map(|t| {
                            Type::Reference(syn::TypeReference {
                                and_token: token::And {
                                    spans: [Span::call_site(); 1],
                                },
                                lifetime: None,
                                mutability: None,
                                elem: Box::new(t),
                            })
                        })
                        .collect(),
                }),
                _ => panic!(),
            };

            let ts = quote! {
                #vs fn #sp (&self) -> #rt {
                    match self {
                        #arm
                        _ => panic!()
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn unwrap_mut_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("unwrap_mut_{nm}").as_str(), Span::call_site());
            let arm = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);
            let mutability = Some(token::Mut {
                span: Span::call_site(),
            });

            let rt = match return_type {
                ty @ Type::Path(_) => Type::Reference(syn::TypeReference {
                    and_token: token::And {
                        spans: [Span::call_site(); 1],
                    },
                    lifetime: None,
                    mutability,
                    elem: Box::new(ty),
                }),
                Type::Tuple(tuple) => Type::Tuple(syn::TypeTuple {
                    paren_token: tuple.paren_token,
                    elems: tuple
                        .elems
                        .into_iter()
                        .map(|t| {
                            Type::Reference(syn::TypeReference {
                                and_token: token::And {
                                    spans: [Span::call_site(); 1],
                                },
                                lifetime: None,
                                mutability,
                                elem: Box::new(t),
                            })
                        })
                        .collect(),
                }),
                _ => panic!("Unexpected type."),
            };

            let ts = quote! {
                #vs fn #sp (&mut self) -> #rt {
                    match self {
                        #arm
                        _ => panic!()
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_tag_function(&mut self) {
        let vs = self.input.vis();
        let sp = Ident::new("to_tag", Span::call_site());

        let tag_ident = Ident::new(
            format!("{}Tag", self.input.0.ident).as_str(),
            Span::call_site(),
        );

        let mut arms = vec![];
        for i in 0..self.input.variant_count() {
            let variant_ident = &self.input.variant(i).ident;
            let body = quote! {
                #tag_ident :: #variant_ident
            };

            arms.push(Arm {
                attrs: vec![],
                guard: None,
                fat_arrow_token: token::FatArrow {
                    spans: [Span::call_site(), Span::call_site()],
                },
                comma: Some(token::Comma {
                    spans: [Span::call_site()],
                }),
                body: Box::new(Expr::Verbatim(body)),
                pat: self.input.match_variant(i, None),
            });
        }

        let ts = quote! {
            #vs fn #sp (&self) -> #tag_ident {
                match self {
                    #(#arms)*
                }
            }
        };

        let mut ifn = None;
        parse_function(ts, &mut ifn);

        if let Some(ifn) = ifn {
            self.functions.push(ifn);
        } else {
            panic!()
        }
    }

    pub(crate) fn as_ref_functions(&mut self) {
        let vs = self.input.vis();
        let sp = Ident::new("as_ref", Span::call_site());

        let ref_ident = Ident::new(
            format!("{}Ref", self.input.0.ident).as_str(),
            Span::call_site(),
        );

        let mut arms = vec![];
        for i in 0..self.input.variant_count() {
            let _variant_ident = &self.input.variant(i).ident;
            let syn::Arm {
                attrs,
                pat,
                guard,
                fat_arrow_token,
                comma,
                body,
            } = self.input.match_variant_to_tuple(i, None);

            let variant_ident = &self.input.variant(i).ident;
            let body = match &self.input.variant(i).fields {
                Fields::Unit => Expr::Verbatim(quote! {
                    #ref_ident :: #variant_ident ,
                }),
                Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.len() == 1 {
                        Expr::Verbatim(quote! {
                            #ref_ident :: #variant_ident ( #body )
                        })
                    } else {
                        Expr::Verbatim(quote! {
                            #ref_ident :: #variant_ident #body
                        })
                    }
                }
                Fields::Named(named) => {
                    let group = proc_macro2::Group::new(
                        proc_macro2::Delimiter::Parenthesis,
                        proc_macro2::TokenStream::new(),
                    );
                    let body = Expr::Struct(syn::ExprStruct {
                        attrs: vec![],
                        qself: None,
                        brace_token: token::Brace {
                            span: group.delim_span(),
                        },
                        dot2_token: None,
                        rest: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![
                                syn::PathSegment {
                                    arguments: syn::PathArguments::None,
                                    ident: ref_ident.clone(),
                                },
                                syn::PathSegment {
                                    arguments: syn::PathArguments::None,
                                    ident: variant_ident.clone(),
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                        fields: named
                            .named
                            .iter()
                            .map(|f| FieldValue {
                                attrs: vec![],
                                colon_token: None,
                                member: syn::Member::Named(f.ident.clone().unwrap()),
                                expr: Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: [syn::PathSegment {
                                            arguments: syn::PathArguments::None,
                                            ident: f.ident.clone().unwrap(),
                                        }]
                                        .into_iter()
                                        .collect(),
                                    },
                                }),
                            })
                            .collect(),
                    });
                    Expr::Verbatim(quote! {
                        #body
                    })
                }
            };

            arms.push(Arm {
                attrs,
                guard,
                fat_arrow_token,
                comma,
                body: Box::new(body),
                pat,
            });
        }

        let mut generics = self.input.generics().clone();
        generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: vec![],
                lifetime: Lifetime::new("'_", Span::call_site()),
                bounds: Punctuated::new(),
                colon_token: None,
            }));

        let ts = quote! {
            #vs fn #sp (&self) -> #ref_ident #generics {
                match self {
                    #(#arms)*
                }
            }
        };

        let mut ifn = None;
        parse_function(ts, &mut ifn);

        if let Some(ifn) = ifn {
            self.functions.push(ifn);
        } else {
            panic!()
        }
    }

    pub(crate) fn as_mut_functions(&mut self) {
        let vs = self.input.vis();
        let sp = Ident::new("as_mut", Span::call_site());

        let ref_ident = Ident::new(
            format!("{}Mut", self.input.0.ident).as_str(),
            Span::call_site(),
        );

        let mut arms = vec![];
        for i in 0..self.input.variant_count() {
            let _variant_ident = &self.input.variant(i).ident;
            let syn::Arm {
                attrs,
                pat,
                guard,
                fat_arrow_token,
                comma,
                body,
            } = self.input.match_variant_to_tuple(i, None);

            let variant_ident = &self.input.variant(i).ident;
            let body = match &self.input.variant(i).fields {
                Fields::Unit => Expr::Verbatim(quote! {
                    #ref_ident :: #variant_ident ,
                }),
                Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.len() == 1 {
                        Expr::Verbatim(quote! {
                            #ref_ident :: #variant_ident ( #body )
                        })
                    } else {
                        Expr::Verbatim(quote! {
                            #ref_ident :: #variant_ident #body
                        })
                    }
                }
                Fields::Named(named) => {
                    let group = proc_macro2::Group::new(
                        proc_macro2::Delimiter::Parenthesis,
                        proc_macro2::TokenStream::new(),
                    );
                    let body = Expr::Struct(syn::ExprStruct {
                        attrs: vec![],
                        qself: None,
                        brace_token: token::Brace {
                            span: group.delim_span(),
                        },
                        dot2_token: None,
                        rest: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![
                                syn::PathSegment {
                                    arguments: syn::PathArguments::None,
                                    ident: ref_ident.clone(),
                                },
                                syn::PathSegment {
                                    arguments: syn::PathArguments::None,
                                    ident: variant_ident.clone(),
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                        fields: named
                            .named
                            .iter()
                            .map(|f| FieldValue {
                                attrs: vec![],
                                colon_token: None,
                                member: syn::Member::Named(f.ident.clone().unwrap()),
                                expr: Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: [syn::PathSegment {
                                            arguments: syn::PathArguments::None,
                                            ident: f.ident.clone().unwrap(),
                                        }]
                                        .into_iter()
                                        .collect(),
                                    },
                                }),
                            })
                            .collect(),
                    });
                    Expr::Verbatim(quote! {
                        #body
                    })
                }
            };

            arms.push(Arm {
                attrs,
                guard,
                fat_arrow_token,
                comma,
                body: Box::new(body),
                pat,
            });
        }

        let mut generics = self.input.generics().clone();
        generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: vec![],
                lifetime: Lifetime::new("'_", Span::call_site()),
                bounds: Punctuated::new(),
                colon_token: None,
            }));

        let ts = quote! {
            #vs fn #sp (&mut self) -> #ref_ident #generics {
                match self {
                    #(#arms)*
                }
            }
        };

        let mut ifn = None;
        parse_function(ts, &mut ifn);

        if let Some(ifn) = ifn {
            self.functions.push(ifn);
        } else {
            panic!()
        }
    }

    pub(crate) fn get_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("get_{nm}").as_str(), Span::call_site());
            let syn::Arm { pat, body, .. } = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);
            let ts = quote! {
                #vs fn #sp (self) -> Option<#return_type> {
                    match self {
                        #pat => { Some ( #body ) },
                        _ => None
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn get_ref_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("get_ref_{nm}").as_str(), Span::call_site());
            let syn::Arm { pat, body, .. } = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);

            let rt = match return_type {
                ty @ Type::Path(_) => Type::Reference(syn::TypeReference {
                    and_token: token::And {
                        spans: [Span::call_site(); 1],
                    },
                    lifetime: None,
                    mutability: None,
                    elem: Box::new(ty),
                }),
                Type::Tuple(tuple) => Type::Tuple(syn::TypeTuple {
                    paren_token: tuple.paren_token,
                    elems: tuple
                        .elems
                        .into_iter()
                        .map(|t| {
                            Type::Reference(syn::TypeReference {
                                and_token: token::And {
                                    spans: [Span::call_site(); 1],
                                },
                                lifetime: None,
                                mutability: None,
                                elem: Box::new(t),
                            })
                        })
                        .collect(),
                }),
                _ => panic!(),
            };

            let ts = quote! {
                #vs fn #sp (&self) -> Option<#rt> {
                    match self {
                        #pat => { Some ( #body ) },
                        _ => None
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn get_mut_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("get_mut_{nm}").as_str(), Span::call_site());
            let syn::Arm { pat, body, .. } = self.input.match_variant_to_tuple(i, None);

            let return_type = self.input.variant_type(i);
            let mutability = Some(token::Mut {
                span: Span::call_site(),
            });

            let rt = match return_type {
                ty @ Type::Path(_) => Type::Reference(syn::TypeReference {
                    and_token: token::And {
                        spans: [Span::call_site(); 1],
                    },
                    lifetime: None,
                    mutability,
                    elem: Box::new(ty),
                }),
                Type::Tuple(tuple) => Type::Tuple(syn::TypeTuple {
                    paren_token: tuple.paren_token,
                    elems: tuple
                        .elems
                        .into_iter()
                        .map(|t| {
                            Type::Reference(syn::TypeReference {
                                and_token: token::And {
                                    spans: [Span::call_site(); 1],
                                },
                                lifetime: None,
                                mutability,
                                elem: Box::new(t),
                            })
                        })
                        .collect(),
                }),
                _ => panic!(),
            };

            let ts = quote! {
                #vs fn #sp (&mut self) -> Option<#rt> {
                    match self {
                        #pat => { Some ( #body ) },
                        _ => panic!()
                    }
                }
            };

            let mut ufn = None;
            parse_function(ts, &mut ufn);

            if let Some(ufn) = ufn {
                self.functions.push(ufn);
            } else {
                panic!()
            }
        }
    }

    pub(crate) fn token_stream(&self) -> TokenStream {
        let _visibility = &self.input.vis();
        let enum_name = &self.input.0.ident;
        let functions = &self.functions;
        let (impl_g, type_g, where_clause) = self.input.generics().split_for_impl();

        if !functions.is_empty() {
            let functions = quote! {
                impl #impl_g #enum_name #type_g #where_clause {
                    #(#functions)*
                }
            };
            TokenStream::from(functions)
        } else {
            TokenStream::new()
        }
    }
}

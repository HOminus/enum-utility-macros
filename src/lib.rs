#![doc = include_str!("../README.md")]

use functions_builder::EnumFunctionsBuilder;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use ref_enum_builder::RefEnumBuilder;
use syn::{
    parse::Parser,
    parse_macro_input,
    punctuated::Punctuated,
    token::{self},
    Expr, Fields, ItemEnum, ItemFn, Token, Type, TypeTuple, Variant, Visibility,
};
use tag_enum_builder::TagEnumBuilder;

pub(crate) mod functions_builder;
pub(crate) mod ref_enum_builder;
pub(crate) mod tag_enum_builder;

#[proc_macro_attribute]
pub fn generate_enum_helper(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut enum_stream = item.clone();

    let parser = Punctuated::<Ident, Token![,]>::parse_separated_nonempty;
    let attributes = parser.parse(attr).unwrap();
    let input = parse_macro_input!(item as ItemEnum);

    let mut generate_tag_enum = false;
    let mut generate_ref_enum = false;
    let mut generate_mut_enum = false;

    let mut create_is_functions = false;
    let mut create_unwrap_functions = false;
    let mut create_unwrap_ref_functions = false;
    let mut create_unwrap_ref_mut_functions = false;
    let mut create_to_tag_functions = false;
    let mut create_as_ref_functions = false;
    let mut create_as_mut_functions = false;
    let mut create_get_functions = false;
    let mut create_get_ref_functions = false;
    let mut create_get_mut_functions = false;
    for item in attributes {
        match item.to_string().as_str() {
            "TagEnum" => generate_tag_enum = true,
            "RefEnum" => generate_ref_enum = true,
            "MutEnum" => generate_mut_enum = true,
            "is" => create_is_functions = true,
            "unwrap" => create_unwrap_functions = true,
            "unwrap_ref" => create_unwrap_ref_functions = true,
            "unwrap_mut" => create_unwrap_ref_mut_functions = true,
            "to_tag" => create_to_tag_functions = true,
            "as_ref" => create_as_ref_functions = true,
            "as_mut" => create_as_mut_functions = true,
            "get" => create_get_functions = true,
            "get_ref" => create_get_ref_functions = true,
            "get_mut" => create_get_mut_functions = true,
            _ => panic!(),
        }
    }

    let input_enum = InputEnum(input);
    if create_is_functions
        || create_unwrap_functions
        || create_unwrap_ref_functions
        || create_unwrap_ref_mut_functions
        || create_to_tag_functions
        || create_as_ref_functions
        || create_as_mut_functions
        || create_get_functions
        || create_get_ref_functions
        || create_get_mut_functions
    {
        let mut functions_builder = EnumFunctionsBuilder::new(&input_enum);
        if create_is_functions {
            functions_builder.is_functions();
        }
        if create_unwrap_functions {
            functions_builder.unwrap_functions();
        }
        if create_unwrap_ref_functions {
            functions_builder.unwrap_ref_functions();
        }
        if create_unwrap_ref_mut_functions {
            functions_builder.unwrap_mut_functions();
        }
        if create_to_tag_functions {
            functions_builder.to_tag_function();
        }
        if create_as_ref_functions {
            functions_builder.as_ref_functions();
        }
        if create_as_mut_functions {
            functions_builder.as_mut_functions();
        }
        if create_get_functions {
            functions_builder.get_functions();
        }
        if create_get_ref_functions {
            functions_builder.get_ref_functions();
        }
        if create_get_mut_functions {
            functions_builder.get_mut_functions();
        }

        let ts = functions_builder.token_stream();
        enum_stream.extend([ts]);
    }

    if generate_tag_enum {
        let mut tag_enum_builder = TagEnumBuilder::new(&input_enum);
        if create_is_functions {
            tag_enum_builder.is_functions();
        }
        let ts = tag_enum_builder.token_stream();
        enum_stream.extend([ts]);
    }

    if generate_ref_enum {
        let mut ref_enum_builder = RefEnumBuilder::new(&input_enum, false);
        if create_is_functions {
            ref_enum_builder.is_functions();
        }
        if create_unwrap_functions {
            ref_enum_builder.unwrap_functions();
        }
        if create_to_tag_functions {
            ref_enum_builder.to_tag_functions();
        }
        if create_get_functions {
            ref_enum_builder.get_functions();
        }
        let ts = ref_enum_builder.token_stream();
        enum_stream.extend([ts]);
    }

    if generate_mut_enum {
        let mut ref_enum_builder = RefEnumBuilder::new(&input_enum, true);
        if create_is_functions {
            ref_enum_builder.is_functions();
        }
        if create_unwrap_functions {
            ref_enum_builder.unwrap_functions();
        }
        if create_to_tag_functions {
            ref_enum_builder.to_tag_functions();
        }
        if create_get_functions {
            ref_enum_builder.get_functions();
        }
        let ts = ref_enum_builder.token_stream();
        enum_stream.extend([ts]);
    }

    enum_stream
}

pub(crate) struct InputEnum(ItemEnum);

impl InputEnum {
    fn vis(&self) -> &Visibility {
        &self.0.vis
    }

    fn name(&self) -> String {
        format!("{}", self.0.ident)
    }

    fn variant_snake_case_name(&self, i: usize) -> String {
        let variant_name = self.0.variants[i].ident.to_string();
        let mut snake_case_name = String::new();
        for c in variant_name.chars() {
            if c.is_uppercase() && snake_case_name.is_empty() {
                snake_case_name += format!("{}", c.to_ascii_lowercase()).as_str();
            } else if c.is_uppercase() {
                snake_case_name += format!("_{}", c.to_ascii_lowercase()).as_str();
            } else {
                snake_case_name += format!("{c}").as_str();
            }
        }
        snake_case_name
    }

    fn generics(&self) -> &syn::Generics {
        &self.0.generics
    }

    fn attributes(&self) -> &Vec<syn::Attribute> {
        &self.0.attrs
    }

    fn iter_variants(&self) -> impl Iterator<Item = &Variant> {
        self.0.variants.iter()
    }

    fn variant_count(&self) -> usize {
        self.0.variants.len()
    }

    fn variant(&self, i: usize) -> &Variant {
        &self.0.variants[i]
    }

    fn variant_type(&self, i: usize) -> Type {
        let elems: Punctuated<_, _> = self.0.variants[i]
            .fields
            .iter()
            .map(|f| f.ty.clone())
            .collect();

        if elems.len() == 1 {
            return (*elems.first().unwrap()).clone();
        }

        let group = proc_macro2::Group::new(
            proc_macro2::Delimiter::Parenthesis,
            proc_macro2::TokenStream::new(),
        );
        syn::Type::Tuple(TypeTuple {
            paren_token: token::Paren {
                span: group.delim_span(),
            },
            elems,
        })
    }

    fn match_variant(&self, i: usize, enum_ident: Option<Ident>) -> syn::Pat {
        let variant = self.variant(i);
        let enum_name = enum_ident.as_ref().unwrap_or(&self.0.ident);
        let variant_name = &self.variant(i).ident;
        let pattern = match &variant.fields {
            Fields::Unit => {
                quote! {
                    #enum_name :: #variant_name
                }
            }
            Fields::Named(_) => {
                quote! {
                    #enum_name :: #variant_name { .. }
                }
            }
            Fields::Unnamed(fields) => {
                let wild_pattern = vec![
                    syn::Pat::Wild(syn::PatWild {
                        attrs: vec![],
                        underscore_token: token::Underscore {
                            spans: [Span::call_site(); 1]
                        },
                    });
                    fields.unnamed.len()
                ];

                quote! {
                    #enum_name :: #variant_name ( #(#wild_pattern ,)* )
                }
            }
        };

        syn::Pat::Verbatim(pattern)
    }

    fn match_variant_to_tuple(&self, i: usize, enum_ident: Option<Ident>) -> syn::Arm {
        let (pat, body) = match &self.variant(i).fields {
            Fields::Unit => {
                let group = proc_macro2::Group::new(
                    proc_macro2::Delimiter::Parenthesis,
                    proc_macro2::TokenStream::new(),
                );

                (
                    self.match_variant(i, None),
                    Box::new(Expr::Tuple(syn::ExprTuple {
                        attrs: vec![],
                        paren_token: token::Paren {
                            span: group.delim_span(),
                        },
                        elems: Punctuated::new(),
                    })),
                )
            }
            Fields::Unnamed(fields) => {
                let mut patterns = Punctuated::new();
                let mut elements = Punctuated::new();

                for (index, _field) in fields.unnamed.iter().enumerate() {
                    let name = format!("e{index}");
                    let ident = Ident::new(name.as_str(), Span::call_site());
                    patterns.push(syn::Pat::Path(syn::PatPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::PathSegment {
                            arguments: syn::PathArguments::None,
                            ident: ident.clone(),
                        }
                        .into(),
                    }));

                    elements.push(Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::PathSegment {
                            arguments: syn::PathArguments::None,
                            ident,
                        }
                        .into(),
                    }))
                }

                let pattern_path = {
                    let mut punctuated = Punctuated::new();
                    punctuated.push(syn::PathSegment {
                        ident: enum_ident.unwrap_or(self.0.ident.clone()),
                        arguments: syn::PathArguments::None,
                    });
                    punctuated.push(syn::PathSegment {
                        ident: self.variant(i).ident.clone(),
                        arguments: syn::PathArguments::None,
                    });

                    syn::Path {
                        leading_colon: None,
                        segments: punctuated,
                    }
                };

                let group = proc_macro2::Group::new(
                    proc_macro2::Delimiter::Parenthesis,
                    proc_macro2::TokenStream::new(),
                );
                let pat = syn::Pat::TupleStruct(syn::PatTupleStruct {
                    attrs: vec![],
                    qself: None,
                    path: pattern_path,
                    paren_token: token::Paren {
                        span: group.delim_span(),
                    },
                    elems: patterns,
                });

                let body = if elements.len() == 1 {
                    let syn::Expr::Path(syn::ExprPath { path, ..}) = elements.first().unwrap() else {
                        panic!()
                    };
                    Box::new(Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: path.clone(),
                    }))
                } else {
                    Box::new(Expr::Tuple(syn::ExprTuple {
                        attrs: vec![],
                        paren_token: token::Paren {
                            span: group.delim_span(),
                        },
                        elems: elements,
                    }))
                };

                (pat, body)
            }
            Fields::Named(fields) => {
                // Unnify with unnamed
                let mut patterns = Punctuated::new();
                let mut elements = Punctuated::new();

                for field in fields.named.iter() {
                    patterns.push(syn::FieldPat {
                        attrs: vec![],
                        member: syn::Member::Named(field.ident.clone().unwrap()),
                        colon_token: None, // Shorthand field pattern
                        pat: Box::new(syn::Pat::Path(syn::PatPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::PathSegment {
                                arguments: syn::PathArguments::None,
                                ident: field.ident.clone().unwrap(),
                            }
                            .into(),
                        })),
                    });

                    elements.push(Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::PathSegment {
                            arguments: syn::PathArguments::None,
                            ident: field.ident.clone().unwrap(),
                        }
                        .into(),
                    }))
                }

                let pattern_path = {
                    let mut punctuated = Punctuated::new();
                    punctuated.push(syn::PathSegment {
                        ident: enum_ident.unwrap_or(self.0.ident.clone()),
                        arguments: syn::PathArguments::None,
                    });
                    punctuated.push(syn::PathSegment {
                        ident: self.variant(i).ident.clone(),
                        arguments: syn::PathArguments::None,
                    });

                    syn::Path {
                        leading_colon: None,
                        segments: punctuated,
                    }
                };

                let group = proc_macro2::Group::new(
                    proc_macro2::Delimiter::Parenthesis,
                    proc_macro2::TokenStream::new(),
                );
                let pat = syn::Pat::Struct(syn::PatStruct {
                    attrs: vec![],
                    qself: None,
                    path: pattern_path,
                    brace_token: token::Brace {
                        span: group.delim_span(),
                    },
                    fields: patterns,
                    rest: None,
                });

                let body = if elements.len() == 1 {
                    let syn::Expr::Path(syn::ExprPath { path, ..}) = elements.first().unwrap() else {
                        panic!()
                    };
                    Box::new(Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: path.clone(),
                    }))
                } else {
                    Box::new(Expr::Tuple(syn::ExprTuple {
                        attrs: vec![],
                        paren_token: token::Paren {
                            span: group.delim_span(),
                        },
                        elems: elements,
                    }))
                };

                (pat, body)
            }
        };

        syn::Arm {
            attrs: vec![],
            guard: None,
            fat_arrow_token: token::FatArrow {
                spans: [Span::call_site(); 2],
            },
            comma: Some(token::Comma {
                spans: [Span::call_site(); 1],
            }),
            pat,
            body,
        }
    }
}

pub(crate) fn parse_function(
    ts: proc_macro2::TokenStream,
    ifn: &mut Option<ItemFn>,
) -> TokenStream {
    let r = TokenStream::from(ts);
    let r2 = r.clone();
    let pifn = parse_macro_input!(r2 as ItemFn);
    *ifn = Some(pifn);
    r
}

fn filter_derive_attributes(
    attrs: &[syn::Attribute],
    filtered_out: &[&str],
) -> Vec<syn::Attribute> {
    let mut result = vec![];
    for attr in attrs {
        match &attr.meta {
            syn::Meta::List(ml) if ml.path.to_token_stream().to_string() == "derive" => {
                let punctuated_parser = Punctuated::<syn::Path, Token![,]>::parse_terminated;
                let punctuated = punctuated_parser.parse2(ml.tokens.clone()).unwrap();

                let mut punctuated_result = Punctuated::<_, Token![,]>::new();
                for item in punctuated.into_iter() {
                    let last_segment = item.segments.last().unwrap().ident.to_string();
                    if filtered_out.contains(&last_segment.as_str()) {
                        continue;
                    }
                    punctuated_result.push(item);
                }
                result.push(syn::Attribute {
                    pound_token: attr.pound_token,
                    style: attr.style,
                    bracket_token: attr.bracket_token,
                    meta: syn::Meta::List(syn::MetaList {
                        path: ml.path.clone(),
                        delimiter: ml.delimiter.clone(),
                        tokens: punctuated_result.to_token_stream(),
                    }),
                });
            }
            _ => result.push(attr.clone()),
        }
    }
    result
}

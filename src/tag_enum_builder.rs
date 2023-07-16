use crate::{parse_function, InputEnum};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Fields, ItemFn, Variant, Visibility};

pub(crate) struct TagEnumBuilder<'a> {
    input: &'a InputEnum,
    visibility: Visibility,
    tag_enum_name: Ident,
    variants: Vec<Variant>,
    functions: Vec<ItemFn>,
}

impl<'a> TagEnumBuilder<'a> {
    pub(crate) fn new(input: &'a InputEnum) -> Self {
        let vis = input.vis().clone();
        let tag_enum_name = Ident::new(format!("{}Tag", input.name()).as_str(), Span::call_site());
        let mut this = Self {
            input,
            visibility: vis,
            tag_enum_name,
            variants: vec![],
            functions: vec![],
        };
        this.map_variants();
        this
    }

    fn map_variants(&mut self) {
        for variant in self.input.iter_variants() {
            self.variants.push(Variant {
                attrs: variant.attrs.clone(),
                ident: variant.ident.clone(),
                fields: Fields::Unit,
                discriminant: variant.discriminant.clone(),
            });
        }
    }

    pub(crate) fn is_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("is_{nm}").as_str(), Span::call_site());

            let eident = &self.tag_enum_name;
            let vident = &self.variants[i].ident;

            let ts = quote! {
                #vs fn #sp (&self) -> bool {
                    matches!(self, #eident :: #vident)
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

    pub(crate) fn token_stream(&self) -> TokenStream {
        let visibility = &self.visibility;
        let tag_enum_name = &self.tag_enum_name;
        let tag_enum_variants = &self.variants;
        let tag_enum = quote! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            #visibility enum #tag_enum_name {
                #(#tag_enum_variants ,)*
            }
        };
        let mut tag_enum_stream = TokenStream::from(tag_enum);

        if !self.functions.is_empty() {
            let functions = &self.functions;
            let tag_functions = quote! {
                impl #tag_enum_name {
                    #(#functions)*
                }
            };
            tag_enum_stream.extend([TokenStream::from(tag_functions)]);
        }

        tag_enum_stream
    }
}

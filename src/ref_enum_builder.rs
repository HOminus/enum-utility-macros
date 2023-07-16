use crate::{parse_function, InputEnum};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{self, And},
    ItemFn, Lifetime, Type, TypeReference, TypeTuple, Variant, Visibility,
};

pub(crate) struct RefEnumBuilder<'a> {
    input: &'a InputEnum,
    mutable: bool,
    generics: syn::Generics,
    lifetime: Lifetime,
    visibility: Visibility,
    ref_enum_name: Ident,
    variants: Vec<Variant>,
    functions: Vec<ItemFn>,
}

impl<'a> RefEnumBuilder<'a> {
    pub(crate) fn new(input: &'a InputEnum, mutable: bool) -> Self {
        let lifetime = Lifetime::new("'reb", Span::call_site());

        let ident = if mutable {
            Ident::new(format!("{}Mut", input.name()).as_str(), Span::call_site())
        } else {
            Ident::new(format!("{}Ref", input.name()).as_str(), Span::call_site())
        };
        let mut this = Self {
            input,
            mutable,
            visibility: input.0.vis.clone(),
            ref_enum_name: ident,
            generics: input.generics().clone(),
            lifetime,
            variants: vec![],
            functions: vec![],
        };
        this.map_variants();
        this.adjust_generics();
        this
    }

    fn map_variants(&mut self) {
        let mutability = if self.mutable {
            Some(token::Mut {
                span: Span::call_site(),
            })
        } else {
            None
        };
        for variant in self.input.iter_variants() {
            let mut fields = variant.fields.clone();
            fields.iter_mut().for_each(|f| {
                f.ty = Type::Reference(TypeReference {
                    and_token: And {
                        spans: [Span::call_site(); 1],
                    },
                    lifetime: Some(self.lifetime.clone()),
                    mutability,
                    elem: Box::new(f.ty.clone()),
                })
            });
            self.variants.push(Variant {
                attrs: variant.attrs.clone(),
                ident: variant.ident.clone(),
                fields,
                discriminant: variant.discriminant.clone(),
            });
        }
    }

    fn variant_type(&self, i: usize) -> syn::Type {
        let elems: Punctuated<_, _> = self.variants[i]
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

    fn adjust_generics(&mut self) {
        self.generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: vec![],
                bounds: Punctuated::new(),
                colon_token: None,
                lifetime: self.lifetime.clone(),
            }));
    }

    pub(crate) fn is_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("is_{nm}").as_str(), Span::call_site());
            let pat = self
                .input
                .match_variant(i, Some(self.ref_enum_name.clone())); // Same match clause
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
            let arm = self
                .input
                .match_variant_to_tuple(i, Some(self.ref_enum_name.clone()));

            let return_type = self.variant_type(i);
            let lifetime = &self.lifetime;
            let ts = if self.mutable {
                quote! {
                    #vs fn #sp (& #lifetime mut self) -> #return_type {
                        match self {
                            #arm
                            _ => panic!()
                        }
                    }
                }
            } else {
                quote! {
                    #vs fn #sp (& #lifetime self) -> #return_type {
                        match self {
                            #arm
                            _ => panic!()
                        }
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

    pub(crate) fn get_functions(&mut self) {
        let vs = self.input.vis();
        for i in 0..self.input.variant_count() {
            let nm = self.input.variant_snake_case_name(i);
            let sp = Ident::new(format!("get_{nm}").as_str(), Span::call_site());
            let syn::Arm { pat, body, .. } = self
                .input
                .match_variant_to_tuple(i, Some(self.ref_enum_name.clone()));

            let return_type = self.variant_type(i);
            let lifetime = &self.lifetime;
            let ts = if self.mutable {
                quote! {
                    #vs fn #sp (& #lifetime mut self) -> Option<#return_type> {
                        match self {
                            #pat => { Some( #body ) },
                            _ => None
                        }
                    }
                }
            } else {
                quote! {
                    #vs fn #sp (& #lifetime self) -> Option<#return_type> {
                        match self {
                            #pat => { Some ( #body ) },
                            _ => None
                        }
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
    pub(crate) fn to_tag_functions(&mut self) {
        let vs = self.input.vis();
        let sp = Ident::new("to_tag", Span::call_site());

        let tag_ident = Ident::new(
            format!("{}Tag", self.input.0.ident).as_str(),
            Span::call_site(),
        );

        let mut arms = vec![];
        for i in 0..self.input.variant_count() {
            let variant_ident = &self.variants[i].ident;
            let body = quote! {
                #tag_ident :: #variant_ident
            };

            arms.push(syn::Arm {
                attrs: vec![],
                guard: None,
                fat_arrow_token: token::FatArrow {
                    spans: [Span::call_site(), Span::call_site()],
                },
                comma: Some(token::Comma {
                    spans: [Span::call_site()],
                }),
                body: Box::new(syn::Expr::Verbatim(body)),
                pat: self
                    .input
                    .match_variant(i, Some(self.ref_enum_name.clone())),
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

    pub(crate) fn token_stream(&self) -> TokenStream {
        let visibility = &self.visibility;
        let ref_enum_name = &self.ref_enum_name;
        let ref_enum_variants = &self.variants;

        let attributes = self.input.attributes();
        let attributes = if self.mutable {
            super::filter_derive_attributes(attributes.as_slice(), &["Clone", "Copy"])
        } else {
            super::filter_derive_attributes(attributes.as_slice(), &[])
        };

        let generics = &self.generics;
        let ref_enum = quote! {
            #(#attributes)*
            #visibility enum #ref_enum_name #generics {
                #(#ref_enum_variants ,)*
            }
        };
        let mut ref_enum_stream = TokenStream::from(ref_enum);

        if !self.functions.is_empty() {
            let (impl_g, type_g, where_g) = self.generics.split_for_impl();
            let functions = &self.functions;
            let ref_functions = quote! {
                impl #impl_g #ref_enum_name #type_g #where_g {
                    #(#functions)*
                }
            };
            ref_enum_stream.extend([TokenStream::from(ref_functions)]);
        }

        ref_enum_stream
    }
}

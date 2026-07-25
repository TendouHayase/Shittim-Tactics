extern crate proc_macro;

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, Ident, ItemTrait, Signature, Token, TraitItem, parse::Parse,
    parse::ParseStream, parse_macro_input,
};

#[proc_macro_attribute]
pub fn unreachable_impl_for_empty(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_trait = parse_macro_input!(item as ItemTrait);

    let trait_vis = &input_trait.vis;
    let trait_name = &input_trait.ident;
    let trait_generics = &input_trait.generics;
    let trait_items = &input_trait.items;

    let colon_token = &input_trait.colon_token;
    let supertraits = &input_trait.supertraits;

    let (impl_generics, ty_generics, where_clause) = trait_generics.split_for_impl();

    let mut stub_methods = Vec::new();
    for trait_item in trait_items {
        if let TraitItem::Fn(method) = trait_item {
            let sig = &method.sig; // 메서드 시그니처 추출
            let attrs = &method.attrs;
            stub_methods.push(quote! {
                #(#attrs)*
                #[allow(unused_variables, unused_mut)]
                #sig {
                    unreachable!()
                }
            });
        }
    }

    // 최종 코드 생성 (원본 트레잇 정의 + () 구현체)
    let expanded = quote! {
        // 원래 트레잇 유지
        #trait_vis trait #trait_name #trait_generics #colon_token #supertraits {
            #(#trait_items)*
        }

        // () 에 대한 unreachable! 구현 자동 추가
        impl #impl_generics #trait_name #ty_generics for () #where_clause {
            #(#stub_methods)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumAccessors)]
pub fn enum_accessors_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("EnumAccessors can only be applied to enums."),
    };

    let mut methods = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        let snake_name = (&variant_ident.to_string()).to_snake_case();
        let is_fn = format_ident!("is_{}", snake_name);
        let as_fn = format_ident!("as_{}", snake_name);

        match &variant.fields {
            Fields::Unit => {
                methods.push(quote! {
                    #[inline]
                    pub fn #is_fn(&self) -> bool {
                        matches!(self, #enum_name::#variant_ident)
                    }
                });
            }

            // Tuple variant: 필드 개수와 무관하게 처리
            Fields::Unnamed(fields) => {
                let n = fields.unnamed.len();
                let bind_idents: Vec<_> = (0..n).map(|i| format_ident!("v{}", i)).collect();
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();

                let is_pattern = if n == 0 {
                    quote! { #enum_name::#variant_ident() }
                } else {
                    quote! { #enum_name::#variant_ident(..) }
                };

                methods.push(quote! {
                    #[inline]
                    pub fn #is_fn(&self) -> bool {
                        matches!(self, #is_pattern)
                    }
                });

                if n == 1 {
                    let ty = field_types[0];
                    methods.push(quote! {
                        #[inline]
                        pub fn #as_fn(&self) -> Option<&#ty> {
                            match self {
                                #enum_name::#variant_ident(v) => Some(v),
                                _ => None,
                            }
                        }
                    });
                } else if n > 1 {
                    // 다중 필드는 참조 튜플로 리턴
                    methods.push(quote! {
                        #[inline]
                        pub fn #as_fn(&self) -> Option<(#(&#field_types),*)> {
                            match self {
                                #enum_name::#variant_ident(#(#bind_idents),*) => {
                                    Some((#(#bind_idents),*))
                                }
                                _ => None,
                            }
                        }
                    });
                }
            }

            // Named struct variant: 필드별 개별 게터도 함께 생성
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                methods.push(quote! {
                    #[inline]
                    pub fn #is_fn(&self) -> bool {
                        matches!(self, #enum_name::#variant_ident { .. })
                    }
                });

                // 전체를 참조 튜플로 반환하는 as_xxx
                methods.push(quote! {
                    #[inline]
                    pub fn #as_fn(&self) -> Option<(#(&#field_types),*)> {
                        match self {
                            #enum_name::#variant_ident { #(#field_names),* } => {
                                Some((#(#field_names),*))
                            }
                            _ => None,
                        }
                    }
                });

                // 필드 이름 기반 개별 게터: as_variant_field
                for (fname, fty) in field_names.iter().zip(field_types.iter()) {
                    let getter_fn = format_ident!("{}_{}", snake_name, fname);
                    methods.push(quote! {
                        #[inline]
                        pub fn #getter_fn(&self) -> Option<&#fty> {
                            match self {
                                #enum_name::#variant_ident { #fname, .. } => Some(#fname),
                                _ => None,
                            }
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        impl #enum_name {
            #(#methods)*
        }
    };

    TokenStream::from(expanded)
}

struct DispatchInput {
    enum_name: Ident,
    sig: Signature,
    variants: Vec<Ident>,
}

impl Parse for DispatchInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // fn 키워드 없이 시그니처만 오므로 fn을 임시로 붙여서 파싱
        let sig: Signature = {
            let fork = input.fork();
            let _ = fork; // 필요시 fn 토큰 삽입 전략 사용
            input.parse()?
        };
        input.parse::<Token![,]>()?;

        let variants = syn::punctuated::Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

        Ok(DispatchInput {
            enum_name,
            sig,
            variants: variants.into_iter().collect(),
        })
    }
}

#[proc_macro]
pub fn dispatch_method(input: TokenStream) -> TokenStream {
    let DispatchInput {
        enum_name,
        sig,
        variants,
    } = parse_macro_input!(input as DispatchInput);

    let fn_name = &sig.ident;
    let generics = &sig.generics; // 라이프타임, 제네릭, where절 전부 포함
    let inputs = &sig.inputs; // &self, arg: Type, ... 전부 포함
    let output = &sig.output;

    // self를 제외한 나머지 인자 이름만 추출
    let arg_names: Vec<_> = inputs
        .iter()
        .skip(1)
        .filter_map(|fn_arg| {
            if let syn::FnArg::Typed(pat_type) = fn_arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(&pat_ident.ident);
                }
            }
            None
        })
        .collect();

    let arms = variants.iter().map(|v| {
        quote! {
            #enum_name::#v(x) => x.#fn_name(#(#arg_names),*)
        }
    });

    let expanded = quote! {
        pub fn #fn_name #generics (#inputs) #output {
            match self {
                #(#arms,)*
            }
        }
    };

    TokenStream::from(expanded)
}

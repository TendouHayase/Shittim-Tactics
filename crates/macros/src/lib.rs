extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemTrait, TraitItem, parse_macro_input};

#[proc_macro_attribute]
pub fn unreachable_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

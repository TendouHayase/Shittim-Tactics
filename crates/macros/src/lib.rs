extern crate proc_macro;

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DeriveInput, Fields, Ident, ItemStruct, ItemTrait, LitInt, Signature, Token, TraitItem,
    Type, parse::Parse, parse::ParseStream, parse_macro_input, parse_quote,
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
        let snake_name = variant_ident.to_string().to_snake_case();
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

/// Generates the mechanical half of a skill: the five fields every skill carries, the
/// [`SkillMeta`] implementation, and [`FromParams`].
///
/// The struct must be a unit struct, because every field is generated. Per-skill data belongs
/// in the `params` type.
///
/// ```ignore
/// #[skill(owner = Student, ty = Ex, index = 0, params = params::ExParams)]
/// #[derive(Debug)]
/// pub struct ExSkill;
///
/// impl SkillOps for ExSkill {
///     fn skill_effects(&self) -> Vec<SkillEffect> { /* ... */ }
///     fn apply<'a: 'b, 'b, 'c: 'b>(/* ... */) { /* ... */ }
/// }
/// ```
///
/// | argument | required | meaning                                                    |
/// | -------- | -------- | ---------------------------------------------------------- |
/// | `owner`  | yes      | `Student` or `Boss`; picks the `Character` variant          |
/// | `ty`     | yes      | a `SkillType` variant                                       |
/// | `index`  | yes      | second half of `id`; Ex 0, Basic 1, Sub 2                   |
/// | `params` | no       | numeric parameters; defaults to `()` for skills without any |
///
/// `cost`, `duration` and `frames` are always delegated to [`SkillParams`]. A proc macro only
/// sees the tokens handed to it, so it cannot tell whether the params type has a `cost` field;
/// the trait's default implementations decide that instead.
///
/// Paths are not qualified, because these source files are copied verbatim into `core` by
/// xtask and must compile in both crates. `Character`, `CharacterOps`, `SkillMeta`,
/// `SkillParams`, `SkillType`, `FromParams` and the owner type must all be in scope at the
/// call site.
#[proc_macro_attribute]
pub fn skill(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let args = match syn::parse::<SkillArgs>(attr) {
        Ok(args) => args,
        Err(err) => return skill_fallback(&input, None, err),
    };

    match expand_skill(&args, &input) {
        Ok(expanded) => expanded.into(),
        Err(err) => skill_fallback(&input, Some(&args), err),
    }
}

/// Emits the struct alongside the error so that the failure stays a single diagnostic.
///
/// Dropping the struct would make every later reference to it a second "cannot find type" error
/// and bury the real one.
fn skill_fallback(input: &ItemStruct, args: Option<&SkillArgs>, err: syn::Error) -> TokenStream {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let name = &input.ident;
    let params = skill_params_type(args.and_then(|a| a.params.as_ref()));
    let owner = args.map(|a| {
        let owner = &a.owner;
        quote! { owner: ::std::ptr::NonNull<#owner>, }
    });

    let err = compile_errors(err);

    quote! {
        #err

        #(#attrs)*
        #vis struct #name {
            #owner
            skill_mask_offset: usize,
            name: String,
            id: (u32, u8),
            params: #params,
        }
    }
    .into()
}

/// Turns a `syn::Error` into `compile_error!` invocations that keep their spans.
///
/// `syn::Error::into_compile_error` cannot be used here: it emits `::core::compile_error!`, and
/// this workspace has a crate of its own named `core` that shadows the standard one, so that
/// path fails to resolve.
fn compile_errors(err: syn::Error) -> TokenStream2 {
    err.into_iter()
        .map(|err| {
            let message = err.to_string();
            quote_spanned! { err.span() => ::std::compile_error!(#message); }
        })
        .collect()
}

struct SkillArgs {
    /// `Student` or `Boss`. Doubles as the `Character` variant and the `NonNull` pointee.
    owner: Ident,
    ty: Ident,
    index: LitInt,
    params: Option<Type>,
}

impl Parse for SkillArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Held for the "missing argument" errors, which have no token of their own to point at.
        let span = input.span();

        let mut owner = None;
        let mut ty = None;
        let mut index = None;
        let mut params = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "owner" => {
                    let value: Ident = input.parse()?;
                    if value != "Student" && value != "Boss" {
                        return Err(syn::Error::new_spanned(
                            &value,
                            "owner must be `Student` or `Boss`",
                        ));
                    }
                    set_once(&mut owner, value, &key)?;
                }
                "ty" => set_once(&mut ty, input.parse::<Ident>()?, &key)?,
                "index" => set_once(&mut index, input.parse::<LitInt>()?, &key)?,
                "params" => set_once(&mut params, input.parse::<Type>()?, &key)?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        "unknown argument; expected `owner`, `ty`, `index` or `params`",
                    ));
                }
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(SkillArgs {
            owner: owner
                .ok_or_else(|| syn::Error::new(span, "missing `owner = Student | Boss`"))?,
            ty: ty.ok_or_else(|| syn::Error::new(span, "missing `ty = <SkillType variant>`"))?,
            index: index.ok_or_else(|| syn::Error::new(span, "missing `index = <u8>`"))?,
            params,
        })
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &Ident) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new_spanned(
            key,
            format!("`{key}` is given more than once"),
        ));
    }
    *slot = Some(value);
    Ok(())
}

/// Skills without numeric parameters still go through [`SkillParams`], via its `()` impl.
fn skill_params_type(params: Option<&Type>) -> Type {
    params.cloned().unwrap_or_else(|| parse_quote!(()))
}

fn expand_skill(args: &SkillArgs, input: &ItemStruct) -> syn::Result<TokenStream2> {
    if !matches!(input.fields, Fields::Unit) {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "#[skill] generates every field, so the struct must be a unit struct; \
             put per-skill data in the `params` type",
        ));
    }

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "#[skill] does not support generic parameters",
        ));
    }

    let attrs = &input.attrs;
    let vis = &input.vis;
    let name = &input.ident;

    let SkillArgs {
        owner, ty, index, ..
    } = args;
    let params = skill_params_type(args.params.as_ref());

    // Reached only when the assembly code hands a skill the wrong kind of owner, which is a bug
    // in the generator rather than in the data.
    let wrong_owner = format!("`{name}` was built with an owner that is not a {owner}");

    Ok(quote! {
        #(#attrs)*
        #vis struct #name {
            owner: ::std::ptr::NonNull<#owner>,
            skill_mask_offset: usize,
            name: String,
            id: (u32, u8),
            params: #params,
        }

        impl SkillMeta for #name {
            fn name(&self) -> &str {
                &self.name
            }

            fn owner(&self) -> Character<'_> {
                // SAFETY: an owner is allocated behind a `Box` before any of its skills exist
                // and is pinned afterwards, so the address recorded at construction stays valid
                // for as long as the skill does.
                unsafe { Character::#owner(self.owner.as_ref()) }
            }

            fn cost(&self) -> u8 {
                SkillParams::cost(&self.params)
            }

            fn duration(&self) -> u16 {
                SkillParams::duration(&self.params)
            }

            fn frames(&self) -> u16 {
                SkillParams::frames(&self.params)
            }

            fn skill_mask_offset(&self) -> usize {
                self.skill_mask_offset
            }

            fn skill_type(&self) -> SkillType {
                SkillType::#ty
            }
        }

        impl FromParams for #name {
            type Params = #params;

            fn new(
                name: &str,
                owner: Character<'_>,
                skill_mask_offset: usize,
                params: Self::Params,
            ) -> Self {
                let Character::#owner(owner) = owner else {
                    panic!(#wrong_owner)
                };

                Self {
                    owner: ::std::ptr::NonNull::from_ref(owner),
                    skill_mask_offset,
                    name: name.to_string(),
                    id: (CharacterOps::id(owner), #index),
                    params,
                }
            }
        }
    })
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
            if let syn::FnArg::Typed(pat_type) = fn_arg
                && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
            {
                return Some(&pat_ident.ident);
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
        fn #fn_name #generics (#inputs) #output {
            match self {
                #(#arms,)*
            }
        }
    };

    TokenStream::from(expanded)
}

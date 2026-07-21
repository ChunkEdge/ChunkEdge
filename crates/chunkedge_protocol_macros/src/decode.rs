use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse2, parse_quote, Data, DeriveInput, Error, Fields, Result};

use crate::{add_trait_bounds, decode_split_for_impl, pair_variants_with_discriminants};

pub(super) fn derive_decode(item: TokenStream) -> Result<TokenStream> {
    let mut input = parse2::<DeriveInput>(item)?;

    let input_name = input.ident.clone();

    if input.generics.lifetimes().count() > 1 {
        return Err(Error::new(
            input.generics.params.span(),
            "type deriving `Decode` must have no more than one lifetime",
        ));
    }

    // Use the lifetime specified in the type definition or just use `'a` if not
    // present.
    let lifetime = input
        .generics
        .lifetimes()
        .next()
        .map_or_else(|| parse_quote!('a), |l| l.lifetime.clone());

    match input.data {
        Data::Struct(struct_) => {
            let decode_fields = match struct_.fields {
                Fields::Named(fields) => {
                    let init = fields.named.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap();
                        let ty = &f.ty;
                        let ctx = format!("failed to decode field `{name}` in `{input_name}`");
                        let field_name_str = name.to_string();
                        let type_name_str = quote!(#ty).to_string();

                        quote! {
                            #name: {
                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                let __dbg_start = *_r;
                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                ::chunkedge_protocol::debug::log_field_start(
                                    Some(#field_name_str),
                                    #type_name_str,
                                );
                                let __res = {
                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                    let _guard = ::chunkedge_protocol::debug::IndentGuard::new();
                                    <#ty as Decode>::decode(_r)
                                };
                                match __res {
                                    Ok(__val) => {
                                        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                        ::chunkedge_protocol::debug::log_field_success(
                                            #type_name_str,
                                            &__val,
                                            __dbg_start,
                                            *_r,
                                        );
                                        __val
                                    }
                                    Err(__e) => {
                                        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                        ::chunkedge_protocol::debug::log_field_error(
                                            &__e,
                                            __dbg_start,
                                        );
                                        return Err(__e).context(#ctx);
                                    }
                                }
                            },
                        }
                    });

                    quote! {
                        Self {
                            #(#init)*
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let init = (0..fields.unnamed.len())
                        .zip(fields.unnamed.iter())
                        .map(|(i, f)| {
                            let ty = &f.ty;
                            let ctx =
                                format!("failed to decode field `{i}` in `{input_name}`");
                            let field_name_str = i.to_string();
                            let type_name_str = quote!(#ty).to_string();

                            quote! {
                                {
                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                    let __dbg_start = *_r;
                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                    ::chunkedge_protocol::debug::log_field_start(
                                        Some(#field_name_str),
                                        #type_name_str,
                                    );
                                    let __res = {
                                        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                        let _guard = ::chunkedge_protocol::debug::IndentGuard::new();
                                        <#ty as Decode>::decode(_r)
                                    };
                                    match __res {
                                        Ok(__val) => {
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            ::chunkedge_protocol::debug::log_field_success(
                                                #type_name_str,
                                                &__val,
                                                __dbg_start,
                                                *_r,
                                            );
                                            __val
                                        }
                                        Err(__e) => {
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            ::chunkedge_protocol::debug::log_field_error(
                                                &__e,
                                                __dbg_start,
                                            );
                                            return Err(__e).context(#ctx);
                                        }
                                    }
                                },
                            }
                        })
                        .collect::<TokenStream>();

                    quote! {
                        Self(#init)
                    }
                }
                Fields::Unit => quote!(Self),
            };

            add_trait_bounds(
                &mut input.generics,
                quote!(::chunkedge_binary::__private::Decode<#lifetime>),
            );

            let (impl_generics, ty_generics, where_clause) =
                decode_split_for_impl(input.generics, lifetime.clone());

            Ok(quote! {
                #[allow(unused_imports, unexpected_cfgs)]
                impl #impl_generics ::chunkedge_binary::__private::Decode<#lifetime> for #input_name #ty_generics
                #where_clause
                {
                    fn decode(_r: &mut &#lifetime [u8]) -> ::chunkedge_binary::__private::Result<Self> {
                        use ::chunkedge_binary::__private::{Decode, Context, ensure};

                        Ok(#decode_fields)
                    }
                }
            })
        }
        Data::Enum(enum_) => {
            let variants = pair_variants_with_discriminants(enum_.variants)?;
            let input_name_str = input_name.to_string();

            let decode_arms = variants
                .iter()
                .map(|(disc, variant)| {
                    let name = &variant.ident;
                    let name_str = name.to_string();

                    match &variant.fields {
                        Fields::Named(fields) => {
                            let fields = fields
                                .named
                                .iter()
                                .map(|f| {
                                    let field = f.ident.as_ref().unwrap();
                                    let ty = &f.ty;
                                    let ctx = format!(
                                        "failed to decode field `{field}` in variant `{name}` in \
                                         `{input_name}`",
                                    );
                                    let field_name_str = field.to_string();
                                    let type_name_str = quote!(#ty).to_string();

                                    quote! {
                                        #field: {
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            let __dbg_start = *_r;
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            ::chunkedge_protocol::debug::log_field_start(
                                                Some(#field_name_str),
                                                #type_name_str,
                                            );
                                            let __res = {
                                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                let _guard = ::chunkedge_protocol::debug::IndentGuard::new();
                                                <#ty as Decode>::decode(_r)
                                            };
                                            match __res {
                                                Ok(__val) => {
                                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                    ::chunkedge_protocol::debug::log_field_success(
                                                        #type_name_str,
                                                        &__val,
                                                        __dbg_start,
                                                        *_r,
                                                    );
                                                    __val
                                                }
                                                Err(__e) => {
                                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                    ::chunkedge_protocol::debug::log_field_error(
                                                        &__e,
                                                        __dbg_start,
                                                    );
                                                    return Err(__e).context(#ctx);
                                                }
                                            }
                                        },
                                    }
                                })
                                .collect::<TokenStream>();

                            quote! {
                                #disc => {
                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                    ::chunkedge_protocol::debug::log_variant(#name_str);
                                    Ok(Self::#name { #fields })
                                },
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let init = (0..fields.unnamed.len())
                                .zip(fields.unnamed.iter())
                                .map(|(i, f)| {
                                    let ty = &f.ty;
                                    let ctx = format!(
                                        "failed to decode field `{i}` in variant `{name}` in \
                                         `{input_name}`",
                                    );
                                    let field_name_str = i.to_string();
                                    let type_name_str = quote!(#ty).to_string();

                                    quote! {
                                        {
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            let __dbg_start = *_r;
                                            #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                            ::chunkedge_protocol::debug::log_field_start(
                                                Some(#field_name_str),
                                                #type_name_str,
                                            );
                                            let __res = {
                                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                let _guard = ::chunkedge_protocol::debug::IndentGuard::new();
                                                <#ty as Decode>::decode(_r)
                                            };
                                            match __res {
                                                Ok(__val) => {
                                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                    ::chunkedge_protocol::debug::log_field_success(
                                                        #type_name_str,
                                                        &__val,
                                                        __dbg_start,
                                                        *_r,
                                                    );
                                                    __val
                                                }
                                                Err(__e) => {
                                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                                    ::chunkedge_protocol::debug::log_field_error(
                                                        &__e,
                                                        __dbg_start,
                                                    );
                                                    return Err(__e).context(#ctx);
                                                }
                                            }
                                        },
                                    }
                                })
                                .collect::<TokenStream>();

                            quote! {
                                #disc => {
                                    #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                    ::chunkedge_protocol::debug::log_variant(#name_str);
                                    Ok(Self::#name(#init))
                                },
                            }
                        }
                        Fields::Unit => quote! {
                            #disc => {
                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                ::chunkedge_protocol::debug::log_variant(#name_str);
                                Ok(Self::#name)
                            },
                        },
                    }
                })
                .collect::<TokenStream>();

            add_trait_bounds(
                &mut input.generics,
                quote!(::chunkedge_binary::__private::Decode<#lifetime>),
            );

            let (impl_generics, ty_generics, where_clause) =
                decode_split_for_impl(input.generics, lifetime.clone());

            Ok(quote! {
                #[allow(unused_imports, unexpected_cfgs)]
                impl #impl_generics ::chunkedge_binary::__private::Decode<#lifetime> for #input_name #ty_generics
                #where_clause
                {
                    fn decode(_r: &mut &#lifetime [u8]) -> ::chunkedge_binary::__private::Result<Self> {
                        use ::chunkedge_binary::__private::{Decode, Context, VarInt, bail};

                        let ctx = concat!("failed to decode enum discriminant in `", stringify!(#input_name), "`");
                        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                        let __disc_start = *_r;
                        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                        ::chunkedge_protocol::debug::log_field_start(
                            Some("discriminant"),
                            "VarInt",
                        );
                        let disc_res = VarInt::decode(_r);
                        #[allow(unused_variables)]
                        let disc = match disc_res {
                            Ok(__v) => {
                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                ::chunkedge_protocol::debug::log_field_success(
                                    "VarInt",
                                    &__v,
                                    __disc_start,
                                    *_r,
                                );
                                __v.0
                            }
                            Err(__e) => {
                                #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
                                ::chunkedge_protocol::debug::log_field_error(
                                    &__e,
                                    __disc_start,
                                );
                                return Err(__e).context(ctx);
                            }
                        };
                        match disc {
                            #decode_arms
                            n => bail!("unexpected enum discriminant {} in `{}`", disc, #input_name_str),
                        }
                    }
                }
            })
        }
        Data::Union(u) => Err(Error::new(
            u.union_token.span(),
            "cannot derive `Decode` on unions",
        )),
    }
}

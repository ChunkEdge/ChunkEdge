use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Field, Fields, Member, Result, parse2};

use crate::{add_trait_bounds, pair_variants_with_discriminants};

pub(super) fn derive_encode(item: TokenStream) -> Result<TokenStream> {
    let mut input = parse2::<DeriveInput>(item)?;

    let input_name = input.ident;

    add_trait_bounds(
        &mut input.generics,
        quote!(::chunkedge_binary::__private::Encode),
    );

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        Data::Struct(struct_) => {
            let encode_fields = struct_
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let member = field
                        .ident
                        .clone()
                        .map_or_else(|| Member::from(index), Member::Named);
                    let name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), ToString::to_string);
                    let ctx = format!("failed to encode field `{name}` in `{input_name}`");
                    encode_field(field, quote!(&self.#member), &ctx)
                })
                .collect::<Result<TokenStream>>()?;

            Ok(quote! {
                #[allow(unused_imports)]
                impl #impl_generics ::chunkedge_binary::__private::Encode for #input_name #ty_generics
                #where_clause
                {
                    fn encode(&self, mut _w: impl ::std::io::Write) -> ::chunkedge_binary::__private::Result<()> {
                        use ::chunkedge_binary::__private::{Encode, Context};

                        #encode_fields

                        Ok(())
                    }
                }
            })
        }
        Data::Enum(enum_) => {
            let variants = pair_variants_with_discriminants(&input.attrs, enum_.variants)?;

            let encode_arms = variants
                .iter()
                .map(|(disc, variant)| {
                    let variant_name = &variant.ident;

                    let disc_ctx = format!(
                        "failed to encode enum discriminant {disc} for variant `{variant_name}` \
                         in `{input_name}`",
                    );

                    if let Some(codec) = crate::variant_options(&variant.attrs)?.with {
                        crate::check_variant_field_codecs(variant)?;
                        let (pattern, fields) = crate::variant_bindings(variant);
                        let ctx = format!("failed to encode payload for variant `{variant_name}`");
                        return Ok(quote! {
                            #pattern => {
                                VarInt(#disc).encode(&mut _w).context(#disc_ctx)?;
                                #codec::encode((#(#fields,)*), &mut _w).context(#ctx)
                            }
                        });
                    }

                    Ok(match &variant.fields {
                        Fields::Named(fields) => {
                            let field_names = fields
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().unwrap())
                                .collect::<Vec<_>>();

                            let encode_fields = field_names
                                .iter()
                                .zip(fields.named.iter())
                                .map(|(name, field)| {
                                    let ctx = format!(
                                        "failed to encode field `{name}` in variant \
                                         `{variant_name}` in `{input_name}`",
                                    );

                                    encode_field(field, quote!(#name), &ctx)
                                })
                                .collect::<Result<TokenStream>>()?;

                            quote! {
                                Self::#variant_name { #(#field_names,)* } => {
                                    VarInt(#disc).encode(&mut _w).context(#disc_ctx)?;

                                    #encode_fields
                                    Ok(())
                                }
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let field_names = (0..fields.unnamed.len())
                                .map(|i| Ident::new(&format!("_{i}"), Span::call_site()))
                                .collect::<Vec<_>>();

                            let encode_fields = field_names
                                .iter()
                                .zip(fields.unnamed.iter())
                                .map(|(name, field)| {
                                    let ctx = format!(
                                        "failed to encode field `{name}` in variant \
                                         `{variant_name}` in `{input_name}`"
                                    );

                                    encode_field(field, quote!(#name), &ctx)
                                })
                                .collect::<Result<TokenStream>>()?;

                            quote! {
                                Self::#variant_name(#(#field_names,)*) => {
                                    VarInt(#disc).encode(&mut _w).context(#disc_ctx)?;

                                    #encode_fields
                                    Ok(())
                                }
                            }
                        }
                        Fields::Unit => quote! {
                            Self::#variant_name => Ok(
                                VarInt(#disc)
                                    .encode(&mut _w)
                                    .context(#disc_ctx)?
                            ),
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(quote! {
                #[allow(unused_imports, unreachable_code)]
                impl #impl_generics ::chunkedge_binary::__private::Encode for #input_name #ty_generics
                #where_clause
                {
                    fn encode(&self, mut _w: impl ::std::io::Write) -> ::chunkedge_binary::__private::Result<()> {
                        use ::chunkedge_binary::__private::{Encode, VarInt, Context};

                        // Break any recursion loop by erasing the type
                        let mut _w: &mut dyn ::std::io::Write = &mut _w;

                        match self {
                            #(#encode_arms)*
                            _ => unreachable!(),
                        }
                    }
                }
            })
        }
        Data::Union(u) => Err(Error::new(
            u.union_token.span(),
            "cannot derive `Encode` on unions",
        )),
    }
}

fn encode_field(field: &Field, value: TokenStream, ctx: &str) -> Result<TokenStream> {
    let encode = match crate::field_codec(&field.attrs)? {
        Some(codec) => quote!(#codec::encode(#value, &mut _w)),
        None => quote!(Encode::encode(#value, &mut _w)),
    };
    Ok(quote!(#encode.context(#ctx)?;))
}
